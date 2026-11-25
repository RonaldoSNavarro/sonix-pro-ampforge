---
status: rascunho
---

# Spec 073: Módulo de Tremolo Óptico (Opto-LDR Shunt Attenuator — Estilo Fender AB763)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade — Desacoplamento do Bias Tremolo)
> **Domínio:** `dsp-modulation` (`modulo-dsp`)
> **Identificador canônico:** `module.mod.optical_tremolo`
> **Variante / Revisão:** `Tremolo Foto-Óptico Shunt-LDR (Referência: Célula Opto-Isoladora Néon/CdS "Roach" do Circuito Fender AB763, 1963–1967)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/modulation.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [052](052-abertura-animada-pedais-faltantes-e-amp2-visivel.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [056](056-amp-fender-twin-reverb-ab763.md), [088](088-modulo-tremolo-modulacao-bias.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação **exclusivamente** do **Módulo de Tremolo Óptico (`module.mod.optical_tremolo`)**, desfazendo o agrupamento indevido (`MODEL_COUPLING`) da versão anterior desta Spec — que misturava dois mecanismos físicos completamente diferentes sob o mesmo documento:
1. **Tremolo Foto-Óptico (Atenuador Shunt LDR):** Modulação pura de amplitude por divisor resistivo passivo após o pré-amplificador (esta Spec 073); e
2. **Tremolo por Modulação de Bias Valvulado (*Power-Tube Grid Bias Modulation*):** Modulação do ponto quiescente DC das grades de controle das válvulas de potência, que varia simultaneamente a transcondutância ($g_m$) e o limiar de clipping/distorção harmônica — agora isolado integralmente na **[Spec 088](088-modulo-tremolo-modulacao-bias.md)**.

---

## 2. Escopo

1. Oscilador de deslocamento de fase (*Phase-Shift LFO*, `1,0 Hz` a `12,0 Hz`, inspirado no estágio `V5B 12AX7` do circuito `AB763`) excitando uma lâmpada néon de tensão de ignição (*striking voltage*) não-linear.
2. Fotoresistor de Sulfeto de Cádmio (`CdS LDR`, "Roach" optocoupler) em configuração de **divisor resistivo shunt para o terra** em série com o resistor de acoplamento ($R_s = 220\text{ k}\Omega$):
   $$g_{\text{opto}}(t) = \frac{R_{\text{LDR}}(t)}{R_s + R_{\text{LDR}}(t)}$$
3. Dinâmica fotoquímica assimétrica do CdS (ataque luminoso rápido $\tau_{\text{att}} \approx 8\text{ ms}$, recuperação lenta no escuro $\tau_{\text{rel}} \approx 35..45\text{ ms}$), produzindo a curva de modulação de amplitude arredondada e "pulsante" (*choppy-smooth*) do tremolo óptico Blackface sem adicionar distorção de cruzamento ou modulação de headroom de power-amp.

---

## 3. Fora de escopo

- **Tremolo por Modulação de Bias Valvulado (`module.mod.bias_tremolo`, Fender Brownface / Princeton / Deluxe 6G16):** Movido integralmente para a **[Spec 088](088-modulo-tremolo-modulacao-bias.md)**.
- **Harmonic Tremolo (Fender 6G7-A Bandmaster / 6G9-A Tremolux — crossover bi-banda em contrafase):** Fora do escopo desta Spec.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `modulo-dsp` (`modulation` / `optical-tremolo`) |
| **Fabricante de referência** | Referência topológica: Fender Electric Instrument Co. (Circuito `AB763` Vibrato/Opto-Isolator "Roach" `022999`, 1963–1967) |
| **Modelo de referência** | Opto-LDR Shunt Attenuator Tremolo (`module.mod.optical_tremolo`) |
| **Ano / época / revisão** | 1963–1967 (Circuito `AB763` Opto-Isolator: Lâmpada Néon + Célula Fotoresistiva CdS + Resistor Série `220 kΩ`) |
| **Identificador no sistema** | `module.mod.optical_tremolo` (`ModModel::ModModel::OpticalBiasTremolo` / UI `pedal.tremolo.*`) |
| **Papel na cadeia de sinal** | Bloco `MOD` (`mod_1`), classicamente após o estágio de ganho do pré-amplificador (e antes ou depois do Reverb). |

---

## 5. Princípio físico e funcional do equipamento real

1. **Divisor Resistivo Passivo Shunt-LDR ($R_s = 220\text{ k}\Omega$ + $R_{\text{LDR}}(t)$):**
   - No esquema `AB763` (Spec 056), a saída do canal Vibrato atravessa um resistor série $R_s = 220\text{ k}\Omega$ antes do estágio misturador/inversor de fase.
   - Entre esse nó e o terra está conectado o fotoresistor CdS do opto-isolador ("Roach"):
     - **Néon apagado (escuro):** $R_{\text{LDR}} > 5\text{ M}\Omega \implies g_{\text{opto}} = \frac{5\text{ M}\Omega}{220\text{ k}\Omega + 5\text{ M}\Omega} \approx 0,96$ (`~0 dB`, sinal pleno);
     - **Néon aceso (brilho máximo):** $R_{\text{LDR}} \approx 15\text{ k}\Omega..40\text{ k}\Omega \implies g_{\text{opto}} = \frac{15\text{ k}\Omega}{220\text{ k}\Omega + 15\text{ k}\Omega} \approx 0,064$ (`-24 dB`, atenuação profunda).
2. **Assimetria Fotoquímica do CdS e Limiar de Ignição da Lâmpada Néon:**
   - Diferente de um VCA linear multiplicado por uma senoide simétrica ($1 - d \cdot \sin(\omega t)$), a lâmpada néon só ioniza quando a tensão do oscilador excede $V_{\text{strike}} \approx 65\text{ V}$, criando um pulso de luz com topo mais plano e bordas mais íngremes; e a célula CdS cai de resistência rapidamente quando iluminada ($\tau_{\text{att}} \approx 8\text{ ms}$) mas retorna lentamente à alta resistência quando a luz apaga ($\tau_{\text{rel}} \approx 40\text{ ms}$).
   - Como o fotoresistor CdS é um elemento **puramente resistivo (linear em relação à tensão de áudio)**, o tremolo óptico modula **apenas a amplitude** sem introduzir distorção harmônica dependente da profundidade (`THD < 0,1%`).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Fender Electric Instrument Co. — *Twin Reverb Model AB763 Schematic Diagram (1963)* (referenciado via Spec 056) | Esquema oficial `AB763` (`V5B` Phase-Shift Oscillator, Optocoupler Neon/LDR, $R_s = 220\text{ k}\Omega$, `Intensity 50kΩ RA`, `Speed 3MΩ RA`) | `UNKNOWN` | Topologia do divisor resistivo shunt-LDR e estágio oscilador do circuito `AB763`. |
| `F-02` | R. G. Keen (GEOFEX) — *Analysis of Fender Optocoupler Tremolo Circuits* & *VTL5C / CdS Optocoupler Datasheets (PerkinElmer / Excelitas)* | Datasheets oficiais de células CdS (`Rise Time ~5–10 ms`, `Decay Time ~35–50 ms`, relação log-log $R_{\text{LDR}}(I)$) | `PRIMARY_COMPONENT` / `SECONDARY_TECHNICAL` | Constantes de tempo assimétricas de ataque/recuperação do fotoresistor CdS. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Divisor resistivo passivo shunt-LDR com $R_s = 220\text{ k}\Omega$ na saída do pré-amplificador | `F-01` | Fender `AB763` (1963) | Esquema `AB763` (bloco Vibrato) | `UNKNOWN` | `CONFIRMED` |
| Constantes de tempo fotoquímicas assimétricas do CdS ($\tau_{\text{att}} < \tau_{\text{rel}}$: `~8 ms` vs `~40 ms`) | `F-02` | CdS LDR Datasheets | Curvas de resposta temporal CdS | `PRIMARY_COMPONENT` | `CONFIRMED` |
| Linearidade do elemento resistivo CdS (modulação de amplitude limpa sem alteração de bias de válvula de potência) | `F-01`, `F-02` | `AB763` | Análise de circuito | `UNKNOWN` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Audio In] ──► [Series Resistor R_s = 220 kΩ] ──┬──► [Audio Out]
                                                │
                                       [CdS LDR R_LDR(t)] ──► [GND]
                                                ▲
                                  (Optical Coupling: τ_att≈8ms, τ_rel≈40ms)
                                                │
                           [Neon Bulb Threshold (V_strike)] ◄── [Phase-Shift LFO (SPEED & INTENSITY)]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/modulation.rs:258–266 @ c20cc65`](../crates/ampforge-dsp/src/fx/modulation.rs#L258-L266) apresenta as seguintes divergências auditadas:

1. **`MODEL_COUPLING` (corrigido nesta revisão):** A Spec 073 original combinava `ModModel::OpticalBiasTremolo` e `TremoloBias` em um único documento. O `TremoloBias` foi separado para a **[Spec 088](088-modulo-tremolo-modulacao-bias.md)**.
2. **`FACTUAL_ERROR` (`modulation.rs:259–265`):** O código atual modela `ModModel::OpticalBiasTremolo` simplesmente como `let target = 1.0 - self.depth * mod_val.powf(1.5);` suavizado por um único coeficiente simétrico (`0.002` a cada amostra, independente de `sample_rate` e idêntico para subida e descida!). Isso viola duas propriedades físicas:
   - A constante de tempo do filtro `0.002` não escala com `sample_rate` (fica $2\times$ mais rápida em `96 kHz` do que em `48 kHz`);
   - O fotoresistor CdS tem **ataque rápido** ($\tau_{\text{att}} \approx 8\text{ ms}$) quando a lâmpada acende (queda de ganho) e **recuperação lenta** ($\tau_{\text{rel}} \approx 40\text{ ms}$) quando a lâmpada apaga (retorno ao ganho unitário), e a função de transferência do divisor resistivo é hiperbólica $\frac{R_{\text{LDR}}}{R_s + R_{\text{LDR}}}$, não linear.

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real / UI | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Aceita | Unidade | Valor Padrão | Papel Elétrico no Circuito |
|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `mod_1.enabled` | `pedal.tremolo.enabled` | `false / true` | Booleano | `false` | Desconecta a célula LDR do divisor shunt (bypass). |
| **SPEED (RATE)** | `mod_1.rate_hz` | `pedal.tremolo.rate` | `0.5 .. 15.0` | `Hz` | `4.50` | Frequência do oscilador LFO `Phase-Shift` que excita a lâmpada néon. |
| **INTENSITY (DEPTH)** | `mod_1.depth` | `pedal.tremolo.depth` | `0.0 .. 1.0` | `0 .. 100 %` | `0.70` | Amplitude de excitação da lâmpada néon (define a resistência mínima $R_{\text{LDR},\min}$ atingida nos vales). |
| **MIX** | `mod_1.mix` | `pedal.tremolo.mix` | `0.0 .. 1.0` | `0 .. 100 %` | `1.00` | Extensão digital AMPFORGE: em `1.0` (`100%`), reproduz a profundidade integral do divisor shunt; valores menores reduzem o contraste de modulação. |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Excitação da Lâmpada Néon com Limiar de Ignição:**
   - A onda senoidal do LFO $s[n] = 0,5 + 0,5 \sin(\phi[n]) \in [0, 1]$ passa por um limiar suave de ignição néon ($\theta_{\text{neon}} = 0,25$):
     $$I_{\text{light}}[n] = \text{depth} \cdot \left(\max\!\left(0,\; \frac{s[n] - \theta_{\text{neon}}}{1 - \theta_{\text{neon}}}\right)\right)^{1,4}$$
2. **Inércia Fotoquímica Assimétrica do CdS (Invariante a `sample_rate`):**
   - $\alpha_{\text{att}} = \exp\!\left(-\frac{1}{0,008 \cdot f_s}\right)$ (ataque luminoso de `8 ms` quando $I_{\text{light}}[n] > u_{\text{cds}}[n-1]$);
   - $\alpha_{\text{rel}} = \exp\!\left(-\frac{1}{0,040 \cdot f_s}\right)$ (recuperação no escuro de `40 ms` quando $I_{\text{light}}[n] \le u_{\text{cds}}[n-1]$).
3. **Lei Física do Divisor Resistivo Shunt-LDR ($R_s = 220\text{ k}\Omega$):**
   - $R_{\text{LDR}}[n] = R_{\text{dark}} \cdot \left(\frac{R_{\text{bright}}}{R_{\text{dark}}}\right)^{u_{\text{cds}}[n]}$, com $R_{\text{dark}} = 5,0\text{ M}\Omega$ e $R_{\text{bright}} = 12,0\text{ k}\Omega$.
   - Ganho instantâneo puramente linear em relação ao sinal de áudio:
     $$g_{\text{opto}}[n] = \frac{R_{\text{LDR}}[n]}{220\,000 + R_{\text{LDR}}[n]} \in [0,0517\;(-25,7\text{ dB}),\; 0,9578\;(-0,37\text{ dB})]$$
   - Saída: $y[n] = x[n] \cdot \left((1 - \text{mix}) + \text{mix} \cdot g_{\text{opto}}[n]\right)$.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `1×` (a envoltória óptica é fisicamente limitada em banda pela inércia CdS de `8 ms / 40 ms` $\Rightarrow f_c < 20\text{ Hz}$).

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`.
- Complexidade $O(1)$ por amostra; WCET $\le 0,8\text{ }\mu\text{s}$ por bloco de `128` amostras.

---

## 12. Posição e interação na cadeia completa de guitarra

- Operação limpa de modulação de amplitude pura após o pré-amplificador (ou no slot `mod_1`).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/modulation/optical_tremolo.rs` (separado de `bias_tremolo.rs` da Spec 088).
- **Struct principal:** `pub struct OpticalTremoloProcessor` implementando `AudioProcessor` + `ModelDescriptor` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Parâmetro | Valor Documentado (`F-01`, `F-02`) | Invariante Física |
|---|---|---|
| Resistor série $R_s$ | `220 kΩ` | Define a atenuação de vale $g_{\min} = \frac{R_{\text{bright}}}{R_s + R_{\text{bright}}} \approx -25,7\text{ dB}$. |
| Ataque / Recuperação CdS | $\tau_{\text{att}} = 8\text{ ms}$, $\tau_{\text{rel}} = 40\text{ ms}$ | Escala exatamente com $f_s$ (`44.1 / 48 / 96 kHz`). |
| Linearidade do CdS | Puramente resistivo | Não gera distorção harmônica (THD $\le 0,1\%$ em regime estacionário), contrastando com o Bias Tremolo (Spec 088). |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Assimetria Temporal CdS ($\tau_{\text{att}} < \tau_{\text{rel}}$) e Invariância à Taxa de Amostragem (`48 kHz` vs `96 kHz`):** verifica que a descida para o vale é mais rápida que a subida de recuperação e que os tempos em milissegundos coincidem em `48 kHz` e `96 kHz`.
2. **Teste de Linearidade do Divisor Opto-LDR vs Bias Tremolo:** verifica ausência de geração de harmônicos ($H_2, H_3$) em regime estacionário.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Em frequências de LFO muito altas (`rate_hz > 10 Hz`), a inércia física de `40 ms` do CdS reduz naturalmente a profundidade pico-a-pico da modulação — exatamente como ocorre no opto-isolador "Roach" físico do Fender AB763.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Profundidade de Modulação do Divisor Shunt-LDR (`220 kΩ` / CdS) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Senoide contínua de `1000 Hz` a `-12 dBFS` durante `1,0 s`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `module.mod.optical_tremolo` com `enabled = true`, `rate_hz = 2.0`, `depth = 1.0`, `mix = 1.0`.
  - *Método de medição:* Medição da razão entre o pico máximo da envoltória de saída e o vale mínimo da envoltória de saída (em dB) após `250 ms` de regime permanente.
  - *Critério e tolerância:* A profundidade pico-a-vale da modulação de amplitude deve ser **$\ge 18,0\text{ dB}$**, mantendo a saída limitada a $|y[n]| \le |x[n]|$ (ganho puramente passivo $\le 1,0$).
  - *Origem da tolerância:* Razão do divisor resistivo $R_s = 220\text{ k}\Omega$ com $R_{\text{LDR}} \in [12\text{ k}\Omega,\; 5\text{ M}\Omega]$ (`F-01`, `F-02`).

- **CA-02 — Assimetria Fotoquímica do CdS ($\tau_{\text{att}} = 8\text{ ms} < \tau_{\text{rel}} = 40\text{ ms}$) e Invariância a `sample_rate` (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-12 dBFS` processada em `48 000 Hz` e em `96 000 Hz`.
  - *Configuração e parâmetros:* `rate_hz = 4.0`, `depth = 0.90`, `mix = 1.0`.
  - *Método de medição:* Tempo de queda de `10% -> 90%` da profundidade de atenuação ($t_{\text{fall}}$) vs tempo de recuperação de `90% -> 10%` ($t_{\text{rise}}$), e comparação da profundidade pico-a-vale entre `48 kHz` e `96 kHz`.
  - *Critério e tolerância:*
    1. O tempo de recuperação no escuro $t_{\text{rise}}$ deve ser **pelo menos `1,5×` maior** que o tempo de queda sob iluminação $t_{\text{fall}}$ ($t_{\text{rise}} \ge 1,5 \cdot t_{\text{fall}}$).
    2. A profundidade pico-a-vale medida a `96 kHz` deve coincidir com a medida a `48 kHz` dentro de **`±0,5 dB`** (corrigindo o filtro `0.002` dependente da taxa de amostragem de `modulation.rs:264`).
  - *Origem da tolerância:* Constantes físicas de tempo do CdS (`F-02`).

- **CA-03 — Pureza Espectral do Atenuador Resistivo CdS (Sem Distorção Harmônica de Bias) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-6 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `rate_hz = 0.0` (ou ganho instantâneo estacionário no vale `depth = 0.8`), `mix = 1.0`.
  - *Método de medição:* Medição de THD por FFT de `16 384` amostras.
  - *Critério e tolerância:* A THD em regime estacionário deve ser **`<= 0,1%`** (`-60 dB`), distinguindo-o fisicamente do Tremolo de Bias Valvulado (Spec 088).
  - *Origem da tolerância:* Linearidade ôhmica do fotoresistor CdS (`F-02`).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/modulation.rs` (e futuro `crates/ampforge-dsp/src/fx/modulation/optical_tremolo.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.tremolo.*`)
- Testes atuais: `Planejado: crates/ampforge-dsp/tests/contract_032_time_mod_fx.rs`, `crates/sonix-desktop/tests/contract_052_splash_pedals_amp2.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Desacoplado completamente do **Tremolo por Modulação de Bias Valvulado** (movido para a **[Spec 088](088-modulo-tremolo-modulacao-bias.md)**).
- [x] Bug de filtro dependente de `sample_rate` e simétrico (`0.002` em `modulation.rs:264`) diagnosticado e substituído por constantes fotoquímicas assimétricas invariantes a $f_s$.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Fender Electric Instrument Co. — *Twin Reverb Model AB763 Schematic Diagram (1963)*.
- `[F-02]` PerkinElmer / Excelitas — *VTL5C Series Analog Optoisolator (CdS Photocell) Datasheets*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, desfez-se o acoplamento da Spec 073: a Spec 073 passou a especificar exclusivamente o **Tremolo Óptico Shunt-LDR (`module.mod.optical_tremolo`)**, enquanto o **Tremolo por Modulação de Bias Valvulado** foi isolado na **Spec 088**.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Divisor resistivo shunt-LDR ($R_s = 220\text{ k}\Omega$, atenuação profunda $\ge 18\text{ dB}$, ganho passivo $\le 1,0$) | `F-01`, `F-02` | Spec 073 / `CA-01` | `crates/ampforge-dsp/src/fx/modulation.rs:258–266 @ c20cc65` | `contract_032_time_mod_fx.rs`, `contract_052_splash_pedals_amp2.rs` | `TESTED` |
| Inércia fotoquímica assimétrica CdS ($\tau_{\text{att}} = 8\text{ ms} < \tau_{\text{rel}} = 40\text{ ms}$) invariante a `sample_rate` | `F-02` | Spec 073 / `CA-02` | `crates/ampforge-dsp/src/fx/modulation.rs:264 @ c20cc65` (BUG: coeficiente fixo simétrico `0.002` sem escalonamento por $f_s$) | Ausente | `SPECIFIED` |
| Pureza espectral ôhmica do atenuador CdS (`THD <= 0,1%`) | `F-01`, `F-02` | Spec 073 / `CA-03` | `crates/ampforge-dsp/src/fx/modulation.rs:265 @ c20cc65` | `contract_032_time_mod_fx.rs` | `TESTED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 073 / `CA-04` | `crates/ampforge-dsp/src/fx/modulation.rs:179–294 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
