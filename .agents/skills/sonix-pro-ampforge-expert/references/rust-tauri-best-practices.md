# SONIX PRO // AMPFORGE™ — Boas Práticas de Engenharia em Rust e Tauri v2

Este documento estabelece os padrões obrigatórios de código Rust (núcleo DSP e host) e de integração desktop Tauri v2 para o SONIX PRO // AMPFORGE™.

---

## Parte 1 — Engenharia de Software em Rust (Áudio e Tempo Real)

### 1.1 Ownership, Borrowing e Estado por Instância
- **Estado 100% por instância:** filtros, delays, envelopes, memória de válvulas, estados ADAA e buffers de trabalho pertencem à struct do processador (`&mut self`). Nunca use `static mut` ou estado global compartilhado para áudio.
- **Empréstimo explícito de buffers:** prefira assinaturas baseadas em slices (`&[f32]`, `&mut [f32]`) ou views tipadas de bloco (`AudioBlockRef` / `AudioBlockMut`) sem copiar nem alocar.
- **Ciclo de vida em fases com fronteira de thread explícita:**
  1. `prepare(&mut self, spec: AudioSpec) -> Result<(), DspError>` — **executado na thread de controle (off-RT)**: pode alocar buffers dimensionados para `spec.max_sample_rate` / `spec.max_block_size` e pré-computar tabelas.
  2. `set_param(&mut self, id: ParamId, val: ParamValue) -> Result<(), ParamError>` — **executado na thread RT** (via drenagem da `SpscBuffer` no início do callback): atualiza escalares/smoothers e coeficientes locais em tempo $O(1)$ com **zero alocação**. Mudanças estruturais pesadas (ex.: nova IR, nova topologia DAG, novo processador boxed) nunca usam `set_param`; são construídas off-RT na thread de controle e enviadas prontas por `SpscOwnedQueue`.
  3. `process(&mut self, ctx: &ProcessContext, input: &AudioBlockRef<'_>, output: &mut AudioBlockMut<'_>)` — **executado na thread RT**: zero-alloc, zero-dealloc, zero-lock, tempo determinístico.
  4. `reset(&mut self)` — **RT-safe em $O(1)$**: zera registradores de filtros ou invalida caudas via contador monotônico (`samples_since_clear = 0`), sem executar `memset` de megabytes no callback.

### 1.2 Tipos Fortes, Contratos e Traits
- **Traits enxutos onde reduzem acoplamento:** use traits (`AudioProcessor`, `AudioDeviceEnumerator`) na fronteira do grafo/host; evite abstrações genéricas profundas que ocultem diferenças topológicas entre equipamentos.
- **Newtypes e unidades físicas explícitas:** nunca confunda `dB` com ganho linear, `Hz` com radianos, `ms` com amostras ou `0..10` com `0..1`. Prefira tipos explícitos (`SampleRateHz`, `GainDb`, `FreqHz`, `TimeMs`, `ParamId`) ou sufixos rigorosos de unidade (`_db`, `_hz`, `_ms`, `_ohms`, `_ma`, `_v`).
- **Enums para estados e modelos discretos:** substitua strings soltas no núcleo DSP por `enum` exaustivo (`AmpModelId`, `DriveModel`, `GateState`, `AudioMode`). Conversões de/para string ficam isoladas na borda de API/serialização.

### 1.3 Tratamento de Erros e Proibição de Panic
- Use `Result<T, E>` com erros tipados derivados via `thiserror` (`DspEngineError`, `InvalidAudioConfigError`, `AudioDriverError`, `InvalidImpulseResponseError`, `CoreError`).
- **Proibido em código operacional e de áudio:** `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()` e indexação direta sem verificação de limite quando o tamanho vier de entrada externa.
- Em perfil `--release`, `Cargo.toml` define `panic = "abort"`: um panic encerra o processo inteiro imediatamente.

### 1.4 Concorrência, Atomics e Lock-Free
- **Memory Ordering:** use `Ordering::Acquire` / `Ordering::Release` em índices de filas SPSC e publicação de ponteiros/versões; `Ordering::Relaxed` apenas para telemetria/contadores independentes (`adc_clips`, picos VU). Documente o motivo de cada `Ordering` não trivial.
- **Reclamation off-RT (`RetiredReaper`):** nunca deixe o último `Arc::drop` ou `Box::drop` acontecer na thread de áudio, e nunca use `std::mem::forget` para descartar recursos antigos caso um slot único esteja ocupado — use fila SPSC com capacidade suficiente para devolver todo recurso aposentado à thread de controle.

### 1.5 Política de `unsafe`
- Prefira Rust seguro por padrão.
- `unsafe` só é aceito quando estritamente necessário (ex.: buffer SPSC com `UnsafeCell<MaybeUninit<T>>` em `queue.rs`, FFI de driver ou SIMD explícito medido em benchmark).
- Todo bloco `unsafe` exige: escopo mínimo, abstração segura encapsulada, comentário `// SAFETY:` enumerando as pré-condições provadas e cobertura por testes de concorrência.

### 1.6 Testes, Lints, Benchmarks e Dependências
- **Qualidade contínua:** `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --check`.
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
