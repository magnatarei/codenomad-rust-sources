// ============================================================
// CodeNomad Academy  |  Sources Rust  |  09. Calculadora de datas
// ------------------------------------------------------------
// O que faz: calcula dias entre duas datas, o dia da semana de uma data, a
//            idade exata em anos, meses e dias, soma ou subtrai dias de uma
//            data e diz quantos dias faltam para uma data futura.
// Como usar: rustc -O --edition 2021 -D warnings 09_calculadora_datas.rs -o datas.exe
//            datas.exe entre 01/01/2024 25/12/2024      (dias entre duas datas)
//            datas.exe semana 15/11/1889                (dia da semana)
//            datas.exe idade 15/03/1990                 (idade exata até hoje)
//            datas.exe somar 31/01/2024 30              (negativo para subtrair)
//            datas.exe faltam 25/12/2026                (dias até a data, contando de hoje)
//            datas.exe --demo
// O que você aprende: struct com derive, implementar os traits FromStr e
//            Display, converter data em número de dias (e voltar) sem biblioteca
//            de calendário, div_euclid e rem_euclid, SystemTime para saber hoje.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::fmt;
use std::process::ExitCode;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

const DIAS_SEMANA: [&str; 7] =
    ["domingo", "segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado"];

/// Uma data do calendário. Copy porque são só três inteiros: passar por valor é mais simples.
#[derive(Clone, Copy, PartialEq)]
struct Data {
    ano: i64,
    mes: u32,
    dia: u32,
}

fn bissexto(ano: i64) -> bool {
    (ano % 4 == 0 && ano % 100 != 0) || ano % 400 == 0
}

fn dias_no_mes(ano: i64, mes: u32) -> u32 {
    match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if bissexto(ano) { 29 } else { 28 },
        _ => 0,
    }
}

impl Data {
    /// Só cria a data se ela existir de verdade: 30/02 e 31/04 param aqui.
    fn nova(ano: i64, mes: u32, dia: u32) -> Result<Data, String> {
        if !(1..=9999).contains(&ano) {
            return Err(format!("ano fora da faixa de 1 a 9999: {ano}"));
        }
        if !(1..=12).contains(&mes) {
            return Err(format!("mês inválido: {mes} (use de 1 a 12)"));
        }
        let limite = dias_no_mes(ano, mes);
        if dia == 0 || dia > limite {
            return Err(format!("o mês {mes:02}/{ano} tem {limite} dias, recebi o dia {dia}"));
        }
        Ok(Data { ano, mes, dia })
    }

    /// Converte a data em "número de dias desde 01/01/1970". Com isso, subtrair
    /// datas vira subtrair inteiros. O truque de começar o ano em março deixa
    /// fevereiro, o único mês de tamanho variável, por último na conta.
    fn em_dias(self) -> i64 {
        let ano = if self.mes <= 2 { self.ano - 1 } else { self.ano };
        let era = ano.div_euclid(400);
        let ano_da_era = ano - era * 400;
        let mes_deslocado = i64::from((self.mes + 9) % 12);
        let dia_do_ano = (153 * mes_deslocado + 2) / 5 + i64::from(self.dia) - 1;
        let dia_da_era = ano_da_era * 365 + ano_da_era / 4 - ano_da_era / 100 + dia_do_ano;
        era * 146_097 + dia_da_era - 719_468
    }

    /// O caminho de volta: de "dias desde 1970" para ano, mês e dia.
    fn de_dias(dias: i64) -> Data {
        let z = dias + 719_468;
        let era = z.div_euclid(146_097);
        let dia_da_era = z - era * 146_097;
        let ano_da_era = (dia_da_era - dia_da_era / 1460 + dia_da_era / 36_524 - dia_da_era / 146_096) / 365;
        let dia_do_ano = dia_da_era - (365 * ano_da_era + ano_da_era / 4 - ano_da_era / 100);
        let mes_deslocado = (5 * dia_do_ano + 2) / 153;
        let dia = (dia_do_ano - (153 * mes_deslocado + 2) / 5 + 1) as u32;
        let mes = (if mes_deslocado < 10 { mes_deslocado + 3 } else { mes_deslocado - 9 }) as u32;
        Data { ano: ano_da_era + era * 400 + i64::from(mes <= 2), mes, dia }
    }

    fn somar_dias(self, dias: i64) -> Data {
        Data::de_dias(self.em_dias() + dias)
    }

    /// 01/01/1970 caiu numa quinta-feira (índice 4). rem_euclid garante índice
    /// positivo mesmo para datas anteriores a 1970, onde em_dias é negativo.
    fn dia_da_semana(self) -> &'static str {
        DIAS_SEMANA[(self.em_dias() + 4).rem_euclid(7) as usize]
    }

    /// Data de hoje a partir do relógio do sistema, em UTC.
    fn hoje() -> Result<Data, String> {
        let desde_1970 = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?;
        Ok(Data::de_dias((desde_1970.as_secs() / 86_400) as i64))
    }
}

/// Com FromStr implementado, "15/03/1990".parse::<Data>() passa a funcionar.
impl FromStr for Data {
    type Err = String;

    fn from_str(texto: &str) -> Result<Data, String> {
        let partes: Vec<&str> = texto.trim().split(['/', '-']).collect();
        if partes.len() != 3 {
            return Err(format!("data inválida: \"{texto}\" (use DD/MM/AAAA)"));
        }
        let numero = |p: &str, nome: &str| p.parse::<u32>().map_err(|_| format!("{nome} inválido em \"{texto}\": \"{p}\""));
        let (dia, mes, ano) = (numero(partes[0], "dia")?, numero(partes[1], "mês")?, numero(partes[2], "ano")?);
        Data::nova(i64::from(ano), mes, dia)
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}/{:02}/{:04}", self.dia, self.mes, self.ano)
    }
}

/// Idade em anos, meses e dias completos. Quando o dia ainda não chegou no
/// mês atual, pega-se um mês emprestado, como numa subtração feita à mão.
fn idade_exata(nascimento: Data, hoje: Data) -> Result<(i64, i64, i64), String> {
    if nascimento.em_dias() > hoje.em_dias() {
        return Err(format!("nascimento {nascimento} é depois de {hoje}"));
    }
    let anos = hoje.ano - nascimento.ano;
    let mut meses = i64::from(hoje.mes) - i64::from(nascimento.mes);
    let mut dias = i64::from(hoje.dia) - i64::from(nascimento.dia);
    if dias < 0 {
        let (ano_ant, mes_ant) = if hoje.mes == 1 { (hoje.ano - 1, 12) } else { (hoje.ano, hoje.mes - 1) };
        dias += i64::from(dias_no_mes(ano_ant, mes_ant));
        meses -= 1;
    }
    // Mesmo empréstimo, agora de um ano para os meses.
    let (anos, meses) = if meses < 0 { (anos - 1, meses + 12) } else { (anos, meses) };
    Ok((anos, meses, dias))
}

fn op_entre(a: Data, b: Data) {
    let dias = (b.em_dias() - a.em_dias()).abs();
    println!("Entre {a} e {b}: {dias} dias ({} semanas e {} dias)", dias / 7, dias % 7);
}

fn op_semana(data: Data) {
    println!("{data} é {}{}", data.dia_da_semana(), if bissexto(data.ano) { " (ano bissexto)" } else { "" });
}

fn op_idade(nascimento: Data, hoje: Data) -> Result<(), String> {
    let (anos, meses, dias) = idade_exata(nascimento, hoje)?;
    let total = hoje.em_dias() - nascimento.em_dias();
    println!("Nascido em {nascimento}, em {hoje}: {anos} anos, {meses} meses e {dias} dias ({total} dias vividos)");
    Ok(())
}

fn op_somar(data: Data, dias: i64) {
    let nova = data.somar_dias(dias);
    let verbo = if dias < 0 { "menos" } else { "mais" };
    println!("{data} {verbo} {} dias = {nova} ({})", dias.abs(), nova.dia_da_semana());
}

fn op_faltam(alvo: Data, hoje: Data) {
    match alvo.em_dias() - hoje.em_dias() {
        0 => println!("{alvo} é hoje!"),
        d if d > 0 => println!("Faltam {d} dias para {alvo} (hoje é {hoje}, {})", hoje.dia_da_semana()),
        d => println!("{alvo} já passou há {} dias (hoje é {hoje})", -d),
    }
}

fn arg<'a>(args: &'a [String], i: usize, nome: &str) -> Result<&'a str, String> {
    args.get(i).map(String::as_str).ok_or_else(|| format!("faltou o argumento {nome}"))
}

fn executar(args: &[String]) -> Result<(), String> {
    match args[0].as_str() {
        "entre" => op_entre(arg(args, 1, "DATA1")?.parse()?, arg(args, 2, "DATA2")?.parse()?),
        "semana" => op_semana(arg(args, 1, "DATA")?.parse()?),
        "idade" => {
            // Sem o segundo argumento, a idade é calculada até hoje.
            let hoje = match args.get(2) { Some(texto) => texto.parse()?, None => Data::hoje()? };
            op_idade(arg(args, 1, "NASCIMENTO")?.parse()?, hoje)?;
        }
        "somar" => {
            let texto = arg(args, 2, "DIAS")?;
            let dias = texto.parse::<i64>().map_err(|_| format!("DIAS inválido: \"{texto}\" não é um inteiro"))?;
            op_somar(arg(args, 1, "DATA")?.parse()?, dias);
        }
        "faltam" => op_faltam(arg(args, 1, "DATA")?.parse()?, Data::hoje()?),
        outro => return Err(format!("operação desconhecida: \"{outro}\". Rode sem argumentos para ver o uso")),
    }
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== Demonstração da calculadora de datas ===");
    op_entre("01/01/2024".parse()?, "25/12/2024".parse()?);
    op_entre("29/02/2024".parse()?, "01/03/2023".parse()?);
    op_semana("15/11/1889".parse()?);
    op_idade("15/03/1990".parse()?, "27/10/2026".parse()?)?;
    op_idade("31/12/2000".parse()?, "01/01/2026".parse()?)?;
    op_somar("31/01/2024".parse()?, 30);
    op_somar("01/03/2024".parse()?, -1);
    let hoje = Data::hoje()?;
    println!("Hoje, pelo relógio do sistema: {hoje}, {}", hoje.dia_da_semana());
    op_faltam(Data::nova(hoje.ano + 1, 1, 1)?, hoje);
    let recusa = "30/02/2025".parse::<Data>().err().unwrap_or_default();
    println!("Validação de data inexistente: {recusa}");
    Ok(())
}

fn imprimir_uso() {
    println!("Calculadora de datas (CodeNomad Academy)");
    println!("Uso: datas.exe OPERAÇÃO ARGUMENTOS   (datas no formato DD/MM/AAAA)");
    println!("  entre DATA1 DATA2        dias entre as duas datas");
    println!("  semana DATA              dia da semana da data");
    println!("  idade NASCIMENTO [HOJE]  idade exata em anos, meses e dias");
    println!("  somar DATA DIAS          soma dias (negativo subtrai)");
    println!("  faltam DATA              quantos dias faltam, contando de hoje");
    println!("  --demo                   roda uma demonstração completa");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None | Some("--help") | Some("-h") => {
            imprimir_uso();
            Ok(())
        }
        Some("--demo") => demo(),
        Some(_) => executar(&args),
    };
    match resultado {
        Ok(()) => ExitCode::SUCCESS,
        Err(mensagem) => {
            eprintln!("Erro: {mensagem}");
            ExitCode::FAILURE
        }
    }
}

// ------------------------------------------------------------
// Para ir além
// 1. Aceite também o formato AAAA-MM-DD na entrada e ofereça a flag --iso
//    para imprimir nesse formato, trocando só o Display e o FromStr.
// 2. Acrescente a operação "dias-uteis DATA1 DATA2", que pula sábados e
//    domingos usando dia_da_semana em um laço.
// 3. Crie a operação "calendario MM/AAAA" que imprime o mês inteiro em
//    grade, alinhando o primeiro dia na coluna certa da semana.
// ------------------------------------------------------------
