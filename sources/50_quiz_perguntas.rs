// ============================================================
// CodeNomad Academy  |  Sources Rust  |  50. Quiz de perguntas e respostas
// ------------------------------------------------------------
// O que faz: quiz de múltipla escolha. Lê as perguntas de um arquivo texto
//            simples (ou usa o banco embutido sobre programação), embaralha,
//            pontua cada resposta e, no fim, revisa as perguntas erradas com
//            a resposta certa. No --demo um aluno automático responde.
// Como usar: 50_quiz_perguntas.exe                      (banco embutido)
//            50_quiz_perguntas.exe --arquivo minhas.txt --sem-embaralhar
//            50_quiz_perguntas.exe --demo
// Formato do arquivo: linhas "P: pergunta", "A) opção", "B) opção", ... e
//            "R: B" com a letra certa; uma linha vazia separa as perguntas.
// O que você aprende: um analisador de formato de texto linha a linha com
//            mensagens de erro que dizem a linha do problema, struct com
//            Vec de opções, embaralhamento Fisher-Yates feito à mão,
//            leitura de stdin validada e relatório final com filter.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::io::{self, BufRead, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs, process};

/// Banco embutido no mesmo formato do arquivo, para o leitor ser um só
const BANCO: &str = "P: Qual palavra-chave declara uma variável imutável em Rust?
A) var
B) let
C) const mut
R: B

P: O que o operador ? faz numa função que devolve Result?
A) Converte o erro em pânico
B) Devolve o erro para quem chamou, se houver
C) Ignora o erro
R: B

P: Qual estrutura guarda pares chave e valor?
A) Vec
B) HashMap
C) String
R: B

P: O que é ownership?
A) Cada valor tem um único dono responsável por liberá-lo
B) Uma licença de software
C) Um tipo de ponteiro nulo
R: A

P: Qual tipo representa a ausência opcional de valor?
A) Option
B) Null
C) Void
R: A

P: O que faz o método iter().map()?
A) Ordena a coleção
B) Transforma cada item com uma função
C) Remove duplicados
R: B

P: Qual é o resultado de 7 / 2 com inteiros?
A) 3.5
B) 3
C) 4
R: B

P: Para que serve o trait Display?
A) Para desenhar na tela gráfica
B) Para definir como um valor vira texto em println!
C) Para comparar valores
R: B
";

struct Pergunta { texto: String, opcoes: Vec<String>, certa: usize }

impl Pergunta {
    fn letra(i: usize) -> char { (b'A' + i as u8) as char }
    fn mostrar(&self, numero: usize) {
        println!("\n{numero}) {}", self.texto);
        for (i, op) in self.opcoes.iter().enumerate() { println!("   {}) {op}", Self::letra(i)); }
    }
}

/// Lê o formato de texto. Cada erro diz em que linha está o problema.
fn ler_perguntas(texto: &str) -> Result<Vec<Pergunta>, String> {
    let mut perguntas = Vec::new();
    let mut atual: Option<Pergunta> = None;
    let fechar = |p: Option<Pergunta>, lista: &mut Vec<Pergunta>, linha: usize| -> Result<(), String> {
        if let Some(p) = p {
            if p.opcoes.len() < 2 { return Err(format!("linha {linha}: a pergunta \"{}\" precisa de pelo menos 2 opções", p.texto)); }
            if p.certa >= p.opcoes.len() { return Err(format!("linha {linha}: a pergunta \"{}\" não tem resposta (R:) válida", p.texto)); }
            lista.push(p);
        }
        Ok(())
    };
    for (n, linha) in texto.lines().enumerate() {
        let (n, linha) = (n + 1, linha.trim());
        if linha.is_empty() { continue; }
        if let Some(p) = linha.strip_prefix("P:") {
            fechar(atual.take(), &mut perguntas, n)?;
            atual = Some(Pergunta { texto: p.trim().to_string(), opcoes: Vec::new(), certa: usize::MAX });
            continue;
        }
        let Some(pergunta) = atual.as_mut() else { return Err(format!("linha {n}: esperava \"P:\" antes de \"{linha}\"")); };
        if let Some(r) = linha.strip_prefix("R:") {
            let letra = r.trim().chars().next().map(|c| c.to_ascii_uppercase()).ok_or(format!("linha {n}: R: sem letra"))?;
            pergunta.certa = (letra as usize).checked_sub('A' as usize).ok_or(format!("linha {n}: letra inválida em R:"))?;
        } else if linha.len() > 2 && linha.as_bytes()[1] == b')' && linha.as_bytes()[0].is_ascii_alphabetic() {
            pergunta.opcoes.push(linha[2..].trim().to_string());
        } else {
            return Err(format!("linha {n}: não entendi \"{linha}\" (use P:, A), B)... ou R:)"));
        }
    }
    fechar(atual.take(), &mut perguntas, texto.lines().count())?;
    if perguntas.is_empty() { return Err("nenhuma pergunta encontrada".into()); }
    Ok(perguntas)
}

/// Fisher-Yates: percorre do fim ao início trocando cada posição com uma sorteada antes dela
fn embaralhar<T>(itens: &mut [T], semente: u64) {
    let mut x = semente | 1;
    for i in (1..itens.len()).rev() {
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        itens.swap(i, (x % (i as u64 + 1)) as usize);
    }
}

/// Uma resposta dada: o índice da pergunta e a opção escolhida
struct Resposta { pergunta: usize, escolhida: usize }

fn corrigir(perguntas: &[Pergunta], respostas: &[Resposta]) {
    let acertos = respostas.iter().filter(|r| perguntas[r.pergunta].certa == r.escolhida).count();
    println!("\n=== Resultado: {acertos} de {} ({:.0}%) ===", respostas.len(), 100.0 * acertos as f64 / respostas.len().max(1) as f64);
    let erradas: Vec<_> = respostas.iter().filter(|r| perguntas[r.pergunta].certa != r.escolhida).collect();
    if erradas.is_empty() { println!("Tudo certo, parabéns!"); return; }
    println!("Revisão das {} erradas:", erradas.len());
    for r in erradas {
        let p = &perguntas[r.pergunta];
        println!("  * {}\n    você marcou {}) {}\n    certa:      {}) {}", p.texto,
            Pergunta::letra(r.escolhida), p.opcoes[r.escolhida], Pergunta::letra(p.certa), p.opcoes[p.certa]);
    }
}

fn ler_letra(max: usize) -> Option<usize> {
    loop {
        print!("Resposta (A-{}): ", Pergunta::letra(max - 1));
        let _ = io::stdout().flush();
        let mut linha = String::new();
        if io::stdin().lock().read_line(&mut linha).ok()? == 0 { return None; }
        match linha.trim().chars().next().map(|c| c.to_ascii_uppercase()) {
            Some(c) if c.is_ascii_uppercase() && ((c as usize) - ('A' as usize)) < max => return Some(c as usize - 'A' as usize),
            _ => println!("Digite uma letra de A a {}.", Pergunta::letra(max - 1)),
        }
    }
}

fn jogar(perguntas: &[Pergunta]) -> Result<(), String> {
    println!("=== Quiz: {} perguntas. Digite a letra e Enter. ===", perguntas.len());
    let mut respostas = Vec::new();
    for (i, p) in perguntas.iter().enumerate() {
        p.mostrar(i + 1);
        let Some(escolhida) = ler_letra(p.opcoes.len()) else { println!("\nEntrada encerrada."); break };
        println!("{}", if escolhida == p.certa { "Certo!" } else { "Errado." });
        respostas.push(Resposta { pergunta: i, escolhida });
    }
    if respostas.is_empty() { return Err("nenhuma resposta dada".into()); }
    corrigir(perguntas, &respostas);
    Ok(())
}

fn executar(args: &[String]) -> Result<(), String> {
    let (mut arquivo, mut embaralha) = (None, true);
    let mut it = args.iter();
    while let Some(opcao) = it.next() {
        match opcao.as_str() {
            "--arquivo" => arquivo = Some(it.next().ok_or("--arquivo precisa de um caminho logo depois")?.clone()),
            "--sem-embaralhar" => embaralha = false,
            outro => return Err(format!("opção desconhecida: {outro}")),
        }
    }
    let texto = match &arquivo {
        Some(caminho) => fs::read_to_string(caminho).map_err(|e| format!("não consegui ler {caminho}: {e}"))?,
        None => BANCO.to_string(),
    };
    let mut perguntas = ler_perguntas(&texto)?;
    if embaralha {
        let semente = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        embaralhar(&mut perguntas, semente);
    }
    jogar(&perguntas)
}

fn demo() -> Result<(), String> {
    println!("=== Quiz: demonstração com aluno automático ===");
    let mut perguntas = ler_perguntas(BANCO)?;
    embaralhar(&mut perguntas, 2026); // semente fixa: a ordem é sempre a mesma na demonstração
    let mut respostas = Vec::new();
    for (i, p) in perguntas.iter().enumerate() {
        p.mostrar(i + 1);
        // O aluno automático erra de propósito a 2ª, a 5ª e a 7ª pergunta
        let escolhida = if [1, 4, 6].contains(&i) { (p.certa + 1) % p.opcoes.len() } else { p.certa };
        println!("   aluno responde {} -> {}", Pergunta::letra(escolhida), if escolhida == p.certa { "certo" } else { "errado" });
        respostas.push(Resposta { pergunta: i, escolhida });
    }
    corrigir(&perguntas, &respostas);
    println!("\nArquivo malformado de propósito: {:?}", ler_perguntas("P: Só uma?\nA) sim\nR: A").err());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        Some("--demo") => demo(),
        Some("--ajuda") | Some("-h") => {
            println!("Uso: 50_quiz_perguntas.exe [--arquivo perguntas.txt] [--sem-embaralhar]\n     50_quiz_perguntas.exe --demo");
            return;
        }
        _ => executar(&args),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Embaralhe também as opções de cada pergunta, lembrando de atualizar o
//    índice da resposta certa junto.
// 2. Acrescente um campo opcional "E: explicação" ao formato e mostre-o na
//    revisão das erradas.
// 3. Guarde o histórico de pontuações num arquivo e mostre, ao final, se o
//    aluno superou a própria melhor marca.
// ------------------------------------------------------------
