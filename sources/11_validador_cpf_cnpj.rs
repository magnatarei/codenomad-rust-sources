// ============================================================
// CodeNomad Academy  |  Sources Rust  |  11. Validador de CPF e CNPJ
// ------------------------------------------------------------
// O que faz: Valida os dígitos verificadores de CPF e CNPJ (inclusive o
//            CNPJ alfanumérico), aceita com ou sem pontuação, rejeita
//            sequências repetidas, formata o número e gera números válidos
//            para usar em testes.
// Como usar: 11_validador_cpf_cnpj.exe 123.456.789-09 11222333000181
//            11_validador_cpf_cnpj.exe gerar cpf 3
//            11_validador_cpf_cnpj.exe --demo
// O que você aprende: enums com Display, Result com erros descritivos,
//            iteradores com zip e sum, fatias de string, struct com impl,
//            um gerador pseudoaleatório simples (xorshift).
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::collections::hash_map::RandomState;
use std::fmt;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

// Pesos oficiais de cada dígito verificador. O CPF usa pesos decrescentes;
// o CNPJ usa uma tabela fixa que "reinicia" no meio.
const PESOS_CPF_1: [u32; 9] = [10, 9, 8, 7, 6, 5, 4, 3, 2];
const PESOS_CPF_2: [u32; 10] = [11, 10, 9, 8, 7, 6, 5, 4, 3, 2];
const PESOS_CNPJ_1: [u32; 12] = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
const PESOS_CNPJ_2: [u32; 13] = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tipo {
    Cpf,
    Cnpj,
}

impl fmt::Display for Tipo {
    // `f.pad` (e não `write!`) respeita largura e alinhamento, como em "{:<5}".
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(match self {
            Tipo::Cpf => "CPF",
            Tipo::Cnpj => "CNPJ",
        })
    }
}

/// Um documento já validado: só dígitos (e letras, no CNPJ), sem pontuação.
struct Documento {
    tipo: Tipo,
    limpo: String,
}

impl Documento {
    /// Devolve o número com a pontuação usual. As fatias por byte são seguras
    /// porque `limpo` só contém símbolos ASCII.
    fn formatado(&self) -> String {
        let s = &self.limpo;
        match self.tipo {
            Tipo::Cpf => format!("{}.{}.{}-{}", &s[0..3], &s[3..6], &s[6..9], &s[9..11]),
            Tipo::Cnpj => format!("{}.{}.{}/{}-{}", &s[0..2], &s[2..5], &s[5..8], &s[8..12], &s[12..14]),
        }
    }
}

/// Mantém apenas letras e dígitos, em maiúsculas. Pontos, traços e barras
/// são descartados, então "123.456.789-09" e "12345678909" viram iguais.
fn limpar(entrada: &str) -> String {
    entrada.chars().filter(char::is_ascii_alphanumeric).map(|c| c.to_ascii_uppercase()).collect()
}

/// Valor de cada símbolo no cálculo: '0'..'9' valem 0..9 e, no CNPJ
/// alfanumérico, 'A'..'Z' valem 17..42 (código ASCII menos 48).
fn valor(c: char) -> u32 {
    c as u32 - '0' as u32
}

/// Regra comum aos dois documentos: soma ponderada, resto por 11,
/// e restos 0 ou 1 viram dígito 0.
fn digito_verificador(valores: &[u32], pesos: &[u32]) -> u32 {
    let soma: u32 = valores.iter().zip(pesos).map(|(v, p)| v * p).sum();
    let resto = soma % 11;
    if resto < 2 { 0 } else { 11 - resto }
}

/// Calcula os dois dígitos verificadores a partir da parte "base" do número.
fn calcular_digitos(base: &[u32], tipo: Tipo) -> (u32, u32) {
    let (pesos1, pesos2): (&[u32], &[u32]) = match tipo {
        Tipo::Cpf => (&PESOS_CPF_1, &PESOS_CPF_2),
        Tipo::Cnpj => (&PESOS_CNPJ_1, &PESOS_CNPJ_2),
    };
    let d1 = digito_verificador(base, pesos1);
    let mut com_d1 = base.to_vec();
    com_d1.push(d1);
    (d1, digito_verificador(&com_d1, pesos2))
}

fn validar(entrada: &str) -> Result<Documento, String> {
    let limpo = limpar(entrada);
    let tipo = match limpo.len() {
        11 => Tipo::Cpf,
        14 => Tipo::Cnpj,
        n => return Err(format!("esperava 11 dígitos (CPF) ou 14 (CNPJ), encontrei {n}")),
    };
    let simbolos: Vec<char> = limpo.chars().collect();
    let (base, verificadores) = simbolos.split_at(simbolos.len() - 2);
    if tipo == Tipo::Cpf && base.iter().any(|c| !c.is_ascii_digit()) {
        return Err("CPF só pode conter dígitos".to_string());
    }
    if verificadores.iter().any(|c| !c.is_ascii_digit()) {
        return Err("os dois dígitos verificadores precisam ser numéricos".to_string());
    }
    if base.iter().all(|c| *c == base[0]) {
        return Err("sequência repetida (todos os símbolos iguais) não é um documento válido".to_string());
    }
    let valores: Vec<u32> = base.iter().map(|c| valor(*c)).collect();
    let (d1, d2) = calcular_digitos(&valores, tipo);
    let (r1, r2) = (valor(verificadores[0]), valor(verificadores[1]));
    if (d1, d2) != (r1, r2) {
        return Err(format!("dígitos verificadores errados: esperava {d1}{d2}, encontrei {r1}{r2}"));
    }
    Ok(Documento { tipo, limpo })
}

/// Gerador xorshift64: suficiente para dados de teste. A semente vem do
/// RandomState (aleatório por processo) misturada com o relógio.
struct Sorteio(u64);

impl Sorteio {
    fn novo() -> Self {
        let relogio = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        let semente = RandomState::new().build_hasher().finish() ^ relogio;
        Sorteio(if semente == 0 { 0x9E37_79B9_7F4A_7C15 } else { semente })
    }

    fn proximo(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn digito(&mut self) -> u32 {
        (self.proximo() % 10) as u32
    }
}

/// Gera um documento válido: sorteia a base, calcula os verificadores.
/// No CNPJ a base termina em "0001", a filial matriz mais comum.
fn gerar(tipo: Tipo, sorteio: &mut Sorteio) -> Documento {
    loop {
        let mut base: Vec<u32> = match tipo {
            Tipo::Cpf => (0..9).map(|_| sorteio.digito()).collect(),
            Tipo::Cnpj => (0..8).map(|_| sorteio.digito()).chain([0, 0, 0, 1]).collect(),
        };
        if base.iter().all(|d| *d == base[0]) {
            continue; // raríssimo, mas uma sequência repetida seria rejeitada
        }
        let (d1, d2) = calcular_digitos(&base, tipo);
        base.extend([d1, d2]);
        let limpo: String = base.iter().map(|d| char::from_digit(*d, 10).unwrap_or('0')).collect();
        return Documento { tipo, limpo };
    }
}

fn mostrar(entrada: &str) -> bool {
    match validar(entrada) {
        Ok(doc) => {
            println!("  {entrada:<22} VÁLIDO    {} {}", doc.tipo, doc.formatado());
            true
        }
        Err(motivo) => {
            println!("  {entrada:<22} INVÁLIDO  {motivo}");
            false
        }
    }
}

fn demo() {
    println!("=== Validador de CPF e CNPJ: demonstração ===\n");
    println!("Validando uma lista de entradas:");
    let casos = [
        "123.456.789-09", "12345678909", "111.111.111-11", "123.456.789-00",
        "11.222.333/0001-81", "11222333000181", "11.222.333/0001-82",
        "12.ABC.345/01DE-35", "12345", "ab.cde.fgh-ij",
    ];
    let validos = casos.iter().filter(|c| mostrar(c)).count();
    println!("\n{validos} válidos de {} testados.\n", casos.len());

    println!("Gerando números válidos para teste (mudam a cada execução):");
    let mut sorteio = Sorteio::novo();
    for tipo in [Tipo::Cpf, Tipo::Cnpj] {
        let doc = gerar(tipo, &mut sorteio);
        let confere = validar(&doc.limpo).is_ok();
        println!("  {:<5} {}  (revalidado: {})", doc.tipo, doc.formatado(), if confere { "ok" } else { "falhou" });
    }
}

fn uso() {
    println!("Uso:");
    println!("  11_validador_cpf_cnpj.exe <número> [outro número ...]   valida CPF ou CNPJ");
    println!("  11_validador_cpf_cnpj.exe gerar cpf|cnpj [quantidade]   gera números de teste");
    println!("  11_validador_cpf_cnpj.exe --demo                        demonstração completa");
}

fn executar(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        None => uso(),
        Some("--demo") => demo(),
        Some("gerar") => {
            let tipo = match args.get(1).map(|s| s.to_ascii_lowercase()).as_deref() {
                Some("cpf") => Tipo::Cpf,
                Some("cnpj") => Tipo::Cnpj,
                Some(outro) => return Err(format!("tipo desconhecido '{outro}': use cpf ou cnpj")),
                None => return Err("informe o tipo: gerar cpf ou gerar cnpj".to_string()),
            };
            let quantidade: usize = match args.get(2) {
                Some(q) => q.parse().map_err(|_| format!("quantidade inválida: '{q}'"))?,
                None => 1,
            };
            let mut sorteio = Sorteio::novo();
            for _ in 0..quantidade.clamp(1, 100) {
                println!("{}", gerar(tipo, &mut sorteio).formatado());
            }
        }
        Some(_) => {
            let todos_validos = args.iter().fold(true, |acc, a| mostrar(a) && acc);
            if !todos_validos {
                return Err("pelo menos uma entrada é inválida".to_string());
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
// 1. Aceite o CNPJ com prefixo "CNPJ:" ou "CPF:" e use o prefixo para forçar
//    o tipo, em vez de descobrir pelo tamanho.
// 2. Leia um arquivo com um documento por linha e imprima um resumo:
//    quantos válidos, quantos inválidos e os motivos mais comuns.
// 3. Transforme `validar` em um `impl FromStr for Documento`, para poder
//    escrever `"123.456.789-09".parse::<Documento>()`.
