// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Fuzzing da linha de comando de outro processo
// A linha vem de um processo qualquer da conta; dela sai a pasta do perfil do WebView2 que decide o que se fecha.
#![no_main]
use libfuzzer_sys::fuzz_target;

#[path = "../../src/leitura.rs"]
#[allow(dead_code)]
mod leitura;

fuzz_target!(|linha: &str| {
    leitura::propriedades::linha_de_comando(linha);
});
