---
status: rascunho
---

# Spec 076: Componente de Entrada — Condicionamento de Sinal, Simulação RLC de Captadores e Impedância Virtual

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-shared` (`componente`)
> **Identificador canônico:** `component.input.conditioning_and_pickup_rlc`
> **Variante / Revisão:** `Condicionamento de Entrada (DC Blocker 15 Hz, Trim/Pad -12 dB, Modelagem Matemática RLC de Captador/Cabo/Carga Virtual)`
> **Crates / Módulos afetados:** `crates/sonix-desktop/src/audio.rs`, `crates/ampforge-dsp/src/lib.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [039](039-amplificadores-fidelidade-e-robustez.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [054](054-amp-marshall-1959-super-lead-plexi.md), [063](063-pedal-dallas-arbiter-fuzz-face-ge.md)

---

## 1. Objetivo

Especificar o contrato técnico, físico, matemático e de verificação do **Componente de Entrada (`component.input.conditioning_and_pickup_rlc`)**, estabelecendo com rigor físico a fronteira entre o que o hardware da interface de áudio faz e o que o DSP pode modelar:
1. **Fronteira Física Inegociável (Hardware vs DSP):** Nenhum software DSP pode alterar fisicamente a impedância elétrica real ($Z_{\text{HW}}$) do jack `Hi-Z` da interface de áudio USB do usuário! O que o módulo faz é aplicar um **modelo matemático de filtro RLC equivalente** que simula o carregamento resistivo/capacitivo de diferentes impedâncias de entrada virtuais (`22 kΩ`, `100 kΩ`, `470 kΩ`, `1 MΩ`) e capacitâncias de cabo (`100 pF` a `1000 pF`) sobre uma guitarra ligada em uma interface Hi-Z padrão (`1 MΩ`), além de permitir *voicing* de compensação de captadores (`SingleCoil`, `Humbucker`, `P90`, `Active`).
2. **Condicionamento Obrigatório de Entrada:** Bloqueio DC (`15 Hz`, 1ª ordem), atenuador de entrada `Pad` (`-12 dB` / `0 dB`), ajuste de `Input Gain` (`-24 dB` a `+24 dB`), proteção contra amostras não-finitas (`NaN`/`Inf` sanitização) e medição de pico/RMS de entrada (*Input Meter*) para calibração do headroom nominal (`-18 dBFS = 0 dBu / +0 dBV` referência de entrada de amplificador).

---

## 2. Escopo

1. Sanitização de entrada (`NaN`/`Inf` $\to 0,0$) e **DC Blocker** de 1ª ordem em `15 Hz` ($y[n] = x[n] - x[n-1] + R \cdot y[n-1]$).
2. Controle de **`Input Gain`** (`-24 dB` a `+24 dB`) e chave **`Input Pad`** (`0 dB` / `-12 dB` para captadores ativos de alta saída como EMG 81 ou Humbuckers de alta impedância).
3. **Modelo Matemático de Carregamento RLC Virtual (*Virtual Pickup/Cable/Input Impedance Loading*):**
   - Simulação analítica do circuito RLC equivalente formado pela indutância e resistência DC do captador ($L_p, R_p$), capacitância do cabo ($C_{\text{cable}}$) e resistência de carga virtual ($R_{\text{load}} \in \{10\text{ k}\Omega\text{ (Fuzz Face)},\; 68\text{ k}\Omega\text{ (Low Input)},\; 1\text{ M}\Omega\text{ (Hi-Z)}\}$):
     $$H_{\text{RLC}}(s) = \frac{1}{L_p C_{\text{cable}} s^2 + \left(R_p C_{\text{cable}} + \frac{L_p}{R_{\text{load}}}\right) s + \left(1 + \frac{R_p}{R_{\text{load}}}\right)}$$
4. **Calibração de Nível de Referência (*Headroom Calibration*):** Padronização da conversão entre sinal digital em `dBFS` da interface de áudio e tensão elétrica equivalente em `Volts` na grade da válvula `V1` (`0 dBFS ≡ V_ref_pk`, padrão calibrado em `1,0 FS = +12 dBu` ou `1,0 V_rms` em `-12 dBFS`).

---

## 3. Fora de escopo

- Alteração elétrica real da impedância física de hardware do conversor ADC da placa de som (fisicamente impossível via software, salvo interfaces específicas com relé Hi-Z controlado por driver proprietário externo).
- Afinador YIN (coberto pela **[Spec 083](083-componente-afinador-yin-e-looper-de-performance.md)**) e Noise Gate (coberto pela **[Spec 065](065-pedal-noise-gate-histerese-sidechain.md)**).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `componente` (`dsp-shared` / `input-conditioning`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (baseado na teoria eletromagnética de captadores de guitarra de Helmuth Lemme & Donald Tillman) |
| **Modelo de referência** | Input Conditioning & Virtual RLC Pickup/Cable Loader (`component.input.conditioning_and_pickup_rlc`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 076) |
| **Identificador no sistema** | `component.input.conditioning_and_pickup_rlc` (`input.*` em `crates/sonix-desktop/src/audio.rs`) |
| **Papel na cadeia de sinal** | Estágio `0` (primeiro processamento após a captura do callback de áudio CPAL/ASIO, antes do Afinador YIN, Gate, Wah, Pitch e Amplificadores). |

---

## 5. Princípio físico e funcional do circuito real

1. **Circuito Equivalente de um Captador Magnético Passivo de Guitarra:**
   - Um captador magnético passivo comporta-se como uma fonte de tensão $V_{\text{ind}}(t)$ em série com uma grande indutância ($L_p \approx 2,2\text{ H}$ para um *Single-Coil* Stratocaster; $L_p \approx 4,5..8,0\text{ H}$ para um *Humbucker* PAF/High-Output) e resistência DC do enrolamento ($R_p \approx 6\text{ k}\Omega$ a $16\text{ k}\Omega$), em paralelo com a capacitância parasita da bobina + cabo coaxial ($C_{\text{tot}} \approx 400..800\text{ pF}$).
   - Essa rede RLC de 2ª ordem produz um **pico ressonante** característico em:
     $$f_0 = \frac{1}{2\pi \sqrt{L_p C_{\text{tot}}}} \in [2,0\text{ kHz}\text{ (Humbucker)},\; 4,5\text{ kHz}\text{ (Single-Coil)}]$$
     com amortecimento (fator $Q$) governado pela resistência de carga $R_{\text{load}}$ vista pelo captador (potenciômetros de volume/tom da guitarra em paralelo com a impedância de entrada do primeiro equipamento!).
2. **Efeito do Carregamento por Impedância Baixa (ex.: Fuzz Face `~10 kΩ` ou Entrada `LOW` `68 kΩ`):**
   - Quando $R_{\text{load}}$ cai de `1 MΩ` (entrada `Hi-Z` / `HIGH` valvulada) para `68 kΩ` ou `10 kΩ` (Arbiter Fuzz Face, Spec 063), o termo de amortecimento $\frac{L_p}{R_{\text{load}}}$ cresce drasticamente: o pico ressonante de `3,5–4,5 kHz` é **amortecido** (*damped*) e o nível de sinal cai em banda larga, suavizando os agudos antes mesmo do clipping.
   - **Nota física rigorosa:** Como a guitarra real do usuário já passou pelo seu próprio captador RLC carregado pela entrada `Hi-Z` de `1 MΩ` da placa de som, o simulador de carregamento virtual em DSP aplica a **função de transferência diferencial** $\Delta H(s) = \frac{H_{\text{RLC}}(s, R_{\text{target}}, C_{\text{target}})}{H_{\text{RLC}}(s, R_{\text{ref}}=1\text{ M}\Omega, C_{\text{ref}})}$ (um filtro biquad realizável de fase mínima e ganho $\le 1$) — nunca aplicando um segundo pico RLC duplicado quando o usuário já está em modo `1 MΩ` plano (`Bypass` do carregador)!

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Helmuth Lemme — *The Secrets of Electric Guitar Pickups* (Elektor / Guitar Electronics Technical Reference) | Literatura técnica clássica de engenharia elétrica de captadores magnéticos | `SECONDARY_TECHNICAL` | Circuito equivalente RLC de captadores Single-Coil (`2,2 H / 6 kΩ`), P90 (`6,5 H / 8 kΩ`) e Humbucker (`4,5–8 H / 8–16 kΩ`), capacitância de cabo e amortecimento por $R_{\text{load}}$. |
| `F-02` | Julius O. Smith III — *Introduction to Digital Filters: DC Blocker* (CCRMA, Stanford) | `https://ccrma.stanford.edu/~jos/pasp/` (HTTP 200 verificado) | `ACADEMIC` | Equação exata do filtro bloqueador DC de 1ª ordem $H(z) = \frac{1 - z^{-1}}{1 - R z^{-1}}$ com polo em $R = 1 - \frac{2\pi f_c}{f_s}$. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Circuito equivalente RLC de 2ª ordem de captador magnético ($L_p, R_p, C_{\text{cable}}, R_{\text{load}}$) e queda do fator $Q$ ao reduzir $R_{\text{load}}$ | `F-01` | Guitar Pickup Engineering | §"Electrical Model of a Pickup" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Bloqueador DC de 1ª ordem ($f_c \approx 15\text{ Hz}$) removendo offset de ADC antes de estágios não-lineares | `F-02` | CCRMA | §"DC Blocker" | `ACADEMIC` | `CONFIRMED` |
| Impossibilidade física de alterar $Z_{\text{HW}}$ real do jack ADC apenas por software (modelagem via filtro diferencial $\Delta H(s)$) | Princípio físico de circuitos | N/A | §1, §5 | `ANALYTICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[ADC Input (CPAL/ASIO)] ──► [NaN/Inf Sanitizer] ──► [DC Blocker (15 Hz 1st-Order)]
                                                            │
                                                            ▼
                                               [Input Pad (0 / -12 dB) + Input Trim (±24 dB)]
                                                            │
                                                            ▼
                                               [Virtual Load / Cable RLC Differential Filter ΔH(s)]
                                               (Transparent 0 dB when Virtual Z = 1 MΩ & Cable = 0 pF)
                                                            │
                                                            ▼
                                               [Input Peak/RMS Telemetry Meter] ──► [Tuner / Gate / Chain]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/sonix-desktop/src/audio.rs:4558–4602 @ c20cc65`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/sonix-desktop/src/audio.rs#L4558-L4602) (parâmetros `input.*`) e [`audio.rs:1456–1515`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/sonix-desktop/src/audio.rs#L1456-L1515) apresenta os seguintes achados auditados:

1. **`CODE_REFERENCE_STALE` (corrigido nesta revisão):** A Spec 076 original citava números de linha desatualizados para o bloco de entrada em `audio.rs`. Após a revisão da Spec 052, o manipulador de parâmetros `input.*` reside em [`crates/sonix-desktop/src/audio.rs:4558–4602 @ c20cc65`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/sonix-desktop/src/audio.rs#L4558-L4602) (controlando `input.gain_db`, `input.pad`, `input.channel_mode`, `input.impedance`, `input.pickup_type`).
2. **`ARCHITECTURE_CONFLICT` (`audio.rs`):** O condicionamento de entrada ainda reside inline dentro de `RealtimeDspChain` em `sonix-desktop/src/audio.rs` em vez de ser um bloco reutilizável em `ampforge-dsp` (`component.input.conditioning_and_pickup_rlc`) compartilhado entre standalone desktop e futuros formatos CLAP/VST3.

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica (Spec 014) | Faixa Aceita | Unidade | Valor Padrão | Papel no Condicionamento |
|---|---|---|---|---|---|
| **INPUT GAIN** | `input.gain_db` | `-24.0 .. +24.0` | `dB` | `0.0` | Ajuste de ganho linear de entrada para calibrar a interface ao headroom do amplificador. |
| **INPUT PAD** | `input.pad` | `false / true` | Booleano | `false` | Atenuação instantânea de `-12,0 dB` para captadores ativos ou saída muito alta. |
| **CHANNEL MODE** | `input.channel_mode` | `"mono_l"`, `"mono_r"`, `"stereo_sum"` | Enum | `"mono_l"` | Seleciona o canal físico do ADC onde a guitarra está plugada. |
| **VIRTUAL INPUT Z** | `input.impedance` | `"1m"`, `"470k"`, `"100k"`, `"22k"`, `"10k"` | Enum | `"1m"` (plano) | Seleciona $R_{\text{load}}$ no filtro diferencial RLC $\Delta H(s)$ (em `"1m"`, bypass plano `0,0 dB`). |
| **PICKUP MODEL** | `input.pickup_type` | `"flat"`, `"single_coil"`, `"humbucker"`, `"p90"`, `"active"` | Enum | `"flat"` | Define os parâmetros $(L_p, R_p)$ de referência para o cálculo de $\Delta H(s)$ quando `Virtual Input Z < 1 MΩ`. |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Sanitização + DC Blocker (`15 Hz`, 1ª ordem):**
   - Se `!x.is_finite()`, substitui por `0.0`.
   - $R_{\text{dc}} = 1,0 - \frac{2\pi \cdot 15,0}{f_s}$, $y_{\text{dc}}[n] = x[n] - x[n-1] + R_{\text{dc}} \cdot y_{\text{dc}}[n-1]$ (em `f64`).
2. **Ganho de Trim e Pad Suavizado (Anti-Zipper `5 ms`):**
   - $G_{\text{target}} = 10^{(\text{gain\_db} - (12,0 \text{ se pad})) / 20}$.
3. **Filtro Diferencial de Carregamento Virtual $\Delta H(s)$:**
   - Quando `input.impedance == "1m"` (ou `pickup_type == "flat"`), $\Delta H(z) \equiv 1$ (transparência absoluta `0,00 dB`).
   - Quando `input.impedance < 1m` (ex.: `"22k"` ou `"10k"` para simular carregamento de Fuzz Face / entrada baixa com $L_p = 2,5\text{ H}, C_{\text{ref}} = 470\text{ pF}$), o biquad diferencial amortiza o pico de `~4 kHz` e atenua os agudos acima de `2 kHz` conforme $\frac{H_{\text{RLC}}(s, R_{\text{target}})}{H_{\text{RLC}}(s, 1\text{ M}\Omega)}$.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Precisão numérica:** `f64` no DC blocker (`15 Hz` tem polo muito próximo de $z = 1$: $R_{\text{dc}} \approx 0,99804$ a `48 kHz`) e no biquad RLC.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block`; telemetria de pico/RMS escrita em `AtomicU32` sem bloqueios.
- WCET $\le 0,6\text{ }\mu\text{s}$ por bloco de `128` amostras.

---

## 12. Posição e interação na cadeia completa de guitarra

- Estágio `0` da cadeia: todo bloco subsequente (`Tuner`, `Gate`, `Wah`, `Pitch`, `Drive`, `Amp`) recebe sinal limpo de offset DC e livre de `NaN`/`Inf`.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/input.rs` (`pub struct InputConditionerProcessor` implementando `DspBlock` + `AutomatableParams`, Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Valor / Regra |
|---|---|
| Remoção de DC | Ganho em `0 Hz` $= -\infty\text{ dB}$ (`<= -60 dB` após `0,5 s`), atenuação em `82,41 Hz` (corda E2) `<= 0,20 dB`. |
| Transparência em modo `"1m"` / `"flat"` | Ganho plano **`0,00 dB ± 0,05 dB`** de `80 Hz` a `20 kHz`. |
| Ação do Pad | Redução exata de **`-12,00 dB ± 0,05 dB`** em toda a banda audível. |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Rejeição DC e Preservação de E2 (`82,41 Hz`):** injeta offset constante `+0,5` somado a senoide de `82,41 Hz` e verifica eliminação do offset DC com preservação da fundamental.
2. **Teste de Sanitização (`NaN`/`Inf`) e Pad `-12 dB`:** verifica recuperação imediata para `0.0` sem contaminar os estados internos dos filtros.
3. **Teste do Carregamento RLC Virtual (`1 MΩ` vs `10 kΩ`):** verifica atenuação de agudos (`4 kHz`) ao reduzir a impedância virtual mantendo `1 MΩ` completamente plano.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- A interface de usuário deve rotular explicitamente o controle de impedância como **"Impedância Virtual (Simulação RLC)"** (e não sugerir modificação elétrica do jack da placa de som).

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Bloqueio DC (`15 Hz`) e Preservação da Fundamental Grave E2 (`82,41 Hz`) (`ANALYTICAL`):**
  - *Sinal de entrada:* Offset DC constante `+0,50` somado a senoide de `82,41 Hz` (`-12 dBFS`) durante `1,0 s`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `input.gain_db = 0.0`, `input.pad = false`, `input.impedance = "1m"`.
  - *Método de medição:* Média aritmética (DC residual) nos últimos `250 ms` e amplitude da componente `82,41 Hz` via DFT.
  - *Critério e tolerância:* O DC residual deve cair para **`<= 0,001`** (`>= 54 dB` de rejeição), enquanto a fundamental `82,41 Hz` deve sofrer atenuação **`<= 0,25 dB`**.
  - *Origem da tolerância:* Função de transferência do DC Blocker de 1ª ordem em `15 Hz` (`F-02`).

- **CA-02 — Precisão do Atenuador `Pad (-12 dB)` e Sanitização de Amostras Não-Finitas (`ANALYTICAL` / `REGRESSION`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-6 dBFS`, com injeção de `f32::NAN` e `f32::INFINITY` em duas amostras isoladas.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Comparação entre `input.pad = false` e `input.pad = true`.
  - *Método de medição:* Diferença de nível RMS e verificação de `is_finite()` em 100% da saída após a injeção de `NaN`/`Inf`.
  - *Critério e tolerância:* `input.pad = true` deve atenuar o sinal em **`-12,00 dB ± 0,05 dB`**; `100%` das amostras de saída (incluindo as subsequentes à injeção de `NaN`/`Inf`) devem ser finitas (`is_finite() == true`).

- **CA-03 — Modelo Diferencial de Carregamento RLC Virtual (`1 MΩ` Plano vs `10 kΩ` Amortecido) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Tons senoidais em `200 Hz` e `4000 Hz` a `-18 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `input.pickup_type = "single_coil"`, comparando `input.impedance = "1m"` vs `input.impedance = "10k"`.
  - *Método de medição:* Diferença de ganho $G(4000\text{ Hz}) - G(200\text{ Hz})$ em dB.
  - *Critério e tolerância:* Em `"1m"`, a resposta deve ser plana (`0,0 dB ± 0,1 dB`); em `"10k"`, o carregamento virtual do captador Single-Coil deve amortecer os agudos em `4000 Hz` em **pelo menos `4,0 dB`** relativos a `200 Hz`.
  - *Origem da tolerância:* Equação RLC diferencial de captador Single-Coil ($L_p \approx 2,2\text{ H}$, `F-01`).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/sonix-desktop/src/audio.rs` (`L4558–4602`, e futuro `crates/ampforge-dsp/src/input.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`input.*`)

---

## 19. Critérios de conclusão da Spec

- [x] Distinção física clara entre impedância real da interface de áudio e simulação matemática diferencial RLC $\Delta H(s)$.
- [x] Referências de linhas em `audio.rs` atualizadas (`L4558–4602`).
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Lemme, H. — *The Secrets of Electric Guitar Pickups*.
- `[F-02]` Smith, J. O. — *Introduction to Digital Filters: DC Blocker* (`https://ccrma.stanford.edu/~jos/pasp/`).

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, atualizou-se o mapeamento de linhas de `audio.rs:4558–4602` e formalizou-se que a simulação de impedância em software opera como filtro diferencial $\Delta H(s)$ sobre a entrada `Hi-Z` física, sem jamais alegar modificação elétrica do hardware do ADC.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| DC Blocker `15 Hz` (1ª ordem `f64`) e sanitização `NaN`/`Inf` | `F-02` | Spec 076 / `CA-01`, `CA-02` | `crates/sonix-desktop/src/audio.rs` (DC blocker inline no callback) | `contract_044_sdd_integrity.rs` | `IMPLEMENTED_UNVERIFIED` |
| `Input Gain (±24 dB)` e `Input Pad (-12 dB)` | Spec 014 | Spec 076 / `CA-02` | `crates/sonix-desktop/src/audio.rs:4558–4602 @ c20cc65` | `contract_040_pedals_parity.rs` | `TESTED` |
| Filtro diferencial RLC de carregamento virtual de captadores (`1 MΩ` plano vs `10 kΩ`) | `F-01` | Spec 076 / `CA-03` | `crates/sonix-desktop/src/audio.rs:4580–4600 @ c20cc65` (aproximação simples de filtro de entrada, ainda não modularizada em `ampforge-dsp/src/input.rs`) | Ausente | `SPECIFIED` |
| Execução zero-alloc em tempo real | Spec 053 | Spec 076 / `CA-04` | `crates/sonix-desktop/src/audio.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
