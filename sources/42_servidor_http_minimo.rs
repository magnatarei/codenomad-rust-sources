// ============================================================
// CodeNomad Academy  |  Sources Rust  |  42. Servidor HTTP mínimo
// ------------------------------------------------------------
// O que faz: sobe um servidor HTTP/1.1 só com TcpListener, serve os arquivos
//            de uma pasta (com tipo de conteúdo pelo sufixo), responde JSON
//            em /api/hora e /api/eco e registra cada pedido no terminal.
// Como usar: 42_servidor_http_minimo.exe ./site --porta 8080
//            (depois abra http://127.0.0.1:8080/ ou /api/hora no navegador)
//            42_servidor_http_minimo.exe --demo
// O que você aprende: TcpListener e TcpStream, BufReader para ler linha a
//            linha, threads com Arc e AtomicBool para encerrar com elegância,
//            match em tuplas (método, caminho), proteção contra "..", e como
//            converter segundos desde 1970 em data e hora sem bibliotecas.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs, process, thread};

/// O que o cliente pediu: só a parte que este servidor usa
struct Pedido { metodo: String, caminho: String, consulta: String, corpo: String }

/// Lê a linha de pedido, os cabeçalhos e, se houver Content-Length, o corpo
fn ler_pedido(conexao: &mut TcpStream) -> Result<Pedido, String> {
    let mut leitor = BufReader::new(conexao);
    let mut primeira = String::new();
    leitor.read_line(&mut primeira).map_err(|e| format!("falha ao ler o pedido: {e}"))?;
    let mut partes = primeira.split_whitespace();
    let (metodo, alvo) = (partes.next().unwrap_or(""), partes.next().unwrap_or("/"));
    if metodo.is_empty() { return Err("pedido vazio".into()); }
    let (caminho, consulta) = alvo.split_once('?').unwrap_or((alvo, ""));
    let mut tamanho = 0usize;
    loop { // cabeçalhos até a linha vazia
        let mut linha = String::new();
        leitor.read_line(&mut linha).map_err(|e| e.to_string())?;
        if linha.trim().is_empty() { break; }
        if let Some((nome, valor)) = linha.split_once(':') {
            if nome.eq_ignore_ascii_case("content-length") { tamanho = valor.trim().parse().unwrap_or(0); }
        }
    }
    let mut corpo = vec![0u8; tamanho.min(1 << 20)]; // no máximo 1 MiB de corpo
    leitor.read_exact(&mut corpo).map_err(|e| format!("corpo incompleto: {e}"))?;
    Ok(Pedido { metodo: metodo.into(), caminho: caminho.into(), consulta: consulta.into(), corpo: String::from_utf8_lossy(&corpo).into_owned() })
}

/// Converte segundos desde 1970 em (ano, mês, dia, hora, minuto, segundo) em UTC
fn data_hora(segundos: u64) -> (u64, u64, u64, u64, u64, u64) {
    let (dias, resto) = (segundos / 86_400, segundos % 86_400);
    // Algoritmo clássico de calendário civil: desloca a origem para 1º de março do ano 0
    let z = dias + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = doy - (153 * mp + 2) / 5 + 1;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 };
    let ano = yoe + era * 400 + u64::from(mes <= 2);
    (ano, mes, dia, resto / 3600, resto % 3600 / 60, resto % 60)
}

fn agora_iso() -> (u64, String) {
    let segundos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (a, m, d, h, mi, s) = data_hora(segundos);
    (segundos, format!("{a:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z"))
}

/// Deixa um texto seguro para entrar numa string JSON
fn escapar_json(texto: &str) -> String {
    texto.chars().map(|c| match c { '"' => "\\\"".into(), '\\' => "\\\\".into(), '\n' => "\\n".into(), '\r' => "".into(), c => c.to_string() }).collect()
}

fn tipo_conteudo(caminho: &Path) -> &'static str {
    match caminho.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8", "css" => "text/css", "js" => "text/javascript",
        "json" => "application/json", "png" => "image/png", "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml", "txt" | "md" => "text/plain; charset=utf-8", _ => "application/octet-stream",
    }
}

/// Decide a resposta: (status, tipo de conteúdo, corpo em bytes)
fn responder(pedido: &Pedido, pasta: &Path) -> (u16, &'static str, Vec<u8>) {
    match (pedido.metodo.as_str(), pedido.caminho.as_str()) {
        ("GET", "/api/hora") => {
            let (epoch, iso) = agora_iso();
            (200, "application/json", format!("{{\"iso\": \"{iso}\", \"epoch\": {epoch}}}").into_bytes())
        }
        ("GET" | "POST", "/api/eco") => {
            let corpo = format!("{{\"metodo\": \"{}\", \"consulta\": \"{}\", \"corpo\": \"{}\"}}",
                pedido.metodo, escapar_json(&pedido.consulta), escapar_json(&pedido.corpo));
            (200, "application/json", corpo.into_bytes())
        }
        ("GET", caminho) => servir_arquivo(pasta, caminho),
        _ => (405, "text/plain; charset=utf-8", "Método não permitido".as_bytes().to_vec()),
    }
}

fn servir_arquivo(pasta: &Path, caminho: &str) -> (u16, &'static str, Vec<u8>) {
    // Monta o caminho só com segmentos normais: ".." e caminhos absolutos ficam de fora
    let mut alvo: PathBuf = pasta.to_path_buf();
    for parte in caminho.split('/').filter(|p| !p.is_empty()) {
        if parte == ".." || parte.contains(':') || parte.starts_with('\\') { return (403, "text/plain; charset=utf-8", "Caminho proibido".as_bytes().to_vec()); }
        alvo.push(parte);
    }
    if alvo.is_dir() { alvo.push("index.html"); }
    match fs::read(&alvo) {
        Ok(bytes) => (200, tipo_conteudo(&alvo), bytes),
        Err(_) => (404, "text/plain; charset=utf-8", format!("Não encontrei {caminho}").into_bytes()),
    }
}

/// Atende uma conexão do começo ao fim e devolve a linha de log
fn atender(mut conexao: TcpStream, pasta: &Path) -> String {
    let _ = conexao.set_read_timeout(Some(Duration::from_secs(5)));
    let (resumo, status, tipo, corpo) = match ler_pedido(&mut conexao) {
        Ok(pedido) => {
            let (status, tipo, corpo) = responder(&pedido, pasta);
            (format!("{} {}", pedido.metodo, pedido.caminho), status, tipo, corpo)
        }
        Err(erro) => ("pedido inválido".to_string(), 400, "text/plain; charset=utf-8", erro.into_bytes()),
    };
    let motivo = match status { 200 => "OK", 400 => "Bad Request", 403 => "Forbidden", 404 => "Not Found", _ => "Method Not Allowed" };
    let cabeca = format!("HTTP/1.1 {status} {motivo}\r\nContent-Type: {tipo}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", corpo.len());
    let _ = conexao.write_all(cabeca.as_bytes()).and_then(|_| conexao.write_all(&corpo));
    let (_, hora) = agora_iso();
    format!("[{}] {resumo} -> {status} ({} bytes)", &hora[11..19], corpo.len())
}

/// Laço principal: aceita conexões até a bandeira `parar` ser levantada
fn servir(ouvinte: TcpListener, pasta: PathBuf, parar: Arc<AtomicBool>) {
    for conexao in ouvinte.incoming() {
        if parar.load(Ordering::SeqCst) { break; }
        match conexao {
            Ok(c) => println!("{}", atender(c, &pasta)),
            Err(e) => eprintln!("conexão recusada: {e}"),
        }
    }
}

/// Um cliente de bolso, só para a demonstração pedir ao próprio servidor
fn pedir(porta: u16, alvo: &str) -> Result<String, String> {
    let mut c = TcpStream::connect(("127.0.0.1", porta)).map_err(|e| format!("não conectei: {e}"))?;
    c.write_all(format!("GET {alvo} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").as_bytes()).map_err(|e| e.to_string())?;
    let mut resposta = String::new();
    c.read_to_string(&mut resposta).map_err(|e| e.to_string())?;
    Ok(resposta)
}

fn executar(args: &[String]) -> Result<(), String> {
    let pasta = PathBuf::from(&args[0]);
    if !pasta.is_dir() { return Err(format!("{} não é uma pasta", pasta.display())); }
    let porta: u16 = match args.get(1).map(String::as_str) {
        Some("--porta") => args.get(2).and_then(|p| p.parse().ok()).ok_or("--porta precisa de um número de 1 a 65535")?,
        Some(outro) => return Err(format!("opção desconhecida: {outro}")),
        None => 8080,
    };
    let ouvinte = TcpListener::bind(("127.0.0.1", porta)).map_err(|e| format!("não consegui abrir a porta {porta}: {e}"))?;
    println!("Servindo {} em http://127.0.0.1:{porta}/  (Ctrl+C para parar)", pasta.display());
    servir(ouvinte, pasta, Arc::new(AtomicBool::new(false)));
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Servidor HTTP mínimo: demonstração ===");
    let pasta = env::temp_dir().join("codenomad_servidor_demo");
    fs::create_dir_all(&pasta).map_err(|e| e.to_string())?;
    fs::write(pasta.join("index.html"), "<h1>Olá da CodeNomad Academy</h1>").map_err(|e| e.to_string())?;
    // Porta 0 pede ao sistema uma porta livre qualquer
    let ouvinte = TcpListener::bind(("127.0.0.1", 0)).map_err(|e| format!("não consegui abrir uma porta: {e}"))?;
    let porta = ouvinte.local_addr().map_err(|e| e.to_string())?.port();
    println!("Servidor no ar em http://127.0.0.1:{porta}/ servindo {}\n", pasta.display());
    let parar = Arc::new(AtomicBool::new(false));
    let (bandeira, pasta_servidor) = (Arc::clone(&parar), pasta.clone());
    let servidor = thread::spawn(move || servir(ouvinte, pasta_servidor, bandeira));
    for alvo in ["/api/hora", "/index.html", "/api/eco?nome=Aluno", "/nao_existe.txt", "/../segredo"] {
        let resposta = pedir(porta, alvo)?;
        let (cabeca, corpo) = resposta.split_once("\r\n\r\n").unwrap_or((&resposta, ""));
        println!("GET {alvo}\n  {}\n  corpo: {corpo}", cabeca.lines().next().unwrap_or(""));
    }
    // Levanta a bandeira e faz uma última conexão só para o accept() acordar e sair do laço
    parar.store(true, Ordering::SeqCst);
    let _ = TcpStream::connect(("127.0.0.1", porta));
    servidor.join().map_err(|_| "a thread do servidor falhou")?;
    let _ = fs::remove_dir_all(&pasta);
    println!("\nServidor encerrado e pasta temporária removida.");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None | Some("--ajuda") | Some("-h") => {
            println!("Uso: 42_servidor_http_minimo.exe <pasta> [--porta 8080]\n     42_servidor_http_minimo.exe --demo");
            return;
        }
        Some("--demo") => demo(),
        _ => executar(&args),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Gere uma listagem em HTML quando a pasta pedida não tiver index.html,
//    com um link por arquivo (fs::read_dir e o tamanho de cada um).
// 2. Atenda várias conexões ao mesmo tempo: crie uma thread por conexão
//    dentro de servir() e meça a diferença com dois navegadores abertos.
// 3. Acrescente a rota POST /api/soma que lê um corpo como "2,3,5" e
//    responde {"soma": 10}; devolva 400 quando algum número for inválido.
// ------------------------------------------------------------
