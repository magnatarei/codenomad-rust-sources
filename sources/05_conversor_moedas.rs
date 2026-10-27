// ============================================================
// CodeNomad Academy  |  Sources Rust  |  05. Conversor de moedas com tabela de taxas
// ------------------------------------------------------------
// O que faz: converte um valor entre quaisquer duas moedas de uma tabela
//            embutida no código (USD, EUR, GBP, JPY, ARS, BRL e outras), com a
//            data em que a tabela foi conferida. Lista todas as taxas.
// Como usar: rustc -O --edition 2021 -D warnings 05_conversor_moedas.rs -o moedas.exe
//            moedas.exe 100 USD BRL          (valor, moeda de origem, moeda de destino)
//            moedas.exe 250,50 BRL EUR
//            moedas.exe --taxas
//            moedas.exe --demo
//            Para atualizar a tabela: troque os números em TAXAS pelos de uma
//            fonte oficial de câmbio (quantos reais vale 1 unidade de cada
//            moeda) e mude DATA_TABELA. Nada mais precisa ser alterado.
// O que você aprende: HashMap construído com collect, referências &'static,
//            struct com métodos, Option para Result com ok_or_else, ordenação
//            com sort_by, formatação de números no padrão brasileiro.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::collections::HashMap;
use std::env;
use std::process::ExitCode;

/// Data em que os valores foram conferidos. Atualize junto com a tabela.
const DATA_TABELA: &str = "27/10/2026";

struct Moeda {
    codigo: &'static str,
    nome: &'static str,
    reais_por_unidade: f64,
}

/// Cada linha diz quantos reais vale 1 unidade da moeda. O real é a base
/// (1,0), por isso toda conversão passa por ele. Valores aproximados, para estudo.
const TAXAS: &[Moeda] = &[
    Moeda { codigo: "BRL", nome: "real brasileiro", reais_por_unidade: 1.0 },
    Moeda { codigo: "USD", nome: "dólar americano", reais_por_unidade: 5.35 },
    Moeda { codigo: "EUR", nome: "euro", reais_por_unidade: 5.85 },
    Moeda { codigo: "GBP", nome: "libra esterlina", reais_por_unidade: 6.80 },
    Moeda { codigo: "JPY", nome: "iene japonês", reais_por_unidade: 0.035 },
    Moeda { codigo: "ARS", nome: "peso argentino", reais_por_unidade: 0.0038 },
    Moeda { codigo: "CHF", nome: "franco suíço", reais_por_unidade: 6.20 },
    Moeda { codigo: "CAD", nome: "dólar canadense", reais_por_unidade: 3.90 },
    Moeda { codigo: "AUD", nome: "dólar australiano", reais_por_unidade: 3.50 },
    Moeda { codigo: "CNY", nome: "yuan chinês", reais_por_unidade: 0.75 },
    Moeda { codigo: "MXN", nome: "peso mexicano", reais_por_unidade: 0.29 },
    Moeda { codigo: "CLP", nome: "peso chileno", reais_por_unidade: 0.0057 },
    Moeda { codigo: "UYU", nome: "peso uruguaio", reais_por_unidade: 0.13 },
    Moeda { codigo: "PYG", nome: "guarani paraguaio", reais_por_unidade: 0.00072 },
    Moeda { codigo: "INR", nome: "rúpia indiana", reais_por_unidade: 0.063 },
];

/// A tabela indexada por código. As referências são &'static porque apontam
/// para a const TAXAS, que vive durante o programa inteiro.
struct Tabela {
    por_codigo: HashMap<&'static str, &'static Moeda>,
}

/// O resultado de uma conversão, com tudo que a saída precisa mostrar.
struct Conversao {
    valor: f64,
    origem: &'static Moeda,
    destino: &'static Moeda,
    taxa: f64,
    resultado: f64,
}

impl Tabela {
    /// collect monta o HashMap a partir de um iterador de pares (chave, valor).
    fn carregar() -> Tabela {
        let por_codigo = TAXAS.iter().map(|moeda| (moeda.codigo, moeda)).collect();
        Tabela { por_codigo }
    }

    /// Aceita "usd" ou "USD": normaliza para maiúsculas antes de procurar.
    fn buscar(&self, codigo: &str) -> Result<&'static Moeda, String> {
        let chave = codigo.trim().to_ascii_uppercase();
        self.por_codigo
            .get(chave.as_str())
            .copied()
            .ok_or_else(|| format!("moeda desconhecida: \"{codigo}\". Use --taxas para ver a lista"))
    }

    fn converter(&self, valor: f64, de: &str, para: &str) -> Result<Conversao, String> {
        if valor < 0.0 {
            return Err(format!("o valor não pode ser negativo, recebi {valor}"));
        }
        let origem = self.buscar(de)?;
        let destino = self.buscar(para)?;
        // Leva para reais e depois para o destino. A taxa direta é a razão entre as duas.
        let taxa = origem.reais_por_unidade / destino.reais_por_unidade;
        Ok(Conversao { valor, origem, destino, taxa, resultado: valor * taxa })
    }

    fn listar(&self) {
        // values() vem em ordem imprevisível, por isso ordena pelo código.
        let mut moedas: Vec<&Moeda> = self.por_codigo.values().copied().collect();
        moedas.sort_by(|a, b| a.codigo.cmp(b.codigo));
        println!("Tabela de taxas conferida em {DATA_TABELA} ({} moedas)", moedas.len());
        println!("{:<7} {:<20} {:>18} {:>20}", "Código", "Moeda", "1 unidade em R$", "R$ 1,00 em unidades");
        println!("{}", "-".repeat(68));
        for moeda in moedas {
            println!(
                "{:<7} {:<20} {:>18} {:>20}",
                moeda.codigo,
                moeda.nome,
                formatar(moeda.reais_por_unidade, 4),
                formatar(1.0 / moeda.reais_por_unidade, 4)
            );
        }
    }
}

/// Formata no padrão brasileiro: ponto nos milhares e vírgula nas casas.
fn formatar(valor: f64, casas: usize) -> String {
    let texto = format!("{:.*}", casas, valor.abs());
    let (inteiro, fracao) = texto.split_once('.').unwrap_or((texto.as_str(), ""));
    let mut saida = String::new();
    for (i, digito) in inteiro.chars().enumerate() {
        if i > 0 && (inteiro.len() - i) % 3 == 0 {
            saida.push('.');
        }
        saida.push(digito);
    }
    if !fracao.is_empty() {
        saida.push(',');
        saida.push_str(fracao);
    }
    saida
}

fn ler_numero(texto: &str) -> Result<f64, String> {
    match texto.trim().replace(',', ".").parse::<f64>() {
        Ok(valor) if valor.is_finite() => Ok(valor),
        _ => Err(format!("valor inválido: \"{texto}\" não é um número")),
    }
}

fn imprimir(c: &Conversao) {
    // Resultados pequenos (iene virando dólar, por exemplo) ganham mais casas.
    let casas = if c.resultado.abs() < 1.0 { 4 } else { 2 };
    println!(
        "{:>14} {} = {:>16} {}   (1 {} = {} {})",
        formatar(c.valor, 2),
        c.origem.codigo,
        formatar(c.resultado, casas),
        c.destino.codigo,
        c.origem.codigo,
        formatar(c.taxa, 4),
        c.destino.codigo
    );
}

fn executar(args: &[String]) -> Result<(), String> {
    let tabela = Tabela::carregar();
    if args[0] == "--taxas" {
        tabela.listar();
        return Ok(());
    }
    if args.len() != 3 {
        return Err(format!("esperava VALOR DE PARA (3 argumentos), recebi {}", args.len()));
    }
    let valor = ler_numero(&args[0])?;
    let conversao = tabela.converter(valor, &args[1], &args[2])?;
    imprimir(&conversao);
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Demonstração: 100 unidades em seis pares de moedas ===");
    println!();
    let tabela = Tabela::carregar();
    let pares = [("USD", "BRL"), ("BRL", "USD"), ("EUR", "GBP"), ("JPY", "BRL"), ("BRL", "ARS"), ("GBP", "JPY")];
    for (de, para) in pares {
        imprimir(&tabela.converter(100.0, de, para)?);
    }
    println!();
    tabela.listar();
    Ok(())
}

fn imprimir_uso() {
    println!("Conversor de moedas (CodeNomad Academy)");
    println!();
    println!("Uso: moedas.exe VALOR DE PARA");
    println!("     moedas.exe --taxas");
    println!();
    println!("Exemplos:");
    println!("  moedas.exe 100 USD BRL");
    println!("  moedas.exe 250,50 BRL EUR");
    println!("  moedas.exe --demo       roda uma demonstração completa");
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
// 1. Leia a tabela de um arquivo "taxas.txt" (uma moeda por linha) com
//    std::fs::read_to_string, mantendo a const como reserva se o arquivo faltar.
// 2. Acrescente o comando "inverso": dado o valor em reais, quanto compra em
//    cada uma das moedas da tabela, em uma só listagem.
// 3. Aceite vários valores de uma vez ("10 20 50 USD BRL") e some o total.
// ------------------------------------------------------------
