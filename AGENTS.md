# AGENTS.md — SONIX PRO // AMPFORGE™

## Fluxo de desenvolvimento — SDD

Este projeto segue **Spec Driven Development**: toda mudança de comportamento nasce de uma spec
em `specs/NNN-nome.md`, **antes** do código. As regras, a convenção de nomes e o **índice de
specs** (que é o changelog do projeto) estão em `specs/README.md` — leia-o antes de implementar.

- **Sem spec, sem código.** Se o pedido muda o que o sistema faz, o primeiro entregável é a spec.
- Critérios de aceite (`CA-n`) são afirmações **verificáveis**; testes citam o número no nome.
- **Regime SDD + TDD estrito:** cada `CA-n` vira um teste que falha antes da implementação (*red → green*).
- Spec e implementação são revisadas no **mesmo PR**.

> **Estado atual (2026-11-22):** Ordem de serviço `ANTIGRAVITY-auditoria-2026-11-20-ordem-de-servico.md` (`AUD-01`..`AUD-47`) e ordem corretiva `ANTIGRAVITY-auditoria-2026-11-20-correcao-da-entrega.md` (`COR-01`..`COR-08`) executadas na branch `fix/auditoria-2026-11-20`. Specs `001`–`052` implementadas (com ressalvas em `001`, `002`, `007`, `008`, `009`, `012`, `018` CA-6, `028`) e Specs `053`–`092` em `status: rascunho`. Portão final literal (§2.3): `cargo test --workspace --locked --no-fail-fast` (`576 passed; 0 failed; 0 ignored`, incluindo `contract_013::test_ca_02` sem `--skip`), `cargo test -p sonix-desktop --locked --no-fail-fast --features fault-injection` (`235 passed; 0 failed; 0 ignored`), `npm test` (`133 pass; 0 fail`), `node --test tests/contract/048-*.test.js` (`3 pass; 0 fail`, SSIM Main `0.8999` / Main_stage `0.8512` / Audio `0.8643`), `npm run test:e2e` (`E2E 033 GREEN`), UI build e 3 builds release (`sonix-app --features custom-protocol`, `sonix-desktop`, `sonix-plugin`) aprovados. Bloqueios aceitos (§0.7): `CA-36`/ASIO (`CPAL_ASIO_DIR`), hardware MIDI/USB para desconexão, medição física por cabo loopback (`018-CA-10.md`, `022-CA-14.md`, `T-25`) e validadores externos `pluginval`/`clap-validator`. Relatório completo em `docs/handoff/RELATORIO-auditoria-2026-11-20-execucao.md`. Design de referência em `docs/ui-desktop/canvas/`, renderizado em `docs/ui-desktop/referencia/`.







<!-- BEGIN CORTEX RULES -->
# Regras Promovidas do Projeto Cortex

## Rule [20261108-162135-sdd-6a25]
# Padrão Spec Driven Development (SDD) e Regime TDD Estrito

O projeto **SONIX PRO // AMPFORGE™** (`simulador-de-efeitos`) adota formalmente o padrão **Spec Driven Development (SDD)** definido em `.agents/skills/spec-driven-development` e documentado no índice de especificações `specs/README.md`.

## Regras Obrigatórias:
1. **Sem Spec, Sem Código**: Toda mudança de comportamento deve obrigatoriamente nascer de uma spec numerada em `specs/NNN-nome.md` (a partir de `specs/_template.md`) e ser registrada no índice de `specs/README.md` antes de qualquer linha de código de produção ser escrita.
2. **Critérios de Aceite Verificáveis (`CA-n`)**: Toda spec define critérios de aceite atômicos, auditáveis e reprováveis (`CA-1`, `CA-2`, etc.), incluindo casos de erro e não-regressão.
3. **Regime SDD + TDD Estrito**: Cada critério `CA-n` vira um teste de contrato automatizado que **falha antes** da implementação (*red → green*), citando o número do `CA-n` no nome do caso de teste.
4. **Unidade de Revisão**: A especificação, os testes de contrato e o código de produção são revisados e integrados no mesmo PR/commit.

## Rule [20261111-023654-dsp-fd17]
# Zero-Alloc Realtime Audio Safety e Desacoplamento de Pitch Detection (Spec 026)

## Regra Obrigatória de Engenharia de Áudio em Tempo Real
**O callback de áudio nunca pode alocar memória na heap, bloquear threads (locks/mutexes) ou executar processamento algorítmico pesado de análise.**

### Detalhes de Implementação:
1. **Ring Buffers e Cópia Lock-Free**: O callback [`RealtimeDspChain::process`](crates/sonix-desktop/src/audio.rs) apenas alimenta amostras do sinal limpo de entrada (após DC blocker e antes de distorções) para um acumulador circular pré-alocado.
2. **Scratch Buffers Pré-Alocados**: O algoritmo de autocorrelação/diferença de pitch (YIN) em [`TunerProcessor`](crates/ampforge-dsp/src/tuner.rs) opera exclusivamente sobre arrays estáticos pré-dimensionados (`d_scratch`, `d_prime_scratch` de até 2048 amostras), eliminando alocações dinâmicas de vetores.
3. **Execução Desacoplada da Thread de Áudio**: A análise pesada de pitch é disparada sob demanda ou em taxa de telemetria (30 a 60 Hz) na thread de I/O / API HTTP, enquanto a telemetria reporta `estimated_roundtrip_ms = driver_input_latency_ms + driver_output_latency_ms + (algorithmic_latency_samples / sample_rate) * 1000.0` (sem afirmar "< 1,5 ms" fixo).
4. **Verificação Automatizada**: Todo novo componente de análise de áudio deve incluir teste de contrato específico (`test_ca_08_audio_callback_zero_alloc_and_zero_locks`) validando ausência de alocação de memória e invariância de desempenho sob carga contínua.

## Rule [20261113-195929-regra-0a2f]
# Todo controle visível funciona no backend como o equipamento real

**Origem:** pedido explícito do usuário (2026-11-13): "cada componente, botões, pedais e etc devem ter todos os controles funcionando no backend como o equipamento real".

## Regra
Não existe controle decorativo. Todo knob, chave, jack, botão, pedal, footswitch ou controle de rack visível na UI precisa ter:
1. **entrada no registro único** `controls.registry.ts`: `id`, chave do parâmetro no namespace da Spec 014, tipo, faixa, padrão, unidade e **efeito esperado do equipamento real**;
2. **chave aceita e validada pelo backend** (tabela única da Spec 014, validação atômica);
3. **teste de efeito**: mover o controle de ponta a ponta (ou trocar a opção) com sinal de teste definido muda a saída de forma mensurável **no sentido do equipamento real**. Exemplos:
   - `standby` silencia;
   - no Plexi 1959, a entrada LOW fica 6 dB abaixo da HIGH;
   - footswitch em bypass deixa a saída igual à entrada;
   - BASS/MIDDLE/TREBLE interagem como a rede FMV.

## Como aplicar
- Teste de inventário: reprova controle visível sem registro ou com chave rejeitada pelo backend.
- Controle cujo DSP ainda não existe aparece **desabilitado** com a dica "em implementação — Spec 0NN" e conta como **pendência**. A spec só vai para `implementada` sem pendências.
- Divergência de nome entre UI e motor (ex.: `decay` do pedal de gate → `release_ms`) fica no registro como mapeamento, sem chave duplicada.
- Controles só visuais (ex.: seletor do VU do compressor) são declarados como tal no registro.

Referência: Spec 033 §3, CA-4 e CA-5.

## Rule [20261114-131934-regra-d272]
# regua-medida-para-invariantes: invariante só conta como verificada por régua medida

**Origem:** review manual de 2026-11-14. Dois contratos passavam verdes enquanto o comportamento estava errado.

## Regra
Um CA que afirma uma invariante precisa de um teste que **meça** o comportamento e que **reprove** quando ele é violado. Ler o código ou cronometrar não serve como régua.

1. **Tempo real (Specs 037 CA-7, 044 CA-3):** zero-alloc no callback é verificado com um **alocador global de contagem**, ativo só na thread que chama `RealtimeDspChain::process`. Esse alocador conta alocações e desalocações. O teste precisa falhar quando se injeta um `Vec::with_capacity` no `process`.
2. **Fidelidade visual (Spec 048 CA-10):** spec de UI com design de referência tem régua visual. Um E2E captura cada tela em 1440×900 com dados de fixture e compara com o PNG de `docs/ui-desktop/referencia/` ([[20261114-131933-referencia-3a73]]). O teste reprova com SSIM < 0,85 por região ou < 0,80 na tela inteira.
3. **Teste de contrato que não falha quando se quebra o comportamento é tautológico:** antes do green, demonstre o red com a quebra real.

## Por quê
- A Spec 033 passou em todos os CAs (inventário, IPC, ausência de scroll) com uma UI que não se parecia com o design, porque não tinha CA de fidelidade.
- Os testes "zero-alloc" passavam enquanto o callback liberava a IR antiga, usava `Mutex` no osciloscópio e silenciava ao carregar IR ou preset.

<!-- END CORTEX RULES -->

