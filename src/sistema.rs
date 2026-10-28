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

// sem WebView2 nenhuma janela abre: explica o que fazer e oferece a página oficial de download
pub fn avisar_sem_webview2() {
    let texto = "O Giz na Tela precisa do WebView2 da Microsoft, que desenha a barra e a tinta. Ele vem no Windows 11 \
        e nos Windows 10 atualizados, mas não foi encontrado neste computador.\n\nPara resolver: use o instalador do \
        Giz na Tela (ele instala o WebView2 junto) ou baixe o \"WebView2 Runtime\" no site da Microsoft.\n\n\
        Abrir a página de download agora?";
    let r = unsafe { MessageBoxW(0, largo(texto).as_ptr(), largo("Giz na Tela").as_ptr(), 0x4 | 0x30) };  // Sim/Não, alerta
    if r == 6 {
        let url = largo("https://developer.microsoft.com/microsoft-edge/webview2/");
        unsafe { ShellExecuteW(0, largo("open").as_ptr(), url.as_ptr(), std::ptr::null(), std::ptr::null(), 1) };
    }
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

// tira (ou devolve) a janela das capturas de tela do Windows: WDA_EXCLUDEFROMCAPTURE
pub fn fora_da_captura(hwnd: isize, sim: bool) {
    unsafe { SetWindowDisplayAffinity(hwnd, if sim { 0x11 } else { 0 }) };
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
