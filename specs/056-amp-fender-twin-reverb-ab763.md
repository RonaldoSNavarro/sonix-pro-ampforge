---
spec: 056
titulo: Amplificador Fender Twin Reverb AB763 (Rev. 1963–1967 Blackface, 85W)
status: rascunho
tipo: amplificador
atualizado: 2026-11-20
dependencias: [053, 077, 078, 079]
---
# 056 — Amplificador Fender Twin Reverb AB763 (Rev. 1963–1967 Blackface)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `amp.fender.twin_reverb_ab763` (aliases legados: `twin_ab763`, `us_twin_blackface_65`, `twin_reverb`, `usdeluxe`)
- **Revisão (`model_revision`):** `1965-ab763-blackface-85w`
- **Categoria:** Amplificador valvulado (`amplificador`)
- **Natureza do módulo:** Equipamento físico real
- **Objetivo:** Especificar o circuito **Fender Twin Reverb AB763 (85W Blackface, 1963–1967)** — incluindo sua topologia de Tone Stack **entre V1a e V1b** (`100 kΩ / 250 pF / 0,1 µF / 0,047 µF` com potenciômetro `Middle 10 kΩ` em ambos os canais Normal e Vibrato), capacitor `Bright` (`120 pF`) sobre o potenciômetro de `Volume` pós-tone stack, tanque de Spring Reverb valvulado e opto-tremolo exclusivos do canal Vibrato, inversor LTP `12AT7` e estágio de potência `4×6L6GC` fixed-bias de alto NFB — separando rigorosamente especificações do circuito físico de metas de engenharia do software (`ENGINEERING_TARGET`).

## 2. Escopo
- Canais `Normal` (`Input 1` High / `Input 2` Low −6 dB, `Bright`, `Volume`, `Treble`, `Middle`, `Bass`) e `Vibrato` (`Input 1` High / `Input 2` Low −6 dB, `Bright`, `Volume`, `Treble`, `Middle`, `Bass`, `Reverb`, `Speed`, `Intensity`).
- Posicionamento real do Tone Stack AB763 **após a placa de V1a (`R_src ≈ 38 kΩ`) e antes do potenciômetro de `Volume` (+ `Bright 120 pF`) e do 2º estágio de ganho V1b/V2b**.
- Eliminação da dupla contagem do mid-scoop de `450 Hz` (`amp.rs:822`) e desconexão de `Presence` e `Depth` (controles inexistentes no Twin Reverb AB763).

## 3. Fora de escopo
- Versões Silverface (`AC568` / anos 1970 com Master Volume pull-boost ou estágio ultralinear de 135 W).
- Fender Deluxe Reverb AB763 (`2×6V6`, retificador `GZ34` e resistor fixo `6,8 kΩ` sem knob `Middle` no canal Normal — documentado em F1).
- Simulação do gabinete 2×12 Open Back Jensen C12N / JBL D120F (coberta pela Spec 080).

## 4. Equipamento ou componente de referência
- **Fabricante:** Fender Musical Instruments Corporation (Fullerton, California, USA).
- **Modelo exato:** Twin Reverb Amp (Circuito `AB763`, Blackface 85W).
- **Período / Ano:** Julho de 1963 – 1967 (`AB763`).
- **Papel na cadeia de sinal:** Amplificador combo valvulado de 85 W RMS em `4 Ω` com altíssimo headroom limpo, recorte passivo de médios em `~400–500 Hz` e efeitos integrados de Spring Reverb e Opto-Tremolo ("Vibrato").

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `AB763 (Fender Twin Reverb, 1963–1967, 85W RMS em 4 Ω, retificação por diodos de silício, 4×6L6GC fixed-bias)`.
- **Distinção histórica entre modelos da família AB763 (confirmada em F1 e F2):**
  - Diferentemente do Deluxe Reverb AB763 e do Super Reverb AB763 (cujo canal Normal omite o potenciômetro `Middle` e usa um resistor fixo de `6,8 kΩ` para terra), o **Twin Reverb AB763** possui um potenciômetro **`Middle` de `10 kΩ` (`10K-A`) em ambos os canais (`Normal` e `Vibrato`)** (confirmado em F1, seção *Twin Reverb*).
  - O canal `Normal` usa `V1a` e `V1b` (`7025 / 12AX7`) e vai direto ao acoplamento do inversor de fase, sem passar pelo tanque de Spring Reverb nem pelo modulador de Tremolo.
  - O canal `Vibrato` usa `V2a` e `V2b` (`7025 / 12AX7`), passa pelo circuito de Spring Reverb (`V3 = 12AT7` driver + transformador de reverb + tanque Accutronics + `V4a = 12AX7` recovery + `V4b = 12AX7` mixer) e pelo opto-tremolo (`V5 = 12AX7` oscilador/driver + acoplador óptico lâmpada/LDR "roach").

## 6. Fontes técnicas e evidência por informação

### 6.1 Catálogo de fontes verificadas
| ID | Fonte | Fabricante / Modelo / Revisão | Página / Folha / Seção | URL / Caminho direto | Consulta | Classificação | Limitações da fonte |
|---|---|---|---|---|---|---|---|
| F1 | Rob Robinette — *AB763 Model Differences* | Fender Blackface AB763 Family (Twin Reverb vs Deluxe/Super Reverb) | Seção *Twin Reverb*: 85W, 4×6L6GC, Solid State Rectifier, Middle control on Normal channel, Dual Showman OT | `https://robrobinette.com/AB763_Model_Differences.htm` | 2026-11-20 | `SECONDARY_TECHNICAL` | Análise técnica comparativa baseada nos diagramas de serviço Fender AB763 |
| F2 | Rob Robinette — *How the AB763 Circuit Works* | Fender AB763 Circuit | Seções: Input Jacks (-6 dB), Preamp V1a/V1b, Tone Stack between V1a & V1b, Reverb, Opto-Tremolo, 12AT7 LTP, NFB | `https://robrobinette.com/How_The_AB763_Deluxe_Reverb_Works.htm` | 2026-11-20 | `SECONDARY_TECHNICAL` | Usa o diagrama AB763 detalhando cada estágio e valores de componentes |
| F3 | Rob Robinette — *How the TMB Tone Stack Works* & *How Spring Reverb Works* | Rede Fender AB763 (`100k / 250pF / 0.1µF / 0.047µF`) | Seções Fender TMB Mid Scoop & Accutronics Spring Reverb | `https://robrobinette.com/How_The_TMB_Tone_Stack_Works.htm` e `https://robrobinette.com/How_Spring_Reverb_Works.htm` | 2026-11-20 | `SECONDARY_TECHNICAL` | Análise de resposta em frequência da rede AB763 |
| F4 | D. T. Yeh & J. O. Smith — *Discretization of the '59 Fender Bassman Tone Stack* (DAFx-06) | Rede passiva FMV | pp. 1–8 (Equações KCL e impedância de fonte $R_{\text{src}}$) | `https://ccrma.stanford.edu/~dtyeh/papers/yeh06_dafx.pdf` | 2026-11-20 | `ACADEMIC` | Equações KCL para discretização exata da rede FMV |
| F5 | RCA / GE — *7025 / 12AX7, 12AT7 & 6L6GC Vacuum Tube Datasheets* | Válvulas padrão do Twin Reverb AB763 | Características estáticas e operação push-pull AB1 | Acervo de datasheets RCA (`6L6GC` 440–460V fixed bias, `12AT7` LTP/Reverb driver) | 2026-11-20 | `PRIMARY_COMPONENT` | Dados nominais das válvulas |
| F6 | Specs 020, 031, 039 e `docs/amps/voicing-fontes.md` | Contratos internos AMPFORGE | `Spec 031 CA-6`, `Spec 039 CA-5`, `voicing-fontes.md` §1 | `specs/031-*.md`, `specs/039-*.md`, `docs/amps/voicing-fontes.md` | 2026-11-20 | `INTERNAL_REFERENCE` | Documenta estado atual da implementação e metas de engenharia |

### 6.2 Evidência por afirmação técnica
| Informação / Parâmetro de Circuito | Fonte (ID) | Revisão | Página / Folha / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Potência nominal `85 W RMS` em carga de `4 Ω` (`2×12"` `8 Ω` em paralelo) com `4×6L6GC` e retificação por diodos de silício | F1 | Twin Reverb AB763 | Seção *Twin Reverb* | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| Presença de potenciômetro `Middle` (`10 kΩ`) em **ambos** os canais `Normal` e `Vibrato` no Twin Reverb AB763 | F1 | Twin Reverb AB763 | Seção *Twin Reverb* | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| Posição do Tone Stack **entre V1a e V1b** (alimentado pela placa de V1a com `Rp = 100 kΩ`, `Rk = 1,5 kΩ || 25 µF`, antes do `Volume 1 MΩ` + `Bright 120 pF`) | F2, F3 | AB763 | Seção *Preamp & Tone Stack* | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| Valores da rede Tone Stack AB763: `R_slope = 100 kΩ`, `C1 = 250 pF`, `C2 = 0,1 µF (100 nF)`, `C3 = 0,047 µF (47 nF)`, `R_treble = 250 kΩ-A`, `R_bass = 250 kΩ-A`, `R_mid = 10 kΩ-A` | F2, F3, F4 | AB763 | Seção *Tone Stack* / DAFx-06 | `SECONDARY_TECHNICAL` / `ACADEMIC` | Confirmado em fonte secundária e artigo |
| Entradas `Input 1` (High, `1 MΩ` + `68 kΩ`) e `Input 2` (Low, divisor `68 kΩ / 68 kΩ` = `-6 dB`) em cada canal | F2 | AB763 | Seção *Input Jacks* | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| Spring Reverb (`12AT7` driver + tanque de mola + `12AX7` recovery) e Opto-Tremolo (LDR "roach" shunt) exclusivos do canal `Vibrato` | F1, F2, F3 | AB763 | Seções *Reverb* e *Tremolo (Vibrato)* | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| Limite de `THD < 1,5%` a `-12 dBFS` de entrada com `Volume = 5.0` no simulador | F6 | AMPFORGE v1.0 | `Spec 056 / CA-2` | `INFERENCE` (`ENGINEERING_TARGET`) | Meta de engenharia do software (não é especificação de manual Fender) |

## 7. Fundamentação elétrica
- **Entradas por canal (`Input 1` High / `Input 2` Low — F2):** `Input 1` apresenta impedância de `1 MΩ` com grid stopper de `34 kΩ` (`68 kΩ || 68 kΩ`); `Input 2` forma divisor resistivo `68 kΩ / 68 kΩ` (**−6,02 dB**) com impedância de entrada de `136 kΩ`.
- **Arquitetura Pré-Tone-Stack (V1a → Tone Stack → Volume/Bright → V1b — F2, F3):**
  - Diferentemente do Marshall (onde o Tone Stack fica após toda a cascata de pré-amplificação, acionado por um cathode follower de baixa impedância), no Fender AB763 o sinal entra em **V1a** (`7025/12AX7`, `Rp = 100 kΩ`, `Rk = 1,5 kΩ || 25 µF`), sai pela **placa de V1a** (`R_src = Rp || r_p ≈ 38 kΩ`) e atravessa imediatamente o **Tone Stack passivo AB763** (`R_slope = 100 kΩ`, `C1 = 250 pF`, `C2 = 100 nF`, `C3 = 47 nF`, `Treble = 250 kΩ`, `Bass = 250 kΩ`, `Middle = 10 kΩ`).
  - Na saída do Tone Stack encontra-se o potenciômetro de **`Volume` (`1 MΩ` audio)** com a chave **`Bright` (`120 pF`)** ligando a entrada ao cursor do `Volume`, seguida pelo 2º estágio triodo **V1b** (`Rp = 100 kΩ`, `Rk = 1,5 kΩ || 25 µF`).
  - **Consequência física:** ajustar `Bass`, `Middle` e `Treble` altera quais frequências excitam e saturam V1b (e V4b no canal Vibrato), como verificado em `contract_039_amps_fidelity.rs:512–546`.
- **Caminho exclusivo do Canal Vibrato (F2, F3):**
  - Após V2b, o sinal divide-se entre o resistor de `3,3 MΩ || 10 pF` (caminho seco) e o driver de reverb `V3 (12AT7)` → tanque de mola → recuperação `V4a (12AX7)` → potenciômetro `Reverb` (`100 kΩ`), somando-se na grade de `V4b (12AX7)`.
  - Na saída do canal Vibrato antes do acoplamento ao inversor LTP, o foto-resistor (LDR) do opto-isolador deriva periodicamente parte do sinal para terra conforme o oscilador `V5` (`Speed` ≈ `1,5–9,5 Hz`, `Intensity` `0–10`).
- **Inversor de Fase e Potência (F1, F2):** LTP `V6 (12AT7)` com tail resistor elevado + NFB global forte (`820 Ω / 100 Ω` do secundário de `4 Ω`) acionando `4×6L6GC` fixed-bias (`B+ ≈ 440–460 V`) e transformador de saída Schumacher `022855` (85 W). **Não existem controles `Master Volume`, `Presence` nem `Depth`.**

## 8. Comportamento esperado
- **Headroom limpo elevado (meta de engenharia `ENGINEERING_TARGET` calibrada ao nível nominal `-12 dBFS`):** com `Volume = 5.0`, `B = M = T = 5.0`, `reverb = 0.0`, `vibrato_depth = 0.0`, senoide de `1 kHz` a `-12 dBFS` (`0.2512` pico) produz `THD < 1,5%` (meta do software AMPFORGE para representar a reserva de headroom limpo de 85 W das `4×6L6GC`).
- **Mid-scoop produzido pela rede passiva AB763 sem dupla contagem:** o vale em `~400–500 Hz` decorre da função de transferência KCL de `100 kΩ / 250 pF / 100 nF / 47 nF`; o filtro redundante `peaking(450 Hz, -4.5 dB)` em `amp.rs:822` deve ser removido na implementação da Spec 056.
- **Isolamento do Canal Normal:** quando `input_jack` está em `normal` ou `normal_low`, os controles `reverb`, `vibrato_speed` e `vibrato_depth` são desconectados do caminho de áudio (`diferença <= 1e-6`), enquanto `bass`, `middle` e `treble` continuam ativos.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Origem (`Físico` vs `Mod Shop`) | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|---|
| Power | `amp_1.power` | boolean | bool | `false/true` | `true` | discreto | `Físico` | Alimentação AC | Mute suave `<= 2,5 ms` |
| Standby | `amp_1.standby` | boolean | bool | `false/true` | `false` | discreto | `Físico` | Alta tensão B+ | Rampa exponencial invariante à taxa |
| Canal / Entrada | `amp_1.input_jack` | enum | id | `normal, vibrato, normal_low, low` | `vibrato` | discreto | `Físico` | Seleciona canal Normal vs Vibrato e Input 1 vs Input 2 (−6 dB) | Roteamento de reverb/tremolo + pad `-6,02 dB` |
| Bright | `amp_1.bright` | boolean | bool | `false/true` | `false` | discreto | `Físico` | Conecta capacitor `120 pF` sobre pot `Volume` pós-tone stack | Filtro passa-altas dependente de `gain` entre Tone Stack e V1b |
| Volume | `amp_1.gain` | number | scale | `0.0..=10.0` | `5.0` | audio log (`1 MΩ`) | `Físico` | Atenuação entre Tone Stack e V1b | Ganho pós-tone stack / pré-V1b |
| Treble | `amp_1.treble` | number | scale | `0.0..=10.0` | `6.0` | audio log (`250 kΩ`) | `Físico` | Braço agudo `250 pF` AB763 | KCL 5×5 + bilinear `f64` entre V1a e V1b |
| Middle | `amp_1.middle` | number | scale | `0.0..=10.0` | `5.0` | linear/audio (`10 kΩ`) | `Físico` | Braço médio `10 kΩ / 47 nF` (ativo em Normal e Vibrato) | KCL 5×5 + bilinear `f64` |
| Bass | `amp_1.bass` | number | scale | `0.0..=10.0` | `5.5` | audio log (`250 kΩ`) | `Físico` | Braço grave `100 nF` AB763 | KCL 5×5 + bilinear `f64` |
| Reverb | `amp_1.reverb` | number | scale | `0.0..=10.0` | `3.0` | audio log (`100 kΩ`) | `Físico` (Só Vibrato) | Nível de retorno do tanque de mola | Tanque dispersivo 4 linhas + matriz Hadamard |
| Speed | `amp_1.vibrato_speed` | number | scale | `0.0..=10.0` | `4.0` | linear (`1,5–9,5 Hz`) | `Físico` (Só Vibrato) | Frequência do oscilador do opto-tremolo | Frequência LFO em Hz invariante à taxa |
| Intensity | `amp_1.vibrato_depth` | number | scale | `0.0..=10.0` | `3.5` | log | `Físico` (Só Vibrato) | Profundidade de condução do LDR opto-acoplador | Atenuação AM periódica no canal Vibrato |

## 10. Contratos de integração
- Implementa `AudioProcessor` (Spec 053) com `module_id = "amp.fender.twin_reverb_ab763"` e `model_revision = "1965-ab763-blackface-85w"`.
- Rejeita `amp_1.master`, `amp_1.presence`, `amp_1.depth`, `amp_1.cut`, `amp_1.volume_2`, `amp_1.jumpered`, `amp_1.spongy` e `amp_1.channel_mode` com `E_VALIDATION:`.

## 11. Arquitetura de implementação
- **Código atual mapeado (HEAD `d891389`):** `crates/ampforge-dsp/src/amp.rs` (linhas `325–338, 493–549, 820–823, 1160–1165, 1209, 1363–1380, 1395–1400, 1506–1515`) e `crates/ampforge-dsp/src/tonestack.rs` (linhas `65–78`).
- **Problemas e dívidas encontrados no código atual:**
  1. `FACTUAL_ERROR` (**Dupla contagem do mid-scoop** em `amp.rs:820–823`): `create_voicing_filters` aplica `peaking(450.0, -4.5, 1.0, sr)` em cascata com o `PassiveToneStack` (`FenderTmbMidScoop`), duplicando o recorte de médios e contradizendo `docs/amps/voicing-fontes.md:17–19`.
  2. `FACTUAL_ERROR` (**Posição física do capacitor `Bright`** em `amp.rs:1395–1400`): o filtro `bright_state` está sendo aplicado na entrada bruta `s_in` *antes* de V1a, quando no esquema AB763 o capacitor de `120 pF` fica sobre o potenciômetro de `Volume` **após** o Tone Stack e antes de V1b.
  3. `FACTUAL_ERROR` (**`Presence` e `Depth` ativos no Twin Reverb** em `amp.rs:1161–1164`): os filtros `presence_filter` e `depth_filter` continuam no caminho de sinal mesmo no Twin Reverb AB763, que não possui nenhum desses controles.

## 12. Estratégia DSP
- **Circuit-level (`f64`):** V1a → Tone Stack AB763 (`100 kΩ / 250 pF / 100 nF / 47 nF`) via KCL + bilinear (Spec 078) → potenciômetro `Volume` com rede `Bright` (`120 pF`) dependente do cursor → V1b.
- **Behavioral calibrado com memória (`f64`):** triodos `7025/12AX7` de alto headroom (`B+_pre = 310 V`), tanque `SpringReverbTank` (4 linhas escaladas por taxa + matriz Hadamard + damping `3,65 kHz`), modulador opto-LDR e estágio push-pull `4×6L6GC` (`B+ = 440 V`, `nfb = 8.0`, `sag = 2.0`).
- **Oversampling:** seletivo (`2x` padrão, `4x`/`8x` selecionável — Spec 079).

## 13. Requisitos de tempo real
- `alloc_count == 0` e `dealloc_count == 0` em `process()` na thread RT; buffers do tanque de mola (`[[f64; 2048]; 4]`) estaticamente alocados na struct.

## 14. Gestão de estado
- Estados de V1a, Tone Stack, Bright, V1b, `SpringReverbTank`, `lfo_phase` e `power_stage` isolados na struct `FenderTwinReverbAb763`.

## 15. Latência
- Igual à latência do oversampler ativo (Spec 079): `4` (`2x`), `8` (`4x`) ou `16` (`8x`) amostras no modo `MinPhaseIir`.

## 16. Consumo de CPU
- Meta (`ENGINEERING_TARGET`): `<= 9,0%` do orçamento de `128` frames @ `48 kHz` em `2x` OS com Spring Reverb ativo (`--release`).

## 17. Testes
- Sinais determinísticos em `44,1`, `48` e `96 kHz`, `block_size = 128`: senoides `100 Hz`, `200 Hz`, `400 Hz`, `1 kHz`, `4 kHz`, `5 kHz`; burst de `10 ms` em `1 kHz` para medição da cauda de mola (`500 ms`); varredura de `Bass` em ganho alto verificando alteração de `H3` pós-V1b.

## 18. Critérios de aceite
- **CA-1 (`ANALYTICAL` — Tone Stack AB763 Pré-V1b Exato):** Com `sample_rate = 48000 Hz` (`8x` OS = `384 kHz` interno), `block_size = 128`, `B = M = T = 5.0`, a resposta do `PassiveToneStack` (`FenderTmbMidScoop`: `R_slope=100 kΩ, C1=250 pF, C2=100 nF, C3=47 nF, R_treble=250 kΩ, R_bass=250 kΩ, R_mid=10 kΩ`) medida em `100, 400, 1000 e 4000 Hz` coincide com a função de transferência analítica KCL dentro de `±0,5 dB`; además, em `gain = 8.5` com senoide de `100 Hz` a `0.14` pico (`-17,1 dBFS`), varrer `bass` de `0.0` para `10.0` altera a razão harmônica `H3/H1` (`300 Hz / 100 Hz`) em `> 0.015` (comprovando que o Tone Stack precede o estágio não linear V1b, F2/F3).
- **CA-2 (`ENGINEERING_TARGET` — Reserva de Headroom Limpo Calibrada):** Com senoide de `1 kHz` a `-12 dBFS` (`0.2512` pico, nível nominal calibrado da Spec 019) ou `0.06` pico (`-24,4 dBFS`), `sample_rate = 48000 Hz`, `block_size = 128`, `gain = 5.0`, `B = M = T = 5.0`, `reverb = 0.0`, `vibrato_depth = 0.0`, a `THD` medida em regime permanente após `2400` amostras permanece **`< 1,5%`** (a `0.06` pico) e **`< 3,0%`** (a `-12 dBFS`), permanecendo inferior a `0.5 × THD(BritJcm800_2203)` nas mesmas condições (meta de engenharia do software para representar o headroom limpo de 85 W das `4×6L6GC`).
- **CA-3 (`DOCUMENTED` — Isolamento Normal vs Vibrato e Middle em Ambos os Canais):** Com senoide de `1 kHz` a `-20 dBFS`, `sample_rate = 48000 Hz`, `block_size = 128`: (a) em `input_jack = "normal"`, variar `reverb` (`0→10`) ou `vibrato_depth` (`0→10`) altera a saída em `<= 1e-6`, enquanto variar `middle` (`0→10`) altera o ganho em `500 Hz` em `>= 6,0 dB` (F1); (b) em `input_jack = "vibrato"`, um burst de `10 ms` com `reverb = 10.0` gera cauda sustentada entre `250 ms` e `500 ms` com RMS `> -45 dBFS`, e `vibrato_depth = 8.0` modula o envelope na frequência `1.5 + 0.8 * vibrato_speed` (`±0,2 Hz`).
- **CA-4 (`DOCUMENTED` — Chave Bright `120 pF` e Invariância Multi-Taxa):** Com senoides de `200 Hz` e `5 kHz` a `-34 dBFS`, `block_size = 128`, `gain = 2.0`, `bright = true`, a inclinação `G(5 kHz) - G(200 Hz)` eleva-se em `>= 3,0 dB` relativamente a `bright = false`, e essa inclinação coincide entre `44,1`, `48` e `96 kHz` dentro de `±0,8 dB` (`contract_039_amps_fidelity.rs:165–191`).
- **CA-5 (`DOCUMENTED` — Ausência de Master, Presence e Depth):** `master`, `presence` e `depth` não alteram a saída do `twin_ab763` (`diferença <= 1e-6`) e enviar qualquer uma dessas chaves via `api::dispatch` retorna HTTP 400 (`E_VALIDATION:`).
- **CA-6 (`ENGINEERING_TARGET` — Segurança RT):** `CountingAllocator` na thread RT (`48 kHz`, `block_size = 128`, `100` blocos) registra `alloc_count == 0` e `dealloc_count == 0`; entradas `NaN`/`±Inf` são sanitizadas.
- **CA-7 (`REGRESSION` — Não-regressão):** Os testes `contract_031_authentic_amps.rs` e `contract_039_amps_fidelity.rs` aplicáveis ao `UsTwinBlackface65` continuam passando.

## 19. Dependências
- **Obrigatórias:** `Spec 053`, `Spec 077`, `Spec 078`, `Spec 079`.
- **Relacionadas / Legadas:** `Specs 003, 020, 031, 039`.

## 20. Limitações conhecidas
- O tanque de mola integrado utiliza rede FDN Hadamard de 4 linhas (`SpringReverbTank`) otimizada para latência zero e baixo custo de CPU dentro do bloco do amplificador.

## 21. Riscos técnicos
- Remover o filtro duplicado `peaking(450 Hz, -4.5 dB)` em `amp.rs:822` aumentará a energia de médios do Twin Reverb em ~4,5 dB; calibrar `tonestack_recovery_gain` para preservar a separação inter-modelos exigida em `contract_039_amps_fidelity.rs` (`test_ca_09`).

## 22. Matriz de rastreabilidade
| Requisito | Fonte | Spec/CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Tone Stack AB763 (`100k / 250pF / 100nF / 47nF`) posicionado entre V1a e V1b | F2, F3, F4 | `Spec 056 / CA-1` | `crates/ampforge-dsp/src/tonestack.rs:65–78`, `amp.rs:1506–1515` | `crates/ampforge-dsp/tests/contract_039_amps_fidelity.rs` (`test_ca_01`, `test_ca_09`) | `TESTED` |
| Potenciômetro `Middle 10 kΩ` ativo em ambos os canais `Normal` e `Vibrato` | F1 | `Spec 056 / CA-3` | `crates/ampforge-dsp/src/amp.rs:1142–1148, 1506–1515` | `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs` (`ca_6`) | `TESTED` |
| Isolamento de `Reverb` e `Vibrato` no canal `Normal` | F1, F2 | `Spec 056 / CA-3` | `crates/ampforge-dsp/src/amp.rs:1366–1379` | `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs` (`ca_6`) | `TESTED` |
| Remoção da dupla contagem de mid-scoop (`peaking 450 Hz -4.5 dB`) e posição pós-Tone-Stack do `Bright 120 pF` | F2, F3, F6 | `Spec 056 / CA-1, CA-4` | `crates/ampforge-dsp/src/amp.rs:820–823, 1395–1400` *(dívidas registradas)* | Planejado: `contract_056_amp_fender_twin_ab763.rs` | `SPECIFIED` |
| Desconexão total de `Presence` e `Depth` no caminho de sinal do Twin Reverb | F1, F2 | `Spec 056 / CA-5` | `crates/ampforge-dsp/src/amp.rs:1161–1164` *(dívida: filtros ainda ativos)* | Planejado: `contract_056_amp_fender_twin_ab763.rs` | `SPECIFIED` |
