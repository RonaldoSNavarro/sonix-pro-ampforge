---
status: rascunho
---

# Spec 066: Rack Compressor FET UREI 1176LN (Peak Limiter Revision D/E)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-dynamics` (`rack`)
> **Identificador canônico:** `rack.urei.1176ln`
> **Variante / Revisão:** `1970–1973 UREI Model 1176LN Revision D / E ("Blackface" Low-Noise, JFET 2N5457 / 2N3391, Classe A Output)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/dynamics.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [031](031-amplificadores-fieis-aos-esquemas-e-controles-originais.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação do compressor de estúdio **UREI Model 1176LN Solid-State Peak Limiter (Revision D/E Blackface)** como processador modular independente (`rack.urei.1176ln`), fundamentado no manual técnico oficial da UREI (`https://media.uaudio.com/assetlibrary/1/1/1176ln_manual.pdf`, `PRIMARY_OFFICIAL`), separando-o estritamente das outras variantes hoje agrupadas em `CompressorProcessor` (`PedalRossDyna`, `OpticalLa2a`, `VcaMasterBus`), e diagnosticando a diferença entre a topologia real de **realimentação (*feedback*) com divisor JFET `2N5457`** do 1176LN e o detector *feedforward* simplificado atualmente implementado em [`crates/ampforge-dsp/src/fx/dynamics.rs:433–530 @ c20cc65`](../crates/ampforge-dsp/src/fx/dynamics.rs#L433-L530).

---

## 2. Escopo

1. Atenuador de entrada (`INPUT`) e divisor de tensão variável na entrada usando o JFET canal-N `Q1` (`2N5457`) como resistor controlado por tensão ($R_{\text{DS}}(V_{\text{GS}})$) em série com o resistor de entrada de `27 kΩ`, com circuito de linearização *Low-Noise (`LN`)* de Brad Plunkett (capacitores/resistor dreno-porta reduzindo distorção de 2ª ordem do JFET).
2. Amplificador de pré-ênfase de sinal (`+26 dB`) e estágio de saída Classe A acoplado por transformador (`+10 dB`, totalizando até `+40..45 dB` de ganho de linha).
3. **Topologia de Detecção por Realimentação (*Feedback Sidechain*):** O detector retificador de onda completa (`CR2`, `CR3`) lê o sinal **após o estágio amplificador de sinal (antes do potenciômetro `OUTPUT`)**, comparando-o contra uma tensão de limiar de polarização (*bias threshold*) dependente da chave de **`RATIO` (`4:1`, `8:1`, `12:1`, `20:1`)** e do modo **`ALL-BUTTONS IN` ("British Mode")**.
4. Tempos ultra-rápidos de **`ATTACK` (`20 µs` a `800 µs`)** e **`RELEASE` (`50 ms` a `1100 ms`)**, além de blend paralelo (`MIX`) e telemetria real de redução de ganho (`GR dB`) para o medidor VU.

---

## 3. Fora de escopo

- **Outros modelos de compressor em `CompressorModel` (`dynamics.rs:212–217`):**
  - `PedalRossDyna` (Ross Compressor / MXR Dyna Comp baseados no OTA `CA3080`) — fora do escopo desta Spec.
  - `OpticalLa2a` (Teletronix LA-2A baseado na célula eletroluminescente-fotoresistiva `T4B`) — fora do escopo desta Spec.
  - `VcaMasterBus` (SSL G-Series Master Bus VCA `DBX 202C` / `THAT 2180`) — fora do escopo desta Spec.
- Revisões `Rev A/B "Bluestripe"` (sem circuito `LN` de linearização dreno-porta) ou `Rev F/G/H` (estágio de saída Classe AB push-pull em vez de Classe A com transformador `B11148`).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `rack` (`dynamics` / `fet-peak-limiter`) |
| **Fabricante de referência** | UREI — United Recording Electronics Industries (Bill Putnam & Brad Plunkett, North Hollywood, CA, EUA) |
| **Modelo de referência** | Model 1176LN Solid-State Peak Limiter |
| **Ano / época / revisão** | 1970–1973 Revision D / E ("Blackface" Low-Noise, `2N5457` FET, Transformador de saída `B11148` Classe A) |
| **Identificador no sistema** | `rack.urei.1176ln` (`CompressorModel::Fet1176` / UI `pedal.comp.*`) |
| **Papel na cadeia de sinal** | Bloco `COMP` (compressor/limiter dinâmico rápido antes do pré-amplificador ou no pós-cab para controle de transientes e *squash* estilo estúdio). |

---

## 5. Princípio físico e funcional do equipamento real

De acordo com o manual oficial de operação e serviço da UREI (`F-01`, Seções I–IV e Esquema Elétrico):

1. **Atenuador JFET `Q1` (`2N5457`) na Entrada + Topologia *Feedback*:**
   - O JFET `Q1` atua na região ôhmica (triodo) como o braço shunt para o terra de um divisor de tensão com o resistor série $R_5 = 27\text{ k}\Omega$ logo após o atenuador T-pad de `INPUT` (`600 Ω`).
   - Quando não há redução de ganho, a porta (*gate*) de `Q1` é mantida em uma tensão negativa próxima ao *pinch-off* ($V_p \approx -2,0\text{ V}$ a $-3,0\text{ V}$), fazendo $R_{\text{DS}} \to \infty$ (atenuação zero).
   - O sinal após o JFET é amplificado pelo estágio de pré-amplificação (`+26 dB`), e a saída desse estágio alimenta **simultaneamente** o potenciômetro `OUTPUT` (que vai para o amplificador de linha Classe A) e o **Amplificador de Controle do Sidechain (*Gain Reduction Control Amp*)** — configurando uma **malha fechada de realimentação (*Feedback Compressor*)**!
   - Consequentemente:
     - O controle **`INPUT`** governa simultaneamente o nível que entra no circuito **e a quantidade de compressão (limiar efetivo)**, pois no 1176LN real o limiar interno do sidechain é fixo para cada `Ratio`!
     - O controle **`OUTPUT`** está posicionado **fora** da malha de detecção do sidechain, atuando exclusivamente como *makeup gain* final no estágio Classe A.
2. **Faixas Oficiais de `Attack` (`20 µs` a `800 µs`) e `Release` (`50 ms` a `1100 ms`):**
   - Especificações oficiais UREI (`F-01`, p. 1-2 §1.5 Specifications):
     - **Attack Time:** continuamente variável de **`20 microssegundos` (`0,02 ms`) a `800 microssegundos` (`0,80 ms`)**.
     - **Release Time:** continuamente variável de **`50 milissegundos` a `1,1 segundos` (`1100 ms`)**.
3. **Seleção de `Ratio` (`4:1`, `8:1`, `12:1`, `20:1`) e Modo *All-Buttons In* ("British Mode"):**
   - No 1176LN, os botões de `Ratio` alteram duas coisas ao mesmo tempo no divisor do amplificador de controle do sidechain: a **inclinação (ganho DC do sidechain)** e a **tensão de polarização negativa de limiar (*threshold bias*)**! Em ratios menores (`4:1`), o limiar é mais baixo e o joelho (*knee*) é mais suave; em `20:1` (limiting), o limiar sobe vários dB e o joelho torna-se abrupto.
   - Quando todos os 4 botões mecânicos são pressionados simultaneamente (*All-Buttons In*), os resistores divisores do banco de chaves entram em paralelo: o `Ratio` resultante situa-se entre `12:1` e `20:1`, o ponto de polarização do JFET desloca-se abruptamente, a constante de tempo do detector sofre um atraso inicial (*lag plateau*) que deixa o transiente explodir antes de esmagar o sinal, e a distorção harmônica sobe significativamente.

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | UREI — *Model 1176LN Solid-State Peak Limiter Instruction Manual (Operating Instructions, Theory of Operation, Maintenance & Schematic Diagram)* | `https://media.uaudio.com/assetlibrary/1/1/1176ln_manual.pdf` (HTTP 200 PDF oficial verificado, 2,45 MB) | `PRIMARY_OFFICIAL` | Especificações oficiais de fábrica (`Attack 20–800 µs`, `Release 50–1100 ms`, `Ratios 4:1, 8:1, 12:1, 20:1`, circuito `LN` de linearização do FET `Q1` e esquema completo Rev C/D/E). |
| `F-02` | ON Semiconductor / Motorola — *2N5457 / 2N5458 N-Channel JFET Datasheet* | Especificação oficial do componente ($V_{\text{GS(off)}} \in [-0,5\text{ V}, -6,0\text{ V}]$, $r_{\text{ds(on)}} \approx 250\text{ }\Omega$) | `PRIMARY_COMPONENT` | Curva de condutância canal-dreno-fonte do JFET atenuador `Q1`. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Tempo de Attack continuamente variável de `20 µs` (`0,02 ms`) a `800 µs` (`0,8 ms`) | `F-01` | UREI 1176LN Rev C/D/E | Section I, p. 1-2 §1.5 "Specifications — Attack Time" | `UNKNOWN` | `CONFIRMED` |
| Tempo de Release continuamente variável de `50 ms` a `1,1 s` (`1100 ms`) | `F-01` | UREI 1176LN Rev C/D/E | Section I, p. 1-2 §1.5 "Specifications — Release Time" | `UNKNOWN` | `CONFIRMED` |
| Ratios selecionáveis por pushbutton: `4:1`, `8:1`, `12:1` e `20:1` alterando simultaneamente ganho do amplificador de controle e limiar de threshold | `F-01` | UREI 1176LN Rev C/D/E | Section I §1.5 & Section III "Theory of Operation" pp. 3-1 a 3-3 | `UNKNOWN` | `CONFIRMED` |
| Atenuador JFET `Q1` (`2N5457`) com resistor série $R_5 = 27\text{ k}\Omega$, circuito `LN` dreno-porta e topologia *feedback* lida antes do pot `OUTPUT` | `F-01` | UREI 1176LN Rev C/D/E | Section III p. 3-1 Fig. 3-1 Block Diagram & Schematic | `UNKNOWN` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input] ──► [Input Attenuator (INPUT Knob)]
                     │
                     ▼
            [R5 (27 kΩ) ──┬──► Signal Preamp (+26 dB, LN Drain-Gate Linearization)]
                          │                     │
             [Q1 2N5457 JFET Shunt R_ds(V_c)]   ├──────────────────────────► [OUTPUT Pot] ──► [Class-A Line Amp (+10 dB) + B11148 XFMR] ──► [Wet Out]
                          ▲                     │
                          │                     ▼ (Feedback Sidechain Loop)
                          │          [Ratio Divider & Threshold Bias (4:1 / 8:1 / 12:1 / 20:1 / All-Buttons)]
                          │                     │
                          │                     ▼
                          └───────── [Full-Wave Rectifier CR2/CR3 + Attack (20–800 µs) / Release (50–1100 ms) RC Network]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/dynamics.rs`](../crates/ampforge-dsp/src/fx/dynamics.rs#L211-L551) possui as seguintes divergências auditadas:

1. **`MODEL_COUPLING` (`dynamics.rs:212–217`):** O `CompressorProcessor` agrupa 4 arquiteturas físicas incompatíveis (`PedalRossDyna`, `Fet1176`, `OpticalLa2a`, `VcaMasterBus`) dentro de um único método `process_sample`. Esta Spec isola exclusivamente o contrato do **`rack.urei.1176ln` (`Fet1176`)**.
- Removido em 2026-11-20 (AUD-38): identificador inexistente em qualquer commit
3. **`FACTUAL_ERROR` (`controls.registry.ts:575` vs `F-01` p. 1-2):** No registro de UI (`pedal.comp.attack`), a faixa de `comp.attack_ms` está declarada como `0.02 .. 50 ms`, enquanto no `1176LN` real o tempo máximo de Attack é **`0,80 ms` (`800 µs`)**. Para manter compatibilidade com os demais modelos de `CompressorModel` sem violar a física do `1176LN`, quando o processador `rack.urei.1176ln` está selecionado, a constante de tempo física do capacitor de carga do detector JFET deve mapear a excursão do controle para a faixa documentada da UREI (`0,02 ms` a `0,80 ms`, ou aceitar o valor em `ms` grampeado à faixa física do circuito 1176LN quando em modo estrito 1176LN).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real / UI | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Aceita | Unidade | Valor Padrão | Papel Elétrico no Circuito |
|---|---|---|---|---|---|---|
| **POWER / BYPASS** | `comp.enabled` | `pedal.comp.enabled` | `false / true` | Booleano | `false` | Bypass da redução de ganho / bloco `comp`. |
| **INPUT (SUSTAIN)** | `comp.sustain` | `pedal.comp.sustain` | `0.0 .. 1.0` | `0 .. 100 %` | `0.50` | Atenuador de entrada (`-24 dB` a `+24 dB` relativos ao limiar do sidechain), governando o drive para dentro do divisor JFET `Q1`. |
| **OUTPUT (LEVEL)** | `comp.level` | `pedal.comp.level` | `0.0 .. 1.0` | `0 .. 100 %` | `0.50` | Atenuador *pós-malha de feedback* na entrada do amplificador de linha Classe A (`-24 dB` a `+24 dB`). |
| **ATTACK** | `comp.attack_ms` | `pedal.comp.attack` | `0.02 .. 50.0` (1176LN físico: `0.02 .. 0.80`) | `ms` | `0.20` (`200 µs`) | Resistor variável de carga do capacitor do detector (`CR2/CR3`). |
| **RELEASE** | `comp.release_ms` | `pedal.comp.release` | `50.0 .. 1100.0` | `ms` | `150.0` | Resistor variável de descarga do capacitor do detector (`50 ms` a `1,1 s`). |
| **RATIO** | `comp.ratio` | `pedal.comp.ratio` | `"4:1"`, `"8:1"`, `"12:1"`, `"20:1"`, `"all"` | Enum | `"4:1"` | Banco de pushbuttons que seleciona o ganho do sidechain e a tensão de bias de limiar (incluindo modo *All-Buttons In*). |
| **MIX (PARALLEL)** | `comp.mix` | `pedal.comp.mix` | `0.0 .. 1.0` | `0 .. 100 %` | `1.00` | Extensão de estúdio (*New York Parallel Compression*): mistura entre o sinal seco alinhado e a saída do estágio Classe A. |
| **VU MODE** | *(visual)* | `pedal.comp.vu_mode` | `"GR"`, `"OUT"` | Enum UI | `"GR"` | Chave visual do medidor VU (lê telemetria `gain_reduction_db` do backend). |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Divisor Atenuador JFET `Q1` (`2N5457`) + Pré-Amplificador `+26 dB` (Malha Fechada de 1 Amostra de Atraso a $f_s$ ou $2\times f_s$):**
   - Seja $v_{\text{in}}[n] = x[n] \cdot G_{\text{input}}(\text{sustain})$.
   - A condutância dreno-fonte do JFET `2N5457` em função da tensão de controle negativa da porta $V_c[n-1] \in [V_p,\; 0]$ (com circuito de linearização `LN`) resulta em um fator de atenuação do divisor $R_5 = 27\text{ k}\Omega$:
     $$g_{\text{FET}}[n] = \frac{R_{\text{DS}}(V_c[n-1])}{R_5 + R_{\text{DS}}(V_c[n-1])} \in [g_{\min},\; 1,0]$$
   - A saída do pré-amplificador de sinal (com leve saturação de 2ª/3ª ordem residual do JFET + circuito `LN`) é:
     $$v_{\text{pre}}[n] = \mathcal{N}_{\text{LN}}\!\left(v_{\text{in}}[n] \cdot g_{\text{FET}}[n]\right)$$
2. **Detector Retificador de Onda Completa em Realimentação (*Feedback Sidechain*) e Banco de Ratios:**
   - Para cada seleção de `Ratio` $r \in \{4, 8, 12, 20, \text{All}\}$, define-se o limiar de polarização $V_{\text{th}}(r)$ (crescente de `4:1` para `20:1`, e rebaixado com platô de atraso no modo `All`) e o ganho do amplificador de controle $K_{\text{sc}}(r)$.
   - O excesso retificado acima do bias $e[n] = \max(0,\; |v_{\text{pre}}[n]| - V_{\text{th}}(r))$ carrega o capacitor do detector com constante de tempo $\tau_{\text{att}}$ e descarrega com $\tau_{\text{rel}}$ (incluindo o efeito de *program-dependent release* pela combinação paralela RC do 1176LN em transientes longos vs curtos).
3. **Estágio de Saída Classe A (`+10 dB` + Transformador `B11148`):**
   - O potenciômetro `OUTPUT` escala $v_{\text{pre}}[n]$ **fora** da malha de feedback, aplicando uma saturação assimétrica suave de transformador/Classe A ($\text{THD} \le 0,5\%$ em nível nominal) antes do controle `Mix`.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `2×` ou `4×` opcional para o laço de feedback do JFET e saturação do estágio de saída Classe A, mantendo latência sub-milissegundo.
- **Precisão numérica:** `f64` ou `f32` sem denormais no integrador RC do detector.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`.
- Publicação lock-free de `gain_reduction_db` (`AtomicU32` / telemetria a `30–60 Hz`) para o medidor VU da interface.

---

## 12. Posição e interação na cadeia completa de guitarra

- Pode operar tanto antes do amplificador (nivelando dedilhados limpos ou *funk/country picking*) quanto em modo paralelo (`Mix < 100%`) preservando o ataque seco enquanto adiciona corpo comprimido.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/racks/urei_1176ln.rs` (isolado dos futuros módulos de `RossDyna`, `La2a` e `SslBus`).
- **Struct principal:** `pub struct Urei1176LnProcessor` implementando `DspBlock` + `AutomatableParams` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Parâmetro do 1176LN | Valor Oficial (`F-01` p. 1-2) | Invariante Física |
|---|---|---|
| `Attack Time` | `20 µs` a `800 µs` (`0,02..0,80 ms`) | Resposta de pico ultra-rápida capaz de capturar o primeiro ciclo de `1 kHz`. |
| `Release Time` | `50 ms` a `1100 ms` | Recuperação exponencial controlada pelo resistor de descarga do detector. |
| `Ratios` | `4:1`, `8:1`, `12:1`, `20:1`, `All-Buttons` | Aumentar o `Ratio` aumenta simultaneamente a taxa de compressão e o limiar de polarização (*threshold bias*). |
| `OUTPUT` fora do loop | Pós-detector feedback | Alterar `comp.level` (`OUTPUT`) **não altera** a redução de ganho `gain_reduction_db` medida no VU! |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Invariância da Redução de Ganho ao Controle `OUTPUT` (Prova de Topologia Feedback/Post-Loop Makeup):** verifica que variar `comp.level` de `0.1` a `1.0` muda o nível de saída, mas mantém `gain_reduction_db` rigorosamente invariante.
2. **Teste de Curva de Compressão por Ratio (`4:1`, `8:1`, `12:1`, `20:1`) e Modo *All-Buttons In*:** verifica que a redução de ganho e o conteúdo harmônico obedecem à progressão dos botões e que o modo `"all"` eleva a THD e altera a envoltória de ataque.
3. **Teste de Resposta Ultra-Rápida de Attack (`20 µs`):** verifica atenuação de degrau em menos de `1 ms`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- No 1176LN hardware original, girar os knobs `ATTACK` e `RELEASE` no sentido horário (`1 → 7`) torna os tempos **mais rápidos** (`CCW = lento`, `CW = rápido`). No AMPFORGE, como a UI exibe explicitamente o valor numérico em milissegundos (`ms`) no mostrador do knob (`controls.registry.ts:575, 585`), o parâmetro numérico `comp.attack_ms` e `comp.release_ms` representa diretamente o tempo físico em milissegundos (`menor valor em ms = mais rápido`), evitando ambiguidade de unidade.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Isolamento do Controle `OUTPUT` Fora da Malha de Detecção (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Senoide contínua de `1000 Hz` a `-6 dBFS` por `500 ms`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `rack.urei.1176ln` com `comp.enabled = true`, `comp.sustain = 0.75` (`INPUT`), `comp.ratio = "8:1"`, `comp.attack_ms = 0.2`, `comp.release_ms = 100.0`, `comp.mix = 1.0`, comparando `comp.level = 0.25` vs `comp.level = 0.85` (`OUTPUT`).
  - *Método de medição:* Leitura da telemetria `gain_reduction_db()` e do nível RMS de saída.
  - *Critério e tolerância:* Ao variar `comp.level` (`OUTPUT`) de `0.25` para `0.85`, o nível RMS de saída deve subir **pelo menos `+12,0 dB`**, enquanto a redução de ganho `gain_reduction_db()` medida pelo sidechain deve permanecer invariante dentro de **`±0,05 dB`**, enquanto variar `comp.sustain` (`INPUT`) de `0.25` para `0.85` deve aumentar `gain_reduction_db()` em **pelo menos `+6,0 dB`**.
  - *Origem da tolerância:* Diagrama de blocos oficial do UREI 1176LN (`F-01` Fig. 3-1 p. 3-1: potenciômetro `OUTPUT` situado após a derivação para o *Control Amp*).

- **CA-02 — Progressão das Taxas de Compressão (`4:1`, `8:1`, `12:1`, `20:1`) (`DOCUMENTED` / `REGRESSION`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` testada em dois níveis acima do limiar: `-12 dBFS` e `-2 dBFS` (`+10,0 dB` de variação na entrada).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras (`500 ms` de acomodação).
  - *Configuração e parâmetros:* `comp.sustain = 0.80`, `comp.attack_ms = 0.2`, `comp.release_ms = 100.0`, `comp.mix = 1.0`.
  - *Método de medição:* Redução de ganho em regime permanente `GR(ratio)` em `-2 dBFS` e variação do nível de saída $\Delta Y_{\text{dB}} = Y(-2\text{ dBFS}) - Y(-12\text{ dBFS})$.
  - *Critério e tolerância:* A variação do nível de saída $\Delta Y_{\text{dB}}$ para um degrau de `+10,0 dB` na entrada em regime de compressão plena deve diminuir monotonamente com o aumento do `Ratio`:
    $$\Delta Y_{\text{dB}}(\text{"4:1"}) > \Delta Y_{\text{dB}}(\text{"8:1"}) > \Delta Y_{\text{dB}}(\text{"12:1"}) > \Delta Y_{\text{dB}}(\text{"20:1"})$$
  - *Origem da tolerância:* Especificação oficial dos pushbuttons de Ratio do 1176LN (`F-01` p. 1-2 §1.5).

- **CA-03 — Modo *All-Buttons In* ("British Mode"): Saturação JFET e Aumento de THD (`DOCUMENTED` / `REGRESSION`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-6 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras, FFT de `16 384` amostras.
  - *Configuração e parâmetros:* `comp.sustain = 0.75`, `comp.attack_ms = 0.2`, `comp.release_ms = 100.0`, `comp.mix = 1.0`, comparando `comp.ratio = "4:1"` vs `comp.ratio = "all"`.
  - *Método de medição:* Medição da THD (razão RMS de $H_2..H_{10}$ sobre $H_1$) na saída.
  - *Critério e tolerância:* No modo `"all"`, a THD gerada pelo deslocamento de bias do JFET `Q1` e sobrecarga do amplificador de controle deve ser **pelo menos `1,5×` maior** (`+3,5 dB`) do que no modo `"4:1"` sob o mesmo nível de entrada.
  - *Origem da tolerância:* Comportamento elétrico do banco de botões pressionados simultaneamente no 1176LN (`F-01` §3).

- **CA-04 — Tempo de Attack Ultra-Rápido (`20 µs` a `800 µs`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Degrau senoidal de `1000 Hz` saltando de `-40 dBFS` para `0 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `comp.sustain = 0.85`, `comp.ratio = "20:1"`, `comp.attack_ms = 0.05` (`50 µs`), `comp.release_ms = 200.0`.
  - *Método de medição:* Número de amostras necessárias após o degrau para atingir `63,2%` ($1 - e^{-1}$) da redução de ganho final em dB.
  - *Critério e tolerância:* O detector deve atingir `63,2%` da redução de ganho final em **menos de `1,0 ms` (`<= 48` amostras a `48 kHz`)**.
  - *Origem da tolerância:* Especificação oficial `Attack Time: 20 µs to 800 µs` (`F-01` p. 1-2 §1.5).

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `sustain`, `level`, `attack`, `release`, `ratio`, `mix`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/dynamics.rs` (`CompressorProcessor` e futuro `crates/ampforge-dsp/src/fx/racks/urei_1176ln.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.comp.*`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_031_pedals_dsp.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Manual oficial da UREI (`1176ln_manual.pdf`, `2,45 MB`) verificado com link direto HTTP 200 (`PRIMARY_OFFICIAL`).
- [x] Modelos `PedalRossDyna`, `OpticalLa2a` e `VcaMasterBus` removidos do escopo desta Spec para eliminar acoplamento indevido.
- [x] Critérios `CA-01` a `CA-05` classificados e com condições completas de medição.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` United Recording Electronics Industries (UREI). *Model 1176LN Solid-State Peak Limiter Instruction Manual* (`https://media.uaudio.com/assetlibrary/1/1/1176ln_manual.pdf`).
- `[F-02]` ON Semiconductor — *2N5457 N-Channel JFET Datasheet*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, substituiu-se o link genérico `uaudio.com` pelo PDF oficial do manual de serviço do UREI 1176LN (`F-01`), removeram-se `Ross/Dyna`, `LA-2A` e `VCA Bus` do escopo desta Spec, e documentou-se a topologia feedback com atenuador JFET `Q1` (`2N5457`).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Controle `INPUT` (`sustain`) governa a compressão e `OUTPUT` (`level`) atua fora da malha de detecção | `F-01` Fig. 3-1 p. 3-1 | Spec 066 / `CA-01` | `crates/ampforge-dsp/src/fx/dynamics.rs:437–527 @ c20cc65` | `contract_040_pedals_parity.rs` | `IMPLEMENTED_UNVERIFIED` |
| Ratios selecionáveis `4:1`, `8:1`, `12:1`, `20:1` com deslocamento de limiar e inclinação | `F-01` p. 1-2, p. 3-2 | Spec 066 / `CA-02` | `crates/ampforge-dsp/src/fx/dynamics.rs:451–489 @ c20cc65` | `contract_031_pedals_dsp.rs`, `contract_040_pedals_parity.rs` | `TESTED` |
| Modo *All-Buttons In* (`"all"`) com saturação JFET e platô de ataque | `F-01` §3 | Spec 066 / `CA-03` | `crates/ampforge-dsp/src/fx/dynamics.rs:451–457 @ c20cc65, 503–510` | `contract_031_pedals_dsp.rs` (`test_ca_06_comp_1176_all_buttons_and_ratios`) | `TESTED` |
| Attack ultra-rápido (`20 µs` a `800 µs`) e Release (`50 ms` a `1100 ms`) | `F-01` p. 1-2 §1.5 | Spec 066 / `CA-04` | `crates/ampforge-dsp/src/fx/dynamics.rs:394–402 @ c20cc65` | `contract_040_pedals_parity.rs` | `IMPLEMENTED_UNVERIFIED` |
| Topologia de malha fechada *Feedback Sidechain* no divisor JFET $R_5 = 27\text{ k}\Omega / Q_1$ | `F-01` Fig. 3-1 | Spec 066 / §7.1, §9 | `crates/ampforge-dsp/src/fx/dynamics.rs:480 @ c20cc65` (atualmente usa detector *feedforward* sobre `driven.abs()`) | Ausente | `SPECIFIED` |
| Execução zero-alloc em tempo real | Spec 053 | Spec 066 / `CA-05` | `crates/ampforge-dsp/src/fx/dynamics.rs:433–530 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
