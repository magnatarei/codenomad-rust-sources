// ============================================================
// CodeNomad Academy  |  Sources Rust  |  19. Conversor de bases numéricas
// ------------------------------------------------------------
// O que faz: Converte números entre qualquer base de 2 a 36 (binário,
//            octal, base 10, hexadecimal...), explica o cálculo passo a
//            passo e codifica texto em binário (um byte por grupo de 8
//            dígitos) e de volta para texto.
// Como usar: 19_conversor_bases.exe 255 10 2
//            19_conversor_bases.exe ff 16 10 --passos
//            19_conversor_bases.exe codificar "Olá"
//            19_conversor_bases.exe decodificar "01001111 01101100 11000011 10100001"
//            19_conversor_bases.exe --demo
// O que você aprende: u128 com checked_mul e checked_add (estouro vira
//            erro, não valor errado), char::to_digit e from_digit com base,
//            divisões sucessivas, pow com u32, String::from_utf8 com erro
//            tratado, formatação {:08b} e bytes() de uma String.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

fn validar_base(texto: &str) -> Result<u32, String> {
    let base: u32 = texto.parse().map_err(|_| format!("base inválida: '{texto}' (use um inteiro de 2 a 36)"))?;
    if (2..=36).contains(&base) { Ok(base) } else { Err(format!("base {base} fora da faixa: use de 2 a 36")) }
}

/// Nome amigável das bases mais comuns, para as mensagens.
fn nome_da_base(base: u32) -> String {
    match base {
        2 => "binário".to_string(),
        8 => "octal".to_string(),
        10 => "base 10".to_string(),
        16 => "hexadecimal".to_string(),
        outra => format!("base {outra}"),
    }
}

/// Lê um número escrito em `base` e devolve o valor. Cada dígito multiplica
/// o acumulado pela base e soma: "1A" em hexadecimal é (1 * 16) + 10.
/// checked_* devolve None no estouro, e o ok_or transforma isso em erro.
fn para_valor(texto: &str, base: u32) -> Result<u128, String> {
    let mut valor: u128 = 0;
    for c in digitos_de(texto)? {
        let digito = c.to_digit(base).ok_or_else(|| format!("'{c}' não é um dígito válido em {}", nome_da_base(base)))?;
        valor = valor
            .checked_mul(u128::from(base))
            .and_then(|v| v.checked_add(u128::from(digito)))
            .ok_or_else(|| format!("'{}' é grande demais (o limite é 2 elevado a 128)", texto.trim()))?;
    }
    Ok(valor)
}

/// Tira espaços, um sinal de mais e os sublinhados de separação ("1_000").
fn digitos_de(texto: &str) -> Result<Vec<char>, String> {
    let digitos: Vec<char> = texto.trim().trim_start_matches('+').chars().filter(|c| *c != '_').collect();
    if digitos.is_empty() { Err("número vazio".to_string()) } else { Ok(digitos) }
}

/// Escreve `valor` em `base`: divide repetidamente e junta os restos
/// de trás para frente. O resto de cada divisão é um dígito.
fn de_valor(mut valor: u128, base: u32) -> String {
    if valor == 0 {
        return "0".to_string();
    }
    let mut digitos: Vec<char> = Vec::new();
    while valor > 0 {
        let resto = (valor % u128::from(base)) as u32;
        // from_digit só falha com resto >= base, o que o módulo impede.
        digitos.push(char::from_digit(resto, base).unwrap_or('?').to_ascii_uppercase());
        valor /= u128::from(base);
    }
    digitos.iter().rev().collect()
}

fn converter(texto: &str, de: u32, para: u32) -> Result<String, String> {
    Ok(de_valor(para_valor(texto, de)?, para))
}

/// Mostra a soma posicional: cada dígito vezes a base elevada à posição.
fn explicar_para_valor(texto: &str, base: u32) -> Result<u128, String> {
    let valor = para_valor(texto, base)?;
    let digitos = digitos_de(texto)?;
    println!("  Lendo '{}' em {} (posição 0 é a da direita):", texto.trim(), nome_da_base(base));
    let ultimo = digitos.len() - 1;
    for (i, c) in digitos.iter().enumerate() {
        let posicao = (ultimo - i) as u32;
        let d = c.to_digit(base).unwrap_or(0);
        let peso = u128::from(base).pow(posicao);
        println!("    {c} x {base}^{posicao} = {d} x {peso} = {}", u128::from(d) * peso);
    }
    println!("    soma = {valor}");
    Ok(valor)
}

/// Mostra as divisões sucessivas e o resultado lido de baixo para cima.
fn explicar_de_valor(valor: u128, base: u32) -> String {
    println!("  Escrevendo {valor} em {} (divisões sucessivas):", nome_da_base(base));
    let mut atual = valor;
    while atual > 0 {
        let (quociente, resto) = (atual / u128::from(base), atual % u128::from(base));
        let digito = char::from_digit(resto as u32, base).unwrap_or('?').to_ascii_uppercase();
        println!("    {atual:>8} / {base} = {quociente:<8} resto {resto:>2} -> dígito {digito}");
        atual = quociente;
    }
    let resultado = de_valor(valor, base);
    println!("    restos lidos de baixo para cima: {resultado}");
    resultado
}

/// Cada byte do texto (UTF-8) vira 8 dígitos binários. Letras acentuadas
/// ocupam dois bytes, então "á" gera dois grupos.
fn texto_para_binario(texto: &str) -> String {
    texto.bytes().map(|b| format!("{b:08b}")).collect::<Vec<String>>().join(" ")
}

/// O caminho de volta: grupos de 8 dígitos viram bytes, e os bytes viram
/// texto. Se os bytes não formarem UTF-8 válido, from_utf8 avisa.
fn binario_para_texto(binario: &str) -> Result<String, String> {
    let compacto: String = binario.chars().filter(|c| !c.is_whitespace()).collect();
    if compacto.is_empty() || compacto.len() % 8 != 0 {
        return Err(format!("esperava grupos de 8 dígitos binários, recebi {} dígitos", compacto.len()));
    }
    let bytes: Vec<u8> = compacto
        .as_bytes()
        .chunks(8)
        .map(|grupo| {
            let texto = std::str::from_utf8(grupo).map_err(|e| e.to_string())?;
            u8::from_str_radix(texto, 2).map_err(|_| format!("'{texto}' não é um byte binário válido"))
        })
        .collect::<Result<_, _>>()?;
    String::from_utf8(bytes).map_err(|e| format!("os bytes não formam texto UTF-8 válido: {e}"))
}

fn demo() -> Result<(), String> {
    println!("=== Conversor de bases numéricas: demonstração ===\n");
    println!("Conversões diretas:");
    let casos = [("255", 10, 2), ("255", 10, 16), ("11111111", 2, 10), ("777", 8, 10), ("ff", 16, 2), ("zz", 36, 10), ("2024", 10, 36), ("1010_1010", 2, 16)];
    for (numero, de, para) in casos {
        println!("  {numero:>10} ({:<11}) = {:>12} ({})", nome_da_base(de), converter(numero, de, para)?, nome_da_base(para));
    }
    println!("\nPasso a passo de 2024 para binário:");
    explicar_de_valor(2024, 2);
    println!("\nPasso a passo de 'FF' hexadecimal para base 10:");
    explicar_para_valor("FF", 16)?;
    println!("\nTexto em binário:");
    let palavra = "Olá";
    let codificado = texto_para_binario(palavra);
    println!("  '{palavra}' -> {codificado}");
    println!("  de volta  -> '{}'", binario_para_texto(&codificado)?);
    println!("\nTentando converter 'G' em hexadecimal:");
    match converter("G", 16, 10) {
        Ok(v) => println!("  {v}"),
        Err(e) => println!("  erro tratado: {e}"),
    }
    Ok(())
}

fn uso() {
    println!("Uso:");
    println!("  19_conversor_bases.exe <número> <base de origem> <base de destino> [--passos]");
    println!("  19_conversor_bases.exe codificar <texto>          texto para binário");
    println!("  19_conversor_bases.exe decodificar <grupos de 8>  binário para texto");
    println!("  19_conversor_bases.exe --demo");
}

fn executar(args: &[String]) -> Result<(), String> {
    match args {
        [] => uso(),
        [c] if c == "--demo" => demo()?,
        [c, texto @ ..] if c == "codificar" && !texto.is_empty() => println!("{}", texto_para_binario(&texto.join(" "))),
        [c, grupos @ ..] if c == "decodificar" && !grupos.is_empty() => println!("{}", binario_para_texto(&grupos.join(" "))?),
        [c] if c == "codificar" || c == "decodificar" => return Err(format!("faltou o conteúdo depois de '{c}'")),
        [numero, de, para, resto @ ..] => {
            let (de, para) = (validar_base(de)?, validar_base(para)?);
            match resto {
                [] => println!("{}", converter(numero, de, para)?),
                [flag] if flag == "--passos" => {
                    let valor = explicar_para_valor(numero, de)?;
                    println!();
                    explicar_de_valor(valor, para);
                }
                [outro, ..] => return Err(format!("argumento inesperado '{outro}' (a única opção é --passos)")),
            }
        }
        _ => return Err("informe <número> <base de origem> <base de destino>; veja o uso sem argumentos".to_string()),
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
// 1. Aceite números negativos usando complemento de dois com largura fixa
//    (8, 16 ou 32 bits) e mostre a representação em binário.
// 2. Converta a parte fracionária também ("0,625" em base 10 é "0,101"
//    em binário), multiplicando a fração pela base repetidas vezes.
// 3. Aceite os prefixos 0b, 0o e 0x para descobrir a base de origem sozinho.
