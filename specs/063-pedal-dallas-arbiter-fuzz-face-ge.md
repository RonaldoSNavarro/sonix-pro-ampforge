---
spec: 063
titulo: Pedal Dallas Arbiter Fuzz Face Germanium (Rev. 1966–1968 AC128 / NKT275)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053, 076, 079]
---
# 063 — Pedal Dallas Arbiter Fuzz Face Germanium (Rev. 1966–1968)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.arbiter.fuzz_face_ge` (alias legado: `fuzzface`, `fuzz_face`, `fuzz_face_sige`)
- **Revisão (`model_revision`):** `1967-pnp-germanium-ac128`
- **Categoria:** Pedal de Fuzz (módulo independente)
- **Objetivo:** Especificar integralmente o circuito minimalista de **2 transistores PNP de germânio acoplados diretamente com realimentação global**, baixa impedância de entrada (`Z_in ≈ 5–10 kΩ`), limpeza orgânica pelo volume da guitarra e controle de **Bias Trim** do **Dallas Arbiter Fuzz Face Germanium**.

## 2. Escopo
- Topologia clássica de 2 transistores PNP (`Q1` e `Q2`, ex.: `AC128` / `NKT275`, ganhos `h_FE ≈ 70–80` em Q1 e `110–130` em Q2), capacitor de entrada `2.2 µF`, resistor de realimentação global `100 kΩ` do emissor de Q2 à base de Q1, potenciômetro `Fuzz` (`1 kΩ` log) variando bypass do emissor de Q2 (`20 µF`), `Volume` (`500 kΩ` log) e `Bias` (mod externa sobre coletor de Q2, nominal `~4.5 V`).
- Contrato explícito de interação com impedância de captador (Spec 076).

## 3. Fora de escopo
- Fuzz Face Silicon (`BC108C` de 1969+, ganho muito mais alto e clipping mais brilhante/agressivo — poderá virar variante ou spec futura separada).
- Simulação térmica ambiental da deriva de fuga (`I_CBO`) por temperatura em graus Celsius (exposta indiretamente via controle `Bias`).

## 4. Equipamento ou componente de referência
- **Fabricante:** Dallas Arbiter.
- **Modelo exato:** Fuzz Face (PNP Germanium).
- **Período / Ano:** 1966–1968.
- **Papel na cadeia de sinal:** Primeiro bloco ideal imediatamente após a guitarra (antes de buffers), reagindo fortemente ao volume da guitarra e à impedância do captador passivo.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1967 Arbiter Fuzz Face PNP Germanium (AC128/NKT275, C_in=2.2µF, C_out=10nF, R_fb=100kΩ, R_c2=8.2kΩ+470Ω)`.
- **Variantes históricas:** Silicon BC108 (1969), mod "Bias / Voltage Starve" (presente no controle `drive.bias` do AMPFORGE).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | ElectroSmash — *Fuzz Face Analysis* | Dallas Arbiter Fuzz Face Ge (AC128) | Input/Output Impedance, Q1/Q2 Biasing, Feedback, Cleanup | https://www.electrosmash.com/fuzz-face | 2026-11-19 | `[fonte secundária]` | `Z_in` baixo (~5–10 kΩ), `h_FE` ideal Q1≈70–80 / Q2≈110–130, realimentação `100 kΩ`, `4.5 V` coletor Q2, interação direta com captador |
| F2 | R. G. Keen — *The Technology of the Fuzz Face* | Fuzz Face Ge/Si | Análise nodal e polarização | http://www.geofex.com/article_folders/fuzzface/fffram.htm | 2026-11-19 | `[fonte secundária]` | Deslocamento de ponto de operação, efeito do `Bias` e gating assimétrico |
| F3 | Spec 032 CA-1 e Spec 041 (Fora de escopo / CA-5) | Contratos existentes | `fx/drive.rs` | `specs/032-*.md`, `specs/041-*.md` | 2026-11-19 | `[confirmado em documentação]` | Assimetria Ge, Bias Trim e nota sobre interação de impedância |

## 7. Fundamentação elétrica
- **Baixa impedância de entrada (`Z_in ≈ 5–10 kΩ`):** a base de `Q1` recebe o sinal via `2.2 µF` sem buffer prévio, carregando diretamente a indutância/resistência do captador passivo e amortecendo seu pico de ressonância `[fonte secundária: F1]`. Quando um buffer (ex.: pedal bufferizado) precede o Fuzz Face, essa carga desaparece e o timbre fica mais estridente e comprimido.
- **Topologia Q1–Q2 com realimentação global:**
  - `Q1` (emissor aterrado, `R_c1 = 33 kΩ`) acoplado diretamente à base de `Q2` (`R_c2 = 8.2 kΩ + 470 Ω` divisor de saída).
  - Resistor `100 kΩ` realimenta o emissor de `Q2` à base de `Q1`, estabilizando o ponto quiescente (~`4.5 V` no coletor de `Q2`) e acoplando a dinâmica entre os dois transistores `[fonte secundária: F1, F2]`.
  - Potenciômetro `Fuzz` (`1 kΩ`) controla o desvio AC (`20 µF`) do emissor de `Q2` para terra: em mínimo, máxima realimentação local (som quase limpo); em máximo, ganho total e saturação fortemente assimétrica de germânio.
- **Assimetria e Bias Trim:** deslocar o ponto de polarização de `Q2` fora do "ponto doce" (~4,5 V, `Bias = 5.0`) empurra a forma de onda contra corte ou saturação unilateral, gerando forte 2º harmônico e efeito *gated / velcro* nas caudas das notas `[fonte secundária: F2]`.

## 8. Comportamento esperado
- **Limpeza dinâmica com nível de entrada ("Guitar Volume Cleanup"):** com `Fuzz = 8.5` e `Bias = 5.0`, reduzir a amplitude de entrada de `-12 dBFS` para `-30 dBFS` reduz drasticamente a `THD` (sinal limpa de forma orgânica sem perder brilho excessivo), diferenciando-o do Big Muff Pi.
- **Assimetria harmônica de Germânio:** em `Bias = 5.0`, a saída possui 2º harmônico (`H2`) forte por assimetria de corte/saturação de Q2.
- **Efeito do `Bias`:** em `Bias = 5.0`, sem gating perceptível; em extremos (`Bias < 2.5` ou `> 8.0`), introduz deslocamento assimétrico acentuado e gating *velcro* mensurável em sinais fracos.
- **Interação de impedância:** quando precedido por entrada passiva modelada (Spec 076, `source_impedance = passive_pickup`), aplica amortecimento suave da ressonância de agudos; quando precedido por buffer (`low_z`), exibe mais brilho e ganho inicial.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `drive.enabled` | boolean | bool | `false/true` | `true` | discreto | Ativa/bypassa pedal | Crossfade sem cliques `<= 5 ms` |
| Fuzz | `drive.drive` | number | scale | `0.0..=10.0` | `7.5` | reverse-log (`1 kΩ`) | Varia bypass de emissor de Q2 (`20 µF`) | Ganho + profundidade de realimentação Q1-Q2 |
| Volume | `drive.level` | number | scale | `0.0..=10.0` | `6.0` | audio log (`500 kΩ`) | Atenuação de saída pós-Q2 | Ganho de saída suavizado |
| Bias (Trim) | `drive.bias` | number | scale | `0.0..=10.0` | `5.0` | linear (~2 V–7 V Q2) | Polarização do coletor de Q2 (`5.0` = 4,5 V doce; extremos = velcro/gated) | Deslocamento DC interno + limiar de corte/gating + bloqueio DC na saída |
| Tone (LPF suave) | `drive.tone` | number | scale | `0.0..=10.0` | `5.0` | log | Ajuste opcional de roll-off HF (não existe no original 1967; ver §11) | LPF pós-germânio (neutro em padrão ou removível na UI do Fuzz Face) |

## 10. Contratos de integração
- `module_id = "pedal.arbiter.fuzz_face_ge"`, `model_revision = "1967-pnp-germanium-ac128"`.
- Declara `input_impedance_ohms = 8_000` (baixa impedância) no `ModelDescriptor` para interação com a Spec 076 (entrada/captador) e aviso/comportamento realista quando posicionado atrás de buffers.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/drive.rs` (linhas 441–443, 627–662).
- **Dívidas registradas:**
  1. Acoplado em `DriveProcessor`; usa um único shaper assimétrico com `ff_sag_env`, ainda sem modelagem explícita dos 2 estágios acoplados Q1–Q2 com realimentação `100 kΩ` nem interação real com impedância de fonte (apontada como fora de escopo na Spec 041 e agora endereçada via Spec 076).
  2. O Fuzz Face 1967 real não possui knob `Tone` externo (apenas `Volume` e `Fuzz`, mais trim interno `Bias`); na UI atual `drive.tone` é compartilhado. Na modularização, definir `drive.tone` como não pertencente ao painel frontal vintage (ou explicitamente marcado como mod/capacitância de saída).
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/fuzz_face_ge.rs` (`ArbiterFuzzFaceGe`).

## 12. Estratégia DSP
- **Hybrid / State-Space simplificado:** filtro de acoplamento de entrada dependente de `source_impedance` (Spec 076) → modelo de 2 estágios acoplados com memória de bias (`sag_env` + deslocamento de ponto de operação + gating de corte em `Bias` extremo) em `8x` oversampling (Spec 079) → DC blocker `f64` na saída (essencial devido ao clipping fortemente assimétrico!).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; coeficientes de envelope e filtros invariantes à taxa (`44,1 / 48 / 88,2 / 96 kHz`).

## 14. Gestão de estado
- `ff_sag_env`, estados de acoplamento/DC blocker e oversampler encapsulados na instância `ArbiterFuzzFaceGe`.

## 15. Latência
- Latência do oversampler local (Spec 079).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 3,5%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Teste de "cleanup" (`-12 dBFS` vs `-30 dBFS` em `Fuzz=8.5`); medição de `H2/H3` (assimetria Ge); medição de gating em `Bias=1.5` vs `Bias=5.0`; medição de resíduo DC `< -60 dBFS` após DC blocker; aliasing em `5 kHz`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Assimetria de Germânio e DC Blocker):** Com `Fuzz = 7.5`, `Bias = 5.0` e senoide `1 kHz` a `-12 dBFS`, o 2º harmônico (`2 kHz`) supera `-24 dB` abaixo da fundamental, e o offset DC na saída permanece `< -60 dBFS`.
- **CA-2 (Limpeza com Volume de Entrada):** Com `Fuzz = 8.5`, `Bias = 5.0`, reduzir a entrada em `18 dB` (`-12 → -30 dBFS`) reduz a `THD` em `>= 40%`.
- **CA-3 (Ação Real do Bias Trim):** Mover `Bias` de `5.0` para `1.5` reduz a saída de sinais fracos (`-36 dBFS`) em `>= 6 dB` (efeito gated/velcro) e altera a razão `H2/H3`.
- **CA-4 (Anti-Aliasing e RT Safety):** Alias residual `< -55 dBFS` (5 kHz @ `-6 dBFS`, `Fuzz = 10`); zero alocações/desalocações em `process()`; entradas `NaN`/`Inf` sanitizadas.
- **CA-5 (Não-regressão):** Testes das Specs 032 e 041 para `FuzzFaceSiGe` continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 076` (Impedância de entrada/captador) + `Spec 079` (Oversampling).
- **Relacionadas / Legadas:** `Specs 032, 041`.

## 20. Limitações conhecidas
- Interação completa com a indutância física do captador requer ativação do modelo de entrada/captador (Spec 076); com DI já gravada via interface Hi-Z, o modelo aplica filtro aproximado de carga selecionável.

## 21. Riscos técnicos
- Clipping fortemente assimétrico pode introduzir offset DC audível (estalos no bypass/gate) se o DC blocker pós-clipper não estiver calibrado em `f64`.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `032`, `041`.
- **Código atual:** `crates/ampforge-dsp/src/fx/drive.rs`.
- **Testes existentes / futuros:** `contract_032_pedals_fx_cabs.rs`, `contract_041_dynamics_and_drives.rs`, futuro `contract_063_pedal_fuzz_face_ge.rs`.
