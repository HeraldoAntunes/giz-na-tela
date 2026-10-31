// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Versão nova pelo Sobre, como no Giz Livre
// Só consulta a internet quando o professor clica em "Procurar versão nova" e só instala quando ele confirma. Baixa o
// instalador da release mais recente no GitHub, confere o SHA-256 com o SHA256SUMS.txt da mesma release e abre o
// instalador em silêncio, que fecha este programa, instala por cima e o abre de novo. Só fala com o GitHub, por HTTPS,
// com o WinHTTP do próprio Windows (sem biblioteca a mais).
use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::AppHandle;
use crate::leitura::{notas, numeros, soma_publicada};

const REPO: &str = "HeraldoAntunes/giz-na-tela";
// o GitHub entrega os anexos da release por outros endereços dele; qualquer outro destino é recusado
const HOSTS: [&str; 4] = ["api.github.com", "github.com", "objects.githubusercontent.com", "release-assets.githubusercontent.com"];

#[link(name = "winhttp")]
extern "system" {
    fn WinHttpOpen(agente: *const u16, acesso: u32, proxy: *const u16, sem_proxy: *const u16, flags: u32) -> isize;
    fn WinHttpConnect(sessao: isize, host: *const u16, porta: u16, r: u32) -> isize;
    fn WinHttpOpenRequest(conexao: isize, verbo: *const u16, caminho: *const u16, versao: *const u16,
        referencia: *const u16, tipos: *const *const u16, flags: u32) -> isize;
    fn WinHttpSetOption(h: isize, opcao: u32, valor: *const u32, tam: u32) -> i32;
    fn WinHttpSendRequest(pedido: isize, cab: *const u16, tam_cab: u32, opcional: *const u8, tam_opcional: u32,
        total: u32, contexto: usize) -> i32;
    fn WinHttpReceiveResponse(pedido: isize, r: *mut u8) -> i32;
    fn WinHttpQueryHeaders(pedido: isize, nivel: u32, nome: *const u16, buf: *mut u16, tam: *mut u32, indice: *mut u32) -> i32;
    fn WinHttpReadData(pedido: isize, buf: *mut u8, tam: u32, lidos: *mut u32) -> i32;
    fn WinHttpCloseHandle(h: isize) -> i32;
}

fn largo(t: &str) -> Vec<u16> { t.encode_utf16().chain([0]).collect() }

// os três "handles" do WinHTTP se fecham sozinhos, em qualquer saída
struct Alca(isize);
impl Drop for Alca {
    fn drop(&mut self) { if self.0 != 0 { unsafe { WinHttpCloseHandle(self.0); } } }
}

fn cabecalho(pedido: isize, qual: u32) -> Option<String> {
    let mut buf = vec![0u16; 2048];
    let mut tam = (buf.len() * 2) as u32;
    let ok = unsafe { WinHttpQueryHeaders(pedido, qual, std::ptr::null(), buf.as_mut_ptr(), &mut tam, std::ptr::null_mut()) };
    (ok != 0).then(|| String::from_utf16_lossy(&buf[..tam as usize / 2]))
}

fn host_e_caminho(url: &str) -> Result<(String, String), String> {
    let resto = url.strip_prefix("https://").ok_or("só HTTPS")?;
    let (host, caminho) = resto.split_once('/').map(|(h, c)| (h, format!("/{c}"))).unwrap_or((resto, "/".into()));
    if !HOSTS.contains(&host) {
        return Err(format!("endereço fora do GitHub recusado: {host}"));
    }
    Ok((host.to_string(), caminho))
}

// GET por HTTPS, seguindo os redirecionamentos à mão para conferir que todos ficam no GitHub
fn baixar(url: &str, limite: usize, mut andamento: impl FnMut(u64, u64)) -> Result<Vec<u8>, String> {
    let agente = largo(&format!("GizNaTela/{}", crate::VERSAO));
    let sessao = Alca(unsafe { WinHttpOpen(agente.as_ptr(), 4, std::ptr::null(), std::ptr::null(), 0) });  // proxy do sistema
    if sessao.0 == 0 { return Err("o Windows não abriu a conexão".into()); }
    let mut url = url.to_string();
    for _ in 0..6 {
        let (host, caminho) = host_e_caminho(&url)?;
        let conexao = Alca(unsafe { WinHttpConnect(sessao.0, largo(&host).as_ptr(), 443, 0) });
        let pedido = Alca(unsafe { WinHttpOpenRequest(conexao.0, largo("GET").as_ptr(), largo(&caminho).as_ptr(),
            std::ptr::null(), std::ptr::null(), std::ptr::null(), 0x0080_0000) });   // WINHTTP_FLAG_SECURE
        if pedido.0 == 0 { return Err("o Windows não preparou o pedido".into()); }
        let nunca = 0u32;
        unsafe { WinHttpSetOption(pedido.0, 88, &nunca, 4) };   // WINHTTP_OPTION_REDIRECT_POLICY: nunca sozinho
        let cab = largo("Accept: application/vnd.github+json, application/octet-stream\r\n");
        let enviou = unsafe { WinHttpSendRequest(pedido.0, cab.as_ptr(), u32::MAX, std::ptr::null(), 0, 0, 0) != 0
            && WinHttpReceiveResponse(pedido.0, std::ptr::null_mut()) != 0 };
        if !enviou { return Err("sem resposta do GitHub (sem internet?)".into()); }
        let status: u32 = cabecalho(pedido.0, 19).and_then(|s| s.trim().parse().ok()).unwrap_or(0);   // STATUS_CODE
        if matches!(status, 301 | 302 | 303 | 307 | 308) {
            url = cabecalho(pedido.0, 33).ok_or("redirecionamento sem destino")?;   // LOCATION
            continue;
        }
        if status != 200 { return Err(format!("o GitHub respondeu {status}")); }
        let total: u64 = cabecalho(pedido.0, 5).and_then(|s| s.trim().parse().ok()).unwrap_or(0);   // CONTENT_LENGTH
        let (mut corpo, mut pedaco) = (Vec::new(), vec![0u8; 64 * 1024]);
        loop {
            let mut lidos = 0u32;
            if unsafe { WinHttpReadData(pedido.0, pedaco.as_mut_ptr(), pedaco.len() as u32, &mut lidos) } == 0 {
                return Err("o download parou no meio".into());
            }
            if lidos == 0 { return Ok(corpo); }
            corpo.extend_from_slice(&pedaco[..lidos as usize]);
            if corpo.len() > limite { return Err("arquivo maior que o esperado".into()); }
            andamento(corpo.len() as u64, total);
        }
    }
    Err("redirecionamentos demais".into())
}

struct Versao { numero: String, notas: String, instalador: Option<(String, String)>, somas: Option<String> }

fn mais_recente() -> Result<Versao, String> {
    let r = baixar(&format!("https://api.github.com/repos/{REPO}/releases/latest"), 1 << 20, |_, _| {})?;
    let v: Value = serde_json::from_slice(&r).map_err(|_| "resposta estranha do GitHub")?;
    let numero = v["tag_name"].as_str().ok_or("release sem número")?.trim_start_matches('v').to_string();
    let anexo = |nome: &str| v["assets"].as_array().and_then(|a| a.iter().find(|x| x["name"] == nome))
        .and_then(|x| x["browser_download_url"].as_str().map(String::from));
    // desde a 1.4.0 o instalador vai dentro de um zip (o .exe solto levava um alerta no VirusTotal, de causa não confirmada); o .exe
    // solto das versões anteriores continua aceito
    let instalador = [format!("Instalar-GizNaTela-{numero}.zip"), format!("Instalar-GizNaTela-{numero}.exe")]
        .into_iter().find_map(|nome| anexo(&nome).map(|u| (nome, u)));
    Ok(Versao { notas: notas(v["body"].as_str().unwrap_or("")), instalador, somas: anexo("SHA256SUMS.txt"), numero })
}

// Só a cópia instalada para a pessoa se atualiza sozinha. A portátil (portatil.txt), a de desenvolvimento e a
// instalada para todos em Arquivos de Programas (pelo Inno Setup das versões até a 1.3.2) mostram a página.
pub fn modo_instalado() -> Option<()> {
    let exe = std::env::current_exe().ok()?;
    let pasta = exe.parent()?;
    if pasta.join("portatil.txt").exists() {
        return None;
    }
    let p = pasta.to_string_lossy().to_lowercase();
    let em = |var: &str, sub: &str| std::env::var(var).ok().map(|d| format!(r"{d}\{sub}").to_lowercase());
    (em("LOCALAPPDATA", r"Programs\Giz na Tela").as_deref() == Some(p.as_str())).then_some(())
}

pub fn procurar() -> Result<Value, String> {
    let v = mais_recente()?;
    Ok(if numeros(&v.numero) > numeros(crate::VERSAO) {
        json!({ "nova": true, "versao": v.numero, "notas": v.notas, "instalavel": modo_instalado().is_some() })
    } else {
        json!({ "nova": false, "versao": crate::VERSAO })
    })
}

// baixa o instalador da versão mais recente e confere o SHA-256 com o publicado; devolve o arquivo conferido
pub fn baixar_e_conferir(andamento: impl FnMut(u64, u64)) -> Result<(String, PathBuf), String> {
    let v = mais_recente()?;
    if numeros(&v.numero) <= numeros(crate::VERSAO) {
        return Err(format!("a versão publicada ({}) não é mais nova que esta", v.numero));
    }
    let (nome, url) = v.instalador.ok_or("a release não tem o instalador")?;
    let somas = baixar(&v.somas.ok_or("a release não tem o SHA256SUMS.txt")?, 1 << 16, |_, _| {})?;
    let esperado = soma_publicada(&String::from_utf8_lossy(&somas), &nome).ok_or("o instalador não está no SHA256SUMS.txt")?;
    let dados = baixar(&url, 200 << 20, andamento)?;
    if crate::sistema::sha256_hex(&dados)? != esperado {
        return Err("o arquivo baixado não confere com o SHA-256 publicado; nada foi instalado".into());
    }
    // pasta com nome que ninguém adivinha; o arquivo é conferido de novo já gravado, logo antes de rodar
    let pasta = std::env::temp_dir().join(format!("GizNaTela-atualizacao-{}", &crate::sistema::token_novo().map_err(|e| e.to_string())?[..16]));
    std::fs::create_dir_all(&pasta).map_err(|e| e.to_string())?;
    let arq = pasta.join(&nome);
    std::fs::write(&arq, &dados).map_err(|e| e.to_string())?;
    let gravado = std::fs::read(&arq).map_err(|e| e.to_string())?;
    if crate::sistema::sha256_hex(&gravado)? != esperado {
        let _ = std::fs::remove_dir_all(&pasta);
        return Err("o instalador gravado não confere com o SHA-256 publicado; nada foi instalado".into());
    }
    if !nome.ends_with(".zip") {
        return Ok((v.numero, arq));
    }
    // o zip conferido é aberto pelo tar do próprio Windows (Windows 10 de 2018 em diante); dentro vem o instalador
    extrair_zip(&arq, &pasta)?;
    let exe = pasta.join(format!("Instalar-GizNaTela-{}.exe", v.numero));
    if !exe.is_file() {
        return Err("o zip publicado não traz o instalador; nada foi instalado".into());
    }
    Ok((v.numero, exe))
}

fn extrair_zip(zip: &std::path::Path, pasta: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let tar = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("System32").join("tar.exe");
    let r = std::process::Command::new(&tar).arg("-xf").arg(zip).arg("-C").arg(pasta)
        .creation_flags(0x0800_0000)   // CREATE_NO_WINDOW
        .status().map_err(|e| format!("o Windows não abriu o zip ({e})"))?;
    if !r.success() { return Err("o Windows não conseguiu abrir o zip; nada foi instalado".into()); }
    Ok(())
}

// pela ajuda, depois da confirmação do professor: o andamento aparece lá
pub fn instalar(app: &AppHandle) -> Result<(), String> {
    modo_instalado()
        .ok_or("esta cópia não veio do instalador (é a portátil ou a de desenvolvimento): baixe a versão nova pela página")?;
    let mut ultimo = u64::MAX;
    let (_, arq) = baixar_e_conferir(|feito, total| {
        let pct = if total > 0 { feito * 100 / total } else { 0 };
        if pct != ultimo { ultimo = pct; crate::js(app, "ajuda", &format!("andamento({pct})")); }
    })?;
    crate::gravar_config(app);
    // o instalador é o próprio programa novo (src/instalador.rs): /S em silêncio, /reabrir=1 abre de novo no fim
    std::process::Command::new(&arq)
        .args(["/S", "/reabrir=1"])
        .spawn().map_err(|e| format!("não consegui abrir o instalador ({e})"))?;
    let a = app.clone();
    std::thread::spawn(move || { std::thread::sleep(std::time::Duration::from_millis(800)); a.exit(0); });
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn compara_versoes() {
        assert!(numeros("v1.10.0") > numeros("1.9.9"));
        assert!(numeros("1.2.1") > numeros("1.2.0"));
        assert!(numeros("1.2.1") == numeros("v1.2.1"));
    }
    #[test]
    fn so_github_por_https() {
        assert!(host_e_caminho("https://github.com/a/b").is_ok());
        assert!(host_e_caminho("http://github.com/a").is_err());
        assert!(host_e_caminho("https://github.com.mal.com/a").is_err());
        assert!(host_e_caminho("https://exemplo.com/a").is_err());
    }
    #[test]
    fn le_a_soma_publicada() {
        let s = format!("{}  Instalar-GizNaTela-1.3.0.exe\r\n{}  outro.zip\r\n", "A".repeat(64), "b".repeat(64));
        assert_eq!(soma_publicada(&s, "Instalar-GizNaTela-1.3.0.exe"), Some("a".repeat(64)));
        assert_eq!(soma_publicada(&s, "Instalar-GizNaTela-1.3.exe"), None);
    }
    // a soma tem de ser 64 dígitos hexadecimais: 64 letras quaisquer falham já na leitura (GNT-X07, auditoria de
    // 30/10/2026; a conferência do arquivo baixado continua depois)
    #[test]
    fn soma_publicada_so_hexadecimal() {
        let s = format!("{}  Instalar-GizNaTela-1.4.0.zip\n", "z".repeat(64));
        assert_eq!(soma_publicada(&s, "Instalar-GizNaTela-1.4.0.zip"), None);
        let s = format!("{}  Instalar-GizNaTela-1.4.0.zip\n", "0123456789abcdef".repeat(4));
        assert!(soma_publicada(&s, "Instalar-GizNaTela-1.4.0.zip").is_some());
    }
    #[test]
    fn abre_o_zip_do_instalador() {
        let base = std::env::temp_dir().join(format!("giz-teste-zip-{}", std::process::id()));
        let (de, para) = (base.join("de"), base.join("para"));
        std::fs::create_dir_all(&de).unwrap();
        std::fs::create_dir_all(&para).unwrap();
        std::fs::write(de.join("Instalar-GizNaTela-9.9.9.exe"), b"conteudo de teste").unwrap();
        let zip = base.join("Instalar-GizNaTela-9.9.9.zip");
        let ok = std::process::Command::new("tar").arg("-a").arg("-cf").arg(&zip).arg("-C").arg(&de)
            .arg("Instalar-GizNaTela-9.9.9.exe").status().unwrap().success();
        assert!(ok);
        extrair_zip(&zip, &para).unwrap();
        assert_eq!(std::fs::read(para.join("Instalar-GizNaTela-9.9.9.exe")).unwrap(), b"conteudo de teste");
        let _ = std::fs::remove_dir_all(&base);
    }
    // a mesma função existe na 1.3.2: o Sobre dela mostra este começo antes do "a release não tem o instalador"
    #[test]
    fn notas_da_1_4_0_avisam_quem_tem_a_1_3() {
        let n = notas(include_str!("../docs/NOTAS-1.4.0.md"));
        assert!(n.starts_with("ATENÇÃO, QUEM TEM A 1.3.x"), "{}", &n[..80.min(n.len())]);
        assert!(n.contains("Instalar-GizNaTela-1.4.0.zip") && !n.contains("<p>") && !n.contains("**"));
    }
    #[test]
    fn notas_sem_imagens() {
        let n = notas("tabela\n## O que mudou\nPacote.\n<p><img src=x></p>\n- **Laço:** novo\n\n**Conferir o download:** x");
        assert_eq!(n, "Pacote.\n- Laço: novo");
    }
}
