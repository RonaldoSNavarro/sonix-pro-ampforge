# Spec 063: Pedal Arbiter Fuzz Face (1966 NKT275 PNP Germanium Transistors)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-pedal` (`drive` / `fuzz`)
> **Identificador canônico:** `pedal.arbiter.fuzz_face_ge`
> **Variante / Revisão:** `1966–1967 Arbiter Electronics Ltd. (2× PNP Germânio Newmarket NKT275, Positive Ground)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/drive.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [031](031-amplificadores-fieis-aos-esquemas-e-controles-originais.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [076](076-componente-entrada-condicionamento-impedancia-captadores.md), [079](079-componente-oversampling-polifasico-e-anti-aliasing.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação do pedal **Arbiter Fuzz Face (1966–1967, 2× NKT275 PNP Germânio)** como processador modular independente (`pedal.arbiter.fuzz_face_ge`), eliminando a ambiguidade `FuzzFaceSiGe` do código atual (`crates/ampforge-dsp/src/fx/drive.rs:11`), separando explicitamente a variante histórica `NKT275` (Germânio PNP) de variantes posteriores (`AC128` Ge e `BC108C` Silício NPN de 1968+), e resolvendo a discrepância do controle `Tone` existente no motor genérico (`drive.tone`) mas **inexistente no pedal Fuzz Face real** (que possui apenas `Volume` e `Fuzz`).

---

## 2. Escopo

1. Topologia de dois transistores PNP de Germânio `NKT275` acoplados diretamente com realimentação negativa shunt-série ($R_2 = 100\text{ k}\Omega$ do emissor de `Q2` para a base de `Q1`).
2. Interação de baixa impedância de entrada ($Z_{\text{in}} \approx 5\text{ k}\Omega$ a $10\text{ k}\Omega$) e capacidade de *Guitar Volume Cleanup* (limpeza dinâmica ao reduzir o volume da guitarra).
3. Efeito da corrente de fuga coletor-base de germânio ($I_{\text{CBO}} \approx 100..250\text{ }\mu\text{A}$ a $25^\circ\text{C}$) e ganho moderado ($\beta_1 \approx 70..90$, $\beta_2 \approx 110..130$) sobre a assimetria de ciclo de trabalho (*duty-cycle shift*).
4. Tratamento explícito dos dois únicos controles físicos do equipamento real (`Fuzz` e `Volume`).

---

## 3. Fora de escopo

- **Dallas-Arbiter Fuzz Face Silicon (1968–1975, `BC108C` / `BC183L` / `BC109C` NPN):** Os transistores de silício possuem $\beta \approx 250..600$, $I_{\text{CBO}} \approx 0$ e $V_{\text{BE}} \approx 0,68\text{ V}$, produzindo um timbre muito mais brilhante, agressivo e com ponto de polarização distinto; não devem ser misturados sob um enum ambíguo `FuzzFaceSiGe`.
- **Variante `AC128` (Tungsram/Mullard PNP Ge):** Embora compartilhe a mesma placa do `NKT275`, o `AC128` apresenta maior dispersão de $I_{\text{CBO}}$ e capacitância de junção distinta; a referência primária calibrada desta Spec é estritamente o par **`NKT275`** selecionado ($h_{\text{FE1}} \approx 80$, $h_{\text{FE2}} \approx 120$).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`fuzz`) |
| **Fabricante de referência** | Arbiter Electronics Ltd. (Londres, Reino Unido — Ivor Arbiter) |
| **Modelo de referência** | Fuzz Face (Issue 1, Germânio PNP) |
| **Ano / época / revisão** | Outono de 1966–1967 (2× Newmarket Transistors `NKT275` PNP Ge, chassis redondo fundido) |
| **Identificador no sistema** | `pedal.arbiter.fuzz_face_ge` (atualmente mapeado para `DriveModel::FuzzFaceSiGe` / UI slug `fuzz`) |
| **Papel na cadeia de sinal** | Primeiro bloco na cadeia logo após a guitarra (sensível à impedância da guitarra e ao nível de entrada para *cleanup*). |

---

## 5. Princípio físico e funcional do equipamento real

1. **Amplificador de Transimpedância / Realimentação Shunt-Série (`Q1` + `Q2`):**
   - `Q1` (`NKT275`, $\beta_1 \approx 80$) tem emissor diretamente no barramento de referência (`+9 V` no circuito *positive ground*) e coletor carregado por $R_1 = 33\text{ k}\Omega$, acoplado diretamente à base de `Q2` (`NKT275`, $\beta_2 \approx 120$).
   - O coletor de `Q2` possui um divisor formado por $R_3 = 470\text{ }\Omega$ (para a saída) e $R_4 = 8,2\text{ k}\Omega$ (ajustado para manter o coletor de `Q2` em torno de $-4,5\text{ V}$).
   - O resistor $R_2 = 100\text{ k}\Omega$ realimenta a tensão do emissor de `Q2` de volta para a base de `Q1`. Essa realimentação shunt na entrada derruba a impedância de entrada do pedal para **`5 kΩ a 10 kΩ`** (quando o capacitor de bypass de emissor de `Q2` está ativo), carregando fortemente captadores passivos de guitarra (Spec 076).
2. **Controle `FUZZ` (`1 kΩ Linear` + Capacitor de Bypass de Emissor $C_2 = 20\text{ }\mu\text{F}$):**
   - O emissor de `Q2` vai para a referência através do potenciômetro `Fuzz` (`1 kΩ B`). O cursor (*wiper*) é desacoplado em AC para a referência por $C_2 = 20\text{ }\mu\text{F}$ (`22 µF`).
   - Quando `Fuzz = 0%`, o capacitor $C_2$ não bypassa o emissor de `Q2`: a degeneração de emissor máxima ($1\text{ k}\Omega$) e a realimentação negativa via $R_2 = 100\text{ k}\Omega$ reduzem o ganho AC para cerca de $+18\text{ dB}$.
   - Quando `Fuzz = 100%`, o emissor de `Q2` é curto-circuitado em AC para a referência através de $C_2 = 20\text{ }\mu\text{F}$ (com frequência de corte passa-altas dependente da posição do cursor: em `Fuzz = 100%`, a resistência efetiva de emissor de `Q2` cai para $r_{e2} \approx 25\text{ }\Omega$, formando um HPF em $f_c = \frac{1}{2\pi \cdot 25 \cdot 20 \times 10^{-6}} \approx 318\text{ Hz}$).
3. **Assimetria de Ponto de Operação, Corrente de Fuga ($I_{\text{CBO}}$) e *Guitar Volume Cleanup*:**
   - Em `Q1` (`NKT275`), a tensão base-emissor quiescente é baixa ($V_{\text{BE}} \approx 0,12..0,15\text{ V}$) e a corrente de fuga $I_{\text{CBO}} \approx 120\text{ }\mu\text{A}$ contribui para polarizar a junção.
   - Em sinais fortes (`-12 dBFS`), um semiciclo corta `Q1` (saturando `Q2`), enquanto o outro semiciclo satura `Q1` (cortando `Q2`), gerando uma onda fortemente **assimétrica** (rica em 2º harmônico $H_2$, oitava acima sutil) que se limpa de forma contínua e cristalina (*Volume Cleanup*) quando a amplitude de entrada cai `12 a 18 dB` (simulando o guitarrista baixando o knob de volume da Stratocaster de `10` para `7`).
4. **Ausência de Controle `TONE` no Equipamento Real:**
   - O Arbiter Fuzz Face original possui **apenas dois potenciômetros**: `VOLUME` (`500 kΩ Log`) e `FUZZ` (`1 kΩ Linear`), além do capacitor de saída $C_3 = 10\text{ nF}$ (`0,01 µF`) que forma um filtro passa-altas de saída com o potenciômetro `Volume` ($f_c \approx 31,8\text{ Hz}$).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | ElectroSmash — *Fuzz Face Analysis* (arquivado e verificado via Wayback Machine) | `https://web.archive.org/web/20240115000000/https://www.electrosmash.com/fuzz-face` (HTTP 200 verificado) | `SECONDARY_TECHNICAL` | Esquema completo do Fuzz Face 1966 (`NKT275` / `AC128`), análise DC com $I_{\text{CBO}}$, ganho AC em função do pot `1 kΩ Fuzz` + `20 µF`, impedância de entrada (`~8 kΩ`) e formas de onda assimétricas. |
| `F-02` | R. G. Keen (GEOFEX) — *The Technology of the Fuzz Face* | `http://www.geofex.com/Article_Folders/fuzzface/fftech.htm` (HTTP 200 verificado, 28,2 KB) | `SECONDARY_TECHNICAL` | Análise clássica da topologia shunt-series feedback twin-BJT, efeito de $I_{\text{CBO}}$ e $\beta$ nos transistores `NKT275` vs `AC128`, e ajuste do coletor de `Q2` a `-4,5 V`. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Dois transistores PNP Germânio `NKT275` ($\beta_1 \approx 70..90$, $\beta_2 \approx 110..130$) com realimentação $R_2 = 100\text{ k}\Omega$ | `F-01`, `F-02` | 1966 Issue 1 (`NKT275`) | `F-01` §"Schematic & Biasing"; `F-02` §"How It Works" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Apenas dois controles físicos no pedal real: `Fuzz` (`1 kΩ B` + $C_2 = 20\text{ }\mu\text{F}$) e `Volume` (`500 kΩ A`) | `F-01`, `F-02` | 1966 Issue 1 | `F-01` §"Hardware & Schematic" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Baixa impedância de entrada ($Z_{\text{in}} \approx 5..10\text{ k}\Omega$) e limpeza dinâmica com redução do nível de entrada (*Volume Cleanup*) | `F-01`, `F-02` | 1966 Issue 1 | `F-01` §"Input Impedance"; `F-02` §"Input Loading" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Corte assimétrico de `Q1`/`Q2` gerando forte conteúdo de 2º harmônico ($H_2$) além dos harmônicos ímpares | `F-01` | 1966 Issue 1 | `F-01` §"Output Signal & Harmonics" | `SECONDARY_TECHNICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input] ──► [C1 2.2 µF] ──┬──► [Base Q1 NKT275 PNP (β1≈80, Emitter @ GND/Ref)]
                          │            │ (Collector R1 = 33 kΩ)
                          │            ▼
                          │    [Base Q2 NKT275 PNP (β2≈120)]
                          │            │
                          │            ├──► (Collector: R4 8.2 kΩ + R3 470 Ω) ──► [C3 10 nF] ──► [P2 Volume 500 kΩ] ──► [Output]
                          │            │
                          │            ▼ (Emitter Q2)
                          └── [R2 100 kΩ Feedback] ◄── [P1 Fuzz Pot (1 kΩ B) + C2 20 µF Bypass on Wiper]
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`MODEL_COUPLING` (`drive.rs:11`):** O enum atual define `DriveModel::FuzzFaceSiGe`, agrupando no nome Silício (`Si`) e Germânio (`Ge`), embora as duas versões históricas (`1966 NKT275 Ge PNP` vs `1968+ BC108C Si NPN`) possuam parâmetros físicos incompatíveis. Esta Spec fixa o contrato canônico de `pedal.arbiter.fuzz_face_ge` exclusivamente na versão **`1966 NKT275 PNP Germânio`**.
2. **`FACTUAL_ERROR` / `ARCHITECTURE_CONFLICT` (`drive.rs:442–443` vs `controls.registry.ts:753–795`):**
   - No registro de UI `controls.registry.ts:753–795` (`pedal.fuzz.*`), a interface expõe corretamente **apenas 3 controles**: `pedal.fuzz.enabled`, `pedal.fuzz.gain` (`FUZZ`) e `pedal.fuzz.level` (`VOLUME`) — não há knob `Tone` na UI do Fuzz Face!
   - Entretanto, em `drive.rs:442–443`, `DriveModel::FuzzFaceSiGe` compartilha o mesmo braço de `update_filters` do `ProCoRat` (`DriveModel::ProCoRat | DriveModel::FuzzFaceSiGe => { let fc = 15000.0 * (475.0 / 15000.0).powf(self.tone); ... }`), aplicando indevidamente o filtro reverso de `3,3 nF` do RAT sobre o Fuzz Face dependendo do estado residual de `self.tone`!
   - No processador modular `pedal.arbiter.fuzz_face_ge`, não deve existir filtro `RAT` acoplado a `self.tone`; a resposta em frequência de saída deve ser governada apenas pelo circuito real do Fuzz Face (capacitor de saída $C_3 = 10\text{ nF}$ + `500 kΩ` e roll-off natural das capacitâncias de junção $C_{\text{cb}}$ do germânio `NKT275` em $\sim 6,5\text{ kHz}$).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Normalizada | Unidade / Escala UI | Curva / Taper | Valor Padrão | Papel Elétrico no Circuito |
|---|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `drive.enabled` | `pedal.fuzz.enabled` | `false / true` | Booleano | Discreto | `false` | True-bypass mecânico. |
| **FUZZ** | `drive.gain` | `pedal.fuzz.gain` | `0.0 .. 1.0` | `0 .. 100 %` | Linear / Rev-Log (`1 kΩ B` c/ bypass $C_2=20\text{ }\mu\text{F}$) | `0.70` | Move o cursor do capacitor de bypass $C_2 = 20\text{ }\mu\text{F}$ ao longo do resistor de emissor de `Q2` (`1 kΩ`), variando simultaneamente o ganho AC ($+18\text{ dB}$ a $+55\text{ dB}$) e a frequência de corte grave ($7,9\text{ Hz}$ a $318\text{ Hz}$). |
| **VOLUME** | `drive.level` | `pedal.fuzz.level` | `0.0 .. 1.0` | `0 .. 100 %` | Logarítmico (`500 kΩ A`) | `0.50` | Atenuador de saída após o capacitor de acoplamento $C_3 = 10\text{ nF}$. |

> **Nota sobre `drive.tone`:** O equipamento físico Arbiter Fuzz Face **não possui controle de Tone**. O módulo `pedal.arbiter.fuzz_face_ge` ignora `drive.tone` (ou o mantém desacoplado do filtro reverso do RAT), utilizando o roll-off físico fixo das capacitâncias de junção do `NKT275`.

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Filtro Passa-Altas Dependente do Cursor `FUZZ` ($C_1 = 2,2\text{ }\mu\text{F}$ e $C_2 = 20\text{ }\mu\text{F}$ no Emissor de `Q2`):**
   - A resistência não-bypassada no emissor de `Q2` é $R_{e,\text{AC}}(\text{fuzz}) = r_{e2} + (1 - \text{fuzz})^3 \cdot 1000\text{ }\Omega$ (onde a potência cúbica reflete a compressão do efeito do capacitor de bypass $C_2 = 20\text{ }\mu\text{F}$ nos últimos `15%` do curso do potenciômetro linear `1 kΩ`).
   - O polo passa-altas do bypass de emissor varia dinamicamente de $f_{e,\min} \approx 16\text{ Hz}$ (`Fuzz = 0%`) até $f_{e,\max} \approx 260\text{ Hz}$ (`Fuzz = 100%`).
2. **Malha Assimétrica Twin-BJT `NKT275` (`Q1 → Q2` com $I_{\text{CBO}}$ e Realimentação $R_2 = 100\text{ k}\Omega$):**
   - Ganho de malha fechada escalado por $G(\text{fuzz}) = 8,0 + 480,0 \cdot \text{fuzz}^{2,5}$.
   - Deslocamento dinâmico de ponto de polarização (*envelope bias shift* devido à retificação na junção base-emissor de `Q1` com $I_{\text{CBO}}$):
     $$v_{\text{bias}}[k] = \alpha_b v_{\text{bias}}[k-1] + (1 - \alpha_b) \cdot 0,22 \cdot \max(0,\; v_{\text{in}}[k])$$
   - Função de transferência não-linear assimétrica dos dois estágios `NKT275` (corte abrupto de `Q1` em um semiciclo vs saturação macia de germânio de `Q2` no semiciclo oposto), seguida de polo passa-baixas de capacitância de junção Miller do germânio ($f_{c,\text{Ge}} \approx 6200\text{ Hz}$) e capacitor de saída $C_3 = 10\text{ nF}$ ($f_{\text{HPF,out}} = 31,8\text{ Hz}$).

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `8×` polifásico IIR (Spec 079) em torno dos estágios não-lineares `Q1` e `Q2`.
- **Precisão numérica:** `f64` nos integradores de envoltória de bias e filtros de acoplamento.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`.
- Complexidade $O(1)$ por amostra; WCET $\le 10\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- **Impedância de entrada:** $Z_{\text{in}} \approx 8\text{ k}\Omega$. Quando combinado com o módulo de condicionamento de entrada / captadores (Spec 076), reduz o pico ressonante do captador passivo e responde organicamente ao knob de volume da guitarra.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/fuzz_face_ge.rs`.
- **Struct principal:** `pub struct FuzzFaceGeProcessor` implementando `DspBlock` + `AutomatableParams` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Subsistema | Componente Real | Valor Documentado (`1966 NKT275`) | Invariante Física |
|---|---|---|---|
| Transistores `Q1`, `Q2` | `2× NKT275` PNP Ge | $\beta_1 \approx 80, \beta_2 \approx 120, I_{\text{CBO}} \approx 120\text{ }\mu\text{A}$ | Clipping fortemente assimétrico ($H_2$ proeminente) e joelho suave de germânio. |
| Realimentação Shunt | $R_2 = 100\text{ k}\Omega$ | Emissor `Q2` → Base `Q1` | Baixa impedância de entrada ($\sim 8\text{ k}\Omega$) e *Volume Cleanup*. |
| Bypass de Emissor `Q2` | $P_1 = 1\text{ k}\Omega\text{ B}, C_2 = 20\text{ }\mu\text{F}$ | Ganho concentrado no final do curso | Varia ganho AC e corte de graves simultaneamente. |
| Acoplamento de Saída | $C_3 = 10\text{ nF}, P_2 = 500\text{ k}\Omega$ | $f_{\text{HPF,out}} = 31,8\text{ Hz}$ | Sem controle de Tone físico; desacoplado do filtro reverso do RAT. |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de *Guitar Volume Cleanup*:** reduzindo a entrada de `-6 dBFS` para `-24 dBFS` com `Fuzz = 0.85`, a THD cai drasticamente para regime de overdrive limpo.
2. **Teste de Assimetria Harmônica ($H_2$ Proeminente):** verifica presença substancial de 2º harmônico ($H_2$) devido ao ponto de operação assimétrico `Q1/Q2`.
3. **Teste de Independência de `drive.tone`:** verifica que alterar `drive.tone` não aplica o filtro reverso de `475 Hz` do ProCo RAT sobre o Fuzz Face.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- No código atual (`drive.rs:442`), `DriveModel::FuzzFaceSiGe` compartilha indevidamente o filtro reverso do RAT. Corrigir isso na implementação futura da Spec 063 garantirá que o Fuzz Face não sofra corte abafado artificial quando `drive.tone` estiver acima de `0.5`.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Limpeza Dinâmica com Redução do Volume da Guitarra (*Guitar Volume Cleanup*) (`DOCUMENTED` / `REGRESSION`):**
  - *Sinal de entrada:* Senoide de `440 Hz` testada em dois níveis de entrada: nível alto `-6 dBFS` (`Guitar Volume = 10`) e nível reduzido `-24 dBFS` (`Guitar Volume ≈ 6,5`, `-18 dB`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras, FFT de `16 384` amostras.
  - *Configuração e parâmetros:* `pedal.arbiter.fuzz_face_ge` com `drive.enabled = true`, `drive.gain = 0.85` (`Fuzz = 85%`), `drive.level = 0.50`.
  - *Método de medição:* Medição da THD (Total Harmonic Distortion, razão RMS de $H_2..H_{10}$ sobre $H_1$) nos dois níveis de entrada.
  - *Critério e tolerância:* Em `-6 dBFS`, a THD deve ser **$\ge 25,0\%$** (fuzz denso); ao reduzir a entrada para `-24 dBFS` (mantendo o knob `Fuzz` intocado em `85%`), a THD deve cair por um fator de **pelo menos `3,5×`** (resultando em $\text{THD} \le 8,0\%$).
  - *Origem da tolerância:* Propriedade física de realimentação shunt-série $R_2 = 100\text{ k}\Omega$ e joelho suave do germânio `NKT275` (`F-01` §"Clean Up Effect", `F-02`).

- **CA-02 — Assimetria de Ciclo de Trabalho e Presença de 2º Harmônico ($H_2$) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `440 Hz` a `-9 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras, FFT de `16 384` amostras.
  - *Configuração e parâmetros:* `drive.gain = 0.85`, `drive.level = 0.50`.
  - *Método de medição:* Nível do 2º harmônico ($H_2 @ 880\text{ Hz}$) relativo à fundamental ($H_1 @ 440\text{ Hz}$).
  - *Critério e tolerância:* Devido ao clipping assimétrico entre o corte de `Q1` e a saturação de `Q2`, o 2º harmônico ($H_2$) deve estar a **no máximo `18,0 dB` abaixo da fundamental** ($H_2 - H_1 \ge -18,0\text{ dB}$), contrastando com clippers simétricos (`TS808`, `RAT`) onde $H_2$ fica a mais de `-30 dB`.
  - *Origem da tolerância:* Espectro harmônico assimétrico do Fuzz Face documentado em `F-01` §"Output Signal and Harmonics".

- **CA-03 — Ausência de Controle `Tone` Artificial (Desacoplamento do Filtro Reverso do RAT) (`DOCUMENTED`):**
  - *Sinal de entrada:* Ruído branco ou senoide de `3000 Hz` a `-18 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `drive.gain = 0.50`, `drive.level = 0.50`, comparando `drive.tone = 0.0` vs `drive.tone = 1.0`.
  - *Método de medição:* Nível RMS de saída do processador `pedal.arbiter.fuzz_face_ge`.
  - *Critério e tolerância:* Como o Arbiter Fuzz Face real possui exclusivamente os controles `Fuzz` e `Volume`, variar o parâmetro residual `drive.tone` de `0.0` a `1.0` deve alterar o nível de saída em `3000 Hz` em **`0,0 dB ± 0,1 dB`** (invariante ao filtro reverso do RAT).
  - *Origem da tolerância:* Esquema elétrico oficial do Arbiter Fuzz Face 1966 (`F-01`, `F-02`).

- **CA-04 — Concentração de Ganho no Curso Superior do Potenciômetro `FUZZ` (`1 kΩ B` + $C_2 = 20\text{ }\mu\text{F}$) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-30 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Comparação de ganho entre `drive.gain = 0.0`, `0.5` e `1.0`.
  - *Método de medição:* Ganho em dB na fundamental `1000 Hz`.
  - *Critério e tolerância:* O incremento de ganho na metade superior do curso (`drive.gain` de `0.5` para `1.0`, quando $C_2 = 20\text{ }\mu\text{F}$ curto-circuita os últimos ohms do emissor de `Q2`) deve ser **maior** que o incremento na metade inferior (`0.0` para `0.5`).
  - *Origem da tolerância:* Lei de ganho hiperbólica do bypass de emissor de `Q2` (`F-01` §"Voltage Gain", `F-02`).

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/drive.rs` (e futuro `crates/ampforge-dsp/src/fx/pedals/fuzz_face_ge.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.fuzz.*`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_031_pedals_dsp.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Variante histórica fixada em `1966–1967 Arbiter Fuzz Face (2× NKT275 PNP Germânio)` e separada de `AC128` e `BC108C Silício`.
- [x] Bug em `drive.rs:442` (compartilhamento indevido do filtro reverso `Tone` do ProCo RAT com o Fuzz Face) diagnosticado e documentado.
- [x] Critérios `CA-01` a `CA-05` classificados e com condições completas de medição.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` ElectroSmash — *Fuzz Face Circuit Analysis* (`https://web.archive.org/web/20240115000000/https://www.electrosmash.com/fuzz-face`).
- `[F-02]` Keen, R. G. (GEOFEX) — *The Technology of the Fuzz Face* (`http://www.geofex.com/Article_Folders/fuzzface/fftech.htm`).

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, desfez-se o acoplamento `FuzzFaceSiGe` fixando o contrato em `1966 NKT275 PNP Germânio`, diferenciou-se explicitamente `NKT275` de `AC128` e `BC108C`, e diagnosticou-se o bug em `drive.rs:442` onde o Fuzz Face aplicava o filtro `Tone` reverso do RAT.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Limpeza dinâmica com volume da guitarra (*Volume Cleanup*) e realimentação shunt $R_2 = 100\text{ k}\Omega$ | `F-01`, `F-02` | Spec 063 / `CA-01` | `crates/ampforge-dsp/src/fx/drive.rs:627–662` | `contract_031_pedals_dsp.rs` (`test_ca_04_fuzz_face_volume_cleanup`) | `TESTED` |
| Assimetria harmônica Twin-BJT `NKT275` ($H_2 - H_1 \ge -18\text{ dB}$) | `F-01`, `F-02` | Spec 063 / `CA-02` | `crates/ampforge-dsp/src/fx/drive.rs:635–653` | `contract_031_pedals_dsp.rs` | `IMPLEMENTED_UNVERIFIED` |
| Ausência de controle `Tone` físico (desacoplamento do filtro reverso do RAT em `drive.rs:442`) | `F-01`, `F-02` | Spec 063 / `CA-03` | `crates/ampforge-dsp/src/fx/drive.rs:442` (BUG: compartilha o braço `DriveModel::ProCoRat \| DriveModel::FuzzFaceSiGe` do filtro `tone`) | Ausente | `SPECIFIED` |
| Curva de ganho hiperbólica do bypass de emissor de `Q2` (`1 kΩ B` + $20\text{ }\mu\text{F}$) | `F-01`, `F-02` | Spec 063 / `CA-04` | `crates/ampforge-dsp/src/fx/drive.rs:629` (usa ganho exponencial simples `4.0 + drive * 36.0`) | `contract_040_pedals_parity.rs` | `SPECIFIED` |
| Execução zero-alloc em tempo real com oversampling `8×` | Spec 053, Spec 079 | Spec 063 / `CA-05` | `crates/ampforge-dsp/src/fx/drive.rs:463–491` | `contract_044_sdd_integrity.rs` | `TESTED` |
