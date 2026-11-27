# Biblioteca Pública Sem Uso Direto em Produção (Spec 044 CA-2)

Este documento lista **todo item público (`pub struct`, `pub enum`, `pub trait`, `pub type`)** das crates de núcleo (`ampforge-dsp` e `sonix-core`) que ainda não é referenciado diretamente em `crates/sonix-desktop/src` ou `apps/sonix-app/src-tauri/src`, informando se é um subtipo transitivo interno ou se aguarda integração pela spec correspondente.

> Auditoria automática em `crates/sonix-desktop/tests/contract_044_sdd_integrity.rs` (`test_ca_02_every_pub_item_in_core_crates_is_used_in_prod_or_documented`). À medida que cada spec abaixo for implementada em produção, os itens correspondentes devem ser removidos desta lista.

## `crates/ampforge-dsp/src`

| Item público | Arquivo | Origem | Motivo / Spec de retomada |
|---|---|---|---|
| `AmpPanelTheme`, `AmpChannelVoicing`, `AmpModelDescriptor`, `AmpFrontPanel`, `ChassisModSettings`, `AmpModelCatalog`, `BiquadFilter`, `BiquadTdf2State`, `BiquadTdf2F32`, `Jcm800PeakingNetwork`, `PlexiBrightCapFilter` | `amp.rs`, `biquad.rs`, `cab.rs` | Spec 003 / 005 / AUD-27 / AUD-45 | Subtipos/filtros internos dos módulos `amp`, `cab`, `dag` e `fx/*`, ou metadados de catálogo pendentes de exposição direta na UI/backend pela **Spec 039** (amps autênticos) e **Spec 038** (cab/IR). |
| `EllipSos`, `AmpOversamplerBank`, `Marshall1959SuperLead`, `MarshallJcm800_2203`, `FenderTwinReverbAb763`, `MesaDualRectifierRevG`, `VoxAc30TopBoost1964` | `amps/*.rs` | Specs 054–058 | Amplificadores modulares da Fase 2A registrados em `ProcessorRegistry`, pendentes de ligação no `SignalGraph` / host em tempo real na **Spec 079** (Fase 2B). |
| `AudioBlockMut`, `AudioBlockRef`, `AudioProcessor`, `AudioSpec`, `ChannelLayout`, `DspError`, `ModelDescriptor`, `ModelRevision`, `ModuleId`, `ParamDescriptor`, `ParamError`, `ParamId`, `ParamTaper`, `ParamValue`, `ProcessContext`, `ProcessorFactory`, `RetiredResource`, `RetiredSink`, `SpscRetiredSink` | `processor.rs` | Spec 053 | Contratos genéricos de arquitetura modular de blocos (`AudioProcessor`, `ModelDescriptor`, `ParamDescriptor`, `SpscRetiredSink`), aguardando integração ao host/DAG na **Spec 079** (Fase 2B). |
| `ModularParallelExecutor`, `PdcDelayLine` | `dag.rs` | Spec 053 | Executor paralelo modular com compensação automática de latência (PDC), aguardando integração ao `SignalGraph` na **Spec 079** (Fase 2B). |
| `FmvToneStackSolver`, `JamesBaxandallToneStackSolver`, `MesaRectifierToneStackSolver`, `VoxTopBoostToneStackSolver` | `tonestack.rs` | Spec 078 | Solvers analíticos individuais de Tone Stack (`f64`), consumidos pelos amplificadores modulares (`amps/*.rs`) e via fachada `PassiveToneStack`, aguardando exposição direta no host pela **Spec 079**. |
| `BiasMode`, `BiasTopology`, `CathodeFollowerStage`, `OutputTransformer`, `OutputTransformerModel`, `PentodePreampStage`, `PhaseInverterLtpStage`, `PowerSupplyAndRectifier`, `PowerSupplySag`, `PushPullPowerStage`, `ReactiveLoadAndNfb`, `RectifierKind`, `TriodeParams`, `TriodeStage` | `tube/*.rs` | Spec 077 | Biblioteca física de válvulas e estágios analógicos usada pelos amplificadores modulares (`amps/*.rs`); entra em produção ao ligar o `SignalGraph` na **Spec 079** (Fase 2B). |
| `MicSlotConfig`, `ImpedanceCurve`, `CabAcousticConditioning` | `cab.rs` | Spec 005 / 022 | Subtipos de modelagem física de microfones e curva de impedância preparados para integração completa no caminho de áudio pela **Spec 038** (convolução IR particionada, mics 1/2, distância/ângulo e ressonância do gabinete). |
| `AudioConfig` | `config.rs` | Spec 001 | Configuração abstrata de motor offline; será substituída/ligada ao host de áudio real pela **Spec 036** ou mantida somente para `DspEngine`. |
| `SlotId`, `BlockType`, `BlockInstance`, `SlotCell`, `RowInput`, `RowOutput`, `GridMatrix4x12`, `DagCycleError`, `DagGraph` | `dag.rs` | Spec 002 | Topologia dinâmica do grafo 4×12 ainda executa cadeia fixa em `RealtimeDspChain`; integração real planejada pela **Spec 045**. |
| `DspEngine`, `InvalidAudioConfigError`, `DspEngineError` | `engine.rs`, `errors.rs` | Spec 001 | Fachada histórica de testes da Spec 001 (substituída em produção por `RealtimeDspChain`); manter documentado ou consolidar após **Spec 045**. |
| `LooperState` | `looper.rs` | Spec 008 | Enum de estado do looper consumido internamente via `LooperProcessor::state()`; telemetria detalhada/UI ligada pela **Spec 043**. |
| `ToneStackPosition`, `PowerClass`, `CrtScopeMode`, `CrtScopeTelemetry`, `ChassisWorkbench` | `modshop.rs` | Spec 004 / 021 | Subtipos de bancada/chassi e telemetria CRT aguardando dados reais de bias/tensão/espectro no backend pela **Spec 047**. |
| `OversamplingFactor`, `HalfBandOversampler` | `oversampling.rs` | Spec 001 / 044 | Filtro polyphase half-band Kaiser 127-tap (>= 96 dB medido via DFT pela **Spec 044 CA-5**); integração em todos os estágios não-lineares de drives/amps pela **Spec 041** e **Spec 039**. |
| `IirBiquad64` | `precision.rs` | Spec 001 | Utilitário de precisão mista usado na Spec 001; integração sistemática em filtros < 100 Hz pela **Spec 039 / 040**. |
| `ParamSmoother` | `smoother.rs` | AUD-11 | Rampa linear de 20 ms usada internamente em `amp.rs`, `cab.rs`, `fx/eq.rs` e `fx/wah.rs` (não instanciada diretamente em `sonix-desktop/src`). |
| `SpscTelemetryQueue`, `TripleBuffer`, `TripleProducer`, `TripleConsumer`, `DspCommand`, `SpscCommandQueue` | `queue.rs` | Spec 001 / AUD-17 / Spec 093 | Filas SPSC e Triple Buffer lock-free da camada `ampforge-dsp` (`SpscTelemetryQueue` usa `TripleBuffer` internamente; produção usa `TelemetryBuffer`/`LockFreeParamQueue` em `sonix-desktop/src/audio.rs` e `MidiRouter` direto após AUD-17). |
| `FmvComponents`, `PassiveToneStack` | `tonestack.rs` | Spec 003 / 031 | Subtipos de rede passiva usados internamente por `WhiteBoxAmpProcessor`; **Spec 039** ampliará validação e uso por modelo (AB763 / AC30 TB). |
| `TunerMode`, `TunerTemperament`, `PolyphonicStringStatus` | `tuner.rs` | Spec 008 | Modos polifônico/temperamentos do afinador aguardando ligação completa ao fluxo de áudio/API/UI pela **Spec 043**. |
| `ClippingDiode` | `fx/drive.rs` | Spec 006 / 023 | Subtipo interno de diodos de ceifamento em `DriveProcessor`; **Spec 041** traz oversampling/anti-aliasing e exposição completa. |
| `NoteDivision` | `fx/modulation.rs` | Spec 007 | Divisões rítmicas de BPM preparadas na biblioteca; ligação ao sincronismo em produção pelas **Specs 040 / 045**. |
| `PitchModel`, `PitchKey`, `ScaleType`, `PitchProcessor` | `fx/pitch.rs` | Spec 007 | Stub de biblioteca criado na Spec 007 sem processamento em produção; implementação real do pitch shifter monofônico e harmonizador diatônico pela **Spec 042**. |

## `crates/sonix-core/src`

| Item público | Arquivo | Origem | Motivo / Spec de retomada |
|---|---|---|---|
| `FootswitchMode`, `FootswitchUnit`, `FootswitchArray` | `footswitch.rs` | Spec 009 | Modos dinâmicos e etiquetas dos 8 footswitches ainda sem integração ao runtime/UI; ligação completa pela **Spec 045**. |
| `ModifierSource`, `ModifierBinding`, `ModifierMatrix` | `modifiers.rs` | Spec 009 | Matriz universal de modifiers (clique direito em knob, EXP/LFO/ENV/MIDI CC); ligação real pela **Spec 045** e **Spec 036**. |
| `PresetSchemaValidationError`, `PackageIntegrityError`, `SceneDto`, `BlockDto`, `ModifierDto`, `SforgePackage` | `persistence.rs` | Spec 010 / 046 | Subtipos do schema de preset e empacotamento `.sforgepkg`; endurecidos na **Spec 046** e integrados à hierarquia completa de UI/cenas pela **Spec 045**. |
| `CoreError`, `BlockChannel`, `BlockChannelState`, `BlockNode`, `Scene`, `Setlist` | `preset.rs` | Spec 009 | Hierarquia completa `Setlist → Preset → 8 Scenes → 4 Canais` aguardando ligação em produção pela **Spec 045**. |
