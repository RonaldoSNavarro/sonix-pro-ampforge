---
status: rascunho
---

# Spec 081: Componente Compartilhado — Motor Algorítmico de Delay, Modos Estéreo, Ducking e Spillover

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-shared` (`componente`)
> **Identificador canônico:** `component.delay.multi_engine`
> **Variante / Revisão:** `Motor Algorítmico Multi-Modo (Digital Clean, Analog BBD, Dual Delay, Reverse com Janelamento 5 ms, Ping-Pong Estéreo, Ducking e Spillover/Trails)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/delay.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md), [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [068](068-rack-roland-re201-space-echo.md)

---

## 1. Objetivo

Especificar o contrato técnico, algorítmico e de verificação do componente compartilhado **Motor de Delay Algorítmico, Ducking e Spillover (`component.delay.multi_engine`)** em [`crates/ampforge-dsp/src/fx/delay.rs`](../crates/ampforge-dsp/src/fx/delay.rs#L1-L559), isolando claramente a unidade eletromecânica **Roland RE-201 Space Echo** em sua especificação dedicada (**[Spec 068](068-rack-roland-re201-space-echo.md)**) e separando cada um dos **6 modos algorítmicos digitais/analógicos** (`DelayModel::PristineDigitalDelay`, `DelayModel::AnalogBbdDelay`, `DualDelay`, `Reverse`, `DelayModel::PingPongStereo`, além dos subsistemas transversais `Ducking`, `Spillover/Trails` e `Tap Tempo / Subdivisões Rítmicas`).

---

## 2. Escopo

1. **6 Sub-Motores Algorítmicos Distintos:**
   - **`DelayModel::PristineDigitalDelay`:** Linha de atraso de banda larga transparente (`20 Hz – 20 kHz`, `20..2000 ms`) com interpolação cúbica Lagrange e realimentação limpa com filtros opcionais `Low Cut` / `High Cut`.
   - **`DelayModel::AnalogBbdDelay` (estilo Boss DM-2 / MXR Carbon Copy `MN3005`):** Linha de atraso com degradação progressiva de agudos (`~2,5–3,5 kHz` por repetição), compressão/saturação quente tipo compander (`NE570` / `MN3005`) no loop e modulação sutil de clock (`Mod Depth`).
   - **`DualDelay` (estilo TC 2290 / U2 Dotted-8th + Quarter):** Duas linhas independentes (`Time L` e `Time R` em proporção configurável, ex.: `1,0 : 0,75` colcheia pontuada) com realimentação independente ou cruzada.
   - **`Reverse` (Reverso Granular Sem Cliques):** Gravação de grãos de comprimento `time_ms` e leitura em sentido reverso com **envelope de crossfade Hann/linear obrigatório nas bordas do grão (`>= 5,0 ms` de ataque e `>= 5,0 ms` de release)**, eliminando os cliques de salto de amostra nos limites de cada bloco reverso!
   - **`DelayModel::PingPongStereo` (Estéreo Cruzado L ↔ R):** A saída da linha esquerda alimenta a entrada da linha direita e vice-versa (`L → R → L → R`), com controle de largura estéreo.
   - **`Ducking` (Transversal / Dinâmico estilo TC 2290):** Detector de envoltória no sinal seco de entrada que atenua dinamicamente o retorno das repetições (`wet`) enquanto o guitarrista toca, liberando as caudas de eco apenas nas pausas das frases!
2. **Spillover / Trails (Transição Sem Corte de Caudas):** Ao colocar o pedal em bypass com `trails = true`, o motor zera apenas a **injeção de entrada** (`send = 0.0`) enquanto mantém o loop de leitura e realimentação ativo até a cauda decair abaixo de `-90 dBFS`.
3. **Sincronização `Tap Tempo` / BPM e Subdivisões Rítmicas (`1/4`, `1/8`, `1/8D` colcheia pontuada, `1/8T` tercina, `1/16`).**

---

## 3. Fora de escopo

- **Roland RE-201 Space Echo (3 cabeças de fita `H1/H2/H3`, 12 modos e tanque de mola integrado):** Coberto integralmente na **[Spec 068](068-rack-roland-re201-space-echo.md)**.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `componente` (`dsp-shared` / `multi-mode-delay-engine`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (Arquitetura algorítmica modular inspirada em delays digitais e BBD de estúdio) |
| **Modelo de referência** | Multi-Mode Stereo Delay, Ducking & Spillover Engine (`component.delay.multi_engine`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 081) |
| **Identificador no sistema** | `component.delay.multi_engine` (`crates/ampforge-dsp/src/fx/delay.rs`) |
| **Papel na cadeia de sinal** | Bloco `DELAY` pós-pré-amplificador ou pós-gabinete IR (antes do Reverb). |

---

## 5. Princípio físico e funcional

1. **Spillover / Trails Verdadeiro:**
   - Em um bypass simples (`if !enabled { return input; }`), desligar o delay durante um solo corta instantaneamente as repetições com um clique abrupto.
   - No modo **Spillover / Trails**, a chave de bypass desconecta apenas a entrada da linha de atraso (`in_send = if enabled { x } else { 0.0 }`), somando o sinal seco $x[n]$ com a cauda residual que continua circulando no buffer até extinguir-se naturalmente!
2. **Eliminação de Cliques no Modo `Reverse`:**
   - Ler um buffer circular de trás para frente exige saltar o ponteiro de leitura ao atingir o início do segmento de duração $T$. Sem janelamento, a descontinuidade $x[k] \to x[k+T]$ gera estalos periódicos a cada $T$ milissegundos. O motor `Reverse` utiliza duas cabeças reversas em contrafase ou envelope de rampa de **`>= 5,0 ms`** nas duas extremidades de cada segmento para garantir transição $C^0$-contínua.

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Udo Zölzer — *DAFX: Digital Audio Effects* (2nd ed., Wiley, 2011), Cap. 2 (*Delay-Based Audio Effects*) | Literatura acadêmica de referência | `ACADEMIC` | Linhas de atraso fracionárias, interpolação Lagrange/Hermite, feedback comb, Ping-Pong estéreo e modulação. |
| `F-02` | Julius O. Smith III — *Physical Audio Signal Processing: Delay Lines & Fractional Delay Interpolation* | `https://ccrma.stanford.edu/~jos/pasp/` (HTTP 200 verificado) | `ACADEMIC` | Precisão de fase e magnitude de interpoladores polinomiais cúbicos em atrasos variáveis. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Interpolação cúbica Lagrange/Hermite em linhas de atraso moduladas preserva agudos sem aliasing de passo | `F-01`, `F-02` | DAFX / PASP | `F-01` Cap. 2 §2.2 | `ACADEMIC` | `CONFIRMED` |
| Janelamento de borda (`>= 5 ms`) previne descontinuidades audíveis em leitura reversa por segmentos | `F-01` | DAFX 2nd ed. | Cap. 2 / Cap. 6 | `ACADEMIC` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Input L/R] ──┬─────────────────────────────────────────────────────────────────► (Dry) ──┐
              │                                                                           ├──► [Out L/R]
              ▼ (Send Gate: 1.0 if enabled, 0.0 if Spillover/Trails bypass)               │
     [Mode Router: Digital | BBD | Dual | Reverse (5ms win) | DelayModel::PingPongStereo] ──► [Ducking VCA] ┘ (Wet)
                                    ▲                                           ▲
                                    └── [Feedback + Low/High Cut] ──┘   [Dry Envelope Sidechain]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/fx/delay.rs`](../crates/ampforge-dsp/src/fx/delay.rs#L1-L559) apresenta os seguintes achados auditados:

1. **`MODEL_COUPLING` (`delay.rs:14–22`):** `DelayProcessor` mistura o equipamento eletromecânico `DelayModel::MultiHeadTapeEcho` (agora isolado na **Spec 068**) com os 5 modos algorítmicos digitais/BBD desta Spec 081.
2. **`MISSING_VALIDATION` (`delay.rs:480–515`):** O modo `Reverse` e o controle de `Ducking` precisam de testes de contrato dedicados validando ausência de cliques de borda no `Reverse` e atenuação mensurável em dB do `Ducking` enquanto há sinal de entrada.

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica (Spec 014) | Faixa Aceita | Unidade | Valor Padrão | Papel |
|---|---|---|---|---|---|
| **BYPASS** | `delay.enabled` | `false / true` | Booleano | `false` | Liga/desliga a injeção de entrada (respeitando `trails`). |
| **MODEL** | `delay.model` | `"digital"`, `"analog_bbd"`, `"tape_re201"`, `"dual"`, `"reverse"`, `"ping_pong"` | Enum | `"digital"` | Seleciona o motor algorítmico (ou delega para Spec 068 em `"tape_re201"`). |
| **TIME** | `delay.time_ms` | `20.0 .. 2000.0` | `ms` | `380.0` | Tempo de atraso principal (ou derivado de `bpm` + `subdivision` quando `sync = true`). |
| **FEEDBACK** | `delay.feedback` | `0.0 .. 0.98` | `%` | `0.35` | Realimentação das repetições. |
| **MIX** | `delay.mix` | `0.0 .. 1.0` | `%` | `0.30` | Balanço Dry/Wet. |
| **HIGH CUT / LOW CUT** | `delay.high_cut_hz` / `delay.low_cut_hz` | `1000..18000` / `20..1000` | `Hz` | `8000` / `80` | Filtros no loop de realimentação. |
| **DUCKING** | `delay.ducking` | `0.0 .. 1.0` | `%` | `0.0` | Intensidade da atenuação dinâmica do Wet pelo envelope do sinal Dry. |
| **SPILLOVER (TRAILS)** | `delay.trails` | `false / true` | Booleano | `true` | Mantém a cauda das repetições decaindo suavemente ao acionar bypass. |

---

## 9. Modelo matemático e algoritmo DSP proposto

- **Buffers Circulares Estéreo Pré-Alocados em `prepare()`:** Capacidade máxima de `2,5 s` por canal (`240 000` amostras/canal a `96 kHz`), sempre em potência de 2 (`mask = len - 1`) para indexação bit-a-bit $O(1)$ sem módulo.
- **Spillover:** Multiplicador de entrada `send_gain` suavizado em `5 ms` (`1.0 → 0.0` ao desligar com `trails = true`), mantendo o loop de feedback e saída `wet` ativos.
- **Ducking:** Envelope seguidor do sinal seco ($\tau_{\text{att}} = 5\text{ ms}, \tau_{\text{rel}} = 180\text{ ms}$) atenuando o ganho de saída Wet em até `-18 dB` proporcionalmente a `delay.ducking`.
- **Reverse:** Janela de crossfade de `min(5 ms, time_ms * 0.25)` nas bordas de cada bloco reverso.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `1×` com interpolação cúbica Lagrange de 4 pontos para transições suaves de tempo de atraso e modulação de clock BBD.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Buffers pré-alocados integralmente em `prepare()`. Zero alocações na heap em `process_block`.
- WCET $\le 3,5\text{ }\mu\text{s}$ por bloco de `128` amostras estéreo a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Pós-pré-amplificador (FX Loop) ou pós-gabinete IR (antes do Reverb da Spec 082).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/delay/` (separando `digital.rs`, `bbd.rs`, `dual.rs`, `reverse.rs`, `ping_pong.rs` e `spillover.rs`, com `RolandRe201Processor` da Spec 068 desacoplado).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Regra Verificável |
|---|---|
| Precisão temporal (`DelayModel::PristineDigitalDelay`) | Impulso unitário reaparece exatamente em `round(time_ms * fs / 1000)` $\pm 1$ amostra. |
| Alternância `DelayModel::PingPongStereo` | 1ª repetição exclusivamente em `L`, 2ª exclusivamente em `R`, 3ª em `L`. |
| Continuidade `Spillover / Trails` | Ao desligar `enabled = false` com `trails = true`, repetições já gravadas continuam saindo sem corte abrupto. |

---

## 15. Estratégia de testes, validação e benchmark

1. Testes individuais para cada modo (`DelayModel::PristineDigitalDelay`, `DelayModel::AnalogBbdDelay`, `DelayModel::PingPongStereo`, `Reverse` sem cliques, `Ducking`, `Spillover`).

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Em `sample_rate = 96 kHz` com atraso máximo de `2,0 s`, cada canal requer `~768 KB` de RAM (`~1,5 MB` estéreo), alocados uma única vez em `prepare()`.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Precisão Temporal (`DelayModel::PristineDigitalDelay`) e Alternância Estéreo (`DelayModel::PingPongStereo`) (`ANALYTICAL` / `REGRESSION`):**
  - *Sinal de entrada:* Impulso unitário mono (`L = R = 1.0` em $n = 0$).
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `delay.time_ms = 200.0` (`9600` amostras), `delay.feedback = 0.50`, `delay.mix = 1.0`.
  - *Critério e tolerância:*
    1. Em `DelayModel::PristineDigitalDelay`, picos exatos em `9600 ± 1` e `19200 ± 1` amostras com atenuação de `-6,02 dB ± 0,2 dB` entre repetições.
    2. Em `DelayModel::PingPongStereo`, a 1ª repetição (`200 ms`) possui razão `L / R >= 40 dB` e a 2ª repetição (`400 ms`) possui razão `R / L >= 40 dB`.

- **CA-02 — Spillover / Trails Contínuo ao Acionar Bypass (`REGRESSION`):**
  - *Sinal de entrada:* Impulso em $t = 0$; em $t = 100\text{ ms}$ (antes da 1ª repetição de `250 ms`), comuta-se `delay.enabled = false` com `delay.trails = true`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Critério e tolerância:* Em $t = 250\text{ ms}$ (com `enabled = false`), a repetição gravada antes do bypass **deve continuar audível** (`>= -12 dBFS`), enquanto um novo impulso injetado em $t = 150\text{ ms}$ **não** gera repetição em $t = 400\text{ ms}$ (`<= -80 dBFS`).

- **CA-03 — Atenuação Dinâmica `Ducking` Durante a Execução (`ANALYTICAL`):**
  - *Sinal de entrada:* Impulso inicial em $t = 0$ seguido de senoide contínua `-6 dBFS` em `t ∈ [200 ms, 450 ms]` (cobrindo a 1ª repetição em `300 ms`) e silêncio em `t > 450 ms` (liberando a 2ª repetição em `600 ms`).
  - *Configuração e parâmetros:* `delay.time_ms = 300.0`, `delay.feedback = 0.70`, `delay.ducking = 1.0`, `delay.mix = 0.50`.
  - *Critério e tolerância:* O nível do retorno Wet durante a presença da senoide seca (`300 ms`) deve ser atenuado em **pelo menos `8,0 dB`** em comparação com o mesmo teste com `delay.ducking = 0.0`.

- **CA-04 — Modo `Reverse` Livre de Cliques de Borda (`Janela >= 5 ms`) (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoide contínua de `440 Hz` a `-12 dBFS` por `1,5 s`.
  - *Configuração e parâmetros:* `delay.model = "reverse"`, `delay.time_ms = 250.0`, `delay.mix = 1.0`.
  - *Método de medição:* Derivada máxima amostra-a-amostra $\max |y[n] - y[n-1]|$ nas fronteiras de reinício do grão reverso (`250 ms, 500 ms, 750 ms`).
  - *Critério e tolerância:* A derivada máxima na fronteira do grão não deve exceder **`2,5×`** a derivada máxima da senoide de `440 Hz` (`<= 0,04` a `48 kHz`), comprovando ausência de estalos de borda.

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `time_ms`, `feedback`, `mix`, `model`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/delay.rs`
- Testes atuais: `crates/ampforge-dsp/tests/contract_032_time_mod_fx.rs`, `crates/sonix-desktop/tests/contract_040_pedals_parity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Roland RE-201 isolado exclusivamente na Spec 068; modos algorítmicos (`Digital`, `BBD`, `Dual`, `Reverse`, `DelayModel::PingPongStereo`, `Ducking`, `Spillover`) especificados separadamente.
- [x] Critérios `CA-01` a `CA-05` classificados e mensuráveis.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Zölzer, U. (2011). *DAFX: Digital Audio Effects* (2nd ed.). Wiley, Cap. 2.
- `[F-02]` Smith, J. O. (2010). *Physical Audio Signal Processing* (`https://ccrma.stanford.edu/~jos/pasp/`).

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, formalizou-se a separação dos modos algorítmicos de delay da unidade eletromecânica RE-201 (Spec 068) e exigiu-se janelamento mínimo de `5 ms` nas bordas do modo `Reverse` (`CA-04`).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Precisão temporal (`DelayModel::PristineDigitalDelay`) e alternância estéreo (`DelayModel::PingPongStereo`) | `F-01` | Spec 081 / `CA-01` | `crates/ampforge-dsp/src/fx/delay.rs:452–530 @ c20cc65` | `contract_032_time_mod_fx.rs`, `contract_040_pedals_parity.rs` | `TESTED` |
| `Spillover / Trails` mantendo caudas sem corte ao acionar bypass | Spec 032 | Spec 081 / `CA-02` | `crates/ampforge-dsp/src/fx/delay.rs:384–392 @ c20cc65` | `contract_032_time_mod_fx.rs` (`test_ca_04_delay_spillover_and_reverb_tails`) | `TESTED` |
| Atenuação dinâmica `Ducking` pelo envelope Dry | Spec 032 | Spec 081 / `CA-03` | `crates/ampforge-dsp/src/fx/delay.rs:515–525 @ c20cc65` | `contract_040_pedals_parity.rs` | `IMPLEMENTED_UNVERIFIED` |
| Modo `Reverse` com janelamento `>= 5 ms` livre de cliques de borda | `F-01` | Spec 081 / `CA-04` | `crates/ampforge-dsp/src/fx/delay.rs:480–510 @ c20cc65` | Ausente (sem validação de derivada nas bordas) | `IMPLEMENTED_UNVERIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 081 / `CA-05` | `crates/ampforge-dsp/src/fx/delay.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
