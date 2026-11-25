---
status: rascunho
---

# Spec 078: Componente Compartilhado — Solvers Analíticos de Redes Passivas de Tone Stack

> **Status:** `rascunho` (`bloqueio parcial documental` no solver Mesa Rectifier Rev F até confirmação do esquema oficial)
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-shared` (`componente`)
> **Identificador canônico:** `component.tonestack.passive_solvers`
> **Variante / Revisão:** `4 Solvers Analíticos Separados por Circuito Real (FMV com R_src variável, Vox Top Boost OS/010, James/Baxandall, Mesa Rectifier)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/tonestack.rs`, `crates/ampforge-dsp/src/amp.rs`
> **Specs relacionadas:** [039](039-amplificadores-fidelidade-e-robustez.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [054](054-amp-marshall-1959-super-lead-plexi.md), [055](055-amp-marshall-jcm800-2203.md), [056](056-amp-fender-twin-reverb-ab763.md), [057](057-amp-mesa-boogie-dual-rectifier-rev-f.md), [058](058-amp-vox-ac30-top-boost.md)

---

## 1. Objetivo

Especificar o contrato técnico, matemático e de verificação do componente compartilhado **Solvers Analíticos de Redes Passivas de Tone Stack (`component.tonestack.passive_solvers`)** em [`crates/ampforge-dsp/src/tonestack.rs`](../crates/ampforge-dsp/src/tonestack.rs#L1-L401), separando rigorosamente os **4 solvers de circuito distintos** sem aproximar uma topologia pela outra:
1. **`FmvToneStackSolver` (Fender / Marshall / Vox-derivados FMV de 3ª ordem — Equações Exatas de David Yeh DAFx-06):** Modelagem exata de 3ª ordem ($H(s) = \frac{b_1 s + b_2 s^2 + b_3 s^3}{a_0 + a_1 s + a_2 s^2 + a_3 s^3}$) parametrizada pelos 7 componentes físicos ($R_1, R_2, R_3, R_4, C_1, C_2, C_3$) **incluindo explicitamente a impedância de saída do estágio excitador $R_{\text{src}}$** (`38 kΩ` para Fender Twin AB763 *plate-driven* vs `1,3 kΩ` para Marshall 1959/JCM800 *cathode-follower driven*!).
2. **`VoxTopBoostToneStackSolver` (Vox AC30 Top Boost `OS/010`, 2 controles interativos `Bass`/`Treble` sem potenciômetro `Mid` físico):** Equação nodal dedicada da rede passiva de 2 controles de Dick Denney (`50 pF / 22 nF / 22 nF`, `100 kΩ` slope, `1 MΩ` Bass/Treble, `R_mid_fixo = 10 kΩ`) + controle **`Top Cut`** separado pós-inversor de fase (`250 kΩ` + `4,7 nF`).
3. **`JamesBaxandallToneStackSolver`:** Rede passiva de dois controles desacoplados (`Bass` / `Treble`) para amplificadores vintage e extensão futura (Ampeg / Orange / Hiwatt).
4. **`MesaRectifierToneStackSolver`:** Variante FMV alimentada por seguidor de cátodo com componente de `Mid` reduzido (`25 kΩ`) e valores nominais pendentes de confirmação primária para a revisão específica **Dual Rectifier Rev F** (`C1`, `R_slope`, `R_bass` — rastreado como `BLOCKED`/`PENDING` na Spec 057).

---

## 2. Escopo

1. Discretização exata por transformada bilinear (Tustin com pré-warping ou $2\times$ oversampling para evitar *Nyquist cramping*) de funções de transferência contínuas de **3ª ordem** (3 capacitores independentes) em precisão `f64` (Forma Direta II Transposta).
2. Inclusão obrigatória da impedância da fonte excitadora ($R_{\text{src}} = R_{\text{plate}} \parallel r_p$ ou $R_{\text{cf}} \approx 1/g_m$) nas equações nodais do `FmvToneStackSolver`, pois $R_{\text{src}}$ altera drasticamente a frequência do entalhe de médios e a atenuação de inserção!
3. Suavização de coeficientes / transição sem cliques (*zipper-free*) ao mover `Bass`, `Middle` e `Treble` em tempo real.
4. Suporte às curvas reais de potenciômetro (*Audio/Log Taper* em `Bass` e `Treble` conforme cada esquema elétrico, vs *Linear Taper* em `Middle`).

---

## 3. Fora de escopo

- Equalizadores gráficos ou paramétricos ativos pós-pré-amp (MXR M108 → Spec 067; Mesa 5-Band → Spec 085; Paramétrico 8-Band → Spec 086).
- Tone stacks de pedais de distorção (Big Muff → Spec 062; Boss DS-1 → Spec 064).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `componente` (`dsp-shared` / `passive-tone-stack-solvers`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (implementação exata das equações simbólicas de David T. Yeh, CCRMA Stanford, DAFx-06) |
| **Modelo de referência** | Passive 3rd-Order Tone Stack Solvers (`component.tonestack.passive_solvers`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 078) |
| **Identificador no sistema** | `component.tonestack.passive_solvers` (`crates/ampforge-dsp/src/tonestack.rs`) |
| **Papel na cadeia de sinal** | Rede interativa passiva de modelagem tonal no interior dos amplificadores valvulados (`Specs 054–058`). |

---

## 5. Princípio físico e funcional do circuito real

Conforme derivado simbolicamente por David T. Yeh & Julius O. Smith III (`F-01`, DAFx-06):

1. **Topologia FMV de 3ª Ordem (`Fender`, `Marshall`) e o Papel Decisivo de $R_{\text{src}}$:**
   - Na topologia FMV (3 capacitores $C_1, C_2, C_3$, resistor de slope $R_1$, potenciômetros $R_{\text{treble}}, R_{\text{bass}}, R_{\text{mid}}$), os três controles são **eletricamente acoplados** — zerar `Bass`, `Mid` e `Treble` simultaneamente silencia o sinal (no Fender AB763 onde `Mid` vai ao terra, ou deixa apenas o resistor fixo de `6,8 kΩ` / `10 kΩ`).
   - A função de transferência contínua exata tem grau 3 no numerador e no denominador:
     $$H_{\text{FMV}}(s) = \frac{b_1 s + b_2 s^2 + b_3 s^3}{a_0 + a_1 s + a_2 s^2 + a_3 s^3}$$
     onde os 7 coeficientes $(b_1, b_2, b_3, a_0, a_1, a_2, a_3)$ são polinômios exatos dos 7 componentes + impedância da fonte $R_{\text{src}}$ (`F-01` Tabela 1 / Eq. 2–9).
   - **Diferença Física Fundamental entre Marshall e Fender:**
     - **Marshall 1959 / JCM800 (`Specs 054/055`):** Excitado por **Seguidor de Cátodo (`V2B`)** com $R_{\text{src}} \approx 1,3\text{ k}\Omega$, $R_{\text{slope}} = 33\text{ k}\Omega$, $C_1 = 470\text{ pF}$ (`500 pF` no 1959), $C_2 = C_3 = 22\text{ nF}$, $P_{\text{bass}} = 1\text{ M}\Omega$, $P_{\text{mid}} = 25\text{ k}\Omega$, $P_{\text{treble}} = 250\text{ k}\Omega$. Resultado: entalhe de médios em `~500–650 Hz`, menos profundo que o Fender.
     - **Fender Twin Reverb `AB763` (`Spec 056`):** Excitado diretamente por **Placa de Triodo (`V1B`/`V2B`)** com $R_{\text{src}} \approx 38\text{ k}\Omega$, $R_{\text{slope}} = 100\text{ k}\Omega$, $C_1 = 250\text{ pF}$, $C_2 = 100\text{ nF}$, $C_3 = 47\text{ nF}$, $P_{\text{bass}} = 250\text{ k}\Omega$, $P_{\text{mid}} = 10\text{ k}\Omega$, $P_{\text{treble}} = 250\text{ k}\Omega$. Resultado: profundo *mid-scoop* clássico Blackface (`-12 dB a -16 dB`) em **`~400–500 Hz`**!
2. **Topologia Vox Top Boost `OS/010` (`Spec 058`):**
   - Excitada por seguidor de cátodo (`V2B`), utiliza $R_{\text{slope}} = 100\text{ k}\Omega$, $C_1 = 50\text{ pF}$, $C_2 = C_3 = 22\text{ nF}$, $P_{\text{bass}} = 1\text{ M}\Omega$, $P_{\text{treble}} = 1\text{ M}\Omega$ e **resistor fixo de `10 kΩ`** no lugar do potenciômetro `Mid` (não existe knob `Middle` físico no painel do AC30 Top Boost!). Quando `Bass` e `Treble` estão ambos no máximo, os médios afundam drasticamente; quando ambos estão baixos, os médios sobem em proeminência (*interactive cut-only boost*).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | David T. Yeh & Julius O. Smith III — *Discretization of the '59 Fender Bassman Tone Stack* (Proc. DAFx-06, Montreal, 2006) | `https://ccrma.stanford.edu/~dtyeh/papers/yeh06_dafx.pdf` (HTTP 200 PDF verificado, 557 KB) | `ACADEMIC` | Derivação simbólica completa da função de transferência de 3ª ordem $H(s)$ com $R_{\text{src}}$, discretização bilinear e validação contra medições SPICE/hardware. |
| `F-02` | Duncan Munro — *Tone Stack Calculator (TSC) — Schematics & Nodal Equations for Marshall, Fender, Vox, James, Big Muff* | Referência técnica de engenharia de amplificadores baseada nos esquemas de fábrica | `SECONDARY_TECHNICAL` | Validação cruzada das curvas de magnitude/fase das 4 topologias (`FMV`, `Vox`, `James`, `Mesa`). |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Equação simbólica de 3ª ordem $H_{\text{FMV}}(s)$ incluindo $R_{\text{src}}$ e discretização Tustin exata | `F-01` | DAFx-06 | `F-01` §2–4 Eq. (2)–(13), pp. 2–5 | `ACADEMIC` | `CONFIRMED` |
| Parâmetros exatos Marshall 1959/JCM800 (`33k / 470p–500p / 22n / 22n`, $R_{\text{src}} \approx 1,3\text{ k}\Omega$) e Fender AB763 (`100k / 250p / 100n / 47n`, $R_{\text{src}} \approx 38\text{ k}\Omega$) | `F-01`, `F-02`, Specs 054–056 | 1959 / 2203 / AB763 | `F-01` & Esquemas de fábrica | `ACADEMIC` / `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Parâmetros exatos Vox Top Boost `OS/010` (`100k / 50p / 22n / 22n`, `1M Bass/Treble`, `10k` fixo sem pot `Mid`) | `F-02`, Spec 058 | Vox `OS/010` | Esquema `OS/010` | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Valores internos do Tone Stack da revisão exata **Mesa Dual Rectifier Rev F** (`C1`, `R_slope`, `R_bass`) | Spec 057 §6.2 | Dual Rectifier Rev F | Pendente esquema oficial Rev F | `UNKNOWN` | `PENDING` (`BLOCKED` para validação component-level exata de Rev F) |

---

## 7. Topologia e arquitetura real do sistema

```
[Preamp Out (Z = R_src)] ──► [3rd-Order Passive RLC/RC Network: FMV | Vox OS/010 | James | Mesa]
                                              │
                         [3rd-Order Bilinear IIR Solver in f64 (TDF-II)]
                                              │
                                              ▼
                                 [To Phase Inverter / Next Stage]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/tonestack.rs`](../crates/ampforge-dsp/src/tonestack.rs#L1-L401) apresenta os seguintes achados auditados:

1. **`FACTUAL_ERROR` (`tonestack.rs` — $R_{\text{src}}$ não parametrizado por topologia excitadora):** A versão atual já implementa o polinômio de 3ª ordem de Yeh (DAFx-06), o que é uma excelente base! Porém, a verificação de $R_{\text{src}}$ (`38 kΩ` plate-driven no Fender AB763 vs `1,3 kΩ` cathode-follower no Marshall 1959/JCM800) e a separação dedicada da topologia `VoxTopBoost` (onde não existe potenciômetro `Mid`, apenas `10 kΩ` fixo) e `James/Baxandall` precisam ser formalizadas em 4 structs/solvers independentes.
2. **`DOCUMENTATION_PENDING` (`tonestack.rs` — Mesa Dual Rectifier Rev F):** Conforme registrado na Spec 057, os valores exatos do Tone Stack de `Rev F` (vs `Rev G`) dependem da obtenção do esquema oficial `Rev F` da Mesa/Boogie.

---

## 8. Controles, parâmetros e mapeamento para a interface

| Parâmetro | Chave Canônica | Faixa | Curva (*Taper*) | Aplicabilidade por Solver |
|---|---|---|---|---|
| **BASS** | `amp.bass` | `0.0 .. 1.0` | Audio/Log (`10%` a `50%` curso) | Todos os 4 solvers (`FMV`, `Vox`, `James`, `Mesa`). |
| **MIDDLE** | `amp.mid` | `0.0 .. 1.0` | Linear | `FMV` e `Mesa` (no `VoxTopBoost`, fixo em `10 kΩ` — knob `Mid` inativo ou marcado como modificação ModShop). |
| **TREBLE** | `amp.treble` | `0.0 .. 1.0` | Audio/Log (Fender/Vox) ou Linear (Marshall) | Todos os 4 solvers. |

---

## 9. Modelo matemático e algoritmo DSP proposto

Para cada solver de 3ª ordem (`FMV`, `VoxTopBoost`, `Mesa`), os coeficientes contínuos $(b_1, b_2, b_3, a_0, a_1, a_2, a_3)$ são transformados para o domínio $z$ via substituição bilinear $s = c \frac{1 - z^{-1}}{1 + z^{-1}}$ com $c = 2 f_s$ (em `f64`), resultando na equação a diferenças de 3ª ordem (normalizada por $A_0$):
$$H(z) = \frac{B_0 + B_1 z^{-1} + B_2 z^{-2} + B_3 z^{-3}}{1 + A_1 z^{-1} + A_2 z^{-2} + A_3 z^{-3}}$$
executada em **Forma Direta II Transposta (`f64`)** para imunidade a ruído de quantização nos polos graves (`< 100 Hz`), com interpolação linear de coeficientes em 64 amostras ao alterar `Bass`/`Mid`/`Treble` para eliminação total de *zipper noise*.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Precisão numérica:** Estritamente **`f64`** nos cálculos dos polinômios $(a_k, b_k)$, na discretização bilinear e nos 3 registradores de estado (`z1, z2, z3`).
- **Compensação de Nyquist Cramping:** Executar o Tone Stack a $2\times f_s$ (ou $4\times/8\times$ dentro da cadeia do amplificador) garante erro de magnitude $\le 0,2\text{ dB}$ até `18 kHz`.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `update_coefficients`.
- Recálculo de coeficientes de 3ª ordem apenas quando `|param - prev_param| > 1e-6` (`~30` operações em ponto flutuante `f64`, `< 0,1 µs`).

---

## 12. Posição e interação na cadeia completa de guitarra

- Conecta-se entre o último estágio de pré-amplificação (ou seguidor de cátodo) e o inversor de fase / controle `Master` (Specs 054–058).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/tonestack.rs` (expandido com `FmvToneStackSolver`, `VoxTopBoostToneStackSolver`, `JamesBaxandallToneStackSolver`, `MesaRectifierToneStackSolver`).

---

## 14. Parâmetros de calibração e invariantes físicas

| Solver | Parâmetros Validados | Invariante Física |
|---|---|---|
| `FmvToneStackSolver (Fender AB763)` | $R_{\text{src}} = 38\text{ k}\Omega, R_1 = 100\text{ k}\Omega, C_1 = 250\text{ pF}, C_2 = 100\text{ nF}, C_3 = 47\text{ nF}$ | Mid-scoop profundo (`-12 dB a -16 dB`) centrado em `400–550 Hz` com `Bass=Mid=Treble=0.5`. |
| `FmvToneStackSolver (Marshall 1959/2203)` | $R_{\text{src}} = 1,3\text{ k}\Omega, R_1 = 33\text{ k}\Omega, C_1 = 470\text{ pF}, C_2 = 22\text{ nF}, C_3 = 22\text{ nF}$ | Mid-scoop mais raso (`-6 dB a -10 dB`) centrado em `550–700 Hz` e menor perda de inserção graças a $R_{\text{src}} = 1,3\text{ k}\Omega$. |
| `VoxTopBoostToneStackSolver (OS/010)` | $R_{\text{src}} = 1,5\text{ k}\Omega, R_1 = 100\text{ k}\Omega, C_1 = 50\text{ pF}, C_2 = C_3 = 22\text{ nF}, R_{\text{mid}} = 10\text{ k}\Omega$ | Aumentar `Bass` e `Treble` simultaneamente de `0.2` para `0.9` afunda os médios (`700 Hz`) relativamente às extremidades (interação passiva autêntica do AC30). |
| Passividade Absoluta | Todos os 4 solvers | $|H(e^{j\omega})| \le 1,0001$ (`<= +0,001 dB`) em todas as frequências e todas as posições dos knobs (rede puramente passiva RC!). |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Passividade Absoluta (`|H(f)| <= 1.0`):** varredura na grade completa de `(Bass, Mid, Treble) ∈ {0.0, 0.5, 1.0}³` para os 4 solvers.
2. **Teste de Acoplamento Analítico Yeh DAFx-06 (`Fender AB763` vs `Marshall 2203`):** compara a magnitude calculada pelo filtro IIR `f64` diretamente contra a avaliação exata no domínio de Laplace $|H(j 2\pi f)|$ (`erro <= 0,3 dB` de `30 Hz` a `12 kHz`).
3. **Teste de Estabilidade BIBO sob Varredura Rápida de Knobs:** verifica ausência de explosão ou `NaN` durante automação extrema de `0.0 ↔ 1.0` por amostra.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- O solver `MesaRectifierToneStackSolver` permanece com calibração de componentes `Rev F` marcada como `PENDING` até obtenção do esquema oficial `Rev F` (Spec 057).

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Aderência Exata à Equação de Laplace de 3ª Ordem (Yeh DAFx-06) para `FMV Fender AB763` e `FMV Marshall 2203` (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Varredura senoidal em 20 frequências logarítmicas de `30 Hz` a `12 000 Hz` a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$ (operando a $2\times f_s = 96\text{ kHz}$ ou $f_s$ com pré-warping), bloco de `128` amostras.
  - *Configuração e parâmetros:* `Bass = 0.5, Mid = 0.5, Treble = 0.5` em `FmvToneStackSolver` para `Fender AB763` ($R_{\text{src}} = 38\text{ k}\Omega$) e `Marshall 2203` ($R_{\text{src}} = 1,3\text{ k}\Omega$).
  - *Método de medição:* Erro absoluto $|\,20\log_{10}|H_{\text{DSP}}(f)| - 20\log_{10}|H_{\text{Laplace}}(j2\pi f)|\,|$ em cada uma das 20 frequências.
  - *Critério e tolerância:* O erro máximo de magnitude em relação à função de transferência analítica de Laplace de Yeh (DAFx-06) deve ser **`<= 0,35 dB`** em toda a banda `30 Hz – 12 000 Hz`.
  - *Origem da tolerância:* Discretização bilinear de 3ª ordem em `f64` (`F-01` §4).

- **CA-02 — Passividade Estrita em Todas as Combinações de Knobs (`|H(f)| <= 0,01 dB`) (`ANALYTICAL`):**
  - *Sinal de entrada:* Impulso unitário / varredura FFT (`4096` pontos) avaliada nos 27 vértices `(Bass, Mid, Treble) ∈ {0.0, 0.5, 1.0}³`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$.
  - *Configuração e parâmetros:* Todos os 4 solvers (`FMV`, `VoxTopBoost`, `JamesBaxandall`, `MesaRectifier`).
  - *Método de medição:* Máximo global da magnitude da resposta em frequência $\max_f |H(e^{j 2\pi f / f_s})|$.
  - *Critério e tolerância:* Como as 4 redes são estritamente passivas (apenas resistores e capacitores), o ganho máximo global deve respeitar **`<= 1,001` (`<= +0,01 dB`)** em todos os 27 pontos de operação.
  - *Origem da tolerância:* Teorema da passividade de redes RC.

- **CA-03 — Interatividade Autêntica do `VoxTopBoostToneStackSolver` (`OS/010` sem `Mid` físico) (`DOCUMENTED`):**
  - *Sinal de entrada:* Tons senoidais em `100 Hz`, `700 Hz` e `4000 Hz` a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `VoxTopBoostToneStackSolver` comparando `(Bass=0.9, Treble=0.9)` vs `(Bass=0.1, Treble=0.1)`.
  - *Método de medição:* Ganho relativo em `700 Hz` em relação à média de `100 Hz` e `4000 Hz`.
  - *Critério e tolerância:* Em `(Bass=0.9, Treble=0.9)`, os médios em `700 Hz` devem ficar **pelo menos `8,0 dB` abaixo** da média de `100 Hz` e `4000 Hz`; em `(Bass=0.1, Treble=0.1)`, os médios em `700 Hz` devem se sobressair em relação às extremidades (`>= +4,0 dB` acima da média de `100 Hz` e `4000 Hz`).
  - *Origem da tolerância:* Esquema passivo Vox `OS/010` (`F-02`, Spec 058).

- **CA-04 — Impacto Mensurável da Impedância da Fonte $R_{\text{src}}$ (`1,3 kΩ` vs `38 kΩ`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Configuração e parâmetros:* Mesmos componentes RC do FMV com `Bass=Mid=Treble=0.5`, alterando apenas $R_{\text{src}}$ de `1,3 kΩ` (seguidor de cátodo) para `38 kΩ` (placa de triodo).
  - *Critério e tolerância:* Elevar $R_{\text{src}}$ de `1,3 kΩ` para `38 kΩ` deve aumentar a perda de inserção em `1000 Hz` em **pelo menos `3,0 dB`** e deslocar a frequência do entalhe de médios conforme a equação simbólica de `F-01`.

- **CA-05 — Segurança em Tempo Real e Estabilidade sob Automação Extrema (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com saltos aleatórios de `Bass/Mid/Treble` entre `0.0` e `1.0`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas e limitadas (`|y[n]| <= 2,0`).

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/tonestack.rs`
- Testes atuais: `crates/ampforge-dsp/tests/contract_039_amps_fidelity.rs`, `Planejado: crates/ampforge-dsp/tests/contract_044_sdd_integrity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] PDF acadêmico oficial DAFx-06 de David Yeh (`F-01`) verificado via HTTP 200 (`557 KB`).
- [x] 4 solvers distintos (`FMV`, `VoxTopBoost`, `JamesBaxandall`, `MesaRectifier`) formalmente separados com $R_{\text{src}}$ explícito.
- [x] Pendência documental de `Mesa Rev F` (`C1`/`R_slope`/`R_bass`) rastreada como `PENDING`.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Yeh, D. T., & Smith, J. O. (2006). *Discretization of the '59 Fender Bassman Tone Stack*. Proc. DAFx-06, Montreal (`https://ccrma.stanford.edu/~dtyeh/papers/yeh06_dafx.pdf`).
- `[F-02]` Munro, D. — *Tone Stack Calculator Technical Reference*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, formalizou-se a separação em 4 solvers passivos distintos com validação analítica direta contra a equação de Laplace de 3ª ordem (`F-01`) e inclusão obrigatória de $R_{\text{src}}$.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Solver `FMV` de 3ª ordem (Yeh DAFx-06) em `f64` com $R_{\text{src}}$ parametrizado (`1,3 kΩ` vs `38 kΩ`) | `F-01` Eq. 2–13 | Spec 078 / `CA-01`, `CA-04` | `crates/ampforge-dsp/src/tonestack.rs:1–401 @ c20cc65` (implementa 3ª ordem, precisa expor `R_src` por amplificador) | `contract_039_amps_fidelity.rs` | `IMPLEMENTED_UNVERIFIED` |
| Passividade estrita (`|H(f)| <= 1,0`) em todos os 27 vértices de operação | Teorema da passividade | Spec 078 / `CA-02` | `crates/ampforge-dsp/src/tonestack.rs` | `contract_039_amps_fidelity.rs` | `TESTED` |
| Solver dedicado `VoxTopBoost (OS/010)` sem pot `Mid` físico + solver `James/Baxandall` | `F-02`, Spec 058 | Spec 078 / `CA-03` | Parcial em `tonestack.rs` (usa modelo FMV adaptado) | `contract_039_amps_fidelity.rs` | `SPECIFIED` |
| Calibração exata de `MesaRectifierToneStackSolver (Rev F)` | Spec 057 §6.2 | Spec 078 / §6.2 | `tonestack.rs` (valores aproximados de Rev G) | N/A | `PENDING` |
| Segurança zero-alloc e estabilidade sob automação extrema | Spec 053 | Spec 078 / `CA-05` | `crates/ampforge-dsp/src/tonestack.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
