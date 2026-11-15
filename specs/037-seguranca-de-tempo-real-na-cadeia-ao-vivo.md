---
spec: 037
titulo: A cadeia ao vivo aloca e libera memória no callback e silencia durante carga de IR e preset
status: implementada
atualizado: 2026-11-14
---
# 037 — Segurança de tempo real na cadeia ao vivo (zero alocação, zero bloqueio, sem buraco)

## Objetivo
Cumprir de fato a regra do projeto: o callback de áudio nunca aloca, libera ou espera, e carregar uma IR ou um preset não abre buraco no som.

## Contexto
Review manual (2026-11-14) de `RealtimeDspChain` e da API, por leitura de código. A regra "Zero-Alloc Realtime Audio Safety" (AGENTS.md, Spec 026) é violada nestes pontos:

| Ponto | Evidência |
|---|---|
| `String` na thread de áudio | `DspParamUpdate::InputSource` → `set_input_source(s)` faz `source.to_string()` (`audio.rs:1564-1571, 1723-1725`). `SplitCrossoverHz` → `set_split_mode` faz `mode.to_string()` (`audio.rs:2227-2229`). `TunerEnabled(false)` cria `"---".to_string()` (`audio.rs:1685`). Os três são drenados dentro do callback. |
| `resize` no callback | `process()` redimensiona 18 buffers se `len > 2048` (`audio.rs:2476-2495`). |
| Desalocação no callback | Ao fim do crossfade de IR, `self.custom_ir = self.target_ir.take()` libera o `Vec` da IR antiga dentro de `process` (`ampforge-dsp/src/cab.rs:880`). |
| Silêncio durante carga de IR | `/api/cab/ir` faz o parse e o resample (sinc de 48 taps por amostra) **segurando** o lock bloqueante da cadeia (`api.rs:2693-2696`). O callback faz `try_lock`, falha e emite silêncio (`audio.rs:725-731`). |
| Silêncio durante carga de preset | `apply_preset_params` aplica dezenas de chaves com `dsp_ref.lock()` (`api.rs:3141`). Mesmo efeito. |
| Mutex na telemetria | `push_waveform` usa `Mutex::try_lock` no callback (`audio.rs:2792`); o leitor clona o vetor segurando o lock (`audio.rs:2812-2816`). |
| Teste que não mede | O teste zero-alloc da Spec 001 lê contadores (`rt_alloc_count`) que nenhum código incrementa (`ampforge-dsp/src/engine.rs:21-22,76-86`). |

## Comportamento esperado
- Parâmetros textuais viram enums `Copy` antes de entrar na fila; a cadeia guarda enums, não `String`.
- `prepare(sample_rate, max_block)` dimensiona todos os buffers. Callbacks maiores são processados em fatias de `max_block`, sem `resize`.
- Recursos pesados (IR, DI, tabelas) são construídos fora da thread de áudio, entregues por fila e devolvidos por outra fila para descarte fora da thread de áudio.
- A carga de IR e de preset não segura o lock da cadeia: a preparação acontece fora, e a troca é uma mensagem.
- A telemetria de forma de onda usa um ring lock-free com número de sequência.

## Critérios de aceite
- **CA-1:** Com um alocador global de contagem ativo só na thread que chama `process`, 10 000 blocos de 128 amostras, intercalados com `InputSource`, `SplitCrossoverHz`, `TunerEnabled(true/false)`, `LooperAction(*)`, `AmpModel` e `Cab1Speaker`, registram **0 alocações e 0 desalocações**.
- **CA-2:** Depois de `prepare(48000, 256)`, `process` com 1024 amostras produz a mesma saída que 4 chamadas de 256 (diferença ≤ 1e-6) e registra 0 alocações.
- **CA-3:** Durante e depois do fim de um crossfade de IR, a thread de áudio registra 0 desalocações. A IR antiga é liberada numa thread de serviço (contador de descarte = 1).
- **CA-4:** Carregar uma IR de 500 ms a 96 kHz durante a reprodução de uma senoide não gera nenhum bloco de saída com RMS < −60 dBFS (zero blocos silenciosos), nem aumenta `dsp_contention_silence_blocks`.
- **CA-5:** Carregar um preset durante a reprodução de uma senoide não gera bloco silencioso. O tempo máximo de qualquer seção crítica compartilhada com o callback é ≤ 50 µs (medido no teste).
- **CA-6:** `push_waveform` e `snapshot` não usam `Mutex`. Um leitor que lê em loop por 1 s não faz o escritor perder nenhum frame (número de sequência contínuo).
- **CA-7:** O teste zero-alloc da Spec 001 (`test_ca_08_audio_callback_zero_alloc_and_zero_locks`) passa a usar o alocador de contagem sobre `RealtimeDspChain::process`. Com uma alocação injetada de propósito, ele **falha**.
- **CA-8 (não-regressão):** Os contratos das Specs 014, 024, 026 e 028 continuam passando.

## Fora de escopo
- O algoritmo de convolução (FIR direto → particionado) fica na Spec 038.
- A unificação do caminho de parâmetros em fila única fica na Spec 034.

## Invariantes tocadas
- **Zero-alloc / zero-lock no callback:** esta spec é a implementação verificável dessa invariante.
- **Pitch detection desacoplada (Spec 026):** mantida; o afinador continua fora do callback.

## Testes que cobrem esta spec
- `crates/sonix-desktop/tests/contract_037_realtime_safety.rs` (CA-1, CA-2, CA-3, CA-4, CA-5, CA-6, CA-7, CA-8 — 10 testes passando com alocador global de contagem por thread)
- `crates/sonix-desktop/tests/contract_026_live_tuner.rs` (`test_ca_08_audio_callback_zero_alloc_and_zero_locks_with_tuner_active` com alocador de contagem)
- `crates/ampforge-dsp/tests/contract_001_nucleo_dsp.rs` (`test_ca_03_zero_alloc_and_zero_lock_during_process_block` com alocador de contagem)
