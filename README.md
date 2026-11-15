# SONIX PRO // AMPFORGE™

Simulador de pedaleira de guitarra multi-efeitos e amplificadores com modelagem de circuito White-Box (100% algorítmica em nível de componentes discretos), motor DSP nativo em Rust e Design System Skeuomórfico Fotorrealista.

---

## 🎛️ Visão Geral e Arquitetura Desktop

O projeto é direcionado exclusivamente a **plataformas Desktop nativas** (sem alvo Web no escopo atual):
- **Desktop Standalone (`Tauri v2 + Rust`)**: Executável de palco/estúdio com janela nativa acelerada por GPU, drivers de áudio (`WASAPI`/onboard, evolução para `ASIO` / MIDI na Spec 036) e motor DSP nativo em Rust.
- **Plugins de DAW (`sonix-plugin`)**: Arquitetura compartilhada planejada (`Spec 012` em estado *passthrough*; empacotamento `VST3` e `CLAP` fora da entrega atual enquanto `sonix-plugin` não processa áudio completo).
- **SONIX PRO**: Interface de estúdio e simulação de hardware físico da pedaleira de chão (8 footswitches RGB, *scribble strips* OLED, pedal de expressão, matriz de roteamento 4×12).
- **AMPFORGE™**: Motor DSP White-Box (64-bit Newton-Raphson) e bancada (*Open Chassis Mod Shop*) com controle de válvulas (`12AX7`, `EL34`, etc.), tensão de placa, bias, sag da fonte e osciloscópio CRT em tempo real.

---

## 📐 Metodologia: Spec Driven Development (SDD) & TDD Estrito

Toda funcionalidade e resolução de lacunas é guiada por especificações formais numeradas em `specs/` com critérios de aceite verificáveis `CA-n`.

- Índice e Changelog de Specs: [specs/README.md](file:///mnt/g/Dev/Projetos/Simulador%20de%20efeitos/specs/README.md)
- Regras de desenvolvimento e TDD estrito: [AGENTS.md](file:///mnt/g/Dev/Projetos/Simulador%20de%20efeitos/AGENTS.md)
- Fontes da verdade: [01-SRS-Levantamento-de-Requisitos.md](file:///mnt/g/Dev/Projetos/Simulador%20de%20efeitos/docs/01-SRS-Levantamento-de-Requisitos.md) e [02-Design-System-Specification.md](file:///mnt/g/Dev/Projetos/Simulador%20de%20efeitos/docs/02-Design-System-Specification.md)

---

## 📂 Estrutura do Repositório

```text
├── docs/
│   ├── 01-SRS-Levantamento-de-Requisitos.md    # Especificação formal de requisitos (RFs, RNFs e JSON Schema)
│   └── 02-Design-System-Specification.md      # Especificação visual, tokens de materiais e anatomia dos componentes
├── specs/                                     # Especificações SDD numeradas (001 a 012)
│   ├── README.md                              # Índice canônico de specs e status
│   └── ...
└── design-system/                             # Casca visual do Design System (Web Components / CSS / Canvas)
    ├── index.html                             # Layout em 3 zonas, palco skeuomórfico e hardware shell
    ├── styles.css                             # Tokens canônicos de materiais (:root), microtexturas e iluminação
    └── app.js                                 # Simulação visual, osciloscópio CRT e barramento de parâmetros
```

---

## 🚀 Como Visualizar a Casca do Design System

A casca de apresentação do Design System (utilizada pela WebView Desktop do Tauri v2) pode ser inspecionada localmente:

```bash
cd design-system
python3 -m http.server 8080
```

---

## 📄 Licença

Projeto pessoal, sem fins comerciais, distribuído sob a **GNU General Public License v3.0 ou posterior** (`GPL-3.0-or-later`) — texto completo em [LICENSE](LICENSE).

A GPLv3 é compatível com as dependências de áudio previstas: SDK ASIO da Steinberg pela opção GPLv3 da licença dupla, SDK VST3 3.8+ (MIT), CLAP (MIT) e bindings Rust de VST3 em GPLv3. O SDK ASIO **não** é versionado neste repositório (ver Spec 049, CA-36). Os nomes e logos "ASIO" e "VST" são marcas da Steinberg e só podem aparecer segundo as regras de marca dela.
