// ============================================================
// CodeNomad Academy  |  Sources Rust  |  41. Cliente HTTP sem dependências
// ------------------------------------------------------------
// O que faz: faz um GET numa URL http:// usando apenas TcpStream, mostra
//            status, cabeçalhos e corpo, interpreta o JSON da resposta e
//            extrai campos por caminho (ex.: loja.produtos.0.nome). Timeout
//            e erro de rede viram mensagens claras, nunca um travamento.
// Como usar: 41_cliente_http.exe http://127.0.0.1:8080/api/hora --campo iso --timeout 5
//            41_cliente_http.exe --demo
// O que você aprende: TcpStream com timeout, o formato de uma requisição
//            HTTP escrita à mão, um analisador de JSON recursivo com enum,
//            o trait Display num tipo próprio, try_fold com Option e
//            propagação de erros com ? em funções que devolvem Result.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::collections::BTreeMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::{env, fmt, process};

/// As partes de uma URL no formato http://servidor:porta/caminho?consulta
struct Url { host: String, porta: u16, caminho: String }
fn analisar_url(texto: &str) -> Result<Url, String> {
    if texto.starts_with("https://") { return Err("https:// exige TLS, que a biblioteca padrão não tem; use http://".into()); }
    let resto = texto.strip_prefix("http://").ok_or("a URL precisa começar com http://")?;
    // O primeiro "/" separa servidor:porta do caminho; sem ele, o caminho é "/"
    let (autoridade, caminho) = resto.find('/').map_or((resto, "/"), |i| (&resto[..i], &resto[i..]));
    if autoridade.is_empty() { return Err("falta o nome do servidor na URL".into()); }
    let (host, porta) = match autoridade.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().map_err(|_| format!("porta inválida: {p}"))?),
        None => (autoridade, 80),
    };
    Ok(Url { host: host.to_string(), porta, caminho: caminho.to_string() })
}

struct Resposta { status: u16, motivo: String, cabecalhos: Vec<(String, String)>, corpo: String }

fn interpretar_resposta(bruto: &[u8]) -> Result<Resposta, String> {
    // Cabeçalhos e corpo são separados por uma linha vazia (\r\n\r\n)
    let fim = bruto.windows(4).position(|j| j == b"\r\n\r\n").ok_or("resposta sem o fim dos cabeçalhos")?;
    let cabeca = String::from_utf8_lossy(&bruto[..fim]);
    let mut linhas = cabeca.lines();
    let primeira = linhas.next().ok_or("resposta vazia")?;
    let mut partes = primeira.splitn(3, ' ');
    if !partes.next().unwrap_or("").starts_with("HTTP/") { return Err(format!("isto não parece uma resposta HTTP: {primeira}")); }
    let status = partes.next().unwrap_or("").parse::<u16>().map_err(|_| format!("status inválido em: {primeira}"))?;
    let motivo = partes.next().unwrap_or("").to_string();
    let cabecalhos = linhas.filter_map(|l| l.split_once(':')).map(|(n, v)| (n.trim().to_string(), v.trim().to_string())).collect();
    Ok(Resposta { status, motivo, cabecalhos, corpo: String::from_utf8_lossy(&bruto[fim + 4..]).into_owned() })
}

fn buscar(url: &Url, timeout: Duration) -> Result<Resposta, String> {
    let endereco = (url.host.as_str(), url.porta).to_socket_addrs()
        .map_err(|e| format!("não resolvi o nome {}: {e}", url.host))?
        .next().ok_or_else(|| format!("nenhum endereço encontrado para {}", url.host))?;
    let mut conexao = TcpStream::connect_timeout(&endereco, timeout).map_err(|e| format!("não conectei em {endereco}: {e}"))?;
    conexao.set_read_timeout(Some(timeout)).map_err(|e| e.to_string())?;
    // Pedimos HTTP/1.0: o servidor responde inteiro e fecha a conexão, sem corpo em pedaços (chunked)
    let pedido = format!("GET {} HTTP/1.0\r\nHost: {}\r\nUser-Agent: codenomad-cliente/1.0\r\nAccept: */*\r\n\r\n", url.caminho, url.host);
    conexao.write_all(pedido.as_bytes()).map_err(|e| format!("falha ao enviar o pedido: {e}"))?;
    let mut bruto = Vec::new();
    // Se já chegaram dados, o erro é só o servidor demorando a fechar a conexão: ignoramos
    if let (Err(e), true) = (conexao.read_to_end(&mut bruto), bruto.is_empty()) {
        let estourou = matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut);
        return Err(if estourou { format!("o servidor não respondeu em {} s (timeout)", timeout.as_secs()) } else { format!("falha ao ler a resposta: {e}") });
    }
    interpretar_resposta(&bruto)
}

/// Um valor JSON qualquer. BTreeMap mantém as chaves em ordem alfabética ao imprimir.
enum Json { Nulo, Logico(bool), Numero(f64), Texto(String), Lista(Vec<Json>), Objeto(BTreeMap<String, Json>) }

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Json::Nulo => write!(f, "null"),
            Json::Logico(v) => write!(f, "{v}"),
            Json::Numero(n) if n.fract() == 0.0 && n.abs() < 1e15 => write!(f, "{}", *n as i64),
            Json::Numero(n) => write!(f, "{n}"),
            Json::Texto(t) => write!(f, "\"{}\"", t.replace('\\', "\\\\").replace('"', "\\\"")),
            Json::Lista(itens) => write!(f, "[{}]", itens.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")),
            Json::Objeto(c) => write!(f, "{{{}}}", c.iter().map(|(k, v)| format!("\"{k}\": {v}")).collect::<Vec<_>>().join(", ")),
        }
    }
}

/// Analisador recursivo: cada método lê um tipo de valor e avança a posição
struct Leitor<'a> { bytes: &'a [u8], pos: usize }

impl Leitor<'_> {
    fn espiar(&self) -> Option<u8> { self.bytes.get(self.pos).copied() }
    fn pular_espacos(&mut self) { while self.espiar().is_some_and(|c| c.is_ascii_whitespace()) { self.pos += 1; } }
    /// Consome o símbolo esperado ou devolve um erro dizendo onde parou
    fn esperar(&mut self, c: u8) -> Result<(), String> {
        self.pular_espacos();
        if self.espiar() != Some(c) { return Err(format!("esperava '{}' na posição {}", c as char, self.pos)); }
        self.pos += 1;  Ok(())
    }
    fn valor(&mut self) -> Result<Json, String> {
        self.pular_espacos();
        match self.espiar() {
            None => Err("o JSON terminou antes da hora".into()),
            Some(b'{') => self.objeto(),
            Some(b'[') => self.lista(),
            Some(b'"') => Ok(Json::Texto(self.texto()?)),
            Some(b't') => self.literal("true", Json::Logico(true)),
            Some(b'f') => self.literal("false", Json::Logico(false)),
            Some(b'n') => self.literal("null", Json::Nulo),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.numero(),
            Some(c) => Err(format!("símbolo inesperado '{}' na posição {}", c as char, self.pos)),
        }
    }
    fn literal(&mut self, palavra: &str, valor: Json) -> Result<Json, String> {
        if !self.bytes[self.pos..].starts_with(palavra.as_bytes()) { return Err(format!("esperava {palavra} na posição {}", self.pos)); }
        self.pos += palavra.len();  Ok(valor)
    }
    fn numero(&mut self) -> Result<Json, String> {
        let inicio = self.pos;
        while self.espiar().is_some_and(|c| c.is_ascii_digit() || b"+-.eE".contains(&c)) { self.pos += 1; }
        let trecho = String::from_utf8_lossy(&self.bytes[inicio..self.pos]);
        trecho.parse::<f64>().map(Json::Numero).map_err(|_| format!("número inválido: {trecho}"))
    }
    fn texto(&mut self) -> Result<String, String> {
        self.esperar(b'"')?;
        let mut bytes = Vec::new(); // juntamos bytes e só no fim viramos String, por causa do UTF-8
        loop {
            let c = self.espiar().ok_or("texto JSON sem aspa de fechamento")?;
            self.pos += 1;
            match c {
                b'"' => return Ok(String::from_utf8_lossy(&bytes).into_owned()),
                b'\\' => { // escapes: \n \t \r e qualquer outro vale ele mesmo (\" \\ \/)
                    let escape = self.espiar().ok_or("escape incompleto no fim do texto")?;
                    self.pos += 1;
                    bytes.push(match escape { b'n' => b'\n', b't' => b'\t', b'r' => b'\r', outro => outro });
                }
                _ => bytes.push(c),
            }
        }
    }
    fn lista(&mut self) -> Result<Json, String> {
        self.esperar(b'[')?;
        let mut itens = Vec::new();
        while self.esperar(b']').is_err() {
            if !itens.is_empty() { self.esperar(b',')?; } // entre um item e outro tem de haver vírgula
            itens.push(self.valor()?);
        }
        Ok(Json::Lista(itens))
    }
    fn objeto(&mut self) -> Result<Json, String> {
        self.esperar(b'{')?;
        let mut campos = BTreeMap::new();
        while self.esperar(b'}').is_err() {
            if !campos.is_empty() { self.esperar(b',')?; }
            let chave = self.texto()?;
            self.esperar(b':')?;
            let valor = self.valor()?;
            campos.insert(chave, valor);
        }
        Ok(Json::Objeto(campos))
    }
}

fn interpretar_json(texto: &str) -> Result<Json, String> {
    let mut leitor = Leitor { bytes: texto.as_bytes(), pos: 0 };
    let valor = leitor.valor()?;
    leitor.pular_espacos();
    if leitor.pos == leitor.bytes.len() { Ok(valor) } else { Err(format!("sobrou conteúdo depois do JSON, na posição {}", leitor.pos)) }
}

/// Caminha pelo JSON seguindo um caminho como "loja.produtos.0.nome"
fn buscar_campo<'a>(raiz: &'a Json, caminho: &str) -> Option<&'a Json> {
    caminho.split('.').try_fold(raiz, |atual, parte| match atual {
        Json::Objeto(campos) => campos.get(parte),
        Json::Lista(itens) => itens.get(parte.parse::<usize>().ok()?),
        _ => None,
    })
}

fn mostrar(resposta: &Resposta, campos: &[String]) -> Result<(), String> {
    println!("Status: {} {}", resposta.status, resposta.motivo);
    println!("Cabeçalhos ({}):", resposta.cabecalhos.len());
    for (nome, valor) in &resposta.cabecalhos { println!("  {nome}: {valor}"); }
    // Nomes de cabeçalho não diferenciam maiúsculas: Content-Type e content-type são o mesmo
    let tipo = resposta.cabecalhos.iter().find(|(n, _)| n.eq_ignore_ascii_case("content-type")).map(|(_, v)| v.as_str());
    let parece_json = tipo.is_some_and(|t| t.contains("json")) || resposta.corpo.trim_start().starts_with(['{', '[']);
    if !parece_json {
        if !campos.is_empty() { return Err("o corpo não é JSON, então não dá para extrair campos".into()); }
        println!("Corpo ({} bytes):\n{}", resposta.corpo.len(), resposta.corpo.trim_end());
        return Ok(());
    }
    let json = interpretar_json(&resposta.corpo)?;
    println!("Corpo (JSON interpretado e reescrito pelo Display):\n  {json}");
    for campo in campos {
        match buscar_campo(&json, campo) { Some(v) => println!("Campo {campo} = {v}"), None => println!("Campo {campo}: não encontrado") }
    }
    Ok(())
}

fn executar(args: &[String]) -> Result<(), String> {
    let url = analisar_url(&args[0])?;
    let (mut campos, mut timeout) = (Vec::new(), 10u64);
    let mut opcoes = args[1..].iter();
    while let Some(opcao) = opcoes.next() { // cada opção vem seguida do seu valor
        let valor = opcoes.next().ok_or(format!("{opcao} precisa de um valor logo depois"))?;
        match opcao.as_str() {
            "--campo" => campos.push(valor.clone()),
            "--timeout" => timeout = valor.parse().ok().filter(|t| *t > 0).ok_or(format!("timeout inválido: {valor}"))?,
            outro => return Err(format!("opção desconhecida: {outro}")),
        }
    }
    println!("GET http://{}:{}{}", url.host, url.porta, url.caminho);
    mostrar(&buscar(&url, Duration::from_secs(timeout))?, &campos)
}

fn demo() -> Result<(), String> {
    println!("=== Cliente HTTP: demonstração sem acessar a rede ===\nInterpretando uma resposta de exemplo como se tivesse chegado pela conexão.\n");
    let corpo = r#"{"loja": {"nome": "Mercearia da Esquina", "aberta": true, "produtos": [
        {"nome": "Café torrado 500g", "preco": 24.9, "estoque": 12},
        {"nome": "Pão de queijo", "preco": 8.5, "estoque": 0},
        {"nome": "Goiabada cascão", "preco": 12, "estoque": 7}]},
        "atualizado_em": "2026-03-01T10:00:00Z", "observacao": null}"#;
    let bruto = format!("HTTP/1.0 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nServer: exemplo/1.0\r\nContent-Length: {}\r\n\r\n{corpo}", corpo.len());
    let resposta = interpretar_resposta(bruto.as_bytes())?;
    let campos = ["loja.nome", "loja.produtos.1.preco", "loja.aberta", "loja.produtos.9.nome"].map(String::from);
    mostrar(&resposta, &campos)?;
    let json = interpretar_json(&resposta.corpo)?;
    if let Some(Json::Lista(produtos)) = buscar_campo(&json, "loja.produtos") {
        // Percorre a lista e soma só os preços que são números
        let precos = produtos.iter().filter_map(|p| match buscar_campo(p, "preco") { Some(Json::Numero(n)) => Some(*n), _ => None });
        println!("Soma dos preços dos {} produtos: {:.2}", produtos.len(), precos.sum::<f64>());
    }
    println!("Erro tratado de propósito: {:?}", analisar_url("ftp://arquivos.exemplo").err());
    println!("JSON quebrado de propósito: {:?}", interpretar_json("{\"a\": [1, 2").err());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None | Some("--ajuda") | Some("-h") => {
            println!("Uso: 41_cliente_http.exe <URL http://...> [--campo caminho]... [--timeout segundos]\n     41_cliente_http.exe --demo");
            return;
        }
        Some("--demo") => demo(),
        _ => executar(&args),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Aceite --cabecalho "Nome: valor" (repetível) e envie esses cabeçalhos
//    extras no pedido, útil para testar APIs que exigem um token.
// 2. Siga redirecionamentos: quando o status for 301 ou 302, leia o
//    cabeçalho Location e refaça o GET, com um limite de 5 saltos.
// 3. Escreva uma função que imprima o JSON como árvore com recuo, um
//    nível por objeto ou lista, e use-a no lugar do Display no modo --demo.
// ------------------------------------------------------------
