// ============================================================
// CodeNomad Academy  |  Sources Rust  |  02. Simulador de financiamento (Price e SAC)
// ------------------------------------------------------------
// O que faz: recebe o valor financiado, a taxa mensal e o prazo em meses.
//            Gera a tabela de parcelas nos dois sistemas mais usados no
//            Brasil (Price, de parcelas fixas, e SAC, de amortização
//            constante) e compara o total pago em cada um.
// Como usar: rustc -O --edition 2021 -D warnings 02_simulador_financiamento.rs -o financiamento.exe
//            financiamento.exe 50000 1 12          (valor, taxa % ao mês, prazo em meses)
//            financiamento.exe 250000 0,75 360
//            financiamento.exe --demo
// O que você aprende: enum com Display, Clone e Copy, match para escolher o
//            cálculo, powi, Vec de structs, iteradores (map, sum), Result com ?.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::fmt;
use std::process::ExitCode;

/// Os dois sistemas de amortização. Clone e Copy deixam passar o enum por valor, como um número.
#[derive(Clone, Copy)]
enum Sistema {
    Price,
    Sac,
}

/// Display define como o enum aparece em println!("{}", sistema).
impl fmt::Display for Sistema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sistema::Price => write!(f, "Tabela Price (parcelas fixas)"),
            Sistema::Sac => write!(f, "SAC (amortização constante)"),
        }
    }
}

/// Uma parcela: quanto se paga, quanto é juros, quanto abate a dívida e quanto ainda falta.
struct Parcela {
    numero: u32,
    valor: f64,
    juros: f64,
    amortizacao: f64,
    saldo_devedor: f64,
}

struct Financiamento {
    valor: f64,
    taxa: f64,
    prazo: u32,
}

/// Totais de um sistema, para a comparação final.
struct Resumo {
    sistema: Sistema,
    total_pago: f64,
    total_juros: f64,
    primeira: f64,
    ultima: f64,
}

impl Financiamento {
    fn novo(valor: f64, taxa_percentual: f64, prazo: u32) -> Result<Financiamento, String> {
        if valor <= 0.0 {
            return Err(format!("o valor financiado precisa ser maior que zero, recebi {valor}"));
        }
        if !(0.0..=50.0).contains(&taxa_percentual) {
            return Err(format!("taxa mensal fora da faixa de 0 a 50%: {taxa_percentual}"));
        }
        if prazo == 0 || prazo > 600 {
            return Err(format!("o prazo precisa estar entre 1 e 600 meses, recebi {prazo}"));
        }
        Ok(Financiamento { valor, taxa: taxa_percentual / 100.0, prazo })
    }

    /// Prestação fixa da Price: PV * i / (1 - (1 + i)^-n). Com taxa zero a
    /// fórmula divide por zero, então esse caso vira uma divisão simples.
    fn prestacao_price(&self) -> f64 {
        let n = f64::from(self.prazo);
        if self.taxa == 0.0 {
            return self.valor / n;
        }
        // O prazo foi validado até 600, então o cast para i32 é seguro.
        let fator = (1.0 + self.taxa).powi(self.prazo as i32);
        self.valor * self.taxa * fator / (fator - 1.0)
    }

    /// Monta a tabela do sistema pedido: só a parcela e a amortização mudam entre Price e SAC.
    fn parcelas(&self, sistema: Sistema) -> Vec<Parcela> {
        let prestacao_fixa = self.prestacao_price();
        let amortizacao_fixa = self.valor / f64::from(self.prazo);
        let mut saldo = self.valor;
        let mut tabela = Vec::with_capacity(self.prazo as usize);
        for numero in 1..=self.prazo {
            let juros = saldo * self.taxa;
            let (valor, amortizacao) = match sistema {
                Sistema::Price => (prestacao_fixa, prestacao_fixa - juros),
                Sistema::Sac => (amortizacao_fixa + juros, amortizacao_fixa),
            };
            saldo -= amortizacao;
            // Ponto flutuante acumula resíduo: no fim pode sobrar 0,0000001. Zera.
            if saldo.abs() < 0.005 {
                saldo = 0.0;
            }
            tabela.push(Parcela { numero, valor, juros, amortizacao, saldo_devedor: saldo });
        }
        tabela
    }
}

fn resumir(sistema: Sistema, tabela: &[Parcela]) -> Resumo {
    Resumo {
        sistema,
        total_pago: tabela.iter().map(|p| p.valor).sum(),
        total_juros: tabela.iter().map(|p| p.juros).sum(),
        primeira: tabela.first().map_or(0.0, |p| p.valor),
        ultima: tabela.last().map_or(0.0, |p| p.valor),
    }
}

/// Formata no padrão brasileiro: 1234.5 vira "1.234,50".
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

fn ler_numero(texto: &str, nome: &str) -> Result<f64, String> {
    let limpo = texto.trim().replace(',', ".");
    match limpo.parse::<f64>() {
        Ok(valor) if valor.is_finite() => Ok(valor),
        _ => Err(format!("valor inválido para {nome}: \"{texto}\" não é um número")),
    }
}

fn imprimir_tabela(sistema: Sistema, tabela: &[Parcela]) {
    println!("{sistema}");
    println!("{:>4}  {:>14}  {:>13}  {:>14}  {:>15}", "N.", "Parcela", "Juros", "Amortização", "Saldo devedor");
    println!("{}", "-".repeat(70));
    for p in tabela {
        println!(
            "{:>4}  {:>14}  {:>13}  {:>14}  {:>15}",
            p.numero, moeda(p.valor), moeda(p.juros), moeda(p.amortizacao), moeda(p.saldo_devedor)
        );
    }
    println!();
}

fn imprimir_comparacao(fin: &Financiamento, price: &Resumo, sac: &Resumo) {
    println!("Comparação para {} a {}% ao mês em {} meses", moeda(fin.valor), formatar(fin.taxa * 100.0, 2), fin.prazo);
    println!("{:<32} {:>16} {:>16}", "", "Price", "SAC");
    println!("{:<32} {:>16} {:>16}", "Primeira parcela", moeda(price.primeira), moeda(sac.primeira));
    println!("{:<32} {:>16} {:>16}", "Última parcela", moeda(price.ultima), moeda(sac.ultima));
    println!("{:<32} {:>16} {:>16}", "Total de juros", moeda(price.total_juros), moeda(sac.total_juros));
    println!("{:<32} {:>16} {:>16}", "Total pago", moeda(price.total_pago), moeda(sac.total_pago));
    let diferenca = price.total_pago - sac.total_pago;
    // Qual sai mais barato no total: com taxa zero empatam.
    let (mais_barato, economia) = if diferenca > 0.005 {
        (sac.sistema, diferenca)
    } else if diferenca < -0.005 {
        (price.sistema, -diferenca)
    } else {
        println!("Os dois sistemas custam o mesmo neste cenário.");
        return;
    };
    println!();
    println!("No total, {} sai {} mais barato.", mais_barato, moeda(economia));
    println!("Em troca, o SAC começa com parcelas maiores e a Price mantém a parcela igual do início ao fim.");
}

fn rodar(fin: &Financiamento) {
    let price = fin.parcelas(Sistema::Price);
    let sac = fin.parcelas(Sistema::Sac);
    imprimir_tabela(Sistema::Price, &price);
    imprimir_tabela(Sistema::Sac, &sac);
    imprimir_comparacao(fin, &resumir(Sistema::Price, &price), &resumir(Sistema::Sac, &sac));
}

fn executar(args: &[String]) -> Result<(), String> {
    if args.len() != 3 {
        return Err(format!("esperava 3 argumentos (valor, taxa, prazo), recebi {}", args.len()));
    }
    let valor = ler_numero(&args[0], "valor financiado")?;
    let taxa = ler_numero(&args[1], "taxa")?;
    let prazo = args[2].trim().parse::<u32>()
        .map_err(|_| format!("prazo inválido: \"{}\" não é um inteiro positivo", args[2]))?;
    let fin = Financiamento::novo(valor, taxa, prazo)?;
    rodar(&fin);
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Demonstração: R$ 50.000 a 1% ao mês em 12 meses ===");
    let fin = Financiamento::novo(50_000.0, 1.0, 12)?;
    rodar(&fin);
    Ok(())
}

fn imprimir_uso() {
    println!("Simulador de financiamento, Price e SAC (CodeNomad Academy)");
    println!("Uso: financiamento.exe VALOR TAXA_MENSAL PRAZO");
    println!("  VALOR        quanto foi financiado, em reais (ex.: 50000)");
    println!("  TAXA_MENSAL  juros ao mês, em porcentagem (ex.: 1 ou 0,75)");
    println!("  PRAZO        quantidade de parcelas, de 1 a 600");
    println!();
    println!("Exemplos:");
    println!("  financiamento.exe 250000 0,75 360");
    println!("  financiamento.exe --demo      roda uma demonstração completa");
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
// 1. Acrescente uma entrada, por exemplo 20% do valor, e financie só o resto.
// 2. Simule uma amortização extra no mês N e mostre quantos meses o prazo encurta.
// 3. Some uma tarifa fixa mensal a cada parcela e compare de novo os dois sistemas.
// ------------------------------------------------------------
