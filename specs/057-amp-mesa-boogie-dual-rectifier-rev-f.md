---
spec: 057
titulo: Amplificador Mesa/Boogie Dual Rectifier Solo Head (Rev. F, 1992, 100W — Com Requisitos Pendentes de Esquema Primário Bloqueados)
status: rascunho
tipo: amplificador
atualizado: 2026-11-20
dependencias: [053, 077, 078, 079]
---
# 057 — Amplificador Mesa/Boogie Dual Rectifier Solo Head (Rev. F, 1992)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `amp.mesa.dual_rectifier_rev_f` (aliases legados: `dual_rect`, `cali_dual_recto_revf`, `dual_rectifier`, `rectifier_solo_100`, `recto`)
- **Revisão (`model_revision`):** `1992-rev-f-2ch-solo-100`
- **Categoria:** Amplificador valvulado (`amplificador`)
- **Natureza do módulo:** Equipamento físico real
- **Objetivo:** Especificar a arquitetura documentada do **Mesa/Boogie Dual Rectifier Solo Head Revision F (1992, 2 canais, 100W)** — chaves `Bold / Spongy` (Variac), retificação comutável `Silicon Diodes / Dual 5U4G`, desconexão de NFB global no modo `Red Modern` e cascata high-gain de 4 estágios `12AX7` — **isolando e mantendo explicitamente bloqueados (`BLOCKED`) os parâmetros internos de circuito (`C1`, `R_slope`, `R_bass` e rede `Presence` do canal Red Modern) para os quais existem apenas reconstruções comunitárias divergentes baseadas na Rev G, sem esquema primário oficial autenticado da Rev F**.

## 2. Escopo
- Arquitetura de 2 canais da **Revision F (1992)**: Canal Orange (`Clean` / `Variable High Gain Vintage`) e Canal Red (`Vintage High Gain` / `Modern High Gain`), além do mapeamento explícito do modo `Pushed` (originário do Dual Rectifier de 3 canais de 2000, mantido apenas como extensão documentada de compatibilidade de presets — ver §5).
- Comutação da fonte de alimentação: `Rectifier Select` (`Silicon Diodes` vs `Dual 5U4G`) e `Power (Variac)` (`Bold` 100% AC vs `Spongy` redução de tensão AC ~15–20%).
- Desconexão real da malha de realimentação negativa global (NFB) no modo `Modern` (`eff_nfb = 0`), e registro do problema factual onde o knob `Presence` no canal Red Modern real atua como controle passivo de agudos pré-inversor fora da malha NFB.

## 3. Fora de escopo
- Dual Rectifier Multi-Watt (2010+) ou Road King / Roadster.
- Aprovação definitiva dos valores exatos do Tone Stack e da rede `Presence` da placa `RF-1F` enquanto não houver esquema primário `PRIMARY_OFFICIAL` ou medição direta em chassi Rev F (mantidos `BLOCKED` em §6.2, §18 e §22).

## 4. Equipamento ou componente de referência
- **Fabricante:** Mesa/Boogie Ltd. (Petaluma, California, USA).
- **Modelo exato:** Dual Rectifier Solo Head (100W, 2-Channel).
- **Período / Ano:** 1992–1993 (**Revision F**, placa de pré `RF-1F`).
- **Papel na cadeia de sinal:** Cabeçote valvulado high-gain de 100 W com 5×`12AX7`, 4×`6L6GC` (ou `EL34` via chave de bias traseira) e 2×`5U4G`.

## 5. Revisão e variantes (Auditoria Crítica de Divergências Documentais)
- **Revisão de referência selecionada:** `1992 Revision F (2-Channel Dual Rectifier Solo Head, serial ~R-000506 a ~R-0031xx, PCB RF-1F)`.
- **Diferenças históricas entre Revisões (não misturadas):**
  1. **Rev C / D / E (início de 1992):** primeiras 500 unidades (circuitos derivados do protótipo/SLO, loop serial inicial).
  2. **Rev F (meados de 1992 – final de 1993, selecionada):** 2 canais (`Orange` e `Red`) com chaveamento *Channel Cloning* (`Orange to Modern` / `Red to Vintage`), transformador de saída Schumacher inicial, loop de efeitos serial. **Não possui modo `Pushed` no hardware original.**
  3. **Rev G (1994–2000):** revisão de 2 canais mais numerosa; alterou o loop de efeitos para paralelo e modificou o acoplamento da fonte e dos estágios de ganho. A grande maioria dos esquemas na internet intitulados "Dual Rectifier 2-Channel Schematic" são redesenhos comunitários baseados na **Rev G** (ou híbridos não oficiais Rev F/G).
  4. **3-Channel Dual Rectifier (2000–2009):** redesenhou o amplificador para 3 canais independentes (`Channel 1: Clean / Pushed`, `Channel 2 & 3: Raw / Vintage / Modern`).
- **Resolução documental das 4 divergências obrigatórias (`§5.4`):**
  1. **Origem do modo `Pushed` (`REVISION_MISMATCH`):** O modo `Pushed` **não existe** no Dual Rectifier Rev F de 2 canais (1992); ele pertence ao Canal 1 do **Dual Rectifier de 3 canais (2000)**. No AMPFORGE, `pushed` já faz parte do contrato público da Spec 031 (`CA-5`) e `controls.registry.ts:316`. Portanto, fica classificado explicitamente como **Extensão de Compatibilidade (originária da versão 3-Channel 2000)** e não como característica física da Rev F de 1992.
  2. **Capacitor de Treble `C1` (`250 pF` vs `470 pF` vs `500–680 pF`) e Resistor `R_slope` (`39 kΩ` vs `47 kΩ`):**
     - `docs/amps/voicing-fontes.md:39` declarava `R_slope = 39 kΩ, C1 = 250 pF`;
     - `crates/ampforge-dsp/src/tonestack.rs:94–101 @ c20cc65` implementa `ToneStackTopology::MesaMarkPreGain` com `R_slope = 39 kΩ, C1 = 470 pF` (valores do Soldano SLO-100);
     - Transcrições comunitárias da placa `RF-1F` (`SECONDARY_TECHNICAL`) indicam `R_slope = 47 kΩ` e capacitores de agudos distintos entre o canal Orange (`600 pF` / `680 pF`) e o canal Red (`500 pF` / `680 pF` comutado por LDR).
     - **Decisão de auditoria:** Como não há esquema primário oficial da Mesa/Boogie para a placa `RF-1F` no repositório e as fontes secundárias divergem entre si, a definição final de `C1` e `R_slope` permanece **`BLOCKED` (`PENDING` em CA-7)** até obtenção de esquema primário `RF-1F` verificado ou medição em hardware Rev F.
  3. **Potenciômetro de Graves `R_bass` (`250 kΩ` vs `1 MΩ` vs `25 kΩ`):**
     - `docs/amps/voicing-fontes.md:39` declarava `1 MΩ`; `tonestack.rs:97` implementa `250 kΩ`; transcrições comunitárias `RF-1F` citam `250 kΩ log` (ou `25 kΩ` em anotações parciais). Mantido **`BLOCKED` (`PENDING` em CA-7)** pela mesma ausência de esquema primário `RF-1F`.
  4. **Topologia do `Presence` no modo `Modern` (`FACTUAL_ERROR` no código atual `amp.rs:1151–1164`):**
     - No Dual Rectifier real (confirmado no Owner's Manual oficial da Mesa/Boogie F1), o modo `Red Modern` desconecta a malha de realimentação negativa global (`NFB = 0`), mas o knob **`Presence` do canal Red continua plenamente funcional**, atuando como uma rede passiva variável de atenuação/contorno de altas frequências pós-Tone Stack / pré-Phase Inverter (fora do loop NFB).
     - No código atual (`amp.rs:1151–1164`) e no teste `contract_039_amps_fidelity.rs:300–316`, desconectar o NFB em `Modern` zera o ganho do filtro de `Presence` (`pres_db = 0.0`), tornando o knob `Presence` **inoperante (morto)** no modo padrão (`Modern`) do amplificador!
     - Os valores RC exatos dessa rede passiva de `Presence` do canal Red (`25 kΩ` audio pot + capacitor shunt `3,3 nF` / `4,7 nF`) dependem de confirmação no esquema `RF-1F` (`BLOCKED`).

## 6. Fontes técnicas e evidência por informação

### 6.1 Catálogo de fontes verificadas
| ID | Fonte | Fabricante / Modelo / Revisão | Página / Folha / Seção | URL / Caminho direto | Consulta | Classificação | Limitações da fonte |
|---|---|---|---|---|---|---|---|
| F1 | *Mesa/Boogie Dual Rectifier Solo Head Owner's Manual (2-Channel)* | Mesa/Boogie Ltd. / Dual Rectifier Solo Head | Controles Frontais, Channel Cloning (Orange/Red, Clean/Vintage/Modern), Presence, Bold/Spongy Variac, Silicon Diodes vs 5U4G, Bias Select 6L6/EL34 | Arquivo oficial de manuais Mesa/Boogie (`https://www.mesaboogie.com/support/user-manuals.html (inacessível — redireciona para Gibson)`) | 2026-11-20 | `PRIMARY_OFFICIAL` | Manual de operação oficial; confirma funções de painel, desconexão de NFB em Modern, Bold/Spongy e 5U4G/Diode, mas não publica o esquema elétrico interno `RF-1F` |
| F2 | Rob Robinette — *How the Soldano SLO-100 / Rectifier Preamp Works* | Linhagem Soldano SLO-100 / Dual Rectifier high-gain cascade | Cascata de 4 estágios 12AX7, Cold Clipper `39 kΩ`, Cathode Follower + Tone Stack pós-ganho | `https://robrobinette.com/How_the_Saldano_SLO-100_Works.htm` | 2026-11-20 | `SECONDARY_TECHNICAL` | Analisa a topologia ancestral SLO-100 (`39k / 470pF / 22nF / 22nF`), que não comprova sozinha as alterações específicas da placa Mesa `RF-1F` |
| F3 | Reconstruções comunitárias do esquema `RF-1F` / `Rev G` (Schematic Heaven / Rectifier Guide) | Dual Rectifier Rev F / Rev G (2-Channel) | Preamp & Switching Matrix (`LDR`) | Fontes comunitárias não oficiais | 2026-11-20 | `SECONDARY_TECHNICAL` | **Atenção:** reconstruções comunitárias com divergências entre Rev F e Rev G; insuficientes como prova primária de valores de componentes |
| F4 | Specs 020, 031, 039 e `docs/amps/voicing-fontes.md` | Contratos internos AMPFORGE | `Spec 031 CA-5`, `Spec 039 CA-8`, `voicing-fontes.md` §4 | `specs/031-*.md`, `specs/039-*.md`, `docs/amps/voicing-fontes.md` | 2026-11-20 | `INTERNAL_REFERENCE` | Documenta apenas a implementação atual do software |

### 6.2 Evidência por afirmação técnica
| Informação / Parâmetro de Circuito | Fonte (ID) | Revisão | Página / Folha / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| 2 canais no Rev F original (`Orange` e `Red`) com modos `Clean`, `Vintage High Gain` e `Modern High Gain` | F1 | 2-Channel Solo Head (1992) | Seção *Controls & Channel Cloning* | `PRIMARY_OFFICIAL` | Confirmado |
| Modo `Pushed` pertence ao Canal 1 do Dual Rectifier de **3 canais (2000+)**, não à Rev F de 1992 | F1 | 2ch (1992) vs 3ch (2000) | Comparação de manuais 2ch vs 3ch | `PRIMARY_OFFICIAL` | Confirmado (divergência de revisão documentada) |
| Chave `Power`: `Bold` (tensão AC nominal) vs `Spongy` (Variac interno reduzindo tensão B+ e headroom) | F1 | 2-Channel Solo Head (1992) | Seção *Spongy / Bold Power Switch* | `PRIMARY_OFFICIAL` | Confirmado |
| Chave `Rectifier Select`: `Silicon Diodes` (ataque rápido, baixo sag) vs `Vacuum Tubes (2×5U4G)` (maior sag e compressão dinâmica) | F1 | 2-Channel Solo Head (1992) | Seção *Rectifier Select Switch* | `PRIMARY_OFFICIAL` | Confirmado |
| Desconexão de NFB global no modo `Modern` (e atuação de `Presence` fora do loop NFB no canal Red Modern) | F1, F3 | 2-Channel Solo Head (1992) | Seção *Presence & Modern Mode* | `PRIMARY_OFFICIAL` / `SECONDARY_TECHNICAL` | Função confirmada; valores RC exatos `PENDING` / `BLOCKED` |
| Valores exatos do Tone Stack da placa `RF-1F` (`C1`: `250 pF` vs `470 pF` vs `680 pF`; `R_slope`: `39 kΩ` vs `47 kΩ`; `R_bass`: `250 kΩ` vs `1 MΩ`) | F2, F3, F4 | Placa `RF-1F` (1992) | — | `UNKNOWN` (divergência entre fontes secundárias) | **BLOQUEADO (`BLOCKED`)** — exige esquema primário `RF-1F` |

## 7. Fundamentação elétrica
- **Cascata High-Gain (5×`12AX7` — F1, F2, F3):**
  - `Clean`: 2 estágios de ganho triodo de alto headroom.
  - `Pushed` *(extensão 3-Channel)*: 2–3 estágios com ganho de entrada elevado para transição clean→crunch.
  - `Vintage`: 4 estágios de ganho em cascata (incluindo *cold clipper* intermediário `Rk = 39 kΩ` na topologia SLO/Recto) acionando seguidor de cátodo + Tone Stack pós-ganho, com NFB global conectado no estágio de potência.
  - `Modern`: 4 estágios de ganho com filtragem passa-altas inter-estágio mais firme nos subgraves, ganho elevado e **malha de NFB global desconectada** no estágio de potência `4×6L6GC`.
- **Fonte de Alimentação Dual Rectifier + Variac (`Bold / Spongy`):**
  - `Bold` opera com tensão B+ plena (`~460–470 V`); `Spongy` reduz a tensão primária do transformador de força (efeito Variac), reduzindo B+ em `~15–20%` (`~375–390 V`).
  - `Silicon Diodes` apresenta baixa resistência interna (`R_i` baixa, recuperação rápida `~20 ms`); `Dual 5U4G` apresenta queda de tensão e resistência interna elevadas, gerando sag dinâmico pronunciado e recuperação lenta (`~120 ms`).

## 8. Comportamento esperado
- **Progressão monotônica de saturação entre os modos:** `THD(clean) < THD(pushed) < THD(vintage) <= THD(modern)` com `THD(modern) > 3 × THD(clean)`.
- **Queda real de B+ em `Spongy`:** ativar `spongy = true` reduz `current_b_plus_v()` em `15%..=25%` e antecipa a compressão de pico.
- **Diferença dinâmica `Silicon Diode` vs `5U4GB`:** sob burst transiente forte, `5u4gb` produz queda de B+ (`sag_state`) `>= 2×` maior e tempo de recuperação `>= 2×` mais longo que `diode`.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Origem (`Físico Rev F` vs `Extensão`) | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|---|
| Power | `amp_1.power` | boolean | bool | `false/true` | `true` | discreto | `Físico Rev F` | Alimentação AC | Mute suave `<= 2,5 ms` |
| Standby | `amp_1.standby` | boolean | bool | `false/true` | `false` | discreto | `Físico Rev F` | Alta tensão B+ | Rampa exponencial |
| Channel Mode | `amp_1.channel_mode` | enum | id | `clean, pushed, vintage, modern` | `modern` | discreto | `Clean/Vintage/Modern`: Físico Rev F; `Pushed`: Extensão 3ch (2000) | Comuta cascata de pré e NFB (`modern` → NFB=0) | Comutação de estágios com crossfade 10 ms |
| Gain | `amp_1.gain` | number | scale | `0.0..=10.0` | `6.5` | audio log | `Físico Rev F` | Excitação da cascata de pré | Ganho pré-V1/inter-estágio |
| Treble | `amp_1.treble` | number | scale | `0.0..=10.0` | `6.0` | audio log | `Físico Rev F` | Braço agudo do Tone Stack | KCL 5×5 + bilinear `f64` |
| Mid | `amp_1.middle` | number | scale | `0.0..=10.0` | `5.0` | log | `Físico Rev F` | Braço médio do Tone Stack | KCL 5×5 + bilinear `f64` |
| Bass | `amp_1.bass` | number | scale | `0.0..=10.0` | `5.5` | audio log | `Físico Rev F` | Braço grave do Tone Stack | KCL 5×5 + bilinear `f64` |
| Presence | `amp_1.presence` | number | scale | `0.0..=10.0` | `5.0` | log | `Físico Rev F` | Na malha NFB em Clean/Vintage; filtro passivo pré-PI em Modern (`BLOCKED` RC exato) | Filtro NFB / pré-PI |
| Master | `amp_1.master` | number | scale | `0.0..=10.0` | `5.0` | audio log | `Físico Rev F` | Nível enviado ao LTP e `4×6L6GC` | Ganho pós-tone stack + drive power |
| Power Mode | `amp_1.spongy` | boolean | bool | `false (Bold) / true (Spongy)` | `false` | discreto | `Físico Rev F` | Variac AC (`Spongy` reduz B+ em ~20%) | `variac_scale = 0.80` em B+ |
| Rectifier | `amp_1.rectifier` | enum | id | `diode, 5u4gb` (`gz34` Mod Shop) | `5u4gb` | discreto | `Físico Rev F` | Diodo silício vs `2×5U4G` | Parâmetros de `PowerSupplySag` |

## 10. Contratos de integração
- Implementa `AudioProcessor` (Spec 053) com `module_id = "amp.mesa.dual_rectifier_rev_f"` e `model_revision = "1992-rev-f-2ch-solo-100"`.
- Rejeita `volume_2`, `jumpered`, `cut`, `reverb`, `vibrato_speed`, `vibrato_depth`, `bright` com `E_VALIDATION:`.

## 11. Arquitetura de implementação
- **Código atual mapeado (HEAD `d891389`):** `crates/ampforge-dsp/src/amp.rs` (linhas `339–352, 824–827, 1150–1164, 1252–1281, 1516–1541`) e `crates/ampforge-dsp/src/tonestack.rs` (linhas `94–107`).
- **Problemas e dívidas encontrados no código atual:**
  1. `DOCUMENTATION_PENDING` / `FACTUAL_ERROR` (`tonestack.rs:94–107` vs `docs/amps/voicing-fontes.md:39`): o enum chama-se incorretamente `MesaMarkPreGain` (embora seja usado como pós-ganho no Dual Rectifier) e usa valores `39 kΩ / 470 pF / 250 kΩ` sem comprovação primária na placa `RF-1F`.
  2. `FACTUAL_ERROR` (`amp.rs:1151–1164` e `contract_039_amps_fidelity.rs:298–316`): o código e o teste `test_ca_08` da Spec 039 assumem que com `eff_nfb == 0` no modo `Modern`, `Presence` deve ter efeito zero (`0 dB`), deixando o controle `Presence` morto no modo `Modern`, quando no hardware real o canal Red Modern possui circuito `Presence` ativo fora da malha NFB.
  3. `FACTUAL_ERROR` (`amp.rs:1517–1540`): trocar `channel_mode` comuta instantaneamente de 2 para 4 estágios sem crossfade interno.

## 12. Estratégia DSP
- Cascata de 2 a 4 estágios triodo ADAA1 em `f64` com acoplamento HPF inter-estágio dependente do modo, Tone Stack KCL + bilinear `f64`, chave real de NFB global (`eff_nfb = 0` em `Modern`), sag dinâmico `diode` vs `5u4gb` e escala Variac `Bold`/`Spongy`, dentro de oversampling seletivo (`2x/4x/8x`, Spec 079).

## 13. Requisitos de tempo real
- `alloc_count == 0` e `dealloc_count == 0` em `process()` e na troca de `channel_mode` na thread RT.

## 14. Gestão de estado
- Estados dos 4 estágios ADAA, filtros inter-estágio, Tone Stack, power stage e `sag_state` isolados na struct `MesaDualRectifierRevF`.

## 15. Latência
- Igual à latência do oversampler ativo (Spec 079): `4` (`2x`), `8` (`4x`) ou `16` (`8x`) amostras no modo `MinPhaseIir`.

## 16. Consumo de CPU
- Meta (`ENGINEERING_TARGET`): `<= 11,0%` do orçamento de `128` frames @ `48 kHz` em `2x` OS no modo `Modern` (4 estágios, `--release`).

## 17. Testes
- Senoides `100 Hz`, `1 kHz`, `4,2 kHz`, `5 kHz` em `48 kHz`, `block_size = 128`; burst de `-6 dBFS` para medição de sag `diode` vs `5u4gb` e `Bold` vs `Spongy`.

## 18. Critérios de aceite
- **CA-1 (`DOCUMENTED` — Progressão de Ganho dos Modos de Canal):** Com senoide de `1 kHz` a `-12 dBFS` (`0.2512` pico), `sample_rate = 48000 Hz`, `block_size = 128`, `gain = 6.5`, `master = 5.0`, a THD medida após `2400` amostras satisfaz `THD(clean) < THD(pushed) < THD(vintage) <= THD(modern)` com `THD(modern) > 3.0 * THD(clean)` (`Spec 031 CA-5`).
- **CA-2 (`DOCUMENTED` — Chave Power Bold vs Spongy Variac):** Com `sample_rate = 48000 Hz`, `block_size = 128`, alternar `spongy` de `false` (`Bold`) para `true` (`Spongy`) reduz `current_b_plus_v()` em **`15%..=25%`** (F1) e reduz o nível de pico saturado sem introduzir offset DC (`|DC| < -60 dBFS`).
- **CA-3 (`DOCUMENTED` — Retificador Silicon Diode vs Dual 5U4GB):** Alimentado por um burst senoidal de `1 kHz` a `-6 dBFS` durante `200 ms` seguido de `200 ms` de silêncio (`sample_rate = 48000 Hz`, `block_size = 128`, `sag = 6.0`), `rectifier = "5u4gb"` produz queda máxima de tensão B+ (`sag_state`) `>= 2,0×` maior e tempo de recuperação (para `90%` do B+ nominal) `>= 2,0×` mais longo do que `rectifier = "diode"` (F1).
- **CA-4 (`DOCUMENTED` — Desconexão de NFB Global no Modo Modern):** No modo `channel_mode = "modern"`, a realimentação negativa global efetiva do estágio de potência é `eff_nfb == 0.0`, elevando o ganho de ressonância de carga reativa em `98 Hz` em `>= 1,5 dB` comparado ao modo `"vintage"` com `nfb = 8.0` (F1).
- **CA-5 (`ENGINEERING_TARGET` — Anti-Aliasing em High-Gain 4 Estágios):** Com `channel_mode = "modern"`, `gain = 10.0`, senoide de `5 kHz` a `-6 dBFS`, `sample_rate = 48000 Hz`, `block_size = 128`, DFT Hann de `16384` pontos, o aliasing não-harmônico em `50 Hz–20 kHz` permanece **`< -60 dBFS`** em `4x` e `8x` OS.
- **CA-6 (`ENGINEERING_TARGET` — Troca de Modo Sem Estalo e Segurança RT):** Alternar `channel_mode` de `clean` para `modern` durante senoide de `100 Hz` a `-12 dBFS` (`48 kHz`, `block_size = 128`) mantém derivada máxima amostra-a-amostra `<= 1,5×` a derivada em regime permanente; `CountingAllocator` registra `alloc_count == 0` e `dealloc_count == 0`.
- **CA-7 (`PENDING` — **[BLOQUEADO POR FALTA DE ESQUEMA PRIMÁRIO `RF-1F`]** — Valores Exatos do Tone Stack Rev F e Rede `Presence` fora do NFB em Modern):** A validação definitiva da curva do Tone Stack da Rev F (`C1`: `250 pF` vs `470 pF` vs `680 pF`; `R_slope`: `39 kΩ` vs `47 kΩ`; `R_bass`: `250 kΩ` vs `1 MΩ`) e da função de transferência passiva do controle `Presence` no canal Red `Modern` permanece **`PENDING` / `BLOCKED`** até a incorporação de esquema primário oficial autenticado da placa Mesa `RF-1F` (1992) ou medição física em unidade Rev F.
- **CA-8 (`REGRESSION` — Não-regressão):** Os testes existentes `contract_031_authentic_amps.rs` e `contract_039_amps_fidelity.rs` aplicáveis ao `CaliDualRectoRevF` continuam passando até que o CA-7 seja desbloqueado.

## 19. Dependências
- **Obrigatórias:** `Spec 053`, `Spec 077`, `Spec 078`, `Spec 079`.
- **Relacionadas / Legadas:** `Specs 003, 020, 031, 039`.

## 20. Limitações conhecidas e bloqueios documentais
- **Bloqueio Documental Ativo (`BLOCKED`):** Não existe esquema primário oficial da Mesa/Boogie para a placa `RF-1F` (1992) no repositório. Proibido implementar alterações nos valores de `C1`, `R_slope`, `R_bass` ou na rede `Presence` do modo `Modern` com base em suposições ou redesenhos da Rev G sem antes resolver este bloqueio documental.

## 21. Riscos técnicos
- Desbloquear e corrigir o controle `Presence` no modo `Modern` (CA-7) exigirá atualizar o teste `test_ca_08_presence_and_depth_scale_with_effective_nfb` em `contract_039_amps_fidelity.rs:298–316` (que atualmente testa `Presence == 0 dB` em `Modern`).

## 22. Matriz de rastreabilidade
| Requisito | Fonte | Spec/CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Cascata de 2 a 4 estágios (`Clean, Pushed [ext. 3ch], Vintage, Modern`) | F1, F2 | `Spec 057 / CA-1` | `crates/ampforge-dsp/src/amp.rs:1516–1541 @ c20cc65` | `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs` (`ca_5`) | `TESTED` |
| Chave `Bold / Spongy` (Variac reduzindo tensão B+) | F1 | `Spec 057 / CA-2` | `crates/ampforge-dsp/src/amp.rs:1252–1261 @ c20cc65` | `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs` (`ca_5`) | `TESTED` |
| Chave `Rectifier Select` (`Silicon Diode` vs `Dual 5U4GB`) | F1 | `Spec 057 / CA-3` | `crates/ampforge-dsp/src/modshop.rs:64–95 @ c20cc65`, `amp.rs:344` | `crates/ampforge-dsp/tests/contract_021_workbench_live.rs` | `TESTED` |
| Desconexão de NFB global (`eff_nfb = 0`) no modo `Modern` | F1 | `Spec 057 / CA-4` | `crates/ampforge-dsp/src/amp.rs:1150–1158 @ c20cc65` | `crates/ampforge-dsp/tests/contract_039_amps_fidelity.rs` (`test_ca_08`) | `TESTED` |
| Valores exatos do Tone Stack da placa `RF-1F` (`C1`, `R_slope`, `R_bass`) | F3 (divergente) | `Spec 057 / CA-7` | `crates/ampforge-dsp/src/tonestack.rs:94–107 @ c20cc65` (usa valores SLO `39k/470pF/250k`) | Sem fonte primária `RF-1F` | `BLOCKED` |
| Topologia RC passiva do `Presence` no canal Red `Modern` fora do loop NFB | F1, F3 | `Spec 057 / CA-7` | `crates/ampforge-dsp/src/amp.rs:1162 @ c20cc65` *(anula Presence em Modern)* | Sem esquema primário `RF-1F` | `BLOCKED` |
