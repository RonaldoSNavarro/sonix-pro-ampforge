---
spec: 064
titulo: Pedal Boss DS-1 Distortion (Rev. 1978–1994 TA7136AP / BA728N)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053, 079]
---
# 064 — Pedal Boss DS-1 Distortion (Rev. Clássica)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.boss.ds1` (alias legado: `ds1`, `boss_ds1`, `orange_boss_ds`)
- **Revisão (`model_revision`):** `1980-miJ-ta7136ap-1n4148`
- **Categoria:** Pedal de Distortion (módulo independente)
- **Objetivo:** Especificar integralmente o circuito do **Boss DS-1 Distortion** — buffer de entrada, transistor booster pré-ganho com filtro passa-altas, estágio de alto ganho por op-amp com hard clipping para terra e **Tone Stack passivo tilt/V-curve (mid-scoop)** — como módulo DSP autônomo.

## 2. Escopo
- Buffer JFET/BJT de entrada (`Z_in ≈ 470 kΩ`), transistor pré-booster BJT com pré-ênfase HPF, estágio de ganho por op-amp (`TA7136AP` / `M5223AL`, ganho ajustável via `Dist` até ~+60 dB), hard clipping por par de diodos de silício (`1N4148` / `1S2473`) para terra, rede tonal passiva tilt/V-curve inspirada no Big Muff (scoop em médios ~500–800 Hz) e buffer de saída com `Level`.

## 3. Fora de escopo
- Modificações pós-fábrica (Keeley Seeing Eye / Ultra Mod com LEDs; modificação de diodos só via Mod Shop).
- Boss DS-2 Turbo Distortion (modo Turbo com realce de médios).

## 4. Equipamento ou componente de referência
- **Fabricante:** Roland / Boss.
- **Modelo exato:** DS-1 Distortion.
- **Período / Ano:** 1978–1994 (circuito clássico MIJ/MIT com `TA7136AP` / `BA728N` / `M5223AL`).
- **Papel na cadeia de sinal:** Distorção de alto ganho e ataque cortante com recorte de médios (V-curve) projetada historicamente para trabalhar bem com amplificadores já levemente saturados (ex.: Marshall Plexi/JCM800).

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `Boss DS-1 Clássico (pre-2016 thru-hole, 1N4148 hard clip, Tone Stack passivo V-curve)`.
- **Variantes históricas:** mudança do op-amp `TA7136AP` → `BA728N` → `M5223AL` → `NJM3404AL` e migração SMD (2016+), mantendo a mesma topologia de pré-booster + op-amp hard clip + Tone Stack passivo V-curve.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | ElectroSmash — *Boss DS-1 Analysis* | Boss DS-1 | Input Buffer, Transistor Booster, Op-Amp Gain + Hard Clip, Tone Control V-Curve | https://www.electrosmash.com/boss-ds1-analysis | 2026-11-19 | `[fonte secundária]` | `Z_in ≈ 470 kΩ`, transistor booster pré-ganho, hard clipping de diodos para terra, tone stack passivo tipo tilt com scoop de médios (V-curve) |
| F2 | Boss DS-1 Service Notes / Schematic | DS-1 | Diagrama de blocos e valores RC do Tone Stack | Service Manuals Roland/Boss | 2026-11-19 | `[fonte secundária]` | Rede tonal passiva LP/HP (`6.8 kΩ / 2.2 kΩ / 22 nF / 100 nF / pot 20 kΩ`) e hard clip `1S2473/1N4148` |
| F3 | Spec 032 CA-1 e Spec 041 CA-5 | Contratos existentes | `fx/drive.rs` | `specs/032-*.md`, `specs/041-*.md` | 2026-11-19 | `[confirmado em documentação]` | Assinatura harmônica distinta e anti-aliasing |

## 7. Fundamentação elétrica
- **Impedâncias:** `Z_in ≈ 470 kΩ` (buffer BJT `2SC2240`); `Z_out ≈ 1–10 kΩ` (buffer de saída) `[fonte secundária: F1]`.
- **Transistor Booster Pré-Ganho:** amplifica o sinal e aplica corte de graves antes do op-amp (`~200–240 Hz`), prevenindo saturação turva nas notas mais graves `[fonte secundária: F1]`.
- **Estágio Op-Amp + Hard Clipping:** o potenciômetro `Dist` (`100 kΩ` log) ajusta o ganho não-inversor do op-amp (de moderado a muito alto, ~+35 a +60 dB). Na saída do op-amp, um resistor série alimenta dois diodos de silício antiparalelos para terra (`1N4148`/`1S2473`), gerando hard clipping incisivo com leve assimetria prática devido à polarização/carregamento `[fonte secundária: F1, F2]`.
- **Tone Stack Passivo V-Curve:** rede passiva LP + HP controlada pelo pot `Tone` (`20 kΩ` linear):
  - Em `Tone = 5.0` (meio-dia), produz a clássica **curva em V (mid-scoop em ~500–800 Hz)** com graves e agudos elevados em relação aos médios `[fonte secundária: F1]`.
  - Em `Tone < 4`, realça graves e corta agudos (ponto tradicionalmente usado com amplificadores brilhantes); em `Tone > 6`, corta graves e eleva agudos acima de `2 kHz`.

## 8. Comportamento esperado
- **Curva em V verificável:** com `Tone = 5.0`, o espectro apresenta atenuação de médios em `500–900 Hz` `>= 3,5 dB` relativamente a `180 Hz` e `3,5 kHz`.
- **Hard Clipping de alto ganho:** mesmo em `Dist = 4.0`, produz clipping incisivo rico em harmônicos ímpares e pares leves (distinto do soft clipping em realimentação do TS808 e do slew lento do RAT).
- **Varredura de `Tone` sem instabilidade:** `Tone = 0` escurece fortemente acima de `2 kHz`, enquanto `Tone = 10` projeta agudos cortantes.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `drive.enabled` | boolean | bool | `false/true` | `true` | discreto | Ativa/bypassa pedal | Crossfade sem cliques `<= 5 ms` |
| Dist | `drive.drive` | number | scale | `0.0..=10.0` | `5.5` | audio log (`100 kΩ`) | Ganho do op-amp antes do hard clipping | Ganho pré-clipper após HPF |
| Tone | `drive.tone` | number | scale | `0.0..=10.0` | `4.5` | linear (`20 kΩ`) | Tilt passivo LP/HP com V-curve central | Rede nodal passiva DS-1 em `f64` |
| Level | `drive.level` | number | scale | `0.0..=10.0` | `6.0` | audio log (`100 kΩ`) | Volume de saída pós-Tone | Ganho de saída suavizado |

## 10. Contratos de integração
- `module_id = "pedal.boss.ds1"`, `model_revision = "1980-mij-ta7136ap-1n4148"`.
- Expõe `input_impedance_ohms = 470_000`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/drive.rs` (linhas 445–448, 665–688).
- **Dívidas registradas:** acoplado em `DriveProcessor`; o Tone atual usa um `peaking(720 Hz, -4.5 dB)` fixo + `high_shelf(2100 Hz)` em `f32`, que não captura a interação real do tilt passivo LP/HP em `Tone = 0` e `Tone = 10`. Migrar para função de transferência nodal da rede passiva DS-1 em `f64`.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/boss_ds1.rs` (`BossDs1`).

## 12. Estratégia DSP
- **Circuit-level:** rede tonal passiva tilt/V-curve do DS-1 discretizada por bilinear em `f64`.
- **Hybrid:** HPF pré-ganho (`~240 Hz`, `f64`) → ganho + hard clipping algébrico/Shockley levemente assimétrico em `8x` oversampling (Spec 079) → Tone Stack passivo `f64` → `Level`.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; invariante a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- Estados dos filtros e oversampler isolados em `BossDs1`.

## 15. Latência
- Latência do oversampler local (Spec 079).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 3%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Sweep 20 Hz–20 kHz para validar V-curve em `Tone = 5.0` e tilt em `Tone = 0 / 10`; medição de THD e assinatura harmônica distinta de TS808/RAT/Muff; senoide `5 kHz` a `-6 dBFS` para aliasing `< -55 dBFS`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Tone Stack V-Curve):** Com `Tone = 5.0`, a magnitude em `500–900 Hz` fica `>= 3,5 dB` abaixo da média de `180 Hz` e `3,5 kHz`; variar `Tone` `0→10` inverte a inclinação grave/agudo em `>= 12 dB`.
- **CA-2 (Assinatura Hard Clipping):** Com `Dist = 5.0` e senoide `1 kHz` a `-12 dBFS`, produz THD superior ao TS808 e assinatura espectral distinta do RAT (sem a queda HF por slew-rate do LM308).
- **CA-3 (Anti-Aliasing e RT Safety):** Alias residual `< -55 dBFS` (5 kHz @ `-6 dBFS`, `Dist = 10`); zero alocações/desalocações em `process()`; entradas `NaN`/`Inf` sanitizadas.
- **CA-4 (Invariância de Taxa e Não-regressão):** Resposta do Tone Stack coincide em `±0,5 dB` entre `44,1`, `48` e `96 kHz`; testes das Specs 032 e 041 para `OrangeBossDs` continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 079` (Oversampling).
- **Relacionadas / Legadas:** `Specs 032, 041`.

## 20. Limitações conhecidas
- Saturação do estágio transistorizado de entrada (antes do op-amp) só ocorre em sinais extremamente quentes (> +6 dBu) e é aproximada por headroom linear limpo no modelo base.

## 21. Riscos técnicos
- Ganho alto (até 43×+) exige anti-aliasing rigoroso (`8x` OS) para evitar aliasing metálico acima de `3 kHz`.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `032`, `041`.
- **Código atual:** `crates/ampforge-dsp/src/fx/drive.rs`.
- **Testes existentes / futuros:** `contract_032_pedals_fx_cabs.rs`, `contract_041_dynamics_and_drives.rs`, futuro `contract_064_pedal_boss_ds1.rs`.
