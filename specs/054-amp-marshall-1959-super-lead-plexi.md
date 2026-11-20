---
spec: 054
titulo: Amplificador Marshall 1959 Super Lead / Plexi (1968–1969 JMP, 100W)
status: rascunho
tipo: amplificador
atualizado: 2026-11-19
dependencias: [053, 077, 078, 079]
---
# 054 — Amplificador Marshall 1959 Super Lead / Plexi (Rev. 1968–1969)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `amp.marshall.1959_super_lead` (alias legado: `plexi_1959`, `brit_plexi_1959`, `plexi`)
- **Revisão (`model_revision`):** `1968-1969-jmp-sl-a`
- **Categoria:** Amplificador valvulado (módulo independente)
- **Objetivo:** Especificar de forma autônoma e rastreável o circuito real, os controles autênticos, o comportamento elétrico/dinâmico, a estratégia DSP e os critérios de validação do amplificador **Marshall 1959 Super Lead 100W (Plexi, rev. 1968–1969)**, desacoplando-o do processador monolítico atual (`amp.rs`).

## 2. Escopo
- Modelagem dedicada dos dois canais paralelos independentes (Canal I High Treble e Canal II Normal), suas 4 entradas (`High I`, `Low I`, `High II`, `Low II`) e modo `Jumpered` (I-High + patch I-Low→II-High).
- Estágios V1a/V1b paralelos, rede de mistura 470 kΩ, V2a + V2b Cathode Follower, Tone Stack passivo FMV Marshall (33 kΩ / 470 pF / 22 nF / 22 nF), inversor de fase LTP (ECC83), estágio de potência push-pull 4×EL34 fixed-bias, NFB global com `Presence`, fonte de estado sólido com sag dinâmico e interação com carga reativa.
- Exposição estrita apenas dos controles reais da revisão (sem Master Volume fantasma) + overrides documentados da bancada Mod Shop (Spec 077).

## 3. Fora de escopo
- Variantes 1959 anteriores (1965–1967 JTM100 com retificador/transformadores distintos e bright cap de 100 pF) ou posteriores (1959SLP reedição com FX Loop — o 1968–69 original não possui FX loop).
- Master Volume pós-PI (PPIMV) ou modificações pós-fábrica (cobertos por presets/Mod Shop, não pelo painel padrão do 1959).
- Modelagem de gabinete 4×12 Greenback (coberta pela Spec 080).

## 4. Equipamento ou componente de referência
- **Fabricante:** Marshall Amplification.
- **Modelo exato:** Model 1959 Super Lead 100W ("Plexi").
- **Período / Ano:** 1968–1969 (circuito JMP pré-julho de 1969, referência adicional na reedição handwired 1959HW).
- **Papel na cadeia de sinal:** Cabeçote valvulado non-master volume de 100 W; depende da excitação dos canais paralelos e da saturação do inversor de fase e estágio de potência 4×EL34.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1968-1969 JMP Super Lead (SL/A pré-julho 1969, split cathode V1)`.
- **Variantes históricas relevantes (não misturadas):**
  - *1965–1967 JTM100 / 1959 shared cathode:* V1a e V1b compartilham `Rk=820 Ω / Ck=250–330 µF`, bright cap menor (`100 pF`), resposta mais gorda e escura.
  - *1968–1969 split cathode (selecionada):* Canal I `Rk=2.7 kΩ / Ck=0.68 µF` (ou `820 Ω / 0.68 µF` conforme sub-lote; ver §6/§7), bright cap `5000 pF` (`4.7 nF`) no Volume I, Canal II `Rk=820 Ω / Ck=250–330 µF`.
  - *1959HW Reissue:* segue a topologia SL/A pré-julho 1969 e confirma ordem frontal dos controles no manual oficial em `docs/ui-desktop/fontes/`.
- **Pendências de confirmação de revisão:** confirmar por medição física em unidade de referência a curva exata do transformador Drake 1202-119 e se o lote-alvo usa `Rk(V1b) = 2.7 kΩ` (pós-68) ou `820 Ω` (early-68).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | *Marshall 1959HW Owners Manual* | 1959HW (circuito 1969 SL/A) | Painel frontal, itens 1–13, Figs. 1–2 | `docs/ui-desktop/fontes/marshall-1959HW-owners-manual.pdf` | 2026-11-19 | `[confirmado em documentação]` | Ordem frontal dos controles, 4 entradas (Low −6 dB), jumper I-Low→II-High, bright cap 0,005 µF em Vol I, Mid 10% log, NFB na saída do falante |
| F2 | `docs/amps/voicing-fontes.md` | Marshall 1959SLP 1967–1969 | Seção 3 | `docs/amps/voicing-fontes.md` | 2026-11-19 | `[confirmado em documentação]` | Tone stack 33k/470pF/22nF/22nF, rede 470k||470pF, ausência de Master Volume |
| F3 | Rob Robinette — *Marshall 1959 Super Lead Circuit Analysis* | 1959 1968–1969 | Preamp / Cathode Follower / LTP | https://robrobinette.com/ | 2026-11-19 | `[fonte secundária]` | Topologia paralela V1a/V1b, acoplamento DC V2a→V2b cathode follower, 4×EL34 push-pull |
| F4 | D. T. Yeh & J. O. Smith, *Discretization of the '59 Fender Bassman Tone Stack* (DAFx-06) | Rede passiva FMV | Seções 2–4 | Proc. DAFx-06 (CCRMA Stanford) | 2026-11-19 | `[confirmado em documentação]` | Equacionamento nodal KCL + transformada bilinear do tone stack FMV |
| F5 | Mullard / Philips Datasheets — `ECC83` e `EL34` | Válvulas padrão | Curvas de placa / ratings | Datasheets históricos Mullard | 2026-11-19 | `[confirmado em documentação]` | `µ≈100` (ECC83), limites e características de potência (EL34) |

## 7. Fundamentação elétrica
- **Entradas e Jacks (4 entradas 2×2):**
  - Canal I (High Treble): `High I` (1 MΩ + grid stopper 68 kΩ) e `Low I` (divisor resistivo 68 kΩ/68 kΩ = **−6 dB** e impedância de entrada reduzida `[confirmado em documentação: F1]`).
  - Canal II (Normal): `High II` e `Low II` (−6 dB `[confirmado em documentação: F1]`).
  - `Jumpered`: guitarra em `High I` com cabo patch `Low I → High II`, alimentando ambos os canais em paralelo com interação de impedância `[confirmado em documentação: F1]`.
- **Pré-amplificação (V1 = ECC83/12AX7, V2 = ECC83/12AX7):**
  - **V1a (Canal II Normal):** `Rp = 100 kΩ`, `Rk = 820 Ω` com `Ck = 250–330 µF`, acoplamento `0.022 µF`, `Volume II` `1 MΩ` audio taper `[fonte secundária: F3]`.
  - **V1b (Canal I High Treble):** `Rp = 100 kΩ`, `Rk = 2.7 kΩ` (ou `820 Ω` em unidades início-68) com `Ck = 0.68 µF` (ênfase > ~100–300 Hz), acoplamento `0.0022 µF` (`2.2 nF`), `Volume I` `1 MΩ` com **bright cap `0.005 µF` (`5000 pF`)** `[confirmado em documentação: F1, F3]`.
  - **Rede de mistura:** dois resistores de `470 kΩ` somando Canal I e II; no ramo do Canal I há bypass `470 pF` (peaking em médios-agudos ~2,4 kHz) `[confirmado em documentação: F2]`.
  - **V2a + V2b (Cathode Follower):** estágio de ganho V2a (`Rk = 820 Ω`, `Ck = 0.68 µF` opcional/revisão) acoplado em DC ao seguidor de cátodo V2b (`Rk = 100 kΩ`), que apresenta baixa impedância de saída para acionar o tone stack e produz compressão assimétrica suave em condução de grade quando sobrecarregado `[fonte secundária: F3]`.
- **Tone Stack (FMV Marshall):** `R_slope = 33 kΩ`, `C1 = 470 pF`, `C2 = 22 nF`, `C3 = 22 nF`, `R_treble = 220–250 kΩ`, `R_bass = 1 MΩ`, `R_mid = 22–25 kΩ` (taper 10% log `[confirmado em documentação: F1]`), `R_load = 1 MΩ` `[confirmado em documentação: F2, F4]`.
- **Inversor de fase e Potência:** LTP com `V3 = ECC83` (`R_tail`, `47 kΩ / 100 kΩ`), acoplando `4×EL34` em push-pull Classe AB fixed-bias (`B+ ≈ 460 V` nominal `[fonte secundária: F3]`, bias típico `~36 mA` por válvula `[aproximação DSP]`).
- **NFB e Presence:** realimentação negativa global retirada da saída do alto-falante (tap de saída) com resistor de realimentação (`47 kΩ`/`100 kΩ` dependendo do ano) retornando ao cátodo do LTP; potenciômetro `Presence` com capacitor (`0.1 µF` ou `0.68 µF`) desviando agudos da malha NFB para terra `[confirmado em documentação: F1]`.
- **Fonte, Transformador e Carga:** retificação por ponte de diodos de silício (`SolidStateDiode`), transformador de saída Drake `1202-119` (seleção `4 / 8 / 16 Ω`), interação reativa com gabinete 4×12.

## 8. Comportamento esperado
- **Sem Master Volume:** o drive do estágio de potência e a abertura de dinâmica vêm diretamente da soma de `Volume I` (`gain`) e `Volume II` (`volume_2`). `amp_1.master` não existe neste amplificador e não pode afetar a saída (Spec 039 CA-3).
- **Bright Cap dependente de Volume I:** com `Volume I` entre `1` e `4`, o capacitor de `5000 pF` eleva agudos acima de `2 kHz` em `+4 a +10 dB`; à medida que `Volume I → 10`, o efeito do bright cap desaparece (curto-circuito sobre o potenciômetro).
- **Entradas Low:** `low1` e `low2` reduzem o sinal limpo em **`6,0 ± 0,3 dB`** relativamente a `high1` e `high2` (uma única atenuação, sem pad duplo).
- **Interação Jumpered:** com `input_jack = "jumpered"`, elevar `Volume II` adiciona corpo em `100–400 Hz` e aumenta a saturação do V2a/LTP (`ΔRMS >= 3 dB` entre `vol2=0` e `vol2=10` com `vol1=2`).
- **Dinâmica limpa → crunch:** com palhetada leve ou volume da guitarra reduzido em −12 dB, a distorção cai rapidamente (THD < 2% em `Volume I = 3`), passando a crunch aberto com harmônicos pares e ímpares equilibrados quando `Volume I >= 7` (saturação principal no V2b cathode follower, LTP e 4×EL34).
- **Presence acoplado ao NFB:** eficácia de `Presence` em `4 kHz` reduz-se em `>= 3 dB` quando `nfb = 0` vs `nfb = 10` (Mod Shop).

## 9. Parâmetros e controles
> Ordem física do painel (da esquerda para a direita, Manual 1959HW F1): `POWER · STANDBY · INDICATOR · PRESENCE · BASS · MIDDLE · TREBLE · VOLUME I · VOLUME II · INPUTS (I-High/Low, II-High/Low)`.

| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Power (Mains) | `amp_1.power` | boolean | bool | `false/true` | `true` | discreto | Liga/desliga alimentação AC | Mute suave `<= 2,5 ms` até `< 1e-6` |
| Standby | `amp_1.standby` | boolean | bool | `false/true` | `false` | discreto | Corta alta tensão B+ mantendo filamentos | Rampa exponencial `tau` invariante à taxa |
| Presence | `amp_1.presence` | number | scale | `0.0..=10.0` | `5.0` | log/reverso | Reduz NFB acima de ~2–4 kHz | Filtro na malha NFB escalado por `nfb` |
| Bass | `amp_1.bass` | number | scale | `0.0..=10.0` | `5.5` | audio 10% log | Braço de graves do FMV 33k/22nF | KCL 5×5 + bilinear `f64` |
| Middle | `amp_1.middle` | number | scale | `0.0..=10.0` | `5.0` | 10% log (F1) | Braço de médios 22kΩ FMV | KCL 5×5 + bilinear `f64` |
| Treble | `amp_1.treble` | number | scale | `0.0..=10.0` | `6.0` | híbrido log | Divisor agudo 220kΩ/470pF FMV | KCL 5×5 + bilinear `f64` |
| Volume I (High Treble) | `amp_1.gain` | number | scale | `0.0..=10.0` | `6.5` | audio log | Ganho Canal I + ação inversa do bright cap 5000pF | Ganho pré-V1b + filtro bright dependente do pot |
| Volume II (Normal) | `amp_1.volume_2` | number | scale | `0.0..=10.0` | `4.5` | audio log | Ganho Canal II Normal (mais grave/cheio) | Ramo paralelo V1a somado antes de V2a |
| Input Jack | `amp_1.input_jack` | enum | id | `high1, low1, high2, low2, jumpered` | `high1` | discreto | Seleciona canal, pad −6 dB ou jumper | Roteamento de ramos + pad único `-6.0 dB` |
| Jumper | `amp_1.jumpered` | boolean | bool | `false/true` | `false` | discreto | Atalho para `input_jack = jumpered` | Habilita soma paralela Canal I + II |

*(Controles avançados de chassi — `preamp_tube`, `power_tube`, `rectifier`, `impedance_ohms`, `bias_ma`, `sag`, `nfb`, `xformer_sat`, `preamp_plate_v`, `triode_asym`, `tonestack` — seguem contrato da Spec 077 com defaults nominais do 1959: `12ax7`, `el34`, `diode`, `16 Ω`, `36 mA`, `sag=4.0`, `nfb=6.0`, `xformer_sat=5.0`, `B+_pre=250 V`, `B+_power=460 V`.)*

## 10. Contratos de integração
- Implementa `AudioProcessor` (Spec 053) com `module_id = "amp.marshall.1959_super_lead"` e `model_revision = "1968-1969-jmp-sl-a"`.
- Rejeita `amp_1.master`, `amp_1.cut`, `amp_1.reverb`, `amp_1.spongy` e `amp_1.channel_mode` com `E_VALIDATION:` quando este modelo estiver ativo (sem efeitos ocultos).
- Recebe `speaker_resonance_hz` do gabinete conectado via `ProcessContext` (Spec 080) sem importar `VirtualCabProcessor`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/amp.rs` (linhas 311–324, 816–819, 1230–1268, 1449–1471) e `crates/ampforge-dsp/src/tonestack.rs` (linhas 51–64).
- **Dívidas identificadas no código atual:**
  1. Compartilha struct monolítica `WhiteBoxAmpProcessor` e estados `adaa_*` com os outros 4 amplificadores.
  2. `amp_1.overdrive` existe no `AmpFrontPanel` genérico e ainda atua como trim extra (`amp.rs:1211`), embora o 1959 real não possua controle `Overdrive` — na modularização, o trim deve ficar restrito a `1.0` fixo no modelo 1959 (ou removido do descritor do modelo).
  3. O filtro de voicing fixo em `amp.rs:816–819` (`peaking 2500 Hz +2.6 dB`, `low_shelf 160 Hz +2.8 dB`) diverge ligeiramente dos ganhos documentados em `docs/amps/voicing-fontes.md` (`2400 Hz +2.5 dB`, `120 Hz +1.5 dB`); alinhar com derivação explícita da rede 470k||470pF e `Ck=330 µF`.
- **Estrutura alvo:** `crates/ampforge-dsp/src/amps/plexi_1959.rs` (`Marshall1959Plexi`), compondo `TubeTriodeStage`, `CathodeFollowerStage`, `PassiveToneStack`, `PushPullPowerStage` (Specs 077–079).

## 12. Estratégia DSP
- **Circuit-level (nodal/bilinear `f64`):** Tone Stack FMV 33k/470pF/22nF/22nF (`PassiveToneStack` KCL 5×5) e rede de bright cap dependente da posição do potenciômetro `Volume I`.
- **Behavioral calibrado com memória:** estágios V1a/V1b paralelos, V2a/V2b cathode follower (compressão assimétrica), LTP + 4×EL34 push-pull com ADAA1, saturação LF do transformador Drake (150 Hz), sag dinâmico de fonte de diodo e acoplamento reativo de impedância.
- **Oversampling:** seletivo (padrão `2x`, selecionável `4x`/`8x`) apenas sobre os estágios não lineares e tone stack (Spec 079).
- **Simplificações declaradas:** não resolve MNA global de 40+ nós em RT; corrente de grade de bloqueio (blocking distortion) e microfonia de válvulas não são simuladas no nível físico completo nesta revisão.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap em `process()` (alocador de contagem = 0).
- Zero locks, zero I/O, zero panic; todos os coeficientes derivados de `sample_rate` em `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- Estado DSP interno isolado por instância (`v1a_state`, `v1b_state`, `v2_state`, `tonestack_z`, `power_state`, `sag_state`, `oversampler_state`), zerado em `reset()`.
- Troca de amplificador executa a instância antiga e a nova em paralelo por `20 ms` (crossfade equal-power) sem compartilhar registradores ADAA.

## 15. Latência
- Dependente exclusivamente do oversampler configurado (Spec 079): `0` amostras em `1x`; latência exata do filtro polifásico/IIR em `2x/4x/8x`, declarada por `latency_samples()`.

## 16. Consumo de CPU
- Orçamento alvo: **`<= 8%`** do deadline de 128 frames @ 48 kHz em `2x` OS e **`<= 20%`** em `8x` OS (build `--release`, CPU x86_64 de referência).

## 17. Testes
- **Sinais:** senoide 100 Hz / 1 kHz / 5 kHz, sweep 20 Hz–20 kHz, multitone (1,9 + 2,0 kHz), burst forte→silêncio, DI real de guitarra com volume em 100%, 50% e 25%.
- **Métricas:** resposta FMV `±0,5 dB` vs função analítica KCL; atenuação `low1`/`low2` = `-6,0 ± 0,3 dB`; variação bright cap `vol1=2` vs `vol1=9`; efeito `volume_2` em `jumpered`; alias residual `< -60 dBFS` em 4×/8× OS; isolamento completo de `master` herdado de outro amp; derivada de troca `<= 1,5×` em 100 Hz; zero-alloc RT.
- **Comparação A/B:** quando houver captura de 1959 real via load box + mesmo IR 4×12 Greenback, level-match `±0,1 dB RMS` na mesma DI e sample rate.

## 18. Critérios de aceite
- **CA-1 (Tone Stack FMV 1959):** Com `B=M=T=5` e extremos `0/10`, a curva do tone stack (50 Hz–10 kHz) permanece dentro de `±0,5 dB` da referência analítica KCL (33 kΩ, 470 pF, 22 nF, 22 nF).
- **CA-2 (4 Entradas e Jumper):** `low1` e `low2` atenuam `6,0 ± 0,3 dB` em relação a `high1` e `high2` em regime linear; em `jumpered` com `gain=2`, variar `volume_2` de `0` para `10` produz `ΔRMS >= 3 dB` e aumento de energia em `100–300 Hz`.
- **CA-3 (Bright Cap Dinâmico):** Com `gain=2` (Volume I baixo), o ganho relativo em `4 kHz` vs `250 Hz` é `>= 4 dB` maior do que com `gain=9` no Canal I (redução progressiva do bright cap ao abrir o volume).
- **CA-4 (Ausência de Master Fantasma):** Carregar `plexi_1959` após `jcm800_2203` com `master=10` gera saída idêntica (`erro <= 1e-6`) ao carregamento a partir do estado inicial; enviar `amp_1.master` retorna erro de validação explícito.
- **CA-5 (Presence acoplado ao NFB):** Variar `presence` `0→10` em `4 kHz` produz diferença `>= 3 dB` maior com `nfb=10` do que com `nfb=0`.
- **CA-6 (Aliasing e Taxas Múltiplas):** Com senoide `5 kHz` a `-6 dBFS` e `gain=10`, aliasing residual `< -60 dBFS` em `4x`/`8x` OS; resposta do bright cap e tempo de standby (`2,5 ± 0,5 ms`) coincidem em `44,1`, `48` e `96 kHz` (`<= 0,5 dB`).
- **CA-7 (Erro e Segurança RT):** Entradas `NaN`/`Inf` são sanitizadas para saída finita em `[-1.0, 1.0]`; zero alocações e zero desalocações em `process()`.
- **CA-8 (Não-regressão):** Testes existentes das Specs 003, 020, 031 e 039 aplicáveis ao `BritPlexi1959` continuam passando.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 077` (estágios valvulados/power/sag) + `Spec 078` (Tone Stack FMV) + `Spec 079` (Oversampling).
- **Relacionadas / Legadas:** `Specs 003, 020, 031, 039, 047`.

## 20. Limitações conhecidas
- Falta medição física de unidade 1959 de 1968–69 com load box reativa para calibração fina da saturação do transformador Drake 1202-119.
- Blocking distortion severa em condução extrema de grade está simplificada.

## 21. Riscos técnicos
- Divergência entre o voicing atual (`amp.rs:816–819`) e `docs/amps/voicing-fontes.md` pode alterar presets existentes se não calibrada com level-matching e testes de não-regressão.
- Saturação excessiva do V2b/power em `Volume I = 10` + `Volume II = 10` pode exigir ajuste cuidadoso de headroom interno antes do limiter final.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `003`, `020`, `031`, `039`.
- **Documentos locais:** `docs/ui-desktop/fontes/marshall-1959HW-owners-manual.pdf`, `docs/amps/voicing-fontes.md`.
- **Código atual:** `crates/ampforge-dsp/src/amp.rs`, `crates/ampforge-dsp/src/tonestack.rs`.
- **Testes existentes / futuros:** `contract_031_authentic_amps.rs`, `contract_039_amps_fidelity.rs`, futuro `contract_054_amp_marshall_1959.rs`.
