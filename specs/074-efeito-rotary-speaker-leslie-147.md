---
status: rascunho
---

# Spec 074: Efeito Eletromecânico Rotary Speaker Leslie 147 (Bi-Banda 800 Hz com Rotores Independentes Horn & Drum)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-modulation` (`cab-rotary` / `efeito-eletromecanico`)
> **Identificador canônico:** `cab.leslie.147_rotary_speaker`
> **Variante / Revisão:** `Electro Music / Leslie Model 147 (Crossover Passivo 800 Hz, Horn Rotor Jensen V21 + Drum Rotor 15" Woofer, Power Amp 40W 2× 6550)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/modulation.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md), [052](052-abertura-animada-pedais-faltantes-e-amp2-visivel.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md)

---

## 1. Objetivo

Especificar o contrato técnico, acústico, eletromecânico e de verificação do gabinete rotativo **Leslie Model 147 (`cab.leslie.147_rotary_speaker`)**, diagnosticando a simplificação severa presente no código atual ([`crates/ampforge-dsp/src/fx/modulation.rs:275–286 @ c20cc65`](../crates/ampforge-dsp/src/fx/modulation.rs#L275-L286)) — que aplica uma única linha de atraso modulada sobre o sinal de banda inteira com uma única velocidade instantânea — e estabelecendo o modelo físico bi-banda fiel ao manual de serviço da Electro Music / Leslie:
1. Estágio amplificador valvulado interno de `40 W` (`12AU7` + par push-pull `2× 6550`);
2. Crossover passivo LC de **`800 Hz`** dividindo o sinal entre o **Treble Horn Rotor (corneta giratória `Jensen V21`)** e o **Bass Drum Rotor (tambor defletor de madeira acoplado ao woofer de `15"`)**;
3. **Dois rotores mecanicamente independentes** com velocidades distintas em **`CHORALE` (`Slow`)** e **`TREMOLO` (`Fast`)** e **constantes de inércia de aceleração/desaceleração radicalmente diferentes** (a corneta leve acelera em `~0,8–1,2 s`, enquanto o tambor pesado de madeira leva `~4,0–6,0 s` para atingir rotação máxima!);
4. Modulação simultânea de **Efeito Doppler (atraso variável)**, **Diretividade Acústica (AM + LPF dependente do ângulo $\theta(t)$)** e **Captação Estéreo por Par de Microfones** posicionados a $90^\circ$ ou $180^\circ$ em torno do gabinete de madeira.

---

## 2. Escopo

1. Estágio de saturação quente do amplificador valvulado interno `40 W` (`2× 6550`) do Leslie 147.
2. Crossover de fase coerente (`Linkwitz-Riley 2ª/4ª ordem`) em **`800 Hz`** separando a banda grave (`< 800 Hz`, Drum) da banda aguda (`> 800 Hz`, Horn).
3. Simulação física independente dos dois motores e correias (`Horn` vs `Drum`), com velocidades nominais **Chorale (Slow)** e **Tremolo (Fast)**, rampas de inércia de aceleração/desaceleração separadas e rotação em sentidos opostos ou independentes.
4. Modelagem acústica simultânea de:
   - **Desvio Doppler** proporcional à velocidade tangencial da boca da corneta ($r_{\text{horn}} \approx 18\text{ cm}$) e do tambor ($r_{\text{drum}} \approx 22\text{ cm}$);
   - **Modulação de Amplitude e Filtro Direcional (AM + LPF off-axis):** quando a corneta aponta para trás do microfone ($\theta = \pi$), não só o nível cai como os agudos acima de `2 kHz` sofrem sombra acústica severa do gabinete;
   - **Captação Estéreo (L/R)** com microfones virtuais em quadratura ($90^\circ$) ou oposição ($180^\circ$) e somatório mono compatível.

---

## 3. Fora de escopo

- **Fender Vibratone / Leslie 16 / 18 (rotor único de espuma de `10"` sem corneta de agudos e sem amplificador `6550`):** Possui apenas um rotor de banda completa, topologia diferente do Leslie 147 de dois rotores.
- **Leslie 122:** Acusticamente quase idêntico ao 147 (mesmo gabinete, crossover `800 Hz`, horn `V21`, woofer `15"` e `2× 6550`), diferindo apenas na interface de entrada balanceada vs desbalanceada do 147 e no controle de relé de velocidade; o 147 é o modelo canônico para guitarra por possuir entrada de linha/instrumento desbalanceada.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `cab-rotary` (`electromechanical-rotary-speaker`) |
| **Fabricante de referência** | Electro Music / CBS Musical Instruments (Don Leslie, Pasadena, CA, EUA) |
| **Modelo de referência** | Leslie Speaker Model 147 |
| **Ano / época / revisão** | 1963–1980 (Amplificador `147 Amp` 40 W `2× 6550`, Crossover `800 Hz`, Driver `Jensen V21`, Woofer `15"` `Jensen P15LL`, Motores Duplos `Chorale / Tremolo`) |
| **Identificador no sistema** | `cab.leslie.147_rotary_speaker` (`ModModel::RotaryLeslie` / UI `pedal.rotary.*`) |
| **Papel na cadeia de sinal** | Bloco `MOD` (`mod_1` ou `mod_2`, ou pós-amp/pós-cab), clássico em gravações de guitarra (Beatles, Stevie Ray Vaughan, David Gilmour, Jimi Hendrix). |

---

## 5. Princípio físico e funcional do equipamento real

De acordo com o *Leslie Model 147 Service Manual* (`F-01`) e a literatura acadêmica de síntese física do Leslie (Julius O. Smith III / Pekonen et al., `F-02`):

1. **Crossover Passivo de `800 Hz` (Separação Física dos Transdutores):**
   - O sinal de saída do transformador do amplificador valvulado interno `2× 6550` (`40 W RMS`) passa por um crossover passivo LC de 2ª ordem em **`800 Hz`**:
     - **Banda Aguda (`> 800 Hz`):** Alimenta o driver de compressão `Jensen V21` acoplado a um rotor plástico leve de duas cornetas (apenas **uma** corneta é acusticamente aberta; a segunda é tampada e serve apenas como contrapeso dinâmico!).
     - **Banda Grave (`< 800 Hz`):** Alimenta o alto-falante de `15"` apontado para baixo contra um tambor rotativo pesado de madeira com uma abertura lateral inclinada a $45^\circ$.
2. **Velocidades Nominais e Inércias Mecânicas Independentes (`Horn` vs `Drum`):**
   - Cada rotor possui um par de motores AC acoplados por polias e correias com massas rotacionais muito distintas:
     - **Treble Horn Rotor (leve):**
       - *Chorale (Slow):* `~48 RPM` (`0,80 Hz`);
       - *Tremolo (Fast):* `~400 RPM` (`6,67 Hz`);
       - *Aceleração (`Slow → Fast`):* **`~0,8 s` a `1,2 s`**;
       - *Desaceleração (`Fast → Slow`):* **`~1,2 s` a `1,8 s`**.
     - **Bass Drum Rotor (pesado):**
       - *Chorale (Slow):* `~40 RPM` (`0,67 Hz`);
       - *Tremolo (Fast):* `~342 RPM` (`5,70 Hz`);
       - *Aceleração (`Slow → Fast`):* **`~4,5 s` a `6,0 s`**;
       - *Desaceleração (`Fast → Slow`):* **`~5,5 s` a `8,0 s`**.
   - **Por que a independência dos dois rotores é essencial:** Quando o guitarrista aciona a chave de velocidade (`Slow ↔ Fast`), a corneta de agudos dispara rapidamente para `6,67 Hz` em ~1 segundo enquanto o tambor de graves ainda está ganhando embalo lentamente ao longo de 5 segundos! Além disso, mesmo em regime estacionário, as velocidades (`6,67 Hz` vs `5,70 Hz`) **não são múltiplos inteiros**, gerando um batimento orgânico nunca repetitivo.
3. **Acoplamento Físico Doppler + Diretividade Espectral (*Off-Axis Shadowing*):**
   - Seja $r_h \approx 0,18\text{ m}$ o raio efetivo da boca da corneta e $c = 343\text{ m/s}$ a velocidade do som. À medida que o ângulo $\theta_h(t)$ gira em relação ao microfone, a distância acústica varia em $\pm r_h \cos(\theta_h(t))$, produzindo um atraso variável:
     $$\tau_h(t) = \tau_{0,h} - \frac{r_h}{c} \cos(\theta_h(t)) \implies \Delta\tau_h = \pm 0,525\text{ ms}$$
     e um desvio Doppler instantâneo $\frac{\Delta f}{f} = \frac{r_h \omega_h}{c} \sin(\theta_h(t)) \approx \pm 2,2\%$ em `400 RPM` ($\pm 38\text{ cents}$!).
   - Simultaneamente, quando a corneta aponta para o microfone ($\cos\theta_h = +1$), a pressão sonora e os agudos atingem o máximo; quando aponta na direção oposta ($\cos\theta_h = -1$), ocorre forte atenuação de amplitude e corte passa-baixas (*off-axis shadowing* do gabinete).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Electro Music / CBS Musical Instruments — *Leslie Speaker Model 147 / 147RV Service Manual & Schematic* | Manual de serviço oficial Leslie 147 (`800 Hz` crossover, `Jensen V21`, `15"` woofer, `2× 6550` 40W amp, velocidades `Chorale/Tremolo` dos motores) | `UNKNOWN` | Especificações eletromecânicas, crossover `800 Hz`, amplificador `6550` e velocidades dos rotores Horn e Drum. |
| `F-02` | Julius O. Smith III, Stefania Serafin, Jonathan Abel, David Berners — *Doppler Simulation and the Leslie* (DAFx-02, Hamburg, 2002) & CCRMA PASP | `https://ccrma.stanford.edu/~jos/pasp/` (HTTP 200 verificado) | `ACADEMIC` | Equações físicas exatas do atraso Doppler, diretividade dependente do ângulo, frequências de rotação (`~0,8/6,7 Hz` Horn vs `~0,67/5,7 Hz` Drum) e constantes de inércia de aceleração/desaceleração. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Crossover passivo de `800 Hz` dividindo o sinal entre corneta `Jensen V21` e tambor de graves `15"` | `F-01`, `F-02` | Leslie 147 | `F-01` Schematic; `F-02` §2 | `UNKNOWN` / `ACADEMIC` | `CONFIRMED` |
| Velocidades distintas de Horn (`~48 / 400 RPM` $\approx 0,80 / 6,67\text{ Hz}$) e Drum (`~40 / 342 RPM` $\approx 0,67 / 5,70\text{ Hz}$) com inércias mecânicas independentes (`~1 s` Horn vs `~5 s` Drum) | `F-01`, `F-02` | Leslie 147 | `F-02` §2–3 Table 1 | `UNKNOWN` / `ACADEMIC` | `CONFIRMED` |
| Modulação simultânea de Doppler ($\Delta\tau_h \approx \pm 0,525\text{ ms}$) e filtro direcional *off-axis* (AM + LPF dependente de $\theta$) | `F-02` | DAFx-02 / PASP | `F-02` §3–4 | `ACADEMIC` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Input] ──► [Leslie 147 40W Tube Pre/Power Warmth (2x 6550)]
                     │
                     ▼
          [Linkwitz-Riley 800 Hz Crossover]
              ├──► (High Band > 800 Hz) ──► [Horn Rotor (r=18cm, τ_acc≈1.0s, ω_h: 0.80..6.67 Hz)]
              │                              ├──► Doppler Delay Line τ_h(θ_h)
              │                              └──► Directional AM + Off-Axis LPF(θ_h) ──┐
              │                                                                        ├──► [Cabinet Resonance & Stereo/Mono Mic Mix] ──► [Out]
              └──► (Low Band < 800 Hz)  ──► [Drum Rotor (r=22cm, τ_acc≈5.0s, ω_d: 0.67..5.70 Hz)]
                                             ├──► Doppler Delay Line τ_d(θ_d)          │
                                             └──► Directional AM + Off-Axis LPF(θ_d) ──┘
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/modulation.rs:275–286 @ c20cc65`](../crates/ampforge-dsp/src/fx/modulation.rs#L275-L286) possui simplificações severas que não representam um Leslie 147:

1. **`FACTUAL_ERROR` (`modulation.rs:275–286` — Ausência de Crossover `800 Hz` e Rotor Único):**
   - O código atual processa o sinal de banda inteira com **uma única linha de atraso** (`delay_ms = 2.0 + sin * depth * 1.5`) e um único multiplicador de amplitude `am = 1.0 - depth * 0.35 * (0.5 + 0.5 * cos)`!
   - Não existe crossover em `800 Hz`, não existe separação entre **Horn** e **Drum**, não existe rampa de inércia mecânica (`~1 s` vs `~5 s`) ao trocar velocidade, e não existe filtro direcional passa-baixas dependente do ângulo de rotação!

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real / UI | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Aceita | Unidade | Valor Padrão | Papel Eletromecânico |
|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `mod_1.enabled` | `pedal.rotary.enabled` | `false / true` | Booleano | `false` | Ativa/desativa o bloco Leslie 147. |
| **SPEED / RATE** | `mod_1.rate_hz` | `pedal.rotary.rate` | `0.4 .. 8.0` | `Hz` | `0.80` (`Chorale`) | Velocidade alvo da corneta (`Horn`, com `Drum` escalado fisicamente em `0,855 × rate_hz` seguindo `342/400 RPM`, acelerando e desacelerando com inércias mecânicas distintas). |
| **DEPTH (HORN/MIC DISTANCE)** | `mod_1.depth` | `pedal.rotary.depth` | `0.0 .. 1.0` | `0 .. 100 %` | `0.75` | Controla a proximidade virtual dos microfones (profundidade do Doppler, modulação AM e contraste *off-axis*). |
| **MIX (BALANCE)** | `mod_1.mix` | `pedal.rotary.mix` | `0.0 .. 1.0` | `0 .. 100 %` | `1.00` | Em `1.0` (`100%`), reproduz a saída integral microfonada do gabinete Leslie 147. |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Crossover Coerente em Fase em `800 Hz` (`LR4` / `LR2` Complementar):**
   - Separa $x[n]$ em $x_{\text{low}}[n]$ (`< 800 Hz`) e $x_{\text{high}}[n]$ (`> 800 Hz`) tais que $x_{\text{low}} + x_{\text{high}}$ tem magnitude plana (`±0,05 dB`) na frequência de cruzamento `800 Hz`.
2. **Dinâmica de Inércia Mecânica Independente dos Dois Motores (`Horn` vs `Drum`):**
   - Dada a velocidade alvo da corneta $f_{h,\text{target}} = \text{rate\_hz}$ e do tambor $f_{d,\text{target}} = 0,855 \cdot \text{rate\_hz}$:
     - **Horn:** acelera com $\tau_{h,\text{acc}} = 0,90\text{ s}$ e desacelera com $\tau_{h,\text{dec}} = 1,40\text{ s}$;
     - **Drum:** acelera com $\tau_{d,\text{acc}} = 4,80\text{ s}$ e desacelera com $\tau_{d,\text{dec}} = 6,20\text{ s}$.
   - As fases instantâneas $\theta_h[n]$ e $\theta_d[n]$ integram $2\pi f_h[n] / f_s$ e $2\pi f_d[n] / f_s$ (em sentidos opostos, $\Delta\theta_d < 0$).
3. **Doppler + Diretividade AM + Sombra Acústica LPF em Cada Rotor:**
   - **Horn (`> 800 Hz`):**
     $$\tau_h[n] = \tau_{0,h} - \text{depth} \cdot 0,525\text{ ms} \cdot \cos(\theta_h[n])$$
     $$g_h[n] = 1,0 - \text{depth} \cdot 0,48 \cdot (0,5 - 0,5 \cos(\theta_h[n])), \qquad f_{\text{lpf},h}[n] = 1600 + 6400 \cdot (0,5 + 0,5 \cos(\theta_h[n]))\text{ Hz}$$
   - **Drum (`< 800 Hz`):**
     $$\tau_d[n] = \tau_{0,d} - \text{depth} \cdot 0,64\text{ ms} \cdot \cos(\theta_d[n])$$
     $$g_d[n] = 1,0 - \text{depth} \cdot 0,32 \cdot (0,5 - 0,5 \cos(\theta_d[n]))$$
   - Para saída estéreo (L/R), o microfone direito lê o ângulo defasado de $\Delta\theta_{\text{mic}} = \pi/2$ (`90°`) ou $\pi$ (`180°`).

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Interpolação de Doppler:** Lagrange cúbica de 4 pontos nas duas linhas de atraso (`Horn` e `Drum`), mantendo resposta de alta frequência limpa durante o desvio Doppler de `±38 cents`.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Buffers de atraso pré-alocados em `prepare()`; zero alocações em `process_block`.
- WCET $\le 3,5\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Classicamente posicionado após o amplificador (ou em paralelo no final da cadeia), substituindo ou complementando a projeção espacial de um gabinete estático.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/modulation/leslie_147.rs`.
- **Struct principal:** `pub struct Leslie147Processor` implementando `AudioProcessor` + `ModelDescriptor` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Parâmetro Leslie 147 | Valor Documentado (`F-01`, `F-02`) | Invariante Física |
|---|---|---|
| Frequência de Crossover | `800 Hz` (`LR4` complementar) | Graves (`< 800 Hz`) vão exclusivamente ao Drum; Agudos (`> 800 Hz`) vão exclusivamente ao Horn. |
| Razão de Velocidade Drum / Horn | $342 / 400 = 0,855$ | Os dois rotores giram em frequências assíncronas. |
| Inércia Mecânica (`Slow → Fast`) | Horn $\tau \approx 0,9\text{ s}$ vs Drum $\tau \approx 4,8\text{ s}$ | O Horn atinge `90%` da nova velocidade pelo menos `3×` mais rápido que o Drum. |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Separação Bi-Banda em `800 Hz` e Assincronia dos Rotores:** excitação simultânea com tom grave (`200 Hz`, Drum) e tom agudo (`2500 Hz`, Horn) comprova taxas de modulação distintas (`f_drum ≈ 0,855 * f_horn`).
2. **Teste de Inércia Mecânica Diferencial (`Chorale 0,8 Hz → Tremolo 6,67 Hz`):** verifica que após `1,5 s` do comando de aceleração, a banda aguda (`2500 Hz`, Horn) já atingiu `> 70%` da nova velocidade, enquanto a banda grave (`200 Hz`, Drum) ainda está abaixo de `45%` da aceleração final.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- A atualização de coeficientes do filtro direcional *off-axis* `f_lpf_h[n]` deve ser suavizada ou atualizada via SVF (*State Variable Filter* TPT de Zavalishin) para evitar qualquer artefato de modulação rápida durante a rotação em `400 RPM` (`6,67 Hz`).

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Separação Bi-Banda no Crossover de `800 Hz` e Velocidades Assíncronas (`Horn` vs `Drum`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Dois testes independentes com senoide grave de `200 Hz` (`< 800 Hz`, banda Drum) e senoide aguda de `2500 Hz` (`> 800 Hz`, banda Horn) a `-12 dBFS` por `4,0 s`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `cab.leslie.147_rotary_speaker` com `enabled = true`, `rate_hz = 6.67` (`Tremolo 400 RPM` após regime permanente), `depth = 0.85`, `mix = 1.0`.
  - *Método de medição:* Frequência fundamental da envoltória AM (via FFT do envelope retificado) na saída para a entrada de `2500 Hz` ($f_{\text{mod,horn}}$) e para a entrada de `200 Hz` ($f_{\text{mod,drum}}$).
  - *Critério e tolerância:*
    1. Para `2500 Hz` (Horn), a taxa de modulação deve ser **`6,67 Hz ± 0,25 Hz`**.
    2. Para `200 Hz` (Drum), a taxa de modulação deve ser **`5,70 Hz ± 0,25 Hz`** (razão $f_{\text{drum}}/f_{\text{horn}} \in [0,80,\; 0,91]$), comprovando o crossover físico em `800 Hz` e os dois rotores independentes.
  - *Origem da tolerância:* Velocidades nominais `400 RPM` (Horn) e `342 RPM` (Drum) no Leslie 147 (`F-01`, `F-02`).

- **CA-02 — Inércia Mecânica Diferencial de Aceleração (`Horn ~1 s` vs `Drum ~5 s`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Comutação em degrau do parâmetro `rate_hz` de `0.80 Hz` (`Chorale`) para `6.67 Hz` (`Tremolo`) em $t = 0$.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Método de medição:* Velocidade angular instantânea dos rotores `horn_freq_hz(t)` e `drum_freq_hz(t)` em $t = 1,5\text{ s}$ após a comutação.
  - *Critério e tolerância:* Em $t = 1,5\text{ s}$ após o degrau `Chorale -> Tremolo`, a corneta (Horn) deve ter completado **$\ge 70\%$** da transição de velocidade, enquanto o tambor de graves (Drum) deve ter completado **$\le 45\%$** da sua transição (tempo de aceleração do Drum pelo menos `3×` mais lento que o do Horn).
  - *Origem da tolerância:* Constantes de inércia medidas do Leslie 147 (`F-02` Table 1).

- **CA-03 — Acoplamento Físico de Efeito Doppler + Diretividade Espectral (*Off-Axis Shadowing*) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `3000 Hz` a `-12 dBFS` (banda Horn) com `rate_hz = 6.67`, `depth = 0.85`, `mix = 1.0`.
  - *Método de medição:* Desvio de frequência de pico-a-pico por Doppler ($\Delta f_{\text{Doppler}}$) e contraste de amplitude AM (dB) ao longo de 1 rotação da corneta.
  - *Critério e tolerância:* A senoide de `3000 Hz` deve exibir simultaneamente desvio de pitch Doppler de pico **$\ge \pm 30\text{ Hz}$** (`±17 cents` a `±40 cents`) **e** modulação de amplitude direcional **$\ge 4,0\text{ dB}$**.
  - *Origem da tolerância:* Geometria física $r_h \approx 0,18\text{ m}$ a `400 RPM` (`F-02`).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `rate_hz`, `depth`, `mix`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/modulation.rs` (e futuro `crates/ampforge-dsp/src/fx/modulation/leslie_147.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.rotary.*`)
- Testes atuais: `Planejado: crates/ampforge-dsp/tests/contract_032_time_mod_fx.rs`, `crates/sonix-desktop/tests/contract_052_splash_pedals_amp2.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Arquitetura eletromecânica do Leslie 147 documentada com crossover `800 Hz`, rotores independentes Horn/Drum, inércias diferenciais e acoplamento Doppler + diretividade.
- [x] Simplificação de rotor único sem crossover em `modulation.rs:275–286` diagnosticada.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Electro Music / CBS Musical Instruments — *Leslie Speaker Model 147 Service Manual & Schematic*.
- `[F-02]` Smith, J. O., Serafin, S., Abel, J., & Berners, D. (2002). *Doppler Simulation and the Leslie*. Proc. DAFx-02, Hamburg (`https://ccrma.stanford.edu/~jos/pasp/`).

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, diagnosticou-se que `ModModel::RotaryLeslie` em `modulation.rs:275–286` não possuía crossover em `800 Hz`, nem dois rotores independentes, nem rampa de inércia mecânica; todos esses requisitos físicos foram especificados em `CA-01` a `CA-03`.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Crossover `800 Hz` e dois rotores independentes (`Horn 400 RPM` vs `Drum 342 RPM`) | `F-01`, `F-02` | Spec 074 / `CA-01` | `crates/ampforge-dsp/src/fx/modulation.rs:275–286 @ c20cc65` (SIMPLIFICADO: banda única, rotor único sem crossover `800 Hz`) | Ausente | `SPECIFIED` |
| Inércia mecânica diferencial de aceleração/desaceleração (`Horn ~1 s` vs `Drum ~5 s`) | `F-01`, `F-02` | Spec 074 / `CA-02` | Ausente em `modulation.rs` (mudança de `rate_hz` é instantânea) | Ausente | `SPECIFIED` |
| Acoplamento Doppler + modulação direcional AM e filtro *off-axis* | `F-02` | Spec 074 / `CA-03` | `crates/ampforge-dsp/src/fx/modulation.rs:276–285 @ c20cc65` (Doppler + AM presentes em banda única, sem filtro *off-axis*) | `contract_032_time_mod_fx.rs`, `contract_052_splash_pedals_amp2.rs` | `IMPLEMENTED_UNVERIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 074 / `CA-04` | `crates/ampforge-dsp/src/fx/modulation.rs:179–294 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
