// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Fuzzing do cabeçalho HTTP da API local
// O primeiro byte escolhe quantos bytes vão para os tamanhos dos pedaços da leitura (uma conexão lenta); o resto é o
// que chegaria pela rede.
#![no_main]
use libfuzzer_sys::fuzz_target;

#[path = "../../src/leitura.rs"]
#[allow(dead_code)]
mod leitura;

fuzz_target!(|dados: &[u8]| {
    let (n, resto) = match dados.split_first() { Some((n, r)) => ((*n as usize % 8).min(r.len()), r), None => return };
    let (tamanhos, pedido) = resto.split_at(n);
    leitura::propriedades::cabecalho(pedido, tamanhos);
});
