// ============================================================
// CodeNomad Academy  |  Sources Rust  |  43. Informações do sistema
// ------------------------------------------------------------
// O que faz: mostra sistema operacional, arquitetura, versão do compilador,
//            usuário, pasta atual, número de CPUs, memória (quando o sistema
//            expõe de forma simples), variáveis de ambiente filtradas por
//            prefixo e espaço livre em disco da pasta atual.
// Como usar: 43_info_sistema.exe --env PATH --env HOME
//            43_info_sistema.exe --env JAVA --sem-disco
//            43_info_sistema.exe --demo
// O que você aprende: std::env (consts, vars, current_dir, temp_dir),
//            thread::available_parallelism, cfg! para código por sistema,
//            process::Command com saída capturada e tratamento de falha,
//            Option encadeado com and_then/map e uma struct com Display.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::fmt;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs, process, thread};

/// Um par rótulo e valor; o Display alinha tudo em colunas
struct Linha { rotulo: &'static str, valor: String }

impl fmt::Display for Linha {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.rotulo.is_empty() { write!(f, "  {}", self.valor) } else { write!(f, "  {:<22} {}", self.rotulo, self.valor) }
    }
}

fn linha(rotulo: &'static str, valor: impl Into<String>) -> Linha { Linha { rotulo, valor: valor.into() } }

/// Roda um programa externo e devolve a saída como texto, ou None se ele não existir ou falhar
fn saida_de(programa: &str, args: &[&str]) -> Option<String> {
    let saida = Command::new(programa).args(args).output().ok()?;
    if !saida.status.success() { return None; }
    Some(String::from_utf8_lossy(&saida.stdout).trim().to_string())
}

/// Formata bytes como KiB, MiB ou GiB com uma casa depois da vírgula
fn humano(bytes: u64) -> String {
    const UNIDADES: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut valor = bytes as f64;
    let mut i = 0;
    while valor >= 1024.0 && i < UNIDADES.len() - 1 { valor /= 1024.0; i += 1; }
    format!("{valor:.1} {}", UNIDADES[i])
}

fn versao_compilador() -> String {
    // A biblioteca padrão não guarda a própria versão; perguntamos ao rustc instalado
    saida_de("rustc", &["--version"]).unwrap_or_else(|| "rustc não encontrado no PATH".into())
}

fn usuario() -> String {
    // Windows usa USERNAME; Linux e macOS usam USER. Tentamos os dois.
    env::var("USERNAME").or_else(|_| env::var("USER")).unwrap_or_else(|_| "desconhecido".into())
}

fn nome_maquina() -> String {
    env::var("COMPUTERNAME").ok().or_else(|| saida_de("hostname", &[])).unwrap_or_else(|| "desconhecido".into())
}

/// Memória total e livre. No Linux lemos /proc/meminfo; nos demais a std não expõe.
fn memoria() -> String {
    let Ok(texto) = fs::read_to_string("/proc/meminfo") else {
        return "a biblioteca padrão não expõe neste sistema".into();
    };
    let campo = |nome: &str| -> Option<u64> {
        texto.lines().find(|l| l.starts_with(nome))?.split_whitespace().nth(1)?.parse::<u64>().ok().map(|kib| kib * 1024)
    };
    match (campo("MemTotal:"), campo("MemAvailable:")) {
        (Some(total), Some(livre)) => format!("{} total, {} disponível", humano(total), humano(livre)),
        _ => "não consegui ler /proc/meminfo".into(),
    }
}

/// Espaço livre na unidade da pasta. Usa a ferramenta nativa de cada sistema.
fn disco(pasta: &Path) -> String {
    let pasta_texto = pasta.to_string_lossy();
    if cfg!(windows) {
        // "dir /-C" imprime os números sem separador de milhar; a última linha tem os bytes livres
        let Some(saida) = saida_de("cmd", &["/c", "dir", "/-C", &pasta_texto]) else { return "não consegui consultar".into() };
        let ultima = saida.lines().last().unwrap_or("");
        return match ultima.split_whitespace().filter_map(|p| p.parse::<u64>().ok()).last() {
            Some(bytes) => format!("{} livres na unidade de {}", humano(bytes), pasta_texto),
            None => "não entendi a resposta do sistema".into(),
        };
    }
    // Linux e macOS: df -k mostra 1K-blocos; a 4ª coluna da última linha é o disponível
    let Some(saida) = saida_de("df", &["-k", &pasta_texto]) else { return "não consegui consultar".into() };
    match saida.lines().last().and_then(|l| l.split_whitespace().nth(3)).and_then(|v| v.parse::<u64>().ok()) {
        Some(kib) => format!("{} livres na partição de {}", humano(kib * 1024), pasta_texto),
        None => "não entendi a resposta do sistema".into(),
    }
}

fn variaveis_filtradas(prefixos: &[String]) -> Vec<Linha> {
    let mut todas: Vec<(String, String)> = env::vars().collect();
    todas.sort();
    todas.into_iter()
        .filter(|(nome, _)| prefixos.iter().any(|p| nome.to_uppercase().starts_with(&p.to_uppercase())))
        .map(|(nome, valor)| {
            // Valores longos (como PATH) são cortados para caber na tela
            let curto = if valor.chars().count() > 70 { format!("{}...", valor.chars().take(67).collect::<String>()) } else { valor };
            Linha { rotulo: "", valor: format!("{nome}={curto}") }
        })
        .collect()
}

fn relatorio(prefixos: &[String], com_disco: bool) -> Result<(), String> {
    let pasta = env::current_dir().map_err(|e| format!("não consegui ler a pasta atual: {e}"))?;
    let cpus = thread::available_parallelism().map(|n| n.get().to_string()).unwrap_or_else(|_| "desconhecido".into());
    let epoch = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let exe = env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "desconhecido".into());
    println!("=== Informações do sistema ===");
    let basicas = [
        linha("Sistema", format!("{} ({})", env::consts::OS, env::consts::FAMILY)),
        linha("Arquitetura", format!("{}, ponteiros de {} bits", env::consts::ARCH, usize::BITS)),
        linha("Ordem de bytes", if cfg!(target_endian = "little") { "little-endian" } else { "big-endian" }),
        linha("Compilador", versao_compilador()),
        linha("Usuário", usuario()),
        linha("Máquina", nome_maquina()),
        linha("Pasta atual", pasta.display().to_string()),
        linha("Pasta temporária", env::temp_dir().display().to_string()),
        linha("Executável", exe),
        linha("CPUs lógicas", cpus),
        linha("Memória", memoria()),
        linha("Segundos desde 1970", epoch.to_string()),
    ];
    for l in &basicas { println!("{l}"); }
    if com_disco { println!("{}", linha("Disco", disco(&pasta))); }
    let filtradas = variaveis_filtradas(prefixos);
    println!("\nVariáveis de ambiente com prefixo {:?} ({} de {}):", prefixos, filtradas.len(), env::vars().count());
    if filtradas.is_empty() { println!("  (nenhuma)"); }
    for l in &filtradas { println!("{l}"); }
    Ok(())
}

fn executar(args: &[String]) -> Result<(), String> {
    let (mut prefixos, mut com_disco) = (Vec::new(), true);
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--env" => prefixos.push(it.next().ok_or("--env precisa de um prefixo logo depois")?.clone()),
            "--sem-disco" => com_disco = false,
            outro => return Err(format!("opção desconhecida: {outro} (use --env PREFIXO ou --sem-disco)")),
        }
    }
    if prefixos.is_empty() { prefixos = vec!["PATH".into(), "HOME".into(), "TEMP".into()]; }
    relatorio(&prefixos, com_disco)
}

fn demo() -> Result<(), String> {
    println!("=== Informações do sistema: demonstração ===\n");
    relatorio(&["PATH".into(), "HOME".into(), "USER".into(), "TEMP".into(), "LANG".into()], true)?;
    println!("\nTamanhos legíveis: {} | {} | {}", humano(512), humano(3_500_000), humano(17_179_869_184));
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None | Some("--ajuda") | Some("-h") => {
            println!("Uso: 43_info_sistema.exe [--env PREFIXO]... [--sem-disco]\n     43_info_sistema.exe --demo");
            return;
        }
        Some("--demo") => demo(),
        _ => executar(&args),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Acrescente o tempo ligado (uptime): no Linux leia /proc/uptime; no
//    Windows use "net statistics workstation" e extraia a data de início.
// 2. Mostre o tamanho total da pasta atual somando os arquivos com
//    fs::read_dir recursivo, e o maior arquivo encontrado.
// 3. Exporte o relatório em JSON com --json, reaproveitando a lista de
//    Linha e escapando as aspas e barras dos valores.
// ------------------------------------------------------------
