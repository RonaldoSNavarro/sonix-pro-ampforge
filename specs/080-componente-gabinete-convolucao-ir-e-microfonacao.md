# Spec 080: Componente Compartilhado — Gabinete, Convolução IR Zero-Latency e Microfonação

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade)
> **Domínio:** `dsp-shared` (`componente`)
> **Identificador canônico:** `component.cab.ir_convolution_and_mics`
> **Variante / Revisão:** `Convolução Particionada Uniforme Zero-Latency (Partição 0 FIR Direta B=128 + FDL FFT N=256) + Dual-IR Mixer + 7 Microfones + Gestão Lock-Free de Troca de IR`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/cab.rs`, `crates/sonix-desktop/src/audio.rs`, `docs/cab-microfones-fontes.md`
> **Specs relacionadas:** [037](037-seguranca-de-tempo-real-na-cadeia-ao-vivo.md), [044](044-integridade-do-sdd-codigo-morto-e-testes-tautologicos.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [077](077-componente-estagios-valvulados-power-amp-sag-carga-reativa.md)

---

## 1. Objetivo

Especificar o contrato técnico, matemático, de memória em tempo real e de verificação do **Componente de Gabinete, Convolução IR Zero-Latency e Microfonação (`component.cab.ir_convolution_and_mics`)** em [`crates/ampforge-dsp/src/cab.rs`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/cab.rs#L1-L1206), corrigindo três pontos críticos identificados na auditoria:
1. **Definição matemática precisa de "Convolução Particionada Zero-Latency":** A Partição `0` (`h[0..B-1]`) é computada por convolução FIR direta no domínio do tempo (ou bloco particionado sem latência adicional de enquadramento), enquanto as partições de cauda `1..P-1` operam via *Frequency-Domain Delay Line (FDL)* com FFT de tamanho `2B` (`Overlap-Save` / `Overlap-Add`), garantindo **`0` amostras de latência algorítmica**;
2. **Reconciliação da divergência do filtro do microfone `SM57` (`cab.rs:1084` vs `docs/cab-microfones-fontes.md`):** Documentação rigorosa dos 7 microfones clássicos (`SM57`, `R121`, `U87`, `MD421`, `E906`, `C414`, `SM7B`) com base em seus datasheets oficiais (`PRIMARY_COMPONENT`), registrando a divergência verificada no `SM57` (`5600 Hz, +4,0 dB, Q=2,5` no código atual `cab.rs:1084` vs curva publicada pela Shure `4,5–6,0 kHz, +5,0 dB`);
3. **Diagnóstico do vazamento de memória intencional (`std::mem::forget`) em [`crates/ampforge-dsp/src/cab.rs:1199`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/cab.rs#L1199):** O código atual evita desalocar a IR antiga na thread de áudio chamando `std::mem::forget(old_ir)` — o que satisfaz `zero-dealloc` no callback ao custo de **vazar memória permanentemente a cada troca de IR ou preset**! Esta Spec especifica a devolução da IR substituída por fila SPSC *lock-free* para coleta (*garbage reclamation*) na Control Thread.

---

## 2. Escopo

1. **Convolução Particionada Zero-Latency (`B = 128`, `N_FFT = 256`):**
   - Partição `0` (`h[0..127]`): avaliada com `0` amostras de latência algorítmica (`y[n]` depende imediatamente de `h[0] * x[n]`);
   - Partições `1..P-1`: FDL (*Frequency-Domain Delay Line*) com FFT complexa/real pré-planejada em `prepare()`, suportando IRs de até `4096` amostras (`~85 ms` a `48 kHz`) ou `16 384` amostras (`~341 ms`).
2. **Dual-IR Mixer com Alinhamento de Fase e Microfonação Virtual (`Slot A` + `Slot B`):**
   - Carregamento de IRs `.wav` de usuário (reamostragem automática off-RT na Control Thread para `sample_rate` ativo + normalização RMS);
   - Microfones parametrizados pelos datasheets oficiais (`F-01`–`F-07`): `SM57` (dinâmico), `R121` (fita bidirecional), `U87` (condensador diafragma grande), `MD421`, `E906`, `C414`, `SM7B`;
   - Controles de geometria acústica: `Axis` (*Cap Center* `0%` $\to$ *Cone Edge* `100%`, deslocando o LPF de diretividade do alto-falante), `Distance` (`Proximity Effect` bass boost próximo ao cone + atraso de propagação acústica $\Delta t = d / 343\text{ m/s}$ para alinhamento de fase entre `Mic A` e `Mic B`), `Phase Invert` (`0° / 180°`), `Low Cut` (`HPF`), `High Cut` (`LPF`) e `Blend`.
3. **Troca de IR *Glitch-Free* com Crossfade de `15 ms` e Reciclagem Lock-Free Sem Vazamento (`Zero-Leak`):**
   - Trocar IR ou preset aplica crossfade de potência constante (`15 ms`) entre o convolver antigo e o novo (sem silenciamento nem clique), e devolve o buffer da IR antiga pela fila `rt_to_ctrl_trash_queue` para desalocação segura fora da thread de áudio (eliminando `std::mem::forget` de `cab.rs:1199`).

---

## 3. Fora de escopo

- Convolução de salas longas de Reverb (`> 1,5 s`, que exigem particionamento não-uniforme Gardner/Wefers — ver Spec 082 para algoritmos de Reverb).
- Emulação de carga reativa do amplificador sobre a bobina do alto-falante (pertence ao estágio de potência na **[Spec 077](077-componente-estagios-valvulados-power-amp-sag-carga-reativa.md)**).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `componente` (`dsp-shared` / `cabinet-ir-convolver`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (baseado no algoritmo FDL de Soo & Pang / Gardner e nos datasheets oficiais Shure, Royer, Neumann, Sennheiser, AKG) |
| **Modelo de referência** | Zero-Latency Partitioned IR Convolver & Dual-Mic Cabinet Engine (`component.cab.ir_convolution_and_mics`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 080) |
| **Identificador no sistema** | `component.cab.ir_convolution_and_mics` (`crates/ampforge-dsp/src/cab.rs`) |
| **Papel na cadeia de sinal** | Bloco `CAB` pós-amplificador (e segundo gabinete independente em modo Dual-Amp / AMP 2, Spec 052). |

---

## 5. Princípio físico e funcional

1. **Convolução Linear Exata com Latência Zero:**
   - A resposta ao impulso $h[k]$ ($k = 0..L-1$) de um gabinete de guitarra `4×12` / `2×12` / `1×12` microfonado captura simultaneamente a resposta modal do cone de papel, a difração da madeira do gabinete, as reflexões internas e a resposta do microfone real.
   - Dividindo $h[k]$ em $P = \lceil L / B \rceil$ partições de tamanho $B = 128$:
     $$y[n] = \underbrace{\sum_{k=0}^{B-1} h_0[k]\,x[n-k]}_{\text{Partição 0: 0 amostras de latência}} + \underbrace{\sum_{p=1}^{P-1} (h_p * x_{\text{hist},p})[n]}_{\text{Partições 1..P-1: FDL FFT (N=256)}}$$
   - O resultado é matematicamente idêntico (até a precisão numérica de ponto flutuante `<= -110 dB`) à convolução direta de $L$ taps, porém com custo $O(\log B + P)$ em vez de $O(L)$ e **latência algorítmica estritamente igual a `0` amostras**!
2. **Filtros de Microfonação Calibrados por Datasheets Oficiais (`docs/cab-microfones-fontes.md`):**
   - Cada microfone modela a diferença espectral documentada nos gráficos de resposta em frequência dos fabricantes (ex.: pico de presença de `+4..5 dB` entre `4,5–6 kHz` e rolagem grave abaixo de `100 Hz` do **Shure SM57**; rolagem suave acima de `8–10 kHz` e efeito de proximidade acentuado de figura-8 do **Royer R-121**; clareza estendida até `15 kHz` do **Neumann U87**).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | William G. Gardner — *Efficient Convolution without Input-Output Delay* (JAES, Vol. 43, No. 3, 1995) | Literatura acadêmica fundadora da convolução particionada zero-latency | `ACADEMIC` | Partição 0 sem atraso + partições de cauda em FDL FFT com igualdade matemática exata à convolução direta. |
| `F-02` | Shure Incorporated — *SM57 & SM7B Cardioid Dynamic Microphone User Guides / Frequency Response Curves* | Referência consolidada em [`docs/cab-microfones-fontes.md`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/docs/cab-microfones-fontes.md) | `PRIMARY_COMPONENT` | Curvas de resposta em frequência (`SM57` pico `4,5–6 kHz`, rolagem `< 100 Hz`; `SM7B` presença suave). |
| `F-03` | Royer Labs — *R-121 Studio Ribbon Microphone Datasheet* | Referência consolidada em [`docs/cab-microfones-fontes.md`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/docs/cab-microfones-fontes.md) | `PRIMARY_COMPONENT` | Padrão figura-8, efeito de proximidade acentuado e rolagem suave de agudos (`> 8 kHz`). |
| `F-04` | Neumann (`U87 Ai`), Sennheiser (`MD 421-II`, `e 906`), AKG (`C414 XLS`) — *Official Technical Datasheets* | Referência consolidada em [`docs/cab-microfones-fontes.md`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/docs/cab-microfones-fontes.md) | `PRIMARY_COMPONENT` | Curvas de resposta em frequência e diretividade dos demais 4 microfones do banco. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Partição 0 direta (`B=128`) + cauda FDL FFT (`N=256`) resulta em latência `0` amostras e erro de reconstrução `< -100 dB` | `F-01` | JAES 1995 | §2–3 | `ACADEMIC` | `CONFIRMED` |
| Respostas nominais dos 7 microfones (`SM57`, `R121`, `U87`, `MD421`, `E906`, `C414`, `SM7B`) | `F-02`, `F-03`, `F-04` | Datasheets oficiais | [`docs/cab-microfones-fontes.md`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/docs/cab-microfones-fontes.md) | `PRIMARY_COMPONENT` | `CONFIRMED` |
| Divergência de calibração do `SM57` entre código atual (`5600 Hz, +4,0 dB, Q=2,5` em `cab.rs:1084`) e referência anterior (`4,8 kHz, +5,0 dB, Q=1,4`) | `F-02` & auditoria `cab.rs:1084` | SM57 Datasheet | `docs/cab-microfones-fontes.md` | `PRIMARY_COMPONENT` | `CONFIRMED` (reconciliado na doc e especificado para calibração consistente) |

---

## 7. Topologia e arquitetura real do sistema

```
[Amp Out] ──┬──► [Slot A: Zero-Latency Partitioned IR Convolver A] ──► [Mic A Filter + Axis + Distance/Delay + Phase] ──┐
            │                                                                                                            ├──► [Blend + HPF/LPF] ──► [Out]
            └──► [Slot B: Zero-Latency Partitioned IR Convolver B] ──► [Mic B Filter + Axis + Distance/Delay + Phase] ──┘
                                       ▲
                    [Control Thread: Load WAV → Resample → FFT Partitions → SPSC Swap]
                    [RT Thread: 15ms Crossfade → Return Old IR to Trash Queue (ZERO LEAK!)]
```

### 7.1. Auditoria do código atual e classificação dos achados

O código atual em [`crates/ampforge-dsp/src/cab.rs`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/cab.rs#L1-L1206) apresenta os seguintes achados auditados:

1. **`FACTUAL_ERROR` / Risco de Vazamento de Memória (`cab.rs:1199` — `std::mem::forget` na Troca de IR):**
   - Na linha `1199` de `crates/ampforge-dsp/src/cab.rs`, ao trocar de IR no callback RT, o código chama `std::mem::forget` sobre a estrutura de IR anterior para evitar chamar `dealloc` na thread de áudio!
   - Embora passe no teste de alocador zero-alloc em uma sessão curta, em uma sessão longa de estúdio onde o músico alterna centenas de presets ou audita IRs de usuário, `std::mem::forget` **vaza memória RAM indefinidamente** até esgotamento do processo!
   - **Correção especificada:** A IR antiga (após terminar o crossfade de `15 ms`) deve ser enviada por uma fila SPSC lock-free `trash_tx.push(old_ir)` (Spec 053 §11 / Spec 084) para ser desalocada (`drop`) exclusivamente pela **Control Thread** — garantindo simultaneamente **zero dealloc na RT** e **zero vazamento de memória no processo**.
2. **`CODE_REFERENCE_STALE` / Divergência `SM57` (`cab.rs:1084`):**
   - `cab.rs:1084` aplica pico em `5600 Hz, +4,0 dB, Q = 2,5` (já reconciliado em `docs/cab-microfones-fontes.md` nesta rodada corretiva).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica (Spec 014) | Faixa Aceita | Unidade | Valor Padrão | Papel |
|---|---|---|---|---|---|
| **CAB ENABLED** | `cab.enabled` | `false / true` | Booleano | `true` | Ativa/desativa simulação de gabinete. |
| **CAB MODEL / IR A** | `cab.model` / `cab.ir_path` | Presets internos / WAV | String | `"4x12_v30"` | Seleciona o gabinete / arquivo WAV da IR principal (`Slot A`). |
| **MIC A / MIC B** | `cab.mic` / `cab.mic_b` | `sm57`, `r121`, `u87`, `md421`, `e906`, `c414`, `sm7b` | Enum | `"sm57"` / `"r121"` | Seleciona o voicing de microfone calibrado por datasheet. |
| **AXIS (CAP ↔ EDGE)** | `cab.axis` / `cab.axis_b` | `0.0 .. 1.0` | `%` | `0.20` | `0.0` = centro do domo (brilhante); `1.0` = borda do cone (escuro). |
| **DISTANCE** | `cab.distance` / `cab.distance_b` | `0.0 .. 1.0` | `cm` (virtual `2..40 cm`) | `0.15` | Controla efeito de proximidade (`bass boost` em curta distância) e atraso acústico (`d / 343 m/s`). |
| **PHASE INVERT** | `cab.phase_b` | `false / true` | Booleano | `false` | Inverte polaridade ($180^\circ$) do microfone B. |
| **BLEND (A ↔ B)** | `cab.blend` | `0.0 .. 1.0` | `%` | `0.0` (100% A) | Mistura entre `Slot A` e `Slot B`. |
| **LOW CUT / HIGH CUT** | `cab.low_cut_hz` / `cab.high_cut_hz` | `20..500 Hz` / `2000..20000 Hz` | `Hz` | `75 Hz` / `9500 Hz` | Filtros pós-microfone de limpeza de mixagem (12 dB/oitava). |

---

## 9. Modelo matemático e algoritmo DSP proposto

1. **Convolução Particionada Zero-Latency (`B = 128`, `N = 256`):**
   - Partição `0`: soma direta FIR no domínio do tempo (ou bloco paralelo zero-delay) dos primeiros `128` taps (`h[0..127]`);
   - Partições `1..P-1`: multiplicação acumulada no domínio da frequência (`FDL`) com `IFFT(256)` realizada a cada `128` amostras.
2. **Troca Lock-Free Sem Vazamento (*Zero-Alloc, Zero-Leak*):**
   - `Control Thread`: carrega `.wav`, reamostra para `fs`, calcula `P` partições FFT em `PreparedIr`, empurra `Box<PreparedIr>` na fila SPSC `ir_swap_tx`;
   - `RT Thread`: ao receber nova `PreparedIr`, inicia rampa linear/coseno de `15 ms` (`720` amostras a `48 kHz`) entre `active_ir` e `next_ir`; ao fim dos `15 ms`, promove `next_ir` para `active_ir` e empurra `old_ir` na fila SPSC `ir_trash_tx` (nunca chama `drop` nem `std::mem::forget` na RT!);
   - `Control Thread`: drena `ir_trash_rx` periodicamente e executa `drop(old_ir)` com segurança.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `1×` (convolução linear é um operador LTI e nunca gera novos harmônicos!).
- **Reamostragem de IRs WAV:** Executada exclusivamente na Control Thread via reamostrador Sinc polifásico de alta qualidade ao carregar IRs com taxa de amostragem diferente da sessão (`44,1 / 48 / 96 kHz`).

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações e **zero desalocações** na thread RT; **zero vazamentos de memória** após `1000` trocas de IR consecutivas.
- WCET para Dual-IR de `2048` taps a `48 kHz` (`B = 128`): `<= 25 µs`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Estágio acústico final antes dos efeitos de ambiência pós-gabinete (`Delay` / `Reverb`).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo:** `crates/ampforge-dsp/src/cab.rs` (`CabProcessor`, `PartitionedConvolver`, `PreparedIr`, `IrReclamationQueue`).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Regra Verificável |
|---|---|
| Latência algorítmica da convolução | Estritamente **`0` amostras** (`y[0] = h[0] * x[0]`). |
| Exatidão matemática vs convolução direta | Erro máximo absoluto $|y_{\text{part}}[n] - y_{\text{dir}}[n]| \le 10^{-5}$ (`<= -100 dB`) em qualquer IR até `4096` taps. |
| Zero vazamento na troca de IR | Número de `PreparedIr` vivos na heap após `500` trocas sucessivas permanece constante (`<= 2`). |

---

## 15. Estratégia de testes, validação e benchmark

1. **Teste de Exatidão Matemática e Latência Zero:** compara a saída do `PartitionedConvolver` contra a convolução direta referência $y[n] = \sum h[k] x[n-k]$ em impulso unitário e ruído branco.
2. **Teste de Troca Sem Cliques e Sem Vazamento de Memória (`500` trocas consecutivas):** verifica continuidade de derivada durante o crossfade de `15 ms`, `0` alocações na thread RT, e contador de `Drop` de `PreparedIr` igual a `500` na Control Thread!

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Caso a fila `ir_trash_tx` esteja momentaneamente cheia (cenário extremo de automação frenética), a RT thread segura o ponteiro em um slot fixo de espera e tenta reenviar no próximo bloco em vez de chamar `drop` ou `std::mem::forget`.

---

## 17. Critérios de aceite (CA-01 a CA-05)

- **CA-01 — Latência Estritamente Zero e Exatidão Matemática vs Convolução Direta (`ANALYTICAL` / `REGRESSION`):**
  - *Sinal de entrada:* Impulso unitário (`x[0] = 1.0`) e sinal de ruído branco de `8192` amostras a `-12 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras (`B = 128, N_FFT = 256`).
  - *Configuração e parâmetros:* IR sintética arbitrária de `2048` taps com $h[0] = 0,75$.
  - *Método de medição:* Inspeção da primeira amostra $y[0]$ e erro RMS / pico contra a convolução direta $y_{\text{ref}}[n] = \sum_{k=0}^{2047} h[k]\,x[n-k]$.
  - *Critério e tolerância:*
    1. $y[0] = 0,7500 \pm 10^{-5}$ (comprovando latência algorítmica de **`0` amostras**).
    2. $\max_n |y_{\text{part}}[n] - y_{\text{ref}}[n]| \le 10^{-5}$ (`<= -100 dB`) em todas as `8192` amostras.
  - *Origem da tolerância:* Propriedade exata da convolução particionada de Gardner (`F-01`) em ponto flutuante IEEE-754.

- **CA-02 — Troca de IR *Glitch-Free* (Crossfade `15 ms`) e Zero Vazamento de Memória (`Zero-Leak`, Substituição de `std::mem::forget`) (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* Senoide contínua de `440 Hz` a `-12 dBFS` durante `500` trocas de IR consecutivas entre duas IRs distintas.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Método de medição:* Alocador global na thread RT + contador atômico `DROPPED_IR_COUNT` no `Drop` de `PreparedIr` na Control Thread + inspeção de continuidade da forma de onda durante o crossfade.
  - *Critério e tolerância:*
    1. `0` alocações e `0` desalocações na thread RT durante todas as `500` trocas.
    2. Após drenar `ir_trash_rx` na Control Thread, `DROPPED_IR_COUNT == 500` (**zero vazamento de memória**, eliminando `std::mem::forget` de `cab.rs:1199`).
    3. Nenhum silenciamento abrupto (energia RMS não cai para `0.0` durante a troca).

- **CA-03 — Calibração Espectral dos Microfones (`SM57` vs `R121` vs `U87`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Varredura senoidal em `100 Hz`, `1000 Hz`, `5000 Hz` e `12 000 Hz` a `-18 dBFS`.
  - *Taxa de amostragem e bloco:* $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração e parâmetros:* Comparação dos filtros de microfone `sm57`, `r121` e `u87` com `axis = 0.0`, `distance = 0.15`.
  - *Método de medição:* Ganho em `5000 Hz` e `12 000 Hz` relativo a `1000 Hz`.
  - *Critério e tolerância:*
    1. `sm57` deve apresentar reforço de presença em `5000 Hz` de **`+2,5 dB` a `+5,5 dB`** relativo a `1000 Hz` e queda em `100 Hz` relativa a `1000 Hz`;
    2. `r121` (fita) deve ser **pelo menos `4,0 dB` mais escuro** em `12 000 Hz` do que `u87` (condensador).
  - *Origem da tolerância:* Datasheets oficiais Shure SM57, Royer R-121 e Neumann U87 (`F-02`, `F-03`, `F-04`).

- **CA-04 — Ação de `Axis` (`Cap` vs `Edge`), `Distance` (Proximidade + Atraso) e `Phase Invert` (`DOCUMENTED`):**
  - *Sinal de entrada:* Ruído branco a `-18 dBFS`.
  - *Critério e tolerância:* Mover `axis` de `0.0` (`Cap`) para `1.0` (`Edge`) deve reduzir o centróide espectral em **pelo menos `400 Hz`**; ativar `phase_b = true` com `IR A == IR B`, `Mic A == Mic B`, `blend = 0.5` deve cancelar o sinal em **`>= 60 dB`**.

- **CA-05 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Sinal de entrada:* `10 000` blocos de `128` amostras a `48 kHz` com automação de `axis`, `distance`, `blend`, `low_cut`, `high_cut`.
  - *Critério e tolerância:* `0` alocações na heap e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/cab.rs`, `docs/cab-microfones-fontes.md`
- Testes atuais: `crates/ampforge-dsp/tests/contract_037_ir_realtime.rs`, `crates/ampforge-dsp/tests/contract_044_sdd_integrity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Convolução particionada zero-latency (`B = 128` direto + `N = 256` FFT FDL) definida matematicamente.
- [x] Vazamento de memória por `std::mem::forget` em `cab.rs:1199` diagnosticado e solução de reciclagem SPSC *lock-free* especificada em `CA-02`.
- [x] Divergência do filtro `SM57` (`cab.rs:1084`) reconciliada com `docs/cab-microfones-fontes.md`.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Gardner, W. G. (1995). *Efficient Convolution without Input-Output Delay*. JAES, 43(3), 127–136.
- `[F-02]`–`[F-04]` Datasheets oficiais Shure (`SM57`, `SM7B`), Royer (`R-121`), Neumann (`U87 Ai`), Sennheiser (`MD 421-II`, `e 906`), AKG (`C414 XLS`), consolidados em [`docs/cab-microfones-fontes.md`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/docs/cab-microfones-fontes.md).

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, diagnosticou-se o vazamento de memória permanente causado por `std::mem::forget` em `cab.rs:1199` e especificou-se o mecanismo lock-free `ir_trash_tx/rx` para manter simultaneamente zero-dealloc na RT e zero-leak no processo.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Convolução particionada zero-latency (`B=128` direto + `N=256` FDL) com erro `<= -100 dB` | `F-01` | Spec 080 / `CA-01` | `crates/ampforge-dsp/src/cab.rs:1–700` | `contract_037_ir_realtime.rs` | `TESTED` |
| Troca de IR com crossfade `15 ms` + fila de reciclagem lock-free **sem `std::mem::forget`** | Spec 053, Spec 084 | Spec 080 / `CA-02` | `crates/ampforge-dsp/src/cab.rs:1199` (RISCO: usa `std::mem::forget`, vazando memória a cada troca de IR) | `contract_044_sdd_integrity.rs` (verifica zero-alloc, não verifica leak) | `SPECIFIED` |
| Banco de 7 microfones calibrados por datasheets (`SM57`, `R121`, `U87`, `MD421`, `E906`, `C414`, `SM7B`) | `F-02`–`F-04` | Spec 080 / `CA-03` | `crates/ampforge-dsp/src/cab.rs:1080–1150` | `contract_037_ir_realtime.rs` | `IMPLEMENTED_UNVERIFIED` |
| Controles `Axis`, `Distance`, `Phase Invert`, `Blend`, `Low/High Cut` | Spec 014 | Spec 080 / `CA-04` | `crates/ampforge-dsp/src/cab.rs:950–1180` | `contract_040_pedals_parity.rs` | `TESTED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 080 / `CA-05` | `crates/ampforge-dsp/src/cab.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
