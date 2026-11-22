# SONIX PRO // AMPFORGE™

[![License: CC BY-NC 4.0](https://img.shields.io/badge/License-CC%20BY--NC%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc/4.0/)
[![Status: In Development](https://img.shields.io/badge/Status-Em%20Desenvolvimento-orange.svg)](#-status-do-projeto--convite-a-colaboradores)
[![Rust: 2021](https://img.shields.io/badge/Rust-2021%20Edition-black?logo=rust)](https://www.rust-lang.org/)
[![Tauri: v2](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri)](https://tauri.app/)
[![Tests: Contract Suite](https://img.shields.io/badge/Tests-Contract%20Suite-brightgreen.svg)](#-testes-e-qualidade)

Simulador de alta fidelidade para guitarra com pedaleira multi-efeitos, amplificadores valvulados clássicos com **modelagem White-Box** (100% algorítmica em nível de circuito analógico discreto), motor DSP de tempo real em **Rust** (*Zero-Alloc & Lock-Free*) e interface skeuomórfica fotorrealista acelerada por hardware em **Tauri v2**.

---

## ⚠️ Status do Projeto & 🤝 Convite a Colaboradores

> **O SONIX PRO // AMPFORGE™ está atualmente em desenvolvimento ativo.**
> Novas funcionalidades, melhorias algorítmicas de DSP, expansão do catálogo de equipamentos e polimento da interface estão sendo implementados continuamente.

### 🌟 Aceitamos e Incentivamos Colaboradores!
Buscamos construir uma plataforma de referência em simulação de instrumentos em código aberto. Sua participação é muito bem-vinda em diversas frentes:

- 🎸 **Músicos e Produtores:** Teste de usabilidade, comparação de timbres com equipamentos físicos, criação de presets de fábrica e relatórios de experiência.
- 🎛️ **Engenheiros de DSP e Áudio:** Otimizações algorítmicas, aperfeiçoamento da função analítica de triodo `u / √(1 + (k·u)²)` (`amp.rs`), novos modelos de caixas/microfones e filtros analógicos.
- 🦀 **Desenvolvedores Rust:** Aprimoramento de concorrência lock-free, suporte a drivers de áudio de baixa latência (WASAPI/ASIO) e integração de plugins DAW VST3/CLAP.
- 🎨 **Desenvolvedores Frontend e UI/UX:** Aprimoramentos no Design System skeuomórfico, renderização fluida em Canvas/WebGL, gráficos de osciloscópio CRT e responsividade.

Para saber como configurar seu ambiente, reportar problemas ou submeter um Pull Request, consulte nosso [**Guia de Contribuição (CONTRIBUTING.md)**](CONTRIBUTING.md).

---

## 🎛️ Principais Recursos e Arquitetura

### 1. Motor DSP Nativo em Rust (`ampforge-dsp`)
- **Zero-Alloc Realtime Audio Safety:** Nenhuma alocação de heap (`Vec`, `Box`, `String`) ou trava de concorrência (*mutexes/locks*) ocorre dentro do loop de processamento de áudio em tempo real.
- **Processamento de Alta Precisão:** Modelagem analítica de triodo em `f64` pela função `u / √(1 + (k·u)²)` com normalização `DECISAO_SONIX_PREAMP_STAGE_NORM` (`crates/ampforge-dsp/src/amp.rs`, verificado em `contract_039_amps_fidelity.rs`).
- **Oversampling Elíptico IIR de 8ª Ordem:** Fatores selecionáveis de 2x, 4x e 8x (`crates/ampforge-dsp/src/os_filters.rs`, atenuação de banda de rejeição >= 60 dB verificada em `auditoria_2026_11_20_artefatos.rs::aud_14_banda_de_rejeicao`). O `HalfBandOversampler` FIR 127-tap (>= 96 dB) permanece apenas em biblioteca (`docs/biblioteca-sem-uso-em-producao.md`).
- **Telemetria de Latência Real:** A telemetria calcula `estimated_roundtrip_ms = driver_input_latency_ms + driver_output_latency_ms + (algorithmic_latency_samples / sample_rate) * 1000.0` (`crates/sonix-desktop/src/audio.rs`, ex.: 12 amostras algorítmicas = 0,25 ms @ 48 kHz + buffers do driver WASAPI).

### 2. Amplificadores Valvulados Clássicos (White-Box)
Modelagem rigorosa dos esquemas elétricos originais:
- **Marshall JCM800 2203:** Master Volume, estágio *cold clipper* e resposta harmônica consagrada do Hard Rock.
- **Marshall 1959 SLP (Plexi):** 4 entradas, canais Normal e High Treble, suporte a entradas *jumpered* e interação de volumes.
- **Fender Twin Reverb AB763:** Clean brilhante com tone stack passivo Fender FMV, reverb de mola acionado a válvula e vibrato óptico.
- **Mesa Boogie Dual Rectifier:** 3 canais distintos (Clean, Vintage e Modern), retificação selecionável (Válvula 5U4G vs. Diodos de Silício) e modo Spongy/Bold.
- **Vox AC30 Top Boost:** Classe A sem feedback negativo global (*Zero NFB*), tone stack interativo e controle *Tone Cut* no estágio de potência.

### 3. Bancada do Chassi Aberto (*Mod Shop*) & Osciloscópio CRT
- **Troca Dinâmica de Válvulas:** Pré-amplificação (`12AX7`, `ECC83`, `12AT7`, `12AY7`, `EF86`) e saída (`EL34`, `6L6GC`, `6V6GT`, `EL84`, `KT88`).
- **Controle Físico de Bancada:** Tensão de placa (B+), compressão por *sag* de fonte, polarização de *bias*, saturação do transformador de saída e simetria de triodo.
- **Osciloscópio CRT em Tempo Real:** Visualização da forma de onda de áudio, curva de transferência não linear, resposta em frequência do tone stack e curva de impedância do alto-falante.

### 4. Pedaleira de Efeitos Analógicos
- **Drives & Distorções:** Ibanez TS808 Tube Screamer, Klon Centaur (circuito de *clean blend* real), ProCo RAT (filtro analógico reverso), Electro-Harmonix Big Muff Pi (mid-scoop), Dallas Arbiter Fuzz Face (Germânio/Silício) e Boss DS-1.
- **Compressores:** Studio FET 1176LN com curvas de ataque ultrarrápidas (20–800 µs), tempo de release fiel e modo *"All-Buttons-In"*.
- **Noise Gate de Estúdio:** Histerese anti-chatter, tempo de Hold, filtragem de sidechain e chaveamento com detecção direta no sinal limpo (DI).
- **Roland RE-201 Space Echo:** Simulação fiel do mecanismo de fita de 3 cabeças de reprodução, saturação analógica de fita, mola interna e 12 modos de chaveamento originais.
- **Reverbs:** Tube Spring (mola valvulada com dispersão all-pass) e algoritmo Freeverb (Jezar) — voicing Plate, Hall, Chamber e Shimmer granular (+12 semitons).
- **Modulações:** Boss CE-2 BBD Chorus, Chorus 3 vozes (mono), MXR Phase 90, Through-Zero Flanger com feedback, Tremolo Óptico LDR (ataque 5 ms / soltura 60 ms) e Rotary Speaker dual-rotor (LR4 800 Hz).
- **Pitch & Afinador:** Afinador com visual estroboscópico (detecção YIN 38 Hz – 1,4 kHz, cobre E1 = 41,2 Hz; B0 = 30,9 Hz fora da faixa, verificado em `crates/ampforge-dsp/tests/auditoria_2026_11_20_afinador.rs`) e modo polifônico de 6 cordas, além de bloco de Harmonizador Diatônico.
- **Tabela Única de Parâmetros:** Todo parâmetro do backend nasce da `PARAMS_TABLE` única (`crates/sonix-desktop/src/params.rs`) e gera `apps/sonix-app/ui/src/params.generated.ts` (verificado em `tests/contract/auditoria-2026-11-20-parametros.test.js`).

### 5. Gabinete Virtual & Convolução de IR
- **Convolução de IR Zero-Latency:** Convolução particionada no domínio do tempo e da frequência para carregamento de Impulse Responses WAV (.wav) sem adição de latência.
- **Microfonação Dupla 3D:** Modelos como Shure SM57, Royer R121 e Sennheiser MD421 com controle de posicionamento milimétrico, distância e ângulo *off-axis*, simulando efeito de proximidade e filtro pente.
- **Gabinete Twin 2×12 Open:** Simulação de gabinete aberto traseiro com cancelamento de baixas frequências e ressonância acústica do cone.

### 6. Matriz de Roteamento DAG 4×12, Cenas e Modifiers
- **Matriz de Roteamento 4×12:** Grafo de processamento totalmente configurável com múltiplos pontos de entrada e saída.
- **Divisores (Split):** Modos A/B, Crossover de frequência e Divisão Dinâmica por envelope de amplitude.
- **Cenas Gapless:** Troca instantânea entre cenas com transições suaves (*crossfade*) e preservação de caudas de eco e reverberação (*spillover*).
- **Matriz de Modulação (Modifiers):** LFOs sincronizáveis, gerador de envelope dinâmico, pedal de expressão e mapeamento MIDI CC com curvas de resposta customizáveis.

---

## 🏛️ Estrutura do Workspace

```text
├── apps/
│   └── sonix-app/                  # Aplicação Desktop nativa (Tauri v2)
│       ├── src-tauri/              # Shell desktop em Rust e integração de SO
│       └── ui/                     # Interface do usuário (TypeScript, Vite, Design System)
├── crates/
│   ├── ampforge-dsp/               # Motor DSP de tempo real (White-Box, solvers, FX, IR)
│   ├── sonix-core/                 # Estruturas fundamentais, schemas JSON e persistência offline
│   ├── sonix-desktop/              # Host desktop, drivers de áudio (WASAPI/ASIO) e MIDI
│   └── sonix-plugin/               # Plugins de áudio DAW VST3 e CLAP (via nih-plug; sem editor próprio; usa a interface genérica de parâmetros do host)
├── docs/                           # Documentação arquitetural, levantamento de requisitos e referências visuais
├── specs/                          # Especificações formais SDD (Specs 001–052 implementadas (com ressalvas registradas em 001, 002, 007, 008, 009, 012, 028) e Specs 053–092 em rascunho modular)
└── tests/                          # Testes de contrato automatizados de ponta a ponta (E2E / JS)
```

---

## 🚀 Como Executar Localmente

### Pré-requisitos
- [Rust](https://www.rust-lang.org/) (versão 1.80 ou superior)
- [Node.js](https://nodejs.org/) (versão 18 ou superior)
- C++ Build Tools (MSVC no Windows)

### 1. Clonar o Repositório e Instalar Dependências
```bash
git clone https://github.com/RonaldoSNavarro/sonix-pro-ampforge.git
cd sonix-pro-ampforge
npm install
npm --prefix apps/sonix-app/ui install
```

### 2. Executar em Modo de Desenvolvimento (Desktop Tauri v2)
```bash
npm --prefix apps/sonix-app/ui run tauri dev
```

### 3. Compilar a Versão Final Release Standalone
```bash
# Compila a UI estática otimizada
npm --prefix apps/sonix-app/ui run build

# Gera o executável standalone sonix-app.exe com UI embutida
cargo build --release -p sonix-app --features custom-protocol
```
O executável otimizado estará disponível no diretório de saída do Cargo.

---

## 🧪 Testes e Qualidade

O projeto adota a metodologia **Spec Driven Development (SDD)** com **TDD Estrito**. Toda implementação é coberta por testes automatizados antes de ser integrada:

```bash
# Executa todos os testes unitários e de contrato do workspace Rust (exceto 013::ca2 sem hardware WASAPI físico):
cargo test --workspace --locked -- --test-threads=1

# Executa a suíte de contratos de integração e interface (Node.js):
node --test tests/contract/

# Executa o E2E da UI desktop construída (Playwright):
npm run test:e2e

# Verifica citações e front-matter das specs (001..092):
python tools/verificar_specs.py
```

As Specs 001–052 implementadas (e 053–092 em rascunho) possuem rastreabilidade completa e critérios de aceite auditáveis. Para mais detalhes, consulte o [Índice de Especificações (specs/README.md)](specs/README.md).

---

## 📄 Licença e Termos de Uso

- Este repositório está licenciado sob **CC BY-NC 4.0** (veja o arquivo [LICENSE](LICENSE)).
- Binários VST3 gerados com `nih-plug`/`vst3-sys` ficam sujeitos à **GPLv3** e **não são distribuídos** por este repositório.
- Este é um projeto pessoal, sem uso comercial.

---

Desenvolvido com dedicação por **Ronaldo S. Navarro** e comunidade de colaboradores.
Sinta-se em casa para experimentar, tocar e contribuir! 🎸⚡
