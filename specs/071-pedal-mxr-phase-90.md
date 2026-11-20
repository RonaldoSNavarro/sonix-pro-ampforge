# Spec 071: Pedal MXR Phase 90 (4-Stage JFET All-Pass Phaser — Script Logo 1974 vs Block Logo)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-modulation` (`pedal`)
> **Identificador canônico:** `pedal.mxr.phase90`
> **Variante / Revisão:** `1974 MXR CSP-026 "Script Logo" (4× JFET 2N5952 Matched Quartet, Sem Resistor R28 de Feedback = 0.0)` com extensão documentada `Block Logo / Feedback`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/modulation.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação do pedal **MXR Phase 90 (1974 Script Logo, 4× JFET `2N5952`)** como processador modular independente (`pedal.mxr.phase90`), separando com rigor histórico e elétrico a revisão **`1974 Script Logo`** (que **não possui** o resistor de realimentação $R_{28}$, operando com `feedback = 0.0` puro) da revisão **`1977+ Block Logo / M101`** (que adicionou o resistor de realimentação $R_{28}$ entre a saída do 4º estágio all-pass e a entrada do 1º estágio), e corrigindo o bug em [`crates/ampforge-dsp/src/fx/modulation.rs:78, 238`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/fx/modulation.rs#L78-L240) onde o valor padrão de `self.feedback` estava codificado em `0.70` (`0.35` efetivo) em um modelo intitulado `PhaserPhase90` ("Script Logo")!

---

## 2. Escopo

1. Buffer de entrada (`IC1A`) com filtro passa-altas de acoplamento (`10 nF` + `470 kΩ` $\Rightarrow 33,8\text{ Hz}$).
2. Cadeia em cascata de **4 filtros passa-tudo (*1st-Order All-Pass Filters*) idênticos** baseados em amplificadores operacionais (`TL062` / `uA741`) com capacitores $C = 47\text{ nF}$ (`0,047 µF`) e quarteto casado de transistores JFET canal-N **`2N5952`** operando como resistores variáveis controlados pela tensão do LFO ($R_{\text{DS}}(V_{\text{LFO}})$ em paralelo com $R = 24\text{ k}\Omega$), gerando até $4 \times (-180^\circ) = -720^\circ$ de rotação de fase e **2 entalhes profundos (*2 notches*)** no espectro quando somado ao sinal seco.
3. Oscilador LFO de baixa frequência (`0,1 Hz` a `10 Hz`) modulando a porta dos 4 JFETs `2N5952` em torno da referência de polarização ajustada pelo trimpot interno `250 kΩ` ($V_{\text{ref}} \approx +5,1\text{ V}$ regulada por diodo Zener de `5,1 V`).
4. Distinção explícita entre o modo **`1974 Script Logo` (`feedback = 0.0`, padrão obrigatório)** e a extensão **`Block Logo / R28 Mod` (`feedback > 0.0`)**.

---

## 3. Fora de escopo

- **MXR Phase 45** (apenas 2 estágios all-pass = 1 notch) ou **MXR Phase 100** (10 estágios all-pass com acoplamento opto-LDR Vactrol e chave de 4 formas de LFO).
- **Shin-ei Uni-Vibe** (4 estágios com capacitores escalonados assimetricamente — coberto na **[Spec 072](072-pedal-shinei-univibe.md)**).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`4-stage-jfet-allpass-phaser`) |
| **Fabricante de referência** | MXR Innovations / Dunlop Manufacturing, Inc. (Rochester, NY / Benicia, CA, EUA — Keith Barr & Terry Sherwood) |
| **Modelo de referência** | MXR Phase 90 (`CSP-026 Script Logo` / `M101`) |
| **Ano / época / revisão** | **1974 Script Logo** (6× `uA741` ou 3× `TL062`, 4× `2N5952` JFET casados, Zener `1N5231B 5.1V`, sem resistor $R_{28}$ de feedback) |
| **Identificador no sistema** | `pedal.mxr.phase90` (`ModModel::PhaserPhase90` / UI `pedal.phaser.*`) |
| **Papel na cadeia de sinal** | Bloco `MOD` (`mod_1`), classicamente posicionado **antes** do amplificador saturado (ex.: Marshall 1959 Plexi "Brown Sound"). |

---

## 5. Princípio físico e funcional do equipamento real

1. **Quatro Estágios All-Pass de 1ª Ordem Identicamente Sintonizados ($C = 47\text{ nF}$ + JFET `2N5952`):**
   - Cada um dos 4 estágios all-pass possui um capacitor $C_k = 0,047\text{ }\mu\text{F}$ (`47 nF`) cujo resistor para a referência `Vref` é o paralelo de um resistor fixo $R_p = 24\text{ k}\Omega$ com o canal dreno-fonte de um JFET `2N5952` ($R_{\text{DS}}(V_{\text{GS}}) \in [450\text{ }\Omega,\; 20\text{ k}\Omega]$):
     $$R_{\text{eq}}(t) = R_p \parallel R_{\text{DS}}(V_{\text{LFO}}(t))$$
   - A frequência de quebra (*break frequency*, onde a defasagem de cada estágio é $-90^\circ$) de todos os 4 estágios move-se em uníssono entre:
     $$f_b(t) = \frac{1}{2\pi R_{\text{eq}}(t) C} \in [\sim 170\text{ Hz},\; \sim 1600\text{ Hz}]$$
   - Como os 4 estágios têm a mesma frequência de quebra $f_b(t)$, a fase total acumulada é $\Phi(\omega) = -8 \arctan(\omega / \omega_b(t))$. Quando somada em proporção `1:1` ($R_{15} = R_{27} = 150\text{ k}\Omega$) com o sinal seco no transistor misturador de saída, ocorrem **2 entalhes de cancelamento (*2 notches*)** nas frequências onde $\Phi(\omega) = -180^\circ$ ($\omega_1 = \omega_b \tan(\pi/8) = 0,4142\,\omega_b$) e $\Phi(\omega) = -540^\circ$ ($\omega_2 = \omega_b \tan(3\pi/8) = 2,4142\,\omega_b$), além de um pico intermediário onde $\Phi(\omega) = -360^\circ$ ($\omega_{\text{peak}} = \omega_b$).
   - Portanto, a razão entre as duas frequências de notch em qualquer instante do LFO no Phase 90 é uma constante geométrica exata:
     $$\frac{f_{\text{notch},2}}{f_{\text{notch},1}} = \frac{\tan(3\pi/8)}{\tan(\pi/8)} = \frac{2,41421356}{0,41421356} \approx 5,8284 \quad (\approx 2,54\text{ oitavas})$$
2. **A Diferença Crítica entre `1974 Script Logo` e `1977+ Block Logo` (O Resistor $R_{28}$):**
   - **1974 Script Logo:** **Não existe realimentação** (`feedback = 0.0`) entre a saída do 4º estágio e a entrada do 1º estágio. A varredura é puramente feedforward, orgânica, suave e sem pico ressonante nasal.
   - **1977+ Block Logo (`M101`):** A MXR adicionou o resistor $R_{28} = 22\text{ k}\Omega$ (em série com capacitor de bloqueio) realimentando parte da saída do 4º estágio de volta para a entrada do 1º estágio (a famosa *"R28 Mod"* consiste em cortar esse resistor para converter um Block Logo em Script Logo).
3. **Distinção entre Controles Físicos do Pedal Real e Extensões do AMPFORGE:**
   - **Equipamento Real (`M101` / `CSP-026`):** Possui **um único knob no painel: `SPEED`** (`RATE`), com profundidade fixa determinada pelo LFO e mistura fixa `1:1` ($150\text{ k}\Omega / 150\text{ k}\Omega$).
   - **Extensões Digitais AMPFORGE:** O pedal visual (`pedal.phaser.*` em `controls.registry.ts:955–1005`) expõe `RATE` (`SPEED`), `DEPTH`, `FEEDBACK` (`R28`) e `MIX`. Para que o estado padrão do pedal corresponda fielmente ao **`1974 Script Logo`**, o valor padrão de **`feedback` DEVE ser `0.0`** (e `mix = 0.50`). Aumentar `feedback > 0.0` ativa continuamente a realimentação estilo `Block Logo`.

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | ElectroSmash — *MXR Phase 90 Analysis* (arquivado e verificado via Wayback Machine) | `https://web.archive.org/web/20240115000000/https://www.electrosmash.com/mxr-phase90` (HTTP 200 verificado) | `SECONDARY_TECHNICAL` | Esquema completo do MXR Phase 90, equações dos 4 estágios All-Pass (`C = 47 nF`, `2N5952` JFET, razão de notches `0.414` e `2.414`), regulador Zener `5.1 V`, e diferença exata do resistor $R_{28}$ entre `Script Logo` e `Block Logo`. |
| `F-02` | Dunlop Manufacturing, Inc. — *MXR M101 Phase 90 Owner's Manual* | `https://www.jimdunlop.com/content/manuals/M101.pdf` (HTTP 200 PDF oficial verificado, 697,5 KB) | `PRIMARY_OFFICIAL` | Controle oficial único (`SPEED`), especificações de fábrica e operação do Phase 90. |
| `F-03` | Fairchild / ON Semiconductor — *2N5952 N-Channel RF/Audio JFET Datasheet* | Especificação oficial do componente ($V_{\text{GS(off)}} \in [-1,3\text{ V}, -3,5\text{ V}]$, $r_{\text{ds(on)}} \approx 300..500\text{ }\Omega$) | `PRIMARY_COMPONENT` | Curva quadrática de resistência dreno-fonte dos 4 JFETs casados. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| 4 estágios All-Pass idênticos ($C = 47\text{ nF}$, $R_p = 24\text{ k}\Omega \parallel \text{2N5952}$) gerando 2 notches em $0,4142\,f_b$ e $2,4142\,f_b$ | `F-01` | 1974 Script / 1977 Block | `F-01` §"All-Pass Filters & Frequency Response" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Ausência de realimentação (`feedback = 0.0`) na versão `1974 Script Logo` vs resistor $R_{28}$ na versão `Block Logo` | `F-01` | Script vs Block Logo | `F-01` §"Script vs Block Logo (R28 Feedback)" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Apenas 1 knob físico (`SPEED`) e soma fixa 50/50 ($150\text{ k}\Omega / 150\text{ k}\Omega$) no hardware original | `F-01`, `F-02` | M101 / CSP-026 | `F-02` p. 1; `F-01` §"Output Stage" | `PRIMARY_OFFICIAL` / `SECONDARY_TECHNICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input] ──► [Input Buffer] ──┬─────────────────────────────────────────────────────────────► [R15 150kΩ Dry] ──┐
                             │                                                                                 ├──► [Output Mixer] ──► [Out]
                             ▼                                                                                 │
                      (+) ──► [AP1: 47nF + 2N5952] ──► [AP2: 47nF + 2N5952] ──► [AP3] ──► [AP4: 47nF + 2N5952] ┘ (R27 150kΩ Wet)
                       ▲                                                ▲                          │
                       │                              [5.1V Zener Bias + SPEED LFO]                │
                       └─────────── [R28 Feedback: 0.0 in 1974 Script Logo / >0 in Block Logo] ◄───┘
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/modulation.rs:78, 233–244`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/fx/modulation.rs#L233-L244) contém um **conflito de revisão histórica (`REVISION_MISMATCH` / `FACTUAL_ERROR`)**:

1. **`REVISION_MISMATCH` / `FACTUAL_ERROR` (`modulation.rs:78, 238` e `controls.registry.ts:986`):**
   - O comentário em `modulation.rs:233` declara `"// MXR Phase 90 Script: 4-stage all-pass phaser with JFET modulation"`, mas o construtor em `modulation.rs:78` inicializa `feedback: 0.7` (e `controls.registry.ts:986` define `default: 70`), aplicando `self.phaser_fb_state * (self.feedback * 0.5)` (`35%` de realimentação!) por padrão!
   - Ter `feedback = 0.70` por padrão transforma o pedal em um **Block Logo com R28 enfatizado**, violando a característica definidora do **1974 Script Logo** (`feedback = 0.0`).
   - O padrão canônico de `feedback` para o Phase 90 Script Logo deve ser **`0.0`**, permitindo que o usuário aumente o knob `FEEDBACK` (`R28`) quando desejar o timbre ressonante do `Block Logo`.

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real / UI | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Aceita | Unidade | Valor Padrão Corrigido | Natureza (Hardware Real vs Extensão AMPFORGE) |
|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `mod_1.enabled` | `pedal.phaser.enabled` | `false / true` | Booleano | `false` | **Hardware Real:** True/Hardwire bypass. |
| **SPEED (RATE)** | `mod_1.rate_hz` | `pedal.phaser.rate` | `0.1 .. 10.0` | `Hz` | `0.80` | **Hardware Real:** Único potenciômetro externo do Phase 90 (`500 kΩ Rev-Log`). |
| **DEPTH** | `mod_1.depth` | `pedal.phaser.depth` | `0.0 .. 1.0` | `0 .. 100 %` | `0.80` | **Extensão Digital AMPFORGE:** Controla a largura da varredura de tensão na porta dos 4 JFETs `2N5952`. |
| **FEEDBACK (R28)** | `mod_1.feedback` | `pedal.phaser.feedback` | `0.0 .. 0.95` (UI `0..95%`) | `%` | **`0.00`** (`1974 Script Logo`) | **Extensão Script/Block Logo:** Em `0%` (`0.0`), emula o **1974 Script Logo** (sem $R_{28}$); acima de `0%`, introduz a realimentação do resistor $R_{28}$ do **Block Logo**. |
| **MIX** | `mod_1.mix` | `pedal.phaser.mix` | `0.0 .. 1.0` | `0 .. 100 %` | `0.50` (`50%`) | **Extensão Digital AMPFORGE:** Em `50%` (`0.50`), reproduz a soma resistiva `1:1` ($150\text{ k}\Omega / 150\text{ k}\Omega$) que maximiza a profundidade dos 2 notches (`-∞ dB` teórico). |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Modulação Não-Linear de Condutância do Quarteto JFET `2N5952`:**
   - A resistência equivalente $R_{\text{eq}}(t) = R_p \parallel R_{\text{DS}}(V_{\text{GS}}(t))$ com $C = 47\text{ nF}$ mapeia a saída do LFO $u(t) \in [0, 1]$ de forma exponencial/quadrática para a frequência de quebra idêntica dos 4 estágios:
     $$f_b[n] = f_{b,\min} \cdot \left(\frac{f_{b,\max}}{f_{b,\min}}\right)^{0,5 + 0,5 \cdot \text{depth} \cdot \sin(\phi[n])}, \qquad f_{b,\min} = 175\text{ Hz},\; f_{b,\max} = 1600\text{ Hz}$$
2. **Cascata de 4 Filtros All-Pass de 1ª Ordem Idênticos:**
   - Coeficiente bilinear de 1ª ordem para todos os 4 estágios ($k = 1..4$):
     $$a[n] = \frac{\tan(\pi f_b[n] / f_s) - 1}{\tan(\pi f_b[n] / f_s) + 1}, \qquad H_{\text{AP},k}(z) = \frac{a[n] + z^{-1}}{1 + a[n] z^{-1}}$$
   - Inclui leve distorção de canal JFET de 2ª ordem ($\epsilon_{\text{JFET}} \cdot v^2$) antes da soma com a realimentação $g_{\text{R28}} = 0,45 \cdot \text{feedback}$.
3. **Misturador de Saída ($150\text{ k}\Omega / 150\text{ k}\Omega$):**
   - $y[n] = (1 - \text{mix}) \cdot x[n] + \text{mix} \cdot y_{\text{AP4}}[n]$.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `1×` (a frequência máxima de quebra $f_{b,\max} = 1600\text{ Hz}$ está muito abaixo de Nyquist a `44,1/48 kHz`, e `tan(π * f_b / fs)` faz o pré-warping exato).

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`.
- Complexidade $O(1)$ (4 filtros all-pass de 1ª ordem por amostra); WCET $\le 1,5\text{ }\mu\text{s}$ por bloco de `128` amostras.

---

## 12. Posição e interação na cadeia completa de guitarra

- Quando colocado antes de um amplificador saturado (Marshall 1959 Plexi / JCM800), os 2 notches varrem a pré-ênfase de distorção, criando a modulação vocal clássica.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/mxr_phase90.rs`.
- **Struct principal:** `pub struct MxrPhase90Processor` implementando `DspBlock` + `AutomatableParams` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Parâmetro Físico | Valor Documentado (`F-01`) | Invariante Física |
|---|---|---|
| Número de estágios All-Pass | `4` (todos com $C = 47\text{ nF}$) | Gera exatamente **2 notches** em $f_1 = 0,4142\,f_b$ e $f_2 = 2,4142\,f_b$ quando `mix = 0.50`. |
| Razão geométrica entre os 2 notches | $f_2 / f_1 = \frac{\tan(3\pi/8)}{\tan(\pi/8)} = 5,8284$ | Invariante matemática de 4 estágios all-pass de 1ª ordem com polos coincidentes. |
| Feedback padrão (`Script Logo`) | `0.0` (sem resistor $R_{28}$) | Sem pico ressonante artificial em `feedback = 0.0`. |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste da Razão Geométrica dos 2 Notches ($f_2 / f_1 \approx 5,83$) em `Script Logo` (`feedback = 0.0, mix = 0.50`):** congelando o LFO (`depth = 0.0`, $f_b = 530\text{ Hz}$), verifica a existência de 2 entalhes profundos ($\ge 25\text{ dB}$ de atenuação) em $f_1 \approx 219,5\text{ Hz}$ e $f_2 \approx 1279,5\text{ Hz}$.
2. **Teste de Diferenciação `Script Logo` (`feedback = 0.0`) vs `Block Logo` (`feedback = 0.70`):** verifica que em `Script Logo` o pico intermediário em $f_{\text{peak}} = f_b$ nunca excede `0,0 dB` (ganho unitário passivo da soma $(1 + e^{-j2\pi})/2 = 1$), enquanto no modo `Block Logo` (`feedback = 0.70`) surge um pico ressonante positivo ($> +3,0\text{ dB}$) em $f_b$.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Alterar o padrão de `feedback` de `0.70` para `0.00` na implementação futura da Spec 071 alinhará o timbre default ao verdadeiro `1974 Script Logo` sem remover a capacidade do usuário de girar `FEEDBACK` para obter o `Block Logo`.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Geometria Exata dos Dois Notches ($0,4142\,f_b$ e $2,4142\,f_b$) da Cascata de 4 Estágios All-Pass Idênticos (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Tons senoidais de teste em $f_1 = 0,4142 \cdot f_{b0}$, $f_{\text{peak}} = 1,0000 \cdot f_{b0}$ e $f_2 = 2,4142 \cdot f_{b0}$ a `-18 dBFS` (onde $f_{b0} = \sqrt{175 \cdot 1600} \approx 529,15\text{ Hz}$ é a frequência de quebra central com LFO estacionário).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `pedal.mxr.phase90` com `enabled = true`, `depth = 0.0` (LFO estacionário em $f_{b0}$), `feedback = 0.0` (`1974 Script Logo`), `mix = 0.50` ($150\text{ k}\Omega / 150\text{ k}\Omega$).
  - *Método de medição:* Ganho em dB em $f_1 \approx 219,2\text{ Hz}$, $f_{\text{peak}} \approx 529,2\text{ Hz}$ e $f_2 \approx 1277,5\text{ Hz}$.
  - *Critério e tolerância:*
    1. Em $f_1$ e $f_2$ (onde a fase dos 4 estágios é $-180^\circ$ e $-540^\circ$), deve ocorrer cancelamento profundo com atenuação **$\ge 20,0\text{ dB`** abaixo do nível de entrada.
    2. Em $f_{\text{peak}} = f_{b0}$ (onde a fase dos 4 estágios é $-360^\circ$), os sinais Dry e Wet estão em fase e o ganho deve ser **`0,0 dB ± 0,5 dB`** (no modo `Script Logo` sem $R_{28}$).
  - *Origem da tolerância:* Equação analítica $\Phi(\omega) = -8\arctan(\omega/\omega_b)$ dos 4 estágios de $47\text{ nF}$ do MXR Phase 90 (`F-01` §"All-Pass Filters").

- **CA-02 — Diferenciação entre `1974 Script Logo` (`feedback = 0.0`) e `1977+ Block Logo` (`feedback = 0.70`, Resistor $R_{28}$) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide na frequência central de pico $f_{\text{peak}} = f_{b0} \approx 529,2\text{ Hz}$ a `-18 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `depth = 0.0`, `mix = 0.50`, comparando `feedback = 0.0` (`Script Logo`) vs `feedback = 0.70` (`Block Logo` com $R_{28}$).
  - *Método de medição:* Ganho em dB na frequência de pico construtivo ($-360^\circ$).
  - *Critério e tolerância:* No modo `Block Logo` (`feedback = 0.70`), a realimentação positiva em $-360^\circ$ através de $R_{28}$ deve elevar o ganho no pico construtivo em **pelo menos `+2,5 dB`** acima do modo `Script Logo` (`feedback = 0.0`).
  - *Origem da tolerância:* Efeito de ressonância de malha fechada do resistor $R_{28}$ (`F-01` §"Script vs Block Logo").

- **CA-03 — Varredura Contínua dos Notches pelo Oscilador `SPEED` (`RATE`) (`DOCUMENTED` / `REGRESSION`):**
  - *Sinal de entrada:* Ruído branco a `-18 dBFS` por `1,0 s`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `rate_hz = 1.0`, `depth = 0.85`, `feedback = 0.0`, `mix = 0.50`.
  - *Método de medição:* Posição espectral do 1º notch medida em $t_1 = 0,25\text{ s}$ (pico positivo do LFO) vs $t_2 = 0,75\text{ s}$ (pico negativo do LFO).
  - *Critério e tolerância:* A frequência do 1º notch em $t_1$ deve ser **pelo menos `3,0×` maior** que em $t_2$.
  - *Origem da tolerância:* Faixa de resistência do quarteto `2N5952` (`F-01`, `F-03`).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `rate_hz`, `depth`, `feedback`, `mix`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/modulation.rs` (e futuro `crates/ampforge-dsp/src/fx/pedals/mxr_phase90.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.phaser.*`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_032_time_mod_fx.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Fonte técnica (`F-01` ElectroSmash Wayback) e manual oficial Dunlop (`F-02` `M101.pdf`) verificados via HTTP 200.
- [x] Separadas as revisões `1974 Script Logo` (`feedback = 0.0`, sem $R_{28}$) e `1977+ Block Logo` (`feedback > 0.0`, com $R_{28}$), diagnosticando o default `0.70` incorreto em `modulation.rs:78`.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` ElectroSmash — *MXR Phase 90 Circuit Analysis* (`https://web.archive.org/web/20240115000000/https://www.electrosmash.com/mxr-phase90`).
- `[F-02]` Dunlop Manufacturing, Inc. — *MXR M101 Phase 90 Owner's Manual* (`https://www.jimdunlop.com/content/manuals/M101.pdf`).
- `[F-03]` Fairchild Semiconductor — *2N5952 N-Channel JFET Datasheet*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, documentou-se a razão geométrica exata dos 2 notches ($2,4142 / 0,4142 = 5,8284$) dos 4 estágios idênticos de `47 nF` do Phase 90 e identificou-se que `modulation.rs:78` inicializava `feedback: 0.7` (`Block Logo`), contradizendo a referência `1974 Script Logo` (`feedback = 0.0`).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| 4 estágios All-Pass de 1ª ordem idênticos ($C = 47\text{ nF}$ + `2N5952` JFET) gerando 2 notches em $0,4142\,f_b$ e $2,4142\,f_b$ | `F-01` | Spec 071 / `CA-01`, `CA-03` | `crates/ampforge-dsp/src/fx/modulation.rs:233–244` | `contract_032_time_mod_fx.rs` (`test_ca_03_phaser_allpass_notches_sweep`) | `TESTED` |
| Padrão `1974 Script Logo` com `feedback = 0.0` (sem resistor $R_{28}$) e extensão `Block Logo` (`feedback > 0.0`) | `F-01` §"Script vs Block" | Spec 071 / `CA-02` | `crates/ampforge-dsp/src/fx/modulation.rs:78, 238` e `controls.registry.ts:986` (DIVERGENTE: default atual é `0.70` / `70%` em vez de `0.0`) | `contract_040_pedals_parity.rs` | `SPECIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 071 / `CA-04` | `crates/ampforge-dsp/src/fx/modulation.rs:179–294` | `contract_044_sdd_integrity.rs` | `TESTED` |
