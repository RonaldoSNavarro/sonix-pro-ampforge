---
status: rascunho
---

# Spec 088: Módulo de Tremolo por Modulação de Bias Valvulado (Power-Tube Grid Bias Modulation)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Criado na Rodada Corretiva por Desacoplamento da Spec 073)
> **Domínio:** `dsp-modulation` (`modulo-dsp`)
> **Identificador canônico:** `module.mod.bias_tremolo`
> **Variante / Revisão:** `Modulação Dinâmica de Tensão Negativa de Grade de Potência (Referência: Fender Princeton AA1164 / Deluxe 6G3 / Vibrolux 6G11)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/modulation.rs`, `crates/sonix-desktop/src/audio.rs`
> **Specs relacionadas:** [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [073](073-pedal-tremolo-optico-e-bias.md), [077](077-componente-estagios-valvulados-power-amp-sag-carga-reativa.md)

---

## 1. Objetivo

Especificar o contrato técnico, físico-matemático e de verificação do **Módulo de Tremolo por Modulação de Bias Valvulado (`module.mod.bias_tremolo`)**, desacoplando-o definitivamente do **Tremolo Foto-Óptico Shunt-LDR** (**[Spec 073](073-pedal-tremolo-optico-e-bias.md)**). Diferente do tremolo óptico (que é um atenuador resistivo linear de amplitude), o tremolo de bias injeta a tensão do oscilador LFO diretamente na **tensão negativa de polarização DC (*fixed bias*) das grades de controle das válvulas de potência (`6V6GT` / `6L6GC`)**, modulando **simultaneamente** a transcondutância ($g_m$, amplitude) **e** o ponto de corte / conteúdo harmônico (distorção dependente do nível do sinal)!

---

## 2. Escopo

1. **Modulação Simultânea de Transcondutância e Limiar de Saturação:**
   - Quando o LFO empurra a grade para uma tensão mais negativa (*cold bias*), a corrente de repouso $I_q$ cai e a válvula aproxima-se do corte (*Class B/C*): sinais **fracos** são fortemente atenuados, enquanto transientes **fortes** ainda rompem parcialmente o corte com calorosa compressão e distorção harmônica!
   - Quando o LFO puxa a grade para uma tensão menos negativa (*hot bias*), a válvula conduz com transcondutância máxima e saturação suave de pico.
2. **Efeito Psicoacústico de "Profundidade Dependente do Toque" (*Touch-Sensitive Tremolo*):** Em acordes atacados com força, o tremolo de bias soa mais suave e saturado nos picos de ataque, aprofundando-se organicamente na cauda de decaimento da nota — assinatura inconfundível do Fender Princeton Reverb (`AA1164`) e do Brownface Deluxe (`6G3`).
3. Bloqueio rigoroso de componente DC / *thump* de baixa frequência na saída (`HPF 25 Hz`) para evitar vazamento do LFO para o áudio.

---

## 3. Fora de escopo

- **Tremolo Foto-Óptico Shunt-LDR (Fender AB763 Twin Reverb — linear em relação ao áudio):** Coberto exclusivamente na **[Spec 073](073-pedal-tremolo-optico-e-bias.md)**.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `modulo-dsp` (`modulation` / `tube-bias-tremolo`) |
| **Fabricante de referência** | Referência topológica: Fender Electric Instrument Co. (Circuitos `6G3 Deluxe`, `6G11 Vibrolux`, `AA1164 Princeton Reverb`, 1961–1966) |
| **Modelo de referência** | Power-Tube Grid Bias Modulation Tremolo (`module.mod.bias_tremolo`) |
| **Ano / época / revisão** | 1961–1966 (Oscilador `12AX7` acoplado à malha de bias negativo das válvulas `2× 6V6GT` push-pull) |
| **Identificador no sistema** | `module.mod.bias_tremolo` (`ModModel::TremoloBias` em `crates/ampforge-dsp/src/fx/modulation.rs:267–274 @ c20cc65`) |
| **Papel na cadeia de sinal** | Bloco `MOD` (`mod_1`), simulando a modulação orgânica de estágio de saída push-pull. |

---

## 5. Princípio físico e funcional do equipamento real

Em um estágio push-pull `2× 6V6GT` com bias negativo $V_{\text{bias}}(t) = V_{b0} - \Delta V \cdot \sin(\omega_{\text{LFO}} t)$:
- A corrente de placa de cada válvula do par push-pull obedece aproximadamente à lei de Child-Langmuir $i_p(v_g) \propto \max(0,\; v_g - V_{\text{cutoff}})^{3/2}$:
  - Ramo positivo: $i_+(t) = f(V_{\text{bias}}(t) + x(t))$;
  - Ramo negativo: $i_-(t) = f(V_{\text{bias}}(t) - x(t))$;
  - Saída diferencial no transformador de saída: $y(t) \propto i_+(t) - i_-(t)$.
- Observe duas consequências físicas extraordinárias dessa simetria diferencial push-pull:
  1. **Cancelamento do *LFO Thump* em Modo Comum:** Como $V_{\text{bias}}(t)$ é aplicado em fase igual nas duas válvulas e o transformador subtrai $i_+ - i_-$, o próprio LFO DC é cancelado na saída ($x=0 \implies i_+ = i_- \implies y = 0$)!
  2. **Modulação de Transcondutância + Distorção de Cruzamento (*Dynamic Crossover Compression*):** Quando $V_{\text{bias}}(t)$ torna-se muito negativo, abre-se uma zona morta (*deadband* de classe B/C) em torno de $x = 0$, atenuando fortemente notas fracas e gerando harmônicos ímpares ($H_3$) no vale da modulação, enquanto no pico do ciclo o estágio conduz em plena Classe AB quente!

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Fender Electric Instrument Co. — *Princeton Reverb Model AA1164 & Deluxe 6G3 Schematic Diagrams (1961–1964)* | Esquemas oficiais de fábrica Fender (injeção do oscilador de vibrato diretamente no barramento `-V_bias` das grades de `2× 6V6GT`) | `UNKNOWN` | Topologia de modulação de bias em modo comum sobre par push-pull `6V6GT`. |
| `F-02` | Pakarinen & Yeh (2009) — *A Review of Digital Techniques for Modeling Vacuum-Tube Guitar Amplifiers* | Literatura acadêmica de referência (CMJ 2009) | `ACADEMIC` | Equação diferencial push-pull $i_+(V_b + x) - i_-(V_b - x)$ e lei de Child-Langmuir $3/2$. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Modulação em modo comum de $V_{\text{bias}}(t)$ em par push-pull cancela o LFO DC e varia simultaneamente $g_m$ e distorção harmônica | `F-01`, `F-02` | `AA1164` / `6G3` | Esquema de fábrica / §Power Stage | `UNKNOWN` / `ACADEMIC` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
                 ┌──► (+) ──► [Tube A: Child-Langmuir f(V_bias(t) + x)] ──┐
[Audio In x(t)] ─┤                                                        ├──► (-) Differential Transformer Out ──► [Out]
                 └──► (-) ──► [Tube B: Child-Langmuir f(V_bias(t) - x)] ──┘
                                            ▲
                       [Common-Mode LFO: V_bias(t) = V_b0 - depth · ΔV · sin(φ)]
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`MODEL_COUPLING` (corrigido nesta revisão):** Estava acoplado na Spec 073 com o Tremolo Óptico.

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica | Faixa Aceita | Unidade | Valor Padrão |
|---|---|---|---|---|
| **BYPASS** | `mod_1.enabled` | `false / true` | Booleano | `false` |
| **SPEED (RATE)** | `mod_1.rate_hz` | `0.5 .. 12.0` | `Hz` | `4.00` |
| **INTENSITY (DEPTH)** | `mod_1.depth` | `0.0 .. 1.0` | `%` | `0.70` |
| **MIX** | `mod_1.mix` | `0.0 .. 1.0` | `%` | `1.00` |

---

## 9. Modelo matemático e algoritmo DSP proposto

- Seja a função de condução suave da válvula de potência $f(u) = \text{softplus}(u, k)^{1,5} \cdot \text{sat}(u)$ com ponto quiescente $V_{b0} = 0,65$ e modulação em modo comum $V_b[n] = V_{b0} - \text{depth} \cdot 0,58 \cdot (0,5 + 0,5\sin(\phi[n]))$.
- Saída push-pull balanceada (executada a `2×` ou `4×` oversampling para evitar aliasing da não-linearidade):
  $$y_{\text{pp}}[n] = K_{\text{norm}} \cdot \left(f(V_b[n] + x[n]) - f(V_b[n] - x[n])\right)$$
  seguida de `HPF 25 Hz` de proteção contra desbalanceamento residual.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `4×` polifásico IIR (Spec 079) em torno do estágio push-pull não-linear $f(V_b \pm x)$ para garantir ausência de aliasing nos harmônicos de bias.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block`; WCET $\le 2,0\text{ }\mu\text{s}$ por bloco de `128` amostras.

---

## 12. Posição e interação na cadeia completa de guitarra

- Ideal após o pré-amplificador (ou no bloco `MOD`), oferecendo tremolo orgânico sensível à dinâmica da palhetada.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/modulation/bias_tremolo.rs` (`pub struct BiasTremoloProcessor` implementando `AudioProcessor` + `ModelDescriptor`, Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Regra Verificável |
|---|---|
| Zero vazamento DC / *LFO Thump* em silêncio | Para `x[n] == 0.0`, `|y[n]| <= 1e-6` mesmo com `depth = 1.0`. |
| Modulação conjunta de amplitude e conteúdo harmônico | No vale do bias, a THD é estritamente maior do que no Tremolo Óptico (Spec 073) e a profundidade percebida aumenta conforme o nível de entrada cai (*touch-sensitive*). |

---

## 15. Estratégia de testes, validação e benchmark

1. Teste de cancelamento em modo comum (`x = 0 => y = 0`, zero *LFO thump*).
2. Teste de diferenciação física frente ao Tremolo Óptico (Spec 073): presença mensurável de conteúdo harmônico gerado pela variação do ponto de operação $V_b(t)$ e maior profundidade de modulação em sinais fracos (`-24 dBFS`) do que em sinais fortes (`-3 dBFS`).

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Um desbalanceamento mínimo opcional (`1–2%`) entre os ramos A e B pode ser incluído para adicionar calor de 2º harmônico sutil sem violar a rejeição de *LFO thump* graças ao `HPF 25 Hz`.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Cancelamento de *LFO Thump* em Modo Comum (`x = 0`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Silêncio absoluto (`x[n] = 0.0`) por `1,0 s`, $f_s = 48\,000\text{ Hz}$.
  - *Configuração:* `rate_hz = 5.0`, `depth = 1.0`, `mix = 1.0`.
  - *Critério e tolerância:* A saída residual na frequência do LFO (`5 Hz`) e em toda a banda deve permanecer **`<= -90 dBFS`** (`|y[n]| <= 3e-5`).

- **CA-02 — Profundidade Dependente da Dinâmica de Entrada (*Touch-Sensitive Bias Compression*) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `440 Hz` testada em sinal forte (`-3 dBFS`) e sinal fraco (`-24 dBFS`), $f_s = 48\,000\text{ Hz}$.
  - *Configuração:* `rate_hz = 4.0`, `depth = 0.90`, `mix = 1.0`.
  - *Critério e tolerância:* A profundidade de modulação pico-a-vale (em dB) medida em `-24 dBFS` deve ser **pelo menos `+2,0 dB` maior** do que a profundidade medida em `-3 dBFS`, comprovando o comportamento autêntico de bias valvulado (enquanto no Tremolo Óptico linear da Spec 073 a profundidade em dB é idêntica em ambos os níveis!).

- **CA-03 — Presença de Harmônicos de Válvula Dependentes do Ciclo do LFO (Diferenciação frente à Spec 073) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `440 Hz` a `-6 dBFS`.
  - *Configuração:* LFO estacionário no ponto de bias mais frio (`depth = 0.9`).
  - *Critério e tolerância:* A THD gerada pela curvatura de condução da válvula deve ser **`>= 1,0%`** (`>= -40 dB`), diferenciando-o fisicamente do Tremolo Óptico (`THD <= 0,1%`, Spec 073 CA-03).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Critério e tolerância:* `0` alocações na heap em `10 000` blocos de `128` amostras e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/modulation.rs` (e futuro `crates/ampforge-dsp/src/fx/modulation/bias_tremolo.rs`)

---

## 19. Critérios de conclusão da Spec

- [x] Desacoplado integralmente da Spec 073 (Tremolo Óptico).
- [x] Topologia diferencial push-pull (`AA1164 / 6G3`) com *touch-sensitivity* e cancelamento de *LFO thump* especificada em `CA-01` a `CA-03`.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Fender Electric Instrument Co. — *Princeton Reverb AA1164 & Deluxe 6G3 Schematics*.
- `[F-02]` Pakarinen, J., & Yeh, D. T. (2009). *A Review of Digital Techniques for Modeling Vacuum-Tube Guitar Amplifiers*. CMJ, 33(2).

---

## 21. Histórico de decisões

- **2026-11-20:** Criada a Spec 088 na rodada corretiva para separar o Tremolo de Bias Valvulado do Tremolo Óptico (Spec 073).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Par push-pull diferencial com modulação de $V_{\text{bias}}(t)$ e zero *LFO thump* em silêncio | `F-01`, `F-02` | Spec 088 / `CA-01` | `crates/ampforge-dsp/src/fx/modulation.rs:267–274 @ c20cc65` (usa aproximação single-ended `tanh`) | `contract_032_time_mod_fx.rs` | `SPECIFIED` |
| Profundidade sensível à dinâmica de toque (`-24 dBFS` > `-3 dBFS`) e conteúdo harmônico $\ge 1,0\%$ | `F-01`, `F-02` | Spec 088 / `CA-02`, `CA-03` | Parcial em `modulation.rs:267–274` | Ausente | `SPECIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 088 / `CA-04` | `crates/ampforge-dsp/src/fx/modulation.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
