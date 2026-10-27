// ============================================================
// CodeNomad Academy  |  Sources Rust  |  33. Renomeador de arquivos em lote
// ------------------------------------------------------------
// O que faz: Renomeia todos os arquivos de uma pasta seguindo regras que você
//            combina: prefixo, sufixo, numeração, troca de texto e minúsculas.
//            Sempre mostra a prévia; só mexe nos arquivos com --aplicar e
//            nunca sobrescreve um arquivo que já existe.
// Como usar: rustc -O --edition 2021 33_renomeador_lote.rs -o renomear.exe
//            renomear.exe ./fotos --ext .jpg --trocar "IMG_" "viagem_" --minusculas --numerar
//            renomear.exe ./fotos --prefixo 2026_ --sufixo _final --aplicar
//            renomear.exe --demo
// O que você aprende: fs::read_dir, fs::rename e canonicalize, Peekable com
//            next_if para opções com valor opcional, enums com dados para o
//            estado de cada plano, HashSet para detectar colisões e PathBuf.
// Requisitos: Rust 1.70+, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::collections::HashSet;
use std::fs;
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::slice::Iter;

const USO: &str = "Uso: renomear <pasta> [regras] [--ext .jpg] [--aplicar]
  --prefixo <texto>       coloca o texto no começo do nome
  --sufixo <texto>        coloca o texto antes da extensão
  --trocar <de> <para>    troca um trecho do nome (a extensão não muda)
  --minusculas            deixa o nome todo em minúsculas
  --numerar [inicio]      numera na ordem alfabética: 01_, 02_, ...
  --ext <.ext>            só mexe nos arquivos com essa extensão
  --aplicar               sem esta opção o programa só mostra a prévia
  --demo                  roda uma demonstração completa";

struct Opcoes {
    pasta: PathBuf, prefixo: String, sufixo: String, trocar: Option<(String, String)>,
    minusculas: bool, numerar: Option<usize>, ext: Option<String>, aplicar: bool,
}

enum Estado { Renomear, SemMudanca, Conflito(&'static str) }

struct Plano { de: String, para: String, estado: Estado }

fn pegar<'a>(iter: &mut Peekable<Iter<'a, String>>, opcao: &str) -> Result<&'a String, String> {
    iter.next().ok_or(format!("a opção {opcao} precisa de um valor"))
}

fn ler_opcoes(args: &[String]) -> Result<Opcoes, String> {
    let mut o = Opcoes {
        pasta: PathBuf::new(), prefixo: String::new(), sufixo: String::new(), trocar: None,
        minusculas: false, numerar: None, ext: None, aplicar: false,
    };
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--prefixo" => o.prefixo = pegar(&mut iter, arg)?.clone(),
            "--sufixo" => o.sufixo = pegar(&mut iter, arg)?.clone(),
            "--trocar" => o.trocar = Some((pegar(&mut iter, arg)?.clone(), pegar(&mut iter, arg)?.clone())),
            "--minusculas" => o.minusculas = true,
            "--aplicar" => o.aplicar = true,
            "--ext" => o.ext = Some(pegar(&mut iter, arg)?.to_lowercase()),
            // next_if só consome o próximo argumento se ele for um número.
            "--numerar" => {
                let inicio = iter.next_if(|v| v.chars().all(|c| c.is_ascii_digit()) && !v.is_empty());
                o.numerar = Some(inicio.and_then(|v| v.parse().ok()).unwrap_or(1));
            }
            outro if outro.starts_with("--") => return Err(format!("opção desconhecida: {outro}\n{USO}")),
            pasta if o.pasta.as_os_str().is_empty() => o.pasta = PathBuf::from(pasta),
            extra => return Err(format!("argumento inesperado: '{extra}' (a pasta já foi informada)")),
        }
    }
    if o.pasta.as_os_str().is_empty() { return Err(format!("informe a pasta\n{USO}")); }
    if !o.pasta.is_dir() { return Err(format!("'{}' não é uma pasta", o.pasta.display())); }
    if let Some((de, _)) = &o.trocar { if de.is_empty() { return Err("o trecho a trocar não pode ser vazio".into()); } }
    Ok(o)
}

fn listar_arquivos(pasta: &Path, ext: Option<&str>) -> Result<Vec<String>, String> {
    let entradas = fs::read_dir(pasta).map_err(|e| format!("não consegui abrir a pasta {}: {e}", pasta.display()))?;
    let mut nomes: Vec<String> = entradas
        .filter_map(|r| r.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| ext.map_or(true, |x| n.to_lowercase().ends_with(x)))
        .collect();
    nomes.sort_by_key(|n| n.to_lowercase()); // a numeração segue a ordem alfabética
    Ok(nomes)
}

fn novo_nome(nome: &str, o: &Opcoes, indice: usize, largura: usize) -> String {
    // Separa "relatorio.final.pdf" em ("relatorio.final", ".pdf"); ".bashrc" fica inteiro.
    let (mut tronco, ext) = match nome.rsplit_once('.') {
        Some((t, e)) if !t.is_empty() => (t.to_string(), format!(".{e}")),
        _ => (nome.to_string(), String::new()),
    };
    if let Some((de, para)) = &o.trocar { tronco = tronco.replace(de.as_str(), para); }
    let mut novo = format!("{}{tronco}{}{ext}", o.prefixo, o.sufixo);
    if o.minusculas { novo = novo.to_lowercase(); }
    if let Some(inicio) = o.numerar { novo = format!("{:0largura$}_{novo}", inicio + indice); }
    novo
}

// No Windows "Foto.JPG" e "foto.jpg" são o mesmo arquivo; canonicalize resolve isso.
fn mesmo_arquivo(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) { (Ok(x), Ok(y)) => x == y, _ => false }
}

fn planejar(o: &Opcoes) -> Result<Vec<Plano>, String> {
    let nomes = listar_arquivos(&o.pasta, o.ext.as_deref())?;
    let largura = nomes.len().to_string().len().max(2);
    let mut reservados = HashSet::new();
    let mut planos = Vec::new();
    for (i, de) in nomes.iter().enumerate() {
        let para = novo_nome(de, o, i, largura);
        let destino = o.pasta.join(&para);
        let estado = if para == *de {
            Estado::SemMudanca
        } else if !reservados.insert(para.to_lowercase()) {
            Estado::Conflito("outro arquivo da lista já vai receber este nome")
        } else if destino.exists() && !mesmo_arquivo(&o.pasta.join(de), &destino) {
            Estado::Conflito("já existe um arquivo com este nome")
        } else {
            Estado::Renomear
        };
        planos.push(Plano { de: de.clone(), para, estado });
    }
    Ok(planos)
}

fn mostrar(planos: &[Plano]) -> usize {
    let (mut renomear, mut conflitos) = (0, 0);
    for p in planos {
        match &p.estado {
            Estado::Renomear => { renomear += 1; println!("  {:<26} -> {}", p.de, p.para); }
            Estado::SemMudanca => println!("  {:<26}    (sem mudança)", p.de),
            Estado::Conflito(motivo) => { conflitos += 1; println!("  {:<26} XX {} ({motivo})", p.de, p.para); }
        }
    }
    println!("{} arquivo(s) na lista, {renomear} para renomear, {conflitos} conflito(s).", planos.len());
    renomear
}

fn executar(args: &[String]) -> Result<(), String> {
    let o = ler_opcoes(args)?;
    let planos = planejar(&o)?;
    if planos.is_empty() { return Err(format!("nenhum arquivo em {} para renomear", o.pasta.display())); }
    println!("Prévia em {}:", o.pasta.display());
    let quantos = mostrar(&planos);
    if !o.aplicar {
        println!("Nada foi alterado. Repita o comando com --aplicar para renomear de verdade.");
        return Ok(());
    }
    let mut feitos = 0;
    for p in planos.iter().filter(|p| matches!(p.estado, Estado::Renomear)) {
        fs::rename(o.pasta.join(&p.de), o.pasta.join(&p.para)).map_err(|e| format!("falha ao renomear {}: {e}", p.de))?;
        feitos += 1;
    }
    println!("Pronto: {feitos} de {quantos} arquivo(s) renomeado(s).");
    Ok(())
}

fn demo() -> Result<(), String> {
    let pasta = std::env::temp_dir().join(format!("codenomad_renomear_{}", std::process::id()));
    fs::create_dir_all(&pasta).map_err(|e| format!("não consegui criar a pasta temporária: {e}"))?;
    let originais = ["Foto de Ferias 1.JPG", "Foto de Ferias 2.jpg", "IMG_0001.jpg", "Relatorio Final.docx", "notas.txt"];
    for nome in originais { fs::write(pasta.join(nome), nome).map_err(|e| e.to_string())?; }
    println!("=== Demonstração: renomeador de arquivos em lote ===");
    println!("Pasta temporária: {}", pasta.display());
    println!("Arquivos criados: {}\n", originais.join(", "));
    let p = pasta.to_string_lossy().to_string();
    let etapas: [(&str, &[&str]); 3] = [
        ("1. Prévia: só as fotos, trocando o texto, em minúsculas e numeradas",
         &["--ext", ".jpg", "--trocar", "Foto de Ferias ", "viagem_", "--minusculas", "--numerar"]),
        ("2. A mesma regra, agora com --aplicar",
         &["--ext", ".jpg", "--trocar", "Foto de Ferias ", "viagem_", "--minusculas", "--numerar", "--aplicar"]),
        ("3. Trocar 1 por 2 em tudo: um dos nomes novos já existe e é protegido",
         &["--trocar", "1", "2", "--aplicar"]),
    ];
    for (titulo, regras) in etapas {
        println!("--- {titulo} ---");
        println!("$ renomear <pasta> {}", regras.join(" "));
        let mut args = vec![p.clone()];
        args.extend(regras.iter().map(|r| r.to_string()));
        executar(&args)?;
        println!();
    }
    println!("Arquivos no final: {}", listar_arquivos(&pasta, None)?.join(", "));
    fs::remove_dir_all(&pasta).map_err(|e| format!("não consegui apagar a pasta temporária: {e}"))?;
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
// 1. Adicione --data para colocar a data de modificação do arquivo (fs::metadata) no nome.
// 2. Grave um arquivo "desfazer.txt" com os pares de nomes e crie a opção --desfazer.
// 3. Faça --trocar aceitar um padrão simples com "*" (ex.: "IMG_*" vira "foto_").
// ------------------------------------------------------------
