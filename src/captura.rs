// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Retrato da tela (GDI), imagem salva e envio ao Giz Livre
use crate::sistema::{self, Agora};
use crate::Estado;
use base64::Engine;
use serde_json::{json, Value};
use std::time::Duration;
use tauri::{AppHandle, Manager};

// Retrato com o GDI do próprio Windows (sem biblioteca de captura): a tela (monitor) onde a camada do anotador está
#[repr(C)]
struct Retangulo { esq: i32, topo: i32, dir: i32, base: i32 }
#[repr(C)]
struct InfoMonitor { tam: u32, monitor: Retangulo, trabalho: Retangulo, flags: u32 }
#[repr(C)]
struct InfoBmp { tam: u32, largura: i32, altura: i32, planos: u16, bits: u16, compressao: u32, tam_imagem: u32,
    xppm: i32, yppm: i32, usadas: u32, importantes: u32, cores: [u32; 3] }

#[link(name = "user32")]
extern "system" {
    fn GetDC(hwnd: isize) -> isize;
    fn ReleaseDC(hwnd: isize, dc: isize) -> i32;
    fn MonitorFromWindow(hwnd: isize, flags: u32) -> isize;
    fn GetMonitorInfoW(monitor: isize, info: *mut InfoMonitor) -> i32;
}
#[link(name = "gdi32")]
extern "system" {
    fn CreateCompatibleDC(dc: isize) -> isize;
    fn CreateCompatibleBitmap(dc: isize, largura: i32, altura: i32) -> isize;
    fn SelectObject(dc: isize, obj: isize) -> isize;
    fn BitBlt(dest: isize, x: i32, y: i32, largura: i32, altura: i32, orig: isize, xo: i32, yo: i32, op: u32) -> i32;
    fn GetDIBits(dc: isize, bmp: isize, inicio: u32, linhas: u32, bits: *mut u8, info: *mut InfoBmp, uso: u32) -> i32;
    fn DeleteObject(obj: isize) -> i32;
    fn DeleteDC(dc: isize) -> i32;
}
#[link(name = "shcore")]
extern "system" {
    fn GetDpiForMonitor(monitor: isize, tipo: u32, x: *mut u32, y: *mut u32) -> i32;
}

// imagem em RGBA, linha a linha de cima para baixo
pub struct Imagem { pub largura: u32, pub altura: u32, pub rgba: Vec<u8> }

fn retrato_gdi(camada: isize) -> Result<(Imagem, f32), String> {
    unsafe {
        let m = MonitorFromWindow(camada, 1);  // MONITOR_DEFAULTTOPRIMARY
        let mut info = InfoMonitor { tam: std::mem::size_of::<InfoMonitor>() as u32,
            monitor: Retangulo { esq: 0, topo: 0, dir: 0, base: 0 }, trabalho: Retangulo { esq: 0, topo: 0, dir: 0, base: 0 },
            flags: 0 };
        if m == 0 || GetMonitorInfoW(m, &mut info) == 0 {
            return Err("tela do anotador não encontrada".into());
        }
        let r = &info.monitor;
        let (l, a) = (r.dir - r.esq, r.base - r.topo);
        let (mut dpi, mut dpi_y) = (96u32, 96u32);
        if GetDpiForMonitor(m, 0, &mut dpi, &mut dpi_y) != 0 || dpi == 0 {  // MDT_EFFECTIVE_DPI
            dpi = 96;
        }
        let tela = GetDC(0);
        if tela == 0 {
            return Err("não consegui ler a tela".into());
        }
        let mem = CreateCompatibleDC(tela);
        let bmp = CreateCompatibleBitmap(tela, l, a);
        let antigo = SelectObject(mem, bmp);
        let copiou = BitBlt(mem, 0, 0, l, a, tela, r.esq, r.topo, 0x00CC0020) != 0;  // SRCCOPY
        let mut cab = InfoBmp { tam: 40, largura: l, altura: -a, planos: 1, bits: 32, compressao: 0, tam_imagem: 0,
            xppm: 0, yppm: 0, usadas: 0, importantes: 0, cores: [0; 3] };  // altura negativa: de cima para baixo
        let mut px = vec![0u8; l as usize * a as usize * 4];
        SelectObject(mem, antigo);  // o GetDIBits pede o bitmap fora do DC
        let leu = copiou && GetDIBits(mem, bmp, 0, a as u32, px.as_mut_ptr(), &mut cab, 0) == a;
        DeleteObject(bmp);
        DeleteDC(mem);
        ReleaseDC(0, tela);
        if !leu {
            return Err("não consegui copiar a tela".into());
        }
        // o GDI entrega azul, verde, vermelho e um quarto byte sem uso; a tela é sempre opaca
        for p in px.chunks_exact_mut(4) {
            p.swap(0, 2);
            p[3] = 255;
        }
        Ok((Imagem { largura: l as u32, altura: a as u32, rgba: px }, dpi as f32 / 96.0))
    }
}

// Para retratar o que está EMBAIXO do anotador, as janelas pedidas saem da captura só durante o retrato; fora desse
// instante continuam aparecendo em gravações e no compartilhamento de tela.
pub fn retratar(app: &AppHandle, sem: &[&str]) -> Result<(Imagem, f32), String> {
    let janelas: Vec<isize> = sem.iter().filter_map(|n| app.get_webview_window(n))
        .filter_map(|w| w.hwnd().ok().map(|h| h.0 as isize)).collect();
    janelas.iter().for_each(|h| sistema::fora_da_captura(*h, true));
    std::thread::sleep(Duration::from_millis(150)); // o Windows aplica a exclusão no próximo quadro da tela
    let camada = app.get_webview_window("camada").and_then(|w| w.hwnd().ok()).map(|h| h.0 as isize).unwrap_or(0);
    let r = retrato_gdi(camada);
    janelas.iter().for_each(|h| sistema::fora_da_captura(*h, false));
    r
}

pub fn png(img: &Imagem) -> Result<Vec<u8>, String> {
    let mut saida = Vec::new();
    let mut cod = ::png::Encoder::new(&mut saida, img.largura, img.altura);
    cod.set_color(::png::ColorType::Rgba);
    cod.set_depth(::png::BitDepth::Eight);
    cod.set_compression(::png::Compression::Fast);
    let mut escritor = cod.write_header().map_err(|e| e.to_string())?;
    escritor.write_image_data(&img.rgba).map_err(|e| e.to_string())?;
    escritor.finish().map_err(|e| e.to_string())?;
    Ok(saida)
}

// redução pela média de cada bloco de pixels: nítida o bastante para o agente ler a tela
fn reduzir(img: &Imagem, l: u32, a: u32) -> Imagem {
    let (lo, ao) = (img.largura as u64, img.altura as u64);
    let mut rgba = Vec::with_capacity(l as usize * a as usize * 4);
    for y in 0..a as u64 {
        let (y0, y1) = (y * ao / a as u64, ((y + 1) * ao / a as u64).max(y * ao / a as u64 + 1));
        for x in 0..l as u64 {
            let (x0, x1) = (x * lo / l as u64, ((x + 1) * lo / l as u64).max(x * lo / l as u64 + 1));
            let mut soma = [0u64; 4];
            for yy in y0..y1 {
                let linha = &img.rgba[((yy * lo + x0) * 4) as usize..((yy * lo + x1) * 4) as usize];
                for p in linha.chunks_exact(4) {
                    (0..4).for_each(|c| soma[c] += p[c] as u64);
                }
            }
            let n = (y1 - y0) * (x1 - x0);
            rgba.extend(soma.iter().map(|s| (s / n) as u8));
        }
    }
    Imagem { largura: l, altura: a, rgba }
}

// ---- área de transferência: a tela com a tinta, para colar num documento ou numa prova ----
#[link(name = "user32")]
extern "system" {
    fn OpenClipboard(dona: isize) -> i32;
    fn CloseClipboard() -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(formato: u32, dados: isize) -> isize;
    fn RegisterClipboardFormatW(nome: *const u16) -> u32;
}
#[link(name = "kernel32")]
extern "system" {
    fn GlobalAlloc(flags: u32, tam: usize) -> isize;
    fn GlobalLock(h: isize) -> *mut u8;
    fn GlobalUnlock(h: isize) -> i32;
    fn GlobalFree(h: isize) -> isize;
}

// bloco de memória que a área de transferência passa a possuir (se o Windows recusar, devolvemos)
fn bloco(dados: &[u8]) -> Result<isize, String> {
    unsafe {
        let h = GlobalAlloc(0x0002, dados.len());  // GMEM_MOVEABLE
        let p = if h == 0 { std::ptr::null_mut() } else { GlobalLock(h) };
        if p.is_null() {
            if h != 0 { GlobalFree(h); }
            return Err("memória insuficiente para copiar".into());
        }
        std::ptr::copy_nonoverlapping(dados.as_ptr(), p, dados.len());
        GlobalUnlock(h);
        Ok(h)
    }
}

// Dois formatos: o bitmap clássico (CF_DIB, que todo programa cola) e o PNG (que Word, navegador e outros preferem).
// O DIB vai de baixo para cima, o jeito que os programas antigos esperam.
fn para_area_de_transferencia(img: &Imagem, png: &[u8]) -> Result<(), String> {
    let (l, a) = (img.largura as usize, img.altura as usize);
    let mut dib = Vec::with_capacity(40 + l * a * 4);
    for v in [40u32, l as u32, a as u32] { dib.extend_from_slice(&v.to_le_bytes()); }
    dib.extend_from_slice(&1u16.to_le_bytes());
    dib.extend_from_slice(&32u16.to_le_bytes());
    for v in [0u32, (l * a * 4) as u32, 0, 0, 0, 0] { dib.extend_from_slice(&v.to_le_bytes()); }
    for y in (0..a).rev() {
        for p in img.rgba[y * l * 4..(y + 1) * l * 4].chunks_exact(4) {
            dib.extend_from_slice(&[p[2], p[1], p[0], 255]);
        }
    }
    unsafe {
        // outro programa pode estar com a área de transferência aberta por um instante
        let mut aberta = false;
        for _ in 0..20 {
            if OpenClipboard(0) != 0 { aberta = true; break; }
            std::thread::sleep(Duration::from_millis(25));
        }
        if !aberta {
            return Err("a área de transferência está ocupada por outro programa".into());
        }
        EmptyClipboard();
        let mut r = Ok(());
        let nome_png: Vec<u16> = "PNG".encode_utf16().chain([0]).collect();
        for (formato, dados) in [(8u32, &dib[..]), (RegisterClipboardFormatW(nome_png.as_ptr()), png)] {  // 8 = CF_DIB
            match bloco(dados) {
                Ok(h) => if SetClipboardData(formato, h) == 0 { GlobalFree(h); r = Err("o Windows recusou a cópia".to_string()); },
                Err(e) => r = Err(e),
            }
        }
        CloseClipboard();
        r
    }
}

// captura.copiar: a tela com a tinta (só a barra sai), como em Salvar, mas para a área de transferência
pub fn copiar(app: &AppHandle) -> Result<Value, String> {
    let (img, _) = retratar(app, &["barra", "ajuda"])?;
    para_area_de_transferencia(&img, &png(&img)?)?;
    let msg = "Tela copiada: cole com Ctrl+V num documento, numa prova ou no e-mail";
    crate::js(app, "camada", &format!("camada.aviso({})", json!(msg)));
    Ok(json!({ "largura": img.largura, "altura": img.altura, "mensagem": msg }))
}

pub fn data_url(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png))
}

// captura.ler: retrato com a tinta e sem a barra, reduzido para caber no pedido do agente
pub fn ler(app: &AppHandle, largura_max: u32) -> Result<Value, String> {
    let (mut img, _) = retratar(app, &["barra", "ajuda"])?;
    if img.largura > largura_max {
        let alt = (img.altura as u64 * largura_max as u64 / img.largura as u64).max(1) as u32;
        img = reduzir(&img, largura_max, alt);
    }
    let dados = png(&img)?;
    Ok(json!({ "largura": img.largura, "altura": img.altura, "png_base64":
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
        match mandar_ao_giz(app, &dados, img.largura, img.altura, escala, &t) {
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
