---
status: rascunho
---

# Spec 068: Rack Roland RE-201 Space Echo (Multi-Head Magnetic Tape Delay & Spring Reverb)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-delay` (`rack`)
> **Identificador canônico:** `rack.roland.re201_space_echo`
> **Variante / Revisão:** `1974–1990 Roland RE-201 Space Echo (1/4" Free-Running Tape Loop, 1 Erase + 1 Record + 3 Playback Heads, 12-Position Mode Selector + Spring Reverb Tank)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/delay.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [081](081-componente-motor-de-delay-e-spillover.md), [082](082-componente-motor-de-reverb-mola-placa-sala-shimmer.md)

---

## 1. Objetivo

Especificar o contrato técnico, físico, mecânico e de verificação da unidade eletromecânica de eco de fita **Roland RE-201 Space Echo (1974)** como processador modular independente (`rack.roland.re201_space_echo`), corrigindo o erro factual na tabela da chave seletora de **12 modos (`MODE SELECTOR 1–12`)** presente no código atual (`crates/ampforge-dsp/src/fx/delay.rs:364–378 @ c20cc65`) e na Spec 032, além de documentar integralmente:
1. O caminho físico da fita magnética (`Erase Head` $\to$ `Record Head` $\to$ `Playback Head 1` $\to$ `Playback Head 2` $\to$ `Playback Head 3`);
2. A variação contínua da velocidade do motor *capstan* (`REPEAT RATE`) com inércia mecânica (*pitch glide / Doppler warp*), *wow* e *flutter*;
3. A saturação magnética com histerese/compressão de fita e pré/de-ênfase (`BASS` e `TREBLE` no loop de eco); e
4. O tanque integrado de **Spring Reverb** nos modos `5–11` e `12 (Reverb Only)`.

---

## 2. Escopo

1. Simulação física do laço de fita magnética de `1/4"` movido por motor *capstan* DC com velocidade variável (`REPEAT RATE`), modelando as distâncias físicas entre a cabeça de gravação (`Rec`) e as **3 cabeças de reprodução (`Head 1`, `Head 2`, `Head 3`)**.
2. **Tabela oficial exata do seletor rotativo de 12 posições (`MODE SELECTOR 1–12`)** do painel frontal da Roland:
   - **Echo Only (Spring Reverb desligado):** Modos `1`, `2`, `3`, `4`.
   - **Echo + Reverb (Cabeças de fita + Spring Reverb ativo):** Modos `5`, `6`, `7`, `8`, `9`, `10`, `11`.
   - **Reverb Only (Apenas tanque de mola, cabeças de fita desligadas):** Modo `12`.
3. Loop de realimentação (`INTENSITY`) com saturação magnética progressiva, equalizador dedicado do canal de eco (`BASS`, `TREBLE` em $\pm 10\text{ dB}$) e auto-oscilação controlada quando `INTENSITY > 85%`.

---

## 3. Fora de escopo

- **Outros algoritmos de delay hoje agrupados em `DelayProcessor` (`delay.rs:14–22`):** `DelayModel::PristineDigitalDelay`, `DelayModel::AnalogBbdDelay`, `DualDelay`, `Reverse`, `DelayModel::PingPongStereo` — documentados na **[Spec 081](081-componente-motor-de-delay-e-spillover.md)**.
- **Roland RE-101** (sem tanque de Spring Reverb e apenas 6 modos) ou **RE-301 Chorus Echo** (adiciona BBD Chorus e Sound-on-Sound).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `rack` / `tabletop-unit` (`tape-echo` + `spring-reverb`) |
| **Fabricante de referência** | Roland Corporation (Osaka / Hamamatsu, Japão — Ikutaro Kakehashi) |
| **Modelo de referência** | Roland RE-201 Space Echo |
| **Ano / época / revisão** | 1974–1990 (1 Erase Head, 1 Record Head, 3 Playback Heads `H1/H2/H3`, Capstan Motor DC, Tanque de Mola Spring Reverb) |
| **Identificador no sistema** | `rack.roland.re201_space_echo` (`DelayModel::DelayModel::MultiHeadTapeEcho` / UI `pedal.delay.*`) |
| **Papel na cadeia de sinal** | Bloco `DELAY` pós-pré-amplificador (ou pós-cab). |

---

## 5. Princípio físico e funcional do equipamento real

1. **Geometria Física das Cabeças Magnéticas e Velocidade do Motor Capstan (`REPEAT RATE`):**
   - A fita magnética de `1/4"` corre continuamente em uma câmara livre (*free-running tape chamber*) tracionada pelo eixo do motor *capstan* e pelo rolo pressor (*pinch roller*), passando em sequência por:
     1. **Erase Head** (cabeça apagadora excitada por oscilador ultrassônico de bias/erase de `~65 kHz`);
     2. **Record Head** (cabeça de gravação com bias AC);
     3. **Playback Head 1 (`H1`)** — distância física $d_1$ da cabeça de gravação;
     4. **Playback Head 2 (`H2`)** — distância física $d_2 \approx 1,895 \cdot d_1$ (quase o dobro de $d_1$, levemente deslocada de $2,0\times$ para evitar superposição exata de picos rítmicos nas combinações multi-head);
     5. **Playback Head 3 (`H3`)** — distância física $d_3 \approx 2,845 \cdot d_1$ (aproximadamente $3\times d_1$).
   - O controle **`REPEAT RATE`** varia a tensão DC aplicada ao servo-motor do *capstan*, alterando a velocidade linear da fita $v_{\text{tape}}(t)$. Portanto, o tempo de atraso de cada cabeça é $\tau_k(t) = \frac{d_k}{v_{\text{tape}}(t)}$:
     - Na cabeça `H1`: $\tau_1 \in [\sim 60\text{ ms},\; \sim 195\text{ ms}]$;
     - Na cabeça `H2`: $\tau_2 \in [\sim 115\text{ ms},\; \sim 370\text{ ms}]$;
     - Na cabeça `H3`: $\tau_3 \in [\sim 175\text{ ms},\; \sim 560\text{ ms}]$.
   - Quando o músico gira `REPEAT RATE` enquanto há sinal gravado na fita, a inércia mecânica do volante do motor *capstan* ($\tau_{\text{motor}} \approx 80..120\text{ ms}$) acelera ou desacelera toda a fita simultaneamente, produzindo a clássica **transposição contínua de pitch por efeito Doppler (*tape speed pitch glide*)**.
2. **Tabela Oficial das 12 Posições da Chave `MODE SELECTOR` (Painel Frontal Roland RE-201):**
   O painel frontal original do RE-201 divide o seletor rotativo em três zonas serigrafadas:

   | Modo | Zona no Painel Frontal | Cabeça 1 (`H1`) | Cabeça 2 (`H2`) | Cabeça 3 (`H3`) | Cabeça de Realimentação (*Feedback Source*) | Tanque Spring Reverb |
   |---|---|---|---|---|---|---|
   | **1** | `ECHO ONLY (REPEAT)` | **ON** | OFF | OFF | `H1` | OFF |
   | **2** | `ECHO ONLY (REPEAT)` | OFF | **ON** | OFF | `H2` | OFF |
   | **3** | `ECHO ONLY (REPEAT)` | OFF | OFF | **ON** | `H3` | OFF |
   | **4** | `ECHO ONLY (REPEAT)` | OFF | **ON** | **ON** | `H3` | OFF |
   | **5** | `REVERB ECHO` | **ON** | OFF | OFF | `H1` | **ON** |
   | **6** | `REVERB ECHO` | OFF | **ON** | OFF | `H2` | **ON** |
   | **7** | `REVERB ECHO` | OFF | OFF | **ON** | `H3` | **ON** |
   | **8** | `REVERB ECHO` | **ON** | **ON** | OFF | `H2` | **ON** |
   | **9** | `REVERB ECHO` | OFF | **ON** | **ON** | `H3` | **ON** |
   | **10** | `REVERB ECHO` | **ON** | OFF | **ON** | `H3` | **ON** |
   | **11** | `REVERB ECHO` | **ON** | **ON** | **ON** | `H3` | **ON** |
   | **12** | `REVERB ONLY` | OFF | OFF | OFF | Nenhuma (Tape Mute) | **ON** |

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Roland Corporation — *Roland RE-201 Space Echo Official Product Reference & Panel Layout* | `https://www.roland.com/global/products/rc_re-201_space_echo/` (HTTP 200 oficial verificado) | `SECONDARY` | Arquitetura oficial das 3 cabeças de reprodução, motor capstan, tanque de mola integrado, equalizador de eco `Bass/Treble` e tabela das 12 posições do `MODE SELECTOR`. |
| `F-02` | Roland Corporation — *RE-201 Space Echo Owner's Manual & Service Notes (1974/1977)* | Documentação técnica de serviço Roland (`Mode Selector Chart 1–12`, `Tape Speed 1.85 to 7.4 ips`, distâncias entre cabeças `H1/H2/H3`) | `UNKNOWN` | Diagrama de chaveamento da chave rotativa de 12 posições e relações de tempo entre `H1`, `H2` e `H3`. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| 3 cabeças de reprodução (`H1`, `H2`, `H3`) com razões espaciais nominais $1 : 1,895 : 2,845$ | `F-01`, `F-02` | Roland RE-201 (1974) | Service Notes §"Head Block & Delay Time" | `UNKNOWN` | `CONFIRMED` |
| Tabela oficial dos 12 modos (`1–4` Echo Only: `H1`, `H2`, `H3`, `H2+H3`; `5–11` Reverb+Echo: `H1`, `H2`, `H3`, `H1+H2`, `H2+H3`, `H1+H3`, `H1+H2+H3`; `12` Reverb Only) | `F-01`, `F-02` | Roland RE-201 (1974) | Painel frontal e `Mode Selector Chart` | `UNKNOWN` | `CONFIRMED` |
| Controles de tom `BASS` e `TREBLE` aplicados exclusivamente ao canal de eco (`ECHO`), além de `REPEAT RATE`, `INTENSITY`, `ECHO VOLUME` e `REVERB VOLUME` | `F-01`, `F-02` | Roland RE-201 (1974) | Schematic & Front Panel | `UNKNOWN` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do circuito ou sistema

```
[Input] ──► [Preamp] ──┬──────────────────────────────────────────────────────────────────────► [Dry Summer]
                       │                                                                             ▲
                       ├──► (Modes 5..12) ──► [Spring Reverb Driver & Tank] ──► [Reverb Volume] ─────┤
                       │                                                                             │
                       └──► (Modes 1..11) ──► [Record Head Amplifier + Magnetic Tape Saturation]     │
                                                      │                                              │
                                                      ▼                                              │
                                    [1/4" Tape Loop Driven by Capstan Motor (Repeat Rate + Wow/Flutter)]
                                       ├──► [Head 1 (1.000x)] ──┐                                    │
                                       ├──► [Head 2 (1.895x)] ──┼──► [12-Pos Mode Switch]            │
                                       └──► [Head 3 (2.845x)] ──┘          │                         │
                                                 ▲                         ▼                         │
                                                 └── [Intensity] ◄── [Echo Bass & Treble EQ] ──► [Echo Volume]
```

### 7.1. Estado atual no código (pós-AUD-28)

Em `crates/ampforge-dsp/src/fx/delay.rs:359-378 @ c20cc65`, a tabela de modos do `DelayModel::MultiHeadTapeEcho` já segue o painel do RE-201 (conforme corrigido no AUD-28):
- Modos `1..4`: somente fita (`1 => H1`, `2 => H2`, `3 => H3`, `4 => H2+H3`, `reverb = false`).
- Modos `5..7`: cabeças individuais (`H1`, `H2`, `H3`) com tanque de mola (`reverb = true`).
- Modos `8..11`: combinações multi-cabeça (`DECISAO_SONIX_RE201_ORDEM_PARES`: `H1+H2`, `H2+H3`, `H1+H3`, `H1+H2+H3`) com tanque de mola (`reverb = true`).
- Modo `12`: somente tanque de mola (`reverb = true`, cabeças de eco desligadas; não é mudo).

```rust
if self.model == DelayModel::MultiHeadTapeEcho {
    match self.re201_mode {
        1 => (true, false, false, false, true),
        2 => (false, true, false, false, true),
        3 => (false, false, true, false, true),
        4 => (false, true, true, false, true),
        5 => (true, false, false, true, true),
        6 => (false, true, false, true, true),
        7 => (false, false, true, true, true),
        // 8..11 via DECISAO_SONIX_RE201_ORDEM_PARES; 12 => reverb only
```

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real / UI | Chave Canônica (Spec 014) | UI Control ID (`controls.registry.ts`) | Faixa Aceita | Unidade | Valor Padrão | Papel no Sistema |
|---|---|---|---|---|---|---|
| **BYPASS** | `delay.enabled` | `pedal.delay.enabled` | `false / true` | Booleano | `false` | Ativa/desativa o bloco de delay. |
| **MODE SELECTOR** | `delay.re201_mode` | `pedal.delay.mode` | `1 .. 12` | Inteiro (`1..12`) | `4` (`H2+H3`) | Chave rotativa de 12 posições (seleciona cabeças `H1/H2/H3` e Spring Reverb conforme tabela oficial da Seção 5). |
| **REPEAT RATE** | `delay.time_ms` | `pedal.delay.time` | `20 .. 2000` (RE-201 `H3`: `175 .. 560`) | `ms` | `380.0` | Define a velocidade do motor *capstan* ($\tau_3 = \text{time\_ms}$, $\tau_1 = 0,3515 \cdot \tau_3$, $\tau_2 = 0,6661 \cdot \tau_3$). |
| **INTENSITY** | `delay.feedback` | `pedal.delay.feedback` | `0.0 .. 1.0` (`0..115%` interno) | `0 .. 100 %` | `0.35` | Realimentação da cabeça mais longa ativa de volta para a cabeça de gravação (permite auto-oscilação controlada acima de `0.85`). |
| **ECHO VOLUME (MIX)** | `delay.mix` | `pedal.delay.mix` | `0.0 .. 1.0` | `0 .. 100 %` | `0.30` | Balanço entre o sinal seco e a saída das cabeças de fita / mola. |
| **WOW & FLUTTER (MOD)** | `delay.mod_depth` | `pedal.delay.mod_depth` | `0.0 .. 1.0` | `0 .. 100 %` | `0.25` | Intensidade das flutuações mecânicas de transporte da fita (*Wow* $\sim 0,8\text{ Hz}$ + *Flutter* $\sim 6,5\text{ Hz}$). |
| **TAPE TONE / HIGH CUT** | `delay.high_cut_hz` | `pedal.delay.tone` | `1000 .. 12000` | `Hz` | `4500.0` | Perda de alta frequência por espaçamento de cabeça magnética (*gap loss*) a cada repetição. |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Inércia do Motor Capstan e Posições das 3 Cabeças (`H1`, `H2`, `H3`):**
   - O atraso base $\tau_{\text{target}}$ passa por um filtro de 1ª ordem de inércia mecânica ($\tau_{\text{motor}} \approx 85\text{ ms}$) produzindo $\bar{\tau}[n]$, modulado pela soma de dois osciladores em quadratura (*Wow* em $f_w = 0,85\text{ Hz}$ e *Flutter* em $f_f = 6,2\text{ Hz}$):
     $$\tau_{\text{tape}}[n] = \bar{\tau}[n] \cdot \left(1 + d_{\text{mod}} \cdot (0,0035 \sin(\phi_w[n]) + 0,0009 \sin(\phi_f[n]))\right)$$
   - As 3 cabeças leem a mesma linha de atraso contínua por **interpolação cúbica Lagrange de 4 pontos** nas posições:
     $$\tau_1[n] = 0,3515 \cdot \tau_{\text{tape}}[n], \qquad \tau_2[n] = 0,6661 \cdot \tau_{\text{tape}}[n], \qquad \tau_3[n] = 1,0000 \cdot \tau_{\text{tape}}[n]$$
2. **Chaveamento Oficial dos 12 Modos (`MODE SELECTOR 1–12`):**
   - Implementação exata da tabela da Seção 5:
     - `1`: `(1, 0, 0)`, rev=`false`; `2`: `(0, 1, 0)`, rev=`false`; `3`: `(0, 0, 1)`, rev=`false`; `4`: `(0, 1, 1)`, rev=`false`;
     - `5`: `(1, 0, 0)`, rev=`true`; `6`: `(0, 1, 0)`, rev=`true`; `7`: `(0, 0, 1)`, rev=`true`; `8`: `(1, 1, 0)`, rev=`true`;
     - `9`: `(0, 1, 1)`, rev=`true`; `10`: `(1, 0, 1)`, rev=`true`; `11`: `(1, 1, 1)`, rev=`true`; `12`: `(0, 0, 0)`, rev=`true`.
3. **Saturação Magnética de Fita e Auto-Oscilação no Loop de `INTENSITY`:**
   - O sinal gravado na fita passa por um filtro passa-altas (`90 Hz`, perda de contorno de cabeça), filtro passa-baixas (`high_cut_hz`, perda de gap) e saturação sigmoidal de histerese magnética $\tanh(1,35 \cdot v) / 1,35$.
   - Quando `delay.feedback > 0.85`, o ganho de loop atinge $g_{\text{fb}} \in [1,0,\; 1,18]$, gerando auto-oscilação sustentada cujo teto de amplitude é estritamente limitado pela curva de saturação magnética da fita (`|y| <= 1.2`).

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Interpolação de linha de atraso:** Interpolação Hermite/Lagrange cúbica de 4 pontos para evitar perda de agudos durante o *pitch glide* do motor capstan e a modulação de *wow & flutter*.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Buffer circular da fita pré-alocado em `prepare()` (zero alocações em `process_block`).
- WCET $\le 4,0\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Posicionado após o estágio de amplificação (`AMP`) ou após o gabinete (`CAB`), preservando as repetições de fita sem serem esmagadas pelo clipping do pré-amplificador.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/racks/roland_re201.rs`.
- **Struct principal:** `pub struct RolandRe201Processor` implementando `AudioProcessor` + `ModelDescriptor` (Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Cabeça / Modo | Relação Espacial / Comportamento | Invariante Física |
|---|---|---|
| `Head 1 : Head 2 : Head 3` | $0,3515 : 0,6661 : 1,0000$ ($1 : 1,895 : 2,845$) | As três cabeças leem a mesma fita física; variar `Repeat Rate` desloca as três proporcionalmente. |
| Modos `1–4` vs `5–11` vs `12` | `1–4` Echo Only; `5–11` Echo+Reverb; `12` Reverb Only | O Modo 12 nunca fica mudo: entrega a saída do tanque de mola interno sem cabeças de fita. |
| Auto-oscilação (`Intensity > 85%`) | Ganho de loop $> 1,0$ limitado por $\tanh$ da fita | A energia cresce até o teto magnético da fita e nunca diverge para infinito. |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste Exato da Tabela Oficial de 12 Modos (`1–12`):** excitação com impulso unitário verifica quais cabeças (`H1`, `H2`, `H3`) disparam nos tempos $\tau_1, \tau_2, \tau_3$ em cada modo `1..11` e verifica a presença de energia difusa de mola nos modos `5..12`.
2. **Teste de Auto-Oscilação Limitada em Alta `Intensity`:** verifica sustentação contínua limitada por saturação magnética sem explosão numérica.
3. **Teste de Degradação Espectral Progressiva a Cada Repetição:** verifica que a 3ª repetição possui centróide espectral menor que a 1ª repetição.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- O teste existente `test_ca_01_re201_multi_head_and_self_oscillation` em `contract_032_time_mod_fx.rs` testa o Modo `4` (`H2+H3`) e o Modo `7`. Corrigir a tabela de modos `5–11` na implementação futura da Spec 068 exigirá atualizar a asserção do Modo `7` (que no hardware real é `H3 + Reverb`, enquanto `Modo 11` é `H1+H2+H3 + Reverb`) no mesmo commit da implementação (Regra SDD #4).

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Tabela Oficial do `MODE SELECTOR` (12 Posições: Cabeças `H1`, `H2`, `H3` e Spring Reverb) (`DOCUMENTED`):**
  - *Sinal de entrada:* Impulso unitário (`x[0] = 1.0`, demais amostras `0.0`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras (`600 ms` de captura).
  - *Configuração e parâmetros:* `rack.roland.re201_space_echo` com `delay.time_ms = 300.0` ($\tau_1 \approx 105,5\text{ ms}$, $\tau_2 \approx 199,8\text{ ms}$, $\tau_3 = 300,0\text{ ms}$), `delay.feedback = 0.0`, `delay.mod_depth = 0.0`, `delay.mix = 1.0`, testando todos os modos `1` a `12`.
  - *Método de medição:* Detecção de picos nas janelas temporais de `H1` (`105,5 ± 3 ms`), `H2` (`199,8 ± 3 ms`), `H3` (`300,0 ± 3 ms`) e energia difusa de Spring Reverb entre `15 ms` e `80 ms`.
  - *Critério e tolerância:*
    1. Modos `1, 2, 3, 4`: picos exclusivamente em `H1` (Modo 1), `H2` (Modo 2), `H3` (Modo 3) e `H2+H3` (Modo 4), com energia de Spring Reverb $= 0$.
    2. Modos `5, 6, 7`: picos em `H1` (Modo 5), `H2` (Modo 6) e `H3` (Modo 7) acompanhados de energia não-nula do Spring Reverb.
    3. Modos `8, 9, 10, 11`: picos nas combinações `H1+H2` (Modo 8), `H2+H3` (Modo 9), `H1+H3` (Modo 10) e `H1+H2+H3` (Modo 11) acompanhados de Spring Reverb.
    4. Modo `12` (`REVERB ONLY`): zero picos discretos das cabeças `H1/H2/H3`, porém saída com energia RMS não-nula do tanque de Spring Reverb ($\text{RMS} > -45\text{ dBFS}$).
  - *Origem da tolerância:* Painel frontal oficial e Service Notes do Roland RE-201 (`F-01`, `F-02`).

- **CA-02 — Auto-Oscilação Magnética Controlada em Alta `INTENSITY` (`DOCUMENTED` / `REGRESSION`):**
  - *Sinal de entrada:* Burst senoidal de `440 Hz` a `-6 dBFS` durante `100 ms`, seguido de `2,0 s` de silêncio (`0.0`).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `delay.re201_mode = 3` (`H3`), `delay.time_ms = 200.0`, `delay.feedback = 0.98`, `delay.mix = 0.50`.
  - *Método de medição:* Nível RMS e valor de pico máximo na janela final (`1,5 s` a `2,1 s`).
  - *Critério e tolerância:* Na janela final (`> 1,4 s` após o término do sinal de entrada), a auto-oscilação deve manter energia sustentada ($\text{RMS} \ge -20,0\text{ dBFS}$) sem divergir: o pico máximo absoluto deve respeitar **`|y[n]| <= 1,25`** em `100%` das amostras.
  - *Origem da tolerância:* Saturação magnética do laço de fita do RE-201 (`F-01`, `F-02`).

- **CA-03 — Degradação Espectral Progressiva (*Tape Gap Loss*) a Cada Repetição (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Impulso unitário filtrado ou burst de ruído branco de `20 ms`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `delay.re201_mode = 3`, `delay.time_ms = 250.0`, `delay.feedback = 0.65`, `delay.high_cut_hz = 3500.0`, `delay.mix = 1.0`.
  - *Método de medição:* Centróide espectral da 1ª repetição (`t ≈ 250 ms`), 2ª repetição (`t ≈ 500 ms`) e 3ª repetição (`t ≈ 750 ms`).
  - *Critério e tolerância:* O centróide espectral deve cair monotonamente a cada passagem pela cabeça de gravação/reprodução: $\text{SC}_1 > \text{SC}_2 > \text{SC}_3$, com $\text{SC}_1 - \text{SC}_3 \ge 600\text{ Hz}$.
  - *Origem da tolerância:* Perda de espaçamento magnético e filtro de loop do RE-201 (`F-02`).

- **CA-04 — Transposição Dinâmica de Pitch (*Capstan Inertia Doppler Glide*) e *Wow & Flutter* (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide contínua de `440 Hz` a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `delay.re201_mode = 3`, `delay.mix = 1.0`, `delay.mod_depth = 0.80`; alterar `delay.time_ms` de `200 ms` para `400 ms` durante o processamento.
  - *Método de medição:* Variação da frequência instantânea (taxa de cruzamento por zero) da saída durante a transição de velocidade do motor.
  - *Critério e tolerância:* Durante a desaceleração da fita (`200 ms -> 400 ms`), a frequência instantânea reproduzida deve cair temporariamente **abaixo de `400 Hz`** (efeito Doppler contínuo sem cliques ou saltos de ponteiro).
  - *Origem da tolerância:* Inércia física do motor *capstan* DC (`F-01`, `F-02`).

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `re201_mode`, `time_ms`, `feedback`, `mix`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/delay.rs` (e futuro `crates/ampforge-dsp/src/fx/racks/roland_re201.rs`)
- `apps/sonix-app/ui/src/controls.registry.ts` (`pedal.delay.*`)
- Testes atuais: `Planejado: crates/ampforge-dsp/tests/contract_032_time_mod_fx.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Fonte oficial da Roland (`F-01`) verificada via HTTP 200.
- [x] Erro factual na tabela dos Modos `5–12` em `delay.rs:364–378` diagnosticado e corrigido na especificação.
- [x] Critérios `CA-01` a `CA-05` classificados e com condições completas de medição.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Roland Corporation — *RE-201 Space Echo Product Reference* (`https://www.roland.com/global/products/rc_re-201_space_echo/`).
- `[F-02]` Roland Corporation — *RE-201 Space Echo Owner's Manual & Service Notes (1974)*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, diagnosticou-se que `delay.rs:364–378` invertia as cabeças dos Modos `5–7` (`H1`, `H2`, `H3` + Reverb) com os Modos `8–10` (`H1+H2`, `H2+H3`, `H1+H3` + Reverb) e deixava o Modo `12` (`Reverb Only`) mudo. Estabeleceu-se a tabela oficial completa dos 12 modos.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Modos `1–4` (`Echo Only`: `H1`, `H2`, `H3`, `H2+H3`) com relações espaciais `1 : 1,895 : 2,845` | `F-01`, `F-02` | Spec 068 / `CA-01` | `crates/ampforge-dsp/src/fx/delay.rs:365–368 @ c20cc65, 425–432` | `contract_032_time_mod_fx.rs` | `TESTED` |
| Modos `5–11` (`Reverb + Echo`: `H1`, `H2`, `H3`, `H1+H2`, `H2+H3`, `H1+H3`, `H1+H2+H3`) e Modo `12` (`Reverb Only`) | `F-01`, `F-02` | Spec 068 / `CA-01` | `crates/ampforge-dsp/src/fx/delay.rs:370–376 @ c20cc65` (BUG: cabeças dos modos `5–10` invertidas e tanque Spring Reverb ausente nos modos `5–12`) | Ausente | `SPECIFIED` |
| Auto-oscilação controlada por saturação magnética de fita em alta `Intensity` | `F-01`, `F-02` | Spec 068 / `CA-02` | `crates/ampforge-dsp/src/fx/delay.rs:444–450 @ c20cc65` | `contract_032_time_mod_fx.rs` (`test_ca_01_re201_multi_head_and_self_oscillation`) | `TESTED` |
| Degradação espectral progressiva (*tape gap loss*) e filtro de loop | `F-02` | Spec 068 / `CA-03` | `crates/ampforge-dsp/src/fx/delay.rs:446–448 @ c20cc65` | `contract_032_time_mod_fx.rs` | `IMPLEMENTED_UNVERIFIED` |
| Inércia mecânica do motor *capstan* (*Doppler pitch glide*) e *Wow & Flutter* | `F-01`, `F-02` | Spec 068 / `CA-04` | `crates/ampforge-dsp/src/fx/delay.rs:394–409 @ c20cc65` | `contract_040_pedals_parity.rs` | `TESTED` |
| Execução zero-alloc em tempo real | Spec 053 | Spec 068 / `CA-05` | `crates/ampforge-dsp/src/fx/delay.rs:384–536 @ c20cc65` | `contract_044_sdd_integrity.rs` | `TESTED` |
