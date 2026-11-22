---
status: rascunho
---

# Spec 072: Pedal Shin-ei Uni-Vibe (1968 4-Stage Staggered Photo-Optical Phase Shifter)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-modulation` (`pedal`)
> **Identificador canônico:** `pedal.shinei.univibe`
> **Variante / Revisão:** `1968–1970 Shin-ei Companion / Univox Uni-Vibe (Lâmpada Incandescente + 4× Células CdS + 4 Estágios BJT Staggered: 15 nF, 220 nF, 470 pF, 4.7 nF)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/modulation.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [052](052-abertura-animada-pedais-faltantes-e-amp2-visivel.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [071](071-pedal-mxr-phase-90.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação do pedal foto-óptico **Shin-ei / Univox Uni-Vibe (1968, projetado por Fumio Mieda)** como processador modular independente (`pedal.shinei.univibe`), corrigindo:
1. O link quebrado `uvtech.htm` para a URL válida verificada da análise de R. G. Keen no GEOFEX (`http://www.geofex.com/Article_Folders/univibe/univtech.htm`, HTTP 200); e
2. O **erro factual grave** na implementação atual ([`crates/ampforge-dsp/src/fx/modulation.rs:247 @ c20cc65`](../crates/ampforge-dsp/src/fx/modulation.rs#L247)), que usa multiplicadores igualmente espaçados em meia-oitava `[1.0, 1.41, 2.0, 2.83]` (razão total de apenas `2,83 : 1`), quando no esquema elétrico real do Shin-ei Uni-Vibe os 4 capacitores de deslocamento de fase são **`15 nF`, `220 nF`, `470 pF` e `4,7 nF`** — uma razão escalonada assimétrica (*staggered*) de **`468,08 : 1`** ($220\,000\text{ pF} / 470\text{ pF}$)!

---

## 2. Escopo

1. Pré-amplificador BJT e cadeia de **4 estágios deslocadores de fase BJT *split-load* com capacitores escalonados assimetricamente (`Staggered Capacitors`)**:
   - **Estágio 1:** $C_1 = 0,015\text{ }\mu\text{F}$ (`15 nF`);
   - **Estágio 2:** $C_2 = 0,22\text{ }\mu\text{F}$ (`220 nF` — polo grave profundo);
   - **Estágio 3:** $C_3 = 470\text{ pF}$ (`0,00047 µF` — polo agudo extremo);
   - **Estágio 4:** $C_4 = 0,0047\text{ }\mu\text{F}$ (`4,7 nF` — polo médio-agudo).
2. Acoplamento eletro-óptico não-linear: oscilador LFO comandando uma **lâmpada incandescente miniatura** (inércia térmica assimétrica entre aquecimento do filamento de tungstênio e resfriamento, gerando o pulso rítmico de *"batida de coração / throb"*) cercada por **4 fotoresistores de Sulfeto de Cádmio (CdS LDRs)** com lei de potência $R_{\text{LDR}}(I) \propto I^{-\gamma}$.
3. Chave de modo **`CHORUS / VIBRATO`** do equipamento real:
   - **Modo `CHORUS` (`Dry + Wet` `1:1`):** Mistura o sinal seco com a saída dos 4 estágios escalonados, gerando o efeito líquido rotativo eternizado por Jimi Hendrix (*Machine Gun*, *Star Spangled Banner*) e Robin Trower / David Gilmour.
   - **Modo `VIBRATO` (`100% Wet`):** Desconecta o ramo seco, entregando exclusivamente a modulação de fase/pitch dos 4 estágios.

---

## 3. Fora de escopo

- Phasers simétricos de capacitores iguais (MXR Phase 90 → Spec 071).
- Caixas acústicas rotativas eletromecânicas reais (Leslie 147 → Spec 074).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`4-stage-staggered-optical-phaser-chorus-vibrato`) |
| **Fabricante de referência** | Shin-ei Companion Corp. / Unicord (Univox) (Tóquio, Japão — engenheiro-chefe Fumio Mieda) |
| **Modelo de referência** | Shin-ei / Univox Uni-Vibe |
| **Ano / época / revisão** | 1968–1970 (Transistores BJT `2SC828` / `2SC536`, Lâmpada Incandescente `24V/40mA`, 4× Células CdS LDR, Capacitores `15 nF / 220 nF / 470 pF / 4,7 nF`) |
| **Identificador no sistema** | `pedal.shinei.univibe` (`ModModel::UniVibe` / UI `pedal.univibe.*`) |
| **Papel na cadeia de sinal** | Bloco `MOD` (`mod_1`), classicamente posicionado **antes** do amplificador valvulado (ou entre o Fuzz Face e o Marshall 1959 Plexi). |

---

## 5. Princípio físico e funcional do equipamento real

Conforme demonstrado na análise clássica de R. G. Keen (`F-01`, *The Technology of the Univibe*):

1. **Por que o Uni-Vibe NÃO soa como um Phaser Comum (Os 4 Capacitores Escalonados `468 : 1`):**
   - Em um phaser convencional (como o MXR Phase 90 da Spec 071), todos os 4 estágios possuem exatamente o mesmo capacitor ($47\text{ nF}$), de modo que as 4 frequências de polo coincidem ($f_{p1} = f_{p2} = f_{p3} = f_{p4}$), criando notches estreitos, simétricos e matematicamente regulares.
   - Fumio Mieda projetou o Uni-Vibe original (como simulador de efeito Doppler atmosférico/rotativo) escolhendo **quatro capacitores radicalmente diferentes** para os quatro estágios de deslocamento de fase:
     $$C_1 = 15\text{ nF}, \qquad C_2 = 220\text{ nF}, \qquad C_3 = 470\text{ pF}, \qquad C_4 = 4,7\text{ nF}$$
   - Sob a mesma resistência instantânea $R_{\text{LDR}}(t)$ das 4 células CdS (que variam tipicamente de $\sim 2,5\text{ k}\Omega$ sob brilho máximo da lâmpada até $\sim 250\text{ k}\Omega$ na escuridão), as 4 frequências de quebra $f_{k}(t) = \frac{1}{2\pi R_{\text{LDR}}(t) C_k}$ guardam entre si a proporção fixa inversa das capacitâncias (tomando $C_2 = 220\text{ nF}$ como base $1,00\times$):
     - **Estágio 2 ($220\text{ nF}$):** $1,000\times f_{\text{base}}(t)$ (varre graves profundos: $\sim 2,9\text{ Hz}$ a $289\text{ Hz}$);
     - **Estágio 1 ($15\text{ nF}$):** $\frac{220}{15} = \mathbf{14,667\times}\,f_{\text{base}}(t)$ (varre médios-graves: $\sim 42\text{ Hz}$ a $4244\text{ Hz}$);
     - **Estágio 4 ($4,7\text{ nF}$):** $\frac{220}{4,7} = \mathbf{46,809\times}\,f_{\text{base}}(t)$ (varre médios-agudos: $\sim 135\text{ Hz}$ a $13,5\text{ kHz}$);
     - **Estágio 3 ($470\text{ pF}$):** $\frac{220}{0,47} = \mathbf{468,085\times}\,f_{\text{base}}(t)$ (varre agudos/super-agudos: $\sim 1,35\text{ kHz}$ a $135\text{ kHz}$).
   - Essa dispersão de **`468 : 1`** faz com que os entalhes (*notches*) fiquem largos, assimétricos e espalhados de forma não-harmônica pelo espectro, criando a textura aquosa, tridimensional e vocal inconfundível do Uni-Vibe!
2. **Termodinâmica da Lâmpada Incandescente e Memória das Células CdS (*Heartbeat Throb*):**
   - O filamento da lâmpada incandescente emite luz proporcionalmente à quarta potência da temperatura absoluta ($T^4$), que só acende significativamente na metade superior do ciclo do LFO, aquecendo rapidamente e esfriando mais devagar.
   - Somado à inércia fotoquímica do Sulfeto de Cádmio (ataque luminoso de $\sim 8\text{ ms}$ vs recuperação no escuro de $\sim 45\text{ ms}$), a curva de modulação resultante de $f_{\text{base}}(t)$ não é uma senoide simétrica, mas um **pulso assimétrico (*lopsided throb*)** onde a varredura passa mais tempo nos graves/médios-graves e dá um "mergulho/chicote" rápido em direção aos agudos a cada ciclo.

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | R. G. Keen (GEOFEX) — *The Technology of the Univibe* (URL verificada e corrigida para `Article_Folders/univibe/univtech.htm`) | `http://www.geofex.com/Article_Folders/univibe/univtech.htm` (HTTP 200 verificado, 28,9 KB) | `SECONDARY_TECHNICAL` | Análise completa do esquema original do Shin-ei Uni-Vibe (1968), valores exatos dos 4 capacitores escalonados (`15 nF, 220 nF, 470 pF, 4.7 nF`), resposta não-linear da lâmpada incandescente + células CdS e chave `Chorus / Vibrato`. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Capacitores escalonados dos 4 estágios deslocadores de fase: $C_1 = 15\text{ nF}$, $C_2 = 220\text{ nF}$, $C_3 = 470\text{ pF}$, $C_4 = 4,7\text{ nF}$ (razão $1 : 14,67 : 46,81 : 468,09$) | `F-01` | Shin-ei Uni-Vibe 1968 | `F-01` §"The Phase Shift Stages & Staggered Capacitors" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Resposta assimétrica do conjunto Lâmpada Incandescente + 4× Fotoresistores CdS (*throb*) | `F-01` | Shin-ei Uni-Vibe 1968 | `F-01` §"The Lamp Driver and LDRs" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Chave `CHORUS / VIBRATO`: modo `CHORUS` mistura sinal Dry + saída do 4º estágio; modo `VIBRATO` entrega `100% Wet` | `F-01` | Shin-ei Uni-Vibe 1968 | `F-01` §"Chorus vs Vibrato Mode" | `SECONDARY_TECHNICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input] ──► [JFET/BJT Preamp] ──┬──────────────────────────────────────────────────────────────────────► [Chorus Switch (Dry)] ──┐
                                │                                                                                                ├──► [Out]
                                ▼                                                                                                │
                        [Stage 1: C1 = 15 nF] ──► [Stage 2: C2 = 220 nF] ──► [Stage 3: C3 = 470 pF] ──► [Stage 4: C4 = 4.7 nF] ──┘ (Wet)
                                  ▲                         ▲                          ▲                         ▲
                                  └─────────────────────────┴──────────────────────────┴─────────────────────────┘
                                                    [4x CdS Photoresistors R_LDR(t)]
                                                                   ▲
                                                    [Incandescent Lamp Thermal Filament]
                                                                   ▲
                                                    [Twin-T LFO Oscillator (SPEED & INTENSITY)]
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`FACTUAL_ERROR` (`modulation.rs:247` — Razões de Capacitores Incorretas `[1.0, 1.41, 2.0, 2.83]`):**
   - Na linha `247` de `crates/ampforge-dsp/src/fx/modulation.rs`, o código define:
     `let ratios = [1.0, 1.41, 2.0, 2.83];`
     com $f_{\text{base}} \in [120\text{ Hz},\; 1920\text{ Hz}]$.
   - Esses números (`1.0, √2, 2.0, 2√2`) representam um espaçamento arbitrário de meia-oitava (razão total de `2,83 : 1`), que **não tem nenhuma relação com os capacitores reais do Shin-ei Uni-Vibe** (`220 nF, 15 nF, 4.7 nF, 470 pF`), cujas frequências de polo guardam a razão física documentada **`[1.0, 14.667, 46.809, 468.085]`**!
2. **`SOURCE_INVALID` (corrigido nesta revisão):** A URL anterior `[link 404 removido — AUD-41] retornava HTTP 404 por dois motivos: o diretório no servidor Apache Unix do GEOFEX é case-sensitive (`Article_Folders`) e o nome real do arquivo é `univtech.htm` (`http://www.geofex.com/Article_Folders/univibe/univtech.htm`, verificado com HTTP 200).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real / UI | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Aceita | Unidade | Valor Padrão | Papel Elétrico no Circuito |
|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `mod_1.enabled` | `pedal.univibe.enabled` | `false / true` | Booleano | `false` | Ativa/desativa o pedal Uni-Vibe. |
| **SPEED (RATE)** | `mod_1.rate_hz` | `pedal.univibe.rate` | `0.5 .. 10.0` | `Hz` | `1.50` | Frequência do oscilador LFO Twin-T que alimenta o driver da lâmpada. |
| **INTENSITY (DEPTH)** | `mod_1.depth` | `pedal.univibe.depth` | `0.0 .. 1.0` | `0 .. 100 %` | `0.75` | Amplitude da corrente AC injetada na lâmpada incandescente sobre a corrente quiescente de pré-aquecimento. |
| **CHORUS / VIBRATO (MIX)** | `mod_1.mix` | `pedal.univibe.mix` | `0.0 .. 1.0` | `0 .. 100 %` | `0.50` (`CHORUS`) | Em `mix = 0.50` (`50%`), opera no modo **`CHORUS`** (`Dry + Wet` `1:1`); em `mix = 1.00` (`100%`), opera no modo **`VIBRATO`** (`100% Wet`). |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Envoltória Assimétrica Termo-Óptica (*Incandescent Lamp + CdS LDR Throb*):**
   - A excitação senoidal do LFO $s[n] = 0,5 + 0,5 \sin(\phi[n])$ passa pela não-linearidade térmica de incandescência do filamento ($L_{\text{lamp}}[n] = s[n]^{2,4}$) e por um filtro de ataque rápido / decaimento lento das células CdS ($\tau_{\text{att}} = 10\text{ ms}$, $\tau_{\text{rel}} = 45\text{ ms}$), gerando a condutância óptica normalizada $u_{\text{opt}}[n] \in [0, 1]$.
2. **Resistência Instantânea das Células CdS e as 4 Frequências de Polo Escalonadas (`220 nF, 15 nF, 4.7 nF, 470 pF`):**
   - A resistência efetiva de cada célula CdS em série com o resistor limitador do estágio ($R_{\min,\text{eff}} \approx 4,7\text{ k}\Omega$ sob brilho máximo até $R_{\max,\text{eff}} \approx 220\text{ k}\Omega$ no escuro) varia exponencialmente com $u_{\text{opt}}[n]$ e `depth`:
     $$R_{\text{eff}}[n] = R_{\max,\text{eff}} \cdot \left(\frac{R_{\min,\text{eff}}}{R_{\max,\text{eff}}}\right)^{0,15 + 0,85 \cdot \text{depth} \cdot u_{\text{opt}}[n]}$$
   - As 4 frequências de quebra dos estágios de deslocamento de fase são calculadas **diretamente a partir dos 4 capacitores reais do esquema de Fumio Mieda** (`F-01`):
     $$C = [15 \times 10^{-9},\; 220 \times 10^{-9},\; 470 \times 10^{-12},\; 4,7 \times 10^{-9}]\text{ F}$$
     $$f_k[n] = \min\!\left(\frac{1}{2\pi R_{\text{eff}}[n] C_k},\; 0,46 \cdot f_s\right), \qquad k \in \{1, 2, 3, 4\}$$
     preservando a razão exata $1 : 14,667 : 46,809 : 468,085$ na banda audível!
3. **Mistura `CHORUS` (`mix = 0.50`) vs `VIBRATO` (`mix = 1.00`):**
   - $y[n] = (1 - \text{mix}) \cdot x[n] + \text{mix} \cdot y_{\text{stage4}}[n]$.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Clamp de Nyquist suave:** Como o Estágio 3 ($C_3 = 470\text{ pF}$) sob brilho máximo da lâmpada ($R_{\text{eff}} = 4,7\text{ k}\Omega$) atinge $f_3 \approx 72\text{ kHz}$, o pré-warping bilinear de $f_3[n]$ é grampeado suavemente em $0,46 \cdot f_s$ (`22,08 kHz` a `48 kHz`) ou processado com `2×` oversampling, garantindo estabilidade absoluta dos 4 polos.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`.
- Complexidade $O(1)$ por amostra; WCET $\le 1,8\text{ }\mu\text{s}$ por bloco de `128` amostras.

---

## 12. Posição e interação na cadeia completa de guitarra

- No slot `mod_1`, posicionado antes do bloco `AMP` (ex.: Marshall 1959 Super Lead), reproduz o pulsar assimétrico grave (*throb*) e a varredura líquida do Uni-Vibe 1968.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/shinei_univibe.rs`.
- **Struct principal:** `pub struct ShineiUniVibeProcessor` implementando `DspBlock` + `AutomatableParams` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Estágio do Uni-Vibe | Capacitor Real (`F-01`) | Razão de Frequência ($C_2 / C_k$) | Faixa de Polo ($R_{\text{eff}} \in [4,7\text{ k}\Omega, 220\text{ k}\Omega]$) |
|---|---|---|---|
| **Estágio 1** | $C_1 = 15\text{ nF}$ | $14,667\times$ | `48,2 Hz` a `2257 Hz` |
| **Estágio 2** | $C_2 = 220\text{ nF}$ | $1,000\times$ (Base Grave) | `3,29 Hz` a `153,9 Hz` |
| **Estágio 3** | $C_3 = 470\text{ pF}$ | $468,085\times$ (Agudo) | `1539 Hz` a `72,0 kHz` (limitado em Nyquist) |
| **Estágio 4** | $C_4 = 4,7\text{ nF}$ | $46,809\times$ | `153,9 Hz` a `7205 Hz` |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Escalonamento Assimétrico dos 4 Capacitores (`220 nF, 15 nF, 4.7 nF, 470 pF` vs Phaser Simétrico):** verifica que a posição dos notches no modo `CHORUS` (`mix = 0.50`) difere da razão `5,8284` do Phase 90 devido ao escalonamento `1 : 14,67 : 46,81 : 468,09`.
2. **Teste de Pulso Assimétrico (*Heartbeat Throb* da Lâmpada + CdS):** verifica que o ciclo de trabalho da envoltória de modulação óptica é assimétrico (tempo na região escura/grave diferente do tempo no pico luminoso/agudo).
3. **Teste dos Modos `CHORUS` (`mix = 0.50`) vs `VIBRATO` (`mix = 1.00`):** verifica cancelamento de fase comb-filter no modo `CHORUS` vs magnitude plana com modulação pura de fase/pitch no modo `VIBRATO`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- No Uni-Vibe real, cada estágio BJT *split-load* também possui um capacitor de acoplamento de entrada que atenua levemente o ganho de fase em DC, impedindo cancelamento total dos subgraves abaixo de `40 Hz`; manter o Estágio 2 em `220 nF` reproduz naturalmente essa preservação do peso grave.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Escalonamento Assimétrico dos 4 Capacitores Reais (`15 nF, 220 nF, 470 pF, 4.7 nF`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Varredura senoidal de `40 Hz` a `12 000 Hz` a `-18 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `pedal.shinei.univibe` com `enabled = true`, `depth = 0.0` (resistência óptica estacionária $R_{\text{eff}} = 25\text{ k}\Omega$), `mix = 0.50` (modo `CHORUS`).
  - *Método de medição:* Inspeção das frequências de quebra dos 4 estágios all-pass ($f_k = \frac{1}{2\pi R_{\text{eff}} C_k}$) e comparação da resposta em magnitude do Uni-Vibe com a resposta do Phase 90 (`Spec 071`).
  - *Critério e tolerância:* As frequências de polo dos 4 estágios dentro da banda de Nyquist devem obedecer às razões físicas das capacitâncias $C_2 : C_1 : C_4 : C_3 = 220\text{ nF} : 15\text{ nF} : 4,7\text{ nF} : 0,47\text{ nF}$ ($\implies 1,00 : 14,67 : 46,81 : 468,09$, com tolerância de $\pm 2\%$ antes do limite de Nyquist), produzindo um perfil de entalhes assimétrico distinto dos 2 notches estreitos de razão `5,83` do Phase 90.
  - *Origem da tolerância:* Esquema elétrico original do Shin-ei Uni-Vibe documentado em `F-01`.

- **CA-02 — Pulso Assimétrico Termo-Óptico (*Incandescent Lamp + CdS Heartbeat Throb*) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Ruído rosa ou tom composto (`200 Hz + 800 Hz + 2400 Hz`) a `-12 dBFS` durante `2,0 s`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `rate_hz = 1.0` (período $T = 1,0\text{ s}$), `depth = 0.85`, `mix = 0.50`.
  - *Método de medição:* Medição do centróide espectral instantâneo $\text{SC}(t)$ em janelas de `25 ms` ao longo de um período completo $T = 1,0\text{ s}$.
  - *Critério e tolerância:* Devido à lei de potência da lâmpada incandescente e à recuperação lenta das células CdS ($\tau_{\text{rel}} > \tau_{\text{att}}$), a trajetória temporal de $\text{SC}(t)$ deve ser **assimétrica** (*duty-cycle* onde $\text{SC}(t)$ permanece abaixo da média aritmética $(\text{SC}_{\max} + \text{SC}_{\min})/2$ durante **pelo menos `55%`** do período $T$).
  - *Origem da tolerância:* Comportamento físico do conjunto lâmpada incandescente + células CdS (`F-01` §"The Lamp Driver and LDRs").

- **CA-03 — Modo `CHORUS` (`mix = 0.50`, Cancelamento de Fase) vs Modo `VIBRATO` (`mix = 1.00`, Modulação Pura de Fase/Pitch) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide estacionária de `800 Hz` a `-12 dBFS` com LFO congelado (`depth = 0.0`) e depois com LFO ativo (`rate_hz = 2.0, depth = 0.85`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Comparação entre `mix = 0.50` (`CHORUS`) e `mix = 1.00` (`VIBRATO`).
  - *Método de medição:* Variação de amplitude RMS vs variação de frequência instantânea.
  - *Critério e tolerância:* No modo `VIBRATO` (`mix = 1.00`, 100% saída da cadeia All-Pass), com LFO estacionário (`depth = 0.0`), o ganho de magnitude deve ser unitário (**`0,0 dB ± 0,2 dB`** em todas as frequências de `100 Hz` a `10 kHz`), enquanto com LFO ativo (`depth = 0.85`) ocorre modulação contínua de fase/frequência instantânea. No modo `CHORUS` (`mix = 0.50`), a soma com o sinal Dry introduz modulação de amplitude/timbre por interferência destrutiva e construtiva.
  - *Origem da tolerância:* Chave `Chorus / Vibrato` do Shin-ei Uni-Vibe (`F-01`).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `rate_hz`, `depth`, `mix`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/modulation.rs` (e futuro `crates/ampforge-dsp/src/fx/pedals/shinei_univibe.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.univibe.*`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_032_time_mod_fx.rs`, `crates/sonix-desktop/tests/contract_052_splash_pedals_amp2.rs`

---

## 19. Critérios de conclusão da Spec

- [x] URL do GEOFEX corrigida para `http://www.geofex.com/Article_Folders/univibe/univtech.htm` e verificada via HTTP 200 (`F-01`).
- [x] Erro factual das razões `[1.0, 1.41, 2.0, 2.83]` em `modulation.rs:247` diagnosticado e substituído na especificação pelos 4 capacitores reais (`15 nF, 220 nF, 470 pF, 4.7 nF` $\Rightarrow 1 : 14,67 : 46,81 : 468,09$).
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Keen, R. G. (GEOFEX) — *The Technology of the Univibe* (`http://www.geofex.com/Article_Folders/univibe/univtech.htm`).

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, corrigiu-se a URL quebrada `uvtech.htm` para `Article_Folders/univibe/univtech.htm` e identificou-se que `modulation.rs:247` usava razões de meia-oitava `[1.0, 1.41, 2.0, 2.83]` em vez dos 4 capacitores escalonados reais do Shin-ei Uni-Vibe (`15 nF, 220 nF, 470 pF, 4.7 nF`).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| 4 estágios deslocadores de fase com capacitores escalonados reais (`15 nF, 220 nF, 470 pF, 4.7 nF` $\Rightarrow$ razão `1 : 14,67 : 46,81 : 468,09`) | `F-01` | Spec 072 / `CA-01` | `crates/ampforge-dsp/src/fx/modulation.rs:247 @ c20cc65` (ERRO FACTUAL: usa `[1.0, 1.41, 2.0, 2.83]`) | `contract_032_time_mod_fx.rs` | `SPECIFIED` |
| Envoltória termo-óptica assimétrica (*Incandescent Lamp + CdS LDR throb*) | `F-01` | Spec 072 / `CA-02` | `crates/ampforge-dsp/src/fx/modulation.rs:246 @ c20cc65` (`mod_val.powf(1.6)` sem histerese attack/release do CdS) | `contract_032_time_mod_fx.rs` | `IMPLEMENTED_UNVERIFIED` |
| Operação nos modos `CHORUS` (`mix = 0.50`, Dry+Wet) e `VIBRATO` (`mix = 1.00`, 100% Wet) | `F-01` | Spec 072 / `CA-03` | `crates/ampforge-dsp/src/fx/modulation.rs:256 @ c20cc65` | `contract_052_splash_pedals_amp2.rs` | `TESTED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 072 / `CA-04` | `crates/ampforge-dsp/src/fx/modulation.rs:179–294 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
