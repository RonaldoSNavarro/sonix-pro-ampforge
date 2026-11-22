---
status: rascunho
---

# Spec 069: Pedal Boss CE-2 Chorus (1979 Analog Bucket-Brigade Device MN3007/MN3101)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade — Desacoplamento do Tri-Chorus)
> **Domínio:** `dsp-modulation` (`pedal`)
> **Identificador canônico:** `pedal.boss.ce2_chorus`
> **Variante / Revisão:** `1979–1982 Made in Japan Silver/Black Screw (1024-Stage BBD Matsushita MN3007 + Clock Generator MN3101)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/modulation.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [052](052-abertura-animada-pedais-faltantes-e-amp2-visivel.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [087](087-modulo-stereo-tri-chorus.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação **exclusivamente** do pedal **Boss CE-2 Chorus (1979, Matsushita `MN3007` 1024-stage BBD + `MN3101` Clock Generator)** como processador modular independente (`pedal.boss.ce2_chorus`), desfazendo o agrupamento indevido (`MODEL_COUPLING`) da versão anterior desta Spec — que misturava o Boss CE-2 analógico mono de 1 linha BBD com o algoritmo **Stereo Tri-Chorus** de 3 linhas a $0^\circ / 120^\circ / 240^\circ$ (agora isolado na **[Spec 087](087-modulo-stereo-tri-chorus.md)**) — e distinguindo com rigor o circuito físico original do CE-2 (que possui apenas **2 potenciômetros: `RATE` e `DEPTH`**, soma resistiva fixa 50/50 Dry/Wet e redes complementares de Pré-Ênfase/De-Ênfase) das extensões digitais do AMPFORGE (controle `Mix`).

---

## 2. Escopo

1. Circuito analógico do **Boss CE-2 (1979)** baseado no chip Bucket-Brigade Device de `1024` estágios **`MN3007`** acionado pelo gerador de clock bifásico CMOS **`MN3101`**.
2. Redes complementares de **Pré-Ênfase** (`+15 dB` shelving de agudos em `IC1A` antes do BBD) e **De-Ênfase** (`-15 dB` shelving de agudos na saída do BBD) combinadas com os filtros ativos passa-baixas anti-aliasing / reconstrução de **5ª ordem** (`Q2`/`Q3` Sallen-Key + RC em $f_c \approx 6,8..7,2\text{ kHz}$) no ramo *wet*, enquanto o ramo *dry* permanece com banda completa sem passar pelo BBD.
3. Oscilador de baixa frequência (LFO) **Triangular** (`IC2 TL022CP`, $f_{\text{LFO}} \in [0,4\text{ Hz},\; 4,5\text{ Hz}]$) modulando a frequência de clock do `MN3101` ($f_{\text{clk}} \in [40\text{ kHz},\; 100\text{ kHz}] \implies \tau_{\text{BBD}} = \frac{512}{f_{\text{clk}}} \in [5,12\text{ ms},\; 12,8\text{ ms}]$).
4. Distinção explícita entre os controles do equipamento físico (`Rate`, `Depth`, mistura fixa `1:1` por resistores de `47 kΩ`) e a extensão de conveniência digital `Mix` (`mod_1.mix`).

---

## 3. Fora de escopo

- **Módulo Stereo Tri-Chorus (`3× BBD` defasados em $120^\circ$ estilo DyTronics CS-5 / Fulltone Choralflange):** Movido integralmente para a **[Spec 087](087-modulo-stereo-tri-chorus.md)**.
- **Boss CE-1 Chorus Ensemble (1976, `MN3002` 512-stage BBD, pré-amplificador com ganho de entrada variável e modo Vibrato)** ou **Boss CE-2W Waza Craft** (que adiciona chave CE-1/Vibrato e saída estéreo por inversão de fase).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`analog-bbd-chorus`) |
| **Fabricante de referência** | Boss Corporation / Roland Corporation (Osaka / Hamamatsu, Japão) |
| **Modelo de referência** | Boss CE-2 Chorus |
| **Ano / época / revisão** | Outubro de 1979–1982 (MIJ Silver Screw / Black Screw; `IC1 NJM4558D`, `IC2 TL022CP`, `IC3 MN3007`, `IC4 MN3101`) |
| **Identificador no sistema** | `pedal.boss.ce2_chorus` (`ModModel::ChorusCe2` / UI `pedal.chorus.*`) |
| **Papel na cadeia de sinal** | Bloco `MOD` (`mod_1`), posicionado antes do amplificador (estilo anos 80) ou no FX Loop pós-pré-amplificador. |

---

## 5. Princípio físico e funcional do equipamento real

De acordo com a análise esquemática do Boss CE-2 (`F-01`) e o manual oficial da Boss (`F-02`):

1. **Ramo Seco (*Dry*) Puro vs Ramo Molhado (*Wet*) com Pré-Ênfase e De-Ênfase Complementares:**
   - Após o buffer de entrada `Q1`, o sinal divide-se em dois caminhos:
     - **Ramo Dry:** Passa diretamente por um filtro acoplador até o resistor somador $R_{19} = 47\text{ k}\Omega$ na entrada de `IC1B`, preservando `100%` dos agudos originais da guitarra.
     - **Ramo Wet (BBD):** Entra no amplificador operacional `IC1A` (`NJM4558D`), que aplica uma **Pré-Ênfase de Agudos** ($R_6 = 47\text{ k}\Omega, R_7 = 6,8\text{ k}\Omega, C_5 = 0,015\text{ }\mu\text{F} \Rightarrow f_z \approx 225\text{ Hz}, f_p \approx 1560\text{ Hz}$, elevando agudos em **`+15,4 dB`**) para colocar o sinal acima do piso de ruído de chaveamento dos `1024` capacitores MOS do `MN3007`.
     - Na saída dos pinos `7` e `8` do `MN3007`, uma rede simétrica de **De-Ênfase** ($-15,4\text{ dB}$ acima de `225 Hz`) restaura o balanço tonal plano do sinal atrasado enquanto enterra o chiado (*BBD clock hiss*) do `MN3007` em `-15,4 dB`!
2. **Filtros Passa-Baixas Ativos de 5ª Ordem Anti-Aliasing e Reconstrução (`~6,8 kHz`):**
   - Antes da entrada do `MN3007` (pino `3`), um filtro ativo `Q2` (`2SC945`) de 3ª ordem ($f_c \approx 7,0\text{ kHz}$) evita aliasing na amostragem analógica do BBD.
   - Após a saída do `MN3007` (pinos `7` e `8`), um filtro ativo `Q3` (`2SC945`) Sallen-Key de 2ª/3ª ordem ($f_c \approx 6,8\text{ kHz}$) remove a portadora de clock do `MN3101`.
3. **Relação Fundamental do BBD de `1024` Estágios (`MN3007`) e LFO Triangular (`TL022CP`):**
   - Um BBD de $N_{\text{stages}} = 1024$ estágios comandado por um clock bifásico de frequência $f_{\text{clk}}$ transfere pacotes de carga a cada meio ciclo, resultando em um atraso físico exato de:
     $$\tau_{\text{BBD}}(t) = \frac{N_{\text{stages}}}{2 \cdot f_{\text{clk}}(t)} = \frac{512}{f_{\text{clk}}(t)}$$
   - O gerador `TL022CP` (`IC2`) é um integrador + Schmitt Trigger que produz uma **onda estritamente triangular** (não senoidal!), evitando que o pitch estagne nos picos da excursão. No CE-2 real, a frequência do LFO varia de $\sim 0,4\text{ Hz}$ (`Rate = 0%`) a $\sim 4,5\text{ Hz}$ (`Rate = 100%`), modulando o atraso do `MN3007` em torno do centro $\tau_0 \approx 7,5\text{ ms}$ (excursão total de `5,0 ms` a `12,8 ms`).
4. **Distinção entre o Pedal Físico Real e a Extensão Digital do AMPFORGE:**
   - **Equipamento Real (Boss CE-2):** Possui **apenas 2 knobs** (`RATE` e `DEPTH`), saída **Mono**, e soma fixa **50/50 (`1:1`)** entre o ramo Dry ($R_{19} = 47\text{ k}\Omega$) e o ramo Wet ($R_{18} = 47\text{ k}\Omega$) no estágio `IC1B`.
   - **Extensão Digital AMPFORGE:** O chassi da UI (`pedal.chorus.mix` em `controls.registry.ts:893`) expõe um 3º knob `MIX` (`0..100%`). Quando `Mix = 50%` (`0.50`, valor padrão), o processador reproduz exatamente a proporção `1:1` ($47\text{ k}\Omega / 47\text{ k}\Omega$) do hardware real; valores acima de `50%` permitem chegar a `100% Wet` (modo *Pitch Vibrato* estilo CE-1/CE-2W).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | ElectroSmash — *Boss CE-2 Chorus Analysis* (URL corrigida com hífen `ce-2` e verificada via Wayback Machine) | `https://web.archive.org/web/20240115000000/https://www.electrosmash.com/boss-ce-2-analysis` (HTTP 200 verificado) | `SECONDARY_TECHNICAL` | Esquema completo do Boss CE-2 (1979), equações de Pré-Ênfase/De-Ênfase (`225 Hz / 1560 Hz, ±15 dB`), filtros anti-aliasing `Q2/Q3` (`~7 kHz`), LFO triangular `TL022CP` e linha BBD `MN3007` + `MN3101`. |
| `F-02` | Boss Corporation — *Boss CE-2 Chorus Owner's Manual & Specifications* | `https://www.boss.info/global/support/by_product/ce-2/owners_manuals/` (HTTP 200 oficial verificado) | `PRIMARY_OFFICIAL` | Controles oficiais do CE-2 (`RATE`, `DEPTH`), alimentação ACA e especificações de fábrica. |
| `F-03` | Matsushita Electric Industrial (Panasonic) — *MN3007 1024-Stage Low Noise BBD & MN3101 Clock Generator Datasheets* | Especificação oficial dos componentes (`1024 stages`, $\tau = 512 / f_{\text{clk}}$, `S/N = 80 dB`, `THD = 0.5%`) | `PRIMARY_COMPONENT` | Equação física de atraso $\tau = 512 / f_{\text{clk}}$ e faixa de clock `10 kHz – 100 kHz`. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| BBD `MN3007` de 1024 estágios com atraso $\tau = 512 / f_{\text{clk}} \in [5,12\text{ ms},\; 12,8\text{ ms}]$ para $f_{\text{clk}} \in [40\text{ kHz},\; 100\text{ kHz}]$ | `F-01`, `F-03` | Boss CE-2 1979 | `F-01` §"BBD Stage (MN3007 & MN3101)"; `F-03` p. 1 | `SECONDARY_TECHNICAL` / `PRIMARY_COMPONENT` | `CONFIRMED` |
| Redes complementares de Pré-Ênfase (`+15,4 dB`) e De-Ênfase (`-15,4 dB`) entre `225 Hz` e `1560 Hz` + filtros anti-aliasing/reconstrução `~6,8 kHz` exclusivamente no ramo Wet | `F-01` | Boss CE-2 1979 | `F-01` §"Pre-Emphasis and De-Emphasis / Anti-Aliasing Filters" | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| LFO de onda triangular (`TL022CP`) e soma fixa 50/50 Dry/Wet ($R_{18}=R_{19}=47\text{ k}\Omega$) com apenas 2 knobs físicos (`Rate`, `Depth`) | `F-01`, `F-02` | Boss CE-2 1979 | `F-01` §"LFO Stage & Output Mixer"; `F-02` | `SECONDARY_TECHNICAL` / `UNKNOWN` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input] ──► [Q1 Input Buffer] ──┬──────────────────────────────────────────────────────────────────────► [R19 47kΩ Dry] ──┐
                                │                                                                                         │
                                ▼ (Wet BBD Branch)                                                                        ├──► [IC1B Output Mixer] ──► [Output]
                      [IC1A Pre-Emphasis (+15.4 dB > 225 Hz)]                                                             │
                                │                                                                                         │
                                ▼                                                                                         │
                      [Q2 3rd-Order Anti-Aliasing LPF (~7.0 kHz)]                                                         │
                                │                                                                                         │
                                ▼                                                                                         │
                      [IC3 MN3007 1024-Stage BBD (τ = 512 / f_clk)] ◄── [IC4 MN3101 Clock] ◄── [IC2 TL022 Triangular LFO] │
                                │                                                               (RATE & DEPTH Pots)       │
                                ▼                                                                                         │
                      [Q3 Reconstruction LPF (~6.8 kHz) + De-Emphasis (-15.4 dB)] ─────────────────────► [R18 47kΩ Wet] ──┘
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/modulation.rs:195–218 @ c20cc65`](../crates/ampforge-dsp/src/fx/modulation.rs#L195-L218) apresenta as seguintes divergências auditadas:

1. **`MODEL_COUPLING` (`modulation.rs:196–218`):** O braço `ModModel::ChorusCe2` em `modulation.rs:196–218` implementa na verdade um **Tri-Chorus de 3 fases senoidais** (`p1 = 0`, `p2 = 1/3`, `p3 = 2/3` a $120^\circ$, somando `(w1 + w2 + w3) / 3.0`) em torno de `15 ms ± 6 ms`!
   - Isso **não é o circuito do Boss CE-2**! O Boss CE-2 real possui **uma única linha BBD (`MN3007`)** modulada por um **LFO Triangular** em torno de $\tau_0 \approx 7,5\text{ ms}$ (`5,12..12,8 ms`).
   - Nesta rodada corretiva, o **Stereo Tri-Chorus (3 linhas defasadas em $120^\circ$)** foi separado em sua própria especificação (**[Spec 087](087-modulo-stereo-tri-chorus.md)**), enquanto a **Spec 069** especifica exclusivamente o circuito autêntico de linha única BBD `MN3007` do **Boss CE-2**.
2. **`SOURCE_INVALID` (corrigido nesta revisão):** O link anterior `https://www.electrosmash.com/boss-ce2-analysis` não possuía o hífen em `ce-2` (retornando HTTP 404 mesmo no arquivo histórico). O slug correto verificado com HTTP 200 no Wayback Machine é `https://web.archive.org/web/20240115000000/https://www.electrosmash.com/boss-ce-2-analysis` (`F-01`).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real / UI | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Aceita | Unidade | Valor Padrão | Natureza (Hardware Real vs Extensão AMPFORGE) |
|---|---|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `mod_1.enabled` | `pedal.chorus.enabled` | `false / true` | Booleano | `false` | **Hardware Real:** Chaveamento FET silencioso do ramo Wet (`Q4`). |
| **RATE** | `mod_1.rate_hz` | `pedal.chorus.rate` | `0.1 .. 10.0` (CE-2 real: `0.4 .. 4.5`) | `Hz` | `1.20` | **Hardware Real:** Frequência do oscilador triangular `IC2` (`TL022CP`). |
| **DEPTH** | `mod_1.depth` | `pedal.chorus.depth` | `0.0 .. 1.0` | `0 .. 100 %` | `0.60` | **Hardware Real:** Amplitude da excursão da onda triangular aplicada ao VCO do clock `MN3101`. |
| **MIX** | `mod_1.mix` | `pedal.chorus.mix` | `0.0 .. 1.0` | `0 .. 100 %` | `0.50` (`50%`) | **Extensão Digital AMPFORGE:** No CE-2 real, a mistura é fixa em `50%` ($47\text{ k}\Omega / 47\text{ k}\Omega$). Em `mix = 0.50`, reproduz o hardware real; em `mix = 1.00`, opera em modo Vibrato (`100% Wet`). |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Gerador LFO estritamente Triangular (`TL022CP`):**
   - Fase normalizada $\phi[n] = (\phi[n-1] + f_{\text{rate}} / f_s) \bmod 1,0$.
   - Onda triangular simétrica em $[-1, +1]$:
     $$L_{\text{tri}}(\phi) = 1,0 - 4,0 \cdot \left|\left((\phi + 0,25) \bmod 1,0\right) - 0,5\right|$$
2. **Modulação do Clock `MN3101` e Atraso Físico do `MN3007` (`1024` estágios):**
   - A frequência instantânea do clock bifásico é modulada em torno de $f_{\text{clk,0}} = 68\text{ kHz}$ ($\tau_0 = \frac{512}{68\,000} \approx 7,53\text{ ms}$):
     $$\tau_{\text{BBD}}[n] = \tau_0 + \Delta\tau_{\max} \cdot \text{depth} \cdot L_{\text{tri}}(\phi[n]), \qquad \tau_0 = 7,53\text{ ms},\; \Delta\tau_{\max} = 3,60\text{ ms}$$
     cobrindo a excursão física documentada de `3,93 ms` a `11,13 ms`.
3. **Caminho Wet do `MN3007` (Pré-Ênfase $\to$ Anti-Aliasing $\to$ BBD $\to$ Reconstrução $\to$ De-Ênfase):**
   - **Pré-Ênfase (`IC1A`):** Filtro shelving de 1ª ordem ($f_z = 225,8\text{ Hz}, f_p = 1560,0\text{ Hz}, G_{\text{HF}} = +15,4\text{ dB}$).
   - **Anti-Aliasing (`Q2`):** Filtro passa-baixas Butterworth em $f_c = 6800\text{ Hz}$.
   - **Não-linearidade Sutil de Transferência de Carga MOS do `MN3007`:** $\mathcal{N}_{\text{BBD}}(v) = v - 0,045 \cdot v^3$ ($\text{THD} \approx 0,5\%$ nominal conforme datasheet `F-03`).
   - **Linha de Atraso Fracionária:** Leitura por interpolação cúbica Lagrange de 4 pontos em $\tau_{\text{BBD}}[n] \cdot f_s$.
   - **Reconstrução (`Q3`) + De-Ênfase:** Filtro passa-baixas em $6800\text{ Hz}$ em cascata com o filtro inverso exato da pré-ênfase ($f_p = 225,8\text{ Hz}, f_z = 1560,0\text{ Hz}, G_{\text{HF}} = -15,4\text{ dB}$), resultando em resposta plana até `~6 kHz` e rolagem suave de `-24 dB/oitava` acima de `6,8 kHz` exclusivamente no sinal Wet.
4. **Somador de Saída `IC1B`:**
   - $y[n] = (1 - \text{mix}) \cdot x_{\text{dry}}[n] + \text{mix} \cdot x_{\text{wet}}[n]$ (onde $\text{mix} = 0,50$ corresponde aos resistores iguais $R_{18} = R_{19} = 47\text{ k}\Omega$ do Boss CE-2 real).

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `1×` (o ramo Wet já é fisicamente limitado em banda em `6,8 kHz` pelos filtros anti-aliasing e de reconstrução `Q2`/`Q3` do `MN3007`).
- **Interpolação:** Lagrange cúbica de 4 pontos em `f64`/`f32`.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Buffer de atraso pré-alocado em `prepare()` para até `25 ms` (`2400` amostras a `96 kHz`).
- Zero alocações na heap em `process_block` e `apply_param_rt`.

---

## 12. Posição e interação na cadeia completa de guitarra

- No slot `mod_1`, entrega o chorus analógico clássico de linha única BBD com agudos secos preservados pelo ramo Dry e calor médio-grave vindo do ramo Wet filtrado em `6,8 kHz`.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/boss_ce2_chorus.rs` (separado do módulo `stereo_tri_chorus.rs` da Spec 087).
- **Struct principal:** `pub struct BossCe2ChorusProcessor` implementando `DspBlock` + `AutomatableParams` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Subsistema | Componente Real | Valor Documentado (`F-01`, `F-03`) | Invariante Física |
|---|---|---|---|
| Chip BBD | Matsushita `MN3007` | `1024` estágios ($\tau = 512 / f_{\text{clk}}$) | Atraso central $\tau_0 \approx 7,53\text{ ms}$ (linha BBD única, não Tri-Chorus). |
| Oscilador LFO | `IC2 TL022CP` | Onda Triangular (`0,4..4,5 Hz`) | Derivada de atraso $|d\tau / dt|$ constante por semiciclo (desvio de pitch $\pm \Delta f$ constante em cada rampa). |
| Pré / De-Ênfase | $R_6=47\text{ k}\Omega, R_7=6,8\text{ k}\Omega, C_5=15\text{ nF}$ | $f_1 = 225,8\text{ Hz}, f_2 = 1560\text{ Hz}$ ($\pm 15,4\text{ dB}$) | Produto $H_{\text{pre}}(s) \cdot H_{\text{de}}(s) = 1,0$ (`0,0 dB` plano antes do corte de `6,8 kHz`). |
| Filtros `Q2` / `Q3` | Sallen-Key ativos | $f_c \approx 6,8\text{ kHz}$ (apenas no ramo Wet) | Ramo Dry preserva `100%` da banda até Nyquist; ramo Wet corta acima de `6,8 kHz`. |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Desvio de Pitch Doppler Triangular de Linha Única BBD:** em `100% Wet` (`mix = 1.0`), excitação senoidal de `440 Hz` deve exibir alternância entre dois patamares de frequência ($440 \cdot (1 \pm k)$) característica da derivada de uma onda triangular de **uma única linha BBD**, e não a soma estacionária de 3 senoides a $120^\circ$.
2. **Teste de Cancelamento Complementar Pré-Ênfase / De-Ênfase + Rolagem `6,8 kHz` no Ramo Wet vs Ramo Dry Plano:** verifica que o ramo Wet (`mix = 1.0, depth = 0.0`) é plano entre `100 Hz` e `3 kHz` e atenua `12 kHz` em pelo menos `12 dB`, enquanto o ramo Dry (`mix = 0.0`) é plano até `20 kHz`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- O teste existente `test_ca_02_chorus_ce2_bbd_pitch_modulation` em `contract_032_time_mod_fx.rs` verifica a presença de modulação temporal e filtragem BBD; a separação do Tri-Chorus para a Spec 087 mantém total compatibilidade com esse contrato enquanto corrige a física de linha única do CE-2.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Linha BBD Única (`MN3007`, $\tau_0 \approx 7,53\text{ ms}$) Modulada por LFO Triangular (`TL022CP`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Impulso unitário ou senoide contínua de `440 Hz` a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `pedal.boss.ce2_chorus` com `mod_1.enabled = true`, `mod_1.rate_hz = 2.0`, `mod_1.depth = 0.0` (para medir $\tau_0$ com impulso) e depois `mod_1.depth = 0.80` (para medir modulação de pitch com senoide de `440 Hz`), `mod_1.mix = 1.0` (`100% Wet`).
  - *Método de medição:* Instante do pico da resposta ao impulso em `depth = 0.0`, e medição da frequência instantânea por cruzamentos por zero em `depth = 0.80`.
  - *Critério e tolerância:*
    1. Em `depth = 0.0`, deve existir **um único pico de linha BBD** centrado em **`7,53 ms ± 0,50 ms`** (comprovando ausência das 3 linhas paralelas de `15 ms` do Tri-Chorus).
    2. Em `depth = 0.80`, a frequência instantânea de `440 Hz` deve oscilar periodicamente acima e abaixo de `440 Hz` com desvio de pico $\Delta f \ge 2,5\text{ Hz}$.
  - *Origem da tolerância:* Equação física $\tau = 512 / f_{\text{clk}}$ do `MN3007` com clock central `68 kHz` (`F-01`, `F-03`).

- **CA-02 — Cancelamento Complementar Pré-Ênfase / De-Ênfase (`225 Hz / 1560 Hz`) e Filtro de Reconstrução `6,8 kHz` no Ramo Wet (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoides de teste em `200 Hz`, `2000 Hz` e `12 000 Hz` a `-18 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `mod_1.depth = 0.0` (atraso estacionário), comparando `mod_1.mix = 1.0` (ramo Wet isolado) vs `mod_1.mix = 0.0` (ramo Dry isolado).
  - *Método de medição:* Ganho em dB em `200 Hz`, `2000 Hz` e `12 000 Hz`.
  - *Critério e tolerância:*
    1. No ramo Wet (`mix = 1.0`), como a De-Ênfase (`-15,4 dB`) cancela a Pré-Ênfase (`+15,4 dB`) na banda média, a diferença de ganho entre `2000 Hz` e `200 Hz` deve estar dentro de **`±1,5 dB`**, enquanto em `12 000 Hz` (acima do corte de `6,8 kHz` de `Q2/Q3`) o sinal Wet deve ser atenuado em **pelo menos `12,0 dB`** relativo a `1000 Hz`.
    2. No ramo Dry (`mix = 0.0`), o ganho em `12 000 Hz` deve ser **`0,0 dB ± 0,1 dB`** (banda completa preservada).
  - *Origem da tolerância:* Curva de resposta do ramo Wet vs Dry do Boss CE-2 (`F-01` §"Pre-Emphasis/De-Emphasis & Anti-Aliasing Filters").

- **CA-03 — Paridade da Mistura Hardware 50/50 (`mix = 0.50`) vs Extensão Vibrato (`mix = 1.00`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Impulso unitário (`x[0] = 1.0`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `mod_1.depth = 0.0`, testando `mod_1.mix = 0.50` (padrão hardware $47\text{ k}\Omega / 47\text{ k}\Omega$) e `mod_1.mix = 1.00`.
  - *Método de medição:* Energia no instante $n = 0$ (Dry) e energia na janela do atraso BBD (`7,0..8,5 ms`, Wet).
  - *Critério e tolerância:* Em `mix = 0.50`, ambos os pulsos (Dry em $n=0$ e Wet em $\sim 7,53\text{ ms}$) devem estar presentes com energia igual na banda grave (`200 Hz`, diferença $\le 0,5\text{ dB}$); em `mix = 1.00`, a saída direta em $n = 0$ deve ser zero (`|y[0]| <= 1e-6`).
  - *Origem da tolerância:* Somador `IC1B` com $R_{18} = R_{19} = 47\text{ k}\Omega$ (`F-01` §"Output Mixer").

- **CA-04 — Não-Linearidade Sutil do BBD `MN3007` (`THD` controlada $\le 1,5\%$) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `mod_1.depth = 0.0`, `mod_1.mix = 1.0`.
  - *Método de medição:* THD do ramo Wet medida por FFT de `16 384` amostras.
  - *Critério e tolerância:* A THD do ramo BBD deve estar entre **`0,15%` e `1,50%`** (calor analógico sutil do `MN3007` sem clipping destrutivo).
  - *Origem da tolerância:* Datasheet Matsushita `MN3007` (`THD typ. 0.5%`, `F-03`).

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `rate_hz`, `depth`, `mix`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/modulation.rs` (e futuro `crates/ampforge-dsp/src/fx/pedals/boss_ce2_chorus.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.chorus.*`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_032_time_mod_fx.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] URL corrigida para `boss-ce-2-analysis` e verificada via HTTP 200 no Wayback Machine (`F-01`).
- [x] Desacoplado completamente do **Stereo Tri-Chorus** (que agora possui especificação própria na **[Spec 087](087-modulo-stereo-tri-chorus.md)**).
- [x] Distinguido o hardware real do CE-2 (`Rate`, `Depth`, soma fixa 50/50, LFO triangular, linha única `MN3007`, Pré/De-Ênfase) da extensão digital `Mix`.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` ElectroSmash — *Boss CE-2 Chorus Circuit Analysis* (`https://web.archive.org/web/20240115000000/https://www.electrosmash.com/boss-ce-2-analysis`).
- `[F-02]` Boss Corporation — *CE-2 Owner's Manual* (`https://www.boss.info/global/support/by_product/ce-2/owners_manuals/`).
- `[F-03]` Matsushita Electric Industrial — *MN3007 1024-Stage Low Noise BBD & MN3101 Clock Generator Datasheets*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, desfez-se o acoplamento entre o Boss CE-2 e o Stereo Tri-Chorus: a Spec 069 ficou exclusiva para o **Boss CE-2 (1979, linha única BBD `MN3007` com LFO triangular e Pré/De-Ênfase)**, enquanto o **Stereo Tri-Chorus (3 linhas a $120^\circ$)** foi movido para a **Spec 087**. Corrigiu-se também o link quebrado `boss-ce2-analysis` para `boss-ce-2-analysis`.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Linha BBD única `MN3007` (`1024` estágios, $\tau_0 \approx 7,53\text{ ms}$) com LFO Triangular (`TL022CP`) | `F-01`, `F-03` | Spec 069 / `CA-01` | `crates/ampforge-dsp/src/fx/modulation.rs:196–218 @ c20cc65` (DIVERGENTE: implementa 3 linhas senoidais defasadas em $120^\circ$ a `15 ms` — Tri-Chorus) | `contract_032_time_mod_fx.rs` | `SPECIFIED` |
| Redes complementares de Pré-Ênfase / De-Ênfase (`225 Hz / 1560 Hz, ±15,4 dB`) e filtro de reconstrução `6,8 kHz` no ramo Wet | `F-01` | Spec 069 / `CA-02` | `crates/ampforge-dsp/src/fx/modulation.rs:146 @ c20cc65, 213` (possui apenas LPF simples em `7500 Hz`, sem Pré/De-Ênfase) | `contract_032_time_mod_fx.rs` | `SPECIFIED` |
| Mistura 50/50 Dry/Wet ($R_{18}=R_{19}=47\text{ k}\Omega$) em `mix = 0.50` e extensão `Mix` (`0..100%`) | `F-01`, `F-02` | Spec 069 / `CA-03` | `crates/ampforge-dsp/src/fx/modulation.rs:217 @ c20cc65` | `contract_040_pedals_parity.rs` | `TESTED` |
| Não-linearidade sutil de transferência de carga do `MN3007` (`THD ~ 0.5%`) | `F-03` | Spec 069 / `CA-04` | `crates/ampforge-dsp/src/fx/modulation.rs:214 @ c20cc65` (`(wet_lp * 1.1).tanh() / 1.1`) | `contract_032_time_mod_fx.rs` | `TESTED` |
| Execução zero-alloc em tempo real | Spec 053 | Spec 069 / `CA-05` | `crates/ampforge-dsp/src/fx/modulation.rs:179–294 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
