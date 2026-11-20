# SONIX PRO // AMPFORGE™ — Arquitetura Efetiva (Rust + Tauri v2 + Windows)

Este documento descreve a **arquitetura efetivamente implementada** no repositório SONIX PRO // AMPFORGE™ e sua direção de evolução modular (Spec 053), confrontada diretamente com `Cargo.toml`, `Cargo.lock`, `tauri.conf.json` e o código-fonte vigente.

## 1. Perfil Confirmado do Produto vs. Estado Efetivo

| Dimensão | Decisão de Produto Confirmada | Estado Efetivo no Repositório (2026-11-19) |
|---|---|---|
| Linguagem núcleo | Rust | Rust edition 2021 (`rustc 1.99.1`), licença `CC-BY-NC-4.0`. |
| App Desktop | Standalone Windows 10/11 | **Tauri v2** (`tauri 2.12.5`, `tauri-build 2.7.4`) + Vite 8.3.5 + TypeScript 7.0.2 em `apps/sonix-app/`, com `custom-protocol` no release (zero HTTP/TCP). |
| Áudio Windows | WASAPI e ASIO no primeiro release | `cpal 0.18.2`. WASAPI Shared / Shared Low-Latency ativos por padrão (`audio.rs`); **ASIO** compilável via feature `asio` (`CPAL_ASIO_DIR` + LLVM/Clang, Spec 049 CA-36 `[verificação pendente]` em hardware/SDK/licenciamento); `WasapiExclusive` reportado honestamente como `false` em `has_mode_support` até integração nativa dedicada. |
| Taxas e Buffers | Múltiplas taxas negociadas | `44100`, `48000`, `88200`, `96000` Hz; períodos de `32..=2048` frames (incl. 441/480/882/960 do WASAPI compartilhado) com fallback seguro. |
| Entrada / DI / Reamp | Hi-Z/Linha, DI limpa e Reamp | Modos `instrument` / `line` (`input.mode`), fontes `device`/`test_riff`/`di_file`, calibração (`calibration.rs`) e alinhamento loopback (`reamp.rs`). |
| Modelagem DSP | Híbrida (circuit-level + behavioral + IR) | Tone stacks KCL 5×5 + bilinear (`tonestack.rs`), estágios ADAA1 (`amp.rs`), drives 8× OS (`fx/drive.rs`), convolução particionada zero-latency (`cab.rs`). |
| Plugins DAW | Núcleo desacoplado reutilizável | `crates/sonix-plugin` já entrega alvos **VST3 e CLAP** via `nih_plug` sobre `RealtimeDspChain` (validadores externos pendentes). |

> **Nota de auditoria (2026-11-19):** Versões anteriores desta referência citavam `egui`/`eframe` e `JUCE`. Nenhum dos dois faz parte da stack do projeto. O frontend oficial é Tauri v2 + TypeScript em `apps/sonix-app/ui/` (a pasta `design-system/` é legada, servida apenas pelo host HTTP sob feature `dev-http`).

---

## 2. Estrutura Real do Workspace Cargo

```text
Simulador de efeitos/
├── Cargo.toml                        # Workspace Rust (resolver = "2", edition = "2021")
├── Cargo.lock                        # Versões fixadas (tauri 2.12.5, cpal 0.18.2, ringbuf 0.4.8)
├── .cargo/config.toml                # target-dir fora do caminho com espaços (windres GNU)
├── crates/
│   ├── ampforge-dsp/                 # Núcleo DSP puro (sem I/O, sem Tauri, sem cpal)
│   │   └── src/
│   │       ├── amp.rs                # 5 amps White-Box (em evolução modular: Specs 054–058)
│   │       ├── tonestack.rs          # Redes passivas KCL 5×5 + bilinear f64 (Spec 078)
│   │       ├── modshop.rs            # Válvulas, retificadores, bias, CRT bancada (Spec 077)
│   │       ├── cab.rs                # Convolução particionada FIR+FFT, WAV IR, 7 mics (Spec 080)
│   │       ├── oversampling.rs       # HalfBandOversampler polifásico Kaiser 1x/2x/4x/8x (Spec 079)
│   │       ├── precision.rs          # IirBiquad64 e solver Newton-Raphson (Spec 077)
│   │       ├── dag.rs                # Matriz 4×12, SplitNode, MixerNode (Spec 084)
│   │       ├── queue.rs              # SpscBuffer, SpscOwnedQueue lock-free (Spec 084)
│   │       ├── smoother.rs           # OnePoleSmoother (5–20 ms)
│   │       ├── tuner.rs / looper.rs  # Afinador YIN/Polifônico e Looper O(1) (Spec 083)
│   │       ├── config.rs / cpu.rs / errors.rs / engine.rs
│   │       └── fx/                   # drive.rs, dynamics.rs, eq.rs, modulation.rs, delay.rs, reverb.rs, pitch.rs
│   ├── sonix-core/                   # Domínio de presets, cenas, canais, modifiers, footswitches, .sonixpreset/.sforgepkg
│   ├── sonix-desktop/                # Host cpal, RealtimeDspChain, api::dispatch, calibration, midi, reamp, server (dev-http)
│   └── sonix-plugin/                 # Plugins DAW VST3 e CLAP via nih_plug sobre RealtimeDspChain
├── apps/sonix-app/
│   ├── src-tauri/                    # Shell Tauri v2 (commands.rs, streams.rs, lib.rs, capabilities/main.json)
│   └── ui/                           # Frontend Vite + TypeScript (controls.registry.ts, transport.ts, store.ts, screens/)
├── specs/                            # Especificações SDD (001–052 existentes; 053–084 arquitetura modular)
├── tests/contract/                   # Contratos JS (Node test runner + fixtures IPC + regressão visual SSIM)
└── docs/                             # Fontes de manuais, voicing-fontes.md, cab-microfones-fontes.md, handoffs
```

---

## 3. Visão em Camadas e Fluxo de Dados

```text
┌──────────────────────────────────────────────────────────────────────┐
│ UI Frontend — apps/sonix-app/ui (Vite + TypeScript, sem framework)   │
│ Telas (Estúdio, Chassi, Gabinete, Palco, Biblioteca, Áudio)          │
│ controls.registry.ts (registro único) → transport.ts (ParamCoalescer)│
└───────────────────────────────────┬──────────────────────────────────┘
                                    │ Tauri v2 IPC (invoke / Channel ~30 Hz)
                                    ▼
┌──────────────────────────────────────────────────────────────────────┐
│ Camada Tauri v2 — apps/sonix-app/src-tauri (commands.rs, streams.rs) │
│ Validação de borda (E_VALIDATION/E_ENGINE) • Managed AppRuntime      │
│ Sem processamento DSP • Sem HTTP/TCP no release (custom-protocol)    │
└───────────────────────────────────┬──────────────────────────────────┘
                                    │ chamada direta em memória
                                    ▼
┌──────────────────────────────────────────────────────────────────────┐
│ Control Plane — crates/sonix-desktop (api.rs, calibration, reamp)    │
│ api::dispatch (tabela única Spec 014, validação atômica)             │
│ Carregamento WAV/IR + resampling sinc • Compilação DAG/Modifiers     │
│ Coleta de recursos aposentados (RetiredReaper off-RT)                │
└───────────────────────────────────┬──────────────────────────────────┘
                                    │ LockFreeParamQueue / LockFreeResourceQueue
                                    ▼
┌──────────────────────────────────────────────────────────────────────┐
│ Real-Time Audio Thread — cpal callback → RealtimeDspChain::process   │
│ ZERO heap alloc/dealloc • ZERO Mutex/RwLock • ZERO I/O • ZERO panic  │
├──────────────────────────────────────────────────────────────────────┤
│ Input/ADC Clip → DC Blocker f64 → CleanInputRing (Tuner) →           │
│ Looper(Pre) → DAG 4×12 (Gate → Pitch → Drive → Split A/B →           │
│ Amp1/Amp2 → Cab1/Cab2 → Comp2/Mod2/EQ1 → Merge) →                    │
│ Delay1 → Reverb1 → Looper(Post) → Master Trim → Safe Limiter ≤0 dBFS │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 4. Dependências Fixadas (Conferidas em `Cargo.lock`)

| Crate / Pacote | Versão Fixada | Camada | Papel |
|---|---|---|---|
| `tauri` / `tauri-build` | `2.12.5` / `2.7.4` | `sonix-app` | Runtime desktop, IPC, `Channel`, capabilities |
| `tauri-plugin-{single-instance,window-state,dialog,fs}` | `2.5.5` / `2.5.0` / `2.8.4` / `2.7.1` | `sonix-app` | Instância única, janela, diálogos nativos, arquivos |
| `@tauri-apps/api` / `vite` / `typescript` | `2.12.4` / `8.3.5` / `7.0.2` | `ui` | Ponte JS/TS e bundle estático embutido |
| `cpal` | `0.18.2` | `sonix-desktop` | Enumeração e streams WASAPI + feature `asio` |
| `ringbuf` | `0.4.8` | `sonix-desktop` | Ring buffers auxiliares de áudio |
| `serde` / `serde_json` | `1.0.229` / `1.0.154` | Workspace | DTOs IPC e presets `.sonixpreset` (somente fora da RT) |
| `thiserror` | `1.0.69` | Workspace | Tipos de erro estruturados |
| `sha2` / `getrandom` | `0.10.9` / `0.3.4` | `sonix-core` / `sonix-desktop` | Integridade `.sforgepkg` e token CSPRNG (`dev-http`) |
| `nih_plug` | git (`robbert-vdh/nih-plug`) | `sonix-plugin` | Exportação VST3 e CLAP |

---

## 5. Direção de Evolução Modular (Spec 053)

O diagnóstico do código atual identificou acoplamentos a serem desfeitos de forma incremental e orientada por specs:
1. **`amp.rs` (1738 linhas)** concentra os 5 amplificadores numa única struct `WhiteBoxAmpProcessor` com `match self.model_id` — evoluirá para módulos independentes por amplificador (Specs 054–058) consumindo blocos compartilhados (Specs 077–079).
2. **`fx/drive.rs`, `fx/modulation.rs`, `fx/delay.rs`, `fx/reverb.rs`, `fx/dynamics.rs`** agrupam múltiplos modelos em processadores monolíticos — evoluirão para implementações independentes por pedal/modelo (Specs 059–075) integradas por traits e registries estáveis (Spec 053).
3. **`RealtimeDspChain` (`sonix-desktop/src/audio.rs`, 5093 linhas)** acopla a orquestração DSP ao crate de host de áudio — o grafo executável e os contratos de processador convergirão para a camada DSP pura (`ampforge-dsp`), mantendo `sonix-desktop` focado em host de áudio/MIDI/IPC.
4. **`BiquadFilter` duplicado** em `amp.rs`, `cab.rs`, `fx/drive.rs`, `fx/dynamics.rs`, `fx/delay.rs` e `precision.rs` — convergirá para primitivas compartilhadas em `f64` (TDF-II) com política única de estabilidade.

---

## 6. Fontes Primárias da Stack

- **Tauri v2 Documentation (Capabilities, IPC, Channels, State):** https://v2.tauri.app/
- **CPAL (RustAudio WASAPI / ASIO):** https://docs.rs/cpal/0.18.2/cpal/ e https://github.com/RustAudio/cpal
- **Steinberg Developer Portal (ASIO / VST3 SDK & Licensing):** https://www.steinberg.net/developers/
- **NIH-plug (VST3/CLAP em Rust):** https://github.com/robbert-vdh/nih-plug
- **CLAP Audio Plugin API:** https://github.com/free-audio/clap
