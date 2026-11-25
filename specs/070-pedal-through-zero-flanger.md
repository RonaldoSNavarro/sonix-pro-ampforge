---
status: rascunho
---

# Spec 070: Módulo Algorítmico Through-Zero Flanger (Dual-Line Tape-Style Cancellation)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-modulation` (`modulo-dsp`)
> **Identificador canônico:** `module.mod.through_zero_flanger`
> **Variante / Revisão:** `Módulo Algorítmico Nativo AMPFORGE (Linha Seca com Atraso de Referência Fixo τ_ref + Linha Modulada Cruzando τ_mod(t) = τ_ref com Feedback Bipolar)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/modulation.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [052](052-abertura-animada-pedais-faltantes-e-amp2-visivel.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md)

---

## 1. Objetivo

Especificar o contrato técnico, físico-matemático e de verificação do **Módulo Algorítmico Through-Zero Flanger (`module.mod.through_zero_flanger`)**, reclassificando-o honestamente como `tipo: modulo-dsp` e diagnosticando o **erro fundamental de princípio físico** no código atual ([`crates/ampforge-dsp/src/fx/modulation.rs:219–231 @ c20cc65`](../crates/ampforge-dsp/src/fx/modulation.rs#L219-L231)), onde o ramo seco não possui linha de atraso de referência ($\tau_{\text{dry}} = 0$) enquanto o ramo modulado varia apenas no semieixo positivo (`0,1 ms` a `5,1 ms`), tornando matematicamente impossível o cruzamento pelo zero ($\Delta\tau(t) = \tau_{\text{mod}}(t) - \tau_{\text{ref}} = 0$)!

---

## 2. Escopo

1. Distinção técnica e física rigorosa entre as 5 arquiteturas de Flanging:
   - Flanger convencional de linha única;
   - Flanger de linha dupla (*Dual-Line*);
   - *Through-Zero Flanging (TZF)*;
   - Flanging histórico com duas máquinas de fita (*Two-Tape-Machine Reel-to-Reel Flanging*); e
   - Pedais dedicados de Through-Zero Flanger.
2. Implementação da arquitetura **Dual-Line Through-Zero**:
   - **Linha 1 (Referência Seca Atrasada):** Atraso fixo $\tau_{\text{ref}} = 3,0\text{ ms}$;
   - **Linha 2 (Linha Modulada / Servo-Tape):** Atraso variável $\tau_{\text{mod}}(t) \in [0,5\text{ ms},\; 5,5\text{ ms}]$ que **antecipa** ($\tau_{\text{mod}} < \tau_{\text{ref}} \implies \Delta\tau < 0$), **cruza exatamente o ponto zero** ($\tau_{\text{mod}} = \tau_{\text{ref}} \implies \Delta\tau = 0,000\text{ ms}$) e **atrasa** ($\tau_{\text{mod}} > \tau_{\text{ref}} \implies \Delta\tau > 0$) em relação à Linha 1!
3. Realimentação regenerativa (`Feedback` / `Regen`) e inversão de polaridade opcional (*Subtractive Zero-Crossing Cancellation* vs *Additive Zero-Crossing Peak*).
4. Exposição completa do controle `pedal.flanger.feedback` (`mod_1.feedback` / `mod_2.feedback`) na UI e em `controls.registry.ts`.

---

## 3. Fora de escopo

- Emulação circuit-level de um pedal BBD convencional incapaz de Through-Zero (como o Electro-Harmonix Electric Mistress `SAD1024` ou o Boss BF-2 `MN3207` de linha única, onde $\tau_{\text{dry}} = 0$ e $\tau_{\text{BBD}} \ge 0,8\text{ ms}$).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `modulo-dsp` (`modulation` / `through-zero-flanger`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (inspirado na técnica de estúdio de duas máquinas de fita *Reel-to-Reel* de Ken Townsend / George Martin / Eddie Kramer, 1966–1968, e no Foxrox Paradox TZF) |
| **Modelo de referência** | Dual-Delay-Line Through-Zero Tape Flanger (`module.mod.through_zero_flanger`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 070) |
| **Identificador no sistema** | `module.mod.through_zero_flanger` (`ModModel::Flanger` / UI `pedal.flanger.*`) |
| **Papel na cadeia de sinal** | Bloco `MOD` (`mod_1` ou `mod_2`), gerando varreduras comb-filter profundas com inversão completa pelo ponto de atraso relativo zero. |

---

## 5. Princípio físico e funcional: As 5 Arquiteturas de Flanging

Para evitar qualquer confusão conceitual, esta Spec estabelece a distinção exata entre os cinco conceitos:

1. **Flanger Convencional (Linha Única BBD, ex.: Boss BF-2 / MXR M117 / EHX Electric Mistress):**
   - O sinal seco tem atraso $\tau_{\text{dry}} = 0\text{ ms}$. Uma única linha BBD atrasa o sinal molhado por $\tau_{\text{wet}}(t) \in [1,0\text{ ms},\; 10,0\text{ ms}]$.
   - Como um BBD físico não pode ter atraso zero nem negativo, a diferença relativa $\Delta\tau(t) = \tau_{\text{wet}}(t) - 0$ **nunca atinge zero**: a primeira frequência de entalhe (*first comb notch* $f_0 = \frac{1}{2\Delta\tau}$) sobe no máximo até $\frac{1}{2 \cdot 1\text{ ms}} = 500\text{ Hz}$ e volta, sem jamais varrer até o infinito ($f_0 \to \infty$) nem cruzar o ponto de cancelamento total.
2. **Flanging Histórico com Duas Máquinas de Fita (*Two-Tape-Machine Flanging*, Abbey Road / Olympic Studios, 1966–1967):**
   - O mesmo sinal era gravado e reproduzido simultaneamente em **duas máquinas de fita magnética** (`Tape Machine A` e `Tape Machine B`) sincronizadas.
   - O engenheiro pressionava o dedo na borda do carretel (*flange*) da Máquina B (desacelerando-a para trás da Máquina A, $\Delta\tau > 0$) e depois na borda da Máquina A (fazendo a Máquina B **ultrapassar** a Máquina A no tempo, $\Delta\tau < 0$!).
   - No instante exato em que a Máquina B alcançava e ultrapassava a Máquina A ($\tau_A = \tau_B \implies \Delta\tau = 0$), todas as frequências do espectro entravam em fase (ou se cancelavam totalmente em `-∞ dB` quando uma das máquinas estava com polaridade invertida — *Subtractive Flanging*), produzindo o efeito "jato virando do avesso" (*jet plane turning inside-out*).
3. **Dual-Line Flanger:**
   - Qualquer flanger que utiliza duas linhas de atraso em paralelo ($\tau_1$ e $\tau_2$), permitindo seja a modulação em contrafase das duas linhas, seja o atraso da linha de referência.
4. **Through-Zero Flanging (TZF — Princípio Matemático):**
   - Caso específico do Dual-Line Flanger onde a linha de referência seca é atrasada por um tempo fixo $\tau_{\text{ref}} > 0$ (ex.: `3,0 ms`) e a linha modulada varre um intervalo simétrico em torno de $\tau_{\text{ref}}$, por exemplo $\tau_{\text{mod}}(t) \in [\tau_{\text{ref}} - \Delta\tau_{\max},\; \tau_{\text{ref}} + \Delta\tau_{\max}]$.
   - Dessa forma, o atraso relativo causal $\Delta\tau(t) = \tau_{\text{mod}}(t) - \tau_{\text{ref}}$ assume valores **negativos** ($-\Delta\tau_{\max}$), **cruza exatamente o zero** ($\Delta\tau = 0$) e assume valores **positivos** ($+\Delta\tau_{\max}$)!
5. **Pedais Modernos de Through-Zero Flanger (ex.: Foxrox Paradox TZF, Catalinbread Zero Point, Strymon Orbit modo TZF):**
   - Implementam o princípio TZF acima usando duas linhas de atraso (duas cadeias BBD `MN3007` onde uma tem clock fixo e outra clock variável, ou duas linhas DSP de alta resolução) com opção de polaridade subtrativa para cancelamento completo no cruzamento por zero.

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Julius O. Smith III — *Physical Audio Signal Processing (PASP): Flanging and Phasing / Comb Filters* | `https://ccrma.stanford.edu/~jos/pasp/` (HTTP 200 verificado) | `ACADEMIC` | Equações exatas de filtros comb feedforward e feedback, posição dos zeros no plano $z$ em função de $\Delta\tau(t)$ e cancelamento subtrativo em $\Delta\tau = 0$. |
| `F-02` | Udo Zölzer — *DAFX: Digital Audio Effects* (2nd ed., Wiley, 2011), Cap. 2.6 (*Modulated Delay Lines — Flanger & Through-Zero Flanging*) | Literatura acadêmica de referência | `ACADEMIC` | Arquitetura Dual-Line com atraso fixo $\tau_{\text{ref}}$ no ramo Dry e atraso variável $\tau_{\text{mod}}(t)$ cruzando $\tau_{\text{ref}}$. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Para que $\Delta\tau(t) = \tau_{\text{mod}}(t) - \tau_{\text{ref}}$ cruze zero em um sistema causal em tempo real, o ramo Dry deve ser atrasado por $\tau_{\text{ref}} \ge \Delta\tau_{\max}$ | `F-01`, `F-02` | PASP / DAFX | `F-02` Cap. 2 §2.6 | `ACADEMIC` | `CONFIRMED` |
| No instante do cruzamento por zero ($\tau_{\text{mod}} = \tau_{\text{ref}}$), a soma subtrativa $x(t - \tau_{\text{ref}}) - x(t - \tau_{\text{mod}})$ produz cancelamento de banda completa (`0.0`) | `F-01`, `F-02` | PASP / DAFX | `F-01` §"Flanging" | `ACADEMIC` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
                      ┌──► [Reference Delay Line 1: Fixed τ_ref = 3.0 ms] ───────────────────────┐
                      │                                                                          ▼
[Input] ──► (+) ──────┤                                                                   [Subtractive / Additive TZF Mixer] ──► [Output]
             ▲        │                                                                          ▲
             │        └──► [Modulated Delay Line 2: τ_mod(t) = τ_ref + Δτ_max * LFO(t)] ─────────┤
             │                                  │  (sweeps 0.5 ms .. 3.0 ms .. 5.5 ms)           │
             └──────── [× Feedback + Tape LPF] ◄┘                                         (Polarity & Mix)
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/modulation.rs:219–231 @ c20cc65`](../crates/ampforge-dsp/src/fx/modulation.rs#L219-L231) e [`apps/sonix-app/ui/src/controls.registry.ts:909–951`](../apps/sonix-app/ui/src/controls.registry.ts#L909-L951) possui duas falhas graves:

1. **`FACTUAL_ERROR` (`modulation.rs:219–231` — Falso Through-Zero Flanger com $\tau_{\text{dry}} = 0$):**
   - O comentário na linha `219` diz `"// Through-Zero Flanger: 0.1..5ms delay with feedback"`, mas na linha `230` soma `input * (1.0 - self.mix) + wet * self.mix`, onde `input` tem atraso **`0,0 ms`** e `wet` tem atraso `delay_ms = 0.1 + mod_val * self.depth * 5.0` ($\in [0,1\text{ ms},\; 5,1\text{ ms}]$)!
   - Como `delay_ms >= 0.1 ms` e `input` está em `0.0 ms`, o atraso relativo $\Delta\tau = \tau_{\text{wet}} - \tau_{\text{dry}}$ **jamais cruza zero**! Trata-se apenas de um flanger convencional de linha única rotulado erroneamente como Through-Zero.
2. **`ARCHITECTURE_CONFLICT` (`controls.registry.ts:909–951` — Controle `Feedback` Ausente no Pedal Visual de Flanger):**
   - Embora `ModulationProcessor` possua o campo `self.feedback` (`mod_1.feedback` / `mod_2.feedback` aceito em `audio.rs:3651, 3691`), o pedal visual `pedal.flanger.*` em `controls.registry.ts:909–951` expõe apenas `enabled`, `rate`, `depth` e `mix` — faltando `pedal.flanger.feedback`!

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica (`mod_1` / `mod_2`) | UI Control ID (`controls.registry.ts`) | Faixa Aceita | Unidade | Valor Padrão | Papel no Algoritmo TZF |
|---|---|---|---|---|---|---|
| **BYPASS** | `mod_1.enabled` / `mod_2.enabled` | `pedal.flanger.enabled` | `false / true` | Booleano | `false` | Ativa/desativa o processador Through-Zero Flanger. |
| **RATE** | `mod_1.rate_hz` / `mod_2.rate_hz` | `pedal.flanger.rate` | `0.05 .. 10.0` | `Hz` | `0.40` | Frequência de varredura do LFO (triangular/senoidal suavizado). |
| **DEPTH** | `mod_1.depth` / `mod_2.depth` | `pedal.flanger.depth` | `0.0 .. 1.0` | `0 .. 100 %` | `0.70` | Amplitude da excursão $\Delta\tau_{\max} = \text{depth} \cdot 2,5\text{ ms}$ em torno do ponto zero $\tau_{\text{ref}} = 3,0\text{ ms}$. |
| **FEEDBACK (REGEN)** | `mod_1.feedback` / `mod_2.feedback` | `pedal.flanger.feedback` *(a registrar na UI)* | `-0.95 .. +0.95` (UI `0..100%`) | `%` | `0.50` | Realimentação regenerativa da linha modulada com saturação suave de fita (`tanh`). |
| **MIX** | `mod_1.mix` / `mod_2.mix` | `pedal.flanger.mix` | `0.0 .. 1.0` | `0 .. 100 %` | `0.50` | Balanço entre a Linha 1 de referência ($\tau_{\text{ref}} = 3,0\text{ ms}$) e a Linha 2 modulada ($\tau_{\text{mod}}(t)$). Em `mix = 0.50` com polaridade subtrativa, o cruzamento por $\tau_{\text{mod}} = \tau_{\text{ref}}$ produz cancelamento total. |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Duas Leituras na Linha de Atraso (Linha 1 Fixa $\tau_{\text{ref}}$ e Linha 2 Modulada $\tau_{\text{mod}}[n]$):**
   - Define-se o atraso de referência central fixo $\tau_{\text{ref}} = 3,0\text{ ms}$ ($N_{\text{ref}} = 0,003 \cdot f_s = 144\text{ amostras}$ a `48 kHz`).
   - O atraso da linha modulada varia simetricamente para trás e para a frente de $\tau_{\text{ref}}$:
     $$\tau_{\text{mod}}[n] = \tau_{\text{ref}} + \text{depth} \cdot \Delta\tau_{\max} \cdot L(\phi[n]), \qquad \Delta\tau_{\max} = 2,50\text{ ms},\; L(\phi) \in [-1, +1]$$
   - Consequentemente, quando $L(\phi) < 0$, $\tau_{\text{mod}} \in [0,5\text{ ms},\; 3,0\text{ ms})$ (a linha modulada **antecipa** a linha de referência: $\Delta\tau < 0$); quando $L(\phi) = 0$, **$\tau_{\text{mod}} = \tau_{\text{ref}} = 3,0\text{ ms}$ ($\Delta\tau = 0,000\text{ ms}$, cruzamento exato pelo zero!)**; e quando $L(\phi) > 0$, $\tau_{\text{mod}} \in (3,0\text{ ms},\; 5,5\text{ ms}]$ ($\Delta\tau > 0$).
2. **Realimentação Regenerativa e Cancelamento no Ponto Zero:**
   - O sinal gravado no buffer é $w[n] = x[n] + g_{\text{fb}} \cdot \tanh(y_{\text{mod}}[n-1])$, onde $y_{\text{mod}}[n] = \text{read\_cubic}(\tau_{\text{mod}}[n] \cdot f_s)$ e a linha de referência lê o sinal limpo atrasado $x_{\text{ref}}[n] = x[n - N_{\text{ref}}]$.
   - No modo subtrativo de fita (*Subtractive TZF*), quando $\text{mix} = 0,50$ e $\tau_{\text{mod}}[n] = \tau_{\text{ref}}$ (com `feedback = 0`), a saída $y[n] = (1 - \text{mix}) x_{\text{ref}}[n] - \text{mix} \cdot y_{\text{mod}}[n]$ anula-se identicamente ($y[n] = 0,0$).

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Interpolação:** Lagrange cúbica de 4 pontos na Linha 2 modulada, garantindo que no instante $\tau_{\text{mod}}[n] = N_{\text{ref}}$ (fração $= 0,0$) o polinômio cúbico retorne exatamente a amostra $x[n - N_{\text{ref}}]$ sem erro de amplitude ou fase.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Buffers circulares pré-alocados em `prepare()`; zero alocações em `process_block`.
- WCET $\le 2,5\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Pode operar tanto em `mod_1` quanto no slot dedicado de Flanger `mod_2` (Spec 052), antes do amplificador ou pós-pré-amplificador.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/modulation/through_zero_flanger.rs`.
- **Struct principal:** `pub struct ThroughZeroFlangerProcessor` implementando `AudioProcessor` + `ModelDescriptor` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Parâmetro | Valor | Invariante Física |
|---|---|---|
| Atraso de referência $\tau_{\text{ref}}$ | `3,00 ms` (`144` amostras a `48 kHz`) | Garante causalidade para excursões negativas de até $-\Delta\tau_{\max} = -2,50\text{ ms}$. |
| Cruzamento pelo zero ($\Delta\tau = 0$) | Ocorre $2\times$ por ciclo do LFO ($L(\phi) = 0$) | Em $\Delta\tau = 0$, a diferença temporal entre Linha 1 e Linha 2 é exatamente `0,000 ms`. |
| Estabilidade de feedback | $|g_{\text{fb}}| \le 0,95$ + $\tanh$ no loop | Ressonância metálica intensa sem divergência ilimitada. |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Cruzamento Real pelo Ponto Zero ($\Delta\tau < 0 \to \Delta\tau = 0 \to \Delta\tau > 0$):** verifica que a linha modulada antecipa a linha seca na metade negativa do LFO, iguala-se exatamente a $\tau_{\text{ref}}$ no zero do LFO (produzindo cancelamento subtrativo $\ge 40\text{ dB}$ quando `mix = 0.5, feedback = 0.0`), e atrasa-se na metade positiva do LFO.
2. **Teste de Ressonância por Feedback (`feedback = 0.0` vs `feedback = 0.85`):** verifica aguçamento do fator $Q$ dos picos/notches do filtro comb.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- O atraso de referência $\tau_{\text{ref}} = 3,0\text{ ms}$ no ramo seco existe **apenas quando o Through-Zero Flanger está ativo (`enabled = true`)**, pois é uma exigência física da causalidade para permitir $\Delta\tau(t) < 0$. Em bypass (`enabled = false`), a latência do bloco volta a `0` amostras.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Cruzamento Efetivo pelo Zero ($\Delta\tau(t) = \tau_{\text{mod}}(t) - \tau_{\text{ref}}$ mudando de sinal e cancelando em $\Delta\tau = 0$) (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-12 dBFS` (ou ruído branco banda larga) durante um ciclo completo do LFO (`2,0 s`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `module.mod.through_zero_flanger` com `enabled = true`, `rate_hz = 0.5`, `depth = 0.80` ($\Delta\tau \in [-2,0\text{ ms},\; +2,0\text{ ms}]$ em torno de $\tau_{\text{ref}} = 3,0\text{ ms}$), `feedback = 0.0`, `mix = 0.50` (modo subtrativo).
  - *Método de medição:* Medição do atraso relativo $\Delta\tau[n] = \tau_{\text{mod}}[n] - \tau_{\text{ref}}$ e da energia RMS em janelas curtas (`10 ms`) ao longo do ciclo do LFO.
  - *Critério e tolerância:*
    1. O atraso relativo $\Delta\tau[n]$ deve assumir valores **estritamente negativos** ($\le -1,5\text{ ms}$) em um semiciclo e **estritamente positivos** ($\ge +1,5\text{ ms}$) no outro semiciclo.
    2. No instante de cruzamento pelo zero ($\Delta\tau \approx 0$), a atenuação de cancelamento subtrativo (`mix = 0.50, feedback = 0.0`) deve ser **$\ge 30,0\text{ dB`** em relação ao nível máximo do ciclo.
  - *Origem da tolerância:* Definição matemática de Through-Zero Flanging (`F-01`, `F-02` Cap. 2 §2.6).

- **CA-02 — Varredura de Comb-Filtering e Ressonância Regenerativa (`Feedback`) (`ANALYTICAL` / `REGRESSION`):**
  - *Sinal de entrada:* Ruído branco a `-12 dBFS` durante `1,0 s`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `rate_hz = 0.5`, `depth = 0.70`, `mix = 0.50`, comparando `feedback = 0.0` vs `feedback = 0.85`.
  - *Método de medição:* Razão pico-a-vale (*peak-to-valley ratio* em dB) do espectro FFT curto (`2048` amostras) fora do ponto zero ($\Delta\tau = 1,0\text{ ms}$, picos/notches espaçados de `1000 Hz`).
  - *Critério e tolerância:* Com `feedback = 0.85`, a proeminência dos picos ressonantes do filtro comb em relação à média espectral deve aumentar em **pelo menos `+8,0 dB`** em comparação com `feedback = 0.0`, mantendo a saída estritamente estável (`|y[n]| <= 1,5`).
  - *Origem da tolerância:* Ganho de pico de um filtro comb IIR $\frac{1}{1 - g_{\text{fb}}}$ (`F-01`).

- **CA-03 — Paridade de Controles (`rate`, `depth`, `feedback`, `mix`) (`REGRESSION`):**
  - *Configuração e parâmetros:* Verificação de aceitação das chaves `mod_1.feedback` e `mod_2.feedback` no backend e registro do controle `pedal.flanger.feedback` em `controls.registry.ts`.
  - *Critério e tolerância:* Mover `feedback`, `rate`, `depth` ou `mix` de ponta a ponta altera mensuravelmente o sinal de saída (`RMS diff > 1e-3`).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `rate_hz`, `depth`, `feedback`, `mix`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/modulation.rs` (e futuro `crates/ampforge-dsp/src/fx/modulation/through_zero_flanger.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.flanger.*`)
- Testes atuais: `Planejado: crates/ampforge-dsp/tests/contract_032_time_mod_fx.rs`, `crates/sonix-desktop/tests/contract_052_splash_pedals_amp2.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Reclassificado como `tipo: modulo-dsp` (`module.mod.through_zero_flanger`).
- [x] Os 5 conceitos de flanging (convencional, dual-line, TZF, duas máquinas de fita, pedais TZF) explicados e diferenciados na Seção 5.
- [x] Erro de ausência de atraso de referência $\tau_{\text{ref}}$ no ramo Dry em `modulation.rs:219–231` e ausência de `pedal.flanger.feedback` em `controls.registry.ts` diagnosticados.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Smith, J. O. (2010). *Physical Audio Signal Processing: Flanging and Phasing*. CCRMA, Stanford University (`https://ccrma.stanford.edu/~jos/pasp/`).
- `[F-02]` Zölzer, U. (2011). *DAFX: Digital Audio Effects* (2nd ed.). Wiley, Chapter 2 §2.6.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, diagnosticou-se que `ModModel::Flanger` em `modulation.rs:219–231` não possuía atraso $\tau_{\text{ref}}$ na linha seca ($\tau_{\text{dry}} = 0$), impedindo o cruzamento pelo zero. Especificou-se a arquitetura Dual-Line TZF real ($\tau_{\text{ref}} = 3,0\text{ ms}$, $\tau_{\text{mod}} \in [0,5,\; 5,5\text{ ms}]$).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Arquitetura Dual-Line TZF com atraso fixo $\tau_{\text{ref}} = 3,0\text{ ms}$ no ramo Dry e $\tau_{\text{mod}}(t)$ cruzando $\tau_{\text{ref}}$ ($\Delta\tau < 0 \to 0 \to > 0$) | `F-01`, `F-02` §2.6 | Spec 070 / `CA-01` | `crates/ampforge-dsp/src/fx/modulation.rs:219–231 @ c20cc65` (BUG: ramo Dry tem `0 ms` de atraso e ramo Wet varia `0.1..5.1 ms`, nunca cruzando zero) | Ausente | `SPECIFIED` |
| Realimentação regenerativa (`feedback`) com saturação suave no loop | `F-01`, `F-02` | Spec 070 / `CA-02` | `crates/ampforge-dsp/src/fx/modulation.rs:226–229 @ c20cc65` | `contract_032_time_mod_fx.rs` | `TESTED` |
| Exposição do controle `pedal.flanger.feedback` em `controls.registry.ts` | Regra Cortex `0a2f` | Spec 070 / `CA-03` | Aceito no backend (`audio.rs:3651, 3691`), porém ausente em `controls.registry.ts:909–951` | `contract_052_splash_pedals_amp2.rs` | `SPECIFIED` |
| Execução zero-alloc em tempo real | Spec 053 | Spec 070 / `CA-04` | `crates/ampforge-dsp/src/fx/modulation.rs:179–294 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
