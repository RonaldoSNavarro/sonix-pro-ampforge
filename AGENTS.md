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

<!-- END CORTEX RULES -->
