---
status: rascunho
---

# Spec 084: Componente Compartilhado — Roteamento DAG, Split/Merge, Fila SPSC Lock-Free e Matriz de Modifiers

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-core` (`componente`)
> **Identificador canônico:** `component.routing.dag_spsc_modifiers`
> **Variante / Revisão:** `Grafo Acíclico de Roteamento (Série, Paralelo Dual-Amp, Wet-Dry-Wet, Bi-Amp Crossover) + Fila SPSC Lock-Free Auditada + Matriz de Modifiers (LFO / Envelope / MIDI CC)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/dag.rs`, `crates/ampforge-dsp/src/queue.rs`, `crates/sonix-desktop/src/audio.rs`
> **Specs relacionadas:** [044](044-integridade-do-sdd-codigo-morto-e-testes-tautologicos.md), [052](052-abertura-animada-pedais-faltantes-e-amp2-visivel.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [080](080-componente-gabinete-convolucao-ir-e-microfonacao.md)

---

## 1. Objetivo

Especificar o contrato técnico, de concorrência em memória (*lock-free memory ordering*), de roteamento de sinal e de verificação do **Componente de Roteamento DAG, Split/Merge, Fila SPSC e Modifiers (`component.routing.dag_spsc_modifiers`)** em [`crates/ampforge-dsp/src/dag.rs`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/dag.rs#L1-L529), [`crates/ampforge-dsp/src/queue.rs`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/queue.rs#L1-L275) e [`crates/sonix-desktop/src/audio.rs:2558–2693 @ c20cc65`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/sonix-desktop/src/audio.rs#L2558-L2693), corrigindo e auditando três pontos críticos encontrados no código real:
1. **Risco de Alocação na Heap em `MixerNode::configure_channel` (`dag.rs`):** A função atual realiza `Vec::resize` caso `channel_index >= self.channels.len()`, o que **nunca pode ser invocado na thread de áudio RT**;
2. **Auditoria de Invariantes `unsafe` da Fila SPSC (`queue.rs:1–275`):** Formalização obrigatória das provas de segurança `// SAFETY:` (`UnsafeCell`, ordenação de memória `Acquire`/`Release` nos índices `head`/`tail`, invariante estrito de produtor único e consumidor único, e comportamento `Drop` seguro sem vazamentos);
3. **Atualização das Referências de Código em `audio.rs`:** Atualização dos números de linha do motor de roteamento paralelo `AMP 1 / AMP 2` (implementado na Spec 052) em [`crates/sonix-desktop/src/audio.rs:2558–2693 @ c20cc65`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/sonix-desktop/src/audio.rs#L2558-L2693).

---

## 2. Escopo

1. **Grafo de Roteamento Acíclico (DAG) Pré-Alocado (`dag.rs`):**
   - **Série (`Serial`):** Cadeia linear clássica `Input → Gate → Wah → Pitch → Comp → Drive → Amp1 → Cab1 → Mod1 → Mod2 → Delay → Reverb → Out`.
   - **Paralelo Dual-Amp (`ParallelDualAmp` — Spec 052):** Bifurcação (`SplitterNode`) enviando sinal para `Path A (Amp1 + Cab1)` e `Path B (Amp2 + Cab2)` com `Pan`, `Level`, `Phase Invert (180°)` e `Delay Compensation` no `MixerNode`!
   - **Bi-Amp Crossover (`CrossoverSplit`):** `SplitterNode` com crossover complementar Linkwitz-Riley `LR4` ajustável (`100 Hz – 4000 Hz`), enviando graves (`< f_x`) para um amplificador (ex.: Fender Twin limpo) e agudos (`> f_x`) para outro (ex.: Dual Rectifier saturado).
   - **Wet-Dry-Wet (`WDW`):** Sinal seco de gabinete no centro (`Mono`) somado a retornos `100% Wet` estéreo (`Delay` + `Reverb`) nas laterais L/R.
2. **Fila SPSC Lock-Free Bidirecional (`queue.rs`):**
   - `Control → RT`: envio de comandos atômicos de parâmetros, troca de preset e ponteiros de novas IRs (`PreparedIr`).
   - `RT → Control`: envio de telemetria (VU meters, GR, tuner snapshots) e **fila de devolução de recursos antigos (`trash_tx/rx`)** para desalocação segura na Control Thread (Spec 080 CA-02).
3. **Matriz de Modifiers em Tempo Real (`Modifiers`):**
   - Fontes de modulação (`Expression Pedal`, `MIDI CC 0..127`, `Envelope Follower`, `Global LFO`) acopláveis a qualquer parâmetro contínuo (`wah.position`, `pitch.expression`, `amp.gain`, `delay.mix`, etc.) com curva `Min/Max/Mid` e suavização `Damping ms`.

---

## 3. Fora de escopo

- Grafos cíclicos arbitrários com realimentação entre blocos externos sem atraso explícito (todos os grafos de usuário são validados como DAG estritamente acíclicos antes do envio à thread RT).
- Protocolos de áudio do sistema operacional (`WASAPI` / `ASIO` / `CoreAudio`, cobertos pelas Specs de infraestrutura desktop).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `componente` (`dsp-core` / `routing-spsc-modifiers`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (Arquitetura de infraestrutura de tempo real em Rust) |
| **Modelo de referência** | Zero-Alloc Audio DAG, SPSC Queue & Modifier Matrix (`component.routing.dag_spsc_modifiers`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 084) |
| **Identificador no sistema** | `component.routing.dag_spsc_modifiers` (`dag.rs`, `queue.rs`, `audio.rs:2558–2693`) |
| **Papel na cadeia de sinal** | Espinha dorsal de roteamento, comunicação entre threads e automação de todo o motor de áudio. |

---

## 5. Princípio físico e funcional

1. **Ordenação de Memória `Acquire` / `Release` em Fila SPSC Lock-Free (`queue.rs`):**
   - Em arquiteturas modernas (x86-64 e ARM64), uma fila circular SPSC com capacidade $N$ (potência de 2) usando `AtomicUsize` `head` (escrito apenas pelo consumidor RT) e `tail` (escrito apenas pelo produtor Control) garante ausência de *data races* e de bloqueios (*wait-free*) sob as seguintes ordens de memória:
     - **Produtor (`push`):** lê `tail` (`Relaxed`), lê `head` (`Acquire`), escreve no slot `UnsafeCell<MaybeUninit<T>>`, e publica `tail.store(next_tail, Release)`.
     - **Consumidor (`pop`):** lê `head` (`Relaxed`), lê `tail` (`Acquire`), lê do slot via `MaybeUninit::assume_init_read()`, e publica `head.store(next_head, Release)`.
   - Toda operação `unsafe` em `queue.rs` deve possuir comentário `// SAFETY:` auditável e ser verificada sob teste de estresse concorrente multi-thread (e `Miri` quando aplicável).
2. **Coerência de Fase e Alinhamento Automático de Latência (*PDC — Plugin/Path Delay Compensation*) no `MixerNode`:**
   - Quando o ramo `Path A` possui latência $D_A$ amostras (ex.: `8×` FIR oversampler ou `Whammy`) e o ramo `Path B` possui latência $D_B \ne D_A$, o `MixerNode` atrasa automaticamente o ramo mais rápido por $|D_A - D_B|$ amostras (usando linhas de compensação pré-alocadas em `prepare()`) para evitar cancelamento *comb-filter* na soma!

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Mara Bos — *Rust Atomics and Locks: Low-Level Concurrency in Practice* (O'Reilly, 2023) | Referência oficial de concorrência lock-free em Rust (`Acquire`/`Release`, `UnsafeCell`, `MaybeUninit`) | `SECONDARY_TECHNICAL` | Padrão formal de provas `// SAFETY:` e implementação correta de ring buffers SPSC em Rust. |
| `F-02` | Ross Bencina — *Real-time audio programming 101: time waits for nothing* (2011) | Referência clássica da indústria de áudio em tempo real | `SECONDARY_TECHNICAL` | Regras de proibição de locks, alocações, `Drop` na RT e uso de filas SPSC para troca de estado e reciclagem de memória. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Ordenação `Acquire`/`Release` em `head`/`tail` + `UnsafeCell<MaybeUninit<T>>` é necessária e suficiente para SPSC sem *Undefined Behavior* | `F-01` | 2023 | Cap. 3 & Cap. 5 | `SECONDARY_TECHNICAL` | `CONFIRMED` |
| Proibição de `Vec::resize` ou alocação/desalocação na thread de áudio RT | `F-02`, Regra Cortex `d272` | 2011 / 2026 | `AGENTS.md` | `INTERNAL_REFERENCE` / `SECONDARY_TECHNICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Control / UI Thread] ═══(SPSC cmd_tx)═══► [RT Audio Thread: DAG Engine]
                                              ├── Input Conditioning (Spec 076)
                                              ├── Modifiers Matrix (Env/LFO/MIDI CC)
                                              ├── SplitterNode (Parallel / LR4 Crossover)
                                              │      ├── Path A (Amp 1 + Cab 1)
                                              │      └── Path B (Amp 2 + Cab 2)
                                              ├── MixerNode (PDC + Pan + Level + Phase)
                                              └── Post-FX (Delay + Reverb + Looper)
[Control / UI Thread] ◄══(SPSC trash_rx / telemetry_rx)═══ [RT Audio Thread]
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`FACTUAL_ERROR` / Risco RT (`crates/ampforge-dsp/src/dag.rs` — `MixerNode::configure_channel`):** A função `configure_channel` chama `Vec::resize` dinamicamente se o índice exceder `len()`. Para garantir invariante zero-alloc absoluta, `MixerNode` deve usar capacidade fixa pré-alocada (`MAX_CHANNELS = 8`, array estático `[ChannelConfig; 8]`), eliminando qualquer `Vec::resize` em tempo de execução!
2. **`CODE_REFERENCE_STALE` (corrigido nesta revisão):** O código de roteamento `AMP 1 / AMP 2` paralelo e a aplicação de parâmetros de roteamento residem atualmente em [`crates/sonix-desktop/src/audio.rs:2558–2693 @ c20cc65`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/sonix-desktop/src/audio.rs#L2558-L2693) (atualizado pela Spec 052).
3. **`MISSING_VALIDATION` (`crates/ampforge-dsp/src/queue.rs`):** Os blocos `unsafe` do ring buffer SPSC precisam ter cada invariante `// SAFETY:` explicitamente documentado e coberto por teste de estresse concorrente multi-thread (1 milhão de mensagens) validando ordem FIFO exata e zero perda/corrupção sob contenção.

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica (Spec 014 / Spec 052) | Faixa Aceita | Unidade | Valor Padrão | Papel |
|---|---|---|---|---|---|
| **ROUTING MODE** | `routing.mode` | `"serial"`, `"parallel"`, `"crossover"`, `"wet_dry_wet"` | Enum | `"serial"` | Topologia ativa do DAG. |
| **AMP 2 ENABLED** | `amp2.enabled` | `false / true` | Booleano | `false` | Ativa a cadeia paralela `Path B` (`Amp 2 + Cab 2`). |
| **SPLIT CROSSOVER HZ** | `routing.crossover_hz` | `100.0 .. 4000.0` | `Hz` | `500.0` | Frequência de corte do crossover `LR4` no modo `"crossover"`. |
| **PATH A / B PAN & LEVEL** | `routing.pan_a/b`, `routing.level_a/b` | `-1.0..+1.0` / `-24..+12 dB` | Pan / dB | `0.0` / `0.0 dB` | Balanço estéreo e ganho dos ramos no `MixerNode`. |
| **PATH B PHASE** | `routing.phase_b` | `false / true` | Booleano | `false` | Inversão de polaridade ($180^\circ$) de `Path B`. |

---

## 9. Modelo matemático e algoritmo DSP proposto

- **`MixerNode` Estático (`MAX_CHANNELS = 8`):** Array inline `[MixerChannel; 8]` sem `Vec`, com lei de pan de potência constante (`-3 dB` no centro: $g_L = \cos(\theta), g_R = \sin(\theta)$) e compensação de atraso `PDC` por canal.
- **Topologia `CrossoverSplit`:** Crossover Linkwitz-Riley de 4ª ordem (`LR4`), garantindo soma plana $H_{\text{LP,LR4}}(s) + H_{\text{HP,LR4}}(s) \equiv 1,0$ (`0,0 dB` de ripple na frequência de cruzamento!).
- **Modifiers:** Mapeamento contínuo $p_{\text{out}} = p_{\min} + (p_{\max} - p_{\min}) \cdot f_{\text{curve}}(m_{\text{smooth}}[n])$ com `clamp` estrito à faixa declarada do parâmetro alvo.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Precisão:** Estados dos filtros `LR4` do `SplitterNode` em `f64`; buffers de roteamento entre nós pré-alocados em `prepare()` para o `max_block_size` da sessão (`<= 4096` amostras).

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações e zero desalocações em todo o grafo DAG durante `process_block` e `configure_channel` (eliminação de `Vec::resize`).
- Zero `Mutex` / `RwLock` na fila SPSC `queue.rs`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Orquestra a execução topológica de todos os blocos (`Specs 054–092`) garantindo sincronia de bloco, PDC e comunicação lock-free com a UI Tauri v2.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulos:** `crates/ampforge-dsp/src/dag.rs`, `crates/ampforge-dsp/src/queue.rs`, `crates/ampforge-dsp/src/modifiers.rs` (futuro).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Regra Verificável |
|---|---|
| Soma Plana do Crossover `LR4` | `Low + High` em fase produz magnitude plana **`0,00 dB ± 0,05 dB`** de `20 Hz` a `20 kHz`. |
| Integridade FIFO da Fila SPSC | `1 000 000` de mensagens enviadas entre 2 threads chegam em ordem estritamente crescente `0..999_999` sem perda, duplicação ou *data race*. |
| Zero `Vec::resize` em `MixerNode` | Configurar todos os canais (`0..7`) resulta em `0` alocações na heap. |

---

## 15. Estratégia de testes, validação e benchmark

1. Teste de soma plana do `SplitterNode` `LR4` + `MixerNode`.
2. Teste de estresse concorrente multi-thread da fila SPSC `queue.rs` com `1 000 000` de elementos e verificação de `Drop` exato de tipos não-`Copy` (`Box<T>`).
3. Teste zero-alloc de `MixerNode::configure_channel` e do roteamento `ParallelDualAmp`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Todos os tipos enviados pela fila SPSC `queue.rs` devem implementar `Send`; tipos contendo ponteiros de heap (como `Box<PreparedIr>`) nunca podem ser descartados na thread RT se a fila estiver cheia — o produtor ou consumidor RT deve reter o valor em slot pré-alocado até drenagem.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Soma Plana de Crossover `LR4` (`0,00 dB ± 0,05 dB`) e Cancelamento de Fase no `MixerNode` (`ANALYTICAL`):**
  - *Sinal de entrada:* Varredura senoidal de `40 Hz` a `16 000 Hz` (incluindo a frequência exata de cruzamento `500 Hz`) a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* `SplitterNode` em modo `CrossoverSplit (500 Hz)` conectado diretamente ao `MixerNode` (`level_a = level_b = 0.0 dB`, `phase_b = false`).
  - *Critério e tolerância:* A soma `Low + High` deve apresentar magnitude plana de **`0,00 dB ± 0,05 dB`** em toda a faixa `40 Hz – 16 000 Hz` (incluindo `500 Hz`). Em modo `Parallel` com `Path A == Path B` e `phase_b = true`, a saída somada deve cancelar para **`<= -100 dBFS`**.

- **CA-02 — Integridade Concorrente FIFO e Drop Seguro da Fila SPSC (`queue.rs`) (`ENGINEERING_TARGET`):**
  - *Procedimento:* Duas threads concorrentes (`Producer` e `Consumer`) trocam `1 000 000` de mensagens sequenciais (`0..1_000_000`) através de uma `SpscQueue` de capacidade `256` (forçando milhares de voltas *wrap-around* e condições de fila cheia/vazia), além de um teste dedicado com `Box<DropCounter>` verificando que ao dropar uma fila parcialmente preenchida todos os itens remanescentes têm seu `Drop` executado exatamente `1` vez.
  - *Critério e tolerância:* `100%` dos valores recebidos na ordem exata `0..999_999`, `0` valores perdidos/corrompidos e contagem exata de `Drop` (sem vazamento nem *double-free*).

- **CA-03 — Zero-Alloc em `MixerNode::configure_channel` e Roteamento Dual-Amp (`ENGINEERING_TARGET` / `REGRESSION`):**
  - *Procedimento:* Invocar `configure_channel` para os canais `0..7` e processar `10 000` blocos de `128` amostras em modo `ParallelDualAmp` sob alocador global instrumentado na thread de teste (Regra Cortex `d272`).
  - *Critério e tolerância:* **`0` alocações e `0` desalocações** na heap (eliminando o `Vec::resize` atual de `dag.rs`).

- **CA-04 — Suavização Anti-Zipper e Limites Estritos da Matriz de Modifiers (`ANALYTICAL`):**
  - *Sinal de entrada:* Degrau instantâneo de `0.0 -> 1.0` na fonte `Expression Pedal` mapeada para `wah.position` com `damping_ms = 20.0`.
  - *Critério e tolerância:* A saída modulada deve permanecer estritamente dentro de `[min, max]` em `100%` das amostras e atingir `63,2%` do degrau em `20,0 ms ± 2,0 ms`.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/dag.rs`, `crates/ampforge-dsp/src/queue.rs`, `crates/sonix-desktop/src/audio.rs` (`L2558–2693`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_044_sdd_integrity.rs`, `crates/sonix-desktop/tests/contract_052_splash_pedals_amp2.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Risco de alocação `Vec::resize` em `MixerNode::configure_channel` (`dag.rs`) diagnosticado e coberto por `CA-03`.
- [x] Invariantes `unsafe` / `Acquire`-`Release` da fila SPSC (`queue.rs`) especificados e cobertos por `CA-02`.
- [x] Referências de linhas em `audio.rs` atualizadas (`L2558–2693`).
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Bos, M. (2023). *Rust Atomics and Locks*. O'Reilly Media.
- `[F-02]` Bencina, R. (2011). *Real-time audio programming 101: time waits for nothing*.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, auditou-se `dag.rs` (sinalizando `Vec::resize` em `MixerNode::configure_channel`) e `queue.rs` (exigindo provas `// SAFETY:` e teste de estresse concorrente + `Drop` seguro de tipos não-`Copy`).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| `SplitterNode` (`Parallel` e `LR4 Crossover` plano `0,00 dB ± 0,05 dB`) + `MixerNode` (`Phase/Pan/PDC`) | Spec 052, Spec 053 | Spec 084 / `CA-01` | `crates/ampforge-dsp/src/dag.rs:1–529 @ c20cc65`, `audio.rs:2558–2693` | `contract_052_splash_pedals_amp2.rs` | `IMPLEMENTED_UNVERIFIED` |
| Fila SPSC Lock-Free (`Acquire`/`Release`, `// SAFETY:`, estresse `1M` msgs, `Drop` sem leak) | `F-01`, `F-02` | Spec 084 / `CA-02` | `crates/ampforge-dsp/src/queue.rs:1–275 @ c20cc65` | `contract_044_sdd_integrity.rs` | `IMPLEMENTED_UNVERIFIED` |
| `MixerNode::configure_channel` zero-alloc (substituindo `Vec::resize` por array estático `[MixerChannel; 8]`) | `F-02`, Regra `d272` | Spec 084 / `CA-03` | `crates/ampforge-dsp/src/dag.rs` (RISCO: usa `Vec::resize`) | Ausente | `SPECIFIED` |
| Matriz de Modifiers (`Expression` / `MIDI CC` / `Envelope` / `LFO`) com suavização anti-zipper | Spec 053 | Spec 084 / `CA-04` | Parcial em `audio.rs` (sem módulo dedicado `modifiers.rs`) | Ausente | `SPECIFIED` |
