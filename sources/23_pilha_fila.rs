// ============================================================
// CodeNomad Academy  |  Sources Rust  |  23. Pilha e fila na prática
// ------------------------------------------------------------
// O que faz: implementa pilha (LIFO) e fila (FIFO) com as operações
//            clássicas e usa as duas em três problemas reais: parênteses
//            balanceados, desfazer/refazer e fila de atendimento com espera.
// Como usar: rustc -O --edition 2021 -D warnings 23_pilha_fila.rs -o pilha_fila.exe
//            pilha_fila.exe parenteses "{[(1 + 2) * 3]}"
//            pilha_fila.exe editor ola mundo dos programadores
//            pilha_fila.exe fila Ana:0:5 Bruno:1:3 Carla:2:4
//            pilha_fila.exe --demo
// O que você aprende: structs genéricas (Pilha<T>, Fila<T>), VecDeque,
//            Option como resposta de "tirar", match com guarda, let-else,
//            std::mem::replace e padrões de slice.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::collections::VecDeque;

/// Pilha: o último que entra é o primeiro que sai. Um Vec já faz isso com
/// push e pop; a struct dá nomes claros e esconde o que não interessa.
struct Pilha<T> { itens: Vec<T> }

impl<T> Pilha<T> {
    fn nova() -> Self { Pilha { itens: Vec::new() } }
    fn empilhar(&mut self, item: T) { self.itens.push(item); }
    fn desempilhar(&mut self) -> Option<T> { self.itens.pop() }
    fn topo(&self) -> Option<&T> { self.itens.last() }
    fn esta_vazia(&self) -> bool { self.itens.is_empty() }
    fn tamanho(&self) -> usize { self.itens.len() }
}

/// Fila: o primeiro que entra é o primeiro que sai. VecDeque tira do começo
/// em tempo constante; um Vec teria de deslocar todos os outros itens.
struct Fila<T> { itens: VecDeque<T> }

impl<T> Fila<T> {
    fn nova() -> Self { Fila { itens: VecDeque::new() } }
    fn enfileirar(&mut self, item: T) { self.itens.push_back(item); }
    fn desenfileirar(&mut self) -> Option<T> { self.itens.pop_front() }
    fn frente(&self) -> Option<&T> { self.itens.front() }
    fn esta_vazia(&self) -> bool { self.itens.is_empty() }
    fn tamanho(&self) -> usize { self.itens.len() }
}

/// Problema 1: parênteses balanceados. Abertura empilha; fechamento exige
/// que o topo seja o par certo. No fim, a pilha precisa estar vazia.
fn conferir_parenteses(texto: &str) -> Result<(), String> {
    let mut pilha = Pilha::nova();
    for (i, c) in texto.chars().enumerate() {
        match c {
            '(' | '[' | '{' => pilha.empilhar(c),
            ')' | ']' | '}' => {
                let esperado = match c {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                };
                match pilha.topo() {
                    Some(&abre) if abre == esperado => {
                        pilha.desempilhar();
                    }
                    Some(&abre) => return Err(format!("posição {i}: '{c}' fecha '{abre}', que estava aberto")),
                    None => return Err(format!("posição {i}: '{c}' sem abertura correspondente")),
                }
            }
            _ => {}
        }
    }
    if pilha.esta_vazia() {
        Ok(())
    } else {
        Err(format!("{} abertura(s) sem fechamento", pilha.tamanho()))
    }
}

/// Problema 2: desfazer e refazer. Toda edição guarda o estado anterior na
/// pilha de desfazer; desfazer move o estado atual para a pilha de refazer.
struct Editor {
    texto: String,
    pilha_desfazer: Pilha<String>,
    pilha_refazer: Pilha<String>,
}

impl Editor {
    fn novo() -> Self {
        Editor { texto: String::new(), pilha_desfazer: Pilha::nova(), pilha_refazer: Pilha::nova() }
    }
    fn digitar(&mut self, palavra: &str) {
        self.pilha_desfazer.empilhar(self.texto.clone());
        // Uma edição nova invalida tudo o que estava na pilha de refazer.
        self.pilha_refazer = Pilha::nova();
        if !self.texto.is_empty() {
            self.texto.push(' ');
        }
        self.texto.push_str(palavra);
    }
    fn desfazer(&mut self) -> bool {
        match self.pilha_desfazer.desempilhar() {
            // mem::replace troca o texto atual pelo anterior e devolve o atual.
            Some(anterior) => {
                let atual = std::mem::replace(&mut self.texto, anterior);
                self.pilha_refazer.empilhar(atual);
                true
            }
            None => false,
        }
    }
    fn refazer(&mut self) -> bool {
        match self.pilha_refazer.desempilhar() {
            Some(seguinte) => {
                let atual = std::mem::replace(&mut self.texto, seguinte);
                self.pilha_desfazer.empilhar(atual);
                true
            }
            None => false,
        }
    }
    fn mostrar(&self, acao: &str) {
        println!(
            "    {acao:<12} texto = \"{}\"  (desfazer: {}, refazer: {})",
            self.texto,
            self.pilha_desfazer.tamanho(),
            self.pilha_refazer.tamanho()
        );
    }
}

fn rodar_editor(palavras: &[String]) {
    let mut editor = Editor::novo();
    for palavra in palavras {
        editor.digitar(palavra);
        editor.mostrar(&format!("digitou '{palavra}'"));
    }
    for _ in 0..2 {
        if editor.desfazer() {
            editor.mostrar("desfez");
        }
    }
    if editor.refazer() {
        editor.mostrar("refez");
    }
    editor.digitar("fim");
    editor.mostrar("digitou 'fim'");
    println!("    refazer agora: {} (a edição nova apagou a pilha de refazer)", editor.refazer());
}

/// Problema 3: fila de atendimento. Cada cliente chega em um minuto e leva
/// alguns minutos para ser atendido; a fila diz quanto cada um esperou.
struct Cliente {
    nome: String,
    chegada: u32,
    duracao: u32,
}

fn analisar_cliente(texto: &str) -> Result<Cliente, String> {
    let partes: Vec<&str> = texto.split(':').collect();
    // let-else: se o padrão não casa, o bloco else tem de sair da função.
    let [nome, chegada, duracao] = partes.as_slice() else {
        return Err(format!("cliente inválido: '{texto}' (use nome:chegada:duração)"));
    };
    let chegada = chegada.parse().map_err(|_| format!("chegada inválida em '{texto}'"))?;
    let duracao = duracao.parse().map_err(|_| format!("duração inválida em '{texto}'"))?;
    Ok(Cliente { nome: nome.to_string(), chegada, duracao })
}

fn simular_atendimento(clientes: Vec<Cliente>) {
    let mut fila = Fila::nova();
    for cliente in clientes {
        fila.enfileirar(cliente);
    }
    let primeiro = fila.frente().map_or("ninguém", |c| c.nome.as_str());
    println!("    {} pessoa(s) na fila; primeiro a ser atendido: {primeiro}", fila.tamanho());
    println!("    {:<10} {:>7} {:>7} {:>7} {:>7}", "cliente", "chegou", "início", "espera", "saiu");
    let (mut relogio, mut total_espera, mut atendidos) = (0u32, 0u32, 0u32);
    while let Some(c) = fila.desenfileirar() {
        let inicio = relogio.max(c.chegada);
        let espera = inicio - c.chegada;
        relogio = inicio + c.duracao;
        total_espera += espera;
        atendidos += 1;
        println!("    {:<10} {:>7} {:>7} {:>7} {:>7}", c.nome, c.chegada, inicio, espera, relogio);
    }
    if atendidos > 0 {
        println!("    espera média: {:.1} min; fila vazia no fim? {}", f64::from(total_espera) / f64::from(atendidos), fila.esta_vazia());
    }
}

fn uso() {
    println!("Uso: pilha_fila parenteses \"<texto com ( [ {{ >\"");
    println!("     pilha_fila editor <palavras...>");
    println!("     pilha_fila fila nome:chegada:duração ...");
    println!("     pilha_fila --demo");
}

fn executar(args: &[String]) -> Result<(), String> {
    match (args[0].as_str(), &args[1..]) {
        ("parenteses", [texto]) => match conferir_parenteses(texto) {
            Ok(()) => println!("'{texto}': balanceado"),
            Err(motivo) => println!("'{texto}': NÃO balanceado ({motivo})"),
        },
        ("editor", palavras) if !palavras.is_empty() => rodar_editor(palavras),
        ("fila", entradas) if !entradas.is_empty() => {
            let clientes = entradas.iter().map(|e| analisar_cliente(e)).collect::<Result<Vec<_>, _>>()?;
            simular_atendimento(clientes);
        }
        (comando, _) => return Err(format!("comando ou argumentos inválidos: '{comando}'. Veja o uso sem argumentos.")),
    }
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== CodeNomad Academy: pilha e fila em três problemas ===");
    println!("\n--- 1. Parênteses balanceados (pilha) ---");
    for texto in ["{[(1 + 2) * 3]}", "(a + b]", "((x)", "fn soma(a: i32) -> i32 { a + 1 }"] {
        match conferir_parenteses(texto) {
            Ok(()) => println!("    '{texto}': balanceado"),
            Err(motivo) => println!("    '{texto}': NÃO balanceado ({motivo})"),
        }
    }
    println!("\n--- 2. Desfazer e refazer (duas pilhas) ---");
    let palavras: Vec<String> = ["aprender", "Rust", "com", "calma"].iter().map(|p| p.to_string()).collect();
    rodar_editor(&palavras);
    println!("\n--- 3. Fila de atendimento (fila) ---");
    let entradas = ["Ana:0:5", "Bruno:1:3", "Carla:2:4", "Diego:10:2", "Elisa:11:6"];
    let clientes = entradas.iter().map(|e| analisar_cliente(e)).collect::<Result<Vec<_>, _>>()?;
    simular_atendimento(clientes);
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resultado = match args.first().map(String::as_str) {
        None => {
            uso();
            Ok(())
        }
        Some("--demo") => demo(),
        Some(_) => executar(&args),
    };
    if let Err(mensagem) = resultado {
        eprintln!("Erro: {mensagem}");
        std::process::exit(1);
    }
}

// ------------------------------------------------------------
// Para ir além:
// 1. Implemente a Fila com dois Vec (um de entrada, um de saída) sem VecDeque
//    e compare o custo de desenfileirar.
// 2. Dê ao Editor um limite de 5 estados de desfazer, descartando o mais antigo.
// 3. Na fila de atendimento, abra dois guichês e veja como a espera média cai.
// ------------------------------------------------------------
