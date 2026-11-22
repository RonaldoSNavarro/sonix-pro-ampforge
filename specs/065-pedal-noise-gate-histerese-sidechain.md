---
status: rascunho
---

# Spec 065: Módulo Algorítmico de Noise Gate / Expander com Histerese, Hold e Sidechain LR4

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-dynamics` (`modulo-dsp`)
> **Identificador canônico:** `module.dynamics.noise_gate_lr4`
> **Variante / Revisão:** `Módulo Algorítmico Nativo AMPFORGE (Sidechain Passa-Altas Linkwitz-Riley 4ª Ordem 24 dB/oct + Detector com Histerese Dupla e Hold)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/dynamics.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [031](031-amplificadores-fieis-aos-esquemas-e-controles-originais.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md)

---

## 1. Objetivo

Especificar o contrato técnico, matemático e de verificação do **Módulo Algorítmico de Noise Gate / Expander com Histerese, Hold e Sidechain Linkwitz-Riley de 4ª Ordem (`module.dynamics.noise_gate_lr4`)**, reclassificando honestamente a natureza do processador como `tipo: modulo-dsp` (algoritmo digital nativo do AMPFORGE) e removendo a alegação incorreta de que o código atual modela o circuito físico do Boss NS-2 ou as patentes analógicas do ISP Decimator (`US 6,091,013`, `US 6,944,305`, `US 7,035,413`).

---

## 2. Escopo

1. Filtro passa-altas de sidechain Linkwitz-Riley de 4ª ordem (`LR4`, `24 dB/oitava`, cascata de dois biquads Butterworth de 2ª ordem com $Q = \frac{1}{\sqrt{2}}$) ajustável entre `20 Hz` e `500 Hz`.
2. Detector de envoltória de pico/RMS acoplado com histerese de duplo limiar ($\theta_{\text{open}}$ e $\theta_{\text{close}} = \theta_{\text{open}} - H_{\text{dB}}$).
3. Temporizador determinístico de `Hold` (`0..500 ms`) e máquina de estados contínua (`Open`, `Holding`, `Closing`, `Closed`) com balística exponencial (`Attack`, `Release` / `Decay`) e piso de atenuação configurável (`Range` / `Reduction` de `-80 dB` a `0 dB`).
4. Mapeamento completo dos 6 controles expostos no pedal visual (`controls.registry.ts:624–688`).

---

## 3. Fora de escopo

- **Emulação circuit-level do Boss NS-2 Noise Suppressor:** O Boss NS-2 físico possui apenas 2 knobs (`Threshold`, `Decay`) + chave `Mode (Reduction / Mute)` e loop de 4 cabos (`Send/Return`), utilizando um VCA analógico controlado por envoltória de entrada sem sidechain LR4 ajustável e sem controles de `Hold` ou `Hysteresis` dedicados.
- **Emulação circuit-level do ISP Technologies Decimator:** O ISP Decimator utiliza *Time Vector Processing* analógico proprietário (constante de tempo adaptativa dependente da derivada da envoltória, Patentes `US 6,944,305` / `US 7,035,413`) com um único knob `Threshold`.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `modulo-dsp` (`dynamics` / `noise-gate-expander`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (Arquitetura Algorítmica Nativa de Dinâmica para Guitarra High-Gain) |
| **Modelo de referência** | Dual-Threshold Hysteresis Noise Gate with LR4 High-Pass Sidechain |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 065) |
| **Identificador no sistema** | `module.dynamics.noise_gate_lr4` (`NoiseGateProcessor` / UI `pedal.gate.*`) |
| **Papel na cadeia de sinal** | Bloco `GATE` (início da cadeia sobre o sinal DI limpo, ou no loop pós-pré-amplificador para supressão de piso de ruído de alto ganho sem *chattering*). |

---

## 5. Princípio físico e funcional do sistema

Em cadeias de guitarra de alto ganho (como JCM800 + TS808 ou Dual Rectifier Red Modern), um gate convencional de limiar simples sofre de dois problemas clássicos:
1. **Falso disparo por hum de 50/60 Hz e ressonância mecânica grave (*palm-mute rumble*):** Resolvido filtrando exclusivamente o sinal do **detector (sidechain)** com um filtro passa-altas Linkwitz-Riley de 4ª ordem (`24 dB/oitava`), de forma que o detector só enxerga a energia das cordas acima de $f_{\text{SC}}$ (`20..500 Hz`), enquanto o caminho de áudio principal permanece com resposta de frequência `100%` plana até `20 Hz`.
2. **Oscilação rápida na cauda da nota (*chattering* / *flutter*):** Resolvido combinando **Histerese Dupla** (o gate abre em $\theta_{\text{open}}$, mas só inicia o fechamento quando o nível cai abaixo de $\theta_{\text{close}} = \theta_{\text{open}} - H_{\text{dB}}$, onde $H_{\text{dB}} \in [0,\; 12\text{ dB}]$) com um contador de **Hold** ($\tau_{\text{hold}} \in [0,\; 500\text{ ms}]$) que mantém o gate aberto por um período mínimo após a última travessia de limiar.

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Udo Zölzer — *DAFX: Digital Audio Effects* (2nd ed., Wiley, 2011), Cap. 4 (*Dynamic Range Control — Noise Gates & Expanders with Hysteresis*) | Literatura acadêmica de referência em processamento dinâmico digital | `ACADEMIC` | Equações de detecção de envoltória recursiva, curva estática de noise gate com histerese dupla ($\theta_{\text{open}}, \theta_{\text{close}}$), `Hold` e suavização de ganho (`Attack`/`Release`). |
| `F-02` | Siegfried Linkwitz — *Active Crossover Networks for Noncoincident Drivers* (JAES, Vol. 24, No. 1, 1976) | Literatura acadêmica de referência para filtros Linkwitz-Riley (`LR4`) | `ACADEMIC` | Topologia em cascata de dois filtros Butterworth de 2ª ordem ($Q = 0,7071$) totalizando `-6,02 dB` em $f_c$ e inclinação assintótica de `24 dB/oitava`. |
| `F-03` | Boss Corporation — *NS-2 Noise Suppressor Owner's Manual* | `https://www.boss.info/global/support/by_product/ns-2/owners_manuals/` (HTTP 200 verificado) | `PRIMARY_OFFICIAL` | Referência de contraste: comprova que o NS-2 real possui apenas `Threshold`, `Decay` e `Mode`, justificando a classificação deste processador de 6 controles como `modulo-dsp`. |
| `F-04` | Buck Waller (ISP Technologies) — *US Patent 6,944,305 / 7,035,413: Dynamic Adaptive Time Constant Circuit for Noise Reduction* | USPTO Patent Archive | `UNKNOWN` | Referência de contraste: documenta a topologia analógica de vetor de tempo do Decimator, distinta da arquitetura digital `LR4 + Hysteresis + Hold` deste módulo. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Cascata de dois filtros Butterworth de 2ª ordem ($Q = 1/\sqrt{2}$) forma um filtro Linkwitz-Riley de 4ª ordem (`24 dB/oitava`, `-6 dB` em $f_c$) | `F-02` | JAES 1976 | §2–3 | `ACADEMIC` | `CONFIRMED` |
| Histerese dupla ($\theta_{\text{close}} = \theta_{\text{open}} - H_{\text{dB}}$) previne *chattering* quando a envoltória oscila próximo ao limiar | `F-01` | DAFX 2nd ed. | Cap. 4 §4.2 | `ACADEMIC` | `CONFIRMED` |
| Distinção entre o módulo algorítmico de 6 parâmetros do AMPFORGE e os pedais comerciais NS-2 / ISP Decimator | `F-03`, `F-04` | Manuais/Patentes oficiais | Especificações gerais | `UNKNOWN` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Audio In] ──┬────────────────────────────────────────────────────────► (× Gain Envelope g[n]) ──► [Audio Out]
             │                                                                    ▲
             ▼ (Sidechain Branch)                                                 │
      [HPF Biquad 1 (Butterworth 2nd, Q=0.7071)]                                  │
             │                                                                    │
             ▼                                                                    │
      [HPF Biquad 2 (Butterworth 2nd, Q=0.7071)]  ==> LR4 (24 dB/oct @ f_sc)      │
             │                                                                    │
             ▼                                                                    │
      [Envelope Detector |x_sc[n]|] ──► [Dual-Threshold Hysteresis (θ_open, θ_close) + Hold Counter]
                                                          │
                                                          ▼
                                            [Attack / Release Ballistics + Range Floor]
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`MODEL_COUPLING` / `FACTUAL_ERROR` (corrigido nesta Spec):** A versão inicial da Spec 065 atribuía este algoritmo ao hardware "Boss NS-2 / ISP Decimator". Na realidade, o código em [`crates/ampforge-dsp/src/fx/dynamics.rs:34–205 @ c20cc65`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/fx/dynamics.rs#L34-L205) implementa um Noise Gate digital de estúdio com 6 controles (`threshold_db`, `hysteresis_db`, `attack_ms`, `hold_ms`, `release_ms`, `range_db`, `sidechain_hpf_hz`) e filtro sidechain LR4 (`sc_hpf_1`, `sc_hpf_2`). A reclassificação para `tipo: modulo-dsp` (`module.dynamics.noise_gate_lr4`) alinha 100% a documentação ao código e à UI.
2. **`CODE_REFERENCE_STALE` (verificado):** Em `controls.registry.ts:648`, o controle visual `pedal.gate.decay` mapeia para a chave backend `gate.release_ms` (nota explícita em `controls.registry.ts:650`), e `pedal.gate.hold` mapeia para `gate.hold_ms` (`0..500 ms`).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Real | Unidade | Valor Padrão | Papel no Algoritmo DSP |
|---|---|---|---|---|---|---|
| **BYPASS** | `gate.enabled` | `pedal.gate.enabled` | `false / true` | Booleano | `false` | Ativa/desativa o processador de Noise Gate. |
| **THRESHOLD** | `gate.threshold_db` | `pedal.gate.threshold` | `-80.0 .. 0.0` | `dB` | `-45.0` | Limiar de abertura $\theta_{\text{open}}$ em dBFS. |
| **HYSTERESIS** | `gate.hysteresis_db` | `pedal.gate.hysteresis` | `0.0 .. 12.0` | `dB` | `4.0` | Diferença $H_{\text{dB}}$ que define o limiar de fechamento $\theta_{\text{close}} = \theta_{\text{open}} - H_{\text{dB}}$. |
| **ATTACK** | `gate.attack_ms` | `pedal.gate.attack` | `0.1 .. 50.0` | `ms` | `1.0` | Constante de tempo exponencial de abertura do gate. |
| **HOLD** | `gate.hold_ms` | `pedal.gate.hold` | `0.0 .. 500.0` | `ms` | `20.0` | Tempo mínimo em que o gate permanece aberto após o sinal cair abaixo de $\theta_{\text{close}}$. |
| **DECAY (RELEASE)** | `gate.release_ms` | `pedal.gate.decay` | `5.0 .. 1000.0` | `ms` | `80.0` | Constante de tempo exponencial de fechamento após expirar o `Hold`. |
| **RANGE** | `gate.range_db` | *(interno / API)* | `-80.0 .. 0.0` | `dB` | `-80.0` | Atenuação máxima quando o gate está totalmente fechado ($g_{\min} = 10^{\text{range\_db}/20}$). |
| **SIDECHAIN HPF** | `gate.sidechain_hpf_hz` | `pedal.gate.sidechain_hpf` | `20.0 .. 500.0` | `Hz` | `120.0` | Frequência de corte ($-6\text{ dB}$) do filtro passa-altas LR4 (`24 dB/oitava`) no ramo do detector. |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Filtro Sidechain Linkwitz-Riley de 4ª Ordem (`LR4`):**
   - Dois filtros biquad passa-altas Butterworth de 2ª ordem idênticos em cascata ($Q = \frac{1}{\sqrt{2}} \approx 0,70710678$, frequência de corte $f_{\text{SC}} \in [20,\; 500\text{ Hz}]$):
     $$H_{\text{LR4}}(s) = \left(\frac{s^2}{s^2 + \sqrt{2}\omega_c s + \omega_c^2}\right)^2$$
   - Em $f = f_{\text{SC}}$, cada estágio Butterworth apresenta $-3,01\text{ dB}$, totalizando $-6,02\text{ dB}$ e inclinação de $+24\text{ dB/oitava}$ abaixo de $f_{\text{SC}}$.
2. **Detecção e Histerese com Máquina de Estados (`is_open`, `hold_counter`):**
   - Limiar linear de abertura $T_{\text{open}} = 10^{\theta_{\text{open}}/20}$ e limiar linear de fechamento $T_{\text{close}} = 10^{(\theta_{\text{open}} - H_{\text{dB}})/20}$.
   - Se $|x_{\text{SC}}[n]| \ge T_{\text{open}}$: `is_open = true`, `hold_counter = round(hold_ms * 0.001 * fs)`.
   - Se `is_open == true` e $|x_{\text{SC}}[n]| < T_{\text{close}}$:
     - Se `hold_counter > 0`: decrementa `hold_counter` e mantém ganho alvo $= 1,0$;
     - Caso contrário: `is_open = false`, ganho alvo $= g_{\min} = 10^{\text{range\_db}/20}$.
3. **Suavização Exponencial de Ganho (`Attack` / `Release`):**
   - Coeficientes de 1 polo: $\alpha_{\text{att}} = \exp\left(-\frac{1}{\tau_{\text{att}} f_s}\right)$ e $\alpha_{\text{rel}} = \exp\left(-\frac{1}{\tau_{\text{rel}} f_s}\right)$.
   - A saída de áudio é $y[n] = x[n] \cdot g[n]$ (sem aplicar o filtro sidechain sobre $x[n]$).

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `1×` (opera na taxa nativa $f_s$; a multiplicação por envoltória suave exponencial $\tau_{\text{att}} \ge 0,1\text{ ms}$ não introduz aliasing audível).
- **Precisão numérica:** Coeficientes e estados dos biquads `LR4` em `f64` ou `f32` estabilizado sem underflow/denormais.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`.
- Complexidade $O(1)$ por amostra; WCET $\le 1,5\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- **Posição padrão:** Primeiro bloco da cadeia (`GATE`), atuando sobre o sinal de alta relação sinal-ruído antes da compressão e dos estágios de distorção (`DRIVE` e `AMP`).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo atual / alvo:** `crates/ampforge-dsp/src/fx/dynamics.rs` (`pub struct NoiseGateProcessor`).
- Implementa `DspBlock` + `AutomatableParams` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Parâmetro | Invariante Matemática |
|---|---|
| `theta_close <= theta_open` | Para todo $H_{\text{dB}} \ge 0$, o limiar de fechamento é sempre menor ou igual ao de abertura ($T_{\text{close}} \le T_{\text{open}}$). |
| Transparência espectral em estado aberto | Quando $g[n] = 1,0$, a saída $y[n]$ é bit-idêntica à entrada $x[n]$ ($|y[n] - x[n]| \le 10^{-7}$), pois o filtro `LR4` só existe no ramo do sidechain. |
| Inclinação do Sidechain `LR4` | $+24,0\text{ dB/oitava}$ ($\pm 1,5\text{ dB}$) uma oitava abaixo de $f_{\text{SC}}$. |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Histerese Dupla ($\theta_{\text{open}}$ vs $\theta_{\text{close}}$):** verifica que um sinal intermediário ($\theta_{\text{close}} < A < \theta_{\text{open}}$) permanece fechado se o gate estava fechado, mas permanece aberto se o gate já havia sido aberto.
2. **Teste de Rejeição de Hum Grave pelo Sidechain LR4 + Transparência do Caminho Principal:** verifica que um hum de `50 Hz` abaixo do corte LR4 de `250 Hz` não abre o gate, mas quando o gate está aberto um sinal grave de `50 Hz` passa sem qualquer atenuação no áudio principal.
3. **Teste de Temporização de `Hold` e `Release`:** verifica o número exato de amostras de retenção antes do início do decaimento exponencial.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Tempos de `Attack` excessivamente curtos (`< 0,1 ms`) em ondas graves podem causar um clique sutil na abertura; o limite inferior de `0,1 ms` com suavização de 1 polo previne descontinuidade de passo.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Histerese de Duplo Limiar ($\theta_{\text{open}}$ vs $\theta_{\text{close}} = \theta_{\text{open}} - H_{\text{dB}}$) (`ANALYTICAL` / `REGRESSION`):**
  - *Sinal de entrada:* Senoide de `1000 Hz` em três etapas sequenciais de `200 ms` cada: Etapa 1 em `-36 dBFS` (abaixo de $\theta_{\text{open}} = -30\text{ dBFS}$), Etapa 2 em `-24 dBFS` (acima de $\theta_{\text{open}}$), e Etapa 3 novamente em `-36 dBFS` (abaixo de $\theta_{\text{open}}$, porém acima de $\theta_{\text{close}} = -30 - 10 = -40\text{ dBFS}$).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `module.dynamics.noise_gate_lr4` com `gate.threshold_db = -30.0`, `gate.hysteresis_db = 10.0`, `gate.attack_ms = 0.5`, `gate.hold_ms = 0.0`, `gate.release_ms = 20.0`, `gate.range_db = -80.0`, `gate.sidechain_hpf_hz = 80.0`.
  - *Método de medição:* Ganho instantâneo $g[n] = |y[n]| / |x[n]|$ ao final de cada uma das três etapas.
  - *Critério e tolerância:*
    1. Ao final da Etapa 1 (`-36 dBFS`, partindo fechado): o gate deve estar **fechado** ($g \le -60\text{ dB}$).
    2. Ao final da Etapa 2 (`-24 dBFS`): o gate deve estar **aberto** ($g = 0,0\text{ dB} \pm 0,05\text{ dB}$).
    3. Ao final da Etapa 3 (`-36 dBFS`, partindo aberto, dentro da janela de histerese `[-40, -30] dBFS`): o gate deve **permanecer aberto** ($g = 0,0\text{ dB} \pm 0,05\text{ dB}$).
  - *Origem da tolerância:* Definição matemática exata de histerese de duplo limiar (`F-01` Cap. 4).

- **CA-02 — Rejeição de Hum Grave no Sidechain LR4 (`24 dB/oitava`) e Transparência no Caminho de Áudio (`ANALYTICAL`):**
  - *Sinal de entrada:*
    - Caso A: Senoide de `60 Hz` (hum de rede) com amplitude `-24 dBFS`.
    - Caso B: Senoide de `60 Hz` (`-12 dBFS`) somada a senoide de `1000 Hz` (`-12 dBFS`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `gate.threshold_db = -30.0`, `gate.hysteresis_db = 3.0`, `gate.sidechain_hpf_hz = 240.0` (`60 Hz` está 2 oitavas abaixo de `240 Hz` $\Rightarrow$ atenuação LR4 teórica de $\approx 48\text{ dB}$ no sidechain, reduzindo `-24 dBFS` para `~-72 dBFS < -30 dBFS`).
  - *Método de medição:* Nível de saída no Caso A, e ganho da componente de `60 Hz` no caminho principal quando o gate é aberto pela componente de `1000 Hz` no Caso B.
  - *Critério e tolerância:*
    1. No Caso A (`60 Hz @ -24 dBFS` sozinho): o gate **não deve abrir** (atenuação de saída $\ge 60\text{ dB}$).
    2. No Caso B (gate aberto pelo tom de `1000 Hz`): a componente de `60 Hz` no caminho principal de áudio deve passar com ganho unitário (**`0,0 dB ± 0,05 dB`**), comprovando que o filtro `LR4` atua exclusivamente no sidechain.
  - *Origem da tolerância:* Atenuação de 4ª ordem Linkwitz-Riley ($2 \text{ oitavas} \times 24\text{ dB/oct} = 48\text{ dB}$, `F-02`).

- **CA-03 — Temporizador Determinístico de `Hold` (`ANALYTICAL`):**
  - *Sinal de entrada:* Burst senoidal de `1000 Hz` a `-12 dBFS` por `50 ms`, seguido de queda instantânea para `-50 dBFS` por `200 ms`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `gate.threshold_db = -25.0`, `gate.hysteresis_db = 3.0`, `gate.hold_ms = 100.0` (`4800` amostras a `48 kHz`), `gate.release_ms = 50.0`.
  - *Método de medição:* Medição do ganho $g[n]$ em $t = 95\text{ ms}$ após o fim do burst e em $t = 160\text{ ms}$ após o fim do burst.
  - *Critério e tolerância:* Em $t = 95\text{ ms}$ após a queda do sinal (dentro da janela de `Hold = 100 ms`), o ganho deve permanecer em **`1,0` (`0,0 dB ± 0,01 dB`)**; em $t = 160\text{ ms}$ (após expirar o `Hold` + $1,2 \times \tau_{\text{release}}$), o ganho deve ter caído em **pelo menos `20,0 dB`**.
  - *Origem da tolerância:* Contador exato de amostras $N_{\text{hold}} = \text{round}(\text{hold\_ms} \cdot 10^{-3} \cdot f_s)$.

- **CA-04 — Paridade de Mapeamento de Controles (`pedal.gate.decay` $\to$ `gate.release_ms`) (`REGRESSION`):**
  - *Configuração e parâmetros:* Variação de `gate.release_ms` entre `20 ms` e `500 ms` com `gate.hold_ms = 0.0`.
  - *Método de medição:* Tempo necessário para o envelope de ganho cair `20 dB` após cessar uma senoide de `1000 Hz @ -12 dBFS`.
  - *Critério e tolerância:* O tempo de queda em `release_ms = 500 ms` deve ser **`25× ± 2,5×`** maior que o tempo de queda em `release_ms = 20 ms`.
  - *Origem da tolerância:* Proporcionalidade linear da constante de tempo $\tau_{\text{rel}}$ no decaimento exponencial (`F-01`).

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de todos os 7 parâmetros.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/dynamics.rs` (`NoiseGateProcessor`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.gate.*`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_031_pedals_dsp.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Reclassificado honestamente como `tipo: modulo-dsp` (`module.dynamics.noise_gate_lr4`), removendo a falsa atribuição de circuito físico ao Boss NS-2 e ao ISP Decimator.
- [x] Todos os 6 controles visuais (`threshold`, `hysteresis`, `attack`, `hold`, `decay`/`release_ms`, `sidechain_hpf`) + `range_db` documentados e rastreados.
- [x] Critérios `CA-01` a `CA-05` classificados e com condições completas de medição.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Zölzer, U. (2011). *DAFX: Digital Audio Effects* (2nd ed.). Wiley, Chapter 4.
- `[F-02]` Linkwitz, S. (1976). *Active Crossover Networks for Noncoincident Drivers*. JAES, 24(1), 2–8.
- `[F-03]` Boss Corporation — *NS-2 Noise Suppressor Owner's Manual* (`https://www.boss.info/global/support/by_product/ns-2/owners_manuals/`).
- `[F-04]` Waller, B. — *US Patent 6,944,305 / 7,035,413* (ISP Decimator reference contrast).

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, reclassificou-se a Spec 065 de `pedal` ("Boss NS-2 / ISP Decimator") para `modulo-dsp` (`module.dynamics.noise_gate_lr4`), pois o processador de 6 controles com sidechain Linkwitz-Riley de 4ª ordem (`24 dB/oitava`), histerese configurável e `Hold` é uma arquitetura algorítmica digital própria do AMPFORGE, distinta dos circuitos analógicos de 1 ou 2 knobs do Decimator e do NS-2.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Histerese de duplo limiar ($\theta_{\text{open}}$, $\theta_{\text{close}} = \theta_{\text{open}} - H_{\text{dB}}$) | `F-01` Cap. 4 | Spec 065 / `CA-01` | `crates/ampforge-dsp/src/fx/dynamics.rs:146–176 @ c20cc65` | `contract_031_pedals_dsp.rs` (`test_ca_05_noise_gate_hysteresis_and_sidechain`) | `TESTED` |
| Sidechain Passa-Altas Linkwitz-Riley de 4ª Ordem (`2× Butterworth 2nd Q=0.707`, `24 dB/oitava`) sem filtrar o caminho principal | `F-02` | Spec 065 / `CA-02` | `crates/ampforge-dsp/src/fx/dynamics.rs:124–129 @ c20cc65, 148–149` | `contract_031_pedals_dsp.rs` (`test_ca_05_noise_gate_hysteresis_and_sidechain`) | `TESTED` |
| Temporizador determinístico de `Hold` (`0..500 ms`) | `F-01` Cap. 4 | Spec 065 / `CA-03` | `crates/ampforge-dsp/src/fx/dynamics.rs:156–167 @ c20cc65` | `contract_040_pedals_parity.rs` | `TESTED` |
| Balística exponencial `Attack` (`0.1..50 ms`), `Decay/Release` (`5..1000 ms`) e `Range` (`-80..0 dB`) | `F-01` Cap. 4 | Spec 065 / `CA-04` | `crates/ampforge-dsp/src/fx/dynamics.rs:131–133 @ c20cc65, 178–194` | `contract_040_pedals_parity.rs` | `TESTED` |
| Execução zero-alloc em tempo real | Spec 053 | Spec 065 / `CA-05` | `crates/ampforge-dsp/src/fx/dynamics.rs:144–204 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
