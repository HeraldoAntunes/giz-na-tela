// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Leitores do que vem de fora, sem Tauri nem Windows
// Cabeçalho HTTP da API local, linha de comando de outro processo, SHA256SUMS.txt, notas e número da release. Ficam
// num módulo só de Rust puro para os alvos de fuzzing (pasta fuzz/) e o teste de propriedade os exercitarem com o
// mesmo código do programa.
use std::io::Read;

pub const MAX_CABECALHO: usize = 16 * 1024;

// cabeçalho até a linha em branco, com limite; devolve o que já leu e onde o cabeçalho termina.
// Err(true): grande demais; Err(false): a conexão fechou antes
pub fn ler_cabecalho(c: &mut impl Read) -> Result<(Vec<u8>, usize), bool> {
    let mut buf = Vec::new();
    let mut pedaco = [0u8; 4096];
    loop {
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") { return Ok((buf, i)); }
        if buf.len() > MAX_CABECALHO { return Err(true); }
        match c.read(&mut pedaco) { Ok(0) | Err(_) => return Err(false), Ok(n) => buf.extend_from_slice(&pedaco[..n]) }
        // o fim do cabeçalho pode chegar no mesmo bloco que passa do limite: o limite vale para o cabeçalho inteiro
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            return if i + 4 > MAX_CABECALHO { Err(true) } else { Ok((buf, i)) };
        }
    }
}

// campos que só podem vir uma vez: dois Content-Length (mesmo iguais), dois Host ou dois tokens deixam a leitura
// ambígua, e o corpo em pedaços (Transfer-Encoding) não é usado por nenhum cliente desta API
pub fn campos_repetidos(cab: &str) -> Option<String> {
    let nomes: Vec<String> = cab.split("\r\n").skip(1).filter_map(|l| Some(l.split_once(':')?.0.trim().to_ascii_lowercase())).collect();
    if nomes.iter().any(|n| n == "transfer-encoding") { return Some("Transfer-Encoding não é aceito".into()); }
    ["content-length", "host", "x-giz-tela", "origin"].iter()
        .find(|u| nomes.iter().filter(|n| n == *u).count() > 1)
        .map(|u| format!("cabeçalho {u} repetido"))
}

// separa a linha de comando em argumentos com as regras do Windows (as do CommandLineToArgvW): espaço separa fora das
// aspas; 2n barras antes de aspas viram n barras e as aspas abrem ou fecham; 2n+1 barras viram n barras e uma aspa;
// barras sem aspas depois ficam como estão
pub fn argumentos(linha: &str) -> Vec<String> {
    let (mut todos, mut atual, mut aspas, mut tem) = (Vec::new(), String::new(), false, false);
    let mut c = linha.chars().peekable();
    while let Some(ch) = c.next() {
        match ch {
            '\\' => {
                let mut barras = 1;
                while c.peek() == Some(&'\\') { c.next(); barras += 1; }
                if c.peek() == Some(&'"') {
                    atual.extend(std::iter::repeat_n('\\', barras / 2));
                    if barras % 2 == 1 { c.next(); atual.push('"'); }
                } else {
                    atual.extend(std::iter::repeat_n('\\', barras));
                }
                tem = true;
            }
            '"' => {
                if aspas && c.peek() == Some(&'"') { c.next(); atual.push('"'); } else { aspas = !aspas; }
                tem = true;
            }
            ' ' | '\t' if !aspas => {
                if tem { todos.push(std::mem::take(&mut atual)); tem = false; }
            }
            _ => { atual.push(ch); tem = true; }
        }
    }
    if tem { todos.push(atual); }
    todos
}

// --user-data-dir="C:\...\EBWebView" (com ou sem aspas), só como argumento próprio: o texto dentro do valor de outro
// argumento não conta, e duas chaves próprias são ambíguas (nenhuma vale)
pub fn pasta_do_perfil_na_linha(linha: &str) -> Option<String> {
    let mut achados = argumentos(linha).into_iter().skip(1).filter_map(|a| a.strip_prefix("--user-data-dir=").map(str::to_owned));
    let valor = achados.next()?;
    if achados.next().is_some() { return None; }
    Some(valor.trim_end_matches('\\').to_lowercase())
}

pub fn numeros(v: &str) -> Vec<u32> { v.trim_start_matches('v').split('.').map(|p| p.parse().unwrap_or(0)).collect() }

// "O que mudou" das notas da release, sem as imagens e a tabela de download, para caber na ajuda
pub fn notas(corpo: &str) -> String {
    let parte = corpo.split_once("## O que mudou").map(|(_, r)| r).unwrap_or(corpo);
    let parte = parte.split("**Conferir o download:**").next().unwrap_or(parte);
    let linhas: Vec<&str> = parte.lines().map(str::trim_end).filter(|l| !l.trim_start().starts_with("<p>")).collect();
    let texto = linhas.join("\n").replace("**", "").trim().to_string();
    if texto.chars().count() > 2000 { texto.chars().take(2000).collect::<String>() + "..." } else { texto }
}

// linha do SHA256SUMS.txt: "<64 hex>  <nome>"
pub fn soma_publicada(somas: &str, nome: &str) -> Option<String> {
    somas.lines().find_map(|l| {
        let (h, n) = l.trim().split_once(char::is_whitespace)?;
        (n.trim().trim_start_matches('*') == nome && h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
            .then(|| h.to_lowercase())
    })
}

// o que tem de valer para qualquer entrada: os alvos de fuzzing e o teste de propriedade chamam estas conferências
// (o programa em si não as usa)
#[allow(dead_code)]
pub mod propriedades {
    use super::*;

    // entrega os bytes em pedaços dos tamanhos pedidos, como uma conexão lenta
    struct EmPedacos<'a> { dados: &'a [u8], tamanhos: &'a [u8], i: usize }
    impl Read for EmPedacos<'_> {
        fn read(&mut self, saida: &mut [u8]) -> std::io::Result<usize> {
            let quer = self.tamanhos.get(self.i % self.tamanhos.len().max(1)).map_or(4096, |t| *t as usize + 1);
            self.i += 1;
            let n = quer.min(saida.len()).min(self.dados.len());
            saida[..n].copy_from_slice(&self.dados[..n]);
            self.dados = &self.dados[n..];
            Ok(n)
        }
    }

    pub fn cabecalho(dados: &[u8], tamanhos: &[u8]) {
        let r = ler_cabecalho(&mut EmPedacos { dados, tamanhos, i: 0 });
        if let Ok((buf, fim)) = &r {
            assert!(fim + 4 <= MAX_CABECALHO, "cabeçalho aceito acima do limite");
            assert_eq!(&buf[*fim..fim + 4], b"\r\n\r\n");
            assert!(!buf[..*fim + 3].windows(4).any(|w| w == b"\r\n\r\n"), "não parou na primeira linha em branco");
            assert!(dados.starts_with(buf), "devolveu bytes que não vieram da conexão");
        }
        // pedaços diferentes não mudam a resposta (só o quanto foi lido além do fim)
        let inteiro = ler_cabecalho(&mut &dados[..]);
        assert_eq!(r.as_ref().map(|(_, f)| *f).map_err(|e| *e), inteiro.as_ref().map(|(_, f)| *f).map_err(|e| *e));
        let texto = String::from_utf8_lossy(dados);
        let _ = campos_repetidos(&texto);
    }

    pub fn linha_de_comando(linha: &str) {
        let args = argumentos(linha);
        assert!(args.len() <= linha.chars().count());
        if let Some(p) = pasta_do_perfil_na_linha(linha) {
            let chaves = args.iter().skip(1).filter(|a| a.starts_with("--user-data-dir=")).count();
            assert_eq!(chaves, 1, "perfil aceito com {chaves} chaves próprias");
            assert!(!p.ends_with('\\'));
        }
    }

    pub fn release(texto: &str, nome: &str) {
        if let Some(h) = soma_publicada(texto, nome) {
            assert!(h.len() == 64 && h.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
        }
        assert!(notas(texto).chars().count() <= 2003);
        let _ = numeros(texto);
    }
}

#[cfg(test)]
mod testes {
    use super::propriedades;

    // teste de propriedade com sorteio próprio (sem biblioteca a mais): entradas montadas com os pedaços que mais
    // confundem estes leitores; a semente é fixa, então uma falha se repete igual
    #[test]
    fn propriedades_valem_para_entradas_sorteadas() {
        const PECAS: [&str; 14] = ["\r\n", "\r\n\r\n", "\\", "\\\\", "\"", " ", "\t", "--user-data-dir=", "C:\\a\\",
            "Content-Length: 1", "Host: x", "## O que mudou", "**", "0123456789abcdefABCDEF"];
        let mut s = 0x9E37_79B9_7F4A_7C15u64;
        let mut sorteio = move || { s ^= s << 13; s ^= s >> 7; s ^= s << 17; s };
        for _ in 0..3000 {
            let mut texto = String::new();
            for _ in 0..sorteio() % 24 {
                match sorteio() % 4 {
                    0 => texto.push(char::from_u32((sorteio() % 0x3000) as u32).unwrap_or('x')),
                    1 => texto.push_str(&"a".repeat((sorteio() % 300) as usize)),
                    _ => texto.push_str(PECAS[(sorteio() % PECAS.len() as u64) as usize]),
                }
            }
            let tamanhos: Vec<u8> = (0..sorteio() % 5).map(|_| sorteio() as u8).collect();
            propriedades::cabecalho(texto.as_bytes(), &tamanhos);
            propriedades::linha_de_comando(&texto);
            propriedades::release(&texto, "Instalar-GizNaTela-1.4.0.zip");
        }
        // cabeçalho grande, perto e acima do limite, em pedaços pequenos
        let grande = format!("GET / HTTP/1.1\r\nX: {}\r\n\r\n", "a".repeat(super::MAX_CABECALHO));
        propriedades::cabecalho(grande.as_bytes(), &[7, 200, 3]);
    }
}
