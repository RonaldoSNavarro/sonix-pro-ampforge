# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

SONIX PRO // AMPFORGE™ — simulador desktop de pedaleira/amplificadores de guitarra: motor DSP
"White-Box" em Rust + app Tauri v2. Projeto, specs e testes são escritos em português.

**Leia [AGENTS.md](AGENTS.md) antes de qualquer mudança** — ele contém as regras obrigatórias
(SDD + TDD estrito, áudio zero-alloc, "todo controle visível funciona no backend") e o trabalho
em andamento. O índice/changelog das specs está em [specs/README.md](specs/README.md).

## Comandos

```bash
# Rust (workspace na raiz)
cargo check --workspace
cargo test --workspace
cargo test -p ampforge-dsp --test contract_031_authentic_amps        # só os testes de uma spec
cargo test -p ampforge-dsp --test contract_031_authentic_amps ca_4   # um CA específico (filtro por nome)

# Contratos JS (Node 26 importa os .ts da UI diretamente — sem build)
npm test                                                             # node --test tests/contract/*.test.js
node --test tests/contract/033-app-desktop-tauri-v2-telas-e-controles-reais.test.js
node --test --test-name-pattern="CA-4" tests/contract/033-*.test.js

# UI do app (Vite + TypeScript, sem framework)
npm --prefix apps/sonix-app/ui run dev      # Vite em :1420 (devUrl do Tauri)
npm --prefix apps/sonix-app/ui run build    # tsc --noEmit + vite build → ui/dist

# App Tauri (crate sonix-app em apps/sonix-app/src-tauri)
cargo build -p sonix-app
cargo test  -p sonix-app --test contract_033_desktop_shell

# Host legado com servidor HTTP local (127.0.0.1:7890, servindo design-system/)
cargo build --release -p sonix-desktop      # usado por INICIAR_SONIX_PRO.bat
```

Os testes de `crates/sonix-desktop/tests` que exercitam o servidor HTTP declaram
`required-features = ["dev-http"]` (feature default). O app Tauri depende de `sonix-desktop`
com `default-features = false` — o release não pode ter HTTP/TCP nem depender do Edge (Spec 033).

## Arquitetura

Workspace Cargo com camadas em sentido único:

- **`crates/ampforge-dsp`** — motor DSP puro: amps (`amp.rs`, `tonestack.rs`), bancada/Mod Shop
  (`modshop.rs`), gabinete/mic/IR (`cab.rs`), efeitos (`fx/`), afinador YIN (`tuner.rs`), looper,
  DAG de roteamento 4×12 (`dag.rs`), filas SPSC lock-free e smoothers. Sem I/O.
- **`crates/sonix-core`** — presets/cenas/canais/modifiers/footswitches e persistência offline
  (`.sonixpreset`, `.sforgepkg`).
- **`crates/sonix-desktop`** — host de áudio (`cpal`/WASAPI, `audio.rs`), cadeia em tempo real
  `RealtimeDspChain`, calibração de entrada, MIDI e a **API de controle** (`api.rs`):
  `dispatch(...)` é o ponto único que recebe parâmetros da UI, valida atomicamente contra a tabela
  única de parâmetros (namespace da Spec 014, ex. `gate.threshold_db`, `amp.gain`) e os envia ao
  callback via `LockFreeParamQueue`. `server.rs` (HTTP local) só existe com `dev-http`.
- **`crates/sonix-plugin`** — base dos plugins VST3/CLAP sobre o mesmo núcleo.
- **`apps/sonix-app/src-tauri`** — casca Tauri v2: embute `sonix-desktop` sem servidor;
  `commands.rs` expõe os comandos IPC (que reaproveitam `api::dispatch`), `streams.rs` envia
  telemetria/afinador/osciloscópio por `Channel`s.
- **`apps/sonix-app/ui`** — frontend TypeScript. `transport.ts` faz o IPC (`ParamCoalescer`
  agrupa escritas em no máx. um `set_params` por frame); `controls.registry.ts` é o **registro
  único de controles** — todo controle visível precisa estar lá com chave aceita pelo backend
  (ou `visualOnly`), e os testes de inventário reprovam o que faltar. Telas em `src/screens/`.
- **`design-system/`** — casca HTML/JS legada servida pelo host HTTP; `docs/ui-desktop/canvas/`
  é o design de referência das telas do app.

Fluxo de um controle: UI → `controls.registry.ts` → `set_params` (IPC) → `api::dispatch`
(validação) → fila lock-free → `RealtimeDspChain::process` no callback de áudio.

## Convenções de teste

- Uma spec ⇒ arquivos de contrato com o número dela: Rust em `crates/*/tests/contract_NNN_*.rs`
  (e `apps/sonix-app/src-tauri/tests/`), JS em `tests/contract/NNN-*.test.js`.
- Cada caso cita o CA no nome (`ca4_...` em Rust, `'CA-4: ...'` em JS); o cabeçalho do arquivo
  lista os CAs cobertos.
- Testes JS de UI frequentemente leem os fontes/registro diretamente (inventário de controles,
  ausência de HTTP no release) — mudanças de nomes de arquivos/chaves podem quebrá-los.
- Controle novo exige teste de **efeito mensurável no sentido do equipamento real**
  (ex.: `crates/sonix-desktop/tests/contract_033_control_effects.rs`) e atualização de
  `docs/ui-desktop/inventario-controles.md`.
