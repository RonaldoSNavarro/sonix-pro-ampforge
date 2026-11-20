---
spec: 053
titulo: Arquitetura DSP modular, contratos de processadores, ciclo de vida e independência de módulos
status: rascunho
tipo: arquitetura-dsp
atualizado: 2026-11-19
dependencias: [001, 002, 014, 025, 033, 037, 044, 045, 049]
---
# 053 — Arquitetura DSP Modular e Contratos Comuns de Processadores

## 1. Identificação e objetivo
- **Identificador estável (`module_id`):** `ampforge.core.architecture.v1`
- **Categoria:** Arquitetura Central DSP (Spec de fundação para Specs 054–084)
- **Objetivo:** Definir os contratos técnicos comuns do motor DSP modular do **SONIX PRO // AMPFORGE™** — ciclo de vida, formatos de áudio, descritores de parâmetros tipados, registro de modelos, gerenciamento de estado, latência, tempo real e independência entre módulos — para que cada amplificador, pedal e componente compartilhado seja implementado, testado e evoluído de forma isolada sem acoplamento indevido no engine central.

## 2. Escopo
- Contratos de trait e structs centrais do engine em `crates/ampforge-dsp`: `AudioSpec`, `ChannelLayout`, `ModuleId`, `ModelRevision`, `ParamId`, `ParamDescriptor`, `ParamTaper`, `ParamValue`, `AudioBlockRef`/`AudioBlockMut`, `ProcessContext`, `AudioProcessor`, `ModelDescriptor` e `ProcessorFactory`/`ProcessorRegistry`.
- Ciclo de vida padronizado: criação (off-RT), `prepare(spec)` (off-RT), `process(ctx, input, output)` (RT zero-alloc), `set_param(id, value)` (RT zero-alloc), `reset()` (O(1) RT-safe), `latency_samples()`, `retire()` (devolução de memória para thread de controle).
- Separação entre **estado DSP efêmero** (filtros, atrasos, envelopes) e **estado persistido de preset** (`model_id`, `model_revision`, `schema_version`, `params`).
- Regras de acoplamento zero entre modelos: o engine executa o grafo e distribui parâmetros por contrato, sem conhecer topologia interna de nenhum amplificador ou pedal.
- Contrato de compatibilidade com a camada de controle (`crates/sonix-desktop/src/api.rs`), plugins DAW (`crates/sonix-plugin`), persistência (`crates/sonix-core`) e UI (`apps/sonix-app/ui/src/controls.registry.ts`).

## 3. Fora de escopo
- Reescrita imediata dos amplificadores, pedais ou `RealtimeDspChain` nesta etapa (esta spec define o contrato; a implementação ocorrerá somente após revisão manual e aprovação).
- Alterações de protocolo de driver de áudio Windows (WASAPI/ASIO — cobertos pelas Specs 018, 036 e 049).
- Alterações visuais ou de telas da UI Tauri v2 (cobertas pelas Specs 033, 048 e 052).
- Introdução de crates externas sem justificativa técnica formal.

## 4. Equipamento ou componente de referência
- **Fabricante / Origem:** Arquitetura interna SONIX PRO // AMPFORGE™.
- **Modelo exato:** Engine DSP Modular v1 (`ampforge-dsp` + camada de orquestração).
- **Papel na cadeia de sinal:** Infraestrutura comum de execução do grafo 4×12, registro de módulos, gerenciamento de recursos e contratos de tempo real que hospeda todos os blocos de processamento (Specs 054–084).

## 5. Revisão e variantes
- **Revisão de referência:** `v1.0` (evolução da arquitetura da 1ª geração entregue nas Specs 001–051).
- **Variantes de hospedagem suportadas pelo mesmo núcleo:**
  1. Standalone Desktop Windows (`apps/sonix-app` Tauri v2 + `crates/sonix-desktop` via `cpal`).
  2. Plugins DAW VST3 e CLAP (`crates/sonix-plugin` via `nih_plug`).
  3. Harness determinístico offline (`cargo test` / benchmarks Criterion sem hardware de áudio).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | Código-fonte `ampforge-dsp` | Workspace v1.0.0 | `config.rs`, `engine.rs`, `dag.rs`, `queue.rs`, `amp.rs` | `crates/ampforge-dsp/src/` | 2026-11-19 | `[confirmado em documentação]` | Estado atual do núcleo DSP e acoplamentos existentes |
| F2 | Código-fonte `sonix-desktop` | Workspace v1.0.0 | `audio.rs` (`RealtimeDspChain`), `api.rs` | `crates/sonix-desktop/src/` | 2026-11-19 | `[confirmado em documentação]` | Orquestração atual de 5093 linhas, filas SPSC, `RetiredReaper` |
| F3 | Código-fonte `sonix-app` | Tauri `2.12.5` | `commands.rs`, `streams.rs`, `controls.registry.ts` | `apps/sonix-app/` | 2026-11-19 | `[confirmado em documentação]` | Fronteira IPC Tauri v2 e registro único de controles |
| F4 | Specs 001, 014, 025, 033, 037, 044, 049 | v1.0 | Contratos de RT, IPC, SDD e Plugins | `specs/` | 2026-11-19 | `[confirmado em documentação]` | Invariantes já aprovadas e testes existentes |
| F5 | Ross Bencina, *Real-time audio programming 101: time waits for nothing* | Artigo técnico | Seções de alocação, locks e reclamation | https://www.rossbencina.com/code/real-time-audio-programming-101-time-waits-for-nothing | 2026-11-19 | `[fonte secundária]` | Fundamentos de programação hard real-time em áudio |
| F6 | Tauri v2 Architecture & IPC Docs | Tauri v2 (`2.12.x`) | Security, Capabilities, IPC, Channels | https://v2.tauri.app/ | 2026-11-19 | `[confirmado em documentação]` | Limites IPC, `Channel` e isolamento de estado |

## 7. Fundamentação elétrica
Não aplicável diretamente a um circuito analógico único; porém o contrato deve preservar invariantes elétricas e físicas na troca de sinais entre módulos:
- **Referência nominal de sinal (0 dBFS / nível interno):** barramento `f32` normalizado onde `-12 dBFS` (~`0.251` pico) representa o nível calibrado nominal de palhetada forte de guitarra (Spec 019, `calibration.rs:81`), preservando headroom interno entre estágios sem clipping prematuro de ponto flutuante e com limitação segura `[-1.0, 1.0]` (`<= 0 dBFS`) apenas na saída final da cadeia (Spec 016).
- **Unidades físicas explícitas:** parâmetros devem carregar unidade física real (`dB`, `Hz`, `ms`, `s`, `V`, `mA`, `Ohm`, `cm`, `deg`, `%`, `st`, `scale_0_10`) em vez de escalas opacas sem significado.
- **Propagação de metadados físicos entre módulos adjacentes (quando aplicável):** suporte a sinalização desacoplada de impedância nominal de saída/entrada (ex.: interação captador↔Fuzz Face, Spec 076/063; frequência de ressonância do gabinete↔power amp, Spec 080/077) via portas ou sinais de acoplamento tipados no `ProcessContext`, sem que um módulo importe structs privadas de outro.

## 8. Comportamento esperado
1. **Grafo de processamento modular:** o engine compila uma topologia declarativa (grade 4×12 com nós de entrada, blocos, Split, Mixer e taps de saída) fora da thread RT, produzindo um plano de execução pré-alocado (`CompiledGraph`) enviado ao callback via fila lock-free e trocado atomicamente com crossfade sem estalos (≤ 20 ms).
2. **Processadores autônomos por contrato:** cada amplificador (Specs 054–058), pedal/rack (Specs 059–075) e componente compartilhado (Specs 076–084) implementa o contrato comum `AudioProcessor` + `ModelDescriptor`. O engine invoca `prepare`, `process`, `apply_param`, `reset` e `latency_samples` sem `match` gigante sobre detalhes internos de modelos.
3. **Formatos de áudio e canais explícitos:**
   - Taxas oficiais: `44_100`, `48_000`, `88_200`, `96_000` Hz (extensível via `SampleRateHz` validado).
   - `max_block_size`: suporta qualquer bloco recebido no callback (incluindo blocos não potências de 2 como `441`, `480`, `882`, `960` frames) subdividindo em chunks internos `<= prepared_max_block`.
   - `ChannelLayout`: explícito (`Mono`, `Stereo`, `MonoToStereo`, `MultiChannel(n)`) — proibido reduzir estéreo para mono ou duplicar canais silenciosamente sem declaração no descritor do bloco.
   - Formato de barramento interno: `f32` não-intercalado por canal (planar), com precisão mista `f64` interna em filtros críticos (`< 200 Hz`, tone stacks, solvers não lineares e DC blocker).
4. **Parâmetros tipados, taper e suavização:**
   - Identificador estável composto (`block_instance.param_key` + `ParamId(u32)` numérico para a thread RT sem strings/hashing no callback).
   - Cada parâmetro declara tipo (`Continuous`, `Boolean`, `DiscreteEnum`), unidade, faixa `[min, max]`, valor padrão, curva de taper (`Linear`, `AudioLog10Pct`, `LogFreq`, `InvertedLog`, `CustomLookup`), política de suavização (`None`, `OnePoleMs(f32)`, `CrossfadeMs(f32)`) e se exige recálculo de coeficientes.
   - Validação atômica por modelo: parâmetros incompatíveis com o modelo ativo (ex.: `master` no Plexi 1959 ou `middle` no AC30 Top Boost) são rejeitados com `E_VALIDATION:` e nunca alteram o estado interno silenciosamente.
5. **Bypass padronizado e spillover:**
   - Todo módulo suporta bypass sem estalo (rampa equal-power ou linear ≤ 10 ms).
   - Módulos baseados em tempo (`Delay`, `Reverb`) declaram suporte a `Spillover`: ao bypassar com spillover ativo, zeram a alimentação da entrada mantendo a cauda somada ao sinal direto.
6. **Compensação de latência (PDC):** cada nó reporta `latency_samples() -> usize`. Quando dois ramos paralelos (Linha A e Linha B) são somados em um `MixerNode`, o engine calcula o atraso diferencial `|lat_A - lat_B|` na compilação do grafo e insere linha de atraso compensadora pré-alocada no ramo mais rápido, além de reportar a latência algorítmica total real à telemetria/host DAW.
7. **Persistência e versionamento de presets:**
   - Presets salvam `schema_version`, `model_id` estável, `model_revision` (ex.: `"1968-jmp"`, `"ab763-1965"`, `"rev-f-1992"`) e mapa de parâmetros nominais — **jamais** o estado interno de filtros/buffers.
   - Carregar preset com `model_id` ou `model_revision` desconhecido retorna erro recuperável explícito; nunca substitui silenciosamente por outro equipamento.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito / engine | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Oversampling Global | `engine.oversampling` | enum | fator | `1x, 2x, 4x, 8x` (`2x, 4x, 8x` UI) | `2x` | discreto | Controla fator de sobreamostragem dos estágios não lineares | Recalcula filtros/latência fora do RT ou troca pré-alocada |
| Nível Master Saída | `master.level_db` | number | dB | `-60.0..=12.0` | `0.0` | linear em dB | Trim mestre pós-cadeia (`<= -59.9 dB` muta) | Ganho suavizado antes do limiter seguro `<= 0 dBFS` |
| Bypass de Bloco | `<block>.enabled` | boolean | bool | `false / true` | por bloco | discreto | True bypass / bypass com spillover | Crossfade suave ≤ 10 ms sem cliques |
| Seleção de Modelo | `<block>.model` | enum | id estável | catálogo do bloco | por bloco | discreto | Seleciona módulo + revisão no `ProcessorRegistry` | Troca por sombra pré-alocada + crossfade ≤ 20 ms |

## 10. Contratos de integração
- **Engine ↔ Módulos (`ampforge-dsp`):**
  - `pub trait AudioProcessor: Send` com métodos: `descriptor(&self) -> &ModelDescriptor`, `prepare(&mut self, spec: AudioSpec) -> Result<(), DspError>`, `reset(&mut self)`, `set_param(&mut self, id: ParamId, val: ParamValue) -> Result<(), ParamError>`, `process(&mut self, ctx: &ProcessContext, input: &AudioBlockRef<'_>, output: &mut AudioBlockMut<'_>)`, `latency_samples(&self) -> usize`, `drain_retired(&mut self, sink: &mut dyn RetiredSink)`.
  - `ProcessorRegistry` registra fábricas por `(BlockCategory, ModuleId, ModelRevision)` sem depender de `match` centralizado hardcoded em `sonix-desktop`.
- **Engine ↔ Control Plane (`sonix-desktop::api`):**
  - `api::dispatch` consulta os descritores do `ProcessorRegistry` para validar faixas, tipos, enums e disponibilidade por modelo de forma atômica, convertendo nomes externos (`"amp_1.gain"`) para `ParamId` numérico antes de enfileirar na `SpscBuffer`.
- **Engine ↔ UI (`apps/sonix-app/ui/src/controls.registry.ts`):**
  - A UI consome descritores de controles (`id`, `backendKey`, `type`, `range`, `unit`, `options`, `defaultValue`) sem conhecer a topologia elétrica interna de nenhum módulo. Todo controle visível continua obrigatório no registro único (Regra `20261113-195929-regra-0a2f`).
- **Engine ↔ Plugins DAW (`crates/sonix-plugin`):**
  - `sonix-plugin` instancia o mesmo grafo/módulos de `ampforge-dsp` sem importar `sonix-desktop::server` nem `tauri`.

## 11. Arquitetura de implementação
- **Código atual mapeado:**
  - `crates/ampforge-dsp/src/{config.rs, errors.rs, queue.rs, smoother.rs, engine.rs, dag.rs}`
  - `crates/sonix-desktop/src/audio.rs` (`RealtimeDspChain`, linhas 2542–5093)
  - `crates/sonix-desktop/src/api.rs` (tabela de parâmetros e validação)
  - `crates/sonix-core/src/{preset.rs, persistence.rs, modifiers.rs, footswitch.rs}`
  - `crates/sonix-plugin/src/lib.rs`
- **Dívidas e acoplamentos registrados no diagnóstico (2026-11-19):**
  1. `RealtimeDspChain` (5093 linhas em `sonix-desktop/src/audio.rs`) mistura orquestração DSP, slots fixos e código de host `cpal`, enquanto `ampforge-dsp/src/engine.rs` (`DspEngine`) ficou como fachada legada da Spec 001.
  2. `WhiteBoxAmpProcessor` (`amp.rs`, 1738 linhas) concentra os 5 amps via `match self.model_id`; `DriveProcessor` (`fx/drive.rs`) concentra 6 drives; `ModulationProcessor` (`fx/modulation.rs`) concentra 7 modelos; `DelayProcessor` (`fx/delay.rs`) concentra 6 modelos; `ReverbProcessor` (`fx/reverb.rs`) concentra 5 modelos.
  3. `BiquadFilter` está reimplementado de forma duplicada em 6 arquivos (`amp.rs`, `cab.rs`, `fx/drive.rs`, `fx/dynamics.rs`, `fx/delay.rs`, `precision.rs`), sendo alguns em `f32` DF-I em vez de `f64` TDF-II.
  4. `sonix-plugin` ainda depende de `sonix-desktop` (com `default-features = false`) apenas para acessar `RealtimeDspChain`; com a migração do grafo executável para `ampforge-dsp`, `sonix-plugin` dependerá apenas de `ampforge-dsp` + `sonix-core`.
  5. Os nomes de parâmetros ainda usam strings em várias bordas (`api.rs`, `preset.rs`, `sonix-plugin/src/lib.rs`) em vez de `ParamId` derivado do descritor de cada modelo.
- **Estrutura modular alvo em `crates/ampforge-dsp/src/`:**
  - `contracts/` (`audio_spec.rs`, `processor.rs`, `params.rs`, `registry.rs`, `block.rs`, `retired.rs`)
  - `primitives/` (`biquad64.rs`, `svf64.rs`, `dc_blocker.rs`, `smoother.rs`, `delay_line.rs`, `oversampling.rs`)
  - `shared/` (`tube_stage.rs`, `power_amp.rs`, `tonestack.rs`, `cab_ir.rs`, `input_stage.rs`, `tuner.rs`, `looper.rs`)
  - `amps/` (`plexi_1959.rs`, `jcm800_2203.rs`, `twin_ab763.rs`, `dual_rect_revf.rs`, `ac30_tb.rs`)
  - `pedals/` (`ts808.rs`, `klon.rs`, `rat.rs`, `big_muff.rs`, `fuzz_face.rs`, `ds1.rs`, `noise_gate.rs`, `comp_1176.rs`, `eq_mxr10.rs`, `re201.rs`, `ce2.rs`, `flanger.rs`, `phase90.rs`, `univibe.rs`, `tremolo.rs`, `rotary.rs`, `pitch.rs`)
  - `graph/` (`dag.rs`, `compiler.rs`, `executor.rs`, `pdc.rs`)

## 12. Estratégia DSP
- **Política híbrida sem dogma:** cada módulo escolhe sua estratégia (MNA, WDF, State-Space, ADAA1/2, convolução particionada, behavioral com memória) e a declara em sua própria spec.
- **Precisão mista obrigatória:** barramento entre módulos em `f32` planar; acumuladores de filtros `< 200 Hz`, tone stacks, solvers não lineares e DC blocker em `f64`.
- **Proteção contra denormals e não-finitos:** sanitização de `NaN`/`Inf` na fronteira do grafo e corte de denormals (`< 1e-24`) em loops recursivos IIR/comb/reverb.

## 13. Requisitos de tempo real
- **Zero heap allocation e zero deallocation** em `process()` e `set_param()`, verificado obrigatoriamente por alocador global de contagem na thread RT (Regra `20261114-131934-regra-d272`, Spec 037 CA-7).
- **Zero locks bloqueantes (`Mutex`, `RwLock`), zero I/O, zero logging síncrono, zero panic.**
- **Troca segura de grafo/modelo:** preparação na thread de controle → entrega via `SpscOwnedQueue` → crossfade equal-power ≤ 20 ms → transferência do objeto aposentado para `RetiredReaper` na thread de controle.
- **Imutabilidade de capacidade em RT:** nenhuma estrutura (`Vec`, `HashMap`, buffers de delay/FFT) pode crescer ou realocar dentro de `process()`.

## 14. Gestão de estado
- **Separação estrita:**
  - `ModulePresetState { module_id, model_revision, schema_version, params: BTreeMap<ParamKey, ParamValue> }` — serializável e determinístico.
  - Estado DSP interno (registradores de filtros, linhas de atraso, envoltórias, fases LFO, `sag_state`) — exclusivamente em memória dentro da instância do processador, zerado por `reset()`.
- **Determinismo:** duas instâncias recém-criadas + `prepare()` + mesma sequência de `set_param()` + mesma entrada produzem saídas bit-a-bit equivalentes (ou dentro de `1e-6` em arquiteturas com FMA distinto).

## 15. Latência
- Cada `AudioProcessor::latency_samples()` retorna o atraso algorítmico exato em amostras na `sample_rate` preparada.
- O compilador do grafo calcula:
  - `path_latency[node]` para cada caminho do DAG;
  - atrasos de compensação PDC nos nós `MixerNode`;
  - `total_algorithmic_latency_samples` publicado atomicamente para telemetria (`get_audio_status`) e reportado ao host VST3/CLAP.

## 16. Consumo de CPU
- Overhead da infraestrutura do grafo modular (dispatch de nós, PDC, suavização de bypass, sanitização) deve permanecer **`<= 3%`** do orçamento de CPU de 128 frames @ 48 kHz (`2,67 ms`) em build `--release` em CPU x86_64 moderna, medido via benchmark Criterion dedicado.
- O orçamento total de uma cadeia completa típica (Gate + Drive + Amp @ 2x OS + Cab IR + Delay + Reverb) deve permanecer **`< 45%`** do deadline de 128 frames @ 48 kHz em `--release`.

## 17. Testes
- **Testes de contrato de arquitetura (`contract_053_modular_dsp_architecture.rs`):**
  - Verificação de ciclo de vida (`prepare` em 44,1/48/88,2/96 kHz e blocos 32/64/128/441/480/512/1024).
  - Teste com alocador global de contagem comprovando `0` alocações e `0` desalocações em `process()`, `set_param()`, troca de modelo, troca de grafo e `reset()`.
  - Teste de compensação de latência PDC em grafo paralelo (ramo A com latência `L` vs ramo B com latência `0`, validando alinhamento de impulso em `±0` amostras no merge).
  - Teste de isolamento de modelos: alterar código/parâmetros de um módulo não altera saída de outro módulo no registro.
  - Teste de erro: rejeição de `sample_rate` inválida, `ChannelLayout` incompatível, `ParamId` desconhecido para o modelo, valor fora de faixa/`NaN`/`Inf`, e `model_id`/`model_revision` inexistente em preset.
  - Benchmark Criterion comparando overhead do grafo modular vs cadeia fixa atual.

## 18. Critérios de aceite
- **CA-1 (Contratos de processador):** Todos os amplificadores, pedais e componentes compartilhados implementam o trait comum `AudioProcessor` e expõem `ModelDescriptor` com `module_id`, `model_revision`, `ChannelLayout`, `latency_samples()` e lista completa de `ParamDescriptor`.
- **CA-2 (Independência do engine central):** O executor do grafo em `ampforge-dsp` executa qualquer cadeia registrada no `ProcessorRegistry` sem `match` ou dependência direta sobre structs concretas de amplificadores ou pedais individuais.
- **CA-3 (Validação atômica por modelo):** Enviar um parâmetro inexistente no `ModelDescriptor` do modelo ativo (ou fora da faixa `[min, max]`, ou `NaN`/`Inf`) retorna `Err(ParamError)` sem alterar nenhum outro parâmetro da requisição.
- **CA-4 (Suporte a blocos arbitrários e taxas múltiplas):** O mesmo grafo processa corretamente blocos de `32`, `64`, `128`, `441`, `480` e `1024` amostras em `44,1`, `48`, `88,2` e `96 kHz` com diferença de ganho `<= 0,2 dB` na banda passante linear e sem artefatos de borda de bloco.
- **CA-5 (Compensação de latência paralela — PDC):** Em um split paralelo onde o Ramo A possui `L > 0` amostras de latência e o Ramo B possui `0`, um impulso unitário na entrada chega alinhado (`diferença = 0 amostras`) na soma do `MixerNode`, e `total_algorithmic_latency_samples()` reporta exatamente `L`.
- **CA-6 (Zero-alloc e zero-lock medidos):** Com alocador global de contagem ativo na thread RT, `process()`, `set_param()`, troca de modelo (20 ms crossfade) e aposentadoria de recursos registram `alloc_count == 0` e `dealloc_count == 0`.
- **CA-7 (Separação preset vs estado DSP):** Presets serializam `schema_version`, `model_id`, `model_revision` e parâmetros nominais; carregar um preset com `model_id` ou `model_revision` ausente retorna erro explícito sem substituição silenciosa.
- **CA-8 (Desacoplamento de hosts):** `ampforge-dsp` compila sem dependências de `tauri`, `cpal`, `sonix-desktop` ou I/O de rede/sistema de arquivos; `sonix-plugin` e `sonix-desktop` consomem a mesma API do grafo.
- **CA-9 (Estabilidade sob entrada malformada):** Injetar `NaN`, `+Inf`, `-Inf` ou impulsos de `±1e6` na entrada do grafo não gera panic, não corrompe permanentemente filtros após retorno a sinal normal e mantém toda amostra de saída finita em `[-1.0, 1.0]`.
- **CA-10 (Não-regressão):** Todas as suítes de contrato existentes (Specs 001–051) continuam passando após a migração incremental para o novo contrato.

## 19. Dependências
- **Obrigatórias (fundação existente):** `Spec 001` (núcleo RT), `Spec 002` (DAG 4×12), `Spec 014` (ponte de controle), `Spec 025` (presets/cenas), `Spec 033` (registro único de controles), `Spec 037` (zero-alloc medido), `Spec 044` (integridade SDD), `Spec 049` (plugins/DAG/modifiers).
- **Dependentes desta Spec:** Todas as specs modulares `054` a `084`.

## 20. Limitações conhecidas
- Processamento estéreo verdadeiro (L/R independentes em toda a cadeia, habilitando Ping-Pong estéreo e Tri-Chorus estéreo real) exigirá migração gradual dos blocos atualmente mono; esta spec já prepara `ChannelLayout` explícito para suportar ambos sem quebra de contrato.
- A migração de `RealtimeDspChain` (5093 linhas) deve ocorrer por fases controladas, módulo a módulo, preservando todos os testes de contrato verdes a cada etapa.

## 21. Riscos técnicos
- **Risco de indirection/vtable overhead:** uso de `Box<dyn AudioProcessor>` por bloco (não por amostra) amortiza chamadas virtuais em ~128+ amostras; ainda assim, deve ser medido via Criterion para garantir overhead `<= 3%`.
- **Risco de quebra de compatibilidade de chaves IPC/presets:** manter camada de compatibilidade/aliases versionados na borda de `api.rs` e `sonix-core` durante a transição.
- **Risco de over-engineering:** evitar abstrações genéricas profundas demais; traits devem permanecer pequenos e orientados a blocos.

## 22. Referências e rastreabilidade
- **Specs relacionadas:** `001`, `002`, `014`, `025`, `033`, `037`, `044`, `045`, `049`, `051`.
- **Código atual:** `crates/ampforge-dsp/src/lib.rs`, `crates/ampforge-dsp/src/engine.rs`, `crates/ampforge-dsp/src/dag.rs`, `crates/sonix-desktop/src/audio.rs`, `crates/sonix-desktop/src/api.rs`, `crates/sonix-plugin/src/lib.rs`.
- **Testes futuros:** `crates/ampforge-dsp/tests/contract_053_modular_dsp_architecture.rs`.
