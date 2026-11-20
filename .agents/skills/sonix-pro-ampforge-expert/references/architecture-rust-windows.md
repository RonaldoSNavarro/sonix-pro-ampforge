# SONIX PRO // AMPFORGE™ — Arquitetura Efetiva (Rust + Tauri v2 + Windows)

Este documento descreve a **arquitetura efetivamente implementada** no repositório SONIX PRO // AMPFORGE™ (conferida em HEAD `d891389` / 2026-11-20) e sua direção de evolução modular auditada (Specs 053–092), confrontada diretamente com `Cargo.toml`, `Cargo.lock`, `tauri.conf.json` e o código-fonte vigente.

## 1. Perfil Confirmado do Produto vs. Estado Efetivo

| Dimensão | Decisão de Produto Confirmada | Estado Efetivo no Repositório (2026-11-20) |
|---|---|---|
| Linguagem núcleo | Rust | Rust edition 2021 (`rustc 1.99.1`), licença `CC-BY-NC-4.0`. |
| App Desktop | Standalone Windows 10/11 | **Tauri v2** (`tauri 2.12.5`, `tauri-build 2.7.4`) + Vite 8.3.5 + TypeScript 7.0.2 em `apps/sonix-app/`, com `custom-protocol` no release (zero HTTP/TCP). |
| Áudio Windows | WASAPI e ASIO no primeiro release | `cpal 0.18.2`. WASAPI Shared / Shared Low-Latency ativos por padrão (`audio.rs`); **ASIO** compilável via feature `asio` (`CPAL_ASIO_DIR` + LLVM/Clang, Spec 049 CA-36 `[verificação pendente]` em hardware/SDK/licenciamento); `WasapiExclusive` reportado honestamente como `false` em `has_mode_support` até integração nativa dedicada. |
| Taxas e Buffers | Múltiplas taxas negociadas | `44100`, `48000`, `88200`, `96000` Hz; períodos de `32..=2048` frames (incl. 441/480/882/960 do WASAPI compartilhado) com fallback seguro. |
| Entrada / DI / Reamp | Hi-Z/Linha, DI limpa e Reamp | Modos `instrument` / `line` (`input.mode`), fontes `device`/`test_riff`/`di_file`, calibração (`calibration.rs`) e alinhamento loopback (`reamp.rs`). |
| Modelagem DSP | Híbrida (circuit-level + behavioral + IR) | Tone stacks KCL 5×5 + bilinear (`tonestack.rs`), estágios ADAA1 (`amp.rs`), drives 8× OS (`fx/drive.rs`), wah SVF (`fx/wah.rs`), convolução particionada zero-latency (`cab.rs`). |
| Plugins DAW | Núcleo desacoplado reutilizável | `crates/sonix-plugin` expõe alvos **VST3 e CLAP** via `nih_plug` sobre `RealtimeDspChain` (validadores externos pendentes). |

> **Nota de auditoria:** Versões antigas citavam `egui`/`eframe` e `JUCE`. Nenhum dos dois faz parte da stack do projeto. O frontend oficial é Tauri v2 + TypeScript em `apps/sonix-app/ui/` (a pasta `design-system/` é legada, servida apenas pelo host HTTP sob feature `dev-http`).

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
│   │       ├── amp.rs                # 5 amps White-Box (1738 linhas; evolução modular: Specs 054–058)
│   │       ├── tonestack.rs          # Redes passivas KCL 5×5 + bilinear f64 (401 linhas; Spec 078)
│   │       ├── modshop.rs            # Válvulas, retificadores, bias, CRT bancada (506 linhas; Spec 077)
│   │       ├── cab.rs                # Convolução particionada FIR+FFT, WAV IR, 7 mics (1206 linhas; Spec 080)
│   │       ├── oversampling.rs       # HalfBandOversampler polifásico Kaiser 1x/2x/4x/8x (316 linhas; Spec 079)
│   │       ├── precision.rs          # IirBiquad64 e solver Newton-Raphson (102 linhas; Spec 077)
│   │       ├── dag.rs                # Matriz 4×12, SplitNode, MixerNode (529 linhas; Spec 084)
│   │       ├── queue.rs              # SpscBuffer, SpscOwnedQueue lock-free (275 linhas; Spec 084)
│   │       ├── smoother.rs           # OnePoleSmoother (5–20 ms)
│   │       ├── tuner.rs / looper.rs  # Afinador YIN/Polifônico (493 l.; Spec 083) e Looper (275 l.; Spec 092)
│   │       ├── config.rs / cpu.rs / errors.rs / engine.rs
│   │       └── fx/                   # drive.rs (689 l.), dynamics.rs (552 l.), eq.rs (229 l.), modulation.rs (297 l.), delay.rs (559 l.), reverb.rs (337 l.), pitch.rs (416 l.), wah.rs (210 l.)
│   ├── sonix-core/                   # Domínio de presets, cenas, canais, modifiers, footswitches, .sonixpreset/.sforgepkg
│   ├── sonix-desktop/                # Host cpal, RealtimeDspChain (audio.rs: 5190 l.), api::dispatch (api.rs: 5066 l.), calibration, midi, reamp
│   └── sonix-plugin/                 # Plugins DAW VST3 e CLAP via nih_plug sobre RealtimeDspChain
├── apps/sonix-app/
│   ├── src-tauri/                    # Shell Tauri v2 (commands.rs, streams.rs, lib.rs, capabilities/main.json)
│   └── ui/                           # Frontend Vite + TypeScript (controls.registry.ts, transport.ts, store.ts, screens/)
├── specs/                            # Especificações SDD (001–052 implementadas; 053–092 arquitetura modular em rascunho)
├── tests/contract/                   # Contratos JS (Node test runner + fixtures IPC + regressão visual SSIM)
└── docs/                             # Fontes de manuais PDF, voicing-fontes.md, cab-microfones-fontes.md, handoffs
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
│ Preparação off-RT (prepare(), WAV/IR + resampling sinc, DAG compile) │
│ Coleta de recursos aposentados (RetiredReaper off-RT)                │
└───────────────────────────────────┬──────────────────────────────────┘
                                    │ LockFreeParamQueue / LockFreeResourceQueue
                                    ▼
┌──────────────────────────────────────────────────────────────────────┐
│ Real-Time Audio Thread — cpal callback → RealtimeDspChain::process   │
│ ZERO heap alloc/dealloc • ZERO Mutex/RwLock • ZERO I/O • ZERO panic  │
├──────────────────────────────────────────────────────────────────────┤
│ Input/ADC Clip → DC Blocker f64 → CleanInputRing (Tuner) →           │
│ Looper(Pre) → DAG 4×12 (Gate → Wah → Pitch → Drive → Split A/B →     │
│ Amp1/Amp2 → Cab1/Cab2 → Comp2/Mod2/EQ1 → Merge) →                    │
│ Delay1 → Reverb1 → Looper(Post) → Master Trim → Safe Limiter ≤0 dBFS │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 4. Direção de Evolução Modular Auditada (Specs 053–092)

A auditoria documental e arquitetural separou estritamente equipamentos físicos reais de módulos algorítmicos:
1. **Arquitetura central (`053`):** Contratos `AudioProcessor`, `ModelDescriptor`, `ProcessorRegistry`, `ProcessContext`, separação explícita entre preparação na thread de controle (off-RT) e execução zero-alloc na thread de áudio (RT).
2. **Amplificadores físicos individuais (`054–058`):** Marshall 1959 Super Lead (`054`), Marshall JCM800 2203 (`055`), Fender Twin Reverb AB763 (`056`), Mesa/Boogie Dual Rectifier Rev F (`057` — com itens pendentes de esquema primário bloqueados), Vox AC30/6 Top Boost (`058`).
3. **Pedais, Racks e Módulos Algorítmicos individuais (`059–075` e `085–091`):**
   - *Equipamentos físicos:* TS808 (`059`), Klon Centaur (`060`), ProCo RAT LM308N (`061`), EHX Big Muff Pi V3 (`062`), Arbiter Fuzz Face Ge NKT275/AC128 (`063`), Boss DS-1 TA7136AP (`064`), UREI 1176LN Rev D/E (`066`), MXR M108 10-Band EQ (`067`), Mesa Mark 5-Band Graphic EQ (`085`), Roland RE-201 Space Echo (`068`), Boss CE-2 Chorus (`069`), MXR Phase 90 Script/Block (`071`), Shin-Ei Uni-Vibe (`072`), Leslie 147 Rotary (`074`), DigiTech Whammy (`075`), DigiTech Drop (`089`), Eventide H3000 MicroPitch (`091`).
   - *Módulos algorítmicos (`tipo: modulo-dsp`):* Noise Gate LR4 (`065`), EQ Paramétrico 8-Band (`086`), Stereo Tri-Chorus (`087`), Through-Zero Flanger (`070`), Tremolo Óptico (`073`), Tremolo por Modulação de Bias (`088`), Harmonizer Diatônico (`090`).
4. **Componentes compartilhados (`076–084` e `092`):** Entrada/Impedância (`076`), Submodelos Valvulados/Potência/Fonte/Carga (`077`), Redes Passivas de Tone Stack Separadas (`078`), Oversampling Seletivo (`079`), Gabinete/IR/Microfones (`080`), Motor de Delay Algorítmico (`081`), Arquiteturas Distintas de Reverb (`082`), Afinador YIN/Polifônico (`083`), Looper de Performance (`092`), Roteamento DAG 4×12 e Filas SPSC (`084`).

---

## 5. Fontes Primárias da Stack

- **Tauri v2 Documentation:** https://v2.tauri.app/
- **CPAL (RustAudio WASAPI / ASIO):** https://docs.rs/cpal/0.18.2/cpal/
- **Steinberg Developer Portal:** https://www.steinberg.net/developers/
- **NIH-plug (VST3/CLAP em Rust):** https://github.com/robbert-vdh/nih-plug
- **CLAP Audio Plugin API:** https://github.com/free-audio/clap
