// ============================================================
// CodeNomad Academy  |  Sources Rust  |  31. Lista de tarefas em arquivo JSON
// ------------------------------------------------------------
// O que faz: Guarda tarefas num arquivo JSON e deixa você adicionar, listar,
//            concluir, remover e buscar pela linha de comando. Cada tarefa tem
//            prioridade e data opcional; a listagem sai ordenada por ambas.
// Como usar: rustc -O --edition 2021 31_lista_tarefas.rs -o tarefas.exe
//            tarefas.exe adicionar "Revisar capítulo 3" -p alta -d 2026-11-05
//            tarefas.exe listar --todas --arquivo minhas_tarefas.json
//            tarefas.exe --demo
// O que você aprende: structs e enums com impl, Option e Result com ? e let-else,
//            um leitor de JSON escrito à mão (recursão e closures genéricas),
//            ordenação por tupla com sort_by_key, std::fs, std::env e temp_dir.
// Requisitos: Rust 1.70+, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================
use std::fs;
use std::path::{Path, PathBuf};
const USO: &str = "Uso: tarefas <comando> [opções] [--arquivo caminho.json]
  adicionar \"título\" [-p alta|normal|baixa] [-d AAAA-MM-DD]
  listar [--todas]        mostra as pendentes (ou todas)
  concluir <id> | remover <id>   marca como feita | apaga
  buscar <termo>          procura no título
  --demo                  roda uma demonstração completa";

// A ordem das variantes define a ordem de prioridade (Alta < Normal < Baixa).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prioridade { Alta, Normal, Baixa }

impl Prioridade {
    fn parse(texto: &str) -> Result<Self, String> {
        match texto.to_lowercase().as_str() {
            "alta" => Ok(Prioridade::Alta), "normal" => Ok(Prioridade::Normal), "baixa" => Ok(Prioridade::Baixa),
            outro => Err(format!("prioridade inválida: '{outro}' (use alta, normal ou baixa)")),
        }
    }
    fn nome(self) -> &'static str { match self { Self::Alta => "alta", Self::Normal => "normal", Self::Baixa => "baixa" } }
}

struct Tarefa { id: u32, titulo: String, prioridade: Prioridade, data: Option<String>, feita: bool }

// Um JSON em memória: cada variante é um tipo de valor possível.
enum Json { Nulo, Bool(bool), Num(f64), Texto(String), Lista(Vec<Json>), Objeto(Vec<(String, Json)>) }

// Leitor recursivo: anda pelos bytes do texto e monta um Json.
struct Leitor<'a> { b: &'a [u8], i: usize }

impl Leitor<'_> {
    fn pular_espacos(&mut self) {
        while self.b.get(self.i).is_some_and(|c| c.is_ascii_whitespace()) { self.i += 1; }
    }
    fn consome(&mut self, esperado: u8) -> Result<(), String> {
        self.pular_espacos();
        if self.b.get(self.i) == Some(&esperado) { self.i += 1; return Ok(()); }
        Err(format!("JSON inválido: esperava '{}' na posição {}", esperado as char, self.i))
    }
    // Lê itens separados por vírgula até o byte de fechamento; a closure "ler" lê cada item.
    fn itens<T>(&mut self, fecha: u8, mut ler: impl FnMut(&mut Self) -> Result<T, String>) -> Result<Vec<T>, String> {
        self.i += 1;
        let mut saida = Vec::new();
        loop {
            self.pular_espacos();
            if self.b.get(self.i) == Some(&fecha) { self.i += 1; return Ok(saida); }
            if !saida.is_empty() { self.consome(b',')?; }
            saida.push(ler(self)?);
        }
    }
    fn valor(&mut self) -> Result<Json, String> {
        self.pular_espacos();
        match self.b.get(self.i).copied() {
            Some(b'[') => self.itens(b']', |l| l.valor()).map(Json::Lista),
            Some(b'{') => self.itens(b'}', |l| {
                let Json::Texto(chave) = l.valor()? else { return Err("JSON inválido: chave precisa ser texto".into()) };
                l.consome(b':')?;
                Ok((chave, l.valor()?))
            }).map(Json::Objeto),
            Some(b'"') => {
                self.i += 1;
                let mut bytes = Vec::new();
                loop {
                    let c = self.b.get(self.i).copied().ok_or("JSON inválido: texto sem aspas de fechamento")?;
                    self.i += 1;
                    match c {
                        b'"' => return String::from_utf8(bytes).map(Json::Texto).map_err(|_| "texto com UTF-8 inválido".into()),
                        b'\\' => {
                            let escape = self.b.get(self.i).copied().ok_or("JSON inválido: escape no fim do texto")?;
                            self.i += 1;
                            bytes.push(match escape {
                                b'n' => b'\n', b't' => b'\t', b'r' => b'\r', b'"' | b'\\' | b'/' => escape,
                                _ => return Err("JSON inválido: escape desconhecido".into()),
                            });
                        }
                        outro => bytes.push(outro),
                    }
                }
            }
            Some(c) if c == b'-' || c.is_ascii_digit() => {
                let inicio = self.i;
                while self.b.get(self.i).is_some_and(|c| b"-+.eE".contains(c) || c.is_ascii_digit()) { self.i += 1; }
                let trecho = std::str::from_utf8(&self.b[inicio..self.i]).unwrap_or("");
                trecho.parse().map(Json::Num).map_err(|_| format!("JSON inválido: número '{trecho}'"))
            }
            _ => {
                for (literal, valor) in [("true", Json::Bool(true)), ("false", Json::Bool(false)), ("null", Json::Nulo)] {
                    if self.b[self.i..].starts_with(literal.as_bytes()) { self.i += literal.len(); return Ok(valor); }
                }
                Err(format!("JSON inválido na posição {}", self.i))
            }
        }
    }
}

// A barra invertida tem de ser a primeira troca, senão ela escaparia as outras.
fn escapar(s: &str) -> String {
    let limpo = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\t', "\\t").replace('\r', "\\r");
    format!("\"{limpo}\"")
}

fn salvar(caminho: &Path, tarefas: &[Tarefa]) -> Result<(), String> {
    let linhas: Vec<String> = tarefas.iter().map(|t| {
        let data = t.data.as_deref().map(escapar).unwrap_or_else(|| "null".to_string());
        format!("  {{\"id\": {}, \"titulo\": {}, \"prioridade\": \"{}\", \"data\": {data}, \"feita\": {}}}",
            t.id, escapar(&t.titulo), t.prioridade.nome(), t.feita)
    }).collect();
    fs::write(caminho, format!("[\n{}\n]\n", linhas.join(",\n")))
        .map_err(|e| format!("não consegui gravar {}: {e}", caminho.display()))
}

fn carregar(caminho: &Path) -> Result<Vec<Tarefa>, String> {
    if !caminho.exists() { return Ok(Vec::new()); }
    let texto = fs::read_to_string(caminho).map_err(|e| format!("não consegui ler {}: {e}", caminho.display()))?;
    let mut leitor = Leitor { b: texto.as_bytes(), i: 0 };
    let Json::Lista(itens) = leitor.valor()? else { return Err("o arquivo deveria conter uma lista JSON".into()) };
    itens.into_iter().map(|item| {
        let Json::Objeto(campos) = item else { return Err("cada tarefa deveria ser um objeto JSON".to_string()) };
        let pega = |nome: &str| campos.iter().find(|(chave, _)| chave == nome).map(|(_, v)| v);
        Ok(Tarefa {
            id: match pega("id") { Some(Json::Num(n)) => *n as u32, _ => return Err("tarefa sem id".into()) },
            titulo: match pega("titulo") { Some(Json::Texto(s)) => s.clone(), _ => return Err("tarefa sem título".into()) },
            prioridade: match pega("prioridade") { Some(Json::Texto(s)) => Prioridade::parse(s)?, _ => Prioridade::Normal },
            data: match pega("data") { Some(Json::Texto(s)) => Some(s.clone()), _ => None },
            feita: matches!(pega("feita"), Some(Json::Bool(true))),
        })
    }).collect()
}

fn validar_data(texto: &str) -> Result<String, String> {
    let partes: Vec<u32> = texto.split('-').filter_map(|p| p.parse().ok()).collect();
    match partes.as_slice() {
        [ano, mes, dia] if (2000..=2100).contains(ano) && (1..=12).contains(mes) && (1..=31).contains(dia) => Ok(texto.to_string()),
        _ => Err(format!("data inválida: '{texto}' (use AAAA-MM-DD)")),
    }
}

fn imprimir(tarefas: &[&Tarefa]) {
    if tarefas.is_empty() { println!("Nenhuma tarefa para mostrar."); return; }
    println!("{:>3}  {:<3} {:<7} {:<10}  {}", "id", "ok", "prior.", "data", "título");
    for t in tarefas {
        println!("{:>3}  {:<3} {:<7} {:<10}  {}", t.id, if t.feita { "[x]" } else { "[ ]" },
            t.prioridade.nome(), t.data.as_deref().unwrap_or("-"), t.titulo);
    }
}

fn executar(args: &[String]) -> Result<(), String> {
    // A opção --arquivo pode vir em qualquer posição; o resto é o comando.
    let mut caminho = PathBuf::from("tarefas.json");
    let mut resto: Vec<&str> = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--arquivo" { caminho = PathBuf::from(iter.next().ok_or("--arquivo precisa de um caminho")?); }
        else { resto.push(arg); }
    }
    let mut tarefas = carregar(&caminho)?;
    let comando = resto.first().copied().unwrap_or("");
    match comando {
        "adicionar" => {
            let titulo = resto.get(1).map(|t| t.trim()).filter(|t| !t.is_empty())
                .ok_or("informe o título: adicionar \"texto da tarefa\"")?.to_string();
            let (mut prioridade, mut data) = (Prioridade::Normal, None);
            for par in resto[2..].chunks(2) {
                let valor = par.get(1).ok_or(format!("a opção {} precisa de um valor", par[0]))?;
                match par[0] {
                    "-p" | "--prioridade" => prioridade = Prioridade::parse(valor)?,
                    "-d" | "--data" => data = Some(validar_data(valor)?),
                    outra => return Err(format!("opção desconhecida: {outra}")),
                }
            }
            let id = tarefas.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            tarefas.push(Tarefa { id, titulo, prioridade, data, feita: false });
            salvar(&caminho, &tarefas)?;
            println!("Tarefa #{id} adicionada.");
        }
        "listar" => {
            let todas = resto.contains(&"--todas");
            let mut visiveis: Vec<&Tarefa> = tarefas.iter().filter(|t| todas || !t.feita).collect();
            // Tupla como chave: pendentes primeiro, depois prioridade, depois quem tem data, depois a data.
            visiveis.sort_by_key(|t| (t.feita, t.prioridade, t.data.is_none(), t.data.clone(), t.id));
            imprimir(&visiveis);
        }
        "concluir" | "remover" => {
            let texto = resto.get(1).ok_or("informe o número da tarefa")?;
            let id: u32 = texto.parse().map_err(|_| format!("'{texto}' não é um número de tarefa"))?;
            let pos = tarefas.iter().position(|t| t.id == id).ok_or(format!("não existe tarefa #{id}"))?;
            if comando == "remover" { println!("Tarefa #{id} removida: {}", tarefas.remove(pos).titulo); }
            else { tarefas[pos].feita = true; println!("Tarefa #{id} concluída: {}", tarefas[pos].titulo); }
            salvar(&caminho, &tarefas)?;
        }
        "buscar" => {
            let termo = resto.get(1).ok_or("informe o termo: buscar palavra")?.to_lowercase();
            let achadas: Vec<&Tarefa> = tarefas.iter().filter(|t| t.titulo.to_lowercase().contains(&termo)).collect();
            println!("{} tarefa(s) com \"{termo}\":", achadas.len());
            imprimir(&achadas);
        }
        outro => return Err(format!("comando desconhecido: '{outro}'\n{USO}")),
    }
    Ok(())
}

fn demo() -> Result<(), String> {
    let arquivo = std::env::temp_dir().join(format!("codenomad_tarefas_{}.json", std::process::id()));
    let caminho = arquivo.to_string_lossy().to_string();
    println!("=== Demonstração: lista de tarefas em JSON ===");
    println!("Arquivo temporário: {caminho}\n");
    let passos: [&[&str]; 9] = [
        &["adicionar", "Revisar capítulo 3 de Rust", "-p", "alta", "-d", "2026-11-05"],
        &["adicionar", "Comprar café"], &["adicionar", "Organizar as fotos", "-p", "baixa"],
        &["adicionar", "Pagar a conta de luz", "-p", "alta", "-d", "2026-11-02"],
        &["listar"], &["concluir", "2"], &["buscar", "conta"], &["remover", "3"], &["listar", "--todas"],
    ];
    for passo in passos {
        let bonito: Vec<String> = passo.iter().map(|p| if p.contains(' ') { format!("\"{p}\"") } else { p.to_string() }).collect();
        println!("$ tarefas {}", bonito.join(" "));
        let mut args: Vec<String> = passo.iter().map(|s| s.to_string()).collect();
        args.extend(["--arquivo".to_string(), caminho.clone()]);
        executar(&args)?;
        println!();
    }
    println!("Conteúdo final do arquivo JSON:");
    print!("{}", fs::read_to_string(&arquivo).map_err(|e| e.to_string())?);
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
// 1. Crie o comando "editar" para trocar título, prioridade ou data sem remover a tarefa.
// 2. Marque as tarefas atrasadas (data menor que hoje, via std::time::SystemTime) na listagem.
// 3. Escreva a função inversa do Leitor: de um Json para texto, e use-a dentro de salvar().
// ------------------------------------------------------------
