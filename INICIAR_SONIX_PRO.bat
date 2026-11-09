@echo off
chcp 65001 > nul
title SONIX PRO // AMPFORGE™

echo ============================================================
echo      🎛️  INICIALIZANDO SONIX PRO // AMPFORGE™
echo ============================================================

cd /d "%~dp0"

:: 1. Verifica se já está em execução na porta 7890
netstat -ano | findstr /R /C:":7890 .*LISTENING" > nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo [i] Sonix Pro já está em execução no loopback 127.0.0.1:7890.
    echo [i] Focando/reabrindo janela dedicada...
    goto open_window
)

echo [1/3] Conectando host de áudio nativo (WASAPI P2/P3)...
echo [2/3] Iniciando motor DSP White-Box e servidor local 127.0.0.1...

set "BIN_PATH="
if exist "%~dp0SonixPro.exe" (
    set "BIN_PATH=%~dp0SonixPro.exe"
) else if exist "%~dp0target\release\sonix-desktop.exe" (
    set "BIN_PATH=%~dp0target\release\sonix-desktop.exe"
) else if exist "%~dp0target\debug\sonix-desktop.exe" (
    set "BIN_PATH=%~dp0target\debug\sonix-desktop.exe"
)

if "%BIN_PATH%"=="" (
    echo.
    echo [ERRO] Executável do Sonix Pro não foi encontrado!
    echo Execute "cargo build --release -p sonix-desktop" para gerar o binário nativo.
    echo.
    pause
    exit /b 1
)

:: Evita que o executável abra janela duplicada, pois este script abre abaixo
set SONIX_NO_BROWSER=1
start "" /b "%BIN_PATH%"

timeout /t 2 /nobreak > nul

:open_window
echo [3/3] Abrindo janela dedicada do aplicativo...
where msedge > nul 2>&1
if %ERRORLEVEL% equ 0 (
    start "" msedge --app=http://127.0.0.1:7890 --window-size=1280,820
) else (
    where chrome > nul 2>&1
    if %ERRORLEVEL% equ 0 (
        start "" chrome --app=http://127.0.0.1:7890 --window-size=1280,820
    ) else (
        start http://127.0.0.1:7890
    )
)

echo.
echo 🎸 Sonix Pro em execução! Pode minimizar esta janela.
