---
status: rascunho
---

# Spec 064: Pedal Boss DS-1 Distortion (Revisão 1978–1993 MIJ Toshiba TA7136AP)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-pedal` (`drive` / `distortion`)
> **Identificador canônico:** `pedal.boss.ds1`
> **Variante / Revisão:** `1978–1993 Made in Japan / Early Taiwan (Preamp SIP IC Toshiba TA7136AP, Diodos 1S1588 / 1N4148)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/drive.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [031](031-amplificadores-fieis-aos-esquemas-e-controles-originais.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [079](079-componente-oversampling-polifasico-e-anti-aliasing.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação do pedal **Boss DS-1 Distortion (Revisão 1978–1993 com CI SIP Toshiba `TA7136AP`)** como processador modular independente (`pedal.boss.ds1`), eliminando o termo vago "Rev. Clássica", separando-a explicitamente da revisão pós-1994 (`DS-1A` com dual op-amp `Rohm BA728N` / `Mitsubishi M5223AL` / `NJM2904`), e corrigindo a implementação do **Tone Stack Passivo em Ponte Assimétrica (*Scooped Passive Bridge*)** atualmente reduzido a dois biquads em [`crates/ampforge-dsp/src/fx/drive.rs:445–456 @ c20cc65`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/fx/drive.rs#L445-L456).

---

## 2. Escopo

1. Buffer de entrada BJT (`Q1 2SC732TM`) e estágio de ataque Transistor Booster (`Q2 2SC732TM`, filtro passa-altas seletivo centrado acima de `~400 Hz`).
2. Estágio de ganho variável baseado no amplificador operacional SIP Toshiba `TA7136AP` com potenciômetro `DIST` (`100 kΩ B`) variando simultaneamente o ganho e a resistência da perna inversora ($C_7 = 0,47\text{ }\mu\text{F}$) + capacitor de compensação $C_8 = 250\text{ pF}$.
3. Estágio de hard-clipping paralelo de silício (`D4`, `D5` — `1S1588` / `1N4148`) com resistor série $R_{14} = 2,2\text{ k}\Omega$ e capacitor shunt $C_{10} = 0,01\text{ }\mu\text{F}$ (`10 nF`).
4. **Tone Stack Passivo em Ponte Assimétrica (*Scooped Bridge*):** Rede passiva de 1ª ordem formada por $R_{15} = 6,8\text{ k}\Omega$, $C_{11} = 0,022\text{ }\mu\text{F}$ (`22 nF`), $R_{16} = 2,2\text{ k}\Omega$, $C_{12} = 0,1\text{ }\mu\text{F}$ (`100 nF`), $R_{17} = 6,8\text{ k}\Omega$ e potenciômetro `TONE` (`20 kΩ B`), que produz um vale de médios centrado em **`~500 Hz`** (`-6 dB a -12 dB`).
5. Controle `LEVEL` (`100 kΩ B`) e buffer de saída BJT (`Q3`).

---

## 3. Fora de escopo

- **Revisão 1994–presente (`DS-1A`, PCB com Dual Op-Amp `BA728N` / `M5223AL` / `NJM2904`):** Após a descontinuação do CI SIP Toshiba `TA7136AP` em 1994, a Roland/Boss redesenhou a placa usando um op-amp duplo padrão alimentado simetricamente sem a mesma curva de sobrecarga de trilho e compensação de `33 pF` nos pinos 5–6 do `TA7136AP`.
- **DS-1W (Waza Craft modo Custom) ou DS-1X (MDP digital) ou Keeley Ultra/Seeing Eye Mod:** Fora do escopo do modelo clássico `pedal.boss.ds1`.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`distortion`) |
| **Fabricante de referência** | Boss Corporation / Roland Corporation (Hamamatsu, Japão) |
| **Modelo de referência** | Boss DS-1 Distortion |
| **Ano / época / revisão** | **1978–1993 Revisão Original (`TA7136AP` SIP 7-pin Preamp IC, Diodos `1S1588`)** |
| **Identificador no sistema** | `pedal.boss.ds1` (`DriveModel::BossDs1` / UI slug `ds1`) |
| **Papel na cadeia de sinal** | Bloco `DRIVE` pré-amplificador (distortion de ataque rápido, graves contidos pelo `Q2` booster e *mid-scoop* em `~500 Hz`). |

---

## 5. Princípio físico e funcional do equipamento real

1. **Estágio Transistor Booster `Q2` (`2SC732TM`) — Pré-Ênfase de Ataque:**
   - Antes mesmo de chegar ao CI `TA7136AP`, o sinal atravessa um amplificador BJT emissor-comum (`Q2`) cujo emissor possui $R_7 = 2,2\text{ k}\Omega$ bypassado por um capacitor pequeno $C_5 = 68\text{ nF}$ (`0,068 µF`) e resistor de coletor $R_6 = 10\text{ k}\Omega$ com capacitor de realimentação $C_4 = 250\text{ pF}$.
   - O bypass de emissor de `68 nF` eleva o ganho acima de $f_c \approx \frac{1}{2\pi \cdot 470 \cdot 68 \times 10^{-9}} \approx 4,98\text{ kHz}$ (com transição começando em $\sim 350\text{ Hz}$), fornecendo até **`+17 dB`** de pré-amplificação nos médios/agudos enquanto mantém subgraves enxutos.
2. **Estágio de Ganho Variável com Potenciômetro `DIST` em Config. *Split-Wiper* (`TA7136AP`):**
   - No DS-1, as duas extremidades do potenciômetro `DIST` (`100 kΩ B`) estão conectadas uma à realimentação negativa (com $C_8 = 250\text{ pF}$ em paralelo) e a outra à perna de terra em série com $R_9 = 4,7\text{ k}\Omega$ e $C_7 = 0,47\text{ }\mu\text{F}$, enquanto o cursor (*wiper*) vai à entrada inversora do `TA7136AP`!
   - Consequentemente, girar `DIST` de `0%` a `100%` altera **simultaneamente** o numerador ($R_{\text{fb}} = \alpha \cdot 100\text{ k}\Omega$) e o denominador ($R_{\text{gnd}} = 4,7\text{ k}\Omega + (1 - \alpha) \cdot 100\text{ k}\Omega$) do ganho não-inversor:
     $$G_{\text{TA7136}}(\alpha) = 1 + \frac{\alpha \cdot 100\text{ k}\Omega}{4,7\text{ k}\Omega + (1 - \alpha) \cdot 100\text{ k}\Omega} \in [1,0\times\;(0\text{ dB}),\; 22,28\times\;(+26,96\text{ dB})]$$
   - Além disso, a frequência de corte passa-altas da perna inversora ($C_7 = 0,47\text{ }\mu\text{F}$) **desloca-se dinamicamente** com o knob `DIST` de $f_{\text{HPF}}(\alpha=0) = \frac{1}{2\pi \cdot 104,7\text{ k}\Omega \cdot 0,47\text{ }\mu\text{F}} \approx 3,23\text{ Hz}$ até $f_{\text{HPF}}(\alpha=1) = \frac{1}{2\pi \cdot 4,7\text{ k}\Omega \cdot 0,47\text{ }\mu\text{F}} \approx 72,05\text{ Hz}$!
3. **Hard-Clipping Shunt de Silício (`D4`, `D5` — `1S1588`) + Capacitor Shunt $C_{10} = 10\text{ nF}$:**
   - A saída do `TA7136AP` passa por $C_9 = 0,47\text{ }\mu\text{F}$ e $R_{14} = 2,2\text{ k}\Omega$ para o par antiparalelo `1S1588` em paralelo com $C_{10} = 10\text{ nF}$ ($f_{\text{LPF,shunt}} = \frac{1}{2\pi \cdot 2200 \cdot 10 \times 10^{-9}} \approx 7234\text{ Hz}$ quando os diodos não estão conduzindo).
4. **Tone Stack Passivo em Ponte Assimétrica (*Scooped Passive Bridge* — Notch em `~500 Hz`):**
   - A saída ceifada divide-se em dois braços passivos de 1ª ordem que alimentam as extremidades do potenciômetro `TONE` (`20 kΩ B`):
     - **Braço Superior (Passa-Baixas):** Resistor $R_{15} = 6,8\text{ k}\Omega$ seguido de capacitor $C_{11} = 22\text{ nF}$ para o terra ($f_{\text{LPF}} = \frac{1}{2\pi \cdot 6800 \cdot 22 \times 10^{-9}} \approx 1063,7\text{ Hz}$), conectado ao terminal anti-horário (`CCW`) do pot `TONE` e carregado por $R_{17} = 6,8\text{ k}\Omega$ para o terra.
     - **Braço Inferior (Passa-Altas):** Capacitor $C_{12} = 100\text{ nF}$ (`0,1 µF`) seguido de resistor $R_{16} = 2,2\text{ k}\Omega$ para o nó do terminal horário (`CW`) do pot `TONE` ($f_{\text{HPF}} \approx 723,4\text{ Hz}$ interagindo com a ponte).
   - A interação resistivo-capacitiva dos dois braços produz um **vale de médios (*mid-scoop*) permanente centrado em torno de `500 Hz`** (`-6 dB a -12 dB`), que é mais raso que o do Big Muff e posicionado uma oitava abaixo (`500 Hz` no DS-1 vs `1000 Hz` no Big Muff).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | David T. Yeh, Jonathan S. Abel, Julius O. Smith III — *Simplified, Physically-Informed Models of Distortion and Overdrive Guitar Effects Pedals* (DAFx-07, Bordeaux, 2007) | `https://ccrma.stanford.edu/~dtyeh/papers/yeh07_dafx_distortion.pdf` (HTTP 200 PDF verificado, 200 KB) | `ACADEMIC` | Modelagem física de estado (ODE) e função de transferência exata do Boss DS-1 (§2.2 Clipping Stage, §3.2 Tone Stack Fig. 7–8 pp. 3–5). |
| `F-02` | ElectroSmash — *Boss DS-1 Distortion Analysis* (arquivado e verificado via Wayback Machine) | `https://web.archive.org/web/20240115000000/https://www.electrosmash.com/boss-ds1-analysis` (HTTP 200 verificado) | `SECONDARY_TECHNICAL` | Esquema completo da revisão `TA7136AP` vs revisão `1994+ Dual Op-Amp`, análise do transistor booster `Q2`, potenciômetro split-wiper `DIST` e Tone Stack (`500 Hz` mid-scoop). |
| `F-03` | Boss Corporation — *Boss DS-1 Distortion Owner's Manual & Specifications* | `https://www.boss.info/global/support/by_product/ds-1/owners_manuals/` (HTTP 200 verificado) | `PRIMARY_OFFICIAL` | Controles oficiais (`TONE`, `LEVEL`, `DIST`), impedâncias nominais e especificações de fábrica. |
| `F-04` | Toshiba — *TA7136P / TA7136AP Low-Noise Pre-Amplifier SIP IC Datasheet* | Especificação oficial do componente (SIP-7, pino 5–6 compensação externa `33 pF`, alimentação única `9 V`) | `PRIMARY_COMPONENT` | Características elétricas do CI original `TA7136AP` (1978–1993). |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Diferenciação entre revisão `1978–1993 TA7136AP SIP` e revisão `1994+ Dual Op-Amp (BA728N / M5223AL)` | `F-02`, `F-04` | 1978–1993 vs 1994+ | `F-02` §"Op-Amp Stage / Hardware Revisions" | `SECONDARY_TECHNICAL` / `PRIMARY_COMPONENT` | `CONFIRMED` |
| Estágio Transistor Booster `Q2` (`2SC732`, $R_7=2,2\text{ k}\Omega, C_5=68\text{ nF}, C_4=250\text{ pF}$) fornecendo até `+17 dB` de pré-ênfase | `F-02` | 1978–1993 (`TA7136AP`) | `F-02` §"Transistor Booster Stage" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Potenciômetro `DIST` (`100 kΩ B`) em configuração split-wiper com $R_9=4,7\text{ k}\Omega, C_7=0,47\text{ }\mu\text{F}, C_8=250\text{ pF}$ ($G \in [0\text{ dB}, +27\text{ dB}]$) | `F-01`, `F-02` | 1978–1993 (`TA7136AP`) | `F-01` §2.2 Fig. 3 p. 2; `F-02` §"Operational Amplifier Stage" | `ACADEMIC` / `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Tone Stack passivo em ponte ($R_{15}=6,8\text{ k}\Omega, C_{11}=22\text{ nF}, C_{12}=100\text{ nF}, R_{16}=2,2\text{ k}\Omega, R_{17}=6,8\text{ k}\Omega, P_{\text{tone}}=20\text{ k}\Omega$) com *mid-scoop* em `~500 Hz` | `F-01`, `F-02` | 1978–1993 | `F-01` §3.2 Fig. 7–8 pp. 4–5; `F-02` §"Tone Control Stage" | `ACADEMIC` / `SECONDARY_TECHNICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input] ──► [Q1 2SC732 Input Buffer]
                  │
                  ▼
            [Q2 2SC732 Transistor Booster (R7=2.2kΩ || C5=68nF, C4=250pF, +17dB High-Mid Pre-Emphasis)]
                  │
                  ▼
            [IC1 Toshiba TA7136AP Non-Inverting Stage]
            (Split-Wiper P1 DIST 100kΩ: Feedback α*100kΩ || C8=250pF; Ground (1-α)*100kΩ + R9=4.7kΩ + C7=0.47µF)
                  │
                  ▼
            [C9 0.47µF + R14 2.2kΩ] ──┬──► [D4||D5 (2x 1S1588 Si Hard Clip) || C10 (10 nF Shunt LPF @ 7.23 kHz)]
                                      │
                                      ├──► (Braço Low: R15 6.8kΩ + C11 22nF to GND + R17 6.8kΩ) ──┐
                                      │                                                           ├──► [P2 Tone Wiper (20 kΩ B)]
                                      └──► (Braço High: C12 100nF + R16 2.2kΩ) ───────────────────┘          │
                                                                                                             ▼
                                                                                      [P3 Level (100 kΩ B)] ──► [Q3 Output Buffer]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/drive.rs`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/fx/drive.rs#L445-L456) e [`665-685`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/fx/drive.rs#L665-L685) possui as seguintes divergências auditadas:

- Removido em 2026-11-20 (AUD-38): identificador inexistente em qualquer commit
- Removido em 2026-11-20 (AUD-38): identificador inexistente em qualquer commit

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Normalizada | Unidade / Escala UI | Curva / Taper | Valor Padrão | Papel Elétrico no Circuito |
|---|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `drive.enabled` | `pedal.ds1.enabled` | `false / true` | Booleano | Discreto | `false` | Buffered FET switching (`Q4`/`Q5` + buffers `Q1`/`Q3`). |
| **TONE** | `drive.tone` | `pedal.ds1.tone` | `0.0 .. 1.0` | `0 .. 100 %` | Linear (`20 kΩ B`) | `0.50` | Move o cursor do potenciômetro `20 kΩ` na ponte passiva assimétrica (`500 Hz` mid-scoop). |
| **LEVEL** | `drive.level` | `pedal.ds1.level` | `0.0 .. 1.0` | `0 .. 100 %` | Log / Linear (`100 kΩ`) | `0.50` | Divisor resistivo após o cursor do `Tone` e antes do buffer de saída `Q3`. |
| **DIST** | `drive.gain` | `pedal.ds1.gain` | `0.0 .. 1.0` | `0 .. 100 %` | Linear (`100 kΩ B`) | `0.50` | Desloca o cursor do potenciômetro `100 kΩ` entre a malha de realimentação e a perna de terra do `TA7136AP` (configuração split-wiper). |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Estágio 1 — Transistor Booster `Q2` (`2SC732`):**
   - Filtro shelving passa-altas/passa-faixa com ganho de $+4\text{ dB}$ em graves (`< 150 Hz`) subindo para $+17\text{ dB}$ em `1,5 kHz–5 kHz` e rolagem superior em `12 kHz` ($C_4 = 250\text{ pF}$).
2. **Estágio 2 — Amplificador Não-Inversor Split-Wiper `TA7136AP` (Yeh DAFx-07 §2.2):**
   - Para a posição do cursor $\alpha = \text{drive.gain} \in [0, 1]$:
     $$R_a(\alpha) = 4700 + (1 - \alpha) \cdot 100\,000\text{ }\Omega, \qquad R_b(\alpha) = \alpha \cdot 100\,000\text{ }\Omega$$
   - A função de transferência linear dependente de $\alpha$ com $C_7 = 0,47\text{ }\mu\text{F}$ e $C_8 = 250\text{ pF}$ é:
     $$H_{\text{TA7136}}(s, \alpha) = 1 + \frac{s R_b(\alpha) C_7}{(1 + s R_a(\alpha) C_7)(1 + s R_b(\alpha) C_8)}$$
   - Seguida de saturação assimétrica suave de trilho do `TA7136AP` (`+9 V` single supply).
3. **Estágio 3 — Hard-Clipping Shunt (`2× 1S1588`) + Capacitor Paralelo $C_{10} = 10\text{ nF}$ (Yeh DAFx-07 Eq. 5):**
   - Equação diferencial do nó dos diodos com $R_{14} = 2,2\text{ k}\Omega$ e $C_{10} = 10\text{ nF}$ ($f_c = 7234,3\text{ Hz}$):
     $$C_{10} \frac{dv_d}{dt} + \frac{v_d - v_{\text{op}}}{R_{14}} + 2 I_s \sinh\left(\frac{v_d}{N V_T}\right) = 0$$
4. **Estágio 4 — Tone Stack Passivo em Ponte Assimétrica (Yeh DAFx-07 §3.2 Fig. 7–8):**
   - Discretização bilinear da função de transferência nodal $H_{\text{DS1\_Tone}}(s, \beta)$ calculada diretamente a partir dos 6 componentes passivos documentados ($R_{15}=6,8\text{ k}\Omega, C_{11}=22\text{ nF}, C_{12}=100\text{ nF}, R_{16}=2,2\text{ k}\Omega, R_{17}=6,8\text{ k}\Omega, P_{\text{tone}}=20\text{ k}\Omega$).

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `8×` polifásico IIR (Spec 079) em torno do estágio `TA7136AP` e do clipper shunt `1S1588` + $C_{10} = 10\text{ nF}$.
- **Precisão numérica:** `f64` para todos os estados dos filtros e solver do nó `1S1588`.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`.
- Complexidade $O(1)$ por amostra; WCET $\le 12\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- **Impedância de entrada:** $470\text{ k}\Omega$ (buffer `Q1`).
- **Impedância de saída:** $1\text{ k}\Omega$ (buffer `Q3`).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/boss_ds1.rs`.
- **Struct principal:** `pub struct BossDs1Processor` implementando `DspBlock` + `AutomatableParams` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Subsistema | Componente Real | Valor Documentado (`1978–1993 TA7136AP`) | Invariante Física |
|---|---|---|---|
| Booster `Q2` | $R_7=2,2\text{ k}\Omega, C_5=68\text{ nF}, R_6=10\text{ k}\Omega, C_4=250\text{ pF}$ | Ganho até $+17\text{ dB} @ >1\text{ kHz}$ | Pré-ênfase fixa de ataque antes do op-amp. |
| Op-Amp `TA7136AP` | Split-wiper $P_1=100\text{ k}\Omega, R_9=4,7\text{ k}\Omega, C_7=0,47\text{ }\mu\text{F}, C_8=250\text{ pF}$ | $G \in [0\text{ dB},\; +26,96\text{ dB}]$ | Ganho cresce de forma hiperbólica nos últimos `25%` do knob `DIST`. |
| Clipper Shunt | $R_{14}=2,2\text{ k}\Omega, C_{10}=10\text{ nF}, 2\times\text{1S1588}$ | $f_{\text{LPF,shunt}} = 7234\text{ Hz}$ | Polo passa-baixas de `7,23 kHz` paralelo aos diodos. |
| Tone Stack Bridge | $R_{15}=6,8\text{ k}\Omega, C_{11}=22\text{ nF}, C_{12}=100\text{ nF}, R_{16}=2,2\text{ k}\Omega, R_{17}=6,8\text{ k}\Omega, P_2=20\text{ k}\Omega$ | Mid-scoop em `~500 Hz` | Vale de médios posicionado uma oitava abaixo do Big Muff (`500 Hz` vs `1000 Hz`). |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste do Mid-Scoop do Tone Stack em `~500 Hz` (`Tone = 0.5`):** verifica vale centrada em `500 Hz` relativo a `120 Hz` e `2500 Hz`.
2. **Teste da Lei de Ganho Split-Wiper do Potenciômetro `DIST`:** verifica que o ganho do estágio `TA7136AP` obedece à relação $1 + \frac{\alpha \cdot 100}{4,7 + (1-\alpha)\cdot 100}$.
3. **Teste de Basculamento do Controle `Tone` (`Tone = 0.0` vs `Tone = 1.0`):** verifica inversão espectral.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Como o potenciômetro `DIST` (`100 kΩ B`) é linear mas está conectado em configuração *split-wiper* (aumentando o numerador enquanto diminui o denominador), o ganho real do circuito em dB sobe suavemente até `70%` do curso e acelera rapidamente nos últimos `30%` — comportamento autêntico do pedal hardware.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Mid-Scoop em `~500 Hz` do Tone Stack em Ponte Assimétrica (`Tone = 0.5`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Tons senoidais de teste na entrada do Tone Stack em `120 Hz` (grave), `500 Hz` (centro do mid-scoop do DS-1) e `2500 Hz` (médio-agudo).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `pedal.boss.ds1` com `drive.tone = 0.50`.
  - *Método de medição:* Magnitude da função de transferência do Tone Stack em `120 Hz`, `500 Hz` e `2500 Hz`.
  - *Critério e tolerância:* Em `drive.tone = 0.50`, a magnitude em `500 Hz` deve apresentar um vale (*mid-scoop*) de **`5,0 dB` a `11,0 dB` abaixo** da média das magnitudes em `120 Hz` e `2500 Hz`.
  - *Origem da tolerância:* Curva teórica e medida do Tone Stack do Boss DS-1 em `F-01` Fig. 8 p. 5 e `F-02` §"Tone Control Frequency Response".

- **CA-02 — Lei de Ganho Split-Wiper do Estágio `TA7136AP` (`DIST = 0%`, `50%`, `100%`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Senoide de pequeno sinal (`-60 dBFS`, regime linear antes da condução dos diodos `1S1588`) em `1500 Hz`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Comparação do ganho do estágio `TA7136AP` em `drive.gain = 0.0` ($\alpha=0$), `drive.gain = 0.5` ($\alpha=0,5$) e `drive.gain = 1.0` ($\alpha=1,0$).
  - *Método de medição:* Ganho linear em dB do estágio `TA7136AP` em `1500 Hz`.
  - *Critério e tolerância:*
    1. Em $\alpha = 0,0$: ganho de **`0,0 dB ± 0,5 dB`** ($G = 1,0\times$).
    2. Em $\alpha = 0,5$: ganho teórico $20\log_{10}(1 + \frac{50}{54,7}) = +5,64\text{ dB}$, tolerância **`+5,6 dB ± 1,0 dB`**.
    3. Em $\alpha = 1,0$: ganho teórico $20\log_{10}(1 + \frac{100}{4,7}) \approx +26,96\text{ dB}$ (atenuado levemente por $C_8 = 250\text{ pF}$ para $\sim +26,4\text{ dB}$), tolerância **`+26,4 dB ± 1,5 dB`**.
  - *Origem da tolerância:* Equação exata do divisor split-wiper $R_9 = 4,7\text{ k}\Omega, P_1 = 100\text{ k}\Omega$ (`F-01` §2.2, `F-02` §"Op-Amp Stage").

- **CA-03 — Pré-Ênfase do Transistor Booster `Q2` (`2SC732`, $C_5 = 68\text{ nF}$) (`DOCUMENTED`):**
  - *Sinal de entrada:* Tons senoidais de pequeno sinal (`-60 dBFS`) em `80 Hz` e `2500 Hz`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Medição na saída do estágio `Q2` (ou com `drive.gain = 0.0` antes do Tone Stack).
  - *Método de medição:* Diferença de ganho $G(2500\text{ Hz}) - G(80\text{ Hz})$.
  - *Critério e tolerância:* O estágio `Q2` deve fornecer **pelo menos `+8,0 dB`** a mais de ganho em `2500 Hz` do que em `80 Hz`.
  - *Origem da tolerância:* Curva de resposta do estágio Transistor Booster `Q2` ($R_7 = 2,2\text{ k}\Omega \parallel C_5 = 68\text{ nF}$) em `F-02` §"Transistor Booster Frequency Response".

- **CA-04 — Basculamento do Controle `Tone` (`Tone = 0.0` vs `Tone = 1.0`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `440 Hz` a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `drive.gain = 0.75`, `drive.level = 0.50`, comparando `drive.tone = 0.0` vs `drive.tone = 1.0`.
  - *Método de medição:* Centróide espectral da saída.
  - *Critério e tolerância:* O centróide espectral em `drive.tone = 1.0` deve exceder o centróide espectral em `drive.tone = 0.0` em **pelo menos `800 Hz`**.
  - *Origem da tolerância:* Excursão do Tone Stack do DS-1 (`F-01` Fig. 8, `F-02`).

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/drive.rs` (e futuro `crates/ampforge-dsp/src/fx/pedals/boss_ds1.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.ds1.*`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_031_pedals_dsp.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Revisão histórica fixada em `1978–1993 Toshiba TA7136AP SIP` e separada de `1994+ Dual Op-Amp`.
- [x] Omissão do Transistor Booster `Q2` e da topologia split-wiper de `DIST` em `drive.rs:446, 668` diagnosticada e documentada.
- [x] Critérios `CA-01` a `CA-05` classificados e com condições completas de medição.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Yeh, D. T., Abel, J. S., & Smith, J. O. (2007). *Simplified, Physically-Informed Models of Distortion and Overdrive Guitar Effects Pedals*. Proc. DAFx-07 (`https://ccrma.stanford.edu/~dtyeh/papers/yeh07_dafx_distortion.pdf`).
- `[F-02]` ElectroSmash — *Boss DS-1 Distortion Analysis* (`https://web.archive.org/web/20240115000000/https://www.electrosmash.com/boss-ds1-analysis`).
- `[F-03]` Boss Corporation — *DS-1 Owner's Manual* (`https://www.boss.info/global/support/by_product/ds-1/owners_manuals/`).

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, substituiu-se "Rev. Clássica" por `1978–1993 MIJ Toshiba TA7136AP SIP`, separando-a da revisão `1994+ Dual Op-Amp`. Documentou-se a topologia split-wiper do knob `DIST`, o estágio Transistor Booster `Q2` e a equação exata de DAFx-07 para a ponte passiva de Tone.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Tone Stack passivo em ponte assimétrica ($6,8\text{ k}\Omega / 22\text{ nF} / 100\text{ nF} / 2,2\text{ k}\Omega$) com *mid-scoop* em `~500 Hz` | `F-01` §3.2, `F-02` | Spec 064 / `CA-01`, `CA-04` | `crates/ampforge-dsp/src/fx/drive.rs:448–450 @ c20cc65, 678–682` (usa dois biquads de 2ª ordem em `450 Hz` e `1100 Hz`) | `contract_031_pedals_dsp.rs`, `contract_040_pedals_parity.rs` | `IMPLEMENTED_UNVERIFIED` |
| Estágio `TA7136AP` com potenciômetro `DIST` (`100 kΩ`) em configuração *split-wiper* ($G \in [0\text{ dB}, +26,96\text{ dB}]$) | `F-01` §2.2, `F-02` | Spec 064 / `CA-02` | `crates/ampforge-dsp/src/fx/drive.rs:668 @ c20cc65` (usa ganho linear simples `1.0 + drive * 40.0` e HPF fixo de `70 Hz`) | Ausente | `SPECIFIED` |
| Estágio Transistor Booster `Q2` (`2SC732`, $C_5 = 68\text{ nF}$, $+17\text{ dB}$ pré-ênfase) | `F-02` §"Transistor Booster" | Spec 064 / `CA-03` | Ausente em `drive.rs` | Ausente | `SPECIFIED` |
| Hard-clipping shunt de silício (`2× 1S1588`) + capacitor shunt $C_{10} = 10\text{ nF}$ (`7,23 kHz`) | `F-01` §2.2, `F-02` | Spec 064 / §9 | `crates/ampforge-dsp/src/fx/drive.rs:447 @ c20cc65, 670–675` (LPF fixo em `6500 Hz`) | `contract_031_pedals_dsp.rs` | `IMPLEMENTED_UNVERIFIED` |
| Execução zero-alloc em tempo real com oversampling `8×` | Spec 053, Spec 079 | Spec 064 / `CA-05` | `crates/ampforge-dsp/src/fx/drive.rs:463–491 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
