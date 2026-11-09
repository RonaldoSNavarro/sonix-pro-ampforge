@echo off
chcp 65001 > nul
title SONIX PRO // AMPFORGE™

echo ============================================================
echo      🎛️  INICIALIZANDO SONIX PRO // AMPFORGE™
echo ============================================================
echo [1/3] Conectando host de áudio nativo (WASAPI P2/P3)...
echo [2/3] Iniciando motor DSP White-Box e servidor local 127.0.0.1...

cd /d "%~dp0"

if exist "%~dp0SonixPro.exe" (
    start "" /b "%~dp0SonixPro.exe"
) else if exist "%~dp0target\release\sonix-desktop.exe" (
    start "" /b "%~dp0target\release\sonix-desktop.exe"
)

timeout /t 2 /nobreak > nul

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
