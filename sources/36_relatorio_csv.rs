// ============================================================
// CodeNomad Academy  |  Sources Rust  |  36. Relatório de vendas a partir de CSV
// ------------------------------------------------------------
// O que faz: Lê um CSV de vendas (data, produto, categoria, quantidade, valor
//            unitário), calcula o faturamento por categoria e por mês, o
//            ticket médio, os produtos mais vendidos e imprime um relatório
//            alinhado em colunas, com valores no formato brasileiro.
// Como usar: rustc -O --edition 2021 36_relatorio_csv.rs -o relatorio.exe
//            relatorio.exe vendas.csv
//            relatorio.exe vendas.csv --top 3
//            relatorio.exe --demo
// O que você aprende: a trait FromStr para converter texto em struct com ?,
//            BTreeMap para agrupar em ordem, iteradores com map, sum, fold e
//            max_by, formatação com largura e alinhamento ({:<12} {:>12}).
// Requisitos: Rust 1.70+, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

const USO: &str = "Uso: relatorio <vendas.csv> [--top N]
  O CSV precisa do cabeçalho: data,produto,categoria,quantidade,valor
  (data em AAAA-MM-DD, valor unitário com ponto decimal, ex.: 2026-10-05,Caneca,Casa,3,29.90)
  --top N     quantos produtos mostrar no ranking (padrão: 5)
  --demo      gera 60 vendas de exemplo e imprime o relatório";

struct Venda { data: String, produto: String, categoria: String, quantidade: u32, valor: f64 }

impl Venda {
    fn total(&self) -> f64 { self.quantidade as f64 * self.valor }
    // Os sete primeiros bytes de "2026-10-05" formam o mês: "2026-10".
    fn mes(&self) -> &str { &self.data[..7] }
}

// Implementar FromStr permite escrever linha.parse::<Venda>(), como se faz com números.
impl FromStr for Venda {
    type Err = String;
    fn from_str(linha: &str) -> Result<Self, String> {
        let campos: Vec<&str> = linha.split(',').map(str::trim).collect();
        let [data, produto, categoria, quantidade, valor] = campos.as_slice() else {
            return Err(format!("esperava 5 campos, achei {}", campos.len()));
        };
        let partes: Vec<u32> = data.split('-').filter_map(|p| p.parse().ok()).collect();
        if partes.len() != 3 || data.len() != 10 || !(1..=12).contains(&partes[1]) || !(1..=31).contains(&partes[2]) {
            return Err(format!("data inválida '{data}' (use AAAA-MM-DD)"));
        }
        if produto.is_empty() || categoria.is_empty() { return Err("produto e categoria não podem ficar vazios".into()); }
        let quantidade = quantidade.parse().map_err(|_| format!("quantidade inválida '{quantidade}'"))?;
        let valor: f64 = valor.parse().map_err(|_| format!("valor inválido '{valor}' (use ponto decimal)"))?;
        if quantidade == 0 || valor < 0.0 { return Err("quantidade precisa ser maior que zero e valor não pode ser negativo".into()); }
        Ok(Venda { data: data.to_string(), produto: produto.to_string(), categoria: categoria.to_string(), quantidade, valor })
    }
}

fn ler_vendas(caminho: &Path) -> Result<Vec<Venda>, String> {
    let texto = fs::read_to_string(caminho).map_err(|e| format!("não consegui ler {}: {e}", caminho.display()))?;
    let mut linhas = texto.lines().enumerate().filter(|(_, l)| !l.trim().is_empty());
    let (_, cabecalho) = linhas.next().ok_or("o arquivo está vazio")?;
    if cabecalho.replace(' ', "").to_lowercase() != "data,produto,categoria,quantidade,valor" {
        return Err(format!("cabeçalho inesperado: '{cabecalho}'\n{USO}"));
    }
    linhas.map(|(n, linha)| linha.parse().map_err(|e| format!("linha {}: {e}", n + 1))).collect()
}

// Formato brasileiro: ponto nos milhares, vírgula nos centavos.
fn moeda(valor: f64) -> String {
    let centavos = (valor * 100.0).round() as i64;
    let inteiro = (centavos / 100).to_string();
    let mut com_pontos = String::new();
    for (i, c) in inteiro.chars().enumerate() {
        if i > 0 && (inteiro.len() - i) % 3 == 0 { com_pontos.push('.'); }
        com_pontos.push(c);
    }
    format!("R$ {com_pontos},{:02}", centavos % 100)
}

fn linha_tabela(nome: &str, qtd: u32, total: f64, geral: f64) {
    println!("  {:<16} {:>6} {:>16} {:>6.1}%", nome, qtd, moeda(total), total / geral * 100.0);
}

fn imprimir_relatorio(vendas: &[Venda], top: usize) {
    let faturamento: f64 = vendas.iter().map(Venda::total).sum();
    let itens: u32 = vendas.iter().map(|v| v.quantidade).sum();
    let (inicio, fim) = vendas.iter().fold((&vendas[0].data, &vendas[0].data), |(a, b), v| (a.min(&v.data), b.max(&v.data)));
    println!("=== Relatório de vendas ===");
    println!("  Período:        {inicio} a {fim}");
    println!("  Vendas:         {} ({itens} itens)", vendas.len());
    println!("  Faturamento:    {}", moeda(faturamento));
    println!("  Ticket médio:   {}", moeda(faturamento / vendas.len() as f64));

    // O BTreeMap agrupa e já deixa as chaves em ordem alfabética (ou cronológica, no caso do mês).
    let mut por_categoria: BTreeMap<&str, (u32, f64)> = BTreeMap::new();
    let mut por_mes: BTreeMap<&str, (u32, f64)> = BTreeMap::new();
    let mut por_produto: BTreeMap<&str, (u32, f64)> = BTreeMap::new();
    for v in vendas {
        for (mapa, chave) in [(&mut por_categoria, v.categoria.as_str()), (&mut por_mes, v.mes()), (&mut por_produto, v.produto.as_str())] {
            let acumulado = mapa.entry(chave).or_insert((0, 0.0));
            acumulado.0 += v.quantidade;
            acumulado.1 += v.total();
        }
    }
    for (titulo, mapa) in [("Por categoria", &por_categoria), ("Por mês", &por_mes)] {
        println!("\n--- {titulo} ---");
        println!("  {:<16} {:>6} {:>16} {:>7}", "", "itens", "total", "parte");
        for (chave, (qtd, total)) in mapa { linha_tabela(chave, *qtd, *total, faturamento); }
    }
    println!("\n--- Produtos mais vendidos (por itens) ---");
    let mut ranking: Vec<(&str, u32, f64)> = por_produto.iter().map(|(p, (q, t))| (*p, *q, *t)).collect();
    ranking.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (n, (produto, qtd, total)) in ranking.iter().take(top).enumerate() {
        println!("  {}. {:<14} {:>6} {:>16}", n + 1, produto, qtd, moeda(*total));
    }
    // max_by recebe uma closure de comparação porque f64 não implementa Ord.
    if let Some((produto, _, total)) = ranking.iter().max_by(|a, b| a.2.total_cmp(&b.2)) {
        println!("\n  Maior faturamento: {produto} com {}", moeda(*total));
    }
}

fn executar(args: &[String]) -> Result<(), String> {
    let mut caminho = None;
    let mut top = 5;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--top" => {
                let n = iter.next().ok_or("--top precisa de um número")?;
                top = n.parse().ok().filter(|t| *t > 0).ok_or(format!("--top inválido: '{n}'"))?;
            }
            outro if outro.starts_with("--") => return Err(format!("opção desconhecida: {outro}\n{USO}")),
            arquivo if caminho.is_none() => caminho = Some(arquivo),
            extra => return Err(format!("argumento inesperado: '{extra}'")),
        }
    }
    let caminho = caminho.ok_or(format!("informe o arquivo CSV\n{USO}"))?;
    let vendas = ler_vendas(Path::new(caminho))?;
    if vendas.is_empty() { return Err("o arquivo não tem nenhuma venda".into()); }
    imprimir_relatorio(&vendas, top);
    Ok(())
}

// Gerador congruente linear: números pseudoaleatórios previsíveis, sem depender de crates.
struct Sorteio(u64);
impl Sorteio {
    fn proximo(&mut self, limite: u64) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) % limite
    }
}

fn demo() -> Result<(), String> {
    let catalogo = [
        ("Caneca", "Casa", 29.90), ("Luminária", "Casa", 119.00), ("Camiseta", "Roupas", 59.90),
        ("Moletom", "Roupas", 149.90), ("Fone", "Eletrônicos", 199.00), ("Teclado", "Eletrônicos", 259.00),
        ("Caderno", "Papelaria", 24.50), ("Caneta", "Papelaria", 7.90),
    ];
    let mut sorteio = Sorteio(2026);
    let mut csv = String::from("data,produto,categoria,quantidade,valor\n");
    for _ in 0..60 {
        let (produto, categoria, valor) = catalogo[sorteio.proximo(catalogo.len() as u64) as usize];
        let (mes, dia, qtd) = (8 + sorteio.proximo(3), 1 + sorteio.proximo(28), 1 + sorteio.proximo(4));
        csv.push_str(&format!("2026-{mes:02}-{dia:02},{produto},{categoria},{qtd},{valor:.2}\n"));
    }
    let arquivo = std::env::temp_dir().join(format!("codenomad_vendas_{}.csv", std::process::id()));
    fs::write(&arquivo, &csv).map_err(|e| format!("não consegui gravar o CSV temporário: {e}"))?;
    println!("=== Demonstração: relatório de vendas ===");
    println!("CSV com 60 vendas gerado em {}", arquivo.display());
    println!("Primeiras linhas:");
    for linha in csv.lines().take(4) { println!("  | {linha}"); }
    println!();
    executar(&[arquivo.to_string_lossy().to_string(), "--top".into(), "3".into()])?;
    let _ = fs::remove_file(&arquivo);
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None => { println!("{USO}"); Ok(()) }
        Some("--demo") => demo(),
        Some(_) => executar(&args),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); std::process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Aceite --inicio AAAA-MM-DD e --fim AAAA-MM-DD para filtrar o período antes de calcular.
// 2. Grave o relatório também em CSV (categoria,itens,total) com a opção --destino.
// 3. Desenhe um gráfico de barras em texto para o faturamento por mês, proporcional ao maior valor.
// ------------------------------------------------------------
