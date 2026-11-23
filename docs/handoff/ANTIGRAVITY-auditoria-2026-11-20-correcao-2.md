# Ordem de serviço corretiva 2 — entrega das correções COR-01..08

| Campo | Valor |
|---|---|
| Executor | Antigravity |
| Base | ponta de `fix/auditoria-2026-11-20` (`746943e` ou posterior que contenha este arquivo) |
| Ordens anteriores | [ordem original](ANTIGRAVITY-auditoria-2026-11-20-ordem-de-servico.md) e [corretiva 1](ANTIGRAVITY-auditoria-2026-11-20-correcao-da-entrega.md) |
| Itens | COR-09 a COR-13 |
| Resultado da revisão de `746943e` | **Entrega não aceita.** O portão literal reprova de forma intermitente, o critério do AUD-15 não foi atingido e foi rotulado como FEITO, e o COR-07 mudou o processamento dos drives sem teste dourado real |
| Decisão do dono | **O critério do AUD-15 fica mantido, sem exceção** |

---

## 0. Contrato

### 0.1 O que continua valendo
Continuam valendo integralmente:
- a §0 da ordem original;
- a §0 da corretiva 1, inclusive as proibições adicionais da §0.3 e as regras de Git da §0.4 (corpo de commit obrigatório; nenhum commit mistura itens).

Onde houver divergência, vale este documento.

### 0.2 Proibições adicionais desta rodada
Cada uma reprova a entrega.
- Marcar como **FEITO** um item cujo critério de pronto não foi atingido. O único estado possível nesse caso é **NÃO ATENDIDO**, com a medição.
- Levar para "Objeções" um critério que a ordem marca como obrigatório.
- Reduzir oversampling, número de *taps*, ordem de filtro, número de estágios, ou trocar `f64` por `f32`, para ganhar CPU. A exceção é um teste dourado do COR-10 que prove a saída igual dentro da tolerância.
- Remover `flush_f32`/`flush_f64` ou proteção `is_finite` de estado recursivo.
- Declarar contagem de testes que não saiu de uma execução colada no relatório.

### 0.3 Ordem
Execute COR-09 a COR-13 nesta ordem.
- O COR-10 precisa vir antes do COR-12: a reversão do oversampling dos drives aumenta o custo, e é esse custo que o COR-12 precisa recuperar.
- Depois do COR-11 e do COR-12, rode o portão literal da §2.3.

---

## 1. Estado medido na revisão (2026-11-22, `746943e`, Pentium G3250, Windows 10)

| Comando | Resultado |
|---|---|
| `cargo test --workspace --locked --no-fail-fast` | **575 aprovados, 1 reprovado**: `cor_07_regressao_cpu::cor_07_zero_alloc` (`left: 8, right: 0`) |
| O mesmo arquivo, 10 execuções seguidas (`cargo test -p ampforge-dsp --test cor_07_regressao_cpu`) | **6/10**. As 4 falhas deram `left: 2` |
| `cor_07_zero_alloc` isolado, 10 execuções | 10/10 |
| `cargo test -p sonix-desktop --locked --no-fail-fast --features fault-injection` | Verde. O `contract_013::test_ca_02` passa |
| `npm test` | 133/133 |
| 048, `npm run test:e2e`, build da UI, 3 builds de *release* | Verdes |
| `git status --porcelain` após o portão | Limpo |

Medido pelo próprio relatório (seção 5) e confirmado pelo corpo do commit `72fe513`:
- `pesada` 96k/32: `excesso_sobre_ruido` = **163**, mediana das execuções 162, 163 e 216. O critério é ≤ 0. O item está marcado como FEITO.
- `padrao` 48k/128: 1,135× a referência `8cfd49a`. Há objeção com medição atribuída à FIR do AUD-32, aceita como prevê o item 3 do COR-07.

O relatório afirma `576 passed; 0 failed` para o `--workspace`. A revisão não reproduziu isso.

---

## 2. Itens

Formato fixo: **Evidência**, **Decisão**, **Testes**, **Pronto quando**.

### COR-09 · `cor_07_zero_alloc` conta alocações de outros *threads*

**Evidência:**
- Em [cor_07_regressao_cpu.rs:10-30](../../crates/ampforge-dsp/tests/cor_07_regressao_cpu.rs#L10-L30), o contador é um `AtomicBool`/`AtomicUsize` **global**. Ele conta qualquer alocação do processo enquanto está ligado, inclusive as do executor de testes e de outros casos do mesmo binário.
- O alocador contador obrigatório da 037 CA-7 é por *thread*: [contract_037_realtime_safety.rs:21-24](../../crates/sonix-desktop/tests/contract_037_realtime_safety.rs#L21-L24) usa `thread_local!` com `Cell`.

**Decisão:**
1. Troque o contador do `cor_07_regressao_cpu.rs` pelo mesmo esquema da 037: `thread_local!` para o liga/desliga e para as contagens de alocação e desalocação.
2. Conte alocações e desalocações separadamente. As duas precisam ser 0.
3. O `TEST_SERIAL` deixa de ser necessário. Apague-o.

**Testes:**
- **Red:** cole a saída das 10 execuções do arquivo antes da correção, mostrando a intermitência.
- **Green:** depois da correção:
  - 50 execuções seguidas de `cargo test -p ampforge-dsp --test cor_07_regressao_cpu`;
  - prova de que o teste ainda pega alocação real: injete temporariamente um `Vec::with_capacity(1)` em `VirtualCabProcessor::process`, mostre a falha e reverta.

**Pronto quando:** 50/50, mais a saída da injeção colada.

### COR-10 · COR-07 mudou o processamento dos drives sem teste dourado real

**Evidência:**
- **Oversampling a 96 kHz caiu de 8× para 4×:**
  - em [drive.rs:430-431](../../crates/ampforge-dsp/src/fx/drive.rs#L430-L431) e no cálculo do decimador Chebyshev (~l. 500), com `if sr > 64_000.0 { 4.0 } else { 8.0 }`;
  - em `os_steps`, nas l. 634, 654, 724, 758 e 831.
  - Isso muda o processamento (taxa interna de 768 kHz para 384 kHz) e o desenho do filtro de decimação.
- **Proteções removidas do `DriveBiquad64`** ([drive.rs:238](../../crates/ampforge-dsp/src/fx/drive.rs#L238)):
  - o `flush_f64` da saída e de `x1`;
  - a proteção `if y.is_finite() { y } else { 0.0 }`.
- **Os "dourados" do COR-07 não comparam antes com depois:**
  - [cor_07_regressao_cpu.rs:82](../../crates/ampforge-dsp/tests/cor_07_regressao_cpu.rs#L82) compara blocos de 32 contra um bloco de 512 no **mesmo** código;
  - a l. 115 só confere que o RMS é finito e maior que 0,05.

**Decisão:**
1. **Reverta** a escolha de oversampling por taxa. O drive volta a 8× em todas as taxas, como estava em `9ab9bb8`, inclusive no decimador e em todos os `os_steps`.
2. **Restaure** no `DriveBiquad64` o `flush_f64` da saída e de `x1` e a proteção `is_finite`, exatamente como em `9ab9bb8`.
3. **Fixtures dourados reais:**
   - num *worktree* de `9ab9bb8` (o commit anterior ao COR-07), crie o exemplo `cargo run -p ampforge-dsp --example gerar_dourados_cor10`;
   - ele grava em `tests/fixtures/cor-10/` a saída de cada processador para um sinal fixo: 2 s de seno de 220 Hz + 880 Hz a −12 dBFS, mais 0,5 s de ruído branco de semente fixa;
   - processadores: **todos** os `DriveModel`, os 5 `AmpModelId` e o `VirtualCabProcessor` sem IR;
   - taxas: 48 kHz e 96 kHz;
   - formato: `f32` *little-endian*, um arquivo por processador e taxa;
   - copie o exemplo e os *fixtures* para a branch.
4. Substitua `cor_07_cab_speaker_fir_saida_igual`, `cor_07_drive_ts808_saida_igual` e `cor_07_amp_saida_igual` por `cor_10_<processador>_<sr>_igual_ao_dourado`, um caso por arquivo de *fixture*.
   - **Critério:** `max|y − y_dourado| ≤ 1e-6 · max|y_dourado|`.
5. As otimizações do `72fe513` que mantêm o resultado (`sqrt().sqrt().sqrt()` no lugar de `powf(0.125)`, coeficientes içados para fora do laço no `amp.rs`, histórico contíguo da FIR em `cab.rs`) podem **ficar**, desde que os dourados passem.

**Testes:** os `cor_10_*` do item 4.
- **Red:** rode-os sobre `746943e`, antes dos itens 1 e 2, e cole a falha dos drives a 96 kHz.

**Pronto quando:** todos os `cor_10_*` passam, e não existe mais nenhum `sample_rate > 64_000` que mude o oversampling em `drive.rs`.

### COR-11 · *Thread* de áudio sem prioridade de tempo real; bancada sem a mesma prioridade

**Evidência:**
- O `cpal` 0.18.2 só eleva a prioridade do *thread* de áudio do WASAPI com a *feature* `realtime`:
  - `boost_current_thread_priority` usa `audio_thread_priority::promote_current_thread_to_real_time` e, se falhar, `THREAD_PRIORITY_TIME_CRITICAL`;
  - código em `cpal-0.18.2/src/host/wasapi/stream.rs:714-728`, sob `#[cfg(feature = "realtime")]`.
- Em [sonix-desktop/Cargo.toml:11](../../crates/sonix-desktop/Cargo.toml#L11), o `cpal` entra sem `realtime`. **O callback de áudio do produto roda hoje com prioridade normal**, exposto às mesmas preempções de 220–280 µs que o relatório mede na bancada.
- A `bancada_auditoria` também mede num *thread* de prioridade normal.

**Decisão:**
1. **Produção:** a *feature* `host-audio` passa a ser `["dep:cpal", "cpal/realtime"]`. Atualize o `Cargo.lock` e versione-o: o portão usa `--locked`.
2. **Bancada:**
   - antes de medir **qualquer** cenário, inclusive o `ruido`, promova o *thread* de medição com `audio_thread_priority::promote_current_thread_to_real_time(bloco, sr)`, a mesma função que o `cpal` usa;
   - use como `dev-dependency` a mesma versão que o `Cargo.lock` resolver para o `cpal`;
   - se a promoção falhar, a bancada **aborta** com código ≠ 0 e a mensagem de erro. Não há *fallback* silencioso;
   - o JSON de cada cenário ganha `"prioridade": "tempo_real"`.
3. A definição do critério do AUD-15 **não muda**: `excesso_sobre_ruido = blocos_acima_100(cenário) − blocos_acima_100(ruido)`, com os dois cenários na mesma prioridade, na mesma execução e com o mesmo número de blocos.

**Testes:**
- `cor_11_host_audio_ativa_realtime`: lê `crates/sonix-desktop/Cargo.toml` e o `Cargo.lock` e reprova se `host-audio` não incluir `cpal/realtime` ou se `audio_thread_priority` não estiver no *lock*. *Red* antes do item 1.
- `cargo tree -p sonix-plugin | grep cpal` continua vazio (não regredir o AUD-45).

**Pronto quando:** o teste passa, o *release* compila, e a bancada emite `"prioridade": "tempo_real"` em todos os cenários.

### COR-12 · Critério do AUD-15

**Evidência:**
- `excesso_sobre_ruido` = 163 em `746943e`.
- Composição declarada pelo relatório para a `pesada` 96k/32 (média de cerca de 117 µs por bloco, contra 333 µs de orçamento):
  - 2 × `WhiteBoxAmp` a 192 kHz: cerca de 50 µs;
  - TS808: cerca de 19 µs, que vai subir com a reversão do COR-10;
  - IR de 0,49 s: cerca de 14 µs.
- Cálculos por amostra que dependem só de parâmetros, encontrados no laço de amostras de `amp.rs`:
  - `alpha_grid_jcm = (2π·6400/sr).clamp(…)`;
  - `tb_v2_gain = triode_stage_gain(100_000.0, 220_000.0, 1_500.0) / DECISAO_SONIX_PREAMP_STAGE_NORM`.
  - Procure outros do mesmo tipo.

**Decisão:**
1. Meça depois do COR-10 e do COR-11, 3 execuções, mediana.
2. Se `excesso_sobre_ruido > 0`, reduza o custo **só** com otimizações que mantêm a saída. Cada uma é provada pelos dourados do COR-10. Permitido:
   - içar para fora do laço de amostras todo cálculo que dependa só de parâmetros ou da taxa: coeficientes, `exp`, `powf`, `sqrt`, `triode_stage_gain` e constantes de filtro;
   - recalcular coeficientes só quando o parâmetro muda, ou por sub-bloco durante rampa, como no AUD-11;
   - remover trabalho repetido entre as duas instâncias de amp quando a segunda não está em transição: fora da janela de 20 ms do crossfade do AUD-10, a instância de transição não pode ser processada;
   - eliminar cópias de buffer, desvios por amostra e indexação com módulo;
   - estruturar laços para autovetorização, sem mudar a ordem das operações de forma que os dourados reprovem.
3. **Proibido:** tudo da §0.2 desta ordem.
4. Para cada otimização: commit próprio (`perf(COR-12): …`), com os µs por bloco antes e depois no corpo, medidos na bancada `pesada` 96k/32.

**Testes:** os dourados do COR-10 e `cor_07_zero_alloc` (já corrigido no COR-09) verdes depois de cada commit.

**Pronto quando:** `pesada` 96k/32 com `excesso_sobre_ruido ≤ 0` na mediana de 3 execuções, com `"prioridade": "tempo_real"`. **Sem exceção e sem objeção substituta.** Enquanto isso não for atingido, o AUD-15 e o COR-12 ficam **NÃO ATENDIDO** no relatório, com a medição.

### COR-13 · Relatório e `AGENTS.md`

**Decisão:**
1. **Seção 1 do relatório:**
   - o AUD-15 só passa a FEITO se o COR-12 atingir o critério;
   - o COR-07 passa a "FEITO (custo), substituído pelo COR-10 (fidelidade)";
   - a linha atual do COR-07, que afirma `576 passed; 0 failed`, é corrigida com a execução real.
2. **Seção 2:** cada comando literal da §2.3 com a contagem colada. O `cargo test --workspace --locked --no-fail-fast` roda **3 vezes seguidas**, e as 3 contagens são coladas. Uma única reprovação em qualquer das 3 reprova o portão.
3. **Seção 3:** os 5 cenários do COR-06, 3 execuções cada, com `prioridade`.
4. **Seção 8:** acrescente COR-09 a COR-12 na mesma tabela.
5. **`AGENTS.md`:**
   - a linha "Estado atual" com os números do portão final;
   - a regra Cortex "Zero-Alloc Realtime" ganha uma frase: "o *thread* de áudio roda com prioridade de tempo real (`cpal/realtime`, COR-11)".

**Pronto quando:** todo número do relatório se reproduz com a tolerância da §0.10 da ordem original, e as 3 execuções do `--workspace` estão verdes.

---

## 3. Fora desta ordem
- Itens dados como confirmados nas revisões anteriores: COR-01 a COR-06 e a objeção do `padrao` 48k/128 atribuída à FIR do AUD-32.
- Qualquer coisa das specs 053–092.
- Bloqueios aceitos da §0.7 da ordem original.
