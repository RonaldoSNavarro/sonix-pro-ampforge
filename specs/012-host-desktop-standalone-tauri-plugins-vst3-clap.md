---
spec: 012
titulo: Arquitetura Desktop nativa (Tauri v2 + Rust) e plugins de DAW (VST3 e CLAP)
status: implementada
atualizado: 2026-11-08
---
# 012 — Arquitetura Desktop nativa (Tauri v2 + Rust) e plugins de DAW (VST3 e CLAP)

## Objetivo
Entregar o **SONIX PRO // AMPFORGE™** exclusivamente como aplicação **Desktop Standalone nativa** (`Tauri v2 + Rust`) e como **Plugins de DAW de 64 bits** (`VST3` e `CLAP`), garantindo acesso direto a drivers de áudio de ultrabaixa latência, controladores MIDI USB físicos e integração com projetos de DAW, sem qualquer dependência de navegador Web.

## Contexto
O protótipo visual inicial em `design-system/` foi construído como página HTML/CSS/JS estática executada via `python3 -m http.server` no navegador (`README.md:28-36`). Contudo, um processador profissional de guitarra/baixo com latência algorítmica `< 1,5 ms` (**RNF-001**), acesso exclusivo a interfaces de áudio (`ASIO`, `WASAPI`, `CoreAudio`, `ALSA`, `JACK`), leitura direta do sistema de arquivos (`.sonixpreset`, `.sforgepkg`, `.wav`) e inserção em pistas de gravação profissional (`VST3` / `CLAP`) exige uma arquitetura 100% Desktop nativa.

Além disso, na consolidação de requisitos ficou definido que:
1. Todas as lacunas entre o protótipo `design-system/` e as especificações formais seguem estritamente o que está definido em `docs/01-SRS-Levantamento-de-Requisitos.md` e `docs/02-Design-System-Specification.md`.
2. A aplicação é **estritamente Desktop** (`Tauri v2 + Rust` para o executável Standalone e `VST3` + `CLAP` para os plugins de DAW), ficando a versão Web/Browser fora de escopo.

Fontes da verdade e evidências no repositório:
- `docs/01-SRS-Levantamento-de-Requisitos.md:7,18-22,29-80` — Define o motor nativo **AmpForge™ DSP Core Engine** em **Rust**, a camada de hardware de áudio (`ASIO / WASAPI / CoreAudio / ALSA / JACK / Host DAW`), MIDI USB e a camada de apresentação Desktop (`Tauri v2 / Plugin WebView`).
- `docs/02-Design-System-Specification.md:1-138` — Define o Design System completo em CSS Tokens (`:root`), tipografia local e Web Components (`<sonix-knob>`, `<ampforge-vacuum-tube>`, `<ampforge-crt-scope>`, `<sonix-cab-mics>`, `<sonix-footswitch>`) renderizados na WebView nativa da janela Desktop e do Plugin.
- `README.md:28-36` — Mostra que o repositório atual possui apenas a execução estática do protótipo via servidor HTTP, sem o workspace Rust (`Cargo.toml`), sem o host `Tauri v2` e sem os alvos `VST3`/`CLAP`.

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Plataforma de execução | Protótipo aberto em navegador via `python3 -m http.server` (`README.md:34-35`) | Aplicativo **Desktop Standalone** nativo (`Tauri v2 + Rust`) e **Plugins 64-bit** (`VST3` e `CLAP`), sem versão Web |
| Drivers de Áudio (Standalone) | Oscilador JS simulado em `requestAnimationFrame` (`design-system/app.js:310`) | Host de áudio nativo em Rust (`cpal`/drivers nativos) com suporte a `ASIO`, `WASAPI Exclusive/Shared`, `CoreAudio`, `ALSA` e `JACK` |
| Hardware MIDI / Pedal Expressão | Slider HTML `#expPedalInput` (`design-system/index.html:518`) | Entrada MIDI nativa (`midir` no Standalone e eventos MIDI/Note/CC da DAW nos plugins) alimentando a `SPSC Command Queue` |
| Integração com DAW (`VST3` / `CLAP`) | Inexistente | Empacotamento `.vst3` e `.clap` (`nih-plug` / wrapper Rust) com automação de parâmetros sample-accurate, sincronia de BPM com o transporte da DAW e persistência do estado na sessão da DAW |
| Ponte UI ↔ Backend Rust | Estado JS em memória (`design-system/app.js:6-71`) | IPC nativo Tauri / canal de WebView embutida enviando comandos tipados para a `SPSC Command Queue` e recebendo snapshots binários da `SPSC Telemetry Queue` |

**Alternativas descartadas:**
1. *Manter suporte simultâneo a aplicação Web (WebAssembly + AudioWorklet) nesta fase:* descartada por decisão explícita de produto ("a aplicação deverá ser desktop, sem web por enquanto"), evitando restrições de sandbox de navegador, latência extra de WebAudio e duplicação de camadas de persistência.
2. *Reescrever a interface do Design System em widgets nativos C++/Qt ou JUCE puro descartando o CSS/HTML existente:* descartada porque toda a especificação oficial em `docs/02-Design-System-Specification.md` (gradientes cônicos, tokens `--surface-*`, `--led-*`, `--shadow-*` e Web Components) já é projetada sobre motor de renderização CSS/SVG acelerado por GPU, que roda nativamente dentro do `Tauri v2` (Standalone) e de WebView embutida nos plugins `VST3`/`CLAP`.
3. *Electron (Chromium + Node.js completo):* descartada pelo alto consumo de RAM/disco e overhead de processos quando múltiplos plugins `VST3`/`CLAP` são abertos em uma mesma sessão de DAW; `Tauri v2 + Rust` gera binários enxutos e compartilha diretamente o processo Rust com o motor DSP.

## Comportamento esperado
1. **Arquitetura de Workspace em Rust (3 Crates Compartilhados):**
   - `ampforge-dsp`: biblioteca pura em Rust contendo todo o núcleo de processamento White-Box, DAG 4×12, convolução particionada zero-latency, afinador, looper, filas lock-free (`SPSC`) e serialização `.sonixpreset` / `.sforgepkg` (**Specs 001 a 010**);
   - `sonix-desktop`: aplicativo Desktop Standalone em **Tauri v2 + Rust** que gerencia a janela nativa (`1920×1080` responsiva), o host de áudio nativo (`ASIO / WASAPI / CoreAudio / ALSA / JACK`), as portas físicas MIDI USB e o acesso ao sistema de arquivos local;
   - `sonix-plugin`: alvos de plugin de áudio **VST3** e **CLAP** (64-bit) que expõem o motor `ampforge-dsp` e a janela de editor com o Design System (`docs/02-Design-System-Specification.md`) para qualquer DAW compatível.
2. **Host de Áudio e MIDI Nativo no Modo Desktop Standalone:**
   - Permite enumerar e selecionar o driver/dispositivo de entrada e saída de áudio do sistema operacional, configurar a taxa de amostragem (`44100`, `48000`, `88200`, `96000 Hz`), o tamanho do buffer (`32`, `64`, `128`, `256`, `512` samples) e os canais físicos roteados para as 4 linhas do Grid.
   - Permite enumerar e conectar dispositivos MIDI USB/físicos, convertendo mensagens `Control Change (CC 0–127)`, `Program Change` (troca de Preset/Scene) e `MIDI Clock` em comandos lock-free na `SPSC Command Queue`.
3. **Execução como Plugin de DAW (`VST3` e `CLAP`):**
   - O binário `.vst3` e `.clap` processa buffers de áudio estéreo (e sidechain/multicanal quando roteado) diretamente no callback de tempo real da DAW usando `ampforge-dsp`.
   - Expõe os parâmetros do painel frontal, Mod Shop, blocos de efeito e macros de Modifiers para automação da DAW (com suporte a modulação polifônica/não-destrutiva no `CLAP` e automação sample-accurate no `VST3`/`CLAP`).
   - Quando o modo de sincronia de tempo com o host (`hostTempoSync`) está ativo no plugin, o `tempoBpm` do preset acompanha dinamicamente o BPM e o estado de transporte da DAW.
   - Salvar o projeto na DAW grava o estado completo do preset ativo no chunk de estado do plugin (`getState` / `setState`) de forma 100% compatível com o schema `.sonixpreset`.
4. **Resolução Canônica das Lacunas do Protótipo:**
   - A interface Desktop empacotada carrega os assets de `design-system/` corrigidos segundo `docs/01-SRS-Levantamento-de-Requisitos.md` e `docs/02-Design-System-Specification.md` (Grid 4×12 de 48 slots, Sala de Cab com distância `0–30 cm` e ângulo `0°/45°`, fontes locais offline sem Google Fonts CDN, schema `.sonixpreset`/`.sforgepkg` completo e todos os tokens `:root` canônicos).

## Critérios de aceite
- **CA-1:** O modo Desktop Standalone (`sonix-desktop` / `Tauri v2`) enumera os dispositivos de áudio nativos do sistema operacional (`WASAPI`/`ASIO` no Windows, `CoreAudio` no macOS, `ALSA`/`JACK` no Linux) e inicia o stream full-duplex nas taxas de `44.1 kHz` a `96 kHz` e buffers de `32` a `512` samples alimentando `ampforge-dsp`.
- **CA-2:** O modo Desktop Standalone enumera portas de entrada MIDI nativas e encaminha mensagens `MIDI CC (0..127)` e `Program Change` recebidas para a `SPSC Command Queue` sem realizar alocação de memória na thread de áudio.
- **CA-3:** O build do projeto gera artefatos válidos de plugin `VST3` e `CLAP` de 64 bits (`sonix-plugin`) que inicializam, reportam seus descritores de áudio/MIDI/parâmetros ao host e processam blocos de áudio através do mesmo núcleo `ampforge-dsp`.
- **CA-4:** Nos formatos `VST3` e `CLAP`, salvar o estado do plugin pelo host DAW (`state save`) e restaurá-lo (`state restore`) preserva integralmente todas as 8 Scenes, os 4 canais por bloco, parâmetros White-Box, microfones do gabinete e Modifiers (equivalência exata com o documento `.sonixpreset`).
- **CA-5:** Nos formatos `VST3` e `CLAP`, com `hostTempoSync = true`, uma mudança de BPM no transporte da DAW (ex.: de `120.0 BPM` para `140.0 BPM`) atualiza automaticamente `tempoBpm` no motor DSP e na UI, recalculando o tempo dos blocos `MOD`/`DELAY` sincronizados por `Note Division` e a frequência de pulsação do footswitch `TAP TEMPO`.
- **CA-6:** A aplicação Desktop Standalone e os plugins `VST3`/`CLAP` renderizam a interface do Design System localmente via WebView nativa acoplada por IPC ao backend Rust, sem expor servidor HTTP de rede e sem depender de navegador Web externo.
- **CA-7:** Caso de erro — se o dispositivo de áudio nativo for desconectado abruptamente ou falhar ao abrir no modo Standalone, o host captura a falha (`AudioDeviceDisconnectedError` / `AudioDriverInitError`), zera os buffers de saída sem produzir ruído explosivo, mantém o estado do preset intacto na memória/disco e exibe alerta de reconexão na UI sem abortar (*panic/crash*) o aplicativo Desktop.
- **CA-8:** Não-regressão — um arquivo `.sonixpreset` ou pacote `.sforgepkg` exportado na versão Desktop Standalone é importado nos plugins `VST3` e `CLAP` (e vice-versa) produzindo exatamente o mesmo grafo DSP e a mesma saída de áudio para o mesmo sinal de entrada.

## Fora de escopo
- Compilação, hospedagem ou execução da aplicação em navegadores Web (`WebAssembly`, `AudioWorklet`, `WebAudio`, PWA).
- Formato de plugin `AU (Audio Units)` ou `AAX` nesta primeira entrega (restrito a `Desktop Standalone`, `VST3` e `CLAP`).
- Compilação para hardware embarcado Linux RT / ARM64 dedicado.

## Invariantes tocadas
- **RNF-001 (Latência < 1,5 ms):** Os callbacks de áudio de `cpal`/drivers nativos (Standalone) e de `VST3`/`CLAP` invocam diretamente `ampforge-dsp` sem camada intermediária de navegador.
- **RNF-005 (Real-Time Safe):** A ponte entre os callbacks nativos de áudio (`ASIO`/`CoreAudio`/`ALSA`/`VST3`/`CLAP`) e a UI `Tauri v2`/WebView utiliza estritamente as filas lock-free `SPSC` e `RCU` definidas na **Spec 001**.
- **RNF-009 (100% Offline):** A janela Desktop e os plugins carregam todos os recursos (fontes, ícones SVG, presets de fábrica, IRs) do pacote binário local e do sistema de arquivos do SO.

## Testes que cobrem esta spec
- `crates/sonix-desktop/tests/contract_012_desktop_plugin.rs` → CA-1 a CA-8 (8 testes unitários de contrato em Rust, 100% ok).
- `tests/contract/012-host-desktop-standalone-tauri-plugins-vst3-clap.test.js` → Suíte de integração e validação de critérios CA-1 a CA-8.

## Notas
- Rastreabilidade SRS: Seção 1.1.2, Seção 2.1 (`Camada de Hardware / Host Audio & DAW`, `Camada de Apresentação & Controle`), **RNF-001**, **RNF-002**, **RNF-005**, **RNF-008**, **RNF-009**.
