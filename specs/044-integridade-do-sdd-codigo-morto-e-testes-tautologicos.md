---
spec: 044
titulo: Specs marcadas como implementadas entregam código que a produção não usa e testes que não podem falhar
status: implementada
atualizado: 2026-11-14
---
# 044 — Integridade do SDD: código morto entregue como "implementado" e testes tautológicos

## Objetivo
Fazer o índice de specs e o inventário de controles dizerem a verdade sobre o que roda em produção, e garantir que cada teste de contrato possa reprovar o comportamento que diz verificar.

## Contexto
Review manual crate por crate das Specs 001–026 (2026-11-14). Vários entregáveis existem como tipos e testes, mas não fazem parte do caminho de áudio do app:

| Spec | Entregável | Uso em produção | Evidência |
|---|---|---|---|
| 001 | `DspEngine` (laço RT), `CpuMonitor`, `HalfBandOversampler`, `MixedPrecisionSolver` | Nenhum. `process_block` só aplica um ganho; `CpuMonitor` nunca recebe tempo medido; o oversampler interpola linearmente, faz média em caixa e **declara** 98–106 dB de rejeição sem medir; o solver de Newton parte da própria solução de `x^1.5 = v^1.5`. | `ampforge-dsp/src/engine.rs`, `cpu.rs`, `oversampling.rs:30-95`, `precision.rs:70-95`; grep sem uso fora de testes |
| 001 | Contadores zero-alloc/zero-lock | `rt_alloc_count` e `rt_lock_count` nunca são incrementados: o teste não pode falhar. | `engine.rs:21-22, 76-86` |
| 002 | `DagGraph`/`GridMatrix4x12` (4×12, ciclos, bypass) | Só estrutura de dados; o áudio usa uma cadeia fixa em `RealtimeDspChain`. | `ampforge-dsp/src/dag.rs:370-529`; `sonix-desktop/src/audio.rs:2473-2717` |
| 009 | `Preset`/`Scene`/`BlockNode`/`ModifierMatrix`/`FootswitchArray`/`GaplessCrossfader` | Nenhum (só `persistence` é usado). `count_active_channels_in_scene` devolve a constante 1; `GaplessCrossfader` ignora `crossfade_samples`. | `sonix-core/src/preset.rs:318-320, 415-424`; `modifiers.rs` |
| 012 | Plugins VST3/CLAP | `SonixPluginProcessor::process` copia a entrada na saída; não há binário VST3/CLAP. | `sonix-plugin/src/lib.rs:112-118` |
| 012/013 | Host MIDI | Portas fixas; as mensagens vão para uma fila que ninguém consome. | `sonix-desktop/src/midi.rs` |
| 014 | Suavização de ganho (CA-8) | `gain_smoother` avança, mas o valor nunca é aplicado ao áudio; o teste lê `current_effective_gain()`. | `audio.rs:2522-2525, 1784-1786` |
| 033 | Inventário de controles "OK" | Itens marcados OK sem efeito: modelo de microfone, ressonância do gabinete, troca de dispositivo, ratio/modo do gate, modelo de modulação. | `docs/ui-desktop/inventario-controles.md`; Specs 036, 038, 040, 041 |

## Comportamento esperado
- O índice (`specs/README.md`) usa estados honestos: CAs entregues só como biblioteca ou só em teste ficam marcados `implementada (CA-n revertida)`, com o motivo nas Notas da spec.
- Todo tipo público entregue por spec é usado no caminho de produção **ou** aparece em `docs/biblioteca-sem-uso-em-producao.md`, com a spec que vai ligá-lo ou removê-lo.
- Os testes de contrato verificam o efeito observável (áudio, resposta da API), nunca um campo interno que o teste mesmo alimenta.
- O inventário de controles só marca OK o que tem teste de efeito citado.

## Critérios de aceite
- **CA-1:** No índice, as linhas das Specs 001, 002, 007, 008, 009, 012 e 014 mostram `implementada (CA-… revertida)` nos CAs da tabela acima, e cada spec ganha uma nota com o motivo e a spec que retoma o item (034–043).
- **CA-2:** Um teste lista os itens `pub` de `ampforge-dsp` e `sonix-core` e falha se algum não for referenciado em `sonix-desktop`/`sonix-app` **nem** listado em `docs/biblioteca-sem-uso-em-producao.md`.
- **CA-3:** O teste zero-alloc usa um alocador global de contagem sobre `RealtimeDspChain::process`. Ao injetar `Vec::new()` com capacidade no `process`, o teste **falha** (mutação verificada no PR).
- **CA-4:** O teste de suavização da Spec 014 mede a saída de áudio: um degrau de ganho 2→8 com senoide de entrada produz envelope que atinge ≥ 63% do novo nível em ≤ 20 ms e < 63% em < 5 ms. Com a suavização removida, o teste falha.
- **CA-5:** `HalfBandOversampler::stopband_attenuation_db` devolve a atenuação **medida** do filtro implementado (teste por FFT); se o filtro continuar sendo interpolação linear, o valor relatado é o medido e a Spec 001 CA-5 é marcada revertida.
- **CA-6:** `CpuMonitor` recebe o tempo real de cada callback da cadeia ao vivo, ou é removido com a nota correspondente.
- **CA-7:** `docs/ui-desktop/inventario-controles.md` tem, em cada linha "OK", o caminho do teste de efeito. As linhas sem teste viram `pendente — Spec 0NN`. Um teste confere que todo caminho citado existe.
- **CA-8:** A descrição da Spec 012 e o README deixam de afirmar plugins VST3/CLAP entregues enquanto `sonix-plugin` for passthrough.

## Fora de escopo
- Implementar o DAG 4×12 dinâmico, os modifiers ou os plugins: cada um exige spec própria. Aqui só se corrige o registro e os testes.
- Reescrever specs antigas: elas recebem nota e status honesto, não são reescritas (regra do SDD).

## Invariantes tocadas
- **SDD + TDD estrito:** esta spec restaura a premissa de que cada CA tem um teste que falha quando o comportamento falta.

## Testes que cobrem esta spec
- `crates/sonix-desktop/tests/contract_044_sdd_integrity.rs` → CA-1, CA-2, CA-3, CA-5, CA-6, CA-7 e CA-8 verificados com régua estática e alocador de contagem global.
- `crates/sonix-desktop/tests/contract_014_ui_dsp_bridge.rs` (`test_ca_08_parameter_smoothing_step_response`) → CA-4 medido na saída real de áudio.
- `crates/ampforge-dsp/tests/contract_001_nucleo_dsp.rs` (`test_ca_05_polyphase_half_band_oversampling_rejection_over_96db`) → CA-5 filtro polyphase half-band Kaiser 127-tap com atenuação medida via DFT (`>= 96 dB`).
