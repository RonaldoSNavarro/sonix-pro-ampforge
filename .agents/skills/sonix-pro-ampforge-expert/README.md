# SONIX PRO // AMPFORGE™ — Expert Skill (`sonix-pro-ampforge-expert`)

Skill especialista de domínio e engenharia para agentes que projetam, especificam, implementam, depuram, auditam e validam o **SONIX PRO // AMPFORGE™** — cobrindo amplificadores valvulados, circuitos analógicos, pedais, guitarras/captadores, gabinetes/microfones/IR, DSP avançado, Rust em tempo real e integração desktop com Tauri v2.

## Conteúdo da Skill

- `SKILL.md` — instruções centrais, hierarquia dual de autoridade (governança vs evidência factual vs implementação), 10 regras inegociáveis de fidelidade documental, fundamentos de amplificadores/pedais/IR e checklist de auditoria.
- `references/architecture-rust-windows.md` — arquitetura efetiva do workspace Rust (`ampforge-dsp`, `sonix-core`, `sonix-desktop`, `sonix-plugin`, `apps/sonix-app`), Tauri v2, `cpal` WASAPI/ASIO, pipeline em tempo real e mapa modular completo (Specs 053–092).
- `references/dsp-engineering.md` — guia técnico de modelagem de circuitos (MNA, WDF, State-Space, ADAA, solvers não lineares), filtros, oversampling, convolução particionada, separação thread de controle vs thread RT e validação objetiva.
- `references/rust-tauri-best-practices.md` — padrões de engenharia Rust para áudio RT e boas práticas de integração desktop Tauri v2 (IPC, `Channel`, `capabilities`, segurança e as 7 regras obrigatórias).
- `references/engineering-backlog.md` — estado real dos marcos implementados (Specs 001–052) e roadmap modular auditado (Specs 053–092).

## Perfil Tecnológico Efetivo do Repositório

- **Núcleo e host:** Rust (edition 2021, `rustc 1.99.x`, licença `CC-BY-NC-4.0`).
- **Desktop Standalone (Windows 10/11):** Tauri v2 (`tauri 2.12.5`, `@tauri-apps/api 2.12.4`) + Vite 8 + TypeScript 7 (UI vanilla sem framework) embutido via `custom-protocol` (sem HTTP/TCP no release; servidor local apenas com feature `dev-http`).
- **Áudio Windows:** `cpal 0.18.2` com WASAPI ativo por padrão e ASIO via feature `asio` (`CPAL_ASIO_DIR` + LLVM/Clang; requisito confirmado de produto com verificação em hardware/SDK/licenciamento registrada como pendente em Spec 049 CA-36).
- **Plugins DAW:** `crates/sonix-plugin` (`nih_plug`) expondo VST3 e CLAP sobre o mesmo núcleo `RealtimeDspChain` (validadores externos `clap-validator` / VST3 SDK `validator` pendentes).
- **Modelagem DSP:** estratégia híbrida (circuit-level/nodal KCL + behavioral com memória/ADAA + convolução IR particionada zero-latency).

## Localização e Carregamento no Antigravity

A skill está ativa no repositório no caminho canônico:

`.agents/skills/sonix-pro-ampforge-expert/`

O Antigravity carrega automaticamente o manifesto YAML presente no topo de `SKILL.md` (`name: sonix-pro-ampforge-expert`). Ao evoluir esta skill, mantenha todos os caminhos relativos dentro de `references/` válidos e sincronizados com `AGENTS.md`, `CLAUDE.md` e `specs/README.md`.
