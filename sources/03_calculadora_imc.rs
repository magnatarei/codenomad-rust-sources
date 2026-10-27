// ============================================================
// CodeNomad Academy  |  Sources Rust  |  03. Calculadora de IMC
// ------------------------------------------------------------
// O que faz: recebe peso e altura, valida as faixas, e devolve o IMC, a
//            classificação oficial e a faixa de peso saudável para aquela
//            altura. Aceita várias pessoas de uma vez e monta uma tabela.
// Como usar: rustc -O --edition 2021 -D warnings 03_calculadora_imc.rs -o imc.exe
//            imc.exe 70 1,75                 (peso em kg, altura em metros)
//            imc.exe 70 175 95 1,80 48 160   (altura em cm também vale; pares peso altura)
//            imc.exe --demo
// O que você aprende: enum de erro com impl Display, enum de classificação
//            com método construtor, struct com impl, chunks para andar de dois
//            em dois, Result com ? e map_err, formatação alinhada de tabela.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::fmt;
use std::process::ExitCode;

/// Cada coisa que pode dar errado vira uma variante. O chamador decide o que
/// fazer com cada uma; aqui a decisão é só imprimir uma mensagem clara.
enum ErroImc {
    NumeroInvalido { campo: &'static str, texto: String },
    PesoForaDaFaixa(f64),
    AlturaForaDaFaixa(f64),
    ArgumentosImpares(usize),
}

impl fmt::Display for ErroImc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroImc::NumeroInvalido { campo, texto } => write!(f, "{campo} inválido: \"{texto}\" não é um número"),
            ErroImc::PesoForaDaFaixa(p) => write!(f, "peso fora da faixa aceita (2 a 500 kg): {p}"),
            ErroImc::AlturaForaDaFaixa(a) => write!(f, "altura fora da faixa aceita (0,50 a 2,72 m): {a}"),
            ErroImc::ArgumentosImpares(n) => write!(f, "os argumentos vêm em pares peso altura, mas recebi {n}"),
        }
    }
}

/// Faixas da classificação oficial do IMC para adultos.
enum Classificacao {
    AbaixoDoPeso,
    PesoNormal,
    Sobrepeso,
    ObesidadeGrau1,
    ObesidadeGrau2,
    ObesidadeGrau3,
}

impl Classificacao {
    /// O match com guardas (if) percorre as faixas de baixo para cima; a
    /// primeira que casa vence, por isso a ordem importa.
    fn de(imc: f64) -> Classificacao {
        match imc {
            x if x < 18.5 => Classificacao::AbaixoDoPeso,
            x if x < 25.0 => Classificacao::PesoNormal,
            x if x < 30.0 => Classificacao::Sobrepeso,
            x if x < 35.0 => Classificacao::ObesidadeGrau1,
            x if x < 40.0 => Classificacao::ObesidadeGrau2,
            _ => Classificacao::ObesidadeGrau3,
        }
    }

    fn nome(&self) -> &'static str {
        match self {
            Classificacao::AbaixoDoPeso => "Abaixo do peso",
            Classificacao::PesoNormal => "Peso normal",
            Classificacao::Sobrepeso => "Sobrepeso",
            Classificacao::ObesidadeGrau1 => "Obesidade grau I",
            Classificacao::ObesidadeGrau2 => "Obesidade grau II",
            Classificacao::ObesidadeGrau3 => "Obesidade grau III",
        }
    }
}

struct Pessoa {
    nome: String,
    peso: f64,
    altura: f64,
}

/// O resultado da avaliação, separado da pessoa: dados de entrada de um lado,
/// dados calculados do outro.
struct Avaliacao {
    imc: f64,
    classificacao: Classificacao,
    peso_minimo: f64,
    peso_maximo: f64,
}

impl Pessoa {
    /// Altura acima de 3 só pode ser centímetro (ninguém tem 175 metros), então
    /// o programa converte sozinho em vez de obrigar o usuário a escolher.
    fn nova(nome: String, peso: f64, altura_informada: f64) -> Result<Pessoa, ErroImc> {
        let altura = if altura_informada > 3.0 { altura_informada / 100.0 } else { altura_informada };
        if !(2.0..=500.0).contains(&peso) {
            return Err(ErroImc::PesoForaDaFaixa(peso));
        }
        if !(0.5..=2.72).contains(&altura) {
            return Err(ErroImc::AlturaForaDaFaixa(altura_informada));
        }
        Ok(Pessoa { nome, peso, altura })
    }

    /// IMC = peso / altura². A faixa saudável inverte a conta: peso = IMC * altura².
    fn avaliar(&self) -> Avaliacao {
        let quadrado = self.altura * self.altura;
        let imc = self.peso / quadrado;
        Avaliacao {
            imc,
            classificacao: Classificacao::de(imc),
            peso_minimo: 18.5 * quadrado,
            peso_maximo: 24.9 * quadrado,
        }
    }
}

/// Aceita vírgula ou ponto nas casas: "1,75" e "1.75" valem o mesmo.
fn ler_numero(texto: &str, campo: &'static str) -> Result<f64, ErroImc> {
    let limpo = texto.trim().replace(',', ".");
    match limpo.parse::<f64>() {
        Ok(valor) if valor.is_finite() => Ok(valor),
        _ => Err(ErroImc::NumeroInvalido { campo, texto: texto.to_string() }),
    }
}

/// Troca o ponto por vírgula para a saída ficar no padrão brasileiro.
fn casas(valor: f64, n: usize) -> String {
    format!("{valor:.n$}").replace('.', ",")
}

fn imprimir_tabela(pessoas: &[Pessoa]) {
    println!("{:<10} {:>8} {:>8} {:>7}  {:<20} {:<22}", "Nome", "Peso", "Altura", "IMC", "Classificação", "Faixa saudável");
    println!("{}", "-".repeat(80));
    for pessoa in pessoas {
        let av = pessoa.avaliar();
        println!(
            "{:<10} {:>8} {:>8} {:>7}  {:<20} {:<22}",
            pessoa.nome,
            format!("{} kg", casas(pessoa.peso, 1)),
            format!("{} m", casas(pessoa.altura, 2)),
            casas(av.imc, 1),
            av.classificacao.nome(),
            format!("{} a {} kg", casas(av.peso_minimo, 1), casas(av.peso_maximo, 1))
        );
    }
    println!();
    println!("Faixas: abaixo de 18,5 = abaixo do peso | 18,5 a 24,9 = normal | 25 a 29,9 = sobrepeso");
    println!("        30 a 34,9 = obesidade I | 35 a 39,9 = obesidade II | 40 ou mais = obesidade III");
    println!("O IMC é uma triagem simples: não substitui avaliação de um profissional de saúde.");
}

/// Lê os argumentos de dois em dois. chunks(2) entrega fatias de tamanho 2 e,
/// se sobrar um argumento sozinho, a verificação anterior já barrou.
fn executar(args: &[String]) -> Result<(), ErroImc> {
    if args.len() % 2 != 0 {
        return Err(ErroImc::ArgumentosImpares(args.len()));
    }
    let mut pessoas = Vec::new();
    for (i, par) in args.chunks(2).enumerate() {
        let peso = ler_numero(&par[0], "peso")?;
        let altura = ler_numero(&par[1], "altura")?;
        pessoas.push(Pessoa::nova(format!("Pessoa {}", i + 1), peso, altura)?);
    }
    imprimir_tabela(&pessoas);
    Ok(())
}

fn demo() -> Result<(), ErroImc> {
    println!("=== Demonstração: três pessoas com alturas e pesos diferentes ===");
    println!();
    let pessoas = vec![
        Pessoa::nova("Ana".to_string(), 62.0, 1.68)?,
        Pessoa::nova("Bruno".to_string(), 95.0, 180.0)?,
        Pessoa::nova("Carla".to_string(), 44.5, 1.60)?,
    ];
    imprimir_tabela(&pessoas);
    println!();
    let bruno = pessoas[1].avaliar();
    println!(
        "Exemplo de leitura: Bruno está com IMC {} e precisaria ficar entre {} e {} kg para o IMC normal.",
        casas(bruno.imc, 1),
        casas(bruno.peso_minimo, 1),
        casas(bruno.peso_maximo, 1)
    );
    Ok(())
}

fn imprimir_uso() {
    println!("Calculadora de IMC (CodeNomad Academy)");
    println!();
    println!("Uso: imc.exe PESO ALTURA [PESO ALTURA ...]");
    println!("  PESO    em quilos, de 2 a 500 (ex.: 70 ou 70,5)");
    println!("  ALTURA  em metros (1,75) ou em centímetros (175)");
    println!();
    println!("Exemplos:");
    println!("  imc.exe 70 1,75");
    println!("  imc.exe 70 175 95 1,80 48 160");
    println!("  imc.exe --demo       roda uma demonstração completa");
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
        Err(erro) => {
            eprintln!("Erro: {erro}");
            ExitCode::FAILURE
        }
    }
}

// ------------------------------------------------------------
// Para ir além
// 1. Aceite um nome antes de cada par (nome peso altura) e use-o na tabela.
// 2. Calcule quantos quilos a pessoa precisa ganhar ou perder para entrar na
//    faixa normal, e mostre isso em uma coluna extra.
// 3. Leia as pessoas de um arquivo de texto, uma por linha, com std::fs e
//    lines(), e trate linha mal formada sem derrubar o programa inteiro.
// ------------------------------------------------------------
