// ============================================================
// CodeNomad Academy  |  Sources Rust  |  01. Calculadora de juros compostos
// ------------------------------------------------------------
// O que faz: recebe capital inicial, taxa mensal, número de meses e um aporte
//            mensal opcional. Mostra o montante final, os juros ganhos e uma
//            tabela mês a mês (saldo inicial, juros, aporte, saldo final).
// Como usar: rustc -O --edition 2021 -D warnings 01_calculadora_juros_compostos.rs -o juros.exe
//            juros.exe 1000 0,8 24 200     (capital, taxa % ao mês, meses, aporte mensal)
//            juros.exe 5000 1 12           (sem aporte)
//            juros.exe --demo
// O que você aprende: struct e impl, Vec e iteradores (map, sum, last), Result
//            com ? e map_err, parse de texto para número, formatação de tabela
//            com largura fixa e ExitCode para devolver o código de saída.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::process::ExitCode;

/// Parâmetros da simulação. A taxa fica guardada em fração: 0,8% vira 0,008.
struct Simulacao {
    capital: f64,
    taxa: f64,
    meses: u32,
    aporte: f64,
}

/// Uma linha da tabela: o que aconteceu com o dinheiro em um mês.
struct Linha {
    mes: u32,
    saldo_inicial: f64,
    juros: f64,
    aporte: f64,
    saldo_final: f64,
}

impl Simulacao {
    /// Valida tudo antes de criar a simulação. Entrada ruim para aqui, com
    /// uma mensagem que explica o problema, e nunca chega ao cálculo.
    fn nova(capital: f64, taxa_percentual: f64, meses: u32, aporte: f64) -> Result<Simulacao, String> {
        if capital < 0.0 || aporte < 0.0 {
            return Err("capital e aporte não podem ser negativos".to_string());
        }
        if capital == 0.0 && aporte == 0.0 {
            return Err("sem capital e sem aporte não há o que simular".to_string());
        }
        if !(0.0..=100.0).contains(&taxa_percentual) {
            return Err(format!("taxa mensal fora da faixa de 0 a 100%: {taxa_percentual}"));
        }
        if meses == 0 || meses > 1200 {
            return Err(format!("o número de meses precisa estar entre 1 e 1200, recebi {meses}"));
        }
        Ok(Simulacao { capital, taxa: taxa_percentual / 100.0, meses, aporte })
    }

    /// Gera a tabela mês a mês. Convenção: os juros rendem sobre o saldo do
    /// início do mês e o aporte entra no fim, por isso só rende no mês seguinte.
    fn simular(&self) -> Vec<Linha> {
        let mut linhas = Vec::with_capacity(self.meses as usize);
        let mut saldo = self.capital;
        for mes in 1..=self.meses {
            let juros = saldo * self.taxa;
            let saldo_final = saldo + juros + self.aporte;
            linhas.push(Linha { mes, saldo_inicial: saldo, juros, aporte: self.aporte, saldo_final });
            saldo = saldo_final;
        }
        linhas
    }

    /// Quanto saiu do bolso: capital mais todos os aportes.
    fn total_investido(&self) -> f64 {
        self.capital + self.aporte * f64::from(self.meses)
    }

    /// Taxa anual equivalente: (1 + i)^12 - 1, em porcentagem.
    fn taxa_anual(&self) -> f64 {
        ((1.0 + self.taxa).powi(12) - 1.0) * 100.0
    }
}

/// Formata um número no padrão brasileiro: ponto nos milhares e vírgula nas casas.
/// Exemplo: formatar(1234.5, 2) devolve "1.234,50".
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
    // Só mostra o sinal se sobrou algum dígito diferente de zero depois do arredondamento.
    let tem_valor = saida.chars().any(|c| c != '0' && c != '.' && c != ',');
    if valor < 0.0 && tem_valor {
        format!("-{saida}")
    } else {
        saida
    }
}

fn moeda(valor: f64) -> String {
    format!("R$ {}", formatar(valor, 2))
}

/// Lê um número aceitando vírgula ou ponto nas casas: "0,8" e "0.8" valem o mesmo.
fn ler_numero(texto: &str, nome: &str) -> Result<f64, String> {
    let limpo = texto.trim().replace(',', ".");
    match limpo.parse::<f64>() {
        Ok(valor) if valor.is_finite() => Ok(valor),
        _ => Err(format!("valor inválido para {nome}: \"{texto}\" não é um número")),
    }
}

fn imprimir_tabela(linhas: &[Linha]) {
    println!("{:>4}  {:>15}  {:>13}  {:>12}  {:>15}", "Mês", "Saldo inicial", "Juros", "Aporte", "Saldo final");
    println!("{}", "-".repeat(68));
    for linha in linhas {
        println!(
            "{:>4}  {:>15}  {:>13}  {:>12}  {:>15}",
            linha.mes,
            moeda(linha.saldo_inicial),
            moeda(linha.juros),
            moeda(linha.aporte),
            moeda(linha.saldo_final)
        );
    }
}

fn imprimir_resumo(sim: &Simulacao, linhas: &[Linha]) {
    // last() devolve Option: se a lista estivesse vazia, o montante seria o capital.
    let montante = linhas.last().map_or(sim.capital, |linha| linha.saldo_final);
    let juros_total: f64 = linhas.iter().map(|linha| linha.juros).sum();
    let investido = sim.total_investido();
    println!();
    println!("Capital inicial .........: {:>15}", moeda(sim.capital));
    println!("Aportes .................: {:>15}  ({} x {})", moeda(investido - sim.capital), sim.meses, moeda(sim.aporte));
    println!("Total investido .........: {:>15}", moeda(investido));
    println!("Juros ganhos ............: {:>15}", moeda(juros_total));
    println!("Montante final ..........: {:>15}", moeda(montante));
    println!("Taxa anual equivalente ..: {:>14}%", formatar(sim.taxa_anual(), 2));
    println!("Ganho sobre o investido .: {:>14}%", formatar(juros_total / investido * 100.0, 2));
}

fn rodar(sim: &Simulacao) {
    println!(
        "Simulação: {} a {}% ao mês por {} meses, aporte mensal de {}",
        moeda(sim.capital),
        formatar(sim.taxa * 100.0, 2),
        sim.meses,
        moeda(sim.aporte)
    );
    println!();
    let linhas = sim.simular();
    imprimir_tabela(&linhas);
    imprimir_resumo(sim, &linhas);
}

/// Modo normal: interpreta os argumentos e roda. Qualquer erro sobe com ? e
/// vira uma mensagem única no main.
fn executar(args: &[String]) -> Result<(), String> {
    if args.len() < 3 || args.len() > 4 {
        return Err(format!("esperava 3 ou 4 argumentos, recebi {}. Rode sem argumentos para ver o uso", args.len()));
    }
    let capital = ler_numero(&args[0], "capital")?;
    let taxa = ler_numero(&args[1], "taxa")?;
    let meses = args[2]
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("meses inválido: \"{}\" não é um inteiro positivo", args[2]))?;
    let aporte = match args.get(3) {
        Some(texto) => ler_numero(texto, "aporte")?,
        None => 0.0,
    };
    let sim = Simulacao::nova(capital, taxa, meses, aporte)?;
    rodar(&sim);
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Demonstração: R$ 1.000 a 0,8% ao mês por 24 meses, com aporte de R$ 200 ===");
    println!();
    let sim = Simulacao::nova(1000.0, 0.8, 24, 200.0)?;
    rodar(&sim);
    Ok(())
}

fn imprimir_uso() {
    println!("Calculadora de juros compostos (CodeNomad Academy)");
    println!();
    println!("Uso: juros.exe CAPITAL TAXA_MENSAL MESES [APORTE]");
    println!("  CAPITAL      valor inicial, em reais (ex.: 1000)");
    println!("  TAXA_MENSAL  juros ao mês, em porcentagem (ex.: 0,8 para 0,8%)");
    println!("  MESES        quantidade de meses, de 1 a 1200");
    println!("  APORTE       depósito mensal opcional, em reais (ex.: 200)");
    println!();
    println!("Exemplos:");
    println!("  juros.exe 1000 0,8 24 200");
    println!("  juros.exe 5000 1 12");
    println!("  juros.exe --demo       roda uma demonstração completa");
}

fn main() -> ExitCode {
    // skip(1) pula o nome do programa, que sempre vem como primeiro argumento.
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
// 1. Aceite a taxa anual (ex.: "10a") e converta para mensal com
//    (1 + anual)^(1/12) - 1 antes de simular.
// 2. Acrescente a opção --csv para gravar a tabela em um arquivo, usando
//    std::fs::File e writeln!, e abra o resultado em uma planilha.
// 3. Inverta o problema: dado o montante desejado, descubra com busca binária
//    qual aporte mensal chega lá no prazo informado.
// ------------------------------------------------------------
