---
name: rust-best-practices
description: Boas práticas oficiais de engenharia em Rust baseadas na documentação estável em https://doc.rust-lang.org/stable/ (The Rust Book, Reference, Rustonomicon, Cargo Book, Clippy, Edition Guide 2024, std/core e Rust API Guidelines), com foco em segurança de memória, concorrência lock-free, tipagem forte, ergonomia, performance determinística e áudio/DSP em tempo real. Use sempre que escrever, revisar, otimizar, refatorar ou depurar código Rust — especialmente em conjunto com a skill sonix-pro-ampforge-expert.
---

# Rust Best Practices — Guia Oficial & Tempo Real (`doc.rust-lang.org/stable`)

Esta skill consolida as práticas oficiais da documentação estável do Rust (`https://doc.rust-lang.org/stable/`, toolchain atual do workspace `rustc 1.99+`) integradas às restrições de sistemas críticos, áudio de baixíssima latência e DSP determinístico do **SONIX PRO // AMPFORGE™**.

> **Integração:** Em qualquer tarefa de DSP, motor de áudio, host WASAPI/ASIO, plugin VST3/CLAP ou backend Tauri deste repositório, aplique esta skill em conjunto com [sonix-pro-ampforge-expert](../sonix-pro-ampforge-expert/SKILL.md).

---

## 1. Fontes Oficiais de Referência (`doc.rust-lang.org/stable`)

| Fonte Oficial | URL | O que governa |
|---|---|---|
| **The Rust Programming Language (Book)** | `https://doc.rust-lang.org/stable/book/` | Ownership, borrowing, lifetimes, `Result` vs `panic!`, traits, generics, newtype pattern, testes |
| **The Rust Reference** | `https://doc.rust-lang.org/stable/reference/` | Comportamento Indefinido (UB), aliasing (`&T`/`&mut T`), semântica IEEE 754 de `f32`/`f64`, atributos `#[inline]` e `#[target_feature]` |
| **The Rustonomicon** | `https://doc.rust-lang.org/stable/nomicon/` | `unsafe`, data races vs race conditions, `Send`/`Sync`, `UnsafeCell`, atomics (modelo de memória C++20), `MaybeUninit`, FFI |
| **The Cargo Book** | `https://doc.rust-lang.org/stable/cargo/` | `[workspace.lints]`, `[profile.*]` (`opt-level`, `lto`, `codegen-units`, `panic`), features, `--locked` |
| **Clippy Lints** | `https://doc.rust-lang.org/stable/clippy/` | Grupos `correctness`, `suspicious`, `perf`, `pedantic` e seleção cirúrgica de `restriction` |
| **The Edition Guide (2024)** | `https://doc.rust-lang.org/stable/edition-guide/` | RPIT `use<...>`, `unsafe extern`, atributos `#[unsafe(...)]`, `unsafe_op_in_unsafe_fn`, palavra reservada `gen` |
| **Standard Library (`std`/`core`)** | `https://doc.rust-lang.org/stable/std/` | `std::sync::atomic`, `std::arch`, `f32`/`f64` (`mul_add`, `is_finite`, `total_cmp`), `slice::chunks_exact`, `MaybeUninit` |
| **Rust API Guidelines** | `https://rust-lang.github.io/api-guidelines/` | `C-NEWTYPE`, `C-CUSTOM-TYPE`, `C-COMMON-TRAITS`, `C-GOOD-ERR`, `C-VALIDATE`, `C-SEND-SYNC` |

---

## 2. Tipagem Forte, Modelagem de Domínio e Ergonomia (Book + API Guidelines)

1. **Newtypes para Unidades Físicas e Escalas (`C-NEWTYPE`):**
   - Nunca passe `f32`/`f64` "nu" quando houver risco de misturar escalas (ex.: `Volts` vs digital `FullScale`, `Decibels` vs ganho linear, `Hz` vs frequência normalizada, `Samples` vs `Milliseconds`, sinal `PreBoost` vs `PostBoost`).
   - Envolva valores críticos em tuple structs `#[repr(transparent)]` (`struct Volts(f64)`, `struct Db(f32)`, `struct Hz(f64)`) — **custo zero em runtime**, mas elimina silenciosamente bugs catastróficos de escala (como aplicar `lf_flux` na escala `power_sat` dentro de `power_out` já escalado por `0.30`, ou confundir `1.0 FS` com `1.0 V` na entrada de um pedal).
2. **Estados Inválidos Irrepresentáveis (`C-CUSTOM-TYPE`):**
   - Prefira `enum` tipados em vez de `bool` ambíguos ou `String` livres no domínio interno.
   - Use **exaustividade do `match`** (evite `_ => ()` silencioso quando adicionar novas variantes de modelos/parâmetros!). Se um novo variante de parâmetro ou modelo for adicionado, o compilador deve obrigar a tratá-lo em todos os pontos do motor.
3. **Nomenclatura e Traits Idiomáticos (`C-CASE`, `C-CONV`, `C-GETTER`, `C-COMMON-TRAITS`):**
   - Getters não usam prefixo `get_` (`fn sample_rate(&self) -> f32`).
   - Conversões seguem `as_` (barata, emprestada), `to_` (aloca/copia), `into_` (consome `self`) e traits padrão `From`/`TryFrom`/`AsRef`.
   - Derivar `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` sempre que aplicável.
4. **Ownership e Zero Cópias Desnecessárias:**
   - Prefira slices `&[T]` / `&mut [T]` em assinaturas de DSP e utilitários, nunca `&Vec<T>` ou `&String`.
   - Evite `.clone()` defensivo em laços ou hot paths; passe referências com lifetimes explícitos ou índices/handles leves.

---

## 3. Tratamento de Erros: `Result` vs `panic!` (Book Cap. 9 + Clippy)

1. **`Result<T, E>` para toda falha recuperável; `panic!` apenas para bug irrecuperável de invariante:**
   - Em código de produção (especialmente thread de áudio, host de driver, parser de preset/IR/WAV, IPC), **proíba `.unwrap()` e `.expect()`** (`clippy::unwrap_used`, `clippy::expect_used`).
   - Em bibliotecas (`ampforge-dsp`, `sonix-core`), defina enums de erro estruturados com `thiserror` implementando `std::error::Error + Display + Debug` (`C-GOOD-ERR`).
   - Nunca silencie erros com `let _ = ...` em operações com efeitos críticos (IO, persistência, enfileiramento de parâmetros, sincronização de estado). Se o descarte for intencional, documente o motivo.
2. **Estratégia `panic = "unwind"` vs `panic = "abort"` (Cargo Book):**
   - `panic = "abort"` reduz binário e elimina tabelas de unwind, **mas torna `std::panic::catch_unwind` inoperante** (qualquer panic aborta o processo inteiro imediatamente, derrubando a DAW em um plugin VST3/CLAP ou fechando o app desktop sem silenciar a saída).
   - Se o host de áudio ou plugin usa `catch_unwind` para isolar falhas e colocar o motor em *fault-mute* seguro (como este workspace em `Cargo.toml`), mantenha **`panic = "unwind"`** e garanta que o hot path nunca entre em pânico (sem indexação cega `slice[i]`, sem divisão inteira por zero, sem `assert!` em dados vindos do usuário).
   - Documente sempre a escolha real do `Cargo.toml` — nunca assuma `abort` se o perfil usa `unwind`.

---

## 4. Segurança de Memória, `unsafe`, Atomics e Concorrência (Reference + Rustonomicon)

### 4.1 Regras Absolutas contra Undefined Behavior (UB)
Conforme [The Rust Reference — Behavior Considered Undefined](https://doc.rust-lang.org/stable/reference/behavior-considered-undefined.html):
1. **Data Races são sempre UB:** Dois acessos concorrentes ao mesmo endereço onde ao menos um é escrita e ao menos um é não-atômico = **UB imediato**, mesmo para inteiros ou `f32`.
2. **Aliasing de Referências:** Nunca crie `&mut T` enquanto existir qualquer outro `&T` ou `&mut T` ativo para o mesmo objeto. Mutação através de `&T` só é legal dentro de `UnsafeCell<T>` devidamente sincronizado.
3. **Memória Não Inicializada (`MaybeUninit<T>`):** Nunca leia `assume_init()`/`assume_init_read()` sem prova estrita de inicialização prévia e posse exclusiva daquele slot.
4. **`Send` / `Sync` manuais são `unsafe impl`:** Só implemente `unsafe impl Sync` quando puder provar ausência total de data races em **todos** os métodos `&self` sob intercalação arbitrária de threads.
5. **Instruções SIMD / `target_feature`:** Chamar intrínsecos `std::arch` sem garantir suporte de CPU (via `#[cfg(target_feature = "...")]` ou `is_x86_feature_detected!()` + função `#[target_feature(enable = "...")]`) e alinhamento exigido (ex.: `_mm_load_ps` exige alinhamento 16 bytes; caso contrário usar `_mm_loadu_ps`) é **UB**.

### 4.2 Disciplina de `unsafe` e `// SAFETY:`
- **Todo bloco `unsafe { ... }` e todo `unsafe impl` exige um comentário `// SAFETY:` imediatamente acima** provando todas as pré-condições exigidas pela operação.
- Ative `unsafe_op_in_unsafe_fn = "deny"` (padrão no Rust 2024): mesmo dentro de `unsafe fn`, encapsule apenas a operação insegura mínima em um bloco `unsafe { ... }`.
- **Nunca escreva lock-free caseiro com `unsafe` quando existir crate auditada (`ringbuf`, `rtrb`, `arc-swap`, `crossbeam`):**
  - *Armadilha real evitada:* Em um ring buffer SPSC com `UnsafeCell<MaybeUninit<T>>`, implementar `push_overwrite` onde o produtor avança `tail` via CAS enquanto o consumidor pode estar lendo `buffer[tail]` em `pop()` gera **data race / UB clássico**. Se precisar de "último valor vence" (ex.: telemetria/osciloscópio), use `seqlock`/triple-buffer, sobrescrita baseada em slots atômicos (`AtomicU32` para `f32::to_bits`), ou descarte controlado sem tocar em slot em leitura pelo consumidor.
- Valide qualquer estrutura `unsafe` com **Miri** (`cargo +nightly miri test`) e testes de estresse concorrente.

### 4.3 Modelo de Memória Atômico (C++20 / Rustonomicon)
- `Ordering::Relaxed`: contadores independentes, flags de UI/telemetria sem dados dependentes (ex.: `input_peak_bits`, `visible`).
- `Ordering::Release` (escritor) + `Ordering::Acquire` (leitor): publicação de dados em ring buffers SPSC, snapshots de coeficientes, troca de buffers pré-alocados. Garante *happens-before* de toda escrita anterior ao `Release` para qualquer leitura após o `Acquire`.
- `Ordering::AcqRel`: operações RMW (`fetch_add`, `compare_exchange`) que consomem e publicam estado simultaneamente.
- `Ordering::SeqCst`: raramente necessário; use apenas quando houver exigência de ordem total global entre múltiplas variáveis atômicas independentes.

---

## 5. Áudio em Tempo Real (Hard Real-Time) & Performance Determinística

### 5.1 Contrato da Thread de Áudio (Inviolável)
No callback de áudio (`process()`):
- **ZERO alocações/desalocações na heap** (`Vec::push`, `resize`, `Box::new`, `String`, `format!`, `drop` de buffers grandes).
- **ZERO locks bloqueantes** (`Mutex::lock()`, `RwLock`, `Condvar`, `thread::sleep`, syscalls, IO de arquivo, logging).
- **ZERO `Mutex::try_lock()` no caminho de áudio com fallback que vaza áudio cru!** O motor DSP deve pertencer **exclusivamente** à thread de áudio (ou ter fronteira de comando/estado 100% lock-free via SPSC + triple-buffer/atomic snapshots). A UI/API nunca trava o motor DSP; apenas envia comandos atômicos/SPSC e lê telemetria atômica/SPSC.
- **ZERO recálculo pesado de filtros/FFTs dentro de `process()`:** Construção de FIR de fase mínima, carregamento/particionamento de IR e redimensionamento de buffers ocorrem em thread de fundo e são trocados no callback por troca de ponteiro/índice lock-free com descarte diferido (*deferred reclamation*) na thread não-RT.

### 5.2 Aritmética de Ponto Flutuante, Denormais e Vetorização
1. **Proteção contra `NaN`/`Inf` e Denormais:**
   - Habilite FTZ/DAZ no hardware (`DenormalGuard`) no início de cada callback de áudio e sanitize entradas/estados críticos com `.is_finite()`.
   - Nunca compare floats por igualdade exata sem intenção explícita (`clippy::float_cmp`); para ordenação use `f32::total_cmp` / `f64::total_cmp`.
2. **Precisão e Estabilidade Numérica:**
   - Use `f64` para coeficientes e estados internos de filtros IIR/biquads, integradores de circuitos analógicos (TPT/ZDF, tone stacks bilineares) e acumuladores de fase; converta para `f32` nas bordas de I/O.
   - Use `f32::mul_add` / `f64::mul_add` onde FMA melhora precisão e throughput (verificando se `fma` está habilitado no alvo ou se o fallback em software não degrada performance!).
3. **Processamento em Blocos e Auto-Vetorização:**
   - **Nunca chame um processador de bloco com fatia de 1 amostra (`&[x]`) dentro de um laço `for i in 0..len`** (gera overhead de setup/recalculo de coeficientes por amostra e destrói vetorização).
   - Use `slice.chunks_exact(N)` / iteradores `zip` com comprimentos provados iguais (`let len = input.len().min(output.len()); let input = &input[..len]; let output = &mut output[..len];`) para que o LLVM **elimine bounds checks** e auto-vetorize sem precisar de `get_unchecked`.
   - Use `#[inline]` em funções pequenas chamadas por amostra em outro módulo/crate; evite `#[inline(always)]` indiscriminado (infla I-cache).

---

## 6. Cargo, Workspace Lints, Clippy e Prontidão Rust 2024

### 6.1 `[workspace.lints]` Recomendado no `Cargo.toml` Raiz
Conforme o **Cargo Book** e **Clippy**:
```toml
[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "deny"
unused_must_use = "deny"
dead_code = "warn"
unreachable_patterns = "deny"

[workspace.lints.clippy]
correctness = { level = "deny", priority = -1 }
suspicious = { level = "warn", priority = -1 }
perf = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
# Cherry-pick cirúrgico de `restriction` (nunca habilite o grupo todo):
undocumented_unsafe_blocks = "deny"
unwrap_used = "warn"
expect_used = "warn"
indexing_slicing = "warn"
float_cmp = "warn"
# Alívios pragmáticos para DSP numérico (evita ruído falso-positivo do pedantic):
cast_possible_truncation = "allow"
cast_precision_loss = "allow"
cast_sign_loss = "allow"
module_name_repetitions = "allow"
```
E em cada crate membro:
```toml
[lints]
workspace = true
```

### 6.2 Perfis de Compilação e Reprodutibilidade
- Use sempre `cargo test --workspace --locked` e `cargo build --release --locked` em CI e portões de aceite.
- Em `[profile.release]`: `opt-level = 3`, `lto = "thin"` (ótimo equilíbrio entre otimização inter-crate e tempo de link), `codegen-units = 1` (máxima otimização para o motor DSP).
- Mantenha `panic = "unwind"` se o binário depende de `catch_unwind` nas fronteiras de áudio/plugin; use `panic = "abort"` apenas em binários standalone que não usam `catch_unwind`.

### 6.3 Persistência Atômica Segura no Windows
- **Nunca faça `if target.exists() { remove_file(target); } rename(tmp, target);`** — uma falha ou queda de energia entre `remove_file` e `rename` corrompe/destrói o preset ou calibração do usuário, e nomes de arquivo temporário só com `pid()` colidem entre threads concorrentes.
- Crie arquivo temporário único no mesmo diretório (`pid` + contador atômico/UUID), execute `sync_all()` antes de fechar e substitua atomicamente (`tempfile::NamedTempFile::persist` ou `ReplaceFileW` no Windows).

### 6.4 Prontidão para Rust 2024 Edition
- Prefira sintaxe compatível com Edition 2024 desde já:
  - Blocos `unsafe { ... }` explícitos dentro de `unsafe fn` (`unsafe_op_in_unsafe_fn`).
  - `unsafe extern "C" { ... }` e `#[unsafe(no_mangle)]` ao migrar para Edition 2024.
  - Precisão de captura de lifetimes em RPIT com `use<...>` quando retornar `impl Iterator`/`impl Trait`.
  - Evite usar `gen` como identificador (palavra reservada na Edition 2024).

---

## 7. Checklist Rápido de Code Review Rust

Antes de aprovar ou marcar qualquer código Rust como pronto:
- [ ] Zero `unwrap()`/`expect()` ou indexação cega em código de produção / RT.
- [ ] Zero `Mutex::lock()` ou `try_lock()` bloqueando ou desviando o callback de áudio para sinal seco.
- [ ] Zero alocação, `Vec::resize`, `String`, ou construção de FFT/FIR dentro do caminho `process()`.
- [ ] Todo `unsafe` tem comentário `// SAFETY:` verificável e invariantes reais (sem data races em SPSC / `MaybeUninit`).
- [ ] Unidades físicas e escalas (`V`, `dB`, `Hz`, linear, pré/pós-boost) são explícitas e consistentes em toda a cadeia.
- [ ] Nenhum campo/parâmetro gravado sem ser lido (`dead_code`), nenhum `match` com fallback silencioso que mascara variante não implementada.
- [ ] `cargo clippy --workspace --all-targets --locked` limpo sem warnings ocultados sem justificativa.
