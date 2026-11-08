# SONIX PRO // AMPFORGE™

Simulador de pedaleira de guitarra multi-efeitos e amplificadores com modelagem de circuito White-Box (100% algorítmica em nível de componentes) e Design System Skeuomórfico Fotorrealista.

---

## 🎛️ Visão Geral

- **SONIX PRO**: Interface de estúdio e simulação de hardware físico da pedaleira de chão (8 footswitches RGB, *scribble strips* OLED, pedal de expressão, matriz de roteamento 4×12).
- **AMPFORGE™**: Motor DSP White-Box (64-bit Newton-Raphson) e bancada de bancada (*Open Chassis Mod Shop*) com controle de válvulas (`12AX7`, `EL34`, etc.), tensão de placa, bias, sag da fonte e osciloscópio CRT em tempo real.

---

## 📂 Estrutura do Repositório

```text
├── docs/
│   ├── 01-SRS-Levantamento-de-Requisitos.md    # Especificação formal de requisitos (RFs, RNFs e JSON Schema)
│   └── 02-Design-System-Specification.md      # Especificação visual, tokens de materiais e anatomia dos componentes
└── design-system/
    ├── index.html                             # Protótipo funcional interativo completo
    ├── styles.css                             # Design tokens, CSS skeuomórfico e iluminação de válvulas
    └── app.js                                 # Simulação de áudio, oscilloscópio CRT, grid 4x12 e cenas gapless
```

---

## 🚀 Como Executar o Protótipo

Basta abrir o arquivo `design-system/index.html` em qualquer navegador web moderno:

```bash
# Ou servir localmente com qualquer servidor estático HTTP
cd design-system
python3 -m http.server 8080
```
