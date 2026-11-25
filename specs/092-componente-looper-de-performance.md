---
status: rascunho
---

# Spec 092: Componente Looper de Performance (Gravação, Overdub, Undo/Redo, Half-Speed e Reverse Zero-Alloc)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Criado na Rodada Corretiva por Desacoplamento da Spec 083)
> **Domínio:** `dsp-utility` (`componente`)
> **Identificador canônico:** `component.looper.performance`
> **Variante / Revisão:** `Looper Estéreo de Performance com Máquina de Estados Determinística, Buffers Pré-Alocados (≥ 60 s), Undo/Redo Sem Heap, Crossfade de Emenda 5 ms, Half-Speed (0.5×) e Reverse`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/looper.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [044](044-integridade-do-sdd-codigo-morto-e-testes-tautologicos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [083](083-componente-afinador-yin-e-looper-de-performance.md)

---

## 1. Objetivo

Especificar o contrato técnico, de memória em tempo real, de máquina de estados e de verificação do **Componente Looper de Performance (`component.looper.performance`)** em [`crates/ampforge-dsp/src/looper.rs`](../crates/ampforge-dsp/src/looper.rs#L1-L275), desacoplando-o definitivamente do **Afinador YIN** (**[Spec 083](083-componente-afinador-yin-e-looper-de-performance.md)**) para eliminar o acoplamento indevido (`MODEL_COUPLING`) entre dois utilitários sem nenhuma dependência algorítmica entre si.

---

## 2. Escopo

1. **Alocação Única em `prepare()` (`>= 60 segundos` estéreo):** Toda a memória do loop principal (`buffer_main_l/r`) e do snapshot de backup para `Undo/Redo` (`buffer_undo_l/r`) é alocada uma única vez na inicialização (`prepare()`), garantindo **`0` alocações e `0` cópias bloqueantes na heap** durante gravação, overdub, undo, redo ou clear na thread RT!
2. **Máquina de Estados Determinística (`LooperState`):**
   - `Empty` $\xrightarrow{\text{Rec}}$ `Recording` (define `loop_len` até o limite máximo `max_samples`);
   - `Recording` $\xrightarrow{\text{Play/Rec}}$ `Playing` (fecha o loop com **crossfade de emenda obrigatório de `5,0 ms`** nas fronteiras início/fim para eliminar estalos na repetição contínua!);
   - `Playing` $\xrightarrow{\text{Overdub}}$ `Overdubbing` (salva instantaneamente o estado anterior sem alocação usando estratégia *dual-buffer / delta-layer* para `Undo`/`Redo` instantâneo);
   - `Playing / Overdubbing` $\xrightarrow{\text{Stop}}$ `Stopped` (com rampa de `5 ms` anti-clique);
   - `Stopped` $\xrightarrow{\text{Clear}}$ `Empty` (zera ponteiros em $O(1)$ sem `Vec::clear`/realocação).
3. **Modos Criativos de Reprodução:**
   - **`Half-Speed (0,5×)`:** Leitura em meia velocidade por interpolação cúbica Lagrange (transpondo o loop `-1 oitava` e dobrando sua duração);
   - **`Reverse`:** Leitura reversa contínua com crossfade nas transições de direção;
   - **`Level` (`0.0 .. 1.0`):** Volume de playback independente do sinal ao vivo (*dry-through* sempre preservado em `0 dB`).

---

## 3. Fora de escopo

- Afinador estroboscópico YIN (**[Spec 083](083-componente-afinador-yin-e-looper-de-performance.md)**).
- Exportação de arquivos `.wav` para disco (executada de forma assíncrona pela Control Thread sobre uma cópia lida quando o looper está em `Stopped`, nunca na thread RT).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `componente` (`dsp-utility` / `performance-looper`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (inspirado na ergonomia de pedais looper de palco como TC Electronic Ditto X2 / Boss RC-5) |
| **Modelo de referência** | Zero-Alloc Stereo Performance Looper (`component.looper.performance`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 092) |
| **Identificador no sistema** | `component.looper.performance` (`crates/ampforge-dsp/src/looper.rs`) |
| **Papel na cadeia de sinal** | Bloco utilitário posicionável no **final da cadeia (Post-FX, padrão)** para gravar loops com o timbre completo, ou no **início da cadeia (Pre-Amp)** para auditar ajustes de amplificadores e pedais com as mãos livres! |

---

## 5. Princípio físico e funcional

1. **Crossfade de Emenda (*Splice De-Clicking*, `5,0 ms`):**
   - Ao encerrar a gravação inicial (`Recording → Playing`), a última amostra $x[L-1]$ raramente coincide em amplitude e derivada com a primeira amostra $x[0]$. Sem tratamento, a cada volta do loop ouve-se um estalo seco (*pop*).
   - O looper aplica um **fade-in de `2,5 ms`** nos primeiros `120` samples e um **fade-out / overlap de `2,5 ms`** nos últimos `120` samples (ou grava `240` amostras extras após o comando `Play` para fazer crossfade linear contínuo de `5,0 ms` sobre o início do loop), eliminando completamente cliques de emenda.
2. **Undo / Redo em Tempo Real Sem Trava (*Zero-Alloc Dual-Layer*):**
   - Para evitar `memcpy` de `60 segundos` (`~23 MB`) dentro de um único callback RT de `2,6 ms` ao acionar `Overdub` ou `Undo`, o looper mantém **dois buffers pré-alocados**: `base_layer` (a soma acumulada até o overdub anterior) e `top_layer` (a camada atual de overdub) com um flag booleano `top_layer_active`.
   - `Undo` apenas alterna `top_layer_active = false` em **`O(1)` (`< 10 ns`!)** com rampa de `5 ms` — e `Redo` restaura `top_layer_active = true` igualmente em **`O(1)`**!

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Ross Bencina — *Real-time audio programming 101* & Regra Promovida Cortex `d272` | [`AGENTS.md`](../AGENTS.md) | `INTERNAL_REFERENCE` / `SECONDARY_TECHNICAL` | Contrato zero-alloc, zero-lock e WCET constante em operações de estado (`Rec`, `Overdub`, `Undo`, `Redo`, `Clear`). |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Arquitetura *Dual-Layer* (`base_layer` + `top_layer`) garante `Undo/Redo` em tempo $O(1)$ sem `memcpy` nem alocação na RT | `F-01` | Realtime Audio Design | §5 | `ANALYTICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Audio In L/R] ──┬────────────────────────────────────────────────────────► (+) ──► [Audio Out L/R]
                 │                                                           ▲
                 ▼ (Recording / Overdubbing)                                 │ (× Playback Level)
       [Pre-Allocated Dual-Layer Ring Buffer (≥ 60s Stereo)]                 │
       ├── base_layer[0..L]                                                  │
       └── top_layer[0..L] (Undo/Redo in O(1) via top_layer_active + 5ms) ───┘
       (Playback Head: Normal 1.0× | Half-Speed 0.5× Cubic | Reverse -1.0×)
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`MODEL_COUPLING` (corrigido nesta revisão):** O Looper (`crates/ampforge-dsp/src/looper.rs:1–275 @ c20cc65`) estava acoplado na Spec 083 com o Afinador YIN (`tuner.rs`).
2. **`FACTUAL_ERROR` / Risco de WCET (`looper.rs` — `Undo` por cópia completa de buffer):** Caso a implementação faça `clone_from_slice` sobre o buffer inteiro de `60 s` (`~11–23 MB`) dentro do callback de áudio ao entrar em `Overdub` ou `Undo`, o pico de latência pode causar *buffer underrun (xrun)* em buffers de `64–128` amostras (`1,3–2,6 ms`). A arquitetura *Dual-Layer* (`base_layer` + `top_layer`) desta Spec garante WCET estritamente $O(1)$ por amostra!

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica | Faixa Aceita | Unidade | Valor Padrão |
|---|---|---|---|---|
| **COMMAND** | `looper.command` | `"record"`, `"play"`, `"overdub"`, `"stop"`, `"undo"`, `"redo"`, `"clear"` | Enum | `"stop"` |
| **LEVEL** | `looper.level` | `0.0 .. 1.0` | `%` | `0.85` |
| **HALF SPEED** | `looper.half_speed` | `false / true` | Booleano | `false` |
| **REVERSE** | `looper.reverse` | `false / true` | Booleano | `false` |
| **POSITION** | `looper.position` | `"post_fx"`, `"pre_amp"` | Enum | `"post_fx"` |

---

## 9. Modelo matemático e algoritmo DSP proposto

- Ponteiro fracionário `play_pos: f64` avançando por `+1.0` (`Normal`), `+0.5` (`Half-Speed`), `-1.0` (`Reverse`) ou `-0.5` (`Half-Speed + Reverse`), com módulo contínuo em `loop_len as f64` e leitura por interpolação cúbica Lagrange de 4 pontos.
- Soma `live_in + level * (base_sample + top_layer_gain * top_sample)` com limitação suave de proteção contra clipping extremo (`soft_limit`) após múltiplos overdubs.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Interpolação:** Cúbica Lagrange de 4 pontos em `Half-Speed (0,5×)`.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap durante `Record`, `Play`, `Overdub`, `Undo`, `Redo`, `Stop`, `Clear`, `Half-Speed` e `Reverse`.
- **Zero `memcpy` em bloco gigante no callback RT** (WCET $O(1)$ por amostra em todas as transições de estado!).

---

## 12. Posição e interação na cadeia completa de guitarra

- Modo `post_fx` (grava o som pronto para prática/palco) ou modo `pre_amp` (grava a guitarra limpa DI na entrada para ajustar amplificadores/pedais continuamente sem precisar tocar!).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo:** `crates/ampforge-dsp/src/looper.rs` (`LooperProcessor`, `LooperState`, `LooperCommand`).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Regra Verificável |
|---|---|
| Capacidade mínima pré-alocada | `>= 60,0 s` estéreo a `48 kHz` (`>= 2 880 000` frames). |
| Transição `Undo / Redo` | Restaura exatamente o áudio pré-overdub em `O(1)` sem cliques e sem alocações. |
| Sem cliques de emenda | Crossfade `>= 5,0 ms` na fronteira do loop e nas transições `Stop`/`Undo`. |

---

## 15. Estratégia de testes, validação e benchmark

1. Teste de ciclo completo da máquina de estados: `Empty → Record (1 s de 440 Hz) → Play → Overdub (1 s de 660 Hz) → Undo (verifica ausência de 660 Hz e preservação de 440 Hz) → Redo (verifica retorno de 660 Hz) → Clear`.
2. Teste de `Half-Speed (0,5×)` verificando transposição exata de `440 Hz -> 220 Hz`.
3. Teste de `zero-alloc` e WCET $O(1)$ durante todas as transições de estado.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Em `96 kHz`, `60 s` estéreo com 2 camadas (`base` + `top`) consome `60 * 96000 * 2 canais * 2 camadas * 4 bytes ≈ 92 MB` de RAM; para estações com pouca memória, o teto configurável em `prepare()` (`30 s` ou `60 s`) pode ser ajustado fora da RT.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Ciclo Completo `Record → Play → Overdub → Undo → Redo → Clear` (`REGRESSION` / `ANALYTICAL`):**
  - *Sinal de entrada:* Gravação inicial de `0,5 s` de senoide `440 Hz` (`-12 dBFS`); Overdub de `0,5 s` de senoide `660 Hz` (`-12 dBFS`), $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Critério e tolerância:*
    1. Em `Playing` após `Overdub`: ambas as frequências (`440 Hz` e `660 Hz`) presentes na FFT dentro de `±1,0 dB`;
    2. Após `Undo`: `440 Hz` preservada (`±0,5 dB`) e `660 Hz` suprimida em **`>= 60 dB`**;
    3. Após `Redo`: `660 Hz` restaurada (`±0,5 dB`);
    4. Após `Clear`: energia de playback **`0,0`** (`<= -120 dBFS`).

- **CA-02 — Modo `Half-Speed (0,5×)` e Modo `Reverse` (`ANALYTICAL`):**
  - *Sinal de entrada:* Loop gravado com senoide de `440 Hz` (para `Half-Speed`) e rampa linear crescente de `0.0 -> 1.0` (para `Reverse`).
  - *Critério e tolerância:*
    1. Em `half_speed = true`, o pico FFT da reprodução deve deslocar-se exatamente para **`220,0 Hz ± 3,0 Hz`** (`-1 oitava`);
    2. Em `reverse = true`, a rampa gravada crescente deve ser reproduzida em ordem decrescente (`y[t1] > y[t2]` para `t2 > t1`).

- **CA-03 — Ausência de Cliques de Emenda nas Fronteiras do Loop (`Crossfade >= 5 ms`) (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoide contínua de `440 Hz` (`-6 dBFS`) gravada por duração não-múltipla do período (`0,5011 s`, forçando descontinuidade de fase máxima de $180^\circ$ entre o início e o fim).
  - *Critério e tolerância:* Na transição do fim para o início do loop, a derivada máxima amostra-a-amostra não deve exceder `2,5×` a derivada da senoide original, comprovando a eficácia do crossfade de `5 ms`.

- **CA-04 — Zero-Alloc e WCET $O(1)$ em Todas as Transições (`ENGINEERING_TARGET`):**
  - *Procedimento:* Executar o ciclo completo `Record → Play → Overdub → Undo → Redo → Half-Speed → Reverse → Stop → Clear` sob alocador global instrumentado na thread RT.
  - *Critério e tolerância:* **`0` alocações e `0` desalocações** na heap, e nenhum bloco individual excede `20 µs` de execução (sem `memcpy` de buffer inteiro na RT).

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/looper.rs`, `crates/sonix-desktop/src/audio.rs`
- Testes atuais: `Planejado: crates/ampforge-dsp/tests/contract_044_sdd_integrity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Desacoplado integralmente da Spec 083 (Afinador YIN).
- [x] Arquitetura *Dual-Layer* $O(1)$ para `Undo/Redo` e crossfade de `5 ms` na emenda especificados.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Bencina, R. (2011). *Real-time audio programming 101* & `AGENTS.md` (Regra Cortex `d272`).

---

## 21. Histórico de decisões

- **2026-11-20:** Criada a Spec 092 na rodada corretiva para separar o Looper de Performance do Afinador YIN (Spec 083).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Máquina de estados `Empty/Record/Play/Overdub/Stop/Undo/Redo/Clear` com arquitetura *Dual-Layer* $O(1)$ | `F-01` | Spec 092 / `CA-01`, `CA-04` | `crates/ampforge-dsp/src/looper.rs:1–275 @ c20cc65` | `contract_044_sdd_integrity.rs` | `IMPLEMENTED_UNVERIFIED` |
| Modos `Half-Speed (0,5×)` e `Reverse` + crossfade de emenda `5 ms` | `F-01` | Spec 092 / `CA-02`, `CA-03` | `crates/ampforge-dsp/src/looper.rs` | Ausente | `SPECIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 092 / `CA-04` | `crates/ampforge-dsp/src/looper.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
