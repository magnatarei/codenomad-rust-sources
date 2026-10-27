// ============================================================
// CodeNomad Academy  |  Sources Rust  |  47. Jogo da velha com computador
// ------------------------------------------------------------
// O que faz: jogo da velha no terminal para dois jogadores ou contra o
//            computador, que usa o algoritmo minimax e por isso nunca perde.
//            Detecta vitória e empate e desenha o tabuleiro a cada jogada.
//            No --demo o computador joga contra ele mesmo.
// Como usar: 47_jogo_velha.exe            (você com X contra o computador)
//            47_jogo_velha.exe --dois     (duas pessoas, X e O)
//            47_jogo_velha.exe --segundo  (o computador começa)
//            47_jogo_velha.exe --demo
// O que você aprende: enum com Copy e Display, array fixo como tabuleiro,
//            recursão (minimax) com pontuação e profundidade, iteradores
//            com max_by_key e filter, leitura de stdin validada, e como
//            separar as regras do jogo da interface com o usuário.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::fmt;
use std::io::{self, BufRead, Write};
use std::{env, process};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Jogador { X, O }

impl Jogador {
    fn outro(self) -> Jogador { if self == Jogador::X { Jogador::O } else { Jogador::X } }
}

impl fmt::Display for Jogador {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", if *self == Jogador::X { 'X' } else { 'O' }) }
}

/// As oito linhas que dão vitória: três horizontais, três verticais, duas diagonais
const LINHAS: [[usize; 3]; 8] = [[0, 1, 2], [3, 4, 5], [6, 7, 8], [0, 3, 6], [1, 4, 7], [2, 5, 8], [0, 4, 8], [2, 4, 6]];

#[derive(Clone, Copy)]
struct Tabuleiro { casas: [Option<Jogador>; 9] }

impl Tabuleiro {
    fn vazio() -> Self { Tabuleiro { casas: [None; 9] } }

    fn vencedor(&self) -> Option<Jogador> {
        LINHAS.iter().find_map(|l| match (self.casas[l[0]], self.casas[l[1]], self.casas[l[2]]) {
            (Some(a), Some(b), Some(c)) if a == b && b == c => Some(a),
            _ => None,
        })
    }

    fn livres(&self) -> impl Iterator<Item = usize> + '_ {
        (0..9).filter(|i| self.casas[*i].is_none())
    }

    fn cheio(&self) -> bool { self.livres().next().is_none() }

    fn jogar(&mut self, casa: usize, jogador: Jogador) -> Result<(), String> {
        match self.casas.get(casa) {
            None => Err(format!("a casa tem de ser de 1 a 9, não {}", casa + 1)),
            Some(Some(_)) => Err(format!("a casa {} já está ocupada", casa + 1)),
            Some(None) => { self.casas[casa] = Some(jogador); Ok(()) }
        }
    }

    /// Minimax: devolve a pontuação da posição para `quem_joga`, do ponto de vista de `eu`.
    /// Vitória vale 10 menos a profundidade: ganhar rápido é melhor, perder tarde é menos ruim.
    fn minimax(&self, quem_joga: Jogador, eu: Jogador, profundidade: i32) -> i32 {
        if let Some(v) = self.vencedor() { return if v == eu { 10 - profundidade } else { profundidade - 10 }; }
        if self.cheio() { return 0; }
        let pontos = self.livres().map(|casa| {
            let mut copia = *self; // o tabuleiro é Copy: cada ramo da recursão altera a própria cópia
            copia.casas[casa] = Some(quem_joga);
            copia.minimax(quem_joga.outro(), eu, profundidade + 1)
        });
        // Na minha vez escolho o máximo; na vez do adversário ele escolhe o mínimo para mim
        if quem_joga == eu { pontos.max().unwrap_or(0) } else { pontos.min().unwrap_or(0) }
    }

    /// A melhor casa para `jogador`. Devolve None só se o tabuleiro estiver cheio.
    fn melhor_jogada(&self, jogador: Jogador) -> Option<usize> {
        self.livres().max_by_key(|casa| {
            let mut copia = *self;
            copia.casas[*casa] = Some(jogador);
            // max_by_key devolve o ÚLTIMO máximo; o sinal negativo no índice desempata pela primeira casa
            (copia.minimax(jogador.outro(), jogador, 1), -(*casa as i32))
        })
    }
}

impl fmt::Display for Tabuleiro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for linha in 0..3 {
            let casas: Vec<String> = (0..3).map(|col| {
                let i = linha * 3 + col;
                self.casas[i].map_or((i + 1).to_string(), |j| j.to_string())
            }).collect();
            writeln!(f, "   {}", casas.join(" | "))?;
            if linha < 2 { writeln!(f, "  ---+---+---")?; }
        }
        Ok(())
    }
}

/// Lê uma casa de 1 a 9 do teclado; None quando a entrada acabou
fn ler_casa(jogador: Jogador) -> Option<usize> {
    let entrada = io::stdin();
    loop {
        print!("Jogador {jogador}, escolha uma casa (1-9): ");
        let _ = io::stdout().flush();
        let mut linha = String::new();
        if entrada.lock().read_line(&mut linha).ok()? == 0 { return None; }
        match linha.trim().parse::<usize>() {
            Ok(n) if (1..=9).contains(&n) => return Some(n - 1),
            _ => println!("Digite um número de 1 a 9."),
        }
    }
}

fn anunciar(tab: &Tabuleiro) {
    match tab.vencedor() {
        Some(v) => println!("Vitória de {v}!"),
        None => println!("Deu velha: empate."),
    }
}

/// Uma partida. `computador` diz qual lado é da máquina (None = duas pessoas).
fn partida(computador: Option<Jogador>) -> Result<(), String> {
    let mut tab = Tabuleiro::vazio();
    let mut vez = Jogador::X;
    println!("{tab}");
    while tab.vencedor().is_none() && !tab.cheio() {
        let casa = if computador == Some(vez) {
            let casa = tab.melhor_jogada(vez).ok_or("sem jogadas")?;
            println!("Computador ({vez}) joga na casa {}.", casa + 1);
            casa
        } else {
            let Some(casa) = ler_casa(vez) else { println!("\nPartida encerrada."); return Ok(()) };
            casa
        };
        if let Err(e) = tab.jogar(casa, vez) { println!("{e}"); continue; }
        println!("{tab}");
        vez = vez.outro();
    }
    anunciar(&tab);
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Jogo da velha: computador contra computador ===\n");
    let mut tab = Tabuleiro::vazio();
    let mut vez = Jogador::X;
    let mut jogada = 1;
    while tab.vencedor().is_none() && !tab.cheio() {
        let casa = tab.melhor_jogada(vez).ok_or("sem jogadas")?;
        tab.jogar(casa, vez)?;
        println!("Jogada {jogada}: {vez} escolhe a casa {}", casa + 1);
        println!("{tab}");
        vez = vez.outro();
        jogada += 1;
    }
    anunciar(&tab);
    println!("Dois minimax perfeitos sempre empatam: o jogo da velha é \"resolvido\".\n");
    // Mostra a defesa: X ameaça fechar a primeira linha e O precisa bloquear a casa 3
    let mut ameaca = Tabuleiro::vazio();
    for (casa, j) in [(0, Jogador::X), (4, Jogador::O), (1, Jogador::X)] { ameaca.jogar(casa, j)?; }
    println!("Posição de teste:\n{ameaca}");
    println!("O deve bloquear na casa {}.", ameaca.melhor_jogada(Jogador::O).map_or(0, |c| c + 1));
    println!("Jogada em casa ocupada: {:?}", ameaca.jogar(4, Jogador::X).err());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None => partida(Some(Jogador::O)),
        Some("--dois") => partida(None),
        Some("--segundo") => partida(Some(Jogador::X)),
        Some("--demo") => demo(),
        Some("--ajuda") | Some("-h") => {
            println!("Uso: 47_jogo_velha.exe [--dois | --segundo]\n     47_jogo_velha.exe --demo");
            return;
        }
        Some(outro) => Err(format!("opção desconhecida: {outro} (use --dois, --segundo ou --demo)")),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Acrescente um nível "fácil" em que o computador joga numa casa livre
//    sorteada, e um "médio" que só bloqueia ameaças imediatas.
// 2. Conte quantas posições o minimax visitou numa jogada (um contador
//    passado por &mut) e mostre como esse número cai ao longo da partida.
// 3. Generalize o tabuleiro para 4x4 usando um Vec e gere as linhas
//    vencedoras com laços em vez da constante LINHAS.
// ------------------------------------------------------------
