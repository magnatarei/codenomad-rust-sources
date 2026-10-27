// ============================================================
// CodeNomad Academy  |  Sources Rust  |  22. Busca binária e busca linear
// ------------------------------------------------------------
// O que faz: implementa busca linear e busca binária do zero, contando os
//            passos de cada uma em vetores de 10, 1.000 e 100.000 posições,
//            e mostra por que a binária só funciona em vetor ordenado.
// Como usar: rustc -O --edition 2021 -D warnings 22_busca_binaria.rs -o busca.exe
//            busca.exe 37 5 42 8 29 37 13 10      (alvo primeiro, depois o vetor)
//            busca.exe --demo
// O que você aprende: slices, Option<usize> como resultado de busca, match
//            com std::cmp::Ordering, enumerate, intervalo meio aberto com
//            usize, e a binary_search que a biblioteca padrão já oferece.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::cmp::Ordering;

/// O que uma busca devolve: onde achou (ou None) e quantos passos gastou.
struct Resultado {
    posicao: Option<usize>,
    passos: usize,
}

impl Resultado {
    fn descrever(&self) -> String {
        match self.posicao {
            Some(i) => format!("achou na posição {i} em {} passo(s)", self.passos),
            None => format!("não achou depois de {} passo(s)", self.passos),
        }
    }

    /// Versão curta para caber na tabela comparativa.
    fn curto(&self) -> String {
        let achou = if self.posicao.is_some() { "achou" } else { "nada" };
        format!("{} passos, {achou}", self.passos)
    }
}

/// Busca linear: olha um por um, do começo ao fim. Funciona em qualquer
/// vetor, ordenado ou não, mas no pior caso visita todas as posições.
fn busca_linear(v: &[i64], alvo: i64) -> Resultado {
    let mut passos = 0;
    for (i, &x) in v.iter().enumerate() {
        passos += 1;
        if x == alvo {
            return Resultado { posicao: Some(i), passos };
        }
    }
    Resultado { posicao: None, passos }
}

/// Busca binária: exige vetor ORDENADO. Olha o meio; se o alvo for menor,
/// descarta a metade direita; se for maior, descarta a esquerda. Cada passo
/// corta o problema pela metade, por isso gasta no máximo log2(n) + 1 passos.
fn busca_binaria(v: &[i64], alvo: i64, mostrar: bool) -> Resultado {
    // Intervalo meio aberto [baixo, alto): evita o erro clássico de
    // "alto - 1" virar negativo quando o tipo é usize.
    let (mut baixo, mut alto) = (0, v.len());
    let mut passos = 0;
    while baixo < alto {
        let meio = baixo + (alto - baixo) / 2;
        passos += 1;
        if mostrar {
            println!("    passo {passos}: intervalo [{baixo}, {alto}), meio = {meio}, valor = {}", v[meio]);
        }
        match v[meio].cmp(&alvo) {
            Ordering::Equal => return Resultado { posicao: Some(meio), passos },
            Ordering::Less => baixo = meio + 1,
            Ordering::Greater => alto = meio,
        }
    }
    Resultado { posicao: None, passos }
}

/// Teto teórico de passos da binária para n elementos: floor(log2 n) + 1,
/// que é exatamente a quantidade de bits necessária para escrever n.
fn limite_teorico(n: usize) -> u32 {
    usize::BITS - n.leading_zeros()
}

fn uso() {
    println!("Uso: busca <alvo> <números inteiros separados por espaço>");
    println!("     busca --demo");
    println!("Exemplo: busca 37 5 42 8 29 37 13 10");
}

fn executar(args: &[String]) -> Result<(), String> {
    let mut numeros = Vec::with_capacity(args.len());
    for arg in args {
        let n: i64 = arg
            .parse()
            .map_err(|_| format!("valor inválido: '{arg}' (esperava um número inteiro)"))?;
        numeros.push(n);
    }
    let (alvo, vetor) = match numeros.as_slice() {
        [alvo, resto @ ..] if !resto.is_empty() => (*alvo, resto),
        _ => return Err("informe o alvo e pelo menos um número para o vetor".to_string()),
    };
    println!("Alvo: {alvo}");
    println!("Vetor como veio: {vetor:?}");
    println!("Linear: {}", busca_linear(vetor, alvo).descrever());
    let mut ordenado = vetor.to_vec();
    ordenado.sort_unstable();
    println!("\nVetor ordenado: {ordenado:?} (a posição muda, por isso a binária só vale no ordenado)");
    println!("Binária:");
    println!("    {}", busca_binaria(&ordenado, alvo, true).descrever());
    println!("Pela biblioteca padrão: ordenado.binary_search(&{alvo}) = {:?}", ordenado.binary_search(&alvo));
    println!("Teto teórico para {} elementos: {} passos", ordenado.len(), limite_teorico(ordenado.len()));
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== CodeNomad Academy: busca linear contra busca binária ===");
    println!("\nVetores ordenados com múltiplos de 3. Alvo presente (último) e alvo ausente.");
    println!("{:>9} {:>9} {:>21} {:>16} {:>10}", "tamanho", "alvo", "linear", "binária", "log2(n)+1");
    for tamanho in [10usize, 1_000, 100_000] {
        let v: Vec<i64> = (0..tamanho as i64).map(|i| i * 3).collect();
        let ultimo = v[tamanho - 1];
        for alvo in [ultimo, ultimo + 1] {
            let linear = busca_linear(&v, alvo);
            let binaria = busca_binaria(&v, alvo, false);
            println!(
                "{tamanho:>9} {alvo:>9} {:>21} {:>16} {:>10}",
                linear.curto(),
                binaria.curto(),
                limite_teorico(tamanho)
            );
        }
    }
    println!("\nDe 10 para 100.000 elementos a linear ficou 10.000 vezes mais cara; a binária, 4 vezes.");

    let v: Vec<i64> = (0..10).map(|i| i * 3).collect();
    println!("\n--- Binária passo a passo em {v:?}, procurando 21 ---");
    println!("    {}", busca_binaria(&v, 21, true).descrever());

    let bagunca = [40, 7, 25, 2, 99, 13];
    println!("\n--- Vetor NÃO ordenado: {bagunca:?}, procurando 7 ---");
    println!("Linear:  {}", busca_linear(&bagunca, 7).descrever());
    println!("Binária:");
    println!("    {}", busca_binaria(&bagunca, 7, true).descrever());
    println!("A binária confiou na ordem, descartou a metade errada no primeiro passo e perdeu o 7.");
    let mut ordenado = bagunca.to_vec();
    ordenado.sort_unstable();
    println!("Depois de ordenar {ordenado:?}: {}", busca_binaria(&ordenado, 7, false).descrever());
    println!("Biblioteca padrão: ordenado.binary_search(&7) = {:?}", ordenado.binary_search(&7));
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None => {
            uso();
            Ok(())
        }
        Some("--demo") => demo(),
        Some(_) => executar(&args),
    };
    if let Err(mensagem) = resultado {
        eprintln!("Erro: {mensagem}");
        std::process::exit(1);
    }
}

// ------------------------------------------------------------
// Para ir além:
// 1. Faça a busca binária devolver, quando não acha, a posição em que o alvo
//    deveria ser inserido para manter a ordem (como a binary_search faz).
// 2. Escreva a versão recursiva da busca binária e compare os passos.
// 3. Adapte as duas buscas para um Vec<String> de nomes em ordem alfabética.
// ------------------------------------------------------------
