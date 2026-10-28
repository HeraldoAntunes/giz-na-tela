; Giz na Tela — © 2026 Heraldo Antunes — Licença MIT (ver LICENSE). Instalador (Inno Setup 6)
; Gerado por installer\build.ps1, que compila o programa antes. Gerar e publicar: só com ordem do professor.
#define Versao "1.2.1"
#define Exe "..\target\release\GizNaTela.exe"

[Setup]
AppId={{4BB282B2-1BF2-444F-BCB1-0F6E51B46035}
AppName=Giz na Tela
AppVersion={#Versao}
AppVerName=Giz na Tela {#Versao}
AppPublisher=Heraldo Antunes (projeto livre)
AppCopyright=Copyright (c) 2026 Heraldo Antunes - Licença MIT
VersionInfoVersion={#Versao}.0
VersionInfoDescription=Instalador do Giz na Tela
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
DefaultDirName={autopf}\Giz na Tela
DefaultGroupName=Giz na Tela
DisableProgramGroupPage=yes
LicenseFile=..\LICENSE
SetupIconFile=..\icons\icon.ico
UninstallDisplayIcon={app}\GizNaTela.exe
UninstallDisplayName=Giz na Tela {#Versao}
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
Compression=lzma2/max
SolidCompression=yes
; o anotador fica aberto por cima da tela: atualizar precisa fechá-lo sem perguntar
CloseApplications=force
OutputDir=..\dist
OutputBaseFilename=Instalar-GizNaTela-{#Versao}

[Languages]
Name: "ptbr"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Tasks]
Name: "desktopicon"; Description: "Criar atalho na Área de Trabalho"; GroupDescription: "Atalhos:"

[Files]
Source: "{#Exe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD-PARTY-NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
; instalador oficial do WebView2 (baixado e conferido pelo build.ps1); só vai para o PC se faltar o WebView2
Source: "MicrosoftEdgeWebview2Setup.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall; Check: FaltaWebView2

[Icons]
Name: "{autoprograms}\Giz na Tela"; Filename: "{app}\GizNaTela.exe"; Comment: "Escrever por cima de qualquer programa"; AppUserModelID: "GizNaTela"
Name: "{autoprograms}\Giz na Tela - imagens salvas"; Filename: "{userdocs}\Giz na Tela\capturas"
Name: "{autoprograms}\Desinstalar o Giz na Tela"; Filename: "{uninstallexe}"; Comment: "Remove o programa; as imagens salvas ficam guardadas"
Name: "{autodesktop}\Giz na Tela"; Filename: "{app}\GizNaTela.exe"; Tasks: desktopicon; AppUserModelID: "GizNaTela"

[Dirs]
Name: "{userdocs}\Giz na Tela\capturas"; Flags: uninsneveruninstall

[Run]
Filename: "{tmp}\MicrosoftEdgeWebview2Setup.exe"; Parameters: "/silent /install"; StatusMsg: "Instalando o WebView2 da Microsoft, que desenha o Giz na Tela (precisa de internet)..."; Flags: waituntilterminated; Check: FaltaWebView2
Filename: "{app}\GizNaTela.exe"; Description: "Abrir o Giz na Tela agora"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/IM GizNaTela.exe /F"; Flags: runhidden; RunOnceId: "FecharGizNaTela"

[Code]
// O Giz na Tela desenha com o WebView2 (motor do Edge). Ele vem no Windows 11 e nos Windows 10 atualizados; a
// conferência é a recomendada pela Microsoft: versão ("pv") na chave do EdgeUpdate, da máquina ou da conta.
const ChaveWebView2 = 'Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';

function VersaoValida(Raiz: Integer; Chave: String): Boolean;
var V: String;
begin
  Result := RegQueryStringValue(Raiz, Chave, 'pv', V) and (V <> '') and (V <> '0.0.0.0');
end;

function TemWebView2: Boolean;
begin
  Result := VersaoValida(HKLM32, 'SOFTWARE\' + ChaveWebView2) or VersaoValida(HKLM64, 'SOFTWARE\' + ChaveWebView2)
    or VersaoValida(HKCU, 'Software\' + ChaveWebView2);
end;

var Faltava: Boolean;

function FaltaWebView2: Boolean;
begin
  Result := Faltava;
end;

function InitializeSetup: Boolean;
begin
  Faltava := not TemWebView2;
  Result := True;
end;

// sem internet o instalador do WebView2 não termina: avisa o que fazer, em vez de deixar o programa sem abrir
procedure CurStepChanged(Etapa: TSetupStep);
begin
  if (Etapa = ssPostInstall) and Faltava and not TemWebView2 then
    MsgBox('O Giz na Tela foi instalado, mas o WebView2 da Microsoft (que desenha a barra e a tinta) não pôde ser ' +
      'instalado agora, provavelmente por falta de internet.' + #13#10#13#10 +
      'Conecte o computador à internet e rode este instalador de novo, ou baixe o "WebView2 Runtime" no site da ' +
      'Microsoft (developer.microsoft.com/microsoft-edge/webview2).', mbInformation, MB_OK);
end;

[Messages]
; a tradução padrão do Inno usa "pra"; aqui fica a forma "para"
ptbr.SetupAppRunningError=O instalador detectou que o %1 está atualmente em execução.%n%nPor favor feche todas as instâncias dele agora, então clique em OK para continuar ou em Cancelar para sair.
ptbr.UninstallAppRunningError=O Desinstalador detectou que o %1 está atualmente em execução.%n%nPor favor feche todas as instâncias dele agora, então clique em OK para continuar ou em Cancelar para sair.
ptbr.PrivilegesRequiredOverrideText1=O %1 pode ser instalado para todos os usuários (requer privilégios administrativos) ou só para você.
ptbr.PrivilegesRequiredOverrideText2=O %1 pode ser instalado só para você ou para todos os usuários (requer privilégios administrativos).
ptbr.PrivilegesRequiredOverrideAllUsers=Instalar para &todos os usuários
ptbr.PrivilegesRequiredOverrideAllUsersRecommended=Instalar para &todos os usuários (recomendado)
ptbr.PrivilegesRequiredOverrideCurrentUser=Instalar só &para mim
ptbr.PrivilegesRequiredOverrideCurrentUserRecommended=Instalar só &para mim (recomendado)
ptbr.ExitSetupMessage=A Instalação não está completa. Se você sair agora o programa não será instalado.%n%nVocê pode executar o instalador novamente outra hora para completar a instalação.%n%nSair do instalador?
ptbr.ButtonYesToAll=Sim para &Todos
ptbr.ButtonNoToAll=Nã&o para Todos
ptbr.SelectLanguageLabel=Selecione o idioma para usar durante a instalação:
ptbr.ClickNext=Clique em Avançar para continuar ou em Cancelar para sair do instalador.
ptbr.PasswordLabel3=Por favor forneça a senha, então clique em Avançar para continuar. As senhas são caso-sensitivo.
ptbr.InfoBeforeClickLabel=Quando você estiver pronto para continuar com o instalador, clique em Avançar.
ptbr.InfoAfterClickLabel=Quando você estiver pronto para continuar com o instalador, clique em Avançar.
ptbr.SelectDirBrowseLabel=Para continuar clique em Avançar. Se você gostaria de selecionar uma pasta diferente, clique em Procurar.
ptbr.DiskSpaceWarning=O instalador requer pelo menos %1 KB de espaço livre para instalar mas o drive selecionado só tem %2 KB disponíveis.%n%nVocê quer continuar de qualquer maneira?
ptbr.SelectComponentsLabel2=Selecione os componentes que você quer instalar; desmarque os componentes que você não quer instalar. Clique em Avançar quando você estiver pronto para continuar.
ptbr.SelectStartMenuFolderBrowseLabel=Para continuar clique em Avançar. Se você gostaria de selecionar uma pasta diferente, clique em Procurar.
ptbr.WizardReady=Pronto para Instalar
ptbr.ReadyLabel1=O instalador está agora pronto para começar a instalar o [name] no seu computador.
ptbr.ReadyLabel2a=Clique em Instalar para continuar com a instalação ou clique em Voltar se você quer revisar ou mudar quaisquer configurações.
ptbr.ReadyLabel2b=Clique em Instalar para continuar com a instalação.
ptbr.WizardPreparing=Preparando para Instalar
ptbr.PreparingDesc=O instalador está se preparando para instalar o [name] no seu computador.
ptbr.PreviousInstallNotCompleted=A instalação/remoção de um programa anterior não foi completada. Você precisará reiniciar o computador para completar essa instalação.%n%nApós reiniciar seu computador execute o instalador novamente para completar a instalação do [name].
ptbr.CannotContinue=O instalador não pode continuar. Por favor clique em Cancelar para sair.
ptbr.ClickFinish=Clique em Concluir para sair do Instalador.
ptbr.FinishedRestartLabel=Para completar a instalação do [name], o instalador deve reiniciar seu computador. Você gostaria de reiniciar agora?
ptbr.FinishedRestartMessage=Para completar a instalação do [name], o instalador deve reiniciar seu computador.%n%nVocê gostaria de reiniciar agora?
ptbr.UninstalledAndNeedsRestart=Para completar a desinstalação do %1, seu computador deve ser reiniciado.%n%nVocê gostaria de reiniciar agora?
ptbr.WelcomeLabel2=Este assistente vai instalar o [name/ver] no seu computador.%n%nÉ um programa livre e gratuito (licença MIT), que funciona sem internet. As imagens salvas ficam em Documentos\Giz na Tela\capturas e são mantidas se você desinstalar.
