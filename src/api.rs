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

// cabeçalho até a linha em branco, com limite; devolve o que já leu e onde o cabeçalho termina.
// Err(true): grande demais; Err(false): a conexão fechou antes
fn ler_cabecalho(c: &mut impl Read) -> Result<(Vec<u8>, usize), bool> {
    let mut buf = Vec::new();
    let mut pedaco = [0u8; 4096];
    loop {
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") { return Ok((buf, i)); }
        if buf.len() > MAX_CABECALHO { return Err(true); }
        match c.read(&mut pedaco) { Ok(0) | Err(_) => return Err(false), Ok(n) => buf.extend_from_slice(&pedaco[..n]) }
        // o fim do cabeçalho pode chegar no mesmo bloco que passa do limite: o limite vale para o cabeçalho inteiro
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            return if i + 4 > MAX_CABECALHO { Err(true) } else { Ok((buf, i)) };
        }
    }
}

// campos que só podem vir uma vez: dois Content-Length (mesmo iguais), dois Host ou dois tokens deixam a leitura
// ambígua, e o corpo em pedaços (Transfer-Encoding) não é usado por nenhum cliente desta API
fn campos_repetidos(cab: &str) -> Option<String> {
    let nomes: Vec<String> = cab.split("\r\n").skip(1).filter_map(|l| Some(l.split_once(':')?.0.trim().to_ascii_lowercase())).collect();
    if nomes.iter().any(|n| n == "transfer-encoding") { return Some("Transfer-Encoding não é aceito".into()); }
    ["content-length", "host", "x-giz-tela", "origin"].iter()
        .find(|u| nomes.iter().filter(|n| n == *u).count() > 1)
        .map(|u| format!("cabeçalho {u} repetido"))
}

// o pedido só vale se o nível dos agentes não mudou desde que ele começou (desligar cancela o que ainda não foi feito)
fn pedido_ainda_vale(epoca_no_inicio: u64, epoca_agora: u64, nivel_agora: &str) -> bool {
    epoca_no_inicio == epoca_agora && nivel_agora != "desligado"
}

fn atender(app: &AppHandle, mut c: TcpStream, porta: u16) {
    let _ = c.set_read_timeout(Some(Duration::from_secs(5)));
    let mut pedaco = [0u8; 4096];
    let (buf, fim) = match ler_cabecalho(&mut c) {
        Ok(r) => r,
        Err(true) => return erro(&mut c, 413, "cabeçalho grande demais"),
        Err(false) => return,
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
    if let Some(motivo) = campos_repetidos(&cab) {
        return erro(&mut c, 400, &motivo);
    }
    let estado = app.state::<Estado>();
    // a época antes do nível: se o professor mudar o nível depois daqui, a conferência antes de despachar pega
    let epoca = estado.epoca_agentes.load(std::sync::atomic::Ordering::SeqCst);
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
        // só na compilação de teste (cargo build, sem --release): roda um trecho de JavaScript na camada, para os
        // testes de eventos de ponteiro (tests/testar_camada.py) usarem o WebView2 de verdade; o programa publicado
        // não tem esta rota
        #[cfg(debug_assertions)]
        ("POST", "/api/teste/camada") if nivel == "agir" => {
            let js = String::from_utf8_lossy(&corpo).to_string();
            return match crate::chamar(app, "camada", &js, Duration::from_secs(20)) {
                Ok(valor) => responder(&mut c, 200, &json!({ "ok": true, "valor": valor })),
                Err(e) => erro(&mut c, 400, &e),
            };
        }
        ("GET", "/api/comandos") => ("comandos.listar".to_string(), json!({})),
        ("POST", "/api/comando") => {
            let Ok(v) = serde_json::from_slice::<Value>(&corpo) else { return erro(&mut c, 400, "corpo precisa ser JSON"); };
            let Some(nome) = v["nome"].as_str() else { return erro(&mut c, 400, "falta \"nome\" do comando"); };
            (nome.to_string(), v.get("parametros").cloned().unwrap_or(json!({})))
        }
        _ => return erro(&mut c, 404, "rotas: GET /api/comandos e POST /api/comando"),
    };
    // conferido de novo logo antes de despachar: o corpo pode ter levado segundos para chegar
    let agora = estado.epoca_agentes.load(std::sync::atomic::Ordering::SeqCst);
    if !pedido_ainda_vale(epoca, agora, &estado.config.lock().unwrap().agentes) {
        return erro(&mut c, 403, "os Agentes de IA foram desligados ou mudaram de nível durante o pedido");
    }
    // a época vai junto: a barra recusa o pedido se o nível mudar enquanto ele espera a vez na fila
    let js = format!("return await executarDeFora({}, {}, {}, {epoca})", json!(nome), parametros, json!(nivel));
    match crate::chamar(app, "barra", &js, Duration::from_secs(60)) {
        Ok(valor) => responder(&mut c, 200, &json!({ "ok": true, "valor": valor })),
        Err(e) => erro(&mut c, 400, &e),
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    // campo que só pode vir uma vez (Content-Length, Host, token) repetido é recusado, mesmo com o mesmo valor, e o
    // corpo em pedaços (Transfer-Encoding) não é aceito (GNT-X01, auditoria de 30/10/2026)
    #[test]
    fn campo_unico_repetido_e_recusado() {
        let cab = |extra: &str| format!("POST /api/comando HTTP/1.1\r\nHost: 127.0.0.1:8833\r\n{extra}");
        assert!(campos_repetidos(&cab("Content-Length: 10")).is_none());
        assert!(campos_repetidos(&cab("Content-Length: 10\r\nContent-Length: 0")).is_some());
        assert!(campos_repetidos(&cab("content-length: 10\r\nCONTENT-LENGTH: 10")).is_some());
        assert!(campos_repetidos(&cab("Host: localhost:8833")).is_some());
        assert!(campos_repetidos(&cab("X-Giz-Tela: a\r\nX-Giz-Tela: b")).is_some());
        assert!(campos_repetidos(&cab("Transfer-Encoding: chunked")).is_some());
    }
    // pedido que começou com os agentes ligados não é despachado se o professor desligou ou mudou o nível no meio
    // (GNT-X04, auditoria de 30/10/2026: desligar cancela o que ainda não foi feito)
    #[test]
    fn pedido_feito_antes_de_desligar_nao_vale() {
        assert!(pedido_ainda_vale(3, 3, "agir"));
        assert!(!pedido_ainda_vale(3, 4, "agir"), "o nível mudou depois do pedido começar");
        assert!(!pedido_ainda_vale(3, 3, "desligado"));
    }
    #[test]
    fn cabecalho_acima_do_limite_e_recusado() {
        let normal = "GET /api/comandos HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        assert!(ler_cabecalho(&mut normal.as_bytes()).is_ok());
        // o fim do cabeçalho chega no mesmo bloco que passa do limite: tem de ser recusado assim mesmo
        let grande = format!("GET / HTTP/1.1\r\nX: {}\r\n\r\n", "a".repeat(MAX_CABECALHO + 1000));
        assert_eq!(ler_cabecalho(&mut std::io::Cursor::new(grande.into_bytes())).err(), Some(true));
        let no_limite = format!("GET / HTTP/1.1\r\nX: {}\r\n\r\n", "a".repeat(MAX_CABECALHO - 40));
        assert!(ler_cabecalho(&mut std::io::Cursor::new(no_limite.into_bytes())).is_ok());
    }
}
