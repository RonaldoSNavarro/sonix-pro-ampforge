# SONIX PRO // AMPFORGE™ — Backlog de Engenharia e Estado dos Marcos

Este documento alinha o backlog histórico do projeto com o **estado real do repositório** (conferido em 2026-11-19 contra `specs/README.md` e `AGENTS.md`).

## Decisões de Produto Confirmadas

- Núcleo DSP em Rust puro, desacoplado de GUI e host.
- Aplicação desktop standalone Windows 10 e 11 via **Tauri v2 + Vite/TypeScript** (sem HTTP/TCP no release).
- **WASAPI e ASIO** como requisitos confirmados do produto (WASAPI operacional; ASIO via feature `asio` pendente de validação em hardware/SDK/licenciamento — Spec 049 CA-36).
- Múltiplas taxas de amostragem (`44,1` / `48` / `88,2` / `96` kHz), canais configuráveis, modo Instrument/Line, captura DI e Reamp.
- Modelagem híbrida (circuit-level/nodal + behavioral com memória/ADAA + convolução IR particionada zero-latency).
- Plugins DAW **VST3 e CLAP** (`crates/sonix-plugin` via `nih_plug`) compartilhando o mesmo núcleo DSP.

---

## Marcos Concluídos (Specs 001–051)

- [x] **Fundação DSP, Lock-Free SPSC e Tempo Real** (Specs 001, 014, 016, 017, 018, 028, 034, 037, 044): callback zero-alloc/zero-lock verificado por alocador global de contagem, `RetiredReaper` off-RT, defesa NaN/Inf e limite `<= 0 dBFS`.
- [x] **Shell Desktop Tauri v2 e Contratos UI↔Backend** (Specs 011, 012, 015, 027, 033, 035, 046, 048, 050, 051): 6 telas fiéis à referência visual (SSIM), `controls.registry.ts` sem controles decorativos, `Channel` 30 Hz / 0 Hz minimizado, build release standalone com `custom-protocol` e startup seguro em Standby.
- [x] **Host de Áudio, Calibração, MIDI, Reamp e Plugins** (Specs 013, 019, 026, 029, 036, 049): WASAPI duplex, calibração de entrada + riff Karplus-Strong, MIDI real, alinhamento loopback de reamp, binários VST3/CLAP em `sonix-plugin`.
- [x] **Equipamentos e Efeitos — 1ª Geração Integrada** (Specs 002–010, 020–025, 030–032, 038–043, 045, 047): 5 amplificadores, 6 pedais de drive, Noise Gate com histerese/LR4, Compressor 1176LN, EQ 8/5/10 bandas, 7 modelos de modulação, RE-201 + delays, 5 reverbs, 4 modos de pitch, afinador YIN/polifônico, looper O(1) e gabinete com convolução IR particionada + 7 microfones.

---

## Aprovado Aguardando Autorização de Implementação

- [ ] **Spec 052** — Abertura animada com emblema SVG vetorial, designs skeuomórficos dos pedais faltantes (Modulação, Pitch e Wah `wah_1` Cry Baby GCB95) e mini-cabeça AMP 2 no Estúdio.

---

## Nova Fase — Arquitetura Modular e Fidelidade por Equipamento (Specs 053–084, em revisão)

Ordem recomendada de implementação após aprovação manual das Specs:

1. **Épico M1 — Contrato Central Modular (Spec 053):**
   - Extrair traits `AudioProcessor`, `AudioSpec`, `ModelDescriptor`, `ParamDescriptor`, `ChannelLayout`, `ModuleId`+revisão e registro tipado de fábricas em `ampforge-dsp`.
   - Unificar primitivas `Biquad64` (TDF-II / SVF) e desacoplar orquestração do grafo executável de `sonix-desktop`.
2. **Épico M2 — Componentes Compartilhados (Specs 076–084):**
   - Consolidar módulos reutilizáveis: Entrada/Impedância/Captadores (076), Estágios Valvulados/Power/Sag/Carga Reativa (077), Redes Passivas Tone Stack com Top Boost real (078), Oversampling Polifásico Unificado (079), Gabinete/IR/Microfones (080), Motor de Delay (081), Motor de Reverb (082), Afinador/Looper (083) e DAG/Split/Merge/Modifiers/Queues (084).
3. **Épico M3 — Amplificadores Independentes por Circuito (Specs 054–058):**
   - Extrair cada amplificador da struct monolítica `WhiteBoxAmpProcessor` para módulo próprio com parâmetros, revisão e testes de fidelidade dedicados: Marshall 1959 SLP (054), Marshall JCM800 2203 (055), Fender Twin Reverb AB763 (056), Mesa/Boogie Dual Rectifier Rev F (057) e Vox AC30/6 Top Boost (058).
4. **Épico M4 — Pedais e Unidades de Efeito Independentes (Specs 059–075):**
   - Desacoplar cada pedal de drive (059–064), dinâmica (065–066), EQ (067), tape echo (068), modulação (069–074) e pitch (075) em processadores independentes com identidade elétrica fiel.

---

## Pendências de Hardware, SDK e Validação Externa

- **ASIO (`Spec 049 CA-36`):** compilação e teste com `CPAL_ASIO_DIR` + Steinberg ASIO SDK + LLVM/Clang + interface física ASIO + revisão formal de licenciamento para distribuição.
- **WASAPI Exclusivo:** implementação nativa e validação em hardware real (atualmente reportado como indisponível em `DesktopAudioHost::has_mode_support`).
- **Medição física por cabo loopback (`T-25`):** aferição de latência round-trip real em interface de referência.
- **Validadores externos DAW (`Spec 049 CA-16`):** execução de `clap-validator` e Steinberg VST3 SDK `validator` sobre o artefato compilado de `sonix-plugin`.
- **Comparação A/B com hardware real / DI de referência:** captura controlada level-matched para validação auditiva e espectral fina de cada modelo (Specs 054–075).
