# SONIX PRO // AMPFORGE™ — Boas Práticas de Engenharia em Rust e Tauri v2

Este documento estabelece os padrões obrigatórios de código Rust (núcleo DSP e host) e de integração desktop Tauri v2 para o SONIX PRO // AMPFORGE™.

---

## Parte 1 — Engenharia de Software em Rust (Áudio e Tempo Real)

### 1.1 Ownership, Borrowing e Estado por Instância
- **Estado 100% por instância:** filtros, delays, envelopes, memória de válvulas, estados ADAA e buffers de trabalho pertencem à struct do processador (`&mut self`). Nunca use `static mut` ou estado global compartilhado para áudio.
- **Empréstimo explícito de buffers:** prefira assinaturas baseadas em slices (`&[f32]`, `&mut [f32]`) ou views tipadas de bloco (`AudioBufferRef` / `AudioBufferMut`) sem copiar nem alocar.
- **Ciclo de vida em duas fases:**
  1. `prepare(spec: AudioSpec)` (thread de controle, pode alocar e pré-computar tabelas).
  2. `process(ctx, input, output)` (thread RT, zero-alloc, zero-lock, tempo determinístico).
  3. `reset()` (zera estados de filtro/delay em O(1) ou fora do caminho crítico).

### 1.2 Tipos Fortes, Contratos e Traits
- **Traits enxutos onde reduzem acoplamento:** use traits (`AudioProcessor`, `AudioDeviceEnumerator`) na fronteira do grafo/host; evite hierarquias genéricas excessivas com dezenas de parâmetros de tipo que dificultem manutenção e inlining.
- **Newtypes e unidades físicas explícitas:** nunca confunda `dB` com ganho linear, `Hz` com radianos, `ms` com amostras ou `0..10` com `0..1`. Prefira tipos explícitos (`SampleRate`, `GainDb`, `FreqHz`, `TimeMs`, `ParamId`) ou sufixos rigorosos de unidade em todos os campos e métodos (`_db`, `_hz`, `_ms`, `_ohms`, `_ma`).
- **Enums para estados e modelos discretos:** substitua strings soltas no núcleo DSP por `enum` exaustivo (`AmpModelId`, `DriveModel`, `GateState`, `AudioMode`). Conversões de/para string (`from_str_name` / `as_str`) ficam isoladas na borda de API/serialização.

### 1.3 Tratamento de Erros e Proibição de Panic
- Use `Result<T, E>` com erros tipados derivados via `thiserror` (`DspEngineError`, `InvalidAudioConfigError`, `AudioDriverError`, `InvalidImpulseResponseError`, `CoreError`).
- **Proibido em código operacional e de áudio:** `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()` e indexação direta sem verificação de limite quando o tamanho vier de entrada externa.
- Use `debug_assert_eq!` apenas para invariantes internas já garantidas pelo chamador; na fronteira pública, limite com `.min(...)`, `.clamp(...)` ou retorne `Err(...)`.
- Em perfil `--release`, `Cargo.toml` define `panic = "abort"`: um panic encerra o processo inteiro imediatamente.

### 1.4 Concorrência, Atomics e Lock-Free
- **Separação estrita de threads:**
  - Thread de áudio (RT): lê apenas atomics, filas SPSC lock-free (`SpscBuffer`, `SpscOwnedQueue`) e seu estado exclusivo.
  - Thread de controle / IPC: executa validação, I/O, carregamento de IR/presets, compilação de DAG e recolhimento de memória (`RetiredReaper`).
- **Memory Ordering:** use `Ordering::Acquire` / `Ordering::Release` em índices de filas SPSC e publicação de ponteiros/versões; `Ordering::Relaxed` apenas para telemetria/contadores independentes (ex.: `adc_clips`, picos VU). Documente o motivo de cada `Ordering` não trivial.
- **Reclamation off-RT:** nunca deixe o último `Arc::drop` ou `Box::drop` acontecer na thread de áudio; transfira o recurso aposentado de volta para uma fila SPSC de retorno que a thread de controle drena e libera.

### 1.5 Política de `unsafe`
- Prefira Rust seguro por padrão.
- `unsafe` só é aceito quando estritamente necessário (ex.: buffer SPSC com `UnsafeCell<MaybeUninit<T>>` em `queue.rs`, FFI de driver ou SIMD explícito medido em benchmark).
- Todo bloco `unsafe` exige: escopo mínimo, abstração segura encapsulada, comentário `// SAFETY:` enumerando as pré-condições provadas e cobertura por testes de concorrência/Miri quando aplicável.

### 1.6 Testes, Lints, Benchmarks e Dependências
- **Qualidade contínua:** `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --check`.
- **Pirâmide de testes no AMPFORGE:**
  - *Unitários:* funções matemáticas puras (KCL, bilinear, ADAA, biquads, taper).
  - *Contrato SDD (`contract_NNN_*.rs`):* 1 teste por `CA-n`, citando o CA no nome, falhando antes da implementação (*red → green*).
  - *Invariantes medidas:* alocador global de contagem na thread RT (Spec 037), medição DFT de stopband/aliasing (Specs 039/041/044), testes em múltiplas taxas (44,1/48/96 kHz) e entradas `NaN`/`Inf`.
  - *Benchmarks:* Criterion em `--release` para comparar custo por amostra/bloco antes de otimizações.
- **Governança de crates:** não adicionar dependências sem necessidade comprovada; fixar versões no `Cargo.lock` e conferir compatibilidade com a licença do workspace (`CC-BY-NC-4.0`) e restrições de redistribuição (especialmente SDKs proprietários como ASIO).

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
1. **Comandos Tauri não implementam DSP:** `commands.rs` valida payload/contrato e delega a `api::dispatch` ou `AudioRuntimeControl`. Zero matemática de filtro ou manipulação de amostras dentro de `#[tauri::command]`.
2. **A interface não controla diretamente o motor de áudio:** a UI consome descritores (`controls.registry.ts`) e invoca comandos via `SonixTransport` (`transport.ts`), com `ParamCoalescer` limitando escritas a no máximo 1 `set_params` por frame de animação.
3. **Fronteira IPC validada e honesta:** toda entrada do frontend é não confiável. Validar faixas, enums, taxas (`44100|48000|88200|96000`), tamanhos de buffer (`32..=2048`), extensões/tamanhos de arquivos e caminhos (anti path-traversal). Padronizar prefixos de erro (`E_VALIDATION:` para 4xx, `E_ENGINE:` para 5xx) para a UI distinguir erro de domínio de queda real do IPC.
4. **Operações pesadas não bloqueiam a interface:** usar comandos `async` ou threads dedicadas para I/O, carga de IR, calibração e reabertura de streams de áudio; manter bloqueios de `Mutex` em `State<'_, AppRuntime>` curtos e nunca segurar lock durante I/O demorado.
5. **Telemetria sob taxa controlada (`tauri::ipc::Channel`):** usar `Channel<T>` dedicado (`TelemetryFrame`, `TunerFrame`, `ScopeFrame` em `streams.rs`) com sequência monotônica `seq`, cadência de ~30 Hz (33 ms) quando visível e **0 Hz quando minimizado/oculto**. Jamais fazer streaming de áudio bruto amostra-a-amostra via IPC.
6. **Capabilities e menor privilégio:**
   - Todo acesso da WebView é negado por padrão no Tauri v2.
   - Declarar permissões mínimas em `src-tauri/capabilities/main.json` vinculadas apenas à janela `["main"]` (ex.: `core:window:allow-*`, `dialog:allow-open/save`, `fs:allow-read-*`, sem permissões globais amplas nem shell arbitrário).
   - Manter CSP estrito em `tauri.conf.json` (`default-src 'self'`, `connect-src ipc: http://ipc.localhost`), sem acesso a rede externa nem servidor HTTP no build release (`--features custom-protocol`).
7. **Sem estado otimista corrompido:** se um comando IPC falhar (ex.: `set_audio_device` rejeitado pelo driver ou `set_params` com valor inválido), o backend executa rollback atômico do stream anterior e a UI reconcilia seu estado com o estado real retornado pelo backend, exibindo aviso claro sem mentir que a operação foi aplicada.

### 2.3 Ciclo de Vida e Gerenciamento de Estado
- **Managed State:** encapsular o estado em `AppRuntime` (`State<'_, AppRuntime>`) com guarda singleton para impedir dois hosts de áudio simultâneos (`ensure_single_audio_host` + `tauri-plugin-single-instance`).
- **Window Events:** conectar `WindowEvent::Resized` (minimizado → 0 Hz), `Focused` e `Destroyed` ao `StreamCoordinator` e garantir `Drop` ordenado (`RuntimeKeepAlive`: para thread fallback e encerra streams `cpal` antes do encerramento do processo).
- **Segurança no Startup (Spec 050):** iniciar em Standby seguro quando não houver perfil de calibração salvo, evitando microfonia ou chiado ao abrir o aplicativo.
- **DTOs explícitos:** evoluir comandos de `serde_json::Value` genérico para structs Rust/TypeScript fortemente tipadas com `#[derive(Serialize, Deserialize)]` e contratos de schema testados em `tests/fixtures/ipc-schemas/`.
