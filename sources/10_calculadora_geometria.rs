// ============================================================
// CodeNomad Academy  |  Sources Rust  |  10. Calculadora de geometria
// ------------------------------------------------------------
// O que faz: calcula área e perímetro de quadrado, retângulo, triângulo (com
//            validação de existência), círculo e trapézio, e volume e área da
//            superfície de cubo, esfera e cilindro. Lista as formas que aceita.
// Como usar: rustc -O --edition 2021 -D warnings 10_calculadora_geometria.rs -o geometria.exe
//            geometria.exe triangulo 3 4 5          (três lados)
//            geometria.exe retangulo 3 5,5          (base e altura)
//            geometria.exe cilindro 2 10            (raio e altura)
//            geometria.exe --listar
//            geometria.exe --demo
// O que você aprende: enum com dados em cada variante, match exaustivo, a
//            constante std::f64::consts::PI, sqrt e powi, Option para
//            resultados que nem toda forma tem, fatias (&[f64]), collect em
//            Result e tabela alinhada com formatação de largura fixa.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::f64::consts::PI;
use std::process::ExitCode;

/// Cada variante carrega exatamente as medidas de que precisa. Assim é
/// impossível criar um círculo sem raio ou um retângulo com três lados.
#[derive(Clone, Copy)]
enum Forma {
    Quadrado { lado: f64 },
    Retangulo { base: f64, altura: f64 },
    Triangulo { a: f64, b: f64, c: f64 },
    Circulo { raio: f64 },
    /// Trapézio isósceles: base maior, base menor e altura. Os lados
    /// inclinados saem por Pitágoras.
    Trapezio { base_maior: f64, base_menor: f64, altura: f64 },
    Cubo { aresta: f64 },
    Esfera { raio: f64 },
    Cilindro { raio: f64, altura: f64 },
}

/// O que a calculadora devolve. Perímetro só existe para formas planas e
/// volume só para sólidos, por isso os dois são Option.
struct Medidas {
    area: f64,
    perimetro: Option<f64>,
    volume: Option<f64>,
}

/// Nome, quantidade de medidas e o que cada uma significa, para o uso e para --listar.
const CATALOGO: [(&str, usize, &str); 8] = [
    ("quadrado", 1, "lado"),
    ("retangulo", 2, "base altura"),
    ("triangulo", 3, "lado_a lado_b lado_c"),
    ("circulo", 1, "raio"),
    ("trapezio", 3, "base_maior base_menor altura"),
    ("cubo", 1, "aresta"),
    ("esfera", 1, "raio"),
    ("cilindro", 2, "raio altura"),
];

impl Forma {
    /// Monta a forma a partir do nome e das medidas já convertidas para número.
    /// Toda validação geométrica mora aqui, antes de qualquer cálculo.
    fn nova(nome: &str, m: &[f64]) -> Result<Forma, String> {
        let (_, esperado, legenda) = CATALOGO
            .iter()
            .find(|(n, _, _)| *n == nome)
            .ok_or_else(|| format!("forma desconhecida: \"{nome}\". Use --listar para ver as formas"))?;
        if m.len() != *esperado {
            return Err(format!("{nome} precisa de {esperado} medida(s): {legenda}. Recebi {}", m.len()));
        }
        if let Some(ruim) = m.iter().find(|v| **v <= 0.0) {
            return Err(format!("toda medida precisa ser maior que zero, recebi {ruim}"));
        }
        let forma = match nome {
            "quadrado" => Forma::Quadrado { lado: m[0] },
            "retangulo" => Forma::Retangulo { base: m[0], altura: m[1] },
            "triangulo" => {
                // Desigualdade triangular: cada lado precisa ser menor que a soma
                // dos outros dois, senão as pontas não se encontram.
                let (a, b, c) = (m[0], m[1], m[2]);
                if a >= b + c || b >= a + c || c >= a + b {
                    return Err(format!("não existe triângulo com lados {a}, {b} e {c}: um lado é maior ou igual à soma dos outros"));
                }
                Forma::Triangulo { a, b, c }
            }
            "circulo" => Forma::Circulo { raio: m[0] },
            "trapezio" => {
                if m[1] >= m[0] {
                    return Err(format!("a base menor ({}) precisa ser menor que a base maior ({})", m[1], m[0]));
                }
                Forma::Trapezio { base_maior: m[0], base_menor: m[1], altura: m[2] }
            }
            "cubo" => Forma::Cubo { aresta: m[0] },
            "esfera" => Forma::Esfera { raio: m[0] },
            "cilindro" => Forma::Cilindro { raio: m[0], altura: m[1] },
            _ => return Err(format!("forma sem construtor: {nome}")),
        };
        Ok(forma)
    }

    fn nome(&self) -> &'static str {
        match self {
            Forma::Quadrado { .. } => "quadrado",
            Forma::Retangulo { .. } => "retângulo",
            Forma::Triangulo { .. } => "triângulo",
            Forma::Circulo { .. } => "círculo",
            Forma::Trapezio { .. } => "trapézio",
            Forma::Cubo { .. } => "cubo",
            Forma::Esfera { .. } => "esfera",
            Forma::Cilindro { .. } => "cilindro",
        }
    }

    /// O match é exaustivo: se alguém criar uma variante nova e esquecer dela
    /// aqui, o compilador recusa o programa. Esse é o ganho do enum.
    fn medidas(&self) -> Medidas {
        match *self {
            Forma::Quadrado { lado } => plana(lado * lado, 4.0 * lado),
            Forma::Retangulo { base, altura } => plana(base * altura, 2.0 * (base + altura)),
            Forma::Triangulo { a, b, c } => {
                // Fórmula de Heron: só precisa dos lados, sem altura.
                let s = (a + b + c) / 2.0;
                plana((s * (s - a) * (s - b) * (s - c)).sqrt(), a + b + c)
            }
            Forma::Circulo { raio } => plana(PI * raio.powi(2), 2.0 * PI * raio),
            Forma::Trapezio { base_maior, base_menor, altura } => {
                let lado = (altura.powi(2) + ((base_maior - base_menor) / 2.0).powi(2)).sqrt();
                plana((base_maior + base_menor) * altura / 2.0, base_maior + base_menor + 2.0 * lado)
            }
            Forma::Cubo { aresta } => solida(6.0 * aresta.powi(2), aresta.powi(3)),
            Forma::Esfera { raio } => solida(4.0 * PI * raio.powi(2), 4.0 / 3.0 * PI * raio.powi(3)),
            Forma::Cilindro { raio, altura } => solida(2.0 * PI * raio * (raio + altura), PI * raio.powi(2) * altura),
        }
    }
}

fn plana(area: f64, perimetro: f64) -> Medidas {
    Medidas { area, perimetro: Some(perimetro), volume: None }
}

fn solida(area_superficie: f64, volume: f64) -> Medidas {
    Medidas { area: area_superficie, perimetro: None, volume: Some(volume) }
}

/// Option vira texto: Some(x) imprime o número, None imprime um traço curto.
fn opcional(valor: Option<f64>) -> String {
    valor.map_or("-".to_string(), |v| format!("{v:.3}"))
}

fn imprimir_linha(forma: &Forma, entrada: &[f64]) {
    let m = forma.medidas();
    let medidas: Vec<String> = entrada.iter().map(|v| v.to_string()).collect();
    println!("{:<10} {:<22} {:>12.3} {:>12} {:>12}", forma.nome(), medidas.join(", "), m.area, opcional(m.perimetro), opcional(m.volume));
}

fn ler_numero(texto: &str) -> Result<f64, String> {
    match texto.trim().replace(',', ".").parse::<f64>() {
        Ok(v) if v.is_finite() => Ok(v),
        _ => Err(format!("medida inválida: \"{texto}\" não é um número")),
    }
}

fn executar(args: &[String]) -> Result<(), String> {
    let nome = args[0].to_lowercase();
    // collect em Result<Vec, _> para na primeira medida inválida e devolve o erro.
    let medidas: Vec<f64> = args[1..].iter().map(|t| ler_numero(t)).collect::<Result<_, _>>()?;
    let forma = Forma::nova(&nome, &medidas)?;
    let m = forma.medidas();
    println!("{} com medidas {:?}", forma.nome(), medidas);
    if let Some(v) = m.volume {
        println!("  Área da superfície: {:.4}", m.area);
        println!("  Volume ...........: {v:.4}");
    } else {
        println!("  Área .....: {:.4}", m.area);
        println!("  Perímetro : {}", opcional(m.perimetro));
    }
    Ok(())
}

fn listar() {
    println!("Formas disponíveis e medidas esperadas:");
    for (nome, n, legenda) in CATALOGO {
        println!("  {nome:<10} {n} medida(s): {legenda}");
    }
}

fn demo() -> Result<(), String> {
    println!("=== Demonstração: todas as formas com medidas de exemplo ===");
    println!();
    let exemplos: [(&str, &[f64]); 8] = [
        ("quadrado", &[4.0]),
        ("retangulo", &[3.0, 5.5]),
        ("triangulo", &[3.0, 4.0, 5.0]),
        ("circulo", &[2.0]),
        ("trapezio", &[8.0, 4.0, 3.0]),
        ("cubo", &[3.0]),
        ("esfera", &[2.0]),
        ("cilindro", &[2.0, 10.0]),
    ];
    println!("{:<10} {:<22} {:>12} {:>12} {:>12}", "Forma", "Medidas", "Área", "Perímetro", "Volume");
    println!("{}", "-".repeat(72));
    for (nome, medidas) in exemplos {
        imprimir_linha(&Forma::nova(nome, medidas)?, medidas);
    }
    println!();
    println!("Conferindo a validação do triângulo com lados 1, 2 e 3:");
    match Forma::nova("triangulo", &[1.0, 2.0, 3.0]) {
        Ok(_) => println!("  passou, o que seria um erro"),
        Err(e) => println!("  {e}"),
    }
    Ok(())
}

fn imprimir_uso() {
    println!("Calculadora de geometria (CodeNomad Academy)");
    println!();
    println!("Uso: geometria.exe FORMA MEDIDAS...");
    println!("  geometria.exe triangulo 3 4 5");
    println!("  geometria.exe cilindro 2 10");
    println!("  geometria.exe --listar     mostra as formas e as medidas de cada uma");
    println!("  geometria.exe --demo       roda uma demonstração completa");
    println!();
    listar();
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None | Some("--help") | Some("-h") => {
            imprimir_uso();
            Ok(())
        }
        Some("--listar") => {
            listar();
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
// 1. Acrescente o losango (diagonais) e o hexágono regular (lado) ao enum e
//    veja o compilador apontar cada match que precisa de uma variante nova.
// 2. Aceite a flag --unidade cm para imprimir "cm²" e "cm³" nos resultados,
//    guardando a unidade em uma struct de configuração.
// 3. Leia várias formas de um arquivo de texto, uma por linha, e some todas
//    as áreas: útil para calcular quanto piso comprar para uma casa.
// ------------------------------------------------------------
