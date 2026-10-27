// ============================================================
// CodeNomad Academy  |  Sources Rust  |  48. Adivinhe o número, nos dois sentidos
// ------------------------------------------------------------
// O que faz: modo 1, você adivinha o número secreto do computador com dicas
//            de "maior" ou "menor" e contagem de tentativas. Modo 2, você
//            pensa num número e o computador o descobre por busca binária,
//            perguntando a cada palpite se o seu é maior, menor ou igual.
// Como usar: 48_adivinhe_numero.exe eu --max 100
//            48_adivinhe_numero.exe computador --max 1000
//            48_adivinhe_numero.exe computador --segredo 73   (responde sozinho)
//            48_adivinhe_numero.exe --demo
// O que você aprende: busca binária explicada passo a passo, Ordering e
//            cmp, enum para a resposta do jogador, trait para separar quem
//            responde (teclado ou automático), leitura validada de stdin,
//            cálculo de log2 para o número máximo de palpites e xorshift.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::cmp::Ordering;
use std::io::{self, BufRead, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, process};

/// Quem responde aos palpites do computador: uma pessoa ou um número fixo
trait Oraculo {
    fn comparar(&mut self, palpite: u32) -> Option<Ordering>;
}

/// Oráculo automático: sabe o segredo e responde na hora
struct Segredo(u32);

impl Oraculo for Segredo {
    fn comparar(&mut self, palpite: u32) -> Option<Ordering> {
        let resposta = self.0.cmp(&palpite);
        let texto = match resposta { Ordering::Greater => "maior", Ordering::Less => "menor", Ordering::Equal => "certo" };
        println!("  resposta: {texto}");
        Some(resposta)
    }
}

/// Oráculo humano: pergunta no terminal até receber uma resposta válida
struct Teclado;

impl Oraculo for Teclado {
    fn comparar(&mut self, palpite: u32) -> Option<Ordering> {
        loop {
            print!("  O seu número é maior, menor ou certo? ");
            let _ = io::stdout().flush();
            let mut linha = String::new();
            if io::stdin().lock().read_line(&mut linha).ok()? == 0 { return None; } // fim da entrada
            match linha.trim().to_lowercase().as_str() {
                "maior" | "+" => return Some(Ordering::Greater),
                "menor" | "-" => return Some(Ordering::Less),
                "certo" | "=" | "sim" => return Some(Ordering::Equal),
                _ => println!("  Responda maior, menor ou certo (palpite: {palpite})."),
            }
        }
    }
}

/// Quantos palpites a busca binária precisa, no pior caso, para `n` valores
fn palpites_maximos(n: u32) -> u32 { u32::BITS - n.leading_zeros() }

/// Modo 2: o computador adivinha por busca binária. Devolve o número de palpites.
fn computador_adivinha(max: u32, oraculo: &mut dyn Oraculo) -> Result<u32, String> {
    let (mut baixo, mut alto, mut palpites) = (1u32, max, 0u32);
    println!("Pense num número de 1 a {max}. Vou acertar em no máximo {} palpites.", palpites_maximos(max));
    while baixo <= alto {
        let meio = baixo + (alto - baixo) / 2; // evita estourar o u32 em intervalos grandes
        palpites += 1;
        println!("Palpite {palpites}: é {meio}? (procurando entre {baixo} e {alto})");
        match oraculo.comparar(meio).ok_or("entrada encerrada antes do fim")? {
            Ordering::Equal => { println!("Acertei em {palpites} palpite(s)!"); return Ok(palpites); }
            Ordering::Greater => baixo = meio + 1,
            Ordering::Less => alto = meio.checked_sub(1).ok_or("o número não pode ser menor que 1")?,
        }
    }
    Err("as respostas se contradizem: nenhum número cabe nelas".into())
}

fn sortear(max: u32) -> u32 {
    let mut x = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(7) | 1;
    x ^= x << 13; x ^= x >> 7; x ^= x << 17;
    (x % u64::from(max)) as u32 + 1
}

/// Modo 1: você adivinha. Lê do teclado até acertar ou a entrada acabar.
fn eu_adivinho(max: u32) -> Result<(), String> {
    let segredo = sortear(max);
    let mut tentativas = 0;
    println!("Escolhi um número de 1 a {max}. Tente adivinhar (\"sair\" encerra).");
    loop {
        print!("Seu palpite: ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        let mut linha = String::new();
        if io::stdin().lock().read_line(&mut linha).map_err(|e| e.to_string())? == 0 || linha.trim() == "sair" {
            println!("\nO número era {segredo}. Até a próxima!"); return Ok(());
        }
        let Ok(palpite) = linha.trim().parse::<u32>() else { println!("Digite um número inteiro."); continue };
        if !(1..=max).contains(&palpite) { println!("Fique entre 1 e {max}."); continue; }
        tentativas += 1;
        match palpite.cmp(&segredo) {
            Ordering::Less => println!("O segredo é maior que {palpite}."),
            Ordering::Greater => println!("O segredo é menor que {palpite}."),
            Ordering::Equal => { println!("Acertou em {tentativas} tentativa(s)!"); return Ok(()); }
        }
    }
}

/// Lê "--max N" e "--segredo N" depois do nome do modo
fn opcoes(args: &[String]) -> Result<(u32, Option<u32>), String> {
    let (mut max, mut segredo) = (100u32, None);
    let mut it = args.iter();
    while let Some(opcao) = it.next() {
        let valor = it.next().ok_or_else(|| format!("{opcao} precisa de um número logo depois"))?;
        let n = valor.parse::<u32>().ok().filter(|n| *n >= 1).ok_or_else(|| format!("{opcao} espera um inteiro positivo, recebi \"{valor}\""))?;
        match opcao.as_str() {
            "--max" => max = n,
            "--segredo" => segredo = Some(n),
            outro => return Err(format!("opção desconhecida: {outro}")),
        }
    }
    if segredo.is_some_and(|s| s > max) { return Err(format!("o segredo tem de estar entre 1 e {max}")); }
    Ok((max, segredo))
}

fn demo() -> Result<(), String> {
    println!("=== Adivinhe o número: o computador descobre 73 por busca binária ===\n");
    let palpites = computador_adivinha(100, &mut Segredo(73))?;
    println!("\nA cada palpite o intervalo cai pela metade: 100 -> 50 -> 25 -> 12 -> 6 -> 3 -> 1.");
    println!("Usou {palpites} de no máximo {} palpites.\n", palpites_maximos(100));
    println!("Limites para outros tamanhos: 1000 -> {} palpites, 1 milhão -> {} palpites.", palpites_maximos(1000), palpites_maximos(1_000_000));
    println!("\nCaso extremo: segredo 1 entre 1 e 8");
    computador_adivinha(8, &mut Segredo(1))?;
    println!("\nErro tratado de propósito: {:?}", opcoes(&["--max".into(), "dez".into()]).err());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None | Some("--ajuda") | Some("-h") => {
            println!("Uso: 48_adivinhe_numero.exe eu [--max 100]");
            println!("     48_adivinhe_numero.exe computador [--max 100] [--segredo N]");
            println!("     48_adivinhe_numero.exe --demo");
            return;
        }
        Some("--demo") => demo(),
        Some("eu") => opcoes(&args[1..]).and_then(|(max, _)| eu_adivinho(max)),
        Some("computador") => opcoes(&args[1..]).and_then(|(max, segredo)| match segredo {
            Some(s) => computador_adivinha(max, &mut Segredo(s)).map(|_| ()),
            None => computador_adivinha(max, &mut Teclado).map(|_| ()),
        }),
        Some(outro) => Err(format!("modo desconhecido: {outro} (use eu, computador ou --demo)")),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. No modo 1, mostre uma dica de "quente" ou "frio" conforme a distância
//    até o segredo, e guarde o recorde de menos tentativas num arquivo.
// 2. Implemente um oráculo que mente uma vez a cada N respostas e adapte a
//    busca para confirmar o acerto antes de comemorar.
// 3. Troque a busca binária por busca ternária (dois pontos de corte) e
//    compare o número de palpites nos dois métodos para max = 1000.
// ------------------------------------------------------------
