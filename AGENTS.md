# AGENTS.md — SONIX PRO // AMPFORGE™

## Fluxo de desenvolvimento — SDD

Este projeto segue **Spec Driven Development**: toda mudança de comportamento nasce de uma spec
em `specs/NNN-nome.md`, **antes** do código. As regras, a convenção de nomes e o **índice de
specs** (que é o changelog do projeto) estão em `specs/README.md` — leia-o antes de implementar.

- **Sem spec, sem código.** Se o pedido muda o que o sistema faz, o primeiro entregável é a spec.
- Critérios de aceite (`CA-n`) são afirmações **verificáveis**; testes citam o número no nome.
- **Regime SDD + TDD estrito:** cada `CA-n` vira um teste que falha antes da implementação (*red → green*).
- Spec e implementação são revisadas no **mesmo PR**.




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
1. **Ring Buffers e Cópia Lock-Free**: O callback [`RealtimeDspChain::process`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/sonix-desktop/src/audio.rs) apenas alimenta amostras do sinal limpo de entrada (após DC blocker e antes de distorções) para um acumulador circular pré-alocado.
2. **Scratch Buffers Pré-Alocados**: O algoritmo de autocorrelação/diferença de pitch (YIN) em [`TunerProcessor`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/tuner.rs) opera exclusivamente sobre arrays estáticos pré-dimensionados (`d_scratch`, `d_prime_scratch` de até 2048 amostras), eliminando alocações dinâmicas de vetores.
3. **Execução Desacoplada da Thread de Áudio**: A análise pesada de pitch é disparada sob demanda ou em taxa de telemetria (30 a 60 Hz) na thread de I/O / API HTTP, garantindo que o callback de áudio mantenha jitter determinístico e latência ultrabaixa (< 1.5 ms).
4. **Verificação Automatizada**: Todo novo componente de análise de áudio deve incluir teste de contrato específico (`test_ca_08_audio_callback_zero_alloc_and_zero_locks`) validando ausência de alocação de memória e invariância de desempenho sob carga contínua.

<!-- END CORTEX RULES -->
