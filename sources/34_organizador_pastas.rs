// ============================================================
// CodeNomad Academy  |  Sources Rust  |  34. Organizador de pasta por tipo
// ------------------------------------------------------------
// O que faz: Olha a extensão de cada arquivo de uma pasta e o move para uma
//            subpasta por categoria (imagens, documentos, planilhas, áudio,
//            vídeo, código, outros). Mostra a prévia, só move com --aplicar,
//            grava um registro e desfaz tudo com --desfazer.
// Como usar: rustc -O --edition 2021 34_organizador_pastas.rs -o organizar.exe
//            organizar.exe ./Downloads              (só a prévia)
//            organizar.exe ./Downloads --aplicar    (move de verdade)
//            organizar.exe ./Downloads --desfazer   (volta tudo ao lugar)
//            organizar.exe --demo
// O que você aprende: enum com derive(Ord) e impl, match em fatias de texto,
//            BTreeMap para relatório ordenado, fs::read_dir, create_dir_all,
//            rename, remove_dir e metadata, e um registro em texto para desfazer.
// Requisitos: Rust 1.70+, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const USO: &str = "Uso: organizar <pasta> [--aplicar | --desfazer]
  sem opção     mostra a prévia: para onde cada arquivo iria
  --aplicar     move os arquivos para as subpastas e grava o registro
  --desfazer    lê o registro e devolve cada arquivo ao lugar original
  --demo        roda uma demonstração completa numa pasta temporária";
const REGISTRO: &str = "organizador_desfazer.txt";

// A ordem das variantes vira a ordem do relatório, graças ao derive(Ord).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Categoria { Imagens, Documentos, Planilhas, Audio, Video, Codigo, Outros }

impl Categoria {
    // O match em fatias de texto aceita vários valores na mesma linha com "|".
    fn da_extensao(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "heic" => Self::Imagens,
            "pdf" | "doc" | "docx" | "odt" | "txt" | "md" | "rtf" | "epub" => Self::Documentos,
            "xls" | "xlsx" | "ods" | "csv" | "tsv" => Self::Planilhas,
            "mp3" | "wav" | "ogg" | "flac" | "aac" | "m4a" => Self::Audio,
            "mp4" | "mkv" | "avi" | "mov" | "webm" | "wmv" => Self::Video,
            "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "h" | "html" | "css" | "json" | "sh" => Self::Codigo,
            _ => Self::Outros,
        }
    }
    fn pasta(self) -> &'static str {
        match self {
            Self::Imagens => "imagens", Self::Documentos => "documentos", Self::Planilhas => "planilhas",
            Self::Audio => "audio", Self::Video => "video", Self::Codigo => "codigo", Self::Outros => "outros",
        }
    }
}

struct Movimento { nome: String, categoria: Categoria, bytes: u64 }

// Lista só os arquivos soltos na raiz da pasta; subpastas e o registro ficam de fora.
fn planejar(pasta: &Path) -> Result<Vec<Movimento>, String> {
    let entradas = fs::read_dir(pasta).map_err(|e| format!("não consegui abrir {}: {e}", pasta.display()))?;
    let mut planos = Vec::new();
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("erro ao ler a pasta: {e}"))?;
        let caminho = entrada.path();
        let nome = entrada.file_name().to_string_lossy().to_string();
        if !caminho.is_file() || nome == REGISTRO { continue; }
        // extension() devolve None para "README" ou ".bashrc"; eles caem em "outros".
        let ext = caminho.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
        let bytes = fs::metadata(&caminho).map(|m| m.len()).unwrap_or(0);
        planos.push(Movimento { nome, categoria: Categoria::da_extensao(&ext), bytes });
    }
    planos.sort_by(|a, b| a.categoria.cmp(&b.categoria).then_with(|| a.nome.cmp(&b.nome)));
    Ok(planos)
}

fn tamanho_legivel(bytes: u64) -> String {
    match bytes {
        b if b >= 1_048_576 => format!("{:.1} MB", b as f64 / 1_048_576.0),
        b if b >= 1024 => format!("{:.1} KB", b as f64 / 1024.0),
        b => format!("{b} B"),
    }
}

fn relatorio(planos: &[Movimento]) {
    // BTreeMap mantém as chaves ordenadas, então o relatório sai na ordem do enum.
    let mut por_categoria: BTreeMap<Categoria, (usize, u64)> = BTreeMap::new();
    for p in planos {
        let total = por_categoria.entry(p.categoria).or_insert((0, 0));
        total.0 += 1;
        total.1 += p.bytes;
    }
    println!("{:<12} {:>8} {:>10}", "categoria", "arquivos", "tamanho");
    for (cat, (n, bytes)) in &por_categoria {
        println!("{:<12} {:>8} {:>10}", cat.pasta(), n, tamanho_legivel(*bytes));
    }
    println!("{:<12} {:>8} {:>10}", "total", planos.len(), tamanho_legivel(planos.iter().map(|p| p.bytes).sum()));
}

fn organizar(pasta: &Path, aplicar: bool) -> Result<(), String> {
    let planos = planejar(pasta)?;
    if planos.is_empty() { return Err(format!("não há arquivos soltos em {}", pasta.display())); }
    println!("{} em {}:", if aplicar { "Organizando" } else { "Prévia" }, pasta.display());
    for p in &planos {
        println!("  {:<28} -> {}/{}  ({})", p.nome, p.categoria.pasta(), p.nome, tamanho_legivel(p.bytes));
    }
    println!();
    relatorio(&planos);
    if !aplicar {
        println!("Nada foi movido. Repita com --aplicar para organizar de verdade.");
        return Ok(());
    }
    let mut registro = String::new();
    for p in &planos {
        let destino_pasta = pasta.join(p.categoria.pasta());
        fs::create_dir_all(&destino_pasta).map_err(|e| format!("não consegui criar {}: {e}", destino_pasta.display()))?;
        let destino = destino_pasta.join(&p.nome);
        if destino.exists() {
            println!("  pulado: {} já existe em {}", p.nome, p.categoria.pasta());
            continue;
        }
        fs::rename(pasta.join(&p.nome), &destino).map_err(|e| format!("falha ao mover {}: {e}", p.nome))?;
        // Cada linha do registro guarda "subpasta/nome" com a barra normal, em qualquer sistema.
        registro.push_str(&format!("{}/{}\n", p.categoria.pasta(), p.nome));
    }
    fs::write(pasta.join(REGISTRO), &registro).map_err(|e| format!("não consegui gravar o registro: {e}"))?;
    println!("Pronto: {} arquivo(s) movido(s). Registro em {REGISTRO}.", registro.lines().count());
    Ok(())
}

fn desfazer(pasta: &Path) -> Result<(), String> {
    let caminho_registro = pasta.join(REGISTRO);
    let texto = fs::read_to_string(&caminho_registro)
        .map_err(|_| format!("não achei o registro {} em {}; nada para desfazer", REGISTRO, pasta.display()))?;
    let (mut voltaram, mut faltando) = (0, 0);
    for linha in texto.lines().filter(|l| !l.trim().is_empty()) {
        let Some((sub, nome)) = linha.split_once('/') else { return Err(format!("linha inválida no registro: '{linha}'")) };
        let origem = pasta.join(sub).join(nome);
        if !origem.is_file() { faltando += 1; println!("  não encontrado: {linha}"); continue; }
        if pasta.join(nome).exists() { faltando += 1; println!("  pulado: {nome} já existe na raiz"); continue; }
        fs::rename(&origem, pasta.join(nome)).map_err(|e| format!("falha ao devolver {nome}: {e}"))?;
        voltaram += 1;
        // remove_dir só apaga pasta vazia, então não corre o risco de levar outros arquivos junto.
        let _ = fs::remove_dir(pasta.join(sub));
    }
    fs::remove_file(&caminho_registro).map_err(|e| format!("não consegui apagar o registro: {e}"))?;
    println!("Desfeito: {voltaram} arquivo(s) de volta na raiz, {faltando} pendente(s).");
    Ok(())
}

fn executar(args: &[String]) -> Result<(), String> {
    let mut pasta: Option<PathBuf> = None;
    let (mut aplicar, mut modo_desfazer) = (false, false);
    for arg in args {
        match arg.as_str() {
            "--aplicar" => aplicar = true,
            "--desfazer" => modo_desfazer = true,
            outro if outro.starts_with("--") => return Err(format!("opção desconhecida: {outro}\n{USO}")),
            caminho if pasta.is_none() => pasta = Some(PathBuf::from(caminho)),
            extra => return Err(format!("argumento inesperado: '{extra}' (a pasta já foi informada)")),
        }
    }
    let pasta = pasta.ok_or(format!("informe a pasta\n{USO}"))?;
    if !pasta.is_dir() { return Err(format!("'{}' não é uma pasta", pasta.display())); }
    if aplicar && modo_desfazer { return Err("escolha --aplicar ou --desfazer, não os dois".into()); }
    if modo_desfazer { desfazer(&pasta) } else { organizar(&pasta, aplicar) }
}

fn listar_arvore(pasta: &Path) -> Result<(), String> {
    let mut entradas: Vec<PathBuf> = fs::read_dir(pasta).map_err(|e| e.to_string())?
        .filter_map(|e| e.ok()).map(|e| e.path()).collect();
    entradas.sort();
    for caminho in entradas {
        let nome = caminho.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if caminho.is_dir() {
            let mut filhos: Vec<String> = fs::read_dir(&caminho).map_err(|e| e.to_string())?
                .filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).collect();
            filhos.sort();
            println!("  {nome}/  [{}]", filhos.join(", "));
        } else {
            println!("  {nome}");
        }
    }
    Ok(())
}

fn demo() -> Result<(), String> {
    let pasta = std::env::temp_dir().join(format!("codenomad_organizar_{}", std::process::id()));
    fs::create_dir_all(&pasta).map_err(|e| format!("não consegui criar a pasta temporária: {e}"))?;
    let bagunca = [
        ("ferias_praia.JPG", 3000), ("logo.png", 800), ("contrato.pdf", 12000), ("notas da aula.txt", 400),
        ("vendas_outubro.xlsx", 5000), ("clientes.csv", 900), ("musica.mp3", 40000), ("aula.mp4", 90000),
        ("main.rs", 1200), ("script.py", 600), ("README", 100), ("backup.zip", 20000),
    ];
    for (nome, tamanho) in bagunca { fs::write(pasta.join(nome), vec![b'x'; tamanho]).map_err(|e| e.to_string())?; }
    println!("=== Demonstração: organizador de pasta por tipo ===");
    println!("Pasta temporária: {}", pasta.display());
    println!("{} arquivos criados, tudo misturado.\n", bagunca.len());
    let p = pasta.to_string_lossy().to_string();
    for (titulo, extra) in [("1. Prévia", None), ("2. Aplicar", Some("--aplicar"))] {
        println!("--- {titulo} ---");
        let mut args = vec![p.clone()];
        args.extend(extra.map(String::from));
        executar(&args)?;
        println!();
    }
    println!("Como ficou a pasta:");
    listar_arvore(&pasta)?;
    println!("\n--- 3. Desfazer ---");
    executar(&[p, "--desfazer".to_string()])?;
    println!("\nA pasta depois de desfazer:");
    listar_arvore(&pasta)?;
    fs::remove_dir_all(&pasta).map_err(|e| format!("não consegui apagar a pasta temporária: {e}"))?;
    println!("\nPasta temporária apagada.");
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
// 1. Deixe o usuário passar um arquivo de regras (extensão=categoria) para criar categorias próprias.
// 2. Organize também por data de modificação (fs::metadata().modified()) em subpastas AAAA-MM.
// 3. Faça o --aplicar renomear "foto (2).jpg" em vez de pular quando o destino já existe.
// ------------------------------------------------------------
