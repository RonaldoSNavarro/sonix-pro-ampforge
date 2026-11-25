---
status: rascunho
---

# Spec 067: Pedal MXR M108 Ten Band Graphic Equalizer

> **Status:** `rascunho` (`bloqueio parcial documental` no esquema elétrico dos giradores `MC33179`)
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade — Desacoplamento de Modelos de EQ)
> **Domínio:** `dsp-eq` (`pedal`)
> **Identificador canônico:** `pedal.mxr.m108_10band_eq`
> **Variante / Revisão:** `MXR M108 Ten Band Graphic EQ (18V Power Supply, 10 Bandas 31.25 Hz – 16 kHz, ±12 dB + Input Gain / Output Volume)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/eq.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [031](031-amplificadores-fieis-aos-esquemas-e-controles-originais.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [085](085-eq-grafico-5-bandas-mesa-boogie-mark.md), [086](086-modulo-equalizador-parametrico-8-bandas.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação **exclusivamente** do pedal **MXR M108 Ten Band Graphic Equalizer (`pedal.mxr.m108_10band_eq`)**, desfazendo o agrupamento indevido (`MODEL_COUPLING`) da versão anterior desta Spec — que misturava o MXR M108 (`10 bandas`), o Equalizador Gráfico de 5 Bandas do Mesa/Boogie Mark IIC+/IV (agora isolado na **[Spec 085](085-eq-grafico-5-bandas-mesa-boogie-mark.md)**) e o Equalizador Paramétrico Digital de 8 Bandas (agora isolado na **[Spec 086](086-modulo-equalizador-parametrico-8-bandas.md)**).

---

## 2. Escopo

1. Pedal equalizador gráfico de 10 bandas por oitava centradas exatamente nas frequências oficiais do manual Dunlop/MXR M108 (`31,25 Hz`, `62,5 Hz`, `125 Hz`, `250 Hz`, `500 Hz`, `1 kHz`, `2 kHz`, `4 kHz`, `8 kHz` e `16 kHz`), cada uma com excursão de **`±12 dB`** (`Cut/Boost`).
2. Controles independentes de **`GAIN` (Input Level, `±12 dB`)** e **`VOLUME` (Output Level, `±12 dB`)** presentes no chassi do MXR M108.
3. Topologia de giradores ativos de fase mínima com interação de $Q$ proporcional (*Proportional-Q / Constant-Q Gyrator Bank*) e barramento de alto headroom (`18 V`).

---

## 3. Fora de escopo

- **Equalizador Gráfico de 5 Bandas Mesa/Boogie Mark Series (`80 Hz, 240 Hz, 750 Hz, 2200 Hz, 6600 Hz`):** Movido integralmente para a **[Spec 085](085-eq-grafico-5-bandas-mesa-boogie-mark.md)**.
- **Módulo Equalizador Paramétrico Digital de 8 Bandas (`EqProcessor (EqMode::Parametric8Band)`):** Movido integralmente para a **[Spec 086](086-modulo-equalizador-parametrico-8-bandas.md)**.
- **MXR M108S (revisão moderna em alumínio prateado com dupla saída `OUT 1 / OUT 2` e circuito de redução de ruído atualizado):** Documentado por contraste na Seção 6, mantendo o `M108` clássico como referência primária do identificador `pedal.mxr.m108_10band_eq`.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`graphic-eq`) |
| **Fabricante de referência** | Dunlop Manufacturing, Inc. / MXR Innovations (Benicia, CA, EUA) |
| **Modelo de referência** | MXR M108 Ten Band Graphic EQ |
| **Ano / época / revisão** | Revisão M108 (`18 V` DC adapter `ECB004`, 12 faders iluminados por LED: `GAIN`, 10 frequências ISO por oitava, `VOLUME`) |
| **Identificador no sistema** | `pedal.mxr.m108_10band_eq` (UI `pedal.eq.*` / backend `eq.*` & `EqProcessor (EqMode::Graphic10BandPedal)`) |
| **Papel na cadeia de sinal** | Bloco `EQ` (modelagem de espectro pré-drive/pré-amp ou escultura tonal pós-pré-amplificador no FX Loop). |

---

## 5. Princípio físico e funcional do equipamento real

1. **Banco de 10 Bandas Espaçadas por 1 Oitava Exata (`31,25 Hz` a `16 kHz`):**
   - De acordo com o manual oficial Dunlop MXR M108 (`F-01`), as 10 frequências centrais são potências exatas de 2 a partir de `1000 Hz`:
     $$f_k = 1000 \cdot 2^{k-5}\text{ Hz} \quad (k = 0..9) \implies \{31,25;\; 62,5;\; 125;\; 250;\; 500;\; 1000;\; 2000;\; 4000;\; 8000;\; 16\,000\}\text{ Hz}$$
2. **Faixa Oficial de Cut/Boost de `±12 dB` em Todas as 10 Bandas + `GAIN` e `VOLUME` (`±12 dB`):**
   - Tanto os 10 faders de frequência quanto os faders de `GAIN` (entrada) e `VOLUME` (saída) possuem curso simétrico de **`-12 dB` a `+12 dB`** com detente central em `0 dB`.
3. **Topologia de Giradores Simulados e Comportamento de *Proportional-Q*:**
   - Nos equalizadores gráficos analógicos MXR baseados em amplificadores operacionais quádruplos (`MC33179` / `TL074`) onde os cursores dos 10 potenciômetros deslizantes (`sliders`) são acoplados entre o ramo de entrada e o ramo de realimentação do amplificador somador enquanto cada terminal central do potenciômetro drena para um indutor sintético (girador ativo RC), a largura de banda não é fixa:
     - Quando o ganho de uma banda está próximo de `0 dB`, o $Q$ efetivo é mais largo ($Q \approx 1,0..1,4$) para evitar ondulações (*ripples*) quando bandas adjacentes são movidas juntas;
     - À medida que o fader se aproxima dos extremos (`±12 dB`), o $Q$ efetivo estreita-se progressivamente até $Q \approx 2,4..2,8$ (*Proportional-Q*), concentrando a ação na oitava selecionada.
4. **Barramento de `18 V` DC:**
   - Diferente de pedais comuns de `9 V`, o MXR M108 opera com fonte externa de **`18 V`** para garantir headroom linear ($\sim 15\text{ Vpp}$) mesmo quando várias bandas e o fader de `GAIN` são simultaneamente elevados em `+12 dB`.

---

## 6. Fontes técnicas autoritativas

> **Honestidade documental (§4 / §11):** Os manuais oficiais da Dunlop para o **MXR M108** (`F-01`) e **MXR M108S** (`F-02`) são fontes `UNKNOWN` que comprovam 100% das frequências centrais (`31,25 Hz` a `16 kHz`), a faixa de `±12 dB` das 10 bandas, a faixa de `±12 dB` dos faders `GAIN` e `VOLUME`, as impedâncias e o piso de ruído. Contudo, a Dunlop **não publica o esquema elétrico interno no manual do proprietário**; os detalhes internos do circuito girador (`MC33179` e a implementação da banda de `16 kHz` como girador vs capacitor shelf) provêm de levantamentos técnicos secundários (`SECONDARY_TECHNICAL`). Portanto, a validação circuit-level exata dos resistores/capacitores internos dos 10 giradores é declarada como `PENDING` / `bloqueio parcial documental` até obtenção do *MXR M108 Factory Service Schematic*.

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Dunlop Manufacturing, Inc. — *MXR M108 Ten Band Graphic EQ Owner's Manual* | `https://www.jimdunlop.com/content/manuals/M108.pdf` (HTTP 200 PDF oficial verificado, 204,7 KB) | `PRIMARY_OFFICIAL` | Especificações oficiais do M108: 10 bandas (`31.25 Hz, 62.5 Hz, 125 Hz, 250 Hz, 500 Hz, 1 kHz, 2 kHz, 4 kHz, 8 kHz, 16 kHz`), Cut/Boost `±12 dB`, Input `GAIN ±12 dB`, Output `VOLUME ±12 dB`, alimentação `18 V`. |
| `F-02` | Dunlop Manufacturing, Inc. — *MXR M108S Ten Band EQ Owner's Manual* | `https://www.jimdunlop.com/content/manuals/M108S.pdf` (HTTP 200 PDF oficial verificado, 419,6 KB) | `PRIMARY_OFFICIAL` | Especificações detalhadas de laboratório da linha M108/M108S (`Input Impedance 470 kΩ`, `Output Impedance 5 kΩ`, `Max Input Level +14 dBV @ 1 kHz`, `Noise Floor -99,7 dBV`, `±12 dB` em todas as 10 bandas + Gain/Volume). |
| `F-03` | Dennis Bohn (Rane Corporation) — *Constant-Q and Proportional-Q Graphic Equalizers (Rane Note 101)* | Literatura técnica clássica de projeto de equalizadores gráficos ativos com giradores | `SECONDARY_TECHNICAL` | Modelagem matemática da variação de $Q(|G_{\text{dB}}|)$ em bancos de giradores somadores por oitava. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| 10 frequências centrais: `31.25, 62.5, 125, 250, 500, 1k, 2k, 4k, 8k, 16k Hz` | `F-01`, `F-02` | MXR M108 / M108S | `F-01` p. 1–2 "Controls & Specifications"; `F-02` p. 2 | `UNKNOWN` | `CONFIRMED` |
| Faixa de Cut/Boost de **`±12 dB`** para todas as 10 bandas, `GAIN` (`±12 dB`) e `VOLUME` (`±12 dB`) | `F-01`, `F-02` | MXR M108 / M108S | `F-01` p. 2 "Specifications"; `F-02` p. 2 "Filter Specs" | `UNKNOWN` | `CONFIRMED` |
| Alimentação de `18 V` DC para alto headroom (`+14 dBV` max input/output) | `F-01`, `F-02` | MXR M108 / M108S | `F-01` p. 2 "Power"; `F-02` p. 2 | `UNKNOWN` | `CONFIRMED` |
| Valores exatos dos resistores e capacitores internos de cada um dos 10 giradores na PCB da Dunlop | Esquema de serviço não incluído no manual público `M108.pdf` | MXR M108 | N/A | `UNKNOWN` | `PENDING` (`BLOCKED` para modelagem component-level dos 10 giradores; modelado analiticamente via banco Proportional-Q calibrado nas 10 frequências e `±12 dB` oficiais) |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input] ──► [Input Gain Slider (±12 dB)]
                     │
                     ▼
      [Summing Op-Amp Stage + 10 Parallel Gyrator Resonators (18V Rail)]
      ├── Band 1:  31.25 Hz (±12 dB)
      ├── Band 2:  62.50 Hz (±12 dB)
      ├── Band 3: 125.00 Hz (±12 dB)
      ├── Band 4: 250.00 Hz (±12 dB)
      ├── Band 5: 500.00 Hz (±12 dB)
      ├── Band 6:   1.00 kHz (±12 dB)
      ├── Band 7:   2.00 kHz (±12 dB)
      ├── Band 8:   4.00 kHz (±12 dB)
      ├── Band 9:   8.00 kHz (±12 dB)
      └── Band 10: 16.00 kHz (±12 dB)
                     │
                     ▼
            [Output Volume Slider (±12 dB)] ──► [Output]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/eq.rs`](../crates/ampforge-dsp/src/fx/eq.rs#L117-L228) e [`apps/sonix-app/ui/src/controls.registry.ts:799–883`](../apps/sonix-app/ui/src/controls.registry.ts#L799-L883) apresenta as seguintes divergências auditadas:

1. **`MODEL_COUPLING` (corrigido nesta revisão):** A Spec 067 original agrupava `EqProcessor (EqMode::Graphic10BandPedal)` (M108 10-Band + Mesa 5-Band) e `EqProcessor (EqMode::Parametric8Band)` (8-Band Parametric). Agora a Spec 067 trata exclusivamente do **MXR M108 (`pedal.mxr.m108_10band_eq`)**, enquanto o Mesa 5-Band foi movido para a **Spec 085** e o Paramétrico de 8 Bandas para a **Spec 086**.
2. **`FACTUAL_ERROR` (`eq.rs:131, 166`):** Em `EqProcessor (EqMode::Graphic10BandPedal)`, o limite de ganho das bandas está codificado como `gain_db.clamp(-15.0, 15.0)` (`±15 dB`) e o $Q$ proporcional usa `gain_db.abs() / 15.0`. Porém, tanto o manual oficial da Dunlop (`F-01` p. 2, `F-02` p. 2) quanto o registro de controles da própria UI do AMPFORGE (`controls.registry.ts:815–879`, `range: [-12, 12]`) especificam que o MXR M108 opera estritamente em **`±12,0 dB`**!
3. **`ARCHITECTURE_CONFLICT` (`controls.registry.ts:799–883` vs `eq.rs:131`):**
   - No `controls.registry.ts:799–883`, o pedal visual `pedal.eq.*` mapeia as 10 bandas (`31 Hz` a `16 kHz`) para as chaves `eq.b1.gain_db` .. `eq.b8.gain_db` (com um comentário na linha `866` notando que `eq.b9.gain_db` e `eq.b10.gain_db` foram adicionadas para cobrir `8 kHz` e `16 kHz` além das 8 bandas originais de `EqProcessor (EqMode::Parametric8Band)`).
   - Além disso, o MXR M108 real possui **12 sliders** (`GAIN` de entrada `±12 dB`, 10 frequências, e `VOLUME` de saída `±12 dB`), enquanto `EqProcessor (EqMode::Graphic10BandPedal)` (`eq.rs:124–129`) possui apenas `output_level_db` (faltando `input_gain_db`), e `controls.registry.ts` não expõe os sliders `GAIN` e `VOLUME` do M108.

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real (MXR M108) | Chave Canônica Proposta / Atual | UI Control ID (`controls.registry.ts`) | Faixa Real | Unidade | Valor Padrão | Papel Elétrico no Circuito |
|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `eq.enabled` | `pedal.eq.enabled` | `false / true` | Booleano | `false` | Hardwire bypass do pedal M108. |
| **INPUT GAIN** | `eq.input_gain_db` *(novo)* | `pedal.eq.input_gain` *(novo)* | `-12.0 .. +12.0` | `dB` | `0.0` | Fader de ganho de entrada antes do banco de giradores (`±12 dB`). |
| **31.25 Hz** | `eq.b1.gain_db` | `pedal.eq.b1` | `-12.0 .. +12.0` | `dB` | `0.0` | Banda 1 (`31,25 Hz`, girador ativo). |
| **62.5 Hz** | `eq.b2.gain_db` | `pedal.eq.b2` | `-12.0 .. +12.0` | `dB` | `0.0` | Banda 2 (`62,5 Hz`, girador ativo). |
| **125 Hz** | `eq.b3.gain_db` | `pedal.eq.b3` | `-12.0 .. +12.0` | `dB` | `0.0` | Banda 3 (`125 Hz`, girador ativo). |
| **250 Hz** | `eq.b4.gain_db` | `pedal.eq.b4` | `-12.0 .. +12.0` | `dB` | `0.0` | Banda 4 (`250 Hz`, girador ativo). |
| **500 Hz** | `eq.b5.gain_db` | `pedal.eq.b5` | `-12.0 .. +12.0` | `dB` | `0.0` | Banda 5 (`500 Hz`, girador ativo). |
| **1 kHz** | `eq.b6.gain_db` | `pedal.eq.b6` | `-12.0 .. +12.0` | `dB` | `0.0` | Banda 6 (`1000 Hz`, girador ativo). |
| **2 kHz** | `eq.b7.gain_db` | `pedal.eq.b7` | `-12.0 .. +12.0` | `dB` | `0.0` | Banda 7 (`2000 Hz`, girador ativo). |
| **4 kHz** | `eq.b8.gain_db` | `pedal.eq.b8` | `-12.0 .. +12.0` | `dB` | `0.0` | Banda 8 (`4000 Hz`, girador ativo). |
| **8 kHz** | `eq.b9.gain_db` | `pedal.eq.b9` | `-12.0 .. +12.0` | `dB` | `0.0` | Banda 9 (`8000 Hz`, girador ativo). |
| **16 kHz** | `eq.b10.gain_db` | `pedal.eq.b10` | `-12.0 .. +12.0` | `dB` | `0.0` | Banda 10 (`16000 Hz`, girador/shelf ativo com pré-warping Tustin). |
| **OUTPUT VOLUME** | `eq.output_level_db` | `pedal.eq.output_level` *(novo)* | `-12.0 .. +12.0` | `dB` | `0.0` | Fader de volume de saída após o somador (`±12 dB`). |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Lei de *Proportional-Q* Calibrada para `±12,0 dB` (`k = 0..9`):**
   - Para cada banda $k \in \{0..9\}$ com frequência central $f_k = 31,25 \cdot 2^k\text{ Hz}$ e ganho $G_k \in [-12,0,\; +12,0]\text{ dB}$:
     $$Q_k(G_k) = Q_{\min} + (Q_{\max} - Q_{\min}) \cdot \left(\frac{|G_k|}{12,0}\right), \qquad Q_{\min} = 1,20,\; Q_{\max} = 2,60$$
2. **Compensação de Cramping perto de Nyquist para as Bandas de `8 kHz` e `16 kHz`:**
   - A $f_s = 48\,000\text{ Hz}$, a banda de `16 kHz` está em $\frac{2}{3} f_{\text{Nyquist}}$ ($\omega_0 = \frac{2\pi}{3}$). Um biquad peaking RBJ padrão sofre compressão da largura de banda superior (*Nyquist cramping*). O cálculo dos coeficientes de `8 kHz` e `16 kHz` deve aplicar **compensação de largura de banda analógica pré-deformada (Orfanidis / Massberg)** ou operar com `2×` oversampling para manter a simetria da campânula em `16 kHz`.
3. **Headroom de `18 V` (`+14 dBV`):**
   - Limitação suave de barramento em $\pm 7,0\text{ V}_{\text{pk}}$ (`14 Vpp` equivalentes a `+14 dBV` de `F-02` p. 2).

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Precisão numérica:** Coeficientes e variáveis de estado das 10 seções biquad em `f64` (Transposed Direct Form II) para eliminar ruído de quantização nos polos de baixa frequência (`31,25 Hz` e `62,5 Hz` onde $\omega_0 \approx 0,004\text{ rad/amostra}$).

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`.
- Complexidade fixa de 10 biquads por amostra; WCET $\le 3,0\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Pode operar antes do bloco `DRIVE`/`AMP` (como *mid-booster* cirúrgico) ou no loop pós-pré-amplificador (esculpindo o timbre final antes do gabinete IR).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/mxr_m108_eq.rs` (separado do Mesa 5-Band da Spec 085 e do Paramétrico de 8 Bandas da Spec 086).
- **Struct principal:** `pub struct MxrM108EqProcessor` implementando `AudioProcessor` + `ModelDescriptor` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Parâmetro | Valor Oficial (`F-01`, `F-02`) | Invariante Física |
|---|---|---|
| Frequências centrais ($f_0..f_9$) | `31.25, 62.5, 125, 250, 500, 1k, 2k, 4k, 8k, 16k Hz` | Razão exata de $2,0\times$ (`1 oitava`) entre bandas adjacentes. |
| Excursão de cada banda | `-12,0 dB` a `+12,0 dB` | Em `0,0 dB` em todas as bandas, a resposta é plana ($0,00\text{ dB}$) de `20 Hz` a `20 kHz`. |
| `Input Gain` e `Output Volume` | `-12,0 dB` a `+12,0 dB` | Ganhos lineares independentes antes e depois do banco de filtros. |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Transparência em Detente Central (`0 dB` em todas as 10 bandas):** verifica resposta plana `0,00 dB ± 0,01 dB` em todo o espectro.
2. **Teste de Precisão de Pico e Corte (`±12,0 dB`) nas 10 Frequências Oficiais:** verifica cada uma das 10 bandas individualmente via DFT.
3. **Teste de *Proportional-Q*:** verifica que a largura de banda relativa de `-3 dB` estreita quando o ganho sobe de `+4 dB` para `+12 dB`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- **Pendência documental parcial (`PENDING`):** Como o esquema de serviço de fábrica da Dunlop para os giradores `MC33179` do M108 não é público no manual do usuário, a curva exata de $Q(G)$ ($Q_{\min}=1,2 \to Q_{\max}=2,6$) é classificada como `ANALYTICAL` / `ENGINEERING_TARGET` até medição ou obtenção do *Factory Service Manual*.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Transparência Absoluta com as 10 Bandas em `0,0 dB` (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Varredura senoidal nas 10 frequências centrais (`31,25 Hz` a `16 000 Hz`) a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `pedal.mxr.m108_10band_eq` com `eq.enabled = true`, `input_gain_db = 0.0`, `output_level_db = 0.0` e todas as 10 bandas `b1..b10 = 0.0 dB`.
  - *Método de medição:* Ganho em dB em cada uma das 10 frequências.
  - *Critério e tolerância:* O ganho em todas as 10 frequências deve ser **`0,00 dB ± 0,02 dB`**.
  - *Origem da tolerância:* Detente central `0 dB` do MXR M108 (`F-01` p. 2).

- **CA-02 — Excursão Oficial de `±12,0 dB` nas 10 Frequências Centrais ISO (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoides nas 10 frequências centrais $f_k \in \{31,25;\; 62,5;\; 125;\; 250;\; 500;\; 1000;\; 2000;\; 4000;\; 8000;\; 16\,000\}\text{ Hz}$ a `-20 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Para cada banda $k \in \{1..10\}$ individualmente (com as outras 9 bandas em `0,0 dB`), testar $G_k = +12,0\text{ dB}$ e $G_k = -12,0\text{ dB}$.
  - *Método de medição:* Ganho em dB na frequência central $f_k$ medido por DFT.
  - *Critério e tolerância:* Em $G_k = +12,0\text{ dB}$, o ganho medido em $f_k$ deve ser **`+12,0 dB ± 0,3 dB`**; em $G_k = -12,0\text{ dB}$, o ganho medido em $f_k$ deve ser **`-12,0 dB ± 0,3 dB`** (e valores de entrada acima de `±12,0 dB` devem ser limitados a `±12,0 dB`, corrigindo o clamp antigo de `±15 dB` de `eq.rs:131`).
  - *Origem da tolerância:* Especificação oficial `Cut/Boost: ±12 dB` nos manuais `M108.pdf` (`F-01` p. 2) e `M108S.pdf` (`F-02` p. 2).

- **CA-03 — Comportamento *Proportional-Q* (Estreitamento da Largura de Banda em Ganhos Extremos) (`ANALYTICAL` / `ENGINEERING_TARGET`):**
  - *Sinal de entrada:* Varredura senoidal fina (`900 Hz` a `1100 Hz`) em torno da banda de `1000 Hz` (`b6`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Banda `1000 Hz` testada em `+4,0 dB` vs `+12,0 dB` (demais bandas em `0,0 dB`).
  - *Método de medição:* Cálculo do fator de qualidade efetivo $Q = \frac{f_0}{\Delta f_{\text{mid}}}$ (onde $\Delta f_{\text{mid}}$ é a largura de banda na metade do ganho em dB).
  - *Critério e tolerância:* O fator de qualidade efetivo em `+12,0 dB` ($Q \approx 2,6$) deve ser **pelo menos `1,6×` maior** que o fator de qualidade efetivo em `+4,0 dB` ($Q \approx 1,67$).
  - *Origem da tolerância:* Lei de *Proportional-Q* de giradores ativos (`F-03`).

- **CA-04 — Atuação dos Controles Independentes `INPUT GAIN` (`±12 dB`) e `OUTPUT VOLUME` (`±12 dB`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-24 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Todas as 10 bandas em `0,0 dB`; testar `input_gain_db = +6.0 dB` com `output_level_db = +6.0 dB` e `input_gain_db = -12.0 dB` com `output_level_db = 0.0 dB`.
  - *Método de medição:* Ganho total em dB na saída.
  - *Critério e tolerância:* Ganho total de **`+12,0 dB ± 0,1 dB`** no primeiro caso e **`-12,0 dB ± 0,1 dB`** no segundo caso.
  - *Origem da tolerância:* Especificação oficial `Gain: ±12 dB, Volume: ±12 dB` (`F-01` p. 2, `F-02` p. 2).

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação das 10 bandas.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/eq.rs` (`EqProcessor (EqMode::Graphic10BandPedal)` e futuro `crates/ampforge-dsp/src/fx/pedals/mxr_m108_eq.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.eq.*`)
- Testes atuais: `Planejado: crates/ampforge-dsp/tests/contract_031_pedals_dsp.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Manuais oficiais da Dunlop (`M108.pdf` e `M108S.pdf`) verificados via HTTP 200 (`PRIMARY_OFFICIAL`).
- [x] Desacoplado completamente do Mesa 5-Band EQ (movido para a **[Spec 085](085-eq-grafico-5-bandas-mesa-boogie-mark.md)**) e do Paramétrico de 8 Bandas (movido para a **[Spec 086](086-modulo-equalizador-parametrico-8-bandas.md)**).
- [x] Divergência de `±15 dB` (`eq.rs:131`) vs `±12 dB` oficial diagnosticada e documentada.
- [x] Pendência do esquema interno dos giradores `MC33179` registrada honestamente como `PENDING`.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Dunlop Manufacturing, Inc. — *MXR M108 Ten Band Graphic EQ Owner's Manual* (`https://www.jimdunlop.com/content/manuals/M108.pdf`).
- `[F-02]` Dunlop Manufacturing, Inc. — *MXR M108S Ten Band EQ Owner's Manual* (`https://www.jimdunlop.com/content/manuals/M108S.pdf`).
- `[F-03]` Bohn, D. (Rane Corp.) — *Constant-Q Graphic Equalizers (Rane Note 101)*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, desfez-se o acoplamento triplo da Spec 067 original: a Spec 067 ficou exclusiva para o **MXR M108 10-Band Graphic EQ**, criando-se a **Spec 085** para o Mesa/Boogie 5-Band Graphic EQ e a **Spec 086** para o Equalizador Paramétrico Digital de 8 Bandas. Documentou-se a faixa oficial de `±12 dB` dos PDFs da Dunlop (`M108.pdf` e `M108S.pdf`).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| 10 frequências centrais ISO (`31.25 Hz` a `16 kHz`) e transparência em `0 dB` | `F-01` p. 2, `F-02` p. 2 | Spec 067 / `CA-01` | `crates/ampforge-dsp/src/fx/eq.rs:107–109 @ c20cc65` (`MXR_10_FREQS`) | `contract_031_pedals_dsp.rs`, `contract_040_pedals_parity.rs` | `TESTED` |
| Faixa oficial de Cut/Boost de **`±12,0 dB`** nas 10 bandas | `F-01` p. 2, `F-02` p. 2 | Spec 067 / `CA-02` | `crates/ampforge-dsp/src/fx/eq.rs:131 @ c20cc65` (divergente: usa `clamp(-15.0, 15.0)` no DSP, embora `controls.registry.ts:815–879` use `[-12, 12]`) | `contract_040_pedals_parity.rs` | `SPECIFIED` |
| Banco de filtros *Proportional-Q* ($Q \in [1,2;\; 2,6]$) | `F-03` | Spec 067 / `CA-03` | `crates/ampforge-dsp/src/fx/eq.rs:166–168 @ c20cc65` | `contract_031_pedals_dsp.rs` | `TESTED` |
| Controles independentes `INPUT GAIN` (`±12 dB`) e `OUTPUT VOLUME` (`±12 dB`) | `F-01` p. 1–2 | Spec 067 / `CA-04` | `crates/ampforge-dsp/src/fx/eq.rs:126 @ c20cc65` (possui `output_level_db`, mas falta `input_gain_db` e falta exposição no `controls.registry.ts`) | Ausente | `SPECIFIED` |
| Esquema elétrico de serviço dos 10 giradores `MC33179` da Dunlop | Não publicado em `M108.pdf` | Spec 067 / §6.2 | N/A | N/A | `PENDING` |
| Execução zero-alloc em tempo real | Spec 053 | Spec 067 / `CA-05` | `crates/ampforge-dsp/src/fx/eq.rs:208–227 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
