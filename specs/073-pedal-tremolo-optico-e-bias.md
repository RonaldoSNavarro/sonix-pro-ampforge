---
spec: 073
titulo: Pedal Tremolo Óptico / Bias (Modulação Pura de Amplitude sem Delay)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053]
---
# 073 — Pedal Tremolo Óptico / Bias

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.mod.optical_bias_tremolo` (`ModModel::OpticalBiasTremolo`, chaves `mod_2.*`)
- **Revisão (`model_revision`):** `v1-opto-am-zero-delay`
- **Categoria:** Pedal de Modulação — Tremolo (módulo independente)
- **Objetivo:** Especificar o **Tremolo Óptico / Bias** como modulador puro de amplitude (AM) **sem nenhuma linha de atraso**, com envoltória suave e zero artefatos de deslocamento de fase ou bandas laterais de delay.

## 2. Escopo
- Modulação de amplitude pura `y[n] = x[n] * g_lfo(t)` com LFO contínuo (`0,05–10 Hz`), profundidade `0–100%`, sincronização opcional a BPM e forma de onda suave (senoidal/opto-LDR).

## 3. Fora de escopo
- Vibrato de pitch (modulação de atraso — Spec 069 em `Mix = 100%`) e Tremolo harmônico bifásico (divisão graves/agudos em contrafase — candidato futuro).

## 4. Equipamento ou componente de referência
- **Referência histórica:** Circuito opto-tremolo clássico (Fender AB763 LDR / Demeter TRM-1 Tremulator).
- **Papel na cadeia de sinal:** Pós-amplificador/pré ou final de cadeia para pulsação rítmica limpa de volume.

## 5. Revisão e variantes
- **Revisão de referência:** `v1-opto-am-zero-delay`.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | Spec 040 CA-2 | Contratos existentes | `fx/modulation.rs` | `specs/040-*.md` | 2026-11-19 | `[confirmado em documentação]` | Envelope modulado em `f_rate ± 0,1 Hz` e ausência de raias de delay (`< -40 dB` fora de `f0 ± 10 Hz`) |
| F2 | R. G. Keen / Fender AB763 Opto-Isolator | Opto-Tremolo | Comportamento AM puro | Esquemas Fender / Demeter | 2026-11-19 | `[fonte secundária]` | Atenuação resistiva pura sem linha de atraso |

## 7. Fundamentação elétrica
- Em um tremolo óptico, um foto-resistor (LDR) acoplado a um LED/lâmpada forma um divisor resistivo variável para terra, modulando apenas a amplitude do sinal (`AM`) sem introduzir atraso de grupo nem modulação de fase `[fonte secundária: F2]`.

## 8. Comportamento esperado
- **AM puro sem artefatos de delay:** alimentado com senoide `1 kHz`, `Depth = 10`, `Rate = 4 Hz`, o envelope de saída pulsa exatamente a `4,0 ± 0,1 Hz`, e toda energia fora de `1 kHz ± 10 Hz` permanece **`< -40 dB`** abaixo do pico (Spec 040 CA-2).
- **Latência zero e transparência em `Depth = 0`:** saída igual à entrada (`erro <= 1e-6`).

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `mod_2.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa/bypassa pedal | Crossfade sem cliques |
| Rate | `mod_2.rate_hz` | number | Hz | `0.05..=10.0` | `4.0` | log | Velocidade do LFO | `2π·f/fs` |
| Depth | `mod_2.depth` | number | scale | `0.0..=10.0` | `6.0` | linear | Profundidade da atenuação AM (`0..100%`) | Escala de modulação `g(t)` |
| Mix | `mod_2.mix` | number | % | `0.0..=100.0` | `100.0` | linear | Atenuação efetiva do tremolo | `dry*(1-m) + wet*m` |

## 10. Contratos de integração
- `module_id = "pedal.mod.optical_bias_tremolo"`, `model_revision = "v1-opto-am-zero-delay"`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/modulation.rs` (linhas 214–218).
- **Dívidas registradas:** acoplado em `ModulationProcessor` (que aloca `delay_buffer` de 8192 floats mesmo quando apenas Tremolo é usado!). Separar `OpticalBiasTremolo` elimina memória ociosa.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/optical_tremolo.rs` (`OpticalBiasTremolo`).

## 12. Estratégia DSP
- Oscilador LFO em `f64` modulando ganho instantâneo `g = 1 - depth_norm * 0.5 * (1 - sin(phase))` (suavizado para zero zipper noise), sem buffers de atraso.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; frequência LFO exata em `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- Apenas `lfo_phase: f64` e smoothers de ganho/depth (pegada de memória mínima `< 128 bytes`).

## 15. Latência
- **`0` amostras**.

## 16. Consumo de CPU
- Orçamento alvo: **`<= 0,3%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Senoide `1 kHz` com `Depth = 10, Rate = 4 Hz`: verificação da frequência de modulação do envelope (`4,0 ± 0,1 Hz`) e energia fora de `1 kHz ± 10 Hz < -40 dB`; transparência em `Depth = 0`; invariância de taxa; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (AM Puro Sem Delay):** Senoide `1 kHz`, `Depth = 10`, `Rate = 4 Hz`, `Mix = 100%` produz envelope modulado em `4,0 ± 0,1 Hz` com energia fora de `1 kHz ± 10 Hz < -40 dB` do pico.
- **CA-2 (Transparência e Latência Zero):** Em `Depth = 0`, `max(|out - in|) <= 1e-6`; `latency_samples() == 0`.
- **CA-3 (RT Safety e Taxas):** Zero alocações/desalocações em `process()`; taxa exata em `44,1 / 48 / 96 kHz`.
- **CA-4 (Não-regressão):** Teste `CA-2` da Spec 040 continua verde.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Relacionadas / Legadas:** `Specs 007, 040, 052`.

## 20. Limitações conhecidas
- Forma de onda única suave na revisão v1; formas adicionais (square/saw/harmonic) ficam para revisão futura.

## 21. Riscos técnicos
- Mudanças abruptas de `Depth` sem suavização podem gerar clique; usar `OnePoleSmoother` em `depth`.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `007`, `040`, `052`.
- **Código atual:** `crates/ampforge-dsp/src/fx/modulation.rs`.
- **Testes existentes / futuros:** `contract_040_mod_delay_reverb.rs`, futuro `contract_073_pedal_tremolo.rs`.
