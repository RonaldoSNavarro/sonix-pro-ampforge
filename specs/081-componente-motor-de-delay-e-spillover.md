---
spec: 081
titulo: Componente Compartilhado de Motor de Delay (BBD, Digital, Dual, Reverse, Ping-Pong), Ducking e Spillover
status: rascunho
tipo: componente-compartilhado
atualizado: 2026-11-19
dependencias: [053]
---
# 081 — Componente de Motor de Delay, Ducking e Spillover

## 1. Identificação e objetivo
- **Identificador estável (`module_id`):** `component.fx.delay_engine`
- **Revisão (`model_revision`):** `v1-fractional-delay-duck-spillover`
- **Categoria:** Componente compartilhado DSP (Linha de atraso fracionário, Ducking, Spillover e algoritmos de Delay)
- **Objetivo:** Especificar as primitivas compartilhadas de atraso fracionário (`FractionalDelayLine`, `clear_tail()` O(1) sem `memset`, glide suavizado sem estalo, ducking dinâmico suave e spillover gapless) e os 5 modelos adicionais de delay (`AnalogBbdDelay`, `PristineDigitalDelay`, `DualParallelDelay`, `ReverseDelay`, `PingPongStereo`) que complementam a unidade dedicada Roland RE-201 (Spec 068).

## 2. Escopo
- Primitiva reutilizável `FractionalDelayLine` (capacidade até `4000 ms @ 96 kHz = 384.000` amostras pré-alocada, leitura interpolada linear/cúbica e invalidação de cauda em `O(1)` via `samples_since_clear`).
- Algoritmos:
  1. **Analog BBD Delay (`AnalogBbdDelay`):** roll-off passa-baixas progressivo (`~2,4 kHz`) + soft clipping `tanh` dentro do laço de feedback (cada repetição escurece).
  2. **Pristine Digital Delay (`PristineDigitalDelay`):** repetições lineares planas (`±0,1 dB` em `5 kHz` na 1ª repetição).
  3. **Dual Parallel Delay (`DualParallelDelay`):** dois taps simultâneos (`0,75×` colcheia pontuada + `1,0×` semínima).
  4. **Reverse Delay (`ReverseDelay`):** captura de grão de duração `delay_time_ms` e leitura invertida no tempo (rampa crescente → rampa decrescente).
  5. **Ping-Pong Stereo (`PingPongStereo`):** contrato estéreo L↔R (enquanto a cadeia permanecer mono, permanece explicitamente sinalizado como pendente de saída estéreo na UI conforme Spec 040 CA-5/Fora de escopo, sem mentir que é estéreo).
- **Ducking dinâmico suave:** reduz o nível do wet enquanto a guitarra está tocando, com variação máxima de ganho `<= 0,5 dB/amostra` (sem estalos em degrau — Spec 040 CA-8).
- **Spillover:** preserva caudas ao bypassar ou trocar cenas (instância única fora do par ativa/sombra — Spec 049 CA-1).

## 3. Fora de escopo
- Roland RE-201 Space Echo de 12 modos / 3 cabeças + mola interna (especificado independentemente na Spec 068, que consome as primitivas desta spec).

## 4. Equipamento ou componente de referência
- **Referências:** Boss DM-2 / EHX Memory Man (BBD), TC Electronic 2290 / Boss DD-3 (Digital Pristine), Dual/Reverse studio delays.

## 5. Revisão e variantes
- **Revisão:** `v1-fractional-delay-duck-spillover`.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | U. Zölzer — *DAFX* (2ª ed.) | Cap. 2 | Fractional Delay Lines, All-pass/Lagrange Interpolation, Feedback Delays | Wiley, 2011 | 2026-11-19 | `[confirmado em documentação]` | Linhas de atraso fracionário e estabilidade de feedback |
| F2 | Spec 040 (`CA-5`, `CA-6`, `CA-7`, `CA-8`, `CA-12`) | Contratos existentes | `fx/delay.rs` | `specs/040-*.md` | 2026-11-19 | `[confirmado em documentação]` | Diferença Digital vs BBD em 5 kHz, Reverse invertido, glide sem degrau, ducking `<= 0,5 dB/amostra`, spillover |

## 7. Fundamentação elétrica / algorítmica
- **BBD vs Digital:** no BBD, a filtragem anti-aliasing de entrada/saída do chip (`MN3005/MN3205`) fica dentro da malha de realimentação, causando atenuação cumulativa de agudos a cada repetição (`>= 3 dB` em `5 kHz` já na 1ª repetição); no Digital, a banda passante da repetição é plana (`±0,1 dB`).
- **Clear O(1):** zerar `samples_since_clear = 0` invalida instantaneamente leituras mais antigas que o contador sem executar `memset` de ~1,5–3 MB dentro do callback RT.

## 8. Comportamento esperado
- **Digital vs BBD verificável:** para senoide `5 kHz`, a 1ª repetição do Digital coincide com `entrada × feedback` em `±0,1 dB`, enquanto no BBD fica `>= 3 dB` abaixo (Spec 040 CA-5).
- **Reverse Delay verdadeiro:** alimentado com rampa crescente de duração `500 ms`, produz trecho decrescente no grão seguinte (Spec 040 CA-6).
- **Ducking suave:** variação amostra-a-amostra de `duck_mult` nunca excede `0,5 dB` (Spec 040 CA-8).
- **Glide sem cliques:** alterar `delay_time_ms` de `300` para `400 ms` mantém derivada máxima `<= 1,5×` (Spec 040 CA-7).

## 9. Parâmetros e controles
| Nome | Chave / Campo | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Model | `delay.model` | enum | id | `bbd, digital, dual, reverse, ping_pong` | `digital` | discreto | Seleciona algoritmo de delay | Polimorfismo por módulo |
| Delay Time | `delay_time_ms` | number | ms | `1.0..=4000.0` | `350.0` | log | Tempo de atraso (ou sincronizado a BPM) | Glide exponencial `tau = 30 ms` |
| Feedback | `feedback_pct` | number | % | `0.0..=150.0` | `35.0` | linear | Realimentação das repetições | Clamp/tanh seguro |
| Mix | `mix_pct` | number | % | `0.0..=100.0` | `40.0` | linear | Proporção dry/wet | `dry*(1-m) + wet*m` |
| Ducking | `ducking_{enabled,threshold_db,attenuation_db,release_ms}` | misto | dB/ms | `-60..0`, `-30..0`, `10..1000` | `off` | linear | Abaixa wet durante sinal forte | Envoltória suavizada `<= 0,5 dB/amostra` |
| Spillover | `spillover` | boolean | bool | `false/true` | `true` | discreto | Mantém caudas ao bypassar | Zera escrita mantendo leitura |

## 10. Contratos de integração
- Primitiva `FractionalDelayLine` compartilhada com `Spec 068` (RE-201) e `Spec 053` (PDC do grafo).
- `PingPongStereo` só é habilitado na UI quando o `ChannelLayout` da saída for `Stereo` (Spec 053).

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/delay.rs` (linhas 1–559).
- **Dívidas registradas:**
  1. `ReverseDelay` lê sem crossfade nas bordas do grão (`delay.rs:518–536`), podendo gerar leve clique na emenda de `grain_len` — adicionar janela de crossfade de 5 ms nas extremidades do grão!
  2. Separar `RolandRe201SpaceEcho` (Spec 068) das primitivas e demais modelos de delay (`shared/delay_line.rs` + `pedals/delay_*.rs`).
  3. `DelayProcessor::new` aloca dois buffers gigantes (`buffer` + `rev_buffer` = ~3 MB) mesmo quando `ReverseDelay` não está selecionado — alocar apenas o buffer necessário por módulo independente reduz memória em 50%.
- **Estrutura alvo:** `crates/ampforge-dsp/src/shared/delay_line.rs` e módulos dedicados em `pedals/`.

## 12. Estratégia DSP
- Buffer circular pré-alocado em `prepare(spec)`, interpolação cúbica/linear, `samples_since_clear` O(1), smoothing de `delay_samples` e `duck_mult`, e janela de crossfade nas bordas do Reverse.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap e zero `memset` em `process()` e `clear_tail()`; zero locks, zero panic; tempos em `ms` invariantes a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- Buffer circular, `write_pos`, `samples_since_clear`, `current_delay_samples`, `ducking_envelope`, `duck_mult`, `bbd_fb_lp` encapsulados por instância.

## 15. Latência
- **`0` amostras** no caminho seco (`dry`).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 1,5%`** por instância @ 128 frames / 48 kHz em build `--release`.

## 17. Testes
- Senoide `5 kHz` comparando Digital (`±0,1 dB`) vs BBD (`>= 3 dB` abaixo); rampa crescente no Reverse verificando trecho decrescente sem clique de borda; mudança `300 → 400 ms` (`derivada <= 1,5×`); ducking (`Δganho <= 0,5 dB/amostra`); `clear_tail()` O(1) zero-alloc; spillover em bypass.

## 18. Critérios de aceite
- **CA-1 (Digital vs BBD):** A 1ª repetição de senoide `5 kHz` no Digital fica dentro de `±0,1 dB` de `entrada × feedback`; no BBD fica `>= 3 dB` abaixo.
- **CA-2 (Reverse Delay Sem Clique de Borda):** Alimentado com rampa crescente de `500 ms`, produz trecho decrescente com transição janelada suave nas emendas.
- **CA-3 (Glide e Ducking Suaves):** Mudar tempo `300→400 ms` mantém derivada `<= 1,5×`; no ducking, variação amostra-a-amostra nunca excede `0,5 dB`.
- **CA-4 (Clear O(1) e RT Safety):** `clear_tail()` e `process()` executam em tempo O(1) por amostra com `0` alocações e `0` desalocações; entradas `NaN`/`Inf` sanitizadas.
- **CA-5 (Não-regressão):** Testes das Specs 007, 024, 040 (`CA-5..CA-8`, `CA-12`) e 049 CA-1 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Dependentes:** `Spec 068` (RE-201).
- **Relacionadas / Legadas:** `Specs 007, 024, 032, 040, 049`.

## 20. Limitações conhecidas
- `PingPongStereo` opera em dual-tap mono até a ativação completa do barramento estéreo L/R (Spec 053).

## 21. Riscos técnicos
- Feedback acima de `100%` (`100..150%`) em `PristineDigitalDelay` exige limitação suave segura para evitar travamento em `±2.0` quadrado indefinidamente.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `007`, `024`, `032`, `040`, `049`.
- **Código atual:** `crates/ampforge-dsp/src/fx/delay.rs`.
- **Testes existentes / futuros:** `contract_007_mod_pitch_delay_reverb.rs`, `contract_040_mod_delay_reverb.rs`, futuro `contract_081_delay_engine.rs`.
