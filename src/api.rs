// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). API local para agentes de IA e testes
// Só em 127.0.0.1, com token. O token existe apenas com "Agentes de IA" ligado na ajuda e fica em <dados>\agente\token,
// que só a conta do professor lê (outra conta do Windows não usa a API). Cada pedido vira um comando do registro,
// executado pela barra pelo mesmo caminho do clique; o nível (ler ou agir) é o que o professor escolheu.
//   GET  /api/comandos            lista de comandos, atalhos e botões
//   POST /api/comando  {"nome": "...", "parametros": {...}}
use crate::Estado;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use tauri::{AppHandle, Manager};

const MAX_CABECALHO: usize = 16 * 1024;
const MAX_CORPO: usize = 1024 * 1024;   // traço com 5000 pontos cabe com folga

pub fn abrir(porta: u16) -> Option<(TcpListener, u16)> {
    // a porta pedida ou, se ocupada, uma das próximas; o número fica em <dados>\agente\porta para o MCP achar
    let ultima = if porta == 0 { 0 } else { porta.saturating_add(9) };
    (porta..=ultima).find_map(|p| TcpListener::bind(("127.0.0.1", p)).ok())
        .and_then(|o| { let p = o.local_addr().ok()?.port(); Some((o, p)) })
}

pub fn servir(app: AppHandle, ouvinte: TcpListener, porta: u16) {
    for c in ouvinte.incoming().flatten() {
        let app = app.clone();
        let _ = std::thread::Builder::new().spawn(move || atender(&app, c, porta));
    }
}

fn responder(c: &mut TcpStream, status: u16, corpo: &Value) {
    let texto = corpo.to_string();
    let nome = match status { 200 => "OK", 400 => "Bad Request", 401 => "Unauthorized", 403 => "Forbidden",
        404 => "Not Found", 413 => "Payload Too Large", _ => "Error" };
    let _ = write!(c, "HTTP/1.1 {status} {nome}\r\nContent-Type: application/json; charset=utf-8\r\n\
        Content-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{texto}", texto.len());
}
fn erro(c: &mut TcpStream, status: u16, msg: &str) {
    responder(c, status, &json!({ "ok": false, "erro": msg }));
}

fn atender(app: &AppHandle, mut c: TcpStream, porta: u16) {
    let _ = c.set_read_timeout(Some(Duration::from_secs(5)));
    // cabeçalho até a linha em branco, com limite
    let mut buf = Vec::new();
    let mut pedaco = [0u8; 4096];
    let fim = loop {
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") { break i; }
        if buf.len() > MAX_CABECALHO { return erro(&mut c, 413, "cabeçalho grande demais"); }
        match c.read(&mut pedaco) { Ok(0) | Err(_) => return, Ok(n) => buf.extend_from_slice(&pedaco[..n]) }
    };
    let cab = String::from_utf8_lossy(&buf[..fim]).to_string();
    let mut linhas = cab.split("\r\n");
    let mut inicio = linhas.next().unwrap_or("").split_whitespace();
    let (metodo, rota) = (inicio.next().unwrap_or("").to_string(), inicio.next().unwrap_or("").to_string());
    let campo = |nome: &str| cab.split("\r\n").skip(1).find_map(|l| {
        let (k, v) = l.split_once(':')?;
        k.trim().eq_ignore_ascii_case(nome).then(|| v.trim().to_string())
    });
    // contra DNS rebinding e página da internet aberta no navegador: só os nomes locais da própria porta, sem Origin
    let host = campo("host").unwrap_or_default().to_ascii_lowercase();
    if ![format!("127.0.0.1:{porta}"), format!("localhost:{porta}")].contains(&host) || campo("origin").is_some() {
        return erro(&mut c, 403, "pedido recusado: só programas deste computador, sem navegador");
    }
    let estado = app.state::<Estado>();
    let (nivel, token) = {
        let cfg = estado.config.lock().unwrap();
        (cfg.agentes.clone(), estado.token.lock().unwrap().clone())
    };
    let Some(token) = token.filter(|_| nivel != "desligado") else {
        return erro(&mut c, 403, "Agentes de IA desligados no Giz na Tela. Peça ao professor para ligar em Ajuda e ajustes (tecla F1).");
    };
    if !crate::sistema::iguais(&campo("x-giz-tela").unwrap_or_default(), &token) {
        return erro(&mut c, 401, "token ausente ou errado (cabeçalho X-Giz-Tela)");
    }
    let tam: usize = match campo("content-length").map(|v| v.parse()) {
        None => 0,
        Some(Ok(n)) if n <= MAX_CORPO => n,
        Some(Ok(_)) => return erro(&mut c, 413, "pedido grande demais"),
        Some(Err(_)) => return erro(&mut c, 400, "Content-Length inválido"),
    };
    let mut corpo = buf[fim + 4..].to_vec();
    while corpo.len() < tam {
        match c.read(&mut pedaco) { Ok(0) | Err(_) => return, Ok(n) => corpo.extend_from_slice(&pedaco[..n]) }
    }
    corpo.truncate(tam);
    let (nome, parametros) = match (metodo.as_str(), rota.as_str()) {
        ("GET", "/api/comandos") => ("comandos.listar".to_string(), json!({})),
        ("POST", "/api/comando") => {
            let Ok(v) = serde_json::from_slice::<Value>(&corpo) else { return erro(&mut c, 400, "corpo precisa ser JSON"); };
            let Some(nome) = v["nome"].as_str() else { return erro(&mut c, 400, "falta \"nome\" do comando"); };
            (nome.to_string(), v.get("parametros").cloned().unwrap_or(json!({})))
        }
        _ => return erro(&mut c, 404, "rotas: GET /api/comandos e POST /api/comando"),
    };
    let js = format!("return await executarDeFora({}, {}, {})", json!(nome), parametros, json!(nivel));
    match crate::chamar(app, "barra", &js, Duration::from_secs(60)) {
        Ok(valor) => responder(&mut c, 200, &json!({ "ok": true, "valor": valor })),
        Err(e) => erro(&mut c, 400, &e),
    }
}
