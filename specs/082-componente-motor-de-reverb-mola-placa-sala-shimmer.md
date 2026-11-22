---
status: rascunho
---

# Spec 082: Componente Compartilhado — Motor de Reverb Multi-Arquitetura (Spring, Plate, Hall, Chamber e Shimmer)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-shared` (`componente`)
> **Identificador canônico:** `component.reverb.multi_architecture`
> **Variante / Revisão:** `5 Arquiteturas Acústicas/Eletromecânicas Distintas (Spring Dispersive Tank, Plate EMT 140, Hall FDN 8×8/16×16, Chamber, Shimmer Pitch-Loop)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/reverb.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [056](056-amp-fender-twin-reverb-ab763.md), [068](068-rack-roland-re201-space-echo.md)

---

## 1. Objetivo

Especificar o contrato técnico, acústico, matemático e de verificação do **Componente Motor de Reverb Multi-Arquitetura (`component.reverb.multi_architecture`)** em [`crates/ampforge-dsp/src/fx/reverb.rs`](../crates/ampforge-dsp/src/fx/reverb.rs#L1-L337), diagnosticando e corrigindo **três defeitos objetivos de implementação** descobertos durante a auditoria do código atual:
1. **Falsa Diferenciação entre `Plate`, `Hall` e `Chamber` (`reverb.rs:169–175`):** No código atual, `Plate`, `Hall` e `Chamber` executam a **mesma rede de 8 combs Schroeder**, diferindo apenas por um multiplicador escalar de comprimento (`0.95`, `1.15`, `0.80`)! Esta Spec exige 5 arquiteturas acusticamente distintas e justificadas fisicamente (`Spring`, `Plate`, `Hall`, `Chamber`, `Shimmer`).
2. **Damping Dependente da Taxa de Amostragem (`reverb.rs:228`):** A linha `self.damp_state[i] = comb_out * (1.0 - damp) + self.damp_state[i] * damp;` aplica o coeficiente `damp` diretamente sem escalonar pela taxa de amostragem $f_s$ — fazendo com que a frequência de corte de absorção de agudos dobre em `96 kHz` em relação a `48 kHz`!
3. **Divergência no Limite de `predelay_ms` (`reverb.rs:114`):** `reverb.rs:114` limita `predelay_ms` a `150.0 ms`, divergindo do limite de `250.0 ms` especificado na UI (`controls.registry.ts`) e na Spec 032.

---

## 2. Escopo — As 5 Arquiteturas Distintas de Reverb

1. **`Spring` (Tanque Eletromecânico de Mola — Estilo Accutronics 4AB3C1B / Fender 6G15 / AB763):**
   - Filtro passa-banda de transdutor eletromagnético (`120 Hz – 4,8 kHz`) + cadeia dispersiva de **all-pass em cascata de alta ordem** (gerando o *chirp* / *boing / drip* metálico característico de ondas transversais e longitudinais em molas helicoidais, onde altas frequências atrasam mais que médias/graves — Välimäki et al., `F-02`) + modulação sutil de tensão da mola.
2. **`Plate` (Placa Metálica — Estilo EMT 140):**
   - **Ataque instantâneo de altíssima densidade modal** (sem eco discreto inicial perceptível, difusão inicial rápida por rede de 4 all-pass densos) + resposta brilhante e dispersão de ondas de flexão em chapa de aço (`1×2 m`), com `RT60` controlado pelo feltro amortecedor (`Damping`).
3. **`Hall` (Sala de Concertos — FDN Hadamard / Householder `8×8` Estéreo):**
   - Matriz de realimentação ortogonal (*Feedback Delay Network* com matriz Hadamard/Householder para conservação exata de energia e densidade modal crescente sem coloração metálica), com reflexões iniciais esparsas (*Early Reflections* 12–45 ms), modulação lenta das linhas para quebra de ressonâncias estacionárias e absorção atmosférica de agudos escalada por $f_s$.
4. **`Chamber` (Câmara Acústica de Estúdio — Estilo Abbey Road / Capitol):**
   - Volume menor (`RT60` curto/médio `0,4–2,2 s`), reflexões iniciais proeminentes (`5–25 ms`) modelando paredes de concreto/azulejo e média-alta densidade modal com foco nos médios (`300 Hz – 5 kHz`).
5. **`Shimmer` (FDN Hall com Transposição `+12 Semitons` no Loop de Realimentação):**
   - Insere um pitch-shifter polifônico de `+1 oitava` (`+12 st`) com suavização de janela dentro do loop de realimentação da rede FDN (`shimmer_amount`), criando a cascata harmônica ascendente estilo Brian Eno / Daniel Lanois / Strymon BigSky.

---

## 3. Fora de escopo

- Convolução de IRs longas de salas reais gravadas (fora do escopo da Spec 080, que foca em IRs de gabinetes de guitarra).
- Tanque de mola embutido especificamente dentro da unidade Roland RE-201 (que consome uma instância leve da arquitetura `Spring` desta Spec dentro da Spec 068).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `componente` (`dsp-shared` / `multi-architecture-reverb`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (fundamentado em Jot FDN, Dattorro Plate e Välimäki Spring Reverb Modeling) |
| **Modelo de referência** | Multi-Architecture Algorithmic Reverb (`component.reverb.multi_architecture`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 082) |
| **Identificador no sistema** | `component.reverb.multi_architecture` (`crates/ampforge-dsp/src/fx/reverb.rs`) |
| **Papel na cadeia de sinal** | Bloco final `REVERB` pós-gabinete IR e pós-delay (com suporte a `Trails/Spillover`). |

---

## 5. Princípio físico e funcional

1. **Equação de Sabina / Decaimento Exato `RT60` Invariante a `sample_rate`:**
   - Para uma linha de atraso de comprimento $L_i$ amostras (tempo $\tau_i = L_i / f_s$ segundos) numa rede FDN ortogonal, o ganho de realimentação que garante um tempo de reverberação exato `RT60` (tempo para a energia cair `-60 dB`) independentemente da taxa de amostragem $f_s$ é:
     $$g_i = 10^{-\frac{3 \cdot L_i}{f_s \cdot \text{RT}_{60}}}$$
2. **Filtro de Absorção de Agudos (`Damping`) Invariante a `sample_rate`:**
   - Um filtro passa-baixas de 1 polo no loop deve ter seu coeficiente calculado pela frequência física de corte $f_{\text{damp}}(\text{damping}) \in [1200\text{ Hz},\; 16\,000\text{ Hz}]$:
     $$a_{\text{damp}} = \exp\!\left(-\frac{2\pi f_{\text{damp}}}{f_s}\right)$$
     garantindo idêntico escurecimento da cauda em `44,1 kHz`, `48 kHz` e `96 kHz` (corrigindo o erro de `reverb.rs:228`).
3. **Dispersão Metálica do Tanque de Mola (`Spring Drip / Chirp`):**
   - Em uma mola helicoidal de aço (`Accutronics`), a velocidade de grupo varia fortemente com a frequência perto da frequência de transição da mola ($f_t \approx 3,5..4,5\text{ kHz}$), modelada por uma cascata de filtros all-pass dispersivos que estiram um impulso seco em um *chirp* descendente característico (`drip`).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Jean-Marc Jot & Antoine Chaigne — *Digital Delay Networks for Designing Artificial Reverberators* (90th AES Convention, Paris, 1991) | Literatura acadêmica fundadora das redes FDN ortogonais | `ACADEMIC` | Matrizes unitárias Hadamard/Householder, calibração exata de `RT60` por frequência ($g_i = 10^{-3\tau_i/T_{60}}$) e filtros de absorção dependentes de $f_s$. |
| `F-02` | Vesa Välimäki, Julian Parker & Jonathan S. Abel — *Parametric Spring Reverberation Effect* (JAES, Vol. 58, No. 7/8, 2010) | Literatura acadêmica de referência para Spring Reverb | `ACADEMIC` | Cadeia dispersiva de all-pass para modelagem física do *chirp / drip* característico de tanques Accutronics / Fender. |
| `F-03` | Jon Dattorro — *Effect Design, Part 1: Reverberator and Other Filters* (JAES, Vol. 45, No. 9, 1997) | Literatura acadêmica clássica de Plate Reverb | `ACADEMIC` | Topologia de difusão inicial densa (*Input Diffusers* + *Figure-8 Tank* modulado) para Plate Reverb estilo EMT 140 / Lexicon. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Calibração de `RT60` ($g_i = 10^{-3\tau_i/\text{RT}_{60}}$) e coeficiente de damping escalado por $f_s$ em redes FDN ortogonais | `F-01` | AES 1991 | §2–4 | `ACADEMIC` | `CONFIRMED` |
| Cascata dispersiva de all-pass gerando *chirp / drip* em Spring Reverb | `F-02` | JAES 2010 | §3–4 | `ACADEMIC` | `CONFIRMED` |
| Difusão imediata de alta densidade no `Plate` vs reflexões iniciais esparsas em `Hall` e `Chamber` | `F-01`, `F-03` | JAES 1997 / AES 1991 | `F-03` §2 | `ACADEMIC` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Input L/R] ──► [Pre-Delay (0..250 ms)] ──► [Architecture Dispatcher]:
                   ├── Spring:  [Bandpass 120Hz–4.8kHz] ──► [Dispersive All-Pass Chain (Chirp/Drip)] ──► [Dual Spring Tank]
                   ├── Plate:   [4-Stage High-Density Input Diffusers] ──► [Dattorro Figure-8 Modulated Tank]
                   ├── Hall:    [Sparse Early Reflections] ──► [8×8 Hadamard Orthogonal FDN + fs-Scaled Damping]
                   ├── Chamber: [Dense Short Early Reflections (5–25ms)] ──► [Compact 8×8 FDN]
                   └── Shimmer: [8×8 FDN] ◄──► [+12 st Octave Shifter in Feedback Loop]
                                                        │
                                                        ▼
                                            [Dry/Wet Mixer + Spillover/Trails] ──► [Output L/R]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/reverb.rs`](../crates/ampforge-dsp/src/fx/reverb.rs#L1-L337) possui **três defeitos auditados**:

1. **`FACTUAL_ERROR` / `MODEL_COUPLING` (`reverb.rs:169–175`):** `Plate`, `Hall` e `Chamber` não possuem arquiteturas distintas — compartilham o mesmo loop de 8 combs Schroeder diferindo apenas em `model_scale = 0.95 / 1.15 / 0.80`!
2. **`FACTUAL_ERROR` (`reverb.rs:228` — Damping Dependente de `sample_rate`):** O filtro passa-baixas de absorção dentro dos combs usa diretamente `self.damp_state[i] * damp + comb_out * (1.0 - damp)` sem conversão via $\exp(-2\pi f_{\text{damp}} / f_s)$, alterando drasticamente o brilho da cauda entre `44,1 kHz`, `48 kHz` e `96 kHz`!
3. **`ARCHITECTURE_CONFLICT` (`reverb.rs:114`):** `set_predelay_ms` limita o valor a `ms.clamp(0.0, 150.0)`, conflitando com a faixa `0.0 .. 250.0 ms` da Spec 032 e do registro de UI (`controls.registry.ts`).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica (Spec 014) | Faixa Aceita | Unidade | Valor Padrão | Papel |
|---|---|---|---|---|---|
| **BYPASS** | `reverb.enabled` | `false / true` | Booleano | `false` | Liga/desliga a injeção de entrada (com `Trails/Spillover` preservado). |
| **MODEL** | `reverb.model` | `"spring"`, `"plate"`, `"hall"`, `"chamber"`, `"shimmer"` | Enum | `"spring"` | Seleciona uma das 5 arquiteturas distintas. |
| **DECAY (RT60)** | `reverb.decay` | `0.2 .. 12.0` (ou `0.0..1.0` mapeado) | `s` / `%` | `0.45` (`~2,5 s`) | Tempo de reverberação `RT60`. |
| **PRE-DELAY** | `reverb.predelay_ms` | `0.0 .. 250.0` | `ms` | `15.0` | Atraso inicial antes da entrada na câmara/placa/FDN (corrigindo o teto `150 ms` de `reverb.rs:114`). |
| **DAMPING (TONE)** | `reverb.damping` | `0.0 .. 1.0` | `%` | `0.40` | Frequência física de absorção de agudos invariante a `sample_rate`. |
| **MIX** | `reverb.mix` | `0.0 .. 1.0` | `%` | `0.25` | Balanço Dry/Wet. |
| **SHIMMER** | `reverb.shimmer` | `0.0 .. 1.0` | `%` | `0.50` (ativo no modo `shimmer`) | Quantidade de sinal transposto `+12 st` realimentado na FDN. |

---

## 9. Modelo matemático e algoritmo DSP proposto

- **Matriz Hadamard `8×8` Ortogonal (`Hall` / `Chamber` / `Shimmer`):** Computada via Fast Walsh-Hadamard Transform (`24` somas/subtrações, zero multiplicações, escalada por $\frac{1}{\sqrt{8}}$), assegurando conservação de energia unitária ($\|H\|_2 = 1,0$) e crescimento exponencial rápido da densidade de ecos.
- **Dispersão `Spring`:** Cascata de `12` filtros all-pass de 1ª ordem em série com coeficiente $a_{\text{disp}} \approx 0,68$ sintonizados na transição de dispersão da mola (`~3,8 kHz`), antecedidos pelo passa-banda do transdutor (`120 Hz – 4,8 kHz`).
- **Difusão `Plate`:** 4 estágios all-pass de entrada (`Dattorro`, coeficientes `0,75 / 0,75 / 0,625 / 0,625`) alimentando um tanque figura-8 estéreo modulado de ataque imediato.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `1×` em `f32`/`f64` com proteção ativa contra números denormais (*flush-to-zero* / micro-offset $\pm 10^{-20}$) nas caudas silenciosas.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Todos os buffers das linhas de atraso (`Pre-Delay` até `250 ms` + `8` linhas FDN + difusores) pré-alocados integralmente em `prepare()`. Zero alocações em `process_block`.
- WCET $\le 7,5\text{ }\mu\text{s}$ por bloco de `128` amostras estéreo a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Último bloco espacial da cadeia (`REVERB`, após `CAB` e `DELAY`), operando em estéreo verdadeiro L/R.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/reverb/` (separando `spring.rs`, `plate.rs`, `hall_fdn.rs`, `chamber.rs`, `shimmer.rs`).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Regra Verificável |
|---|---|
| Estabilidade BIBO de `RT60` | Com `decay < 1.0`, a energia RMS da cauda decai estritamente de forma monotônica (`E(t2) < E(t1)`). |
| Invariância de `Damping` a `sample_rate` | A diferença espectral da cauda entre `48 kHz` e `96 kHz` deve ser `<= 1,5 dB` em `4 kHz`. |
| Distinção real das 5 arquiteturas | `Spring`, `Plate`, `Hall`, `Chamber` e `Shimmer` apresentam assinaturas temporais/espectrais comprovadamente distintas. |

---

## 15. Estratégia de testes, validação e benchmark

1. Teste de invariância de `Damping` e `RT60` entre `48 kHz` e `96 kHz`.
2. Teste de `Pre-Delay` até `250 ms`.
3. Teste de assinatura dispersiva `Spring` (*chirp*), densidade inicial `Plate` vs `Hall`, e deslocamento espectral `+1 oitava` do `Shimmer`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- No modo `Shimmer`, um filtro passa-baixas suave em `10 kHz` no loop de realimentação impede acúmulo excessivo de brilho estridente quando a oitava superior é realimentada várias vezes.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Invariância de `Damping` e `RT60` entre `48 kHz` e `96 kHz` (`ANALYTICAL`):**
  - *Sinal de entrada:* Impulso unitário (`x[0] = 1.0`), capturando `2,0 s` de cauda em $f_s = 48\,000\text{ Hz}$ e em $f_s = 96\,000\text{ Hz}$.
  - *Configuração e parâmetros:* `reverb.model = "hall"`, `reverb.decay = 0.60`, `reverb.damping = 0.60`, `reverb.predelay_ms = 0.0`, `reverb.mix = 1.0`.
  - *Método de medição:* Centróide espectral da cauda na janela `[0,4 s, 1,2 s]` e taxa de decaimento em dB/s nas duas taxas de amostragem.
  - *Critério e tolerância:* O centróide espectral da cauda em `96 kHz` deve concordar com o de `48 kHz` dentro de **`±15%`**, e o `RT60` estimado dentro de **`±10%`** (corrigindo o bug de `reverb.rs:228`).
  - *Origem da tolerância:* Formulação contínua escalada por $f_s$ de Jot (`F-01`).

- **CA-02 — Faixa Completa de `Pre-Delay` (`0 ms` a `250 ms`) (`REGRESSION`):**
  - *Sinal de entrada:* Impulso unitário (`x[0] = 1.0`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `reverb.predelay_ms = 250.0` (`12 000` amostras a `48 kHz`), `reverb.mix = 1.0`.
  - *Critério e tolerância:* Todas as amostras antes de `n = 11 950` (`249 ms`) devem ter amplitude `<= 1e-6` (silêncio absoluto durante o pre-delay), e a primeira energia da cauda deve surgir após `n = 12 000` (`250 ms`, corrigindo o clamp `150 ms` de `reverb.rs:114`).

- **CA-03 — Distinção Arquitetural Comprovada entre `Spring`, `Plate` e `Hall` (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Impulso unitário (`x[0] = 1.0`) a $f_s = 48\,000\text{ Hz}$.
  - *Configuração e parâmetros:* `decay = 0.50`, `damping = 0.30`, `predelay_ms = 0.0`, `mix = 1.0`.
  - *Critério e tolerância:*
    1. `Plate` deve atingir densidade de ecos inicial nos primeiros `30 ms` **estritamente maior** (menor fator de crista / maior curtose negativa) que `Hall` devido aos 4 difusores de entrada Dattorro (`F-03`);
    2. `Spring` deve limitar a energia acima de `6 kHz` em pelo menos `8 dB` abaixo do `Plate` e apresentar dispersão de fase dependente da frequência (`F-02`).

- **CA-04 — Geração de Harmônicos de `+1 Oitava` no Modo `Shimmer` (`DOCUMENTED` / `REGRESSION`):**
  - *Sinal de entrada:* Burst senoidal de `440 Hz` (`-12 dBFS`) por `200 ms`, seguido de `1,5 s` de silêncio.
  - *Configuração e parâmetros:* `reverb.model = "shimmer"`, `reverb.shimmer = 0.85`, `reverb.decay = 0.75`, `reverb.mix = 1.0`.
  - *Método de medição:* Energia FFT na banda de `+1 oitava` (`860–900 Hz`) na janela `[0,4 s, 1,2 s]` comparada ao modo `hall`.
  - *Critério e tolerância:* A energia em `880 Hz` (`+12 st`) na cauda do modo `shimmer` deve exceder a energia em `880 Hz` no modo `hall` em **pelo menos `+18,0 dB`**.

- **CA-05 — Segurança em Tempo Real e Proteção Contra Denormais (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* Impulso seguido de `20 000` blocos de `128` amostras de silêncio (`0.0`) a `48 kHz`.
  - *Critério e tolerância:* `0` alocações na heap, `100%` das amostras finitas e ausência de degradação de CPU por *subnormal/denormal float slowdown*.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/reverb.rs`
- Testes atuais: `crates/ampforge-dsp/tests/contract_032_time_mod_fx.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Os 3 defeitos de `reverb.rs` (`model_scale` artificial em `L169–175`, `damp` dependente de `sample_rate` em `L228`, e `predelay_ms` limitado a `150 ms` em `L114`) diagnosticados e cobertos por `CA-01` a `CA-03`.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Jot, J.-M., & Chaigne, A. (1991). *Digital Delay Networks for Designing Artificial Reverberators*. AES 90th Convention.
- `[F-02]` Välimäki, V., Parker, J., & Abel, J. S. (2010). *Parametric Spring Reverberation Effect*. JAES, 58(7/8), 547–562.
- `[F-03]` Dattorro, J. (1997). *Effect Design, Part 1: Reverberator and Other Filters*. JAES, 45(9), 660–684.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, expuseram-se os 3 defeitos reais em `reverb.rs` (`L114`, `L169–175`, `L228`) e especificaram-se arquiteturas verdadeiramente distintas (`Spring`, `Plate`, `Hall FDN`, `Chamber`, `Shimmer`) invariantes à taxa de amostragem.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| `Damping` e `RT60` invariantes à taxa de amostragem (`48 kHz` vs `96 kHz`) | `F-01` | Spec 082 / `CA-01` | `crates/ampforge-dsp/src/fx/reverb.rs:228 @ c20cc65` (BUG: `damp` não escala com `sample_rate`) | Ausente | `SPECIFIED` |
| `Pre-Delay` até `250 ms` | Spec 032 | Spec 082 / `CA-02` | `crates/ampforge-dsp/src/fx/reverb.rs:114 @ c20cc65` (BUG: limitado a `150.0 ms`) | Ausente | `SPECIFIED` |
| 5 arquiteturas acusticamente distintas (`Spring` dispersivo, `Plate` Dattorro, `Hall` FDN Hadamard, `Chamber`, `Shimmer`) | `F-01`, `F-02`, `F-03` | Spec 082 / `CA-03` | `crates/ampforge-dsp/src/fx/reverb.rs:169–175 @ c20cc65` (BUG: `Plate/Hall/Chamber` só alteram `model_scale` sobre os mesmos 8 combs) | `contract_032_time_mod_fx.rs` | `SPECIFIED` |
| Modo `Shimmer` com transposição `+12 st` no loop de realimentação | Spec 032 | Spec 082 / `CA-04` | `crates/ampforge-dsp/src/fx/reverb.rs:250–295 @ c20cc65` | `contract_032_time_mod_fx.rs` (`test_ca_05_reverb_shimmer_octave_generation`) | `TESTED` |
| Segurança zero-alloc e proteção contra denormais em tempo real | Spec 053 | Spec 082 / `CA-05` | `crates/ampforge-dsp/src/fx/reverb.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
