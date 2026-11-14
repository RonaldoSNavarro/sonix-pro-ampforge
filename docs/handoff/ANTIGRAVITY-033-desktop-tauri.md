# Handoff — Spec 033: App Desktop Tauri v2 (para o Antigravity)

> Spec de origem: `specs/033-app-desktop-tauri-v2-telas-e-controles-reais.md` (status: **aprovada** em 2026-11-13).
> Liberado para começar pela fase 1. As fases 4 e 5 tocam as Specs 030/031/032, que ainda estão em rascunho: implementar DSP delas só depois de aprovadas (até lá, o controle fica desabilitado e conta como pendência).

## 0. Leia antes, nesta ordem
1. `AGENTS.md`: SDD + TDD estrito e regras promovidas do Cortex.
2. `specs/README.md`: fluxo, convenção de nomes e índice.
3. `specs/033-…md` inteira, depois `014`, `015`, `027`, `028`, `030`, `031`, `032`.
4. `.agents/skills/sonix-pro-ampforge-expert/SKILL.md` e `references/architecture-rust-windows.md`.
5. `docs/ui-desktop/canvas/*.dc.html`: o design aprovado. A lógica dos modelos está nos `renderVals()`.
6. `docs/ui-desktop/fontes/`: manuais Marshall usados para os painéis.
7. Cortex: `query` por `rule`, `gotcha`, `tauri`, `desktop`, `spec-033`.

## 1. Regras inegociáveis
- **Sem spec aprovada, sem código.** Comportamento novo que aparecer volta para a spec 033 antes do código.
- **TDD estrito.** Escreva o teste do `CA-n` primeiro, rode e **mostre que falha** (red), depois implemente (green). Todo teste cita o CA no nome: `ca5_standby_silencia`, `CA-6: ordem do painel 2203`.
- **Callback de áudio:** zero alocação, zero lock, zero I/O e zero log. Comandos Tauri **nunca** tocam a thread de áudio; escrevem só pela fila SPSC da Spec 014.
- **Nenhum controle decorativo.** Todo knob, chave, jack, botão, pedal ou footswitch visível precisa de chave no registro, validação no backend e teste de efeito real (Spec 033 §3, CA-4 e CA-5). Controle sem DSP implementado aparece **desabilitado** com dica "em implementação — Spec 0NN" e conta como pendência.
- **Fidelidade:**
  - Ordem e rótulos dos painéis vêm de manual ou esquema com revisão citada. Não invente valores de componentes.
  - Pendências conhecidas, que você deve verificar e documentar antes de implementar:
    - Twin AB763: Middle no canal Normal;
    - Dual Rectifier: revisão do painel;
    - AC30 Top Boost: ordem dos controles.
- **Marcas:** nomes fictícios (BRIT PLEXI, BRIT 800, US TWIN, CALI RECTO, UK 30, SCREAMER 808…). Sem logotipos, assinaturas ou marcas registradas.
- **Git:** **nunca** faça `git commit`, `push`, `rebase` ou `reset` sem autorização explícita do usuário. Deixe as mudanças no working tree e relate.
- **Medições:** não invente números de latência ou CPU. Mostre o valor medido ou `[medida pendente]`.

## 2. Estrutura alvo
```
apps/sonix-app/
  src-tauri/            # crate binário Tauri v2 (membro do workspace)
    Cargo.toml          # tauri 2.x, tauri-plugin-{single-instance,window-state,dialog,fs}
    tauri.conf.json     # janela 1440×900 (mín. 1280×800), decorations:false, CSP, NSIS offlineInstaller
    capabilities/main.json
    src/main.rs         # setup, inicia host de áudio de sonix-desktop, registra comandos e canais
    src/commands.rs     # comandos finos → sonix_desktop::api::dispatch
    src/streams.rs      # Channel<TelemetryFrame>, Channel<TunerFrame>, Channel<ScopeFrame>
    tests/contract_033_desktop_shell.rs
  ui/                   # Vite + TypeScript + Web Components (sem framework)
    src/controls.registry.ts
    src/transport.ts    # invoke/Channel no Tauri; fetch só no modo dev-http
    src/components/     # <sonix-amp-head>, <sonix-pedal>, <sonix-rack>, <sonix-knob>, <sonix-tube>…
    src/screens/        # estudio, bancada, gabinete, palco, biblioteca, audio
    public/fonts/       # Oswald, Inter, JetBrains Mono (locais)
  e2e/033.e2e.ts        # tauri-driver + msedgedriver
crates/sonix-desktop/
  src/api.rs            # NOVO: dispatch(route, method, body) puro, extraído de server.rs
  src/server.rs         # passa a usar api::dispatch; atrás da feature "dev-http"
  tests/contract_033_control_effects.rs
tests/contract/033-app-desktop-tauri-v2-telas-e-controles-reais.test.js
```

## 3. Fases e gates

Cada fase termina com todos os testes verdes e um relatório ao usuário (no formato da seção 5).

**Fase 1 — Extrair `api::dispatch` (sem mudar comportamento)**
- Mova a lógica das rotas de `server.rs` para `api.rs` como funções puras. O `server.rs` vira um adaptador fino.
- Coloque `tiny_http` atrás de `features = ["dev-http"]`.
- **Gate:** `cargo test --workspace --features dev-http` e `npm test` verdes, sem editar nenhum teste existente (CA-14).

**Fase 2 — Shell Tauri**
- Crie `apps/sonix-app` com janela, barra de título própria (`data-tauri-drag-region`), instância única, window-state, capabilities e CSP.
- Comandos `invoke` espelhando as rotas; `Channel` para telemetria, afinador e osciloscópio.
- Remova a abertura do `msedge` do caminho release.
- **Gate:** CA-1, CA-2, CA-9, CA-10, CA-11 e CA-13 red → green.

**Fase 3 — Registro de controles e inventário**
- Crie `controls.registry.ts` a partir dos `renderVals()` do canvas (modelos, chaves, padrões, faixas) e da tabela da Spec 033 §3.
- Escreva o teste de inventário (CA-4) e o de efeitos (CA-5). **Eles vão falhar muito**, e isso é esperado: a lista de falhas é o backlog real.
- Salve essa lista em `docs/ui-desktop/inventario-controles.md`: controle → chave → spec responsável → status.
- **Gate:** relatório do inventário entregue ao usuário. Não "conserte" testes para passar.

**Fase 4 — Telas e componentes**
- Porte os componentes do canvas para Web Components:
  - `AmpHead`: 5 modelos; os painéis Marshall já seguem os manuais.
  - `Pedal`: 9 modelos.
  - `Rack`: 3 modelos.
  - `Knob`, `Tube`, footswitch, LCD e medidores.
- Porte as telas: Estúdio, Bancada, Sala do gabinete, Palco, Biblioteca e Áudio.
- Interações: arrasto com captura de ponteiro, Shift para ajuste fino, roda, duplo clique para reset, setas, ARIA e coalescência por quadro.
- **Gate:** CA-3, CA-6, CA-7, CA-8 e CA-12.

**Fase 5 — Fechar as pendências do inventário**
- Para cada controle pendente, implemente o DSP pela spec responsável, na ordem do `specs/README.md`: 030 (gate), 031 (amps), 032 (pedais, rack, gabinete), seguindo o TDD de cada uma.
- **Gate final:** inventário sem pendências → Spec 033 `implementada`, índice atualizado, Spec 012 marcada `implementada (CA-6 revertido)`.

## 4. Comandos úteis
```
cargo test --workspace --features dev-http
cargo test -p sonix-desktop --test contract_033_control_effects
npm test
cd apps/sonix-app && npm run tauri dev
cd apps/sonix-app && npm run tauri build
```
Antes de adotar uma versão de crate ou pacote npm, confira-a no registro e prefira versões publicadas há pelo menos duas semanas. Registre a versão escolhida no Cortex.

## 5. Relatório de cada fase (formato)
- O que foi feito (arquivos).
- CAs cobertos, com a saída do teste **falhando antes** e **passando depois**.
- Pendências e dúvidas de fidelidade, com a fonte consultada.
- Comando para reproduzir.
- Nada commitado (aguardando autorização).

## 6. Cortex
Ao fim de cada fase:
- `capture` das decisões (ex.: versão do Tauri, formato dos frames do Channel), dos gotchas e dos fatos;
- `consolidate` + `write_page`;
- `promote_rules` se surgir regra nova.

Projeto: `simulador-de-efeitos`.
