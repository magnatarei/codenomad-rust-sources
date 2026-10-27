// ============================================================
// CodeNomad Academy  |  Sources Rust  |  44. Cronômetro Pomodoro no terminal
// ------------------------------------------------------------
// O que faz: alterna ciclos de foco e pausa (com pausa longa a cada quatro
//            focos), mostra a contagem regressiva na mesma linha do terminal,
//            avisa com um sinal sonoro ao fim de cada ciclo e imprime um
//            resumo do que foi feito.
// Como usar: 44_cronometro_pomodoro.exe --foco 25 --pausa 5 --ciclos 4
//            44_cronometro_pomodoro.exe --foco 3 --pausa 1 --ciclos 2 --em-segundos
//            44_cronometro_pomodoro.exe --demo
// O que você aprende: Duration e Instant, thread::sleep, print! com \r e
//            flush para reescrever a mesma linha, enum com métodos, struct
//            de configuração com valores padrão (Default), e um laço de
//            análise de argumentos que valida cada número.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::{env, process, thread};

#[derive(Clone, Copy, PartialEq)]
enum Fase { Foco, Pausa, PausaLonga }

impl Fase {
    fn nome(self) -> &'static str {
        match self { Fase::Foco => "Foco", Fase::Pausa => "Pausa", Fase::PausaLonga => "Pausa longa" }
    }
    fn aviso(self) -> &'static str {
        match self {
            Fase::Foco => "Foco concluído. Levante, beba água, descanse os olhos.",
            Fase::Pausa => "Pausa terminada. De volta ao foco!",
            Fase::PausaLonga => "Pausa longa terminada. Pronto para mais uma rodada?",
        }
    }
}

/// Configuração do cronômetro. Default dá os valores clássicos: 25/5/15, 4 ciclos.
struct Config { foco: Duration, pausa: Duration, pausa_longa: Duration, ciclos: u32, tique: Duration }

impl Default for Config {
    fn default() -> Self {
        let min = |m: u64| Duration::from_secs(m * 60);
        Config { foco: min(25), pausa: min(5), pausa_longa: min(15), ciclos: 4, tique: Duration::from_secs(1) }
    }
}

impl Config {
    fn duracao(&self, fase: Fase) -> Duration {
        match fase { Fase::Foco => self.foco, Fase::Pausa => self.pausa, Fase::PausaLonga => self.pausa_longa }
    }
}

/// Converte segundos em "mm:ss" (ou "h:mm:ss" quando passa de uma hora)
fn mmss(segundos: u64) -> String {
    let (h, m, s) = (segundos / 3600, segundos % 3600 / 60, segundos % 60);
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m:02}:{s:02}") }
}

/// Faz a contagem regressiva de uma fase, reescrevendo a mesma linha a cada tique
fn contar(fase: Fase, duracao: Duration, tique: Duration, rotulo: &str) {
    let inicio = Instant::now();
    let mut saida = io::stdout();
    loop {
        let passou = inicio.elapsed();
        let resta = duracao.saturating_sub(passou);
        // \r volta ao início da linha sem pular; flush obriga o terminal a mostrar já
        print!("\r  [{rotulo}] {:<12} restam {}   ", fase.nome(), mmss(resta.as_secs()));
        let _ = saida.flush();
        if resta.is_zero() { break; }
        // Dorme só até o próximo tique, para o relógio não "atrasar" com o acúmulo
        thread::sleep(resta.min(tique));
    }
    // \x07 é o sinal sonoro do terminal; alguns terminais piscam em vez de apitar
    println!("\x07\n  >> {}", fase.aviso());
}

/// Roda a sessão inteira e devolve o tempo total gasto em cada tipo de fase
fn sessao(cfg: &Config) -> Vec<(Fase, Duration)> {
    let mut registro = Vec::new();
    for ciclo in 1..=cfg.ciclos {
        let rotulo = format!("{ciclo}/{}", cfg.ciclos);
        contar(Fase::Foco, cfg.foco, cfg.tique, &rotulo);
        registro.push((Fase::Foco, cfg.foco));
        if ciclo == cfg.ciclos { break; } // depois do último foco não há pausa
        let pausa = if ciclo % 4 == 0 { Fase::PausaLonga } else { Fase::Pausa };
        contar(pausa, cfg.duracao(pausa), cfg.tique, &rotulo);
        registro.push((pausa, cfg.duracao(pausa)));
    }
    registro
}

fn resumo(registro: &[(Fase, Duration)], decorrido: Duration) {
    println!("\n=== Resumo da sessão ===");
    for fase in [Fase::Foco, Fase::Pausa, Fase::PausaLonga] {
        let (vezes, total) = registro.iter().filter(|(f, _)| *f == fase)
            .fold((0, Duration::ZERO), |(n, t), (_, d)| (n + 1, t + *d));
        if vezes > 0 { println!("  {:<12} {vezes} vez(es), {} no total", fase.nome(), mmss(total.as_secs())); }
    }
    println!("  Tempo real decorrido: {}", mmss(decorrido.as_secs()));
}

/// Lê "--foco 25" e companhia. O número tem de ser inteiro e maior que zero.
fn configurar(args: &[String]) -> Result<Config, String> {
    let mut cfg = Config::default();
    let em_segundos = args.iter().any(|a| a == "--em-segundos");
    let mut it = args.iter().filter(|a| *a != "--em-segundos");
    let mut numeros = Vec::new(); // guardamos (nome, valor) e convertemos no fim, quando já sabemos a unidade
    while let Some(opcao) = it.next() {
        let valor = it.next().ok_or_else(|| format!("{opcao} precisa de um número logo depois"))?;
        let n: u64 = valor.parse().ok().filter(|n| *n > 0).ok_or_else(|| format!("{opcao} espera um inteiro maior que zero, recebi \"{valor}\""))?;
        numeros.push((opcao.as_str(), n));
    }
    let tempo = |n: u64| if em_segundos { Duration::from_secs(n) } else { Duration::from_secs(n * 60) };
    for (nome, n) in numeros {
        match nome {
            "--foco" => cfg.foco = tempo(n),
            "--pausa" => cfg.pausa = tempo(n),
            "--longa" => cfg.pausa_longa = tempo(n),
            "--ciclos" => cfg.ciclos = u32::try_from(n).map_err(|_| "ciclos demais")?.min(100),
            outro => return Err(format!("opção desconhecida: {outro}")),
        }
    }
    Ok(cfg)
}

fn executar(args: &[String]) -> Result<(), String> {
    let cfg = configurar(args)?;
    println!("=== Pomodoro: {} ciclo(s) de foco de {}, pausa de {}, pausa longa de {} ===",
        cfg.ciclos, mmss(cfg.foco.as_secs()), mmss(cfg.pausa.as_secs()), mmss(cfg.pausa_longa.as_secs()));
    println!("  (Ctrl+C encerra a qualquer momento)");
    let inicio = Instant::now();
    let registro = sessao(&cfg);
    resumo(&registro, inicio.elapsed());
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Pomodoro: demonstração com ciclos de 2 segundos ===");
    // Dois ciclos curtos: foco, pausa, foco. Tique de meio segundo deixa a contagem visível.
    let cfg = Config { foco: Duration::from_secs(2), pausa: Duration::from_secs(2), pausa_longa: Duration::from_secs(3), ciclos: 2, tique: Duration::from_millis(500) };
    println!("  Configuração: foco {} s, pausa {} s, {} ciclos", cfg.foco.as_secs(), cfg.pausa.as_secs(), cfg.ciclos);
    println!("  Em uso real, cada \"restam mm:ss\" é reescrito na mesma linha a cada segundo.\n");
    let inicio = Instant::now();
    let registro = sessao(&cfg);
    resumo(&registro, inicio.elapsed());
    println!("\nFormatação de tempos: {} | {} | {}", mmss(59), mmss(1500), mmss(3725));
    println!("Erro tratado de propósito: {:?}", configurar(&["--foco".into(), "zero".into()]).err());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None | Some("--ajuda") | Some("-h") => {
            println!("Uso: 44_cronometro_pomodoro.exe [--foco 25] [--pausa 5] [--longa 15] [--ciclos 4] [--em-segundos]");
            println!("     44_cronometro_pomodoro.exe --demo");
            return;
        }
        Some("--demo") => demo(),
        _ => executar(&args),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Grave cada sessão concluída num arquivo pomodoro.log com data, ciclos
//    e tempo total, e mostre o histórico com --historico.
// 2. Aceite um rótulo de tarefa ("--tarefa 'Estudar ownership'") e mostre-o
//    na linha da contagem; no resumo, agrupe os focos por tarefa.
// 3. Desenhe uma barra de progresso ao lado da contagem, com 20 blocos que
//    vão se preenchendo conforme o tempo passa.
// ------------------------------------------------------------
