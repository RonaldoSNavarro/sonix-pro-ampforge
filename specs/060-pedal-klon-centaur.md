---
status: rascunho
---

# Spec 060: Pedal Klon Centaur Professional Overdrive

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-pedal` (`drive`)
> **Identificador canônico:** `pedal.klon.centaur`
> **Variante / Revisão:** `1994–2000 Gold/Silver Horsie (TL072CP, Charge Pump MAX1044 ±9V/+18V, Diodos Germânio 1N34A)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/drive.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [031](031-amplificadores-fieis-aos-esquemas-e-controles-originais.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [079](079-componente-oversampling-polifasico-e-anti-aliasing.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação do pedal **Klon Centaur Professional Overdrive** (`pedal.klon.centaur`), corrigindo o defeito crítico de desalinhamento de fase entre o ramo limpo e o ramo saturado presente no código atual (`crates/ampforge-dsp/src/fx/drive.rs:553–560 @ c20cc65`) e modelando fielmente o potenciômetro duplo (`Dual-Gang 100 kΩ B`) de `Gain`, o headroom expandido (`+18 V` a `-9 V`, `27 Vpp`) e o equalizador shelving ativo `Treble` centrado no somador inversor `IC2B`.

---

## 2. Escopo

1. Buffer de entrada (`IC1A TL072`).
2. Arquitetura de **três caminhos paralelos** controlada pelo potenciômetro duplo linear (`Dual-Gang 100 kΩ B`, `Gain1` e `Gain2`) e somada no amplificador operacional inversor `IC2B` alimentado por `+18 V / -9 V` via conversor DC-DC `MAX1044` / `ICL7660S`:
   - **Caminho de Ganho / Clipping (`IC1B` + diodos `1N34A`):** Pré-ênfase de médios, ganho variável via `Gain1` e hard-clipping paralelo para a referência por diodos de germânio `1N34A` ($V_f \approx 0,35\text{ V}$).
   - **Caminho Limpo de Graves/Médios (`Feed-Forward 1`):** Passa-baixas atenuado na medida em que `Gain2` avança.
   - **Caminho Limpo Direto (`Feed-Forward 2`):** Passa-baixas fixo de graves/médios ($f_c \approx 830\text{ Hz}$) que injeta corpo limpo no somador `IC2B`.
3. **Alinhamento rigoroso de fase e atraso de grupo** entre os caminhos limpos e o caminho saturado com oversampling `8×`.
4. Controle ativo `Treble` (realimentação/entrada seletiva no somador `IC2B` em torno de $f_c \approx 1,2\text{ kHz}$, $\pm 12\text{ dB}$) e `Output` (`Level`).

---

## 3. Fora de escopo

- Klon KTR (2012, SMD) ou clones modificados com chaves de bass boost ("Fat switch").
- Outros pedais de overdrive/distortion (TS808 → Spec 059; RAT → Spec 061).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`overdrive` / `clean-dirty parallel blender`) |
| **Fabricante de referência** | Klon (Bill Finnegan, Boston, MA, EUA) |
| **Modelo de referência** | Klon Centaur Professional Overdrive |
| **Ano / época / revisão** | 1994–2000 (Circuito original encapsulado em epóxi preto; `2× TL072CP`, `MAX1044`, `2× 1N34A`) |
| **Identificador no sistema** | `pedal.klon.centaur` (`DriveModel::KlonCentaur` / UI slug `klon`) |
| **Papel na cadeia de sinal** | Bloco `DRIVE` pré-amplificador (usado como clean boost de alto headroom ou overdrive transparente com blend limpo). |

---

## 5. Princípio físico e funcional do equipamento real

1. **Conversor Charge-Pump (`MAX1044` / `ICL7660S`):** A bateria de `+9 V` é convertida em barramentos de **`+18 V`** e **`-9 V`** (`27 V` totais de excursão rail-to-rail) que alimentam o segundo dual op-amp `IC2` (`TL072`). Isso impede que o estágio somador (`IC2B`) e o controle de `Treble` saturem o op-amp, mesmo quando `Output` está no máximo empurrando a entrada de um amplificador valvulado.
2. **Potenciômetro Duplo `GAIN` (`Dual-Gang 100 kΩ Linear` — `Gain1a` / `Gain1b`):**
   - `Gain1a` controla o ganho do estágio não-inversor `IC1B` (com passa-altas seletivo que corta graves abaixo de $\sim 100\text{ Hz}$ antes do clipping) que alimenta o par antiparalelo de diodos de germânio `1N34A` ($V_f \approx 0,35\text{ V}$) ligados em **shunt para `Vref`** (hard-clipping suave após resistor série $R_{10} = 1\text{ k}\Omega$ e capacitor $C_9 = 1\text{ }\mu\text{F}$).
   - `Gain1b` controla simultaneamente a proporção entre o sinal ceifado e a rede passiva de **sinal limpo paralelo** (*Clean Blend*), de modo que em `Gain = 0%` o sinal no somador `IC2B` é quase 100% limpo, e em `Gain = 100%` o caminho saturado de médios/agudos domina enquanto o sub-caminho limpo de graves ($f_c \approx 830\text{ Hz}$) preserva a definição das notas fundamentais.
3. **Somador Inversor `IC2A/IC2B` e Equalizador Ativo `Treble`:** Os três ramos convergem no nó virtual do somador inversor `IC2B`, onde o potenciômetro `Treble` (`10 kΩ B`) com o capacitor $C_{14} = 3,9\text{ nF}$ atua como um filtro *high-shelf* ativo bidirecional (corte/reforço acima de $\sim 1,2\text{ kHz}$) sem alterar o ganho na região de graves.

---

## 6. Fontes técnicas autoritativas

> **Transparência documental:** Bill Finnegan nunca publicou um esquema elétrico oficial de fábrica do Klon Centaur (as placas originais eram cobertas com resina epóxi preta opaca). Toda a documentação esquemática mundial provém do *de-gooping* e engenharia reversa física de unidades originais (1994–2000), consolidada na análise técnica do ElectroSmash (`F-01`) e nos datasheets primários dos componentes (`F-02`, `F-03`). Por essa razão, a topologia do circuito é classificada como `SECONDARY_TECHNICAL`, e tolerâncias finas de componentes passivos individuais permanecem documentadas com margem explícita.

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | ElectroSmash — *Klon Centaur Circuit Analysis* (arquivado e verificado via Wayback Machine) | `https://web.archive.org/web/20240115000000/https://www.electrosmash.com/klon-centaur-analysis` (HTTP 200 verificado) | `SECONDARY_TECHNICAL` | Esquema completo extraído por engenharia reversa da PCB original, análise dos 3 caminhos paralelos, `Gain1a/1b`, somador `IC2B`, charge pump `+18V/-9V` e curvas de resposta em frequência. |
| `F-02` | Texas Instruments — *TL072 Low-Noise JFET-Input Operational Amplifier Datasheet* | Especificação oficial do componente (`Slew Rate = 13 V/µs`, `GBW = 3 MHz`) | `PRIMARY_COMPONENT` | Características de headroom e linearidade de `IC1` e `IC2`. |
| `F-03` | Maxim Integrated — *MAX1044 / ICL7660S Switched-Capacitor Voltage Converter Datasheet* | Especificação oficial do componente (dobrador `+18 V` e inversor `-9 V`) | `PRIMARY_COMPONENT` | Geração dos barramentos `+18 V` e `-9 V` (`27 V` rail-to-rail) em `IC2`. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Alimentação de `IC2` em `+18 V` e `-9 V` (`27 V` rail-to-rail) via `MAX1044` | `F-01`, `F-03` | Original 1994–2000 | `F-01` §"Power Supply Stage" | `SECONDARY_TECHNICAL` / `PRIMARY_COMPONENT` | `CONFIRMED` |
| Potenciômetro `GAIN` duplo linear (`100 kΩ B` Dual-Gang) controlando ganho de `IC1B` e atenuação do ramo limpo | `F-01` | Original 1994–2000 | `F-01` §"Gain Stage & Clean Feed-Forward" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Hard-clipping shunt pós-`IC1B` com par antiparalelo de diodos de germânio `1N34A` ($V_f \approx 0,35\text{ V}$) e resistor série $1\text{ k}\Omega$ | `F-01` | Original 1994–2000 | `F-01` §"Clipping Stage" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Somador inversor `IC2B` com controle ativo de `Treble` (`10 kΩ B` + $C_{14} = 3,9\text{ nF}$) | `F-01` | Original 1994–2000 | `F-01` §"Summing Amplifier and Tone Control" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Curva exata de seleção de lote dos diodos `1N34A` originais de Bill Finnegan | N/A | N/A | Sem datasheet oficial de lote da Klon | `UNKNOWN` | `PENDING` (modelado com curva padrão `1N34A` Ge $I_s = 200\text{ nA}, N = 1,45, R_s = 15\text{ }\Omega$) |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input] ──► [IC1A Buffer (TL072, +9V/0V)]
                 │
                 ├────► (Ramo 1: Dirty) ──► [IC1B Variable Gain (Gain1a) + Mid Pre-Emphasis]
                 │                                  │
                 │                                  ▼
                 │                          [R10 1kΩ + D2||D3 (1N34A Ge Shunt Clipping)]
                 │                                  │
                 │                                  ▼
                 │                          [LPF Pós-Clip + R15 15kΩ] ───────────────┐
                 │                                                                   │
                 ├────► (Ramo 2: Clean Mid/Low Variable via Gain1b + C7 82nF) ───────┼──► [IC2B Inverting Summer (+18V/-9V)]
                 │                                                                   │         ▲
                 └────► (Ramo 3: Clean Bass Fixed LPF: R13 1.5kΩ + C8 390nF) ────────┘         │
                                                                                      [Active Treble Pot 10kΩ + C14 3.9nF]
                                                                                               │
                                                                                               ▼
                                                                                      [Output Pot 10kΩ B] ──► [Output]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/drive.rs`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/fx/drive.rs#L465-L565) apresenta um **defeito crítico de engenharia DSP**:

1. **`FACTUAL_ERROR` (`drive.rs:465–480` e `553–560` — Cancelamento Comb-Filter por Desalinhamento de Fase entre Clean e Dirty):**
   - Na linha `465`, quando `self.model == DriveModel::KlonCentaur`, `process_sample` **não** faz oversampling do bloco inteiro (`let needs_oversampling = matches!(self.model, ...)` exclui `KlonCentaur`!).
   - Em vez disso, dentro de `process_sample_inner` (`drive.rs:553–560`), o código calcula `let clean = x * clean_blend;` (com **atraso de 0 amostras**) e passa **apenas** o ramo `dirty` por `self.oversampler.upsample_8x` e `downsample_8x` (`let dirty = self.oversampler.downsample_8x(&os_buf);`) e depois soma `clean + dirty`!
   - Como o filtro polifásico IIR `8×` (`Oversampler8x`) introduz um atraso de grupo não-nulo e rotação de fase dependente da frequência no ramo `dirty`, somar `clean` (0 atraso) com `dirty` (atrasado pelo filtro de oversampling) cria um **cancelamento de fase tipo *comb-filter*** ao longo do espectro audível — fenômeno que jamais existe no pedal analógico real!
- Removido em 2026-11-20 (AUD-38): identificador inexistente em qualquer commit

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Normalizada | Unidade / Escala UI | Curva / Taper | Valor Padrão | Papel Elétrico no Circuito |
|---|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `drive.enabled` | `pedal.klon.enabled` | `false / true` | Booleano | Discreto | `false` | Buffered bypass via `IC1A` e rede resistiva de saída. |
| **GAIN** | `drive.gain` | `pedal.klon.gain` | `0.0 .. 1.0` | `0 .. 100 %` | Linear Duplo (`100 kΩ B Dual-Gang`) | `0.50` | Opera simultaneamente `Gain1a` (ganho de `IC1B`) e `Gain1b` (balanço entre o ramo saturado `1N34A` e os ramos limpos). |
| **TREBLE** | `drive.tone` | `pedal.klon.tone` | `0.0 .. 1.0` | `0 .. 100 %` | Linear (`10 kΩ B`) | `0.50` | Move o cursor do potenciômetro `10 kΩ` do capacitor $C_{14} = 3,9\text{ nF}$ entre a entrada e a realimentação do somador `IC2B` (active high-shelf $\pm 12\text{ dB}$). |
| **OUTPUT** | `drive.level` | `pedal.klon.level` | `0.0 .. 1.0` | `0 .. 100 %` | Linear (`10 kΩ B`) | `0.50` | Atenuador na saída do somador `IC2B` (`+18 V / -9 V`). |

---

## 9. Modelo matemático e algoritmo DSP proposto

### 9.1. Correção Obrigatória de Coerência de Fase (Upsampling Unificado ou Atraso Compensado)

Para eliminar completamente o cancelamento de fase entre `clean` e `dirty`, existem duas implementações equivalentes permitidas (sendo a **Opção A** a obrigatória para coerência perfeita de fase em toda a banda `20 Hz–20 kHz`):
- **Opção A (preferencial):** Executar o `upsample_8x` sobre o sinal de saída do buffer `IC1A` ($x[n]$) **uma única vez**, processar tanto o ramo `dirty` (`IC1B` + diodos `1N34A`) quanto os ramos `clean` na mesma grade temporal de $8\times f_s$ (ou somá-los no nó virtual de `IC2B` em $8\times f_s$), e aplicar `downsample_8x` sobre a saída somada de `IC2B`. Dessa forma, a diferença de atraso de grupo entre os ramos paralelos devida ao oversampler é **exatamente zero ($0,0\text{ amostras}$)** em todas as frequências.

### 9.2. Equações dos Três Ramos Paralelos e Somador `IC2B`

1. **Ramo 1 (`Dirty` — `IC1B` + `1N34A`):**
   - Filtro passa-altas e pré-ênfase de médios de `IC1B` (pico centrado em $\sim 1,0\text{ kHz}$ com ganho proporcional a $g = \text{drive.gain}$, variando de $+2\text{ dB}$ em $g=0$ a $+33\text{ dB}$ em $g=1$).
   - Rede de clipping shunt com resistor série $R_{10} = 1\text{ k}\Omega$ e par antiparalelo de germânio `1N34A` ($V_f \approx 0,35\text{ V}$, resistência ôhmica interna $R_s \approx 15\text{ }\Omega$):
     $$v_{\text{clip}}(v) = V_{f,\text{Ge}} \cdot \operatorname{asinh}\left(\frac{v}{V_{f,\text{Ge}}}\right)$$
   - Filtro passa-baixas pós-clipping ($f_c \approx 4,2\text{ kHz}$) antes do resistor somador $R_{15} = 15\text{ k}\Omega$.
2. **Ramo 2 e Ramo 3 (`Clean Feed-Forward`):**
   - Ramo 2 (atenuado por $1 - g$ via `Gain1b`, LPF em $f_{c2} \approx 1,8\text{ kHz}$) + Ramo 3 (grave fixo LPF em $f_{c3} \approx 830\text{ Hz}$ via $R_{13}=1,5\text{ k}\Omega, C_8=390\text{ nF}$).
3. **Somador Inversor `IC2B` + Equalizador Ativo `Treble` + Headroom `27 Vpp`:**
   - Soma ponderada dos três ramos seguida de filtro *high-shelf* ativo de 1ª ordem com frequência de transição $f_{\text{shelf}} \approx 1200\text{ Hz}$ e ganho $G_{\text{treble}} = 24,0 \cdot (\text{tone} - 0,5)\text{ dB} \in [-12\text{ dB},\; +12\text{ dB}]$.
   - Limitação suave de barramento apenas em $\pm 13,5\text{ V}$ (`27 Vpp` do barramento `+18 V / -9 V`), garantindo ausência de clipping do op-amp `IC2B`.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `8×` polifásico IIR (Spec 079) englobando a bifurcação e a soma dos ramos `clean` e `dirty`, garantindo **zero defasagem diferencial** introduzida pelos filtros de interpolação/decimação.
- **Precisão numérica:** `f64` em todos os filtros de estado e soma nodal.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`.
- Complexidade $O(1)$ por amostra sem laços não-limitados; WCET $\le 12\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- **Entrada:** Alta impedância ($1\text{ M}\Omega$ no pedal real).
- **Saída:** Buffer de baixa impedância e alto headroom (`27 Vpp`), capaz de fornecer até `+15 dB` de *clean boost* na saída (`Output = 1.0, Gain = 0.0`) para saturar autenticamente o estágio de entrada `V1` de amplificadores valvulados (`Spec 054`–`058`).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/klon_centaur.rs`.
- **Struct principal:** `pub struct KlonCentaurProcessor` implementando `DspBlock` + `AutomatableParams` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Subsistema | Componente Real | Valor Documentado | Invariante Física |
|---|---|---|---|
| Barramento `IC2` | `MAX1044` + diodos/caps | `+18 V` e `-9 V` (`27 Vpp`) | `IC2B` não satura antes dos diodos `1N34A` nem em `Gain = 0% / Output = 100%`. |
| Clipping Shunt | `2× 1N34A` Ge + $R_{10}=1\text{ k}\Omega$ | $V_f \approx 0,35\text{ V}$ | Tensão de limiar menor e joelho mais suave que o silício (`0,65 V`). |
| Ramo Clean Grave | $R_{13}=1,5\text{ k}\Omega, C_8=390\text{ nF}$ | $f_c \approx 830\text{ Hz}$ | Graves fundamentais passam limpos mesmo em `Gain = 100%`. |
| Controle `Treble` | Pot `10 kΩ B` + $C_{14}=3,9\text{ nF}$ | Shelf ativo $\pm 12\text{ dB} @ >1,2\text{ kHz}$ | Em `Treble = 50%` (`0.5`), o ganho do shelf é plano ($0\text{ dB}$). |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Coerência de Fase entre Ramos Paralelos (`Clean` vs `Dirty`):** verifica que não há entalhes (*notches*) de comb-filtering ao misturar `clean` e `dirty` em `Gain = 0.5`.
2. **Teste de Clean Boost de Alto Headroom (`Gain = 0.0`):** verifica `THD < 0,5%` com sinal de entrada de `-12 dBFS` e ganho positivo de saída.
3. **Teste de Ação Bidirecional do Shelf Ativo `Treble`:** verifica corte/reforço simétrico acima de `1,2 kHz` mantendo graves (`150 Hz`) invariantes.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- A ausência de esquema oficial publicado pela Klon exige que a classificação documental permaneça `SECONDARY_TECHNICAL`, com a parametrização específica dos diodos `1N34A` registrada como `PENDING` para medição física futura caso uma unidade hardware original esteja disponível.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Coerência de Fase entre Ramo Limpo e Ramo Saturado (Ausência de Comb-Filtering) (`ANALYTICAL` / `ENGINEERING_TARGET`):**
  - *Sinal de entrada:* Varredura senoidal de pequeno sinal (`-40 dBFS`, onde os diodos `1N34A` operam em regime linear) de `100 Hz` a `12 000 Hz` (50 pontos logarítmicos).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `pedal.klon.centaur` com `drive.enabled = true`, `drive.gain = 0.50` (mistura ativa dos ramos `clean` e `dirty`), `drive.tone = 0.50`, `drive.level = 0.50`.
  - *Método de medição:* Diferença de atraso de grupo entre o ramo `clean` e o ramo `dirty` devida ao estágio de oversampling, e inspeção da resposta em magnitude para ausência de *notches* de cancelamento.
  - *Critério e tolerância:* A diferença de atraso introduzida pelo oversampler entre os ramos `clean` e `dirty` deve ser **exatamente `0` amostras**, e a resposta em magnitude entre `200 Hz` e `10 kHz` deve ser suave e livre de entalhes de fase (nenhum vale local $> 1,5\text{ dB}$ abaixo da envoltória monotônica).
  - *Origem da tolerância:* Invariante física de soma analógica em tempo contínuo no nó virtual de `IC2B` (`F-01` §"Summing Amplifier").

- **CA-02 — Regime Clean Boost de Alto Headroom em `Gain = 0.0` (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` com amplitude `-18 dBFS` ($0,1259\text{ FS}$).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras, FFT de `16 384` amostras.
  - *Configuração e parâmetros:* `drive.gain = 0.0`, `drive.tone = 0.50`, `drive.level = 0.85`.
  - *Método de medição:* Medição de THD (Total Harmonic Distortion, razão RMS dos harmônicos $H_2..H_{10}$ sobre a fundamental $H_1$) e ganho da fundamental.
  - *Critério e tolerância:* O ganho da fundamental deve ser **$> +3,0\text{ dB}$** e a distorção harmônica total deve ser **`THD <= 0,5%`** (`-46 dB`), comprovando a operação como *clean boost* sob o barramento de `27 Vpp` (`+18 V / -9 V`).
  - *Origem da tolerância:* Operação linear do `TL072` sob barramento `+18 V / -9 V` com `Gain1a = 0` (`F-01` §"Frequency Response vs Gain", `F-02`).

- **CA-03 — Transição Dual-Gang (`Gain = 0.0` vs `Gain = 1.0`) e Preservação da Fundamental Grave (`DOCUMENTED`):**
  - *Sinal de entrada:* Sinal composto por duas senoides: grave fundamental $f_1 = 110\text{ Hz}$ (`-12 dBFS`) + médio $f_2 = 1000\text{ Hz}$ (`-12 dBFS`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `drive.gain = 1.0`, `drive.tone = 0.50`, `drive.level = 0.50`.
  - *Método de medição:* Razão entre a distorção harmônica gerada por $f_2 = 1000\text{ Hz}$ ($H_3 @ 3000\text{ Hz}$) e a energia da fundamental grave $f_1 = 110\text{ Hz}$.
  - *Critério e tolerância:* Em `drive.gain = 1.0`, o 3º harmônico de `1000 Hz` (`3000 Hz`) deve aumentar em **pelo menos `+20,0 dB`** em relação a `drive.gain = 0.0`, enquanto a fundamental grave de `110 Hz` é preservada pelo ramo limpo paralelo (`LPF 830 Hz`) com atenuação não superior a `4,0 dB` relativa à sua amplitude em `drive.gain = 0.0`.
  - *Origem da tolerância:* Arquitetura de três ramos paralelos (`F-01` §"Clean Feed-Forward Network").

- **CA-04 — Equalizador Shelving Ativo `Treble` (`10 kΩ B` + `3,9 nF`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Varredura senoidal de pequeno sinal (`-30 dBFS`) em `200 Hz` e `4000 Hz`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `drive.gain = 0.25`, `drive.level = 0.50`, comparando `drive.tone = 0.0`, `0.5` e `1.0`.
  - *Método de medição:* Ganho em dB em `4000 Hz` e `200 Hz`.
  - *Critério e tolerância:*
    1. Em `drive.tone = 0.5`, a diferença de ganho entre `4000 Hz` e `200 Hz` devida ao estágio `Treble` deve estar dentro de **$\pm 1,0\text{ dB}$** (neutro).
    2. Em `drive.tone = 1.0`, o ganho em `4000 Hz` deve subir **`+8,0 dB` a `+14,0 dB`** em relação a `drive.tone = 0.5`.
    3. Em `drive.tone = 0.0`, o ganho em `4000 Hz` deve cair **`-8,0 dB` a `-14,0 dB`** em relação a `drive.tone = 0.5`, enquanto o ganho em `200 Hz` varia no máximo **$\pm 1,5\text{ dB}$** em toda a excursão de `tone`.
  - *Origem da tolerância:* Curva de resposta do controle `Treble` em `IC2B` (`F-01` §"Tone Control Frequency Response": $\pm 11,5\text{ dB} @ 4\text{ kHz}$).

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `gain`, `tone`, `level`.
  - *Critério e tolerância:* `0` alocações na heap (alocador instrumentado) e `0` amostras `NaN`/`Inf`.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/drive.rs` (e futuro `crates/ampforge-dsp/src/fx/pedals/klon_centaur.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.klon.*`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_031_pedals_dsp.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Ausência de esquema oficial de fábrica explicitamente declarada; fonte técnica reversa (`F-01`) verificada via Wayback Machine.
- [x] Bug de desalinhamento de fase entre `clean` (0× OS) e `dirty` (8× OS) em `drive.rs:553–560` diagnosticado e especificado para correção.
- [x] Critérios `CA-01` a `CA-05` classificados e com condições completas de medição.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` ElectroSmash — *Klon Centaur Circuit Analysis* (`https://web.archive.org/web/20240115000000/https://www.electrosmash.com/klon-centaur-analysis`).
- `[F-02]` Texas Instruments — *TL072 Low-Noise JFET-Input Operational Amplifier Datasheet*.
- `[F-03]` Maxim Integrated — *MAX1044 Switched-Capacitor Voltage Converter Datasheet*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, identificou-se que `drive.rs:465, 553–560` passava apenas o ramo `dirty` pelo oversampler `8×` enquanto somava `clean` sem atraso, causando comb-filtering por desalinhamento de fase, além de usar um LPF passivo em vez do high-shelf ativo `Treble` em `IC2B`.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Alinhamento de fase `0 amostras` entre ramos `clean` e `dirty` no somador `IC2B` | `F-01` §"Summing Amplifier" | Spec 060 / `CA-01` | `crates/ampforge-dsp/src/fx/drive.rs:465 @ c20cc65, 553–560` (DEFASADO: `clean` sem OS somado a `dirty` com 8× IIR OS) | Ausente | `SPECIFIED` |
| Clean boost de alto headroom (`27 Vpp`, `+18V/-9V`) em `Gain = 0` (`THD <= 0,5%`) | `F-01`, `F-02`, `F-03` | Spec 060 / `CA-02` | `crates/ampforge-dsp/src/fx/drive.rs:538–564 @ c20cc65` | `contract_031_pedals_dsp.rs` | `IMPLEMENTED_UNVERIFIED` |
| Potenciômetro duplo `Gain1a/1b` + clipping shunt `1N34A` + ramo limpo grave `830 Hz` | `F-01` | Spec 060 / `CA-03` | `crates/ampforge-dsp/src/fx/drive.rs:538–558 @ c20cc65` (usa `clean_blend = (1.0 - drive * 0.85)` sem filtro LPF de `830 Hz` no ramo limpo) | `contract_031_pedals_dsp.rs`, `contract_040_pedals_parity.rs` | `SPECIFIED` |
| Equalizador shelving ativo `Treble` ($\pm 12\text{ dB} @ >1,2\text{ kHz}$) no somador `IC2B` | `F-01` §"Tone Control" | Spec 060 / `CA-04` | `crates/ampforge-dsp/src/fx/drive.rs:437 @ c20cc65` (usa LPF simples `2500..10000 Hz`) | `contract_040_pedals_parity.rs` | `SPECIFIED` |
| Curva de lote original dos diodos `1N34A` selecionados pela Klon | Sem fonte oficial de fábrica | Spec 060 / §6.2 | `drive.rs:549` (`vt = 0.35`) | N/A | `PENDING` |
