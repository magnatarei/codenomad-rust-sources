// ============================================================
// CodeNomad Academy  |  Sources Rust  |  07. Calculadora de orçamento mensal
// ------------------------------------------------------------
// O que faz: recebe receitas e despesas por categoria, vindas dos argumentos
//            ou de um arquivo de texto simples. Mostra os totais, o saldo, o
//            percentual de cada categoria sobre a receita, barras de texto e
//            um alerta para a categoria que mais pesa.
// Como usar: rustc -O --edition 2021 -D warnings 07_calculadora_orcamento.rs -o orcamento.exe
//            orcamento.exe receita:Salário=5000 despesa:Moradia=1500 despesa:Mercado=900
//            orcamento.exe --arquivo orcamento.txt    (uma linha por item, # para comentário)
//            orcamento.exe --demo
// O que você aprende: split_once para quebrar texto, enum com PartialEq,
//            filter e sum em iteradores, sort_by com total_cmp, std::fs para
//            ler e gravar arquivo, env::temp_dir, Path e map_err com contexto.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

#[derive(Clone, Copy, PartialEq)]
enum Tipo {
    Receita,
    Despesa,
}

struct Item {
    tipo: Tipo,
    categoria: String,
    valor: f64,
}

/// Orçamento de exemplo, no mesmo formato que o arquivo aceita.
const EXEMPLO: &str = "# Orçamento de exemplo da CodeNomad Academy
receita: Salário = 5200
receita: Trabalho extra = 800
despesa: Moradia = 1800
despesa: Alimentação = 1100
despesa: Transporte = 450
despesa: Saúde = 320
despesa: Educação = 600
despesa: Lazer = 400
despesa: Internet e celular = 180
";

fn ler_numero(texto: &str) -> Result<f64, String> {
    match texto.trim().replace(',', ".").parse::<f64>() {
        Ok(valor) if valor.is_finite() && valor >= 0.0 => Ok(valor),
        _ => Err(format!("\"{}\" não é um valor válido (use número sem sinal, ex.: 1500 ou 1500,50)", texto.trim())),
    }
}

/// Interpreta "despesa: Moradia = 1500". split_once corta no primeiro ':' e
/// depois no primeiro '=', devolvendo None quando o separador não existe.
fn interpretar_item(texto: &str) -> Result<Item, String> {
    let (tipo_texto, resto) = texto
        .split_once(':')
        .ok_or_else(|| format!("\"{texto}\" não está no formato tipo:categoria=valor"))?;
    let (categoria, valor_texto) = resto
        .split_once('=')
        .ok_or_else(|| format!("\"{texto}\" está sem o '=' antes do valor"))?;
    let tipo = match tipo_texto.trim().to_lowercase().as_str() {
        "receita" | "r" => Tipo::Receita,
        "despesa" | "d" => Tipo::Despesa,
        outro => return Err(format!("tipo \"{outro}\" desconhecido em \"{texto}\": use receita ou despesa")),
    };
    let categoria = categoria.trim();
    if categoria.is_empty() {
        return Err(format!("categoria vazia em \"{texto}\""));
    }
    Ok(Item { tipo, categoria: categoria.to_string(), valor: ler_numero(valor_texto)? })
}

/// Linhas vazias e comentários (#) são puladas; o número da linha entra na
/// mensagem de erro para o usuário achar o problema no arquivo.
fn interpretar_linhas(texto: &str) -> Result<Vec<Item>, String> {
    let mut itens = Vec::new();
    for (indice, linha) in texto.lines().enumerate() {
        let limpa = linha.trim();
        if limpa.is_empty() || limpa.starts_with('#') {
            continue;
        }
        itens.push(interpretar_item(limpa).map_err(|erro| format!("linha {}: {erro}", indice + 1))?);
    }
    Ok(itens)
}

fn ler_arquivo(caminho: &Path) -> Result<Vec<Item>, String> {
    let texto = fs::read_to_string(caminho).map_err(|erro| format!("não consegui ler {}: {erro}", caminho.display()))?;
    interpretar_linhas(&texto)
}

/// Formata no padrão brasileiro: 1234.5 vira "R$ 1.234,50".
fn moeda(valor: f64) -> String {
    let texto = format!("{:.2}", valor.abs());
    let (inteiro, fracao) = texto.split_once('.').unwrap_or((texto.as_str(), "00"));
    let mut saida = String::new();
    for (i, digito) in inteiro.chars().enumerate() {
        if i > 0 && (inteiro.len() - i) % 3 == 0 {
            saida.push('.');
        }
        saida.push(digito);
    }
    let sinal = if valor < 0.0 { "-" } else { "" };
    format!("{sinal}R$ {saida},{fracao}")
}

fn percentual(parte: f64, total: f64) -> String {
    format!("{:.1}", parte / total * 100.0).replace('.', ",")
}

/// Cada bloco vale 2% da receita; o teto de 50 blocos evita uma barra sem fim.
fn barra(parte: f64, total: f64) -> String {
    let blocos = (parte / total * 50.0).round().clamp(0.0, 50.0) as usize;
    "█".repeat(blocos)
}

fn relatorio(itens: &[Item]) -> Result<(), String> {
    let receitas: Vec<&Item> = itens.iter().filter(|item| item.tipo == Tipo::Receita).collect();
    let mut despesas: Vec<&Item> = itens.iter().filter(|item| item.tipo == Tipo::Despesa).collect();
    let total_receitas: f64 = receitas.iter().map(|item| item.valor).sum();
    let total_despesas: f64 = despesas.iter().map(|item| item.valor).sum();
    if total_receitas <= 0.0 {
        return Err("informe ao menos uma receita maior que zero".to_string());
    }
    // f64 não tem Ord por causa do NaN; total_cmp resolve. b antes de a inverte a ordem.
    despesas.sort_by(|a, b| b.valor.total_cmp(&a.valor));

    println!("RECEITAS");
    for item in &receitas {
        println!("  {:<20} {:>14} {:>6}%", item.categoria, moeda(item.valor), percentual(item.valor, total_receitas));
    }
    println!();
    println!("DESPESAS (da maior para a menor; cada █ é 2% da receita)");
    for item in &despesas {
        println!(
            "  {:<20} {:>14} {:>6}%  {}",
            item.categoria,
            moeda(item.valor),
            percentual(item.valor, total_receitas),
            barra(item.valor, total_receitas)
        );
    }
    println!();
    let saldo = total_receitas - total_despesas;
    println!("Total de receitas .: {:>14}", moeda(total_receitas));
    println!("Total de despesas .: {:>14}  ({}% da receita)", moeda(total_despesas), percentual(total_despesas, total_receitas));
    println!("Saldo do mês ......: {:>14}", moeda(saldo));
    println!();
    // first() devolve Option: sem despesas não há alerta a dar.
    if let Some(maior) = despesas.first() {
        println!("Alerta: a categoria que mais pesa é {} ({}% da receita).", maior.categoria, percentual(maior.valor, total_receitas));
    }
    if saldo < 0.0 {
        println!("Atenção: as despesas passam da receita em {}.", moeda(-saldo));
    } else {
        println!("Sobram {} ({}% da receita) para poupar ou investir.", moeda(saldo), percentual(saldo, total_receitas));
    }
    Ok(())
}

fn executar(args: &[String]) -> Result<(), String> {
    let itens = if args[0] == "--arquivo" {
        let caminho = args.get(1).ok_or("faltou o caminho depois de --arquivo")?;
        ler_arquivo(Path::new(caminho))?
    } else {
        args.iter().map(|texto| interpretar_item(texto)).collect::<Result<Vec<Item>, String>>()?
    };
    relatorio(&itens)
}

fn demo() -> Result<(), String> {
    println!("=== Demonstração: orçamento de exemplo gravado e lido de um arquivo ===");
    // temp_dir aponta para a pasta temporária do sistema; é o único lugar onde a demonstração escreve.
    let caminho = env::temp_dir().join("codenomad_orcamento_demo.txt");
    fs::write(&caminho, EXEMPLO).map_err(|erro| format!("não consegui gravar {}: {erro}", caminho.display()))?;
    println!("Arquivo de exemplo: {}", caminho.display());
    println!();
    let resultado = ler_arquivo(&caminho).and_then(|itens| relatorio(&itens));
    // Apaga o arquivo antes de devolver o resultado, deu certo ou não.
    let _ = fs::remove_file(&caminho);
    resultado?;
    println!();
    println!("O mesmo por argumentos: orcamento.exe receita:Salário=5200 despesa:Moradia=1800 ...");
    Ok(())
}

fn imprimir_uso() {
    println!("Calculadora de orçamento mensal (CodeNomad Academy)");
    println!();
    println!("Uso: orcamento.exe ITEM [ITEM ...]");
    println!("     orcamento.exe --arquivo CAMINHO");
    println!("  ITEM tem o formato tipo:categoria=valor, com tipo receita ou despesa.");
    println!("  No arquivo vai um item por linha; linhas vazias e as que começam com # são ignoradas.");
    println!();
    println!("Exemplos:");
    println!("  orcamento.exe receita:Salário=5000 despesa:Moradia=1500 despesa:Mercado=900");
    println!("  orcamento.exe --arquivo orcamento.txt");
    println!("  orcamento.exe --demo      roda uma demonstração completa");
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
// 1. Some categorias repetidas (duas linhas "Mercado") em uma só, usando um
//    HashMap<String, f64> antes de montar o relatório.
// 2. Aceite uma meta por categoria ("despesa:Lazer=400/300") e marque com um
//    aviso cada categoria que passou da meta.
// 3. Compare dois meses: leia dois arquivos e mostre, por categoria, quanto
//    subiu ou caiu de um para o outro.
// ------------------------------------------------------------
