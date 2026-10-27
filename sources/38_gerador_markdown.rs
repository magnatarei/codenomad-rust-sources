// ============================================================
// CodeNomad Academy  |  Sources Rust  |  38. Gerador de relatório em Markdown
// ------------------------------------------------------------
// O que faz: Monta um documento Markdown (títulos, parágrafos, listas, tabela
//            alinhada e blocos de código) a partir de um CSV, com resumo
//            automático das colunas numéricas. Também converte só a tabela
//            CSV em tabela Markdown, com números alinhados à direita.
// Como usar: rustc -O --edition 2021 38_gerador_markdown.rs -o markdown.exe
//            markdown.exe tabela notas.csv
//            markdown.exe relatorio notas.csv --titulo "Notas do bimestre" --destino notas.md
//            markdown.exe --demo
// O que você aprende: enum com dados para cada bloco do documento, padrão
//            construtor (métodos que devolvem &mut Self e encadeiam), impl
//            Display, iteradores com zip, fold e max, largura dinâmica em
//            format! ({:<largura$}) e contagem de chars em vez de bytes.
// Requisitos: Rust 1.70+, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::fmt;
use std::fs;

const USO: &str = "Uso: markdown <comando> <dados.csv> [--titulo \"Texto\"] [--destino arquivo.md]
  tabela      converte o CSV numa tabela Markdown alinhada
  relatorio   monta o documento completo: título, resumo, tabela, lista e código
  --demo      gera um relatório de exemplo e imprime o Markdown";

// Cada variante guarda só o que aquele bloco precisa para ser escrito.
enum Bloco {
    Titulo(u8, String),
    Paragrafo(String),
    Lista(Vec<String>),
    Tabela { cabecalho: Vec<String>, linhas: Vec<Vec<String>> },
    Codigo { linguagem: String, texto: String },
}

#[derive(Default)]
struct Documento { blocos: Vec<Bloco> }

// Cada método devolve &mut Self, por isso dá para encadear: doc.titulo(..).paragrafo(..).
impl Documento {
    fn titulo(&mut self, nivel: u8, texto: &str) -> &mut Self { self.blocos.push(Bloco::Titulo(nivel.clamp(1, 6), texto.into())); self }
    fn paragrafo(&mut self, texto: &str) -> &mut Self { self.blocos.push(Bloco::Paragrafo(texto.into())); self }
    fn lista(&mut self, itens: Vec<String>) -> &mut Self { self.blocos.push(Bloco::Lista(itens)); self }
    fn tabela(&mut self, cabecalho: Vec<String>, linhas: Vec<Vec<String>>) -> &mut Self {
        self.blocos.push(Bloco::Tabela { cabecalho, linhas }); self
    }
    fn codigo(&mut self, linguagem: &str, texto: &str) -> &mut Self {
        self.blocos.push(Bloco::Codigo { linguagem: linguagem.into(), texto: texto.into() }); self
    }
}

fn eh_numero(texto: &str) -> bool { texto.trim().replace(',', ".").parse::<f64>().is_ok() }

// Tabela com colunas alinhadas: largura pela maior célula e números à direita.
fn escrever_tabela(cabecalho: &[String], linhas: &[Vec<String>]) -> String {
    let colunas = cabecalho.len();
    // chars().count() conta letras acentuadas como uma só; len() contaria os bytes.
    let largura: Vec<usize> = (0..colunas).map(|c| {
        linhas.iter().map(|l| l.get(c).map_or(0, |v| v.chars().count())).fold(cabecalho[c].chars().count(), usize::max).max(3)
    }).collect();
    let numerica: Vec<bool> = (0..colunas).map(|c| {
        !linhas.is_empty() && linhas.iter().all(|l| l.get(c).is_some_and(|v| eh_numero(v)))
    }).collect();
    let celula = |valor: &str, c: usize| {
        if numerica[c] { format!("{:>largura$}", valor, largura = largura[c]) } else { format!("{:<largura$}", valor, largura = largura[c]) }
    };
    let mut saida = String::new();
    let cab: Vec<String> = cabecalho.iter().enumerate().map(|(c, v)| celula(v, c)).collect();
    saida.push_str(&format!("| {} |\n", cab.join(" | ")));
    let separador: Vec<String> = (0..colunas).map(|c| {
        if numerica[c] { format!("{}:", "-".repeat(largura[c] - 1)) } else { format!(":{}", "-".repeat(largura[c] - 1)) }
    }).collect();
    saida.push_str(&format!("| {} |\n", separador.join(" | ")));
    for linha in linhas {
        let celulas: Vec<String> = (0..colunas).map(|c| celula(linha.get(c).map_or("", String::as_str), c)).collect();
        saida.push_str(&format!("| {} |\n", celulas.join(" | ")));
    }
    saida
}

// Implementar Display permite usar o documento em println!("{doc}") e em to_string().
impl fmt::Display for Documento {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for bloco in &self.blocos {
            match bloco {
                Bloco::Titulo(nivel, texto) => writeln!(f, "{} {texto}\n", "#".repeat(*nivel as usize))?,
                Bloco::Paragrafo(texto) => writeln!(f, "{texto}\n")?,
                Bloco::Lista(itens) => {
                    for item in itens { writeln!(f, "- {item}")?; }
                    writeln!(f)?;
                }
                Bloco::Tabela { cabecalho, linhas } => writeln!(f, "{}", escrever_tabela(cabecalho, linhas))?,
                Bloco::Codigo { linguagem, texto } => writeln!(f, "```{linguagem}\n{}\n```\n", texto.trim_end())?,
            }
        }
        Ok(())
    }
}

fn ler_csv(texto: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut linhas = texto.lines().filter(|l| !l.trim().is_empty()).map(|l| l.split(',').map(|c| c.trim().to_string()).collect::<Vec<String>>());
    let cabecalho = linhas.next().ok_or("o CSV está vazio")?;
    let corpo: Vec<Vec<String>> = linhas.collect();
    for (n, linha) in corpo.iter().enumerate() {
        if linha.len() != cabecalho.len() {
            return Err(format!("linha {}: esperava {} colunas, achei {}", n + 2, cabecalho.len(), linha.len()));
        }
    }
    if corpo.is_empty() { return Err("o CSV só tem o cabeçalho, sem dados".into()); }
    Ok((cabecalho, corpo))
}

fn montar_relatorio(titulo: &str, origem: &str, texto_csv: &str) -> Result<Documento, String> {
    let (cabecalho, linhas) = ler_csv(texto_csv)?;
    let mut doc = Documento::default();
    doc.titulo(1, titulo)
        .paragrafo(&format!("Relatório gerado a partir de `{origem}`: {} registro(s) e {} coluna(s).", linhas.len(), cabecalho.len()))
        .titulo(2, "Dados");
    doc.tabela(cabecalho.clone(), linhas.clone()).titulo(2, "Resumo das colunas numéricas");
    let mut resumo = Vec::new();
    for (c, nome) in cabecalho.iter().enumerate() {
        let valores: Vec<f64> = linhas.iter().filter_map(|l| l[c].replace(',', ".").parse().ok()).collect();
        if valores.len() != linhas.len() { continue; }
        let soma: f64 = valores.iter().sum();
        let (min, max) = valores.iter().fold((f64::MAX, f64::MIN), |(mi, ma), v| (mi.min(*v), ma.max(*v)));
        resumo.push(format!("**{nome}**: média {:.2}, mínimo {min}, máximo {max}, soma {soma}", soma / valores.len() as f64));
    }
    if resumo.is_empty() { resumo.push("Nenhuma coluna numérica encontrada.".into()); }
    doc.lista(resumo).titulo(2, "Origem");
    let amostra: Vec<&str> = texto_csv.lines().take(4).collect();
    doc.paragrafo("As primeiras linhas do arquivo original:").codigo("csv", &amostra.join("\n"));
    Ok(doc)
}

fn executar(args: &[String]) -> Result<(), String> {
    let [comando, arquivo, opcoes @ ..] = args else { return Err(format!("informe o comando e o CSV\n{USO}")) };
    let (mut titulo, mut saida) = ("Relatório".to_string(), None);
    for par in opcoes.chunks(2) {
        let valor = par.get(1).ok_or(format!("a opção {} precisa de um valor", par[0]))?;
        match par[0].as_str() {
            "--titulo" => titulo = valor.clone(),
            "--destino" => saida = Some(valor.clone()),
            outra => return Err(format!("opção desconhecida: {outra}\n{USO}")),
        }
    }
    let texto = fs::read_to_string(arquivo).map_err(|e| format!("não consegui ler {arquivo}: {e}"))?;
    let markdown = match comando.as_str() {
        "tabela" => { let (cab, linhas) = ler_csv(&texto)?; escrever_tabela(&cab, &linhas) }
        "relatorio" => montar_relatorio(&titulo, arquivo, &texto)?.to_string(),
        outro => return Err(format!("comando desconhecido: '{outro}'\n{USO}")),
    };
    match saida {
        Some(destino) => {
            fs::write(&destino, &markdown).map_err(|e| format!("não consegui gravar {destino}: {e}"))?;
            println!("Markdown gravado em {destino} ({} linhas).", markdown.lines().count());
        }
        None => print!("{markdown}"),
    }
    Ok(())
}

fn demo() -> Result<(), String> {
    let csv = "aluno,turma,nota,faltas\nAna Souza,A,8.5,2\nJoão Pereira,B,7,0\nMaíra Costa,A,9.25,1\nPedro Lima,B,6.5,4\n";
    let arquivo = std::env::temp_dir().join(format!("codenomad_notas_{}.csv", std::process::id()));
    fs::write(&arquivo, csv).map_err(|e| format!("não consegui gravar o CSV temporário: {e}"))?;
    let caminho = arquivo.to_string_lossy().to_string();
    println!("=== Demonstração: gerador de relatório em Markdown ===");
    println!("CSV temporário: {caminho}\n");
    println!("--- 1. Só a tabela ---");
    executar(&["tabela".into(), caminho.clone()])?;
    println!("\n--- 2. Relatório completo ---");
    executar(&["relatorio".into(), caminho.clone(), "--titulo".into(), "Notas do bimestre".into()])?;
    println!("--- 3. Documento montado só com o construtor, sem CSV ---");
    let mut doc = Documento::default();
    doc.titulo(1, "Plano de estudos").paragrafo("Três passos para a semana.")
        .lista(vec!["Ler o capítulo 4".into(), "Fazer os exercícios".into(), "Revisar os erros".into()])
        .codigo("rust", "fn main() {\n    println!(\"Olá, Markdown!\");\n}");
    print!("{doc}");
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
// 1. Aceite campos entre aspas no CSV ("Silva, Bruno") sem quebrar a contagem de colunas.
// 2. Adicione o bloco Citacao (linhas começando com "> ") e o bloco Imagem (![alt](caminho)).
// 3. Crie a opção --html que escreve o mesmo Documento em HTML, com um segundo impl de saída.
// ------------------------------------------------------------
