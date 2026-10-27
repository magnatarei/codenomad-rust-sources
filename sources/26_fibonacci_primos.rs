// ============================================================
// CodeNomad Academy  |  Sources Rust  |  26. Fibonacci, fatorial e primos
// ------------------------------------------------------------
// O que faz: calcula Fibonacci de três jeitos (recursivo puro, recursivo com
//            memória e iterativo) medindo o tempo de cada um, fatorial sem
//            estourar o inteiro, teste de primalidade e o crivo de Eratóstenes
//            até N. Explica no caminho por que a recursão pura explode.
// Como usar: rustc -O --edition 2021 -D warnings 26_fibonacci_primos.rs -o primos.exe
//            primos.exe fib 30
//            primos.exe fatorial 20
//            primos.exe primo 97
//            primos.exe crivo 100
//            primos.exe --demo
// O que você aprende: recursão, HashMap como memória (memoização),
//            checked_mul para detectar estouro, u128, Instant para medir
//            tempo, Vec<bool> como crivo e match em comandos.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::collections::HashMap;
use std::env;
use std::process;
use std::time::Instant;

/// Recursão pura: fib(n) chama fib(n-1) e fib(n-2), que repetem o mesmo
/// trabalho. O número de chamadas cresce como 2^n: fib(30) faz mais de
/// 2,6 milhões de chamadas, fib(50) levaria horas.
fn fib_recursivo(n: u32, chamadas: &mut u64) -> u64 {
    *chamadas += 1;
    if n < 2 {
        return n as u64;
    }
    fib_recursivo(n - 1, chamadas) + fib_recursivo(n - 2, chamadas)
}

/// Mesma recursão, mas guarda cada resultado num HashMap. Cada n é
/// calculado uma única vez, então o custo cai para O(n).
fn fib_memoria(n: u32, memo: &mut HashMap<u32, u64>, chamadas: &mut u64) -> u64 {
    *chamadas += 1;
    if n < 2 {
        return n as u64;
    }
    if let Some(&v) = memo.get(&n) {
        return v;
    }
    let v = fib_memoria(n - 1, memo, chamadas) + fib_memoria(n - 2, memo, chamadas);
    memo.insert(n, v);
    v
}

/// Iterativo: só precisa dos dois últimos valores. O mais rápido e o que
/// menos gasta memória. Devolve None se o u64 não aguenta (n > 93).
fn fib_iterativo(n: u32) -> Option<u64> {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        let proximo = a.checked_add(b)?;
        a = b;
        b = proximo;
    }
    Some(a)
}

/// Fatorial com u128 e checked_mul: em vez de dar a volta em silêncio, o
/// estouro vira None e o chamador decide o que fazer.
fn fatorial(n: u32) -> Option<u128> {
    (1..=n as u128).try_fold(1u128, |acc, k| acc.checked_mul(k))
}

/// Teste de primalidade por divisão até a raiz quadrada: se n tem um divisor
/// maior que sqrt(n), o par dele é menor, e já teria sido achado.
fn eh_primo(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n % 2 == 0 {
        return n == 2;
    }
    let mut d = 3;
    while d * d <= n {
        if n % d == 0 {
            return false;
        }
        d += 2;
    }
    true
}

/// Crivo de Eratóstenes: marca os múltiplos de cada primo como compostos.
/// Sobram só os primos. Custo O(n log log n), ótimo para listar muitos.
fn crivo(limite: usize) -> Vec<usize> {
    let mut composto = vec![false; limite + 1];
    let mut primos = Vec::new();
    for i in 2..=limite {
        if composto[i] {
            continue;
        }
        primos.push(i);
        let mut m = i * i;
        while m <= limite {
            composto[m] = true;
            m += i;
        }
    }
    primos
}

fn comparar_fibonacci(n: u32) {
    println!("Fibonacci de {} pelas três estratégias:", n);
    let inicio = Instant::now();
    let mut chamadas = 0u64;
    let r1 = fib_recursivo(n, &mut chamadas);
    println!("    recursivo puro   = {:<12} {:>10} chamadas  {:?}", r1, chamadas, inicio.elapsed());

    let inicio = Instant::now();
    let mut memo = HashMap::new();
    let mut chamadas = 0u64;
    let r2 = fib_memoria(n, &mut memo, &mut chamadas);
    println!("    com memória      = {:<12} {:>10} chamadas  {:?}", r2, chamadas, inicio.elapsed());

    let inicio = Instant::now();
    let r3 = fib_iterativo(n).map(|v| v.to_string()).unwrap_or_else(|| "estouro".into());
    println!("    iterativo        = {:<12} {:>10} laços     {:?}", r3, n, inicio.elapsed());
    println!("    Por que a recursão pura explode: cada chamada gera duas, então o");
    println!("    total dobra a cada n. A memória corta as repetições e cai para O(n).");
}

fn mostrar_fatorial(n: u32) {
    match fatorial(n) {
        Some(v) => println!("{}! = {}", n, v),
        None => println!("{}! não cabe em 128 bits (o maior que cabe é 34!)", n),
    }
}

fn mostrar_crivo(limite: usize) {
    let primos = crivo(limite);
    println!("Primos até {} ({} encontrados):", limite, primos.len());
    let linha: Vec<String> = primos.iter().map(|p| p.to_string()).collect();
    for pedaco in linha.chunks(15) {
        println!("    {}", pedaco.join(" "));
    }
}

fn ler_numero(texto: &str, maximo: u64) -> Result<u64, String> {
    let n: u64 = texto
        .parse()
        .map_err(|_| format!("'{}' não é um número inteiro sem sinal", texto))?;
    if n > maximo {
        return Err(format!("{} é grande demais; o máximo aqui é {}", n, maximo));
    }
    Ok(n)
}

fn demo() {
    println!("=== Fibonacci ===");
    comparar_fibonacci(30);
    println!("\n=== Fatorial ===");
    mostrar_fatorial(20);
    mostrar_fatorial(34);
    mostrar_fatorial(35);
    println!("\n=== Primalidade ===");
    for n in [1u64, 2, 17, 91, 97, 7919, 1_000_003] {
        println!("    {:>9} é primo? {}", n, if eh_primo(n) { "sim" } else { "não" });
    }
    println!("\n=== Crivo de Eratóstenes ===");
    mostrar_crivo(100);
}

fn uso() {
    println!("Uso: fibonacci_primos fib <n>        (n até 40, compara as três versões)");
    println!("     fibonacci_primos fatorial <n>   (n até 1000, avisa se estourar)");
    println!("     fibonacci_primos primo <n>      (testa se n é primo)");
    println!("     fibonacci_primos crivo <n>      (lista os primos até n, n até 10.000.000)");
    println!("     fibonacci_primos --demo");
}

fn executar(comando: &str, valor: &str) -> Result<(), String> {
    match comando {
        "fib" => comparar_fibonacci(ler_numero(valor, 40)? as u32),
        "fatorial" => mostrar_fatorial(ler_numero(valor, 1000)? as u32),
        "primo" => {
            let n = ler_numero(valor, u64::MAX)?;
            println!("{} é primo? {}", n, if eh_primo(n) { "sim" } else { "não" });
        }
        "crivo" => mostrar_crivo(ler_numero(valor, 10_000_000)? as usize),
        outro => return Err(format!("comando desconhecido: '{}'", outro)),
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|a| a == "--demo") {
        demo();
        return;
    }
    if args.len() != 2 {
        uso();
        return;
    }
    if let Err(e) = executar(&args[0], &args[1]) {
        eprintln!("Erro: {}", e);
        uso();
        process::exit(1);
    }
}

// Para ir além:
//   1. Implemente Fibonacci por exponenciação de matrizes, que calcula
//      fib(n) em O(log n), e compare com o iterativo para n = 90.
//   2. Faça o crivo segmentado: processe o intervalo em blocos para listar
//      primos até 100 milhões sem alocar um Vec<bool> gigante.
//   3. Troque o HashMap da memoização por um Vec<Option<u64>> e meça se a
//      versão com vetor fica mais rápida (dica: índice direto é mais barato
//      que calcular hash).
