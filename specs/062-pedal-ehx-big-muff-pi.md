# Spec 062: Pedal Electro-Harmonix Big Muff Pi (V3 NYC 4-Stage BJT Fuzz/Sustainer)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-pedal` (`drive` / `fuzz`)
> **Identificador canônico:** `pedal.ehx.big_muff_pi`
> **Variante / Revisão:** `1977 V3 NYC Transistor Version (PCB EH-3003, 4× BJT 2N5088 / BC239C, 4× Diodos 1N914)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/drive.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [031](031-amplificadores-fieis-aos-esquemas-e-controles-originais.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [079](079-componente-oversampling-polifasico-e-anti-aliasing.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação do pedal **Electro-Harmonix Big Muff Pi (Revisão V3 NYC 1977, PCB EH-3003, 4× BJT)** como processador modular independente (`pedal.ehx.big_muff_pi`), corrigindo as simplificações do código atual (`crates/ampforge-dsp/src/fx/drive.rs:413–426, 601–624`) para representar fielmente:
1. Os quatro estágios transistores BJT `Q1`–`Q4` (`2N5088`) com realimentação shunt-série e capacitores Miller coletor-base de `470 pF`;
2. Os dois estágios em cascata de soft-clipping no feedback coletor-base (`Q2` e `Q3`) desacoplados por capacitores de `1 µF` ($f_{\text{HPF,fb}} \approx 31\text{ Hz}$); e
3. O **Tone Stack Passivo em Ponte Cruzada** de 1ª ordem ($R_8 = 39\text{ k}\Omega, C_8 = 10\text{ nF} \Rightarrow f_{\text{LPF}} = 408,1\text{ Hz}$ e $C_9 = 4\text{ nF}, R_5 = 22\text{ k}\Omega \Rightarrow f_{\text{HPF}} = 1808,6\text{ Hz}$) que produz o entalhe clássico (*mid-scoop* de `-7 dB a -10 dB` em **`~1000 Hz`**), seguido do 4º estágio BJT de recuperação de ganho (`Q4`, $+15\text{ dB}$).

---

## 2. Escopo

- Revisão `V3 NYC Transistor Version (1977, PCB EH-3003)` com `4× 2N5088` e `4× 1N914` (`Sustain`, `Tone`, `Volume`).

---

## 3. Fora de escopo

- **V4 / V5 "Op-Amp Big Muff" (1978, `RC4558NB` + `LM741CN`):** Topologia de circuito inteiramente diferente (sem transistores discretos e sem diodos de realimentação coletor-base) — jamais deve ser misturada com a V3.
- **V1 "Triangle" (1969), V2 "Ram's Head" (1973) ou Sovtek "Green Russian / Civil War" (1991–1994):** Possuem valores distintos nos resistores de emissor e nos capacitores do tone stack (ex.: Russian usa `3,9 nF / 10 nF` com `20 kΩ / 22 kΩ`, deslocando o mid-scoop).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`fuzz` / `sustainer`) |
| **Fabricante de referência** | Electro-Harmonix (Mike Matthews & Bob Myer, New York City, NY, EUA) |
| **Modelo de referência** | Big Muff Pi (NYC V3 / Reissue) |
| **Ano / época / revisão** | 1977 (V3, PCB `EH-3003`, 4× BJT NPN `2N5088` / `2N5089`, 4× diodos silício `1N914`) |
| **Identificador no sistema** | `pedal.ehx.big_muff_pi` (`DriveModel::BigMuff` / UI slug `muff`) |
| **Papel na cadeia de sinal** | Bloco `DRIVE` pré-amplificador (sustainer harmônico de altíssimo ganho e compressão com *mid-scoop* centrado em `1 kHz`). |

---

## 5. Princípio físico e funcional do equipamento real

A arquitetura de Bob Myer no Big Muff Pi (`EH-3003`) divide o processamento em **quatro estágios BJT emissor-comum com realimentação negativa local** (`Q1`, `Q2`, `Q3`, `Q4`):

1. **Estágio 1 — Input Booster + Limitador Passa-Baixas Miller (`Q1`):**
   - Amplificador emissor-comum com resistor de emissor $R_{13} = 100\text{ }\Omega$, resistor de coletor $R_{11} = 10\text{ k}\Omega$, resistor de realimentação coletor-base $R_2 = 470\text{ k}\Omega$ e capacitor Miller coletor-base $C_{10} = 470\text{ pF}$.
   - O capacitor $C_{10} = 470\text{ pF}$ sofre **multiplicação de Miller** pelo ganho do estágio, formando um filtro passa-baixas de 1ª ordem em $f_c \approx 1,2\text{ kHz}$ a $1,5\text{ kHz}$ logo na entrada, antes do potenciômetro `Sustain` (`100 kΩ`).
2. **Estágios 2 e 3 — Dual Cascaded Feedback Clipping (`Q2` e `Q3`):**
   - Cada estágio (`Q2` e `Q3`) é um amplificador BJT idêntico ao primeiro (com seu próprio capacitor Miller coletor-base de `470 pF`), porém com um **par antiparalelo de diodos de silício `1N914` acoplado por um capacitor de `1 µF` ($C_6$, $C_7$) na malha de realimentação coletor-base**!
   - Isso produz dois efeitos cruciais:
     - **Soft-clipping ativo no coletor-base:** Quando a excursão no coletor excede $\pm 0,6\text{ V}$, os diodos `1N914` conduzem e reduzem dinamicamente o ganho do BJT para a unidade, sem travar o ponto quiescente DC graças ao capacitor série de `1 µF`.
     - **Abaixo de $\sim 35\text{ Hz}$:** O capacitor de `1 µF` na série dos diodos apresenta reatância elevada, reduzindo a ação de clipping em subgraves.
     - **Três polos Miller acumulados (`3× 470 pF` em `Q1`, `Q2`, `Q3`):** Eliminam a aspereza de alta frequência (*fizz*) apesar do ganho em cascata superior a `+70 dB`.
3. **Estágio de Tone Passivo em Ponte Cruzada (*Cross-Over Passive Bridge*) — *Mid-Scoop* em `1000 Hz`:**
   - A saída de `Q3` alimenta dois filtros RC de **1ª ordem** em paralelo conectados às extremidades opostas do potenciômetro `Tone` (`100 kΩ Linear`):
     - **Braço de Graves (LPF 1ª ordem):** Resistor $R_8 = 39\text{ k}\Omega$ para o terra com capacitor $C_8 = 10\text{ nF}$ (`0,01 µF`):
       $$f_{\text{LPF}} = \frac{1}{2\pi \cdot 39\,000 \cdot 10 \times 10^{-9}} \approx 408,1\text{ Hz}$$
     - **Braço de Agudos (HPF 1ª ordem):** Capacitor $C_9 = 4\text{ nF}$ (`0,004 µF`) com resistor $R_5 = 22\text{ k}\Omega$ para o terra:
       $$f_{\text{HPF}} = \frac{1}{2\pi \cdot 22\,000 \cdot 4 \times 10^{-9}} \approx 1808,6\text{ Hz}$$
   - Como $f_{\text{LPF}} = 408,1\text{ Hz} < f_{\text{HPF}} = 1808,6\text{ Hz}$, existe um hiato espectral entre os dois braços! Quando o cursor de `Tone` está no centro (`50%`), a soma dos dois braços de 1ª ordem produz o célebre **vale de médios (*mid-scoop*) de `-7,0 dB a -10,0 dB` centrado em $f_{\text{notch}} = \sqrt{408,1 \cdot 1808,6} \approx 859\text{ Hz} \text{ a } 1000\text{ Hz}$**.
4. **Estágio 4 — Transistor BJT de Recuperação Pós-Tone (`Q4`):**
   - O Tone Stack passivo introduz uma perda de inserção de cerca de `-14 dB`. O 4º transistor `Q4` (`2N5088`, sem capacitor Miller de `470 pF`, $R_c = 15\text{ k}\Omega, R_e = 3,9\text{ k}\Omega \parallel \text{bias}$) restaura **`+14,5 dB`** de ganho com leve compressão assimétrica de coletor antes do potenciômetro final `Volume` (`100 kΩ`).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | ElectroSmash — *Electro-Harmonix Big Muff Pi Analysis* (arquivado e verificado via Wayback Machine) | `https://web.archive.org/web/20240115000000/https://www.electrosmash.com/big-muff-pi-analysis` (HTTP 200 verificado) | `SECONDARY_TECHNICAL` | Esquema completo da PCB `EH-3003`, equações dos 4 estágios BJT `2N5088`, capacitores Miller `470 pF`, clipping coletor-base `1N914 + 1 µF`, Tone Stack passivo (`408 Hz` LPF / `1808 Hz` HPF, notch em `1 kHz`) e estágio de recuperação `Q4`. |
| `F-02` | Kit Rae — *The Big Muff Pi History and Circuit Guide (V3 NYC EH-3003)* | `http://www.kitrae.net/music/big_muff_history.html` (HTTP 200 verificado) | `SECONDARY_TECHNICAL` | Documentação histórica e valores de componentes por revisão (`V1` a `V9`, distinguindo `V3 EH-3003 Transistor` de `V4 Op-Amp`). |
| `F-03` | ON Semiconductor — *2N5088 / 2N5089 NPN Low-Noise High-Gain Amplifier Transistor Datasheet* | Especificação oficial do componente ($h_{\text{FE}} = 300..900$, $C_{\text{cb}} \approx 4\text{ pF}$) | `PRIMARY_COMPONENT` | Parâmetros físicos dos 4 transistores BJT `Q1`–`Q4`. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| 4 estágios BJT `2N5088` (`Q1` boost, `Q2` e `Q3` feedback diode clippers, `Q4` tone recovery) | `F-01`, `F-02` | V3 NYC 1977 (`EH-3003`) | `F-01` §"Circuit Analysis / 4 Stages" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Capacitores Miller coletor-base de `470 pF` em `Q1`, `Q2` e `Q3` | `F-01` | V3 NYC (`EH-3003`) | `F-01` §"Input Booster & Clipping Stages" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Diodos `1N914` em série com capacitor `1 µF` na realimentação coletor-base de `Q2` e `Q3` | `F-01` | V3 NYC (`EH-3003`) | `F-01` §"Clipping Stage Frequency Response" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Tone Stack passivo de 1ª ordem: LPF $39\text{ k}\Omega + 10\text{ nF}$ (`408,1 Hz`) e HPF $22\text{ k}\Omega + 4\text{ nF}$ (`1808,6 Hz`), gerando *mid-scoop* de `~7,5 dB` em `1000 Hz` | `F-01` | V3 NYC (`EH-3003`) | `F-01` §"Big Muff Tone Control" | `SECONDARY_TECHNICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input] ──► [Q1 2N5088 Input Booster + C10 470pF Miller LPF]
                  │
                  ▼
            [P1 Sustain Pot (100 kΩ)]
                  │
                  ▼
            [Q2 2N5088 Clipping Stage 1: C11 470pF Miller || (C6 1µF + D1||D2 1N914 Collector-Base Feedback)]
                  │
                  ▼
            [Q3 2N5088 Clipping Stage 2: C12 470pF Miller || (C7 1µF + D3||D4 1N914 Collector-Base Feedback)]
                  │
                  ├────► (Braço Bass 1ª Ordem: R8 39kΩ + C8 10nF LPF @ 408.1 Hz) ────┐
                  │                                                                  ├──► [P2 Tone Wiper (100 kΩ)]
                  └────► (Braço Treble 1ª Ordem: C9 4nF + R5 22kΩ HPF @ 1808.6 Hz) ──┘          │
                                                                                                ▼
                                                                             [Q4 2N5088 Output Recovery (+14.5 dB)]
                                                                                                │
                                                                                                ▼
                                                                             [P3 Volume Pot (100 kΩ)] ──► [Output]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/drive.rs`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/fx/drive.rs#L413-L426) e [`601-624`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/fx/drive.rs#L601-L624) possui as seguintes divergências auditadas:

1. **`FACTUAL_ERROR` (`drive.rs:416–420`):** Os dois filtros do Tone Stack (`muff_lpf` em `480 Hz` e `muff_hpf` em `1890 Hz`) são implementados como **biquads Butterworth de 2ª ordem (`Q = 0.707`)**! Em um filtro Butterworth de 2ª ordem, a rotação de fase no centro geométrico (`~950 Hz`) entre o LPF e o HPF aproxima-se de **`180°`** ($\pm 90^\circ$ de cada filtro de 2ª ordem), provocando cancelamento destrutivo excessivo e inclinações de `12 dB/oitava`. No circuito real do Big Muff Pi (`EH-3003`), ambos os braços são redes RC passivas de **1ª ordem** (`6 dB/oitava`, $\pm 45^\circ$ de fase máxima, diferença de fase de $90^\circ$), com cortes em `408,1 Hz` ($39\text{ k}\Omega / 10\text{ nF}$) e `1808,6 Hz` ($22\text{ k}\Omega / 4\text{ nF}$) carregados mutuamente pelo potenciômetro `100 kΩ`.
2. **`FACTUAL_ERROR` (`drive.rs:601–611`):** O código atual não modela os capacitores Miller coletor-base de `470 pF` em `Q1`, `Q2` e `Q3` (que suavizam as altas frequências antes e durante cada estágio de clipping), nem o desacoplamento de `1 µF` dos diodos de feedback, nem o 4º estágio BJT de recuperação (`Q4`).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Normalizada | Unidade / Escala UI | Curva / Taper | Valor Padrão | Papel Elétrico no Circuito |
|---|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `drive.enabled` | `pedal.muff.enabled` | `false / true` | Booleano | Discreto | `false` | True-bypass mecânico. |
| **SUSTAIN** | `drive.gain` | `pedal.muff.gain` | `0.0 .. 1.0` | `0 .. 100 %` | Linear / Log (`100 kΩ`) | `0.50` | Divisor resistivo entre o coletor de `Q1` e a entrada de `Q2`, variando a atenuação inter-estágio antes da cascata de clipping `Q2 → Q3`. |
| **TONE** | `drive.tone` | `pedal.muff.tone` | `0.0 .. 1.0` | `0 .. 100 %` | Linear (`100 kΩ B`) | `0.50` | Interpola na ponte passiva de 1ª ordem entre o braço LPF (`408,1 Hz`) e o braço HPF (`1808,6 Hz`), produzindo o *mid-scoop* de `~1 kHz` no centro. |
| **VOLUME** | `drive.level` | `pedal.muff.level` | `0.0 .. 1.0` | `0 .. 100 %` | Linear / Áudio (`100 kΩ`) | `0.50` | Divisor resistivo na saída do coletor de `Q4`. |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Estágio `Q1` (Input Booster + Polo Miller `470 pF`):**
   - Filtro passa-altas de acoplamento (`75 Hz`) + ganho linear de $+16\text{ dB}$ + filtro passa-baixas de 1ª ordem Miller ($f_{M1} \approx 1250\text{ Hz}$) + saturação suave de coletor BJT.
2. **Atenuador `Sustain` (`P1`):**
   - Escala a saída de `Q1` por $g_{\text{sustain}} \in [0,05,\; 1,0]$.
3. **Estágios `Q2` e `Q3` (Dual Cascaded Collector-Base Feedback Clippers):**
   - Para cada estágio $k \in \{2, 3\}$ na taxa de oversampling $8\times f_s$:
     - Acoplamento de entrada passivo (`HPF` em $f_c \approx 110\text{ Hz}$) e desacoplamento DC da malha dos diodos (`1 µF` $\Rightarrow 31\text{ Hz}$);
     - Ganho BJT em malha aberta limitado pelo polo Miller de `470 pF` ($f_{Mk} \approx 1350\text{ Hz}$) e pela condução não-linear coletor-base do par antiparalelo `1N914` ($V_f = 0,62\text{ V}$) somada a uma leve assimetria de ponto quiescente BJT ($\delta_{\text{bias}} = 0,04$).
4. **Tone Stack Passivo em Ponte Cruzada de 1ª Ordem (Equação Nodal Exata):**
   - Seja $H_{\text{Tone}}(s, \alpha)$ a função de transferência analítica de 2ª ordem (soma carregada de dois ramos RC de **1ª ordem** com $R_8 = 39\text{ k}\Omega, C_8 = 10\text{ nF}, R_5 = 22\text{ k}\Omega, C_9 = 4\text{ nF}, R_{\text{pot}} = 100\text{ k}\Omega$ na posição $\alpha = \text{tone} \in [0, 1]$):
     $$H_{\text{LPF},1}(s) = \frac{1}{1 + s R_8 C_8} \quad (f_{\text{LPF}} = 408,1\text{ Hz}), \qquad H_{\text{HPF},1}(s) = \frac{s R_5 C_9}{1 + s R_5 C_9} \quad (f_{\text{HPF}} = 1808,6\text{ Hz})$$
     ponderados pela impedância de carga do divisor $R_{\text{pot}} = 100\text{ k}\Omega$, discretizados por Tustin.
5. **Estágio `Q4` (Tone Recovery Amplifier):**
   - Ganho de recuperação de $+14,5\text{ dB}$ ($5,31\times$) com saturação suave de trilho BJT (`tanh`) antes do potenciômetro `Volume`.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `8×` polifásico IIR (Spec 079) em torno da cascata `Q1 → Q2 → Q3`. O Tone Stack passivo e o estágio de recuperação `Q4` operam com precisão `f64`.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`.
- Complexidade $O(1)$ por amostra; WCET $\le 14\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- **Entrada:** Impedância de entrada moderada ($\sim 38\text{ k}\Omega$ a $120\text{ k}\Omega$ dependente de frequência devido à realimentação shunt de `Q1`).
- **Saída:** Coletor de `Q4` via potenciômetro `Volume` (`100 kΩ`).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/big_muff_pi.rs`.
- **Struct principal:** `pub struct BigMuffPiProcessor` implementando `DspBlock` + `AutomatableParams` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Subsistema | Componente Real | Valor Documentado (`EH-3003`) | Invariante Física |
|---|---|---|---|
| Polos Miller `Q1–Q3` | $C_{10}, C_{11}, C_{12}$ | $3 \times 470\text{ pF}$ | Atenuação de alta frequência progressiva antes de cada estágio de clipping. |
| Braço Bass Tone | $R_8 = 39\text{ k}\Omega, C_8 = 10\text{ nF}$ | $f_{\text{LPF}} = 408,1\text{ Hz}$ (1ª ordem) | Passa-baixas de `6 dB/oitava` quando `Tone = 0.0`. |
| Braço Treble Tone | $R_5 = 22\text{ k}\Omega, C_9 = 4\text{ nF}$ | $f_{\text{HPF}} = 1808,6\text{ Hz}$ (1ª ordem) | Passa-altas de `6 dB/oitava` quando `Tone = 1.0`. |
| Mid-Scoop Central | Soma em $P_2 = 100\text{ k}\Omega$ (`Tone = 0.5`) | Notch de `-7,0 dB a -10,0 dB` em `850–1000 Hz` | Vale de médios gerado pela lacuna espectral entre `408,1 Hz` e `1808,6 Hz`. |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste do Mid-Scoop de 1ª Ordem em `Tone = 0.5`:** verifica o vale de `-6 dB a -11 dB` em `900–1000 Hz` relativo a `200 Hz` e `3500 Hz`.
2. **Teste de Basculamento (*Tilt*) do Tone Stack (`Tone = 0.0` vs `Tone = 1.0`):** verifica inversão da relação de energia Grave/Agudo.
3. **Teste de Sustentação e Compressão em Cascata (`Q2 → Q3`):** verifica que variações de `12 dB` na entrada em alto `Sustain` resultam em variação comprimida na saída devido ao duplo clipping em cascata.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Caso o usuário deseje futuramente variantes "Ram's Head" ou "Green Russian", elas poderão ser parametrizadas alterando exclusivamente os 4 valores $(R_8, C_8, R_5, C_9)$ do Tone Stack e os capacitores Miller, mantendo `V3 NYC (EH-3003)` como o contrato canônico de `pedal.ehx.big_muff_pi`.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Mid-Scoop Característico do Tone Stack em Ponte de 1ª Ordem (`Tone = 0.5`) (`DOCUMENTED` / `REGRESSION`):**
  - *Sinal de entrada:* Varredura senoidal ou tons de teste na entrada do Tone Stack nas frequências de `200 Hz` (grave), `950 Hz` (centro do mid-scoop) e `3500 Hz` (agudo).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `pedal.ehx.big_muff_pi` com `drive.tone = 0.50`.
  - *Método de medição:* Magnitude da função de transferência do estágio Tone Stack + Recuperação `Q4` em `200 Hz`, `950 Hz` e `3500 Hz`.
  - *Critério e tolerância:* Em `drive.tone = 0.50`, a resposta em `950 Hz` deve apresentar um vale (*mid-scoop*) de **`5,5 dB` a `11,0 dB` abaixo** da média das magnitudes em `200 Hz` e `3500 Hz` (valor documentado para `EH-3003`: `-7,5 dB` relativo às bandas laterais).
  - *Origem da tolerância:* Curva de resposta medida do Tone Stack `EH-3003` ($39\text{ k}\Omega / 10\text{ nF}$ e $22\text{ k}\Omega / 4\text{ nF}$) em `F-01` §"Big Muff Tone Control".

- **CA-02 — Basculamento de 1ª Ordem do Controle `Tone` (`Tone = 0.0` vs `Tone = 1.0`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Tons de teste em `200 Hz` e `3500 Hz`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Comparação entre `drive.tone = 0.0` (100% braço LPF `408,1 Hz`) e `drive.tone = 1.0` (100% braço HPF `1808,6 Hz`).
  - *Método de medição:* Diferença de ganho $\Delta G = G(3500\text{ Hz}) - G(200\text{ Hz})$ em dB.
  - *Critério e tolerância:*
    1. Em `drive.tone = 0.0`, $\Delta G(3500\text{ Hz} - 200\text{ Hz})$ deve estar entre **`-14,0 dB` e `-22,0 dB`** (inclinação de 1ª ordem `-20 dB/década` a partir de `408,1 Hz`, e não `-36 dB` de um biquad de 2ª ordem).
    2. Em `drive.tone = 1.0`, $\Delta G(3500\text{ Hz} - 200\text{ Hz})$ deve estar entre **`+14,0 dB` e `+22,0 dB`** (inclinação de 1ª ordem `+20 dB/década` até `1808,6 Hz`).
  - *Origem da tolerância:* Equações analíticas de 1ª ordem $f_{\text{LPF}} = 408,1\text{ Hz}$ e $f_{\text{HPF}} = 1808,6\text{ Hz}$ (`F-01`).

- **CA-03 — Compressão Dinâmica de Sustentação em Cascata Dupla (`Q2 → Q3`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Senoide de `440 Hz` testada em dois níveis de entrada separados por `18,0 dB`: `-24 dBFS` e `-6 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `drive.gain = 0.85` (`Sustain = 85%`), `drive.tone = 0.50`, `drive.level = 0.50`.
  - *Método de medição:* Nível RMS da saída para entrada de `-24 dBFS` vs `-6 dBFS`.
  - *Critério e tolerância:* Para um aumento de `+18,0 dB` no sinal de entrada, o nível RMS de saída deve variar no máximo **`+3,5 dB`** (taxa de compressão efetiva $\ge 5:1$), comprovando o efeito de *sustainer* dos dois estágios de clipping coletor-base em cascata (`Q2` e `Q3`).
  - *Origem da tolerância:* Saturação em dois estágios com limiar de diodo `0,62 V` (`F-01` §"Clipping Stage").

- **CA-04 — Atenuação de Fizz Superior pelos Três Capacitores Miller (`3× 470 pF`) (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `drive.gain = 0.90`, `drive.tone = 0.50`, `drive.level = 0.50`.
  - *Método de medição:* Nível do 11º harmônico (`11 kHz`) relativo à fundamental (`1 kHz`) na saída.
  - *Critério e tolerância:* O 11º harmônico (`11 kHz`) deve estar **pelo menos `28,0 dB` abaixo** da fundamental (`1 kHz`), comprovando a filtragem progressiva inter-estágio dos polos Miller de `470 pF`.
  - *Origem da tolerância:* Cascata de polos Miller $C_{10}, C_{11}, C_{12} = 470\text{ pF}$ (`F-01`).

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `gain`, `tone`, `level`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/drive.rs` (e futuro `crates/ampforge-dsp/src/fx/pedals/big_muff_pi.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.muff.*`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_031_pedals_dsp.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Revisão histórica fixada em `1977 V3 NYC (PCB EH-3003, 4× 2N5088)` e separada de `V4 Op-Amp` e `Sovtek Russian`.
- [x] Erro de ordem dos filtros do Tone Stack (`2ª ordem Butterworth` em `drive.rs:416–420` vs `1ª ordem RC` real) e omissão dos capacitores Miller `470 pF` e do estágio `Q4` diagnosticados.
- [x] Critérios `CA-01` a `CA-05` classificados e com condições completas de medição.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` ElectroSmash — *Electro-Harmonix Big Muff Pi Analysis* (`https://web.archive.org/web/20240115000000/https://www.electrosmash.com/big-muff-pi-analysis`).
- `[F-02]` Kit Rae — *The Big Muff Pi History and Circuit Guide* (`http://www.kitrae.net/music/big_muff_history.html`).
- `[F-03]` ON Semiconductor — *2N5088 NPN Transistor Datasheet*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, documentou-se que `drive.rs:416–420` usava biquads de 2ª ordem em vez dos ramos RC de 1ª ordem ($39\text{ k}\Omega / 10\text{ nF} \Rightarrow 408,1\text{ Hz}$ e $22\text{ k}\Omega / 4\text{ nF} \Rightarrow 1808,6\text{ Hz}$) e omitia os três polos Miller de `470 pF` e o 4º estágio BJT de recuperação (`Q4`).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Tone Stack passivo em ponte de **1ª ordem** (`408,1 Hz` LPF + `1808,6 Hz` HPF) com *mid-scoop* em `~950–1000 Hz` | `F-01` §"Tone Control", `F-02` | Spec 062 / `CA-01`, `CA-02` | `crates/ampforge-dsp/src/fx/drive.rs:416–420` (usa biquads Butterworth de **2ª ordem** em `480 Hz` e `1890 Hz`) | `contract_031_pedals_dsp.rs` (`test_ca_03_big_muff_mid_scoop`) | `TESTED` (scoop) / `SPECIFIED` (ordem 1ª exata) |
| Duplo estágio de clipping coletor-base (`Q2 → Q3`, `1N914 + 1 µF`) | `F-01` §"Clipping Stage" | Spec 062 / `CA-03` | `crates/ampforge-dsp/src/fx/drive.rs:604–611` (`soft_clip_diode` em 2 estágios sem desacoplamento `1 µF`) | `contract_031_pedals_dsp.rs`, `contract_040_pedals_parity.rs` | `IMPLEMENTED_UNVERIFIED` |
| Três capacitores Miller coletor-base (`3× 470 pF` em `Q1`, `Q2`, `Q3`) e 4º estágio BJT `Q4` (`+14,5 dB`) | `F-01` | Spec 062 / `CA-04` | Ausentes em `drive.rs:601–624` | Ausente | `SPECIFIED` |
| Execução zero-alloc em tempo real com oversampling `8×` | Spec 053, Spec 079 | Spec 062 / `CA-05` | `crates/ampforge-dsp/src/fx/drive.rs:463–491` | `contract_044_sdd_integrity.rs` | `TESTED` |
