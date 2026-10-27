// ============================================================
// CodeNomad Academy  |  Sources Rust  |  18. Conversor de números romanos
// ------------------------------------------------------------
// O que faz: Converte inteiros de 1 a 3999 para algarismos romanos e
//            romanos para inteiros, rejeita formas mal construídas
//            (IIII, VX, IC, MMMM) e mostra uma tabela de referência.
// Como usar: 18_numeros_romanos.exe 1994 MMXXIV 49 xlii
//            18_numeros_romanos.exe tabela
//            18_numeros_romanos.exe --demo
// O que você aprende: implementar os traits FromStr, Display e TryFrom
//            para um tipo próprio (newtype), tabela de constantes com
//            tuplas, collect em Result<Vec, _>, enum de erro com Display,
//            parse::<Romano>() e try_into().
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::fmt;
use std::str::FromStr;

/// Os treze "blocos" do sistema romano, do maior para o menor. Os pares
/// subtrativos (CM, CD, XC, XL, IX, IV) entram como blocos próprios:
/// assim 900 vira "CM" direto, sem regra especial no algoritmo.
const TABELA: [(u32, &str); 13] = [
    (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"), (50, "L"),
    (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
];

/// Um "newtype": embrulha o u32 para que só valores de 1 a 3999 existam.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Romano(u32);

#[derive(Debug, PartialEq)]
enum Erro {
    ForaDaFaixa(i64),
    Vazio,
    SimboloInvalido(char),
    MalFormado { lido: String, correto: String },
}

impl fmt::Display for Erro {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Erro::ForaDaFaixa(n) => write!(f, "{n} fora da faixa: os romanos clássicos vão de 1 a 3999"),
            Erro::Vazio => write!(f, "texto vazio"),
            Erro::SimboloInvalido(c) => write!(f, "'{c}' não é um algarismo romano (use I, V, X, L, C, D, M)"),
            Erro::MalFormado { lido, correto } => write!(f, "'{lido}' está mal formado; a escrita correta é '{correto}'"),
        }
    }
}

/// Inteiro para Romano: só entra quem está na faixa.
impl TryFrom<i64> for Romano {
    type Error = Erro;
    fn try_from(n: i64) -> Result<Self, Erro> {
        if (1..=3999).contains(&n) { Ok(Romano(n as u32)) } else { Err(Erro::ForaDaFaixa(n)) }
    }
}

/// Romano para texto: percorre a tabela do maior bloco para o menor,
/// subtraindo enquanto couber. É o algoritmo "guloso" clássico.
impl fmt::Display for Romano {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut resto = self.0;
        let mut saida = String::new();
        for (valor, letras) in TABELA {
            while resto >= valor {
                saida.push_str(letras);
                resto -= valor;
            }
        }
        // `pad` (e não `write!`) faz "{:<10}" alinhar o número na tabela.
        f.pad(&saida)
    }
}

fn valor_do_simbolo(c: char) -> Result<u32, Erro> {
    match c {
        'I' => Ok(1),
        'V' => Ok(5),
        'X' => Ok(10),
        'L' => Ok(50),
        'C' => Ok(100),
        'D' => Ok(500),
        'M' => Ok(1000),
        outro => Err(Erro::SimboloInvalido(outro)),
    }
}

/// Texto para Romano. Regra de leitura: cada símbolo soma, a menos que o
/// seguinte seja maior (aí ele subtrai, como o I de IV). A validação vem
/// depois: reescrevemos o valor lido e comparamos com o texto original.
/// Se diferem ("IIII" em vez de "IV"), a entrada estava mal formada.
impl FromStr for Romano {
    type Err = Erro;
    fn from_str(texto: &str) -> Result<Self, Erro> {
        let texto = texto.trim().to_ascii_uppercase();
        if texto.is_empty() {
            return Err(Erro::Vazio);
        }
        // collect em Result: para no primeiro símbolo inválido e devolve o erro.
        let valores: Vec<u32> = texto.chars().map(valor_do_simbolo).collect::<Result<_, _>>()?;
        let mut total: i64 = 0;
        for (i, &atual) in valores.iter().enumerate() {
            let seguinte = valores.get(i + 1).copied().unwrap_or(0);
            if atual < seguinte { total -= i64::from(atual) } else { total += i64::from(atual) }
        }
        let romano = Romano::try_from(total)?;
        let correto = romano.to_string();
        if correto != texto {
            return Err(Erro::MalFormado { lido: texto, correto });
        }
        Ok(romano)
    }
}

/// Decide pela cara do argumento: se é um inteiro, vai para try_into;
/// senão é lido como romano com parse (que chama o FromStr acima).
fn converter(entrada: &str) -> Result<Romano, Erro> {
    let limpo = entrada.trim();
    match limpo.parse::<i64>() {
        Ok(n) => n.try_into(),
        Err(_) => limpo.parse(),
    }
}

fn mostrar(entrada: &str) -> bool {
    match converter(entrada) {
        Ok(romano) => {
            println!("  {entrada:<12} ->  {:>4}  =  {romano}", romano.0);
            true
        }
        Err(erro) => {
            println!("  {entrada:<12} ->  inválido: {erro}");
            false
        }
    }
}

fn tabela() {
    println!("Símbolos e blocos (os pares subtrativos valem como um bloco só):");
    for linha in TABELA.chunks(7) {
        let itens: Vec<String> = linha.iter().map(|(v, s)| format!("{s:>2} = {v:<4}")).collect();
        println!("  {}", itens.join("  "));
    }
    println!("\nRegras: no máximo três repetições seguidas (III, XXX), nunca VV, LL ou DD,");
    println!("e só I, X e C subtraem, sempre do próximo bloco (IV, IX, XL, XC, CD, CM).\n");
    println!("Referência rápida:");
    let exemplos = [1, 4, 9, 14, 40, 90, 400, 500, 1000, 1994, 2024, 3999];
    for grupo in exemplos.chunks(4) {
        let itens: Vec<String> = grupo.iter().map(|&n| format!("{n:>4} = {:<10}", Romano(n))).collect();
        println!("  {}", itens.join(" "));
    }
}

fn demo() {
    println!("=== Conversor de números romanos: demonstração ===\n");
    println!("Inteiro para romano e romano para inteiro:");
    let casos = ["1", "4", "9", "14", "40", "90", "400", "1994", "2024", "3999", "MMXXIV", "xlii", "MCMXC", "cdxliv"];
    let ok = casos.iter().filter(|c| mostrar(c)).count();
    println!("  {ok} de {} convertidos.\n", casos.len());

    println!("Entradas rejeitadas (e o motivo):");
    for ruim in ["IIII", "VX", "IC", "MMMM", "0", "4000", "ABC", "   "] {
        mostrar(ruim);
    }
    println!();
    tabela();
}

fn uso() {
    println!("Uso:");
    println!("  18_numeros_romanos.exe <número ou romano> [outro ...]   converte nos dois sentidos");
    println!("  18_numeros_romanos.exe tabela                            tabela de referência");
    println!("  18_numeros_romanos.exe --demo                            demonstração completa");
}

fn executar(args: &[String]) -> Result<(), String> {
    match args {
        [] => uso(),
        [comando] if comando == "--demo" => demo(),
        [comando] if comando == "tabela" => tabela(),
        entradas => {
            // fold em vez de all: all pararia na primeira falha e esconderia as outras.
            let todas_ok = entradas.iter().fold(true, |acc, e| mostrar(e) && acc);
            if !todas_ok {
                return Err("pelo menos uma entrada não pôde ser convertida".to_string());
            }
        }
    }
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
// 1. Aceite a notação com traço (vinculum) para milhares, em que "V" com
//    traço vale 5000, e estenda a faixa até 3.999.999.
// 2. Implemente `impl std::ops::Add for Romano` com verificação de faixa,
//    para poder escrever `Romano(10) + Romano(5)`.
// 3. Escreva testes (#[test]) que convertem todos os números de 1 a 3999
//    para romano e de volta, conferindo que o ciclo devolve o original.
