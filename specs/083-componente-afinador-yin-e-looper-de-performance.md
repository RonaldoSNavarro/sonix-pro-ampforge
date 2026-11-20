---
spec: 083
titulo: Componente Compartilhado de Afinador (YIN Estroboscópico e Polifônico) e Looper de Performance
status: rascunho
tipo: componente-compartilhado
atualizado: 2026-11-19
dependencias: [053, 076]
---
# 083 — Componente de Afinador (YIN / Polifônico) e Looper de Performance

## 1. Identificação e objetivo
- **Identificador estável (`module_id`):** `component.stage.tuner_and_looper` (chaves `tuner.*`, `looper.*`)
- **Revisão (`model_revision`):** `v1-yin-desacoplado-looper-o1`
- **Categoria:** Componentes utilitários compartilhados de Palco (Afinador + Looper)
- **Objetivo:** Formalizar os contratos modulares e invariantes de tempo real do **Afinador de Alta Precisão** (desacoplado do callback de áudio via `CleanInputRing` lock-free — Regra `20261111-023654-dsp-fd17`) e do **Looper de Performance** (posicionável `PreChain` / `PostChain`, 60 s pré-alocados por taxa, `Undo`/`Redo` O(1) sem alocação, `Reverse` e `Half-Speed`).

## 2. Escopo
- **Afinador (`TunerProcessor`):**
  - Captura lock-free na entrada limpa (`CleanInputRing`, 0 alocações e 0 análise pesada no callback RT!).
  - Algoritmo **YIN com interpolação parabólica** (cobertura `60 Hz–1,4 kHz`, precisão `< ±0,5 Hz` / `±0,1 cent` em senoides limpas) + detecção **polifônica de 6 cordas** (`E2, A2, D3, G3, B3, E4`), temperamentos (`EqualTemperament`, `SweetenedGuitar`), referência `A4` (`430–450 Hz`), retorno honesto `"---"` sem sinal (sem dados fictícios — Spec 051 CA-13) e rampa de mute suave de `5 ms` (`tuner.output = mute | bypass`).
- **Looper (`LooperProcessor`):**
  - Máquina de estados `Idle → Record → Play ↔ Overdub → Stop`, posicionamento `PreChain` (permite ajustar timbres com loop tocando) ou `PostChain` (grava timbre final).
  - Capacidade de 60 s dimensionada em `prepare(sample_rate, 60)` fora do RT.
  - `Undo` / `Redo` em **O(1)** via `std::mem::swap` de buffers pré-alocados (sem perda de camadas — Spec 017 / 043 CA-5), `Reverse` e `Half-Speed` (incluindo overdub em meia velocidade sem sobrescrever/distorcer).

## 3. Fora de escopo
- Síntese de metrônomo/bateria integrada ou exportação multipista de loops para DAW.
- Análise de pitch dentro do callback de áudio (explicitamente **proibida** pela regra `20261111-023654-dsp-fd17`).

## 4. Equipamento ou componente de referência
- **Referências:** Peterson StroboStomp / TC Electronic PolyTune (afinador estroboscópico + polifônico); Boss RC-1 / Ditto Looper (transporte simples + Pre/Post + Reverse/Half-Speed + Undo/Redo).

## 5. Revisão e variantes
- **Revisão:** `v1-yin-desacoplado-looper-o1` (consolidando Specs 008, 017, 026, 029, 043 e 051 CA-13).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | A. de Cheveigné & H. Kawahara — *YIN, a fundamental frequency estimator for speech and music* (JASA 2002) | Algoritmo YIN | Função diferença, CMND (`d'`), limiar absoluto, interpolação parabólica | JASA Vol. 111, No. 4, 2002 | 2026-11-19 | `[confirmado em documentação]` | Equações exatas de YIN e refinamento parabólico sub-amostra |
| F2 | Regra Cortex `20261111-023654-dsp-fd17` (`AGENTS.md`) | Invariante do projeto | Desacoplamento de Pitch Detection | `AGENTS.md` | 2026-11-19 | `[confirmado em documentação]` | Proibição absoluta de rodar YIN no callback de áudio; uso obrigatório de ring buffer + scratch buffers pré-alocados |
| F3 | Specs 008, 017, 026, 029, 043 e 051 CA-13 | Contratos existentes | `tuner.rs`, `looper.rs` | `specs/` | 2026-11-19 | `[confirmado em documentação]` | Precisão `< ±0,5 Hz`, polifônico, temperamento, mute 5 ms, undo/redo O(1), half-speed, telemetria 30 Hz |

## 7. Fundamentação algorítmica
- **YIN desacoplado:** o callback RT apenas grava amostras limpas pós-DC-blocker no `CleanInputRing` atômico (`O(N)` de cópia escalar). A thread de telemetria (`streams.rs`, 30 Hz) lê a janela mais recente (ex.: 2048–4096 amostras) e executa YIN + interpolação parabólica sobre `d_scratch` / `d_prime_scratch` pré-alocados, eliminando completamente jitter na thread de áudio `[confirmado em documentação: F1, F2]`.
- **Detecção de silêncio na cauda (`~22 ms`):** evita exibir nota congelada quando o guitarrista abafa as cordas, retornando `"---"` e `0.0 cents` imediatamente.
- **Looper O(1) Undo/Redo:** mantém `base_buffer`, `overdub_buffer` e `undo_buffer` pré-alocados em `prepare()`; entrar em `Overdub` (vindo de outro estado) copia o snapshot atual para `undo_buffer`, e `undo()`/`redo()` trocam ponteiros via `std::mem::swap` em O(1) sem alocar.

## 8. Comportamento esperado
- **Precisão do afinador:** para senoides nas 6 cordas padrão (`E2=82,41`, `A2=110`, `D3=146,83`, `G3=196`, `B3=246,94`, `E4=329,63 Hz`) e até `1,4 kHz`, erro `< ±0,5 Hz` (alvo `±0,1 cent` em regime estável), reagindo à mudança de `tuner.ref_hz` (`430–450 Hz`).
- **Sem dados fictícios:** com entrada silenciosa (`RMS < -64 dBFS`), retorna `note_name = "---"`, `frequency_hz = 0.0`, `cents_deviation = 0.0`.
- **Mute suave:** com `tuner.enabled = true` e `tuner.output = mute`, silencia a saída em `5 ms` sem clique; em `bypass`, mantém o áudio audível.
- **Looper fiel:** grava até 60 s em qualquer taxa (`44,1–96 kHz`), auto-fecha para `Play` ao atingir capacidade sem panic, preserva camadas em `Undo`/`Redo` e suporta `Reverse` e `Half-Speed` sem artefatos ou alocações.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Tuner On/Off | `tuner.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa leitura e mute (se configurado) | Flag RT + rampa 5 ms |
| Tuner Output | `tuner.output` | enum | id | `mute, bypass` | `mute` | discreto | Silencia ou mantém sinal ao afinar | Ganho de rampa `tuner_mute_gain` |
| Referência A4 | `tuner.ref_hz` | number | Hz | `430.0..=450.0` | `440.0` | linear | Desloca referência MIDI A4 | Propagado ao `TunerProcessor` e `CalibrationSession` |
| Tuner Mode | `tuner.mode` | enum | id | `strobe, polyphonic` | `strobe` | discreto | Monofônico YIN vs 6 cordas polifônico | Seleciona algoritmo |
| Looper Action | `looper.action` | enum | ação | `record, play, overdub, stop, undo, redo, reverse, half_speed, clear` | `stop` | discreto | Comanda FSM do looper | Eventos lock-free |
| Looper Position | `looper.position` | enum | id | `pre, post` | `post` | discreto | Antes ou depois da cadeia de amp/efeitos | Posição no grafo |

## 10. Contratos de integração
- `TunerProcessor` consome `CleanInputRing` fora da thread RT (em `api.rs` / `streams.rs` a 30 Hz); nunca roda dentro de `process()`.
- `LooperProcessor` é mantido em instância única fora do par ativa/sombra da troca de cenas (Spec 049 CA-1) para que o loop não seja interrompido ao trocar de cena/preset!

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/tuner.rs` (493 linhas) e `crates/ampforge-dsp/src/looper.rs` (275 linhas).
- **Dívidas registradas:**
  1. `looper.rs:156` e `174` copiam `overdub_buffer[..loop_length]` no `overdub()` e no `undo()` durante `Overdub`: em um loop de 60 s @ 96 kHz (`5,76 M` floats ≈ `23 MB`), esse `copy_from_slice` dentro do callback RT pode causar pico de CPU (> 2 ms)! **Migrar para cópia incremental/double-buffered ou executar transição de undo fora do caminho crítico de amostra.**
  2. `looper.rs:235` usa leitura truncada `play_pos.floor()` em `half_speed`, o que gera leve degrau (sample-and-hold) em `0.5×`; usar interpolação linear em `half_speed` suaviza drasticamente a reprodução em meia velocidade.
  3. `tuner.rs:194–199` tem `.resize()` condicional em `d_scratch`/`d_prime_scratch` caso `max_lag > 4096`; pré-dimensionar em `prepare(sample_rate)` para `96 kHz` garante zero alocação mesmo fora da RT.
- **Estrutura alvo:** `crates/ampforge-dsp/src/shared/{tuner.rs, looper.rs}`.

## 12. Estratégia DSP
- Afinador: YIN + interpolação parabólica + banco de filtros/Goertzel polifônico para 6 cordas (fora da RT); rampa linear de 5 ms para mute na RT.
- Looper: 3 buffers circulares pré-alocados em `prepare(sr, 60)` com interpolação linear em `half_speed` e troca de ponteiros O(1) para undo/redo.

## 13. Requisitos de tempo real
- Callback RT: zero execução de YIN, zero alocação/desalocação heap, zero locks, zero panic; eliminar `memcpy` grande (> 1 MB) na thread RT no `LooperProcessor::overdub()`.

## 14. Gestão de estado
- `TunerProcessor` (scratch buffers + `last_poly_cents`); `LooperProcessor` (`base_buffer`, `overdub_buffer`, `undo_buffer`, `state`, `play_pos`, `loop_length`). Nenhum buffer de áudio gravado no looper é serializado em presets.

## 15. Latência
- **`0` amostras** no caminho de áudio para ambos (`Tuner` em tap paralelo; `Looper` leitura/escrita direta).

## 16. Consumo de CPU
- Afinador na thread RT: **`< 0,1%`** (apenas cópia escalar para `CleanInputRing` + rampa de mute).
- Looper na thread RT: **`<= 0,5%`** @ 128 frames / 48 kHz (`--release`).

## 17. Testes
- Senoides nas 6 cordas (`E2..E4`) validando erro `< ±0,5 Hz` e resposta a `tuner.ref_hz = 432 / 440 / 444 Hz`; silêncio retornando `"---"`; acorde de 6 cordas no modo polifônico; rampa mute 5 ms sem clique; ciclo completo do Looper (`Record → Play → Overdub → Undo → Redo → Reverse → Half-Speed → Clear`) em `44,1 / 48 / 96 kHz`; zero-alloc RT (incluindo `test_ca_08_audio_callback_zero_alloc_and_zero_locks`).

## 18. Critérios de aceite
- **CA-1 (Precisão YIN e Sem Dados Fictícios):** Para senoides em `E2, A2, D3, G3, B3, E4`, erro `< ±0,5 Hz` com `ref_hz = 440 Hz` e deslocamento coerente em `ref_hz = 432 Hz`; sob silêncio retorna `"---"` e `0.0 cents`.
- **CA-2 (Desacoplamento RT do Afinador):** O callback de áudio nunca invoca `detect_pitch()`; alocador de contagem na thread RT registra `0` alocações e `0` desalocações.
- **CA-3 (Mute Suave 5 ms):** Ativar `tuner.enabled = true` com `output = mute` atenua para `0.0` em `5,0 ± 0,5 ms` nas taxas `44,1`, `48` e `96 kHz` sem degrau.
- **CA-4 (Looper Pre/Post, Undo/Redo, Reverse e Half-Speed):** Capacidade de 60 s respeitada em `44,1`, `48` e `96 kHz`; `Undo`/`Redo` restauram camadas sem perda e sem alocação; `Half-Speed` reproduz a metade da velocidade com interpolação suave.
- **CA-5 (Não-regressão):** Testes das Specs 008, 017, 026, 029, 043 e 051 CA-13 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`, `Spec 076` (`CleanInputRing`).
- **Relacionadas / Legadas:** `Specs 008, 017, 026, 029, 043, 049, 051`.

## 20. Limitações conhecidas
- Modo polifônico foca na afinação padrão de 6 cordas (`E2–E4`) e drop D; afinações muito baixas (< 58 Hz, ex.: 8 cordas F#1) exigem ampliação de `max_lag` na janela de análise.

## 21. Riscos técnicos
- O `copy_from_slice` de `overdub_buffer` (até 23 MB em 60 s @ 96 kHz) na transição de `Overdub` dentro da thread RT é um risco real de dropout; priorizar eliminação desse `memcpy` na implementação da Spec 083.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `008`, `017`, `026`, `029`, `043`, `049`, `051`.
- **Código atual:** `crates/ampforge-dsp/src/{tuner.rs, looper.rs}`, `apps/sonix-app/src-tauri/src/streams.rs`.
- **Testes existentes / futuros:** `contract_008_tuner_looper.rs`, `contract_017_resilience_ir_looper.rs`, `contract_026_live_tuner.rs`, `contract_043_tuner_looper.rs`, futuro `contract_083_tuner_looper.rs`.
