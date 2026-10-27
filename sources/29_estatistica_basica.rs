// ============================================================
// CodeNomad Academy  |  Sources Rust  |  29. Estatística descritiva
// ------------------------------------------------------------
// O que faz: para uma lista de números (argumentos ou arquivo texto) calcula
//            média, mediana, moda, variância, desvio padrão, mínimo, máximo,
//            amplitude e quartis, e desenha um histograma em texto.
// Como usar: rustc -O --edition 2021 -D warnings 29_estatistica_basica.rs -o stats.exe
//            stats.exe 7.5 8 6.5 9 7 8.5 10 6
//            stats.exe --arquivo notas.txt      (um número por linha ou separados por espaço)
//            stats.exe --demo
// O que você aprende: iteradores com sum, fold, max_by, partial_cmp para
//            ordenar f64, HashMap para contar frequências, leitura de arquivo
//            com fs::read_to_string e formatação com largura e precisão.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::collections::HashMap;
use std::env;
use std::fs;
use std::process;

struct Resumo {
    quantidade: usize,
    media: f64,
    mediana: f64,
    modas: Vec<f64>,
    variancia: f64,
    desvio: f64,
    minimo: f64,
    maximo: f64,
    q1: f64,
    q3: f64,
}

/// Ordena um Vec<f64>. f64 não implementa Ord (por causa do NaN), então
/// usamos partial_cmp e tratamos o caso impossível como igual.
fn ordenar(valores: &[f64]) -> Vec<f64> {
    let mut v = valores.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v
}

/// Percentil por interpolação linear num vetor já ordenado (p entre 0 e 1).
fn percentil(ordenado: &[f64], p: f64) -> f64 {
    let pos = p * (ordenado.len() - 1) as f64;
    let i = pos.floor() as usize;
    let fracao = pos - i as f64;
    if i + 1 < ordenado.len() {
        ordenado[i] + fracao * (ordenado[i + 1] - ordenado[i])
    } else {
        ordenado[i]
    }
}

/// Moda: valores com a maior frequência. Pode haver mais de uma, ou nenhuma
/// se todos aparecem uma vez só. A chave do HashMap é o f64 em bits porque
/// f64 não implementa Hash nem Eq.
fn modas(valores: &[f64]) -> Vec<f64> {
    let mut contagem: HashMap<u64, usize> = HashMap::new();
    for v in valores {
        *contagem.entry(v.to_bits()).or_insert(0) += 1;
    }
    let maior = contagem.values().copied().max().unwrap_or(0);
    if maior <= 1 {
        return Vec::new();
    }
    let lista: Vec<f64> = contagem
        .iter()
        .filter(|(_, &c)| c == maior)
        .map(|(&bits, _)| f64::from_bits(bits))
        .collect();
    ordenar(&lista)
}

fn resumir(valores: &[f64]) -> Result<Resumo, String> {
    if valores.is_empty() {
        return Err("preciso de pelo menos um número".to_string());
    }
    let n = valores.len() as f64;
    let ordenado = ordenar(valores);
    let media = valores.iter().sum::<f64>() / n;
    // Variância amostral (divide por n-1); com um único valor fica zero.
    let variancia = if valores.len() > 1 {
        valores.iter().map(|v| (v - media).powi(2)).sum::<f64>() / (n - 1.0)
    } else {
        0.0
    };
    Ok(Resumo {
        quantidade: valores.len(),
        media,
        mediana: percentil(&ordenado, 0.5),
        modas: modas(valores),
        variancia,
        desvio: variancia.sqrt(),
        minimo: ordenado[0],
        maximo: ordenado[ordenado.len() - 1],
        q1: percentil(&ordenado, 0.25),
        q3: percentil(&ordenado, 0.75),
    })
}

fn imprimir_resumo(r: &Resumo) {
    println!("Quantidade:     {}", r.quantidade);
    println!("Média:          {:.3}", r.media);
    println!("Mediana:        {:.3}", r.mediana);
    let moda = if r.modas.is_empty() {
        "nenhuma (todos os valores aparecem uma vez)".to_string()
    } else {
        r.modas.iter().map(|m| format!("{}", m)).collect::<Vec<_>>().join(", ")
    };
    println!("Moda:           {}", moda);
    println!("Variância:      {:.3}", r.variancia);
    println!("Desvio padrão:  {:.3}", r.desvio);
    println!("Mínimo:         {}", r.minimo);
    println!("Máximo:         {}", r.maximo);
    println!("Amplitude:      {}", r.maximo - r.minimo);
    println!("Q1 / Q2 / Q3:   {:.3} / {:.3} / {:.3}", r.q1, r.mediana, r.q3);
}

/// Histograma em texto: divide [mínimo, máximo] em faixas de mesma largura
/// e desenha uma barra proporcional à contagem de cada faixa.
fn histograma(valores: &[f64], faixas: usize, largura_barra: usize) {
    let r = match resumir(valores) {
        Ok(r) => r,
        Err(_) => return,
    };
    let amplitude = r.maximo - r.minimo;
    let passo = if amplitude == 0.0 { 1.0 } else { amplitude / faixas as f64 };
    let mut contagens = vec![0usize; faixas];
    for v in valores {
        let mut i = ((v - r.minimo) / passo) as usize;
        if i >= faixas {
            i = faixas - 1; // o máximo cai na última faixa
        }
        contagens[i] += 1;
    }
    let maior = contagens.iter().copied().max().unwrap_or(1).max(1);
    println!("Histograma ({} faixas):", faixas);
    for (i, c) in contagens.iter().enumerate() {
        let inicio = r.minimo + i as f64 * passo;
        let fim = inicio + passo;
        let tamanho = c * largura_barra / maior;
        println!("    [{:>7.2}, {:>7.2}) {:<w$} {}", inicio, fim, "#".repeat(tamanho), c, w = largura_barra);
    }
}

fn ler_numeros(textos: &[String]) -> Result<Vec<f64>, String> {
    textos
        .iter()
        .flat_map(|t| t.split_whitespace())
        .map(|t| {
            t.replace(',', ".")
                .parse::<f64>()
                .map_err(|_| format!("'{}' não é um número", t))
        })
        .collect()
}

fn ler_arquivo(caminho: &str) -> Result<Vec<f64>, String> {
    let texto = fs::read_to_string(caminho).map_err(|e| format!("não consegui ler '{}': {}", caminho, e))?;
    let linhas: Vec<String> = texto.lines().map(|l| l.to_string()).collect();
    ler_numeros(&linhas)
}

fn analisar(valores: &[f64]) -> Result<(), String> {
    let r = resumir(valores)?;
    imprimir_resumo(&r);
    println!();
    histograma(valores, 6, 30);
    Ok(())
}

fn demo() {
    // Notas de uma turma fictícia de 40 alunos, de 0 a 10.
    let amostra = [
        7.5, 8.0, 6.5, 9.0, 7.0, 8.5, 10.0, 6.0, 7.5, 5.5, 8.0, 7.0, 9.5, 6.5, 7.5, 8.0, 4.0, 7.0, 8.5, 9.0,
        7.5, 6.0, 8.0, 7.0, 5.0, 8.5, 7.5, 9.0, 6.5, 8.0, 7.0, 3.5, 8.0, 7.5, 9.5, 6.0, 7.0, 8.5, 7.5, 8.0,
    ];
    println!("=== Notas de 40 alunos ===");
    println!("Amostra: {:?}\n", &amostra[..10]);
    if let Err(e) = analisar(&amostra) {
        println!("Erro: {}", e);
    }
}

fn uso() {
    println!("Uso: estatistica_basica <n1> <n2> <n3> ...");
    println!("     estatistica_basica --arquivo <caminho>   (números separados por espaço ou linha)");
    println!("     estatistica_basica --demo");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        uso();
        return;
    }
    if args.iter().any(|a| a == "--demo") {
        demo();
        return;
    }
    let valores = if args[0] == "--arquivo" {
        match args.get(1) {
            Some(caminho) => ler_arquivo(caminho),
            None => Err("informe o caminho depois de --arquivo".to_string()),
        }
    } else {
        ler_numeros(&args)
    };
    let resultado = valores.and_then(|v| analisar(&v));
    if let Err(e) = resultado {
        eprintln!("Erro: {}", e);
        process::exit(1);
    }
}

// Para ir além:
//   1. Calcule a assimetria (skewness) e a curtose para dizer se a
//      distribuição é simétrica e se tem caudas pesadas.
//   2. Detecte outliers pela regra do IQR (abaixo de Q1 - 1,5 IQR ou acima
//      de Q3 + 1,5 IQR) e marque-os na saída.
//   3. Aceite um CSV com várias colunas e um argumento --coluna para
//      escolher qual analisar.
