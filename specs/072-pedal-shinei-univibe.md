---
spec: 072
titulo: Pedal Shin-Ei Uni-Vibe (Rev. 1968 — 4 Estágios Escalonados e Fotocélula Assimétrica)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053]
---
# 072 — Pedal Shin-Ei Uni-Vibe (Rev. 1968)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.shinei.univibe` (`ModModel::PhotocellUnivibe`, chaves `mod_2.*`)
- **Revisão (`model_revision`):** `1968-shinei-4cell-ldr`
- **Categoria:** Pedal de Modulação — Vibe / Fotocélula (módulo independente)
- **Objetivo:** Especificar o comportamento elétrico distinto do **Shin-Ei Uni-Vibe (1968)** — **4 estágios all-pass com frequências de corte escalonadas (não idênticas)** acionados por **lâmpada incandescente + 4 fotocélulas CdS (LDR)** com curva LFO fortemente **assimétrica** ("throb / heartbeat") e modos `Chorus` (dry+wet) vs `Vibrato` (100% wet).

## 2. Escopo
- 4 estágios de deslocamento de fase com capacitores escalonados (razões distintas entre si, ex.: `1 : 1.41 : 2.0 : 2.83` ou valores do esquema original `15 nF / 0.22 µF / 470 pF / 4.7 nF` — ver §7), LFO assimétrico modelando inércia térmica da lâmpada + resposta não linear de resistência da fotocélula CdS, modulação de amplitude sutil associada e modos `Chorus` (`Mix = 50%`) / `Vibrato` (`Mix = 100%`).

## 3. Fora de escopo
- MXR Phase 90 de 4 estágios idênticos (Spec 071) e Leslie Rotary Speaker (Spec 074).

## 4. Equipamento ou componente de referência
- **Fabricante:** Shin-Ei Companion / Univox.
- **Modelo exato:** Uni-Vibe (projeto Fumio Mieda).
- **Período / Ano:** 1968–1973.
- **Papel na cadeia de sinal:** Posicionado tipicamente antes do amplificador (ou após Fuzz Face), gerando pulso rítmico assimétrico característico deHendrix / Trower.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1968 Shin-Ei Uni-Vibe (4 LDRs CdS + lâmpada + estágios escalonados + Chorus/Vibrato)`.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | R. G. Keen — *The Technology of the Uni-Vibe* | Shin-Ei Uni-Vibe (1968) | Phase Shift Stages, Lamp/LDR Asymmetry, Staggered Caps | http://www.geofex.com/article_folders/univibe/uvtech.htm | 2026-11-19 | `[fonte secundária]` | 4 estágios all-pass escalonados (não idênticos), resposta exponencial/assimétrica da lâmpada + CdS LDR, modos Chorus/Vibrato |
| F2 | Spec 040 | Contratos existentes | `fx/modulation.rs` | `specs/040-*.md` | 2026-11-19 | `[confirmado em documentação]` | LFO não senoidal de fotocélula e 4 estágios all-pass escalonados |

## 7. Fundamentação elétrica
- **Estágios escalonados (diferença fundamental frente ao Phase 90):** enquanto o Phase 90 usa 4 capacitores idênticos gerando notches simétricos, o Uni-Vibe emprega 4 capacitores diferentes nos 4 estágios de deslocamento de fase (transistores `2SC536` / `2SC828`), espalhando os polos de fase e criando entalhes assimétricos mais largos e musicais `[fonte secundária: F1]`.
- **Dinâmica Lâmpada + Fotocélula CdS:** a lâmpada incandescente aquece mais rápido do que esfria e a resistência do LDR varia não linearmente com a luminosidade (`R(L) ∝ L^-γ`), produzindo uma onda de varredura **assimétrica** (permanecendo mais tempo no extremo grave/escuro e pulsando abruptamente para o agudo — o famoso *throb*) `[fonte secundária: F1]`.

## 8. Comportamento esperado
- **Assimetria temporal do pulso:** a trajetória da frequência dos entalhes ao longo de um ciclo LFO é mensuravelmente assimétrica (tempo acima da mediana ≠ tempo abaixo da mediana, diferenciando-se do Phase 90 senoidal/triangular).
- **Entalhes escalonados:** com LFO congelado, a largura e posição relativa dos entalhes diferem mensuravelmente do Phase 90 (estágios não idênticos).
- **Modo Chorus vs Vibrato:** `Mix = 50%` gera cancelamentos de fase profundos (Chorus); `Mix = 100%` gera modulação de pitch/fase sem cancelamento dry (Vibrato).

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `mod_2.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa/bypassa pedal | Crossfade sem cliques |
| Speed (Rate) | `mod_2.rate_hz` | number | Hz | `0.05..=10.0` | `1.2` | log | Frequência do oscilador da lâmpada | Fase LFO assimétrica |
| Intensity (Depth) | `mod_2.depth` | number | scale | `0.0..=10.0` | `7.5` | linear | Amplitude de brilho da lâmpada | Excursão dos 4 all-pass + AM sutil |
| Mix (Chorus/Vibrato) | `mod_2.mix` | number | % | `0.0..=100.0` | `50.0` | linear | `50%` = Chorus, `100%` = Vibrato | `dry*(1-m) + wet*m` |

## 10. Contratos de integração
- `module_id = "pedal.shinei.univibe"`, `model_revision = "1968-shinei-4cell-ldr"`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/modulation.rs` (linhas 241–250).
- **Dívidas registradas:** acoplado em `ModulationProcessor`; all-pass em `f32` (migrar para `f64` TDF-II/TPT); modelar a inércia térmica de subida/descida distinta da lâmpada (filtro de ataque/release assimétrico sobre o LFO) além da lei de potência `powf(2.2)`.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/shinei_univibe.rs` (`ShinEiUniVibe`).

## 12. Estratégia DSP
- **Hybrid:** gerador LFO com inércia térmica assimétrica (ataque/queda diferentes + curva LDR CdS) modulando 4 estágios all-pass de 1ª ordem em `f64` com frequências escalonadas (`[1.0, 1.41, 2.0, 2.83]` ou razões calibradas do esquema Shin-Ei) + feedback sutil e modulação AM leve (`<= 12%`).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; coeficientes invariantes a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- 4 estados all-pass `f64`, estado térmico da lâmpada e `lfo_phase` isolados na instância.

## 15. Latência
- **`0` amostras**.

## 16. Consumo de CPU
- Orçamento alvo: **`<= 1,5%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Medição de assimetria do ciclo LFO (duty/skew ≠ 50%); comparação espectral com `MxrPhase90` sob mesma taxa/depth provando assinatura distinta; modos `Mix = 50%` vs `100%`; invariância de taxa; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (LFO Assimétrico de Fotocélula):** Ao longo de um ciclo completo do LFO, a forma de onda de modulação apresenta assimetria mensurável (valor médio diferente do ponto médio pico-a-pico em `>= 10%`).
- **CA-2 (Distinção Real frente ao Phase 90):** Sob mesma senoide/ruído e mesmos parâmetros (`rate`, `depth`, `mix = 50%`), a resposta espectral do Uni-Vibe difere da resposta do Phase 90 em `>= 3 dB` RMS na banda `200 Hz–4 kHz`.
- **CA-3 (Invariância de Taxa e RT Safety):** Estável e consistente em `44,1 / 48 / 96 kHz`; zero alocações/desalocações em `process()`.
- **CA-4 (Não-regressão):** Contratos da Spec 040 para `PhotocellUnivibe` continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Relacionadas / Legadas:** `Specs 007, 040, 052`.

## 20. Limitações conhecidas
- O estágio pré-amplificador discreto de entrada do Uni-Vibe original (leve coloração warm mesmo em bypass não-true-bypass) não é replicado quando o bloco está em bypass.

## 21. Riscos técnicos
- Estabilidade dos 4 all-pass escalonados em taxas extremas se `base * 2.83` aproximar-se de Nyquist; aplicar clamp seguro `<= 0.45 * sr`.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `007`, `040`, `052`.
- **Código atual:** `crates/ampforge-dsp/src/fx/modulation.rs`.
- **Testes existentes / futuros:** `contract_040_mod_delay_reverb.rs`, futuro `contract_072_pedal_shinei_univibe.rs`.
