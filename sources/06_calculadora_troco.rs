// ============================================================
// CodeNomad Academy  |  Sources Rust  |  06. Calculadora de troco
// ------------------------------------------------------------
// O que faz: recebe o valor da compra e o valor pago e devolve o troco
//            decomposto no menor número de cédulas e moedas (algoritmo
//            guloso). Avisa quando o pagamento é insuficiente e quanto falta.
// Como usar: rustc -O --edition 2021 -D warnings 06_calculadora_troco.rs -o troco.exe
//            troco.exe 37,35 50               (valor da compra, valor pago)
//            troco.exe 37,35 50 199,99 200    (vários pares de uma vez)
//            troco.exe --demo
// O que você aprende: dinheiro em centavos inteiros (i64) em vez de f64,
//            const com fatia de structs, enum para cédula ou moeda, divisão e
//            resto inteiros, Result propagado com ? ou tratado no local.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::process::ExitCode;

#[derive(Clone, Copy)]
enum Especie {
    Cedula,
    Moeda,
}

struct Dinheiro {
    centavos: i64,
    especie: Especie,
}

/// Cédulas e moedas em circulação, da maior para a menor. O algoritmo guloso
/// depende dessa ordem: pega o máximo da maior e passa para a seguinte.
/// Para este conjunto de valores o guloso sempre acha o menor número de peças.
const DINHEIRO: &[Dinheiro] = &[
    Dinheiro { centavos: 20000, especie: Especie::Cedula },
    Dinheiro { centavos: 10000, especie: Especie::Cedula },
    Dinheiro { centavos: 5000, especie: Especie::Cedula },
    Dinheiro { centavos: 2000, especie: Especie::Cedula },
    Dinheiro { centavos: 1000, especie: Especie::Cedula },
    Dinheiro { centavos: 500, especie: Especie::Cedula },
    Dinheiro { centavos: 200, especie: Especie::Cedula },
    Dinheiro { centavos: 100, especie: Especie::Moeda },
    Dinheiro { centavos: 50, especie: Especie::Moeda },
    Dinheiro { centavos: 25, especie: Especie::Moeda },
    Dinheiro { centavos: 10, especie: Especie::Moeda },
    Dinheiro { centavos: 5, especie: Especie::Moeda },
    Dinheiro { centavos: 1, especie: Especie::Moeda },
];

/// Uma linha do troco: qual peça e quantas vezes.
struct Item {
    dinheiro: &'static Dinheiro,
    quantidade: i64,
}

/// Lê "37,35" e devolve 3735. Guardar dinheiro em centavos inteiros evita o
/// erro clássico do ponto flutuante, em que 0,1 + 0,2 não dá exatamente 0,3.
fn ler_centavos(texto: &str, nome: &str) -> Result<i64, String> {
    let valor = match texto.trim().replace(',', ".").parse::<f64>() {
        Ok(v) if v.is_finite() => v,
        _ => return Err(format!("{nome}: \"{texto}\" não é um valor em reais")),
    };
    if valor < 0.0 {
        return Err(format!("{nome} não pode ser negativo, recebi {texto}"));
    }
    if valor > 1_000_000_000.0 {
        return Err(format!("{nome} grande demais: o limite é um bilhão de reais"));
    }
    // round() arredonda para o centavo mais próximo antes de virar inteiro.
    Ok((valor * 100.0).round() as i64)
}

/// Decompõe um valor em centavos nas peças de DINHEIRO. A divisão inteira
/// diz quantas peças cabem; o resto (%) é o que sobra para as menores.
fn decompor(valor: i64) -> Vec<Item> {
    let mut restante = valor;
    let mut itens = Vec::new();
    for dinheiro in DINHEIRO {
        let quantidade = restante / dinheiro.centavos;
        if quantidade > 0 {
            itens.push(Item { dinheiro, quantidade });
            restante %= dinheiro.centavos;
        }
    }
    itens
}

/// Pagamento menor que a compra é erro de negócio: vira Err com a diferença.
fn calcular_troco(compra: i64, pago: i64) -> Result<Vec<Item>, String> {
    if pago < compra {
        return Err(format!("pagamento insuficiente: faltam {}", moeda(compra - pago)));
    }
    Ok(decompor(pago - compra))
}

/// Formata centavos como "R$ 1.234,56".
fn moeda(centavos: i64) -> String {
    let digitos = (centavos / 100).to_string();
    let mut inteiro = String::new();
    for (i, digito) in digitos.chars().enumerate() {
        if i > 0 && (digitos.len() - i) % 3 == 0 {
            inteiro.push('.');
        }
        inteiro.push(digito);
    }
    format!("R$ {inteiro},{:02}", centavos % 100)
}

fn descrever(item: &Item) -> String {
    let nome = match (item.dinheiro.especie, item.quantidade) {
        (Especie::Cedula, 1) => "cédula",
        (Especie::Cedula, _) => "cédulas",
        (Especie::Moeda, 1) => "moeda",
        (Especie::Moeda, _) => "moedas",
    };
    format!("{} {} de {}", item.quantidade, nome, moeda(item.dinheiro.centavos))
}

fn imprimir_itens(troco: i64, itens: &[Item]) {
    if itens.is_empty() {
        println!("  Valor exato: não há troco.");
        return;
    }
    let pecas: i64 = itens.iter().map(|item| item.quantidade).sum();
    println!("  Troco de {} em {} peça(s):", moeda(troco), pecas);
    for item in itens {
        println!("    {}", descrever(item));
    }
}

/// Modo normal: pares compra pago. Qualquer erro, inclusive pagamento
/// insuficiente, sobe com ? e encerra o programa com código 1.
fn executar(args: &[String]) -> Result<(), String> {
    if args.len() % 2 != 0 {
        return Err(format!("os argumentos vêm em pares compra pago, mas recebi {}", args.len()));
    }
    for par in args.chunks(2) {
        let compra = ler_centavos(&par[0], "valor da compra")?;
        let pago = ler_centavos(&par[1], "valor pago")?;
        let itens = calcular_troco(compra, pago)?;
        println!("Compra de {} paga com {}", moeda(compra), moeda(pago));
        imprimir_itens(pago - compra, &itens);
    }
    Ok(())
}

/// Na demonstração o erro é tratado no local com match, para mostrar o aviso
/// sem interromper os outros casos.
fn demo() -> Result<(), String> {
    println!("=== Demonstração: quatro compras e a decomposição de cada troco ===");
    println!();
    let casos = [(3735, 5000), (19999, 20000), (1250, 10000), (8000, 5000)];
    for (compra, pago) in casos {
        println!("Compra de {} paga com {}", moeda(compra), moeda(pago));
        match calcular_troco(compra, pago) {
            Ok(itens) => imprimir_itens(pago - compra, &itens),
            Err(aviso) => println!("  Aviso: {aviso}"),
        }
        println!();
    }
    let cedulas = DINHEIRO.iter().filter(|d| matches!(d.especie, Especie::Cedula)).count();
    println!("Peças consideradas: {} cédulas e {} moedas, de {} até {}.",
        cedulas, DINHEIRO.len() - cedulas, moeda(DINHEIRO[0].centavos), moeda(DINHEIRO[DINHEIRO.len() - 1].centavos));
    Ok(())
}

fn imprimir_uso() {
    println!("Calculadora de troco (CodeNomad Academy)");
    println!();
    println!("Uso: troco.exe COMPRA PAGO [COMPRA PAGO ...]");
    println!("  COMPRA  valor da compra, em reais (ex.: 37,35)");
    println!("  PAGO    valor entregue pelo cliente, em reais (ex.: 50)");
    println!();
    println!("Exemplos:");
    println!("  troco.exe 37,35 50");
    println!("  troco.exe 37,35 50 199,99 200");
    println!("  troco.exe --demo      roda uma demonstração completa");
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
// 1. Acrescente a opção --sem NOTA (ex.: --sem 200) para simular um caixa que
//    não tem certa cédula, e veja como a decomposição muda.
// 2. Some os trocos de todos os pares e mostre, no fim, quantas peças de cada
//    valor o caixa precisa ter para o dia (use um HashMap<i64, i64>).
// 3. Invente um sistema de moedas em que o guloso erra (por exemplo 1, 3 e 4)
//    e resolva com programação dinâmica para achar o mínimo de verdade.
// ------------------------------------------------------------
