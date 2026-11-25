# SONIX PRO // AMPFORGE™ — Boas Práticas de Engenharia em Rust (Doc Oficial) e Tauri v2

Este documento estabelece os padrões obrigatórios de código Rust (núcleo DSP e host) e de integração desktop Tauri v2 para o SONIX PRO // AMPFORGE™. Ele atua em conjunto com **[`rust-best-practices`](../../rust-best-practices/SKILL.md)**, fundamentada na documentação oficial estável do Rust (`https://doc.rust-lang.org/stable/`).

---

## Parte 1 — Engenharia de Software em Rust (Doc Oficial + Áudio em Tempo Real)

### 1.1 Ownership, Borrowing e Estado por Instância (Rust Book)
- **Estado 100% por instância:** filtros, delays, envelopes, memória de válvulas, estados ADAA e buffers de trabalho pertencem à struct do processador (`&mut self`). Nunca use `static mut` ou estado global compartilhado para áudio.
- **Empréstimo explícito de buffers:** prefira assinaturas baseadas em slices (`&[f32]`, `&mut [f32]`) ou views tipadas de bloco (`AudioBlockRef` / `AudioBlockMut`) sem copiar nem alocar. Use `chunks_exact` e fatias de tamanho provado igual para que o LLVM elimine bounds checks e auto-vetorize.
- **Processamento real em bloco:** nunca invoque um processador de bloco com fatia de 1 amostra (`&[x]`) dentro de um laço sample-by-sample, nem reconstrua FIR/FFT dentro de `process()`.
- **Ciclo de vida em fases com fronteira de thread explícita:**
  1. `prepare(&mut self, spec: AudioSpec) -> Result<(), DspError>` — **executado na thread de controle (off-RT)**: pode alocar buffers dimensionados para `spec.max_sample_rate` / `spec.max_block_size` e pré-computar tabelas/FIRs.
  2. `set_param(&mut self, id: ParamId, val: ParamValue) -> Result<(), ParamError>` — **executado na thread RT** (via drenagem da fila SPSC no início do callback): atualiza escalares/smoothers e coeficientes locais em tempo $O(1)$ com **zero alocação**. Mudanças estruturais pesadas (ex.: nova IR, nova topologia DAG, novo processador boxed) nunca usam `set_param`; são construídas off-RT na thread de controle e enviadas prontas por fila lock-free.
  3. `process(&mut self, ctx: &ProcessContext, input: &AudioBlockRef<'_>, output: &mut AudioBlockMut<'_>)` — **executado na thread RT**: zero-alloc, zero-dealloc, zero-lock, tempo determinístico, `DenormalGuard` (FTZ/DAZ) e guarda `.is_finite()`.
  4. `reset(&mut self)` — **RT-safe em $O(1)$**: zera registradores de filtros ou invalida caudas via contador monotônico (`samples_since_clear = 0`), sem executar `memset` de megabytes no callback.

### 1.2 Tipos Fortes, Newtypes e Contratos (Rust API Guidelines)
- **Traits enxutos onde reduzem acoplamento:** use traits (`AudioProcessor`, `AudioDeviceEnumerator`) na fronteira do grafo/host; evite abstrações genéricas profundas que ocultem diferenças topológicas entre equipamentos.
- **Newtypes e unidades físicas explícitas (`C-NEWTYPE`):** nunca confunda `Volts` com `FullScale`, `dB` com ganho linear, `Hz` com radianos, `ms` com amostras, `PreBoost` com `PostBoost`, nem `0..10` com `0..1`. Prefira newtypes `#[repr(transparent)]` (`Volts`, `Db`, `FreqHz`, `TimeMs`, `ParamId`) ou sufixos rigorosos de unidade (`_db`, `_hz`, `_ms`, `_ohms`, `_ma`, `_v`).
- **Enums exaustivos (`C-CUSTOM-TYPE`):** substitua strings soltas no núcleo DSP por `enum` exaustivo (`AmpModelId`, `DriveModel`, `GateState`, `AudioMode`). Evite `_ => ()` silencioso que esconde parâmetros ou modelos recém-adicionados.

### 1.3 Tratamento de Erros e Política Real de `panic` (Book Cap. 9 + Cargo Book)
- Use `Result<T, E>` com erros tipados derivados via `thiserror` (`DspEngineError`, `InvalidAudioConfigError`, `AudioDriverError`, `InvalidImpulseResponseError`, `CoreError`).
- **Proibido em código operacional e de áudio:** `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()` e indexação direta sem prova de limite quando o índice vier de entrada externa.
- **Perfil `--release` e `panic`:** o `Cargo.toml` raiz define `panic = "unwind"` (com `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`) porque o host de áudio (`audio.rs`) usa `std::panic::catch_unwind` para converter qualquer panic inesperado em *fault-mute* seguro sem derrubar o app desktop nem a DAW hospedeira do plugin. **Não altere para `panic = "abort"` enquanto existir `catch_unwind` como barreira de segurança.**

### 1.4 Concorrência, Atomics e Lock-Free (Rustonomicon & Reference)
- **Dono exclusivo do DSP na thread RT:** proibido `Mutex::lock()` ou `try_lock()` no callback de áudio com fallback que copia entrada crua (`mono_out.copy_from_slice(mono_in)`). A comunicação controle↔áudio deve ser 100% lock-free (SPSC + triple-buffer/atômicos).
- **Memory Ordering (modelo C++20):** use `Ordering::Acquire` / `Ordering::Release` em índices de filas SPSC e publicação de ponteiros/versões; `Ordering::Relaxed` apenas para telemetria/contadores independentes (`adc_clips`, picos VU). Documente o motivo de cada `Ordering` não trivial.
- **Soundness SPSC (`MaybeUninit` / `UnsafeCell`):** **proibido** `push_overwrite` onde o produtor avança `tail` via CAS enquanto o consumidor pode estar executando `assume_init_read()` no mesmo slot (data race = Undefined Behavior). Use crates auditadas (`ringbuf`, `rtrb`) ou triple-buffer/seqlock verificado no Miri.
- **Reclamation off-RT (`RetiredReaper`):** nunca deixe o último `Arc::drop` ou `Box::drop` acontecer na thread de áudio, e nunca use `std::mem::forget` para descartar recursos antigos caso um slot único esteja ocupado — use fila SPSC com capacidade suficiente para devolver todo recurso aposentado à thread de controle.

### 1.5 Política de `unsafe` e Persistência Atômica
- Prefira Rust seguro por padrão; ative `unsafe_op_in_unsafe_fn = "deny"` e `clippy::undocumented_unsafe_blocks = "deny"`.
- `unsafe` só é aceito quando estritamente necessário (FFI de driver ou SIMD explícito medido em benchmark, respeitando `#[cfg(target_feature)]` e alinhamento de memória).
- Todo bloco `unsafe` e `unsafe impl Send/Sync` exige: escopo mínimo, abstração segura encapsulada, comentário `// SAFETY:` enumerando todas as pré-condições provadas e validação com testes concorrentes / Miri.
- **Persistência atômica no Windows:** nunca execute `remove_file(target)` seguido de `rename(tmp, target)` (janela de perda de dados + colisão de `tmp` por PID); grave arquivo temporário único no mesmo diretório, execute `sync_all()` e substitua atomicamente.

### 1.6 `[workspace.lints]`, Clippy, Testes, Benchmarks e Rust 2024
- **Workspace lints (Cargo Book):** centralizar `[workspace.lints.rust]` e `[workspace.lints.clippy]` (`correctness = "deny"`, `suspicious = "warn"`, `perf = "warn"`, `dead_code = "warn"`, `undocumented_unsafe_blocks = "deny"`, `unwrap_used = "warn"`, `indexing_slicing = "warn"`) e herdar com `[lints] workspace = true`.
- **Qualidade contínua:** `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets --locked`, `cargo fmt --check`.
- **Prontidão Rust 2024:** `unsafe` explícito em operações dentro de `unsafe fn`, evitar `gen` como identificador, atenção às regras de captura RPIT `use<...>` e `unsafe extern`.
- **Governança de crates:** não adicionar dependências sem necessidade comprovada; fixar versões no `Cargo.lock` e conferir compatibilidade com a licença do workspace (`CC-BY-NC-4.0`).

---

## Parte 2 — Engenharia Desktop com Tauri v2 (`2.12.x`)

Baseado na documentação oficial do Tauri v2 (https://v2.tauri.app/) e na arquitetura real de `apps/sonix-app/`.

### 2.1 Separação de Camadas
```text
UI (TypeScript)  ──IPC──▶  commands.rs / streams.rs (Tauri v2)
                                      │
                                      ▼
                           Control Plane / Services (sonix-desktop::api)
                                      │
                                      ▼
                           Domain / DSP Pure Core (sonix-core + ampforge-dsp)
```
- `ampforge-dsp` e `sonix-core` **nunca** importam `tauri` nem tipos da UI.
- `apps/sonix-app/src-tauri` é uma casca fina: inicializa o runtime, registra plugins/capabilities, expõe comandos IPC tipados e gerencia o ciclo de vida da janela.

### 2.2 As 7 Regras Fundamentais do Tauri v2 no AMPFORGE
1. **Comandos Tauri não implementam DSP:** `commands.rs` valida payload/contrato e delega a `api::dispatch` ou `AudioRuntimeControl`.
2. **A interface não controla diretamente o motor de áudio:** a UI consome descritores (`controls.registry.ts`) e invoca comandos via `SonixTransport` (`transport.ts`) + `ParamCoalescer`.
3. **Fronteira IPC validada e honesta:** toda entrada do frontend é não confiável. Validar faixas, enums, taxas (`44100|48000|88200|96000`), tamanhos de buffer (`32..=2048`), extensões/tamanhos de arquivos e caminhos. Padronizar prefixos de erro (`E_VALIDATION:`, `E_ENGINE:`).
4. **Operações pesadas não bloqueiam a interface:** usar comandos `async` ou threads dedicadas para I/O, carga de IR, calibração e reabertura de streams de áudio.
5. **Telemetria sob taxa controlada (`tauri::ipc::Channel`):** usar `Channel<T>` dedicado (`TelemetryFrame`, `TunerFrame`, `ScopeFrame` em `streams.rs`) a ~30 Hz quando visível e **0 Hz quando minimizado/oculto**. Jamais transmitir áudio amostra-a-amostra via IPC.
6. **Capabilities e menor privilégio:** declarar permissões mínimas em `src-tauri/capabilities/main.json` vinculadas apenas à janela `["main"]` e manter CSP estrito em `tauri.conf.json` com `custom-protocol` no release (zero servidor HTTP).
7. **Sem estado otimista corrompido:** se um comando IPC falhar, o backend executa rollback atômico e a UI reconcilia seu estado com o estado real do backend.
