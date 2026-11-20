# Spec 079: Componente Compartilhado — Oversampling Polifásico e Anti-Aliasing (2×, 4×, 8×)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-shared` (`componente`)
> **Identificador canônico:** `component.oversampling.polyphase`
> **Variante / Revisão:** `Filtros Half-Band Polifásicos IIR (HIIR / All-Pass Duplo de Mínima Latência) e FIR Fase Linear — Fatores 2×, 4×, 8×`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/oversampling.rs`, `crates/ampforge-dsp/src/precision.rs`
> **Specs relacionadas:** [039](039-amplificadores-fidelidade-e-robustez.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [054](054-amp-marshall-1959-super-lead-plexi.md), [059](059-pedal-ibanez-ts808-tube-screamer.md), [060](060-pedal-klon-centaur.md), [077](077-componente-estagios-valvulados-power-amp-sag-carga-reativa.md)

---

## 1. Objetivo

Especificar o contrato técnico, matemático e de verificação do componente compartilhado de **Oversampling Polifásico e Anti-Aliasing (`component.oversampling.polyphase`)** em [`crates/ampforge-dsp/src/oversampling.rs`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/oversampling.rs#L1-L316), eliminando critérios qualitativos ou incompletos da versão anterior e definindo **especificações completas e mensuráveis de engenharia de filtros** (banda passante, ondulação máxima de passband, banda de transição, atenuação mínima de stopband para IIR e FIR, atraso de grupo máximo, e método rigoroso de medição de aliasing por DFT sob excitação não-linear).

---

## 2. Escopo

1. **Modo Baixa Latência (`PolyphaseIir` — Padrão Tempo Real Ao Vivo):** Cascata de estágios *Half-Band* polifásicos IIR baseados em dois ramos de filtros passa-tudo de 1ª ordem em paralelo ($H(z) = \frac{1}{2}[A_0(z^2) + z^{-1} A_1(z^2)]$, topologia Laurent de Soras / HIIR), garantindo:
   - Banda passante útil: **`20 Hz – 18 000 Hz`** a $f_s = 48\,000\text{ Hz}$ (`20 Hz – 16 500 Hz` a `44,1 kHz`);
   - Ondulação na banda passante (*passband ripple*): **`<= ±0,25 dB`**;
   - Banda de transição: `18 000 Hz – 24 000 Hz` (a `48 kHz`);
   - Atenuação mínima na banda de rejeição (*stopband attenuation*): **`>= 70,0 dB`** (estágio de 12 coeficientes / 6 seções all-pass por ramo) em toda a região de dobramento $[f_s - 18\text{ kHz},\; N \cdot f_s / 2]$;
   - Atraso de grupo na banda passante (`1 kHz`): **`<= 6,0 amostras`** (`<= 0,125 ms` a `48 kHz`) mesmo em `8×` oversampling!
2. **Modo Fase Linear (`PolyphaseFir` — Opcional Render / Mixagem / Caminhos Paralelos):** Filtros *Half-Band* FIR simétricos de fase linear exata com:
   - Atenuação mínima na banda de rejeição (*stopband*): **`>= 96,0 dB`**;
   - Ondulação na banda passante: **`<= ±0,05 dB`**;
   - Latência determinística constante $D_{\text{FIR}}$ declarada via `DspBlock::latency_samples()` (Spec 053).
3. **Regra de Coerência de Fase em Topologias Paralelas (ex.: Klon Centaur — Spec 060):** Proíbe somar um ramo sem oversampling (`0` atraso) com um ramo submetido a `Oversampler8x` IIR; ambos os ramos devem percorrer o mesmo filtro ou receber compensação de atraso/fase exata.

---

## 3. Fora de escopo

- Reamostragem assíncrona de taxa arbitrária (`44,1 kHz ↔ 48 kHz` — *Arbitrary Sample Rate Conversion*).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `componente` (`dsp-shared` / `oversampling-antialiasing`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (baseado na teoria de bancos de filtros Half-Band polifásicos de Ansari, Vaidyanathan e Laurent de Soras / HIIR) |
| **Modelo de referência** | Polyphase Half-Band Oversampler 2× / 4× / 8× (`component.oversampling.polyphase`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 079) |
| **Identificador no sistema** | `component.oversampling.polyphase` (`crates/ampforge-dsp/src/oversampling.rs`) |
| **Papel na cadeia de sinal** | Envoltório obrigatório de todos os processadores não-lineares de alto ganho (Specs `054–064`, `077`). |

---

## 5. Princípio físico e funcional

Quando um sinal senoidal de frequência $f_0$ (ex.: `5000 Hz`, harmônico superior de uma guitarra) atravessa um estágio não-linear de alto ganho (como um `12AX7` ou par de diodos `1N914`), geram-se harmônicos de ordem elevada $k \cdot f_0$ (`10 kHz, 15 kHz, 20 kHz, 25 kHz, 30 kHz, 35 kHz...`). Em taxa nativa $f_s = 48\text{ kHz}$ ($f_{\text{Nyquist}} = 24\text{ kHz}$), todos os harmônicos acima de `24 kHz` dobram-se de volta (*aliasing*) como tons inarmônicos audíveis (`23 kHz, 18 kHz, 13 kHz, 8 kHz...`), criando o som metálico e áspero típico de simuladores digitais baratos.

O oversampling em fator $M \in \{2, 4, 8\}$ eleva a frequência de Nyquist interna para $M \cdot f_s / 2$ (`192 kHz` em `8× @ 48 kHz`), permitindo que os harmônicos até o 38º harmônico de `5 kHz` sejam acomodados sem dobramento antes de serem removidos pelo filtro decimador *Half-Band* antes do retorno a $f_s$.

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | P. P. Vaidyanathan — *Multirate Systems and Filter Banks* (Prentice Hall, 1993) | Literatura acadêmica padrão de filtros polifásicos Half-Band IIR/FIR | `ACADEMIC` | Propriedades matemáticas de decomposição polifásica, filtros all-pass paralelos e supressão de imagens espectrais. |
| `F-02` | Laurent de Soras — *HIIR: Oversampling and Hilbert Transform Library (Half-Band Polyphase IIR Design)* | Referência técnica de projeto de coeficientes elípticos Half-Band em cascata de all-pass de 1ª ordem | `SECONDARY_TECHNICAL` | Cálculo ótimo de coeficientes $a_k$ para atingir `> 70 dB` a `> 96 dB` de stopband com latência mínima. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Decomposição polifásica Half-Band $H(z) = \frac{1}{2}[A_0(z^2) + z^{-1} A_1(z^2)]$ garante resposta plana na banda passante e rejeição simétrica na stopband | `F-01`, `F-02` | Multirate DSP | `F-01` Cap. 4–5 | `ACADEMIC` | `CONFIRMED` |
| Oversampling `8×` com stopband `>= 70 dB` reduz componentes de aliasing inarmônicos abaixo do piso perceptível sob clipping severo | `F-01`, `F-02` | Multirate DSP | `F-01` Cap. 4 | `ACADEMIC` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[x[n] @ fs] ──► [Stage 1: 1×→2× Half-Band] ──► [Stage 2: 2×→4×] ──► [Stage 3: 4×→8×]
                                                                            │
                                                                 [Nonlinear Kernel @ 8×fs]
                                                                            │
[y[n] @ fs] ◄── [Stage 1: 2×→1× Half-Band] ◄── [Stage 2: 4×→2×] ◄── [Stage 3: 8×→4×]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/oversampling.rs`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/oversampling.rs#L1-L316) apresenta os seguintes achados auditados:

1. **`MISSING_VALIDATION` (`oversampling.rs:1–316`):** Os coeficientes dos filtros *Half-Band* IIR existem em `oversampling.rs`, mas não existe teste de contrato formal medindo por DFT a ondulação de banda passante (`20 Hz – 18 kHz <= ±0,25 dB`), a atenuação real de stopband (`>= 70 dB`) e o ganho de rejeição de aliasing sob senoide de `5 kHz` em hard-clipping (`1×` vs `8×`).
2. **`ARCHITECTURE_CONFLICT` (`oversampling.rs`):** Ainda não existe o modo opcional `PolyphaseFir` (`>= 96 dB` stopband, fase linear) para renderização offline ou cenários sensíveis à fase.

---

## 8. Controles, parâmetros e mapeamento para a interface

| Parâmetro | Chave Canônica | Faixa / Opções | Valor Padrão | Papel |
|---|---|---|---|---|
| **OVERSAMPLING FACTOR** | `dsp.oversampling` | `"2x"`, `"4x"`, `"8x"` | `"8x"` | Define o fator $M$ de oversampling para os blocos `DRIVE` e `AMP`. |
| **FILTER MODE** | `dsp.oversampling_mode` *(futuro)* | `"iir_low_latency"`, `"fir_linear_phase"` | `"iir_low_latency"` | Escolhe entre mínima latência ao vivo (`<= 6 amostras`) e fase linear (`>= 96 dB` stopband). |

---

## 9. Modelo matemático e algoritmo DSP proposto

- Cada estágio `2×` IIR implementa dois ramos paralelos de filtros passa-tudo de 1ª ordem em `f64` (ou `f32` em cascata curta):
  - **Estágio 1 (`1× ↔ 2×`, transição estreita `18–24 kHz`):** 12 coeficientes (6 seções por ramo) $\implies$ stopband `>= 72 dB`, ripple `<= 0,10 dB` até `19 kHz`;
  - **Estágio 2 (`2× ↔ 4×`, transição larga `24–72 kHz`):** 4 coeficientes (2 seções por ramo) $\implies$ stopband `>= 80 dB`;
  - **Estágio 3 (`4× ↔ 8×`, transição muito larga `24–168 kHz`):** 2 coeficientes (1 seção por ramo) $\implies$ stopband `>= 85 dB`.
- Essa distribuição assimétrica de ordem (ordem mais alta apenas na taxa mais baixa $2\times$, ordem mínima na taxa mais alta $8\times$) minimiza dramaticamente a CPU mantendo `>= 72 dB` de rejeição total de aliasing!

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Precisão numérica:** Coeficientes e estados dos all-pass em `f64` para evitar degradação da atenuação de stopband acima de `70 dB` por ruído de arredondamento de 24 bits mantissa.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Buffers intermediários de tamanho fixo na stack (`[f64; 8]` por amostra ou `[f64; 1024]` por bloco de 128 amostras); **zero alocações na heap** em `upsample` / `downsample`.
- WCET do par `upsample_8x` + `downsample_8x` para `128` amostras a `48 kHz`: `<= 6,0 µs`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Utilizado internamente por todos os amplificadores (`Specs 054–058`) e pedais de distorção (`Specs 059–064`).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo:** `crates/ampforge-dsp/src/oversampling.rs` (`Oversampler2x`, `Oversampler4x`, `Oversampler8x`, `PolyphaseFilterMode`).

---

## 14. Parâmetros de calibração e invariantes físicas

| Métrica | Modo `PolyphaseIir` (Ao Vivo) | Modo `PolyphaseFir` (Linear Phase) |
|---|---|---|
| Banda passante ($f_s = 48\text{ kHz}$) | `20 Hz – 18 000 Hz` | `20 Hz – 18 500 Hz` |
| Ondulação na banda passante (*Ripple*) | `<= ±0,25 dB` | `<= ±0,05 dB` |
| Atenuação mínima na banda de rejeição (*Stopband*) | `>= 70,0 dB` | `>= 96,0 dB` |
| Atraso de grupo (`1 kHz`, `8×` @ `48 kHz`) | `<= 6,0 amostras` (`<= 0,125 ms`) | Constante e reportado em `latency_samples()` |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Resposta em Frequência Round-Trip (`upsample_8x` $\to$ identidade $\to$ `downsample_8x`):** verifica ripple `<= ±0,25 dB` em `20 Hz – 18 kHz`.
2. **Teste de Atenuação Real de Stopband e Supressão de Aliasing sob Hard-Clipping de `5000 Hz`:** mede via DFT os componentes inarmônicos dobrados abaixo de `5000 Hz` (`1×` vs `8×`).

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- O modo `PolyphaseIir` possui fase não-linear acima de `14 kHz`; por isso, qualquer pedal com caminho paralelo Dry/Wet (como o Klon Centaur, Spec 060) **deve** executar o ramo Dry e o ramo Wet dentro do mesmo domínio `8×` ou aplicar o mesmo filtro *all-pass* de fase equivalente no ramo Dry.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Transparência e Ripple na Banda Passante (`20 Hz – 18 kHz`) (`ANALYTICAL` / `ENGINEERING_TARGET`):**
  - *Sinal de entrada:* Varredura senoidal em 25 frequências entre `20 Hz` e `18 000 Hz` a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `Oversampler8x` (`PolyphaseIir`) em modo *round-trip* (`upsample_8x` $\to$ `y = x` $\to$ `downsample_8x`).
  - *Método de medição:* Magnitude em dB de cada frequência medida por DFT após regime permanente.
  - *Critério e tolerância:* Todas as 25 frequências entre `20 Hz` e `18 000 Hz` devem apresentar ganho de **`0,00 dB ± 0,25 dB`**.
  - *Origem da tolerância:* Especificação de passband do banco polifásico Half-Band (`F-01`, `F-02`).

- **CA-02 — Supressão Efetiva de Aliasing Inarmônico sob Saturação Severa (`5000 Hz` em `tanh(20·x)`) (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoide pura de `5000 Hz` (escolhida para não ser submúltiplo inteiro de `48 000 Hz`, expondo todos os componentes de aliasing em frequências inarmônicas abaixo de `5000 Hz` — ex.: `3 kHz`, `2 kHz`, `1 kHz`!) com amplitude `0 dBFS` (`1,0`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, FFT Blackman-Harris de `16 384` amostras.
  - *Configuração e parâmetros:* Função não-linear severa $y = \tanh(20,0 \cdot x)$ processada a `1×` (sem oversampling) versus dentro de `Oversampler8x` (`PolyphaseIir`).
  - *Método de medição:* Razão sinal-para-aliasing (energia da fundamental `5000 Hz` sobre a maior espúria inarmônica na faixa `100 Hz – 4800 Hz`, onde **nenhum** harmônico legítimo de `5000 Hz` existe!).
  - *Critério e tolerância:* A maior espúria de aliasing na faixa `100 Hz – 4800 Hz` em `Oversampler8x` deve ficar **pelo menos `35,0 dB` abaixo** da espúria medida a `1×` e **pelo menos `65,0 dB` abaixo** da fundamental `5000 Hz`.
  - *Origem da tolerância:* Atenuação de stopband `>= 70 dB` e extensão de Nyquist para `192 kHz` (`F-01`, `F-02`).

- **CA-03 — Latência Ultrabaixa no Modo `PolyphaseIir` (`<= 6,0 amostras`) (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* Impulso unitário (`x[0] = 1.0`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$.
  - *Configuração e parâmetros:* `Oversampler8x` (`PolyphaseIir`) em modo round-trip.
  - *Método de medição:* Índice do pico da resposta ao impulso e atraso de grupo em `1000 Hz`.
  - *Critério e tolerância:* O atraso de grupo em `1000 Hz` deve ser **`<= 6,0 amostras`** (`<= 0,125 ms` a `48 kHz`).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/oversampling.rs`, `crates/ampforge-dsp/src/precision.rs`
- Testes atuais: `crates/ampforge-dsp/tests/contract_039_amps_fidelity.rs`, `crates/ampforge-dsp/tests/contract_044_sdd_integrity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Especificações completas de filtro (passband `20 Hz–18 kHz`, ripple `<= ±0,25 dB`, stopband `>= 70 dB` IIR / `>= 96 dB` FIR, latência `<= 6 amostras`) definidas.
- [x] Método de medição de aliasing por DFT na banda proibida (`100 Hz – 4800 Hz` sob fundamental `5000 Hz`) formalizado em `CA-02`.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Vaidyanathan, P. P. (1993). *Multirate Systems and Filter Banks*. Prentice Hall.
- `[F-02]` de Soras, L. — *HIIR: Half-Band Polyphase IIR Filter Design*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, substituíram-se descrições genéricas de anti-aliasing por uma máscara completa de filtro Half-Band e pelo teste DFT em `5000 Hz` medindo espúrias na faixa livre de harmônicos (`100–4800 Hz`).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Passband `20 Hz – 18 kHz` com ripple `<= ±0,25 dB` | `F-01`, `F-02` | Spec 079 / `CA-01` | `crates/ampforge-dsp/src/oversampling.rs:1–316` | Ausente (sem teste dedicado de ripple DFT) | `IMPLEMENTED_UNVERIFIED` |
| Supressão de aliasing (`>= 65 dB` abaixo da fundamental `5 kHz` em `tanh(20x)`) | `F-01`, `F-02` | Spec 079 / `CA-02` | `crates/ampforge-dsp/src/oversampling.rs` | `contract_039_amps_fidelity.rs` | `IMPLEMENTED_UNVERIFIED` |
| Atraso de grupo `<= 6,0 amostras` em `8×` `PolyphaseIir` | `F-02` | Spec 079 / `CA-03` | `crates/ampforge-dsp/src/oversampling.rs` | Ausente | `IMPLEMENTED_UNVERIFIED` |
| Modo opcional `PolyphaseFir` (`>= 96 dB` stopband, fase linear) | `F-01` | Spec 079 / §2 | Ausente em `oversampling.rs` | Ausente | `SPECIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 079 / `CA-04` | `crates/ampforge-dsp/src/oversampling.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
