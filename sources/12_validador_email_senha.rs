// ============================================================
// CodeNomad Academy  |  Sources Rust  |  12. Validador de e-mail e força de senha
// ------------------------------------------------------------
// O que faz: Valida um e-mail com regras claras (um @, domínio com ponto,
//            sem espaços, tamanhos) e dá uma nota de 0 a 100 para a força
//            de uma senha (comprimento, classes de símbolos, sequências,
//            repetições, senhas comuns), com dicas de melhoria.
// Como usar: 12_validador_email_senha.exe email ana.silva@exemplo.com.br
//            12_validador_email_senha.exe senha "Tr0c@-de-C0rre1a!"
//            12_validador_email_senha.exe --demo
// O que você aprende: Result<(), String> com o operador ?, split e
//            desestruturação de tuplas, windows() em fatias, match com
//            faixas numéricas, f64::log2, structs com métodos.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

// Senhas que aparecem em toda lista de vazamentos: rejeitadas na hora.
const SENHAS_COMUNS: [&str; 12] = [
    "123456", "12345678", "123456789", "senha", "senha123", "password",
    "qwerty", "abc123", "admin", "111111", "iloveyou", "letmein",
];
// Fileiras do teclado: "qwe" ou "asd" são sequências tão fracas quanto "abc".
const FILEIRAS_TECLADO: [&str; 4] = ["qwertyuiop", "asdfghjkl", "zxcvbnm", "1234567890"];

// ---------- E-mail ----------

fn validar_email(email: &str) -> Result<(), String> {
    let email = email.trim();
    if email.is_empty() {
        return Err("endereço vazio".to_string());
    }
    if email.chars().any(char::is_whitespace) {
        return Err("contém espaço em branco".to_string());
    }
    if email.len() > 254 {
        return Err(format!("muito longo: {} bytes, o máximo é 254", email.len()));
    }
    let mut partes = email.split('@');
    // Um e-mail válido tem exatamente duas partes separadas por um único @.
    let (local, dominio) = match (partes.next(), partes.next(), partes.next()) {
        (Some(local), Some(dominio), None) => (local, dominio),
        (_, None, _) => return Err("falta o @".to_string()),
        _ => return Err("tem mais de um @".to_string()),
    };
    validar_parte_local(local)?;
    validar_dominio(dominio)?;
    Ok(())
}

fn validar_parte_local(local: &str) -> Result<(), String> {
    if local.is_empty() {
        return Err("não há nada antes do @".to_string());
    }
    if local.len() > 64 {
        return Err("a parte antes do @ passa de 64 bytes".to_string());
    }
    if let Some(c) = local.chars().find(|c| !c.is_ascii_alphanumeric() && !"._%+-".contains(*c)) {
        return Err(format!("o símbolo '{c}' não é permitido antes do @"));
    }
    if local.starts_with('.') || local.ends_with('.') || local.contains("..") {
        return Err("ponto no começo, no fim ou dois pontos seguidos antes do @".to_string());
    }
    Ok(())
}

fn validar_dominio(dominio: &str) -> Result<(), String> {
    if !dominio.contains('.') {
        return Err(format!("o domínio '{dominio}' precisa de pelo menos um ponto (ex.: exemplo.com)"));
    }
    let rotulos: Vec<&str> = dominio.split('.').collect();
    for rotulo in &rotulos {
        if rotulo.is_empty() || rotulo.len() > 63 {
            return Err("cada trecho do domínio precisa ter de 1 a 63 símbolos".to_string());
        }
        if rotulo.starts_with('-') || rotulo.ends_with('-') {
            return Err(format!("o trecho '{rotulo}' não pode começar nem terminar com hífen"));
        }
        if let Some(c) = rotulo.chars().find(|c| !c.is_ascii_alphanumeric() && *c != '-') {
            return Err(format!("o símbolo '{c}' não é permitido no domínio"));
        }
    }
    // O último trecho é o domínio de topo (com, br, org): só letras, duas ou mais.
    let topo = rotulos[rotulos.len() - 1];
    if topo.len() < 2 || !topo.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(format!("o final '{topo}' não parece um domínio de topo (ex.: com, br, org)"));
    }
    Ok(())
}

// ---------- Senha ----------

struct Avaliacao {
    pontos: u32,
    entropia_bits: f64,
    dicas: Vec<String>,
}

impl Avaliacao {
    fn rotulo(&self) -> &'static str {
        match self.pontos {
            0..=24 => "muito fraca",
            25..=44 => "fraca",
            45..=64 => "razoável",
            65..=84 => "forte",
            _ => "muito forte",
        }
    }
}

/// Três símbolos seguidos em ordem crescente ou decrescente (abc, 321)
/// ou vizinhos numa fileira do teclado (qwe, lkj).
fn tem_sequencia(senha: &str) -> bool {
    let letras: Vec<char> = senha.to_lowercase().chars().collect();
    letras.windows(3).any(|janela| {
        let (a, b, c) = (janela[0] as i32, janela[1] as i32, janela[2] as i32);
        let em_ordem = (b - a == 1 && c - b == 1) || (a - b == 1 && b - c == 1);
        let trecho: String = janela.iter().collect();
        let invertido: String = janela.iter().rev().collect();
        em_ordem || FILEIRAS_TECLADO.iter().any(|f| f.contains(&trecho) || f.contains(&invertido))
    })
}

fn tem_repeticao(senha: &str) -> bool {
    let letras: Vec<char> = senha.chars().collect();
    letras.windows(3).any(|j| j[0] == j[1] && j[1] == j[2])
}

fn avaliar_senha(senha: &str) -> Avaliacao {
    let tamanho = senha.chars().count();
    let minusculas = senha.chars().any(|c| c.is_lowercase());
    let maiusculas = senha.chars().any(|c| c.is_uppercase());
    let digitos = senha.chars().any(|c| c.is_ascii_digit());
    let simbolos = senha.chars().any(|c| !c.is_alphanumeric());
    let classes = [minusculas, maiusculas, digitos, simbolos];
    let n_classes = classes.iter().filter(|c| **c).count() as i32;

    // Entropia: quantos bits um atacante precisa chutar, supondo que ele
    // conhece as classes usadas. Alfabeto possível elevado ao comprimento.
    let alfabeto = [26, 26, 10, 33].iter().zip(classes).filter(|(_, usada)| *usada).map(|(n, _)| n).sum::<u32>();
    let entropia_bits = if alfabeto == 0 { 0.0 } else { tamanho as f64 * f64::from(alfabeto).log2() };

    let mut dicas = Vec::new();
    let mut pontos = (tamanho as i32 * 4).min(40);
    if tamanho >= 12 {
        pontos += 6;
    }
    if tamanho >= 16 {
        pontos += 6;
    }
    if tamanho < 12 {
        dicas.push("use 12 ou mais símbolos; o comprimento é o que mais pesa".to_string());
    }
    pontos += n_classes * 12;
    let faltando: Vec<&str> = ["minúsculas", "maiúsculas", "dígitos", "símbolos"]
        .into_iter()
        .zip(classes)
        .filter(|(_, usada)| !usada)
        .map(|(nome, _)| nome)
        .collect();
    if !faltando.is_empty() {
        dicas.push(format!("misture também: {}", faltando.join(", ")));
    }
    if tem_sequencia(senha) {
        pontos -= 15;
        dicas.push("evite sequências como abc, 123 ou qwe".to_string());
    }
    if tem_repeticao(senha) {
        pontos -= 10;
        dicas.push("evite o mesmo símbolo três vezes seguidas".to_string());
    }
    if SENHAS_COMUNS.contains(&senha.to_lowercase().as_str()) {
        pontos = pontos.min(5);
        dicas.push("esta senha está nas listas de senhas mais vazadas: troque agora".to_string());
    }
    // clamp garante que a nota fique entre 0 e 100 mesmo com as penalidades.
    Avaliacao { pontos: pontos.clamp(0, 100) as u32, entropia_bits, dicas }
}

fn mostrar_email(email: &str) -> bool {
    match validar_email(email) {
        Ok(()) => {
            println!("  {email:<32} VÁLIDO");
            true
        }
        Err(motivo) => {
            println!("  {email:<32} INVÁLIDO  {motivo}");
            false
        }
    }
}

fn mostrar_senha(senha: &str) {
    let a = avaliar_senha(senha);
    println!("  {:<26} {:>3}/100  {:<12} {:>5.1} bits", mascarar(senha), a.pontos, a.rotulo(), a.entropia_bits);
    for dica in &a.dicas {
        println!("  {:<26} dica: {dica}", "");
    }
}

/// Mostra só o começo da senha para não deixá-la inteira no histórico do terminal.
fn mascarar(senha: &str) -> String {
    let visiveis: String = senha.chars().take(3).collect();
    format!("{visiveis}{}", "*".repeat(senha.chars().count().saturating_sub(3)))
}

fn demo() {
    println!("=== Validador de e-mail e força de senha: demonstração ===\n");
    println!("E-mails:");
    let emails = ["ana.silva@exemplo.com.br", "suporte+loja@mail.exemplo.org", "joao@exemplo", "maria silva@exemplo.com", ".carla@exemplo.com"];
    let validos = emails.iter().filter(|e| mostrar_email(e)).count();
    println!("  {validos} válidos de {}.\n", emails.len());

    println!("Senhas (nota, rótulo e entropia estimada):");
    for senha in ["123456", "senha123", "Abc12345", "Tr0c@-de-C0rre1a!", "ventania-azul-37-planeta"] {
        mostrar_senha(senha);
    }
}

fn uso() {
    println!("Uso:");
    println!("  12_validador_email_senha.exe email <endereço> [outro ...]   valida e-mails");
    println!("  12_validador_email_senha.exe senha <senha> [outra ...]      avalia a força");
    println!("  12_validador_email_senha.exe --demo                         demonstração");
}

fn executar(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        None => uso(),
        Some("--demo") => demo(),
        Some("email") if args.len() > 1 => {
            if !args[1..].iter().fold(true, |acc, e| mostrar_email(e) && acc) {
                return Err("pelo menos um e-mail é inválido".to_string());
            }
        }
        Some("senha") if args.len() > 1 => args[1..].iter().for_each(|s| mostrar_senha(s)),
        Some("email" | "senha") => return Err("faltou o valor depois do comando".to_string()),
        Some(outro) => return Err(format!("comando desconhecido '{outro}': use email, senha ou --demo")),
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
// 1. Leia as senhas comuns de um arquivo (uma por linha) em vez da constante,
//    e teste com uma lista maior.
// 2. Estime o tempo para quebrar a senha por força bruta a partir da entropia
//    (2 elevado aos bits, dividido por 10 bilhões de tentativas por segundo).
// 3. Crie um enum `Problema` para os erros de e-mail, em vez de String, e
//    implemente Display para ele.
