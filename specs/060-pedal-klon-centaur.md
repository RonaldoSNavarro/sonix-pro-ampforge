---
spec: 060
titulo: Pedal Klon Centaur (Rev. 1994–2000 Gold/Silver Horsie)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053, 079]
---
# 060 — Pedal Klon Centaur (Rev. 1994–2000)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.klon.centaur` (alias legado: `klon`, `klon_centaur`, `golden_klone_centaur`)
- **Revisão (`model_revision`):** `1995-gold-18v-1n34a`
- **Categoria:** Pedal de Overdrive / Clean Boost Transparente (módulo independente)
- **Objetivo:** Especificar o circuito elétrico real, a mecânica do potenciômetro duplo de **Gain (clean blend + hard clipping germânio)**, o headroom de **charge pump +18 V / −9 V**, o shelving ativo de **Treble** e a modelagem DSP independente do **Klon Centaur**.

## 2. Escopo
- Buffer de entrada TL072 (`Z_in ≈ 1 MΩ`), divisor de caminhos limpo e saturado comandados simultaneamente pelo potenciômetro dual-gang `Gain`, pré-ênfase de médios (~1 kHz) no ramo sujo, hard clipping por diodos de germânio (`1N34A`) para terra, somador de caminhos limpo+sujo com headroom estendido (rails `+18 V / -9 V` = `27 Vpp`), controle ativo de `Treble` (~2,4 kHz) e `Output`.

## 3. Fora de escopo
- Clones com chaves extras de bass boost ou clipping LED não presentes no Centaur original (overrides de diodo só via Mod Shop).
- Klon KTR (chave buffer/true-bypass externa).

## 4. Equipamento ou componente de referência
- **Fabricante:** Klon LLC (Bill Finnegan).
- **Modelo exato:** Centaur Professional Overdrive.
- **Período / Ano:** 1994–2000.
- **Papel na cadeia de sinal:** Clean boost / overdrive de altíssimo headroom que mistura progressivamente o sinal limpo encorpado com clipping de germânio focado em ~1 kHz.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `Gold/Silver Centaur (TL072 + MAX1044/ICL7660 charge pump +18V/-9V + 1N34A Ge)`.
- **Variantes:** versões Gold vs Silver têm circuito essencialmente equivalente (pequena diferença de resistor em série na saída em unidades tardias, impacto desprezível em áudio).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | ElectroSmash — *Klon Centaur Analysis* | Klon Centaur (TL072 / 1N34A) | Charge Pump, Dual-Gang Gain, Summing Amp, Treble | https://www.electrosmash.com/klon-centaur-analysis | 2026-11-19 | `[fonte secundária]` | Charge pump `+18V / -9V` (`27 Vpp`), dual-gang `100kB`, diodos `1N34A`, summing amp, Treble ativo |
| F2 | Datasheets `TL072`, `MAX1044`/`ICL7660S`, `1N34A` | Semicondutores | Tensão de operação e joelho Ge | Datasheets fabricantes | 2026-11-19 | `[confirmado em documentação]` | `Vf ≈ 0.30–0.35 V` germânio suave, alto headroom de TL072 em ±rails estendidos |
| F3 | Spec 032 CA-1/CA-3 e Spec 041 CA-5 | Contratos existentes | `fx/drive.rs` | `specs/032-*.md`, `specs/041-*.md` | 2026-11-19 | `[confirmado em documentação]` | Clean blend em `Gain = 2.0` (`correlação > 0.7`) e anti-aliasing |

## 7. Fundamentação elétrica
- **Alimentação Charge Pump:** converte `9 VDC` em rails internos de `+18 V` e `-9 V` (`27 V` totais), evitando clipping prematuro dos op-amps `TL072` no estágio somador e no `Treble` `[fonte secundária: F1]`.
- **Buffer de entrada:** `Z_in ≈ 1 MΩ`, mantém sinal limpo íntegro e alimenta em paralelo o ramo limpo e o ramo de ganho `[fonte secundária: F1]`.
- **Potenciômetro Dual-Gang `Gain` (100 kΩ duplo):**
  - Seção A atenua progressivamente o ramo limpo à medida que `Gain` sobe (em `Gain = 0` o ramo limpo domina quase 100%; em `Gain = 10` cai para contribuição mínima).
  - Seção B aumenta o ganho do ramo sujo (com filtro passa-banda/médios em `~1,0–1,1 kHz`) que excita dois diodos de germânio `1N34A` em antiparalelo para terra (hard clipping suave de joelho baixo) `[fonte secundária: F1]`.
- **Somador + Treble Ativo:** op-amp somador combina ramo limpo + ramo saturado + shelf ativo de `Treble` centrado em `~2,4 kHz` (`±7 dB` aprox.), seguido por `Output` (`10 kΩ` log).

## 8. Comportamento esperado
- **Clean Boost verdadeiro em `Gain` baixo (`0–2`):** correlação com entrada `> 0,7`, `THD < 2%` em nível nominal `-12 dBFS`, resposta quase plana com leve projeção em ~1 kHz.
- **Transição contínua de textura (`Gain 0 → 10`):** à medida que `Gain` sobe, o ganho do ramo saturado cresce enquanto o peso relativo do sinal limpo recua de forma monotonicamente acoplada (sem salto brusco).
- **Alto headroom de saída:** `Output` permite empurrar amplificadores valvulados com até `+12 dB` limpos sem distorção dura no estágio final do pedal.
- **Treble ativo sem perda de graves:** girar `Treble` de `0` para `10` atua acima de `~1,5–2,4 kHz` preservando a região `< 300 Hz` praticamente intacta (`Δ < 1,5 dB`).

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `drive.enabled` | boolean | bool | `false/true` | `true` | discreto | Ativa/bypassa pedal | Crossfade sem cliques `<= 5 ms` |
| Gain | `drive.drive` | number | scale | `0.0..=10.0` | `4.5` | linear (dual-gang B) | Aumenta ganho do ramo Ge e reduz blend limpo simultaneamente | Controle acoplado `clean_blend(g)` + `dirty_gain(g)` |
| Treble | `drive.tone` | number | scale | `0.0..=10.0` | `5.5` | linear | Shelf ativo pós-soma (~2,4 kHz) | High-shelf `f64` TDF-II |
| Output | `drive.level` | number | scale | `0.0..=10.0` | `6.0` | audio log | Volume de saída de alto headroom | Ganho suave até `+12 dB` |

## 10. Contratos de integração
- `module_id = "pedal.klon.centaur"`, `model_revision = "1995-gold-18v-1n34a"`.
- Expõe `input_impedance_ohms = 1_000_000` (bufferizado).

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/drive.rs` (linhas 389–391, 539–564).
- **Dívidas registradas:** acoplado em `DriveProcessor`; filtros `klon_mid_biquad` e `klon_treble_biquad` em `f32` (migrar para `f64` TDF-II); modelar mais fielmente o filtro passa-altas/banda real do ramo de clipping (~400 Hz–1 kHz) separado do ramo limpo (hoje o `klon_mid_biquad` só atua no ramo sujo, o que já está correto estruturalmente, mas pode ser refinado com valores RC do esquema).
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/klon_centaur.rs` (`KlonCentaur`).

## 12. Estratégia DSP
- **Hybrid:** dois ramos paralelos (ramo limpo linear + ramo sujo com filtro de banda em `f64` e clipping germânio com oversampling `4x`/`8x` da Spec 079 + ADAA1/Shockley suave), somados com pesos determinados pela lei dual-gang do potenciômetro `Gain`, seguidos por `Treble` shelf `f64` e headroom estendido (`±2.0` interno antes do limiter global).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; coeficientes invariantes a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- Estados de filtros e oversampler encapsulados na instância `KlonCentaur`; `reset()` limpa em O(1).

## 15. Latência
- Latência do oversampler do ramo sujo (alinhada ao ramo limpo por linha de atraso igual à latência do oversampler para evitar cancelamento de fase comb na soma limpo+sujo!).
  - **Nota técnica crítica:** em `drive.rs:553–560`, o ramo `dirty` passa por `oversample_4x` enquanto `x * clean_blend` não é compensado em atraso — na implementação modular da Spec 060, **compensar o atraso do ramo limpo** para eliminar filtro comb na soma!

## 16. Consumo de CPU
- Orçamento alvo: **`<= 3%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Correlação limpo/saída em `Gain=2.0`; variação monotônica de `clean_blend` e `THD` de `Gain=0` a `10`; resposta plana `< 300 Hz` sob varredura de `Treble 0→10`; alinhamento de fase limpo+sujo (sem comb em `Gain=2`); aliasing em `5 kHz`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Dual-Gang Clean Blend):** Com `Gain = 2.0`, `Tone = 5.0`, senoide/DI de entrada apresenta correlação `> 0,7` com a entrada e `THD < 5%`; com `Gain = 9.0`, `THD` cresce `>= 4×`.
- **CA-2 (Alinhamento de Fase Limpo/Sujo):** Em `Gain = 2.0` com sweep `20 Hz–15 kHz`, a soma limpo+sujo não apresenta cancelamentos comb periódicos `> 3 dB` acima de `2 kHz` (atraso do oversampler compensado no ramo limpo).
- **CA-3 (Treble Ativo):** Variar `Tone` `0→10` altera `4 kHz` em `>= 10 dB` mantendo `150 Hz` dentro de `±1,5 dB`.
- **CA-4 (Anti-Aliasing e RT Safety):** Alias residual `< -55 dBFS` (5 kHz @ `-6 dBFS`, `Gain = 10`); zero alocações/desalocações em `process()`; entradas `NaN`/`Inf` sanitizadas.
- **CA-5 (Não-regressão):** Testes das Specs 032 e 041 para `GoldenKloneCentaur` continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 079` (Oversampling).
- **Relacionadas / Legadas:** `Specs 032, 041`.

## 20. Limitações conhecidas
- Variações individuais de diodos NOS `1N34A` (dispersão de `Vf` entre unidades vintage) são representadas por valores nominais.

## 21. Riscos técnicos
- Desalinhamento temporal entre ramo limpo e ramo oversampled gera comb filtering em frequências altas se não compensado (endereçado pelo CA-2).

## 22. Referências e rastreabilidade
- **Specs anteriores:** `032`, `041`.
- **Código atual:** `crates/ampforge-dsp/src/fx/drive.rs`.
- **Testes existentes / futuros:** `contract_032_pedals_fx_cabs.rs`, `contract_041_dynamics_and_drives.rs`, futuro `contract_060_pedal_klon_centaur.rs`.
