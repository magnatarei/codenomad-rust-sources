// ============================================================
// CodeNomad Academy  |  Sources Rust  |  40. Backup de pasta com data
// ------------------------------------------------------------
// O que faz: Copia uma pasta inteira (com subpastas) para um destino com
//            carimbo de data e hora no nome, ignora os padrões que você
//            indicar, lista o que foi copiado e compara com o backup anterior
//            mostrando o que foi criado, alterado ou removido.
// Como usar: rustc -O --edition 2021 40_backup_pasta.rs -o backup.exe
//            backup.exe ./projeto ./backups
//            backup.exe ./projeto ./backups --ignorar "*.log" --ignorar target
//            backup.exe --comparar ./backups/backup_2026-10-27_101500 ./backups/backup_2026-10-27_183000
//            backup.exe --demo
// O que você aprende: percurso recursivo de pastas, fs::copy e create_dir_all,
//            caminhos relativos com strip_prefix, BTreeMap para comparar duas
//            árvores, SystemTime para o carimbo e um casamento simples com "*".
// Requisitos: Rust 1.70+, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const USO: &str = "Uso: backup <origem> <pasta_de_backups> [--ignorar padrão]...
       backup --comparar <backup_antigo> <backup_novo>
  --ignorar <padrão>   pula arquivos ou pastas que casem (ex.: *.log, target, rascunho*)
  --demo               roda uma demonstração completa numa pasta temporária";

// Casamento com um único "*": "*.log", "tmp*" ou nome exato. Suficiente para o dia a dia.
fn casa(padrao: &str, nome: &str) -> bool {
    match padrao.split_once('*') {
        Some((inicio, fim)) => nome.len() >= inicio.len() + fim.len() && nome.starts_with(inicio) && nome.ends_with(fim),
        None => padrao == nome,
    }
}

// Lista todos os arquivos abaixo da raiz, como caminhos relativos, pulando o que casar com os padrões.
fn listar(raiz: &Path, ignorar: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut pendentes = vec![raiz.to_path_buf()];
    let mut arquivos = Vec::new();
    while let Some(pasta) = pendentes.pop() {
        let entradas = fs::read_dir(&pasta).map_err(|e| format!("não consegui abrir {}: {e}", pasta.display()))?;
        for entrada in entradas.flatten() {
            let caminho = entrada.path();
            let nome = entrada.file_name().to_string_lossy().to_string();
            if ignorar.iter().any(|p| casa(p, &nome)) { continue; }
            if caminho.is_dir() { pendentes.push(caminho); }
            // strip_prefix tira a raiz e deixa só "sub/arquivo.txt".
            else if let Ok(relativo) = caminho.strip_prefix(raiz) { arquivos.push(relativo.to_path_buf()); }
        }
    }
    arquivos.sort();
    Ok(arquivos)
}

// Carimbo em UTC: a biblioteca padrão não conhece o fuso local, e para ordenar backups isso basta.
fn carimbo() -> String {
    let segundos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (dias, resto) = (segundos / 86_400, segundos % 86_400);
    let z = dias as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let dia_da_era = z - era * 146_097;
    let ano_da_era = (dia_da_era - dia_da_era / 1460 + dia_da_era / 36_524 - dia_da_era / 146_096) / 365;
    let dia_do_ano = dia_da_era - (365 * ano_da_era + ano_da_era / 4 - ano_da_era / 100);
    let mes_deslocado = (5 * dia_do_ano + 2) / 153;
    let dia = dia_do_ano - (153 * mes_deslocado + 2) / 5 + 1;
    let mes = if mes_deslocado < 10 { mes_deslocado + 3 } else { mes_deslocado - 9 };
    let ano = ano_da_era + era * 400 + i64::from(mes <= 2);
    format!("{ano:04}-{mes:02}-{dia:02}_{:02}{:02}{:02}", resto / 3600, resto % 3600 / 60, resto % 60)
}

fn tamanho_legivel(bytes: u64) -> String {
    match bytes {
        b if b >= 1_048_576 => format!("{:.1} MB", b as f64 / 1_048_576.0),
        b if b >= 1024 => format!("{:.1} KB", b as f64 / 1024.0),
        b => format!("{b} B"),
    }
}

// Backups anteriores são as subpastas "backup_*" do destino, em ordem alfabética (que é a cronológica).
fn backups_existentes(destino: &Path) -> Vec<PathBuf> {
    let mut lista: Vec<PathBuf> = fs::read_dir(destino).map(|rd| rd.flatten().map(|e| e.path())
        .filter(|p| p.is_dir() && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("backup_"))).collect())
        .unwrap_or_default();
    lista.sort();
    lista
}

fn fazer_backup(origem: &Path, destino: &Path, ignorar: &[String]) -> Result<PathBuf, String> {
    if !origem.is_dir() { return Err(format!("'{}' não é uma pasta", origem.display())); }
    fs::create_dir_all(destino).map_err(|e| format!("não consegui criar {}: {e}", destino.display()))?;
    let anterior = backups_existentes(destino).pop();
    // Se dois backups caírem no mesmo segundo, o segundo ganha um sufixo em vez de sobrescrever.
    let nome = format!("backup_{}", carimbo());
    let pasta = (1..).map(|n| destino.join(if n == 1 { nome.clone() } else { format!("{nome}_{n}") }))
        .find(|p| !p.exists()).unwrap_or_else(|| destino.join(&nome));
    let arquivos = listar(origem, ignorar)?;
    if arquivos.is_empty() { return Err(format!("nada para copiar em {} (tudo foi ignorado?)", origem.display())); }
    println!("Copiando {} -> {}", origem.display(), pasta.display());
    let mut total = 0;
    for relativo in &arquivos {
        let alvo = pasta.join(relativo);
        if let Some(pai) = alvo.parent() { fs::create_dir_all(pai).map_err(|e| format!("não consegui criar {}: {e}", pai.display()))?; }
        let bytes = fs::copy(origem.join(relativo), &alvo).map_err(|e| format!("falha ao copiar {}: {e}", relativo.display()))?;
        total += bytes;
        println!("  + {:<32} {:>9}", relativo.display(), tamanho_legivel(bytes));
    }
    println!("{} arquivo(s), {} copiados.", arquivos.len(), tamanho_legivel(total));
    match anterior {
        Some(antigo) => { println!("\nComparando com o backup anterior:"); comparar(&antigo, &pasta)?; }
        None => println!("Primeiro backup desta pasta; não há anterior para comparar."),
    }
    Ok(pasta)
}

fn comparar(antigo: &Path, novo: &Path) -> Result<(), String> {
    for p in [antigo, novo] { if !p.is_dir() { return Err(format!("'{}' não é uma pasta", p.display())); } }
    // BTreeMap de caminho relativo para tamanho: as chaves ficam ordenadas e a comparação vira um passeio.
    let mapa = |raiz: &Path| -> Result<BTreeMap<PathBuf, u64>, String> {
        listar(raiz, &[])?.into_iter().map(|rel| {
            let tamanho = fs::metadata(raiz.join(&rel)).map(|m| m.len()).map_err(|e| format!("{}: {e}", rel.display()))?;
            Ok((rel, tamanho))
        }).collect()
    };
    let (a, b) = (mapa(antigo)?, mapa(novo)?);
    let (mut criados, mut alterados, mut removidos) = (0, 0, 0);
    for (rel, tamanho) in &b {
        match a.get(rel) {
            None => { criados += 1; println!("  criado    {}", rel.display()); }
            Some(antes) if antes != tamanho || fs::read(antigo.join(rel)).ok() != fs::read(novo.join(rel)).ok() => {
                alterados += 1;
                println!("  alterado  {}  ({} -> {})", rel.display(), tamanho_legivel(*antes), tamanho_legivel(*tamanho));
            }
            Some(_) => {}
        }
    }
    for rel in a.keys().filter(|rel| !b.contains_key(*rel)) { removidos += 1; println!("  removido  {}", rel.display()); }
    println!("{criados} criado(s), {alterados} alterado(s), {removidos} removido(s), {} sem mudança.", b.len() - criados - alterados);
    Ok(())
}

fn executar(args: &[String]) -> Result<(), String> {
    if let [flag, antigo, novo] = args { if flag == "--comparar" { return comparar(Path::new(antigo), Path::new(novo)); } }
    let mut posicionais: Vec<&str> = Vec::new();
    let mut ignorar = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--ignorar" => ignorar.push(iter.next().ok_or("--ignorar precisa de um padrão")?.clone()),
            outro if outro.starts_with("--") => return Err(format!("opção desconhecida: {outro}\n{USO}")),
            caminho => posicionais.push(caminho),
        }
    }
    let [origem, destino] = posicionais.as_slice() else { return Err(format!("informe a origem e a pasta de backups\n{USO}")) };
    fazer_backup(Path::new(origem), Path::new(destino), &ignorar)?;
    Ok(())
}

fn demo() -> Result<(), String> {
    let base = std::env::temp_dir().join(format!("codenomad_backup_{}", std::process::id()));
    let (origem, destino) = (base.join("projeto"), base.join("backups"));
    fs::create_dir_all(origem.join("src")).map_err(|e| format!("não consegui criar a pasta temporária: {e}"))?;
    let iniciais = [("README.md", "# Projeto\n"), ("src/main.rs", "fn main() {}\n"), ("src/util.rs", "pub fn soma() {}\n"),
        ("notas.txt", "lista de ideias\n"), ("debug.log", "linha 1\nlinha 2\n")];
    for (nome, conteudo) in iniciais { fs::write(origem.join(nome), conteudo).map_err(|e| e.to_string())?; }
    println!("=== Demonstração: backup de pasta com data ===");
    println!("Pasta temporária: {}\n", base.display());
    let (o, d) = (origem.to_string_lossy().to_string(), destino.to_string_lossy().to_string());
    println!("--- 1. Primeiro backup, ignorando *.log ---");
    executar(&[o.clone(), d.clone(), "--ignorar".into(), "*.log".into()])?;
    println!("\n--- 2. Mudanças no projeto: main.rs cresce, config.toml nasce, notas.txt some ---");
    fs::write(origem.join("src/main.rs"), "fn main() {\n    println!(\"olá\");\n}\n").map_err(|e| e.to_string())?;
    fs::write(origem.join("config.toml"), "[app]\nnome = \"demo\"\n").map_err(|e| e.to_string())?;
    fs::remove_file(origem.join("notas.txt")).map_err(|e| e.to_string())?;
    println!("\n--- 3. Segundo backup, com a comparação automática ---");
    executar(&[o, d, "--ignorar".into(), "*.log".into()])?;
    let lista = backups_existentes(&destino);
    println!("\nBackups guardados: {}", lista.iter().filter_map(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).collect::<Vec<_>>().join(", "));
    fs::remove_dir_all(&base).map_err(|e| format!("não consegui apagar a pasta temporária: {e}"))?;
    println!("Pasta temporária apagada.");
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
// 1. Crie --manter N para apagar os backups mais antigos e guardar só os N últimos.
// 2. Faça um backup incremental: copie só o que mudou em relação ao anterior e registre o resto num índice.
// 3. Leia os padrões a ignorar de um arquivo .backupignore na origem, um por linha.
// ------------------------------------------------------------
