// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Instalador num exe próprio (variante de teste)
// Só para comparar no VirusTotal (gestão, 29/10/2026): um exe pequeno, sem o Tauri, que copia o GizNaTela.exe que vem
// ao lado dele, cria os atalhos e registra a desinstalação. É o mesmo código de src/instalador.rs; fica na pasta
// instalada como Desinstalar-GizNaTela.exe. Montagem:
//   cargo build --release --no-default-features --features instalador-separado --bin Instalar-GizNaTela
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub const VERSAO: &str = env!("CARGO_PKG_VERSION");

#[path = "leitura.rs"]
#[allow(dead_code)]
mod leitura;
#[path = "sistema.rs"]
#[allow(dead_code)]
mod sistema;
#[path = "instalador.rs"]
#[allow(dead_code)]
mod instalador;

fn main() {
    std::process::exit(instalador::pedido().unwrap_or(0));
}
