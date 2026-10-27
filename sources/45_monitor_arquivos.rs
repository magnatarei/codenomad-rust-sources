// ============================================================
// CodeNomad Academy  |  Sources Rust  |  45. Monitor de alterações numa pasta
// ------------------------------------------------------------
// O que faz: observa uma pasta por consulta periódica (polling): a cada
//            intervalo tira uma "foto" dos arquivos (data de modificação e
//            tamanho), compara com a anterior e registra, com a hora, o que
//            foi criado, alterado ou removido. Pode descer nas subpastas.
// Como usar: 45_monitor_arquivos.exe ./meus_docs --intervalo 2 --recursivo
//            45_monitor_arquivos.exe ./meus_docs --vezes 10
//            45_monitor_arquivos.exe --demo
// O que você aprende: fs::read_dir e metadata, HashMap como "foto" do
//            estado, comparação de dois mapas para achar diferenças, enum
//            de eventos com Display, SystemTime e Duration, thread::sleep,
//            recursão com Result e escrita em pasta temporária.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs, process, thread};

/// O que sabemos de um arquivo numa "foto": quando mudou e quanto ocupa
#[derive(Clone, Copy, PartialEq)]
struct Estado { modificado: SystemTime, tamanho: u64 }

type Foto = HashMap<PathBuf, Estado>;

enum Evento { Criado(PathBuf), Alterado(PathBuf, u64, u64), Removido(PathBuf) }

impl fmt::Display for Evento {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Evento::Criado(p) => write!(f, "CRIADO    {}", p.display()),
            Evento::Alterado(p, antes, depois) => write!(f, "ALTERADO  {} ({antes} -> {depois} bytes)", p.display()),
            Evento::Removido(p) => write!(f, "REMOVIDO  {}", p.display()),
        }
    }
}

/// Hora atual em UTC no formato hh:mm:ss, sem bibliotecas de data
fn hora_agora() -> String {
    let s = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) % 86_400;
    format!("{:02}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60)
}

/// Tira a foto da pasta. Com `recursivo`, entra em cada subpasta.
fn fotografar(pasta: &Path, recursivo: bool, foto: &mut Foto) -> Result<(), String> {
    let entradas = fs::read_dir(pasta).map_err(|e| format!("não consegui ler {}: {e}", pasta.display()))?;
    for entrada in entradas {
        // Um arquivo pode sumir entre o read_dir e o metadata; nesse caso só o ignoramos
        let Ok(entrada) = entrada else { continue };
        let Ok(meta) = entrada.metadata() else { continue };
        let caminho = entrada.path();
        if meta.is_dir() {
            if recursivo { fotografar(&caminho, true, foto)?; }
            continue;
        }
        let modificado = meta.modified().unwrap_or(UNIX_EPOCH);
        foto.insert(caminho, Estado { modificado, tamanho: meta.len() });
    }
    Ok(())
}

/// Compara duas fotos e devolve os eventos em ordem alfabética de caminho
fn comparar(antes: &Foto, depois: &Foto) -> Vec<Evento> {
    let mut eventos = Vec::new();
    for (caminho, novo) in depois {
        match antes.get(caminho) {
            None => eventos.push(Evento::Criado(caminho.clone())),
            Some(velho) if velho != novo => eventos.push(Evento::Alterado(caminho.clone(), velho.tamanho, novo.tamanho)),
            Some(_) => {}
        }
    }
    eventos.extend(antes.keys().filter(|c| !depois.contains_key(*c)).map(|c| Evento::Removido(c.clone())));
    eventos.sort_by_key(|e| match e { Evento::Criado(p) | Evento::Alterado(p, ..) | Evento::Removido(p) => p.clone() });
    eventos
}

struct Monitor { pasta: PathBuf, recursivo: bool, intervalo: Duration, foto: Foto }

impl Monitor {
    fn novo(pasta: PathBuf, recursivo: bool, intervalo: Duration) -> Result<Self, String> {
        if !pasta.is_dir() { return Err(format!("{} não é uma pasta", pasta.display())); }
        let mut foto = Foto::new();
        fotografar(&pasta, recursivo, &mut foto)?;
        Ok(Monitor { pasta, recursivo, intervalo, foto })
    }

    /// Uma rodada: espera o intervalo, fotografa de novo e imprime as diferenças
    fn rodada(&mut self) -> Result<usize, String> {
        thread::sleep(self.intervalo);
        let mut nova = Foto::new();
        fotografar(&self.pasta, self.recursivo, &mut nova)?;
        let eventos = comparar(&self.foto, &nova);
        for evento in &eventos { println!("[{}] {evento}", hora_agora()); }
        self.foto = nova; // a foto nova vira a referência da próxima rodada
        Ok(eventos.len())
    }
}

fn executar(args: &[String]) -> Result<(), String> {
    let pasta = PathBuf::from(&args[0]);
    let (mut intervalo, mut recursivo, mut vezes) = (2u64, false, u64::MAX);
    let mut it = args[1..].iter();
    while let Some(opcao) = it.next() {
        match opcao.as_str() {
            "--recursivo" => recursivo = true,
            "--intervalo" | "--vezes" => {
                let valor = it.next().ok_or_else(|| format!("{opcao} precisa de um número logo depois"))?;
                let n = valor.parse::<u64>().ok().filter(|n| *n > 0).ok_or_else(|| format!("{opcao} espera um inteiro maior que zero, recebi \"{valor}\""))?;
                if opcao == "--intervalo" { intervalo = n } else { vezes = n }
            }
            outro => return Err(format!("opção desconhecida: {outro}")),
        }
    }
    let mut monitor = Monitor::novo(pasta, recursivo, Duration::from_secs(intervalo))?;
    println!("Observando {} ({} arquivo(s)) a cada {intervalo} s{}. Ctrl+C para parar.",
        monitor.pasta.display(), monitor.foto.len(), if recursivo { ", com subpastas" } else { "" });
    let mut total = 0;
    for _ in 0..vezes { total += monitor.rodada()?; }
    println!("Encerrado após {vezes} rodada(s) com {total} evento(s).");
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Monitor de alterações: demonstração numa pasta temporária ===");
    let pasta = env::temp_dir().join("codenomad_monitor_demo");
    let _ = fs::remove_dir_all(&pasta);
    fs::create_dir_all(pasta.join("sub")).map_err(|e| e.to_string())?;
    fs::write(pasta.join("notas.txt"), "primeira linha\n").map_err(|e| e.to_string())?;
    let mut monitor = Monitor::novo(pasta.clone(), true, Duration::from_millis(300))?;
    println!("Pasta: {}\nFoto inicial: {} arquivo(s)\n", pasta.display(), monitor.foto.len());

    println!("1) Criando sub/relatorio.csv e alterando notas.txt...");
    fs::write(pasta.join("sub").join("relatorio.csv"), "aluno,nota\nAna,9\n").map_err(|e| e.to_string())?;
    fs::write(pasta.join("notas.txt"), "primeira linha\nsegunda linha\n").map_err(|e| e.to_string())?;
    let n1 = monitor.rodada()?;

    println!("2) Removendo notas.txt e crescendo o relatório...");
    fs::remove_file(pasta.join("notas.txt")).map_err(|e| e.to_string())?;
    fs::write(pasta.join("sub").join("relatorio.csv"), "aluno,nota\nAna,9\nBruno,7\n").map_err(|e| e.to_string())?;
    let n2 = monitor.rodada()?;

    println!("3) Rodada sem mudanças (nada deve aparecer)...");
    let n3 = monitor.rodada()?;
    println!("\nEventos por rodada: {n1}, {n2}, {n3}. Total: {}", n1 + n2 + n3);
    let _ = fs::remove_dir_all(&pasta);
    println!("Pasta temporária removida.");
    println!("Erro tratado de propósito: {:?}", Monitor::novo(PathBuf::from("pasta_que_nao_existe"), false, Duration::ZERO).err());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None | Some("--ajuda") | Some("-h") => {
            println!("Uso: 45_monitor_arquivos.exe <pasta> [--intervalo segundos] [--recursivo] [--vezes N]");
            println!("     45_monitor_arquivos.exe --demo");
            return;
        }
        Some("--demo") => demo(),
        _ => executar(&args),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Aceite --filtro "*.txt" e só registre arquivos cujo nome termine com o
//    sufixo dado (ou vários sufixos separados por vírgula).
// 2. Grave os eventos também num arquivo de log, uma linha por evento, e
//    adicione --silencioso para não imprimir no terminal.
// 3. Detecte renomeações: um REMOVIDO e um CRIADO na mesma rodada com o
//    mesmo tamanho e a mesma data de modificação viram um só RENOMEADO.
// ------------------------------------------------------------
