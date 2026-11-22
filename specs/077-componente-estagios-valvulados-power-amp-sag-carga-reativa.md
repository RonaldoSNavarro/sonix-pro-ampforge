---
status: rascunho
---

# Spec 077: Componente Compartilhado — Biblioteca de Estágios Valvulados, Power Amp, Fonte/Sag e Carga Reativa

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-shared` (`componente`)
> **Identificador canônico:** `component.tube.building_blocks`
> **Variante / Revisão:** `Biblioteca Parametrizada de 10 Sub-Modelos Físicos Valvulados (Triode, Pentode, CathodeFollower, PhaseInverterLtp, PushPullPowerStage, BiasTopology, PowerSupplyAndRectifier, Sag, OutputTransformer, ReactiveLoadAndNfb)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/amp.rs`, `crates/ampforge-dsp/src/modshop.rs`, `crates/ampforge-dsp/src/lib.rs`
> **Specs relacionadas:** [039](039-amplificadores-fidelidade-e-robustez.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [054](054-amp-marshall-1959-super-lead-plexi.md), [055](055-amp-marshall-jcm800-2203.md), [056](056-amp-fender-twin-reverb-ab763.md), [057](057-amp-mesa-boogie-dual-rectifier-rev-f.md), [058](058-amp-vox-ac30-top-boost.md), [079](079-componente-oversampling-polifasico-e-anti-aliasing.md)

---

## 1. Objetivo

Especificar o contrato técnico, físico, matemático e de verificação da **Biblioteca Compartilhada de Blocos Construtivos Valvulados (`component.tube.building_blocks`)** que abastece todos os amplificadores modulares (`Specs 054–058`) e o `ModShop` (`crates/ampforge-dsp/src/modshop.rs`), **separando rigorosamente os 10 sub-modelos físicos independentes** para evitar que parâmetros de um amplificador (ex.: `EL34` Classe AB Fixed Bias com NFB de um Marshall) contaminem outro amplificador com física incompatível (ex.: `4× EL84` Classe A Cathode Bias sem NFB de um Vox AC30, ou `4× 6L6GC` de alto headroom de um Fender Twin Reverb).

---

## 2. Escopo — Os 10 Sub-Modelos Físicos Separados e Parametrizáveis

Cada sub-modelo possui sua própria struct, estado físico e tabela de parâmetros validados (sem valores globais *hardcoded*):

1. **`TriodeStage` (`12AX7` / `ECC83`, `12AT7`, `12AU7`, `EF86` triode-strapped):** Equação física de placa e condução de grade (*grid current clipping*, *blocking distortion* com capacitor de acoplamento $C_c$, bypass de cátodo $R_k \parallel C_k$, resistor de placa $R_a$, $B+$ dinâmico e capacitância Miller $C_{ga}$).
2. **`PentodePreampStage` (`EF86`):** Curva de transcondutância e saturação de grade auxiliar (*screen grid*) para pré-amplificadores vintage (extensibilidade AC30/4).
3. **`CathodeFollowerStage` (`DC-Coupled 12AX7`):** Seguidor de cátodo acoplado diretamente ao ânodo do estágio anterior (Marshall 1959 `V2B`, JCM800 `V2B`, Vox AC30 Top Boost `V2B`), produzindo **compressão assimétrica suave em condução de grade-cátodo** ($v_g > v_k$) e **baixa impedância de saída ($R_{\text{src}} \approx 1,3..2,5\text{ k}\Omega$)** para acionar o Tone Stack passivo (Spec 078).
4. **`PhaseInverterLtpStage` (*Long-Tailed Pair* / *Cathodyne*):** Inversor de fase diferencial de cauda longa (`12AX7` em Marshall/Vox/Mesa, `12AT7` em Fender Twin AB763) com desbalanceamento de ganho intencional pelos resistores de placa assimétricos (`82 kΩ / 100 kΩ` ou `47 kΩ / 47 kΩ`) e injeção da malha de realimentação negativa (`NFB`) na grade de referência.
5. **`PushPullPowerStage` (`EL34`, `6L6GC`, `EL84`, `KT88`, `6V6GT`):** Par ou quarteto de saída *push-pull* modelando individualmente os ramos positivo e negativo (saturação de grade de controle, *crossover notch* dependente da polarização, limiar de corte assimétrico e variação de ganho sob queda de $B+$).
6. **`BiasTopology` (`FixedBias` vs `CathodeBias`):**
   - `FixedBias`: tensão negativa de grade fixa (Marshall 1959/JCM800, Fender Twin AB763, Mesa Dual Rectifier), classe AB fria/quente ajustável via ModShop (`bias_offset`);
   - `CathodeBias`: resistor de cátodo comum + capacitor de bypass ($50\text{ }\Omega \parallel 250\text{ }\mu\text{F}$ no Vox AC30, Spec 058), gerando **auto-deslocamento dinâmico de bias (*bias excursion*)** sob sinais fortes!
7. **`PowerSupplyAndRectifier` (`SolidStateSilicon` vs `Tube5U4G` / `TubeGZ34`):** Resistência interna equivalente da fonte ($R_{\text{PSU}}$) e banco de capacitores de filtragem ($C_{\text{filter}}$) determinando a impedância dinâmica de $B+$ e o *ripple* de `100/120 Hz` sob alta corrente de placa.
8. **`PowerSupplySag`:** Dinâmica temporal de queda e recuperação da tensão $B+(t)$ ($\tau_{\text{sag,att}} \in [15,\; 60\text{ ms}]$, $\tau_{\text{sag,rel}} \in [80,\; 350\text{ ms}]$), reduzindo proporcionalmente o headroom dos estágios de potência e do inversor de fase (`V_B+(t)`).
9. **`OutputTransformer` (`Dagnall C1998`, `Drake 1202-119`, `Schumacher`, `Woden`, `Mesa 562003`):** Saturação magnética de núcleo ferromagnético dependente do **fluxo magnético $\Phi(t) = \int v(t)\,dt$** (saturando principalmente em graves profundos de alta potência!), largura de banda primária (`HPF` de indutância de magnetização $L_p$ + `LPF` de indutância de dispersão $L_{\text{leak}}$).
10. **`ReactiveLoadAndNfb`:** Modelo de impedância complexa de alto-falante real $Z_{\text{spk}}(s)$ (pico de ressonância grave $f_r \in [75,\; 110\text{ Hz}]$ + indutância de bobina móvel $L_e$ subindo acima de `1 kHz`), interagindo diretamente com a impedância de saída das válvulas de potência e com a malha de **Realimentação Negativa (`NFB` + `Presence` + `Resonance/Depth`)** quando `nfb_enabled == true` (e rigorosamente `0,0` quando `nfb_enabled == false`, como no Vox AC30 e no canal Red Modern do Dual Rectifier!).

---

## 3. Fora de escopo

- Topologias específicas de um amplificador completo (Specs 054–058 consomem e instanciam estes 10 blocos com os parâmetros verificados de cada esquema elétrico).
- Redes passivas de Tone Stack (cobertas na **[Spec 078](078-componente-redes-passivas-tone-stack.md)**).
- Convolução de Gabinete IR e Microfones (coberta na **[Spec 080](080-componente-gabinete-convolucao-ir-e-microfonacao.md)**).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `componente` (`dsp-shared` / `tube-physics-library`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (fundamentado nos modelos físicos de Koren, Cohen-Helie, Yeh e Pakarinen) |
| **Modelo de referência** | Modular Vacuum Tube & Power Amp Building Blocks (`component.tube.building_blocks`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 077) |
| **Identificador no sistema** | `component.tube.building_blocks` (`crates/ampforge-dsp/src/amp.rs` & `modshop.rs`) |
| **Papel na cadeia de sinal** | Núcleo de simulação física compartilhado pelas Specs `054` (Plexi), `055` (JCM800), `056` (Twin Reverb), `057` (Dual Rectifier) e `058` (Vox AC30). |

---

## 5. Princípio físico e funcional do circuito real

1. **Condução de Grade (*Grid Current*) e *Blocking Distortion* em Triodos `12AX7`:**
   - Quando a tensão na grade de controle torna-se positiva em relação ao cátodo ($v_{gk} > -0,3\text{ V}$), a grade passa a conduzir corrente através da resistência de parada de grade ($R_g$) carregando o capacitor de acoplamento anterior ($C_c$). O resultado é uma **compressão assimétrica no semiciclo positivo** seguida de um deslocamento negativo temporário do ponto quiescente (*grid bias shift*) que recupera segundo $\tau_{\text{grid}} = R_{\text{grid\_leak}} \cdot C_c$.
2. **Saturação de Fluxo do Transformador de Saída ($\Phi = \int v\,dt$):**
   - Ao contrário de um clipper estático $y = \tanh(x)$ (que satura igualmente em `100 Hz` e `4000 Hz`), o núcleo de aço-silício de um transformador de saída satura quando a **densidade de fluxo magnético $B \propto \frac{V}{f}$** excede $B_{\text{sat}} \approx 1,5\text{ T}$. Portanto, um sinal grave de `80 Hz` satura o núcleo do transformador com $10\times$ menos tensão do que um sinal de `800 Hz`, gerando harmônicos ímpares quentes especificamente nos graves pesados (*transformer bloom*)!
3. **Interação Carga Reativa ($Z_{\text{spk}}(s)$) $\leftrightarrow$ Realimentação Negativa (`NFB`):**
   - A curva de impedância de um falante `Celestion G12M` ou `Vintage 30` tem um pico de `~45–75 Ω` na ressonância mecânica (`~75–95 Hz`) e sobe indutivamente acima de `1,5 kHz` (`~25–40 Ω` a `5 kHz`).
   - Em um amplificador valvulado **sem NFB** (Vox AC30, Dual Rectifier Red Modern), a alta impedância de saída dos pentodos/tetrodos transforma diretamente a curva de $Z_{\text{spk}}(s)$ em um reforço pronunciado de graves (`~85 Hz`) e agudos (`> 2 kHz`).
   - Em um amplificador **com NFB** (Marshall 1959/JCM800, Fender Twin AB763), o fator de realimentação negativa amortiza a impedância de saída, reduzindo esse pico — exceto onde o controle **`Presence`** shunta os agudos da malha de NFB para o terra (devolvendo ganho aberto nos agudos) ou onde **`Resonance/Depth`** shunta os graves da malha de NFB!

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Norman Koren — *Improved Vacuum-Tube Models for SPICE Simulations* (JAES / Glass Audio, 1996) | Literatura técnica de referência para modelagem fenomenológica de triodos e pentodos (`12AX7`, `EL34`, `6L6GC`, `EL84`) | `ACADEMIC` | Equações físicas $I_p(V_g, V_p)$ com parâmetros calibrados ($\mu, E_x, K_{g1}, K_p, K_{vb}$) por família de válvula. |
| `F-02` | Jyri Pakarinen & David T. Yeh — *A Review of Digital Techniques for Modeling Vacuum-Tube Guitar Amplifiers* (Computer Music Journal, Vol. 33, No. 2, 2009) | Literatura acadêmica de referência (CCRMA / Helsinki Univ. of Tech.) | `ACADEMIC` | Revisão autoritativa de estágios triodo com condução de grade, seguidor de cátodo acoplado DC, transformador de saída dependente de fluxo, sag de fonte e carga reativa. |
| `F-03` | Ivan Cohen & Thomas Hélie — *Real-Time Simulation of a Guitar Power Amplifier* (DAFx-10, Graz, 2010) | Literatura acadêmica de referência | `ACADEMIC` | Modelagem de estágio push-pull, inversor LTP, transformador de saída e malha de realimentação negativa com carga reativa. |
| `F-04` | Mullard / RCA / Philips — *ECC83 (12AX7), EL34, 6L6GC, EL84 Tube Datasheets* | Datasheets oficiais de válvulas (curvas de placa $I_a$ vs $V_a$, fator $\mu$, $g_m$, $r_p$) | `PRIMARY_COMPONENT` | Limites nominais de operação e curvas características de cada válvula. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Condução de grade + deslocamento de bias dinâmico em estágios triodo e seguidor de cátodo acoplado DC | `F-02`, `F-04` | CMJ 2009 / ECC83 | `F-02` §3–4 | `ACADEMIC` / `PRIMARY_COMPONENT` | `CONFIRMED` |
| Saturação de transformador de saída proporcional ao fluxo magnético $\Phi = \int v\,dt$ (dependente de $1/f$) | `F-02`, `F-03` | CMJ 2009 / DAFx-10 | `F-02` §5.2; `F-03` §3 | `ACADEMIC` | `CONFIRMED` |
| Interação entre pico ressonante de carga reativa $Z_{\text{spk}}(s)$ e fator de amortecimento da malha NFB / Presence | `F-02`, `F-03` | CMJ 2009 / DAFx-10 | `F-03` §4 | `ACADEMIC` | `CONFIRMED` |
| Separação física entre `FixedBias` (Marshall/Fender/Mesa) e `CathodeBias` (Vox AC30 com auto-deslocamento de bias) | `F-02`, `F-04` | Datasheets & CMJ 2009 | Specs 054–058 | `ACADEMIC` / `PRIMARY_COMPONENT` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Preamp Stages: TriodeStage × N (12AX7)] ──► [Optional CathodeFollowerStage] ──► [Tone Stack (Spec 078)]
                                                                                          │
                                                                                          ▼
[NFB Tap + Presence/Depth] ──(if nfb_enabled)──► [PhaseInverterLtpStage] ──► [PushPullPowerStage (EL34/6L6/EL84)]
                                                        ▲                            ▲         │
                                                        └──── [PowerSupplyAndRectifier + Sag B+(t)] + [BiasTopology]
                                                                                               │
                                                                                               ▼
                                                                           [OutputTransformer (Flux Saturation)]
                                                                                               │
                                                                                               ▼
                                                                           [ReactiveLoadAndNfb Z_spk(s)] ──► [Cab IR (Spec 080)]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/amp.rs`](../crates/ampforge-dsp/src/amp.rs#L1-L1738) e [`crates/ampforge-dsp/src/modshop.rs`](../crates/ampforge-dsp/src/modshop.rs#L1-L506) apresenta os seguintes achados auditados:

1. **`MODEL_COUPLING` (`amp.rs:1–1738`):** Os 5 amplificadores atualmente compartilham uma grande struct monolítica `AmpProcessor` com múltiplos `match self.model` internos, em vez de compor instâncias parametrizadas explícitas dos 10 sub-modelos definidos na Seção 2.
2. **`FACTUAL_ERROR` (`amp.rs` — Transformador de Saída Estático):** A modelagem atual de saturação do transformador de saída não integra o fluxo magnético $\Phi(t) = \int v(t)\,dt$ dependente da frequência ($1/f$), saturando de maneira quase uniforme entre graves e agudos.
3. **`FACTUAL_ERROR` (`amp.rs` — Seguidor de Cátodo e `BiasTopology`):** A distinção física entre `CathodeBias` (Vox AC30) e `FixedBias` (Marshall/Fender/Mesa) e a compressão específica do seguidor de cátodo acoplado DC ainda não estão isoladas em structs independentes testáveis por contrato.

---

## 8. Controles, parâmetros e mapeamento para a interface

Os 10 sub-modelos são configurados por structs de parâmetros estáticos (específicos de cada modelo de amplificador nas Specs `054–058`) e modulados em tempo real pelos controles de usuário `amp.power_tubes`, `amp.rectifier`, `amp.sag`, `amp.presence`, `amp.depth`, `amp.master`, `amp.standby` e `modshop.*` (`modshop.rs`).

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **`TriodeStage`:** Aproximação analítica calibrada por família de válvula (`12AX7`, `12AT7`) operando dentro do laço de oversampling `8×` (Spec 079), com acoplamento capacitivo de entrada (modelando *grid current clamping* e recuperação $\tau_{\text{grid}}$), filtro de bypass de cátodo ($R_k \parallel C_k$) e escalonamento dinâmico por $B+(t) / B+_{\text{nom}}$.
2. **`OutputTransformer` Dependente de Fluxo:**
   - Integração com fuga (*leaky integrator*) para estimar o fluxo magnético $\Phi[n] = \alpha_\Phi \Phi[n-1] + (1 - \alpha_\Phi) \frac{v[n]}{\omega_0}$, aplicando permeabilidade saturante $\mu(\Phi) = \frac{1}{\sqrt{1 + (\Phi / \Phi_{\text{sat}})^2}}$ antes da derivada reconstitutiva. Isso garante que graves profundos (`80 Hz`) saturem o núcleo em amplitude muito menor que médios-agudos (`2 kHz`).
3. **`ReactiveLoadAndNfb`:**
   - Biquad de ressonância grave ($f_r \in [75,\; 110\text{ Hz}]$, pico $+5\text{ a }+9\text{ dB}$) + shelf indutivo de bobina móvel (`+4 a +7 dB` acima de `2 kHz`), cuja amplitude efetiva é dividida pelo fator $(1 + \beta_{\text{NFB}}(\omega) \cdot A_{\text{OL}})$ da malha de realimentação negativa. Quando `nfb_enabled == false`, a malha é aberta ($\beta_{\text{NFB}} \equiv 0$) e a interação da carga reativa atua integralmente.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** Todo estágio não-linear (`TriodeStage`, `CathodeFollowerStage`, `PhaseInverterLtpStage`, `PushPullPowerStage`, `OutputTransformer`) executa dentro do domínio de **`8×` oversampling** (Spec 079) em precisão **`f64`** para os estados de integradores e filtros de acoplamento.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Todas as 10 structs são `Copy` ou alocadas estaticamente inline (zero `Vec`, zero `Box`, zero `Mutex`).
- WCET total de uma cadeia completa de amplificador (`4× Triode + CathodeFollower + ToneStack + LTP + PushPull + Transformer + ReactiveLoad` a `8×` OS) $\le 65\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Provê a camada física comum para `Specs 054–058` e permite ao `ModShop` (`Spec 039`) trocar válvulas (`EL34` / `6L6GC` / `EL84` / `KT88`), retificador (`SS` / `Tube 5U4G`) e nível de `NFB` sem quebrar a modularidade de cada amplificador.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/tube/` contendo submódulos dedicados:
  - `triode.rs`, `pentode.rs`, `cathode_follower.rs`, `phase_inverter.rs`, `power_stage.rs`, `bias.rs`, `power_supply.rs`, `output_transformer.rs`, `reactive_load.rs`.

---

## 14. Parâmetros de calibração e invariantes físicas

| Sub-Modelo | Invariante Física Verificável |
|---|---|
| `TriodeStage` | Distorção assimétrica (2º harmônico $H_2$ expressivo em médio drive) e queda suave de ganho sob queda de $B+$. |
| `CathodeFollowerStage` | Ganho linear $< 1,0$ ($\approx 0,92..0,98$) e compressão assimétrica suave apenas no semiciclo de condução de grade-cátodo. |
| `PowerSupplySag` | Sinais sustentados de alta amplitude derrubam $B+(t)$ segundo $\tau_{\text{att}}$ e recuperam segundo $\tau_{\text{rel}} > \tau_{\text{att}}$ (`Tube` > `SolidState`). |
| `OutputTransformer` | Saturação magnética em `80 Hz` ocorre em nível de entrada estritamente menor do que em `2000 Hz` ($\text{THD}(80\text{ Hz}) > \text{THD}(2000\text{ Hz})$ na mesma amplitude). |
| `ReactiveLoadAndNfb` | Com `nfb_enabled = false`, o pico de ressonância grave ($f_r \approx 90\text{ Hz}$) é estritamente maior do que com `nfb_enabled = true`. |

---

## 15. Estratégia de testes, validação e benchmark

1. Testes unitários isolados para cada um dos 10 sub-modelos físicos validando suas invariantes físicas antes mesmo de compor um amplificador completo.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- A malha de realimentação negativa global (`NFB`) discretizada com 1 amostra de atraso a $8\times f_s$ (`384 kHz`, atraso de `2,6 µs`) mantém margem de fase superior a `85°` até `20 kHz`, evitando instabilidade de alta frequência desde que o ganho de laço aberto respeite o critério de Nyquist.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Saturação Dependente de Fluxo Magnético no `OutputTransformer` (`80 Hz` vs `2000 Hz`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Duas senoides testadas separadamente na mesma amplitude de pico (`-3 dBFS` na entrada do transformador): `80 Hz` e `2000 Hz`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$ (`8×` OS = `384 kHz`), bloco de `128` amostras.
  - *Configuração e parâmetros:* `OutputTransformer` calibrado para núcleo `100 W` (`Dagnall C1998`).
  - *Método de medição:* Medição de THD por FFT de `16 384` amostras em `80 Hz` vs `2000 Hz`.
  - *Critério e tolerância:* A THD gerada em `80 Hz` deve ser **pelo menos `2,5×` maior** que a THD gerada em `2000 Hz` sob a mesma amplitude de excitação, comprovando saturação física por fluxo $\Phi = \int v\,dt$ (e não clipper estático independente de frequência).
  - *Origem da tolerância:* Lei de Faraday $V = N \frac{d\Phi}{dt} \implies \Phi_{\text{pk}} \propto \frac{V_{\text{pk}}}{f}$ (`F-02` §5.2, `F-03`).

- **CA-02 — Dinâmica Diferencial de `PowerSupplySag` (`Tube 5U4G` vs `Solid-State Silicon`) (`DOCUMENTED` / `REGRESSION`):**
  - *Sinal de entrada:* Burst de senoide `200 Hz` a `0 dBFS` por `300 ms`, seguido de `500 ms` em `-40 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Comparação de `PowerSupplyAndRectifier` configurado como `Tube5U4G` ($R_{\text{PSU}} \approx 180\text{ }\Omega$) vs `SolidStateSilicon` ($R_{\text{PSU}} \approx 25\text{ }\Omega$).
  - *Método de medição:* Queda percentual máxima de tensão de barramento $\Delta B+ / B+_{\text{nom}}$ durante o burst e tempo de recuperação até `90%` de $B+_{\text{nom}}$.
  - *Critério e tolerância:* Em `Tube5U4G`, a queda de barramento $\Delta B+$ deve ser **pelo menos `2,5×` maior** (ex.: `12%..22%` de sag) do que em `SolidStateSilicon` (`<= 5%`), recuperando suavemente sem overshoot.
  - *Origem da tolerância:* Impedância de placa de válvula retificadora `5U4G`/`GZ34` vs diodos silício `1N4007` (`F-02`, `F-04`).

- **CA-03 — Interação `ReactiveLoadAndNfb` (`NFB ON` vs `NFB OFF` + Controle `Presence`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Varredura senoidal de pequeno sinal (`-30 dBFS`) em `90 Hz` (ressonância do falante), `1000 Hz` e `5000 Hz`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Comparação de `ReactiveLoadAndNfb` com `nfb_enabled = false` vs `nfb_enabled = true` (`presence = 0.0` e `presence = 1.0`).
  - *Método de medição:* Ganho em `90 Hz` e `5000 Hz` relativo a `1000 Hz`.
  - *Critério e tolerância:*
    1. Desligar a malha NFB (`nfb_enabled = false`) deve elevar o pico de ressonância grave em `90 Hz` em **pelo menos `+2,5 dB`** em relação a `nfb_enabled = true`.
    2. Com `nfb_enabled = true`, variar `presence` de `0.0` para `1.0` deve elevar o ganho em `5000 Hz` em **pelo menos `+4,0 dB`**, enquanto com `nfb_enabled = false` o controle `presence` do laço de NFB tem efeito `0,0 dB` (inoperante, como no Dual Rectifier Red Modern).
  - *Origem da tolerância:* Equação de realimentação negativa de potência com carga reativa (`F-03` §4).

- **CA-04 — Distinção Física `CathodeFollowerStage` (Ganho $< 1$ + Compressão Assimétrica) e `CathodeBias` vs `FixedBias` (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide `440 Hz` em pequeno sinal (`-40 dBFS`) e em grande sinal (`0 dBFS`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Método de medição:* Ganho em pequeno sinal do `CathodeFollowerStage` e deslocamento médio DC do ponto de bias em `CathodeBias` sob grande sinal.
  - *Critério e tolerância:* O `CathodeFollowerStage` deve apresentar ganho linear estritamente **`< 1,0`** (`0,85..0,99`) em pequeno sinal e produzir 2º harmônico mensurável ($H_2 > -28\text{ dB}$ relativo a $H_1$) sob grande sinal; o bloco `CathodeBias` deve produzir compressão por auto-deslocamento de bias maior que `FixedBias`.
  - *Origem da tolerância:* Física de seguidores de cátodo acoplados DC e polarização por cátodo (`F-02` §3–4).

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `sag`, `presence`, `depth`, `master`, `bias`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/amp.rs`, `crates/ampforge-dsp/src/modshop.rs` e futuro diretório `crates/ampforge-dsp/src/tube/`
- Testes atuais: `crates/ampforge-dsp/tests/contract_039_amps_fidelity.rs`, `crates/ampforge-dsp/tests/contract_044_sdd_integrity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Biblioteca decomposta rigorosamente nos 10 sub-modelos físicos independentes (`Triode` até `ReactiveLoadAndNfb`).
- [x] Fontes acadêmicas (`Koren`, `Pakarinen/Yeh`, `Cohen/Hélie`) e datasheets de válvulas (`PRIMARY_COMPONENT`) classificados.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Koren, N. (1996). *Improved Vacuum-Tube Models for SPICE Simulations*.
- `[F-02]` Pakarinen, J., & Yeh, D. T. (2009). *A Review of Digital Techniques for Modeling Vacuum-Tube Guitar Amplifiers*. Computer Music Journal, 33(2), 85–100.
- `[F-03]` Cohen, I., & Hélie, T. (2010). *Real-Time Simulation of a Guitar Power Amplifier*. Proc. DAFx-10.
- `[F-04]` Mullard / RCA — *ECC83, EL34, 6L6GC, EL84 Tube Datasheets*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, decompôs-se o bloco monolítico valvulado em 10 sub-modelos físicos parametrizáveis por amplificador, exigindo saturação de transformador dependente de fluxo ($\Phi = \int v\,dt$) e desacoplamento explícito entre `FixedBias` e `CathodeBias` e entre `NFB ON` e `NFB OFF`.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| 10 sub-modelos isolados e parametrizáveis (`Triode` a `ReactiveLoadAndNfb`) | `F-01`–`F-04` | Spec 077 / §2 | `crates/ampforge-dsp/src/amp.rs` (atualmente monolítico em `AmpProcessor`) | `contract_039_amps_fidelity.rs` | `SPECIFIED` |
| Saturação de `OutputTransformer` dependente de fluxo magnético ($\Phi = \int v\,dt$, `80 Hz` vs `2000 Hz`) | `F-02` §5.2, `F-03` | Spec 077 / `CA-01` | `crates/ampforge-dsp/src/amp.rs` (atualmente usa clipper estático independente de frequência) | Ausente | `SPECIFIED` |
| `PowerSupplySag` diferencial (`Tube 5U4G` vs `SolidState`) | `F-02`, `F-04` | Spec 077 / `CA-02` | `crates/ampforge-dsp/src/amp.rs` & `modshop.rs` | `contract_039_amps_fidelity.rs` | `TESTED` |
| `ReactiveLoadAndNfb` acoplado à malha NFB e `Presence` (`NFB ON` vs `OFF`) | `F-03` §4 | Spec 077 / `CA-03` | `crates/ampforge-dsp/src/amp.rs` | `contract_039_amps_fidelity.rs` | `IMPLEMENTED_UNVERIFIED` |
| `CathodeFollowerStage` (ganho `< 1` + assimetria) e `CathodeBias` vs `FixedBias` | `F-02` §3–4 | Spec 077 / `CA-04` | Parcial em `crates/ampforge-dsp/src/amp.rs` | Ausente | `SPECIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 077 / `CA-05` | `crates/ampforge-dsp/src/amp.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
