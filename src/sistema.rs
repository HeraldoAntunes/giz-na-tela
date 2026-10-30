// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Chamadas diretas ao Windows e HTTP local mínimo
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[link(name = "user32")]
extern "system" {
    fn SetWindowDisplayAffinity(hwnd: isize, afinidade: u32) -> i32;
    fn MessageBoxW(hwnd: isize, texto: *const u16, titulo: *const u16, tipo: u32) -> i32;
}
#[link(name = "advapi32")]
extern "system" {
    fn RegGetValueW(chave: isize, sub: *const u16, valor: *const u16, flags: u32, tipo: *mut u32, dados: *mut u16,
        tam: *mut u32) -> i32;
}
#[link(name = "comdlg32")]
extern "system" {
    fn ChooseColorW(c: *mut EscolhaDeCor) -> i32;
}
#[repr(C)]
struct EscolhaDeCor { tam: u32, dona: isize, inst: isize, cor: u32, personalizadas: *mut u32, flags: u32, dados: isize,
    gancho: isize, modelo: *const u16 }

// seletor de cores do Windows (já em português); devolve #rrggbb ou nada se o professor cancelar
pub fn escolher_cor(dona: isize, atual: &str) -> Option<String> {
    let h = u32::from_str_radix(atual.trim_start_matches('#'), 16).unwrap_or(0x2e9d4a);
    let (r, g, b) = ((h >> 16) & 255, (h >> 8) & 255, h & 255);
    let mut personalizadas = [0x00ffffffu32; 16];
    let mut c = EscolhaDeCor { tam: std::mem::size_of::<EscolhaDeCor>() as u32, dona, inst: 0, cor: (b << 16) | (g << 8) | r,
        personalizadas: personalizadas.as_mut_ptr(), flags: 0x1 | 0x2, dados: 0, gancho: 0, modelo: std::ptr::null() };  // CC_RGBINIT | CC_FULLOPEN
    if unsafe { ChooseColorW(&mut c) } == 0 { return None; }
    let v = c.cor;
    Some(format!("#{:02x}{:02x}{:02x}", v & 255, (v >> 8) & 255, (v >> 16) & 255))
}

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(hwnd: isize, op: *const u16, arq: *const u16, par: *const u16, pasta: *const u16, mostrar: i32) -> isize;
}

fn largo(t: &str) -> Vec<u16> { t.encode_utf16().chain([0]).collect() }

// O Giz na Tela desenha com o WebView2 (motor do Edge): vem no Windows 11 e nos Windows 10 atualizados. A conferência
// é a que a Microsoft recomenda (versão "pv" na chave do EdgeUpdate, da máquina ou da conta), a mesma do instalador.
pub fn tem_webview2() -> bool {
    const CHAVE: &str = r"Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
    const HKLM: isize = 0x80000002u32 as i32 as isize;
    const HKCU: isize = 0x80000001u32 as i32 as isize;
    const SO_TEXTO: u32 = 0x2;              // RRF_RT_REG_SZ
    const VISTA_32: u32 = 0x20000;          // RRF_SUBKEY_WOW6432KEY
    let ler = |raiz: isize, sub: String, flags: u32| -> bool {
        let (sub, valor) = (largo(&sub), largo("pv"));
        let mut buf = [0u16; 64];
        let mut tam = (buf.len() * 2) as u32;
        let r = unsafe { RegGetValueW(raiz, sub.as_ptr(), valor.as_ptr(), flags, std::ptr::null_mut(), buf.as_mut_ptr(), &mut tam) };
        if r != 0 { return false; }
        let v = String::from_utf16_lossy(&buf[..(tam as usize / 2).min(buf.len()).saturating_sub(1)]);
        !v.is_empty() && v != "0.0.0.0"
    };
    ler(HKLM, format!(r"SOFTWARE\{CHAVE}"), SO_TEXTO | VISTA_32) || ler(HKLM, format!(r"SOFTWARE\{CHAVE}"), SO_TEXTO)
        || ler(HKCU, format!(r"Software\{CHAVE}"), SO_TEXTO)
}

// ---- WebView2 para escola sem internet ----
// O pacote "para escola sem internet" traz, ao lado do instalador do Giz na Tela, o instalador completo do WebView2,
// que a Microsoft distribui assinado. Só é executado se a assinatura for válida e for da Microsoft (a mesma conferência
// do Bancada Livre).
pub const WEBVIEW2_COMPLETO: &str = "MicrosoftEdgeWebView2RuntimeInstallerX64.exe";

#[link(name = "wintrust")]
extern "system" {
    fn WinVerifyTrust(hwnd: isize, acao: *const Guid, dados: *mut DadosDeConfianca) -> i32;
    fn WTHelperProvDataFromStateData(estado: isize) -> *mut std::ffi::c_void;
    fn WTHelperGetProvSignerFromChain(prov: *mut std::ffi::c_void, signatario: u32, contra: i32, idx: u32) -> *const Signatario;
}
#[link(name = "crypt32")]
extern "system" {
    fn CertGetNameStringW(cert: *const std::ffi::c_void, tipo: u32, flags: u32, para: *const std::ffi::c_void,
        nome: *mut u16, tam: u32) -> u32;
}
#[repr(C)]
struct InfoDoArquivo { tam: u32, caminho: *const u16, arquivo: isize, sujeito: *const Guid }
#[repr(C)]
struct DadosDeConfianca {
    tam: u32, politica: *const u8, sip: *const u8, interface: u32, revogacao: u32, escolha: u32,
    arquivo: *const InfoDoArquivo, acao: u32, estado: isize, url: *const u16, flags: u32, contexto: u32, assinatura: *const u8,
}
#[repr(C)]
struct Signatario { tam: u32, quando: [u32; 2], qtd_certificados: u32, certificados: *const CertificadoDoProvedor }
#[repr(C)]
struct CertificadoDoProvedor { tam: u32, cert: *const std::ffi::c_void }

// assinatura válida (conferida sem internet) e emitida para a Microsoft
pub fn assinado_pela_microsoft(arquivo: &std::path::Path) -> bool {
    let acao = Guid(0x00AAC56B, 0xCD44, 0x11D0, [0x8C, 0xC2, 0x00, 0xC0, 0x4F, 0xC2, 0x95, 0xEE]);  // GENERIC_VERIFY_V2
    let caminho = largo(&arquivo.display().to_string());
    let info = InfoDoArquivo { tam: std::mem::size_of::<InfoDoArquivo>() as u32, caminho: caminho.as_ptr(), arquivo: 0,
        sujeito: std::ptr::null() };
    let mut dados = DadosDeConfianca {
        tam: std::mem::size_of::<DadosDeConfianca>() as u32, politica: std::ptr::null(), sip: std::ptr::null(),
        interface: 2, revogacao: 0, escolha: 1, arquivo: &info, acao: 1, estado: 0, url: std::ptr::null(),
        flags: 0x1000, contexto: 0, assinatura: std::ptr::null(),   // sem tela, sem revogação, só o cache (sem rede)
    };
    let valida = unsafe { WinVerifyTrust(-1, &acao, &mut dados) } == 0;
    let mut da_microsoft = false;
    if valida {
        unsafe {
            let sig = WTHelperGetProvSignerFromChain(WTHelperProvDataFromStateData(dados.estado), 0, 0, 0);
            if !sig.is_null() && (*sig).qtd_certificados > 0 && !(*sig).certificados.is_null() {
                let mut nome = [0u16; 128];
                let n = CertGetNameStringW((*(*sig).certificados).cert, 4, 0, std::ptr::null(), nome.as_mut_ptr(), 128);
                da_microsoft = String::from_utf16_lossy(&nome[..(n as usize).saturating_sub(1)]) == "Microsoft Corporation";
            }
        }
    }
    dados.acao = 2;   // fecha
    unsafe { WinVerifyTrust(-1, &acao, &mut dados) };
    da_microsoft
}

// sem WebView2: se o instalador da Microsoft estiver em `pasta`, instala (perguntando antes, fora do modo silencioso).
// `true`: agora tem.
pub fn instalar_webview2_de(pasta: &std::path::Path, silencio: bool) -> bool {
    let instalador = pasta.join(WEBVIEW2_COMPLETO);
    if !instalador.is_file() || !assinado_pela_microsoft(&instalador) { return false; }
    let texto = "O Giz na Tela precisa do WebView2 da Microsoft, que desenha a barra e a tinta, e ele não está neste \
        computador.\n\nO instalador oficial da Microsoft veio junto com o Giz na Tela e não precisa de internet. \
        Instalar o WebView2 agora? Leva um ou dois minutos.";
    if !silencio && unsafe { MessageBoxW(0, largo(texto).as_ptr(), largo("Giz na Tela").as_ptr(), 0x4 | 0x20) } != 6 {
        return false;
    }
    let _ = std::process::Command::new(&instalador).args(["/silent", "/install"]).status();
    tem_webview2()
}

// sem WebView2 nenhuma janela abre: explica o que fazer e oferece a página oficial de download
pub fn avisar_sem_webview2() {
    let texto = "O Giz na Tela precisa do WebView2 da Microsoft, que desenha a barra e a tinta. Ele vem no Windows 11 \
        e nos Windows 10 atualizados, mas não foi encontrado neste computador.\n\nPara resolver: baixe o \
        \"WebView2 Runtime\" no site da Microsoft. Sem internet, use o pacote \"para escola sem internet\" do Giz \
        na Tela, que já traz o instalador dele.\n\nAbrir a página de download agora?";
    let r = unsafe { MessageBoxW(0, largo(texto).as_ptr(), largo("Giz na Tela").as_ptr(), 0x4 | 0x30) };  // Sim/Não, alerta
    if r == 6 {
        let url = largo("https://developer.microsoft.com/microsoft-edge/webview2/");
        unsafe { ShellExecuteW(0, largo("open").as_ptr(), url.as_ptr(), std::ptr::null(), std::ptr::null(), 1) };
    }
}

// O WebView2 abre sozinho uma conexão com a Microsoft quando o perfil do programa já está registrado no arquivo
// EBWebView\Local State (a entrada profile.info_cache.Default), como achou o pesquisador da gestão em 29/10/2026
// (_Ferramentas/programas-livres/WEBVIEW2-REDE.md). Apagar só esse arquivo antes de abrir as janelas evita a conexão,
// e o WebView2 o recria. O programa não guarda nada no WebView2 (nem localStorage nem cookies), então nada se perde.
// Só esse arquivo, só no perfil do próprio programa, e só se nenhum WebView2 o estiver usando: com outra cópia aberta
// (outra pasta de dados), o processo dela é reaproveitado e regrava o arquivo.
// a pasta do perfil do WebView2 deste programa (o WEBVIEW2_USER_DATA_FOLDER a troca; os testes usam um de rascunho)
fn perfil_do_webview2() -> Option<std::path::PathBuf> {
    let raiz = match std::env::var_os("WEBVIEW2_USER_DATA_FOLDER").filter(|v| !v.is_empty()) {
        Some(p) => std::path::PathBuf::from(p),
        None => std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("br.gizlivre.giznatela"),
    };
    Some(raiz.join("EBWebView"))
}

#[allow(dead_code)]   // fora da variante sem esta feature (Cargo.toml)
pub fn esquecer_perfil_do_webview2() {
    use std::os::windows::fs::OpenOptionsExt;
    let Some(perfil) = perfil_do_webview2() else { return };
    // o WebView2 aberto segura o lockfile sem compartilhar: se não dá para abri-lo, o perfil está em uso
    let trava = perfil.join("lockfile");
    if trava.exists() && std::fs::OpenOptions::new().read(true).share_mode(0).open(&trava).is_err() {
        return;
    }
    let _ = std::fs::remove_file(perfil.join("Local State"));
}

// ---- WebView2 que sobrou de uma queda ----
// Se o Giz na Tela cai (ou é fechado à força), o processo principal do WebView2 dele continua vivo, segurando o perfil,
// e a próxima abertura falha ao criar a janela (0x80070578 ou 0xc0000409). Ao abrir, já com a trava de instância
// única, fecha só os msedgewebview2.exe principais (sem --type) cujo --user-data-dir é exatamente o perfil deste
// programa e cujo programa-pai já não existe; cada um fica anotado no log. (Achado da gestão e do Bancada, 29/10/2026.)
#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
    fn Process32FirstW(foto: isize, p: *mut ProcessoTh) -> i32;
    fn Process32NextW(foto: isize, p: *mut ProcessoTh) -> i32;
    fn TerminateProcess(h: isize, codigo: u32) -> i32;
    fn WaitForSingleObject(h: isize, ms: u32) -> u32;
}
#[link(name = "ntdll")]
extern "system" {
    fn NtQueryInformationProcess(h: isize, classe: u32, dados: *mut u8, tam: u32, volta: *mut u32) -> i32;
}
#[repr(C)]
struct ProcessoTh { tam: u32, uso: u32, pid: u32, heap: usize, modulo: u32, threads: u32, pai: u32, prioridade: i32,
    flags: u32, exe: [u16; 260] }

// linha de comando de outro processo da conta (ProcessCommandLineInformation, Windows 8.1 em diante), pelo handle já
// aberto: o mesmo handle que depois fecha o processo, então um PID reaproveitado não troca de processo no meio
fn linha_de_comando(h: isize) -> Option<String> {
    unsafe {
        let mut buf = vec![0u64; 8192];   // 64 KB alinhados
        let mut volta = 0u32;
        let r = NtQueryInformationProcess(h, 60, buf.as_mut_ptr() as *mut u8, (buf.len() * 8) as u32, &mut volta);
        if r != 0 { return None; }
        // UNICODE_STRING { Length: u16, MaximumLength: u16, Buffer: *const u16 } no começo do buffer
        let base = buf.as_ptr() as *const u8;
        let tam = *(base as *const u16) as usize / 2;
        let texto = *(base.add(8) as *const *const u16);
        if texto.is_null() { return None; }
        Some(String::from_utf16_lossy(std::slice::from_raw_parts(texto, tam)))
    }
}

// separa a linha de comando em argumentos com as regras do Windows (as do CommandLineToArgvW): espaço separa fora das
// aspas; 2n barras antes de aspas viram n barras e as aspas abrem ou fecham; 2n+1 barras viram n barras e uma aspa;
// barras sem aspas depois ficam como estão
fn argumentos(linha: &str) -> Vec<String> {
    let (mut todos, mut atual, mut aspas, mut tem) = (Vec::new(), String::new(), false, false);
    let mut c = linha.chars().peekable();
    while let Some(ch) = c.next() {
        match ch {
            '\\' => {
                let mut barras = 1;
                while c.peek() == Some(&'\\') { c.next(); barras += 1; }
                if c.peek() == Some(&'"') {
                    atual.extend(std::iter::repeat_n('\\', barras / 2));
                    if barras % 2 == 1 { c.next(); atual.push('"'); }
                } else {
                    atual.extend(std::iter::repeat_n('\\', barras));
                }
                tem = true;
            }
            '"' => {
                if aspas && c.peek() == Some(&'"') { c.next(); atual.push('"'); } else { aspas = !aspas; }
                tem = true;
            }
            ' ' | '\t' if !aspas => {
                if tem { todos.push(std::mem::take(&mut atual)); tem = false; }
            }
            _ => { atual.push(ch); tem = true; }
        }
    }
    if tem { todos.push(atual); }
    todos
}

// --user-data-dir="C:\...\EBWebView" (com ou sem aspas), só como argumento próprio: o texto dentro do valor de outro
// argumento não conta, e duas chaves próprias são ambíguas (nenhuma vale)
fn pasta_do_perfil_na_linha(linha: &str) -> Option<String> {
    let mut achados = argumentos(linha).into_iter().skip(1).filter_map(|a| a.strip_prefix("--user-data-dir=").map(str::to_owned));
    let valor = achados.next()?;
    if achados.next().is_some() { return None; }
    Some(valor.trim_end_matches('\\').to_lowercase())
}

pub fn fechar_webview2_orfao(dados: &std::path::Path) {
    let Some(perfil) = perfil_do_webview2() else { return };
    let perfil = perfil.display().to_string().trim_end_matches('\\').to_lowercase();
    let mut fechados = Vec::new();
    unsafe {
        let foto = CreateToolhelp32Snapshot(2, 0);   // TH32CS_SNAPPROCESS
        if foto == -1 { return; }
        let mut p: ProcessoTh = std::mem::zeroed();
        p.tam = std::mem::size_of::<ProcessoTh>() as u32;
        let mut todos = Vec::new();
        let mut ok = Process32FirstW(foto, &mut p);
        while ok != 0 {
            let n = p.exe.iter().position(|&c| c == 0).unwrap_or(260);
            todos.push((p.pid, p.pai, String::from_utf16_lossy(&p.exe[..n])));
            ok = Process32NextW(foto, &mut p);
        }
        CloseHandle(foto);
        for (pid, pai, exe) in &todos {
            if !exe.eq_ignore_ascii_case("msedgewebview2.exe") || processo_vivo(*pai) { continue; }
            // um handle só para conferir e fechar: o processo conferido é o mesmo que fecha
            let h = OpenProcess(0x0010_1001, 0, *pid);   // PROCESS_TERMINATE | SYNCHRONIZE | QUERY_LIMITED_INFORMATION
            if h == 0 { continue; }
            let certo = linha_de_comando(h).is_some_and(|linha| {
                let args = argumentos(&linha);
                // o programa é o WebView2 e é o principal (sem --type: em dúvida, como o texto em qualquer lugar,
                // fica de fora), com o perfil exatamente o deste programa
                args.first().is_some_and(|a| a.to_lowercase().ends_with("msedgewebview2.exe"))
                    && !linha.contains("--type=")
                    && pasta_do_perfil_na_linha(&linha).as_deref() == Some(perfil.as_str())
            });
            // só conta como fechado se o TerminateProcess deu certo e o processo saiu mesmo (até 3 s)
            if certo && TerminateProcess(h, 1) != 0 && WaitForSingleObject(h, 3000) == 0 { fechados.push(*pid); }
            CloseHandle(h);
        }
    }
    if fechados.is_empty() { return; }
    std::thread::sleep(Duration::from_millis(500));   // o processo solta o perfil
    let t = agora();
    let linha = format!("{}-{:02}-{:02} {:02}:{:02}:{:02} versão {}: WebView2 que sobrou de uma queda fechado (pid {:?})\n",
        t.ano, t.mes, t.dia, t.hora, t.minuto, t.segundo, crate::VERSAO, fechados);
    let _ = std::fs::create_dir_all(dados);
    let _ = std::fs::OpenOptions::new().create(true).append(true).open(dados.join("janelas.log"))
        .and_then(|mut f| f.write_all(linha.as_bytes()));
}

pub fn avisar_erro(msg: &str) {
    unsafe { MessageBoxW(0, largo(msg).as_ptr(), largo("Giz na Tela").as_ptr(), 0x10) };
}
#[link(name = "kernel32")]
extern "system" {
    fn GetLocalTime(t: *mut Agora);
    fn CreateMutexW(atributos: *const u8, dono: i32, nome: *const u16) -> isize;
    fn GetLastError() -> u32;
    fn CloseHandle(h: isize) -> i32;
    fn OpenProcess(acesso: u32, herdar: i32, pid: u32) -> isize;
    fn GetExitCodeProcess(h: isize, codigo: *mut u32) -> i32;
}
#[link(name = "bcrypt")]
extern "system" {
    fn BCryptGenRandom(alg: isize, buf: *mut u8, tam: u32, flags: u32) -> i32;
    fn BCryptHash(alg: isize, segredo: *const u8, tam_segredo: u32, dados: *const u8, tam: u32, saida: *mut u8,
        tam_saida: u32) -> i32;
}

// SHA-256 pelo próprio Windows (Windows 10 ou mais novo), para conferir o instalador baixado com o publicado
pub fn sha256_hex(dados: &[u8]) -> Result<String, String> {
    let mut h = [0u8; 32];
    let r = unsafe { BCryptHash(0x41, std::ptr::null(), 0, dados.as_ptr(), dados.len() as u32, h.as_mut_ptr(), 32) };  // BCRYPT_SHA256_ALG_HANDLE
    if r != 0 { return Err("o Windows não calculou o SHA-256".into()); }
    Ok(h.iter().map(|x| format!("{x:02x}")).collect())
}

// tira (ou devolve) a janela das capturas de tela do Windows: WDA_EXCLUDEFROMCAPTURE. Só existe do Windows 10
// versão 2004 em diante: `false` quando o Windows recusa (quem chama esconde a janela durante o retrato)
pub fn fora_da_captura(hwnd: isize, sim: bool) -> bool {
    unsafe { SetWindowDisplayAffinity(hwnd, if sim { 0x11 } else { 0 }) != 0 }
}

pub fn esconder_janela(hwnd: isize, sim: bool) {
    unsafe { ShowWindow(hwnd, if sim { 0 } else { 8 }) };   // SW_HIDE, SW_SHOWNA (sem roubar o foco)
}

#[repr(C)]
#[derive(Default)]
pub struct Agora { pub ano: u16, pub mes: u16, pub dia_semana: u16, pub dia: u16, pub hora: u16, pub minuto: u16, pub segundo: u16, pub ms: u16 }
pub fn agora() -> Agora {
    let mut t = Agora::default();
    unsafe { GetLocalTime(&mut t) };
    t
}

// token dos agentes: 32 bytes do gerador do sistema, em hexadecimal
pub fn token_novo() -> std::io::Result<String> {
    let mut b = [0u8; 32];
    let r = unsafe { BCryptGenRandom(0, b.as_mut_ptr(), b.len() as u32, 2) };  // BCRYPT_USE_SYSTEM_PREFERRED_RNG
    if r != 0 { return Err(std::io::Error::other("o gerador aleatório do Windows falhou")); }
    Ok(b.iter().map(|x| format!("{x:02x}")).collect())
}

// comparação que não para no primeiro byte diferente (não entrega o token pelo tempo de resposta)
pub fn iguais(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

// um anotador por pasta de dados: abrir de novo não cria uma segunda camada por cima da primeira
// `espera`: reaberto pelo instalador depois de uma atualização, o programa antigo pode ainda estar terminando de
// fechar; espera a trava dele soltar em vez de desistir na hora
pub fn ja_aberto(chave: &str, espera: Duration) -> bool {
    let h = chave.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3));
    let nome: Vec<u16> = format!("Local\\GizNaTela-{h:016x}").encode_utf16().chain([0]).collect();
    let limite = std::time::Instant::now() + espera;
    loop {
        unsafe {
            let m = CreateMutexW(std::ptr::null(), 0, nome.as_ptr());
            if m == 0 || GetLastError() != 183 { return false; }  // a trava é nossa e fica aberta até o programa sair
            CloseHandle(m);   // ERROR_ALREADY_EXISTS: solta a nossa cópia da trava, senão ela nunca some
        }
        if std::time::Instant::now() >= limite { return true; }
        std::thread::sleep(Duration::from_millis(500));
    }
}

// o processo ainda existe (de qualquer conta): serve para descartar o arquivo de conexão que sobrou de um programa
// fechado à força
pub fn processo_vivo(pid: u32) -> bool {
    unsafe {
        let h = OpenProcess(0x1000, 0, pid);   // PROCESS_QUERY_LIMITED_INFORMATION
        if h == 0 { return vivo_pela_consulta(None, GetLastError()); }
        let mut codigo = 0u32;
        let consultou = GetExitCodeProcess(h, &mut codigo) != 0;
        CloseHandle(h);
        vivo_pela_consulta(Some(consultou.then_some(codigo)), 0)
    }
}

// aberto: None se o OpenProcess falhou (com o erro dele); Some(None) se abriu mas o código de saída não veio
fn vivo_pela_consulta(aberto: Option<Option<u32>>, erro: u32) -> bool {
    match aberto {
        None => erro != 87,   // ERROR_INVALID_PARAMETER: não existe; acesso negado (5) e o resto: na dúvida, vivo
        Some(None) => true,   // abriu, mas a consulta falhou: na dúvida, vivo
        Some(Some(codigo)) => codigo == 259,   // STILL_ACTIVE
    }
}

// HTTP/1.1 em 127.0.0.1, com Connection: close (o Giz Livre e a API daqui fecham depois de responder)
pub fn http(porta: u16, metodo: &str, rota: &str, cabecalhos: &[(&str, &str)], tipo: &str, corpo: &[u8], espera: Duration)
    -> Result<(u16, Vec<u8>), String> {
    let end = format!("127.0.0.1:{porta}").parse().map_err(|_| "porta inválida".to_string())?;
    let mut c = TcpStream::connect_timeout(&end, Duration::from_millis(800)).map_err(|_| "fechado".to_string())?;
    let _ = c.set_read_timeout(Some(espera));
    let mut cab = format!("{metodo} {rota} HTTP/1.1\r\nHost: 127.0.0.1:{porta}\r\nContent-Type: {tipo}\r\n\
        Content-Length: {}\r\nConnection: close\r\n", corpo.len());
    for (k, v) in cabecalhos {
        cab += &format!("{k}: {v}\r\n");
    }
    cab += "\r\n";
    c.write_all(cab.as_bytes()).and_then(|_| c.write_all(corpo)).map_err(|e| e.to_string())?;
    let mut r = Vec::new();
    c.read_to_end(&mut r).map_err(|e| e.to_string())?;
    let fim = r.windows(4).position(|w| w == b"\r\n\r\n").ok_or("resposta incompleta")?;
    let status = String::from_utf8_lossy(&r[..fim]).split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    Ok((status, r[fim + 4..].to_vec()))
}

// ---- botão na barra de tarefas ----
// Só a barra de ferramentas ganha botão na barra de tarefas (a camada transparente não). O programa usa o mesmo
// AppUserModelID dos atalhos do instalador ("GizNaTela"): ao fixar, o Windows guarda o Giz na Tela, com o ícone dele. A
// janela leva também o comando para reabrir, para a cópia portátil (que não tem atalho) também poder ser fixada.
pub const APP_ID: &str = "GizNaTela";

#[link(name = "shell32")]
extern "system" {
    fn SetCurrentProcessExplicitAppUserModelID(id: *const u16) -> i32;
    fn SHGetPropertyStoreForWindow(hwnd: isize, iid: *const Guid, saida: *mut *mut *const usize) -> i32;
}
#[link(name = "user32")]
extern "system" {
    fn GetWindowLongPtrW(hwnd: isize, indice: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: isize, indice: i32, valor: isize) -> isize;
    fn ShowWindow(hwnd: isize, modo: i32) -> i32;
}
#[repr(C)]
struct Guid(u32, u16, u16, [u8; 8]);
#[repr(C)]
struct ChaveDePropriedade { fmtid: Guid, pid: u32 }
#[repr(C)]
struct Propvariant { vt: u16, r: [u16; 3], texto: *const u16, resto: usize }

pub fn marcar_programa() {
    let id = largo(APP_ID);
    unsafe { SetCurrentProcessExplicitAppUserModelID(id.as_ptr()); }
}

// tudo é opcional: se o Windows recusar, o programa segue normal (só não aparece ou não fixa na barra de tarefas)
pub fn botao_na_barra_de_tarefas(hwnd: isize) {
    const GWL_EXSTYLE: i32 = -20;
    const WS_EX_APPWINDOW: isize = 0x40000;    // janela com dona (a barra pertence à camada) só ganha botão com isto
    unsafe {
        let estilo = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, estilo | WS_EX_APPWINDOW);
        // a barra de tarefas só lê o estilo quando a janela aparece: esconde e mostra de novo, sem tirar o foco
        ShowWindow(hwnd, 0);   // SW_HIDE
        ShowWindow(hwnd, 8);   // SW_SHOWNA
    }
    let exe = std::env::current_exe().map(|e| e.display().to_string()).unwrap_or_default();
    if exe.is_empty() || exe.len() > 250 { return; }
    let iid = Guid(0x886D8EEB, 0x8CF2, 0x4446, [0x8D, 0x02, 0xCD, 0xBA, 0x1D, 0xBD, 0xCF, 0x99]);   // IPropertyStore
    let mut loja: *mut *const usize = std::ptr::null_mut();
    if unsafe { SHGetPropertyStoreForWindow(hwnd, &iid, &mut loja) } != 0 || loja.is_null() { return; }
    type Definir = unsafe extern "system" fn(*mut *const usize, *const ChaveDePropriedade, *const Propvariant) -> i32;
    type Simples = unsafe extern "system" fn(*mut *const usize) -> i32;
    unsafe {
        let tabela = *loja;
        let (definir, gravar, soltar): (Definir, Simples, Simples) =
            (std::mem::transmute(*tabela.add(6)), std::mem::transmute(*tabela.add(7)), std::mem::transmute(*tabela.add(2)));
        // PKEY_AppUserModel_ID (5), RelaunchCommand (2), RelaunchIconResource (3), RelaunchDisplayNameResource (4)
        for (pid, valor) in [(2, format!("\"{exe}\"")), (3, format!("{exe},0")), (4, "Giz na Tela".to_string()),
                             (5, APP_ID.to_string())] {
            let texto = largo(&valor);
            let chave = ChaveDePropriedade { fmtid: Guid(0x9F4C2855, 0x9F79, 0x4B39, [0xA8, 0xD0, 0xE1, 0xD4, 0x2D, 0xE1, 0xD5, 0xF3]), pid };
            let v = Propvariant { vt: 31, r: [0; 3], texto: texto.as_ptr(), resto: 0 };   // VT_LPWSTR
            definir(loja, &chave, &v);
        }
        gravar(loja);
        soltar(loja);
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    // o Edge vem no Windows assinado pela Microsoft; este próprio teste não é assinado
    #[test]
    fn le_a_pasta_do_perfil_na_linha() {
        let l = r#""C:\Program Files (x86)\Microsoft\EdgeWebView\Application\1\msedgewebview2.exe" --embedded-browser-webview=1 --user-data-dir="C:\Users\x\AppData\Local\br.gizlivre.giznatela\EBWebView" --noerrdialogs"#;
        assert_eq!(pasta_do_perfil_na_linha(l).as_deref(), Some(r"c:\users\x\appdata\local\br.gizlivre.giznatela\ebwebview"));
        assert_eq!(pasta_do_perfil_na_linha("x --user-data-dir=C:\\a\\EBWebView --y").as_deref(), Some(r"c:\a\ebwebview"));
        assert_eq!(pasta_do_perfil_na_linha("sem perfil"), None);
    }
    // a chave tem de ser um argumento próprio, nunca um pedaço do valor de outro (GNT-RC3-01, auditoria de 30/10/2026)
    #[test]
    fn parser_nao_confunde_valor_de_outro_argumento_com_perfil() {
        let l = r"msedgewebview2.exe --audit-note=--user-data-dir=C:\alvo\EBWebView --user-data-dir=C:\outro\EBWebView";
        assert_eq!(pasta_do_perfil_na_linha(l).as_deref(), Some(r"c:\outro\ebwebview"));
        // dentro das aspas de outro argumento também não vale
        let l = r#"msedgewebview2.exe "--nota=x --user-data-dir=C:\alvo\EBWebView" --user-data-dir="C:\outro\EBWebView""#;
        assert_eq!(pasta_do_perfil_na_linha(l).as_deref(), Some(r"c:\outro\ebwebview"));
        // duas chaves próprias: ambíguo, não escolhe nenhuma
        let l = r"msedgewebview2.exe --user-data-dir=C:\a\EBWebView --user-data-dir=C:\b\EBWebView";
        assert_eq!(pasta_do_perfil_na_linha(l), None);
        // só a chave dentro de outro valor: não há perfil
        assert_eq!(pasta_do_perfil_na_linha(r"msedgewebview2.exe --nota=--user-data-dir=C:\a\EBWebView"), None);
    }
    // as regras de separação do Windows (as do CommandLineToArgvW): aspas, barras antes de aspas e aspas escapadas
    #[test]
    fn separa_os_argumentos_como_o_windows() {
        assert_eq!(argumentos(r#"a "b c" d"#), ["a", "b c", "d"]);
        assert_eq!(argumentos(r#"a\\"b c" d"#), [r"a\b c", "d"]);
        assert_eq!(argumentos(r#""a\"b" c"#), [r#"a"b"#, "c"]);
        assert_eq!(argumentos(r"C:\a\b\ x"), [r"C:\a\b\", "x"]);
        assert_eq!(argumentos("  a   b  "), ["a", "b"]);
    }
    // só "não existe" (87) ou o código de saída de quem saiu conta como pai morto; qualquer falha da consulta conta
    // como vivo, senão o WebView2 de um programa aberto seria fechado (auditoria do modelo nativo, 29/10/2026)
    #[test]
    fn consulta_que_falha_nao_e_pai_morto() {
        assert!(!vivo_pela_consulta(None, 87));
        assert!(vivo_pela_consulta(None, 5));
        assert!(vivo_pela_consulta(None, 6), "outra falha do OpenProcess");
        assert!(vivo_pela_consulta(Some(None), 0), "o GetExitCodeProcess falhou");
        assert!(vivo_pela_consulta(Some(Some(259)), 0));
        assert!(!vivo_pela_consulta(Some(Some(0)), 0));
    }
    #[test]
    fn so_aceita_assinatura_da_microsoft() {
        let edge = std::path::Path::new(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe");
        if edge.exists() { assert!(assinado_pela_microsoft(edge)); }
        let wv2 = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("installer").join(WEBVIEW2_COMPLETO);
        if wv2.exists() { assert!(assinado_pela_microsoft(&wv2)); }
        assert!(!assinado_pela_microsoft(&std::env::current_exe().unwrap()));
        assert!(!assinado_pela_microsoft(std::path::Path::new(r"C:\nao-existe\nada.exe")));
    }
}
