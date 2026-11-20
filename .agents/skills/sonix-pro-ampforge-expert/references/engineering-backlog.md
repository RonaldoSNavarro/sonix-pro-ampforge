# SONIX PRO // AMPFORGE™ — Backlog de Engenharia e Estado dos Marcos

Este documento alinha o backlog histórico do projeto com o **estado real do repositório** (conferido em 2026-11-20 contra `specs/README.md` e `AGENTS.md`).

## Decisões de Produto Confirmadas

- Núcleo DSP em Rust puro, desacoplado de GUI e host.
- Aplicação desktop standalone Windows 10 e 11 via **Tauri v2 + Vite/TypeScript** (sem HTTP/TCP no release).
- **WASAPI e ASIO** como requisitos confirmados do produto (WASAPI operacional; ASIO via feature `asio` pendente de validação em hardware/SDK/licenciamento — Spec 049 CA-36).
- Múltiplas taxas de amostragem (`44,1` / `48` / `88,2` / `96` kHz), canais configuráveis, modo Instrument/Line, captura DI e Reamp.
- Modelagem híbrida (circuit-level/nodal + behavioral com memória/ADAA + convolução IR particionada zero-latency).
- Plugins DAW **VST3 e CLAP** (`crates/sonix-plugin` via `nih_plug`) compartilhando o mesmo núcleo DSP.

---

## Marcos Implementados (Specs 001–052)

- [x] **Fundação DSP, Lock-Free SPSC e Tempo Real** (Specs 001, 014, 016, 017, 018, 028, 034, 037, 044): callback zero-alloc/zero-lock verificado por alocador global de contagem, `RetiredReaper` off-RT, defesa NaN/Inf e limite `<= 0 dBFS` na saída final.
- [x] **Shell Desktop Tauri v2, Abertura SVG e Contratos UI↔Backend** (Specs 011, 012, 015, 027, 033, 035, 046, 048, 050, 051, 052): 6 telas fiéis à referência visual (SSIM), splash screen SVG offline, 20 pedais + 3 racks skeuomórficos (incluindo Wah Cry Baby GCB95 e mini-AMP 2 no Estúdio), `controls.registry.ts` sem controles decorativos, `Channel` 30 Hz / 0 Hz minimizado e build release standalone com `custom-protocol`.
- [x] **Host de Áudio, Calibração, MIDI, Reamp e Plugins** (Specs 013, 019, 026, 029, 036, 049): WASAPI duplex, calibração de entrada + riff Karplus-Strong, MIDI real, alinhamento loopback de reamp, binários VST3/CLAP em `sonix-plugin`.
- [x] **Equipamentos e Efeitos — 1ª Geração Integrada** (Specs 002–010, 020–025, 030–032, 038–043, 045, 047): 5 amplificadores, 6 pedais de drive, Wah GCB95, Noise Gate com histerese/LR4, Compressor 1176LN, EQ 8/5/10 bandas, 7 modelos de modulação, RE-201 + delays, 5 reverbs, 4 modos de pitch, afinador YIN/polifônico, looper O(1) e gabinete com convolução IR particionada + 7 microfones.

---

## Fase Modular Auditada — Specs 053–092 (`rascunho`, aguardando revisão e aprovação explícita)

1. **Épico M1 — Contrato Central Modular (`Spec 053`):**
   - Traits `AudioProcessor`, `ModelDescriptor`, `ProcessorRegistry`, `ProcessContext`, separação estrita entre preparação na thread de controle (off-RT) e execução zero-alloc na thread RT.
2. **Épico M2 — Componentes Compartilhados (`Specs 076–084` e `092`):**
   - Entrada/Condicionamento/Impedância RLC de Captador (`076`);
   - Submodelos independentes de Triodo, Pentodo, Cathode Follower, LTP, Power Push-Pull, Bias, Fonte/Sag, Transformador, Carga Reativa e NFB (`077`);
   - Redes Passivas de Tone Stack com Solvers Separados: FMV com $R_{\text{src}}$, Vox Top Boost `OS/010`, James/Baxandall e Mesa Rectifier (`078`);
   - Oversampling Seletivo Polifásico/IIR (`079`);
   - Gabinete Virtual, Convolução IR Zero-Latency e 7 Microfones (`080`);
   - Motor de Delay Algorítmico (Digital, BBD, Dual, Reverse com janela, Ping-Pong, Ducking e Spillover) (`081`);
   - Arquiteturas Distintas de Reverb (Mola Dispersiva, Plate, Hall FDN, Chamber FDN e Shimmer `+12 st`) (`082`);
   - Afinador Estroboscópico YIN e Polifônico Desacoplado da RT (`083`);
   - Looper de Performance sem `memcpy` em RT (`092`);
   - Roteamento DAG 4×12, Split/Merge, PDC, Modifiers e Filas SPSC (`084`).
3. **Épico M3 — Amplificadores Independentes por Circuito (`Specs 054–058`):**
   - Marshall 1959 Super Lead / Plexi rev. 1968–69 SL/A (`054`);
   - Marshall JCM800 2203 rev. 1981 (`055`);
   - Fender Twin Reverb AB763 rev. 1965 (`056`);
   - Mesa/Boogie Dual Rectifier Solo Head Rev F 1992 (`057` — requisitos de valores divergentes `C1`/`R_bass`/Presence Modern bloqueados até confirmação em esquema primário `RF-1F`);
   - Vox AC30/6 Top Boost rev. 1964 JMI (`058`).
4. **Épico M4 — Pedais, Racks e Módulos Algorítmicos Independentes (`Specs 059–075` e `085–091`):**
   - Drives/Fuzz: Ibanez TS808 (`059`), Klon Centaur (`060`), ProCo RAT LM308N (`061`), EHX Big Muff Pi V3 (`062`), Dallas Arbiter Fuzz Face Ge NKT275/AC128 (`063`), Boss DS-1 rev. 1978–93 TA7136AP (`064`);
   - Dinâmica: Módulo Algorítmico Noise Gate LR4 (`065`), Rack UREI 1176LN Rev D/E (`066`);
   - Equalizadores: Pedal MXR M108 10-Band EQ (`067`), EQ Gráfico 5 Bandas Mesa/Boogie Mark (`085`), Módulo EQ Paramétrico 8 Bandas (`086`);
   - Tape Echo: Roland RE-201 Space Echo 1974 (`068`);
   - Modulação e Rotary: Pedal Boss CE-2 Chorus (`069`), Módulo Stereo Tri-Chorus (`087`), Módulo Through-Zero Flanger (`070`), Pedal MXR Phase 90 Script/Block (`071`), Pedal Shin-Ei Uni-Vibe (`072`), Módulo Tremolo Óptico (`073`), Módulo Tremolo por Modulação de Bias (`088`), Gabinete Rotary Leslie 147 (`074`);
   - Pitch: Pedal DigiTech Whammy (`075`), Pedal DigiTech Drop (`089`), Módulo Harmonizer Diatônico (`090`), Rack Eventide H3000 MicroPitch (`091`).

---

## Pendências de Hardware, SDK e Validação Documental Externa

- **Esquemas Primários Pendentes (`BLOCKED` / `PENDING`):** confirmação em esquema primário de fábrica para o Mesa Dual Rectifier Rev F (`RF-1F`: valores de `C1`, `R_bass` e rede `Presence` no canal Red Modern), Klon Centaur (circuito original resinado sem esquema oficial do fabricante) e potenciômetro `Treble` (`1 MΩ` no add-on `OS/010` vs `220 kΩ` em variantes integradas) do Vox AC30/6 1964.
- **ASIO (`Spec 049 CA-36`):** compilação e teste com `CPAL_ASIO_DIR` + Steinberg ASIO SDK + LLVM/Clang + interface física ASIO + revisão formal de licenciamento.
- **Medição física por cabo loopback (`T-25`):** aferição de latência round-trip real em interface de referência.
- **Validadores externos DAW (`Spec 049 CA-16`):** execução de `clap-validator` e Steinberg VST3 SDK `validator`.
