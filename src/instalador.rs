// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). O próprio programa se instala, sem Inno, NSIS nem MSI
// Estudo em docs/P-INSTALADOR-PROPRIO.md: o GizNaTela.exe puro deu zero alertas no VirusTotal, e os instaladores não.
// Baixado com o nome Instalar-GizNaTela-<versão>.exe (ou aberto com --instalar), ele se copia para
// %LOCALAPPDATA%\Programs\Giz na Tela (a mesma pasta do Inno e do NSIS), cria os atalhos com o AppUserModelID e
// registra a desinstalação na chave da conta. Nada pede administrador e nada vale para as outras contas.
// /S, /SILENT e /VERYSILENT instalam sem perguntar; /reabrir=1 abre o programa no fim (é o que o Sobre das versões
// 1.3.x manda ao instalador baixado). Os ajustes e as imagens salvas nunca são tocados.
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const CHAVE: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\GizNaTela";
// instalações antigas, tiradas antes de copiar: Inno Setup (até a 1.3.2) e NSIS (só nos candidatos de teste)
const ANTIGAS: [(&str, &[&str]); 2] = [
    (r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{4BB282B2-1BF2-444F-BCB1-0F6E51B46035}_is1",
        &["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"]),
    (r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Giz na Tela", &["/S"]),
];
const EXE: &str = "GizNaTela.exe";
// variante com o instalador num exe próprio: ele fica na pasta com este nome e é quem desinstala
const DESINSTALADOR: &str = "Desinstalar-GizNaTela.exe";
const SEPARADO: bool = cfg!(feature = "instalador-separado");
// os textos que acompanham o programa vão dentro do exe (MIT e Apache pedem o aviso junto do programa)
#[cfg(feature = "avisos-dentro")]
const TEXTOS: [(&str, &[u8]); 3] = [
    ("LICENSE", include_bytes!("../LICENSE")),
    ("THIRD-PARTY-NOTICES.md", include_bytes!("../THIRD-PARTY-NOTICES.md")),
    ("README.md", include_bytes!("../README.md")),
];
// variante da rodada 5: sem os 2 MB de avisos de terceiros dentro do exe
#[cfg(not(feature = "avisos-dentro"))]
const TEXTOS: [(&str, &[u8]); 2] = [("LICENSE", include_bytes!("../LICENSE")), ("README.md", include_bytes!("../README.md"))];
const NOMES_DOS_TEXTOS: [&str; 3] = ["LICENSE", "THIRD-PARTY-NOTICES.md", "README.md"];
const ATALHO: &str = "Giz na Tela.lnk";
const ATALHO_IMAGENS: &str = "Giz na Tela - imagens salvas.lnk";
const ATALHO_DESINSTALAR: &str = "Desinstalar o Giz na Tela.lnk";

const HKCU: isize = 0x80000001u32 as i32 as isize;

#[link(name = "advapi32")]
extern "system" {
    fn RegGetValueW(chave: isize, sub: *const u16, valor: *const u16, flags: u32, tipo: *mut u32, dados: *mut u16,
        tam: *mut u32) -> i32;
    fn RegCreateKeyExW(chave: isize, sub: *const u16, r: u32, classe: *const u16, opcoes: u32, acesso: u32,
        seg: *const u8, saida: *mut isize, disp: *mut u32) -> i32;
    fn RegOpenKeyExW(chave: isize, sub: *const u16, opcoes: u32, acesso: u32, saida: *mut isize) -> i32;
    fn RegSetValueExW(chave: isize, nome: *const u16, r: u32, tipo: u32, dados: *const u8, tam: u32) -> i32;
    fn RegDeleteTreeW(chave: isize, sub: *const u16) -> i32;
    fn RegCloseKey(chave: isize) -> i32;
}
#[link(name = "shell32")]
extern "system" {
    fn SHGetKnownFolderPath(id: *const Guid, flags: u32, token: isize, saida: *mut *mut u16) -> i32;
}
#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(r: *const u8, modo: u32) -> i32;
    fn CoCreateInstance(clsid: *const Guid, fora: *const u8, contexto: u32, iid: *const Guid, saida: *mut *mut *const usize) -> i32;
    fn CoTaskMemFree(p: *mut u16);
}
#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
    fn Process32FirstW(foto: isize, p: *mut Processo) -> i32;
    fn Process32NextW(foto: isize, p: *mut Processo) -> i32;
    fn OpenProcess(acesso: u32, herdar: i32, pid: u32) -> isize;
    fn QueryFullProcessImageNameW(h: isize, flags: u32, nome: *mut u16, tam: *mut u32) -> i32;
    fn CloseHandle(h: isize) -> i32;
}
#[link(name = "user32")]
extern "system" {
    fn MessageBoxW(hwnd: isize, texto: *const u16, titulo: *const u16, tipo: u32) -> i32;
    fn EnumWindows(cada: unsafe extern "system" fn(isize, isize) -> i32, dados: isize) -> i32;
    fn GetWindowThreadProcessId(hwnd: isize, pid: *mut u32) -> u32;
    fn IsWindowVisible(hwnd: isize) -> i32;
    fn GetWindowTextW(hwnd: isize, texto: *mut u16, tam: i32) -> i32;
    fn PostMessageW(hwnd: isize, msg: u32, w: usize, l: isize) -> i32;
}

#[repr(C)]
struct Guid(u32, u16, u16, [u8; 8]);
#[repr(C)]
struct ChaveDePropriedade { fmtid: Guid, pid: u32 }
#[repr(C)]
struct Propvariant { vt: u16, r: [u16; 3], texto: *const u16, resto: usize }
#[repr(C)]
struct Processo { tam: u32, uso: u32, pid: u32, heap: usize, modulo: u32, threads: u32, pai: u32, prioridade: i32,
    flags: u32, exe: [u16; 260] }

fn largo(t: &str) -> Vec<u16> { t.encode_utf16().chain([0]).collect() }
fn largo_p(p: &Path) -> Vec<u16> { largo(&p.display().to_string()) }

fn perguntar(texto: &str, tipo: u32) -> i32 {
    unsafe { MessageBoxW(0, largo(texto).as_ptr(), largo("Giz na Tela").as_ptr(), tipo) }
}

pub fn pasta() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("Programs").join("Giz na Tela"))
}

fn pasta_conhecida(id: Guid) -> Option<PathBuf> {
    let mut p: *mut u16 = std::ptr::null_mut();
    unsafe {
        let r = SHGetKnownFolderPath(&id, 0, 0, &mut p);
        let caminho = (r == 0 && !p.is_null()).then(|| {
            let n = (0..).take_while(|&i| *p.add(i) != 0).count();
            PathBuf::from(String::from_utf16_lossy(std::slice::from_raw_parts(p, n)))
        });
        CoTaskMemFree(p);
        caminho
    }
}
fn menu_iniciar() -> Option<PathBuf> {
    pasta_conhecida(Guid(0xA77F5D77, 0x2E2B, 0x44C3, [0xA6, 0xA2, 0xAB, 0xA6, 0x01, 0x05, 0x4A, 0x51]))
}
fn area_de_trabalho() -> Option<PathBuf> {
    pasta_conhecida(Guid(0xB4BFCC3A, 0xDB2C, 0x424C, [0xB0, 0x29, 0x7F, 0xE9, 0x9A, 0x87, 0xC6, 0x41]))
}
fn documentos() -> Option<PathBuf> {
    pasta_conhecida(Guid(0xFDD39AD0, 0x238F, 0x46AF, [0xAD, 0xB4, 0x6C, 0x85, 0x48, 0x03, 0x69, 0xC7]))
}

// ---- registro da conta ----
fn ler_texto(sub: &str, valor: &str) -> Option<String> {
    let (sub, valor) = (largo(sub), largo(valor));
    let mut buf = vec![0u16; 2048];
    let mut tam = (buf.len() * 2) as u32;
    let r = unsafe { RegGetValueW(HKCU, sub.as_ptr(), valor.as_ptr(), 0x2, std::ptr::null_mut(), buf.as_mut_ptr(), &mut tam) };  // RRF_RT_REG_SZ
    (r == 0).then(|| String::from_utf16_lossy(&buf[..(tam as usize / 2).saturating_sub(1)]))
}

enum Valor { Texto(String), Numero(u32) }

fn gravar_valores(chave: isize, valores: &[(&str, Valor)]) -> Result<(), String> {
    for (nome, v) in valores {
        let n = largo(nome);
        let r = match v {
            Valor::Texto(t) => {
                let d = largo(t);
                unsafe { RegSetValueExW(chave, n.as_ptr(), 0, 1, d.as_ptr() as *const u8, (d.len() * 2) as u32) }   // REG_SZ
            }
            Valor::Numero(x) => unsafe { RegSetValueExW(chave, n.as_ptr(), 0, 4, x as *const u32 as *const u8, 4) },  // REG_DWORD
        };
        if r != 0 { return Err(format!("o Windows não gravou {nome} no registro (código {r})")); }
    }
    Ok(())
}

fn registrar(exe: &Path, desinstalador: &Path, pasta: &Path) -> Result<(), String> {
    let mut chave = 0isize;
    let r = unsafe { RegCreateKeyExW(HKCU, largo(CHAVE).as_ptr(), 0, std::ptr::null(), 0, 0x20006, std::ptr::null(),
        &mut chave, std::ptr::null_mut()) };   // KEY_WRITE
    if r != 0 { return Err(format!("o Windows não criou a chave de desinstalação (código {r})")); }
    let kb = std::fs::metadata(exe).map(|m| (m.len() / 1024) as u32).unwrap_or(0);
    let e = exe.display().to_string();
    let d = desinstalador.display().to_string();
    let res = gravar_valores(chave, &[
        ("DisplayName", Valor::Texto("Giz na Tela".into())),
        ("DisplayVersion", Valor::Texto(crate::VERSAO.into())),
        ("Publisher", Valor::Texto("Heraldo Antunes (projeto livre)".into())),
        ("DisplayIcon", Valor::Texto(format!("{e},0"))),
        ("InstallLocation", Valor::Texto(pasta.display().to_string())),
        ("UninstallString", Valor::Texto(format!("\"{d}\" --desinstalar"))),
        ("QuietUninstallString", Valor::Texto(format!("\"{d}\" --desinstalar /S"))),
        ("URLInfoAbout", Valor::Texto("https://github.com/HeraldoAntunes/giz-na-tela".into())),
        ("NoModify", Valor::Numero(1)),
        ("NoRepair", Valor::Numero(1)),
        ("EstimatedSize", Valor::Numero(kb)),
    ]);
    unsafe { RegCloseKey(chave) };
    res
}

// a cada abertura da cópia instalada: a lista de Aplicativos mostra a versão que está de fato aqui
pub fn conferir_instalacao() {
    // o .old que sobrou de uma atualização só sai com o programa no lugar (sem ele, o .old é a única cópia)
    if let Some(p) = pasta().filter(|p| p.join(EXE).exists()) {
        let _ = std::fs::remove_file(p.join("GizNaTela.old"));
    }
    let mut chave = 0isize;
    if unsafe { RegOpenKeyExW(HKCU, largo(CHAVE).as_ptr(), 0, 0x0002, &mut chave) } != 0 { return; }   // KEY_SET_VALUE
    let _ = gravar_valores(chave, &[("DisplayVersion", Valor::Texto(crate::VERSAO.into()))]);
    unsafe { RegCloseKey(chave) };
}

// ---- atalhos (IShellLinkW, com o AppUserModelID pelo IPropertyStore, gravado pelo IPersistFile) ----
unsafe fn metodo(obj: *mut *const usize, i: usize) -> usize { *(*obj).add(i) }

fn atalho(lnk: &Path, alvo: &Path, args: &str, descricao: &str, icone: Option<&Path>, com_id: bool) -> Result<(), String> {
    let clsid = Guid(0x00021401, 0, 0, [0xC0, 0, 0, 0, 0, 0, 0, 0x46]);         // CLSID_ShellLink
    let iid_link = Guid(0x000214F9, 0, 0, [0xC0, 0, 0, 0, 0, 0, 0, 0x46]);      // IShellLinkW
    let iid_arquivo = Guid(0x0000010B, 0, 0, [0xC0, 0, 0, 0, 0, 0, 0, 0x46]);   // IPersistFile
    let iid_loja = Guid(0x886D8EEB, 0x8CF2, 0x4446, [0x8D, 0x02, 0xCD, 0xBA, 0x1D, 0xBD, 0xCF, 0x99]);   // IPropertyStore
    type Texto = unsafe extern "system" fn(*mut *const usize, *const u16) -> i32;
    type Icone = unsafe extern "system" fn(*mut *const usize, *const u16, i32) -> i32;
    type Pedir = unsafe extern "system" fn(*mut *const usize, *const Guid, *mut *mut *const usize) -> i32;
    type Simples = unsafe extern "system" fn(*mut *const usize) -> i32;
    type Definir = unsafe extern "system" fn(*mut *const usize, *const ChaveDePropriedade, *const Propvariant) -> i32;
    type Salvar = unsafe extern "system" fn(*mut *const usize, *const u16, i32) -> i32;
    unsafe {
        let mut link: *mut *const usize = std::ptr::null_mut();
        if CoCreateInstance(&clsid, std::ptr::null(), 1, &iid_link, &mut link) != 0 || link.is_null() {
            return Err("o Windows não criou o atalho".into());
        }
        let soltar: Simples = std::mem::transmute(metodo(link, 2));
        let (alvo_l, pasta_l) = (largo_p(alvo), largo_p(alvo.parent().unwrap_or(alvo)));
        std::mem::transmute::<usize, Texto>(metodo(link, 20))(link, alvo_l.as_ptr());       // SetPath
        std::mem::transmute::<usize, Texto>(metodo(link, 9))(link, pasta_l.as_ptr());       // SetWorkingDirectory
        std::mem::transmute::<usize, Texto>(metodo(link, 7))(link, largo(descricao).as_ptr());   // SetDescription
        if !args.is_empty() {
            std::mem::transmute::<usize, Texto>(metodo(link, 11))(link, largo(args).as_ptr());   // SetArguments
        }
        if let Some(i) = icone {
            std::mem::transmute::<usize, Icone>(metodo(link, 17))(link, largo_p(i).as_ptr(), 0);   // SetIconLocation
        }
        let pedir: Pedir = std::mem::transmute(metodo(link, 0));
        if com_id {
            let mut loja: *mut *const usize = std::ptr::null_mut();
            if pedir(link, &iid_loja, &mut loja) == 0 && !loja.is_null() {
                let id = largo(crate::sistema::APP_ID);
                let chave = ChaveDePropriedade { fmtid: Guid(0x9F4C2855, 0x9F79, 0x4B39, [0xA8, 0xD0, 0xE1, 0xD4, 0x2D, 0xE1, 0xD5, 0xF3]), pid: 5 };
                let v = Propvariant { vt: 31, r: [0; 3], texto: id.as_ptr(), resto: 0 };   // VT_LPWSTR
                std::mem::transmute::<usize, Definir>(metodo(loja, 6))(loja, &chave, &v);
                std::mem::transmute::<usize, Simples>(metodo(loja, 7))(loja);   // Commit
                std::mem::transmute::<usize, Simples>(metodo(loja, 2))(loja);
            }
        }
        let mut arquivo: *mut *const usize = std::ptr::null_mut();
        let r = if pedir(link, &iid_arquivo, &mut arquivo) == 0 && !arquivo.is_null() {
            let r = std::mem::transmute::<usize, Salvar>(metodo(arquivo, 6))(arquivo, largo_p(lnk).as_ptr(), 1);   // Save
            std::mem::transmute::<usize, Simples>(metodo(arquivo, 2))(arquivo);
            r
        } else { -1 };
        soltar(link);
        if r != 0 { return Err(format!("o Windows não gravou o atalho {}", lnk.display())); }
    }
    Ok(())
}

// ---- programa aberto ----
// cópias do Giz na Tela abertas nesta conta: só as da pasta instalada, ou todas (a portátil e a de testes também)
fn abertos(pasta: Option<&Path>) -> Vec<u32> {
    let mut pids = Vec::new();
    let eu = std::process::id();
    let pasta = pasta.map(|p| p.display().to_string().to_lowercase());
    unsafe {
        let foto = CreateToolhelp32Snapshot(2, 0);   // TH32CS_SNAPPROCESS
        if foto == -1 { return pids; }
        let mut p: Processo = std::mem::zeroed();
        p.tam = std::mem::size_of::<Processo>() as u32;
        let mut ok = Process32FirstW(foto, &mut p);
        while ok != 0 {
            let n = p.exe.iter().position(|&c| c == 0).unwrap_or(260);
            if p.pid != eu && String::from_utf16_lossy(&p.exe[..n]).eq_ignore_ascii_case(EXE) {
                let h = OpenProcess(0x1000, 0, p.pid);   // PROCESS_QUERY_LIMITED_INFORMATION (outra conta: recusado)
                if h != 0 {
                    let mut buf = [0u16; 1024];
                    let mut tam = buf.len() as u32;
                    if QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut tam) != 0 {
                        let caminho = String::from_utf16_lossy(&buf[..tam as usize]).to_lowercase();
                        let dir = Path::new(&caminho).parent().map(|d| d.display().to_string());
                        if pasta.is_none() || dir == pasta { pids.push(p.pid); }
                    }
                    CloseHandle(h);
                }
            }
            ok = Process32NextW(foto, &mut p);
        }
        CloseHandle(foto);
    }
    pids
}

// a barra ("Giz na Tela") ou a camada de um processo: é por ela que o fechar normal passa
fn janela_de(pid: u32) -> Option<isize> {
    struct Busca { pid: u32, achada: isize }
    unsafe extern "system" fn cada(hwnd: isize, dados: isize) -> i32 {
        let b = &mut *(dados as *mut Busca);
        let mut dono = 0u32;
        GetWindowThreadProcessId(hwnd, &mut dono);
        if dono != b.pid || IsWindowVisible(hwnd) == 0 { return 1; }
        let mut buf = [0u16; 64];
        let n = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32).max(0) as usize;
        let titulo = String::from_utf16_lossy(&buf[..n]);
        if titulo == "Giz na Tela" || (titulo == "Giz na Tela - camada" && b.achada == 0) { b.achada = hwnd; }
        1
    }
    let mut b = Busca { pid, achada: 0 };
    unsafe { EnumWindows(cada, &mut b as *mut Busca as isize) };
    (b.achada != 0).then_some(b.achada)
}

// Pede a cada cópia aberta que se feche pelo caminho normal, como o X da barra: uma mensagem só (uma segunda, em
// 3 s, confirmaria e apagaria o desenho). Sem desenho, ela fecha; com desenho, ela mesma mostra "toque no X de novo"
// e o professor decide. Nunca fecha à força: com desenho na tela, espera; fora do silêncio, pergunta se tenta de novo;
// em silêncio, desiste da instalação depois de `prazo`, sem tirar nada.
// O Sobre das versões antigas fecha a própria cópia logo depois de abrir o instalador: ela sai sozinha antes disso.
fn fechar_pedindo(pasta: Option<&Path>, silencio: bool, prazo: Duration) -> Result<(), String> {
    let inicio = Instant::now();
    let mut pedidos: Vec<u32> = Vec::new();
    loop {
        let vivos = abertos(pasta);
        if vivos.is_empty() { return Ok(()); }
        // espera um pouco antes de pedir: a cópia que chamou o instalador (Sobre) já está saindo sozinha
        if inicio.elapsed() > Duration::from_secs(3) {
            for pid in vivos.iter().filter(|p| !pedidos.contains(p)).copied().collect::<Vec<_>>() {
                if let Some(h) = janela_de(pid) { unsafe { PostMessageW(h, 0x0010, 0, 0) }; }   // WM_CLOSE
                pedidos.push(pid);
            }
        }
        if inicio.elapsed() > prazo {
            if silencio {
                return Err("um Giz na Tela aberto tem desenho na tela e não foi fechado; nada foi instalado".into());
            }
            let r = perguntar("Há um Giz na Tela aberto com desenho na tela, e ele não foi fechado para não perder a tinta.\n\n\
                Salve o que quiser (ou toque no X de novo, na barra dele) e depois clique em Tentar novamente.", 0x5 | 0x30);
            if r != 4 { return Err("instalação cancelada: o Giz na Tela aberto continua como estava".into()); }   // IDRETRY
            return fechar_pedindo(pasta, silencio, prazo);
        }
        std::thread::sleep(Duration::from_millis(300));
    }
}

// "C:\...\unins000.exe" /x  ->  (caminho, argumentos)
fn separar(cmd: &str) -> (String, Vec<String>) {
    let c = cmd.trim();
    if let Some((p, resto)) = c.strip_prefix('"').and_then(|r| r.split_once('"')) {
        return (p.to_string(), resto.split_whitespace().map(String::from).collect());
    }
    (c.to_string(), Vec::new())
}

// tira a instalação do Inno ou do NSIS com o desinstalador dela, em silêncio, e espera a chave sumir (até 30 s);
// os dois se copiam para a pasta temporária e voltam antes de terminar
fn migrar() -> Result<(), String> {
    for (chave, args) in ANTIGAS {
        let Some(cmd) = ler_texto(chave, "UninstallString") else { continue };
        let (exe, mut extra) = separar(&cmd);
        if !exe.to_lowercase().ends_with(".exe") || !Path::new(&exe).is_file() { continue; }
        extra.extend(args.iter().map(|a| a.to_string()));
        std::process::Command::new(&exe).args(&extra).status()
            .map_err(|e| format!("não consegui abrir o desinstalador antigo ({e})"))?;
        let limite = Instant::now() + Duration::from_secs(30);
        while ler_texto(chave, "UninstallString").is_some() {
            if Instant::now() > limite { return Err("o desinstalador da versão antiga não terminou".into()); }
            std::thread::sleep(Duration::from_millis(500));
        }
    }
    Ok(())
}

// grava o programa novo: primeiro ao lado (.new) e depois troca o nome. Se o antigo ainda estiver aberto, o Windows
// não deixa sobrescrever, mas deixa renomear: o antigo vira .old e some na próxima abertura.
fn copiar_programa(de: &Path, para: &Path) -> Result<(), String> {
    let novo = para.with_extension("new");
    let velho = para.with_extension("old");
    std::fs::copy(de, &novo).map_err(|e| format!("não consegui copiar o programa ({e})"))?;
    trocar(&novo, para, &velho, |a, b| std::fs::rename(a, b))
}

// a troca de nomes, com a função de renomear de fora para o teste poder fazê-la falhar
fn trocar(novo: &Path, para: &Path, velho: &Path, renomear: impl Fn(&Path, &Path) -> std::io::Result<()>)
    -> Result<(), String> {
    // recuperação pendente: uma troca anterior caiu no meio e só ficou o .old, sem o exe; ele volta ao nome antes de tudo
    if !para.exists() && velho.exists() { let _ = renomear(velho, para); }
    if renomear(novo, para).is_ok() { return Ok(()); }
    // o .old de antes só sai com o exe no lugar: sem ele, o .old pode ser a única cópia do programa
    if para.exists() { let _ = std::fs::remove_file(velho); }
    let erro = match renomear(para, velho) {
        Err(e) => e,   // o antigo ficou onde estava
        Ok(()) => match renomear(novo, para) {
            Ok(()) => return Ok(()),
            // o antigo já saiu do lugar e o novo não entrou: devolve o antigo, senão o programa fica sem exe
            Err(e) => { let _ = renomear(velho, para); e }
        },
    };
    let _ = std::fs::remove_file(novo);
    let antes = if para.exists() { "; o programa de antes continua no lugar" } else { "" };
    Err(format!("não consegui trocar o programa ({erro}){antes}"))
}

pub struct Feito { pub exe: PathBuf }

pub fn instalar(silencio: bool) -> Result<Feito, String> {
    let pasta = pasta().ok_or("pasta da conta (LOCALAPPDATA) não encontrada")?;
    let destino = pasta.join(EXE);
    let eu = std::env::current_exe().map_err(|e| e.to_string())?;
    if eu.parent().map(|p| p.display().to_string().to_lowercase()) == Some(pasta.display().to_string().to_lowercase()) {
        return Err("esta já é a cópia instalada".into());
    }
    unsafe { CoInitializeEx(std::ptr::null(), 0x2) };   // COINIT_APARTMENTTHREADED, para os atalhos
    let mesa = area_de_trabalho();
    // atalho na Área de Trabalho: na primeira instalação, ou se já havia um (o desinstalador antigo o apaga)
    let com_mesa = !destino.exists() || mesa.as_ref().is_some_and(|m| m.join(ATALHO).exists());
    // o desinstalador da 1.3.x fecha à força todo GizNaTela.exe da conta (até um portátil com desenho): antes de
    // chamá-lo, todas as cópias abertas fecham pelo caminho normal; sem instalação antiga, só as da pasta instalada
    let antiga = ANTIGAS.iter().any(|(c, _)| ler_texto(c, "UninstallString").is_some());
    let prazo = Duration::from_secs(if silencio { 120 } else { 8 });
    fechar_pedindo(if antiga { None } else { Some(&pasta) }, silencio, prazo)?;
    migrar()?;
    std::fs::create_dir_all(&pasta).map_err(|e| format!("não consegui criar {} ({e})", pasta.display()))?;
    // o programa: este próprio exe ou, na variante separada, o GizNaTela.exe que veio ao lado deste instalador
    let origem = if SEPARADO { eu.parent().map(|p| p.join(EXE)).filter(|p| p.is_file()).ok_or("falta o GizNaTela.exe ao lado do instalador")? } else { eu.clone() };
    copiar_programa(&origem, &destino)?;
    let desinstalador = if SEPARADO { pasta.join(DESINSTALADOR) } else { destino.clone() };
    if SEPARADO {
        std::fs::copy(&eu, &desinstalador).map_err(|e| format!("não consegui copiar o desinstalador ({e})"))?;
    }
    for (nome, dados) in TEXTOS {
        std::fs::write(pasta.join(nome), dados).map_err(|e| format!("não consegui gravar {nome} ({e})"))?;
    }
    let menu = menu_iniciar().ok_or("pasta do menu Iniciar não encontrada")?;
    atalho(&menu.join(ATALHO), &destino, "", "Escrever por cima de qualquer programa", None, true)?;
    atalho(&menu.join(ATALHO_DESINSTALAR), &desinstalador, "--desinstalar", "Remove o programa; as imagens salvas ficam guardadas",
        Some(&destino), false)?;
    if let Some(doc) = documentos() {
        let capturas = doc.join("Giz na Tela").join("capturas");
        if std::fs::create_dir_all(&capturas).is_ok() {
            let _ = atalho(&menu.join(ATALHO_IMAGENS), &capturas, "", "Imagens salvas pelo Giz na Tela", None, false);
        }
    }
    if let (true, Some(m)) = (com_mesa, &mesa) {
        let _ = atalho(&m.join(ATALHO), &destino, "", "Escrever por cima de qualquer programa", None, true);
    }
    registrar(&destino, &desinstalador, &pasta)?;
    Ok(Feito { exe: destino })
}

// o desinstalador não consegue apagar o próprio exe aberto: se copia para a pasta temporária e roda a cópia
fn desinstalar_pela_copia(silencio: bool) -> i32 {
    if !silencio && perguntar("Tirar o Giz na Tela deste computador?\n\nOs ajustes e as imagens salvas ficam guardados.",
        0x4 | 0x20) != 6 { return 0; }   // Sim/Não, pergunta
    let copia = (|| -> Result<PathBuf, String> {
        let nome = format!("GizNaTela-desinstalar-{}.exe", &crate::sistema::token_novo().map_err(|e| e.to_string())?[..12]);
        let copia = std::env::temp_dir().join(nome);
        std::fs::copy(std::env::current_exe().map_err(|e| e.to_string())?, &copia).map_err(|e| e.to_string())?;
        Ok(copia)
    })();
    let r = copia.and_then(|c| {
        let mut args = vec!["--desinstalar-agora"];
        if silencio { args.push("/S"); }
        std::process::Command::new(&c).args(args).spawn().map(|_| ()).map_err(|e| e.to_string())
    });
    match r {
        Ok(()) => 0,
        Err(e) => { if !silencio { crate::sistema::avisar_erro(&format!("Não consegui desinstalar: {e}")); } 1 }
    }
}

fn desinstalar(silencio: bool) -> Result<(), String> {
    let pasta = pasta().ok_or("pasta da conta (LOCALAPPDATA) não encontrada")?;
    fechar_pedindo(Some(&pasta), silencio, Duration::from_secs(if silencio { 120 } else { 8 }))?;
    // o exe que pediu a desinstalação fecha logo depois de abrir esta cópia: tenta por até 10 s
    let limite = Instant::now() + Duration::from_secs(10);
    for nome in [EXE, "GizNaTela.old", "GizNaTela.new", DESINSTALADOR, NOMES_DOS_TEXTOS[0], NOMES_DOS_TEXTOS[1], NOMES_DOS_TEXTOS[2]] {
        let arq = pasta.join(nome);
        while arq.exists() && std::fs::remove_file(&arq).is_err() {
            if Instant::now() > limite { return Err(format!("não consegui apagar {}", arq.display())); }
            std::thread::sleep(Duration::from_millis(300));
        }
    }
    let _ = std::fs::remove_dir(&pasta);   // só se ficou vazia: o que não é nosso fica
    for lnk in [menu_iniciar().map(|m| m.join(ATALHO)), menu_iniciar().map(|m| m.join(ATALHO_DESINSTALAR)),
                menu_iniciar().map(|m| m.join(ATALHO_IMAGENS)), area_de_trabalho().map(|m| m.join(ATALHO))].into_iter().flatten() {
        let _ = std::fs::remove_file(lnk);
    }
    let r = unsafe { RegDeleteTreeW(HKCU, largo(CHAVE).as_ptr()) };
    if r != 0 && r != 2 { return Err(format!("o Windows não apagou a chave de desinstalação (código {r})")); }
    Ok(())
}

// Chamado no começo do main: devolve o código de saída quando a linha de comando é de instalação ou desinstalação
pub fn pedido() -> Option<i32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tem = |a: &str| args.iter().any(|x| x.eq_ignore_ascii_case(a));
    let silencio = tem("/S") || tem("/SILENT") || tem("/VERYSILENT");
    if tem("--desinstalar") {
        return Some(desinstalar_pela_copia(silencio));
    }
    if tem("--desinstalar-agora") {
        return Some(match desinstalar(silencio) {
            Ok(()) => { if !silencio { perguntar("O Giz na Tela foi tirado deste computador.", 0x40); } 0 }
            Err(e) => { if !silencio { crate::sistema::avisar_erro(&format!("Não consegui desinstalar: {e}")); } 1 }
        });
    }
    let eu = std::env::current_exe().ok()?;
    let nome = eu.file_name()?.to_string_lossy().to_lowercase();
    if !SEPARADO && !tem("--instalar") && !nome.starts_with("instalar-giznatela") {
        return None;
    }
    // o desinstalador do Inno fecha todo GizNaTela.exe da conta: com esse nome, a instalação roda de uma cópia
    if nome == EXE.to_lowercase() && ANTIGAS.iter().any(|(c, _)| ler_texto(c, "UninstallString").is_some()) {
        let copia = std::env::temp_dir().join(format!("Instalar-GizNaTela-{}.exe", crate::VERSAO));
        let r = std::fs::copy(&eu, &copia).and_then(|_| std::process::Command::new(&copia).args(&args).status());
        return Some(r.ok().and_then(|s| s.code()).unwrap_or(1));
    }
    if !silencio {
        let r = perguntar(&format!("Instalar o Giz na Tela {} nesta conta do Windows?\n\n\
            Sim: instala, cria os atalhos no menu Iniciar e na Área de Trabalho e aparece em Aplicativos instalados. \
            Não precisa de administrador. Se ele estiver aberto, será fechado (com desenho na tela, ele pergunta antes).\n\n\
            Não: só abre o programa, sem instalar.", crate::VERSAO), 0x3 | 0x20);   // Sim/Não/Cancelar
        if r == 7 { return if SEPARADO { Some(0) } else { None }; }
        if r != 6 { return Some(0); }
    }
    match instalar(silencio) {
        Ok(f) => {
            // escola sem internet: o instalador do WebView2 da Microsoft veio ao lado deste exe
            if !crate::sistema::tem_webview2() {
                if let Some(origem) = eu.parent() { crate::sistema::instalar_webview2_de(origem, silencio); }
            }
            // /reabrir=1: atualização pelo Sobre; sem silêncio: a pessoa acabou de instalar e quer usar
            let reabrir = tem("/reabrir=1");
            if reabrir || !silencio {
                let mut c = std::process::Command::new(&f.exe);
                if reabrir { c.arg("--depois-de-atualizar"); }
                let _ = c.spawn();
            }
            Some(0)
        }
        Err(e) => {
            if !silencio { crate::sistema::avisar_erro(&format!("Não consegui instalar o Giz na Tela: {e}")); }
            // veio do Sobre, que já fechou a cópia instalada: ela volta a abrir, como estava
            if tem("/reabrir=1") {
                if let Some(antigo) = pasta().map(|p| p.join(EXE)).filter(|p| p.is_file()) {
                    let _ = std::process::Command::new(antigo).spawn();
                }
            }
            Some(1)
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn separa_o_desinstalador() {
        assert_eq!(separar(r#""C:\a b\unins000.exe""#), (r"C:\a b\unins000.exe".to_string(), vec![]));
        assert_eq!(separar(r#""C:\a\uninstall.exe" /x y"#), (r"C:\a\uninstall.exe".to_string(), vec!["/x".into(), "y".into()]));
        assert_eq!(separar(r"C:\a\u.exe"), (r"C:\a\u.exe".to_string(), vec![]));
    }

    // o antigo já virou .old e o novo não consegue tomar o nome (antivírus segurando o .new, por exemplo): o antigo
    // tem de voltar ao lugar, senão o programa fica sem exe (lacuna N-C2 da auditoria do modelo nativo, 29/10/2026)
    #[test]
    fn troca_que_falha_devolve_o_antigo() {
        let pasta = std::env::temp_dir().join(format!("giz-troca-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&pasta);
        std::fs::create_dir_all(&pasta).unwrap();
        let (para, novo, velho) = (pasta.join("GizNaTela.exe"), pasta.join("GizNaTela.new"), pasta.join("GizNaTela.old"));
        std::fs::write(&para, b"antigo").unwrap();
        std::fs::write(&novo, b"novo").unwrap();
        let r = trocar(&novo, &para, &velho, |a, b| {
            if a == novo.as_path() { Err(std::io::Error::other("preso")) } else { std::fs::rename(a, b) }
        });
        assert!(r.is_err());
        assert_eq!(std::fs::read(&para).ok().as_deref(), Some(&b"antigo"[..]), "o programa antigo voltou ao lugar");
        assert!(!velho.exists() && !novo.exists(), "sem sobra .old nem .new");
        let _ = std::fs::remove_dir_all(&pasta);
    }

    // uma queda no meio deixou só o .old (o exe ausente) e as tentativas seguintes também falham: o .old é a única
    // cópia e não pode ser apagado antes de o novo estar no lugar (N-R1 da auditoria do Tarja, 29/10/2026)
    #[test]
    fn falha_repetida_deixa_uma_copia_valida() {
        let pasta = std::env::temp_dir().join(format!("giz-troca-r-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&pasta);
        std::fs::create_dir_all(&pasta).unwrap();
        let (para, novo, velho) = (pasta.join("GizNaTela.exe"), pasta.join("GizNaTela.new"), pasta.join("GizNaTela.old"));
        std::fs::write(&velho, b"antigo").unwrap();   // a sobra: só o .old, sem o exe
        for _ in 0..3 {
            std::fs::write(&novo, b"novo").unwrap();
            let r = trocar(&novo, &para, &velho, |a, b| {
                if a == novo.as_path() { Err(std::io::Error::other("preso")) } else { std::fs::rename(a, b) }
            });
            assert!(r.is_err());
            let copias: Vec<_> = [&para, &velho].iter().filter_map(|p| std::fs::read(p).ok()).collect();
            assert!(copias.iter().any(|c| c == b"antigo"), "sobrou pelo menos uma cópia válida do programa");
        }
        assert_eq!(std::fs::read(&para).ok().as_deref(), Some(&b"antigo"[..]), "a cópia voltou ao nome do programa");
        let _ = std::fs::remove_dir_all(&pasta);
    }
}
