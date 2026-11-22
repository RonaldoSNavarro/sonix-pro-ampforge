---
status: rascunho
---

# Spec 061: Pedal ProCo RAT (Distortion Op-Amp LM308N)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-pedal` (`drive`)
> **Identificador canônico:** `pedal.proco.rat`
> **Variante / Revisão:** `1984–1988 Small-Box "Whiteface" / "Blackface" (Motorola/National LM308N, Cap Compensação 30 pF, Diodos 1N914)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/drive.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [031](031-amplificadores-fieis-aos-esquemas-e-controles-originais.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [079](079-componente-oversampling-polifasico-e-anti-aliasing.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação do pedal **ProCo RAT (1984–1988, LM308N)** como processador modular independente (`pedal.proco.rat`), modelando fielmente:
1. A rede passa-altas dupla na perna inversora ($47\text{ }\Omega + 2,2\text{ }\mu\text{F} \Rightarrow 1539\text{ Hz}$ e $560\text{ }\Omega + 4,7\text{ }\mu\text{F} \Rightarrow 60,4\text{ Hz}$);
2. A limitação física de *slew rate* (`0,3 V/µs`) imposta pelo amplificador operacional `LM308N` compensado externamente com $C_3 = 30\text{ pF}$;
3. O estágio de hard-clipping paralelo de silício (`2× 1N914` / `1N4148`); e
4. O filtro passa-baixas reverso `FILTER` ($R_7 = 1,5\text{ k}\Omega + P_{\text{filter}} = 100\text{ k}\Omega\text{ Log}, C_8 = 3,3\text{ nF}$), seguido do buffer JFET `2N5458`.

---

## 2. Escopo

- Circuito completo da revisão clássica `1984–1988 ProCo RAT (LM308N)` com seus três controles (`Distortion`, `Filter`, `Volume`) e chave `Bypass`.

---

## 3. Fora de escopo

- **RAT2 moderno (pós-2001 com CI Texas Instruments `OP07DP`):** O `OP07DP` possui compensação interna fixa e *slew rate* de `0,3 V/µs` sem pino de compensação externa de `30 pF`, apresentando comportamento de sobrecarga de entrada distinto do `LM308N`.
- **Turbo RAT (1989, clipping por LEDs vermelhos):** Fora do escopo deste modelo.
- **You Dirty RAT (clipping por diodos de germânio `1N34A`) ou Ruetz Mod (corte do resistor de `47 Ω`):** Fora do escopo.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`distortion`) |
| **Fabricante de referência** | Pro Co Sound Inc. (Kalamazoo, MI, EUA — projetado por Scott Burnham e Steve Kiraly) |
| **Modelo de referência** | ProCo RAT ("The RAT") |
| **Ano / época / revisão** | 1984–1988 ("Whiteface" / "Blackface" Small-Box; CI `LM308N`, cap de compensação $C_3 = 30\text{ pF}$, diodos `1N914`, JFET `2N5458`) |
| **Identificador no sistema** | `pedal.proco.rat` (`DriveModel::DriveModel::BlackRodentRat` / UI slug `rat`) |
| **Papel na cadeia de sinal** | Bloco `DRIVE` pré-amplificador (capaz de operar desde overdrive médio até distortion densa com características de fuzz em `Distortion > 75%`). |

---

## 5. Princípio físico e funcional do equipamento real

1. **Estágio Não-Inversor de Ganho Extremo (`LM308N` até `+67 dB`) com Duplo HPF na Perna Inversora:**
   - Diferente do Tube Screamer (que tem apenas um ramo RC para `Vref`), a perna inversora do `LM308N` no RAT possui **dois ramos RC em paralelo** para `Vref`:
     - Ramo 1: $R_4 = 47\text{ }\Omega$ em série com $C_5 = 2,2\text{ }\mu\text{F} \Rightarrow f_{c1} = \frac{1}{2\pi \cdot 47 \cdot 2,2 \times 10^{-6}} \approx 1538,9\text{ Hz}$.
     - Ramo 2: $R_5 = 560\text{ }\Omega$ em série com $C_6 = 4,7\text{ }\mu\text{F} \Rightarrow f_{c2} = \frac{1}{2\pi \cdot 560 \cdot 4,7 \times 10^{-6}} \approx 60,44\text{ Hz}$.
   - O potenciômetro `Distortion` (`100 kΩ Log`) está na malha de realimentação em paralelo com $C_4 = 100\text{ pF}$. Para frequências acima de `1539 Hz`, os dois ramos conduzem em paralelo ($R_{\text{eq}} = 47 \parallel 560 = 43,36\text{ }\Omega$), resultando num ganho máximo de $1 + \frac{100\,000}{43,36} \approx 2307\times$ (**`+67,26 dB`**). Entre `60,4 Hz` e `1539 Hz`, apenas o ramo de `560 Ω` domina ($G_{\max} \approx 1 + \frac{100\,000}{560} \approx 179,6\times = +45,08\text{ dB}$). Abaixo de `60,4 Hz`, o ganho cai para a unidade ($0\text{ dB}$).
2. **Limitação Física de *Slew Rate* (`0,3 V/µs`) e Compensação Externa ($C_3 = 30\text{ pF}$):**
   - O amplificador operacional `LM308N` não possui compensação interna; nos pinos `1` e `8`, o circuito do RAT conecta um capacitor $C_3 = 30\text{ pF}$.
   - De acordo com o datasheet oficial da National Semiconductor (`LM108/LM208/LM308`), com $C_f = 30\text{ pF}$ o *slew rate* máximo de saída é estritamente limitado a **`SR = 0,3 V/µs`** ($300\,000\text{ V/s}$) e a banda de ganho fechado cai drasticamente sob ganhos elevados (`+45 a +67 dB`).
   - Quando a derivada temporal do sinal amplificado $|\frac{dV}{dt}|$ excede $0,3\text{ V/}\mu\text{s}$, o estágio interno do `LM308N` entra em *slew-rate limiting*, transformando senoides de alta frequência e transientes abruptos em rampas triangulares **antes mesmo** de atingir os diodos de clipping! Isso atenua naturalmente os harmônicos superiores ásperos e produz a textura cremosa característica do RAT.
3. **Hard-Clipping Shunt de Silício (`D1`, `D2` — `1N914` / `1N4148`):**
   - A saída do `LM308N` passa por um resistor limitador de corrente $R_6 = 1\text{ k}\Omega$ e capacitor de bloqueio DC $C_7 = 4,7\text{ }\mu\text{F}$ diretamente para um par antiparalelo de diodos de silício `1N914` ($V_f \approx 0,65\text{ V}$) aterrados, ceifando a onda em formato quase trapezoidal/quadrado.
4. **Controle `FILTER` Reverso (`100 kΩ Log` + $R_7 = 1,5\text{ k}\Omega$ + $C_8 = 3,3\text{ nF}$):**
   - O knob `FILTER` opera **ao contrário** de um controle de Tone tradicional: girá-lo no sentido horário (aumentar o valor de `FILTER`) **aumenta** a resistência em série com o capacitor $C_8 = 3,3\text{ nF}$ para o terra, **reduzindo** a frequência de corte do filtro passa-baixas de 1ª ordem:
     $$f_{\text{Filter}}(\alpha) = \frac{1}{2\pi \cdot (1500\text{ }\Omega + \alpha \cdot 100\,000\text{ }\Omega) \cdot 3,3\text{ nF}}$$
     - Em `Filter = 0%` ($\alpha = 0$, totalmente no sentido anti-horário = **mais brilhante**): $f_c = \frac{1}{2\pi \cdot 1500 \cdot 3,3 \times 10^{-9}} \approx 32\,154\text{ Hz}$.
     - Em `Filter = 100%` ($\alpha = 1$, totalmente no sentido horário = **mais escuro**): $f_c = \frac{1}{2\pi \cdot 101\,500 \cdot 3,3 \times 10^{-9}} \approx 475,2\text{ Hz}$.

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | ElectroSmash — *ProCo RAT Analysis* (arquivado e verificado via Wayback Machine) | `https://web.archive.org/web/20240115000000/https://www.electrosmash.com/proco-rat` (HTTP 200 verificado) | `SECONDARY_TECHNICAL` | Esquema completo do ProCo RAT, equações dos dois ramos HPF (`60,4 Hz` e `1539 Hz`), compensação `30 pF` do `LM308N`, clipping `1N914`, filtro reverso `475 Hz..32 kHz` e buffer JFET `2N5458`. |
| `F-02` | National Semiconductor / Texas Instruments — *LM108/LM208/LM308 Operational Amplifiers Datasheet* (SNOSBT2) | Especificação oficial do componente (`Feedforward & Standard Compensation`, `Cf = 30 pF => SR = 0.3 V/µs`) | `PRIMARY_COMPONENT` | Curva oficial de Slew Rate e resposta em frequência do `LM308N` com capacitor de compensação de `30 pF` entre pinos 1 e 8. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Duplo ramo RC na perna inversora do `LM308N`: $47\text{ }\Omega + 2,2\text{ }\mu\text{F}$ (`1539 Hz`) e $560\text{ }\Omega + 4,7\text{ }\mu\text{F}$ (`60,4 Hz`), ganho máx. $+67\text{ dB}$ | `F-01` | ProCo RAT 1984–1988 (`LM308N`) | `F-01` §"Clipping Amplifier / Frequency Response" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Capacitor de compensação externa $C_3 = 30\text{ pF}$ nos pinos 1 e 8 do `LM308N` limitando o Slew Rate a `0,3 V/µs` | `F-01`, `F-02` | ProCo RAT 1984–1988 (`LM308N`) | `F-01` §"LM308 Slew Rate"; `F-02` Typical Performance Curves | `SECONDARY_TECHNICAL` / `PRIMARY_COMPONENT` | `CONFIRMED` |
| Hard-clipping shunt com resistor série $R_6 = 1\text{ k}\Omega$ e par antiparalelo `1N914` | `F-01` | ProCo RAT 1984–1988 | `F-01` §"Clipping Diodes" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Filtro passa-baixas reverso `FILTER`: $R_7 = 1,5\text{ k}\Omega + P_{\text{filter}} = 100\text{ k}\Omega\text{ Log}$, $C_8 = 3,3\text{ nF}$ ($f_c \in [475,2\text{ Hz},\; 32,15\text{ kHz}]$) | `F-01` | ProCo RAT 1984–1988 | `F-01` §"Tone Control (Filter Stage)" | `SECONDARY_TECHNICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input] ──► [C1 22nF + R1 1MΩ / R2 1kΩ] ──► [LM308N (+) Non-Inverting Op-Amp (C3 = 30pF Comp, SR = 0.3 V/µs)]
                                                    │
                    ┌───────────────────────────────┴───────────────────────────────┐
                    │ Malha Feedback (-):                                           │ Perna p/ Vref (4.5V):
                    │  P1 Distortion (100 kΩ Log) || C4 (100 pF)                    │  R4 (47 Ω) + C5 (2.2 µF)   => 1538.9 Hz
                    └───────────────────────────────┬───────────────────────────────┘  || R5 (560 Ω) + C6 (4.7 µF) => 60.44 Hz
                                                    │
                                                    ▼
                                      [R6 (1 kΩ) + C7 (4.7 µF)]
                                                    │
                                                    ├──► [D1 || D2 (2x 1N914 Si Hard Clip to GND)]
                                                    │
                                                    ▼
                                      [Filter Stage: R7 (1.5 kΩ) + P2 Filter (100 kΩ Log) + C8 (3.3 nF to GND)]
                                                    │   (fc = 32.15 kHz @ Filter=0%  ──►  475.2 Hz @ Filter=100%)
                                                    ▼
                                      [Q1 2N5458 JFET Source Follower] ──► [P3 Volume (100 kΩ Log)] ──► [Output]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/drive.rs`](../crates/ampforge-dsp/src/fx/drive.rs#L440-L444) e [`567-598`](../crates/ampforge-dsp/src/fx/drive.rs#L567-L598) contém as seguintes divergências auditadas:

- Removido em 2026-11-20 (AUD-38): identificador inexistente em qualquer commit
- Removido em 2026-11-20 (AUD-38): identificador inexistente em qualquer commit
3. **`FACTUAL_ERROR` (`drive.rs:442–443`):** A frequência do controle `FILTER` é calculada como `let fc = 15000.0 * (475.0 / 15000.0).powf(self.tone);` usando um biquad de **2ª ordem** (`Q=0.707`), quando no circuito real o estágio `FILTER` é um filtro RC passivo de **1ª ordem** ($6\text{ dB/oitava}$) cuja frequência de corte varia hiperbolicamente com o potenciômetro logarítmico `100 kΩ` de `32 154 Hz` (`Filter = 0%`) até `475,2 Hz` (`Filter = 100%`).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Normalizada | Unidade / Escala UI | Curva / Taper | Valor Padrão | Papel Elétrico no Circuito |
|---|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `drive.enabled` | `pedal.rat.enabled` | `false / true` | Booleano | Discreto | `false` | True-bypass mecânico DPDT. |
| **DISTORTION** | `drive.gain` | `pedal.rat.gain` | `0.0 .. 1.0` | `0 .. 100 %` | Logarítmico (`100 kΩ A`) | `0.50` | Varia a resistência de feedback do `LM308N` de $0\text{ }\Omega$ (ganho unitário) até $100\text{ k}\Omega$ ($+67,26\text{ dB}$ acima de $1539\text{ Hz}$). |
| **FILTER** | `drive.tone` | `pedal.rat.tone` | `0.0 .. 1.0` | `0 .. 100 %` | Logarítmico Reverso (`100 kΩ A`) | `0.50` | **Ação reversa:** `0.0` = totalmente brilhante ($R_7 = 1,5\text{ k}\Omega \Rightarrow f_c = 32,15\text{ kHz}$); `1.0` = totalmente escuro ($R = 101,5\text{ k}\Omega \Rightarrow f_c = 475,2\text{ Hz}$). |
| **VOLUME** | `drive.level` | `pedal.rat.level` | `0.0 .. 1.0` | `0 .. 100 %` | Logarítmico (`100 kΩ A`) | `0.50` | Divisor resistivo após o seguidor de fonte JFET `2N5458`. |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Rede Dupla de Pré-Ênfase Não-Inversora (`60,44 Hz` + `1538,9 Hz`) e Polo de Feedback ($C_4 = 100\text{ pF}$):**
   - A admitância equivalente da perna inversora é $Y_i(s) = \frac{s C_5}{1 + s R_4 C_5} + \frac{s C_6}{1 + s R_5 C_6}$, e a impedância de realimentação é $Z_f(s) = \frac{R_{\text{dist}}}{1 + s R_{\text{dist}} C_4}$, onde $R_{\text{dist}}(\text{gain}) = 100\text{ }\Omega + 100\,000\text{ }\Omega \cdot \text{taper}_{\log}(\text{gain})$.
   - A função de transferência linear do estágio `LM308N` é $H_{\text{LM308}}(s) = 1 + Z_f(s) Y_i(s)$, discretizada exatamente via transformada bilinear na taxa de oversampling $8\times f_s$.
2. **Limitador Físico Não-Linear de *Slew Rate* (`LM308N` com $C_3 = 30\text{ pF}$):**
   - Na grade temporal de $8\times f_s$ ($\Delta t = \frac{1}{8 f_s}$), a variação máxima de tensão permitida por sub-amostra pelo `LM308N` (`SR = 0,3 V/µs` escalado para a referência de clipping $V_{\text{diode}} = 0,65\text{ V}$) é:
     $$\Delta v_{\max} = \frac{\text{SR}}{V_{\text{scale}}} \cdot \frac{1}{8 f_s}$$
     $$v_{\text{slew}}[k] = v_{\text{slew}}[k-1] + \operatorname{clamp}\left(v_{\text{target}}[k] - v_{\text{slew}}[k-1],\; -\Delta v_{\max},\; +\Delta v_{\max}\right)$$
   - Seguido de saturação de barramento do op-amp em $\pm 4,0\text{ V}$.
3. **Hard-Clipping Shunt com Resistor Série $R_6 = 1\text{ k}\Omega$ e Diodos `1N914`:**
   - Curva de condução de silício com resistência série $R_6 = 1\text{ k}\Omega$, produzindo ceifamento firme em $\pm 0,68\text{ V}$ com inclinação residual determinada por $\frac{R_{\text{diode}}}{R_6 + R_{\text{diode}}}$.
4. **Filtro Passa-Baixas Reverso de 1ª Ordem (`FILTER`):**
   - Filtro Tustin de 1ª ordem ($6\text{ dB/oitava}$) com frequência de corte exata:
     $$f_c(\text{tone}) = \frac{1}{2\pi \cdot \left(1500,0 + 100\,000,0 \cdot \text{taper}_{\log}(\text{tone})\right) \cdot 3,3 \times 10^{-9}}$$

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `8×` polifásico IIR (Spec 079) englobando o filtro linear $H_{\text{LM308}}(s)$, o limitador de *slew rate* e o hard-clipper `1N914`.
- **Precisão numérica:** `f64` para todos os estados dos filtros e acumulador de *slew rate*.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`.
- Operação em tempo constante $O(1)$ por amostra; WCET $\le 10\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- **Entrada:** Impedância de entrada de `1 MΩ`.
- **Saída:** Buffer JFET `2N5458` de baixa impedância.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/proco_rat.rs`.
- **Struct principal:** `pub struct ProCoRatProcessor` implementando `DspBlock` + `AutomatableParams` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Subsistema | Componente Real | Valor Documentado | Invariante Física |
|---|---|---|---|
| Ramo HPF 1 (perna `-`) | $R_4 = 47\text{ }\Omega, C_5 = 2,2\text{ }\mu\text{F}$ | $f_{c1} = 1538,9\text{ Hz}$ | Eleva o ganho acima de `1,5 kHz` em até `+22 dB` sobre a faixa `100–500 Hz`. |
| Ramo HPF 2 (perna `-`) | $R_5 = 560\text{ }\Omega, C_6 = 4,7\text{ }\mu\text{F}$ | $f_{c2} = 60,44\text{ Hz}$ | Corta ganho de subgraves abaixo de `60 Hz` para evitar *blocking distortion*. |
| Compensação `LM308N` | $C_3 = 30\text{ pF}$ (pinos 1–8) | $\text{SR} = 0,3\text{ V/}\mu\text{s}$ | Limita $|\frac{dv}{dt}| \le 0,3\text{ V/}\mu\text{s}$ antes dos diodos `1N914`. |
| Controle `FILTER` | $R_7 = 1,5\text{ k}\Omega, P_2 = 100\text{ k}\Omega, C_8 = 3,3\text{ nF}$ | $f_c \in [475,2\text{ Hz},\; 32,15\text{ kHz}]$ | `Filter = 0.0` é o ponto mais brilhante; `Filter = 1.0` é o ponto mais escuro ($475,2\text{ Hz}$). |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste da Rede Dupla de Pré-Ênfase (`60,4 Hz` + `1539 Hz`):** verifica o degrau de ganho entre `200 Hz` e `3000 Hz` em pequeno sinal.
2. **Teste de Limitação de Slew Rate (`0,3 V/µs`):** excitação de alta frequência (`5 kHz`) em alto ganho verifica que a inclinação temporal da forma de onda antes do clipping é limitada por $\Delta v_{\max}$.
3. **Teste de Direção Reversa do Knob `FILTER`:** verifica que aumentar `drive.tone` de `0.0` para `1.0` reduz monotonamente o centróide espectral e a energia acima de `2 kHz`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- A convenção reversa do knob `FILTER` (`0%` = brilhante, `100%` = escuro) é uma característica icônica e obrigatória do ProCo RAT real (já exigida pela Spec 031 e testada em `contract_031_pedals_dsp.rs`); a UI e o registro `controls.registry.ts` devem preservar exatamente esse comportamento.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Pré-Ênfase de Dois Degraus na Perna Inversora (`60,4 Hz` e `1538,9 Hz`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Senoides de pequeno sinal (`-60 dBFS`, regime linear sem clipping e sem slew-limiting) em `30 Hz`, `300 Hz` e `3000 Hz`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `pedal.proco.rat` com `drive.gain = 0.50`, `drive.tone = 0.0` (`Filter = 0%`, totalmente aberto em `32,15 kHz`), `drive.level = 0.50`.
  - *Método de medição:* Ganho linear em dB medido na saída do estágio `LM308N` (antes dos diodos `1N914`).
  - *Critério e tolerância:* O ganho em `3000 Hz` (acima de $f_{c1} = 1538,9\text{ Hz}$) deve superar o ganho em `300 Hz` (entre $f_{c2}$ e $f_{c1}$) em **pelo menos `+10,0 dB`**, e o ganho em `300 Hz` deve superar o ganho em `30 Hz` (abaixo de $f_{c2} = 60,44\text{ Hz}$) em **pelo menos `+8,0 dB`**.
  - *Origem da tolerância:* Equação analítica da rede paralela $R_4 C_5 \parallel R_5 C_6$ (`F-01` §"Clipping Amplifier Frequency Response").

- **CA-02 — Limitação Física de Slew Rate (`0,3 V/µs` com $C_3 = 30\text{ pF}$) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Dois tons senoidais separados de mesma amplitude (`-12 dBFS`): um em `250 Hz` e outro em `6000 Hz`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras (`8×` oversampling = `384 000 Hz`).
  - *Configuração e parâmetros:* `drive.gain = 0.90`, `drive.tone = 0.0`, `drive.level = 0.50`.
  - *Método de medição:* Medição da derivada discreta máxima $\max |v_{\text{slew}}[k] - v_{\text{slew}}[k-1]|$ na saída do estágio `LM308N` e comparação da amplitude de pico antes do filtro de saída em `6000 Hz` vs `250 Hz`.
  - *Critério e tolerância:* A variação por sub-amostra no estágio `LM308N` nunca deve exceder $\Delta v_{\max} \cdot (1 + 10^{-6})$, e em `6000 Hz` o *slew-rate limiting* deve reduzir o fator de crista / energia harmônica superior (`> 12 kHz`) em **pelo menos `4,0 dB`** em comparação com um hard-clipper ideal sem limitação de slew rate.
  - *Origem da tolerância:* Especificação `SR = 0,3 V/µs` para $C_f = 30\text{ pF}$ no datasheet `LM308` (`F-01`, `F-02`).

- **CA-03 — Comportamento Reverso do Controle `FILTER` (`32,15 kHz` a `475,2 Hz`, 1ª Ordem) (`DOCUMENTED` / `REGRESSION`):**
  - *Sinal de entrada:* Senoide de `440 Hz` a `-12 dBFS` (ou ruído branco em pequeno sinal para medição direta do estágio `FILTER`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `drive.gain = 0.70`, `drive.level = 0.50`, comparando `drive.tone = 0.0` (`Filter` mínimo = brilhante), `drive.tone = 0.5` e `drive.tone = 1.0` (`Filter` máximo = escuro, $f_c = 475,2\text{ Hz}$).
  - *Método de medição:* Centróide espectral da saída e atenuação do estágio `FILTER` em `5000 Hz` relativa a `200 Hz`.
  - *Critério e tolerância:*
    1. O centróide espectral em `drive.tone = 0.0` deve ser estritamente **maior** que em `drive.tone = 0.5`, que por sua vez deve ser estritamente **maior** que em `drive.tone = 1.0` ($\text{SC}(0,0) > \text{SC}(0,5) > \text{SC}(1,0)$).
    2. Em `drive.tone = 1.0` ($f_c = 475,2\text{ Hz}$), a atenuação do filtro de 1ª ordem em `4752 Hz` ($10 \times f_c$, uma década acima) deve ser de **`20,0 dB ± 2,0 dB`** (inclinação de `-20 dB/década` = 1ª ordem RC, e não `-40 dB/década` de um biquad de 2ª ordem).
  - *Origem da tolerância:* Rede RC de 1ª ordem $R_7 + P_2 = 101,5\text{ k}\Omega$, $C_8 = 3,3\text{ nF}$ (`F-01` §"Filter Stage").

- **CA-04 — Hard-Clipping Simétrico de Silício (`2× 1N914`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `drive.gain = 0.80`, `drive.tone = 0.20`, `drive.level = 0.50`.
  - *Método de medição:* FFT de `16 384` amostras para medir $H_3 - H_2$.
  - *Critério e tolerância:* Predominância de harmônicos ímpares com $H_3 - H_2 \ge 20,0\text{ dB}$.
  - *Origem da tolerância:* Par antiparalelo simétrico `1N914` (`F-01`).

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `gain`, `tone`, `level`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/drive.rs` (e futuro `crates/ampforge-dsp/src/fx/pedals/proco_rat.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.rat.*`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_031_pedals_dsp.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Revisão histórica fixada em `1984–1988 LM308N` e separada de `RAT2 OP07DP` e `Turbo RAT`.
- [x] Omissão do degrau RC de `1538,9 Hz` e uso de biquad de 2ª ordem no `FILTER` em `drive.rs:440–443` diagnosticados.
- [x] Critérios `CA-01` a `CA-05` classificados e com condições completas de medição.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` ElectroSmash — *ProCo RAT Circuit Analysis* (`https://web.archive.org/web/20240115000000/https://www.electrosmash.com/proco-rat`).
- `[F-02]` National Semiconductor / Texas Instruments — *LM108/LM208/LM308 Operational Amplifiers Datasheet* (SNOSBT2).

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, fixou-se a revisão histórica na variante `1984–1988 LM308N` (com capacitor de compensação de `30 pF` nos pinos 1–8), diagnosticou-se a ausência do ramo HPF de `1538,9 Hz` ($47\text{ }\Omega + 2,2\text{ }\mu\text{F}$) em `drive.rs:441` e corrigiu-se a ordem do filtro `FILTER` de 2ª ordem para 1ª ordem RC (`-20 dB/década`).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Duplo ramo HPF na perna inversora: `60,44 Hz` ($560\text{ }\Omega + 4,7\text{ }\mu\text{F}$) e `1538,9 Hz` ($47\text{ }\Omega + 2,2\text{ }\mu\text{F}$) | `F-01` §"Clipping Amplifier" | Spec 061 / `CA-01` | `crates/ampforge-dsp/src/fx/drive.rs:441 @ c20cc65` (possui apenas HPF simples em `60 Hz`, sem o degrau de `1538,9 Hz`) | Ausente | `SPECIFIED` |
| Limitação física de Slew Rate (`0,3 V/µs`, $C_3 = 30\text{ pF}$) no `LM308N` | `F-01`, `F-02` | Spec 061 / `CA-02` | `crates/ampforge-dsp/src/fx/drive.rs:574–584 @ c20cc65` (implementa clamp de derivada seguido de leaky integrator arbitrário `0.65/0.35`) | `contract_031_pedals_dsp.rs` | `IMPLEMENTED_UNVERIFIED` |
| Filtro passa-baixas reverso `FILTER` de 1ª ordem RC ($f_c \in [475,2\text{ Hz},\; 32,15\text{ kHz}]$) | `F-01` §"Filter Stage" | Spec 061 / `CA-03` | `crates/ampforge-dsp/src/fx/drive.rs:442–443 @ c20cc65` (direção reversa implementada, mas usa biquad de 2ª ordem em vez de RC de 1ª ordem) | `contract_031_pedals_dsp.rs` (`test_ca_02_rat_filter_reverso`) | `TESTED` (direção) / `SPECIFIED` (ordem 1ª) |
| Hard-clipping shunt de silício (`2× 1N914` + $R_6 = 1\text{ k}\Omega$) | `F-01` | Spec 061 / `CA-04` | `crates/ampforge-dsp/src/fx/drive.rs:587–596 @ c20cc65` | `contract_031_pedals_dsp.rs`, `contract_040_pedals_parity.rs` | `TESTED` |
| Execução zero-alloc em tempo real com oversampling `8×` | Spec 053, Spec 079 | Spec 061 / `CA-05` | `crates/ampforge-dsp/src/fx/drive.rs:463–491 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
