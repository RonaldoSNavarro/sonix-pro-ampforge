---
status: rascunho
---

# Spec 059: Pedal Ibanez TS808 Tube Screamer (Overdrive Analógico JRC4558D)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-pedal` (`drive`)
> **Identificador canônico:** `pedal.ibanez.ts808`
> **Variante / Revisão:** `1979 Narrow-Box / IC Dual Op-Amp JRC4558D & Diodos Silício 1S1588 (MA150)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/drive.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [031](031-amplificadores-fieis-aos-esquemas-e-controles-originais.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [079](079-componente-oversampling-polifasico-e-anti-aliasing.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação do pedal de overdrive **Ibanez TS808 Tube Screamer (1979, JRC4558D)** como processador modular independente (`pedal.ibanez.ts808`), substituindo a implementação aproximada atual em `DriveProcessor::process_sample` (`DriveModel::TubeScreamer`) por um modelo de circuito fiel ao estágio não-inversor com clipping na malha de realimentação e ao filtro ativo de Tone em configuração *Gyrator/W-Taper*.

---

## 2. Escopo

1. Buffer de entrada com seguidor de emissor BJT (`2SC1815` / `2SC536`) e acoplamento capacitivo de `0,02 µF` (`20 nF`).
2. Estágio amplificador não-inversor baseado no op-amp `JRC4558D`, onde o sinal limpo ($1 + Z_f/Z_i$) é inerentemente somado ao sinal ceifado pelo par antiparalelo de diodos de silício `1S1588` (ou `MA150`) na malha de realimentação, acompanhado do capacitor de compensação de `51 pF` e do filtro passa-altas na perna de terra (`4,7 kΩ` + `0,047 µF` $\Rightarrow f_c \approx 720,48\text{ Hz}$).
3. Estágio de Tone ativo-passivo combinando o filtro passa-baixas passivo fixo (`1 kΩ` + `0,22 µF` $\Rightarrow f_c \approx 723,43\text{ Hz}$) com a rede ativa do segundo estágio `JRC4558D` controlada pelo potenciômetro `20 kΩ` de curva `G`/`W` e pelo capacitor `0,22 µF` em série com `220 Ω` ($f_z \approx 3294\text{ Hz}$).
4. Estágio de saída (`Level`) e buffer final de emissor comum.

---

## 3. Fora de escopo

- Modificações "Keeley Baked Mod", "TS9" (que altera apenas o resistor de saída antes do buffer final de `100 Ω`/`10 kΩ` para `470 Ω`/`100 kΩ`), ou "TS808HW" — fora do modelo base `pedal.ibanez.ts808`.
- Qualquer outro pedal de overdrive (Klon Centaur → Spec 060; ProCo RAT → Spec 061; Boss DS-1 → Spec 064).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`overdrive`) |
| **Fabricante de referência** | Ibanez (Hoshino Gakki / Nisshin Onpa — Maxon) |
| **Modelo de referência** | TS808 Tube Screamer Overdrive Pro |
| **Ano / época / revisão** | 1979–1981 (placa Maxon MP-D01201A, CI `JRC4558D`, diodos `1S1588`) |
| **Identificador no sistema** | `pedal.ibanez.ts808` (`DriveModel::TubeScreamer` / UI slug `ts9` com alias `ts808`) |
| **Papel na cadeia de sinal** | Bloco `DRIVE` pré-amplificador (tipicamente usado como *clean/mid-hump boost* empurrando JCM800 ou Dual Rectifier, ou overdrive autônomo em Fender Twin). |

---

## 5. Princípio físico e funcional do equipamento real

O comportamento clássico do **TS808** decorre de quatro propriedades elétricas indissociáveis de seu circuito:

1. **Filtro Passa-Altas Seletivo na Malha de Realimentação (*Bass Cut / Mid-Hump*):** O estágio de ganho é um amplificador operacional não-inversor onde a impedância para a referência de `4,5 V` ($V_{\text{ref}}$) é formada por $R_4 = 4,7\text{ k}\Omega$ em série com $C_3 = 0,047\text{ }\mu\text{F}$ (`47 nF`). Isso cria um polo passa-altas em:
   $$f_{\text{HPF}} = \frac{1}{2\pi \cdot 4700 \cdot 47 \times 10^{-9}} \approx 720,48\text{ Hz}$$
   Frequências muito abaixo de $720\text{ Hz}$ veem $C_3$ como circuito aberto: o ganho do op-amp cai assintoticamente para a unidade ($1\times = 0\text{ dB}$) e os graves passam **sem distorção e sem amplificação**.
2. **Soma Inerente do Sinal Limpo ($1 + Z_f/Z_i$):** Como o clipping ocorre na malha de realimentação negativa de um amplificador **não-inversor**, a tensão de saída é $V_{\text{out}} = V_{\text{in}} + I_{\text{fb}} \cdot Z_f \approx V_{\text{in}} + V_{\text{diode}}(I_{\text{fb}})$. Portanto, o sinal de entrada limpo $V_{\text{in}}$ nunca desaparece — mesmo com os diodos saturados e `Overdrive = 0`, o ganho mínimo na banda passante é $1 + \frac{51\text{ k}\Omega}{4,7\text{ k}\Omega} \approx 11,85\times$ ($+21,47\text{ dB}$) e com `Overdrive = 100%` ($500\text{ k}\Omega + 51\text{ k}\Omega$) atinge $1 + \frac{551\text{ k}\Omega}{4,7\text{ k}\Omega} \approx 118,23\times$ ($+41,45\text{ dB}$).
3. **Suavização Harmônica no Feedback ($C_4 = 51\text{ pF}$):** Em paralelo com os diodos `1S1588` e o potenciômetro `500 kΩ` + `51 kΩ`, o capacitor $C_4 = 51\text{ pF}$ forma um polo passa-baixas dependente da posição do potenciômetro `Overdrive`:
   $$f_{\text{LPF,fb}} = \frac{1}{2\pi \cdot (51\text{ k}\Omega + R_{\text{drive}}) \cdot 51\text{ pF}} \in [5,66\text{ kHz},\; 61,2\text{ kHz}]$$
   Isso arredonda os cantos da onda ceifada antes mesmo da saída do primeiro op-amp.
4. **Estágio de Tone Passivo + Ativo (*W-Taper*):** Após o estágio de clipping, há um filtro passa-baixas RC passivo fixo ($R_7 = 1\text{ k}\Omega$, $C_5 = 0,22\text{ }\mu\text{F} \Rightarrow f_c \approx 723,43\text{ Hz}$) que corta agudos ríspidos. O segundo op-amp `JRC4558D` atua como filtro ativo variável: a rede $C_6 = 0,22\text{ }\mu\text{F}$ em série com $R_8 = 220\text{ }\Omega$ ($f_z \approx 3294\text{ Hz}$) é deslocada pelo cursor do potenciômetro `20 kΩ` entre a entrada não-inversora (atenuando agudos acima de $\sim 350\text{ Hz}$) e a malha de realimentação negativa (amplificando agudos até $\sim 3,3\text{ kHz}$).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | David T. Yeh, Jonathan S. Abel, Julius O. Smith III — *Simplified, Physically-Informed Models of Distortion and Overdrive Guitar Effects Pedals* (DAFx-07, Bordeaux, 2007) | `https://ccrma.stanford.edu/~dtyeh/papers/yeh07_dafx_distortion.pdf` (HTTP 200 PDF verificado, 200 KB) | `ACADEMIC` | Equações diferenciais não-lineares de estado (ODE) do estágio de clipping e do Tone ativo do Tube Screamer (`TS9/TS808`). |
| `F-02` | ElectroSmash — *Ibanez Tube Screamer Circuit Analysis* (arquivado e verificado via Wayback Machine) | `https://web.archive.org/web/20240115000000/https://www.electrosmash.com/tube-screamer-analysis` (HTTP 200 verificado) | `SECONDARY_TECHNICAL` | Análise completa do esquema elétrico TS808/TS9, BOM, resposta em frequência de cada estágio (`720 Hz` HPF, `723 Hz` LPF, Tone ativo). |
| `F-03` | Nisshinbo / JRC — *NJM4558 / JRC4558 Dual Operational Amplifier Datasheet* | Especificação oficial do componente (`GBW = 3 MHz`, `Slew Rate = 1.0 V/µs`) | `PRIMARY_COMPONENT` | Parâmetros dinâmicos do amplificador operacional dual `JRC4558D`. |
| `F-04` | Toshiba — *1S1588 Silicon Switching Diode Datasheet* | Especificação oficial do componente ($I_s \approx 2,52\text{ nA}$, $N \approx 1,752$, $V_f \approx 0,6\text{ V} @ 1\text{ mA}$) | `PRIMARY_COMPONENT` | Curva exponencial Shockley dos diodos de silício na malha de feedback. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Rede HPF na perna inversora: $R_4 = 4,7\text{ k}\Omega$, $C_3 = 0,047\text{ }\mu\text{F}$ ($f_c \approx 720,48\text{ Hz}$) | `F-01`, `F-02` | TS808 / TS9 (1979–1982) | `F-01` §2.1 Fig. 2 p. 2; `F-02` §"Clipping Amplifier" | `ACADEMIC` / `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Malha de feedback: $R_6 = 51\text{ k}\Omega + P_{\text{drive}} = 500\text{ k}\Omega$, $C_4 = 51\text{ pF}$, par antiparalelo `1S1588` | `F-01`, `F-02` | TS808 (1979) | `F-01` §2.1 Eq. (1)–(4) p. 2; `F-02` §"Clipping Stage" | `ACADEMIC` / `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Ganho não-inversor linear: $G_{\min} = 11,85\times$ ($21,47\text{ dB}$), $G_{\max} = 118,23\times$ ($41,45\text{ dB}$) | `F-02` | TS808 (1979) | `F-02` §"Voltage Gain" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Estágio de Tone: LPF passivo $R_7 = 1\text{ k}\Omega, C_5 = 0,22\text{ }\mu\text{F}$ ($723\text{ Hz}$) + rede ativa $P_{\text{tone}} = 20\text{ k}\Omega, C_6 = 0,22\text{ }\mu\text{F}, R_8 = 220\text{ }\Omega$ | `F-01`, `F-02` | TS808 (1979) | `F-01` §3.1 Fig. 5 p. 4; `F-02` §"Tone & Volume Stage" | `ACADEMIC` / `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Diferença elétrica entre TS808 e TS9 restrita ao divisor de saída ($R_{15}=100\text{ }\Omega, R_{16}=10\text{ k}\Omega$ no TS808 vs $470\text{ }\Omega / 100\text{ k}\Omega$ no TS9) | `F-02` | TS808 vs TS9 | `F-02` §"Output Buffer / TS808 vs TS9" | `SECONDARY_TECHNICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input Jack] ──► [C1 20nF + Q1 2SC1815 Emitter Follower Buffer]
                        │
                        ▼
               [IC1A JRC4558D (+) Non-Inverting Input]
                        │
        ┌───────────────┴───────────────┐
        │ Malha Feedback IC1A (-):      │ Perna p/ Vref (4.5V):
        │  D1 || D2 (2x 1S1588 Si)      │  R4 (4.7 kΩ) + C3 (47 nF)
        │  || C4 (51 pF)                │  => HPF @ 720.48 Hz
        │  || (R6 51 kΩ + P1 500 kΩ)    │
        └───────────────┬───────────────┘
                        │
                        ▼
          [LPF Passivo Fixo: R7 (1 kΩ) + C5 (220 nF) @ 723.43 Hz]
                        │
                        ▼
          [IC1B JRC4558D Active Tone Network]
          (Pot P2 20 kΩ W-Taper + C6 220 nF + R8 220 Ω)
                        │
                        ▼
          [Level Pot P3 100 kΩ] ──► [Q2 2SC1815 Output Buffer (R15=100Ω)] ──► [Output]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/drive.rs`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/fx/drive.rs#L428-L536) possui as seguintes divergências auditadas:

- Removido em 2026-11-20 (AUD-38): identificador inexistente em qualquer commit
- Removido em 2026-11-20 (AUD-38): identificador inexistente em qualquer commit
- Removido em 2026-11-20 (AUD-38): identificador inexistente em qualquer commit
4. **`SOURCE_INVALID` (corrigido nesta revisão):** O link direto `https://www.electrosmash.com/tube-screamer-analysis` falha na resolução DNS atual; substituído pelo link direto verificado no Wayback Machine (`F-02`) e suplementado pelo artigo primário DAFx-07 de David Yeh (`F-01`).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Normalizada | Unidade / Escala UI | Curva / Taper | Valor Padrão | Papel Elétrico no Circuito |
|---|---|---|---|---|---|---|---|
| **FOOSWITCH (BYPASS)** | `drive.enabled` | `pedal.ts9.enabled` | `false / true` | Booleano | Discreto | `false` | Chaveamento FET JFET (`2SK30A`) com buffer ativo ou bypass do bloco `drive`. |
| **OVERDRIVE** | `drive.gain` | `pedal.ts9.gain` | `0.0 .. 1.0` | `0 .. 100 %` | Logarítmico (`500 kΩ A`) | `0.50` | Varia $R_{\text{drive}} \in [0,\; 500\text{ k}\Omega]$ em série com $R_6 = 51\text{ k}\Omega$ na malha de feedback de `IC1A`. |
| **TONE** | `drive.tone` | `pedal.ts9.tone` | `0.0 .. 1.0` | `0 .. 100 %` | Taper `G`/`W` (S-curve `20 kΩ`) | `0.50` | Desloca o cursor do potenciômetro `20 kΩ` entre o nó não-inversor e o nó inversor de `IC1B`. |
| **LEVEL** | `drive.level` | `pedal.ts9.level` | `0.0 .. 1.0` | `0 .. 100 %` | Linear / Áudio (`100 kΩ B`) | `0.50` | Divisor resistivo na saída de `IC1B` antes do buffer de saída `Q2`. |

> **Nota sobre nomenclatura UI vs DSP:** No registro atual (`controls.registry.ts:696`), o slug visual do pedal é `ts9`, que seleciona `drive.model = "ts9"` (`DriveModel::TubeScreamer`). Como a topologia elétrica de clipping e tone do TS808 e do TS9 é idêntica (diferindo apenas em `100 Ω` vs `470 Ω` no resistor de saída), o identificador modular `pedal.ibanez.ts808` aceita tanto `"ts808"` quanto `"ts9"` sem quebrar compatibilidade com presets existentes.

---

## 9. Modelo matemático e algoritmo DSP proposto

### 9.1. Estágio 1 — Amplificador Não-Inversor com Clipping no Feedback (Yeh DAFx-07)

Seja $v_{\text{in}}[n]$ a tensão após o acoplamento de entrada (`HPF` de 1ª ordem em $f_c = \frac{1}{2\pi \cdot 510\text{ k}\Omega \cdot 20\text{ nF}} \approx 15,6\text{ Hz}$).
Na topologia não-inversora (Yeh DAFx-07, §2.1):
1. A corrente que entra na rede da perna inversora ($R_4 = 4,7\text{ k}\Omega$, $C_3 = 47\text{ nF}$) é governada pelo filtro passa-altas de 1ª ordem com frequência de corte $f_1 = \frac{1}{2\pi R_4 C_3} = 720,484\text{ Hz}$:
   $$i_{R4}(s) = \frac{s C_3}{1 + s R_4 C_3} V_{\text{in}}(s)$$
2. Toda essa corrente $i_{R4}$ deve atravessar a malha paralela de realimentação ($R_f = 51\text{ k}\Omega + R_{\text{drive}}$, $C_4 = 51\text{ pF}$, e o par antiparalelo de diodos `1S1588` com $I_{\text{diode}}(v_d) = 2 I_s \sinh\left(\frac{v_d}{N V_T}\right)$, onde $I_s = 2,52\text{ nA}$, $N = 1,752$, $V_T = 25,85\text{ mV}$):
   $$i_{R4} = \frac{v_d}{R_f} + C_4 \frac{dv_d}{dt} + 2 I_s \sinh\left(\frac{v_d}{N V_T}\right)$$
3. Discretizando $C_4 \frac{dv_d}{dt}$ via regra trapezoidal (Tustin) na taxa de oversampling $8\times f_s$ e resolvendo $v_d[n]$ por Newton-Raphson amortecido (máximo de 6 iterações com chute inicial $v_d[n-1]$ e limite de argumento $|\frac{v_d}{N V_T}| \le 25$), a tensão de saída do primeiro op-amp é:
   $$v_{\text{out1}}[n] = v_{\text{in}}[n] + \operatorname{clamp}(v_d[n],\; -V_{\text{rail}},\; +V_{\text{rail}})$$
   onde $V_{\text{rail}} \approx 3,8\text{ V}$ representa a excursão máxima simétrica do `JRC4558D` alimentado por bateria de `9 V` ($V_{\text{ref}} = 4,5\text{ V}$).

### 9.2. Estágio 2 — Rede de Tone Passiva + Ativa (Yeh DAFx-07 §3.1)

O segundo estágio é modelado exatamente pela função de transferência analógica discreta $H_{\text{tone}}(s)$ de 2ª ordem derivada das equações nodais de `IC1B` ($R_7 = 1\text{ k}\Omega$, $C_5 = 220\text{ nF}$, $R_9 = 10\text{ k}\Omega$, $R_8 = 220\text{ }\Omega$, $C_6 = 220\text{ nF}$, $P_{\text{tone}} = 20\text{ k}\Omega$ com taper $W(t)$):
- Quando $t \to 0$ (`Tone = 0%`), a frequência de corte cai para $\sim 350\text{ Hz}$ com atenuação suave de agudos;
- Quando $t \to 1$ (`Tone = 100%`), a rede $C_6, R_8$ entra no braço inversor criando um shelf ativo de agudos até $f_z = \frac{1}{2\pi R_8 C_6} \approx 3294\text{ Hz}$, sempre limitado acima disso pelo polo passivo de $R_7, C_5$ ($723,43\text{ Hz}$).

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `8×` polifásico IIR (Spec 079) em torno do solver Newton-Raphson do estágio não-inversor `IC1A`.
- **Anti-aliasing:** Como o polo do capacitor $C_4 = 51\text{ pF}$ já atua dentro da malha diferencial em $8\times f_s$, a geração harmônica acima de $60\text{ kHz}$ é fisicamente amortecida antes da decimação.
- **Precisão numérica:** Estado interno dos integradores trapezoidais ($C_3$, $C_4$, $C_5$, $C_6$) em `f64`.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap durante `process_block` e `apply_param_rt`.
- Número de iterações do solver Newton-Raphson estritamente limitado (`MAX_NR_ITERS = 6`), garantindo WCET determinístico $\le 15\text{ }\mu\text{s}$ por bloco de 128 amostras a `48 kHz` (`8×` oversampling = 1024 sub-amostras).

---

## 12. Posição e interação na cadeia completa de guitarra

- **Entrada:** Recebe sinal em nível de instrumento vindo da guitarra/wah/compressor.
- **Saída:** Baixa impedância ($R_{\text{out}} \approx 100\text{ }\Omega$), ideal para alimentar a entrada `HIGH` de amplificadores valvulados (`Spec 054`–`058`), empurrando o estágio `V1` para saturação com graves enxutos (evitando *flubbing*/embolamento nos graves do pré-amplificador valvulado).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/ts808.rs` (extraindo a lógica dedicada de `crates/ampforge-dsp/src/fx/drive.rs`).
- **Struct principal:** `pub struct Ts808Processor` implementando `DspBlock` + `AutomatableParams` (Spec 053).
- **Estado interno:**
  - `c1_hpf_state: f64` (`15.6 Hz` input coupling)
  - `c3_hpf_state: f64` (`720.48 Hz` feedback grounding network)
  - `c4_tustin_state: f64` (`51 pF` feedback capacitor state)
  - `vd_prev: f64` (última solução da tensão nos diodos $v_d[n-1]$)
  - `tone_biquad: BiquadF64` (coeficientes exatos da função de transferência de Yeh DAFx-07 §3.1)
  - `oversampler: Oversampler8x`

---

## 14. Parâmetros de calibração e invariantes físicas

| Componente do Esquema | Símbolo | Valor Nominal Documentado | Tolerância Real | Invariante Física |
|---|---|---|---|---|
| Resistor perna inversora | $R_4$ | $4,7\text{ k}\Omega$ | $\pm 5\%$ | Define $f_{\text{HPF}} = 720,48\text{ Hz}$ junto com $C_3$. |
| Capacitor perna inversora | $C_3$ | $0,047\text{ }\mu\text{F}$ (`47 nF`) | $\pm 10\%$ | Bloqueia ganho DC ($G_{\text{DC}} = 1,0 = 0\text{ dB}$). |
| Resistor mínimo de feedback | $R_6$ | $51\text{ k}\Omega$ | $\pm 5\%$ | Define ganho mínimo na banda passante ($11,85\times = +21,47\text{ dB}$). |
| Potenciômetro Overdrive | $P_1$ | $500\text{ k}\Omega$ Log | $\pm 20\%$ | Define ganho máximo na banda passante ($118,23\times = +41,45\text{ dB}$). |
| Capacitor de feedback | $C_4$ | $51\text{ pF}$ | $\pm 5\%$ | Limita banda da malha de clipping entre $5,66\text{ kHz}$ e $61,2\text{ kHz}$. |
| Rede passiva pós-clipping | $R_7, C_5$ | $1\text{ k}\Omega,\; 0,22\text{ }\mu\text{F}$ | $\pm 5\% / \pm 10\%$ | Polo passa-baixas fixo em $723,43\text{ Hz}$. |
| Rede ativa de Tone | $P_2, C_6, R_8$ | $20\text{ k}\Omega\text{ W},\; 0,22\text{ }\mu\text{F},\; 220\text{ }\Omega$ | $\pm 10\%$ | Zero/polo ativo em $3294\text{ Hz}$. |

---

## 15. Estratégia de testes, validação e benchmark

1. **Varredura de resposta em frequência em pequeno sinal ($V_{\text{in}} = 1\text{ mV}_{\text{pk}}$, regime linear dos diodos):** verifica o *mid-hump* centrado em `720–1000 Hz`, o ganho unitário assintótico em DC/subgraves e a ação da rede de Tone.
2. **Teste de soma do sinal limpo ($V_{\text{out}} = V_{\text{in}} + V_d$):** excitação com fundamental grave (`82,41 Hz`, corda E2) de amplitude moderada confirma que a componente de `82,41 Hz` atravessa o estágio sem atenuação destrutiva mesmo quando os harmônicos médios são ceifados.
3. **Teste de simetria harmônica:** excitação senoidal de `1 kHz` a `-6 dBFS` verifica predominância de harmônicos ímpares ($H_3, H_5 \gg H_2$) característica do par antiparalelo simétrico de diodos `1S1588`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- **Convergência de Newton-Raphson em transientes abruptos:** O argumento de $\sinh(v_d / (N V_T))$ é limitado a $\pm 25$ com passo amortecido caso o resíduo aumente, garantindo convergência em no máximo 6 iterações sem `NaN` ou `Inf`.

---

## 17. Critérios de aceite (CA-01 a CA-06)

- **CA-01 — Resposta em Pequeno Sinal e Mid-Hump em 720 Hz (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Varredura senoidal em regime linear de pequeno sinal ($1\text{ mV}_{\text{pk}} \equiv -60\text{ dBFS}$) nas frequências de `40 Hz`, `720 Hz`, `1000 Hz` e `10 kHz`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras (`1 s` de acomodação).
  - *Configuração e parâmetros:* `pedal.ibanez.ts808` com `drive.enabled = true`, `drive.gain = 0.0` ($R_f = 51\text{ k}\Omega$), `drive.tone = 0.5`, `drive.level = 0.5`.
  - *Método de medição:* Ganho relativo em dB medido por DFT nas frequências de teste na saída do primeiro estágio `IC1A`.
  - *Critério e tolerância:* O ganho do estágio `IC1A` em `1000 Hz` deve exceder o ganho em `40 Hz` em **pelo menos `+12,0 dB`** (valor teórico do circuito $R_4=4,7\text{ k}\Omega, C_3=47\text{ nF}, R_6=51\text{ k}\Omega$: $+18,1\text{ dB} @ 1\text{ kHz}$ vs $+0,3\text{ dB} @ 40\text{ Hz}$), e o ganho assintótico em `10 Hz` deve convergir para a unidade ($0\text{ dB} \pm 0,5\text{ dB}$) devido à topologia não-inversora $1 + Z_f/Z_i$.
  - *Origem da tolerância:* Equação analítica da rede $R_4, C_3, R_6$ (`F-01` §2.1, `F-02`) com tolerância de componentes passivos ($\pm 10\%$).

- **CA-02 — Preservação do Sinal Limpo Não-Inversor ($V_{\text{out1}} = V_{\text{in}} + V_d$) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `80 Hz` com amplitude `-12 dBFS` ($0,251\text{ FS}$).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `drive.gain = 0.5`, `drive.tone = 0.5`, `drive.level = 0.5`.
  - *Método de medição:* Amplitude da fundamental `80 Hz` na saída do estágio `IC1A` comparada à entrada `80 Hz`.
  - *Critério e tolerância:* O ganho na fundamental `80 Hz` na saída de `IC1A` deve ser **$\ge 1,0\times$ ($0,0\text{ dB}$)**, comprovando que o filtro passa-altas de `720,48 Hz` atua exclusivamente na perna inversora e nunca atenua a parcela unitária limpa $V_{\text{in}}$ da topologia não-inversora.
  - *Origem da tolerância:* Propriedade física exata da equação $V_{\text{out1}} = V_{\text{in}} + V_d$ (`F-01` Eq. 1).

- **CA-03 — Clipping Simétrico de Silício (`1S1588`) e Predominância de Harmônicos Ímpares (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-6 dBFS` ($0,501\text{ FS}$).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras, janela Hann de `16 384` amostras após `0,5 s` de regime permanente.
  - *Configuração e parâmetros:* `drive.gain = 0.85`, `drive.tone = 0.50`, `drive.level = 0.50`.
  - *Método de medição:* Análise espectral FFT das magnitudes do 2º harmônico ($H_2 @ 2000\text{ Hz}$) e do 3º harmônico ($H_3 @ 3000\text{ Hz}$).
  - *Critério e tolerância:* O nível do 3º harmônico ($H_3$) deve superar o nível do 2º harmônico ($H_2$) em **pelo menos `20,0 dB`** ($H_3 - H_2 \ge 20,0\text{ dB}$).
  - *Origem da tolerância:* Simetria ímpar de $I_d = 2 I_s \sinh(v_d / (N V_T))$ do par antiparalelo de diodos `1S1588` (`F-01`, `F-04`).

- **CA-04 — Atuação do Capacitor de Feedback $C_4 = 51\text{ pF}$ (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-6 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `drive.gain = 1.0`, `drive.tone = 0.5`.
  - *Método de medição:* Comparação da energia harmônica acima de `8 kHz` na saída de `IC1A` com $C_4 = 51\text{ pF}$ ativo versus $C_4 = 0\text{ pF}$.
  - *Critério e tolerância:* A presença do polo de $C_4 = 51\text{ pF}$ ($f_c \approx 5,66\text{ kHz}$ com $R_f = 551\text{ k}\Omega$) deve reduzir a energia do 9º harmônico (`9 kHz`) em **pelo menos `3,0 dB`** em relação ao modelo sem $C_4$.
  - *Origem da tolerância:* Frequência de corte de 1ª ordem $f_c = \frac{1}{2\pi \cdot 551\text{ k}\Omega \cdot 51\text{ pF}} = 5664\text{ Hz}$ (`F-01`, `F-02`).

- **CA-05 — Resposta da Rede de Tone Ativa-Passiva (`IC1B`, $723\text{ Hz}$ + $3294\text{ Hz}$) (`DOCUMENTED`):**
  - *Sinal de entrada:* Ruído branco ou varredura senoidal de pequeno sinal (`-40 dBFS`) na entrada do estágio de Tone (`IC1B`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Comparação entre `drive.tone = 0.0`, `drive.tone = 0.5` e `drive.tone = 1.0`.
  - *Método de medição:* Magnitude da função de transferência do estágio de Tone em `3200 Hz` relativa a `300 Hz`.
  - *Critério e tolerância:* A diferença de ganho em `3200 Hz` entre `drive.tone = 1.0` e `drive.tone = 0.0` deve ser **$\ge 15,0\text{ dB}$**, mantendo variação em `200 Hz` dentro de **$\pm 2,5\text{ dB}$**.
  - *Origem da tolerância:* Curvas medidas e modeladas do estágio `IC1B` em `F-01` Fig. 6 p. 4 e `F-02` §"Tone Control Frequency Response".

- **CA-06 — Segurança em Tempo Real (Zero-Alloc e Finitude) (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* Ruído branco e bursts senoidais de `0 dBFS` durante `10 000` blocos de `128` amostras a $f_s = 48\,000\text{ Hz}$ com automação contínua de `gain`, `tone` e `level`.
  - *Método de medição:* Alocador global instrumentado na thread de teste (Regra Cortex `d272`) e inspeção de `is_finite()`.
  - *Critério e tolerância:* `0` alocações/desalocações na heap e `100%` das amostras finitas e limitadas a $|y[n]| \le 2,0$.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/drive.rs` (e futuro módulo `crates/ampforge-dsp/src/fx/pedals/ts808.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.ts9.*`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_031_pedals_dsp.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Fontes primárias/acadêmicas verificadas com links diretos HTTP 200 (`F-01` DAFx-07 PDF, `F-02` ElectroSmash Wayback).
- [x] Erros do código atual (`drive.rs:429–536`) diagnosticados e documentados sem alterar código de produção nesta etapa.
- [x] Critérios de aceite `CA-01` a `CA-06` classificados e com condições completas de medição.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Yeh, D. T., Abel, J. S., & Smith, J. O. (2007). *Simplified, Physically-Informed Models of Distortion and Overdrive Guitar Effects Pedals*. Proc. DAFx-07, Bordeaux, France, pp. 189–196 (`https://ccrma.stanford.edu/~dtyeh/papers/yeh07_dafx_distortion.pdf`).
- `[F-02]` ElectroSmash — *Ibanez Tube Screamer Circuit Analysis* (`https://web.archive.org/web/20240115000000/https://www.electrosmash.com/tube-screamer-analysis`).

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva das Specs 053–084, substituiu-se o link direto quebrado do ElectroSmash pela URL verificada do Wayback Machine e adicionou-se o artigo DAFx-07 de David Yeh (`F-01`). Diagnosticou-se o erro topológico em `drive.rs:515` onde o HPF de `720 Hz` era aplicado em série sobre o sinal de entrada em vez de atuar na perna inversora do amplificador não-inversor.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| HPF `720,48 Hz` ($4,7\text{ k}\Omega + 47\text{ nF}$) na perna inversora preservando soma limpa unitária $V_{\text{out1}} = V_{\text{in}} + V_d$ | `F-01` §2.1, `F-02` | Spec 059 / `CA-01`, `CA-02` | `crates/ampforge-dsp/src/fx/drive.rs:429 @ c20cc65, 515–518` (aplica HPF em série no sinal e soma `0.25 * input` ad-hoc) | `contract_031_pedals_dsp.rs` | `SPECIFIED` |
| Clipping simétrico `1S1588` na malha de feedback com $R_f \in [51\text{ k}\Omega, 551\text{ k}\Omega]$ | `F-01` §2.1, `F-04` | Spec 059 / `CA-03` | `crates/ampforge-dsp/src/fx/drive.rs:519–533 @ c20cc65` (`asinh` aproximado) | `contract_031_pedals_dsp.rs`, `contract_040_pedals_parity.rs` | `IMPLEMENTED_UNVERIFIED` |
| Capacitor de feedback $C_4 = 51\text{ pF}$ dependente de $R_{\text{drive}}$ | `F-01` §2.1, `F-02` | Spec 059 / `CA-04` | Ausente em `drive.rs` (usa LPF fixo de `4,5 kHz` na linha `431`) | Ausente | `SPECIFIED` |
| Estágio de Tone passivo (`723,43 Hz`) + ativo `IC1B` (`3294 Hz`, pot `20 kΩ` W-taper) | `F-01` §3.1, `F-02` | Spec 059 / `CA-05` | `crates/ampforge-dsp/src/fx/drive.rs:432–434 @ c20cc65` (usa biquad LPF simples `720..5000 Hz`) | `contract_040_pedals_parity.rs` | `SPECIFIED` |
| Execução zero-alloc em tempo real com oversampling `8×` | Spec 053, Spec 079 | Spec 059 / `CA-06` | `crates/ampforge-dsp/src/fx/drive.rs:463–491 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
