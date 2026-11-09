---
spec: 013
titulo: Execução desktop nativa no Windows com captura física P2/P3 e inicialização por duplo clique
status: implementada
atualizado: 2026-11-08
---
# 013 — Execução Desktop Nativa no Windows com Captura Física P2/P3 e Duplo Clique

## Objetivo
Permitir ao guitarrista conectar a guitarra na entrada de microfone/linha P2/P3 do computador, iniciar o Sonix Pro com um único duplo clique no Windows (sem terminal ou linha de comando), processar o sinal em tempo real com o motor DSP AmpForge (Noise Gate, Amp White-Box e Gabinete IR) com ultrabaixa latência e operar via janela desktop dedicada.

## Contexto
As especificações 001 a 012 consolidaram o motor DSP de 64-bit, a matriz de roteamento DAG, modelagem valvulada, convolução de gabinete, presets e o design system skeuomórfico. No entanto:
1. O host de áudio em `crates/sonix-desktop` operava apenas com enumeração de dispositivo simulada (mock) para testes contratuais da spec 012, sem duplex I/O com os drivers de áudio reais do sistema operacional.
2. Não havia pacote executável independente compilado para Windows (`.exe` / `.bat`) na raiz do repositório, exigindo comandos manuais de desenvolvimento no terminal (`cargo`, `node`, `vite`).
3. O usuário especificou o cenário real de uso: entrada física 3.5mm P2/P3 do PC (Realtek / Line-in), sistema Windows, e exigência estrita de usabilidade: inicialização por duplo clique sem interação com linha de comando.

## Comportamento esperado
1. Ao clicar duas vezes no executável (`SonixPro.exe` ou `INICIAR_SONIX_PRO.bat`) na raiz do projeto, a aplicação é iniciada automaticamente.
2. O sistema inicializa o host de áudio via backend WASAPI (Windows Audio Session API) de baixa latência, conectando a entrada padrão do sistema (Microfone P2/P3) e a saída padrão (Fones/Alto-falantes).
3. Um buffer circular lock-free SPSC (*Single-Producer Single-Consumer*) conecta a captura ao processamento DSP e à saída, sem locks nem alocações de memória durante o streaming de áudio.
4. O sinal da guitarra passa em tempo real pelo Noise Gate (para eliminar zumbido da entrada desbalanceada do PC), pelo amplificador valvulado White-Box e pelo simulador de gabinete com resposta ao impulso (IR).
5. Um servidor local leve restrito ao loopback (`127.0.0.1`) é iniciado e abre uma janela desktop de aplicativo nativa (modo App do Windows) contendo o painel completo do simulador, controles giratórios 3D, osciloscópio CRT e afinador.
6. A interface reflete em tempo real os níveis de entrada/saída (VU meters) e a telemetria do sinal da guitarra sem congelar ou degradar a performance de áudio.
7. Caso o microfone seja desconectado ou indisponível, o sistema silencia a saída graciosamente (zero-fill), sem ruído explosivo nem falha crítica (*panic*).

## Critérios de aceite

- **CA-1: Execução Duplo-Clique sem CLI**: O usuário no Windows pode iniciar a aplicação completa clicando duas vezes no arquivo executável (`SonixPro.exe` ou `INICIAR_SONIX_PRO.bat`) na raiz do projeto, abrindo diretamente a janela da aplicação sem necessidade de abrir o terminal nem rodar comandos manuais.
- **CA-2: Enumeração e Seleção de Dispositivo Físico WASAPI**: O host de áudio detecta e inicializa o dispositivo físico de entrada de áudio padrão (Microfone P2/P3 / Line-In) e o dispositivo padrão de saída através do backend de áudio nativo do sistema operacional.
- **CA-3: Duplex Real-Time Lock-Free com Ring Buffer SPSC**: O streaming duplex entre o callback de captura e o callback de reprodução é intermediado por uma estrutura lock-free SPSC (`ringbuf`), garantindo ausência de bloqueios (mutexes) e zero alocações na thread de áudio.
- **CA-4: Cadeia DSP Ativa em Tempo Real**: O sinal da guitarra é processado em tempo real por Noise Gate (supressão de ruído da entrada P2), Amplificador Valvulado White-Box (ganho, EQ de 3 bandas, master) e Convolução de Gabinete IR, com controle de parâmetros dinâmicos em tempo de execução.
- **CA-5: Janela Desktop Dedicada e Servidor Local Isolado**: A aplicação inicializa um servidor local IPC/HTTP restrito a `127.0.0.1` e lança uma janela de desktop dedicada sem abas, barras de navegação ou controles de navegador web.
- **CA-6: Telemetria em Tempo Real (VU / Osciloscópio CRT / Afinador)**: O host disponibiliza leituras atômicas de nível de pico (dBFS) de entrada e saída e buffer de forma de onda para alimentar a interface visual sem bloquear o processamento de áudio.
- **CA-7: Tratamento de Falhas e Desconexão Graciosa**: Em caso de desconexão, underrun ou ausência de microfone, o host zera o buffer de saída (silêncio) sem estalos sonoros (`panic` ou ruído branco estourado) e relata o estado de erro.
- **CA-8: Não-regressão das Specs Anteriores**: Todas as 106 asserções de contrato das especificações 001 a 012 continuam 100% íntegras e passando.

## Fora de escopo
- Instalação ou compilação forçada de drivers proprietários ASIO de terceiros em máquinas onde o WASAPI de baixa latência nativo do Windows já opera satisfatoriamente.
- Empacotamento em instalador MSI/MSIX para a Microsoft Store.
- Processamento em rede externa ou nuvem (o simulador é 100% offline).

## Invariantes tocadas
- Invariante de áudio em tempo real (Spec 001): zero alocações dinâmicas de heap e ausência de travas (lock-free) no callback de áudio.
- Invariante de segurança (Spec 012): bind de rede estritamente local em `127.0.0.1`, rejeitando conexões externas.

## Testes que cobrem esta spec
- `crates/sonix-desktop/tests/contract_013_desktop_realtime_audio.rs` → CA-1 a CA-8 (100% aprovados)
- `tests/contract/013-execucao-desktop-windows-captura-fisica-p2-duplo-clique.test.js` → CA-1 a CA-8 (100% aprovados)

## Notas
- A spec é acompanhada de teste de contrato automatizado estrito (TDD red → green) e da geração do artefato executável para Windows.
