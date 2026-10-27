// ============================================================
// CodeNomad Academy  |  Sources Rust  |  30. Grafos: BFS e DFS num labirinto
// ------------------------------------------------------------
// O que faz: lê um labirinto em texto ('#' parede, '.' corredor, 'S' início,
//            'E' saída), trata cada célula livre como um vértice ligado aos
//            vizinhos, acha o caminho mais curto com busca em largura (BFS),
//            explora com busca em profundidade (DFS) e desenha o caminho
//            encontrado sobre o labirinto.
// Como usar: rustc -O --edition 2021 -D warnings 30_grafo_bfs_dfs.rs -o labirinto.exe
//            labirinto.exe labirinto.txt
//            labirinto.exe --demo
// O que você aprende: VecDeque como fila (BFS) e Vec como pilha (DFS),
//            grade Vec<Vec<char>>, tuplas (usize, usize) como vértices,
//            HashMap para guardar o "pai" de cada célula e reconstruir o
//            caminho, e Option para "não há caminho".
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fs;
use std::process;

type Celula = (usize, usize); // (linha, coluna)

struct Labirinto {
    grade: Vec<Vec<char>>,
    inicio: Celula,
    saida: Celula,
}

/// Resultado de uma busca: o caminho (se existir) e quantas células foram
/// visitadas, para comparar o esforço de BFS e DFS.
struct Busca {
    caminho: Option<Vec<Celula>>,
    visitadas: usize,
}

impl Labirinto {
    fn de_texto(texto: &str) -> Result<Labirinto, String> {
        let grade: Vec<Vec<char>> = texto
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim_end().chars().collect())
            .collect();
        if grade.is_empty() {
            return Err("labirinto vazio".to_string());
        }
        let mut inicio = None;
        let mut saida = None;
        for (i, linha) in grade.iter().enumerate() {
            for (j, &c) in linha.iter().enumerate() {
                match c {
                    'S' => inicio = Some((i, j)),
                    'E' => saida = Some((i, j)),
                    '#' | '.' => {}
                    outro => return Err(format!("caractere inválido '{}' na linha {}, coluna {}", outro, i + 1, j + 1)),
                }
            }
        }
        Ok(Labirinto {
            grade,
            inicio: inicio.ok_or("falta o início 'S'")?,
            saida: saida.ok_or("falta a saída 'E'")?,
        })
    }

    /// Vizinhos livres (cima, baixo, esquerda, direita). É aqui que a grade
    /// vira grafo: cada célula é um vértice e cada vizinho livre, uma aresta.
    fn vizinhos(&self, (l, c): Celula) -> Vec<Celula> {
        let mut v = Vec::with_capacity(4);
        let candidatos = [
            (l.wrapping_sub(1), c),
            (l + 1, c),
            (l, c.wrapping_sub(1)),
            (l, c + 1),
        ];
        for (nl, nc) in candidatos {
            if let Some(&ch) = self.grade.get(nl).and_then(|linha| linha.get(nc)) {
                if ch != '#' {
                    v.push((nl, nc));
                }
            }
        }
        v
    }

    /// BFS: fila. Expande por camadas de distância, então o primeiro
    /// momento em que a saída aparece é também o caminho mais curto.
    fn bfs(&self) -> Busca {
        let mut fila = VecDeque::from([self.inicio]);
        let mut pai: HashMap<Celula, Celula> = HashMap::new();
        let mut visto: HashSet<Celula> = HashSet::from([self.inicio]);
        while let Some(atual) = fila.pop_front() {
            if atual == self.saida {
                return Busca { caminho: Some(reconstruir(&pai, self.inicio, self.saida)), visitadas: visto.len() };
            }
            for viz in self.vizinhos(atual) {
                if visto.insert(viz) {
                    pai.insert(viz, atual);
                    fila.push_back(viz);
                }
            }
        }
        Busca { caminho: None, visitadas: visto.len() }
    }

    /// DFS: pilha. Mergulha num corredor até o fim antes de voltar. Acha um
    /// caminho, mas não necessariamente o mais curto.
    fn dfs(&self) -> Busca {
        let mut pilha = vec![self.inicio];
        let mut pai: HashMap<Celula, Celula> = HashMap::new();
        let mut visto: HashSet<Celula> = HashSet::new();
        while let Some(atual) = pilha.pop() {
            if !visto.insert(atual) {
                continue;
            }
            if atual == self.saida {
                return Busca { caminho: Some(reconstruir(&pai, self.inicio, self.saida)), visitadas: visto.len() };
            }
            for viz in self.vizinhos(atual) {
                if !visto.contains(&viz) {
                    pai.entry(viz).or_insert(atual);
                    pilha.push(viz);
                }
            }
        }
        Busca { caminho: None, visitadas: visto.len() }
    }

    /// Desenha o labirinto com '*' nas células do caminho (menos S e E).
    fn desenhar(&self, caminho: &[Celula]) -> String {
        let marcadas: HashSet<&Celula> = caminho.iter().collect();
        let mut saida = String::new();
        for (i, linha) in self.grade.iter().enumerate() {
            saida.push_str("    ");
            for (j, &c) in linha.iter().enumerate() {
                let desenho = if c == '.' && marcadas.contains(&(i, j)) { '*' } else { c };
                saida.push(desenho);
            }
            saida.push('\n');
        }
        saida
    }
}

/// Segue os "pais" da saída até o início e inverte, obtendo o caminho.
fn reconstruir(pai: &HashMap<Celula, Celula>, inicio: Celula, saida: Celula) -> Vec<Celula> {
    let mut caminho = vec![saida];
    let mut atual = saida;
    while atual != inicio {
        match pai.get(&atual) {
            Some(&p) => {
                caminho.push(p);
                atual = p;
            }
            None => break,
        }
    }
    caminho.reverse();
    caminho
}

fn resolver(lab: &Labirinto) {
    println!("Labirinto ({} linhas x {} colunas), início {:?}, saída {:?}:",
        lab.grade.len(), lab.grade[0].len(), lab.inicio, lab.saida);
    print!("{}", lab.desenhar(&[]));
    for (nome, busca) in [("BFS (largura)", lab.bfs()), ("DFS (profundidade)", lab.dfs())] {
        println!("\n{}: {} células visitadas", nome, busca.visitadas);
        match &busca.caminho {
            Some(c) => {
                println!("    caminho com {} passos:", c.len() - 1);
                print!("{}", lab.desenhar(c));
            }
            None => println!("    sem caminho: a saída não é alcançável a partir do início"),
        }
    }
}

const DEMO_COM_SAIDA: &str = "\
##########
#S.......#
#.######.#
#.#......#
#.#.#.####
#...#....#
###.#.##.#
#...#.#..#
#.###.#.E#
##########";

const DEMO_SEM_SAIDA: &str = "\
########
#S..#..#
#.#.#.E#
#.#.####
#...#..#
########";

fn demo() {
    println!("=== Labirinto com saída ===");
    match Labirinto::de_texto(DEMO_COM_SAIDA) {
        Ok(lab) => resolver(&lab),
        Err(e) => println!("Erro: {}", e),
    }
    println!("\n=== Labirinto sem saída ===");
    match Labirinto::de_texto(DEMO_SEM_SAIDA) {
        Ok(lab) => resolver(&lab),
        Err(e) => println!("Erro: {}", e),
    }
}

fn uso() {
    println!("Uso: grafo_bfs_dfs <arquivo.txt>");
    println!("     o arquivo usa '#' parede, '.' corredor, 'S' início e 'E' saída");
    println!("     grafo_bfs_dfs --demo");
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
    let resultado = fs::read_to_string(&args[0])
        .map_err(|e| format!("não consegui ler '{}': {}", args[0], e))
        .and_then(|texto| Labirinto::de_texto(&texto));
    match resultado {
        Ok(lab) => resolver(&lab),
        Err(e) => {
            eprintln!("Erro: {}", e);
            process::exit(1);
        }
    }
}

// Para ir além:
//   1. Permita movimento nas diagonais e veja como o caminho mais curto muda.
//   2. Adicione pesos (areia '~' custa 3, corredor custa 1) e troque a BFS
//      pelo algoritmo de Dijkstra com BinaryHeap e Reverse.
//   3. Gere labirintos aleatórios com DFS recursivo (backtracker) e resolva
//      cada um com a BFS, contando quantas células a busca visita em média.
