// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Retrato da tela, imagem salva e envio ao Giz Livre
use crate::sistema::{self, Agora};
use crate::Estado;
use base64::Engine;
use serde_json::{json, Value};
use std::time::Duration;
use tauri::{AppHandle, Manager};

// Para retratar o que está EMBAIXO do anotador, as janelas pedidas saem da captura só durante o retrato; fora desse
// instante continuam aparecendo em gravações e no compartilhamento de tela.
pub fn retratar(app: &AppHandle, sem: &[&str]) -> Result<(xcap::image::RgbaImage, f32), String> {
    let janelas: Vec<isize> = sem.iter().filter_map(|n| app.get_webview_window(n))
        .filter_map(|w| w.hwnd().ok().map(|h| h.0 as isize)).collect();
    janelas.iter().for_each(|h| sistema::fora_da_captura(*h, true));
    std::thread::sleep(Duration::from_millis(150)); // o Windows aplica a exclusão no próximo quadro da tela
    let r = (|| -> Result<_, String> {
        let m = xcap::Monitor::all().map_err(|e| e.to_string())?.into_iter()
            .find(|m| m.is_primary().unwrap_or(false)).ok_or("monitor principal não encontrado")?;
        let escala = m.scale_factor().unwrap_or(1.0);
        Ok((m.capture_image().map_err(|e| e.to_string())?, escala))
    })();
    janelas.iter().for_each(|h| sistema::fora_da_captura(*h, false));
    r
}

pub fn png(img: &xcap::image::RgbaImage) -> Result<Vec<u8>, String> {
    use xcap::image::codecs::png::{CompressionType, FilterType, PngEncoder};
    let mut saida = Vec::new();
    img.write_with_encoder(PngEncoder::new_with_quality(&mut saida, CompressionType::Fast, FilterType::Adaptive))
        .map_err(|e| e.to_string())?;
    Ok(saida)
}

pub fn data_url(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png))
}

// captura.ler: retrato com a tinta e sem a barra, reduzido para caber no pedido do agente
pub fn ler(app: &AppHandle, largura_max: u32) -> Result<Value, String> {
    let (mut img, _) = retratar(app, &["barra", "ajuda"])?;
    if img.width() > largura_max {
        let alt = (img.height() as u64 * largura_max as u64 / img.width() as u64).max(1) as u32;
        img = xcap::image::imageops::resize(&img, largura_max, alt, xcap::image::imageops::FilterType::Triangle);
    }
    let dados = png(&img)?;
    Ok(json!({ "largura": img.width(), "altura": img.height(), "png_base64":
        base64::engine::general_purpose::STANDARD.encode(dados) }))
}

fn nome_arquivo(t: &Agora) -> String {
    format!("anotacao-{}{:02}{:02}-{:02}{:02}{:02}", t.ano, t.mes, t.dia, t.hora, t.minuto, t.segundo)
}

pub fn pasta_capturas(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let estado = app.state::<Estado>();
    match &estado.opcoes.capturas {
        Some(p) => Ok(p.clone()),
        None => Ok(app.path().document_dir().map_err(|e| e.to_string())?.join("Giz na Tela").join("capturas")),
    }
}

// salvar: retrato com a tinta (só a barra sai) na pasta de capturas; com giz, vira também um quadro do Giz Livre
pub fn salvar(app: &AppHandle, giz: bool) -> Result<Value, String> {
    let (img, escala) = retratar(app, &["barra", "ajuda"])?;
    let dados = png(&img)?;
    let t = sistema::agora();
    let pasta = pasta_capturas(app)?;
    std::fs::create_dir_all(&pasta).map_err(|e| e.to_string())?;
    let mut arq = pasta.join(format!("{}.png", nome_arquivo(&t)));
    // dois salvamentos no mesmo segundo não se sobrescrevem
    let mut n = 2;
    while arq.exists() {
        arq = pasta.join(format!("{}-{n}.png", nome_arquivo(&t)));
        n += 1;
    }
    std::fs::write(&arq, &dados).map_err(|e| e.to_string())?;
    let arquivo = arq.display().to_string();
    let mut r = json!({ "arquivo": arquivo, "giz": false, "mensagem": format!("Imagem salva em {arquivo}") });
    if giz {
        match mandar_ao_giz(app, &dados, img.width(), img.height(), escala, &t) {
            Ok(titulo) => {
                r["giz"] = json!(true);
                r["quadro"] = json!(titulo);
                r["mensagem"] = json!(format!("Mandado ao Giz Livre: quadro \"{titulo}\". A tinta saiu da tela e a barra \
                    recolheu para você usar o Giz (Ctrl+Z, com a barra aberta, traz a tinta de volta)"));
            }
            Err(e) => r["mensagem"] = json!(format!("Não foi para o Giz ({e}). Imagem salva em {arquivo}")),
        }
    }
    let msg = r["mensagem"].as_str().unwrap_or_default().to_string();
    crate::js(app, "camada", &format!("camada.aviso({})", json!(msg)));
    Ok(r)
}

// ---- Giz Livre aberto neste PC: mesma API da página dele; o token vem de /api/info ----
fn giz(porta: u16, metodo: &str, rota: &str, token: &str, tipo: &str, corpo: &[u8]) -> Result<(u16, String), String> {
    sistema::http(porta, metodo, rota, &[("X-Lousa", token)], tipo, corpo, Duration::from_secs(20))
        .map(|(s, c)| (s, String::from_utf8_lossy(&c).to_string()))
        .map_err(|e| if e == "fechado" { "Giz Livre fechado".to_string() } else { e })
}

// Giz fechado: abre o Giz instalado (só para mim ou para todos) e espera o servidor dele responder.
// Com --porta-giz (teste com um Giz de desenvolvimento) nunca abre o Giz do professor.
fn abrir_giz(app: &AppHandle, porta: u16) -> Result<String, String> {
    if app.state::<Estado>().opcoes.porta_giz.is_some() {
        return Err("Giz Livre fechado".into());
    }
    let exe = [std::env::var("LOCALAPPDATA").map(|d| format!(r"{d}\Programs\Giz Livre\GizLivre.exe")),
               std::env::var("ProgramFiles").map(|d| format!(r"{d}\Giz Livre\GizLivre.exe"))]
        .into_iter().flatten().find(|c| std::path::Path::new(c).exists())
        .ok_or("Giz Livre fechado e não achei o Giz instalado")?;
    crate::js(app, "camada", "camada.aviso('Abrindo o Giz Livre...')");
    std::process::Command::new(&exe).spawn().map_err(|_| "não consegui abrir o Giz Livre".to_string())?;
    for _ in 0..60 {
        std::thread::sleep(Duration::from_millis(500));
        if let Ok((200, info)) = giz(porta, "GET", "/api/info", "", "text/plain", b"") {
            return Ok(info);
        }
    }
    Err("o Giz Livre demorou a abrir".into())
}

fn mandar_ao_giz(app: &AppHandle, png: &[u8], l: u32, a: u32, escala: f32, t: &Agora) -> Result<String, String> {
    let porta = app.state::<Estado>().opcoes.porta_giz.unwrap_or(crate::config::PORTA_GIZ);
    let info = match giz(porta, "GET", "/api/info", "", "text/plain", b"") {
        Ok((_, info)) => info,
        Err(_) => abrir_giz(app, porta)?,
    };
    let token = serde_json::from_str::<Value>(&info).ok()
        .and_then(|v| v["token"].as_str().map(String::from)).ok_or("o Giz Livre não respondeu")?;
    let (s, r) = giz(porta, "POST", "/api/assets", &token, "image/png", png)?;
    let url = serde_json::from_str::<Value>(&r).ok()
        .and_then(|v| v["url"].as_str().map(String::from)).ok_or(format!("o Giz recusou a imagem ({s})"))?;
    let titulo = format!("Anotação {:02}/{:02} {:02}h{:02}", t.dia, t.mes, t.hora, t.minuto);
    let id = nome_arquivo(t);
    let (w, h) = (l as f64 / escala as f64, a as f64 / escala as f64);
    let quadro = json!({ "revision": 0, "board": {
        "version": 2, "title": titulo, "background": { "color": "#ffffff", "pattern": "none" },
        "view": { "x": 0, "y": 0, "zoom": 1 },
        "items": [{ "id": "anotacao", "type": "image", "src": url, "x": 0, "y": 0, "w": w, "h": h }] } });
    let (s, _) = giz(porta, "PUT", &format!("/api/boards/{id}"), &token, "application/json", quadro.to_string().as_bytes())?;
    if s != 200 {
        return Err(format!("o Giz recusou o quadro ({s})"));
    }
    Ok(titulo)
}
