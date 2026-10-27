// ============================================================
// CodeNomad Academy  |  Sources Rust  |  25. Fila de prioridade com heap
// ------------------------------------------------------------
// O que faz: implementa um heap binário mínimo do zero (inserir, extrair,
//            espiar) em cima de um Vec e usa a estrutura numa simulação de
//            pronto-socorro: quem chega com gravidade maior é atendido antes,
//            mesmo tendo chegado depois.
// Como usar: rustc -O --edition 2021 -D warnings 25_fila_prioridade.rs -o fila.exe
//            fila.exe Ana:3 Bruno:1 Carla:5 Davi:2
//            fila.exe --demo
// O que você aprende: structs genéricas com trait bound (T: Ord), métodos
//            que devolvem Option, derive de Ord e PartialOrd em structs,
//            Vec como árvore implícita (filhos em 2i+1 e 2i+2) e swap.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::process;

/// Heap mínimo genérico: o menor elemento fica sempre na posição 0.
/// Para um heap máximo, basta inverter as comparações (ou usar Reverse).
struct HeapMinimo<T: Ord> {
    dados: Vec<T>,
}

impl<T: Ord> HeapMinimo<T> {
    fn novo() -> Self {
        HeapMinimo { dados: Vec::new() }
    }

    fn tamanho(&self) -> usize {
        self.dados.len()
    }

    /// Insere no fim do vetor e "sobe" o elemento até o pai ser menor.
    fn inserir(&mut self, valor: T) {
        self.dados.push(valor);
        let mut i = self.dados.len() - 1;
        while i > 0 {
            let pai = (i - 1) / 2;
            if self.dados[i] < self.dados[pai] {
                self.dados.swap(i, pai);
                i = pai;
            } else {
                break;
            }
        }
    }

    /// Olha o menor sem tirar. Option porque o heap pode estar vazio.
    fn espiar(&self) -> Option<&T> {
        self.dados.first()
    }

    /// Troca a raiz pelo último, remove o último e "desce" a nova raiz.
    fn extrair(&mut self) -> Option<T> {
        if self.dados.is_empty() {
            return None;
        }
        let ultimo = self.dados.len() - 1;
        self.dados.swap(0, ultimo);
        let menor = self.dados.pop();
        let n = self.dados.len();
        let mut i = 0;
        loop {
            let esq = 2 * i + 1;
            let dir = 2 * i + 2;
            let mut alvo = i;
            // Escolhe o menor entre o nó e os dois filhos (se existirem).
            if esq < n && self.dados[esq] < self.dados[alvo] {
                alvo = esq;
            }
            if dir < n && self.dados[dir] < self.dados[alvo] {
                alvo = dir;
            }
            if alvo == i {
                break;
            }
            self.dados.swap(i, alvo);
            i = alvo;
        }
        menor
    }
}

/// Paciente do pronto-socorro. A ordem derivada compara campo a campo, na
/// ordem em que estão declarados: primeiro a prioridade, depois a chegada.
/// Prioridade menor = mais grave (1 é emergência, 5 é caso leve).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Paciente {
    prioridade: u8,
    chegada: usize,
    nome: String,
}

fn descrever_gravidade(p: u8) -> &'static str {
    match p {
        1 => "emergência",
        2 => "muito urgente",
        3 => "urgente",
        4 => "pouco urgente",
        _ => "não urgente",
    }
}

/// Converte "Nome:gravidade" em Paciente. Devolve Err com mensagem clara.
fn ler_paciente(texto: &str, chegada: usize) -> Result<Paciente, String> {
    let (nome, grav) = texto
        .split_once(':')
        .ok_or_else(|| format!("use Nome:gravidade (recebi '{}')", texto))?;
    if nome.trim().is_empty() {
        return Err(format!("nome vazio em '{}'", texto));
    }
    let prioridade: u8 = grav
        .trim()
        .parse()
        .map_err(|_| format!("gravidade inválida em '{}' (use 1 a 5)", texto))?;
    if !(1..=5).contains(&prioridade) {
        return Err(format!("gravidade {} fora da faixa 1 a 5 em '{}'", prioridade, texto));
    }
    Ok(Paciente { prioridade, chegada, nome: nome.trim().to_string() })
}

fn simular_pronto_socorro(pacientes: Vec<Paciente>) {
    println!("Chegada (ordem em que entraram pela porta):");
    for p in &pacientes {
        println!("    {:>2}. {:<8} gravidade {} ({})", p.chegada, p.nome, p.prioridade, descrever_gravidade(p.prioridade));
    }
    let mut heap = HeapMinimo::novo();
    for p in pacientes {
        heap.inserir(p);
    }
    if let Some(primeiro) = heap.espiar() {
        println!("\nEspiar: o próximo a ser chamado é {} (sem retirar do heap)", primeiro.nome);
    }
    println!("\nAtendimento (por gravidade; empate resolvido pela chegada):");
    let mut ordem = 1;
    while let Some(p) = heap.extrair() {
        println!("    {:>2}. {:<8} gravidade {}  chegou em {}º", ordem, p.nome, p.prioridade, p.chegada);
        ordem += 1;
    }
    println!("Heap vazio: espiar() devolve {:?}", heap.espiar().map(|p| &p.nome));
}

fn demo() {
    println!("=== Heap mínimo de inteiros ===");
    let mut heap = HeapMinimo::novo();
    for v in [42, 7, 19, 3, 25, 11, 3] {
        heap.inserir(v);
        println!("    inserir {:>2} -> vetor interno {:?}", v, heap.dados);
    }
    println!("    tamanho: {}, menor no topo: {:?}", heap.tamanho(), heap.espiar());
    let mut saida = Vec::new();
    while let Some(v) = heap.extrair() {
        saida.push(v);
    }
    println!("    extraindo tudo sai ordenado: {:?}", saida);

    println!("\n=== Simulação de pronto-socorro ===");
    let lista = ["Ana:3", "Bruno:1", "Carla:5", "Davi:2", "Elisa:3", "Fábio:1", "Gabi:4"];
    let pacientes: Vec<Paciente> = lista
        .iter()
        .enumerate()
        .filter_map(|(i, t)| ler_paciente(t, i + 1).ok())
        .collect();
    simular_pronto_socorro(pacientes);
}

fn uso() {
    println!("Uso: fila_prioridade Nome:gravidade [Nome:gravidade ...]");
    println!("     gravidade de 1 (emergência) a 5 (não urgente)");
    println!("     fila_prioridade --demo");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        uso();
        return;
    }
    if args.iter().any(|a| a == "--demo") {
        demo();
        return;
    }
    let mut pacientes = Vec::new();
    for (i, a) in args.iter().enumerate() {
        match ler_paciente(a, i + 1) {
            Ok(p) => pacientes.push(p),
            Err(e) => {
                eprintln!("Erro: {}", e);
                process::exit(1);
            }
        }
    }
    simular_pronto_socorro(pacientes);
}

// Para ir além:
//   1. Transforme o heap em máximo usando std::cmp::Reverse ao inserir, sem
//      mudar o código da struct.
//   2. Adicione um método "construir(Vec<T>)" que monta o heap em O(n) usando
//      o algoritmo de heapify de baixo para cima.
//   3. Dê a cada paciente um tempo de atendimento e calcule quanto cada um
//      esperou, comparando com uma fila simples por ordem de chegada.
