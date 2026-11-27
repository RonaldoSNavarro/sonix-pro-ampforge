---
spec: 058
titulo: Amplificador Vox AC30/6 Top Boost (Rev. 1964 JMI, 30W 4×EL84)
status: implementada
tipo: amplificador
atualizado: 2026-11-25
dependencias: [053, 077, 078, 079]
---
# 058 — Amplificador Vox AC30/6 Top Boost (Rev. 1964 JMI)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `amp.vox.ac30_6_top_boost` (aliases legados: `ac30_tb`, `uk_ac30_topboost`, `ac30`, `ac30_top_boost`)
- **Revisão (`model_revision`):** `1964-jmi-ac30-6-integrated-tb`
- **Categoria:** Amplificador valvulado (`amplificador`)
- **Natureza do módulo:** Equipamento físico real
- **Objetivo:** Especificar o circuito **JMI Vox AC30/6 Top Boost (1964)** — canais `Normal` e `Brilliant (Top Boost)`, rede tonal **Vox Top Boost dedicada de 2 controles (`Treble` e `Bass`, sem `Middle`)**, controle **Tone Cut** pós-inversor LTP, estágio de potência `4×EL84` cathode-biased (`50 Ω`) sem NFB global (`nfb = 0`) e retificadora `GZ34` — **removendo controles genéricos inexistentes na revisão de 1964 (`Master Volume`, `Middle`, `Presence`, `Depth`) e documentando a distinção entre o kit add-on `OS/010` (1961) e o painel integrado de 1964 (`OS/056` / `OS/065`)**.

## 2. Escopo
- Canais `Normal` (`High` / `Low` −6 dB, 1 estágio `1/2 ECC83`, bypassa totalmente a seção Top Boost) e `Brilliant / Top Boost` (`High` / `Low` −6 dB, estágio `1/2 ECC83` + estágio extra `ECC83` ganho + seguidor de cátodo + rede passiva Top Boost `Treble` / `Bass`), além do oscilador/modulador `Vib-Trem` do AC30/6.
- Equacionamento nodal KCL próprio da rede **Vox Top Boost** (Spec 078), corrigindo a dívida de `tonestack.rs:79–93` (que usava a matriz FMV com `m_pos = 1.0`).
- Circuito **Tone Cut** (`250 kΩ` + `4,7 nF` shunt diferencial entre as duas saídas do inversor LTP antes das grades das `4×EL84`; girar no sentido horário corta agudos).
- Remoção de `Master Volume`, `Middle`, `Presence` e `Depth` do modelo AC30/6 de 1964 (fidelidade estrita à revisão de 1964 prevalecendo sobre a estrutura genérica herdada).

## 3. Fora de escopo
- Vox AC30/4 (1958–1960 com pentodo `EF86` no canal Normal — `EF86` fica disponível apenas como modificação de bancada na Spec 077).
- Reedições modernas **AC30CC2 / AC30C2 (2005+)** que adicionaram `Master Volume` e `Spring Reverb` (nenhum dos dois existe no JMI AC30/6 de 1964).
- Simulação do gabinete 2×12 Open Back Celestion Alnico Blue `T.530` (coberta pela Spec 080).

## 4. Equipamento ou componente de referência
- **Fabricante:** Jennings Musical Industries (JMI) (Dartford Road, Dartford, Kent, England).
- **Modelo exato:** Vox AC30/6 Twin com Top Boost integrado no painel.
- **Período / Ano:** 1964 (integração de fábrica do circuito Top Boost ao painel principal do AC30/6).
- **Papel na cadeia de sinal:** Amplificador valvulado britânico de 30 W sem realimentação negativa global (`NFB = 0`), famoso pelo brilho harmônico (*chime*), interação reativa intensa com falantes Alnico Blue e saturação suave de `4×EL84` + `GZ34`.

## 5. Revisão e variantes (Relação entre `OS/010` Add-On e Top Boost Integrado 1964)
- **Revisão de referência selecionada:** `1964 JMI AC30/6 Top Boost Integrado (chassis OS/065 + seção Top Boost derivada de OS/010 / OS/056, sem Master Volume, sem NFB)`.
- **Distinção histórica obrigatória (`§5.5`):**
  1. **Kit Add-On Traseiro JMI `OS/010` (desenhado por John Bell em 11/12/1961):** módulo opcional montado na traseira (ou painel superior) contendo uma válvula `ECC83` extra (1/2 ganho + 1/2 cathode follower) e os dois potenciômetros **`Treble` (`1 MΩ` log)** e **`Bass` (`1 MΩ` log)**, capacitor de agudos `50 pF` (ou `500 pF` em leituras de código de cores), capacitores de graves `22 nF` / `22 nF`, slope `100 kΩ` e resistor de retorno `10 kΩ`, inserido após o controle de volume do canal `Brilliant`.
  2. **Circuito Integrado ao Painel do AC30/6 (1964, `OS/056` / `OS/065`):** em 1964, a JMI incorporou os controles `Treble` e `Bass` diretamente no painel de controle do AC30/6 ao lado do `Tone Cut`. Enquanto o esquema `OS/010` especifica **`1 MΩ`** para ambos os potenciômetros `Treble` e `Bass`, algumas referências secundárias e softwares (como `tonestack.rs:81` / `voicing-fontes.md:48`) adotaram `R_treble = 220 kΩ` e `R_bass = 1 MΩ`.
  3. **Decisão documental:** A Spec 058 e a Spec 078 documentam explicitamente a topologia nodal `OS/010` com `R_treble = 1 MΩ` (padrão JMI `OS/010` confirmado em F1) e registram a variante `R_treble = 220 kΩ` como pendência de confirmação contra o diagrama de fábrica `OS/056` (`PENDING`).
  4. **Eliminação de controles anacrônicos (`Master Volume`, `Presence`, `Middle`):** O JMI AC30/6 de 1964 **não possui `Master Volume`**, **não possui `Presence`**, **não possui `Depth`** e **não possui `Middle`**. No código atual (`amp.rs:1282–1285` e `1161–1164`), `master_volume` altera o drive de potência do AC30 e `presence`/`depth` aplicam filtros pós-potência. Na implementação da Spec 058, esses controles genéricos devem ser **removidos** do modelo AC30/6 1964, fazendo o drive do inversor LTP e das `4×EL84` depender exclusivamente do `Volume` (`amp_1.gain`) do canal ativo!

## 6. Fontes técnicas e evidência por informação

### 6.1 Catálogo de fontes verificadas
| ID | Fonte | Fabricante / Modelo / Revisão | Página / Folha / Seção | URL / Caminho direto | Consulta | Classificação | Limitações da fonte |
|---|---|---|---|---|---|---|---|
| F1 | *The Vox AC30 Guide — The Top Boost Circuit (JMI Schematics OS/010 & OS/056)* | JMI Vox AC30 Top Boost (1961–1964) | Análise completa do desenho `OS/010` (11/12/1961, John Bell) e integração no AC30/6 | `https://www.voxac30.org.uk/vox_ac30_top_boost_circuit.html` | 2026-11-20 | `SECONDARY_TECHNICAL` | Transcrição técnica detalhada do esquema JMI `OS/010`; página HTTP verificada (14.668 bytes) |
| F2 | Duncan Amps — *Tone Stack Calculator (TSC) — Vox Tone Stack Topology* | Rede passiva Vox Top Boost | Equações nodais da topologia Vox (distinta do FMV) | `https://www.duncanamps.com/tsc/` | 2026-11-20 | `SECONDARY_TECHNICAL` | Referência de topologia de circuito da rede Vox de 2 potenciômetros |
| F3 | Mullard — *EL84, ECC83 & GZ34 (5AR4) Vacuum Tube Datasheets* | Válvulas padrão do JMI AC30/6 | Operação push-pull cathode-biased `EL84`, triodo `ECC83` e retificadora `GZ34` | Acervo de datasheets Mullard | 2026-11-20 | `PRIMARY_COMPONENT` | Dados oficiais das válvulas |
| F4 | Specs 020, 031, 039, 051 e `docs/amps/voicing-fontes.md` | Contratos internos AMPFORGE | `Spec 031 CA-3`, `Spec 039 Contexto #6 e CA-6`, `Spec 051 T-04` | `specs/031-*.md`, `specs/039-*.md`, `docs/amps/voicing-fontes.md` | 2026-11-20 | `INTERNAL_REFERENCE` | Registra a dívida da matriz FMV usada no lugar da rede Top Boost |

### 6.2 Evidência por afirmação técnica
| Informação / Parâmetro de Circuito | Fonte (ID) | Revisão | Página / Folha / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Seção Top Boost (`OS/010`): 1×`ECC83` extra (V2a ganho `Rp=100 kΩ, Rk=1,5 kΩ` + V2b cathode follower `Rk=56–100 kΩ`) inserida no canal Brilliant | F1 | JMI `OS/010` (1961/1964) | Seção *Circuit Description* | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| Rede tonal Top Boost de 2 knobs (`Treble` e `Bass`, sem `Middle`), `R_slope = 100 kΩ`, `C1 = 50 pF`, `C2 = 22 nF`, `C3 = 22 nF`, resistor de retorno `10 kΩ`, pots `1 MΩ` (`OS/010`) vs `220 kΩ` (`Treble` variante) | F1, F2 | JMI `OS/010` vs `OS/056` | Seção *Tone Controls* | `SECONDARY_TECHNICAL` | Topologia confirmada; variante `1 MΩ` vs `220 kΩ` `PENDING` |
| Canal `Normal` não passa pelo estágio nem pela rede tonal Top Boost | F1 | JMI AC30/6 (`OS/065`) | Seção *Normal vs Brilliant* | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| Controle `Tone Cut` (`250 kΩ` pot + `0,0047 µF = 4,7 nF` cap) posicionado entre as duas fases de saída do inversor LTP antes das `4×EL84` | F1 | JMI AC30/6 (`OS/065`) | Seção *Phase Inverter & Tone Cut* | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| Estágio de potência `4×EL84` com bias de cátodo compartilhado (`50 Ω` + bypass eletrolítico), retificadora `GZ34` e **ausência de NFB global (`NFB = 0`)** | F1, F3 | JMI AC30/6 (`OS/065`) | Power Stage / Mullard EL84 | `SECONDARY_TECHNICAL` / `PRIMARY_COMPONENT` | Confirmado |
| Ausência de `Master Volume`, `Presence` e `Middle` na revisão JMI de 1964 | F1 | JMI AC30/6 1964 | Painel AC30/6 Top Boost | `SECONDARY_TECHNICAL` | Confirmado |

## 7. Fundamentação elétrica
- **Diferença estrutural da Rede Tonal Vox Top Boost vs. FMV (F1, F2):**
  - Na rede FMV (Fender/Marshall), o capacitor `C3` liga a junção do resistor `R_slope` ao potenciômetro `Middle` para terra, com o potenciômetro `Bass` em série entre `C2` e o cursor/topo do `Middle`.
  - Na rede **Vox Top Boost (`OS/010`)**, não existe potenciômetro `Middle`: o potenciômetro `Bass` (`1 MΩ` log) está ligado como reostato/divisor entre os capacitores `C2 = 22 nF` e `C3 = 22 nF` com um resistor fixo de `10 kΩ` à terra, fazendo com que a posição dos cursores de `Bass` e `Treble` (`C1 = 50 pF`, `R_slope = 100 kΩ`) altere simultaneamente a profundidade e a frequência do recorte de médios (~800–1200 Hz).
  - Usar a matriz FMV com `m_pos = 1.0` (`tonestack.rs:92`) **não reproduz** a ligação elétrica do potenciômetro `Bass` entre `C2` e `C3` do esquema `OS/010`.
- **Circuito `Tone Cut` pós-LTP:** posicionado entre os dois ânodos do inversor de fase LTP (`ECC83`), o capacitor de `4,7 nF` em série com o potenciômetro de `250 kΩ` forma um filtro passa-baixas diferencial variável: à medida que a resistência do potenciômetro diminui (sentido horário, `cut: 0 → 10`), as altas frequências dos dois braços em oposição de fase (`180°`) se anulam mutuamente antes de excitar as grades das `4×EL84`.

## 8. Comportamento esperado
- **Rede Top Boost autêntica:** ativa somente no canal `Brilliant / Top Boost` (`brilliant_high`, `brilliant_low`, `top_boost`); no canal `Normal` (`normal_high`, `normal_low`), mover `treble` ou `bass` de `0` para `10` produz variação `<= 0,1 dB`.
- **Ação real do `Tone Cut`:** variar `amp_1.cut` de `0.0` para `10.0` atenua frequências acima de `2 kHz` em **`>= 6,0 dB`**, independentemente do canal selecionado (`Normal` ou `Top Boost`), pois o `Tone Cut` fica após o inversor de fase comum!
- **Sem Master Volume, Presence ou Depth:** o nível de saturação das `4×EL84` depende exclusivamente de `amp_1.gain` (`Volume`); `amp_1.master`, `amp_1.middle`, `amp_1.presence` e `amp_1.depth` são rejeitados para este modelo.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Origem (`Físico 1964` vs `Mod Shop`) | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|---|
| Power | `amp_1.power` | boolean | bool | `false/true` | `true` | discreto | `Físico 1964` | Alimentação AC | Mute suave `<= 2,5 ms` |
| Standby | `amp_1.standby` | boolean | bool | `false/true` | `false` | discreto | `Físico 1964` | Alta tensão B+ | Rampa exponencial |
| Canal / Entrada | `amp_1.input_jack` | enum | id | `normal_high, normal_low, brilliant_high, brilliant_low, top_boost` | `brilliant_high` | discreto | `Físico 1964` | Seleciona canal Normal vs Brilliant (Top Boost) e High vs Low (−6 dB) | Bypass do módulo Top Boost em Normal + pad `-6,02 dB` |
| Volume (Brilliant / Top Boost) | `amp_1.gain` | number | scale | `0.0..=10.0` | `5.5` | audio 25% | `Físico 1964` | Volume do canal Brilliant/Top Boost (`BRILL VOL`) | Ganho de canal Brilliant + drive direto do power amp |
| Normal Volume | `amp_1.volume_2` | number | scale | `0.0..=10.0` | `5.0` | audio 25% | `Físico 1964` | Volume independente do canal Normal (`NORMAL VOL` no AC30/6) | Ganho do canal Normal (bypass Top Boost) |
| Treble | `amp_1.treble` | number | scale | `0.0..=10.0` | `6.0` | audio log | `Físico 1964` | Braço agudo `50 pF` da rede Top Boost | Solver nodal dedicado Vox Top Boost `f64` (Spec 078) |
| Bass | `amp_1.bass` | number | scale | `0.0..=10.0` | `5.5` | audio log | `Físico 1964` | Braço grave `22 nF / 22 nF` da rede Top Boost | Solver nodal dedicado Vox Top Boost `f64` (Spec 078) |
| Tone Cut | `amp_1.cut` | number | scale | `0.0..=10.0` | `5.0` | log invertido | `Físico 1964` | Shunt RC diferencial `4,7 nF + 250 kΩ` pós-LTP (horário corta agudos) | LPF diferencial `f64` pós-PI (`18 kHz → 750 Hz`) |
| Vib-Trem Speed | `amp_1.vibrato_speed` | number | scale | `0.0..=10.0` | `4.0` | linear/exp | `Físico 1964` | Velocidade do oscilador Vib-Trem do AC30/6 | Frequência LFO invariante à taxa |
| Vib-Trem Depth | `amp_1.vibrato_depth` | number | scale | `0.0..=10.0` | `0.0` | log | `Físico 1964` | Intensidade da modulação Vib-Trem | Modulação quando `> 0` |

## 10. Contratos de integração
- Implementa `AudioProcessor` (Spec 053) com `module_id = "amp.vox.ac30_6_top_boost"` e `model_revision = "1964-jmi-ac30-6-integrated-tb"`.
- Conciliação (2026-11-25): aceita `amp_1.volume_2` (`NORMAL VOL`) para o canal Normal e `amp_1.gain` (`BRILL VOL`) para o canal Brilliant/Top Boost (alinhado ao AC30/6 real e `AmpHead.ts`), e ao selecionar `ac30_tb` sem sobrescritas explícitas pareia automaticamente `cab_1.speaker = "alnico_blue_2x12"`.
- Rejeita `amp_1.master`, `amp_1.middle`, `amp_1.presence`, `amp_1.depth`, `amp_1.reverb`, `amp_1.jumpered`, `amp_1.spongy` e `amp_1.channel_mode` com `E_VALIDATION:` (fidelidade estrita à revisão de 1964).

## 11. Arquitetura de implementação
- **Código atual mapeado (HEAD `d891389`):** `crates/ampforge-dsp/src/amp.rs` (linhas `353–366, 828–831, 1161–1164, 1211, 1282–1285, 1345–1347, 1542–1553, 1590–1595`) e `crates/ampforge-dsp/src/tonestack.rs` (linhas `79–93`).
- **Problemas e dívidas encontrados no código atual:**
  1. `FACTUAL_ERROR` (`tonestack.rs:79–93`): `ToneStackTopology::VoxTopBoost` usa a matriz KCL do FMV com `m_pos = 1.0` em vez da função de transferência nodal real da topologia Vox Top Boost `OS/010`.
  2. `REVISION_MISMATCH` / `FACTUAL_ERROR` (`amp.rs:1282–1285` e `api.rs`): o código atual inclui `master_volume` na equação de drive de potência do AC30 (`1.2 + master_pot * 5.0 + (gain / 10.0) * 1.6`), e mantém `presence_filter`, `depth_filter` e `overdrive` (`+18 dB`) ativos no AC30/6 de 1964! Na implementação da Spec 058, o drive das `4×EL84` deve vir exclusivamente do `Volume` (`gain`), sem `master`, `presence`, `depth` ou `overdrive`.

## 12. Estratégia DSP
- **Circuit-level (`f64`):** solver KCL dedicado da rede **Vox Top Boost `OS/010`** discretizado por transformada bilinear em `f64` (Spec 078); filtro `Tone Cut` em `f64` pós-LTP.
- **Behavioral calibrado com memória (`f64`):** V1 (`1/2 ECC83`) + (no canal Top Boost: V2a `ECC83` + V2b Cathode Follower) → LTP → `Tone Cut` → `4×EL84` cathode-biased (`50 Ω`, `nfb = 0.0`) + sag de `GZ34` (`tau_att = 30 ms`, `tau_rel = 80 ms`).
- **Oversampling:** seletivo (`2x` padrão, `4x`/`8x` selecionável — Spec 079).

## 13. Requisitos de tempo real
- `alloc_count == 0` e `dealloc_count == 0` em `process()` na thread RT; coeficientes derivados de `sample_rate` em `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- Estados de V1, V2a/b, `VoxTopBoostNodalNetwork`, `ac30_cut_lp`, `lfo_phase`, `power_stage` e `sag_state` isolados na struct `VoxAc30TopBoost1964`.

## 15. Latência
- Igual à latência do oversampler ativo (Spec 079): `4` (`2x`), `8` (`4x`) ou `16` (`8x`) amostras no modo `MinPhaseIir`.

## 16. Consumo de CPU
- Meta (`ENGINEERING_TARGET`): `<= 8,0%` do orçamento de `128` frames @ `48 kHz` em `2x` OS (`--release`).

## 17. Testes
- Sinais determinísticos em `44,1`, `48` e `96 kHz`, `block_size = 128`: impulso unitário e sweep `50 Hz–10 kHz` para a rede Top Boost `OS/010`; senoides `500 Hz`, `2 kHz`, `4 kHz`, `5 kHz` para `Tone Cut` e aliasing.

## 18. Critérios de aceite
- **CA-1 (`ANALYTICAL` — Rede Tonal Vox Top Boost Dedicada `OS/010`):** Com impulso unitário (`x[0] = 1.0`), `sample_rate = 48000 Hz` (`8x` OS), `block_size = 128`, a função de transferência do Tone Stack utiliza a matriz KCL dedicada da topologia Vox Top Boost `OS/010` (distinta da matriz FMV) e sua magnitude DFT entre `50 Hz` e `10 kHz` coincide com a curva analítica `OS/010` dentro de `±0,5 dB` para `(Bass, Treble) ∈ {(5,5), (10,0), (0,10)}` (F1, F2).
- **CA-2 (`DOCUMENTED` — Ação Real do Tone Cut Pós-LTP):** Com senoide de `4 kHz` a `-20 dBFS`, `sample_rate = 48000 Hz`, `block_size = 128`, variar `amp_1.cut` de `0.0` para `10.0` reduz o nível RMS de saída acima de `2 kHz` em **`>= 6,0 dB`** em ambos os canais `normal_high` e `brilliant_high`, com diferença `<= 0,5 dB` entre `44,1`, `48` e `96 kHz` (F1; `Spec 031 CA-3`).
- **CA-3 (`DOCUMENTED` — Isolamento do Tone Stack no Canal Normal):** Com sweep/senoides em `150 Hz` e `4 kHz` a `-24 dBFS`, `sample_rate = 48000 Hz`, `block_size = 128`: em `input_jack = "normal_high"`, variar `treble` (`0→10`) ou `bass` (`0→10`) altera a saída em `<= 0,1 dB`; em `input_jack = "brilliant_high"` (ou `"top_boost"`), a variação excede `>= 8,0 dB` (F1).
- **CA-4 (`DOCUMENTED` — Ausência de NFB, Sag GZ34 e Rejeição de Controles Anacrônicos 1964):** O modelo inicia com `nfb = 0.0` e `rectifier = "gz34"`; os controles inexistentes na revisão JMI de 1964 (`amp_1.master`, `amp_1.middle`, `amp_1.presence`, `amp_1.depth`, `amp_1.reverb`, `amp_1.spongy`) não têm efeito oculto na saída (`diferença <= 1e-6`) e são rejeitados com HTTP 400 (`E_VALIDATION:`) (F1).
- **CA-5 (`ENGINEERING_TARGET` — Anti-Aliasing e Segurança RT):** Com senoide de `5 kHz` a `-6 dBFS`, `gain = 10.0`, `sample_rate = 48000 Hz`, `block_size = 128`, DFT Hann de `16384` pontos, o aliasing não-harmônico em `50 Hz–20 kHz` permanece **`< -60 dBFS`** em `4x` e `8x` OS; `CountingAllocator` na thread RT registra `alloc_count == 0` e `dealloc_count == 0`.
- **CA-6 (`REGRESSION` — Não-regressão):** Os testes existentes `contract_031_authentic_amps.rs`, `contract_039_amps_fidelity.rs` e `contract_051_manual_test_fixes.rs` (T-04 `top_boost`) aplicáveis ao `UkAc30TopBoost` continuam passando.

## 19. Dependências
- **Obrigatórias:** `Spec 053`, `Spec 077`, `Spec 078`, `Spec 079`.
- **Relacionadas / Legadas:** `Specs 003, 020, 031, 039, 051`.

## 20. Limitações conhecidas
- Variante do valor do potenciômetro `Treble` (`1 MΩ` no desenho `OS/010` vs `220 kΩ` em referências posteriores) permanece registrada para confirmação em diagrama de fábrica `OS/056` de 1964.

## 21. Riscos técnicos
- Remover `amp_1.master` do painel do AC30/6 de 1964 exige atualizar `controls.registry.ts` e presets que enviem `amp_1.master` quando o AC30 estiver selecionado.

## 22. Matriz de rastreabilidade
| Requisito | Fonte | Spec/CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Rede tonal Vox Top Boost dedicada (`OS/010`, sem matriz FMV) | F1, F2 | `Spec 058 / CA-1` | `crates/ampforge-dsp/src/tonestack.rs:79–93 @ c20cc65` *(dívida: usa matriz FMV com `m_pos=1.0`)* | Planejado: `contract_058_amp_vox_ac30_top_boost.rs` | `SPECIFIED` |
| Controle `Tone Cut` pós-LTP (`>= 6 dB` de corte > 2 kHz) | F1 | `Spec 058 / CA-2` | `crates/ampforge-dsp/src/amp.rs:1345–1347 @ c20cc65, 1590–1595` | `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs` (`ca_3`) | `TESTED` |
| Bypass da rede Top Boost no canal `Normal` e ativação em `Brilliant / top_boost` | F1, F4 | `Spec 058 / CA-3` | `crates/ampforge-dsp/src/amp.rs:1542–1565 @ c20cc65` | `crates/sonix-desktop/tests/contract_051_manual_test_fixes.rs` | `TESTED` |
| Ausência de NFB (`nfb = 0`) e retificação `GZ34` por padrão | F1, F3 | `Spec 058 / CA-4` | `crates/ampforge-dsp/src/amp.rs:353–366 @ c20cc65` | `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs` | `TESTED` |
| Remoção de `Master Volume`, `Presence` e `Depth` do modelo JMI 1964 | F1 | `Spec 058 / CA-4` | `crates/ampforge-dsp/src/amp.rs:1161–1164 @ c20cc65, 1282–1285` *(dívida: ainda ativos)* | Planejado: `contract_058_amp_vox_ac30_top_boost.rs` | `SPECIFIED` |
| Confirmação de `R_treble` (`1 MΩ` em `OS/010` vs `220 kΩ` em `OS/056`) | F1 | `Spec 058 / §5, §20` | `crates/ampforge-dsp/src/tonestack.rs:81 @ c20cc65` (`220 kΩ`) | Sem cópia primária de `OS/056` | `PENDING` |
