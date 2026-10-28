// Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Opções da linha de comando e ajustes guardados
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const PORTA_API: u16 = 8833;
pub const PORTA_GIZ: u16 = 8765;

// --dados e --capturas servem à versão portátil e aos testes (pastas temporárias, nunca as do professor)
pub struct Opcoes {
    pub dados: PathBuf,
    pub capturas: Option<PathBuf>,
    pub porta_api: u16,
    pub porta_giz: Option<u16>,
    pub mcp: bool,
}

fn pasta_do_programa() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(Path::to_path_buf)
}

impl Opcoes {
    pub fn da_linha_de_comando() -> Opcoes {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let valor = |nome: &str| args.iter().position(|a| a == nome).and_then(|i| args.get(i + 1)).cloned();
        let porta = |nome: &str| valor(nome).and_then(|v| v.parse::<u16>().ok());
        Opcoes {
            dados: valor("--dados").map(PathBuf::from).unwrap_or_else(pasta_de_dados_padrao),
            capturas: valor("--capturas").map(PathBuf::from),
            porta_api: porta("--porta-api").unwrap_or(PORTA_API),
            porta_giz: porta("--porta-giz"),
            mcp: args.iter().any(|a| a == "--mcp"),
        }
    }
}

// versão portátil (portatil.txt ao lado do programa): tudo na pasta "dados" junto do programa, para o pen drive;
// instalada: %APPDATA%\Giz na Tela, que só a conta do professor lê
pub fn pasta_de_dados_padrao() -> PathBuf {
    if let Some(p) = pasta_do_programa() {
        if p.join("portatil.txt").exists() {
            return p.join("dados");
        }
    }
    std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("Giz na Tela")
}

pub const NIVEIS: [&str; 3] = ["desligado", "ler", "agir"];

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    // posição da barra ABERTA (px físicos); recolhida, o botão fica à direita de onde a barra estava
    pub barra: Option<(i32, i32)>,
    pub recolhida: bool,
    // agentes de IA pela API local e pelo MCP: desligado, ler (só consultar) ou agir
    pub agentes: String,
    pub caneta_vista: bool,
    pub dicas_ink: u32,
    // escolhas da barra que voltam ao abrir (cor, espessura, tipos de seta e de cota, cores das bolinhas)
    pub preferencias: serde_json::Value,
    // nome da tela (monitor) onde a camada ficou, para abrir de novo nela se ainda estiver ligada (o projetor)
    pub tela: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config { barra: None, recolhida: false, agentes: "desligado".into(), caneta_vista: false, dicas_ink: 0,
            preferencias: serde_json::Value::Null, tela: None }
    }
}

impl Config {
    pub fn ler(dados: &Path) -> Config {
        let mut c: Config = std::fs::read_to_string(dados.join("config.json")).ok()
            .and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        if !NIVEIS.contains(&c.agentes.as_str()) {
            c.agentes = "desligado".into();
        }
        c
    }
    // grava num arquivo ao lado e troca: um desligamento no meio não deixa o arquivo pela metade
    pub fn gravar(&self, dados: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dados)?;
        let tmp = dados.join("config.json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self).unwrap_or_default())?;
        std::fs::rename(tmp, dados.join("config.json"))
    }
}
