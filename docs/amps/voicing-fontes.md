# Fontes de Circuitos e Voicing dos Amplificadores (Spec 039 / Auditoria Corretiva 054–058)

Este documento registra as referências documentais auditadas para a modelagem de tone stack, estágios de ganho, realimentação negativa (NFB) e filtros de voicing dos 5 amplificadores do **SONIX PRO // AMPFORGE™**, confrontando os valores documentados com o código atual (`crates/ampforge-dsp/src/amp.rs` e `tonestack.rs`) e classificando cada fonte conforme a taxonomia oficial (`PRIMARY_OFFICIAL`, `PRIMARY_COMPONENT`, `SECONDARY_TECHNICAL`, `ACADEMIC`, `MEASUREMENT`, `INTERNAL_REFERENCE`, `INFERENCE`, `UNKNOWN`).

## 1. Fender Twin Reverb 1965 (`UsTwinBlackface65` — Spec 056)
- **Esquema de referência:** Fender Twin Reverb Service Diagram / Schematic **AB763** (1963–1967, 85W, 4×6L6GC).
  - Fonte secundária verificada: Rob Robinette, *AB763 Model Differences* (`https://robrobinette.com/AB763_Model_Differences.htm`, `SECONDARY_TECHNICAL`) e *How the AB763 Works* (`https://robrobinette.com/How_The_AB763_Deluxe_Reverb_Works.htm`, `SECONDARY_TECHNICAL`).
- **Tone Stack (`FenderTmbMidScoop`, posicionado entre V1a e V1b):**
  - `R_slope = 100 kΩ`
  - `R_treble = 250 kΩ` (audio taper `250K-A`)
  - `R_bass = 250 kΩ` (audio taper `250K-A`)
  - `R_mid = 10 kΩ` (linear/audio `10K-A`, presente em **ambos** os canais Normal e Vibrato do Twin Reverb AB763)
  - `C1 (Treble) = 250 pF`
  - `C2 (Bass) = 0.1 µF` (`100 nF`)
  - `C3 (Mid) = 0.047 µF` (`47 nF`)
  - `R_load = 1 MΩ` (potenciômetro de Volume com chave `Bright` de `120 pF`)
- **Voicing físico e divergência registrada no código atual (`amp.rs:820–823`):**
  - O recorte natural de médios (~400–500 Hz) já é produzido pela rede passiva AB763 (`C2 = 100 nF`, `C3 = 47 nF`, `ACADEMIC`: Yeh & Smith DAFx-06, `https://ccrma.stanford.edu/~dtyeh/papers/yeh06_dafx.pdf`).
  - Alvo documentado (`INFERENCE`): `high_shelf(4500 Hz, +1.5 dB)` (brilho de acoplamento do canal Vibrato) + `low_shelf(80 Hz, +0.5 dB)` (transformador de saída Schumacher `022855` 85W).
  - **Divergência no código atual (`FACTUAL_ERROR` registrado na Spec 056):** `amp.rs:820–823` aplica `high_shelf(4200.0, 3.2)` e `peaking(450.0, -4.5, 1.0)`, contando o mid-scoop em 450 Hz duas vezes sobre a rede passiva AB763.

## 2. Marshall JCM800 2203 (`BritJcm800_2203` — Spec 055)
- **Fontes de referência:**
  - Painel e controles (`SECONDARY`): *Marshall JCM800 2203 Owners Manual* (`BOOK-00067-00 / 2 / 02`, `docs/ui-desktop/fontes/marshall-jcm800-2203-owners-manual-de.pdf`, pp. 17–18; nota: manual da reedição 2203X com FX Loop traseiro bypassável; o 2203 de 1981 não possui FX loop).
  - Circuito interno 1981 (`SECONDARY_TECHNICAL`): arquivo de esquemas Marshall JMP/JCM800 2203 (`https://drtube.com/marshall/`) e análise técnica de Rob Robinette (`https://robrobinette.com/How_the_Marshall_JCM800_Works.htm`).
- **Tone Stack (`MarshallCathodeFollower`, acionado por V2b Cathode Follower):**
  - `R_slope = 33 kΩ`, `R_treble = 220 kΩ`, `R_bass = 1 MΩ`, `R_mid = 22 kΩ`, `C1 = 470 pF`, `C2 = 22 nF`, `C3 = 22 nF`.
- **Voicing físico e divergência registrada no código atual (`amp.rs:812–815`):**
  - No circuito real, entre o *cold clipper* V1b (`Rk = 10 kΩ` sem bypass) e a grade de V2a existe o divisor `470 kΩ || 470 pF` para `470 kΩ` à terra ($f_z \approx 720{,}8\text{ Hz}$, $f_p \approx 1441{,}6\text{ Hz}$).
  - Alvo aproximado por biquads (`INFERENCE`): `peaking(1600 Hz, +2.0 dB, Q=1.1)` + `low_shelf(110 Hz, -1.5 dB)`.
  - **Divergência no código atual (`ARCHITECTURE_CONFLICT` registrado na Spec 055):** `amp.rs:812–815` implementa `peaking(1650.0, 3.8, 1.15)` e `low_shelf(180.0, -4.2)`. Na implementação da Spec 055, substituir o filtro heurístico pela função de transferência de 1ª ordem exata do divisor `470 kΩ || 470 pF` + `470 kΩ`.

## 3. Marshall 1959 Super Lead / Plexi (`BritPlexi1959` — Spec 054)
- **Fontes de referência:**
  - Painel, topologia e notas técnicas (`PRIMARY_OFFICIAL`): *Marshall 1959HW Owners Manual* (`docs/ui-desktop/fontes/marshall-1959HW-owners-manual.pdf`, pp. 2–7, reedição handwired fiel ao circuito 1969 `SL/A` pré-julho de 1969).
  - Esquemas históricos 1968–1969 (`SECONDARY_TECHNICAL`): arquivo Dr. Tube (`https://drtube.com/marshall/`).
- **Transformadores confirmados no manual oficial (`1959HW Owners Manual`, p. 3 — corrige citação anterior ao Drake 1202-119 de 50W):**
  - **Transformador de saída (Output Transformer):** **Dagnall C1998** (especificação de agosto de 1967, `PRIMARY_OFFICIAL`).
  - **Transformador de força (Mains Transformer):** **Drake 1203-80** (especificação de fevereiro de 1967, `PRIMARY_OFFICIAL`).
- **Tone Stack (`MarshallCathodeFollower`):** `33 kΩ` slope, `470 pF` / `22 nF` / `22 nF`, potenciômetro `Middle` com taper **10% log** (Manual 1959HW p. 3, *Tonal Note 2*).
- **Voicing físico e divergência registrada no código atual (`amp.rs:816–819`):**
  - Rede de mistura dos canais I e II (`470 kΩ || 470 pF` no Canal I High Treble e `470 kΩ` no Canal II Normal) + capacitor extra de bypass de cátodo no 2º estágio de ganho V2a (Manual 1959HW p. 3, *Tonal Note 1*) + NFB retirado diretamente da saída do alto-falante (p. 3, *Tonal Note 3*).
  - Alvo aproximado (`INFERENCE`): `peaking(2400 Hz, +2.5 dB, Q=0.9)` + `low_shelf(120 Hz, +1.5 dB)`.
  - **Divergência no código atual (`ARCHITECTURE_CONFLICT` registrado na Spec 054):** `amp.rs:816–819` implementa `peaking(2500.0, 2.6, 0.90)` e `low_shelf(160.0, 2.8)`.

## 4. Mesa/Boogie Dual Rectifier Rev F (`CaliDualRectoRevF` — Spec 057)
- **Fontes de referência e limitações documentais:**
  - Manual de operação (`UNKNOWN`): *Mesa/Boogie Dual Rectifier Solo Head Owner's Manual* (confirma chaves `Bold/Spongy` variac, `Silicon Diodes / 5U4G`, modos `Orange` e `Red Modern` sem NFB global).
  - Esquema `RF-1F` (`SECONDARY_TECHNICAL` / `UNKNOWN`): reconstruções comunitárias da revisão F de 2 canais (1992) apresentam divergências nos valores do tone stack entre os canais Orange e Red (`R_slope = 39 kΩ` vs `47 kΩ`, `C1 = 250 pF` vs `470 pF` vs `500–680 pF`, `R_bass = 250 kΩ` vs `1 MΩ` vs `25 kΩ`). Os requisitos que dependem desses valores permanecem classificados como `PENDING` / `BLOCKED` na Spec 057 até verificação em esquema primário `RF-1F` autenticado.
- **Tone Stack atual no código (`tonestack.rs:94–107`, variante `MesaMarkPreGain`):** `39 kΩ` slope, `C1 = 470 pF`, `C2 = 22 nF`, `C3 = 22 nF`, `R_treble = 250 kΩ`, `R_bass = 250 kΩ`, `R_mid = 25 kΩ` (`INTERNAL_REFERENCE`).
- **Voicing físico e divergências registradas (`amp.rs:824–827, 1151–1164`):**
  - Alvo documentado (`INFERENCE`): `peaking(85 Hz, +2.0 dB, Q=1.4)` + `peaking(750 Hz, -2.0 dB, Q=1.1)`; código atual (`amp.rs:824–827`) usa `+3.8 dB` em `85 Hz` e `-3.5 dB` em `750 Hz`.
  - **NFB e Presence em Modern (`FACTUAL_ERROR` registrado na Spec 057):** no modo `Modern`, o loop de NFB global é desconectado (`eff_nfb = 0`), mas no equipamento real o potenciômetro `Presence` (`25 kΩ`) do canal Red Modern continua atuando como filtro passivo de agudos pré-inversor fora da malha NFB (enquanto `amp.rs:1162` anula o ganho de `Presence` para `0 dB`).

## 5. Vox AC30/6 Top Boost (`UkAc30TopBoost` — Spec 058)
- **Fontes de referência:**
  - Esquemas históricos JMI `OS/010` (módulo add-on Top Boost, 11/12/1961, John Bell) e `OS/056` / `OS/065` (circuito integrado ao painel de 1964) compilados em *The Vox AC30 Guide — The Top Boost Circuit* (`https://www.voxac30.org.uk/vox_ac30_top_boost_circuit.html`, `SECONDARY_TECHNICAL`).
- **Tone Stack (`VoxTopBoost`):**
  - `R_slope = 100 kΩ`, `C1 = 50 pF` (ou `500 pF` conforme notação de época/sub-variante), `C2 = 22 nF`, `C3 = 22 nF`, `R_bass = 1 MΩ`, `R_treble = 1 MΩ` (no desenho `OS/010`; `220 kΩ` no código atual `tonestack.rs:81`), resistor fixo de cauda `10 kΩ`.
  - **Dívida crítica (`Spec 039 CA-6` / `Spec 058` / `Spec 078`):** `tonestack.rs:79–93` ainda reutiliza a matriz KCL do FMV com `m_pos = 1.0` em vez da função de transferência nodal real da topologia Vox Top Boost.
- **Voicing físico e divergência no código atual (`amp.rs:828–831`):**
  - Alvo documentado (`INFERENCE`): `peaking(3800 Hz, +2.5 dB, Q=1.1)` + `low_shelf(140 Hz, -2.5 dB)`; código atual (`amp.rs:828–831`) usa `peaking(3300.0, 3.5, 1.1)` e `low_shelf(150.0, -3.5)`.
