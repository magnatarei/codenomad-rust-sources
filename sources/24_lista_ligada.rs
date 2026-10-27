// ============================================================
// CodeNomad Academy  |  Sources Rust  |  24. Lista ligada do zero
// ------------------------------------------------------------
// O que faz: implementa uma lista simplesmente ligada com inserir no
//            início, no fim e em uma posição, remover, buscar, inverter,
//            tamanho e impressão, e compara com o Vec em uma operação.
// Como usar: rustc -O --edition 2021 -D warnings 24_lista_ligada.rs -o lista.exe
//            lista.exe inicio=5 fim=7 em=1:9 remover=7 buscar=9 inverter
//            lista.exe --demo
// O que você aprende: Option<Box<No>> como ponteiro, take() para mover o
//            dono, percorrer com &mut sem brigar com o borrow checker,
//            iter::successors, Display, Drop em laço e split_once.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::fmt;
use std::time::Instant;

/// Cada nó é dono do próximo: Box guarda o nó no heap e Option representa
/// o fim da lista. Sem ponteiro nulo e sem vazamento.
struct No {
    valor: i32,
    proximo: Option<Box<No>>,
}

struct Lista {
    cabeca: Option<Box<No>>,
    tamanho: usize,
}

impl Lista {
    fn nova() -> Self {
        Lista { cabeca: None, tamanho: 0 }
    }

    fn tamanho(&self) -> usize {
        self.tamanho
    }

    /// Inserir no início custa sempre o mesmo: o nó novo aponta para a
    /// cabeça antiga. take() tira a cabeça da lista e deixa None no lugar.
    fn inserir_inicio(&mut self, valor: i32) {
        let novo = Box::new(No { valor, proximo: self.cabeca.take() });
        self.cabeca = Some(novo);
        self.tamanho += 1;
    }

    /// Inserir no fim exige caminhar até o último nó: custo proporcional ao
    /// tamanho. O laço anda com uma referência mutável, nó a nó.
    fn inserir_fim(&mut self, valor: i32) {
        let mut atual = &mut self.cabeca;
        while let Some(no) = atual {
            atual = &mut no.proximo;
        }
        *atual = Some(Box::new(No { valor, proximo: None }));
        self.tamanho += 1;
    }

    /// Inserir na posição p: anda p passos e encaixa o nó novo ali.
    fn inserir_em(&mut self, posicao: usize, valor: i32) -> Result<(), String> {
        if posicao > self.tamanho {
            return Err(format!("posição {posicao} fora da lista (tamanho {})", self.tamanho));
        }
        let mut atual = &mut self.cabeca;
        for _ in 0..posicao {
            atual = &mut atual.as_mut().ok_or("lista mais curta que o esperado")?.proximo;
        }
        let novo = Box::new(No { valor, proximo: atual.take() });
        *atual = Some(novo);
        self.tamanho += 1;
        Ok(())
    }

    /// Remover da posição p devolve o valor que estava lá.
    fn remover_em(&mut self, posicao: usize) -> Option<i32> {
        let mut atual = &mut self.cabeca;
        for _ in 0..posicao {
            atual = &mut atual.as_mut()?.proximo;
        }
        // take() tira o nó; o anterior passa a apontar para o seguinte dele.
        let removido = atual.take()?;
        let No { valor, proximo } = *removido;
        *atual = proximo;
        self.tamanho -= 1;
        Some(valor)
    }

    /// Remover pelo valor reaproveita buscar e remover_em.
    fn remover_valor(&mut self, valor: i32) -> Option<usize> {
        let posicao = self.buscar(valor)?;
        self.remover_em(posicao);
        Some(posicao)
    }

    /// Iterador sobre os valores: successors começa na cabeça e pede o
    /// próximo de cada nó até receber None. as_deref enxerga o No dentro do Box.
    fn iter(&self) -> impl Iterator<Item = i32> + '_ {
        std::iter::successors(self.cabeca.as_deref(), |no| no.proximo.as_deref()).map(|no| no.valor)
    }

    fn buscar(&self, valor: i32) -> Option<usize> {
        self.iter().position(|v| v == valor)
    }

    /// Inverter: desencaixa nó por nó da lista antiga e empilha na nova.
    fn inverter(&mut self) {
        let mut anterior = None;
        let mut atual = self.cabeca.take();
        while let Some(mut no) = atual {
            atual = no.proximo.take();
            no.proximo = anterior;
            anterior = Some(no);
        }
        self.cabeca = anterior;
    }
}

impl fmt::Display for Lista {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let partes: Vec<String> = self.iter().map(|v| v.to_string()).collect();
        write!(f, "[{}] (tamanho {})", partes.join(" -> "), self.tamanho)
    }
}

/// O Drop automático liberaria os nós um dentro do outro, em recursão, e uma
/// lista com dezenas de milhares de nós estouraria a pilha de chamadas.
/// Este Drop desmonta a lista em laço, um nó por vez.
impl Drop for Lista {
    fn drop(&mut self) {
        let mut atual = self.cabeca.take();
        while let Some(mut no) = atual {
            atual = no.proximo.take();
        }
    }
}

/// Comparação justa: inserir N valores no início. A lista faz isso em tempo
/// constante; o Vec precisa deslocar todos os elementos a cada inserção.
fn comparar_com_vec(quantidade: usize) {
    let inicio = Instant::now();
    let mut lista = Lista::nova();
    for i in 0..quantidade as i32 {
        lista.inserir_inicio(i);
    }
    let tempo_lista = inicio.elapsed();
    let inicio = Instant::now();
    let mut vetor = Vec::new();
    for i in 0..quantidade as i32 {
        vetor.insert(0, i);
    }
    let tempo_vec = inicio.elapsed();
    println!("    inserir {quantidade} valores no início: lista {tempo_lista:.1?}, Vec {tempo_vec:.1?}");
    println!("    (confere: lista com {} nós, Vec com {} itens)", lista.tamanho(), vetor.len());
}

fn aplicar(lista: &mut Lista, comando: &str) -> Result<(), String> {
    let (nome, argumento) = comando.split_once('=').unwrap_or((comando, ""));
    let numero = |texto: &str| {
        texto.parse::<i32>().map_err(|_| format!("'{texto}' não é um número inteiro (em '{comando}')"))
    };
    match nome {
        "inicio" => lista.inserir_inicio(numero(argumento)?),
        "fim" => lista.inserir_fim(numero(argumento)?),
        "em" => {
            let (posicao, valor) = argumento.split_once(':').ok_or(format!("use em=posição:valor (recebi '{comando}')"))?;
            let posicao = posicao.parse::<usize>().map_err(|_| format!("posição inválida em '{comando}'"))?;
            lista.inserir_em(posicao, numero(valor)?)?;
        }
        "remover" => match lista.remover_valor(numero(argumento)?) {
            Some(p) => println!("    removeu {argumento} da posição {p}"),
            None => println!("    {argumento} não está na lista, nada removido"),
        },
        "buscar" => match lista.buscar(numero(argumento)?) {
            Some(p) => println!("    {argumento} está na posição {p}"),
            None => println!("    {argumento} não está na lista"),
        },
        "inverter" => lista.inverter(),
        "comparar" => {
            let quantidade = argumento.parse().map_err(|_| format!("quantidade inválida em '{comando}'"))?;
            comparar_com_vec(quantidade);
        }
        _ => return Err(format!("comando desconhecido: '{nome}'. Use inicio=, fim=, em=, remover=, buscar=, inverter ou comparar=")),
    }
    println!("    {comando:<14} {lista}");
    Ok(())
}

fn uso() {
    println!("Uso: lista <comandos...>   com inicio=V  fim=V  em=P:V  remover=V  buscar=V  inverter  comparar=N");
    println!("     lista --demo");
    println!("Exemplo: lista inicio=5 fim=7 em=1:9 remover=7 buscar=9 inverter");
}

fn executar(args: &[String]) -> Result<(), String> {
    let mut lista = Lista::nova();
    for comando in args {
        aplicar(&mut lista, comando)?;
    }
    Ok(())
}

fn demo() -> Result<(), String> {
    println!("=== CodeNomad Academy: lista ligada passo a passo ===");
    let mut lista = Lista::nova();
    println!("    {:<14} {lista}", "nova");
    let comandos = ["inicio=5", "inicio=3", "fim=9", "fim=12", "em=2:7", "buscar=9", "buscar=42", "remover=9", "remover=42", "inverter"];
    for comando in comandos {
        aplicar(&mut lista, comando)?;
    }
    if let Some(valor) = lista.remover_em(0) {
        println!("    {:<14} {lista}", format!("remover_em 0"));
        println!("    (o valor removido foi {valor})");
    }
    println!("    soma com iter(): {}", lista.iter().sum::<i32>());
    println!("\n--- Lista ligada contra Vec ---");
    comparar_com_vec(20_000);
    println!("    Em compensação, ler a posição 10.000 é imediato no Vec e exige andar 10.000 nós na lista.");
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
// 1. Guarde também um ponteiro para o último nó e faça inserir_fim custar
//    tempo constante (dica: pesquise por Rc<RefCell<No>> ou índices em Vec).
// 2. Torne a lista genérica: Lista<T> com T: Display + PartialEq.
// 3. Implemente o trait Iterator de verdade (struct IterLista) em vez de successors.
// ------------------------------------------------------------
