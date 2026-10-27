// ============================================================
// CodeNomad Academy  |  Sources Rust  |  08. Calculadora de notas e média ponderada
// ------------------------------------------------------------
// O que faz: recebe notas com pesos, calcula a média ponderada, diz a
//            situação (aprovação, recuperação ou reprovação) e quanto o aluno
//            precisa tirar na próxima prova para alcançar a média de corte.
// Como usar: rustc -O --edition 2021 -D warnings 08_calculadora_notas.rs -o notas.exe
//            notas.exe 8:2 6,5:3 7:5                     (nota:peso; sem peso vale 1)
//            notas.exe --aprovacao 6 --peso-prova 2 5:1 7:1
//            notas.exe --demo
// O que você aprende: struct com impl Default, enum com Display, enum com
//            dados (Precisa(f64)), iteradores com map e sum, while let para
//            ler flags dos argumentos, álgebra simples virando código.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::fmt;
use std::process::ExitCode;

struct Nota {
    valor: f64,
    peso: f64,
}

struct Aluno {
    nome: String,
    notas: Vec<Nota>,
}

/// Regras da escola. Default dá os valores mais comuns no Brasil: média 7
/// aprova direto, de 5 a 7 vai para recuperação, abaixo de 5 reprova.
struct Criterio {
    aprovacao: f64,
    recuperacao: f64,
    peso_proxima: f64,
}

impl Default for Criterio {
    fn default() -> Criterio {
        Criterio { aprovacao: 7.0, recuperacao: 5.0, peso_proxima: 1.0 }
    }
}

enum Situacao {
    Aprovacao,
    Recuperacao,
    Reprovacao,
}

impl fmt::Display for Situacao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Situacao::Aprovacao => write!(f, "APROVAÇÃO"),
            Situacao::Recuperacao => write!(f, "RECUPERAÇÃO"),
            Situacao::Reprovacao => write!(f, "REPROVAÇÃO"),
        }
    }
}

/// O que a próxima prova precisa trazer. Variantes com dado carregam a nota.
enum Proxima {
    QualquerNota,
    Precisa(f64),
    Impossivel(f64),
}

impl Aluno {
    fn novo(nome: &str, notas: Vec<Nota>) -> Result<Aluno, String> {
        if notas.is_empty() {
            return Err(format!("{nome}: informe ao menos uma nota"));
        }
        Ok(Aluno { nome: nome.to_string(), notas })
    }

    /// Devolve (soma de nota x peso, soma dos pesos): as duas contas de que
    /// a média e a próxima nota precisam.
    fn somas(&self) -> (f64, f64) {
        let ponderada: f64 = self.notas.iter().map(|n| n.valor * n.peso).sum();
        let pesos: f64 = self.notas.iter().map(|n| n.peso).sum();
        (ponderada, pesos)
    }

    fn media(&self) -> f64 {
        let (ponderada, pesos) = self.somas();
        ponderada / pesos
    }

    fn situacao(&self, criterio: &Criterio) -> Situacao {
        let media = self.media();
        if media >= criterio.aprovacao {
            Situacao::Aprovacao
        } else if media >= criterio.recuperacao {
            Situacao::Recuperacao
        } else {
            Situacao::Reprovacao
        }
    }

    /// Resolve (S + x * p) / (W + p) >= corte para x, em que S é a soma
    /// ponderada, W a soma dos pesos e p o peso da próxima prova.
    fn proxima(&self, criterio: &Criterio) -> Proxima {
        let (ponderada, pesos) = self.somas();
        let p = criterio.peso_proxima;
        let necessaria = (criterio.aprovacao * (pesos + p) - ponderada) / p;
        if necessaria <= 0.0 {
            Proxima::QualquerNota
        } else if necessaria > 10.0 {
            Proxima::Impossivel(necessaria)
        } else {
            Proxima::Precisa(necessaria)
        }
    }
}

fn ler_numero(texto: &str, nome: &str) -> Result<f64, String> {
    match texto.trim().replace(',', ".").parse::<f64>() {
        Ok(valor) if valor.is_finite() => Ok(valor),
        _ => Err(format!("{nome}: \"{texto}\" não é um número")),
    }
}

/// "7,5:2" vira nota 7,5 com peso 2; "7,5" sozinho ganha peso 1.
fn interpretar_nota(texto: &str) -> Result<Nota, String> {
    let (valor_texto, peso_texto) = texto.split_once(':').unwrap_or((texto, "1"));
    let valor = ler_numero(valor_texto, "nota")?;
    let peso = ler_numero(peso_texto, "peso")?;
    if !(0.0..=10.0).contains(&valor) {
        return Err(format!("nota fora da faixa de 0 a 10: {valor_texto}"));
    }
    if peso <= 0.0 {
        return Err(format!("peso precisa ser maior que zero: {peso_texto}"));
    }
    Ok(Nota { valor, peso })
}

fn casas(valor: f64, n: usize) -> String {
    format!("{valor:.n$}").replace('.', ",")
}

fn imprimir(aluno: &Aluno, criterio: &Criterio) {
    let lista: Vec<String> = aluno.notas.iter().map(|n| format!("{} (peso {})", casas(n.valor, 1), casas(n.peso, 0))).collect();
    println!("{}", aluno.nome);
    println!("  Notas: {}", lista.join(" | "));
    println!("  Média ponderada: {}  ->  {}", casas(aluno.media(), 2), aluno.situacao(criterio));
    let frase = match aluno.proxima(criterio) {
        Proxima::QualquerNota => "qualquer nota mantém a média de corte".to_string(),
        Proxima::Precisa(nota) => format!("precisa de ao menos {} para fechar a média em {}", casas(nota, 2), casas(criterio.aprovacao, 1)),
        Proxima::Impossivel(nota) => format!("precisaria de {}, acima de 10: não dá para fechar só com ela", casas(nota, 2)),
    };
    println!("  Próxima prova (peso {}): {frase}", casas(criterio.peso_proxima, 0));
}

/// Lê o valor que vem logo depois de uma flag, como "--aprovacao 6".
fn valor_da_flag(iter: &mut std::slice::Iter<'_, String>, flag: &str) -> Result<f64, String> {
    let texto = iter.next().ok_or_else(|| format!("faltou o valor depois de {flag}"))?;
    ler_numero(texto, flag)
}

fn executar(args: &[String]) -> Result<(), String> {
    let mut criterio = Criterio::default();
    let mut notas = Vec::new();
    // while let consome o iterador até acabar; as flags puxam o item seguinte.
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--aprovacao" => criterio.aprovacao = valor_da_flag(&mut iter, arg)?,
            "--recuperacao" => criterio.recuperacao = valor_da_flag(&mut iter, arg)?,
            "--peso-prova" => criterio.peso_proxima = valor_da_flag(&mut iter, arg)?,
            texto => notas.push(interpretar_nota(texto)?),
        }
    }
    if !(0.0..=10.0).contains(&criterio.aprovacao) || criterio.recuperacao > criterio.aprovacao {
        return Err("a média de aprovação vai de 0 a 10 e a de recuperação não pode passar dela".to_string());
    }
    if criterio.peso_proxima <= 0.0 {
        return Err("o peso da próxima prova precisa ser maior que zero".to_string());
    }
    let aluno = Aluno::novo("Aluno", notas)?;
    imprimir(&aluno, &criterio);
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Demonstração: três alunos, pesos diferentes, próxima prova com peso 3 ===");
    println!();
    let criterio = Criterio { peso_proxima: 3.0, ..Criterio::default() };
    let turma = [
        ("Ana", vec![(8.0, 2.0), (6.5, 3.0), (7.0, 5.0)]),
        ("Bruno", vec![(5.0, 2.0), (6.0, 2.0), (6.5, 1.0)]),
        ("Carla", vec![(3.0, 1.0), (4.0, 1.0), (2.0, 1.0)]),
    ];
    for (nome, pares) in turma {
        let notas = pares.into_iter().map(|(valor, peso)| Nota { valor, peso }).collect();
        imprimir(&Aluno::novo(nome, notas)?, &criterio);
        println!();
    }
    println!("Critério: média {} aprova, {} ou mais vai para recuperação, abaixo disso reprova.",
        casas(criterio.aprovacao, 1), casas(criterio.recuperacao, 1));
    Ok(())
}

fn imprimir_uso() {
    println!("Calculadora de notas e média ponderada (CodeNomad Academy)");
    println!();
    println!("Uso: notas.exe [OPÇÕES] NOTA[:PESO] [NOTA[:PESO] ...]");
    println!("  --aprovacao M      média para aprovar direto (padrão 7)");
    println!("  --recuperacao M    média mínima para recuperação (padrão 5)");
    println!("  --peso-prova P   peso da próxima prova (padrão 1)");
    println!();
    println!("Exemplos:");
    println!("  notas.exe 8:2 6,5:3 7:5");
    println!("  notas.exe --aprovacao 6 --peso-prova 2 5:1 7:1");
    println!("  notas.exe --demo      roda uma demonstração completa");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None | Some("--help") | Some("-h") => {
            imprimir_uso();
            Ok(())
        }
        Some("--demo") => demo(),
        Some(_) => executar(&args),
    };
    match resultado {
        Ok(()) => ExitCode::SUCCESS,
        Err(mensagem) => {
            eprintln!("Erro: {mensagem}");
            ExitCode::FAILURE
        }
    }
}

// ------------------------------------------------------------
// Para ir além
// 1. Leia uma turma inteira de um arquivo (nome;nota:peso;nota:peso) e
//    imprima um ranking pela média, com sort_by e total_cmp.
// 2. Acrescente a regra de frequência: abaixo de 75% de presença reprova
//    mesmo com média alta. Mostre o motivo na situação.
// 3. Em vez de só a próxima prova, considere duas provas restantes e mostre
//    a média que o aluno precisa manter nelas.
// ------------------------------------------------------------
