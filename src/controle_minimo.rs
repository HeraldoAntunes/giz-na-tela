// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Controle para o VirusTotal (rodada 5)
// Um exe Tauri mínimo com a configuração do Giz na Tela (tauri.conf.json, as mesmas páginas dentro): só abre a barra,
// sem nenhum código próprio do programa. Serve de régua: se este acusar, o alerta é do Tauri com o WebView2, não do
// Giz na Tela. Montagem: cargo build --release --no-default-features --features controle-minimo --bin ControleMinimo
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            tauri::WebviewWindowBuilder::new(app, "barra", tauri::WebviewUrl::App("barra.html".into()))
                .title("Giz na Tela")
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("o Tauri não abriu");
}
