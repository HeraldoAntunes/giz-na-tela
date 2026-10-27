# Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Monta o instalador e a versão portátil
# Compila o programa, roda os testes, gera os avisos de terceiros, o instalador (Inno Setup 6) e o zip portátil,
# verifica no Microsoft Defender e escreve dist\SHA256SUMS.txt. Gerar e publicar: só com ordem do professor.
# Uso (na pasta do projeto):  powershell -ExecutionPolicy Bypass -File installer\build.ps1 [-SemTesteDeTela]
param([switch]$SemTesteDeTela)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
$env:Path += ";$env:USERPROFILE\.cargo\bin"
$versao = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
"Giz na Tela $versao"

# 1) programa e testes (o teste de tela cobre a tela por alguns segundos; pule com -SemTesteDeTela)
cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'a compilação falhou' }
python tests\testar_versao.py
if ($LASTEXITCODE -ne 0) { throw 'testar_versao falhou' }
if (-not $SemTesteDeTela) {
  python tests\testar_comandos.py
  if ($LASTEXITCODE -ne 0) { throw 'testar_comandos falhou' }
}

# 2) avisos das bibliotecas que vão dentro do programa
python scripts\avisos_terceiros.py
if ($LASTEXITCODE -ne 0) { throw 'avisos de terceiros falharam' }

# 3) instalador; leva o instalador oficial do WebView2 (para o PC que não o tiver), só se a assinatura for da Microsoft
$wv2 = 'installer\MicrosoftEdgeWebview2Setup.exe'
if (-not (Test-Path $wv2)) { Invoke-WebRequest 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile $wv2 -UseBasicParsing }
$assin = Get-AuthenticodeSignature $wv2
if ($assin.Status -ne 'Valid' -or $assin.SignerCertificate.Subject -notmatch '^CN=Microsoft Corporation,') {
  Remove-Item $wv2; throw "instalador do WebView2 sem assinatura válida da Microsoft ($($assin.Status))"
}
if (Test-Path dist) { Remove-Item dist -Recurse -Force }
New-Item -ItemType Directory dist | Out-Null
$iscc = @("$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe", "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe") |
  Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $iscc) { throw 'Inno Setup 6 não encontrado' }
& $iscc /Q installer\giz-na-tela.iss
if ($LASTEXITCODE -ne 0) { throw 'o Inno Setup falhou' }

# 4) versão portátil: com portatil.txt os ajustes ficam na pasta "dados" ao lado do programa (pen drive)
$port = "dist\GizNaTela-$versao-portatil"
New-Item -ItemType Directory $port | Out-Null
Copy-Item target\release\GizNaTela.exe, LICENSE, THIRD-PARTY-NOTICES.md, README.md $port
Set-Content "$port\portatil.txt" 'Com este arquivo presente, os ajustes do Giz na Tela ficam na pasta dados ao lado do GizNaTela.exe.' -Encoding UTF8
Compress-Archive -Path "$port\*" -DestinationPath "dist\GizNaTela-$versao-portatil.zip"
Remove-Item $port -Recurse -Force

# 5) Microsoft Defender (0 = nada encontrado)
$mp = "$env:ProgramFiles\Windows Defender\MpCmdRun.exe"
if (Test-Path $mp) {
  foreach ($alvo in Get-ChildItem dist -File) {
    $r = & $mp -Scan -ScanType 3 -File $alvo.FullName -DisableRemediation
    if ($LASTEXITCODE -ne 0) { throw "Defender acusou algo (código $LASTEXITCODE) em $($alvo.Name)`n$r" }
    "Defender OK: $($alvo.Name)"
  }
}

# 6) somas SHA-256 para conferir o download
Get-ChildItem dist -File | ForEach-Object { "{0}  {1}" -f (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower(), $_.Name } |
  Set-Content dist\SHA256SUMS.txt -Encoding ASCII
Get-Content dist\SHA256SUMS.txt
