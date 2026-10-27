// ============================================================
// CodeNomad Academy  |  Sources Rust  |  17. Conversor de formatos de texto
// ------------------------------------------------------------
// O que faz: Converte um texto entre snake_case, camelCase, PascalCase,
//            kebab-case e Title Case; gera slug para URL; remove acentos;
//            inverte e capitaliza frases. Entende qualquer formato na
//            entrada, inclusive "camelCaseJáEscrito".
// Como usar: 17_conversor_texto.exe snake "Relatório de vendas do mês"
//            17_conversor_texto.exe todos "configuraçãoAvançadaDoUsuário"
//            17_conversor_texto.exe --demo
// O que você aprende: percorrer chars com enumerate e olhar o vizinho,
//            to_uppercase/to_lowercase que devolvem iteradores, join em
//            Vec<String>, match que devolve closures, String::extend,
//            is_some_and em Option.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

const FORMATOS: [&str; 9] = ["snake", "camel", "pascal", "kebab", "title", "slug", "sem-acentos", "inverter", "capitalizar"];

fn sem_acento(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        'ñ' => 'n',
        'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
        'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
        'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
        'Ç' => 'C',
        'Ñ' => 'N',
        outro => outro,
    }
}

fn remover_acentos(texto: &str) -> String {
    texto.chars().map(sem_acento).collect()
}

/// Quebra o texto em palavras minúsculas, venha ele de onde vier: espaço,
/// hífen, sublinhado e pontuação separam; uma troca de caixa também
/// ("camelCase" vira "camel" e "case"; "HTTPServer" vira "http" e "server").
fn palavras(texto: &str) -> Vec<String> {
    let letras: Vec<char> = texto.chars().collect();
    let mut lista: Vec<String> = Vec::new();
    let mut atual = String::new();
    for (i, &c) in letras.iter().enumerate() {
        if !c.is_alphanumeric() {
            fechar_palavra(&mut atual, &mut lista);
            continue;
        }
        let anterior = if i > 0 { Some(letras[i - 1]) } else { None };
        let proximo = letras.get(i + 1).copied();
        // Começa palavra nova quando uma maiúscula vem depois de minúscula ou
        // dígito, ou quando ela é a última de uma sequência de maiúsculas.
        let fronteira = c.is_uppercase()
            && match anterior {
                Some(a) if a.is_lowercase() || a.is_numeric() => true,
                Some(a) if a.is_uppercase() => proximo.is_some_and(char::is_lowercase),
                _ => false,
            };
        if fronteira {
            fechar_palavra(&mut atual, &mut lista);
        }
        atual.extend(c.to_lowercase());
    }
    fechar_palavra(&mut atual, &mut lista);
    lista
}

fn fechar_palavra(atual: &mut String, lista: &mut Vec<String>) {
    if !atual.is_empty() {
        lista.push(std::mem::take(atual));
    }
}

/// Primeira letra maiúscula, resto como está. to_uppercase devolve um
/// iterador porque algumas letras viram duas (ß vira SS).
fn capitalizar_palavra(palavra: &str) -> String {
    let mut letras = palavra.chars();
    match letras.next() {
        Some(primeira) => primeira.to_uppercase().chain(letras).collect(),
        None => String::new(),
    }
}

fn snake(texto: &str) -> String {
    remover_acentos(&palavras(texto).join("_"))
}

fn kebab(texto: &str) -> String {
    remover_acentos(&palavras(texto).join("-"))
}

fn pascal(texto: &str) -> String {
    remover_acentos(&palavras(texto).iter().map(|p| capitalizar_palavra(p)).collect::<String>())
}

fn camel(texto: &str) -> String {
    let lista = palavras(texto);
    let mut saida = String::new();
    for (i, palavra) in lista.iter().enumerate() {
        saida.push_str(&if i == 0 { palavra.clone() } else { capitalizar_palavra(palavra) });
    }
    remover_acentos(&saida)
}

/// Title Case mantém os acentos: é texto para pessoas, não identificador.
fn title(texto: &str) -> String {
    palavras(texto).iter().map(|p| capitalizar_palavra(p)).collect::<Vec<String>>().join(" ")
}

/// Slug: só a-z, 0-9 e hífen, como em endereços de página.
fn slug(texto: &str) -> String {
    kebab(texto).chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect()
}

fn inverter(texto: &str) -> String {
    texto.chars().rev().collect()
}

/// Primeira letra de cada frase em maiúscula, o resto em minúscula.
fn capitalizar(texto: &str) -> String {
    let mut inicio_de_frase = true;
    texto
        .chars()
        .map(|c| {
            if c.is_alphabetic() {
                let convertida: String = if inicio_de_frase { c.to_uppercase().collect() } else { c.to_lowercase().collect() };
                inicio_de_frase = false;
                convertida
            } else {
                if matches!(c, '.' | '!' | '?') {
                    inicio_de_frase = true;
                }
                c.to_string()
            }
        })
        .collect()
}

fn converter(formato: &str, texto: &str) -> Result<String, String> {
    let funcao: fn(&str) -> String = match formato {
        "snake" => snake,
        "camel" => camel,
        "pascal" => pascal,
        "kebab" => kebab,
        "title" => title,
        "slug" => slug,
        "sem-acentos" => remover_acentos,
        "inverter" => inverter,
        "capitalizar" => capitalizar,
        outro => return Err(format!("formato desconhecido '{outro}'; use um de: {}, todos", FORMATOS.join(", "))),
    };
    Ok(funcao(texto))
}

fn mostrar_todos(texto: &str) -> Result<(), String> {
    println!("Entrada: {texto}");
    for formato in FORMATOS {
        println!("  {formato:<12} {}", converter(formato, texto)?);
    }
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Conversor de formatos de texto: demonstração ===\n");
    for frase in ["Olá, mundo da programação!", "Relatório de vendas do mês de março", "configuraçãoAvançadaDoUsuário"] {
        mostrar_todos(frase)?;
        println!();
    }
    println!("Observe: snake, camel, pascal, kebab e slug perdem os acentos (são identificadores);");
    println!("title e capitalizar mantêm, porque são texto para pessoas lerem.");
    Ok(())
}

fn uso() {
    println!("Uso: 17_conversor_texto.exe <formato> <texto>");
    println!("  formatos: {}, todos", FORMATOS.join(", "));
    println!("  17_conversor_texto.exe --demo");
}

fn executar(args: &[String]) -> Result<(), String> {
    match args {
        [] => uso(),
        [comando] if comando == "--demo" => demo()?,
        [_] => return Err("faltou o texto depois do formato".to_string()),
        [formato, texto @ ..] => {
            let texto = texto.join(" ");
            if texto.trim().is_empty() {
                return Err("o texto está vazio".to_string());
            }
            if formato == "todos" {
                mostrar_todos(&texto)?;
            } else {
                println!("{}", converter(formato, &texto)?);
            }
        }
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
// 1. Adicione o formato "constante" (SCREAMING_SNAKE_CASE) e "frase"
//    (só a primeira letra maiúscula, resto minúsculo, com espaços).
// 2. Faça o conversor ler várias linhas da entrada padrão e converter cada
//    uma, para usar em pipelines do terminal.
// 3. Detecte automaticamente o formato de entrada e informe ao usuário
//    ("a entrada parece camelCase").
