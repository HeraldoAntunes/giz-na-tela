// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Programa principal (Tauri 2 com o WebView2)
// Camada transparente sempre por cima de tudo, onde a tinta fica, e a barra de ferramentas. A barra executa os
// comandos do registro (web/comandos.js); este host faz o que a página não pode: janelas, mouse livre, retrato da
// tela, arquivos, o envio ao Giz Livre, a API local e o MCP (GizNaTela.exe --mcp).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(feature = "api")]
mod api;
mod atualizacao;
mod captura;
mod config;
#[cfg(feature = "instalar")]
mod instalador;
#[cfg_attr(not(feature = "api"), allow(dead_code))]   // o cabeçalho HTTP só serve à API
mod leitura;
#[cfg(feature = "api")]
mod mcp;
mod sistema;

use config::{Config, Opcoes};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent};
#[cfg(feature = "atalho-global")]
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
    // muda a cada troca do nível dos agentes: o pedido que começou antes da troca não é mais feito (src/api.rs)
    pub epoca_agentes: AtomicU64,
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
  preferencias: p => inv('preferencias', { p }),
  escolherCor: atual => inv('escolher_cor', { atual }),
  procurarVersao: () => inv('procurar_versao'),
  instalarVersao: () => inv('instalar_versao'),
  comandoNaBarra: (nome, args) => inv('comando_na_barra', { nome, args: args ?? {} }),
  copiar: () => inv('copiar'),
  telas: () => inv('telas'),
  mudarTela: numero => inv('mudar_tela', { numero }),
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
const DA_CAMADA: [&str; 22] = ["estado", "seta", "cota", "escala", "mover", "cor", "ferramenta", "espessura", "desfazer", "refazer", "limpar", "quadro", "some",
    "preencher", "congelar", "ampliar", "aviso", "adicionar", "escrevendo", "tela", "botao", "selecao"];
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
const BARRA: (f64, f64) = (132.0, 640.0);   // fileiras das canetas prontas (com o rótulo) e de tela/copiar
const BOTAO: f64 = 56.0;
fn deslocamento_do_botao(k: f64) -> i32 { ((BARRA.0 - BOTAO) * k) as i32 }
// a barra aberta cabe na tela: numa tela baixa (notebook de 768 px com zoom) a janela encolhe e o conteúdo rola
fn barra_na_tela(m: Option<tauri::Monitor>) -> (f64, f64) {
    let alt = m.map(|m| m.size().height as f64 / m.scale_factor()).unwrap_or(BARRA.1 + 20.0);
    (BARRA.0, BARRA.1.min(alt - 20.0).max(200.0))
}
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
            let aberta = barra_na_tela(w.current_monitor().ok().flatten());
            let (x, y, tam) = if sim { (p.x + dx, p.y, (BOTAO, BOTAO)) } else { (p.x - dx, p.y, aberta) };
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
        // barra mais alta que a tela (notebook de 768 px com zoom de 125%): encosta no alto em vez de parar o programa
        // (clamp com o limite de baixo acima do de cima é erro fatal no Rust)
        Ok(Some(m)) => {
            let (p, t) = (m.position(), m.size());
            (x.min(p.x + t.width as i32 - lx).max(p.x), y.min(p.y + t.height as i32 - ly).max(p.y))
        }
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
async fn copiar(app: AppHandle) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || captura::copiar(&app)).await.map_err(|e| e.to_string())?
}

// ---- telas (monitores): a camada fica numa tela por vez; trocar leva a camada e a barra para a outra (o projetor) ----
// a 1 é a principal; as outras seguem da esquerda para a direita, como o Windows as mostra
fn telas_em_ordem(app: &AppHandle) -> Vec<tauri::Monitor> {
    let principal = app.primary_monitor().ok().flatten().map(|m| *m.position());
    let mut v = app.available_monitors().unwrap_or_default();
    v.sort_by_key(|m| (Some(*m.position()) != principal, m.position().x, m.position().y));
    v
}
// a tela de agora é a que contém o meio da camada
fn tela_atual(app: &AppHandle, lista: &[tauri::Monitor]) -> usize {
    let meio = app.get_webview_window("camada").and_then(|w| Some((w.outer_position().ok()?, w.outer_size().ok()?)))
        .map(|(p, t)| (p.x + t.width as i32 / 2, p.y + t.height as i32 / 2));
    meio.and_then(|(x, y)| lista.iter().position(|m| {
        let (p, t) = (m.position(), m.size());
        x >= p.x && y >= p.y && x < p.x + t.width as i32 && y < p.y + t.height as i32
    })).map(|i| i + 1).unwrap_or(1)
}
fn descrever_telas(app: &AppHandle) -> Value {
    let lista = telas_em_ordem(app);
    let atual = tela_atual(app, &lista);
    json!({ "atual": atual, "telas": lista.iter().enumerate().map(|(i, m)| json!({ "numero": i + 1,
        "nome": m.name().cloned().unwrap_or_else(|| format!("tela {}", i + 1)), "largura": m.size().width,
        "altura": m.size().height, "principal": i == 0 })).collect::<Vec<_>>() })
}
#[tauri::command]
fn telas(app: AppHandle) -> Value { descrever_telas(&app) }

// leva a camada para cobrir a tela inteira e a barra para o meio da direita dela
fn por_na_tela(app: &AppHandle, m: &tauri::Monitor) -> Result<(), String> {
    let (p, t, k) = (*m.position(), *m.size(), m.scale_factor());
    let camada = app.get_webview_window("camada").ok_or("camada fechada")?;
    // a posição primeiro: ao cruzar para uma tela de outra escala o Windows reajusta o tamanho, que vem depois
    camada.set_position(p).map_err(|e| e.to_string())?;
    camada.set_size(t).map_err(|e| e.to_string())?;
    if let Some(barra) = app.get_webview_window("barra") {
        let recolhida = *app.state::<Estado>().recolhida.lock().unwrap();
        let dx = if recolhida { deslocamento_do_botao(k) } else { 0 };
        let tam = barra_na_tela(Some(m.clone()));
        let x = p.x + t.width as i32 - (148.0 * k) as i32 + dx;
        let y = p.y + ((t.height as f64 - tam.1 * k) / 2.0).max(10.0 * k) as i32;
        barra.set_position(tauri::PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
        if !recolhida { barra.set_size(tauri::LogicalSize::new(tam.0, tam.1)).map_err(|e| e.to_string())?; }
    }
    Ok(())
}
#[tauri::command]
fn mudar_tela(app: AppHandle, numero: usize) -> Result<Value, String> {
    let lista = telas_em_ordem(&app);
    let m = lista.get(numero.wrapping_sub(1)).ok_or(format!("não há a tela {numero} (há {})", lista.len()))?;
    por_na_tela(&app, m)?;
    let estado = app.state::<Estado>();
    estado.config.lock().unwrap().tela = m.name().cloned();
    estado.config_mudou.store(true, Ordering::Relaxed);
    let mut r = descrever_telas(&app);
    r["atual"] = json!(numero);   // a janela pode ainda estar terminando de mudar
    Ok(r)
}
#[tauri::command]
async fn capturar(app: AppHandle, largura: u32) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || captura::ler(&app, largura)).await.map_err(|e| e.to_string())?
}

// versão nova: só pela ajuda (Sobre), nunca por agente; nada é instalado sem o professor confirmar lá
#[tauri::command]
async fn procurar_versao() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(atualizacao::procurar).await.map_err(|e| e.to_string())?
}
#[tauri::command]
async fn instalar_versao(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || atualizacao::instalar(&app)).await.map_err(|e| e.to_string())?
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
        .additional_browser_args(SEM_REDE)
        .build().map_err(|e| e.to_string())?;
    Ok(())
}

// o que a ajuda mostra: versão, pastas, porta e o nível dos agentes
#[tauri::command]
fn info(app: AppHandle) -> Value {
    let estado = app.state::<Estado>();
    let cfg = estado.config.lock().unwrap().clone();
    json!({ "versao": VERSAO, "agentes": cfg.agentes, "porta": *estado.porta_api.lock().unwrap(),
            "preferencias": cfg.preferencias,
            "dados": estado.opcoes.dados.display().to_string(),
            "capturas": captura::pasta_capturas(&app).map(|p| p.display().to_string()).unwrap_or_default(),
            "exe": std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default(),
            "instalado": atualizacao::modo_instalado().is_some() })
}

// liga, desliga ou muda o nível dos agentes; ligar de novo depois de desligar troca o token
#[tauri::command]
fn agentes(app: AppHandle, nivel: String) -> Result<Value, String> {
    if !config::NIVEIS.contains(&nivel.as_str()) {
        return Err("nível inválido".into());
    }
    let estado = app.state::<Estado>();
    estado.config.lock().unwrap().agentes = nivel.clone();
    // nova época: os pedidos dos agentes que ainda não foram feitos (na API ou na fila da barra) são cancelados
    let epoca = estado.epoca_agentes.fetch_add(1, Ordering::SeqCst) + 1;
    js(&app, "barra", &format!("agentesMudaram({epoca})"));
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
        let t = match lido { Some(t) => t, None => sistema::token_novo()? };
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

// a barra manda as escolhas dela a cada mudança; vão para o config.json com os outros ajustes
#[tauri::command]
fn preferencias(app: AppHandle, p: Value) {
    if p.to_string().len() > 4096 { return; }
    let estado = app.state::<Estado>();
    estado.config.lock().unwrap().preferencias = p;
    estado.config_mudou.store(true, Ordering::Relaxed);
}
// seletor de cores do Windows, por cima da barra (segundo toque numa bolinha de cor)
#[tauri::command]
async fn escolher_cor(app: AppHandle, atual: String) -> Option<String> {
    let dona = app.get_webview_window("barra").and_then(|w| w.hwnd().ok()).map(|h| h.0 as isize).unwrap_or(0);
    tauri::async_runtime::spawn_blocking(move || sistema::escolher_cor(dona, &atual)).await.ok().flatten()
}
// a ajuda só pode pedir à barra os comandos desta lista; a barra valida os parâmetros como os de qualquer botão
#[tauri::command]
fn comando_na_barra(app: AppHandle, nome: String, args: Value) {
    if ["cores.restaurar", "caneta.botao"].contains(&nome.as_str()) && args.is_object() && args.to_string().len() < 1024 {
        js(&app, "barra", &format!("executar({}, {}, 'botao').catch(() => {{}})", json!(nome), args));
    }
}

fn iniciar_js(app: &AppHandle) -> String {
    let estado = app.state::<Estado>();
    let cfg = estado.config.lock().unwrap().clone();
    // a dica do Windows Ink aparece no máximo 3 vezes e nunca depois que uma caneta de verdade chegou
    let config = json!({ "versao": VERSAO, "recolhida": cfg.recolhida, "caneta_vista": cfg.caneta_vista,
                         "dica_ink": !cfg.caneta_vista && cfg.dicas_ink < 3, "preferencias": cfg.preferencias });
    format!("window.CONFIG = {config};{PONTE}")
}

// O WebView2 abre conexões com a Microsoft por conta própria (SmartScreen, atualização de componentes, telemetria). As
// páginas daqui não usam a rede (tudo passa pelo programa, e o "Procurar versão nova" é feito em Rust), então os
// serviços de rede dele ficam desligados e o que sobrar vai para um proxy que não existe. As três janelas dividem o
// mesmo WebView2: todas precisam dos mesmos argumentos.
const SEM_REDE: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,AutofillServerCommunication,OptimizationHints,MediaRouter,msEdgeTranslate --disable-background-networking --disable-component-update --disable-domain-reliability --disable-sync --no-pings --disable-client-side-phishing-detection --proxy-server=http://127.0.0.1:9 --proxy-bypass-list=<-loopback>";

// registro das tentativas que falharam ao criar as janelas (só o detalhe técnico; a pessoa vê a mensagem em português)
fn anotar_janelas(dados: &std::path::Path, falhas: &[String]) {
    let t = sistema::agora();
    let linha = format!("{}-{:02}-{:02} {:02}:{:02}:{:02} versão {VERSAO}: tentativa {} de criar as janelas falhou: {}\n",
        t.ano, t.mes, t.dia, t.hora, t.minuto, t.segundo, falhas.len(), falhas.last().map(String::as_str).unwrap_or(""));
    let _ = std::fs::create_dir_all(dados);
    let _ = std::fs::OpenOptions::new().create(true).append(true).open(dados.join("janelas.log"))
        .and_then(|mut f| std::io::Write::write_all(&mut f, linha.as_bytes()));
}

// A camada e a barra. A barra nasce invisível e só aparece pronta (o setup a mostra no fim). A camada nasce visível
// (é transparente): criada escondida, o WebView2 recusa a janela com 0x80070578, em toda tentativa.
fn criar_janelas(app: &tauri::App, inicio: &str, (larg, alt): (f64, f64), aberta: (f64, f64), tam: (f64, f64))
    -> Result<(tauri::WebviewWindow, tauri::WebviewWindow), String> {
    // só na compilação de teste: GIZ_TESTE_FALHAR_JANELA=n faz as n primeiras tentativas falharem como o WebView2
    // falha às vezes (0x80070578), para o teste tests/testar_janelas.py
    #[cfg(debug_assertions)]
    {
        static FEITAS: AtomicU64 = AtomicU64::new(0);
        let falhar: u64 = std::env::var("GIZ_TESTE_FALHAR_JANELA").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        if FEITAS.fetch_add(1, Ordering::Relaxed) < falhar {
            return Err("failed to create webview: WebView2 error 0x80070578 (falha forçada pelo teste)".into());
        }
    }
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
        .initialization_script(inicio)
        .additional_browser_args(SEM_REDE)
        .build().map_err(|e| format!("camada: {e}"))?;
    // a barra pertence à camada: o Windows mantém janela "filha" sempre na frente da dona
    let barra = WebviewWindowBuilder::new(app, "barra", WebviewUrl::App("barra.html".into()))
        .parent(&camada).map_err(|e| e.to_string())?
        .title("Giz na Tela")
        .transparent(true)
        .decorations(false)
        .shadow(false)
        .always_on_top(true)
        .resizable(false)
        .minimizable(false)   // tocar no botão da barra de tarefas não some com a barra
        .visible(false)
        .position(larg - 148.0, ((alt - aberta.1) / 2.0).max(10.0))   // no meio da altura, à direita
        .inner_size(tam.0, tam.1)
        .initialization_script(inicio)
        .additional_browser_args(SEM_REDE)
        .build().map_err(|e| format!("barra: {e}"))?;
    Ok((camada, barra))
}

fn main() {
    sistema::marcar_programa();
    // Instalar-GizNaTela-<versão>.exe, --instalar e --desinstalar: o próprio programa se instala (src/instalador.rs)
    #[cfg(feature = "instalar")]
    {
        if let Some(codigo) = instalador::pedido() {
            std::process::exit(codigo);
        }
        if atualizacao::modo_instalado().is_some() {
            instalador::conferir_instalacao();
        }
    }
    let opcoes = Opcoes::da_linha_de_comando();
    // uma parada inesperada fica anotada em <dados>\erros.txt: o programa não tem console para mostrá-la
    let pasta = opcoes.dados.clone();
    let padrao = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let t = sistema::agora();
        let linha = format!("{}-{:02}-{:02} {:02}:{:02}:{:02} versão {VERSAO}: {info}\n", t.ano, t.mes, t.dia, t.hora,
            t.minuto, t.segundo);
        let _ = std::fs::create_dir_all(&pasta);
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(pasta.join("erros.txt"))
            .and_then(|mut f| std::io::Write::write_all(&mut f, linha.as_bytes()));
        padrao(info);
    }));
    // para conferir a atualização sem instalar nada: procura a versão mais recente, ou baixa e confere o SHA-256
    if std::env::args().any(|a| a == "--procurar-versao") {
        println!("{}", atualizacao::procurar().unwrap_or_else(|e| json!({ "erro": e })));
        return;
    }
    if std::env::args().any(|a| a == "--baixar-versao") {
        match atualizacao::baixar_e_conferir(|_, _| {}) {
            Ok((v, arq)) => println!("{}", json!({ "versao": v, "arquivo": arq.display().to_string(), "sha256": "confere" })),
            Err(e) => println!("{}", json!({ "erro": e })),
        }
        return;
    }
    #[cfg(feature = "api")]
    if opcoes.mcp {
        std::process::exit(mcp::rodar(&opcoes));
    }
    let espera = if std::env::args().any(|a| a == "--depois-de-atualizar") { 15 } else { 0 };
    if sistema::ja_aberto(&opcoes.dados.display().to_string().to_lowercase(), Duration::from_secs(espera)) {
        return;
    }
    // pacote "para escola sem internet": o instalador do WebView2 da Microsoft ao lado do programa
    let ao_lado = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_path_buf()));
    if !sistema::tem_webview2() && !ao_lado.is_some_and(|p| sistema::instalar_webview2_de(&p, false)) {
        sistema::avisar_sem_webview2();
        return;
    }
    // o WebView2 que sobrou de uma queda segura o perfil e faria a janela falhar (já com a trava de instância única)
    sistema::fechar_webview2_orfao(&opcoes.dados);
    #[cfg(feature = "esquecer-perfil")]
    sistema::esquecer_perfil_do_webview2();
    let config = Config::ler(&opcoes.dados);
    let estado = Estado { config: Mutex::new(config), token: Mutex::new(None), porta_api: Mutex::new(0),
        recolhida: Mutex::new(false), config_mudou: AtomicBool::new(false), pendentes: Mutex::new(HashMap::new()),
        proximo: AtomicU64::new(1), epoca_agentes: AtomicU64::new(0), opcoes };
    let nivel = estado.config.lock().unwrap().agentes.clone();
    let _ = preparar_token(&estado, &nivel);
    let resultado = tauri::Builder::default()
        .manage(estado)
        .invoke_handler(tauri::generate_handler![responder, camada, estado_camada, tecla, fixar, recolher, congelar,
            salvar, capturar, fim, ajuda, info, agentes, caneta_vista, dica_ink_mostrada, preferencias, escolher_cor,
            comando_na_barra, copiar, telas, mudar_tela, procurar_versao, instalar_versao]);
    // Ctrl+Alt+F liga e desliga o Mouse livre de qualquer programa (com ele ligado o teclado não chega aqui)
    #[cfg(feature = "atalho-global")]
    let resultado = resultado.plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app, _atalho, ev| {
        if ev.state() == ShortcutState::Pressed {
            js(app, "barra", "tecla('f')");
        }
    }).build());
    let resultado = resultado
        // lembra onde a barra ficou (aberta ou recolhida) para abrir no mesmo lugar da próxima vez
        .on_window_event(|w, ev| {
            if let (WindowEvent::CloseRequested { api, .. }, "camada" | "barra") = (ev, w.label()) {
                api.prevent_close();
                js(w.app_handle(), "barra", "executar('programa.fechar', {}, 'botao').catch(e => aviso(e.message))");
                return;
            }
            if let (WindowEvent::Moved(p), "barra") = (ev, w.label()) {
                let estado = w.state::<Estado>();
                let recolhida = *estado.recolhida.lock().unwrap();
                let dx = if recolhida { deslocamento_do_botao(w.scale_factor().unwrap_or(1.0)) } else { 0 };
                estado.config.lock().unwrap().barra = Some((p.x - dx, p.y));
                estado.config_mudou.store(true, Ordering::Relaxed);
            }
        })
        .setup(|app| {
            #[cfg(feature = "atalho-global")]
            let _ = app.global_shortcut().register(Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyF));
            let h = app.handle().clone();
            let tela = app.primary_monitor()?.ok_or("sem monitor")?;
            let escala = tela.scale_factor();
            let (larg, alt) = (tela.size().width as f64 / escala, tela.size().height as f64 / escala);
            let inicio = iniciar_js(&h);
            let (recolhida, salva) = {
                let c = app.state::<Estado>().config.lock().unwrap().clone();
                (c.recolhida, c.barra)
            };
            *app.state::<Estado>().recolhida.lock().unwrap() = recolhida;
            let aberta = barra_na_tela(Some(tela.clone()));
            let tam = if recolhida { (BOTAO, BOTAO) } else { aberta };
            // O WebView2 às vezes falha ao criar a janela (0x80070578, "identificador da janela inválido"), por exemplo
            // com outro programa abrindo e fechando janelas dele ao mesmo tempo. Tenta de novo; se não der, explica em
            // português e fecha sem caixa técnica. O detalhe vai para <dados>\janelas.log.
            let mut falhas: Vec<String> = Vec::new();
            let (_camada, barra) = loop {
                match criar_janelas(app, &inicio, (larg, alt), aberta, tam) {
                    Ok(j) => break j,
                    Err(e) => {
                        for rotulo in ["barra", "camada"] {
                            if let Some(w) = app.get_webview_window(rotulo) { let _ = w.destroy(); }
                        }
                        falhas.push(e);
                        anotar_janelas(&app.state::<Estado>().opcoes.dados, &falhas);
                        if falhas.len() >= 3 {
                            if std::env::var_os("GIZ_TESTE_FALHAR_JANELA").is_none() {
                                sistema::avisar_erro("O Giz na Tela não conseguiu abrir a janela dele agora.\n\n\
                                    Espere alguns segundos e abra o Giz na Tela de novo. Se continuar, feche os outros \
                                    programas abertos ou reinicie o computador.");
                            }
                            std::process::exit(1);
                        }
                        std::thread::sleep(Duration::from_millis(800 * falhas.len() as u64));
                    }
                }
            };
            // abre na tela da última vez, se ela ainda estiver ligada (o projetor pode ter sido desligado)
            let lembrada = app.state::<Estado>().config.lock().unwrap().tela.clone();
            if let Some(m) = telas_em_ordem(&h).into_iter().skip(1).find(|m| lembrada.is_some() && m.name() == lembrada.as_ref()) {
                let _ = por_na_tela(&h, &m);
            }
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
            // depois do show: ao mostrar, o Tauri refaz o estilo da janela e tiraria a marca do botão
            if let Ok(h) = barra.hwnd() { sistema::botao_na_barra_de_tarefas(h.0 as isize); }
            let _ = barra.set_skip_taskbar(false);   // pede o botão à barra de tarefas (ITaskbarList::AddTab)
            // API local: sempre escutando, mas sem token (Agentes de IA desligados) recusa tudo
            #[cfg(feature = "api")]
            if let Some((ouvinte, porta)) = api::abrir(app.state::<Estado>().opcoes.porta_api) {
                let estado = app.state::<Estado>();
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
