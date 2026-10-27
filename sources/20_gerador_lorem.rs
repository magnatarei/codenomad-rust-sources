// ============================================================
// CodeNomad Academy  |  Sources Rust  |  20. Gerador de texto de preenchimento
// ------------------------------------------------------------
// O que faz: Gera parágrafos, frases ou palavras de texto de preenchimento
//            em latim clássico (lorem ipsum) ou em português, com quantidade
//            escolhível e uma semente opcional para repetir o mesmo texto.
// Como usar: 20_gerador_lorem.exe paragrafos 3 --idioma pt --semente 42
//            20_gerador_lorem.exe frases 5
//            20_gerador_lorem.exe palavras 20 --idioma la
//            20_gerador_lorem.exe --demo
// O que você aprende: gerador pseudoaleatório determinístico (xorshift)
//            dentro de uma struct, enum com FromStr, fatias &[&str] como
//            dicionário, String construída com push_str, laço de opções
//            com while let, semente vinda do relógio quando não informada.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

const LATIM: &[&str] = &[
    "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit", "sed", "do",
    "eiusmod", "tempor", "incididunt", "ut", "labore", "et", "dolore", "magna", "aliqua", "enim",
    "ad", "minim", "veniam", "quis", "nostrud", "exercitation", "ullamco", "laboris", "nisi",
    "aliquip", "ex", "ea", "commodo", "consequat", "duis", "aute", "irure", "in", "reprehenderit",
    "voluptate", "velit", "esse", "cillum", "fugiat", "nulla", "pariatur", "excepteur", "sint",
    "occaecat", "cupidatat", "non", "proident", "sunt", "culpa", "qui", "officia", "deserunt",
    "mollit", "anim", "id", "est", "laborum", "vitae", "porta", "semper", "ornare", "tellus",
];

const PORTUGUES: &[&str] = &[
    "o", "a", "um", "uma", "de", "com", "sem", "para", "entre", "sobre", "casa", "rua", "cidade",
    "janela", "porta", "mesa", "livro", "caderno", "escola", "aluno", "professor", "projeto",
    "ideia", "tempo", "manhã", "tarde", "noite", "chuva", "vento", "sol", "mar", "rio", "montanha",
    "caminho", "viagem", "história", "música", "silêncio", "palavra", "número", "código", "função",
    "página", "memória", "futuro", "passado", "lembrança", "amigo", "família", "trabalho", "descanso",
    "ler", "escrever", "aprender", "ensinar", "construir", "pensar", "olhar", "caminhar", "voltar",
    "grande", "pequeno", "antigo", "novo", "claro", "simples", "comum", "tranquilo", "curioso",
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum Idioma {
    Latim,
    Portugues,
}

impl FromStr for Idioma {
    type Err = String;
    fn from_str(texto: &str) -> Result<Self, String> {
        match texto.to_ascii_lowercase().as_str() {
            "la" | "latim" | "lorem" => Ok(Idioma::Latim),
            "pt" | "portugues" | "português" => Ok(Idioma::Portugues),
            outro => Err(format!("idioma desconhecido '{outro}': use la (latim) ou pt (português)")),
        }
    }
}

/// Xorshift64: três deslocamentos e três ou-exclusivos. Com a mesma semente
/// a sequência é sempre a mesma, e é isso que torna o texto repetível.
struct Sorteio(u64);

impl Sorteio {
    fn com_semente(semente: u64) -> Self {
        // Zero travaria o xorshift em zero para sempre, então trocamos por uma constante.
        Sorteio(if semente == 0 { 0x2545_F491_4F6C_DD1D } else { semente })
    }

    fn proximo(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// Inteiro entre `menor` e `maior`, os dois inclusos.
    fn entre(&mut self, menor: usize, maior: usize) -> usize {
        menor + (self.proximo() % (maior - menor + 1) as u64) as usize
    }
}

struct Gerador {
    sorteio: Sorteio,
    dicionario: &'static [&'static str],
    idioma: Idioma,
    primeira_frase: bool,
}

impl Gerador {
    fn novo(idioma: Idioma, semente: u64) -> Self {
        let dicionario = match idioma {
            Idioma::Latim => LATIM,
            Idioma::Portugues => PORTUGUES,
        };
        Gerador { sorteio: Sorteio::com_semente(semente), dicionario, idioma, primeira_frase: true }
    }

    fn palavra(&mut self) -> &'static str {
        self.dicionario[self.sorteio.entre(0, self.dicionario.len() - 1)]
    }

    fn palavras(&mut self, quantas: usize) -> Vec<&'static str> {
        (0..quantas).map(|_| self.palavra()).collect()
    }

    /// Uma frase tem de 6 a 14 palavras, às vezes uma vírgula no meio,
    /// primeira letra maiúscula e ponto final. Em latim, a primeira frase
    /// gerada é sempre a abertura clássica, como manda a tradição.
    fn frase(&mut self) -> String {
        if self.idioma == Idioma::Latim && self.primeira_frase {
            self.primeira_frase = false;
            return "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".to_string();
        }
        self.primeira_frase = false;
        let quantas = self.sorteio.entre(6, 14);
        let virgula = if quantas >= 9 { Some(self.sorteio.entre(3, quantas - 3)) } else { None };
        let mut frase = String::new();
        for (i, palavra) in self.palavras(quantas).into_iter().enumerate() {
            if i == 0 {
                frase.push_str(&capitalizar(palavra));
            } else {
                frase.push(' ');
                frase.push_str(palavra);
            }
            if virgula == Some(i + 1) {
                frase.push(',');
            }
        }
        frase.push('.');
        frase
    }

    /// Um parágrafo tem de 3 a 6 frases.
    fn paragrafo(&mut self) -> String {
        let quantas = self.sorteio.entre(3, 6);
        (0..quantas).map(|_| self.frase()).collect::<Vec<String>>().join(" ")
    }
}

/// to_uppercase devolve um iterador porque uma letra pode virar duas.
fn capitalizar(palavra: &str) -> String {
    let mut letras = palavra.chars();
    match letras.next() {
        Some(primeira) => primeira.to_uppercase().chain(letras).collect(),
        None => String::new(),
    }
}

fn semente_do_relogio() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(7)
}

fn gerar(tipo: &str, quantidade: usize, idioma: Idioma, semente: u64) -> Result<String, String> {
    let mut gerador = Gerador::novo(idioma, semente);
    let saida = match tipo {
        "paragrafos" => (0..quantidade).map(|_| gerador.paragrafo()).collect::<Vec<String>>().join("\n\n"),
        "frases" => (0..quantidade).map(|_| gerador.frase()).collect::<Vec<String>>().join("\n"),
        "palavras" => gerador.palavras(quantidade).join(" "),
        outro => return Err(format!("tipo desconhecido '{outro}': use paragrafos, frases ou palavras")),
    };
    Ok(saida)
}

fn demo() -> Result<(), String> {
    println!("=== Gerador de texto de preenchimento: demonstração ===\n");
    let semente = 2024;
    for (idioma, nome) in [(Idioma::Latim, "latim"), (Idioma::Portugues, "português")] {
        println!("--- Dois parágrafos em {nome} (semente {semente}) ---");
        let texto = gerar("paragrafos", 2, idioma, semente)?;
        println!("{texto}\n");
        println!("({} palavras, {} frases)\n", texto.split_whitespace().count(), texto.matches('.').count());
    }
    println!("--- Doze palavras em português ---");
    println!("{}\n", gerar("palavras", 12, Idioma::Portugues, semente)?);
    println!("--- A mesma semente repete o texto ---");
    let (a, b) = (gerar("frases", 1, Idioma::Portugues, 99)?, gerar("frases", 1, Idioma::Portugues, 99)?);
    println!("{a}\n{b}\niguais: {}", a == b);
    Ok(())
}

fn uso() {
    println!("Uso: 20_gerador_lorem.exe <paragrafos|frases|palavras> <quantidade> [opções]");
    println!("  --idioma la|pt     latim (padrão) ou português");
    println!("  --semente N        repete exatamente o mesmo texto");
    println!("  --demo             demonstração completa");
}

fn executar(args: &[String]) -> Result<(), String> {
    let [tipo, quantidade, opcoes @ ..] = args else {
        match args.first().map(String::as_str) {
            Some("--demo") => return demo(),
            Some(outro) => return Err(format!("faltou a quantidade depois de '{outro}'")),
            None => {
                uso();
                return Ok(());
            }
        }
    };
    let quantidade: usize = quantidade.parse().map_err(|_| format!("quantidade inválida: '{quantidade}'"))?;
    if !(1..=1000).contains(&quantidade) {
        return Err("a quantidade precisa ficar entre 1 e 1000".to_string());
    }
    let mut idioma = Idioma::Latim;
    let mut semente = semente_do_relogio();
    let mut restante = opcoes.iter();
    while let Some(opcao) = restante.next() {
        let valor = restante.next().ok_or_else(|| format!("{opcao} precisa de um valor"))?;
        match opcao.as_str() {
            "--idioma" => idioma = valor.parse()?,
            "--semente" => semente = valor.parse().map_err(|_| format!("semente inválida: '{valor}' (use um inteiro)"))?,
            outra => return Err(format!("opção desconhecida '{outra}': use --idioma ou --semente")),
        }
    }
    println!("{}", gerar(tipo, quantidade, idioma, semente)?);
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
// 1. Adicione a opção --html para envolver cada parágrafo em <p> e </p>,
//    pronto para colar em uma página.
// 2. Carregue o dicionário de um arquivo (uma palavra por linha) com
//    --dicionario caminho.txt e trate o erro de leitura.
// 3. Faça as frases em português respeitarem uma ordem mínima (artigo,
//    substantivo, verbo) separando o dicionário em três listas.
