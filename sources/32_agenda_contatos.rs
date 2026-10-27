// ============================================================
// CodeNomad Academy  |  Sources Rust  |  32. Agenda de contatos em CSV
// ------------------------------------------------------------
// O que faz: Agenda de contatos (nome, telefone, e-mail e etiqueta) guardada
//            num arquivo CSV. Você adiciona, busca por qualquer campo, ordena,
//            exporta um filtro para outro CSV e detecta duplicados pelo
//            telefone ou pelo e-mail, mesmo com formatação diferente.
// Como usar: rustc -O --edition 2021 32_agenda_contatos.rs -o agenda.exe
//            agenda.exe adicionar "Ana Souza" "(11) 99876-1234" ana@exemplo.com trabalho
//            agenda.exe listar --ordenar nome --arquivo contatos.csv
//            agenda.exe exportar trabalho so_trabalho.csv
//            agenda.exe --demo
// O que você aprende: leitura e escrita de CSV com aspas feitas à mão, structs
//            com métodos, iteradores (filter, any, map, collect), HashMap com
//            entry().or_default(), try_into de Vec para array e let-else.
// Requisitos: Rust 1.70+, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const CAMPOS: [&str; 4] = ["nome", "telefone", "email", "etiqueta"];
const USO: &str = "Uso: agenda <comando> [opções] [--arquivo contatos.csv]
  adicionar \"Nome\" telefone e-mail [etiqueta]
  listar [--ordenar nome|telefone|email|etiqueta]
  buscar <termo>                 procura em qualquer campo
  exportar <termo> <saida.csv>   grava só os contatos que batem com o termo
  duplicados                     agrupa quem repete telefone ou e-mail
  --demo                         roda uma demonstração completa";

#[derive(Clone)]
struct Contato { nome: String, telefone: String, email: String, etiqueta: String }

impl Contato {
    fn campo(&self, nome: &str) -> &str {
        match nome { "nome" => &self.nome, "telefone" => &self.telefone, "email" => &self.email, _ => &self.etiqueta }
    }
    // O termo já chega em minúsculas; aqui a comparação ignora maiúsculas.
    fn contem(&self, termo: &str) -> bool {
        CAMPOS.iter().any(|c| self.campo(c).to_lowercase().contains(termo))
    }
    fn linha_csv(&self) -> String {
        CAMPOS.iter().map(|c| campo_csv(self.campo(c))).collect::<Vec<_>>().join(",")
    }
    fn validar(&self) -> Result<(), String> {
        if self.nome.is_empty() { return Err("o nome não pode ficar vazio".into()); }
        let digitos = so_digitos(&self.telefone).len();
        if !(8..=13).contains(&digitos) {
            return Err(format!("telefone inválido: '{}' (use de 8 a 13 dígitos)", self.telefone));
        }
        let (usuario, dominio) = self.email.split_once('@').ok_or(format!("e-mail inválido: '{}'", self.email))?;
        if usuario.is_empty() || !dominio.contains('.') || dominio.ends_with('.') {
            return Err(format!("e-mail inválido: '{}'", self.email));
        }
        Ok(())
    }
}

fn so_digitos(texto: &str) -> String { texto.chars().filter(|c| c.is_ascii_digit()).collect() }

// Campo com vírgula, aspas ou quebra de linha vai entre aspas; aspas internas viram duas.
fn campo_csv(valor: &str) -> String {
    if valor.contains([',', '"', '\n']) { format!("\"{}\"", valor.replace('"', "\"\"")) } else { valor.to_string() }
}

// Separa uma linha CSV respeitando aspas: "Silva, Bruno" é um campo só.
fn dividir_csv(linha: &str) -> Vec<String> {
    let (mut campos, mut atual, mut entre_aspas) = (Vec::new(), String::new(), false);
    let mut chars = linha.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if entre_aspas && chars.peek() == Some(&'"') => { atual.push('"'); chars.next(); }
            '"' => entre_aspas = !entre_aspas,
            ',' if !entre_aspas => campos.push(std::mem::take(&mut atual)),
            _ => atual.push(c),
        }
    }
    campos.push(atual);
    campos
}

fn carregar(caminho: &Path) -> Result<Vec<Contato>, String> {
    if !caminho.exists() { return Ok(Vec::new()); }
    let texto = fs::read_to_string(caminho).map_err(|e| format!("não consegui ler {}: {e}", caminho.display()))?;
    let mut contatos = Vec::new();
    for (n, linha) in texto.lines().enumerate() {
        if linha.trim().is_empty() || (n == 0 && linha.starts_with("nome,")) { continue; }
        // try_into converte Vec<String> em [String; 4] e devolve o Vec de volta se o tamanho não bater.
        let [nome, telefone, email, etiqueta]: [String; 4] = dividir_csv(linha).try_into()
            .map_err(|c: Vec<String>| format!("linha {} de {}: esperava 4 campos, achei {}", n + 1, caminho.display(), c.len()))?;
        contatos.push(Contato { nome, telefone, email, etiqueta });
    }
    Ok(contatos)
}

fn salvar(caminho: &Path, contatos: &[Contato]) -> Result<(), String> {
    let mut texto = String::from("nome,telefone,email,etiqueta\n");
    for c in contatos { texto.push_str(&c.linha_csv()); texto.push('\n'); }
    fs::write(caminho, texto).map_err(|e| format!("não consegui gravar {}: {e}", caminho.display()))
}

fn imprimir(contatos: &[&Contato]) {
    if contatos.is_empty() { println!("Nenhum contato para mostrar."); return; }
    println!("{:<16} {:<17} {:<26} {}", "nome", "telefone", "e-mail", "etiqueta");
    for c in contatos { println!("{:<16} {:<17} {:<26} {}", c.nome, c.telefone, c.email, c.etiqueta); }
}

// Agrupa por chave normalizada: só os dígitos do telefone, ou o e-mail em minúsculas.
fn duplicados(contatos: &[Contato]) -> Vec<(String, Vec<&Contato>)> {
    let mut grupos: HashMap<String, Vec<&Contato>> = HashMap::new();
    for c in contatos {
        grupos.entry(format!("telefone {}", so_digitos(&c.telefone))).or_default().push(c);
        grupos.entry(format!("e-mail {}", c.email.trim().to_lowercase())).or_default().push(c);
    }
    let mut lista: Vec<_> = grupos.into_iter().filter(|(_, iguais)| iguais.len() > 1).collect();
    lista.sort_by(|a, b| a.0.cmp(&b.0)); // HashMap não tem ordem; ordenar deixa a saída previsível
    lista
}

fn executar(args: &[String]) -> Result<(), String> {
    let mut caminho = PathBuf::from("contatos.csv");
    let mut resto: Vec<&str> = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--arquivo" { caminho = PathBuf::from(iter.next().ok_or("--arquivo precisa de um caminho")?); }
        else { resto.push(arg); }
    }
    let mut contatos = carregar(&caminho)?;
    match resto.first().copied().unwrap_or("") {
        "adicionar" => {
            let [_, nome, telefone, email, etiqueta @ ..] = resto.as_slice() else {
                return Err("uso: adicionar \"Nome\" telefone e-mail [etiqueta]".into());
            };
            let contato = Contato {
                nome: nome.trim().to_string(), telefone: telefone.trim().to_string(),
                email: email.trim().to_string(), etiqueta: etiqueta.first().unwrap_or(&"").trim().to_string(),
            };
            contato.validar()?;
            if contatos.iter().any(|c| c.email.eq_ignore_ascii_case(&contato.email)) {
                return Err(format!("já existe um contato com o e-mail {}", contato.email));
            }
            println!("Contato adicionado: {} <{}>", contato.nome, contato.email);
            contatos.push(contato);
            salvar(&caminho, &contatos)?;
        }
        "listar" => {
            let mut lista: Vec<&Contato> = contatos.iter().collect();
            if let Some(pos) = resto.iter().position(|a| *a == "--ordenar") {
                let campo = resto.get(pos + 1).copied().unwrap_or("nome");
                if !CAMPOS.contains(&campo) {
                    return Err(format!("campo desconhecido para ordenar: '{campo}' (use {})", CAMPOS.join(", ")));
                }
                lista.sort_by_key(|c| c.campo(campo).to_lowercase());
            }
            imprimir(&lista);
        }
        "buscar" => {
            let termo = resto.get(1).ok_or("informe o termo: buscar texto")?.to_lowercase();
            let achados: Vec<&Contato> = contatos.iter().filter(|c| c.contem(&termo)).collect();
            println!("{} contato(s) com \"{termo}\":", achados.len());
            imprimir(&achados);
        }
        "exportar" => {
            let [_, termo, destino] = resto.as_slice() else { return Err("uso: exportar <termo> <saida.csv>".into()) };
            let termo = termo.to_lowercase();
            let filtrados: Vec<Contato> = contatos.iter().filter(|c| c.contem(&termo)).cloned().collect();
            if filtrados.is_empty() { return Err(format!("nenhum contato com \"{termo}\"; nada foi exportado")); }
            salvar(Path::new(destino), &filtrados)?;
            println!("{} contato(s) exportado(s) para {destino}", filtrados.len());
        }
        "duplicados" => {
            let grupos = duplicados(&contatos);
            if grupos.is_empty() { println!("Nenhum duplicado encontrado."); }
            for (chave, iguais) in grupos {
                println!("Mesmo {chave}:");
                for c in iguais { println!("   - {} <{}> {}", c.nome, c.email, c.telefone); }
            }
        }
        outro => return Err(format!("comando desconhecido: '{outro}'\n{USO}")),
    }
    Ok(())
}

fn demo() -> Result<(), String> {
    let pasta = std::env::temp_dir();
    let arquivo = pasta.join(format!("codenomad_contatos_{}.csv", std::process::id()));
    let exportado = pasta.join(format!("codenomad_contatos_{}_familia.csv", std::process::id()));
    let exemplo = "nome,telefone,email,etiqueta
Ana Souza,(11) 99876-1234,ana.souza@exemplo.com,trabalho
\"Silva, Bruno\",11998761234,bruno@exemplo.com,família
Carla Mendes,(21) 3555-0101,carla.mendes@exemplo.com,família
Diego Ramos,(31) 98888-7777,DIEGO@exemplo.com,academia
Elisa Prado,(41) 97777-6666,elisa@exemplo.com,trabalho
Diego Ramos,(31) 98888-7777,diego@exemplo.com,amigos
";
    fs::write(&arquivo, exemplo).map_err(|e| e.to_string())?;
    println!("=== Demonstração: agenda de contatos em CSV ===");
    println!("Arquivo temporário: {}\n", arquivo.display());
    let (caminho, destino) = (arquivo.to_string_lossy().to_string(), exportado.to_string_lossy().to_string());
    let passos: [&[&str]; 6] = [
        &["listar", "--ordenar", "nome"],
        &["adicionar", "Fábio Lima", "(51) 96666-5555", "fabio@exemplo.com", "trabalho"],
        &["adicionar", "Gui", "123", "sem-arroba", "amigos"],
        &["buscar", "trabalho"], &["exportar", "família", &destino], &["duplicados"],
    ];
    for passo in passos {
        let bonito: Vec<String> = passo.iter().map(|p| if p.contains(' ') { format!("\"{p}\"") } else { p.to_string() }).collect();
        println!("$ agenda {}", bonito.join(" "));
        let mut args: Vec<String> = passo.iter().map(|s| s.to_string()).collect();
        args.extend(["--arquivo".to_string(), caminho.clone()]);
        // Um dos passos é inválido de propósito: a demonstração mostra o erro e segue.
        if let Err(erro) = executar(&args) { println!("Erro: {erro}"); }
        println!();
    }
    println!("Conteúdo do CSV exportado:");
    print!("{}", fs::read_to_string(&exportado).map_err(|e| e.to_string())?);
    let _ = fs::remove_file(&arquivo);
    let _ = fs::remove_file(&exportado);
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
// 1. Crie o comando "remover <e-mail>" e o comando "editar" para trocar um campo.
// 2. Faça o "duplicados" sugerir uma fusão: manter o contato mais completo e apagar o resto.
// 3. Aceite um CSV com as colunas em outra ordem, lendo a primeira linha como cabeçalho.
// ------------------------------------------------------------
