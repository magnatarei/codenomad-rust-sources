// ============================================================
// CodeNomad Academy  |  Sources Rust  |  49. Pedra, papel e tesoura com placar
// ------------------------------------------------------------
// O que faz: melhor de N rodadas contra o computador, com placar a cada
//            rodada. O computador conta as jogadas que você mais usa e
//            passa a jogar o que vence a sua preferida; na primeira rodada
//            ele sorteia. No --demo uma série de 7 rodadas é simulada.
// Como usar: 49_pedra_papel_tesoura.exe --rodadas 5
//            (digite pedra, papel ou tesoura a cada rodada)
//            49_pedra_papel_tesoura.exe --demo
// O que você aprende: enum com FromStr e Display, método que decide o
//            vencedor com match em tuplas, HashMap para contar frequências,
//            max_by_key com desempate, struct de placar, leitura de stdin
//            com laço de validação e um gerador xorshift simples.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::collections::HashMap;
use std::fmt;
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, process};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Jogada { Pedra, Papel, Tesoura }

impl Jogada {
    const TODAS: [Jogada; 3] = [Jogada::Pedra, Jogada::Papel, Jogada::Tesoura];

    /// A jogada que ganha desta
    fn vencida_por(self) -> Jogada {
        match self { Jogada::Pedra => Jogada::Papel, Jogada::Papel => Jogada::Tesoura, Jogada::Tesoura => Jogada::Pedra }
    }

    /// Resultado do ponto de vista de `self` contra `outra`
    fn contra(self, outra: Jogada) -> Resultado {
        if self == outra { Resultado::Empate } else if outra.vencida_por() == self { Resultado::Vitoria } else { Resultado::Derrota }
    }
}

impl FromStr for Jogada {
    type Err = String;
    fn from_str(texto: &str) -> Result<Self, Self::Err> {
        match texto.trim().to_lowercase().as_str() {
            "pedra" | "pe" | "1" => Ok(Jogada::Pedra),
            "papel" | "pa" | "2" => Ok(Jogada::Papel),
            "tesoura" | "te" | "3" => Ok(Jogada::Tesoura),
            outro => Err(format!("jogada desconhecida: \"{outro}\" (use pedra, papel ou tesoura)")),
        }
    }
}

impl fmt::Display for Jogada {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self { Jogada::Pedra => "pedra", Jogada::Papel => "papel", Jogada::Tesoura => "tesoura" })
    }
}

#[derive(PartialEq, Debug)]
enum Resultado { Vitoria, Derrota, Empate }

#[derive(Default)]
struct Placar { jogador: u32, computador: u32, empates: u32 }

impl Placar {
    fn registrar(&mut self, r: &Resultado) {
        match r { Resultado::Vitoria => self.jogador += 1, Resultado::Derrota => self.computador += 1, Resultado::Empate => self.empates += 1 }
    }
}

impl fmt::Display for Placar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "você {} x {} computador (empates: {})", self.jogador, self.computador, self.empates)
    }
}

/// O computador: guarda quantas vezes viu cada jogada do adversário
#[derive(Default)]
struct Computador { historico: HashMap<Jogada, u32>, semente: u64 }

impl Computador {
    fn novo() -> Self {
        let semente = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(99) | 1;
        Computador { historico: HashMap::new(), semente }
    }

    fn sortear(&mut self) -> Jogada {
        let x = &mut self.semente;
        *x ^= *x << 13; *x ^= *x >> 7; *x ^= *x << 17;
        Jogada::TODAS[(*x % 3) as usize]
    }

    /// A jogada mais usada pelo adversário, se já houver histórico
    fn preferida_do_adversario(&self) -> Option<Jogada> {
        // Desempate pela ordem fixa de TODAS, para a escolha ser previsível e testável
        Jogada::TODAS.iter().copied().filter(|j| self.historico.contains_key(j)).max_by_key(|j| self.historico[j])
    }

    fn escolher(&mut self) -> Jogada {
        match self.preferida_do_adversario() {
            Some(preferida) => preferida.vencida_por(), // joga o que vence a jogada favorita
            None => self.sortear(),
        }
    }

    fn aprender(&mut self, jogada: Jogada) { *self.historico.entry(jogada).or_insert(0) += 1; }
}

/// Uma rodada completa: o computador escolhe, o jogador joga, o placar é atualizado
fn rodada(numero: u32, jogada: Jogada, computador: &mut Computador, placar: &mut Placar) {
    let maquina = computador.escolher();
    let resultado = jogada.contra(maquina);
    computador.aprender(jogada);
    placar.registrar(&resultado);
    let texto = match resultado { Resultado::Vitoria => "você venceu", Resultado::Derrota => "computador venceu", Resultado::Empate => "empate" };
    println!("Rodada {numero}: você {jogada} x {maquina} computador -> {texto}. Placar: {placar}");
}

fn vencedor_final(placar: &Placar) -> &'static str {
    match placar.jogador.cmp(&placar.computador) {
        std::cmp::Ordering::Greater => "Você venceu a série!",
        std::cmp::Ordering::Less => "O computador venceu a série.",
        std::cmp::Ordering::Equal => "Série empatada.",
    }
}

/// Lê uma jogada do teclado; None quando a entrada acabou ou o jogador pediu para sair
fn ler_jogada() -> Option<Jogada> {
    loop {
        print!("Sua jogada (pedra, papel, tesoura ou sair): ");
        let _ = io::stdout().flush();
        let mut linha = String::new();
        if io::stdin().lock().read_line(&mut linha).ok()? == 0 || linha.trim() == "sair" { return None; }
        match linha.parse::<Jogada>() {
            Ok(jogada) => return Some(jogada),
            Err(erro) => println!("{erro}"),
        }
    }
}

fn jogar(rodadas: u32) -> Result<(), String> {
    let (mut computador, mut placar) = (Computador::novo(), Placar::default());
    let meta = rodadas / 2 + 1;
    println!("=== Pedra, papel e tesoura: melhor de {rodadas} (quem fizer {meta} vence) ===");
    for n in 1..=rodadas {
        let Some(jogada) = ler_jogada() else { println!("\nSérie encerrada. {placar}"); return Ok(()) };
        rodada(n, jogada, &mut computador, &mut placar);
        if placar.jogador >= meta || placar.computador >= meta { break; }
    }
    println!("{}", vencedor_final(&placar));
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Pedra, papel e tesoura: simulação de 7 rodadas ===");
    println!("O jogador simulado gosta de pedra; veja o computador perceber isso.\n");
    let (mut computador, mut placar) = (Computador::novo(), Placar::default());
    computador.semente = 12345; // semente fixa: a demonstração sai igual toda vez
    let jogadas = ["pedra", "pedra", "tesoura", "pedra", "papel", "pedra", "tesoura"];
    for (i, texto) in jogadas.iter().enumerate() {
        rodada(i as u32 + 1, texto.parse()?, &mut computador, &mut placar);
    }
    println!("\nPlacar final: {placar}\n{}", vencedor_final(&placar));
    let mut contagem: Vec<_> = computador.historico.iter().collect();
    contagem.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
    println!("\nO que o computador aprendeu sobre o jogador:");
    for (jogada, n) in contagem { println!("  {:<8} {n} vez(es)", jogada.to_string()); } // to_string para o alinhamento valer
    if let Some(p) = computador.preferida_do_adversario() {
        println!("  Preferida: {p}, então o computador responderá com {}.", p.vencida_por());
    }
    println!("\nJogada inválida de propósito: {:?}", "lagarto".parse::<Jogada>().err());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None => jogar(3),
        Some("--demo") => demo(),
        Some("--rodadas") => match args.get(1).and_then(|v| v.parse::<u32>().ok()).filter(|n| *n >= 1) {
            Some(n) => jogar(n),
            None => Err("--rodadas precisa de um inteiro maior que zero logo depois".into()),
        },
        Some("--ajuda") | Some("-h") => {
            println!("Uso: 49_pedra_papel_tesoura.exe [--rodadas N]\n     49_pedra_papel_tesoura.exe --demo");
            return;
        }
        Some(outro) => Err(format!("opção desconhecida: {outro}")),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Melhore a estratégia: em vez da jogada mais frequente, use a última
//    sequência de duas jogadas do adversário para prever a próxima.
// 2. Acrescente lagarto e Spock (cinco jogadas) e descubra como escrever
//    contra() sem listar todas as 25 combinações.
// 3. Salve o histórico do jogador num arquivo entre partidas, para o
//    computador "lembrar" das preferências na próxima execução.
// ------------------------------------------------------------
