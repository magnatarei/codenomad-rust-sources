// ============================================================
// CodeNomad Academy  |  Sources Rust  |  37. Validador e formatador de JSON
// ------------------------------------------------------------
// O que faz: Lê um arquivo JSON com um leitor escrito do zero. Valida e
//            aponta linha e coluna do erro, formata com recuo, compacta numa
//            linha só, extrai um valor por caminho (cliente.pedidos[0].total)
//            e lista todas as chaves com o tipo de cada valor.
// Como usar: rustc -O --edition 2021 37_json_formatador.rs -o json.exe
//            json.exe formatar pedido.json --recuo 4   (ou validar, compactar, chaves)
//            json.exe consultar pedido.json cliente.pedidos[0].total
//            json.exe --demo
// O que você aprende: enum recursivo (Box não é preciso dentro de Vec), um leitor
//            descendente recursivo com posição e mensagens de erro, escapes \uXXXX
//            com char::from_u32, métodos recursivos e closures que capturam variáveis.
// Requisitos: Rust 1.70+, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::fs;

const USO: &str = "Uso: json <comando> <arquivo.json> [opções]
  validar    confere a sintaxe e aponta linha e coluna do erro
  formatar   reescreve com recuo (--recuo N, padrão 2)
  compactar  reescreve numa linha só, sem espaços
  consultar  mostra o valor de um caminho, ex.: cliente.pedidos[0].total
  chaves     lista todos os caminhos com o tipo de cada valor
  --demo     roda uma demonstração completa";

// Um objeto guarda pares (chave, valor) em Vec para manter a ordem original.
enum Json { Nulo, Bool(bool), Num(f64), Texto(String), Lista(Vec<Json>), Objeto(Vec<(String, Json)>) }

struct Leitor<'a> { b: &'a [u8], i: usize }

impl Leitor<'_> {
    // Converte a posição em linha e coluna, contando quebras de linha até ali.
    fn erro(&self, msg: &str) -> String {
        let antes = &self.b[..self.i.min(self.b.len())];
        let linha = antes.iter().filter(|&&c| c == b'\n').count() + 1;
        let coluna = antes.iter().rev().take_while(|&&c| c != b'\n').count() + 1;
        format!("linha {linha}, coluna {coluna}: {msg}")
    }
    fn pular_espacos(&mut self) { while self.b.get(self.i).is_some_and(|c| c.is_ascii_whitespace()) { self.i += 1; } }
    fn consome(&mut self, esperado: u8) -> Result<(), String> {
        self.pular_espacos();
        if self.b.get(self.i) == Some(&esperado) { self.i += 1; Ok(()) }
        else { Err(self.erro(&format!("esperava '{}'", esperado as char))) }
    }
    fn itens<T>(&mut self, fecha: u8, mut ler: impl FnMut(&mut Self) -> Result<T, String>) -> Result<Vec<T>, String> {
        self.i += 1;
        let mut saida = Vec::new();
        loop {
            self.pular_espacos();
            match self.b.get(self.i) {
                Some(&c) if c == fecha => { self.i += 1; return Ok(saida); }
                None => return Err(self.erro(&format!("faltou fechar com '{}'", fecha as char))),
                _ if !saida.is_empty() => self.consome(b',').map_err(|_| self.erro(&format!("esperava ',' ou '{}'", fecha as char)))?,
                _ => {}
            }
            saida.push(ler(self)?);
        }
    }
    fn texto(&mut self) -> Result<String, String> {
        self.i += 1;
        let mut s = String::new();
        loop {
            let c = *self.b.get(self.i).ok_or_else(|| self.erro("texto sem aspas de fechamento"))?;
            self.i += 1;
            match c {
                b'"' => return Ok(s),
                b'\\' => {
                    let e = *self.b.get(self.i).ok_or_else(|| self.erro("escape incompleto"))?;
                    self.i += 1;
                    match e {
                        b'n' => s.push('\n'), b't' => s.push('\t'), b'r' => s.push('\r'),
                        b'"' | b'\\' | b'/' => s.push(e as char), b'b' => s.push('\u{8}'), b'f' => s.push('\u{c}'),
                        b'u' => {
                            let hex = self.b.get(self.i..self.i + 4).and_then(|h| std::str::from_utf8(h).ok())
                                .and_then(|h| u32::from_str_radix(h, 16).ok()).and_then(char::from_u32)
                                .ok_or_else(|| self.erro("escape \\u precisa de 4 dígitos hexadecimais"))?;
                            s.push(hex);
                            self.i += 4;
                        }
                        outro => return Err(self.erro(&format!("escape desconhecido '\\{}'", outro as char))),
                    }
                }
                _ => {
                    // Um caractere UTF-8 pode ter mais de um byte: recolhe o pedaço inteiro de uma vez.
                    let fim = (self.i..=self.b.len()).find(|&f| matches!(self.b.get(f), Some(b'"' | b'\\') | None)).unwrap_or(self.b.len());
                    s.push_str(std::str::from_utf8(&self.b[self.i - 1..fim]).map_err(|_| self.erro("texto com UTF-8 inválido"))?);
                    self.i = fim;
                }
            }
        }
    }
    fn valor(&mut self) -> Result<Json, String> {
        self.pular_espacos();
        match self.b.get(self.i).copied() {
            Some(b'[') => self.itens(b']', |l| l.valor()).map(Json::Lista),
            Some(b'{') => self.itens(b'}', |l| {
                l.pular_espacos();
                if l.b.get(l.i) != Some(&b'"') { return Err(l.erro("a chave de um objeto precisa estar entre aspas")); }
                let chave = l.texto()?;
                l.consome(b':')?;
                Ok((chave, l.valor()?))
            }).map(Json::Objeto),
            Some(b'"') => self.texto().map(Json::Texto),
            Some(c) if c == b'-' || c.is_ascii_digit() => {
                let inicio = self.i;
                while self.b.get(self.i).is_some_and(|c| c.is_ascii_digit() || b"+-.eE".contains(c)) { self.i += 1; }
                let trecho = std::str::from_utf8(&self.b[inicio..self.i]).unwrap_or("");
                trecho.parse().map(Json::Num).map_err(|_| self.erro(&format!("número inválido '{trecho}'")))
            }
            _ => {
                for (literal, valor) in [("true", Json::Bool(true)), ("false", Json::Bool(false)), ("null", Json::Nulo)] {
                    if self.b[self.i..].starts_with(literal.as_bytes()) { self.i += literal.len(); return Ok(valor); }
                }
                Err(self.erro(if self.i >= self.b.len() { "o JSON terminou antes da hora" } else { "valor inesperado" }))
            }
        }
    }
}

fn ler_json(texto: &str) -> Result<Json, String> {
    let mut leitor = Leitor { b: texto.as_bytes(), i: 0 };
    let raiz = leitor.valor()?;
    leitor.pular_espacos();
    if leitor.i < leitor.b.len() { Err(leitor.erro("conteúdo extra depois do fim do JSON")) } else { Ok(raiz) }
}

fn escapar(s: &str) -> String {
    let mut saida = String::from("\"");
    for c in s.chars() {
        match c {
            '"' | '\\' => { saida.push('\\'); saida.push(c); }
            '\n' => saida.push_str("\\n"), '\t' => saida.push_str("\\t"), '\r' => saida.push_str("\\r"),
            c if (c as u32) < 0x20 => saida.push_str(&format!("\\u{:04x}", c as u32)),
            c => saida.push(c),
        }
    }
    saida + "\""
}

impl Json {
    fn tipo(&self) -> &'static str {
        match self { Self::Nulo => "nulo", Self::Bool(_) => "booleano", Self::Num(_) => "número",
            Self::Texto(_) => "texto", Self::Lista(_) => "lista", Self::Objeto(_) => "objeto" }
    }
    // recuo = 0 gera a forma compacta; qualquer outro valor gera a forma legível.
    fn escrever(&self, recuo: usize, nivel: usize) -> String {
        let (quebra, dois_pontos) = if recuo == 0 { ("", ":") } else { ("\n", ": ") };
        let (dentro, fora) = (" ".repeat(recuo * (nivel + 1)), " ".repeat(recuo * nivel));
        // A closure captura quebra e fora por referência e monta "[...]" ou "{...}".
        let juntar = |itens: Vec<String>, abre: char, fecha: char| format!("{abre}{quebra}{}{quebra}{fora}{fecha}", itens.join(&format!(",{quebra}")));
        match self {
            Self::Nulo => "null".into(),
            Self::Bool(b) => b.to_string(),
            // 29.0 vira "29"; 29.9 continua "29.9".
            Self::Num(n) if n.fract() == 0.0 && n.abs() < 1e15 => format!("{}", *n as i64),
            Self::Num(n) => n.to_string(),
            Self::Texto(s) => escapar(s),
            Self::Lista(v) if v.is_empty() => "[]".into(), Self::Objeto(v) if v.is_empty() => "{}".into(),
            Self::Lista(v) => juntar(v.iter().map(|x| format!("{dentro}{}", x.escrever(recuo, nivel + 1))).collect(), '[', ']'),
            Self::Objeto(v) => juntar(v.iter().map(|(k, x)| format!("{dentro}{}{dois_pontos}{}", escapar(k), x.escrever(recuo, nivel + 1))).collect(), '{', '}'),
        }
    }
    fn consultar(&self, caminho: &str) -> Result<&Json, String> {
        let mut atual = self;
        for parte in caminho.split('.').filter(|p| !p.is_empty()) {
            // "pedidos[0][2]" vira a chave "pedidos" seguida dos índices 0 e 2.
            let (chave, indices) = parte.split_once('[').map(|(c, r)| (c, format!("[{r}"))).unwrap_or((parte, String::new()));
            if !chave.is_empty() {
                let Json::Objeto(campos) = atual else { return Err(format!("'{chave}' pedido num valor que é {}", atual.tipo())) };
                atual = &campos.iter().find(|(k, _)| k == chave).ok_or(format!("chave '{chave}' não existe"))?.1;
            }
            for indice in indices.split('[').skip(1) {
                let n: usize = indice.strip_suffix(']').and_then(|n| n.parse().ok()).ok_or(format!("índice inválido em '{parte}'"))?;
                let Json::Lista(itens) = atual else { return Err(format!("índice [{n}] usado num valor que é {}", atual.tipo())) };
                atual = itens.get(n).ok_or(format!("a lista tem {} item(ns); não existe [{n}]", itens.len()))?;
            }
        }
        Ok(atual)
    }
    fn listar_chaves(&self, prefixo: &str, saida: &mut Vec<String>) {
        match self {
            Self::Objeto(v) => for (k, x) in v {
                let caminho = if prefixo.is_empty() { k.clone() } else { format!("{prefixo}.{k}") };
                saida.push(format!("{caminho}  ({})", x.tipo()));
                x.listar_chaves(&caminho, saida);
            },
            Self::Lista(v) => for (i, x) in v.iter().enumerate() { x.listar_chaves(&format!("{prefixo}[{i}]"), saida); },
            _ => {}
        }
    }
}

fn executar(args: &[String]) -> Result<(), String> {
    let [comando, arquivo, extras @ ..] = args else { return Err(format!("informe o comando e o arquivo\n{USO}")) };
    let texto = fs::read_to_string(arquivo).map_err(|e| format!("não consegui ler {arquivo}: {e}"))?;
    let raiz = ler_json(&texto).map_err(|e| format!("JSON inválido em {arquivo}: {e}"))?;
    match comando.as_str() {
        "validar" => println!("JSON válido: {} na raiz, {} byte(s).", raiz.tipo(), texto.len()),
        "formatar" => {
            let recuo = match extras { [] => 2, [op, n] if op == "--recuo" => n.parse().ok().filter(|r| (1..=8).contains(r))
                .ok_or(format!("--recuo inválido: '{n}' (use de 1 a 8)"))?, _ => return Err("uso: formatar <arquivo> [--recuo N]".into()) };
            println!("{}", raiz.escrever(recuo, 0));
        }
        "compactar" => println!("{}", raiz.escrever(0, 0)),
        "consultar" => println!("{}", raiz.consultar(extras.first().ok_or("informe o caminho, ex.: cliente.pedidos[0].total")?)?.escrever(2, 0)),
        "chaves" => { let mut lista = Vec::new(); raiz.listar_chaves("", &mut lista); for l in lista { println!("{l}"); } }
        outro => return Err(format!("comando desconhecido: '{outro}'\n{USO}")),
    }
    Ok(())
}

fn demo() -> Result<(), String> {
    let exemplo = r#"{"cliente": {"nome": "Loja Horizonte", "ativo": true, "cidade": "Curitiba",
  "pedidos": [{"id": 101, "total": 259.90, "itens": ["teclado", "mouse"]}, {"id": 102, "total": 48, "itens": []}]},
  "observação": "entrega em 3 dias", "desconto": null}"#;
    let arquivo = std::env::temp_dir().join(format!("codenomad_json_{}.json", std::process::id()));
    fs::write(&arquivo, exemplo).map_err(|e| format!("não consegui gravar o arquivo temporário: {e}"))?;
    let caminho = arquivo.to_string_lossy().to_string();
    println!("=== Demonstração: validador e formatador de JSON ===");
    println!("Arquivo temporário: {caminho}\n");
    let passos: [&[&str]; 7] = [
        &["validar"], &["formatar"], &["compactar"], &["consultar", "cliente.pedidos[0].total"],
        &["consultar", "cliente.pedidos[1].itens"], &["consultar", "cliente.pedidos[5].id"], &["chaves"],
    ];
    for passo in passos {
        println!("$ json {} pedido.json {}", passo[0], passo[1..].join(" "));
        let mut args = vec![passo[0].to_string(), caminho.clone()];
        args.extend(passo[1..].iter().map(|s| s.to_string()));
        if let Err(e) = executar(&args) { println!("Erro: {e}"); }
        println!();
    }
    println!("--- Um JSON inválido (falta a vírgula no fim da linha 3; o leitor tropeça na linha 4) ---");
    let quebrado = "{\n  \"nome\": \"Ana\",\n  \"idade\": 30\n  \"cidade\": \"Recife\"\n}";
    for (n, linha) in quebrado.lines().enumerate() { println!("  {} | {linha}", n + 1); }
    match ler_json(quebrado) { Ok(_) => println!("(passou, mas não deveria)"), Err(e) => println!("Erro: {e}") }
    let _ = fs::remove_file(&arquivo);
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None => { println!("{USO}"); Ok(()) }
        Some("--demo") => demo(),
        Some(_) => executar(&args),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); std::process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Trate pares substitutos (surrogates) em \uXXXX para aceitar símbolos fora do plano básico.
// 2. Crie o comando "ordenar" que reescreve as chaves de cada objeto em ordem alfabética.
// 3. Adicione "comparar a.json b.json" mostrando os caminhos que existem só de um lado ou têm valor diferente.
// ------------------------------------------------------------
