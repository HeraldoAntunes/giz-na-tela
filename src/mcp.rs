// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Servidor MCP (GizNaTela.exe --mcp, por stdio)
// Deixa um agente de IA compatível com MCP ver e anotar a tela pelo Giz na Tela já aberto. Não abre o anotador: tudo
// passa pela API local (api.rs), com o token que só existe com "Agentes de IA" ligado na ajuda, no nível escolhido
// pelo professor (só ler ou agir). Protocolo: JSON-RPC 2.0, uma mensagem por linha.
use crate::config::Opcoes;
use crate::sistema;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::time::Duration;

const PROTOCOLOS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];
const INSTRUCOES: &str = "Ferramentas para ver e anotar a tela do professor pelo Giz na Tela (anotador por cima de \
    qualquer programa). Posições em px da tela (estado_giz_na_tela diz largura e altura). Para várias anotações de uma \
    vez, prefira desenhar (um passo só no Desfazer do professor: Ctrl+Z tira tudo). Cada ação aparece na tela com o \
    aviso \"Agente: ...\". O nível (só ler ou agir) é escolhido pelo professor na ajuda do Giz na Tela.";

struct Falha(String);

fn pedir(opcoes: &Opcoes, metodo: &str, rota: &str, corpo: &Value) -> Result<Value, Falha> {
    let pasta = opcoes.dados.join("agente");
    // lidos a cada chamada: o professor pode desligar e religar os agentes (o token muda)
    let token = std::fs::read_to_string(pasta.join("token")).map(|t| t.trim().to_string()).unwrap_or_default();
    if token.is_empty() {
        return Err(Falha("Agentes de IA desligados no Giz na Tela. Peça ao professor para abrir o Giz na Tela e \
            ligar os agentes em Ajuda e ajustes (tecla F1).".into()));
    }
    let porta = std::fs::read_to_string(pasta.join("porta")).ok().and_then(|p| p.trim().parse().ok())
        .unwrap_or(opcoes.porta_api);
    let dados = if corpo.is_null() { Vec::new() } else { corpo.to_string().into_bytes() };
    let (status, resposta) = sistema::http(porta, metodo, rota, &[("X-Giz-Tela", &token)], "application/json", &dados,
        Duration::from_secs(70)).map_err(|_| Falha("O Giz na Tela não está aberto. Peça ao professor para abrir.".into()))?;
    let v: Value = serde_json::from_slice(&resposta).unwrap_or(Value::Null);
    if status == 200 && v["ok"].as_bool() == Some(true) {
        Ok(v["valor"].clone())
    } else {
        Err(Falha(v["erro"].as_str().map(String::from).unwrap_or(format!("o Giz na Tela recusou o pedido (HTTP {status})"))))
    }
}
fn comando(opcoes: &Opcoes, nome: &str, parametros: Value) -> Result<Value, Falha> {
    pedir(opcoes, "POST", "/api/comando", &json!({ "nome": nome, "parametros": parametros }))
}

fn ferramenta(nome: &str, descricao: &str, props: Value, obrig: &[&str], leitura: bool) -> Value {
    json!({ "name": nome, "description": descricao,
        "inputSchema": { "type": "object", "properties": props, "required": obrig, "additionalProperties": false },
        "annotations": { "readOnlyHint": leitura, "destructiveHint": false, "idempotentHint": leitura, "openWorldHint": false } })
}
fn ferramentas() -> Value {
    let item = json!({ "type": "object", "description": "tipo: texto {texto, x, y, cor?, tamanho?}; numero {x, y, \
        numero?, cor?}; correcao {marca: visto|xis, x, y, cor?}; forma {forma: reta|seta|retangulo|elipse, x1, y1, x2, \
        y2, cor?, espessura?: fina|media|grossa, preencher?: nada|translucido|cheio}; traco {pontos: [[x, y], ...], \
        cor?, espessura?, marca?}",
        "properties": { "tipo": { "type": "string", "enum": ["texto", "numero", "correcao", "forma", "traco"] } },
        "required": ["tipo"] });
    json!([
        ferramenta("estado_giz_na_tela", "Estado do anotador: modo (escrevendo ou mouse livre), ferramenta, cor, \
            quantidade de desenhos, se a tela está congelada ou ampliada e o tamanho da tela em px.", json!({}), &[], true),
        ferramenta("listar_comandos", "Lista os comandos do Giz na Tela (os mesmos dos botões e atalhos), com \
            parâmetros e nível, para usar em executar_comando.", json!({}), &[], true),
        ferramenta("capturar_tela", "Retrato da tela com as anotações (sem a barra), em PNG.", json!({
            "largura_max": { "type": "integer", "minimum": 160, "maximum": 3840, "description": "px (padrão 1280)" } }),
            &[], true),
        ferramenta("desenhar", "Anota a tela com vários itens de uma vez (até 100), num passo só do Desfazer: texto, \
            numero (círculo numerado), correcao (visto ou X), forma (reta, seta, retângulo, elipse) e traco (linha \
            livre). Ou todos entram, ou nenhum.", json!({
            "itens": { "type": "array", "minItems": 1, "maxItems": 100, "items": item } }), &["itens"], false),
        ferramenta("executar_comando", "Executa um comando do Giz na Tela pelo nome (veja listar_comandos), por \
            exemplo tela.congelar, quadro.alternar, ferramenta.escolher, desenho.desfazer.", json!({
            "nome": { "type": "string" },
            "parametros": { "type": "object", "description": "parâmetros do comando (vazio: nenhum)" } }),
            &["nome"], false),
    ])
}

fn chamar_ferramenta(opcoes: &Opcoes, nome: &str, a: &Value) -> Result<Value, Falha> {
    let texto = |v: Value| json!({ "content": [{ "type": "text",
        "text": serde_json::to_string_pretty(&v).unwrap_or_default() }] });
    match nome {
        "estado_giz_na_tela" => comando(opcoes, "estado.ler", json!({})).map(texto),
        "listar_comandos" => pedir(opcoes, "GET", "/api/comandos", &Value::Null)
            .map(|v| texto(json!({ "comandos": v["comandos"], "atalho_global": v["atalho_global"] }))),
        "capturar_tela" => {
            let mut p = json!({});
            if let Some(l) = a.get("largura_max") { p["largura_max"] = l.clone(); }
            let v = comando(opcoes, "captura.ler", p)?;
            Ok(json!({ "content": [
                { "type": "image", "mimeType": "image/png", "data": v["png_base64"] },
                { "type": "text", "text": format!("Tela retratada ({} x {} px).", v["largura"], v["altura"]) }] }))
        }
        "desenhar" => comando(opcoes, "desenho.adicionar", json!({ "itens": a.get("itens").cloned().unwrap_or(Value::Null) }))
            .map(texto),
        "executar_comando" => {
            let n = a["nome"].as_str().ok_or(Falha("nome: obrigatório".into()))?;
            comando(opcoes, n, a.get("parametros").cloned().unwrap_or(json!({}))).map(texto)
        }
        _ => Err(Falha(format!("ferramenta desconhecida: {nome}"))),
    }
}

fn tratar(opcoes: &Opcoes, msg: &Value) -> Option<Value> {
    let id = msg.get("id")?.clone();   // sem id é notificação: não tem resposta
    let metodo = msg["method"].as_str().unwrap_or("");
    let resultado = match metodo {
        "initialize" => {
            let pedido = msg["params"]["protocolVersion"].as_str().unwrap_or("");
            let versao = if PROTOCOLOS.contains(&pedido) { pedido } else { PROTOCOLOS[0] };
            json!({ "protocolVersion": versao, "capabilities": { "tools": {} },
                "serverInfo": { "name": "giz-na-tela", "version": crate::VERSAO }, "instructions": INSTRUCOES })
        }
        "ping" => json!({}),
        "tools/list" => json!({ "tools": ferramentas() }),
        "tools/call" => {
            let nome = msg["params"]["name"].as_str().unwrap_or("");
            let args = msg["params"].get("arguments").cloned().unwrap_or(json!({}));
            chamar_ferramenta(opcoes, nome, &args).unwrap_or_else(|Falha(e)|
                json!({ "content": [{ "type": "text", "text": e }], "isError": true }))
        }
        _ => return Some(json!({ "jsonrpc": "2.0", "id": id,
            "error": { "code": -32601, "message": format!("método desconhecido: {metodo}") } })),
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": resultado }))
}

pub fn rodar(opcoes: &Opcoes) -> i32 {
    let entrada = std::io::stdin();
    let mut saida = std::io::stdout();
    for linha in entrada.lock().lines() {
        let Ok(linha) = linha else { break };
        if linha.trim().is_empty() { continue; }
        let resposta = match serde_json::from_str::<Value>(&linha) {
            Ok(msg) => tratar(opcoes, &msg),
            Err(_) => Some(json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": "JSON inválido" } })),
        };
        if let Some(r) = resposta {
            if writeln!(saida, "{r}").and_then(|_| saida.flush()).is_err() { break; }
        }
    }
    0
}
