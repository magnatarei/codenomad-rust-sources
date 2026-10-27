// ============================================================
// CodeNomad Academy  |  Sources Rust  |  46. Jogo da forca
// ------------------------------------------------------------
// O que faz: sorteia uma palavra de um banco com dicas, desenha a forca em
//            texto a cada erro, mostra as letras já tentadas e decide
//            vitória ou derrota. No modo interativo você digita as letras;
//            no --demo um jogador automático joga uma partida inteira.
// Como usar: 46_jogo_forca.exe            (partida interativa, palavra sorteada)
//            46_jogo_forca.exe --tema linguagens
//            46_jogo_forca.exe --demo
// O que você aprende: struct com estado de jogo e métodos que o mudam,
//            enum para o resultado de cada tentativa, HashSet e BTreeSet,
//            iteradores com chars/map/collect, leitura de stdin com
//            tratamento de fim de entrada, arrays de tuplas como banco e
//            um gerador pseudoaleatório simples feito à mão (xorshift).
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::collections::BTreeSet;
use std::io::{self, BufRead, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, process};

/// Banco de palavras: (tema, palavra, dica). Sem acentos para o jogador digitar sem dúvida.
const BANCO: [(&str, &str, &str); 10] = [
    ("linguagens", "rust", "linguagem de sistemas com ownership"),
    ("linguagens", "python", "linguagem famosa pela indentação"),
    ("linguagens", "kotlin", "linguagem da JetBrains para Android"),
    ("conceitos", "iterador", "percorre uma coleção item a item"),
    ("conceitos", "booleano", "valor que só pode ser verdadeiro ou falso"),
    ("conceitos", "compilador", "transforma código fonte em executável"),
    ("conceitos", "ponteiro", "guarda o endereço de outro valor"),
    ("frutas", "jabuticaba", "fruta escura que nasce no tronco"),
    ("frutas", "abacaxi", "fruta com coroa"),
    ("frutas", "goiaba", "fruta rosada de doce famoso"),
];

const MAX_ERROS: usize = 6;

/// Os sete estágios do desenho, do zero ao sexto erro
const FORCA: [&str; 7] = [
    "  +---+\n  |   |\n      |\n      |\n      |\n      |\n=======",
    "  +---+\n  |   |\n  O   |\n      |\n      |\n      |\n=======",
    "  +---+\n  |   |\n  O   |\n  |   |\n      |\n      |\n=======",
    "  +---+\n  |   |\n  O   |\n /|   |\n      |\n      |\n=======",
    "  +---+\n  |   |\n  O   |\n /|\\  |\n      |\n      |\n=======",
    "  +---+\n  |   |\n  O   |\n /|\\  |\n /    |\n      |\n=======",
    "  +---+\n  |   |\n  O   |\n /|\\  |\n / \\  |\n      |\n=======",
];

#[derive(PartialEq, Debug)]
enum Tentativa { Acertou, Errou, Repetida, Invalida }

struct Partida { palavra: Vec<char>, dica: &'static str, tentadas: BTreeSet<char>, erros: usize }

impl Partida {
    fn nova(palavra: &str, dica: &'static str) -> Self {
        Partida { palavra: palavra.chars().collect(), dica, tentadas: BTreeSet::new(), erros: 0 }
    }

    fn tentar(&mut self, letra: char) -> Tentativa {
        let letra = letra.to_ascii_lowercase();
        if !letra.is_ascii_lowercase() { return Tentativa::Invalida; }
        // insert devolve false quando a letra já estava no conjunto
        if !self.tentadas.insert(letra) { return Tentativa::Repetida; }
        if self.palavra.contains(&letra) { Tentativa::Acertou } else { self.erros += 1; Tentativa::Errou }
    }

    /// A palavra com as letras descobertas e "_" nas que faltam
    fn mascara(&self) -> String {
        self.palavra.iter().map(|c| if self.tentadas.contains(c) { *c } else { '_' }).map(|c| format!("{c} ")).collect()
    }

    fn venceu(&self) -> bool { self.palavra.iter().all(|c| self.tentadas.contains(c)) }
    fn perdeu(&self) -> bool { self.erros >= MAX_ERROS }

    fn mostrar(&self) {
        println!("{}", FORCA[self.erros.min(MAX_ERROS)]);
        println!("Palavra: {}   (dica: {})", self.mascara(), self.dica);
        let tentadas: String = self.tentadas.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
        println!("Tentadas: [{tentadas}]   Erros: {}/{MAX_ERROS}\n", self.erros);
    }
}

/// Gerador xorshift: não serve para sorteios sérios, mas basta para escolher uma palavra
fn sortear(limite: usize) -> usize {
    let mut x = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(42) | 1;
    x ^= x << 13; x ^= x >> 7; x ^= x << 17;
    (x % limite as u64) as usize
}

fn escolher(tema: Option<&str>) -> Result<(&'static str, &'static str), String> {
    let candidatas: Vec<_> = BANCO.iter().filter(|(t, _, _)| tema.map_or(true, |tema| *t == tema)).collect();
    if candidatas.is_empty() {
        let temas: BTreeSet<_> = BANCO.iter().map(|(t, _, _)| *t).collect();
        return Err(format!("tema desconhecido; os temas são: {}", temas.into_iter().collect::<Vec<_>>().join(", ")));
    }
    let (_, palavra, dica) = candidatas[sortear(candidatas.len())];
    Ok((palavra, dica))
}

fn anunciar(partida: &Partida) {
    if partida.venceu() { println!("Você venceu! A palavra era \"{}\".", partida.palavra.iter().collect::<String>()); }
    else { println!("Enforcado! A palavra era \"{}\".", partida.palavra.iter().collect::<String>()); }
}

fn interativo(tema: Option<&str>) -> Result<(), String> {
    let (palavra, dica) = escolher(tema)?;
    let mut partida = Partida::nova(palavra, dica);
    let entrada = io::stdin();
    println!("=== Jogo da forca ===  (digite uma letra e Enter; \"sair\" encerra)\n");
    while !partida.venceu() && !partida.perdeu() {
        partida.mostrar();
        print!("Letra: ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        let mut linha = String::new();
        // read_line devolve 0 bytes quando a entrada acabou (Ctrl+D ou Ctrl+Z)
        if entrada.lock().read_line(&mut linha).map_err(|e| e.to_string())? == 0 || linha.trim() == "sair" {
            println!("\nPartida encerrada."); return Ok(());
        }
        let Some(letra) = linha.trim().chars().next() else { continue };
        match partida.tentar(letra) {
            Tentativa::Acertou => println!("Boa, tem \"{letra}\"!"),
            Tentativa::Errou => println!("Não tem \"{letra}\"."),
            Tentativa::Repetida => println!("Você já tentou \"{letra}\"."),
            Tentativa::Invalida => println!("Digite só uma letra de a a z."),
        }
    }
    partida.mostrar();
    anunciar(&partida);
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Jogo da forca: partida automática ===");
    let mut partida = Partida::nova("compilador", "transforma código fonte em executável");
    // O jogador automático segue a ordem de frequência das letras em português
    let ordem = "aeosrindmutclpvgbfhqjzxkwy";
    for (rodada, letra) in ordem.chars().enumerate() {
        if partida.venceu() || partida.perdeu() { break; }
        let resultado = partida.tentar(letra);
        println!("Rodada {}: tenta \"{letra}\" -> {resultado:?}", rodada + 1);
        if resultado == Tentativa::Errou { println!("{}", FORCA[partida.erros]); }
        println!("  {}", partida.mascara());
    }
    println!();
    partida.mostrar();
    anunciar(&partida);
    println!("Tentativa repetida de propósito: {:?}", partida.tentar('a'));
    println!("Tentativa inválida de propósito: {:?}", partida.tentar('7'));
    println!("Tema inexistente: {:?}", escolher(Some("planetas")).err());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None => interativo(None),
        Some("--demo") => demo(),
        Some("--tema") => match args.get(1) {
            Some(tema) => interativo(Some(tema)),
            None => Err("--tema precisa do nome do tema logo depois".into()),
        },
        Some("--ajuda") | Some("-h") => {
            println!("Uso: 46_jogo_forca.exe [--tema linguagens|conceitos|frutas]\n     46_jogo_forca.exe --demo");
            return;
        }
        Some(outro) => Err(format!("opção desconhecida: {outro}")),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Carregue o banco de um arquivo texto (tema;palavra;dica por linha)
//    passado com --banco, mantendo o embutido como reserva.
// 2. Permita chutar a palavra inteira: se o jogador digitar mais de uma
//    letra, compare com a palavra; acerto vence na hora, erro custa 2 vidas.
// 3. Guarde um placar em arquivo com vitórias e derrotas por tema e mostre
//    a porcentagem de acerto ao final de cada partida.
// ------------------------------------------------------------
