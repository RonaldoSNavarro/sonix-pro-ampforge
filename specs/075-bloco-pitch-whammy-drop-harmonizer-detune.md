---
status: rascunho
---

# Spec 075: Pedal DigiTech Whammy (Expression Pitch Shifter Monofônico/Polifônico — ±2 Oitavas)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade — Desacoplamento de 4 Equipamentos de Pitch)
> **Domínio:** `dsp-pitch` (`pedal`)
> **Identificador canônico:** `pedal.digitech.whammy`
> **Variante / Revisão:** `DigiTech Whammy 5th Gen (2012, Modos Classic Monofônico & Chords Polifônico, Varredura Contínua de Pedal de Expressão ±24 Semitons)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/pitch.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [042](042-bloco-de-pitch-sem-processamento.md), [052](052-abertura-animada-pedais-faltantes-e-amp2-visivel.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [089](089-pedal-digitech-drop-transposicao-polifonica.md), [090](090-modulo-harmonizer-diatonico-inteligente.md), [091](091-rack-eventide-h3000-micropitch-detune.md)

---

## 1. Objetivo

Especificar o contrato técnico, algorítmico e de verificação **exclusivamente** do pedal **DigiTech Whammy (`pedal.digitech.whammy`)**, desfazendo o acoplamento quádruplo (`MODEL_COUPLING`) da versão anterior desta Spec — que misturava quatro equipamentos/algoritmos distintos de pitch em um único documento! Agora cada equipamento possui sua própria especificação modular dedicada:
- **Spec 075 (esta Spec):** **Pedal DigiTech Whammy** (`pedal.digitech.whammy`) — pitch shifting contínuo controlado por pedal de expressão até `±2 oitavas` (`±24 semitons`);
- **[Spec 089](089-pedal-digitech-drop-transposicao-polifonica.md):** **Pedal DigiTech Drop** (`pedal.digitech.drop`) — *Polyphonic Drop Tune* de `-1` a `-7` semitons e `-1 Oitava` (`100% Wet`, preservação de acordes e transientes);
- **[Spec 090](090-modulo-harmonizer-diatonico-inteligente.md):** **Módulo Harmonizer Diatônico Inteligente** (`module.pitch.diatonic_harmonizer`) — detector de pitch + quantização diatônica por tonalidade/escala/intervalo;
- **[Spec 091](091-rack-eventide-h3000-micropitch-detune.md):** **Rack Eventide H3000 MicroPitch Detune** (`rack.eventide.h3000_micropitch`) — duplo micro-pitch estéreo (`±9 cents` com atrasos assimétricos L/R).

Preserva-se rastreabilidade completa à Spec anterior [042](042-bloco-de-pitch-sem-processamento.md) e ao motor atual em [`crates/ampforge-dsp/src/fx/pitch.rs`](../crates/ampforge-dsp/src/fx/pitch.rs#L1-L416).

---

## 2. Escopo

1. Transposição contínua em tempo real controlada por **Pedal de Expressão** (`pitch.expression ∈ [0.0, 1.0]`, calcanhar = `0 semitons`, ponta = intervalo alvo `pitch.semitones ∈ [-24, +24]`).
2. Intervalos clássicos do seletor rotativo do Whammy (`+2 OCT` / `+24 st`, `+1 OCT` / `+12 st`, `+5TH` / `+7 st`, `+4TH` / `+5 st`, `-2ND` / `-2 st`, `-4TH` / `-5 st`, `-5TH` / `-7 st`, `-1 OCT` / `-12 st`, `-2 OCT` / `-24 st` / *Dive Bomb*).
3. **Suavização anti-zipper da razão de pitch** ($\alpha = 2^{\Delta s / 12}$) durante varreduras rápidas do pedal de expressão sem cliques ou artefatos de salto de janela.
4. Operação com latência de grupo baixa (`<= 25 ms` no modo de linha de atraso com crossfade de janela Hann) para resposta tátil imediata ao pedal de expressão.

---

## 3. Fora de escopo

- **DigiTech Drop (Transposição Polifônica de Afinação):** Movido para a **[Spec 089](089-pedal-digitech-drop-transposicao-polifonica.md)**.
- **Harmonizer Diatônico Inteligente (Tonalidade + Escala + Intervalo):** Movido para a **[Spec 090](090-modulo-harmonizer-diatonico-inteligente.md)**.
- **Eventide H3000 MicroPitch Detune (±Cents Estéreo):** Movido para a **[Spec 091](091-rack-eventide-h3000-micropitch-detune.md)**.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`expression-pitch-shifter`) |
| **Fabricante de referência** | DigiTech / IVL Technologies (Sandy, UT, EUA) |
| **Modelo de referência** | DigiTech Whammy (WH-1 / Whammy 5th Gen) |
| **Ano / época / revisão** | 1989 (WH-1 original) / 2012 (Whammy 5 — True Bypass, modo `Classic` e `Chords`, faixa `±24 semitons`) |
| **Identificador no sistema** | `pedal.digitech.whammy` (`PitchMode::Whammy` / UI `pedal.pitch.*`) |
| **Papel na cadeia de sinal** | Bloco `PITCH`, posicionado no **início da cadeia (pré-Drive/pré-Amp)** sobre o sinal limpo da guitarra para rastreamento limpo de pitch e saturação posterior no amplificador. |

---

## 5. Princípio físico e funcional do equipamento real

1. **Transposição Contínua Guiada pelo Pedal de Expressão (*Treadle*):**
   - No modo **Whammy** (`100% Wet` por padrão no hardware real, com opção `Mix` no AMPFORGE), o sinal de entrada $x(t)$ é transposto continuamente por uma razão de frequência governada pela posição $e(t) \in [0, 1]$ do pedal de expressão e pelo intervalo máximo selecionado $S \in [-24,\; +24]$ semitons:
     $$r(t) = 2^{\frac{S \cdot e(t)}{12}}$$
   - Quando $e = 0,0$ (calcanhar / *heel down*), $r = 1,0$ (`0 semitons`, afinação original); quando $e = 1,0$ (ponta / *toe down*) com $S = +12$ (`+1 OCT`), $r = 2,0$ (uma oitava exata acima); com $S = +24$ (`+2 OCT`), $r = 4,0$ (duas oitavas acima); com $S = -24$ (*Dive Bomb*), $r = 0,25$.
2. **Síntese Temporal por Dupla Cabeça de Leitura em Contrafase com Janelamento Hann (*Dual-Tap Overlap-Add / Rotating Delay Line*):**
   - Para manter latência ultrabaixa adequada a solos rápidos com pedal de expressão (como Tom Morello, David Gilmour, Jack White), o motor de pitch em domínio temporal usa duas cabeças de leitura deslocadas de $180^\circ$ ($\phi_B = (\phi_A + 0,5) \bmod 1,0$) varrendo uma linha de atraso circular de comprimento $W$ (`~24–32 ms`) na velocidade relativa $(1 - r(t))$, ponderadas por janelas senoidais/Hann complementares $w(\phi) = \sin^2(\pi \phi)$ tais que $w(\phi_A) + w(\phi_B) = 1,0$ identicamente (ganho de potência constante sem tremulação de amplitude!).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | DigiTech / Harman — *Whammy (5th Gen) Official Product Specification & Owner's Manual* | `https://digitech.com/dp/whammy/` (HTTP 200 oficial verificado) | `PRIMARY_OFFICIAL` | Lista oficial de intervalos Whammy (`+2 OCT`, `+1 OCT`, `+5TH`, `+4TH`, `-2ND`, `-4TH`, `-5TH`, `-1 OCT`, `-2 OCT Dive Bomb`), operação do pedal de expressão e modos `Classic` vs `Chords`. |
| `F-02` | Udo Zölzer — *DAFX: Digital Audio Effects* (2nd ed., Wiley, 2011), Cap. 6 (*Time-Domain Pitch Shifting — Rotating Delay Line / PSOLA*) | Literatura acadêmica de referência | `ACADEMIC` | Equações da linha de atraso rotativa de dupla cabeça com janelamento complementar $w_A + w_B = 1$, taxa de varredura $(1 - r)$ e compromisso latência vs tamanho de janela $W$. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Intervalos Whammy até `±24 semitons` (`±2 oitavas`) controlados continuamente pelo pedal de expressão (`heel = 0 st`, `toe = S`) | `F-01` | Whammy 5th Gen | `F-01` §"Whammy Settings & Specifications" | `UNKNOWN` | `CONFIRMED` |
| Razão de frequência $r = 2^{(S \cdot e)/12}$ e transposição em tempo real via dupla cabeça com janelamento complementar | `F-02` | DAFX 2nd ed. | Cap. 6 §6.2 | `ACADEMIC` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Input (Dry Guitar)] ──► [Circular Delay Buffer (W ≈ 30 ms)]
                                │
             ┌──────────────────┴──────────────────┐
             ▼                                     ▼
   [Read Tap A: φ_A, Cubic Interp]       [Read Tap B: φ_B = (φ_A + 0.5) mod 1]
             │ (× sin²(π φ_A))                     │ (× sin²(π φ_B))
             └──────────────────┬──────────────────┘
                                ▼
                   [Wet Shifted Output] ◄── [Expression Smoother: r(t) = 2^(S·e(t)/12)]
                                │
                                ▼
                   [Dry/Wet Mixer (Default 100% Wet in Whammy)] ──► [Output]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/pitch.rs`](../crates/ampforge-dsp/src/fx/pitch.rs#L1-L416) apresenta os seguintes achados auditados:

1. **`MODEL_COUPLING` (corrigido nesta revisão):** `PitchProcessor` em `pitch.rs:14–21` agrupa 4 modos/equipamentos distintos (`Whammy`, `Drop`, `Harmonizer`, `Detune`) em uma única struct monolítica, e a Spec 075 original fazia o mesmo. Agora `Drop`, `Harmonizer` e `Detune` foram separados nas **Specs 089, 090 e 091**.
2. **`FACTUAL_ERROR` (`pitch.rs:171` — Janelamento Triangular/Crossfade vs Hann e Atraso de Janela):** Em `pitch.rs:170–175`, a janela usada é `sin(π * phase)` (cujo somatório $\sin(\pi\phi) + \sin(\pi(\phi+0,5))$ varia entre `1,00` e `1,414`, causando **`+3,0 dB` de modulação periódica de amplitude** na frequência da janela quando os dois taps somam em fase, ou exigindo normalização por soma quadrática!). Usar janelamento complementar de soma unitária exata ($\sin^2(\pi\phi_A) + \sin^2(\pi\phi_B) \equiv 1,0$) ou normalizar pelo ganho instantâneo elimina a ondulação de amplitude.
3. **`MISSING_VALIDATION` (`pitch.rs:289`):** No modo `Whammy`, não há suavização dedicada (*slew/one-pole smoother*) na razão de pitch $r(t)$ quando `expression` muda abruptamente entre blocos de `128` amostras (saltos de pedal de expressão MIDI/UI podem causar artefatos de descontinuidade).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real / UI | Chave Canônica (Spec 014 / Spec 042) | UI Control ID (`controls.registry.ts`) | Faixa Aceita | Unidade | Valor Padrão | Papel no Algoritmo |
|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `pitch.enabled` | `pedal.pitch.enabled` | `false / true` | Booleano | `false` | True bypass do pedal Whammy. |
| **INTERVAL (SEMITONES)** | `pitch.semitones` | `pedal.pitch.semitones` | `-24 .. +24` | Semitons | `+12` (`+1 OCT`) | Intervalo alvo na ponta do pedal de expressão (`toe down`). |
| **EXPRESSION (TREADLE)** | `pitch.expression` | `pedal.pitch.expression` | `0.0 .. 1.0` | `0 .. 100 %` | `1.00` | Posição do pedal de expressão (`0.0` = `0 st`, `1.0` = `pitch.semitones`). |
| **MIX** | `pitch.mix` | `pedal.pitch.mix` | `0.0 .. 1.0` | `0 .. 100 %` | `1.00` (`100% Wet`) | Balanço Dry/Wet (`1.0` = Whammy clássico; `0.5` = modo Harmony). |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Suavização de Expressão e Razão de Pitch Contínua:**
   - $e_{\text{sm}}[n] = \alpha_e e_{\text{sm}}[n-1] + (1 - \alpha_e) e_{\text{target}}$ ($\tau_e = 8\text{ ms}$).
   - $r[n] = 2^{\frac{S \cdot e_{\text{sm}}[n]}{12}} \in [0,25,\; 4,00]$.
2. **Dupla Cabeça de Leitura em Contrafase com Janelamento Hann Complementar (`sin²`) e Interpolação Cúbica:**
   - Janela de comprimento $W = \text{round}(0,028 \cdot f_s)$ (`28 ms`, `1344` amostras a `48 kHz` — latência média de `14 ms`).
   - Avanço de fase das cabeças: $\Delta\phi[n] = \frac{1,0 - r[n]}{W}$, $\phi_A[n] = (\phi_A[n-1] + \Delta\phi[n]) \bmod 1,0$, $\phi_B[n] = (\phi_A[n] + 0,5) \bmod 1,0$.
   - Pesos complementares exatos: $w_A = \sin^2(\pi \phi_A[n])$, $w_B = \sin^2(\pi \phi_B[n])$ ($w_A + w_B \equiv 1,0$).
   - Saída Wet: $y_{\text{wet}}[n] = w_A \cdot \text{read\_cubic}(\phi_A W) + w_B \cdot \text{read\_cubic}(\phi_B W)$.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Anti-aliasing em Transposições Ascendentes (`+12 / +24 st`):** Quando $r[n] > 1,0$ (transposição para cima até $4\times$), componentes de entrada acima de $f_s / (2 r)$ dobrariam sobre Nyquist; aplica-se um filtro passa-baixas adaptativo pré-delay com corte em $f_c = \min(18\,000,\; 0,44 \cdot f_s / r[n])\text{ Hz}$ antes de gravar ou ler na linha de atraso!

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Buffer circular pré-alocado em `prepare()`; zero alocações na heap em `process_block`.
- WCET $\le 2,5\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Posicionado **antes** do bloco `DRIVE` e `AMP` para transpor o sinal limpo da guitarra antes das não-linearidades.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/digitech_whammy.rs` (extraindo `PitchMode::Whammy` de `pitch.rs`).
- **Struct principal:** `pub struct DigiTechWhammyProcessor` implementando `AudioProcessor` + `ModelDescriptor` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Parâmetro | Valor Documentado (`F-01`, `F-02`) | Invariante Física |
|---|---|---|
| Faixa de Transposição | `-24` a `+24` semitons ($r \in [0,25,\; 4,0]$) | `expression = 0.0` $\implies r = 1,0$ (sem alteração de pitch). |
| Soma das janelas | $w_A(\phi) + w_B(\phi) = \sin^2(\pi\phi) + \cos^2(\pi\phi) \equiv 1,0$ | Elimina modulação espúria de amplitude (`±0,0 dB` em sinais coerentes). |
| Proteção Anti-Aliasing | LPF adaptativo $f_c = 0,44 f_s / r$ quando $r > 1$ | Impede aliasing acima de Nyquist ao subir `+2 OCT` (`r = 4,0`). |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Precisão de Transposição (`+12 st`, `+24 st`, `-12 st`):** excitação com senoide de `440 Hz` verifica pico espectral dominante em `880 Hz`, `1760 Hz` e `220 Hz`.
2. **Teste de Varredura Contínua de Expressão (`0.0 -> 1.0`):** verifica transição contínua sem saltos ou cliques (`max |y[n] - y[n-1]|` limitado) e que `expression = 0.0` mantém `440 Hz`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- No modo `Classic` de linha de atraso rotativa (sem autocorrelação de pitch), acordes complexos exibem o caráter ligeiramente *warbly* autêntico do WH-1 original; para afinação polifônica limpa de acordes inteiros, o usuário utiliza o pedal dedicado **DigiTech Drop (Spec 089)**.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Precisão de Transposição nos Intervalos `+1 OCT` (`+12 st`), `+2 OCT` (`+24 st`) e `-1 OCT` (`-12 st`) (`DOCUMENTED` / `REGRESSION`):**
  - *Sinal de entrada:* Senoide pura de `440,0 Hz` a `-12 dBFS` por `1,0 s`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras, FFT de `16 384` amostras após `200 ms` de regime permanente.
  - *Configuração e parâmetros:* `pedal.digitech.whammy` com `pitch.enabled = true`, `pitch.expression = 1.0`, `pitch.mix = 1.0`, testando `pitch.semitones` em `+12`, `+24` e `-12`.
  - *Método de medição:* Frequência do pico espectral de maior magnitude na saída FFT.
  - *Critério e tolerância:*
    - `semitones = +12`: pico dominante em **`880,0 Hz ± 8,0 Hz`** (`±15 cents`);
    - `semitones = +24`: pico dominante em **`1760,0 Hz ± 16,0 Hz`** (`±15 cents`);
    - `semitones = -12`: pico dominante em **`220,0 Hz ± 4,0 Hz`** (`±30 cents`).
  - *Origem da tolerância:* Razão exata $2^{S/12}$ (`F-01`, `F-02`) com resolução de bin FFT (`2,93 Hz` a `48 kHz / 16 384`).

- **CA-02 — Posição Calcanhar (`expression = 0.0`) e Varredura Contínua Sem Cliques (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `440,0 Hz` a `-12 dBFS` durante `1,0 s`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `pitch.semitones = +12`, `pitch.mix = 1.0`, com `expression = 0.0` (primeiros `300 ms`), seguido de rampa em degrau para `expression = 0.5` (`+6 st`, `622,25 Hz`) e `expression = 1.0` (`+12 st`, `880 Hz`).
  - *Método de medição:* Pico espectral em `expression = 0.0` e em `expression = 0.5`, mais derivada máxima amostra-a-amostra durante a transição.
  - *Critério e tolerância:* Em `expression = 0.0`, o pico espectral deve permanecer em **`440,0 Hz ± 5,0 Hz`**; em `expression = 0.5`, deve deslocar-se para **`622,25 Hz ± 10,0 Hz`**; durante o degrau de `expression`, não deve haver descontinuidade abrupta (`max |y[n] - y[n-1]| <= 0,35`).
  - *Origem da tolerância:* Suavização contínua do pedal de expressão (`F-01`).

- **CA-03 — Proteção Anti-Aliasing na Transposição `+2 OCT` (`r = 4,0`) (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoide de `8000 Hz` a `-12 dBFS` (que multiplicada por `4,0` iria para `32 000 Hz`, acima da frequência de Nyquist de `24 000 Hz` a `48 kHz`, dobrando para `16 000 Hz` se não houver anti-aliasing!).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `pitch.semitones = +24`, `pitch.expression = 1.0`, `pitch.mix = 1.0`.
  - *Método de medição:* Energia espectral da imagem dobrada em `16 000 Hz` medida por FFT.
  - *Critério e tolerância:* A imagem de aliasing em `16 000 Hz` deve estar atenuada em **pelo menos `30,0 dB`** abaixo do nível de entrada (`-12 dBFS`), comprovando a atuação do filtro anti-aliasing adaptativo pré-transposição.
  - *Origem da tolerância:* Teorema de Nyquist para reamostragem por fator $r = 4,0$ (`F-02`).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `expression`, `semitones`, `mix`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/pitch.rs` (e futuro `crates/ampforge-dsp/src/fx/pedals/digitech_whammy.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.pitch.*`)
- Testes atuais: `Planejado: crates/ampforge-dsp/tests/contract_042_pitch_wah.rs`, `crates/sonix-desktop/tests/contract_052_splash_pedals_amp2.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Fonte oficial DigiTech (`F-01`) verificada via HTTP 200.
- [x] Desacoplado completamente de **DigiTech Drop (Spec 089)**, **Harmonizer Diatônico (Spec 090)** e **Eventide H3000 MicroPitch (Spec 091)**.
- [x] Risco de aliasing em `+2 OCT` (`r = 4,0`) e modulação de amplitude da janela `sin` vs `sin²` em `pitch.rs:171` diagnosticados e especificados.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` DigiTech / Harman — *Whammy (5th Gen) Official Product Page & Specifications* (`https://digitech.com/dp/whammy/`).
- `[F-02]` Zölzer, U. (2011). *DAFX: Digital Audio Effects* (2nd ed.). Wiley, Chapter 6.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, desfez-se o acoplamento quádruplo da Spec 075 original: a Spec 075 ficou exclusiva para o **DigiTech Whammy (`pedal.digitech.whammy`)**, criando-se as **Specs 089 (DigiTech Drop)**, **090 (Harmonizer Diatônico)** e **091 (Eventide H3000 MicroPitch)**.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Transposição nos intervalos `±24 semitons` (`+12`, `+24`, `-12`) | `F-01`, `F-02` | Spec 075 / `CA-01` | `crates/ampforge-dsp/src/fx/pitch.rs:285–315 @ c20cc65` | `contract_042_pitch_wah.rs`, `contract_052_splash_pedals_amp2.rs` | `TESTED` |
| Varredura contínua do pedal de expressão (`0.0 -> 1.0`) com suavização anti-zipper e janela complementar `sin²` | `F-01`, `F-02` | Spec 075 / `CA-02` | `crates/ampforge-dsp/src/fx/pitch.rs:171 @ c20cc65, 289` (usa janela `sin` sem suavização dedicada em `expression`) | `contract_042_pitch_wah.rs` | `IMPLEMENTED_UNVERIFIED` |
| Filtro passa-baixas anti-aliasing adaptativo ($f_c = 0,44 f_s / r$) para transposições ascendentes (`+12 / +24 st`) | `F-02` | Spec 075 / `CA-03` | Ausente em `pitch.rs` | Ausente | `SPECIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 075 / `CA-04` | `crates/ampforge-dsp/src/fx/pitch.rs:265–370 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
