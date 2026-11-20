---
spec: 067
titulo: Pedal MXR M108 10-Band Graphic EQ (e Modos Boogie 5-Band / Paramétrico 8-Band)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053]
---
# 067 — Pedal MXR M108 10-Band Graphic EQ (e Modos de Equalização)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.mxr.m108_10band_eq` (modos adicionais: `eq.mesa.boogie_5band`, `eq.studio.parametric_8band`; chaves `eq_1.*`)
- **Revisão (`model_revision`):** `m108-18v-iso-10band`
- **Categoria:** Pedal de Equalização Gráfica / Paramétrica (módulo independente)
- **Objetivo:** Especificar os circuitos de filtros giradores/peaking, as frequências ISO exatas, a largura de banda proporcional (Q constante/proporcional estável) e os trims de entrada/saída (`Gain`/`Volume`) do **MXR M108 10-Band EQ**, bem como os contratos dos modos `Graphic5BandBoogie` (Mesa/Boogie Mark 5-Band) e `Parametric8Band`.

## 2. Escopo
- **MXR M108 (10 bandas ISO):** `31.25`, `62.5`, `125`, `250`, `500`, `1000`, `2000`, `4000`, `8000`, `16000 Hz`, curso `±12 dB` (extensível até `±15 dB` conforme contrato UI/API) + sliders independentes `Gain` (pré-EQ, `±12 dB`) e `Volume` (pós-EQ, `±12 dB`).
- **Mesa/Boogie 5-Band Graphic:** `80`, `240`, `750`, `2200`, `6600 Hz` (`±12 dB`), famoso pela curva em "V" dos amps Mark.
- **Paramétrico 8-Band:** 8 bandas com `freq_hz`, `gain_db` e `q` ajustáveis.

## 3. Fora de escopo
- Simulação de clipping de alimentação de 18 V dos op-amps do M108 (operam com altíssimo headroom linear na faixa de guitarra).
- Analisador de espectro RTA visual (telemetria visual, não altera áudio).

## 4. Equipamento ou componente de referência
- **Fabricante:** Dunlop Manufacturing / MXR.
- **Modelo exato:** M108 / M108S Ten Band Graphic EQ (18V).
- **Período / Ano:** 2003–atual (M108 / M108S).
- **Papel na cadeia de sinal:** Escultura tonal cirúrgica pré-amplificador (moldando quais frequências saturam) ou pós-amplificador/loop (moldando o espectro final na mix).

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `MXR M108 (10 bandas ISO em oitavas exatas 31.25 Hz–16 kHz, ±12 dB, Gain/Volume ±12 dB)`.
- **Modos complementares no mesmo slot `eq_1`:** `graphic_5_boogie` (Mesa Mark Series 5-band inductors/gyrators) e `eight_band` (estúdio paramétrico 8 bandas).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | *MXR M108 / M108S Ten Band EQ Manual* | M108 / M108S | Frequências centrais, faixa ±12 dB, Gain/Volume | Jim Dunlop Manuals (`jimdunlop.com`) | 2026-11-19 | `[confirmado em documentação]` | 10 frequências ISO (`31.25 Hz` a `16 kHz`), `±12 dB` por banda, sliders Gain e Volume independentes |
| F2 | *Mesa/Boogie Mark IIC+ / Mark IV Manual* | 5-Band Graphic EQ | Frequências centrais | Mesa Engineering Manuals | 2026-11-19 | `[confirmado em documentação]` | Frequências exatas `80, 240, 750, 2200, 6600 Hz` |
| F3 | R. Bristow-Johnson — *Audio EQ Cookbook* | Peaking EQ Biquad | Coeficientes e Q proporcional | W3C / musicdsp.org | 2026-11-19 | `[confirmado em documentação]` | Equações RBJ peaking em `f64` e relação Q vs largura de banda |
| F4 | Spec 032 CA-6 e Handoff 2026-11-18 (§2.2) | Contratos existentes | `fx/eq.rs` | `specs/032-*.md`, `docs/handoff/` | 2026-11-19 | `[confirmado em documentação]` | Q proporcional estável (`2.1` no MXR 10-band, `1.75` no Boogie 5-band) e troca de modo zero-alloc |

## 7. Fundamentação elétrica
- **Giradores ativos (M108 / Boogie 5-Band):** cada banda gráfica emprega um filtro RLC simulado por op-amp (girador) em arranjo de realimentação paralela/série que produz resposta peaking com **Q proporcional suave** (a largura de banda estreita ligeiramente em boosts/cuts extremos, evitando interferência destrutiva severa entre bandas adjacentes em 1 oitava) `[confirmado em documentação: F1, F3, F4]`.
- **Impedâncias:** `Z_in ≈ 470 kΩ`–`1 MΩ`, `Z_out < 1 kΩ` (bufferizado).
- **Trims Pré/Pós:** `Gain` ajusta o nível antes do banco de filtros; `Volume` compensa o ganho global após a equalização.

## 8. Comportamento esperado
- **Transparência total em flat:** com todas as bandas em `0 dB` e `Gain = Volume = 0 dB`, a saída é idêntica à entrada (`erro <= 1e-6`, bypass matemático exato).
- **Exatidão de ganho central:** elevar qualquer banda `k` em `+6,0 dB` (ou `-6,0 dB`) produz variação de `+6,0 ± 0,5 dB` (`-6,0 ± 0,5 dB`) em sua frequência central `f_k`.
- **Sem colapso entre bandas adjacentes:** elevar simultaneamente duas bandas adjacentes em `+6 dB` no MXR 10-band não produz vale destrutivo > 2 dB entre elas nem instabilidade de fase.
- **Trims `Gain` e `Volume`:** cada um altera o nível em `±12 dB` de forma exata (`±0,2 dB`).

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `eq_1.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa/bypassa EQ | Crossfade sem cliques |
| Modo EQ | `eq_1.mode` | enum | id | `mxr_10band, graphic_5_boogie, eight_band` | `eight_band` | discreto | Seleciona banco de frequências e Q | Reconfigura array fixo `[Band; 10]` zero-alloc |
| Bandas 1..10 | `eq_1.band1..10.gain_db` | number | dB | `-12.0..=15.0` | `0.0` | linear em dB | Ganho da banda `k` em sua frequência ISO | Biquad peaking RBJ em `f64` TDF-II |
| Gain (Pré) | `eq_1.gain_db` | number | dB | `-12.0..=12.0` | `0.0` | linear em dB | Trim de entrada do M108 | Ganho pré-filtros |
| Volume (Pós) | `eq_1.volume_db` | number | dB | `-12.0..=12.0` | `0.0` | linear em dB | Trim de saída do M108 | Ganho pós-filtros |

## 10. Contratos de integração
- `module_id = "pedal.mxr.m108_10band_eq"`, `model_revision = "m108-18v-iso-10band"`.
- A banda 10 (`16 kHz`) a `44,1 kHz` deve limitar `freq_hz <= 0.45 * sample_rate` (clamp seguro com pre-warping / Nyquist guard) para evitar distorção de curva próxima a Nyquist.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/eq.rs` (linhas 1–229).
- **Dívidas registradas:**
  1. `eq.rs:183–206` recalcula `sin/cos/powf` de cada banda ativa a cada bloco `process()` mesmo quando nenhum parâmetro mudou; pré-computar coeficientes em `set_*`/`prepare()` para reduzir CPU.
  2. `eq.rs:211` usa Direct Form I (`[x1, x2, y1, y2]`); migrar para primitiva compartilhada `Biquad64` em **Direct Form II Transposed (TDF-II)** para menor ruído numérico em `31,25 Hz` @ `96 kHz`.
  3. `controls.registry.ts:1107` declara `range: [-12, 15]` enquanto `eq.rs:131` faz clamp em `[-15, 15]` e o M108 real é `±12 dB`; alinhar faixa exata no descritor por modo.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/eq_mxr10.rs` (e separação opcional dos 3 modos sobre o mesmo banco `Biquad64`).

## 12. Estratégia DSP
- **Linear `f64` de precisão:** cascata de até 10 biquads peaking RBJ em `f64` TDF-II com Q proporcional (`Q_eff = Q * (1 + 0.025 * |gain_db|)` no MXR 10-band) e bypass automático de bandas com `|gain_db| < 1e-4` para economia de CPU.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap (incluindo na troca de `eq_1.mode` — array estático de 10 bandas), zero locks, zero panic; coeficientes pré-calculados invariantes a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- `filter_states: [[f64; 2]; 10]` (TDF-II) e parâmetros por banda encapsulados na instância; troca de modo zera estados sem alocar.

## 15. Latência
- **`0` amostras** (filtros IIR de fase mínima).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 1,5%`** do deadline de 128 frames @ 48 kHz com todas as 10 bandas ativas (build `--release`).

## 17. Testes
- Sweep 20 Hz–20 kHz verificando cada uma das 10 frequências ISO do MXR M108 e das 5 frequências do Boogie em `+6/-6 dB`; verificação de transparência flat (`erro <= 1e-6`); validação de `Gain` e `Volume` (`±12 dB`); estabilidade em `31,25 Hz` e `16 kHz` a `44,1`, `48` e `96 kHz`; troca de modo zero-alloc.

## 18. Critérios de aceite
- **CA-1 (Frequências ISO Exatas):** Em modo `mxr_10band`, aplicar `+6,0 dB` em cada banda `1..10` produz `+6,0 ± 0,5 dB` em sua respectiva frequência central (`31.25` a `16000 Hz`).
- **CA-2 (Modo Boogie 5-Band):** Em modo `graphic_5_boogie`, as 5 bandas atuam em `80, 240, 750, 2200, 6600 Hz` com `±0,5 dB` de precisão.
- **CA-3 (Trims Gain e Volume):** `eq_1.gain_db = +6.0` e `eq_1.volume_db = -6.0` produzem ganho líquido `0,0 ± 0,2 dB`; cada trim isolado produz seu ganho nominal em `±0,2 dB`.
- **CA-4 (Transparência Flat e Estabilidade 44,1–96 kHz):** Com todas as bandas em `0 dB`, `max(|out - in|) <= 1e-6`; nenhuma banda diverge ou produz `NaN` em `44,1 / 48 / 96 kHz` (inclusive `16 kHz` a `44,1 kHz`).
- **CA-5 (Zero-Alloc na Troca de Modo e RT Safety):** Trocar `eq_1.mode` e processar blocos registra `0` alocações e `0` desalocações.
- **CA-6 (Não-regressão):** Testes das Specs 006, 024 e 032 CA-6 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Relacionadas / Legadas:** `Specs 006, 024, 032`.

## 20. Limitações conhecidas
- Proximidade de Nyquist na banda de `16 kHz` em `44,1 kHz` produz compressão natural da largura de banda pela transformada bilinear (mitigada por clamp seguro de `w0`; Vicanek/orfanidis peaking opcional em melhoria futura).

## 21. Riscos técnicos
- Recalcular coeficientes com mudanças rápidas de slider sem suavização pode produzir leve zipper noise em ganhos altos (`±12 dB`); aplicar suavização suave de ganho por banda quando alterada.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `006`, `024`, `032`.
- **Código atual:** `crates/ampforge-dsp/src/fx/eq.rs`.
- **Testes existentes / futuros:** `contract_032_pedals_fx_cabs.rs`, futuro `contract_067_pedal_mxr10_eq.rs`.
