// ============================================================
// CodeNomad Academy  |  Sources Rust  |  15. Cifra de César e de Vigenère
// ------------------------------------------------------------
// O que faz: Codifica e decodifica texto com a cifra de César (deslocamento
//            fixo) e a de Vigenère (palavra-chave), preservando maiúsculas,
//            espaços e pontuação. Inclui um quebrador de César que descobre
//            a chave comparando a frequência das letras com a do português.
// Como usar: 15_cifra_cesar_vigenere.exe cesar 3 "Ataque ao amanhecer"
//            15_cifra_cesar_vigenere.exe vigenere segredo "Ataque ao amanhecer"
//            15_cifra_cesar_vigenere.exe quebrar "Dwdtxh dr dpdqkhfhu"
//            15_cifra_cesar_vigenere.exe --demo
// O que você aprende: aritmética com chars e bytes ASCII, rem_euclid para
//            módulo de negativos, closures que capturam estado mutável,
//            ordenação de f64 com partial_cmp, tuplas em Vec, análise de
//            frequência (qui-quadrado).
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::cmp::Ordering;

// Frequência aproximada de cada letra (a..z) em textos de português, em %.
// É a "assinatura" da língua: o quebrador procura o deslocamento que mais
// se parece com ela.
const FREQUENCIA_PT: [f64; 26] = [
    14.63, 1.04, 3.88, 4.99, 12.57, 1.02, 1.30, 1.28, 6.18, 0.40, 0.02, 2.78, 4.74,
    5.05, 10.73, 2.52, 1.20, 6.53, 7.81, 4.34, 4.63, 1.67, 0.01, 0.21, 0.01, 0.47,
];

/// Desloca uma letra ASCII dentro do alfabeto, mantendo a caixa. Qualquer
/// outro símbolo (espaço, vírgula, letra acentuada) passa sem mudar.
fn deslocar(c: char, deslocamento: i32) -> char {
    let base = if c.is_ascii_lowercase() {
        b'a'
    } else if c.is_ascii_uppercase() {
        b'A'
    } else {
        return c;
    };
    let posicao = i32::from(c as u8 - base);
    // rem_euclid devolve sempre 0..25, mesmo quando o deslocamento é negativo.
    let nova = (posicao + deslocamento).rem_euclid(26) as u8;
    char::from(base + nova)
}

fn cesar(texto: &str, deslocamento: i32) -> String {
    texto.chars().map(|c| deslocar(c, deslocamento)).collect()
}

/// A chave vira uma lista de deslocamentos: a=0, b=1, ... z=25.
fn preparar_chave(chave: &str) -> Result<Vec<i32>, String> {
    if chave.is_empty() {
        return Err("a chave não pode ser vazia".to_string());
    }
    if !chave.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(format!("a chave '{chave}' só pode ter letras de A a Z, sem acento"));
    }
    Ok(chave.chars().map(|c| i32::from(c.to_ascii_lowercase() as u8 - b'a')).collect())
}

/// Vigenère: cada letra do texto usa a próxima letra da chave como deslocamento.
/// A chave só avança nas letras, para espaços e pontuação não a "gastarem".
fn vigenere(texto: &str, chave: &[i32], decifrar: bool) -> String {
    let mut indice = 0;
    texto
        .chars()
        .map(|c| {
            if !c.is_ascii_alphabetic() {
                return c;
            }
            let d = chave[indice % chave.len()];
            indice += 1;
            deslocar(c, if decifrar { -d } else { d })
        })
        .collect()
}

/// Qui-quadrado entre as letras do texto e a frequência do português.
/// Quanto menor, mais o texto "parece português".
fn qui_quadrado(texto: &str) -> f64 {
    let mut contagem = [0usize; 26];
    for c in texto.chars().filter(char::is_ascii_alphabetic) {
        contagem[usize::from(c.to_ascii_lowercase() as u8 - b'a')] += 1;
    }
    let total = contagem.iter().sum::<usize>() as f64;
    if total == 0.0 {
        return f64::INFINITY;
    }
    contagem
        .iter()
        .zip(FREQUENCIA_PT)
        .map(|(observado, esperado_pct)| {
            let esperado = total * esperado_pct / 100.0;
            let diferenca = *observado as f64 - esperado;
            diferenca * diferenca / esperado
        })
        .sum()
}

/// Testa os 26 deslocamentos e devolve (chave, pontuação, texto) do mais
/// provável ao menos provável.
fn quebrar_cesar(cifrado: &str) -> Vec<(i32, f64, String)> {
    let mut candidatos: Vec<(i32, f64, String)> = (0..26)
        .map(|chave| {
            let texto = cesar(cifrado, -chave);
            (chave, qui_quadrado(&texto), texto)
        })
        .collect();
    // f64 não implementa Ord (por causa do NaN), então usamos partial_cmp.
    candidatos.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
    candidatos
}

fn mostrar_quebra(cifrado: &str) {
    println!("Cifrado: {cifrado}");
    let candidatos = quebrar_cesar(cifrado);
    println!("Os três deslocamentos mais prováveis:");
    for (chave, pontuacao, texto) in candidatos.iter().take(3) {
        println!("  chave {chave:>2}  (qui-quadrado {pontuacao:>8.1})  {texto}");
    }
    if let Some((chave, _, texto)) = candidatos.first() {
        println!("Melhor palpite: chave {chave}, texto \"{texto}\"");
    }
}

fn demo() -> Result<(), String> {
    println!("=== Cifra de César e de Vigenère: demonstração ===\n");
    let frase = "A lógica é a base de toda programação, e praticar todo dia faz diferença!";
    println!("{:<19}{frase}", "Original:");
    let com_cesar = cesar(frase, 3);
    println!("{:<19}{com_cesar}", "César (+3):");
    println!("{:<19}{}", "César (-3):", cesar(&com_cesar, -3));
    let chave = preparar_chave("nomade")?;
    let com_vigenere = vigenere(frase, &chave, false);
    println!("{:<19}{com_vigenere}", "Vigenère (nomade):");
    println!("{:<19}{}\n", "Decifrado:", vigenere(&com_vigenere, &chave, true));
    println!("Repare que acentos, espaços e pontuação ficam no lugar.\n");

    let segredo = "Quem estuda todos os dias, mesmo que por pouco tempo, aprende mais do que \
quem estuda muitas horas de uma vez. O segredo está na constância e na prática.";
    println!("Quebrando um texto de César sem conhecer a chave (usada: 11):");
    mostrar_quebra(&cesar(segredo, 11));
    Ok(())
}

fn uso() {
    println!("Uso:");
    println!("  15_cifra_cesar_vigenere.exe cesar <deslocamento> <texto>     (negativo decifra)");
    println!("  15_cifra_cesar_vigenere.exe vigenere <chave> <texto>");
    println!("  15_cifra_cesar_vigenere.exe devigenere <chave> <texto>");
    println!("  15_cifra_cesar_vigenere.exe quebrar <texto cifrado com César>");
    println!("  15_cifra_cesar_vigenere.exe --demo");
}

fn executar(args: &[String]) -> Result<(), String> {
    let comando = args.first().map(String::as_str);
    if comando == Some("--demo") {
        return demo();
    }
    let Some(comando) = comando else {
        uso();
        return Ok(());
    };
    // Tudo depois do primeiro parâmetro é o texto, mesmo sem aspas.
    let texto_de = |a_partir: usize| -> Result<String, String> {
        let texto = args[a_partir.min(args.len())..].join(" ");
        if texto.is_empty() { Err("faltou o texto".to_string()) } else { Ok(texto) }
    };
    match comando {
        "cesar" => {
            let bruto = args.get(1).ok_or("faltou o deslocamento, ex.: cesar 3 \"texto\"")?;
            let deslocamento: i32 = bruto.parse().map_err(|_| format!("deslocamento inválido: '{bruto}' (use um inteiro)"))?;
            println!("{}", cesar(&texto_de(2)?, deslocamento));
        }
        "vigenere" | "devigenere" => {
            let chave = preparar_chave(args.get(1).map(String::as_str).unwrap_or(""))?;
            println!("{}", vigenere(&texto_de(2)?, &chave, comando == "devigenere"));
        }
        "quebrar" => mostrar_quebra(&texto_de(1)?),
        outro => return Err(format!("comando desconhecido '{outro}'; use cesar, vigenere, devigenere, quebrar ou --demo")),
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(erro) = executar(&args) {
        eprintln!("Erro: {erro}");
        std::process::exit(1);
    }
}

// Para ir além:
// 1. Estenda `deslocar` para girar também os dígitos (0..9), com módulo 10.
// 2. Quebre a Vigenère: descubra o tamanho da chave pelo índice de
//    coincidência e depois quebre cada "coluna" como se fosse César.
// 3. Leia o texto de um arquivo e grave o resultado em outro, com --origem
//    e --destino, tratando os erros de leitura e escrita.
