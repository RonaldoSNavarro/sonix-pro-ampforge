---
spec: 070
titulo: Pedal Through-Zero Flanger (Atraso Curto Modulado com Realimentação)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053]
---
# 070 — Pedal Through-Zero Flanger

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.mod.through_zero_flanger` (`ModModel::ThroughZeroFlanger`, chaves `mod_2.*`)
- **Revisão (`model_revision`):** `v1-tzf-short-delay-comb`
- **Categoria:** Pedal de Modulação — Flanger (módulo independente)
- **Objetivo:** Especificar o mecanismo físico e DSP do **Through-Zero Flanger** — linha de atraso ultracurta modulada (`0,3–3,3 ms`) com realimentação positiva/negativa (`feedback`) e cruzamento real pelo ponto zero (atraso fixo equivalente no ramo seco permitindo cancelamento completo *through-zero*).

## 2. Escopo
- Linha de atraso modulada de `0,3` a `3,3 ms` (centro `1,8 ms`), realimentação configurável (`-0.95..=+0.95`) com limitação suave para evitar instabilidade, opção de alinhamento de atraso fixo no ramo seco (`1,8 ms`) para verdadeiro cancelamento *through-zero* (efeito "jato" de duas máquinas de fita).

## 3. Fora de escopo
- Chorus BBD (Spec 069) e Phaser all-pass (Spec 071).

## 4. Equipamento ou componente de referência
- **Referência histórica:** Flanging clássico de duas máquinas de fita sincronizadas (anos 1960) e pedais TZF modernos (ex.: Foxrox Paradox TZF / Strymon Deco).
- **Papel na cadeia de sinal:** Varredura harmônica em filtro pente (*comb filter*) com picos ressonantes afiados controlados por `feedback`.

## 5. Revisão e variantes
- **Revisão de referência:** `v1-tzf-short-delay-comb`.
- **Variantes:** feedback positivo (harmônicos pares/ímpares ressonantes) vs feedback negativo (ênfase oca/metálica em harmônicos ímpares).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | U. Zölzer — *DAFX* (2ª ed.) | Cap. 2 (Delay-Based Audio Effects) | Flanger & Comb Filters | Wiley, 2011 | 2026-11-19 | `[confirmado em documentação]` | Equação do filtro pente modulado, espaçamento de picos `f_k = k / τ`, efeito de realimentação |
| F2 | Spec 040 CA-3 | Contratos existentes | `fx/modulation.rs` | `specs/040-*.md` | 2026-11-19 | `[confirmado em documentação]` | Diferença pico-vale `>= 10 dB` com feedback 70% e queda `>= 6 dB` com feedback 0% |

## 7. Fundamentação elétrica
- Somar um sinal com sua versão atrasada em `τ(t)` (`0,3..3,3 ms`) gera um **filtro pente** cujos entalhes ocorrem em `f_n = (2n + 1) / (2τ)`.
- Realimentar a saída da linha de atraso na entrada (`g_fb ∈ [-0.95, +0.95]`) cria picos ressonantes acentuados nos múltiplos de `1/τ`.
- Quando o ramo seco recebe um atraso fixo `τ_ref = 1,8 ms` e o ramo modulado cruza `τ(t) = τ_ref` (com inversão opcional), ocorre o **cancelamento through-zero** momentâneo em todas as frequências `[confirmado em documentação: F1]`.

## 8. Comportamento esperado
- **Picos comb mensuráveis com realimentação:** sobre ruído branco com LFO congelado e `feedback = 0.70`, a diferença entre picos e vales espectrais é **`>= 10 dB`**; com `feedback = 0.0`, essa diferença cai em **`>= 6 dB`** (Spec 040 CA-3).
- **Estabilidade com feedback extremo:** mesmo em `feedback = ±0.95`, a saída permanece limitada e sem divergência ou `NaN`.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito / algoritmo | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `mod_2.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa/bypassa pedal | Crossfade sem cliques |
| Rate | `mod_2.rate_hz` | number | Hz | `0.05..=10.0` | `0.4` | log | Velocidade de varredura | Incremento LFO |
| Depth | `mod_2.depth` | number | scale | `0.0..=10.0` | `6.0` | linear | Excursão em torno de `1,8 ms` (`±1,5 ms`) | Modulação de `d_samples` |
| Feedback (Regen) | `mod_2.feedback` | number | ganho | `-0.95..=0.95` | `0.70` | linear | Ressonância dos picos do filtro pente | Realimentação com soft clip seguro |
| Mix | `mod_2.mix` | number | % | `0.0..=100.0` | `50.0` | linear | Profundidade dos cancelamentos comb | `dry*(1-m) + wet*m` |

## 10. Contratos de integração
- `module_id = "pedal.mod.through_zero_flanger"`, `model_revision = "v1-tzf-short-delay-comb"`.
- Expor `mod_2.feedback` no `controls.registry.ts` e `api.rs` (hoje existe em `ModulationProcessor::set_feedback`, mas ainda não está exposto em `controls.registry.ts` — dívida registrada!).

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/modulation.rs` (linhas 219–231).
- **Dívidas registradas:**
  1. Atraso atual não aplica o atraso fixo de referência no ramo seco para o verdadeiro cruzamento *through-zero* (`τ_wet` nunca passa para o outro lado de `τ_dry = 0`); adicionar atraso fixo compensado (ou reportar latência `1,8 ms` em modo TZF).
  2. `mod_2.feedback` ainda não está registrado em `controls.registry.ts`.
  3. Migrar leitura fracionária para interpolação cúbica/Hermite (reduz atenuação HF no comb).
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/tz_flanger.rs` (`ThroughZeroFlanger`).

## 12. Estratégia DSP
- Linha de atraso circular pré-alocada com interpolação cúbica (Hermite), realimentação saturada suavemente (`tanh` leve em `±4.0` para proteção sem distorcer sinal nominal), LFO contínuo `f64` e atraso de referência para cruzamento through-zero.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; tempos em `ms` invariantes a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- `delay_buffer`, `write_pos`, `flanger_fb_state`, `lfo_phase` encapsulados em `ThroughZeroFlanger`.

## 15. Latência
- `0` amostras no modo clássico; no modo TZF real com atraso de referência `1,8 ms`, declara exatamente `(0.0018 * sr).round()` amostras via `latency_samples()`.

## 16. Consumo de CPU
- Orçamento alvo: **`<= 1,5%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Ruído branco com LFO congelado (`feedback = 0.70` vs `0.0`) validando picos comb (`>= 10 dB`, variação `>= 6 dB`); teste de cruzamento through-zero (cancelamento profundo no cruzamento); estabilidade em `feedback = ±0.95`; invariância de taxa; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Ressonância Comb por Feedback):** Com ruído branco e LFO congelado em `feedback = 0.70`, a diferença pico-vale espectral é `>= 10 dB`; com `feedback = 0.0`, reduz em `>= 6 dB`.
- **CA-2 (Estabilidade em Feedback Máximo):** Em `feedback = +0.95` e `-0.95` com sinal a `-6 dBFS` por 3 s, a saída permanece finita e sem divergência.
- **CA-3 (Invariância de Taxa e RT Safety):** Posição dos entalhes comb coincide em `±2%` entre `44,1`, `48` e `96 kHz`; zero alocações/desalocações em `process()`.
- **CA-4 (Não-regressão):** Teste `CA-3` da Spec 040 continua verde.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Relacionadas / Legadas:** `Specs 007, 040, 052`.

## 20. Limitações conhecidas
- Feedback positivo/negativo em `mod_2.feedback` requer exposição explícita no painel skeuomórfico do pedal (Spec 052).

## 21. Riscos técnicos
- Interpolação linear em atrasos ultracurtos com feedback alto causa amortecimento excessivo de agudos; mitigado por interpolação cúbica.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `007`, `040`, `052`.
- **Código atual:** `crates/ampforge-dsp/src/fx/modulation.rs`.
- **Testes existentes / futuros:** `contract_040_mod_delay_reverb.rs`, futuro `contract_070_pedal_tz_flanger.rs`.
