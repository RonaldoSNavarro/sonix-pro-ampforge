---
spec: 053
titulo: Arquitetura DSP modular, contratos de processadores, ciclo de vida e independência de módulos
status: rascunho
tipo: arquitetura-dsp
atualizado: 2026-11-20
dependencias: [001, 002, 014, 025, 033, 037, 044, 045, 049]
---
# 053 — Arquitetura DSP Modular e Contratos Comuns de Processadores

## 1. Identificação e objetivo
- **Identificador estável (`module_id`):** `ampforge.core.architecture.v1`
- **Revisão (`model_revision`):** `v1.1-rt-control-split`
- **Categoria:** Arquitetura Central DSP (Spec de fundação para Specs 054–092)
- **Natureza do módulo:** Infraestrutura arquitetural compartilhada
- **Objetivo:** Definir os contratos técnicos comuns do motor DSP modular do **SONIX PRO // AMPFORGE™** — ciclo de vida, separação estrita entre preparação na thread de controle (off-RT) e processamento na thread de áudio (RT), formatos de áudio, descritores de parâmetros tipados, registro de modelos, gerenciamento de estado, latência (PDC) e independência entre módulos — para que cada amplificador, pedal, módulo algorítmico e componente compartilhado seja implementado, testado e evoluído sem acoplamento no engine central.

## 2. Escopo
- Contratos de trait e structs centrais do engine em `crates/ampforge-dsp`: `AudioSpec`, `ChannelLayout`, `ModuleId`, `ModelRevision`, `ParamId`, `ParamDescriptor`, `ParamTaper`, `ParamValue`, `AudioBlockRef`/`AudioBlockMut`, `ProcessContext`, `AudioProcessor`, `ModelDescriptor` e `ProcessorFactory`/`ProcessorRegistry`.
- Separação explícita de threads no ciclo de vida:
  - **Thread de controle (Off-RT):** criação de instâncias, `prepare(spec)` (pode alocar buffers para `max_sample_rate` / `max_block_size`), carregamento/resampling de IR, compilação topológica de DAG (`CompiledGraph`), instanciação de novos modelos para substituição estrutural e drenagem do `RetiredReaper`.
  - **Thread de áudio (Hard Real-Time — RT):** `process(ctx, input, output)` (0 alloc / 0 dealloc), `apply_param_rt(id, value)` (atualização escalar $O(1)$ 0 alloc), `reset()` ($O(1)$ RT-safe sem `memset` de megabytes), `latency_samples()`, crossfade entre slots ativo/sombra pré-alocados e publicação de recursos aposentados na fila SPSC do `RetiredReaper`.
- Política de headroom de barramento interno (`f32` planar com headroom acima de `0 dBFS` entre estágios) vs. limitação estrita `[-1.0, 1.0]` (`<= 0 dBFS`) exclusivamente no estágio final de saída para o DAC/host (Spec 016).
- Separação entre **estado DSP efêmero** (filtros, atrasos, envelopes) e **estado persistido de preset** (`model_id`, `model_revision`, `schema_version`, `params`).

## 3. Fora de escopo
- Reescrita imediata dos amplificadores, pedais ou `RealtimeDspChain` nesta etapa documental (a implementação ocorrerá somente após revisão e aprovação das Specs 053–092).
- Alterações de protocolo de driver de áudio Windows (WASAPI/ASIO — Specs 018, 036 e 049).
- Alterações visuais da UI Tauri v2 (Specs 033, 048 e 052).

## 4. Equipamento ou componente de referência
- **Fabricante / Origem:** Arquitetura interna SONIX PRO // AMPFORGE™.
- **Modelo exato:** Engine DSP Modular v1 (`ampforge-dsp` + camada de controle).
- **Papel na cadeia de sinal:** Infraestrutura de execução do grafo 4×12, registro de módulos, compensação de latência paralela (PDC) e contratos de tempo real que hospeda todos os blocos de processamento (Specs 054–092).

## 5. Revisão e variantes
- **Revisão de referência:** `v1.1-rt-control-split`.
- **Variantes de hospedagem suportadas pelo mesmo núcleo:**
  1. Standalone Desktop Windows (`apps/sonix-app` Tauri v2 + `crates/sonix-desktop` via `cpal`).
  2. Plugins DAW VST3 e CLAP (`crates/sonix-plugin` via `nih_plug`).
  3. Harness determinístico offline (`cargo test` / benchmarks Criterion).

## 6. Fontes técnicas e evidência por informação

### 6.1 Catálogo de fontes verificadas
| ID | Fonte | Fabricante / Modelo / Revisão | Página / Folha / Seção | URL / Caminho direto | Consulta | Classificação | Limitações da fonte |
|---|---|---|---|---|---|---|---|
| F1 | Código-fonte `ampforge-dsp` | Workspace SONIX PRO (HEAD `d891389`) | `config.rs`, `engine.rs`, `dag.rs` (529 l.), `queue.rs` (275 l.), `amp.rs` (1738 l.) | `crates/ampforge-dsp/src/` | 2026-11-20 | `INTERNAL_REFERENCE` | Descreve apenas a implementação atual e seus acoplamentos |
| F2 | Código-fonte `sonix-desktop` | Workspace SONIX PRO (HEAD `d891389`) | `audio.rs` (`RealtimeDspChain`, l. 2558–5190), `api.rs` (5066 l.) | `crates/sonix-desktop/src/` | 2026-11-20 | `INTERNAL_REFERENCE` | Orquestração monolítica atual acoplada ao crate desktop |
| F3 | Código-fonte `sonix-app` | Tauri `2.12.5` (HEAD `d891389`) | `commands.rs`, `streams.rs`, `controls.registry.ts` (2292 l.) | `apps/sonix-app/` | 2026-11-20 | `INTERNAL_REFERENCE` | Fronteira IPC Tauri v2 e registro único de controles |
| F4 | Specs 001, 002, 014, 016, 025, 033, 037, 044, 049 | Contratos internos v1.0 | Critérios de aceite e testes existentes | `specs/` | 2026-11-20 | `INTERNAL_REFERENCE` | Comprova requisitos internos já aprovados |
| F5 | Ross Bencina — *Real-time audio programming 101: time waits for nothing* | Artigo técnico de engenharia de áudio | Seções: Memory Allocation, Locks, Lock-Free Queues | `https://web.archive.org/web/20240101000000/http://www.rossbencina.com/code/real-time-audio-programming-101-time-waits-for-nothing` | 2026-11-20 | `SECONDARY_TECHNICAL` | Referência conceitual de restrições hard real-time (domínio direto recusa TLS 443) |
| F6 | Tauri v2 Official Documentation | Tauri `2.x` (`2.12.5`) | Architecture, IPC, Channels, Capabilities | `https://v2.tauri.app/` | 2026-11-20 | `PRIMARY_OFFICIAL` | Documenta apenas a camada desktop/IPC, não o motor DSP |

### 6.2 Evidência por afirmação técnica
| Informação / Requisito Arquitetural | Fonte (ID) | Revisão | Página / Folha / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Proibição de alocação, desalocação e locks na thread de áudio | F4, F5 | Spec 037 / Bencina | `contract_037_realtime_safety.rs` | `INTERNAL_REFERENCE` / `SECONDARY_TECHNICAL` | Confirmado |
| Nível nominal calibrado `-12 dBFS` (`0.2512` pico) e limiter final `[-1.0, 1.0]` | F2, F4 | Spec 016 / Spec 019 | `calibration.rs:81`, `audio.rs:4680–4689` | `INTERNAL_REFERENCE` | Confirmado |
| Acoplamento atual de `RealtimeDspChain` (2633 linhas dentro de `audio.rs`) | F2 | HEAD `d891389` | `crates/sonix-desktop/src/audio.rs:2558–5190 @ c20cc65` | `INTERNAL_REFERENCE` | Confirmado |
| Duplicação de `BiquadFilter` em 6 módulos (`amp`, `cab`, `drive`, `dynamics`, `delay`, `precision`) | F1 | HEAD `d891389` | `amp.rs:11`, `cab.rs:7`, `fx/drive.rs:58`, `fx/dynamics.rs:19`, `fx/delay.rs:15`, `precision.rs:8` | `INTERNAL_REFERENCE` | Confirmado |
| Fronteira IPC Tauri v2 e `Channel` a 30 Hz | F3, F6 | Tauri `2.12.5` | `streams.rs`, `https://v2.tauri.app/` | `PRIMARY_OFFICIAL` / `INTERNAL_REFERENCE` | Confirmado |

## 7. Fundamentação elétrica e de nível de sinal
- **Referência nominal de sinal e headroom interno:** o barramento `f32` planar entre módulos opera com nível nominal calibrado de `-12 dBFS` (`≈ 0.2512` pico, Spec 019 `calibration.rs:81`). Módulos intermediários (ex.: pedais de boost, Klon Centaur com rails `+18 V / -9 V`, ou pré-amplificadores antes do power amp) **podem exceder `±1.0` no barramento interno** (ex.: até `±4.0` linear) sem ceifamento artificial entre nós; toda amostra não-finita (`NaN`/`Inf`) é sanitizada na fronteira de cada nó, enquanto o clamp estrito `[-1.0, 1.0]` (`<= 0 dBFS`) ocorre exclusivamente no **limitador de saída final da cadeia** (`audio.rs:4680–4689`, Spec 016).
- **Acoplamento físico desacoplado via `ProcessContext`:** interações elétricas entre blocos adjacentes — como a impedância de entrada `input_impedance_ohms` do 1º pedal ativo carregando o modelo RLC de captador (Spec 076 ↔ Spec 063) e a curva de impedância / `speaker_resonance_hz` do gabinete interagindo com a malha NFB do estágio de potência (Spec 080 ↔ Spec 077) — são transportadas por campos tipados no `ProcessContext`, sem importação cruzada entre structs de equipamentos.

## 8. Comportamento esperado
1. **Separação estrita entre Thread de Controle (Off-RT) e Thread de Áudio (RT):**
   - **Operações estruturais (Off-RT):** instanciar um novo processador (`ProcessorRegistry::create`), chamar `prepare(spec)`, compilar uma nova topologia DAG (`CompiledGraph`) ou carregar uma IR WAV ocorrem na **thread de controle**, onde alocações de preparação são permitidas. O objeto pronto é transferido por `SpscOwnedQueue` para a thread RT.
   - **Operações em tempo real (RT):** dentro do callback de áudio, `process()`, `apply_param_rt()` (atualização de parâmetros escalares/enums existentes), crossfade de 20 ms entre instância ativa e instância sombra pré-alocada, e envio do recurso antigo para a fila SPSC do `RetiredReaper` executam com **`0` alocações e `0` desalocações na heap**.
2. **Processadores autônomos por contrato:** cada amplificador (Specs 054–058), pedal/rack/módulo (Specs 059–075, 085–091) e componente compartilhado (Specs 076–084, 092) implementa `AudioProcessor` + `ModelDescriptor`.
3. **Formatos de áudio e canais explícitos:**
   - Taxas suportadas: `44_100`, `48_000`, `88_200`, `96_000` Hz.
   - Qualquer tamanho de bloco recebido no callback (incluindo `441`, `480`, `882`, `960` frames do WASAPI compartilhado) é processado subdividindo em fatias `<= prepared_max_block` (`audio.rs:4534–4540`).
   - `ChannelLayout` explícito (`Mono`, `Stereo`, `MonoToStereo`).
4. **Compensação automática de latência paralela (PDC):** cada módulo declara `latency_samples() -> usize`. Quando dois ramos paralelos (Linha A e Linha B) convergem em um `MixerNode`, o compilador do grafo insere atraso compensador `|lat_A - lat_B|` pré-alocado no ramo mais rápido e publica `total_algorithmic_latency_samples`.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Origem (`Físico` vs `Engine`) | Efeito no engine | Implementação DSP |
|---|---|---|---|---|---|---|---|---|---|
| Oversampling Global | `engine.oversampling` | enum | fator | `1x, 2x, 4x, 8x` (`2x, 4x, 8x` UI) | `2x` | discreto | `Engine` | Controla fator de sobreamostragem dos estágios não lineares | Configura estágios pré-alocados (Spec 079) |
| Nível Master Saída | `master.level_db` | number | dB | `-60.0..=12.0` | `0.0` | linear em dB | `Engine` | Trim mestre pós-cadeia (`<= -59.9 dB` muta) | Ganho suavizado antes do limiter `<= 0 dBFS` |
| Bypass de Bloco | `<block>.enabled` | boolean | bool | `false / true` | por bloco | discreto | `Físico / Engine` | True bypass / bypass com spillover | Crossfade suave `<= 10 ms` sem cliques |
| Seleção de Modelo | `<block>.model` | enum | id estável | catálogo do bloco | por bloco | discreto | `Engine` | Seleciona módulo + revisão no `ProcessorRegistry` | Troca por sombra pré-alocada + crossfade `20 ms` |

## 10. Contratos de integração
- **Engine ↔ Módulos (`ampforge-dsp`):**
  - `pub trait AudioProcessor: Send` com `descriptor(&self) -> &ModelDescriptor`, `prepare(&mut self, spec: AudioSpec) -> Result<(), DspError>` (off-RT), `reset(&mut self)` (RT-safe $O(1)$), `apply_param_rt(&mut self, id: ParamId, val: ParamValue) -> Result<(), ParamError>` (RT zero-alloc), `process(&mut self, ctx: &ProcessContext, input: &AudioBlockRef<'_>, output: &mut AudioBlockMut<'_>)` (RT zero-alloc), `latency_samples(&self) -> usize`, `drain_retired(&mut self, sink: &mut dyn RetiredSink)`.
- **Engine ↔ Control Plane (`sonix-desktop::api`):** `api::dispatch` valida faixas, tipos, enums e disponibilidade por modelo de forma atômica contra o `ModelDescriptor` registrado.
- **Engine ↔ UI (`controls.registry.ts`) e Plugins (`sonix-plugin`):** ambos consomem os mesmos identificadores e contratos sem duplicar regras de domínio.

## 11. Arquitetura de implementação
- **Código atual mapeado (HEAD `d891389`):**
  - `crates/ampforge-dsp/src/{config.rs, errors.rs, queue.rs, smoother.rs, engine.rs, dag.rs}`
  - `crates/sonix-desktop/src/audio.rs` (`RealtimeDspChain`, linhas `2558–5190`)
  - `crates/sonix-desktop/src/api.rs` (linhas `1–5066`)
  - `crates/sonix-plugin/src/lib.rs`
- **Problemas e dívidas encontrados no código atual:**
  1. `CODE_REFERENCE_STALE` / `MODEL_COUPLING`: `RealtimeDspChain` (`audio.rs:2558–5190`, 2633 linhas) acopla toda a cadeia DSP ao crate `sonix-desktop`, obrigando `sonix-plugin` a depender de `sonix-desktop`.
  2. `MODEL_COUPLING`: `WhiteBoxAmpProcessor` (`amp.rs`, 1738 linhas), `DriveProcessor` (`fx/drive.rs`, 689 linhas), `ModulationProcessor` (`fx/modulation.rs`, 297 linhas), `DelayProcessor` (`fx/delay.rs`, 559 linhas) e `ReverbProcessor` (`fx/reverb.rs`, 337 linhas) concentram múltiplos equipamentos distintos em structs monolíticas.
  3. `DUPLICATED_REQUIREMENT`: 6 implementações duplicadas de `BiquadFilter` espalhadas pelo workspace.

## 12. Estratégia DSP
- Precisão mista obrigatória: barramento planar em `f32` com headroom interno preservado; filtros recursivos críticos (`< 200 Hz`, tone stacks, SVF, solvers não lineares e DC blocker) em `f64` (TDF-II / TPT).
- Sanitização de `NaN`/`Inf` na saída de cada bloco e corte anti-denormal (`< 1e-24`) em malhas recursivas.

## 13. Requisitos de tempo real
- Na thread RT (`process()`, `apply_param_rt()`, crossfade entre slots pré-alocados e `drain_retired()`): **`alloc_count == 0` e `dealloc_count == 0`** medidos por `CountingAllocator` (`contract_037_realtime_safety.rs`).
- Operações que exigem alocação (`prepare()`, instanciação de novo `Box<dyn AudioProcessor>`, parsing WAV/IR, compilação de `CompiledGraph`) executam **exclusivamente na thread de controle (off-RT)** e entregam o recurso pronto via `SpscOwnedQueue`.

## 14. Gestão de estado
- `ModulePresetState { module_id, model_revision, schema_version, params }` é puramente declarativo e serializável.
- Estado DSP interno pertence exclusivamente à instância do processador e nunca é serializado em preset.

## 15. Latência
- Cada nó reporta `latency_samples()`; `MixerNode` aplica PDC automático em ramos paralelos e publica `total_algorithmic_latency_samples` para telemetria e hosts VST3/CLAP.

## 16. Consumo de CPU
- Meta de engenharia (`ENGINEERING_TARGET`): overhead de infraestrutura do grafo modular (dispatch de nós, PDC, suavização de bypass, sanitização) `<= 3,0%` do orçamento de `128` frames @ `48 kHz` (`2,667 ms`) em build `--release`.

## 17. Testes
- Condições padronizadas: blocos de `32, 64, 128, 441, 480, 1024` frames nas taxas `44100, 48000, 88200, 96000 Hz`; medição com `CountingAllocator` na thread RT; impulso unitário para PDC; injeção de `NaN`, `±Inf` e `±1e6`.

## 18. Critérios de aceite
- **CA-1 (`ENGINEERING_TARGET` — Contratos e Descritores de Processador):** Todos os módulos registrados no `ProcessorRegistry` implementam `AudioProcessor` e expõem `ModelDescriptor` contendo `module_id` não vazio, `model_revision`, `ChannelLayout`, `latency_samples()` e slice estático de `ParamDescriptor` cobrindo 100% dos parâmetros aceitos pelo módulo.
- **CA-2 (`ENGINEERING_TARGET` — Independência do Executor do Grafo):** O executor do grafo em `ampforge-dsp` processa qualquer cadeia válida de `AudioProcessor` sem `match` ou downcast para structs concretas de amplificadores ou pedais individuais.
- **CA-3 (`ENGINEERING_TARGET` — Validação Atômica por Modelo):** Enviar via `api::dispatch` ou `apply_param_rt` um lote de parâmetros contendo qualquer chave inexistente no `ModelDescriptor` do modelo ativo, valor fora de `[min, max]` ou `NaN`/`Inf` retorna `Err(ParamError)` (HTTP 400 / `E_VALIDATION:`) e mantém todos os demais parâmetros do lote inalterados (rollback atômico).
- **CA-4 (`ANALYTICAL` — Invariância a Tamanho de Bloco e Taxa de Amostragem):** Com um grafo linear de referência alimentado por senoide de `1 kHz` a `-12 dBFS` (`0.2512` pico), processar a mesma janela de `4800` amostras fatiada em blocos de `32`, `64`, `128`, `441`, `480` ou `1024` frames produz saída bit-a-bit idêntica (`diferença máxima <= 1e-6`), e a resposta na banda passante linear (`100 Hz–10 kHz`) varia `<= 0,2 dB` entre `44,1`, `48`, `88,2` e `96 kHz` (tolerância derivada do erro de discretização bilinear).
- **CA-5 (`ANALYTICAL` — Compensação de Latência Paralela / PDC):** Em um grafo com `SplitNode` (`BalanceAb`) onde o Ramo A contém um processador com `latency_samples() == L` (`L = 64` amostras @ `48 kHz`, `block_size = 128`) e o Ramo B contém `latency_samples() == 0`, injetar um impulso unitário (`x[0] = 1.0`, `x[n>0] = 0.0`) produz um único pico alinhado na amostra `n = L` na saída do `MixerNode` (`erro temporal = 0 amostras`), e `total_algorithmic_latency_samples()` reporta exatamente `L`.
- **CA-6 (`ENGINEERING_TARGET` — Fronteira Off-RT vs Zero-Alloc RT Medida):** Com o `CountingAllocator` ativo exclusivamente na thread de áudio (`48 kHz`, `block_size = 128`, `200` blocos consecutivos), executar `process()`, `apply_param_rt()`, crossfade de `20 ms` para um processador sombra previamente preparado na thread de controle (enviado via `SpscOwnedQueue`) e enviar o processador antigo ao `RetiredReaper` registra `alloc_count == 0` e `dealloc_count == 0` na thread RT.
- **CA-7 (`ENGINEERING_TARGET` — Persistência Declarativa de Preset):** Serializar e desserializar um preset preserva `schema_version`, `model_id`, `model_revision` e valores nominais de parâmetros sem incluir buffers internos de filtro/delay; tentar carregar um preset com `model_id` ou `model_revision` inexistente retorna erro tipado explícito sem fallback silencioso.
- **CA-8 (`ENGINEERING_TARGET` — Desacoplamento de Crates):** `cargo tree -p ampforge-dsp` confirma ausência total de `tauri`, `cpal`, `sonix-desktop` ou crates de rede/HTTP na árvore de dependências do núcleo DSP.
- **CA-9 (`ENGINEERING_TARGET` — Headroom Interno e Sanitização Não-Finita):** Um módulo intermediário de boost com ganho `+6 dB` alimentado por senoide de `1 kHz` a `-3 dBFS` (`0.7079` pico) entrega `≈ 1.414` pico ao bloco seguinte no barramento interno sem ceifamento em `1.0`, enquanto o estágio final da cadeia limita a saída entregue ao host em `[-1.0, 1.0]`; injetar `NaN`, `+Inf`, `-Inf` ou `±1e6` na entrada produz saída 100% finita em `[-1.0, 1.0]` e recupera operação normal em `<= 2` blocos após retorno da entrada a `-12 dBFS`.
- **CA-10 (`REGRESSION` — Não-regressão Global):** Todas as suítes de contrato existentes das Specs `001` a `052` permanecem 100% aprovadas.

## 19. Dependências
- **Obrigatórias (fundação existente):** `Specs 001, 002, 014, 016, 025, 033, 037, 044, 045, 049`.
- **Dependentes desta Spec:** `Specs 054` a `092`.

## 20. Limitações conhecidas
- Barramento estéreo completo ponta-a-ponta (`ChannelLayout::Stereo` ativo em todos os blocos pós-gabinete) será ativado incrementalmente a partir deste contrato; blocos atualmente mono declaram `ChannelLayout::Mono` honestamente no `ModelDescriptor`.

## 21. Riscos técnicos
- Overhead de despacho dinâmico (`dyn AudioProcessor`) por bloco de `128` amostras deve ser monitorado com Criterion para permanecer `< 3%` do orçamento do período.

## 22. Matriz de rastreabilidade
| Requisito | Fonte | Spec/CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Trait comum `AudioProcessor`, `ModelDescriptor` e `ProcessorRegistry` | F1, F2 | `Spec 053 / CA-1, CA-2` | `crates/ampforge-dsp/src/engine.rs`, `crates/sonix-desktop/src/audio.rs:2558–5190 @ c20cc65` | Planejado: `contract_053_modular_architecture.rs` | `SPECIFIED` |
| Validação atômica por modelo e rejeição de parâmetros incompatíveis | F2, F4 | `Spec 053 / CA-3` | `crates/sonix-desktop/src/api.rs:1400–2450 @ c20cc65` | `crates/sonix-desktop/tests/contract_014_ui_dsp_bridge.rs` | `TESTED` |
| Suporte a blocos arbitrários (`32..2048`, incl. `441/480`) e múltiplas taxas | F2, F4 | `Spec 053 / CA-4` | `crates/sonix-desktop/src/audio.rs:4533–4540 @ c20cc65` | `crates/ampforge-dsp/tests/contract_053_auditoria_039.rs::ca4_invariancia_ao_tamanho_de_bloco` | `TESTED` |
| Compensação automática de latência paralela (PDC) no `MixerNode` | F1 | `Spec 053 / CA-5` | `crates/ampforge-dsp/src/dag.rs:265–345 @ c20cc65` (sem PDC automático ainda) | Planejado: `contract_053_modular_architecture.rs` | `SPECIFIED` |
| Separação Off-RT (`prepare`/queues) vs Zero-Alloc RT (`process`/`RetiredReaper`) | F2, F4, F5 | `Spec 053 / CA-6` | `crates/sonix-desktop/src/audio.rs:2666–2670 @ c20cc65, 4528–4707` | `crates/sonix-desktop/tests/contract_037_realtime_safety.rs`, `contract_044_sdd_integrity.rs` | `TESTED` |
| Persistência declarativa (`model_id`, `model_revision`, `schema_version`) | F4 | `Spec 053 / CA-7` | `crates/sonix-core/src/persistence.rs` | `crates/sonix-core/tests/contract_010_persistence_offline.rs` | `IMPLEMENTED_UNVERIFIED` |
| Isolamento de dependências de `ampforge-dsp` | F1 | `Spec 053 / CA-8` | `crates/ampforge-dsp/Cargo.toml` | `crates/ampforge-dsp/tests/contract_001_nucleo_dsp.rs` | `TESTED` |
| Headroom interno acima de `0 dBFS` + limiter final `[-1.0, 1.0]` + sanitização `NaN/Inf` | F2, F4 | `Spec 053 / CA-9` | `crates/ampforge-dsp/src/amp.rs:2023 @ c20cc65` | — | `NÃO ATENDIDO — o amp limita em ±0,999` |
