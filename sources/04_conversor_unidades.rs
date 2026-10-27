// ============================================================
// CodeNomad Academy  |  Sources Rust  |  04. Conversor de unidades
// ------------------------------------------------------------
// O que faz: converte comprimento, massa, temperatura, volume e velocidade
//            entre quaisquer duas unidades da mesma categoria, a partir de
//            uma tabela de fatores. Lista as unidades disponíveis.
// Como usar: rustc -O --edition 2021 -D warnings 04_conversor_unidades.rs -o unidades.exe
//            unidades.exe 10 km mi           (valor, unidade de origem, unidade de destino)
//            unidades.exe 100 C F
//            unidades.exe --listar
//            unidades.exe --demo
// O que você aprende: const com fatia de structs, enum com PartialEq e Copy,
//            iteradores (find, filter), Option virando Result com ok_or_else,
//            match em texto, e por que temperatura não cabe em um fator.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::fmt;
use std::process::ExitCode;

/// PartialEq permite comparar categorias com ==; Copy permite passar por valor.
#[derive(Clone, Copy, PartialEq)]
enum Categoria {
    Comprimento,
    Massa,
    Temperatura,
    Volume,
    Velocidade,
}

impl fmt::Display for Categoria {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nome = match self {
            Categoria::Comprimento => "comprimento",
            Categoria::Massa => "massa",
            Categoria::Temperatura => "temperatura",
            Categoria::Volume => "volume",
            Categoria::Velocidade => "velocidade",
        };
        write!(f, "{nome}")
    }
}

const CATEGORIAS: [Categoria; 5] = [
    Categoria::Comprimento,
    Categoria::Massa,
    Categoria::Temperatura,
    Categoria::Volume,
    Categoria::Velocidade,
];

/// Cada unidade guarda quanto vale 1 dela na unidade base da categoria.
/// Bases: metro, quilograma, grau Celsius, litro e metro por segundo.
struct Unidade {
    simbolo: &'static str,
    nome: &'static str,
    categoria: Categoria,
    fator: f64,
}

/// A tabela inteira vive em uma const: nada é alocado em tempo de execução.
/// Temperatura tem fator 1,0 só para preencher o campo; a conversão dela é
/// feita à parte, porque envolve soma (0 °C não é 0 °F).
const UNIDADES: &[Unidade] = &[
    Unidade { simbolo: "mm", nome: "milímetro", categoria: Categoria::Comprimento, fator: 0.001 },
    Unidade { simbolo: "cm", nome: "centímetro", categoria: Categoria::Comprimento, fator: 0.01 },
    Unidade { simbolo: "m", nome: "metro", categoria: Categoria::Comprimento, fator: 1.0 },
    Unidade { simbolo: "km", nome: "quilômetro", categoria: Categoria::Comprimento, fator: 1000.0 },
    Unidade { simbolo: "in", nome: "polegada", categoria: Categoria::Comprimento, fator: 0.0254 },
    Unidade { simbolo: "ft", nome: "pé", categoria: Categoria::Comprimento, fator: 0.3048 },
    Unidade { simbolo: "yd", nome: "jarda", categoria: Categoria::Comprimento, fator: 0.9144 },
    Unidade { simbolo: "mi", nome: "milha", categoria: Categoria::Comprimento, fator: 1609.344 },
    Unidade { simbolo: "mg", nome: "miligrama", categoria: Categoria::Massa, fator: 0.000001 },
    Unidade { simbolo: "g", nome: "grama", categoria: Categoria::Massa, fator: 0.001 },
    Unidade { simbolo: "kg", nome: "quilograma", categoria: Categoria::Massa, fator: 1.0 },
    Unidade { simbolo: "t", nome: "tonelada", categoria: Categoria::Massa, fator: 1000.0 },
    Unidade { simbolo: "oz", nome: "onça", categoria: Categoria::Massa, fator: 0.028349523125 },
    Unidade { simbolo: "lb", nome: "libra", categoria: Categoria::Massa, fator: 0.45359237 },
    Unidade { simbolo: "C", nome: "grau Celsius", categoria: Categoria::Temperatura, fator: 1.0 },
    Unidade { simbolo: "F", nome: "grau Fahrenheit", categoria: Categoria::Temperatura, fator: 1.0 },
    Unidade { simbolo: "K", nome: "kelvin", categoria: Categoria::Temperatura, fator: 1.0 },
    Unidade { simbolo: "ml", nome: "mililitro", categoria: Categoria::Volume, fator: 0.001 },
    Unidade { simbolo: "l", nome: "litro", categoria: Categoria::Volume, fator: 1.0 },
    Unidade { simbolo: "m3", nome: "metro cúbico", categoria: Categoria::Volume, fator: 1000.0 },
    Unidade { simbolo: "gal", nome: "galão americano", categoria: Categoria::Volume, fator: 3.785411784 },
    Unidade { simbolo: "floz", nome: "onça líquida", categoria: Categoria::Volume, fator: 0.0295735295625 },
    Unidade { simbolo: "m/s", nome: "metro por segundo", categoria: Categoria::Velocidade, fator: 1.0 },
    Unidade { simbolo: "km/h", nome: "quilômetro por hora", categoria: Categoria::Velocidade, fator: 1.0 / 3.6 },
    Unidade { simbolo: "mph", nome: "milha por hora", categoria: Categoria::Velocidade, fator: 0.44704 },
    Unidade { simbolo: "kn", nome: "nó", categoria: Categoria::Velocidade, fator: 0.514444 },
];

/// find devolve Option; ok_or_else transforma o None em um erro com mensagem.
fn buscar(simbolo: &str) -> Result<&'static Unidade, String> {
    UNIDADES
        .iter()
        .find(|u| u.simbolo.eq_ignore_ascii_case(simbolo.trim()))
        .ok_or_else(|| format!("unidade desconhecida: \"{simbolo}\". Use --listar para ver as disponíveis"))
}

fn para_celsius(valor: f64, simbolo: &str) -> f64 {
    match simbolo {
        "F" => (valor - 32.0) * 5.0 / 9.0,
        "K" => valor - 273.15,
        _ => valor,
    }
}

fn de_celsius(celsius: f64, simbolo: &str) -> f64 {
    match simbolo {
        "F" => celsius * 9.0 / 5.0 + 32.0,
        "K" => celsius + 273.15,
        _ => celsius,
    }
}

fn converter(valor: f64, de: &Unidade, para: &Unidade) -> Result<f64, String> {
    if de.categoria != para.categoria {
        return Err(format!("{} é {} e {} é {}: não há conversão entre categorias", de.nome, de.categoria, para.nome, para.categoria));
    }
    if de.categoria == Categoria::Temperatura {
        return Ok(de_celsius(para_celsius(valor, de.simbolo), para.simbolo));
    }
    if valor < 0.0 {
        return Err(format!("valor negativo só faz sentido em temperatura, recebi {valor}"));
    }
    // Primeiro leva o valor para a unidade base, depois da base para o destino.
    Ok(valor * de.fator / para.fator)
}

/// Escolhe as casas conforme o tamanho do número e tira os zeros sobrando:
/// 1609.344 vira "1609,34" e 0.000621 vira "0,000621".
fn bonito(valor: f64) -> String {
    let casas = if valor.abs() >= 1000.0 { 2 } else if valor.abs() >= 1.0 { 4 } else { 6 };
    let texto = format!("{valor:.casas$}");
    texto.trim_end_matches('0').trim_end_matches('.').replace('.', ",")
}

fn ler_numero(texto: &str) -> Result<f64, String> {
    match texto.trim().replace(',', ".").parse::<f64>() {
        Ok(valor) if valor.is_finite() => Ok(valor),
        _ => Err(format!("valor inválido: \"{texto}\" não é um número")),
    }
}

fn mostrar(valor: f64, de: &str, para: &str) -> Result<(), String> {
    let origem = buscar(de)?;
    let destino = buscar(para)?;
    let resultado = converter(valor, origem, destino)?;
    println!("{:>12} {:<5} = {:>14} {:<5} ({})", bonito(valor), origem.simbolo, bonito(resultado), destino.simbolo, origem.categoria);
    Ok(())
}

fn listar() {
    println!("Unidades disponíveis (o símbolo não diferencia maiúsculas de minúsculas):");
    for categoria in CATEGORIAS {
        println!();
        println!("{categoria}");
        for u in UNIDADES.iter().filter(|u| u.categoria == categoria) {
            println!("  {:<6} {}", u.simbolo, u.nome);
        }
    }
}

fn executar(args: &[String]) -> Result<(), String> {
    if args[0] == "--listar" {
        listar();
        return Ok(());
    }
    if args.len() != 3 {
        return Err(format!("esperava VALOR DE PARA (3 argumentos), recebi {}", args.len()));
    }
    let valor = ler_numero(&args[0])?;
    mostrar(valor, &args[1], &args[2])
}

fn demo() -> Result<(), String> {
    println!("=== Demonstração: doze conversões entre as cinco categorias ===");
    println!();
    let exemplos = [
        (10.0, "km", "mi"), (26.2, "mi", "km"), (180.0, "cm", "ft"), (6.0, "ft", "m"),
        (70.0, "kg", "lb"), (16.0, "oz", "g"), (100.0, "C", "F"), (451.0, "F", "C"),
        (300.0, "K", "C"), (2.0, "l", "gal"), (100.0, "km/h", "m/s"), (60.0, "mph", "km/h"),
    ];
    for (valor, de, para) in exemplos {
        mostrar(valor, de, para)?;
    }
    println!();
    listar();
    Ok(())
}

fn imprimir_uso() {
    println!("Conversor de unidades (CodeNomad Academy)");
    println!();
    println!("Uso: unidades.exe VALOR DE PARA");
    println!("     unidades.exe --listar");
    println!();
    println!("Exemplos:");
    println!("  unidades.exe 10 km mi");
    println!("  unidades.exe 100 C F");
    println!("  unidades.exe 2,5 kg lb");
    println!("  unidades.exe --demo      roda uma demonstração completa");
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
// 1. Acrescente a categoria de tempo (s, min, h, dia, semana) e a de área
//    (m2, km2, hectare, alqueire): só a tabela precisa crescer.
// 2. Aceite entradas como "10km" sem espaço: separe os dígitos do símbolo
//    com split_at no primeiro caractere que não é número.
// 3. Permita uma lista de destinos ("10 km mi ft yd") e imprima uma linha
//    para cada um, reaproveitando a função mostrar.
// ------------------------------------------------------------
