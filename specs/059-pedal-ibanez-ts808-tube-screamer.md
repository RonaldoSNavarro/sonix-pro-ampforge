---
spec: 059
titulo: Pedal Ibanez TS808 Tube Screamer (Rev. 1979 JRC4558D)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053, 079]
---
# 059 — Pedal Ibanez TS808 Tube Screamer (Rev. 1979 JRC4558D)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.ibanez.ts808` (alias legado: `ts808`, `green_screamer_808`)
- **Revisão (`model_revision`):** `1979-maxon-jrc4558d-rc4558p`
- **Categoria:** Pedal de Overdrive (módulo independente)
- **Objetivo:** Especificar de forma autônoma o circuito, impedâncias, estágio de clipping em realimentação com caminho limpo unitário, filtragem de médios em ~720 Hz, controle de Tone e requisitos DSP do **Ibanez TS808 Tube Screamer**.

## 2. Escopo
- Buffer de entrada JFET/BJT (`Z_in ≈ 500 kΩ`), estágio não-inversor de ganho/clipping com op-amp `JRC4558D` e diodos de silício simétricos (`1S1588` / `1N914`) em realimentação + filtro passa-altas `4.7 kΩ + 47 nF` (~720 Hz), filtro passa-baixas pré-tone, controle ativo de `Tone` (2º op-amp), `Level` e modificação de diodos (`ClippingDiode`).

## 3. Fora de escopo
- Variantes TS9 (alteração nos resistores do buffer de saída `470 Ω / 100 kΩ` vs `100 Ω / 10 kΩ` do TS808) ou TS10/Turbo TS9DX.
- Outros pedais de drive (Specs 060–064).

## 4. Equipamento ou componente de referência
- **Fabricante:** Ibanez / Maxon (Nisshin Onpa).
- **Modelo exato:** TS-808 Tube Screamer Overdrive Pro.
- **Período / Ano:** 1979–1981.
- **Papel na cadeia de sinal:** Overdrive de pré-amplificação; comprime médios, enxuga subgraves (< 720 Hz) antes do pré do amplificador e preserva ataque limpo somado ao sinal ceifado.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1979 TS808 (JRC4558D / 1S1588 / 4.7k + 47nF / saída 100 Ω + 10 kΩ)`.
- **Variantes históricas:** TS9 (1982, saída mais brilhante por pequena variação de impedância de saída), mods de diodos assimétricos (SD-1 style) ou LED/MOSFET (suportados como override de Mod Shop via `ClippingDiode`).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | ElectroSmash — *Tube Screamer Analysis* | Ibanez TS808 (JRC4558D) | Input Buffer, Clipping Stage, Tone Stage | https://www.electrosmash.com/tube-screamer-analysis | 2026-11-19 | `[fonte secundária]` | `Z_in ≈ 510 kΩ`, `4.7kΩ + 47nF` → `f_c = 720.5 Hz`, diodos antiparalelos em realimentação, `V_out = V_in + V_clipped` |
| F2 | D. T. Yeh, J. Abel, J. O. Smith — *Simplified, Physically-Informed Models of Distortion and Overdrive Guitar Effects Pedals* (DAFx-07) | TS9 / TS808 | Seção 3 (Non-inverting diode clipper) | Proc. DAFx-07 | 2026-11-19 | `[confirmado em documentação]` | Equação diferencial da rede não-inversora com diodos em realimentação e modelo Shockley |
| F3 | NJRC `NJM4558D` / `1N4148` Datasheets | Semicondutores | Slew rate / curva V-I diodo | Datasheets fabricantes | 2026-11-19 | `[confirmado em documentação]` | `Vf ≈ 0.65–0.7 V` silício, comportamento op-amp dual |
| F4 | Specs 023 e 041 CA-5/CA-6 | Implementação anterior | `fx/drive.rs` | `specs/023-*.md`, `specs/041-*.md` | 2026-11-19 | `[confirmado em documentação]` | Contratos de caminho limpo e anti-aliasing |

## 7. Fundamentação elétrica
- **Impedâncias:** `Z_in ≈ 510 kΩ` (buffer emissor seguidor 2SC1815 — não carrega captadores passivos como um Fuzz Face); `Z_out ≈ 1–10 kΩ` (estágio bufferizado de saída) `[fonte secundária: F1]`.
- **Estágio de Clipping Não-Inversor:** op-amp `JRC4558D` com malha para terra `R = 4.7 kΩ` em série com `C = 47 nF` (`f_c = 1/(2π·4700·47e-9) ≈ 720.5 Hz`) e malha de realimentação com `R_drive = 51 kΩ + pot 500 kΩ` em paralelo com `C_fb = 51 pF` (`f_lp ≈ 6 kHz` em ganho máximo) e dois diodos de silício antiparalelos (`1S1588`) `[confirmado em documentação: F1, F2]`.
  - **Consequência elétrica fundamental:** o ganho mínimo para frequências abaixo de 720 Hz aproxima-se de **1 (0 dB)** e a parcela de sinal limpo soma-se diretamente à saída (`V_out = V_in + V_feedback_clipped`), conferindo ataque percussivo característico mesmo em drive alto.
- **Estágio de Tone:** filtro passivo passa-baixas (`1 kΩ + 0.22 µF`, `f_c ≈ 723 Hz`) seguido de rede ativa de `Tone` (`20 kΩ` W-taper + `220 Ω` + `0.22 µF` no 2º op-amp) que corta agudos abaixo de meio curso e reforça agudos acima de meio curso `[fonte secundária: F1]`.
- **Saída:** `Level` (`100 kΩ` log) + buffer de saída.

## 8. Comportamento esperado
- **Preservação do caminho limpo:** com `Drive = 0` e `Tone = 5`, uma senoide de `200 Hz` a `-30 dBFS` sai com `THD < 1%` e ganho entre `0` e `+3 dB`; mesmo com `Drive = 10`, a correlação com a entrada limpa permanece `> 0,3` (Spec 041 CA-6).
- **Mid-hump real (~720 Hz–1,5 kHz):** sinais abaixo de `400 Hz` sofrem menos clipping que a faixa `720 Hz–3 kHz`, enxugando graves antes de um amp saturado.
- **Simetria de clipping:** com diodos padrão (`SiliconSymmetrical`), predominam harmônicos ímpares (`3f0, 5f0`); ao selecionar `AsymmetricalSiLed`, surgem harmônicos pares (`2f0`) mensuráveis.
- **Interação com captadores e outros pedais:** por ser bufferizado (`Z_in ≈ 510 kΩ`), comporta-se de forma previsível após outros pedais, mas se colocado antes de um Fuzz Face Ge (Spec 063) altera a impedância de fonte vista pelo fuzz.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `drive.enabled` | boolean | bool | `false/true` | `true` | discreto | Ativa/bypassa pedal | Crossfade sem cliques `<= 5 ms` |
| Overdrive | `drive.drive` | number | scale | `0.0..=10.0` | `4.5` | log (`500 kΩ`) | Varia resistência de realimentação + ganho acima de 720 Hz | Escala ganho no ramo HPF + clipper em realimentação |
| Tone | `drive.tone` | number | scale | `0.0..=10.0` | `5.5` | W-taper | Corta/reforça agudos pós-clipping (~723 Hz–3,2 kHz) | Filtro Tone dedicado em `f64` |
| Level | `drive.level` | number | scale | `0.0..=10.0` | `6.0` | audio log (`100 kΩ`) | Atenuação/boost de saída pós-Tone | Ganho de saída suavizado (`0` = silêncio) |
| Clipping Diode (Mod) | `drive.diode` | enum | id | `silicon_sym, asym_si_led, germanium, mosfet, hard_opamp` | `silicon_sym` | discreto | Altera tensão de condução `Vf+ / Vf-` dos diodos | Parâmetros do modelo de diodo / Newton / ADAA |

## 10. Contratos de integração
- `module_id = "pedal.ibanez.ts808"`, `model_revision = "1979-maxon-jrc4558d-rc4558p"`.
- Expõe `input_impedance_ohms = 510_000` e `output_impedance_ohms = 10_000` no descritor para futura modelagem de interação front-end (Spec 076).

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/drive.rs` (linhas 384–387, 508–536).
- **Dívidas registradas:**
  1. Atualmente acoplado à struct monolítica `DriveProcessor` junto com outros 5 pedais.
  2. O estágio de `Tone` atual usa um único `high_shelf(1800 Hz)` em `f32`, simplificando o filtro LP `723 Hz` + rede ativa `20 kΩ` do TS808 real; evoluir para modelo fiel em `f64` (TDF-II) mantendo compatibilidade de ganho.
  3. O método interno chama-se `oversample_4x` apesar de iterar `8` sub-amostras (`drive.rs:473–489`); unificar com o módulo compartilhado da Spec 079.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/ts808.rs` (`IbanezTs808`).

## 12. Estratégia DSP
- **Circuit-level / Hybrid:** discretização do ramo HPF de 1ª ordem (`720,5 Hz`) + LPF `51 pF` em `f64`, solução não linear do par de diodos em realimentação (Newton-Raphson amortecido sobre Shockley ou ADAA1 calibrada em `4x`/`8x` OS) somada ao caminho limpo unitário (`V_out = V_in + V_clip`), seguida pela rede de `Tone` em `f64`.
- **Oversampling:** `4x` ou `8x` seletivo apenas no estágio de clipping (Spec 079), garantindo alias `< -55 dBFS`.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; coeficientes invariantes a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- Estados dos filtros HPF/LPF/Tone e do oversampler contidos na instância `IbanezTs808`; `reset()` limpa integralmente sem alocação.

## 15. Latência
- Latência do oversampler local (Spec 079), reportada via `latency_samples()`.

## 16. Consumo de CPU
- Orçamento alvo: **`<= 3%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Senoide 200 Hz (`Drive=0`, `-30 dBFS`) para caminho limpo (`THD < 1%`); sweep 20 Hz–20 kHz para mid-hump ~720 Hz e curva de Tone (`0/5/10`); senoide 5 kHz a `-6 dBFS` (`Drive=10`) para aliasing `< -50 dBFS` (alvo `< -60 dBFS`); comparação de harmônicos `silicon_sym` vs `asym_si_led`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Caminho Limpo Unitário):** Com `Drive=0`, `Tone=5`, senoide `200 Hz` a `-30 dBFS` tem `THD < 1%` e ganho entre `0` e `+3 dB`; com `Drive=10`, correlação com a entrada limpa permanece `> 0,3`.
- **CA-2 (Mid-Hump 720 Hz):** Com `Drive=8`, `Tone=5`, a energia em `1 kHz` relativa a `150 Hz` supera em `>= 4 dB` a resposta em `Drive=0`.
- **CA-3 (Controle de Tone):** Variar `Tone` de `0` para `10` altera a resposta em `4 kHz` em `>= 10 dB` mantendo estabilidade em `44,1 / 48 / 96 kHz`.
- **CA-4 (Troca de Diodos):** Selecionar `asym_si_led` eleva o 2º harmônico (`2f0`) em `>= 10 dB` comparado a `silicon_sym` em `Drive=7`.
- **CA-5 (Anti-Aliasing e RT Safety):** Alias residual `< -55 dBFS` (5 kHz @ `-6 dBFS`, `Drive=10`); zero alocações/desalocações em `process()`; entrada `NaN`/`Inf` retorna saída finita.
- **CA-6 (Não-regressão):** Testes das Specs 023, 032 e 041 CA-5/CA-6 para TS808 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 079` (Oversampling).
- **Relacionadas / Legadas:** `Specs 006, 023, 032, 041`.

## 20. Limitações conhecidas
- Saturação de alimentação de 9 V do op-amp JRC4558D (rail clipping extremo) não é modelada separadamente quando os diodos já limitam em `±0,7 V`.

## 21. Riscos técnicos
- Divergência de ganho de saída (`Level`) pode alterar presets calibrados com TS808 empurrando o JCM800 — manter level-matching no `Level = 5..6`.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `006`, `023`, `032`, `041`.
- **Código atual:** `crates/ampforge-dsp/src/fx/drive.rs`.
- **Testes existentes / futuros:** `contract_023_drive_808.rs`, `contract_041_dynamics_and_drives.rs`, futuro `contract_059_pedal_ts808.rs`.
