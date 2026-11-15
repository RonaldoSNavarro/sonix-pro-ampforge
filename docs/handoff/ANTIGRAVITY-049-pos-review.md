# Handoff — Spec 049: correções do review da entrega 034–048 (para o Antigravity)

> Origem: code review do working tree da entrega 034–048 e revisão especialista de DSP, feitos em 2026-11-15.
> Substitui `ANTIGRAVITY-034-048-pos-review.md` como trabalho em andamento.
> A Spec 049 é **spec única por decisão do usuário** (40 CAs). Status: `rascunho` — **não comece a implementar antes de o usuário aprová-la** (status `aprovada` no arquivo e no índice).

## 0. Leia antes, nesta ordem
1. `AGENTS.md`: SDD + TDD estrito e as regras promovidas do Cortex.
2. `CLAUDE.md`: comandos de build e teste, arquitetura e fluxo de um controle.
3. `specs/README.md`: índice e o parágrafo **"Review da entrega 034–048 (2026-11-15) → Spec 049"**.
4. `specs/049-correcoes-pos-review-das-specs-034-048.md`, **inteira**: Contexto com evidências `arquivo:linha`, CAs, Fora de escopo, Invariantes e Notas (arquitetura do A1, licenças, ordem).
5. `.agents/skills/sonix-pro-ampforge-expert/SKILL.md` para as Partes A (CA-3, CA-13), C, D e F.
6. Cortex: `query` por `rule`, `gotcha`, `spec-049`, `realtime`, `asio`, `licenca`.

## 1. Regras inegociáveis (iguais ao handoff anterior)
- **TDD estrito.** Teste do `CA-n` primeiro, **mostrado falhando** (red), depois implementação (green). Nome cita o CA (`ca7_...` / `'CA-7: ...'`); arquivos `contract_049_*.rs` e `tests/contract/049-*.test.js`.
- **Callback de áudio:** zero alocação, desalocação, lock, I/O e log — **medido** pelo alocador de contagem da Spec 037.
- **Nenhum controle decorativo.** Sem DSP → `disabled` com a dica "em implementação — Spec 049".
- **Fidelidade:** não invente valores de componentes; sem fonte, pare e relate.
- **Não reescreva teste antigo para passar.** CA substituído vai para as Notas como "revertido", com motivo.
- **Comportamento novo fora da spec:** edite a spec primeiro.
- **Git:** **nunca** rode `commit`, `push`, `rebase` ou `reset` sem autorização explícita.
- **Medições:** valor medido ou `[medida pendente]` (vale para CA-12, CA-13, CA-36, CA-39).
- **Versões:** crate novo conferido no registro, publicado há ≥ 2 semanas, escolha registrada no Cortex.

## 2. Estado do working tree ao receber
Nada commitado. Além da entrega 034–048 ainda não commitada, o working tree contém:
- `specs/049-correcoes-pos-review-das-specs-034-048.md` (nova, `rascunho`) e `specs/README.md` (linha do índice + ordem);
- `LICENSE` (novo, texto oficial da GPLv3);
- `Cargo.toml` do workspace com `license = "GPL-3.0-or-later"` (antes `"Proprietary"`) e `license.workspace = true` nos cinco crates;
- `README.md` com a seção "Licença".

## 3. Partes e gates
Ordem obrigatória: **A → A′ → F → C → D → B**. Cada parte termina com `cargo test --workspace` e `npm test` verdes e relatório no formato da §5. Não comece a parte seguinte com a anterior vermelha.

### Parte A — achados do review (CA-1..CA-14)
- **Comece por CA-7 e CA-14**: removem UB e lock.
  - CA-7: `LockFreeResourceQueue::push` (`audio.rs:1453`) sem lock com vários produtores; contador de `Drop`.
  - CA-14: `try_lock` na telemetria (`api.rs:817`).
- **CA-1..CA-3 (troca de cena):** siga a arquitetura das Notas — looper, delay e reverb com **instância única** fora do par ativa/sombra; a cena controla só o envio. Só amp, drive, EQ e cab ficam duplicados. CA-3 fixa feedback e RT60 (intensity 50 %, RT60 ≥ 2 s).
- **CA-4..CA-6 (dispositivo):** um par de streams por vez (`commands.rs:290–311`), thread de fallback parada (`lib.rs:105`), exclusivo com rollback.
- **CA-8, CA-9:** cena inválida → 400; o `AppState` reflete toda chave aceita (percorra a tabela; `api.rs:1486..1605`).
- **CA-10:** status bar sem literais (`shell.ts:341..420`).
- **CA-11:** faixas MIDI coerentes com a tabela (`midi.rs:135` vs `api.rs:1856`).
- **CA-12:** latência por impulso, ±1 amostra.
- **CA-13:** oversampler — **meça antes de otimizar**; custo ≤ 25 % do atual sem perder stopband, aliasing com senoide de 5 kHz não pior que 1 dB.

### Parte A′ — CA-15
Inventário de controles (`docs/ui-desktop/inventario-controles.md`, 13 rótulos "pendente") e Notas honestos.

### Parte F — requisitos do primeiro release (CA-36..CA-39)
- **ASIO real (CA-36):** o `cpal` hoje não tem a feature `asio`. Regras (Notas da spec):
  - SDK ASIO **fora do repositório**, lido por `CPAL_ASIO_DIR`; usar a opção **GPLv3** da licença dupla;
  - feature de Cargo opcional no `sonix-desktop`; sem SDK o build funciona só com WASAPI;
  - LLVM/Clang para o `bindgen`; nenhum driver empacotado (nem ASIO4ALL);
  - marcas "ASIO"/"VST" só segundo as regras da Steinberg;
  - verificação em hardware registrado ou `[verificação pendente]` — **pergunte ao usuário qual interface ASIO de referência usar**.
- **CA-37:** falha do ASIO → `mode_unavailable`, WASAPI mantido.
- **CA-38:** `input.mode` instrument/line, com aviso de que Hi-Z é propriedade elétrica da interface.
- **CA-39:** reamp alinhado por loopback **medido**, ≤ 1 amostra, aviso de reamp box.

### Parte C — DAG 4×12 ao vivo (CA-23..CA-28)
Harmônicos comparados com RMS igualado; troca de topologia sem buraco nem alocação; ciclo → 400; crossover ±0,2 dB; saída sem cab por nulo contra o tap pré-cab (-90 dBFS) e aviso de nível de linha; bypass isolado.

### Parte D — Setlists, Modifiers, footswitches (CA-29..CA-34, CA-30b)
CA-30b: canal reativado começa com estado zerado (mesma família do A1). CA-32: Modifiers só em parâmetros contínuos (discreto → 400), damping 63 % em 100 ms.

### Parte B — plugins (CA-16..CA-22), por último
- **Licença resolvida:** projeto em GPL-3.0-or-later; **CLAP e VST3 juntos**, do mesmo código, por um framework que exporte os dois (candidato: `nih-plug`). Confira de novo o `LICENSE` do framework e das bindings antes do CA-16.
- CA-17: nulo -100 dBFS com mesma taxa e bloco; -60 dBFS entre blocos de 64 e 512.
- CA-20: mudança de BPM soa como mudança de velocidade do motor da fita (glide), 428,6 ms ±1 ms após 1 s.
- CA-22: editor sem HTTP.

### Parte E — CA-35
Não-regressão de toda a suíte ao fim.

## 4. Fechamento
1. Status `implementada`, ou `implementada (Parte X pendente)` / `(CA-n revertida)` com motivo.
2. "Testes que cobrem esta spec" no formato `arquivo → CA-n`.
3. Linha do índice em `specs/README.md`.
4. `docs/ui-desktop/inventario-controles.md` quando mexer em controles.
5. `AGENTS.md`: bloco "Trabalho em andamento" apontando para a parte atual.

## 5. Relatório de cada parte (formato)
- O que foi feito, com os arquivos.
- CAs cobertos, com a saída do teste **falhando antes** e **passando depois**.
- CAs revertidos ou substituídos, com motivo.
- Medições (CA-12, CA-13, CA-39) com CPU e build, ou `[medida pendente]`.
- Pendências e dúvidas de fidelidade, com a fonte consultada.
- Comando para reproduzir.
- "Nada commitado (aguardando autorização)."

## 6. Cortex
Projeto: `simulador-de-efeitos`. Ao começar, rode `handoff` com `target_agent: "antigravity"` e leia a página da Spec 049 registrada em 2026-11-15 (review, ordem, licenças). Ao fim de cada parte: `capture` de decisões, gotchas e fatos; `consolidate` e `write_page`; `promote_rules` se surgir regra nova.
