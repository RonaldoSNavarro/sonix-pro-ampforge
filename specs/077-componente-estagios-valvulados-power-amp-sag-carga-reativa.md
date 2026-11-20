---
spec: 077
titulo: Componente Compartilhado de Estágios Valvulados, Power Amp, Fonte/Sag e Carga Reativa
status: rascunho
tipo: componente-compartilhado
atualizado: 2026-11-19
dependencias: [053, 079]
---
# 077 — Componente de Estágios Valvulados, Power Amp, Fonte/Sag e Carga Reativa

## 1. Identificação e objetivo
- **Identificador estável (`module_id`):** `component.tubes.stages_power_sag_load`
- **Revisão (`model_revision`):** `v1-adaa1-koren-sag-xformer`
- **Categoria:** Biblioteca interna compartilhada de blocos físicos de amplificadores
- **Objetivo:** Especificar os blocos reutilizáveis que compõem os 5 amplificadores individuais (Specs 054–058) e a bancada **Chassi Aberto / Mod Shop** — estágio de triodo (cátodo comum + cold clipper + cathode follower), inversor de fase LTP, estágio de potência push-pull com bias/crossover e NFB, fonte/retificador com sag dinâmico B+, saturação LF de transformador de saída e acoplamento de carga reativa — garantindo consistência física sem forçar uma topologia única para todos os amps.

## 2. Escopo
- `TubeTriodeStage` (ADAA1 com assimetria calibrada `kp/kn`, ganho de válvula `12AX7/ECC83/12AT7/12AY7/EF86`, headroom escalado por `preamp_plate_v`, e interface pronta para evolução opcional a modelo Norman Koren / Newton-Raphson `f64` de `precision.rs`).
- `CathodeFollowerStage` (compressão assimétrica suave na excursão positiva e baixa impedância de saída).
- `PushPullPowerStage` (LTP + válvulas `EL34/6L6GC/6V6/KT88/EL84`, bias `20–55 mA` com crossover notch em bias frio `< 32 mA` e aquecimento em bias quente, linearização por NFB global, filtros `Presence`/`Depth` modulados pelo NFB).
- `PowerSupplySag` (retificadores `SolidStateDiode`, `Gz34_5Ar4`, `Dual5U4Gb` com `R_i`, `tau_att`, `tau_rel` e leitura instantânea `current_b_plus_v()`).
- `OutputTransformerAndLoad` (saturação de núcleo em baixas frequências `~150 Hz` + acoplamento reativo de impedância `4/8/16 Ω` com `speaker_resonance_hz`).
- Serviços de telemetria real para a bancada CRT (`transfer`, `tonestack`, `impedance`, `waveform` — Spec 047).

## 3. Fora de escopo
- Topologia completa de cada amplificador (cada amp compõe estes blocos à sua maneira nas Specs 054–058).
- Tone stacks passivos (Spec 078) e oversampling (Spec 079).

## 4. Equipamento ou componente de referência
- **Válvulas de referência:** `12AX7/ECC83` (`µ=100`), `12AT7` (`µ=60`), `12AY7` (`µ=45`), `EF86` (pentodo); `EL34`, `6L6GC`, `6V6`, `KT88`, `EL84`; retificadoras `GZ34 (5AR4)` e `5U4GB`.

## 5. Revisão e variantes
- **Revisão:** `v1-adaa1-koren-sag-xformer`.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | Norman Koren — *Improved Vacuum Tube Models for SPICE Simulations* | Triodos/Pentodos | Equações `Ip(Vg, Vp)` | Norman Koren Tubes Page | 2026-11-19 | `[confirmado em documentação]` | Parametrização física de curvas de placa de `12AX7`, `EL34`, `6L6GC`, `EL84` |
| F2 | J. Parker et al. — *Reducing the Aliasing of Nonlinear Waveshaping Using Continuous-Time Convolution* (DAFx-16) | ADAA 1ª/2ª ordem | Antiderivada e guarda `|dx| < eps` | Proc. DAFx-16 | 2026-11-19 | `[confirmado em documentação]` | Fórmula ADAA1 `(F(x[n]) - F(x[n-1])) / (x[n] - x[n-1])` com fallback no ponto médio |
| F3 | Datasheets Mullard / RCA (`12AX7`, `EL34`, `6L6GC`, `EL84`, `GZ34`, `5U4GB`) | Válvulas padrão | `µ`, resistência interna, quedas de tensão | Datasheets fabricantes | 2026-11-19 | `[confirmado em documentação]` | Relações de ganho/headroom e queda de retificação `diode < GZ34 < 5U4GB` |
| F4 | `crates/ampforge-dsp/src/{amp.rs, modshop.rs, precision.rs}` | Workspace v1.0 | `triode_adaa1`, `RectifierType`, `MixedPrecisionSolver` | `crates/ampforge-dsp/src/` | 2026-11-19 | `[confirmado em documentação]` | Implementação existente e parâmetros do Mod Shop |

## 7. Fundamentação elétrica
- **Triodo + ADAA1 (`f64`):** `f(u) = u / sqrt(1 + (k·u)²)` com `k_p` (semiciclo positivo) ≠ `k_n` (semiciclo negativo) modelando assimetria de grade/cátodo; antiderivada fechada `F(w) = (sqrt(1 + (k·w)²) - 1) / k²` com expansão de Taylor para `k·w < 1e-4` garantindo estabilidade numérica sem cancelamento `[confirmado em documentação: F2, F4]`.
- **Bias e Crossover Push-Pull:** bias frio (`20–32 mA`) introduz zona morta próxima ao cruzamento por zero (*crossover notch*, gerando harmônicos ímpares ásperos); bias quente (`40–55 mA`) elimina crossover e antecipa compressão simétrica suave.
- **NFB e Carga Reativa:** `eff_nfb` lineariza a distorção do power amp (`1 / (1 + 0.115 * eff_nfb)`), modula o ganho dos filtros `Presence`/`Depth` e controla o amortecimento do pico de ressonância do alto-falante (`res_boost_db ∝ (10 - eff_nfb) * imp_factor(4/8/16 Ω)`).
- **Sag de Fonte B+:** envoltória assimétrica (`tau_att`, `tau_rel`) dependente da resistência interna `R_i` (`diode: 0.15 / 10ms / 20ms`, `gz34: 0.45 / 30ms / 80ms`, `5u4gb: 0.70 / 40ms / 120ms`) modulando dinamicamente a tensão efetiva `B+` do estágio de potência.
- **Transformador de Saída:** saturação de fluxo em baixas frequências (`LPF 150 Hz` → `tanh` sobre fluxo → perda diferencial LF proporcional a `xformer_sat`).

## 8. Comportamento esperado
- **Troca de válvula de pré sem estalos:** alternar `12AX7 ↔ 12AT7 ↔ 12AY7 ↔ EF86` altera ganho (`1.0 / 0.58 / 0.45 / 1.22`) e joelho com suavização (`~8 ms`) sem cliques (Spec 021 CA-11).
- **Efeito verificável de cada parâmetro da Bancada (Mod Shop):** `preamp_plate_v` (`180–340 V`), `triode_asym` (`0–0,6`), `bias_ma` (`20–55 mA`), `sag` (`0–10`), `rectifier` (`diode/gz34/5u4gb`), `nfb` (`0–10`), `xformer_sat` (`0–10`) e `impedance_ohms` (`4/8/16 Ω`) produzem variações mensuráveis no sinal e refletidas no osciloscópio CRT real (Spec 047).

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito físico | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Válvula Pré | `amp_1.preamp_tube` | enum | id | `12ax7, ecc83, 12at7, 12ay7, ef86` | por amp | discreto | Altera `µ` e joelho | Ganho suavizado + `ef86_knee` |
| Válvula Potência | `amp_1.power_tube` | enum | id | `el34, 6l6gc, 6v6, kt88, el84_variant` | por amp | discreto | Headroom de potência | `power_tube_headroom` |
| Retificador | `amp_1.rectifier` | enum | id | `diode, gz34, 5u4gb` | por amp | discreto | `R_i`, ataque/release de sag B+ | Parâmetros de `PowerSupplySag` |
| Impedância Sec. | `amp_1.impedance_ohms` | enum | Ω | `4, 8, 16` | por amp | discreto | Amortecimento e ressonância | `imp_factor` no pico reativo |
| Bias de Grade | `amp_1.bias_ma` | number | mA | `20.0..=55.0` | por amp | linear | Frio = crossover notch; Quente = drive | Dip de cruzamento + `bias_warm_drive` |
| Sag | `amp_1.sag` | number | scale | `0.0..=10.0` | por amp | linear | Profundidade da queda B+ | Escala `target_drop` |
| NFB | `amp_1.nfb` | number | scale | `0.0..=10.0` | por amp | linear | Linearização, Presence/Depth, damping | Malha de realimentação |
| Sat. Trafo | `amp_1.xformer_sat` | number | scale | `0.0..=10.0` | por amp | linear | Saturação de núcleo LF (~150 Hz) | Fluxo LF `tanh` |
| Tensão Placa B+ | `amp_1.preamp_plate_v` | number | V | `180.0..=340.0` | por amp | linear | Headroom dos estágios de pré | `v1_headroom = plate_v / 250` |
| Assimetria | `amp_1.triode_asym` | number | razão | `0.0..=0.6` | por amp | linear | Harmônicos pares `2f0` | `kp` vs `kn` no ADAA1 |

## 10. Contratos de integração
- Biblioteca pura dentro de `crates/ampforge-dsp/src/shared/` consumida por composição pelas Specs 054–058; não conhece `AmpModelId` nem detalhes específicos de nenhum amplificador!

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/amp.rs` (ADAA1, power stage, sag, xformer), `modshop.rs` (enums e telemetria) e `precision.rs` (`MixedPrecisionSolver`).
- **Dívidas registradas:**
  1. `modshop.rs::generate_scope_telemetry()` (linhas 414–478) contém curvas sintéticas legadas (enquanto a produção da Spec 047 já usa dados reais do motor em `api.rs`); remover/deprecar curvas sintéticas para evitar divergência com `amp.rs`.
  2. `MixedPrecisionSolver` em `precision.rs` ainda não é consumido pelos estágios de válvula em produção (listado em `docs/biblioteca-sem-uso-em-producao.md`); consolidar como utilitário opcional para futuros estágios Koren MNA/Newton-Raphson.
- **Estrutura alvo:** `crates/ampforge-dsp/src/shared/{tube_stage.rs, power_amp.rs, power_supply.rs}`.

## 12. Estratégia DSP
- Toda avaliação não linear e estado de memória roda em **`f64`** dentro da taxa oversampled fornecida pela Spec 079; ADAA1 analítico para baixa latência e excelente supressão de aliasing; expansão opcional de estágios selecionados para solver Newton-Raphson amortecido sobre modelo Koren (`<= 6` iterações determinísticas).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; todas as constantes temporais (`tau_att`, `tau_rel`, `lf_alpha`, `tube_alpha`) derivadas da taxa efetiva.

## 14. Gestão de estado
- Cada sub-bloco (`TubeTriodeStage`, `PushPullPowerStage`, `PowerSupplySag`, `OutputTransformer`) mantém apenas escalares `f64` internos e implementa `reset()`.

## 15. Latência
- **`0` amostras** intrínsecas (latência total vem do oversampler externo da Spec 079).

## 16. Consumo de CPU
- Orçamento alvo: **`< 0,5%`** por `TubeTriodeStage` e **`< 2,5%`** por `PushPullPowerStage + Sag + Transformer` @ 128 frames / 48 kHz em 2× OS (`--release`).

## 17. Testes
- Continuidade e derivada da curva ADAA1 em `dx → 0`; monotonicidade de headroom com `preamp_plate_v`; razão `H2` com `triode_asym = 0` vs `0.4`; crossover distortion com `bias_ma = 20` vs `45`; queda e recuperação de `current_b_plus_v()` para `diode`, `gz34`, `5u4gb`; perda LF em `100 Hz` com `xformer_sat = 10`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Estabilidade ADAA1):** Em toda faixa `u ∈ [-100, +100]` e `|dx| ∈ [0, 1e-12..10]`, `triode_adaa1` e `power_adaa1` nunca retornam `NaN`/`Inf` e coincidem com a curva estática no limite `dx → 0` (`erro <= 1e-6`).
- **CA-2 (Independência dos 10 Controles Mod Shop):** Variar cada um dos 10 parâmetros do chassi de ponta a ponta altera mensuravelmente ganho, THD, razão `H2/H3`, `current_b_plus_v()` ou resposta espectral no sentido físico esperado.
- **CA-3 (Hierarquia de Sag por Retificador):** Sob o mesmo burst, `sag_drop(5u4gb) > sag_drop(gz34) > sag_drop(diode)` e `t_recover(5u4gb) > t_recover(gz34) > t_recover(diode)`.
- **CA-4 (Invariância de Taxa e RT Safety):** Tempos de sag, suavização de válvula e frequência de núcleo do transformador coincidem em `±2%` entre `44,1`, `48` e `96 kHz`; zero alocações/desalocações em `process()`.
- **CA-5 (Não-regressão):** Testes das Specs 004, 021, 031 e 047 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`, `Spec 079`.
- **Dependentes:** `Specs 054, 055, 056, 057, 058`.
- **Relacionadas / Legadas:** `Specs 004, 021, 031, 039, 047`.

## 20. Limitações conhecidas
- Blocking distortion por carregamento de capacitor de acoplamento via corrente de grade não está modelada em MNA completo nesta revisão v1.

## 21. Riscos técnicos
- Overflow de `cosh(x).ln()` para `|x| > 20` no ADAA do `tanh`: manter a proteção assintótica `|x| - ln(2)` já existente (`amp.rs:1609–1618`).

## 22. Referências e rastreabilidade
- **Specs anteriores:** `004`, `021`, `031`, `039`, `047`.
- **Código atual:** `crates/ampforge-dsp/src/{amp.rs, modshop.rs, precision.rs}`.
- **Testes existentes / futuros:** `contract_004_modshop_crt.rs`, `contract_021_workbench_live.rs`, `contract_047_workbench_scope.rs`, futuro `contract_077_tube_power_sag.rs`.
