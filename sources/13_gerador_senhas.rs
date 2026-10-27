// ============================================================
// CodeNomad Academy  |  Sources Rust  |  13. Gerador de senhas seguras
// ------------------------------------------------------------
// O que faz: Gera senhas com comprimento e classes de símbolos à escolha,
//            usando o gerador criptográfico do sistema operacional, garante
//            pelo menos um símbolo de cada classe pedida, sorteia sem viés
//            e estima a entropia em bits e o tempo para quebrar por força bruta.
// Como usar: 13_gerador_senhas.exe 20 --quantidade 3 --sem-ambiguos
//            13_gerador_senhas.exe 12 --sem-simbolos
//            13_gerador_senhas.exe --demo
// O que você aprende: cfg por sistema operacional, leitura de /dev/urandom,
//            uma chamada FFI mínima e documentada, amostragem por rejeição
//            (sem viés de módulo), embaralhamento de Fisher-Yates,
//            Box<dyn Error> para misturar tipos de erro.
// Requisitos: Rust 1.70 ou mais novo, só biblioteca padrão
// Licença MIT. Um projeto da CodeNomad Academy (codenomad.org)
// ============================================================

use std::{error::Error, io};

const MINUSCULAS: &str = "abcdefghijklmnopqrstuvwxyz";
const MAIUSCULAS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITOS: &str = "0123456789";
const SIMBOLOS: &str = "!@#$%&*()-_=+[]{};:,.<>?/";
// Símbolos que se confundem ao ler em voz alta ou em fontes pequenas.
const AMBIGUOS: &str = "l1IO0|";

type Resultado<T> = Result<T, Box<dyn Error>>;

/// Lê bytes do gerador criptográfico do sistema operacional.
fn bytes_do_sistema(quantidade: usize) -> io::Result<Vec<u8>> {
    let mut reserva = vec![0u8; quantidade];
    #[cfg(unix)]
    {
        // Em Linux, macOS e BSDs o kernel expõe o gerador como um arquivo.
        use std::io::Read;
        std::fs::File::open("/dev/urandom")?.read_exact(&mut reserva)?;
    }
    #[cfg(windows)]
    {
        // No Windows a função fica na advapi32: RtlGenRandom, exportada com
        // o nome SystemFunction036. É o mesmo gerador que o sistema usa.
        #[link(name = "advapi32")]
        extern "system" {
            fn SystemFunction036(buffer: *mut u8, tamanho: u32) -> u8;
        }
        // SAFETY: o ponteiro aponta para um Vec vivo com exatamente `quantidade` bytes.
        let ok = unsafe { SystemFunction036(reserva.as_mut_ptr(), quantidade as u32) };
        if ok == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "o sistema não conseguiu gerar bytes aleatórios"));
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        return Err(io::Error::new(io::ErrorKind::Other, "este sistema não tem fonte segura conhecida"));
    }
    Ok(reserva)
}

/// Guarda uma reserva de bytes para não pedir ao sistema um byte por vez.
struct FonteSegura {
    reserva: Vec<u8>,
    posicao: usize,
}

impl FonteSegura {
    fn nova() -> Self { FonteSegura { reserva: Vec::new(), posicao: 0 } }

    fn byte(&mut self) -> io::Result<u8> {
        if self.posicao >= self.reserva.len() {
            self.reserva = bytes_do_sistema(256)?;
            self.posicao = 0;
        }
        let b = self.reserva[self.posicao];
        self.posicao += 1;
        Ok(b)
    }

    /// Sorteia um índice de 0 até `limite` (exclusivo) sem viés: um simples
    /// `valor % limite` favoreceria os primeiros índices, então rejeitamos
    /// a faixa final que não divide certo e sorteamos de novo.
    fn indice(&mut self, limite: usize) -> io::Result<usize> {
        let limite32 = limite as u32;
        let maior_aceito = u32::MAX - (u32::MAX % limite32);
        loop {
            let valor = u32::from_le_bytes([self.byte()?, self.byte()?, self.byte()?, self.byte()?]);
            if valor < maior_aceito {
                return Ok((valor % limite32) as usize);
            }
        }
    }

    /// Fisher-Yates: cada permutação sai com a mesma probabilidade.
    fn embaralhar(&mut self, itens: &mut [char]) -> io::Result<()> {
        for i in (1..itens.len()).rev() {
            let j = self.indice(i + 1)?;
            itens.swap(i, j);
        }
        Ok(())
    }
}

struct Perfil {
    tamanho: usize,
    minusculas: bool,
    maiusculas: bool,
    digitos: bool,
    simbolos: bool,
    sem_ambiguos: bool,
}

impl Perfil {
    fn padrao(tamanho: usize) -> Self {
        Perfil { tamanho, minusculas: true, maiusculas: true, digitos: true, simbolos: true, sem_ambiguos: false }
    }

    /// Uma lista de símbolos por classe pedida, já sem os ambíguos se for o caso.
    fn classes(&self) -> Vec<Vec<char>> {
        [(self.minusculas, MINUSCULAS), (self.maiusculas, MAIUSCULAS), (self.digitos, DIGITOS), (self.simbolos, SIMBOLOS)]
            .into_iter()
            .filter(|(usada, _)| *usada)
            .map(|(_, letras)| letras.chars().filter(|c| !self.sem_ambiguos || !AMBIGUOS.contains(*c)).collect())
            .collect()
    }

    /// Confere que o perfil faz sentido antes de sortear qualquer coisa.
    fn validar(&self) -> Resultado<()> {
        let n_classes = self.classes().len();
        if n_classes == 0 {
            return Err("escolha pelo menos uma classe de símbolos".into());
        }
        if self.tamanho < n_classes {
            return Err(format!("tamanho {} não cabe um símbolo de cada uma das {n_classes} classes", self.tamanho).into());
        }
        Ok(())
    }

    fn descricao(&self) -> String {
        let nomes = [(self.minusculas, "minúsculas"), (self.maiusculas, "maiúsculas"), (self.digitos, "dígitos"), (self.simbolos, "símbolos")];
        let usadas: Vec<&str> = nomes.iter().filter(|(u, _)| *u).map(|(_, n)| *n).collect();
        format!("{} símbolos, {}{}", self.tamanho, usadas.join(" + "), if self.sem_ambiguos { ", sem ambíguos" } else { "" })
    }
}

fn gerar(perfil: &Perfil, fonte: &mut FonteSegura) -> Resultado<String> {
    perfil.validar()?;
    let classes = perfil.classes();
    let uniao: Vec<char> = classes.iter().flatten().copied().collect();
    let mut senha: Vec<char> = Vec::with_capacity(perfil.tamanho);
    // Primeiro um símbolo de cada classe (a garantia), depois o resto da união,
    // e por fim embaralha para a garantia não ficar sempre no começo.
    for classe in &classes {
        senha.push(classe[fonte.indice(classe.len())?]);
    }
    while senha.len() < perfil.tamanho {
        senha.push(uniao[fonte.indice(uniao.len())?]);
    }
    fonte.embaralhar(&mut senha)?;
    Ok(senha.into_iter().collect())
}

/// Entropia aproximada: log2(tamanho do alfabeto) por símbolo sorteado.
fn entropia_bits(perfil: &Perfil) -> f64 {
    let alfabeto: usize = perfil.classes().iter().map(Vec::len).sum();
    perfil.tamanho as f64 * (alfabeto.max(1) as f64).log2()
}

/// Tempo médio para esgotar metade das combinações a 10 bilhões de tentativas por segundo.
fn tempo_para_quebrar(bits: f64) -> String {
    let escalas = [(60.0, "segundos"), (60.0, "minutos"), (24.0, "horas"), (365.25, "dias"), (100.0, "anos")];
    let mut valor = (bits - 1.0).exp2() / 1e10;
    for (fator, nome) in escalas {
        if valor < fator {
            return format!("{valor:.1} {nome}");
        }
        valor /= fator;
    }
    // Acima de um milhão de séculos, notação científica lê melhor ("{:e}").
    if valor < 1e6 { format!("{valor:.0} séculos") } else { format!("{valor:.1e} séculos") }
}

fn mostrar(perfil: &Perfil, quantidade: usize, fonte: &mut FonteSegura) -> Resultado<()> {
    perfil.validar()?;
    let bits = entropia_bits(perfil);
    println!("Perfil: {}  |  entropia ~ {bits:.1} bits  |  força bruta: {}", perfil.descricao(), tempo_para_quebrar(bits));
    for _ in 0..quantidade {
        println!("  {}", gerar(perfil, fonte)?);
    }
    Ok(())
}

fn demo() -> Resultado<()> {
    println!("=== Gerador de senhas seguras: demonstração ===\n");
    let mut fonte = FonteSegura::nova();
    // Sintaxe de atualização de struct: muda um campo e copia o resto do padrão.
    let curto = Perfil { simbolos: false, ..Perfil::padrao(12) };
    let longo = Perfil { sem_ambiguos: true, ..Perfil::padrao(24) };
    for perfil in [curto, Perfil::padrao(16), longo] {
        mostrar(&perfil, 2, &mut fonte)?;
        println!();
    }
    println!("Cada execução gera senhas diferentes: a fonte é o gerador do sistema.");
    Ok(())
}

fn uso() {
    println!("Uso: 13_gerador_senhas.exe <tamanho> [opções]");
    println!("  --quantidade N      gera N senhas (padrão 1)");
    println!("  --sem-minusculas    --sem-maiusculas    --sem-digitos    --sem-simbolos");
    println!("  --sem-ambiguos      tira l 1 I O 0 |");
    println!("  --demo              demonstração completa");
}

fn executar(args: &[String]) -> Resultado<()> {
    if args.is_empty() {
        uso();
        return Ok(());
    }
    if args[0] == "--demo" {
        return demo();
    }
    let tamanho: usize = args[0].parse().map_err(|_| format!("tamanho inválido: '{}'", args[0]))?;
    if !(4..=256).contains(&tamanho) {
        return Err("o tamanho precisa ficar entre 4 e 256".into());
    }
    let mut perfil = Perfil::padrao(tamanho);
    let mut quantidade = 1;
    let mut restante = args[1..].iter();
    while let Some(opcao) = restante.next() {
        match opcao.as_str() {
            "--sem-minusculas" => perfil.minusculas = false,
            "--sem-maiusculas" => perfil.maiusculas = false,
            "--sem-digitos" => perfil.digitos = false,
            "--sem-simbolos" => perfil.simbolos = false,
            "--sem-ambiguos" => perfil.sem_ambiguos = true,
            "--quantidade" => {
                let valor = restante.next().ok_or("--quantidade precisa de um número")?;
                quantidade = valor.parse::<usize>().map_err(|_| format!("quantidade inválida: '{valor}'"))?.clamp(1, 100);
            }
            outra => return Err(format!("opção desconhecida: '{outra}'").into()),
        }
    }
    mostrar(&perfil, quantidade, &mut FonteSegura::nova())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(erro) = executar(&args) {
        eprintln!("Erro: {erro}");
        std::process::exit(1);
    }
}

// Para ir além:
// 1. Adicione a opção --frase para gerar senhas de quatro palavras sorteadas
//    de uma lista embutida, separadas por hífen, e compare a entropia.
// 2. Permita que o usuário passe os próprios símbolos com --simbolos "@#!".
// 3. Escreva um teste (#[test]) que gera mil senhas e confirma que todas
//    contêm pelo menos um símbolo de cada classe pedida.
