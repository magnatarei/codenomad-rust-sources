// ============================================================
// CodeNomad Academy  |  Sources Rust  |  21. Algoritmos de ordenação com passos
// ------------------------------------------------------------
// O que faz: implementa bubble sort, insertion sort, merge sort e quick sort
//            do zero, contando comparações e trocas. Pode mostrar o vetor a
//            cada passo e fecha com uma tabela comparando os quatro.
// Como usar: rustc -O --edition 2021 -D warnings 21_ordenacao_visual.rs -o ordenar.exe
//            ordenar.exe 29 10 14 37 13 5 42 8
//            ordenar.exe 29 10 14 37 13 5 42 8 --passos
//            ordenar.exe --demo
// O que você aprende: funções que recebem &mut [i64], recursão com
//            split_at_mut, ponteiros de função em arrays, structs contadoras,
//            iteradores e medição de tempo com Instant.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::time::Instant;

/// Contadores compartilhados pelos quatro algoritmos. Guardar tudo em uma
/// struct evita passar três variáveis soltas de função em função.
#[derive(Default)]
struct Relato {
    comparacoes: u64,
    trocas: u64,
    mostrar_passos: bool,
}

impl Relato {
    fn novo(mostrar_passos: bool) -> Self {
        Relato { mostrar_passos, ..Default::default() }
    }

    /// Imprime o vetor inteiro quando o modo passo a passo está ligado.
    fn passo(&self, rotulo: &str, v: &[i64]) {
        if self.mostrar_passos {
            println!("    {rotulo:<14} {}", formatar(v));
        }
    }
}

fn formatar(v: &[i64]) -> String {
    let partes: Vec<String> = v.iter().map(|x| x.to_string()).collect();
    format!("[{}]", partes.join(" "))
}

/// Bubble sort: a cada passagem o maior elemento "borbulha" até o fim.
/// Se uma passagem inteira não trocou nada, o vetor já está ordenado.
fn bubble_sort(v: &mut [i64], r: &mut Relato) {
    let n = v.len();
    for passagem in 1..n {
        let mut trocou = false;
        for i in 0..n - passagem {
            r.comparacoes += 1;
            if v[i] > v[i + 1] {
                v.swap(i, i + 1);
                r.trocas += 1;
                trocou = true;
            }
        }
        r.passo(&format!("passagem {passagem}"), v);
        if !trocou {
            break;
        }
    }
}

/// Insertion sort: cada elemento caminha para trás até achar o seu lugar,
/// como quem organiza cartas na mão.
fn insertion_sort(v: &mut [i64], r: &mut Relato) {
    for i in 1..v.len() {
        let atual = v[i];
        let mut j = i;
        while j > 0 {
            r.comparacoes += 1;
            if v[j - 1] <= v[j] {
                break;
            }
            v.swap(j - 1, j);
            r.trocas += 1;
            j -= 1;
        }
        r.passo(&format!("inseriu {atual}"), v);
    }
}

/// Merge sort: divide ao meio, ordena cada metade (recursão) e intercala as
/// duas. Custa sempre n log n, mas usa um vetor auxiliar para intercalar.
fn merge_sort(v: &mut [i64], r: &mut Relato) {
    if v.len() < 2 {
        return;
    }
    let meio = v.len() / 2;
    {
        // split_at_mut empresta as duas metades ao mesmo tempo, sem copiar.
        let (esq, dir) = v.split_at_mut(meio);
        merge_sort(esq, r);
        merge_sort(dir, r);
    }
    let mut mesclado = Vec::with_capacity(v.len());
    let (mut i, mut j) = (0, meio);
    while i < meio && j < v.len() {
        r.comparacoes += 1;
        if v[i] <= v[j] {
            mesclado.push(v[i]);
            i += 1;
        } else {
            mesclado.push(v[j]);
            j += 1;
        }
    }
    mesclado.extend_from_slice(&v[i..meio]);
    mesclado.extend_from_slice(&v[j..]);
    // No merge sort não há troca: cada elemento copiado de volta conta como movimento.
    r.trocas += mesclado.len() as u64;
    v.copy_from_slice(&mesclado);
    r.passo("intercalou", v);
}

/// Quick sort (partição de Lomuto): o último elemento vira pivô, os menores
/// vão para a esquerda dele e a função se repete em cada lado. Rápido na
/// média, mas cai para n ao quadrado quando o pivô é sempre o pior possível.
fn quick_sort(v: &mut [i64], r: &mut Relato) {
    if v.len() < 2 {
        return;
    }
    let ultimo = v.len() - 1;
    let pivo = v[ultimo];
    let mut fronteira = 0;
    for i in 0..ultimo {
        r.comparacoes += 1;
        if v[i] < pivo {
            v.swap(i, fronteira);
            r.trocas += 1;
            fronteira += 1;
        }
    }
    v.swap(fronteira, ultimo);
    r.trocas += 1;
    r.passo(&format!("pivô {pivo}"), v);
    let (esq, dir) = v.split_at_mut(fronteira);
    quick_sort(esq, r);
    quick_sort(&mut dir[1..], r);
}

/// Um ponteiro de função por algoritmo: a tabela comparativa vira um laço.
type Algoritmo = fn(&mut [i64], &mut Relato);

const ALGORITMOS: [(&str, Algoritmo); 4] = [
    ("bubble sort", bubble_sort),
    ("insertion sort", insertion_sort),
    ("merge sort", merge_sort),
    ("quick sort", quick_sort),
];

/// Roda os quatro algoritmos sobre cópias do mesmo vetor e imprime a tabela.
fn comparar(original: &[i64]) {
    let mut esperado = original.to_vec();
    esperado.sort_unstable();
    println!("{:<16} {:>12} {:>12} {:>12}  confere", "algoritmo", "comparações", "trocas", "tempo");
    for (nome, algoritmo) in ALGORITMOS {
        let mut copia = original.to_vec();
        let mut r = Relato::novo(false);
        let inicio = Instant::now();
        algoritmo(&mut copia, &mut r);
        let tempo = format!("{:.1?}", inicio.elapsed());
        // Conferência honesta: o resultado tem de bater com a ordenação da biblioteca.
        let confere = if copia == esperado { "sim" } else { "NÃO" };
        println!("{nome:<16} {:>12} {:>12} {tempo:>12}  {confere}", r.comparacoes, r.trocas);
    }
}

/// Gerador linear congruente: números bagunçados o bastante para testar
/// ordenação sem depender de nenhuma crate externa.
struct Lcg(u64);

impl Lcg {
    fn proximo(&mut self, limite: u64) -> i64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) % limite) as i64
    }
}

fn uso() {
    println!("Uso: ordenar <números inteiros separados por espaço> [--passos]");
    println!("     ordenar --demo");
    println!("Exemplo: ordenar 29 10 14 37 13 5 42 8 --passos");
}

fn executar(args: &[String]) -> Result<(), String> {
    let mostrar_passos = args.iter().any(|a| a == "--passos");
    let mut vetor = Vec::new();
    for arg in args.iter().filter(|a| a.as_str() != "--passos") {
        let n: i64 = arg
            .parse()
            .map_err(|_| format!("valor inválido: '{arg}' (esperava um número inteiro)"))?;
        vetor.push(n);
    }
    if vetor.is_empty() {
        return Err("informe pelo menos um número para ordenar".to_string());
    }
    println!("Vetor original: {}", formatar(&vetor));
    if mostrar_passos {
        for (nome, algoritmo) in ALGORITMOS {
            println!("\n--- {nome} passo a passo ---");
            let mut copia = vetor.clone();
            let mut r = Relato::novo(true);
            algoritmo(&mut copia, &mut r);
            println!("    resultado: {} ({} comparações, {} trocas)", formatar(&copia), r.comparacoes, r.trocas);
        }
    }
    println!();
    comparar(&vetor);
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== CodeNomad Academy: quatro algoritmos de ordenação ===");
    let vetor = vec![29, 10, 14, 37, 13, 5, 42, 8];
    println!("Vetor original: {}", formatar(&vetor));
    println!("\n--- Bubble sort, uma linha por passagem ---");
    let mut copia = vetor.clone();
    let mut r = Relato::novo(true);
    bubble_sort(&mut copia, &mut r);
    println!("    resultado: {} ({} comparações, {} trocas)", formatar(&copia), r.comparacoes, r.trocas);
    println!("\n--- Tabela comparativa com os 8 números ---");
    comparar(&vetor);
    let mut gerador = Lcg(42);
    let grande: Vec<i64> = (0..2000).map(|_| gerador.proximo(1000)).collect();
    println!("\n--- Tabela comparativa com 2.000 números pseudoaleatórios ---");
    comparar(&grande);
    println!("\nCom 8 itens tanto faz. Com 2.000, merge e quick fazem uma fração das comparações.");
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
// 1. Implemente o selection sort e inclua na tabela comparativa.
// 2. Faça o quick sort escolher o pivô pela mediana de três e meça a
//    diferença em um vetor que já chega ordenado.
// 3. Torne as funções genéricas (T: Ord + Copy) para ordenar também textos.
// ------------------------------------------------------------
