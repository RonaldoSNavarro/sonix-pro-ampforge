---
spec: 075
titulo: Bloco de Pitch (Whammy, Drop Capo, Harmonizer Diatônico e Micro Detune)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053]
---
# 075 — Bloco de Pitch (Whammy, Drop Capo, Harmonizer e Micro Detune)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.pitch.shifter_suite` (submodelos: `whammy`, `drop_capo`, `harmonizer`, `micro_detune`; chaves `pitch_1.*`)
- **Revisão (`model_revision`):** `v1-psola-crossfade-diatonic`
- **Categoria:** Pedal de Pitch / Transposição (módulo independente)
- **Objetivo:** Especificar formalmente os 4 modos de processamento de pitch implementados em `fx/pitch.rs` — **Whammy** (`-24..+24 st` expressivo), **Drop Capo** (afinação instantânea), **Harmonizer Diatônico Inteligente** (detecção de fundamental + intervalo diatônico por tonalidade/escala) e **Micro Pitch Detune** — com latência declarada, alinhamento de fase entre grãos e segurança em tempo real.

## 2. Escopo
- Transposição em tempo real (`-24` a `+24` semitons, razão `2^(st/12)`) por síntese granular de dois grãos janelados (Hann) com busca de alinhamento de fase por correlação (`find_aligned_delay`) para minimizar cancelamento/tremolo.
- Harmonizer Diatônico suportando as 12 tonalidades (`C..B`) e 9 escalas (`Major`, `NaturalMinor`, `HarmonicMinor`, `MelodicMinor`, `Dorian`, `Phrygian`, `Lydian`, `Mixolydian`, `Locrian`), detectando `f0` na entrada e transpondo para o grau diatônico correspondente (terça diatônica + offset de semitons).
- Micro Detune (`±cents` finos, padrão `+0,15 st` ≈ `+15 cents` para espessamento estilo Eventide H3000).
- Integração com `EXP 1` / Modifiers (Spec 084) para varredura contínua de Whammy.

## 3. Fora de escopo
- Separação polifônica por Phase Vocoder de alta latência (> 40 ms) ou redes neurais — nesta revisão prioriza-se baixa latência (`~21 ms` a 48 kHz) para tocabilidade ao vivo; evolução para algoritmo polifônico dedicado de acordes em `Drop Capo` fica registrada como limitação conhecida em §20.
- Pedal de Wah (`wah_1`), já especificado na Spec 052 (status `aprovada`, aguardando autorização de implementação).

## 4. Equipamento ou componente de referência
- **Referências históricas:** DigiTech Whammy (`whammy`), DigiTech Drop (`drop_capo`), Boss PS-6 / Eventide Harmonizer (`harmonizer`), Eventide H3000 MicroPitch (`micro_detune`).
- **Papel na cadeia de sinal:** Posicionado tipicamente no início da cadeia (pré-drive/pré-amp) para rastreamento mais limpo da fundamental.

## 5. Revisão e variantes
- **Revisão de referência:** `v1-psola-crossfade-diatonic` (consolidando Spec 042).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | U. Zölzer — *DAFX* (2ª ed.) | Cap. 6 / Cap. 7 | Time-Domain Pitch Shifting (Overlap-Add / SOLA / Crossfaded Delay Lines) | Wiley, 2011 | 2026-11-19 | `[confirmado em documentação]` | Grãos sobrepostos com janela Hann, busca de correlação cruzada para continuidade de fase |
| F2 | Spec 042 (`CA-1`..`CA-7`) | Contratos existentes | `fx/pitch.rs` | `specs/042-bloco-de-pitch-sem-processamento.md` | 2026-11-19 | `[confirmado em documentação]` | Precisão de transposição, harmonização diatônica, latência reportada e RT safety |

## 7. Fundamentação elétrica / algorítmica
- Duas linhas de leitura moduladas em rampa sobre um buffer circular de `16384` amostras com janelas Hann complementares (`w0 + w1 = 1`), defasadas em `0,5` período de grão.
- Ao reiniciar um grão no ponto de peso zero (`w == 0`), `find_aligned_delay()` busca o deslocamento com maior correlação com o grão ativo, alinhando a fase da forma de onda e eliminando batimentos de cancelamento `[confirmado em documentação: F1, F2]`.
- No modo `Harmonizer`, um detector por autocorrelação normalizada (executado periodicamente a cada 512 amostras sobre janela pré-alocada de 2048 amostras) estima `f0` entre `70 Hz` e `1200 Hz` e projeta a nota na escala/tonalidade configurada.

## 8. Comportamento esperado
- **Exatidão de transposição:** em `whammy` / `drop_capo` com `+12 st` (oitava acima) ou `-12 st` (oitava abaixo), a frequência dominante da saída de uma senoide `220 Hz` ou `440 Hz` situa-se dentro de **`±1%` (`±17 cents`)** do alvo (`440 Hz` / `220 Hz`).
- **Harmonização diatônica correta:** em `C Major`, entrada `C4 (261,63 Hz)` produz terça maior `E4 (~329,63 Hz)`, enquanto entrada `D4 (293,66 Hz)` produz terça menor `F4 (~349,23 Hz)`.
- **Alinhamento dry/wet:** o sinal seco é lido em `center_delay` (`1024` amostras @ 48 kHz), alinhado temporalmente ao sinal transposto, evitando eco artificial ao misturar `Mix = 50%`.
- **Latência honesta:** `algorithmic_latency_samples()` retorna `0` quando bypassado ou `Mix = 0%`, e `round(1024 * sr / 48000)` quando ativo (`~21,3 ms` @ 48 kHz).

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no algoritmo | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `pitch_1.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa/bypassa bloco (`latency=0` em bypass) | Bypass limpo |
| Model | `pitch_1.model` | enum | id | `whammy, drop_capo, harmonizer, micro_detune` | `whammy` | discreto | Seleciona algoritmo de razão/detecção | Seleção de modo |
| Semitones | `pitch_1.semitones` | number | st | `-24.0..=24.0` | `0.0` | linear | Razão `2^(st/12)` (ou offset de detune/harmonia) | `transposition_ratio()` |
| Key | `pitch_1.key` | enum | nota | `C..B` (12 tons) | `C` | discreto | Tônica da escala diatônica | `key_root_pc` |
| Scale | `pitch_1.scale` | enum | modo | 9 escalas diatônicas | `major` | discreto | Tabela de 7 graus da escala | `scale_intervals` |
| Mix | `pitch_1.mix` | number | % | `0.0..=100.0` | `100.0` | linear | Proporção dry alinhado / wet transposto | `dry*(1-m) + wet*m` |

## 10. Contratos de integração
- `module_id = "pedal.pitch.shifter_suite"`, `model_revision = "v1-psola-crossfade-diatonic"`.
- Reporta mudança de `latency_samples()` ao grafo (Spec 053) ao ligar/desligar o bloco para compensação PDC em caminhos paralelos.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/pitch.rs` (linhas 1–416).
- **Dívidas registradas:**
  1. `detect_fundamental_hz()` (`pitch.rs:232–280`) executa autocorrelação O(N×lag) dentro do callback `process()` a cada 512 amostras quando em modo `Harmonizer`: medir custo de pico de CPU em `--release` (em 96 kHz, `max_lag` cresce 2×) e limitar/otimizar com decimação 2× ou YIN leve para garantir previsibilidade de pico no deadline de 128 frames!
  2. `PolyphonicDropCapo` atualmente usa o mesmo shifter granular monofônico de `PedalWhammy`; documentado como limitação conhecida em acordes complexos até implementação de Phase Vocoder / FFT overlap-add polifônico.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/pitch_shifter.rs` (`PitchShifterSuite`).

## 12. Estratégia DSP
- Buffer circular de `16384` amostras pré-alocado, 2 grãos janelados Hann com alinhamento de fase por busca de correlação cruzada (`find_aligned_delay`), leitura interpolada linear/cúbica e detector de fundamental otimizado no modo Harmonizer.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap (buffers `buf` e `analysis_buf` pré-alocados em `new()`/`prepare()`), zero locks, zero panic; custo de pico da detecção de `f0` limitado para nunca estourar o deadline do bloco.

## 14. Gestão de estado
- `buf`, `analysis_buf`, `d0`, `d1`, `phase`, `detected_freq_hz` isolados na instância; `prepare()` recalcula comprimentos de grão para `sample_rate`.

## 15. Latência
- `0` amostras quando bypassado ou `Mix = 0%`; `round(1024 * sr / 48000)` amostras (~`21,3 ms`) quando ativo, declarada via `latency_samples()`.

## 16. Consumo de CPU
- Orçamento alvo: **`<= 3%`** em `whammy`/`drop_capo`/`micro_detune` e **`<= 6%`** de pico no bloco de detecção em `harmonizer` (128 frames @ 48 kHz, `--release`).

## 17. Testes
- Senoides `220 Hz` e `440 Hz` em `+12`, `-12`, `+7`, `-2 st` validando frequência de saída `±1%`; teste diatônico `C Major` (`C4 → E4`, `D4 → F4`); verificação de `latency_samples()` ativo vs bypass; teste de pico de CPU no modo `harmonizer` em `48` e `96 kHz`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Transposição Exata):** Com `semitones = +12` e `-12` sobre senoide `220 Hz`, a frequência dominante na saída situa-se em `440 Hz ± 1%` e `110 Hz ± 1%`.
- **CA-2 (Harmonizer Diatônico):** Em `C Major`, entrada `C4 (261,63 Hz)` produz harmonia em `E4 (329,63 Hz ± 2%)` e entrada `D4 (293,66 Hz)` produz `F4 (349,23 Hz ± 2%)`.
- **CA-3 (Latência Declarada e Bypass):** `latency_samples() == 0` em bypass ou `Mix = 0%`, e `1024` amostras @ 48 kHz quando ativo; bypass entrega `max(|out - in|) == 0`.
- **CA-4 (Erro, Estabilidade e RT Safety):** Entradas `NaN`/`Inf` sanitizadas; zero alocações/desalocações em `process()` em todos os 4 modos.
- **CA-5 (Não-regressão):** Contratos da Spec 042 (`contract_042_pitch_shifter.rs` e `contract_042_pitch_api.rs`) continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Relacionadas / Legadas:** `Specs 007, 042, 052`.

## 20. Limitações conhecidas
- Modo `PolyphonicDropCapo` opera atualmente com algoritmo temporal SOLA/granular (excelente em notas únicas e power chords, com modulação residual perceptível em acordes densos de 5–6 cordas); polifonia plena exigirá estágio FFT/Phase-Vocoder em spec futura.

## 21. Riscos técnicos
- Pico de CPU em `detect_fundamental_hz()` a 96 kHz caso executado no mesmo callback que a cadeia completa; mitigar com decimação 2× ou divisão incremental da busca entre blocos.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `007`, `042`, `052`.
- **Código atual:** `crates/ampforge-dsp/src/fx/pitch.rs`.
- **Testes existentes / futuros:** `contract_042_pitch_shifter.rs`, `contract_042_pitch_api.rs`, futuro `contract_075_pedal_pitch_suite.rs`.
