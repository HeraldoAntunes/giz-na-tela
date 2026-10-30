# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Monta o instalador e os pacotes
# Gera os avisos de terceiros (vão dentro do exe), compila, roda os testes e monta em dist\: o instalador (o próprio
# GizNaTela.exe com o nome Instalar-GizNaTela-<versão>.exe, que se instala sozinho: src\instalador.rs), o zip portátil
# e o zip "escola sem internet" (o instalador e, ao lado, o instalador completo do WebView2 da Microsoft). Verifica no
# Microsoft Defender e escreve dist\SHA256SUMS.txt. Publicar: só com ordem do professor.
# Uso (na pasta do projeto):  powershell -ExecutionPolicy Bypass -File installer\build.ps1 [-SemTesteDeTela]
param([switch]$SemTesteDeTela)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
$env:Path += ";$env:USERPROFILE\.cargo\bin"
$versao = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
"Giz na Tela $versao"
# o exe compilado levaria os caminhos deste PC (C:\Users\<conta>\.cargo\registry e a pasta do projeto) nas mensagens
# de erro do Rust: troca pelos curtos. CARGO_ENCODED_RUSTFLAGS separa por 0x1f porque a pasta do projeto tem espaços.
# Fica aqui, e não num .cargo\config.toml versionado, para não gravar caminho nenhum no repositório.
$env:CARGO_ENCODED_RUSTFLAGS = "--remap-path-prefix=$env:USERPROFILE=~$([char]0x1f)--remap-path-prefix=$((Get-Location).Path)=."

# 1) avisos das bibliotecas que vão dentro do programa (antes de compilar: o exe leva o arquivo dentro)
python scripts\avisos_terceiros.py
if ($LASTEXITCODE -ne 0) { throw 'avisos de terceiros falharam' }

# 2) programa e testes (o teste de tela cobre a tela por alguns segundos; pule com -SemTesteDeTela)
cargo build --release --locked
if ($LASTEXITCODE -ne 0) { throw 'a compilação falhou' }
# como o passo 1b da porta: nenhum caminho pessoal dentro do exe (X:\Users\<conta>, a pasta deste projeto e a que
# está acima dela, lidas na hora, sem nome de pasta escrito aqui)
$bytes = [Text.Encoding]::GetEncoding(28591).GetString([IO.File]::ReadAllBytes("$PWD\target\release\GizNaTela.exe"))
$acima = Split-Path (Get-Location).Path -Parent
$padrao = '(?i)[a-z]:[\\/]+Users[\\/]+[^\\/\x00]{1,40}|' + [regex]::Escape((Get-Location).Path) + '|' + [regex]::Escape($acima)
$pessoais = [regex]::Matches($bytes, $padrao)
if ($pessoais.Count) { throw "o exe leva $($pessoais.Count) caminho(s) pessoal(is), ex.: $($pessoais[0].Value)" }
cargo test --release --locked
if ($LASTEXITCODE -ne 0) { throw 'os testes do cargo falharam' }
python tests\testar_versao.py
if ($LASTEXITCODE -ne 0) { throw 'testar_versao falhou' }
if (-not $SemTesteDeTela) {
  python tests\testar_comandos.py
  if ($LASTEXITCODE -ne 0) { throw 'testar_comandos falhou' }
  # reabrir logo depois de uma queda, 10 vezes (o WebView2 que sobra da queda é fechado ao abrir)
  python tests\testar_queda.py
  if ($LASTEXITCODE -ne 0) { throw 'testar_queda falhou' }
  # compilação de teste: otimizada, com debug-assertions (a rota de teste da camada e a falha forçada das janelas).
  # A de debug, sem otimização, já erra a primeira criação das janelas de verdade (ver tests\testar_janelas.py)
  cargo build --release --locked --config profile.release.debug-assertions=true --target-dir target\teste-otimizado
  if ($LASTEXITCODE -ne 0) { throw 'a compilação de teste falhou' }
  # casos da auditoria de 29/10 com eventos de ponteiro no WebView2
  python tests\testar_camada.py
  if ($LASTEXITCODE -ne 0) { throw 'testar_camada falhou' }
  # falha forçada do WebView2 ao criar as janelas: tenta de novo e, se não der, sai em ordem
  python tests\testar_janelas.py
  if ($LASTEXITCODE -ne 0) { throw 'testar_janelas falhou' }
}

if (Test-Path dist) { Remove-Item dist -Recurse -Force }
New-Item -ItemType Directory dist | Out-Null
$exe = 'target\release\GizNaTela.exe'
$instalador = "Instalar-GizNaTela-$versao.exe"

# 3) o instalador: o mesmo exe, com o nome que o faz se instalar, dentro de um zip. O .exe solto levava um alerta
#    no VirusTotal, de causa não confirmada (Trapmine acusa exes Tauri pequenos em geral, até um mínimo); os zips dão zero. O Sobre da
#    1.4.0 em diante baixa este zip; o da 1.3.x procura o .exe solto e mostra as notas da release, que explicam.
$ins = "dist\Instalar-GizNaTela-$versao"
New-Item -ItemType Directory $ins | Out-Null
Copy-Item $exe "$ins\$instalador"
Copy-Item LICENSE, THIRD-PARTY-NOTICES.md, README.md $ins
# padrão dos quatro programas (gestão, 29/10/2026): um LEIA-ME curto dentro do zip do instalador
Set-Content "$ins\LEIA-ME.txt" @(
  "Giz na Tela $versao",
  '',
  "Para instalar: extraia este zip (botão direito > Extrair tudo) e abra o $instalador.",
  'Ele se instala na sua conta, sem pedir administrador, e abre o Giz na Tela.',
  'Se já tiver uma versão anterior, ela é trocada pela nova; os ajustes e as imagens salvas ficam.') -Encoding UTF8
Compress-Archive -Path "$ins\*" -DestinationPath "dist\Instalar-GizNaTela-$versao.zip"
Remove-Item $ins -Recurse -Force

# 4) versão portátil: com portatil.txt os ajustes ficam na pasta "dados" ao lado do programa (pen drive)
$port = "dist\GizNaTela-$versao-portatil"
New-Item -ItemType Directory $port | Out-Null
Copy-Item $exe, LICENSE, THIRD-PARTY-NOTICES.md, README.md $port
Set-Content "$port\portatil.txt" 'Com este arquivo presente, os ajustes do Giz na Tela ficam na pasta dados ao lado do GizNaTela.exe.' -Encoding UTF8
Compress-Archive -Path "$port\*" -DestinationPath "dist\GizNaTela-$versao-portatil.zip"
Remove-Item $port -Recurse -Force

# 5) escola sem internet: o instalador e, ao lado, o instalador completo do WebView2 (~190 MB), só com a assinatura
#    válida da Microsoft; o programa o confere de novo e o oferece se o WebView2 faltar. Fica em cache no installer\.
$wv2 = 'installer\MicrosoftEdgeWebView2RuntimeInstallerX64.exe'
if (-not (Test-Path $wv2)) { Invoke-WebRequest 'https://go.microsoft.com/fwlink/?linkid=2124701' -OutFile $wv2 -UseBasicParsing }
$assin = Get-AuthenticodeSignature $wv2
if ($assin.Status -ne 'Valid' -or $assin.SignerCertificate.Subject -notmatch '^CN=Microsoft Corporation,') {
  Remove-Item $wv2; throw "instalador do WebView2 sem assinatura válida da Microsoft ($($assin.Status))"
}
$sem = "dist\GizNaTela-$versao-sem-internet"
New-Item -ItemType Directory $sem | Out-Null
Copy-Item $exe "$sem\$instalador"
Copy-Item $wv2, LICENSE, THIRD-PARTY-NOTICES.md, README.md $sem
Compress-Archive -Path "$sem\*" -DestinationPath "dist\GizNaTela-$versao-sem-internet.zip"
Remove-Item $sem -Recurse -Force

# 6) Microsoft Defender (0 = nada encontrado)
# obrigatório: sem o Defender, o pacote não sai (auditoria de 29/10: antes ele era pulado em silêncio)
$mp = "$env:ProgramFiles\Windows Defender\MpCmdRun.exe"
if (-not (Test-Path $mp)) { throw 'Microsoft Defender (MpCmdRun.exe) não encontrado: o pacote não pode sair sem a verificação' }
foreach ($alvo in Get-ChildItem dist -File) {
  $r = & $mp -Scan -ScanType 3 -File $alvo.FullName -DisableRemediation
  if ($LASTEXITCODE -ne 0) { throw "Defender acusou algo (código $LASTEXITCODE) em $($alvo.Name)`n$r" }
  "Defender OK: $($alvo.Name)"
}

# 7) somas SHA-256 para conferir o download
Get-ChildItem dist -File | ForEach-Object { "{0}  {1}" -f (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower(), $_.Name } |
  Set-Content dist\SHA256SUMS.txt -Encoding ASCII
Get-Content dist\SHA256SUMS.txt
