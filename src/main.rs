// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Programa principal (Tauri 2 com o WebView2)
// Camada transparente sempre por cima de tudo, onde a tinta fica, e a barra de ferramentas. A barra executa os
// comandos do registro (web/comandos.js); este host faz o que a página não pode: janelas, mouse livre, retrato da
// tela, arquivos, o envio ao Giz Livre, a API local e o MCP (GizNaTela.exe --mcp).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod captura;
mod config;
mod mcp;
mod sistema;

use config::{Config, Opcoes};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub const VERSAO: &str = env!("CARGO_PKG_VERSION");

pub struct Estado {
    pub opcoes: Opcoes,
    pub config: Mutex<Config>,
    pub token: Mutex<Option<String>>,
    pub porta_api: Mutex<u16>,
    recolhida: Mutex<bool>,
    config_mudou: AtomicBool,
    pendentes: Mutex<HashMap<u64, mpsc::Sender<Value>>>,
    proximo: AtomicU64,
}

// a barra, a camada e a ajuda falam com o host por este objeto; window.CONFIG traz o que precisam saber ao abrir
const PONTE: &str = r#"
const inv = (c, a) => window.__TAURI__.core.invoke(c, a);
window.ponte = {
  camada: (nome, valor) => inv('camada', { nome, valor: valor ?? null }),
  fixar: sim => inv('fixar', { sim: !!sim }),
  recolher: sim => inv('recolher', { sim: !!sim }),
  salvar: giz => inv('salvar', { giz: !!giz }),
  capturar: largura => inv('capturar', { largura }),
  congelar: () => inv('congelar'),
  estadoCamada: e => inv('estado_camada', { e }),
  tecla: k => inv('tecla', { k }),
  fim: () => inv('fim'),
  ajuda: () => inv('ajuda'),
  info: () => inv('info'),
  agentes: nivel => inv('agentes', { nivel }),
  canetaVista: () => inv('caneta_vista'),
  dicaInk: () => inv('dica_ink_mostrada'),
};"#;

pub fn js(app: &AppHandle, janela: &str, codigo: &str) {
    if let Some(w) = app.get_webview_window(janela) {
        let _ = w.eval(codigo);
    }
}

// Roda `corpo` (corpo de uma função async) numa janela e espera o valor de volta pelo comando "responder".
// É assim que a API pede um comando à barra e que a barra pede um desenho à camada.
pub fn chamar(app: &AppHandle, janela: &str, corpo: &str, espera: Duration) -> Result<Value, String> {
    let estado = app.state::<Estado>();
    let id = estado.proximo.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = mpsc::channel();
    estado.pendentes.lock().unwrap().insert(id, tx);
    let codigo = format!("(async () => {{ {corpo} }})().then(\
        v => window.__TAURI__.core.invoke('responder', {{ id: {id}, r: {{ ok: true, valor: v ?? null }} }}),\
        e => window.__TAURI__.core.invoke('responder', {{ id: {id}, r: {{ ok: false, erro: String(e && e.message || e) }} }}))");
    let r = match app.get_webview_window(janela) {
        None => Err(format!("janela {janela} fechada")),
        Some(w) => w.eval(&codigo).map_err(|e| e.to_string())
            .and_then(|_| rx.recv_timeout(espera).map_err(|_| "o anotador não respondeu a tempo".to_string())),
    };
    estado.pendentes.lock().unwrap().remove(&id);
    let r = r?;
    if r["ok"].as_bool() == Some(true) { Ok(r["valor"].clone()) } else { Err(r["erro"].as_str().unwrap_or("erro").to_string()) }
}

#[tauri::command]
fn responder(estado: State<Estado>, id: u64, r: Value) {
    if let Some(tx) = estado.pendentes.lock().unwrap().remove(&id) {
        let _ = tx.send(r);
    }
}

// a barra só pode pedir à camada as funções desta lista; o valor vai serializado em JSON, nunca colado como código
const DA_CAMADA: [&str; 15] = ["estado", "cor", "ferramenta", "espessura", "desfazer", "refazer", "limpar", "quadro", "some",
    "preencher", "congelar", "ampliar", "aviso", "adicionar", "escrevendo"];
#[tauri::command]
async fn camada(app: AppHandle, nome: String, valor: Value) -> Result<Value, String> {
    if !DA_CAMADA.contains(&nome.as_str()) {
        return Err(format!("a camada não tem {nome}"));
    }
    tauri::async_runtime::spawn_blocking(move || {
        chamar(&app, "camada", &format!("return await camada.{nome}({valor})"), Duration::from_secs(20))
    }).await.map_err(|e| e.to_string())?
}
// a camada conta à barra o que mudou por conta própria (traço feito à mão, tinta que sumiu, ponto do ampliar)
#[tauri::command]
fn estado_camada(app: AppHandle, e: Value) {
    js(&app, "barra", &format!("estadoCamada({e})"));
}
// atalhos teclados com o foco na camada: vão para a barra, que guarda o estado dos botões
#[tauri::command]
fn tecla(app: AppHandle, k: String) {
    js(&app, "barra", &format!("tecla({})", json!(k)));
}

// Mouse livre: o clique e a caneta passam para os programas de baixo; a tinta continua na tela
#[tauri::command]
fn fixar(app: AppHandle, sim: bool) -> bool {
    if let Some(w) = app.get_webview_window("camada") {
        let _ = w.set_ignore_cursor_events(sim);
        let _ = w.eval(&format!("camada.escrevendo({})", !sim));
    }
    sim
}

// recolher: a barra encolhe para o botão do giz, presa à direita de onde estava; abrir faz o caminho de volta
const BARRA: (f64, f64) = (132.0, 510.0);
const BOTAO: f64 = 56.0;
fn deslocamento_do_botao(k: f64) -> i32 { ((BARRA.0 - BOTAO) * k) as i32 }
#[tauri::command]
fn recolher(app: AppHandle, sim: bool) {
    let estado = app.state::<Estado>();
    {
        let mut r = estado.recolhida.lock().unwrap();
        if *r == sim { return; }
        *r = sim;
    }
    estado.config.lock().unwrap().recolhida = sim;
    estado.config_mudou.store(true, Ordering::Relaxed);
    if let Some(w) = app.get_webview_window("barra") {
        let k = w.scale_factor().unwrap_or(1.0);
        if let Ok(p) = w.outer_position() {
            let dx = deslocamento_do_botao(k);
            let (x, y, tam) = if sim { (p.x + dx, p.y, (BOTAO, BOTAO)) } else { (p.x - dx, p.y, BARRA) };
            // o botão pode ter sido arrastado para perto da borda: a barra aberta não pode sair da tela
            let (x, y) = dentro_do_monitor(&w, x, y, tam, k);
            let _ = w.set_size(tauri::LogicalSize::new(tam.0, tam.1));
            let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
        }
    }
}
fn dentro_do_monitor(w: &tauri::WebviewWindow, x: i32, y: i32, tam: (f64, f64), k: f64) -> (i32, i32) {
    let (lx, ly) = ((tam.0 * k) as i32, (tam.1 * k) as i32);
    match w.current_monitor() {
        Ok(Some(m)) => (x.clamp(m.position().x, m.position().x + m.size().width as i32 - lx),
                        y.clamp(m.position().y, m.position().y + m.size().height as i32 - ly)),
        _ => (x, y),
    }
}

#[tauri::command]
async fn congelar(app: AppHandle) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (img, _) = captura::retratar(&app, &["camada", "barra", "ajuda"])?;
        Ok(captura::data_url(&captura::png(&img)?))
    }).await.map_err(|e| e.to_string())?
}
#[tauri::command]
async fn salvar(app: AppHandle, giz: bool) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || captura::salvar(&app, giz)).await.map_err(|e| e.to_string())?
}
#[tauri::command]
async fn capturar(app: AppHandle, largura: u32) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || captura::ler(&app, largura)).await.map_err(|e| e.to_string())?
}

fn gravar_config(app: &AppHandle) {
    let estado = app.state::<Estado>();
    let c = estado.config.lock().unwrap().clone();
    let _ = c.gravar(&estado.opcoes.dados);
}
// responde antes de sair, para o pedido da API (ou do teste) receber a confirmação
#[tauri::command]
fn fim(app: AppHandle) {
    gravar_config(&app);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        app.exit(0);
    });
}

#[tauri::command]
async fn ajuda(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("ajuda") {
        let _ = w.unminimize();
        return w.set_focus().map_err(|e| e.to_string());
    }
    let camada = app.get_webview_window("camada").ok_or("camada fechada")?;
    WebviewWindowBuilder::new(&app, "ajuda", WebviewUrl::App("ajuda.html".into()))
        .parent(&camada).map_err(|e| e.to_string())?
        .title("Giz na Tela: ajuda e ajustes")
        .inner_size(680.0, 720.0)
        .min_inner_size(480.0, 400.0)
        .center()
        .always_on_top(true)
        .initialization_script(&iniciar_js(&app))
        .build().map_err(|e| e.to_string())?;
    Ok(())
}

// o que a ajuda mostra: versão, pastas, porta e o nível dos agentes
#[tauri::command]
fn info(app: AppHandle) -> Value {
    let estado = app.state::<Estado>();
    let cfg = estado.config.lock().unwrap().clone();
    json!({ "versao": VERSAO, "agentes": cfg.agentes, "porta": *estado.porta_api.lock().unwrap(),
            "dados": estado.opcoes.dados.display().to_string(),
            "capturas": captura::pasta_capturas(&app).map(|p| p.display().to_string()).unwrap_or_default(),
            "exe": std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default() })
}

// liga, desliga ou muda o nível dos agentes; ligar de novo depois de desligar troca o token
#[tauri::command]
fn agentes(app: AppHandle, nivel: String) -> Result<Value, String> {
    if !config::NIVEIS.contains(&nivel.as_str()) {
        return Err("nível inválido".into());
    }
    let estado = app.state::<Estado>();
    estado.config.lock().unwrap().agentes = nivel.clone();
    gravar_config(&app);
    preparar_token(&estado, &nivel).map_err(|e| e.to_string())?;
    Ok(info(app.clone()))
}
fn preparar_token(estado: &Estado, nivel: &str) -> std::io::Result<()> {
    let pasta = estado.opcoes.dados.join("agente");
    let arq = pasta.join("token");
    let mut token = estado.token.lock().unwrap();
    if nivel == "desligado" {
        *token = None;
        if arq.exists() { std::fs::remove_file(&arq)?; }
        return Ok(());
    }
    if token.is_none() {
        let lido = std::fs::read_to_string(&arq).ok().map(|t| t.trim().to_string()).filter(|t| t.len() == 64);
        let t = match lido { Some(t) => t, None => sistema::token_novo() };
        std::fs::create_dir_all(&pasta)?;
        std::fs::write(&arq, &t)?;
        *token = Some(t);
    }
    Ok(())
}

#[tauri::command]
fn caneta_vista(app: AppHandle) {
    let estado = app.state::<Estado>();
    estado.config.lock().unwrap().caneta_vista = true;
    estado.config_mudou.store(true, Ordering::Relaxed);
}
#[tauri::command]
fn dica_ink_mostrada(app: AppHandle) {
    let estado = app.state::<Estado>();
    estado.config.lock().unwrap().dicas_ink += 1;
    estado.config_mudou.store(true, Ordering::Relaxed);
}

fn iniciar_js(app: &AppHandle) -> String {
    let estado = app.state::<Estado>();
    let cfg = estado.config.lock().unwrap().clone();
    // a dica do Windows Ink aparece no máximo 3 vezes e nunca depois que uma caneta de verdade chegou
    let config = json!({ "versao": VERSAO, "recolhida": cfg.recolhida, "caneta_vista": cfg.caneta_vista,
                         "dica_ink": !cfg.caneta_vista && cfg.dicas_ink < 3 });
    format!("window.CONFIG = {config};{PONTE}")
}

fn main() {
    let opcoes = Opcoes::da_linha_de_comando();
    if opcoes.mcp {
        std::process::exit(mcp::rodar(&opcoes));
    }
    if sistema::ja_aberto(&opcoes.dados.display().to_string().to_lowercase()) {
        return;
    }
    if !sistema::tem_webview2() {
        sistema::avisar_sem_webview2();
        return;
    }
    let config = Config::ler(&opcoes.dados);
    let estado = Estado { config: Mutex::new(config), token: Mutex::new(None), porta_api: Mutex::new(0),
        recolhida: Mutex::new(false), config_mudou: AtomicBool::new(false), pendentes: Mutex::new(HashMap::new()),
        proximo: AtomicU64::new(1), opcoes };
    let nivel = estado.config.lock().unwrap().agentes.clone();
    let _ = preparar_token(&estado, &nivel);
    let resultado = tauri::Builder::default()
        .manage(estado)
        .invoke_handler(tauri::generate_handler![responder, camada, estado_camada, tecla, fixar, recolher, congelar,
            salvar, capturar, fim, ajuda, info, agentes, caneta_vista, dica_ink_mostrada])
        // Ctrl+Alt+F liga e desliga o Mouse livre de qualquer programa (com ele ligado o teclado não chega aqui)
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app, _atalho, ev| {
            if ev.state() == ShortcutState::Pressed {
                js(app, "barra", "tecla('f')");
            }
        }).build())
        // lembra onde a barra ficou (aberta ou recolhida) para abrir no mesmo lugar da próxima vez
        .on_window_event(|w, ev| {
            if let (WindowEvent::Moved(p), "barra") = (ev, w.label()) {
                let estado = w.state::<Estado>();
                let recolhida = *estado.recolhida.lock().unwrap();
                let dx = if recolhida { deslocamento_do_botao(w.scale_factor().unwrap_or(1.0)) } else { 0 };
                estado.config.lock().unwrap().barra = Some((p.x - dx, p.y));
                estado.config_mudou.store(true, Ordering::Relaxed);
            }
        })
        .setup(|app| {
            let _ = app.global_shortcut().register(Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyF));
            let h = app.handle().clone();
            let tela = app.primary_monitor()?.ok_or("sem monitor")?;
            let escala = tela.scale_factor();
            let (larg, alt) = (tela.size().width as f64 / escala, tela.size().height as f64 / escala);
            let inicio = iniciar_js(&h);
            let camada = WebviewWindowBuilder::new(app, "camada", WebviewUrl::App("camada.html".into()))
                .title("Giz na Tela - camada")
                .transparent(true)
                .decorations(false)
                .shadow(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .resizable(false)
                .position(0.0, 0.0)
                .inner_size(larg, alt)
                .initialization_script(&inicio)
                .build()?;
            let (recolhida, salva) = {
                let c = app.state::<Estado>().config.lock().unwrap().clone();
                (c.recolhida, c.barra)
            };
            *app.state::<Estado>().recolhida.lock().unwrap() = recolhida;
            let tam = if recolhida { (BOTAO, BOTAO) } else { BARRA };
            // a barra pertence à camada: o Windows mantém janela "filha" sempre na frente da dona
            let barra = WebviewWindowBuilder::new(app, "barra", WebviewUrl::App("barra.html".into()))
                .parent(&camada)?
                .title("Giz na Tela")
                .transparent(true)
                .decorations(false)
                .shadow(false)
                .skip_taskbar(true)
                .always_on_top(true)
                .resizable(false)
                .visible(false)
                .position(larg - 148.0, ((alt - BARRA.1) / 2.0).max(10.0))   // no meio da altura, à direita
                .inner_size(tam.0, tam.1)
                .initialization_script(&inicio)
                .build()?;
            // posição salva: só vale se ainda cair numa tela que existe (o projetor pode ter sido desligado)
            if let Some((x, y)) = salva {
                let dx = if recolhida { deslocamento_do_botao(escala) } else { 0 };
                let existe = app.available_monitors()?.iter().any(|m| {
                    let (mx, my) = (m.position().x, m.position().y);
                    x + dx >= mx && y >= my && x + dx < mx + m.size().width as i32 - 20 && y < my + m.size().height as i32 - 20
                });
                if existe {
                    let (px, py) = dentro_do_monitor(&barra, x + dx, y, tam, escala);
                    barra.set_position(tauri::PhysicalPosition::new(px, py))?;
                }
            }
            barra.show()?;
            // API local: sempre escutando, mas sem token (Agentes de IA desligados) recusa tudo
            let estado = app.state::<Estado>();
            if let Some((ouvinte, porta)) = api::abrir(estado.opcoes.porta_api) {
                *estado.porta_api.lock().unwrap() = porta;
                let pasta = estado.opcoes.dados.join("agente");
                let _ = std::fs::create_dir_all(&pasta).and_then(|_| std::fs::write(pasta.join("porta"), porta.to_string()));
                let h2 = h.clone();
                std::thread::spawn(move || api::servir(h2, ouvinte, porta));
            }
            // ajustes gravados no máximo uma vez por segundo (arrastar a barra gera muitos eventos)
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));
                if h.state::<Estado>().config_mudou.swap(false, Ordering::Relaxed) {
                    gravar_config(&h);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!());
    // sem console (programa de janela), um erro ao abrir só seria visto assim
    if let Err(e) = resultado {
        sistema::avisar_erro(&format!("O Giz na Tela não conseguiu abrir.\n\n{e}"));
    }
}
