// ============================================================
// CodeNomad Academy  |  Sources Rust  |  39. Diário de hábitos com sequência
// ------------------------------------------------------------
// O que faz: Registra hábitos feitos em cada dia num arquivo de texto simples,
//            calcula a sequência atual e a maior sequência de cada hábito e
//            desenha um resumo semanal em grade de texto (X = feito).
// Como usar: rustc -O --edition 2021 39_diario_habitos.rs -o habitos.exe
//            habitos.exe registrar leitura                (hoje)
//            habitos.exe registrar "corrida 5 km" 2026-10-25
//            habitos.exe resumo [--hoje 2026-10-27] [--arquivo habitos.txt]
//            habitos.exe --demo
// O que você aprende: datas sem crate (dias desde 1970 e de volta, dia da
//            semana, SystemTime), BTreeMap e BTreeSet para dados ordenados,
//            iteradores com rev, take_while e windows, e formatação em grade.
// Requisitos: Rust 1.70+, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const USO: &str = "Uso: habitos <comando> [opções] [--arquivo habitos.txt]
  registrar <hábito> [AAAA-MM-DD]   marca o hábito como feito (padrão: hoje)
  resumo [--hoje AAAA-MM-DD]        sequências e grade da última semana
  listar                            mostra todos os registros em ordem
  --demo                            roda uma demonstração completa";
const DIAS_SEMANA: [&str; 7] = ["seg", "ter", "qua", "qui", "sex", "sáb", "dom"];

// Dias corridos desde 1970-01-01. O truque: o ano "começa" em março, assim
// fevereiro (o mês irregular) fica por último e a conta dos meses vira uma fórmula.
fn dias_desde_1970(ano: i64, mes: i64, dia: i64) -> i64 {
    let ano = if mes <= 2 { ano - 1 } else { ano };
    let era = ano.div_euclid(400);
    let ano_da_era = ano - era * 400;
    let mes_deslocado = (mes + 9) % 12;
    let dia_do_ano = (153 * mes_deslocado + 2) / 5 + dia - 1;
    let dia_da_era = ano_da_era * 365 + ano_da_era / 4 - ano_da_era / 100 + dia_do_ano;
    era * 146097 + dia_da_era - 719468
}

// O caminho inverso: de dias corridos para (ano, mês, dia).
fn data_de_dias(dias: i64) -> (i64, i64, i64) {
    let z = dias + 719468;
    let era = z.div_euclid(146097);
    let dia_da_era = z - era * 146097;
    let ano_da_era = (dia_da_era - dia_da_era / 1460 + dia_da_era / 36524 - dia_da_era / 146096) / 365;
    let dia_do_ano = dia_da_era - (365 * ano_da_era + ano_da_era / 4 - ano_da_era / 100);
    let mes_deslocado = (5 * dia_do_ano + 2) / 153;
    let dia = dia_do_ano - (153 * mes_deslocado + 2) / 5 + 1;
    let mes = if mes_deslocado < 10 { mes_deslocado + 3 } else { mes_deslocado - 9 };
    (ano_da_era + era * 400 + i64::from(mes <= 2), mes, dia)
}

fn formatar_data(dias: i64) -> String {
    let (a, m, d) = data_de_dias(dias);
    format!("{a:04}-{m:02}-{d:02}")
}

// Valida convertendo para dias e voltando: "2026-02-30" vira outra data e é recusada.
fn ler_data(texto: &str) -> Result<i64, String> {
    let partes: Vec<i64> = texto.split('-').filter_map(|p| p.parse().ok()).collect();
    let dias = match partes.as_slice() {
        [a, m, d] if (1..=12).contains(m) && (1..=31).contains(d) => dias_desde_1970(*a, *m, *d),
        _ => return Err(format!("data inválida: '{texto}' (use AAAA-MM-DD)")),
    };
    if formatar_data(dias) != texto { return Err(format!("a data '{texto}' não existe no calendário")); }
    Ok(dias)
}

fn hoje() -> i64 {
    let segundos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    (segundos / 86_400) as i64
}

// 1970-01-01 foi uma quinta-feira; somando 3, segunda-feira vira o índice 0.
fn dia_da_semana(dias: i64) -> usize { (dias + 3).rem_euclid(7) as usize }

// Cada hábito aponta para o conjunto de dias em que foi feito; os dois ficam ordenados.
type Diario = BTreeMap<String, BTreeSet<i64>>;

fn carregar(caminho: &Path) -> Result<Diario, String> {
    let mut diario = Diario::new();
    if !caminho.exists() { return Ok(diario); }
    let texto = fs::read_to_string(caminho).map_err(|e| format!("não consegui ler {}: {e}", caminho.display()))?;
    for (n, linha) in texto.lines().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
        let (data, habito) = linha.split_once(' ').ok_or(format!("linha {}: esperava 'AAAA-MM-DD hábito'", n + 1))?;
        let dias = ler_data(data).map_err(|e| format!("linha {}: {e}", n + 1))?;
        diario.entry(habito.trim().to_lowercase()).or_default().insert(dias);
    }
    Ok(diario)
}

fn salvar(caminho: &Path, diario: &Diario) -> Result<(), String> {
    let mut linhas: Vec<(i64, &str)> = diario.iter().flat_map(|(h, dias)| dias.iter().map(move |d| (*d, h.as_str()))).collect();
    linhas.sort();
    let texto: String = linhas.iter().map(|(d, h)| format!("{} {h}\n", formatar_data(*d))).collect();
    fs::write(caminho, texto).map_err(|e| format!("não consegui gravar {}: {e}", caminho.display()))
}

// Sequência atual: conta para trás a partir de hoje (ou de ontem, se hoje ainda não foi feito).
fn sequencia_atual(dias: &BTreeSet<i64>, referencia: i64) -> usize {
    let inicio = if dias.contains(&referencia) { referencia } else { referencia - 1 };
    (0..).take_while(|n| dias.contains(&(inicio - n))).count()
}

// Maior sequência: windows(2) olha cada par de dias vizinhos e vê se são consecutivos.
fn maior_sequencia(dias: &BTreeSet<i64>) -> usize {
    let lista: Vec<i64> = dias.iter().copied().collect();
    let (mut maior, mut atual) = (lista.len().min(1), 1);
    for par in lista.windows(2) {
        atual = if par[1] == par[0] + 1 { atual + 1 } else { 1 };
        maior = maior.max(atual);
    }
    maior
}

fn resumo(diario: &Diario, referencia: i64) {
    if diario.is_empty() { println!("Nenhum registro ainda. Use: registrar <hábito>"); return; }
    let largura = diario.keys().map(|h| h.chars().count()).max().unwrap_or(6).max(6);
    println!("=== Resumo até {} ===", formatar_data(referencia));
    println!("{:<largura$}  {:>5}  {:>7}  {:>5}  {}", "hábito", "atual", "recorde", "total", "última vez");
    for (habito, dias) in diario {
        let ultima = dias.iter().next_back().map(|d| formatar_data(*d)).unwrap_or_default();
        println!("{habito:<largura$}  {:>5}  {:>7}  {:>5}  {ultima}", sequencia_atual(dias, referencia), maior_sequencia(dias), dias.len());
    }
    let semana: Vec<i64> = (referencia - 6..=referencia).collect();
    println!("\n--- Semana de {} a {} ---", formatar_data(semana[0]), formatar_data(referencia));
    let cabecalho: Vec<&str> = semana.iter().map(|d| DIAS_SEMANA[dia_da_semana(*d)]).collect();
    println!("{:<largura$}  {}", "", cabecalho.join(" "));
    for (habito, dias) in diario {
        let marcas: Vec<&str> = semana.iter().map(|d| if dias.contains(d) { " X " } else { " . " }).collect();
        let feitos = semana.iter().filter(|d| dias.contains(d)).count();
        println!("{habito:<largura$}  {}  {feitos}/7", marcas.join(" "));
    }
}

fn executar(args: &[String]) -> Result<(), String> {
    let mut caminho = PathBuf::from("habitos.txt");
    let mut referencia = hoje();
    let mut resto: Vec<&str> = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--arquivo" => caminho = PathBuf::from(iter.next().ok_or("--arquivo precisa de um caminho")?),
            "--hoje" => referencia = ler_data(iter.next().ok_or("--hoje precisa de uma data")?)?,
            outro => resto.push(outro),
        }
    }
    let mut diario = carregar(&caminho)?;
    match resto.as_slice() {
        ["registrar", habito, data @ ..] => {
            let habito = habito.trim().to_lowercase();
            if habito.is_empty() { return Err("o nome do hábito não pode ficar vazio".into()); }
            let dias = match data { [] => referencia, [d] => ler_data(d)?, _ => return Err("uso: registrar <hábito> [AAAA-MM-DD]".into()) };
            if dias > referencia { return Err(format!("{} ainda não chegou; não dá para registrar o futuro", formatar_data(dias))); }
            let novo = diario.entry(habito.clone()).or_default().insert(dias);
            salvar(&caminho, &diario)?;
            println!("{}: '{habito}' em {}.", if novo { "Registrado" } else { "Já estava registrado" }, formatar_data(dias));
        }
        ["resumo"] => resumo(&diario, referencia),
        ["listar"] => {
            let total: usize = diario.values().map(BTreeSet::len).sum();
            println!("{total} registro(s) em {}:", caminho.display());
            for (habito, dias) in &diario {
                let lista: Vec<String> = dias.iter().map(|d| formatar_data(*d)).collect();
                println!("  {habito}: {}", lista.join(", "));
            }
        }
        _ => return Err(format!("comando inválido: '{}'\n{USO}", resto.join(" "))),
    }
    Ok(())
}

fn demo() -> Result<(), String> {
    let arquivo = std::env::temp_dir().join(format!("codenomad_habitos_{}.txt", std::process::id()));
    let caminho = arquivo.to_string_lossy().to_string();
    let hoje_demo = "2026-10-27";
    let referencia = ler_data(hoje_demo)?;
    println!("=== Demonstração: diário de hábitos ===");
    println!("Arquivo temporário: {caminho}");
    println!("Data de referência: {hoje_demo} ({}), com três semanas de registros.\n", DIAS_SEMANA[dia_da_semana(referencia)]);
    // Três hábitos com padrões diferentes nos últimos 21 dias: todo dia, dias pares e só dias úteis.
    let mut diario = Diario::new();
    for n in 0..21 {
        let dia = referencia - n;
        if n != 2 { diario.entry("leitura".into()).or_default().insert(dia); }
        if n % 2 == 0 { diario.entry("corrida 5 km".into()).or_default().insert(dia); }
        if dia_da_semana(dia) < 5 && n > 0 { diario.entry("estudar rust".into()).or_default().insert(dia); }
    }
    salvar(&arquivo, &diario)?;
    let passos: [&[&str]; 5] = [
        &["listar"], &["resumo"], &["registrar", "estudar rust"], &["registrar", "leitura", "2026-10-30"], &["resumo"],
    ];
    for passo in passos {
        println!("$ habitos {}", passo.join(" "));
        let mut args: Vec<String> = passo.iter().map(|s| s.to_string()).collect();
        args.extend(["--arquivo".into(), caminho.clone(), "--hoje".into(), hoje_demo.into()]);
        if let Err(e) = executar(&args) { println!("Erro: {e}"); }
        println!();
    }
    let _ = fs::remove_file(&arquivo);
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None => { println!("{USO}"); Ok(()) }
        Some("--demo") => demo(),
        Some(_) => executar(&args),
    };
    if let Err(erro) = resultado { eprintln!("Erro: {erro}"); std::process::exit(1); }
}

// ------------------------------------------------------------
// Para ir além
// 1. Crie o comando "desmarcar <hábito> <data>" para corrigir um registro feito por engano.
// 2. Mostre a grade do mês inteiro, com uma linha por semana, usando data_de_dias para achar o dia 1.
// 3. Calcule a taxa de acerto dos últimos 30 dias e avise quando uma sequência estiver prestes a quebrar.
// ------------------------------------------------------------
