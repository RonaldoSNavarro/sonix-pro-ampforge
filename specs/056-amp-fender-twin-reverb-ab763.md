---
spec: 056
titulo: Amplificador Fender Twin Reverb AB763 (1965 Blackface, 85W)
status: rascunho
tipo: amplificador
atualizado: 2026-11-19
dependencias: [053, 077, 078, 079]
---
# 056 — Amplificador Fender Twin Reverb AB763 (Rev. 1963–1967 Blackface)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `amp.fender.twin_reverb_ab763` (alias legado: `twin_ab763`, `us_twin_blackface_65`, `twin_reverb`)
- **Revisão (`model_revision`):** `ab763-1965-blackface`
- **Categoria:** Amplificador valvulado (módulo independente)
- **Objetivo:** Especificar integralmente o circuito **Fender Twin Reverb AB763 (85W Blackface, 1963–1967)** — incluindo sua arquitetura de Tone Stack **entre V1a e V1b**, alto headroom de `4×6L6GC`, tanque de Spring Reverb valvulado e Tremolo ("Vibrato") exclusivos do canal Vibrato — como módulo DSP autônomo.

## 2. Escopo
- Canais `Normal` (sem reverb/vibrato) e `Vibrato` (com Spring Reverb e opto-tremolo, ambos possuindo `Middle` de 10 kΩ no Twin AB763), entradas `High` (Input 1) e `Low` (Input 2, −6 dB), chave `Bright` (`120 pF`).
- Topologia AB763: V1a → Tone Stack passivo AB763 (`100 kΩ / 250 pF / 0.1 µF / 0.047 µF`) → Volume + Bright → V1b recuperação → (no canal Vibrato: envio/retorno de Spring Reverb + opto-tremolo) → LTP 12AT7 → 4×6L6GC fixed-bias (alto NFB, sem Master Volume) → transformador Schumacher 022855.

## 3. Fora de escopo
- Versões Silverface pós-1968 com Master Volume (pull-boost) ou estágio ultralinear de 135 W.
- Fender Deluxe Reverb AB763 (que usa 2×6V6, retificador GZ34 e resistor fixo de 6,8 kΩ em vez de pot Middle no canal Normal).
- Simulação do gabinete 2×12 Open Back Jensen/JBL (coberta pela Spec 080).

## 4. Equipamento ou componente de referência
- **Fabricante:** Fender Musical Instruments.
- **Modelo exato:** Twin Reverb (Blackface 85W).
- **Período / Ano:** 1963–1967 (circuito `AB763`).
- **Papel na cadeia de sinal:** Referência de altíssimo headroom limpo, mid-scoop natural em ~400–500 Hz, plataforma de pedais e efeitos integrados de Spring Reverb e Tremolo.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `AB763 (Twin Reverb Blackface, 1965, 85W, retificação diodo silício, 4×6L6GC)`.
- **Variantes históricas relevantes (não misturadas):**
  - *AA763 (1963):* pequenas diferenças de bias/divisor no vibrato; AB763 padronizou o circuito Blackface clássico.
  - *AB763 Deluxe Reverb vs Twin Reverb:* no Twin Reverb AB763, **ambos** os canais Normal e Vibrato possuem potenciômetro `Middle` (10 kΩ linear), verificado na Spec 031 Notas (2026-11-14).
  - *AC568 / Silverface:* alterações na rede de bias e presença de Master Volume nos anos 70.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | Esquema oficial Fender Twin Reverb `AB763` (Jul/1963) | AB763 Twin Reverb | Schematic completo | Schematic Heaven / Fender Service Diagrams | 2026-11-19 | `[confirmado em documentação]` | Tone stack `100k / 250pF / 0.1µF / 0.047µF`, pot Middle 10k em ambos os canais, 12AT7 LTP, 4×6L6GC fixed-bias (~440–460V), Schumacher 022855 |
| F2 | `docs/amps/voicing-fontes.md` | Twin Reverb 1965 AB763 | Seção 1 | `docs/amps/voicing-fontes.md` | 2026-11-19 | `[confirmado em documentação]` | Valores exatos do tone stack `FenderTmbMidScoop` e características do transformador 022855 |
| F3 | Rob Robinette — *How the AB763 Circuit Works* | AB763 | Tone Stack, Reverb, Opto-Tremolo, Power | https://robrobinette.com/ | 2026-11-19 | `[fonte secundária]` | Posição do tone stack **entre V1a e V1b**, driver de mola 12AT7, opto-isolador LDR ("roach") do tremolo |
| F4 | Spec 031 (Notas de 2026-11-14) e Spec 039 CA-5 | AB763 | Verificação Middle e capacitores | `specs/031-*.md`, `specs/039-*.md` | 2026-11-19 | `[confirmado em documentação]` | Confirmação de `C2=100 nF`, `C3=47 nF` e Middle presente em ambos os canais |

## 7. Fundamentação elétrica
- **Entradas:** `Input 1` (High, `1 MΩ`) e `Input 2` (Low, atenuação `−6 dB` e impedância reduzida) para cada canal (`Normal` e `Vibrato`).
- **Posição do Tone Stack (crítica no AB763):** diferente de Marshall (pós-pré via cathode follower), o AB763 coloca o Tone Stack passivo **logo após V1a** (`12AX7`, `Rp = 100 kΩ`, `Rk = 1.5 kΩ || 22–25 µF`) e **antes** do potenciômetro de `Volume` (`1 MΩ` com `Bright` `120 pF`) e do 2º estágio de recuperação **V1b** `[confirmado em documentação: F1, F3]`.
- **Rede Tone Stack AB763:** `R_slope = 100 kΩ`, `C1 = 250 pF`, `C2 = 0.1 µF (100 nF)`, `C3 = 0.047 µF (47 nF)`, `R_treble = 250 kΩ (audio)`, `R_bass = 250 kΩ (audio)`, `R_mid = 10 kΩ (linear)`, `R_load = 1 MΩ` `[confirmado em documentação: F1, F2]`. Produz o recorte natural de médios centrado em `~400–500 Hz`.
- **Canal Vibrato (exclusivo):**
  - **Spring Reverb:** driver `12AT7` acoplado por transformador a tanque Accutronics de molas, recuperado por `1/2 12AX7` e somado ao sinal seco pelo potenciômetro `Reverb` (`100 kΩ`) antes do estágio misturador `[fonte secundária: F3]`.
  - **Vibrato (Opto-Tremolo):** oscilador de deslocamento de fase (`12AX7`) modulando lâmpada/LDR (opto-acoplador) que varia a atenuação de sinal na saída do canal Vibrato (`Speed` ≈ 1,5–9,5 Hz, `Intensity` 0–10) `[fonte secundária: F3]`.
- **Inversor de Fase e Potência:** LTP com `12AT7` (menor µ, alta capacidade de corrente) acionando `4×6L6GC` fixed-bias (`B+ ≈ 440–460 V`, `470 Ω` screen resistors, `1.5 kΩ` grid stoppers `[confirmado em documentação: F1]`), NFB elevado (`820 Ω / 100 Ω` → alto fator de amortecimento e máxima limpeza), retificação por diodos de silício e transformador de saída Schumacher `022855` (85 W).

## 8. Comportamento esperado
- **Headroom limpo superior:** a `-12 dBFS` de entrada e `Volume = 5.0`, THD permanece **`< 1,5%`** (muito abaixo de Plexi/JCM800/Rectifier na mesma posição), mantendo transientes percussivos com alto crest factor.
- **Mid-scoop intrínseco sem contagem dupla:** o vale em `400–500 Hz` advém diretamente da rede KCL AB763 (`100k/250pF/100nF/47nF`); não empilhar segundo notch excessivo de médios sobre a rede passiva.
- **Diferença real entre Canal Normal e Vibrato:** em `Normal`, `reverb` e `vibrato_depth` são fisicamente desconectados (efeito zero na saída); em `Vibrato`, `reverb > 0` adiciona cauda dispersiva de mola e `vibrato_depth > 0` modula amplitude na taxa `vibrato_speed`.
- **Bright Switch (`120 pF`):** eleva brilho acima de `2 kHz` em volumes baixos/médios e reduz seu efeito relativo à medida que `Volume → 10`.
- **Ausência de Master Volume:** controle de volume total é o `Volume` do canal; `amp_1.master` não existe na revisão AB763.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Power | `amp_1.power` | boolean | bool | `false/true` | `true` | discreto | Alimentação AC | Mute suave `<= 2,5 ms` |
| Standby | `amp_1.standby` | boolean | bool | `false/true` | `false` | discreto | Corte B+ | Rampa exponencial invariante à taxa |
| Canal / Entrada | `amp_1.input_jack` | enum | id | `normal, vibrato, normal_low, low` | `vibrato` | discreto | Seleciona canal Normal vs Vibrato e pad −6 dB | Roteamento de reverb/tremolo + pad `-6 dB` |
| Bright | `amp_1.bright` | boolean | bool | `false/true` | `false` | discreto | Capacitor `120 pF` sobre pot Volume | Shelf/HP dependente da posição de `gain` |
| Volume | `amp_1.gain` | number | scale | `0.0..=10.0` | `5.0` | audio log | Ganho pós-tone stack antes de V1b | Ganho suave + ação inversa do Bright |
| Treble | `amp_1.treble` | number | scale | `0.0..=10.0` | `6.0` | audio log | Braço agudo `250k / 250pF` AB763 | KCL 5×5 + bilinear `f64` entre V1a e V1b |
| Middle | `amp_1.middle` | number | scale | `0.0..=10.0` | `5.0` | linear | Braço médio `10k Lin` AB763 | KCL 5×5 + bilinear `f64` |
| Bass | `amp_1.bass` | number | scale | `0.0..=10.0` | `5.5` | audio log | Braço grave `250k / 100nF` AB763 | KCL 5×5 + bilinear `f64` |
| Reverb | `amp_1.reverb` | number | scale | `0.0..=10.0` | `3.0` | audio log | Mix do tanque de mola (somente canal Vibrato) | Tanque dispersivo 4 linhas + matriz Hadamard |
| Vibrato Speed | `amp_1.vibrato_speed` | number | scale | `0.0..=10.0` | `4.0` | linear/exp (1,5–9,5 Hz) | Taxa do oscilador opto-tremolo | Frequência LFO em Hz invariante à taxa |
| Vibrato Intensity | `amp_1.vibrato_depth` | number | scale | `0.0..=10.0` | `3.5` | log | Profundidade da modulação opto-LDR | Modulação de amplitude (somente canal Vibrato) |

*(Defaults de chassi/Mod Shop Spec 077: `12ax7`, `6l6gc`, `diode`, `4 Ω` nominal no combo 2×12 / selecionável `4/8/16`, `bias=35 mA`, `sag=2.0`, `nfb=8.0`, `xformer_sat=2.0`, `B+_pre=310 V`, `B+_power=440 V`.)*

## 10. Contratos de integração
- `module_id = "amp.fender.twin_reverb_ab763"`, `model_revision = "ab763-1965-blackface"`.
- Rejeita `amp_1.master`, `amp_1.presence`, `amp_1.depth`, `amp_1.cut`, `amp_1.volume_2`, `amp_1.jumpered`, `amp_1.spongy`, `amp_1.channel_mode` com `E_VALIDATION:` (controles inexistentes no AB763).

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/amp.rs` (linhas 325–338, 493–549, 820–823, 1366–1379, 1506–1515) e `tonestack.rs:65–78`.
- **Dívidas registradas:**
  1. `amp.rs:820–823` aplica um segundo filtro `peaking(450 Hz, -4.5 dB)` sobre o tone stack AB763, contradizendo `docs/amps/voicing-fontes.md` (que registra `high_shelf(4500 Hz, +1.5 dB)` + `low_shelf(80 Hz, +0.5 dB)` justamente para evitar dupla contagem do mid-scoop). Na modularização, alinhar com `voicing-fontes.md` ou medição direta.
  2. O opto-tremolo atual usa senoide simétrica (`amp.rs:1419`), enquanto o LDR/opto real apresenta curva assimétrica suave de ataque/queda; documentado como refinamento comportamental.
  3. `amp_1.presence` e `amp_1.depth` atualmente ainda são processados em `amp.rs:1630–1631` mesmo quando o Twin Reverb (sem esses knobs) está selecionado — na modularização, devem permanecer neutros (`0 dB`).
- **Estrutura alvo:** `crates/ampforge-dsp/src/amps/twin_ab763.rs` (`FenderTwinReverbAb763`).

## 12. Estratégia DSP
- **Circuit-level:** Tone Stack AB763 KCL 5×5 + bilinear `f64` posicionado **entre V1a e V1b** (Spec 078).
- **Behavioral calibrado com memória:** V1a e V1b de alto headroom (ADAA1), tanque de mola integrado (4 linhas escaladas por `sample_rate` + Hadamard + damping LP), LFO de tremolo com curva LDR suave, estágio 4×6L6GC com NFB alto (`8.0`) e sag contido (`2.0`).
- **Oversampling:** seletivo (`2x` padrão, `4x`/`8x` selecionável, Spec 079).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; comprimentos de linha do tanque de mola pré-alocados até 96 kHz e reescalados sem realocação.

## 14. Gestão de estado
- `SpringReverbTank`, `lfo_phase`, estados V1a/V1b, `PassiveToneStack` e `power_stage` isolados na instância; `reset()` limpa integralmente o tanque de mola em O(1).

## 15. Latência
- Igual à latência do oversampler ativo (Spec 079); tanque de mola em paralelo não adiciona latência ao sinal seco.

## 16. Consumo de CPU
- Orçamento alvo: **`<= 9%`** do deadline de 128 frames @ 48 kHz em `2x` OS (incluindo Spring Reverb ativo).

## 17. Testes
- Sweep 20 Hz–20 kHz validando resposta AB763 (`100k/250pF/100nF/47nF`) vs analítica; senoide 1 kHz a `-12 dBFS` validando `THD < 1,5%` em `Volume=5`; impulso para cauda do Spring Reverb em `Vibrato` vs silêncio de cauda em `Normal`; medição de modulação AM na taxa `vibrato_speed` em `Vibrato` vs ausente em `Normal`; validação multi-taxa `44,1/48/96 kHz`.

## 18. Critérios de aceite
- **CA-1 (Tone Stack AB763 exato):** Com `B=M=T=5`, a curva analógica/digital do tone stack coincide com a referência KCL AB763 (`100k/250pF/100nF/47nF`) dentro de `±0,5 dB` de `20 Hz` a `10 kHz`, exibindo mid-scoop natural em `400–500 Hz`.
- **CA-2 (Alto Headroom Limpo):** Com `Volume = 5.0`, `B=M=T=5`, `reverb=0`, `vibrato_depth=0`, senoide 1 kHz a `-12 dBFS` apresenta `THD < 1,5%`.
- **CA-3 (Roteamento Normal vs Vibrato):** No canal `Normal`, alterar `reverb` (`0→10`) ou `vibrato_depth` (`0→10`) produz diferença `<= 1e-6` na saída; no canal `Vibrato`, `reverb = 7` gera cauda sustentada (`RT > 0,8 s`) e `vibrato_depth = 8` gera modulação de amplitude na frequência configurada por `vibrato_speed` (`±0,2 Hz`).
- **CA-4 (Bright Switch):** Com `Volume = 2.5`, ativar `bright = true` eleva `4 kHz` relativo a `250 Hz` em `>= 3 dB`; com `Volume = 9.5`, essa diferença cai pela metade.
- **CA-5 (Sem Controles Fantasmas):** `master`, `presence` e `depth` não afetam a saída do Twin Reverb e são rejeitados com `E_VALIDATION:`.
- **CA-6 (Erro e RT Safety):** Zero alocações/desalocações em `process()`; entradas `NaN`/`Inf` sanitizadas; saída finita em `[-1.0, 1.0]` em `44,1 / 48 / 96 kHz`.
- **CA-7 (Não-regressão):** Testes existentes das Specs 020, 031 e 039 para `UsTwinBlackface65` continuam passando.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 077` + `Spec 078` + `Spec 079`.
- **Relacionadas / Legadas:** `Specs 003, 020, 031, 039`.

## 20. Limitações conhecidas
- Opto-tremolo (LDR) modelado de forma comportamental; assimetria não linear exata do opto-acoplador original pode ser refinada com medição física.
- Falta gravação DI/A-B de um Twin Reverb AB763 físico de 1965.

## 21. Riscos técnicos
- Remover a dupla contagem de mid-scoop (`amp.rs:822`) exige validar equilíbrio tonal com o gabinete `Twin2x12Open` (Spec 080) e atualizar expectativas de teste que assumiam `-4.5 dB` extras.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `003`, `020`, `031`, `039`.
- **Documentos locais:** `docs/amps/voicing-fontes.md`.
- **Código atual:** `crates/ampforge-dsp/src/amp.rs`, `crates/ampforge-dsp/src/tonestack.rs`.
- **Testes existentes / futuros:** `contract_031_authentic_amps.rs`, `contract_039_amps_fidelity.rs`, futuro `contract_056_amp_fender_twin_ab763.rs`.
