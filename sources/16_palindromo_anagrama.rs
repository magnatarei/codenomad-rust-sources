// ============================================================
// CodeNomad Academy  |  Sources Rust  |  16. Palíndromos e anagramas
// ------------------------------------------------------------
// O que faz: Verifica se uma frase é palíndromo ignorando acentos, espaços,
//            maiúsculas e pontuação; verifica se duas frases são anagramas
//            uma da outra; e agrupa uma lista de palavras por anagrama.
// Como usar: 16_palindromo_anagrama.exe palindromo "Socorram-me, subi no ônibus em Marrocos"
//            16_palindromo_anagrama.exe anagrama "amor" "Roma"
//            16_palindromo_anagrama.exe agrupar amor roma mora pedra perda gato
//            16_palindromo_anagrama.exe --demo
// O que você aprende: normalização de texto com flat_map e filter,
//            comparação de iteradores com eq e rev, sort_unstable em Vec<char>,
//            HashMap com entry().or_default(), into_values, fatias de
//            argumentos com padrões (match em &[String]).
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::collections::HashMap;

/// Troca letras acentuadas pela versão sem acento.
fn sem_acento(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        'ñ' => 'n',
        outro => outro,
    }
}

/// Deixa só letras e dígitos, em minúsculas e sem acento. É isso que torna
/// "Roma me tem amor" comparável letra a letra.
fn normalizar(texto: &str) -> String {
    texto
        .chars()
        .flat_map(char::to_lowercase)
        .map(sem_acento)
        .filter(|c| c.is_alphanumeric())
        .collect()
}

fn eh_palindromo(texto: &str) -> bool {
    let limpo = normalizar(texto);
    // `eq` compara dois iteradores item a item: o texto e ele mesmo de trás para frente.
    !limpo.is_empty() && limpo.chars().eq(limpo.chars().rev())
}

/// A "assinatura" de um anagrama: as letras normalizadas em ordem.
/// "roma", "amor" e "mora" viram todas "amor".
fn chave_anagrama(texto: &str) -> String {
    let mut letras: Vec<char> = normalizar(texto).chars().collect();
    letras.sort_unstable();
    letras.into_iter().collect()
}

fn sao_anagramas(a: &str, b: &str) -> bool {
    let (chave_a, chave_b) = (chave_anagrama(a), chave_anagrama(b));
    !chave_a.is_empty() && chave_a == chave_b && normalizar(a) != normalizar(b)
}

/// Agrupa palavras que são anagramas entre si. Grupos maiores vêm primeiro;
/// empates seguem a ordem alfabética da primeira palavra.
fn agrupar(palavras: &[String]) -> Vec<Vec<String>> {
    let mut grupos: HashMap<String, Vec<String>> = HashMap::new();
    for palavra in palavras {
        grupos.entry(chave_anagrama(palavra)).or_default().push(palavra.clone());
    }
    let mut lista: Vec<Vec<String>> = grupos.into_values().collect();
    lista.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.first().cmp(&b.first())));
    lista
}

fn mostrar_palindromo(texto: &str) {
    let limpo = normalizar(texto);
    let veredito = if eh_palindromo(texto) { "é palíndromo" } else { "não é palíndromo" };
    println!("  {texto:<42} {veredito:<18} ({limpo})");
}

fn mostrar_anagrama(a: &str, b: &str) {
    let veredito = if sao_anagramas(a, b) { "são anagramas" } else { "não são anagramas" };
    println!("  {a:<12} x {b:<12} {veredito:<18} (letras: {} / {})", chave_anagrama(a), chave_anagrama(b));
}

fn mostrar_grupos(palavras: &[String]) {
    let grupos = agrupar(palavras);
    let com_par = grupos.iter().filter(|g| g.len() > 1).count();
    println!("  {} palavras, {} grupos, {} com pelo menos um par:", palavras.len(), grupos.len(), com_par);
    for grupo in &grupos {
        let rotulo = if grupo.len() > 1 { "anagramas" } else { "sozinha  " };
        println!("    {rotulo}  {}", grupo.join(", "));
    }
}

fn demo() {
    println!("=== Palíndromos e anagramas: demonstração ===\n");
    println!("Palíndromos (ignorando acentos, espaços e pontuação):");
    for frase in [
        "A base do teto desaba",
        "Socorram-me, subi no ônibus em Marrocos",
        "Anotaram a data da maratona",
        "Roma me tem amor",
        "Luz azul",
        "12321",
        "Isso aqui não é um palíndromo",
    ] {
        mostrar_palindromo(frase);
    }
    println!("\nPares de palavras:");
    for (a, b) in [("amor", "Roma"), ("pedra", "perda"), ("alegria", "galeria"), ("casa", "saco"), ("porta", "tropa")] {
        mostrar_anagrama(a, b);
    }
    println!("\nAgrupando uma lista:");
    let lista: Vec<String> = ["amor", "roma", "mora", "ramo", "pedra", "perda", "porta", "tropa", "gato", "toga", "casa", "sol"]
        .iter()
        .map(|p| p.to_string())
        .collect();
    mostrar_grupos(&lista);
}

fn uso() {
    println!("Uso:");
    println!("  16_palindromo_anagrama.exe palindromo <frase>");
    println!("  16_palindromo_anagrama.exe anagrama <frase 1> <frase 2>");
    println!("  16_palindromo_anagrama.exe agrupar <palavra> <palavra> ...");
    println!("  16_palindromo_anagrama.exe --demo");
}

fn executar(args: &[String]) -> Result<(), String> {
    // Casar a fatia de argumentos com padrões deixa cada comando explícito.
    match args {
        [] => uso(),
        [comando] if comando == "--demo" => demo(),
        [comando, frase] if comando == "palindromo" => {
            mostrar_palindromo(frase);
            if !eh_palindromo(frase) {
                return Err("a frase não é palíndromo".to_string());
            }
        }
        [comando, a, b] if comando == "anagrama" => {
            mostrar_anagrama(a, b);
            if !sao_anagramas(a, b) {
                return Err("as frases não são anagramas".to_string());
            }
        }
        [comando, palavras @ ..] if comando == "agrupar" && palavras.len() >= 2 => mostrar_grupos(palavras),
        [comando, ..] if comando == "agrupar" => return Err("agrupar precisa de pelo menos duas palavras".to_string()),
        [comando, ..] if comando == "palindromo" => return Err("palindromo precisa de exatamente uma frase (use aspas)".to_string()),
        [comando, ..] if comando == "anagrama" => return Err("anagrama precisa de exatamente duas frases (use aspas)".to_string()),
        [outro, ..] => return Err(format!("comando desconhecido '{outro}'; use palindromo, anagrama, agrupar ou --demo")),
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
// 1. Encontre o maior trecho palíndromo dentro de uma frase qualquer
//    (por exemplo, "ana" dentro de "banana").
// 2. Leia um arquivo com uma palavra por linha e liste só os grupos com
//    três ou mais anagramas.
// 3. Verifique se duas frases são anagramas contando letras com um
//    HashMap<char, usize>, sem ordenar, e compare o desempenho.
