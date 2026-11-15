---
spec: 042
titulo: O bloco de pitch (Spec 007) não processa áudio — não existe método process
status: implementada
atualizado: 2026-11-14
---
# 042 — Bloco de pitch: implementar o processamento ou retirar a promessa

## Objetivo
Resolver a contradição entre a Spec 007 ("pitch polifônico", marcada como implementada) e o código, em que o bloco de pitch não toca nenhuma amostra.

## Contexto
Review manual da Spec 007 (2026-11-14), por leitura de `crates/ampforge-dsp/src/fx/pitch.rs`:
- `PitchProcessor` não tem `process()`. Só guarda `semitones`, `key`, `scale` e `mix` e calcula `transposition_ratio()` (`pitch.rs:48-100`).
- `calculate_harmonic_frequency` reconhece só C4 e D4. Para qualquer outra nota, devolve +4 semitons (`pitch.rs:88-100`), o que não é um harmonizador diatônico.
- O bloco não está na `RealtimeDspChain` nem na tabela única (`api.rs`); `PitchModel` (Whammy, Drop/Capo, Harmonizer, Detune) não chega ao áudio.
- O índice (`specs/README.md`) marca a Spec 007 como "Implementada".

## Comportamento esperado
Decisão do usuário (2026-11-14): implementar o bloco (ver Notas).
- Pitch shifter monofônico de baixa latência (granular/PSOLA ou no domínio da frequência) para Whammy/Drop/Detune, com `semitones` em −24…+24 e `mix`.
- O harmonizador diatônico calcula o intervalo pelo grau da escala em `key`/`scale`, a partir da nota detectada.
- O bloco entra na cadeia e na tabela única (`pitch_1.*`), com controle no registro da UI.

## Critérios de aceite
- **CA-1:** Com `semitones = +12` e mix 100%, uma senoide de 220 Hz na entrada produz saída cuja frequência fundamental detectada é 440 Hz ± 5 cents.
- **CA-2:** Com `semitones = −2` (Drop), uma senoide de 110 Hz sai a 98,0 Hz ± 5 cents.
- **CA-3:** No harmonizador em Dó maior com 3ª acima, as entradas C4, D4, E4, F4, G4, A4 e B4 produzem E4, F4, G4, A4, B4, C5 e D5 (±10 cents).
- **CA-4:** A latência adicionada pelo bloco é declarada em `algorithmic_latency_samples`, e o valor medido (impulso) é igual ao declarado (±1 amostra).
- **CA-5:** `pitch_1.*` é aceito pelo validador, consta no `controls.registry.ts` e tem teste de efeito (regra do projeto).
- **CA-6 (erro):** Com `semitones` fora de −24…+24, a API responde 400 e o estado não muda.
- **CA-7 (não-regressão):** Com o bloco em bypass, a saída é idêntica à entrada (bit a bit).

## Fora de escopo
- Pitch **polifônico** de verdade (shift por corda, estilo POG/Whammy DT): exige outra técnica e outra spec.
- Detecção de tom/escala automática.

## Invariantes tocadas
- **Todo controle visível funciona:** o bloco só aparece na UI quando o CA-5 passar.
- **Zero-alloc no callback:** os buffers de análise e síntese são pré-alocados em `prepare`.

## Testes que cobrem esta spec
- `crates/ampforge-dsp/tests/contract_042_pitch_shifter.rs` (`CA-1`, `CA-2`, `CA-3`, `CA-4`, `CA-7`)
- `crates/ampforge-dsp/tests/contract_007_mod_pitch_delay_reverb.rs` (`CA-2`, `CA-3`)
- `crates/sonix-desktop/tests/contract_042_pitch_api.rs` (`CA-5`, `CA-6`)
- `crates/sonix-desktop/tests/contract_037_realtime_safety.rs` (zero-alloc no callback)

## Notas
**Decisão do usuário (2026-11-14): implementar o bloco.** A alternativa de retirar a promessa (marcar a Spec 007 como revertida e remover `PitchProcessor`) foi descartada.
