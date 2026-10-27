// ============================================================
// CodeNomad Academy  |  Sources Rust  |  28. Operações com matrizes
// ------------------------------------------------------------
// O que faz: soma, subtração, multiplicação, transposta, determinante
//            (recursivo por expansão de Laplace) e matriz identidade, com
//            validação de dimensões e impressão alinhada em colunas.
// Como usar: rustc -O --edition 2021 -D warnings 28_matriz_operacoes.rs -o matriz.exe
//            matriz.exe mult "1 2; 3 4" "5 6; 7 8"
//            matriz.exe det "2 0 1; 1 3 2; 1 1 4"
//            matriz.exe soma "1 2; 3 4" "10 20; 30 40"
//            matriz.exe --demo
// O que você aprende: struct com Vec<Vec<f64>>, implementação dos traits
//            Add, Sub e Mul para usar + - * em tipos próprios, Display
//            para impressão alinhada, recursão e Result com erros de
//            dimensão.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::env;
use std::fmt;
use std::ops::{Add, Mul, Sub};
use std::process;

#[derive(Debug, Clone, PartialEq)]
struct Matriz {
    linhas: usize,
    colunas: usize,
    dados: Vec<Vec<f64>>,
}

impl Matriz {
    fn nova(dados: Vec<Vec<f64>>) -> Result<Matriz, String> {
        let linhas = dados.len();
        if linhas == 0 {
            return Err("matriz vazia".to_string());
        }
        let colunas = dados[0].len();
        // Toda linha precisa ter o mesmo número de colunas.
        if colunas == 0 || dados.iter().any(|l| l.len() != colunas) {
            return Err("todas as linhas precisam ter o mesmo número de colunas".to_string());
        }
        Ok(Matriz { linhas, colunas, dados })
    }

    fn identidade(n: usize) -> Matriz {
        let dados = (0..n).map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect()).collect();
        Matriz { linhas: n, colunas: n, dados }
    }

    /// Lê "1 2; 3 4": ponto e vírgula separa linhas, espaço separa colunas.
    fn de_texto(texto: &str) -> Result<Matriz, String> {
        let mut dados = Vec::new();
        for linha in texto.split(';') {
            let valores: Result<Vec<f64>, String> = linha
                .split_whitespace()
                .map(|v| v.parse::<f64>().map_err(|_| format!("'{}' não é um número", v)))
                .collect();
            dados.push(valores?);
        }
        Matriz::nova(dados)
    }

    fn transposta(&self) -> Matriz {
        let dados = (0..self.colunas).map(|j| (0..self.linhas).map(|i| self.dados[i][j]).collect()).collect();
        Matriz { linhas: self.colunas, colunas: self.linhas, dados }
    }

    /// Determinante por expansão de Laplace na primeira linha. Cada termo
    /// usa o cofator: a submatriz sem a linha 0 e sem a coluna j.
    /// Simples de entender, mas custa O(n!): bom até uns 8x8.
    fn determinante(&self) -> Result<f64, String> {
        if self.linhas != self.colunas {
            return Err(format!("determinante exige matriz quadrada (esta é {}x{})", self.linhas, self.colunas));
        }
        Ok(det_recursivo(&self.dados))
    }

    fn mesma_forma(&self, outra: &Matriz, op: &str) -> Result<(), String> {
        if self.linhas != outra.linhas || self.colunas != outra.colunas {
            return Err(format!(
                "{} exige dimensões iguais ({}x{} e {}x{})",
                op, self.linhas, self.colunas, outra.linhas, outra.colunas
            ));
        }
        Ok(())
    }

    /// Combina elemento a elemento. Serve para soma e subtração.
    fn elemento_a_elemento(&self, outra: &Matriz, f: fn(f64, f64) -> f64) -> Matriz {
        let dados = self
            .dados
            .iter()
            .zip(&outra.dados)
            .map(|(a, b)| a.iter().zip(b).map(|(x, y)| f(*x, *y)).collect())
            .collect();
        Matriz { linhas: self.linhas, colunas: self.colunas, dados }
    }
}

fn det_recursivo(m: &[Vec<f64>]) -> f64 {
    let n = m.len();
    if n == 1 {
        return m[0][0];
    }
    if n == 2 {
        return m[0][0] * m[1][1] - m[0][1] * m[1][0];
    }
    let mut total = 0.0;
    for j in 0..n {
        let sub: Vec<Vec<f64>> = m[1..]
            .iter()
            .map(|linha| linha.iter().enumerate().filter(|(c, _)| *c != j).map(|(_, v)| *v).collect())
            .collect();
        let sinal = if j % 2 == 0 { 1.0 } else { -1.0 };
        total += sinal * m[0][j] * det_recursivo(&sub);
    }
    total
}

// Os operadores devolvem Result porque as dimensões podem não bater.
impl Add for &Matriz {
    type Output = Result<Matriz, String>;
    fn add(self, outra: &Matriz) -> Self::Output {
        self.mesma_forma(outra, "soma")?;
        Ok(self.elemento_a_elemento(outra, |a, b| a + b))
    }
}

impl Sub for &Matriz {
    type Output = Result<Matriz, String>;
    fn sub(self, outra: &Matriz) -> Self::Output {
        self.mesma_forma(outra, "subtração")?;
        Ok(self.elemento_a_elemento(outra, |a, b| a - b))
    }
}

impl Mul for &Matriz {
    type Output = Result<Matriz, String>;
    /// A x B só existe se o número de colunas de A for igual ao de linhas de B.
    fn mul(self, outra: &Matriz) -> Self::Output {
        if self.colunas != outra.linhas {
            return Err(format!(
                "multiplicação exige colunas de A = linhas de B ({}x{} vezes {}x{})",
                self.linhas, self.colunas, outra.linhas, outra.colunas
            ));
        }
        let dados = (0..self.linhas)
            .map(|i| {
                (0..outra.colunas)
                    .map(|j| (0..self.colunas).map(|k| self.dados[i][k] * outra.dados[k][j]).sum())
                    .collect()
            })
            .collect();
        Ok(Matriz { linhas: self.linhas, colunas: outra.colunas, dados })
    }
}

impl fmt::Display for Matriz {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Largura da maior célula, para as colunas ficarem alinhadas.
        let largura = self.dados.iter().flatten().map(|v| v.to_string().len()).max().unwrap_or(1);
        for linha in &self.dados {
            let celulas: Vec<String> = linha.iter().map(|v| format!("{:>w$}", v, w = largura)).collect();
            writeln!(f, "    | {} |", celulas.join("  "))?;
        }
        Ok(())
    }
}

fn mostrar(titulo: &str, resultado: Result<Matriz, String>) {
    match resultado {
        Ok(m) => println!("{} ({}x{}):\n{}", titulo, m.linhas, m.colunas, m),
        Err(e) => println!("{}: erro: {}", titulo, e),
    }
}

fn demo() {
    let a = Matriz::de_texto("1 2; 3 4").unwrap_or_else(|_| Matriz::identidade(2));
    let b = Matriz::de_texto("5 6; 7 8").unwrap_or_else(|_| Matriz::identidade(2));
    let c = Matriz::de_texto("2 0 1; 1 3 2; 1 1 4").unwrap_or_else(|_| Matriz::identidade(3));
    println!("=== Matrizes 2x2 ===");
    println!("A:\n{}B:\n{}", a, b);
    mostrar("A + B", &a + &b);
    mostrar("A - B", &a - &b);
    mostrar("A x B", &a * &b);
    println!("det(A) = {:?}", a.determinante());
    println!("\n=== Matriz 3x3 ===");
    println!("C:\n{}", c);
    println!("Transposta de C:\n{}", c.transposta());
    println!("det(C) = {:?}", c.determinante());
    let i3 = Matriz::identidade(3);
    println!("Identidade 3x3:\n{}", i3);
    println!("C x I = C? {}", (&c * &i3).map(|r| r == c).unwrap_or(false));
    println!("\n=== Validação de dimensões ===");
    mostrar("A + C", &a + &c);
    mostrar("A x C", &a * &c);
}

fn uso() {
    println!("Uso: matriz_operacoes soma|sub|mult \"<A>\" \"<B>\"");
    println!("     matriz_operacoes det|transposta \"<A>\"");
    println!("     matriz_operacoes identidade <n>");
    println!("     formato: \"1 2; 3 4\" (ponto e vírgula separa linhas)");
    println!("     matriz_operacoes --demo");
}

fn executar(args: &[String]) -> Result<(), String> {
    let comando = args[0].as_str();
    match (comando, args.len()) {
        ("soma", 3) | ("sub", 3) | ("mult", 3) => {
            let a = Matriz::de_texto(&args[1])?;
            let b = Matriz::de_texto(&args[2])?;
            let r = match comando {
                "soma" => (&a + &b)?,
                "sub" => (&a - &b)?,
                _ => (&a * &b)?,
            };
            println!("Resultado ({}x{}):\n{}", r.linhas, r.colunas, r);
        }
        ("det", 2) => println!("det = {}", Matriz::de_texto(&args[1])?.determinante()?),
        ("transposta", 2) => print!("{}", Matriz::de_texto(&args[1])?.transposta()),
        ("identidade", 2) => {
            let n: usize = args[1].parse().map_err(|_| format!("'{}' não é um tamanho válido", args[1]))?;
            if n == 0 || n > 20 {
                return Err("o tamanho precisa estar entre 1 e 20".to_string());
            }
            print!("{}", Matriz::identidade(n));
        }
        _ => return Err(format!("comando ou número de argumentos inválido: '{}'", args.join(" "))),
    }
    Ok(())
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
    if let Err(e) = executar(&args) {
        eprintln!("Erro: {}", e);
        uso();
        process::exit(1);
    }
}

// Para ir além:
//   1. Implemente a inversa por Gauss-Jordan e confira que A x inv(A) dá a
//      identidade (com tolerância para erros de ponto flutuante).
//   2. Troque o determinante recursivo por eliminação gaussiana, O(n^3), e
//      meça a diferença numa matriz 10x10.
//   3. Faça a struct genérica (Matriz<T>) com T: Copy + Add + Mul, para
//      funcionar também com inteiros e frações.
