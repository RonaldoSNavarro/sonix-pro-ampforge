---
spec: 054
titulo: Amplificador Marshall 1959 Super Lead / Plexi (Rev. 1969 SL/A Pré-Julho 1969, 100W)
status: implementada
tipo: amplificador
atualizado: 2026-11-25
dependencias: [053, 077, 078, 079]
---
# 054 — Amplificador Marshall 1959 Super Lead / Plexi (Rev. 1969 SL/A)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `amp.marshall.1959_super_lead` (aliases legados: `plexi_1959`, `brit_plexi_1959`, `plexi`)
- **Revisão (`model_revision`):** `1969-jmp-sl-a-pre-july`
- **Categoria:** Amplificador valvulado (`amplificador`)
- **Natureza do módulo:** Equipamento físico real
- **Objetivo:** Especificar o circuito elétrico real, os controles autênticos, o comportamento dinâmico e os critérios de validação do amplificador **Marshall Model 1959 Super Lead 100W ("Plexi", revisão SL/A pré-julho de 1969 documentada no manual oficial 1959HW e esquemas JMP 1968–1969)**, corrigindo acoplamentos e erros factuais do código atual.

## 2. Escopo
- Modelagem dos dois canais paralelos independentes: **Canal I (High Treble)** e **Canal II (Normal)**, com 4 jacks de entrada (`High I`, `Low I`, `High II`, `Low II`) e ligação em ponte `Jumpered` (`High I` + cabo patch `Low I → High II`).
- Estágios V1b (Canal I, split-cathode `Rk = 2,7 kΩ || Ck = 0,68 µF`, bright cap `0,005 µF = 5000 pF` **exclusivo sobre Volume I**) e V1a (Canal II, `Rk = 820 Ω || Ck = 250–330 µF`, sem bright cap).
- Rede passiva de mistura (`470 kΩ || 470 pF` no Canal I + `470 kΩ` no Canal II), 2º estágio de ganho comum V2a (`Rk = 820 Ω` com capacitor de bypass adicional `0,68 µF` conforme *Tonal Note 1* do manual 1959HW p. 3) acoplado diretamente ao seguidor de cátodo V2b (`Rk = 100 kΩ`).
- Tone Stack passivo FMV Marshall (`R_slope = 33 kΩ`, `C1 = 470 pF`, `C2 = 22 nF`, `C3 = 22 nF`, `R_treble = 220 kΩ`, `R_bass = 1 MΩ`, `R_mid = 22–25 kΩ` com **taper 10% log** confirmado na *Tonal Note 2* do manual 1959HW p. 3).
- Inversor de fase LTP (V3 `ECC83`), estágio de potência push-pull `4×EL34` fixed-bias, transformador de saída **Dagnall C1998** (agosto 1967) e transformador de força **Drake 1203-80** (fevereiro 1967) com NFB global retirado **diretamente da saída do alto-falante** (*Tonal Note 3*, p. 3).

## 3. Fora de escopo
- Variantes 1965–1967 JTM100 / early 1959 shared-cathode (`Rk = 820 Ω / 250–330 µF` compartilhado em V1 e bright cap de `100 pF` — registrado em §5).
- Reedição padrão **Marshall 1959SLP** (que utiliza potenciômetro `Middle` linear, NFB fixo no tap de `8 Ω` e FX Loop traseiro — Manual 1959HW pp. 2–3).
- Master Volume (inexistente no 1959 original) e gabinete 4×12 Greenback (coberto pela Spec 080).

## 4. Equipamento ou componente de referência
- **Fabricante:** Marshall Amplification plc (Bletchley, Milton Keynes, England).
- **Modelo exato:** Model 1959 Super Lead 100W Head (e reedição handwired 1959HW baseada na unidade de referência SL/A pré-julho de 1969).
- **Período / Ano:** 1968 – junho de 1969 (painel Plexiglas dourado, circuito `SL/A`).
- **Papel na cadeia de sinal:** Cabeçote valvulado non-master-volume de 100 W RMS; a saturação provém da interação entre os canais paralelos, o seguidor de cátodo V2b, o inversor LTP V3 e o quarteto de potência `4×EL34`.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1969 SL/A pré-julho de 1969 (split-cathode V1, bright cap 0,005 µF em Volume I, bypass extra em V2a, Mid pot 10% log, NFB na saída do falante, OT Dagnall C1998, PT Drake 1203-80)`.
- **Relação documentada entre 1959 original (1965–69), 1959SLP e 1959HW (conferida em F1, pp. 2–5):**
  1. **1959 Original (final de 1965 – 1967):** shared cathode em V1 (`820 Ω || 250–330 µF`), bright cap de **`100 pF`** sobre Volume I (F1, p. 5, *Tonal Note 5*), filtragem B+ menor.
  2. **1959 Original SL/A (1968 – pré-julho de 1969, revisão selecionada) & 1959HW Reissue:** split cathode em V1, bright cap de **`0,005 µF` (`5000 pF`)** sobre Volume I (F1, p. 5, *Tonal Note 5*), capacitor extra de bypass de cátodo no 2º estágio de ganho V2a elevando médios-agudos e agudos (F1, p. 3, *Tonal Note 1*), potenciômetro `Middle` **10% log** (F1, p. 3, *Tonal Note 2*), NFB retirado da saída real do alto-falante variando o amortecimento conforme a chave `4 / 8 / 16 Ω` (F1, p. 3, *Tonal Note 3*), transformador de saída **Dagnall C1998** (spec agosto 1967) e transformador de força **Drake 1203-80** (spec fevereiro 1967) (F1, p. 3).
  3. **1959SLP Reissue (produção padrão posterior):** potenciômetro `Middle` **linear** (F1, p. 3, *Tonal Note 2*) e NFB fixo no tap de **`8 Ω`** do transformador de saída, tornando o fator de amortecimento independente da chave seletora `4 / 8 / 16 Ω` (F1, p. 3, *Tonal Note 3*).
- **Divergências históricas resolvidas vs. mantidas pendentes:**
  - **Resolvida (Transformadores):** corrigido o erro factual anterior que citava "Drake 1202-119" (transformador de 50W do modelo 1987); o manual primário F1 (p. 3) comprova que o 1959 100W utiliza OT **Dagnall C1998** e PT **Drake 1203-80**.
  - **Resolvida (`Rk(V1b)` na revisão SL/A 1969):** fixado como **`2,7 kΩ || 0,68 µF`** para a revisão `1969-jmp-sl-a-pre-july` (`SECONDARY_TECHNICAL`: F2); a sub-variante transicional de início de 1968 (`Rk(V1b) = 820 Ω || 0,68 µF`) não é misturada na configuração padrão e fica registrada como variante histórica separada.
  - **Pendente (`PENDING` / `UNKNOWN`):** curva de magnetização B-H não linear e indutância de dispersão primária-secundária exata do transformador de saída **Dagnall C1998** dependem de medição física em bancada.

## 6. Fontes técnicas e evidência por informação

### 6.1 Catálogo de fontes verificadas
| ID | Fonte | Fabricante / Modelo / Revisão | Página / Folha / Seção | URL / Caminho direto | Consulta | Classificação | Limitações da fonte |
|---|---|---|---|---|---|---|---|
| F1 | *Marshall 1959HW — 100 Watt Head Owners Manual* | Marshall Amplification / 1959HW (circuito 1969 SL/A pré-julho 1969) | pp. 1–7 (Introduction, Technical Information, Tonal Notes 1–5, Front/Rear Panel itens 1–13, Figs. 1–2) | `docs/ui-desktop/fontes/marshall-1959HW-owners-manual.pdf` | 2026-11-20 | `PRIMARY_OFFICIAL` | Descreve topologia, transformadores (C1998 / 1203-80), bright cap 0,005 µF, Mid 10% log, NFB e entradas (-6 dB), mas não traz diagrama esquemático completo com todos os resistores de placa/cátodo |
| F2 | Dr. Tube — *Marshall 1959 Super Lead Schematics Archive (1967–1969)* | Marshall Model 1959 100W Super Lead | Esquemas SL/A 100W Lead (V1, V2, V3, 4×EL34) | `https://drtube.com/marshall/` | 2026-11-20 | `SECONDARY_TECHNICAL` | Acervo de esquemas históricos digitalizados; valores internos sujeitos a variações de sub-lote (1968 vs 1969) |
| F3 | Rob Robinette — *How the Marshall JCM800 / Plexi Preamp & Tone Stack Work* | Marshall 1959 / 2203 | Seções Preamp, Cathode Follower, Tone Stack, LTP | `https://robrobinette.com/How_the_Marshall_JCM800_Works.htm` e `https://robrobinette.com/How_The_TMB_Tone_Stack_Works.htm` | 2026-11-20 | `SECONDARY_TECHNICAL` | Análise técnica independente (não oficial da Marshall) |
| F4 | D. T. Yeh & J. O. Smith — *Discretization of the '59 Fender Bassman Tone Stack* (DAFx-06) | Rede passiva FMV 3ª ordem | pp. 1–8 (Equações KCL e transformada bilinear) | `https://ccrma.stanford.edu/~dtyeh/papers/yeh06_dafx.pdf` | 2026-11-20 | `ACADEMIC` | Fundamenta a discretização matemática exata da rede FMV |
| F5 | Mullard / Philips — *ECC83 (12AX7) & EL34 Vacuum Tube Datasheets* | Mullard `ECC83` e `EL34` | Curvas de transferência e características estáticas | Acervo de datasheets Mullard (`µ=100` ECC83; push-pull AB1 EL34) | 2026-11-20 | `PRIMARY_COMPONENT` | Parâmetros nominais de componente isolado |
| F6 | Specs 003, 020, 031, 039 e `docs/amps/voicing-fontes.md` | Contratos internos AMPFORGE | `Spec 031 CA-1..CA-2`, `Spec 039 CA-1..CA-4, CA-9` | `specs/031-*.md`, `specs/039-*.md`, `docs/amps/voicing-fontes.md` | 2026-11-20 | `INTERNAL_REFERENCE` | Documenta requisitos internos e estado atual do código |

### 6.2 Evidência por afirmação técnica
| Informação / Parâmetro de Circuito | Fonte (ID) | Revisão | Página / Folha / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| 3× `ECC83` (V1, V2, V3) no pré/PI + 4× `EL34` em push-pull Classe AB (100W RMS) | F1 | 1969 SL/A (1959HW) | p. 2 (Intro) e p. 3 (*Valve compliment*) | `PRIMARY_OFFICIAL` | Confirmado |
| V1 dividido em 2 canais paralelos (Ch I High Treble e Ch II Normal) somados passivamente antes de V2a; V2b como Cathode Follower acionando o Tone Stack | F1 | 1969 SL/A (1959HW) | p. 3 (*Technical Information*) | `PRIMARY_OFFICIAL` | Confirmado |
| Capacitor extra de bypass de cátodo no 2º estágio de ganho (V2a) reforçando médios-agudos e agudos | F1 | 1969 SL/A (1959HW) | p. 3 (*Tonal Note 1*) | `PRIMARY_OFFICIAL` | Confirmado |
| Potenciômetro `Middle` com curva **10% log** (diferente do linear da 1959SLP) | F1 | 1969 SL/A (1959HW) | p. 3 (*Tonal Note 2*) | `PRIMARY_OFFICIAL` | Confirmado |
| Transformador de saída **Dagnall C1998** (spec Ago/1967) e transformador de força **Drake 1203-80** (spec Fev/1967) | F1 | 1969 SL/A (1959HW) | p. 3 (*Output Transformer* / *Mains Transformer*) | `PRIMARY_OFFICIAL` | Confirmado |
| NFB retirado da saída real do alto-falante (4 Ω mais solto/ressonante, 16 Ω mais firme/amortecido) | F1 | 1969 SL/A (1959HW) | p. 3 (*Tonal Note 3*) | `PRIMARY_OFFICIAL` | Confirmado |
| Capacitor de filtro B+ principal `50 µF` single-can + ponte de 4 diodos (com `47 nF/630V` EMC sem impacto sônico) | F1 | 1969 SL/A (1959HW) | p. 3 (*Components* / *Note*) | `PRIMARY_OFFICIAL` | Confirmado |
| Bright cap de **`0,005 µF` (`5000 pF`)** sobre `Volume I` (vs `100 pF` em 1965–1967) | F1 | 1969 SL/A (1959HW) | p. 5 (*Item 8 — Tonal Note 5*) | `PRIMARY_OFFICIAL` | Confirmado |
| Entradas `Low Sensitivity` (Itens 11 e 13) são **6 dB mais baixas** e mais escuras por menor impedância de entrada; ligação `Jumpered` em fase (Figs. 1–2) | F1 | 1969 SL/A (1959HW) | pp. 5–6 (*Itens 10–13*, *Performance Note*, Figs. 1–2) | `PRIMARY_OFFICIAL` | Confirmado |
| Valores internos de pré: V1b `Rk = 2,7 kΩ || 0,68 µF`, acoplamento `2,2 nF`; V1a `Rk = 820 Ω || 250–330 µF`, acoplamento `22 nF`; rede de mistura `470 kΩ || 470 pF` + `470 kΩ` | F2, F3 | 1968–1969 JMP 1959 | Folha 1 do esquema 1959 SL/A | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| Valores da rede Tone Stack FMV: `R_slope = 33 kΩ`, `C1 = 470 pF`, `C2 = 22 nF`, `C3 = 22 nF`, `R_treble = 220 kΩ`, `R_bass = 1 MΩ`, `R_mid = 22–25 kΩ` | F2, F3, F4 | 1968–1969 JMP 1959 | Folha 1 do esquema / DAFx-06 | `SECONDARY_TECHNICAL` / `ACADEMIC` | Confirmado em fonte secundária e artigo |
| Curva de saturação magnética B-H física do transformador Dagnall C1998 | — | Dagnall C1998 | — | `UNKNOWN` | Pendente de medição física |

## 7. Fundamentação elétrica
- **Entradas e Jacks (4 entradas 2×2 — F1 pp. 5–6; F2):**
  - Canal I (High Treble): `High I` (`1 MΩ` leak + grid stopper `68 kΩ`) e `Low I` (divisor resistivo `68 kΩ / 68 kΩ` = **−6,02 dB** e menor impedância de entrada).
  - Canal II (Normal): `High II` (`1 MΩ` + `68 kΩ`) e `Low II` (**−6,02 dB**).
  - `Jumpered` (F1 p. 6, Fig. 1): guitarra em `High I` e cabo jumper de `Low I` para `High II`, excitando V1b e V1a em paralelo e em fase.
- **Pré-amplificação (V1 = ECC83, V2 = ECC83 — F1 p. 3; F2):**
  - **V1b (Canal I High Treble):** `Rp = 100 kΩ`, `Rk = 2,7 kΩ || Ck = 0,68 µF` (`f_c ≈ 86,8 Hz`), capacitor de acoplamento `2,2 nF` (`0,0022 µF`), potenciômetro `Volume I` (`1 MΩ` log) com **bright cap `0,005 µF` (`5000 pF`)** em paralelo entre a entrada e o cursor (exclusivo do Canal I!).
  - **V1a (Canal II Normal):** `Rp = 100 kΩ`, `Rk = 820 Ω || Ck = 250–330 µF` (resposta plena em graves), capacitor de acoplamento `22 nF` (`0,022 µF`), potenciômetro `Volume II` (`1 MΩ` log) **sem bright cap**.
  - **Rede de mistura passiva:** resistor `470 kΩ` em paralelo com `470 pF` vindo do cursor de `Volume I` somado ao resistor `470 kΩ` vindo do cursor de `Volume II` na grade de V2a.
  - **V2a + V2b (Cathode Follower):** V2a (`Rp = 100 kΩ`, `Rk = 820 Ω || Ck = 0,68 µF` — F1 p. 3 *Tonal Note 1*) acoplado diretamente em DC à grade de V2b (`Rk = 100 kΩ`), que opera como buffer de ganho unitário e baixa impedância de saída (`R_src ≈ 1,3 kΩ`) com compressão assimétrica suave em sobrecarga para acionar a rede passiva de Tone Stack.
- **Tone Stack (FMV Marshall pós-V2b — F1 p. 3; F2; F4):** `R_slope = 33 kΩ`, `C1 = 470 pF`, `C2 = 22 nF`, `C3 = 22 nF`, `R_treble = 220 kΩ`, `R_bass = 1 MΩ`, `R_mid = 22 kΩ` (**taper 10% log**, F1 p. 3 *Tonal Note 2*).
- **Inversor de Fase, Potência, Fonte e NFB (F1 p. 3; F2):**
  - V3 (`ECC83`) em configuração Long-Tailed Pair (LTP) acionando quarteto `4×EL34` em push-pull Classe AB fixed-bias.
  - Realimentação negativa (NFB) retirada **da saída real do alto-falante** (F1 p. 3 *Tonal Note 3*): selecionar `4 Ω` reduz a tensão de realimentação e o fator de amortecimento (graves mais soltos e maior pico de ressonância), enquanto `16 Ω` aumenta o NFB e o amortecimento (graves mais firmes e controlados).
  - Controle `Presence` na malha NFB desviando altas frequências da realimentação para terra.

## 8. Comportamento esperado
- **Ausência total de Master Volume:** o nível de excitação do inversor LTP e das `4×EL34` depende exclusivamente de `Volume I` (`amp_1.gain`) e `Volume II` (`amp_1.volume_2`).
- **Isolamento do Bright Cap no Canal I:** o bright cap de `0,005 µF` atua **somente** sobre o Canal I (`High I`, `Low I`, e o ramo I do modo `Jumpered`), nunca adicionando brilho agudo artificial ao Canal II (`High II`, `Low II`) quando usado isoladamente!
- **Atenuação exata das entradas Low:** `low1` e `low2` atenuam o sinal de entrada em **`6,0 ± 0,2 dB`** em relação a `high1` e `high2` (F1 p. 6).
- **Dependência de impedância no NFB (`Tonal Note 3`):** em `4 Ω`, o amortecimento é menor e a interação com a ressonância do gabinete é mais pronunciada do que em `16 Ω`.

## 9. Parâmetros e controles
> Ordem física do painel frontal (Manual 1959HW F1, pp. 4–6, itens 1–13): `1. POWER · 2. STANDBY · 3. INDICATOR · 4. PRESENCE · 5. BASS · 6. MIDDLE · 7. TREBLE · 8. VOLUME I · 9. VOLUME II · 10–13. INPUTS (I High/Low, II High/Low)` + Painel traseiro: `OUTPUT IMPEDANCE SELECTOR (4 / 8 / 16 Ohm)`.

| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Origem (`Físico` vs `Mod Shop`) | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|---|
| Power (Mains) | `amp_1.power` | boolean | bool | `false/true` | `true` | discreto | `Físico` (Item 1) | Liga alimentação AC | Mute suave `<= 2,5 ms` |
| Standby | `amp_1.standby` | boolean | bool | `false/true` | `false` | discreto | `Físico` (Item 2) | Aplica alta tensão B+ | Rampa exponencial invariante à taxa |
| Presence | `amp_1.presence` | number | scale | `0.0..=10.0` | `5.0` | log | `Físico` (Item 4) | Reduz NFB em altas frequências | Filtro acoplado ao `eff_nfb` |
| Bass | `amp_1.bass` | number | scale | `0.0..=10.0` | `5.5` | audio 10% log | `Físico` (Item 5) | Braço de graves FMV `1 MΩ / 22 nF` | KCL 5×5 + bilinear `f64` |
| Middle | `amp_1.middle` | number | scale | `0.0..=10.0` | `5.0` | **10% log** (F1 p.3) | `Físico` (Item 6) | Braço de médios FMV `22 kΩ` | KCL 5×5 + bilinear `f64` |
| Treble | `amp_1.treble` | number | scale | `0.0..=10.0` | `6.0` | log | `Físico` (Item 7) | Braço agudo FMV `220 kΩ / 470 pF` | KCL 5×5 + bilinear `f64` |
| Volume I | `amp_1.gain` | number | scale | `0.0..=10.0` | `6.5` | audio log | `Físico` (Item 8) | Volume do Canal I + bright cap `0,005 µF` | Ganho Canal I + bypass capacitivo inverso |
| Volume II | `amp_1.volume_2` | number | scale | `0.0..=10.0` | `4.5` | audio log | `Físico` (Item 9) | Volume do Canal II (Normal, escuro/cheio) | Ganho Canal II paralelo sem bright cap |
| Input Jack | `amp_1.input_jack` | enum | id | `high1, low1, high2, low2, jumpered` | `high1` | discreto | `Físico` (Itens 10–13) | Seleciona canal, pad `-6 dB` ou jumper Fig. 1 | Roteamento V1b/V1a + pad `-6,02 dB` |
| Jumper | `amp_1.jumpered` | boolean | bool | `false/true` | `false` | discreto | `Físico` (Fig. 1) | Atalho para `input_jack = jumpered` | Habilita soma paralela V1b + V1a |
| Output Impedance | `amp_1.impedance_ohms` | enum | Ω | `4, 8, 16` | `16` | discreto | `Físico` (Traseiro Item 2) | Altera tensão NFB e fator de amortecimento (F1 p.3) | Escala amortecimento e pico reativo |

*(Controles adicionais de bancada Mod Shop seguem a Spec 077.)*

## 10. Contratos de integração
- Implementa `AudioProcessor` (Spec 053) com `module_id = "amp.marshall.1959_super_lead"` e `model_revision = "1969-jmp-sl-a-pre-july"`.
- Ao selecionar `amp_1.model = "plexi_1959"` sem sobrescrita explícita de `cab_1.speaker` / `amp_1.gain` / `amp_1.input_jack` no mesmo lote, o backend pareia automaticamente `cab_1.speaker = "greenback_4x12"` e valores nominais do modelo.
- Rejeita `amp_1.master`, `amp_1.cut`, `amp_1.reverb`, `amp_1.spongy` e `amp_1.channel_mode` com `E_VALIDATION:` quando este modelo estiver ativo.

## 11. Arquitetura de implementação
- **Código atual mapeado (HEAD `d891389`):** `crates/ampforge-dsp/src/amp.rs` (linhas `311–324, 816–819, 1211, 1230–1268, 1335–1343, 1395–1407, 1449–1471`) e `crates/ampforge-dsp/src/tonestack.rs` (linhas `51–64`).
- **Problemas e dívidas encontrados no código atual (registrados para correção na implementação):**
  1. `FACTUAL_ERROR` (**Bright cap vazando para o Canal II** em `amp.rs:1395–1406`): o filtro `bright_state` é aplicado em `s_in` *antes* de calcular `s_ch2_base`, fazendo o bright cap de `0,005 µF` do Canal I afetar indevidamente o Canal II Normal (`High2`/`Low2`).
  2. `FACTUAL_ERROR` (**Sentido invertido de `impedance_ohms` na Tonal Note 3** em `amp.rs:1335–1340`): o manual oficial 1959HW (p. 3, *Tonal Note 3*) afirma explicitamente que *"the 4 Ohm input will be more loose and more resonant on the low end, while the 16 Ohm input will be tighter and more controlled"*. Porém, `amp.rs:1335–1339` define `imp_factor = match impedance_ohms { 4 => 0.78, 8 => 1.0, _ => 1.24 }`, tornando `16 Ω` *mais* ressonante (`1.24`) e `4 Ω` *menos* ressonante (`0.78`) — o exato oposto do manual!
  3. `FACTUAL_ERROR` (**Trim fantasma de `overdrive`** em `amp.rs:1211, 1455, 1462`): `amp.panel.overdrive` adiciona até `+18 dB` de ganho pré-V1 embora o Marshall 1959 não possua controle de Overdrive.
  4. `ARCHITECTURE_CONFLICT` (**Divergência de voicing** em `amp.rs:816–819` vs `docs/amps/voicing-fontes.md`): `amp.rs` usa `2500 Hz +2.6 dB` / `160 Hz +2.8 dB` enquanto `voicing-fontes.md` cita `2400 Hz +2.5 dB` / `120 Hz +1.5 dB`.

## 12. Estratégia DSP
- **Circuit-level (`f64`):** Tone Stack FMV (`33 kΩ, 470 pF, 22 nF, 22 nF`, Mid 10% log, `R_src = 1,3 kΩ`) via KCL + transformada bilinear (Spec 078); filtro de bright cap `0,005 µF` dependente de `Volume I` aplicado exclusivamente ao ramo do Canal I; rede de mistura `470 kΩ || 470 pF` (Canal I) + `470 kΩ` (Canal II).
- **Behavioral calibrado com memória (`f64`):** estágios paralelos V1b (`Rk=2,7k||0,68µF`, acoplamento `2,2nF`) e V1a (`Rk=820Ω||330µF`, acoplamento `22nF`) → V2a (`Rk=820Ω||0,68µF`) + V2b Cathode Follower → LTP + `4×EL34` push-pull com NFB escalado por `1 / √Z_tap` (`4 Ω` menos NFB/mais ressonante que `16 Ω`), sag de ponte de diodos (`50 µF`) e saturação LF do transformador Dagnall C1998.
- **Oversampling:** seletivo (`2x` padrão, `4x`/`8x` selecionável — Spec 079).

## 13. Requisitos de tempo real
- `alloc_count == 0` e `dealloc_count == 0` em `process()` na thread RT; coeficientes invariantes a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- Estados de V1a, V1b, V2a/b, Tone Stack, LTP/EL34, sag e oversampler isolados na struct `Marshall1959SuperLead`; `reset()` zera registradores em $O(1)$.

## 15. Latência
- Igual à latência do oversampler configurado (Spec 079): `4` (`2x`), `8` (`4x`) ou `16` (`8x`) amostras no modo `MinPhaseIir`.

## 16. Consumo de CPU
- Meta (`ENGINEERING_TARGET`): `<= 8,0%` do orçamento de `128` frames @ `48 kHz` em `2x` OS (`--release`).

## 17. Testes
- Sinais determinísticos em `44,1`, `48` e `96 kHz`, `block_size = 128`: senoides `100 Hz`, `250 Hz`, `440 Hz`, `1 kHz`, `4 kHz`, `5 kHz`; sweep `20 Hz–10 kHz`; medição de ganho diferencial, THD, harmônicos, aliasing DFT (janela Hann `16384` amostras) e contagem de alocações RT.

## 18. Critérios de aceite
- **CA-1 (`ANALYTICAL` — Tone Stack FMV 1959 com Mid 10% Log):** Com sinal de entrada impulso unitário (`x[0] = 1.0`), `sample_rate = 48000 Hz` (`8x` OS = `384 kHz` interno), `block_size = 128`, nas configurações `(B,M,T) ∈ {(5,5,5), (10,0,10), (0,10,0)}`, a magnitude DFT da rede Tone Stack entre `50 Hz` e `10 kHz` permanece dentro de `±0,5 dB` da função de transferência analítica contínua KCL (`R_slope=33 kΩ, C1=470 pF, C2=22 nF, C3=22 nF, R_treble=220 kΩ, R_bass=1 MΩ, R_mid=22 kΩ` com curva 10% log da F1 p. 3; tolerância de `±0,5 dB` derivada do erro máximo de warping bilinear até `10 kHz` a `>= 96 kHz`).
- **CA-2 (`DOCUMENTED` — 4 Entradas, Pad −6 dB e Modo Jumpered):** Com senoide de `440 Hz` a `-34 dBFS` (`0.02` pico, regime linear), `sample_rate = 48000 Hz`, `block_size = 128`, `gain = 3.0`, `volume_2 = 5.0`, as entradas `low1` e `low2` apresentam nível RMS de saída **`-6,0 ± 0,2 dB`** em relação a `high1` e `high2` (F1 p. 6, Itens 11 e 13); no modo `input_jack = "jumpered"` com `gain = 2.0`, variar `volume_2` de `0.0` para `10.0` eleva o nível RMS de saída em `>= 3,0 dB` (F1 p. 6, Fig. 1).
- **CA-3 (`DOCUMENTED` — Bright Cap `0,005 µF` Exclusivo do Canal I):** Com senoide em `4 kHz` vs `250 Hz` a `-34 dBFS`, `sample_rate = 48000 Hz`, `block_size = 128`: (a) no Canal I (`input_jack = "high1"`), a razão de ganho `G(4 kHz) - G(250 Hz)` com `gain = 2.0` é **`>= 4,0 dB` maior** do que com `gain = 9.0` (ação do bright cap `0,005 µF` sobre Volume I, F1 p. 5 *Tonal Note 5*); (b) no Canal II puro (`input_jack = "high2"`, `gain = 0.0`), o bright cap do Canal I não afeta o sinal e a resposta em `4 kHz` relative a `250 Hz` é `>= 3,0 dB` mais escura que no Canal I em `gain = 2.0`.
- **CA-4 (`DOCUMENTED` — Ausência de Master Volume):** Com senoide de `1 kHz` a `-12 dBFS` (`0.2512` pico), `sample_rate = 48000 Hz`, `block_size = 128`, `gain = 7.0`, carregar o modelo `plexi_1959` após `jcm800_2203` configurado com `master = 10.0` ou `master = 1.0` produz saída idêntica (`diferença máxima amostra-a-amostra <= 1e-6`), e enviar `amp_1.master` via `api::dispatch` quando `plexi_1959` está ativo retorna HTTP 400 (`E_VALIDATION:`).
- **CA-5 (`DOCUMENTED` — NFB na Saída do Falante e Seletor de Impedância `Tonal Note 3`):** Com sweep/senoide na frequência de ressonância do gabinete (`98 Hz`, `-20 dBFS`, `sample_rate = 48000 Hz`, `block_size = 128`, `nfb = 6.0`), selecionar `impedance_ohms = 4` produz pico de ressonância em `98 Hz` **`>= 1,0 dB` maior** (menor fator de amortecimento) do que `impedance_ohms = 16` (F1 p. 3, *Tonal Note 3*), e variar `presence` de `0.0` para `10.0` em `4 kHz` produz diferença `>= 3,0 dB` maior com `nfb = 10.0` do que com `nfb = 0.0`.
- **CA-6 (`ENGINEERING_TARGET` — Anti-Aliasing e Invariância Multi-Taxa):** Com senoide de `5 kHz` a `-6 dBFS` (`0.5012` pico), `gain = 10.0`, `sample_rate = 48000 Hz`, `block_size = 128`, medida por DFT de `16384` pontos com janela Hann após `4096` amostras de warm-up, toda raia de aliasing não-harmônica na faixa `50 Hz–20 kHz` permanece **`< -60 dBFS`** em `4x` e `8x` OS; a inclinação `G(5 kHz) - G(200 Hz)` em `gain = 2.0` varia `<= 0,5 dB` entre `44,1`, `48` e `96 kHz`.
- **CA-7 (`ENGINEERING_TARGET` — Segurança RT e Sanitização):** Com `CountingAllocator` ativo na thread RT (`48 kHz`, `block_size = 128`, `100` blocos), `process()` registra `alloc_count == 0` e `dealloc_count == 0`; entradas `NaN`/`±Inf` produzem saída finita.
- **CA-8 (`REGRESSION` — Não-regressão):** Os testes existentes `contract_031_authentic_amps.rs` e `contract_039_amps_fidelity.rs` aplicáveis ao `BritPlexi1959` continuam passando.

## 19. Dependências
- **Obrigatórias:** `Spec 053`, `Spec 077`, `Spec 078`, `Spec 079`.
- **Relacionadas / Legadas:** `Specs 003, 020, 031, 039, 047`.

## 20. Limitações conhecidas
- A curva B-H física do transformador de saída **Dagnall C1998** não possui datasheet primário com parâmetros magnéticos de núcleo; a saturação LF de transformador (`~150 Hz`) permanece como aproximação comportamental calibrada (`INFERENCE`).

## 21. Riscos técnicos
- Corrigir a direção de `imp_factor` (`4 Ω` mais ressonante que `16 Ω` conforme F1 p. 3 *Tonal Note 3*) e isolar o bright cap do Canal II alterará a resposta de `High2`/`Low2`; verificar todos os testes de regressão da Spec 039 ao implementar.

## 22. Matriz de rastreabilidade
| Requisito | Fonte | Spec/CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Tone Stack FMV `33k / 470pF / 22nF / 22nF` com pot Middle `10% log` | F1 (p. 3), F2, F4 | `Spec 054 / CA-1` | `crates/ampforge-dsp/src/tonestack.rs:51–64 @ c20cc65` | `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs`, `contract_039_amps_fidelity.rs` | `TESTED` |
| 4 Entradas (`High/Low` I e II), pad único `-6 dB` e `Jumpered` | F1 (pp. 5–6, Figs. 1–2) | `Spec 054 / CA-2` | `crates/ampforge-dsp/src/amp.rs:1238–1242 @ c20cc65, 1449–1471` | `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs` (`ca_2`), `contract_039_amps_fidelity.rs` (`test_ca_03`) | `TESTED` |
| Bright cap `0,005 µF` sobre Volume I e **ausente** no Canal II Normal | F1 (p. 5, *Tonal Note 5*), F2 | `Spec 054 / CA-3` | `crates/ampforge-dsp/src/amp.rs:1395–1406 @ c20cc65` *(defeito: afeta Canal II)* | Parcial em `contract_039_amps_fidelity.rs`; isolamento Ch II planejado em `contract_054` | `SPECIFIED` |
| Ausência de Master Volume no 1959 Super Lead | F1 (pp. 2, 4–5) | `Spec 054 / CA-4` | `crates/ampforge-dsp/src/amp.rs:1265–1268 @ c20cc65`, `crates/sonix-desktop/src/api.rs` | `crates/ampforge-dsp/tests/contract_039_amps_fidelity.rs` (`test_ca_03`) | `TESTED` |
| NFB na saída do falante (`4 Ω` mais ressonante/menos amortecido que `16 Ω`) | F1 (p. 3, *Tonal Note 3*) | `Spec 054 / CA-5` | `crates/ampforge-dsp/src/amp.rs:1335–1340 @ c20cc65` *(defeito: sentido invertido)* | Planejado: `contract_054_amp_marshall_1959_plexi.rs` | `SPECIFIED` |
| Transformador de saída Dagnall C1998 (curva B-H física medida) | F1 (p. 3) | `Spec 054 / §5, §20` | `crates/ampforge-dsp/src/amp.rs:1615–1628 @ c20cc65` (aproximação LF `tanh`) | Sem medição de hardware | `PENDING` |
