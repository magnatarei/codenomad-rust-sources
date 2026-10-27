// ============================================================
// CodeNomad Academy  |  Sources Rust  |  14. Contador de palavras e frequência
// ------------------------------------------------------------
// O que faz: Lê um texto (de um arquivo ou do argumento) e conta símbolos,
//            palavras, linhas, frases e parágrafos; estima o tempo de leitura
//            e lista as N palavras mais frequentes, ignorando acentos,
//            maiúsculas e uma lista de palavras vazias do português.
// Como usar: 14_contador_palavras.exe artigo.txt --top 15
//            14_contador_palavras.exe --texto "Uma frase. Outra frase!" --top 5
//            14_contador_palavras.exe --demo
// O que você aprende: fs::read_to_string com erro tratado, HashMap de
//            contagem com entry(), split com closure, flat_map em chars,
//            sort_by com ordenação por dois critérios, structs de resultado.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::collections::HashMap;
use std::fs;

// Palavras tão comuns que não dizem nada sobre o assunto do texto.
// A lista é escrita com acento e normalizada em tempo de execução.
const PALAVRAS_VAZIAS: &[&str] = &[
    "a", "o", "as", "os", "um", "uma", "uns", "umas", "de", "do", "da", "dos", "das", "em", "no",
    "na", "nos", "nas", "por", "para", "com", "sem", "sobre", "entre", "e", "ou", "mas", "que",
    "se", "não", "ao", "aos", "à", "às", "pelo", "pela", "como", "mais", "muito", "já", "também",
    "ele", "ela", "eles", "elas", "eu", "você", "nós", "seu", "sua", "seus", "suas", "meu", "minha",
    "este", "esta", "esse", "essa", "isso", "isto", "foi", "ser", "são", "era", "tem", "há", "é",
    "até", "quando", "onde", "ainda", "só", "cada", "assim",
];

const TEXTO_DEMO: &str = "\
Aprender a programar é parecido com aprender um instrumento. No começo, cada nota sai torta, \
e a vontade de desistir aparece já na primeira semana. Mas a prática diária, mesmo que curta, \
constrói uma base que nenhum atalho substitui.

O segredo não está em decorar comandos. Está em entender o problema antes de escrever a primeira \
linha de código. Quem lê o enunciado com calma, desenha a solução no papel e só depois abre o editor \
costuma errar menos. E quando erra, entende o erro mais rápido!

Ferramentas mudam o tempo todo: linguagens novas, editores novos, bibliotecas novas. A lógica, \
porém, continua a mesma. Laços, condições, funções e dados bem organizados resolvem a maior parte \
dos problemas do dia a dia. Dominar a lógica é o que faz a diferença entre copiar código e criar código.

Por isso, pratique com projetos pequenos. Um contador de palavras, uma calculadora, um jogo simples. \
Cada projeto terminado ensina mais do que dez tutoriais assistidos pela metade. Programar é prática, \
prática e prática.";

struct Estatisticas {
    simbolos: usize,
    simbolos_sem_espaco: usize,
    palavras: usize,
    linhas: usize,
    frases: usize,
    paragrafos: usize,
    tamanho_medio: f64,
    minutos_leitura: f64,
}

/// Troca letras acentuadas pela versão sem acento, para a mesma palavra
/// contar uma vez só, esteja escrita com ou sem acento.
fn sem_acento(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        'ñ' => 'n',
        outro => outro,
    }
}

fn normalizar(palavra: &str) -> String {
    // to_lowercase pode devolver mais de um char (por isso flat_map).
    palavra.chars().flat_map(char::to_lowercase).map(sem_acento).collect()
}

/// Separa o texto em palavras: qualquer coisa que não seja letra ou dígito divide.
fn palavras(texto: &str) -> impl Iterator<Item = &str> {
    texto.split(|c: char| !c.is_alphanumeric()).filter(|p| !p.is_empty())
}

fn contar_frases(texto: &str) -> usize {
    // Uma frase termina num bloco de . ! ou ? (então "!!!" conta uma vez).
    let mut frases = 0;
    let mut dentro_de_fim = false;
    let mut houve_letra = false;
    for c in texto.chars() {
        if matches!(c, '.' | '!' | '?') {
            if !dentro_de_fim && houve_letra {
                frases += 1;
            }
            dentro_de_fim = true;
            houve_letra = false;
        } else {
            dentro_de_fim = false;
            if c.is_alphanumeric() {
                houve_letra = true;
            }
        }
    }
    // Texto que termina sem pontuação ainda tem uma última frase.
    if houve_letra { frases + 1 } else { frases }
}

fn calcular(texto: &str) -> Estatisticas {
    let lista: Vec<&str> = palavras(texto).collect();
    let letras_em_palavras: usize = lista.iter().map(|p| p.chars().count()).sum();
    let paragrafos = texto.replace("\r\n", "\n").split("\n\n").filter(|p| !p.trim().is_empty()).count();
    Estatisticas {
        simbolos: texto.chars().count(),
        simbolos_sem_espaco: texto.chars().filter(|c| !c.is_whitespace()).count(),
        palavras: lista.len(),
        linhas: texto.lines().count(),
        frases: contar_frases(texto),
        paragrafos,
        tamanho_medio: if lista.is_empty() { 0.0 } else { letras_em_palavras as f64 / lista.len() as f64 },
        minutos_leitura: lista.len() as f64 / 200.0,
    }
}

/// As `quantas` palavras mais frequentes, fora as vazias. Empate: ordem alfabética.
fn mais_frequentes(texto: &str, quantas: usize) -> Vec<(String, usize)> {
    let vazias: Vec<String> = PALAVRAS_VAZIAS.iter().map(|p| normalizar(p)).collect();
    let mut contagem: HashMap<String, usize> = HashMap::new();
    for palavra in palavras(texto).map(normalizar) {
        if palavra.chars().count() > 1 && !vazias.contains(&palavra) {
            *contagem.entry(palavra).or_insert(0) += 1;
        }
    }
    let mut ordenada: Vec<(String, usize)> = contagem.into_iter().collect();
    ordenada.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ordenada.truncate(quantas);
    ordenada
}

fn relatorio(texto: &str, quantas: usize) {
    let e = calcular(texto);
    println!("Estatísticas do texto");
    println!("  símbolos (com espaços): {:>6}", e.simbolos);
    println!("  símbolos (sem espaços): {:>6}", e.simbolos_sem_espaco);
    println!("  palavras:               {:>6}", e.palavras);
    println!("  linhas:                 {:>6}", e.linhas);
    println!("  frases:                 {:>6}", e.frases);
    println!("  parágrafos:             {:>6}", e.paragrafos);
    println!("  tamanho médio de palavra: {:.1} letras", e.tamanho_medio);
    println!("  tempo de leitura: ~{:.1} min (a 200 palavras por minuto)", e.minutos_leitura);
    let top = mais_frequentes(texto, quantas);
    println!("\nTop {} palavras (sem acentos, sem palavras vazias):", top.len());
    let maior = top.first().map(|(_, n)| *n).unwrap_or(1);
    for (posicao, (palavra, n)) in top.iter().enumerate() {
        let barra = "#".repeat(n * 30 / maior);
        println!("  {:>2}. {:<14} {:>3}  {barra}", posicao + 1, palavra, n);
    }
}

fn demo() {
    println!("=== Contador de palavras: demonstração ===\n");
    relatorio(TEXTO_DEMO, 10);
}

fn uso() {
    println!("Uso:");
    println!("  14_contador_palavras.exe <arquivo.txt> [--top N]");
    println!("  14_contador_palavras.exe --texto \"<texto>\" [--top N]");
    println!("  14_contador_palavras.exe --demo");
}

fn executar(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        uso();
        return Ok(());
    }
    if args[0] == "--demo" {
        demo();
        return Ok(());
    }
    let mut texto = None;
    let mut quantas = 10;
    let mut restante = args.iter();
    while let Some(arg) = restante.next() {
        match arg.as_str() {
            "--top" => {
                let valor = restante.next().ok_or("--top precisa de um número")?;
                quantas = valor.parse::<usize>().map_err(|_| format!("valor inválido para --top: '{valor}'"))?;
            }
            "--texto" => texto = Some(restante.next().ok_or("--texto precisa do texto entre aspas")?.clone()),
            caminho => {
                let conteudo = fs::read_to_string(caminho).map_err(|e| format!("não consegui ler '{caminho}': {e}"))?;
                texto = Some(conteudo);
            }
        }
    }
    let texto = texto.ok_or("informe um arquivo ou --texto")?;
    if texto.trim().is_empty() {
        return Err("o texto está vazio".to_string());
    }
    relatorio(&texto, quantas);
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(erro) = executar(&args) {
        eprintln!("Erro: {erro}");
        std::process::exit(1);
    }
}

// Para ir além:
// 1. Conte também os bigramas (pares de palavras seguidas) mais comuns.
// 2. Aceite vários arquivos de uma vez e mostre uma tabela comparando-os.
// 3. Exporte a frequência em CSV ("palavra,contagem") com a opção --csv.
