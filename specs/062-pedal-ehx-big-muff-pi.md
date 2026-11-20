---
spec: 062
titulo: Pedal Electro-Harmonix Big Muff Pi (Rev. V3 / NYC 4 Estágios)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053, 079]
---
# 062 — Pedal Electro-Harmonix Big Muff Pi (Rev. V3 / NYC 4 Estágios)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.ehx.big_muff_pi` (alias legado: `muff`, `big_muff_pi`)
- **Revisão (`model_revision`):** `1977-v3-nyc-2n5088`
- **Categoria:** Pedal de Fuzz / Sustainer (módulo independente)
- **Objetivo:** Especificar integralmente a topologia de **4 estágios transistorizados BJT em cascata (Booster → Clipper 1 → Clipper 2 → Tone Stack passivo com notch profundo de médios → Recuperação)** do **Electro-Harmonix Big Muff Pi**.

## 2. Escopo
- Estágio 1 (Booster emissor comum BJT `2N5088` + controle `Sustain`), Estágio 2 (1º clipper BJT + diodos `1N914` em realimentação), Estágio 3 (2º clipper BJT + diodos `1N914` em realimentação), Tone Stack passivo clássico (`39 kΩ / 10 nF` LP + `22 kΩ / 4 nF` HP + pot `100 kΩ` gerando notch pronunciado de médios), Estágio 4 (recuperação de ganho BJT) e `Volume`.

## 3. Fora de escopo
- Versão Op-Amp Big Muff (1978, circuito sem BJTs nos estágios de clipping) e variantes Russian/Triangle/Ram's Head com valores RC distintos (registradas em §5 como variantes futuras).

## 4. Equipamento ou componente de referência
- **Fabricante:** Electro-Harmonix.
- **Modelo exato:** Big Muff Pi (Version 3 / clássico NYC 4-transistor).
- **Período / Ano:** 1977–1978 (base das reedições NYC clássicas).
- **Papel na cadeia de sinal:** Fuzz/sustainer de altíssimo ganho e compressão "violining", caracterizado por dois clippers em cascata e recorte profundo de médios (~800–1000 Hz) no meio do curso de `Tone`.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `V3 / NYC (4× 2N5088, 1N914, Tone Stack 39k/10nF + 22k/4nF)`.
- **Variantes históricas (não misturadas):**
  - *V1 Triangle (1969)* e *V2 Ram's Head (1973):* variações nos capacitores de acoplamento e no notch do tone stack.
  - *Green Russian (Sovtek 1994):* corte de graves mais baixo, notch de médios deslocado para baixo (~450–600 Hz).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | ElectroSmash — *Big Muff Pi Analysis* | EHX Big Muff Pi (4× BJT) | 4 Stages, Diode Feedback Clipping, Passive Tone Stack | https://www.electrosmash.com/big-muff-pi-analysis | 2026-11-19 | `[fonte secundária]` | 4 estágios BJT (`2N5088`), 2 estágios com `1N914` em realimentação, Tone Stack `39k/10nF` + `22k/4nF` com notch de médios |
| F2 | Kit Rae — *Big Muff Pi Schematics & History* | V3 / NYC | Valores de componentes por revisão | http://www.kitrae.net/music/big_muff_history.html | 2026-11-19 | `[fonte secundária]` | Diferenciação de revisões V1, V2, V3, Russian |
| F3 | Spec 032 CA-3 e Spec 041 CA-5 | Contratos existentes | `fx/drive.rs` | `specs/032-*.md`, `specs/041-*.md` | 2026-11-19 | `[confirmado em documentação]` | Notch de médios `>= 4 dB` em `800–1200 Hz` com `Tone = 5.0` e anti-aliasing |

## 7. Fundamentação elétrica
- **Impedâncias:** `Z_in ≈ 100–130 kΩ` (menor que 1 MΩ padrão, apresentando leve carga sobre captadores passivos); `Z_out ≈ 10–25 kΩ` após estágio de recuperação e pot `Volume` `[fonte secundária: F1]`.
- **Estágio 1 (Input Booster):** emissor comum BJT (~+20 dB de ganho) seguido do potenciômetro `Sustain` (`100 kΩ` log) `[fonte secundária: F1]`.
- **Estágio 2 e Estágio 3 (Clippers gêmeos em cascata):** cada estágio possui um BJT `2N5088` com capacitores de acoplamento (`100 nF`) e um par de diodos de silício `1N914` em antiparalelo na malha de realimentação coletor-base em paralelo com `470 pF` (limitando agudos extremos entre estágios) `[fonte secundária: F1]`. A cascata de dois clippers produz onda fortemente comprimida, quase quadrada em `Sustain` alto, gerando sustain prolongado.
- **Tone Stack Passivo Big Muff:** rede em ponte com ramo passa-baixas (`R1 = 39 kΩ`, `C1 = 10 nF`, `f_lp ≈ 408 Hz`) e ramo passa-altas (`R2 = 22 kΩ`, `C2 = 4 nF`, `f_hp ≈ 1808 Hz`) combinados pelo potenciômetro `Tone` (`100 kΩ` linear) `[fonte secundária: F1]`. A perda de inserção (~−12 dB) e a separação entre `f_lp` e `f_hp` produzem um **vale profundo (notch) em torno de 800–1000 Hz** em `Tone = 5.0`.
- **Estágio 4 (Output Recovery):** BJT emissor comum recuperando ~+15 dB de perda do Tone Stack, seguido por `Volume` (`100 kΩ` log).

## 8. Comportamento esperado
- **Notch de médios característico:** com `Tone = 5.0`, a resposta em frequência apresenta vale de médios em `800–1200 Hz` **`>= 6 dB`** (contrato mínimo `>= 4 dB`) abaixo das extremidades (`200 Hz` e `3,5 kHz`).
- **Sustain e compressão extrema:** com `Sustain >= 7`, a saída aproxima-se rapidamente do patamar de saturação mesmo para entradas de `-24 dBFS`, reduzindo o crest factor para `< 5 dB`.
- **Controle de Tone amplo (tilt LP↔HP):** `Tone = 0` entrega som grave/aveludado (`-10 dB` acima de `2 kHz`); `Tone = 10` corta subgraves e projeta agudos rasgados.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `drive.enabled` | boolean | bool | `false/true` | `true` | discreto | Ativa/bypassa pedal | Crossfade sem cliques `<= 5 ms` |
| Sustain | `drive.drive` | number | scale | `0.0..=10.0` | `6.5` | audio log (`100 kΩ`) | Nível após Estágio 1 excitando os 2 clippers em cascata | Ganho entre Estágio 1 e Estágios 2–3 |
| Tone | `drive.tone` | number | scale | `0.0..=10.0` | `5.0` | linear (`100 kΩ`) | Mistura ramo LP (`~408 Hz`) e HP (`~1808 Hz`) | Rede nodal passiva Big Muff em `f64` |
| Volume | `drive.level` | number | scale | `0.0..=10.0` | `6.0` | audio log (`100 kΩ`) | Atenuação após Estágio 4 de recuperação | Ganho de saída suavizado |

## 10. Contratos de integração
- `module_id = "pedal.ehx.big_muff_pi"`, `model_revision = "1977-v3-nyc-2n5088"`.
- Expõe `input_impedance_ohms = 120_000`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/drive.rs` (linhas 436–439, 601–624).
- **Dívidas registradas:**
  1. Atualmente acoplado em `DriveProcessor` e modela os 3 estágios em um loop genérico `for st in 0..3` sem os filtros inter-estágio `470 pF` nem o Estágio 4 de recuperação com saturação sutil.
  2. O Tone Stack atual soma dois biquads 2ª ordem independentes (`360 Hz` LP + `2200 Hz` HP em `f32`), em vez da função de transferência nodal exata da rede RC `39k/10nF + 22k/4nF + 100k` em `f64` (2ª ordem passiva acoplada).
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/big_muff_pi.rs` (`EhxBigMuffPi`).

## 12. Estratégia DSP
- **Circuit-level:** função de transferência nodal exata do Tone Stack Big Muff (`39k/10nF/22k/4nF/100k`) discretizada por bilinear em `f64`.
- **Hybrid / Behavioral calibrado:** Estágio 1 (HPF entrada + booster BJT) → Estágio 2 (clipper feedback + LP inter-estágio `470 pF`) → Estágio 3 (clipper feedback + LP inter-estágio `470 pF`) dentro de `8x` oversampling (Spec 079) → Tone Stack `f64` → Estágio 4 de recuperação suave.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; invariante a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- Estados dos filtros inter-estágio, Tone Stack `f64` e oversampler isolados em `EhxBigMuffPi`.

## 15. Latência
- Latência do oversampler local (Spec 079).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 4%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Sweep 20 Hz–20 kHz para validar curva nodal do Tone Stack (`Tone = 0, 5, 10`) dentro de `±0,5 dB` da referência RC; medição de compressão/crest factor em `Sustain = 2` vs `9`; senoide `5 kHz` a `-6 dBFS` para aliasing `< -55 dBFS`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Tone Stack Nodal Big Muff):** Com `Tone = 5.0`, a resposta em frequência apresenta notch em `800–1200 Hz` `>= 4 dB` abaixo de `200 Hz` e `3,5 kHz`, coincidindo dentro de `±0,5 dB` com a função de transferência RC (`39k/10nF/22k/4nF/100k`) em `Tone = 0`, `5` e `10`.
- **CA-2 (Cascata de Clippers e Sustain):** Em `Sustain = 9`, a saída de uma senoide 440 Hz a `-24 dBFS` já atinge saturação forte (`THD > 25%`) e crest factor `< 6 dB`.
- **CA-3 (Anti-Aliasing em Fuzz Extremo):** Com `Sustain = 10` e senoide `5 kHz` a `-6 dBFS`, aliasing residual `< -55 dBFS`.
- **CA-4 (Invariância de Taxa e RT Safety):** Resposta linear do Tone Stack coincide em `±0,5 dB` entre `44,1`, `48` e `96 kHz`; zero alocações/desalocações em `process()`; entradas `NaN`/`Inf` sanitizadas.
- **CA-5 (Não-regressão):** Testes das Specs 032 CA-3 e 041 CA-5 para `BigMuffPi` continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 079` (Oversampling).
- **Relacionadas / Legadas:** `Specs 032, 041`.

## 20. Limitações conhecidas
- Leve carga de impedância (`~120 kΩ`) sobre captadores passivos depende da integração futura com o modelo de entrada/captador (Spec 076).

## 21. Riscos técnicos
- Dois clippers em cascata geram harmônicos muito elevados; qualquer subdimensionamento do filtro anti-aliasing produzirá aliasing audível em notas agudas.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `032`, `041`.
- **Código atual:** `crates/ampforge-dsp/src/fx/drive.rs`.
- **Testes existentes / futuros:** `contract_032_pedals_fx_cabs.rs`, `contract_041_dynamics_and_drives.rs`, futuro `contract_062_pedal_big_muff_pi.rs`.
