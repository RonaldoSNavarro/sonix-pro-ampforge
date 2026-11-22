---
status: rascunho
---

# Spec 086: Módulo Equalizador Paramétrico Digital de 8 Bandas (Studio EQ)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Criado na Rodada Corretiva por Desacoplamento da Spec 067)
> **Domínio:** `dsp-eq` (`modulo-dsp`)
> **Identificador canônico:** `module.eq.parametric_8band`
> **Variante / Revisão:** `Equalizador Paramétrico Digital de Estúdio (8 Bandas Configuráveis: Bell, Low/High Shelf, HPF/LPF, Notch, ±18 dB, Q 0.2..16.0, Precisão f64 + Anti-Cramping de Nyquist)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/eq.rs`, `crates/sonix-desktop/src/audio.rs`
> **Specs relacionadas:** [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [067](067-pedal-mxr-m108-equalizador-grafico-10-bandas.md), [085](085-eq-grafico-5-bandas-mesa-boogie-mark.md)

---

## 1. Objetivo

Especificar o contrato técnico, matemático e de verificação do **Módulo Equalizador Paramétrico Digital de 8 Bandas (`module.eq.parametric_8band`)**, atualmente implementado em `EqProcessor (EqMode::Parametric8Band)` ([`crates/ampforge-dsp/src/fx/eq.rs:1–92 @ c20cc65`](../crates/ampforge-dsp/src/fx/eq.rs#L1-L92)), desacoplando-o definitivamente do pedal analógico **MXR M108** (**[Spec 067](067-pedal-mxr-m108-equalizador-grafico-10-bandas.md)**) e do **Mesa/Boogie 5-Band** (**[Spec 085](085-eq-grafico-5-bandas-mesa-boogie-mark.md)**), e introduzindo a especificação de **compensação de *Nyquist cramping*** (Orfanidis / Massberg) e precisão `f64` para operação cirúrgica de estúdio (`20 Hz – 20 kHz`).

---

## 2. Escopo

1. **8 bandas totalmente paramétricas e independentes** (`b1..b8`), cada uma com:
   - `enabled: bool` (bypass individual por banda);
   - `filter_type`: `Bell (Peaking)`, `LowShelf`, `HighShelf`, `HighPass (12 dB/oct)`, `LowPass (12 dB/oct)`, `Notch`;
   - `freq_hz`: `20,0 Hz .. 20 000,0 Hz` (limitado internamente a `<= 0,485 * fs` para estabilidade absoluta);
   - `gain_db`: `-18,0 dB .. +18,0 dB` (em `Bell`, `LowShelf`, `HighShelf`);
   - `q`: `0,20 .. 16,00` (desde curvas musicais largas até entalhes cirúrgicos de remoção de ressonâncias de gabinete *whistle/fizz*).
2. Precisão interna **`f64` (Transposed Direct Form II)** e **compensação de compressão de largura de banda próxima de Nyquist (*Orfanidis Matched-Nyquist Biquad*)** nas bandas `Bell` acima de `6 kHz`.

---

## 3. Fora de escopo

- Equalizadores gráficos de frequências fixas (MXR M108 → Spec 067; Mesa 5-Band → Spec 085).
- Equalização dinâmica (compressão multibanda) ou EQ de fase linear via FFT.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `modulo-dsp` (`parametric-studio-eq`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (Arquitetura Digital de Estúdio baseada em RBJ Audio EQ Cookbook & Orfanidis) |
| **Modelo de referência** | 8-Band Parametric Studio Equalizer (`module.eq.parametric_8band`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 086) |
| **Identificador no sistema** | `module.eq.parametric_8band` (`EqProcessor (EqMode::Parametric8Band)` em `crates/ampforge-dsp/src/fx/eq.rs:1–92 @ c20cc65`) |
| **Papel na cadeia de sinal** | Bloco utilitário/mixagem de estúdio pós-gabinete IR (ou pré-amp), ideal para limpeza cirúrgica (`HPF 80 Hz`, `LPF 10 kHz`, notch em `4,1 kHz`). |

---

## 5. Princípio físico e funcional

1. **Estabilidade Numérica em Baixas Frequências (`20–60 Hz` em `f64`):** Em `f32` (24 bits de mantissa), um biquad com `f0 = 30 Hz` e `Q = 10` a `96 kHz` coloca polos a uma distância de $\sim 10^{-5}$ do círculo unitário, gerando ruído de quantização e até instabilidade. Em **TDF-II `f64`** (53 bits de mantissa), o erro de quantização cai abaixo de `-240 dB`.
2. **Compensação de *Nyquist Cramping* (Orfanidis, 1997):** A transformada bilinear padrão (`RBJ`) mapeia $f = \infty$ para $f_s / 2$, esmagando o lado direito da campânula de filtros `Bell` em `10–16 kHz` a `44,1/48 kHz`. A formulação de Orfanidis fixa o ganho exato em Nyquist ($G_N = |H_a(j \pi f_s)|$), restaurando a simetria analógica da curva até `18 kHz` sem necessidade de oversampling!

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Robert Bristow-Johnson — *Cookbook Formulae for Audio EQ Biquad Filter Coefficients* | Referência clássica da indústria DSP para `HPF`, `LPF`, `Bell`, `Shelf` e `Notch` | `SECONDARY_TECHNICAL` | Equações canônicas de biquads de áudio. |
| `F-02` | Sophocles J. Orfanidis — *Digital Parametric Equalizer Design with Prescribed Nyquist-Frequency Gain* (JAES, Vol. 45, No. 6, 1997) | Literatura acadêmica de referência para biquads sem *Nyquist cramping* | `ACADEMIC` | Solução fechada de biquad de 2ª ordem que preserva a resposta analógica perto de $f_s / 2$. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Formulação de Orfanidis preserva largura de banda e ganho de Nyquist de filtros `Bell` em `10–16 kHz` sem oversampling | `F-02` | JAES 1997 | §2–3 | `ACADEMIC` | `CONFIRMED` |
| Forma Direta II Transposta em `f64` garante estabilidade e ruído `< -140 dB` em baixas frequências com alto `Q` | `F-01`, `F-02` | DSP Standard | §Biquad Implementation | `ACADEMIC` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Audio In] ──► [Band 1] ──► [Band 2] ──► [Band 3] ──► [Band 4] ──► [Band 5] ──► [Band 6] ──► [Band 7] ──► [Band 8] ──► [Audio Out]
               (Cada banda: f64 TDF-II Biquad — Bell / LowShelf / HighShelf / HPF / LPF / Notch)
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`MODEL_COUPLING` (corrigido nesta revisão):** `EqProcessor (EqMode::Parametric8Band)` (`crates/ampforge-dsp/src/fx/eq.rs:1–92 @ c20cc65`) estava indevidamente misturado na Spec 067 com o pedal MXR M108.
2. **`FACTUAL_ERROR` (`eq.rs:1–92`):** `EqProcessor (EqMode::Parametric8Band)` utiliza coeficientes RBJ simples em `f32` sem proteção contra *Nyquist cramping* (`Orfanidis`) e sem precisão `f64` nos estados de baixa frequência (`20 Hz, Q = 16`).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Parâmetro (para `k = 1..8`) | Chave Canônica | Faixa Aceita | Unidade | Valor Padrão |
|---|---|---|---|---|
| **GLOBAL ENABLED** | `eq8.enabled` | `false / true` | Booleano | `false` |
| **BAND `k` TYPE** | `eq8.b{k}.type` | `"bell"`, `"low_shelf"`, `"high_shelf"`, `"hpf"`, `"lpf"`, `"notch"` | Enum | `"bell"` |
| **BAND `k` FREQ** | `eq8.b{k}.freq_hz` | `20.0 .. 20000.0` | `Hz` | `[80, 160, 350, 750, 1500, 3000, 6000, 12000]` |
| **BAND `k` GAIN** | `eq8.b{k}.gain_db` | `-18.0 .. +18.0` | `dB` | `0.0` |
| **BAND `k` Q** | `eq8.b{k}.q` | `0.20 .. 16.00` | Adimensional | `1.00` |

---

## 9. Modelo matemático e algoritmo DSP proposto

- 8 seções biquad de 2ª ordem em cascata (TDF-II `f64`), com cálculo de coeficientes `Orfanidis` para `Bell` e `RBJ f64` para `HPF`, `LPF`, `LowShelf`, `HighShelf` e `Notch`. Recálculo de coeficientes somente quando algum parâmetro muda (`dirty` flag).

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `1×` (desnecessário graças à formulação analítica *Matched-Nyquist* de Orfanidis).
- **Precisão:** `f64` em todos os coeficientes e variáveis de estado.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`. WCET $\le 2,2\text{ }\mu\text{s}$ por bloco de `128` amostras.

---

## 12. Posição e interação na cadeia completa de guitarra

- Pós-gabinete IR (mixagem de estúdio) ou pré-amplificador (moldagem cirúrgica).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/eq/parametric_8band.rs` (`pub struct ParametricEq8Processor` implementando `DspBlock` + `AutomatableParams`, Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Regra Verificável |
|---|---|
| Transparência absoluta com ganhos `0 dB` | `0,00 dB ± 0,01 dB` de `20 Hz` a `20 kHz`. |
| Estabilidade extrema (`20 Hz`, `Q = 16`, `+18 dB`) | Zero divergência e zero ruído de quantização audível (`f64` TDF-II). |
| Simetria anti-cramping em `12 kHz` | Erro de largura de banda a `48 kHz` `<= 10%` em relação ao protótipo analógico. |

---

## 15. Estratégia de testes, validação e benchmark

1. Teste de estabilidade em canto extremo (`20 Hz, Q = 16, +18 dB` a `96 kHz`).
2. Teste de atenuação de `Notch` (`>= 40 dB` na frequência central), `HPF`/`LPF` (`-12 dB/oitava`) e simetria `Orfanidis` em `12 kHz` a `48 kHz`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- A atualização abrupta de `freq_hz` com `Q = 16` deve usar suavização exponencial de frequência ($\tau = 5\text{ ms}$) para prevenir transientes de estado interno.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Precisão de Ganho, `Notch` Cirúrgico e Estabilidade em `20 Hz / Q = 16` (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoides de teste em `20 Hz`, `1000 Hz` e `4000 Hz` a `-20 dBFS`, $f_s = 48\,000\text{ Hz}$ e $96\,000\text{ Hz}$.
  - *Critério e tolerância:*
    1. `Bell` em `1000 Hz` com `+18,0 dB` e `-18,0 dB`: ganho medido **`±18,0 dB ± 0,1 dB`**;
    2. `Notch` em `4000 Hz` (`Q = 10`): atenuação em `4000 Hz` **`>= 40,0 dB`**;
    3. `Bell` em `20 Hz`, `Q = 16,0`, `+18,0 dB` a `96 kHz`: estável, sem `NaN`/`Inf` e ganho exato **`+18,0 dB ± 0,2 dB`**.

- **CA-02 — Proteção Contra *Nyquist Cramping* (Orfanidis) em `12 kHz` a `48 kHz` (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Varredura senoidal em `6000 Hz`, `12 000 Hz` e `20 000 Hz` a $f_s = 48\,000\text{ Hz}$.
  - *Configuração:* Banda `Bell` centrada em `12 000 Hz`, `gain_db = +12,0 dB`, `Q = 1,0`.
  - *Critério e tolerância:* O ganho medido em `20 000 Hz` deve concordar com o protótipo analógico contínuo $H_a(j 2\pi \cdot 20\,000)$ dentro de **`±0,5 dB`** (diferente de um biquad RBJ padrão, cujo ganho colapsa artificialmente para `0 dB` em Nyquist `24 kHz`).
  - *Origem da tolerância:* Formulação exata de Orfanidis (`F-02`).

- **CA-03 — Inclinação de `HPF` e `LPF` (`12 dB/oitava`) (`ANALYTICAL`):**
  - *Configuração:* `HPF` em `200 Hz` ($Q = 0,7071$) e `LPF` em `4000 Hz` ($Q = 0,7071$).
  - *Critério e tolerância:* Atenuação de **`-3,01 dB ± 0,1 dB`** na frequência de corte e **`-12,0 dB ± 0,5 dB`** uma oitava além do corte (`100 Hz` no HPF, `8000 Hz` no LPF).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Critério e tolerância:* `0` alocações na heap em `10 000` blocos de `128` amostras com automação de `freq_hz`, `gain_db` e `q`.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/eq.rs` (`EqProcessor (EqMode::Parametric8Band)` e futuro `crates/ampforge-dsp/src/fx/eq/parametric_8band.rs`)

---

## 19. Critérios de conclusão da Spec

- [x] Desacoplado integralmente da Spec 067 (MXR M108) e da Spec 085 (Mesa 5-Band).
- [x] Exigência de `f64` TDF-II e anti-cramping de Orfanidis (`F-02`) especificada em `CA-01` e `CA-02`.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Bristow-Johnson, R. — *Audio EQ Cookbook*.
- `[F-02]` Orfanidis, S. J. (1997). *Digital Parametric Equalizer Design with Prescribed Nyquist-Frequency Gain*. JAES, 45(6), 444–455.

---

## 21. Histórico de decisões

- **2026-11-20:** Criada a Spec 086 na rodada corretiva para isolar o `EqProcessor (EqMode::Parametric8Band)` paramétrico digital da Spec 067 (MXR M108).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| 8 bandas paramétricas (`Bell`, `Shelf`, `HPF`, `LPF`, `Notch`, `±18 dB`, `Q 0.2..16`) em `f64` | `F-01` | Spec 086 / `CA-01`, `CA-03` | `crates/ampforge-dsp/src/fx/eq.rs:1–92 @ c20cc65` (atualmente em `f32`) | `contract_040_pedals_parity.rs` | `IMPLEMENTED_UNVERIFIED` |
| Compensação de *Nyquist cramping* (Orfanidis) em `Bell` de alta frequência | `F-02` | Spec 086 / `CA-02` | Ausente em `eq.rs` | Ausente | `SPECIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 086 / `CA-04` | `crates/ampforge-dsp/src/fx/eq.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
