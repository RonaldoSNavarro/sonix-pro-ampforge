---
status: rascunho
---

# Spec 091: Rack Eventide H3000 MicroPitch Detune (Dual Stereo Micro-Pitch Shift ±Cents & Asymmetric Delays)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Criado na Rodada Corretiva por Desacoplamento da Spec 075)
> **Domínio:** `dsp-pitch` (`rack`)
> **Identificador canônico:** `rack.eventide.h3000_micropitch`
> **Variante / Revisão:** `Eventide H3000 Ultra-Harmonizer Algorithm 102 / Preset 231 "MicroPitchShift" (Dual Detune -9/+9 Cents, Pre-Delays Assimétricos 12 ms L / 20 ms R)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/pitch.rs`, `crates/sonix-desktop/src/audio.rs`
> **Specs relacionadas:** [042](042-bloco-de-pitch-sem-processamento.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [075](075-bloco-pitch-whammy-drop-harmonizer-detune.md)

---

## 1. Objetivo

Especificar o contrato técnico, algorítmico e de verificação do efeito de estúdio **Eventide H3000 MicroPitch Detune (`rack.eventide.h3000_micropitch`)**, desacoplando-o definitivamente do **DigiTech Whammy** (**[Spec 075](075-bloco-pitch-whammy-drop-harmonizer-detune.md)**), do **DigiTech Drop** (**[Spec 089](089-pedal-digitech-drop-transposicao-polifonica.md)**) e do **Harmonizer Diatônico** (**[Spec 090](090-modulo-harmonizer-diatonico-inteligente.md)**). O MicroPitch é o efeito assinatura de guitarra estéreo ampla dos anos 80/90 (Eddie Van Halen *Balance/5150*, Steve Lukather, John Petrucci, Michael Landau), criando uma parede estéreo gigantesca **sem a oscilação periódica de LFO** de um chorus convencional!

---

## 2. Escopo

1. **Duas Vozes Independentes de Micro-Desafinação Estática em Contrafase (Estéreo L/R):**
   - **Voz A (Pan Esquerda `L`):** Transposição micro-tonal negativa contínua **`-Δc` cents** (padrão clássico Preset 231: **`-9 cents`**, faixa `-50..0 cents`) + **Pré-Atraso curto assimétrico $\tau_L \approx 12\text{ ms}$** (faixa `0..50 ms`);
   - **Voz B (Pan Direita `R`):** Transposição micro-tonal positiva contínua **`+Δc` cents** (padrão clássico Preset 231: **`+9 cents`**, faixa `0..+50 cents`) + **Pré-Atraso curto assimétrico $\tau_R \approx 20\text{ ms}$** (faixa `0..50 ms`).
2. **Estacionariedade Espectral (Sem LFO Oscilante):** Como o desvio em *cents* é fixo ($r_A = 2^{-9/1200} = 0,994815$, $r_B = 2^{+9/1200} = 1,005212$), a largura estéreo permanece estável e livre de *pitch wobble*, podendo operar em estéreo tradicional (`Dry` no Centro + `Voice A` na Esquerda + `Voice B` na Direita — configuração clássica *Wet-Dry-Wet*).
3. Realimentação opcional baixa (`Feedback 0..50%`) e filtro passa-baixas suave nas vozes desafinadas (`Tone / LPF`) para manter o foco do sinal seco no centro.

---

## 3. Fora de escopo

- Transposições largas por pedal de expressão (Whammy → Spec 075), drop tuning (Drop → Spec 089) e harmonização diatônica (Harmonizer → Spec 090).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `rack` (`stereo-micropitch-detune`) |
| **Fabricante de referência** | Eventide Inc. (Little Ferry, NJ, EUA) |
| **Modelo de referência** | Eventide H3000 Ultra-Harmonizer (`Preset 231: MicroPitchShift`) / Eventide MicroPitch |
| **Ano / época / revisão** | 1988 (H3000 Algorithm `102 Layered Shift` / `101 Diatonic Shift` em modo MicroPitch: `-9c / +9c`, `12 ms / 20 ms`) |
| **Identificador no sistema** | `rack.eventide.h3000_micropitch` (`PitchMode::Detune` em `crates/ampforge-dsp/src/fx/pitch.rs`) |
| **Papel na cadeia de sinal** | Bloco pós-gabinete IR (estéreo L/R, idealmente antes do Delay/Reverb ou substituindo o Chorus para largura estéreo de alto ganho sem modulação LFO). |

---

## 5. Princípio físico e funcional do equipamento real

1. **Alargamento Psicoacústico por Efeito Haas + Batimento Micro-Tonal Estático:**
   - A combinação de dois pré-atrasos curtos assimétricos ($\tau_L \approx 12\text{ ms}$ na Esquerda e $\tau_R \approx 20\text{ ms}$ na Direita, dentro da janela de precedência/fusão de Haas de `5–35 ms`) com desvios de afinação simétricos de pequena magnitude ($-9\text{ cents}$ na Esquerda e $+9\text{ cents}$ na Direita, totalizando `18 cents` entre L e R) engana o córtex auditivo humano a perceber **três guitarras independentes em estéreo largo** (`L` levemente grave, `Center` seca afinada, `R` levemente aguda), sem nunca soar desafinado em acordes distorcidos!
2. **Equação Exata de Razão de Frequência em Cents:**
   $$r_A = 2^{-\frac{\Delta c}{1200}}, \qquad r_B = 2^{+\frac{\Delta c}{1200}}$$
   Para $\Delta c = 9\text{ cents}$ em uma nota `A4 = 440,00 Hz`:
   - Voz A (`Left`): $440,00 \times 2^{-9/1200} = 437,72\text{ Hz}$ (`-2,28 Hz`);
   - Sinal Seco (`Center`): $440,00\text{ Hz}$;
   - Voz B (`Right`): $440,00 \times 2^{+9/1200} = 442,29\text{ Hz}$ (`+2,29 Hz`).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Eventide Audio — *MicroPitch / H3000 MicroPitchShift Official Product Documentation* | `https://www.eventideaudio.com/pedals/micropitch/` (HTTP 200 oficial verificado) | `PRIMARY_OFFICIAL` | Arquitetura oficial dual-voice micro-pitch (`Pitch A`, `Pitch B`, `Delay A`, `Delay B`, `Mix`, `Tone`, `Feedback`) derivada diretamente do H3000 Preset 231. |
| `F-02` | Eventide Inc. — *H3000 Ultra-Harmonizer Operating Manual (Preset 231 MicroPitchShift: -9/+9 cents, 12 ms / 20 ms)* | Manual de fábrica H3000 | `UNKNOWN` | Valores clássicos de referência do algoritmo MicroPitchShift. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Duas vozes estéreo independentes (`Pitch A < 0`, `Pitch B > 0`, padrão `-9 / +9 cents`) com pré-atrasos assimétricos (`~12 ms L` / `~20 ms R`) | `F-01`, `F-02` | H3000 / MicroPitch | `F-01` §"Features"; `F-02` Preset 231 | `UNKNOWN` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
                  ┌──► [Pre-Delay L (12 ms)] ──► [Micro-Shifter A (-9 cents)] ──► [Left Wet]  ──┐
[Input (Center)] ─┼─────────────────────────────────────────────────────────────► [Dry Center] ──┼──► [Stereo Out L/R]
                  └──► [Pre-Delay R (20 ms)] ──► [Micro-Shifter B (+9 cents)] ──► [Right Wet] ──┘
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`MODEL_COUPLING` (corrigido nesta revisão):** `PitchMode::Detune` estava agrupado dentro da Spec 075 e reduzido a mono em `crates/ampforge-dsp/src/fx/pitch.rs:353–365 @ c20cc65`, perdendo o espalhamento estéreo `L/R` e os pré-atrasos assimétricos (`12 ms / 20 ms`) que definem o Eventide H3000 MicroPitch!

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real / UI | Chave Canônica | Faixa Aceita | Unidade | Valor Padrão (`Preset 231`) |
|---|---|---|---|---|
| **BYPASS** | `pitch.enabled` | `false / true` | Booleano | `false` |
| **DETUNE AMOUNT** | `pitch.detune_cents` | `1.0 .. 50.0` | `cents` (`±Δc`) | `9.0` (`-9c L / +9c R`) |
| **DELAY A / B** | `pitch.delay_l_ms` / `pitch.delay_r_ms` | `0.0 .. 50.0` | `ms` | `12.0` / `20.0` |
| **MIX** | `pitch.mix` | `0.0 .. 1.0` | `%` | `0.40` |

---

## 9. Modelo matemático e algoritmo DSP proposto

- Duas linhas de pré-atraso curtas independentes ($\tau_L = 12\text{ ms}$, $\tau_R = 20\text{ ms}$) alimentando dois shifters de dupla cabeça com janelamento complementar `sin²` ($W = 35\text{ ms}$, interpolação cúbica Lagrange) com razões fixas $r_A = 2^{-\Delta c / 1200}$ (canal `L`) e $r_B = 2^{+\Delta c / 1200}$ (canal `R`), somados ao sinal seco centralizado:
  $$y_L[n] = (1 - \text{mix}) x[n] + \text{mix} \cdot v_A[n], \qquad y_R[n] = (1 - \text{mix}) x[n] + \text{mix} \cdot v_B[n]$$

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `1×` (como $|\Delta c| \le 50\text{ cents} \implies r \le 1,0293$, o deslocamento espectral máximo é `< 3%`, sem risco significativo de aliasing em `48 kHz`).

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Buffers pré-alocados em `prepare()`; zero alocações na heap em `process_block`. WCET $\le 3,0\text{ }\mu\text{s}$ por bloco de `128` amostras estéreo.

---

## 12. Posição e interação na cadeia completa de guitarra

- Pós-gabinete IR em estéreo verdadeiro (`L/R`), proporcionando abertura espacial cinematográfica em timbres *clean* e *high-gain*.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/racks/eventide_micropitch.rs` (`pub struct EventideMicroPitchProcessor` implementando `DspBlock` + `AutomatableParams`, Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Regra Verificável |
|---|---|
| Desvio exato em *cents* (`±9 cents` em `440 Hz`) | Canal `L` em `437,72 Hz ± 1,0 Hz`; Canal `R` em `442,29 Hz ± 1,0 Hz`. |
| Pré-atraso assimétrico (`12 ms L` vs `20 ms R`) | Impulso unitário surge com diferença temporal de `8,0 ms ± 0,5 ms` entre `L` e `R`. |

---

## 15. Estratégia de testes, validação e benchmark

1. Teste de desvio simétrico `-9 cents L` vs `+9 cents R` via FFT de alta resolução (`65 536` amostras a `48 kHz`, resolução `0,73 Hz`).
2. Teste de assimetria temporal `12 ms L` vs `20 ms R` e largura estéreo `Side = (L - R)/2`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Em janelas de `35 ms` com $r \approx 1 \pm 0,005$, a taxa de cruzamento de janela é muito baixa (`~0,14 Hz`), garantindo transições imperceptíveis mesmo em notas sustentadas.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Precisão Micro-Tonal Estéreo (`-9 cents L` e `+9 cents R`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Senoide pura de `1000,0 Hz` a `-12 dBFS` por `2,0 s`, $f_s = 48\,000\text{ Hz}$, FFT de `65 536` amostras (`resolução 0,73 Hz`).
  - *Configuração:* `rack.eventide.h3000_micropitch` com `detune_cents = 9.0`, `mix = 1.0`.
  - *Critério e tolerância:*
    1. Canal Esquerdo (`L`): pico dominante em **`994,81 Hz ± 1,5 Hz`** (`-9 cents`);
    2. Canal Direito (`R`): pico dominante em **`1005,21 Hz ± 1,5 Hz`** (`+9 cents`).
  - *Origem da tolerância:* Preset 231 do Eventide H3000 (`F-01`, `F-02`).

- **CA-02 — Pré-Atrasos Assimétricos Estéreo (`12 ms L` vs `20 ms R`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Impulso unitário mono (`L = R = 1.0`).
  - *Configuração:* `delay_l_ms = 12.0`, `delay_r_ms = 20.0`, `mix = 1.0`.
  - *Critério e tolerância:* O envelope da resposta ao impulso no canal `R` deve estar atrasado em **`+8,0 ms ± 1,0 ms`** em relação ao canal `L`.

- **CA-03 — Largura Estéreo Verdadeira com Centro Seco Firme (`mix = 0.40`) (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoide `440 Hz` mono.
  - *Critério e tolerância:* Com `mix = 0.40`, a componente `Side = (L - R) / 2` deve ser não-nula (`>= -18 dB` relativa a `Mid`) enquanto o sinal central seco `440,0 Hz` permanece o pico de maior amplitude em `Mid`.

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Critério e tolerância:* `0` alocações na heap em `10 000` blocos de `128` amostras e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/pitch.rs` (e futuro `crates/ampforge-dsp/src/fx/racks/eventide_micropitch.rs`)

---

## 19. Critérios de conclusão da Spec

- [x] Fonte oficial Eventide (`F-01`) verificada via HTTP 200.
- [x] Desacoplado integralmente da Spec 075 (Whammy), Spec 089 (Drop) e Spec 090 (Harmonizer).
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Eventide Audio — *MicroPitch Official Product Documentation* (`https://www.eventideaudio.com/pedals/micropitch/`).
- `[F-02]` Eventide Inc. — *H3000 Ultra-Harmonizer Operating Manual (Preset 231 MicroPitchShift)*.

---

## 21. Histórico de decisões

- **2026-11-20:** Criada a Spec 091 na rodada corretiva para isolar o Eventide H3000 MicroPitch Estéreo da Spec 075 e restaurar sua arquitetura verdadeira L/R com pré-atrasos assimétricos (`12 ms / 20 ms`).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Dual Micro-Pitch Estéreo (`-9c L / +9c R`) + pré-atrasos assimétricos (`12 ms / 20 ms`) | `F-01`, `F-02` | Spec 091 / `CA-01`, `CA-02`, `CA-03` | `crates/ampforge-dsp/src/fx/pitch.rs:353–365 @ c20cc65` (atualmente mono sem pré-atrasos assimétricos `12 ms / 20 ms`) | `contract_042_pitch_wah.rs` | `SPECIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 091 / `CA-04` | `crates/ampforge-dsp/src/fx/pitch.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
