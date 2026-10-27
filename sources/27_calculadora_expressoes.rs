// ============================================================
// CodeNomad Academy  |  Sources Rust  |  27. Calculadora de expressões matemáticas
// ------------------------------------------------------------
// O que faz: lê uma expressão em texto com + - * / ^, parênteses e números
//            decimais, converte para notação posfixa (RPN) com o algoritmo
//            shunting-yard, respeitando precedência e associatividade, e
//            avalia o resultado. Erros claros para expressão mal formada e
//            divisão por zero.
// Como usar: rustc -O --edition 2021 -D warnings 27_calculadora_expressoes.rs -o calc.exe
//            calc.exe "2 + 3 * (4 - 1) ^ 2"
//            calc.exe --posfixa "10 / 4 + 1.5"
//            calc.exe --demo
// O que você aprende: enum com dados (Token), tokenizador com Peekable,
//            Vec como pilha, Result<T, String> com ? em cadeia, match com
//            guardas e f64 com verificação de divisão por zero.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::fmt;
use std::process;

/// Cada pedaço da expressão vira um Token. O enum carrega o valor do número
/// e o caractere do operador, então uma única lista descreve tudo.
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Numero(f64),
    Operador(char),
    AbreParentese,
    FechaParentese,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Numero(n) => write!(f, "{}", n),
            Token::Operador(c) => write!(f, "{}", c),
            Token::AbreParentese => write!(f, "("),
            Token::FechaParentese => write!(f, ")"),
        }
    }
}

/// Quebra o texto em tokens. Peekable deixa olhar o próximo caractere sem
/// consumir, o que é essencial para ler um número de vários dígitos.
fn tokenizar(texto: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = texto.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c.is_ascii_digit() || c == '.' {
            let mut lexema = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() || d == '.' {
                    lexema.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            let valor: f64 = lexema
                .parse()
                .map_err(|_| format!("número mal formado: '{}'", lexema))?;
            tokens.push(Token::Numero(valor));
        } else if "+-*/^".contains(c) {
            tokens.push(Token::Operador(c));
            chars.next();
        } else if c == '(' {
            tokens.push(Token::AbreParentese);
            chars.next();
        } else if c == ')' {
            tokens.push(Token::FechaParentese);
            chars.next();
        } else {
            return Err(format!("caractere inesperado: '{}'", c));
        }
    }
    if tokens.is_empty() {
        return Err("expressão vazia".to_string());
    }
    Ok(tokens)
}

/// Precedência maior = resolve antes. A potência é associativa à direita:
/// 2 ^ 3 ^ 2 é 2 ^ (3 ^ 2) = 512, e não (2 ^ 3) ^ 2 = 64.
fn precedencia(op: char) -> u8 {
    match op {
        '+' | '-' => 1,
        '*' | '/' => 2,
        _ => 3,
    }
}

fn associativa_direita(op: char) -> bool {
    op == '^'
}

/// Shunting-yard: números vão direto para a saída; operadores esperam numa
/// pilha até aparecer um de precedência menor ou um parêntese fechando.
fn para_posfixa(tokens: &[Token]) -> Result<Vec<Token>, String> {
    let mut saida = Vec::new();
    let mut pilha: Vec<Token> = Vec::new();
    for t in tokens {
        match t {
            Token::Numero(_) => saida.push(t.clone()),
            Token::Operador(op) => {
                while let Some(Token::Operador(topo)) = pilha.last() {
                    let desempilha = precedencia(*topo) > precedencia(*op)
                        || (precedencia(*topo) == precedencia(*op) && !associativa_direita(*op));
                    if !desempilha {
                        break;
                    }
                    saida.push(pilha.pop().ok_or("pilha vazia")?);
                }
                pilha.push(t.clone());
            }
            Token::AbreParentese => pilha.push(t.clone()),
            Token::FechaParentese => loop {
                match pilha.pop() {
                    Some(Token::AbreParentese) => break,
                    Some(outro) => saida.push(outro),
                    None => return Err("parêntese fechado sem abrir".to_string()),
                }
            },
        }
    }
    while let Some(t) = pilha.pop() {
        if t == Token::AbreParentese {
            return Err("parêntese aberto sem fechar".to_string());
        }
        saida.push(t);
    }
    Ok(saida)
}

/// Avalia a posfixa com uma pilha de valores: número empilha, operador tira
/// dois, calcula e empilha o resultado. No fim deve sobrar exatamente um.
fn avaliar_posfixa(posfixa: &[Token]) -> Result<f64, String> {
    let mut pilha: Vec<f64> = Vec::new();
    for t in posfixa {
        match t {
            Token::Numero(n) => pilha.push(*n),
            Token::Operador(op) => {
                let b = pilha.pop().ok_or(format!("faltou operando para '{}'", op))?;
                let a = pilha.pop().ok_or(format!("faltou operando para '{}'", op))?;
                let r = match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' if b == 0.0 => return Err("divisão por zero".to_string()),
                    '/' => a / b,
                    _ => a.powf(b),
                };
                pilha.push(r);
            }
            _ => return Err("parêntese sobrou na posfixa".to_string()),
        }
    }
    match (pilha.pop(), pilha.is_empty()) {
        (Some(v), true) => Ok(v),
        _ => Err("expressão incompleta: operandos sobrando ou faltando".to_string()),
    }
}

fn calcular(texto: &str) -> Result<(Vec<Token>, f64), String> {
    let tokens = tokenizar(texto)?;
    let posfixa = para_posfixa(&tokens)?;
    let valor = avaliar_posfixa(&posfixa)?;
    Ok((posfixa, valor))
}

fn posfixa_texto(posfixa: &[Token]) -> String {
    posfixa.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(" ")
}

fn demo() {
    println!("=== Avaliando expressões ===");
    let exemplos = ["2 + 3 * 4", "(2 + 3) * 4", "2 ^ 3 ^ 2", "10 / 4 + 1.5", "(1.5 + 2.5) * (7 - 3) / 2"];
    for e in exemplos {
        match calcular(e) {
            Ok((_, v)) => println!("    {:<28} = {}", e, v),
            Err(erro) => println!("    {:<28} erro: {}", e, erro),
        }
    }
    println!("\n=== Notação posfixa (shunting-yard) ===");
    let e = "3 + 4 * (2 - 1) ^ 2";
    if let Ok((posfixa, v)) = calcular(e) {
        println!("    infixa:  {}", e);
        println!("    posfixa: {}", posfixa_texto(&posfixa));
        println!("    valor:   {}", v);
    }
    println!("\n=== Erros tratados ===");
    for e in ["7 / (3 - 3)", "(2 + 3"] {
        match calcular(e) {
            Ok((_, v)) => println!("    {:<16} = {}", e, v),
            Err(erro) => println!("    {:<16} erro: {}", e, erro),
        }
    }
}

fn uso() {
    println!("Uso: calculadora_expressoes \"<expressão>\"");
    println!("     calculadora_expressoes --posfixa \"<expressão>\"   (mostra a RPN também)");
    println!("     calculadora_expressoes --demo");
    println!("     operadores: + - * / ^ e parênteses; decimais com ponto");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        uso();
        return;
    }
    if args.iter().any(|a| a == "--demo") {
        demo();
        return;
    }
    let mostrar_posfixa = args[0] == "--posfixa";
    let expressao = if mostrar_posfixa { args[1..].join(" ") } else { args.join(" ") };
    match calcular(&expressao) {
        Ok((posfixa, valor)) => {
            if mostrar_posfixa {
                println!("posfixa: {}", posfixa_texto(&posfixa));
            }
            println!("{} = {}", expressao.trim(), valor);
        }
        Err(e) => {
            eprintln!("Erro: {}", e);
            process::exit(1);
        }
    }
}

// Para ir além:
//   1. Aceite o sinal de menos unário ("-3 + 5" e "2 * -4"): no tokenizador,
//      um '-' no início ou logo após '(' ou operador vira o número negativo.
//   2. Adicione funções como sqrt(x), abs(x) e constantes pi e e, tratando o
//      nome como um token próprio no shunting-yard.
//   3. Reescreva o avaliador como descida recursiva (expr -> termo -> fator)
//      e compare a legibilidade com a versão por pilha.
