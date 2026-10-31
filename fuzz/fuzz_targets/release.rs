// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Fuzzing do que vem da release no GitHub
// SHA256SUMS.txt, notas da versão e número da versão, lidos pelo "Procurar versão nova" do Sobre.
#![no_main]
use libfuzzer_sys::fuzz_target;

#[path = "../../src/leitura.rs"]
#[allow(dead_code)]
mod leitura;

fuzz_target!(|texto: &str| {
    leitura::propriedades::release(texto, "Instalar-GizNaTela-1.4.0.zip");
});
