// ============================================================
// CodeNomad Academy  |  Sources Rust  |  35. Detector de arquivos duplicados
// ------------------------------------------------------------
// O que faz: Percorre uma pasta (com subpastas), agrupa os arquivos por
//            tamanho e, só entre os de tamanho igual, calcula o SHA-256 de
//            cada um. Lista os grupos de duplicados, quanto espaço seria
//            liberado e, se você pedir, grava um relatório em texto.
// Como usar: rustc -O --edition 2021 35_duplicados_hash.rs -o duplicados.exe
//            duplicados.exe ./fotos
//            duplicados.exe ./fotos --relatorio duplicados.txt
//            duplicados.exe --demo
// O que você aprende: SHA-256 implementado do zero (rotate_right, wrapping_add,
//            to_be_bytes), percurso de pastas com uma pilha em vez de recursão,
//            HashMap<u64, Vec<PathBuf>> para agrupar, sort_by_key com Reverse.
// Requisitos: Rust 1.70+, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const USO: &str = "Uso: duplicados <pasta> [--relatorio saida.txt]
  Lista os arquivos repetidos dentro da pasta e de todas as subpastas.
  --relatorio <arquivo>   grava a mesma lista num arquivo de texto
  --demo                  roda uma demonstração completa";

// Constantes do SHA-256: as primeiras 32 casas da raiz cúbica dos 64 primeiros primos.
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

// SHA-256 segue o padrão FIPS 180-4: acolchoa a mensagem, quebra em blocos de 64 bytes
// e mistura cada bloco em 8 palavras de 32 bits. wrapping_add ignora o estouro de propósito.
fn sha256(dados: &[u8]) -> String {
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut msg = dados.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&((dados.len() as u64) * 8).to_be_bytes());
    for bloco in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, pedaco) in bloco.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([pedaco[0], pedaco[1], pedaco[2], pedaco[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for (ki, wi) in K.iter().zip(w) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let escolha = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(escolha).wrapping_add(*ki).wrapping_add(wi);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maioria = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maioria);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        for (acumulado, novo) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) { *acumulado = acumulado.wrapping_add(novo); }
    }
    h.iter().map(|palavra| format!("{palavra:08x}")).collect()
}

// Uma pilha de pastas pendentes substitui a recursão: funciona para árvores bem fundas.
fn listar_arquivos(raiz: &Path) -> Result<Vec<(PathBuf, u64)>, String> {
    let mut pendentes = vec![raiz.to_path_buf()];
    let mut arquivos = Vec::new();
    while let Some(pasta) = pendentes.pop() {
        let entradas = fs::read_dir(&pasta).map_err(|e| format!("não consegui abrir {}: {e}", pasta.display()))?;
        for entrada in entradas.flatten() {
            let caminho = entrada.path();
            if caminho.is_dir() { pendentes.push(caminho); }
            else if let Ok(meta) = entrada.metadata() { arquivos.push((caminho, meta.len())); }
        }
    }
    arquivos.sort();
    Ok(arquivos)
}

struct Grupo { hash: String, tamanho: u64, caminhos: Vec<PathBuf> }

fn achar_duplicados(raiz: &Path) -> Result<(usize, Vec<Grupo>), String> {
    let arquivos = listar_arquivos(raiz)?;
    // Primeira peneira: tamanhos diferentes nunca são duplicados, então nem precisam de hash.
    let mut por_tamanho: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for (caminho, tamanho) in &arquivos { por_tamanho.entry(*tamanho).or_default().push(caminho.clone()); }
    let mut grupos = Vec::new();
    for (tamanho, candidatos) in por_tamanho.into_iter().filter(|(_, lista)| lista.len() > 1) {
        let mut por_hash: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for caminho in candidatos {
            let conteudo = fs::read(&caminho).map_err(|e| format!("não consegui ler {}: {e}", caminho.display()))?;
            por_hash.entry(sha256(&conteudo)).or_default().push(caminho);
        }
        for (hash, mut caminhos) in por_hash.into_iter().filter(|(_, lista)| lista.len() > 1) {
            caminhos.sort();
            grupos.push(Grupo { hash, tamanho, caminhos });
        }
    }
    // Reverse inverte a ordem: os grupos que mais desperdiçam espaço vêm primeiro.
    grupos.sort_by_key(|g| (Reverse(g.tamanho * (g.caminhos.len() as u64 - 1)), g.hash.clone()));
    Ok((arquivos.len(), grupos))
}

fn tamanho_legivel(bytes: u64) -> String {
    match bytes {
        b if b >= 1_048_576 => format!("{:.1} MB", b as f64 / 1_048_576.0),
        b if b >= 1024 => format!("{:.1} KB", b as f64 / 1024.0),
        b => format!("{b} B"),
    }
}

fn montar_relatorio(raiz: &Path, total: usize, grupos: &[Grupo]) -> String {
    let mut texto = format!("Pasta analisada: {}\nArquivos examinados: {total}\n", raiz.display());
    if grupos.is_empty() { texto.push_str("Nenhum arquivo duplicado.\n"); return texto; }
    let mut liberavel = 0;
    for (n, g) in grupos.iter().enumerate() {
        liberavel += g.tamanho * (g.caminhos.len() as u64 - 1);
        texto.push_str(&format!("\nGrupo {}: {} cópias de {} (sha256 {}...)\n", n + 1, g.caminhos.len(),
            tamanho_legivel(g.tamanho), &g.hash[..12]));
        for c in &g.caminhos {
            let relativo = c.strip_prefix(raiz).unwrap_or(c);
            texto.push_str(&format!("    {}\n", relativo.display()));
        }
    }
    texto.push_str(&format!("\n{} grupo(s) de duplicados. Mantendo uma cópia de cada, você libera {}.\n",
        grupos.len(), tamanho_legivel(liberavel)));
    texto
}

fn executar(args: &[String]) -> Result<(), String> {
    let mut pasta: Option<PathBuf> = None;
    let mut relatorio: Option<PathBuf> = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--relatorio" => relatorio = Some(PathBuf::from(iter.next().ok_or("--relatorio precisa de um caminho")?)),
            outro if outro.starts_with("--") => return Err(format!("opção desconhecida: {outro}\n{USO}")),
            caminho if pasta.is_none() => pasta = Some(PathBuf::from(caminho)),
            extra => return Err(format!("argumento inesperado: '{extra}' (a pasta já foi informada)")),
        }
    }
    let pasta = pasta.ok_or(format!("informe a pasta\n{USO}"))?;
    if !pasta.is_dir() { return Err(format!("'{}' não é uma pasta", pasta.display())); }
    let (total, grupos) = achar_duplicados(&pasta)?;
    let texto = montar_relatorio(&pasta, total, &grupos);
    print!("{texto}");
    if let Some(destino) = relatorio {
        fs::write(&destino, &texto).map_err(|e| format!("não consegui gravar {}: {e}", destino.display()))?;
        println!("Relatório gravado em {}.", destino.display());
    }
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Demonstração: detector de arquivos duplicados ===");
    // Autoteste: o SHA-256 de "abc" é conhecido e serve para conferir a implementação.
    let esperado = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    println!("Autoteste do SHA-256 de \"abc\": {}", if sha256(b"abc") == esperado { "confere" } else { "ERRADO" });
    let pasta = std::env::temp_dir().join(format!("codenomad_duplicados_{}", std::process::id()));
    for sub in ["fotos", "fotos/backup", "docs"] {
        fs::create_dir_all(pasta.join(sub)).map_err(|e| format!("não consegui criar a pasta temporária: {e}"))?;
    }
    let foto = vec![0xAB; 4096];
    let arquivos: [(&str, Vec<u8>); 8] = [
        ("fotos/praia.jpg", foto.clone()), ("fotos/backup/praia (copia).jpg", foto.clone()),
        ("fotos/backup/IMG_0001.jpg", foto), ("fotos/montanha.jpg", vec![0xCD; 4096]),
        ("docs/contrato.pdf", b"contrato assinado".repeat(100)), ("docs/contrato_v2.pdf", b"contrato assinado".repeat(100)),
        ("docs/notas.txt", "anotações da aula".as_bytes().to_vec()), ("docs/lista.txt", b"compras da semana".to_vec()),
    ];
    for (nome, conteudo) in &arquivos { fs::write(pasta.join(nome), conteudo).map_err(|e| e.to_string())?; }
    println!("Pasta temporária: {}", pasta.display());
    println!("{} arquivos criados: 3 fotos iguais, 2 contratos iguais e 3 arquivos únicos", arquivos.len());
    println!("(montanha.jpg tem o mesmo tamanho da praia, mas conteúdo diferente).\n");
    let relatorio = pasta.join("relatorio.txt");
    executar(&[pasta.to_string_lossy().to_string(), "--relatorio".into(), relatorio.to_string_lossy().to_string()])?;
    println!("\nPrimeiras linhas do relatório gravado:");
    for linha in fs::read_to_string(&relatorio).map_err(|e| e.to_string())?.lines().take(3) { println!("  | {linha}"); }
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
// 1. Leia os arquivos grandes em blocos (BufReader + read) em vez de carregar tudo na memória.
// 2. Crie a opção --apagar que mantém a primeira cópia de cada grupo e pede confirmação antes de remover as outras.
// 3. Compare só os primeiros 4 KB antes do SHA-256 completo e meça quanto tempo isso economiza.
// ------------------------------------------------------------
