# Ordem de serviço corretiva — entrega da auditoria 2026-11-20

| Campo | Valor |
|---|---|
| Executor | Antigravity |
| Base | ponta de `fix/auditoria-2026-11-20` (`e26c7e9` ou posterior que contenha este arquivo) |
| Ordem original | [ANTIGRAVITY-auditoria-2026-11-20-ordem-de-servico.md](ANTIGRAVITY-auditoria-2026-11-20-ordem-de-servico.md) |
| Relatório revisado | [RELATORIO-auditoria-2026-11-20-execucao.md](RELATORIO-auditoria-2026-11-20-execucao.md) |
| Itens | COR-01 a COR-08 |
| Resultado da revisão | **Entrega não aceita.** O portão literal da §2.3 está vermelho, um critério de pronto (AUD-15) não foi atingido, e o relatório omite as execuções que mostram isso |

---

## 0. Contrato

### 0.1 O que continua valendo
A §0 inteira da ordem original continua valendo, **sem exceção**: sem specs novas, TDD estrito com *red* colado, decisões tomadas, proibições da §0.5, bloqueios aceitos da §0.7 e só eles, regras de Git da §0.8, portões da §0.9 e verificação independente da §0.10.

Onde este documento e a ordem original divergirem, vale este documento.

### 0.2 Por que a entrega voltou
A revisão reexecutou o portão e a bancada nesta máquina, em 2026-11-22, sobre `e26c7e9`. Os números estão na §1. Os três motivos:

1. **`npm test` reprova** (130/132). O relatório trocou o comando da §2.3 por uma lista escolhida de 9 arquivos e declarou o portão verde.
2. **O AUD-15 não atinge o critério de pronto.** O cenário `pesada` a 96 kHz com bloco 32 não aparece no relatório.
3. **A bancada do relatório só tem `padrao` e `ruido`.** A §6.3 exige os 5 cenários, e três critérios de pronto (AUD-02, AUD-12 e AUD-15) dependem deles.

### 0.3 Proibições adicionais desta rodada
Cada uma reprova a entrega.
- Rodar qualquer comando do portão diferente do texto literal da §2.3. Não troque `--workspace` por pacotes, não acrescente `--test-threads=1`, `--skip` ou filtros, não substitua `npm test` por uma lista de arquivos.
- Omitir do relatório um cenário de bancada, uma execução ou um teste que falhou.
- Classificar como bloqueio aceito algo que não esteja literalmente na §0.7 da ordem original.
- Reescrever histórico para consertar commits antigos (*rebase*, *amend*, *squash*). Os desvios antigos vão para a seção 7 do relatório (COR-08).

### 0.4 Git
- Continue em `fix/auditoria-2026-11-20`. Um commit por item, com título `fix(COR-NN): …`, `test(COR-NN): …` ou `docs(COR-NN): …`.
- **O corpo do commit é obrigatório:** comando de teste e resultado numérico. Commit sem corpo reprova o item.
- Nunca misture dois itens no mesmo commit, nem "commits de portão" que mexam em mais de um item. Se o portão quebrar por causa de um item, o conserto é um commit daquele item.

### 0.5 Ordem
Execute COR-01 a COR-08 nesta ordem. Depois do COR-05 e do COR-07, rode o portão literal da §2.3. O COR-08 é o último.

---

## 1. Estado medido na revisão (2026-11-22, `e26c7e9`, Pentium G3250, Windows 10)

### 1.1 Portão literal da §2.3

| Comando | Resultado |
|---|---|
| `cargo test --workspace --locked --no-fail-fast` | 571 aprovados, **1 reprovado**: `contract_013::test_ca_02_physical_audio_device_enumeration_wasapi_default_duplex` |
| `cargo test -p sonix-desktop --locked --no-fail-fast --features fault-injection` | 234 aprovados, **1 reprovado** (o mesmo `contract_013::test_ca_02`) |
| `npm test` | **130/132**. Falham os *wrappers* das specs 009 e 018 (COR-01) |
| `node --test tests/contract/048-*.test.js` | Verde. Main FULL 0,8999, Main_stage 0,8512, Audio FULL 0,8643 |
| `npm run test:e2e` | Verde |
| `npm --prefix apps/sonix-app/ui run build` | Verde |
| 3 builds de *release* | Verdes |
| `git status --porcelain` após o portão | Limpo |

O `contract_013::test_ca_02` também reprova hoje na cópia de `8f499ac`. **Não é regressão**: é o estado do dispositivo de entrada desta máquina (COR-02).

### 1.2 Bancada (`cargo run --release --locked -p sonix-desktop --example bancada_auditoria -- <cenário> <sr> <bloco>`)

| Cenário | sr | bloco | média | p99 | máx | Piso `ruido` (média / p99) | Critério | Situação |
|---|---|---|---|---|---|---|---|---|
| `padrao` | 48 000 | 128 | 9,06% | 35,3% | 861% | 1,82% / 3,83% | — | — |
| `pesada` | 48 000 | 128 | 31,2% | 185,6% | 2477% | 2,57% / 5,70% | — | — |
| `pesada` | 96 000 | 32 | 60,1% | **206,7%** | 33 113% | 14,2% / 19,6% | AUD-15: 0 blocos > 100% além do piso | **Reprova** (p99 > 100% ⇒ ≥ 300 de 30 000 blocos acima) |
| `pitch` | 48 000 | 64 | 15,2% | 25,4% | 1435% | 3,43% / 4,03% | AUD-12: p99 ≤ 50% | Passa |
| `silencio` | 48 000 | 128 | 12,9% | 46,2% | — | 1,81% / 4,03% | AUD-02: última/primeira janela ≤ 1,5 | Passa (0,507; 0 subnormais) |

### 1.3 CPU média contra o estado de partida
Medida na auditoria, com o *harness* dela, sobre `8f499ac`, na mesma máquina e com as mesmas definições de cadeia:

| Cenário | `8f499ac` | `e26c7e9` | Variação |
|---|---|---|---|
| `padrao`, 48 kHz, bloco 128 | 6,3% | 9,1% | +43% |
| `pesada`, 48 kHz, bloco 128 | 17,9% | 31,2% | +74% |
| `pesada`, 96 kHz, bloco 32 | 40,3% | 60,1% | +49% |

O AUD-15 não fecha enquanto essa regressão existir (COR-07).

### 1.4 O que a revisão confirmou e não precisa ser refeito
- Os testes `aud_*` exigidos existem e mantêm os limiares da ordem original (amostra conferida: AUD-10, 11, 12, 13, 14, 15, 33 e 34).
- Zero `#[ignore]`, `test.skip`, `todo!`, `// TODO`. Zero `.unwrap()`/`.expect(` em `api.rs` e `commands.rs`. `panic = "unwind"`.
- AUD-03: 20/20 nos dois testes, reexecutado pela revisão.
- AUD-02 e AUD-12: critérios de bancada atingidos (tabela 1.2).

---

## 2. Itens

Formato fixo: **Evidência**, **Decisão**, **Testes**, **Pronto quando**.

### COR-01 · `npm test` vermelho: *wrappers* 009 e 018 apontam para testes apagados

**Evidência:**
- [009-cenas-gapless-canais-modifiers-footswitches.test.js:20](../../tests/contract/009-cenas-gapless-canais-modifiers-footswitches.test.js#L20) exige `test_ca_04_gapless_scene_transition_cosine_crossfade_1_5ms ... ok`. O teste foi apagado em `3218d31` (AUD-35, ponto 7, junto com o `GaplessCrossfader`). O substituto é `contract_049_scene_state_and_queues::ca3_009_troca_de_cena_sem_clique_razao_transiente_le_2`.
- [018-latencia-minima-host-audio-placa-onboard.test.js:24](../../tests/contract/018-latencia-minima-host-audio-placa-onboard.test.js#L24) exige `test_ca_04_negotiated_sample_rate_and_graceful_mismatch_fallback`. Em `9087fe4` (AUD-39) ele virou `test_ca_04_negotiated_sample_rate_prepares_chain`.
- [018-…test.js:26](../../tests/contract/018-latencia-minima-host-audio-placa-onboard.test.js#L26) exige `test_ca_06_backend_fallback_cascade_without_panic`. Esse teste usava o `SimulatedAudioHost` para o modo exclusivo e foi apagado em `9087fe4`, como o AUD-39 manda.

**Decisão:**
1. **Wrapper 009:** a linha do CA-4 passa a rodar `cargo test -p sonix-desktop --test contract_049_scene_state_and_queues ca3_009` e a exigir `ca3_009_troca_de_cena_sem_clique_razao_transiente_le_2 ... ok`. Faça o mesmo com o `ca4_009_quatro_canais_por_bloco_comutam`.
2. **Wrapper 018:** troque o nome do CA-4 pelo novo. Apague a linha do CA-6 e tire "CA-6" do título do caso.
3. **Spec 018:** na matriz de rastreabilidade, o CA-6 passa a "NÃO ATENDIDO — depende do modo exclusivo, não implementado (Spec 028 parcial, AUD-39)". Essa é a única edição permitida na spec 018.
4. **Lint que impede a repetição:** crie o caso `'COR-01: todo teste Rust exigido por um wrapper existe'` em `tests/contract/auditoria-2026-11-20-wrappers.test.js`:
   - para cada `tests/contract/*.test.js`, extraia todo nome no formato `test_\w+` ou `ca\d+_\w+` que apareça antes de `\.\.\. ok` numa expressão regular;
   - cada nome precisa existir como `fn <nome>` em `crates/*/tests/*.rs` ou `apps/sonix-app/src-tauri/tests/*.rs`;
   - a mensagem de falha lista o arquivo, a linha e o nome.

**Testes:** o lint do item 4. *Red*: rodado antes dos itens 1 e 2, ele precisa listar exatamente os 3 nomes da evidência. Cole essa saída.

**Pronto quando:** `npm test` termina com 0 reprovados e o resumo (`ℹ tests`, `ℹ pass`, `ℹ fail`) está colado no commit e no relatório.

### COR-02 · `contract_013::test_ca_02` pulado como "bloqueio aceito 2"

**Evidência:**
- O relatório e o `AGENTS.md` dizem que o teste ficou de fora por "bloqueio aceito 2". O bloqueio 2 da §0.7 é **outro**: controlador MIDI ou interface USB para o teste de desconexão.
- A revisão mostrou que o teste reprova também em `8f499ac` nesta máquina, com "Dispositivo de entrada deve suportar taxas de amostragem padrão (44.1k, 48k ou 96k)". Na auditoria de 2026-11-20 ele passava. Mudou o estado do dispositivo de entrada, não o código.

**Decisão:**
1. **Não mude a expectativa do teste.** Só a mensagem de falha muda: ela passa a listar cada dispositivo enumerado com `nome`, `is_input`, `supported_sample_rates` e `supported_buffer_sizes`.
2. Rode o portão literal da §2.3, **sem `--skip`**.
3. Se o teste reprovar, registre em **Objeções** a saída completa da nova mensagem, mais a saída do mesmo teste rodado num *worktree* de `8f499ac`, que prova que não é regressão. A saída da nova mensagem é a medição que a objeção exige.
4. Apague "bloqueio aceito 2" do relatório e do `AGENTS.md` para esse teste.

**Pronto quando:** o teste roda no portão sem `--skip`, e passa ou está em Objeções com as duas saídas.

### COR-03 · `aud_05_erro_do_stream_marca_perda` instável sob carga

**Evidência:** o próprio relatório registra uma execução com 354,25 ms, contra o limite de 200 ms, durante compilação em paralelo.

**Decisão:**
- O limite de 200 ms **não muda**.
- Se o teste falhar no procedimento abaixo, corrija a produção: o laço de controle precisa ver o código de erro em ≤ 200 ms também sob carga. Exemplo: acordar o laço quando o callback de erro grava o código, em vez de depender só da varredura periódica. O callback de erro continua sem alocar e sem *lock*.

**Testes:** rode 20 vezes seguidas, com `cargo build --release -p sonix-desktop` em execução **ao mesmo tempo** num segundo terminal:
```
for i in $(seq 20); do cargo test -p sonix-app --test auditoria_2026_11_20_tempo_real -q || exit 1; done
```

**Pronto quando:** 20/20, com a saída colada.

### COR-04 · AUD-11 com degraus de 0,5 a 9,5 em `master` e `volume_2`

**Evidência:**
- Em [auditoria_2026_11_20_artefatos.rs:380-385](../../crates/ampforge-dsp/tests/auditoria_2026_11_20_artefatos.rs#L380-L385), o `aud_11_degrau_sem_zipper` usa 0,5 ↔ 9,5 para `amp.master` e `amp.volume_2`.
- A ordem original manda degraus de 0 a 10 e de 10 a 0.

**Decisão:**
1. No `aud_11_degrau_sem_zipper`, `amp.master` e `amp.volume_2` passam a 0 ↔ 10. Se reprovar, corrija o código, não o teste.
2. No `aud_11_rampa_termina_em_25ms` os valores 0,5 ↔ 9,5 **ficam**: com 0, a saída é silêncio e o ganho em dB não é definido. Registre isso no relatório, com essa frase.

**Pronto quando:** `aud_11_degrau_sem_zipper` passa com 0 ↔ 10 nos dois controles.

### COR-05 · AUD-40 incompleto no README

**Evidência:** [README.md:9](../../README.md#L9) ainda afirma "**modelagem White-Box** (100% algorítmica em nível de circuito analógico discreto)". Isso é contradito por:
- as constantes `DECISAO_SONIX_` listadas na seção 4 do relatório;
- o 1176, que é "aproximação comportamental" (AUD-47).

**Decisão:** troque a frase por uma descrição verificável, sem adjetivo:
- o que é derivado do circuito, cada parte com o teste que a sustenta (por exemplo, tone stacks contra a forma fechada de Yeh & Smith, `aud_22_stack_igual_a_yeh`);
- o que é decisão de engenharia sem fonte, com link para a seção 4 do relatório;
- o que é aproximação comportamental (1176).

**Pronto quando:** nenhuma frase do README sobre fidelidade fica sem teste ou medição que a sustente.

### COR-06 · Bancada sem contagem de blocos acima do orçamento

**Evidência:**
- [bancada_auditoria.rs:215-232](../../crates/sonix-desktop/examples/bancada_auditoria.rs#L215-L232) só emite média, p99 e máximo.
- O critério do AUD-15 é uma **contagem**: 0 blocos acima de 100%, descontado o piso.

**Decisão:**
1. A saída JSON de cada cenário ganha `blocos_acima_50`, `blocos_acima_100` e `p999_pct`.
2. O piso é definido assim: o cenário `ruido` roda na mesma execução, com o mesmo `sr` e o mesmo bloco, pelo mesmo número de blocos. O valor descontado é `blocos_acima_100(cenário) − blocos_acima_100(ruido)`, publicado no JSON como `excesso_sobre_ruido`.
3. Cada medição de critério roda **3 vezes**. O relatório traz as 3 execuções e a mediana.

**Pronto quando:** a bancada emite os novos campos, e o relatório os traz para os 5 cenários: `padrao` 48k/128, `pesada` 48k/128, `pesada` 96k/32, `pitch` 48k/64 e `silencio` 48k/128.

### COR-07 · Regressão de CPU e critério do AUD-15

**Evidência:** tabelas 1.2 e 1.3. Suspeito com evidência no código, ainda **não medido**:
- [cab.rs:1205-1215](../../crates/ampforge-dsp/src/cab.rs#L1205-L1215): `step_speaker_fir` faz 384 multiplicações por amostra, em laço escalar com um desvio condicional por *tap* para o índice circular. Isso impede a vetorização.
- Ela roda em toda amostra do gabinete, inclusive no `padrao`, em [cab.rs:1386-1411](../../crates/ampforge-dsp/src/cab.rs#L1386-L1411). A FIR entrou no AUD-32 (`9265cc4`).

**Decisão:**
1. **Meça antes de mexer.**
   - A referência é `8cfd49a` (AUD-02), o primeiro commit que tem a `bancada_auditoria`.
   - Use `git worktree add <pasta-fora-do-repo> <commit>` para cada medição. Não troque a branch da árvore principal.
   - Rode `padrao` 48k/128 e `pesada` 48k/128, 3 vezes cada, nos fins de fase: `8cfd49a`, `949560d`, `b150165`, `9505d67`, `2ab27d2`, `c095198`, `e11912c`, `c20cc65`, `ecc3246` e HEAD.
   - Dentro de toda fase em que a média subir mais de 10%, meça **commit a commit** até achar os responsáveis.
   - Publique a tabela no relatório: commit, cenário, mediana da média e variação.
2. **Recupere o custo nos commits responsáveis**, sem mudar comportamento:
   - todos os testes do item de origem continuam verdes, sem mudar tolerância;
   - teste dourado `cor_07_<componente>_saida_igual`: a saída antes e depois da otimização difere em no máximo 1e-6 relativo (a ordem da soma muda o arredondamento);
   - para a FIR do falante, se ela for confirmada como causa, use histórico com o dobro do tamanho (janela contígua, sem desvio por *tap*) ou convolução particionada por FFT, a mesma infraestrutura do AUD-15. Reduzir o número de *taps* só é permitido se o `aud_32_alto_falantes_seguem_datasheet` continuar passando com a tolerância atual de ±2 dB.
3. **Custo inerente a uma decisão da ordem original:** se a medição mostrar que o custo vem da própria decisão (ex.: a ordem do filtro elíptico do AUD-14) e não da implementação, registre em **Objeções** o commit, os µs por bloco atribuídos e a medição. Isso **não** dispensa o item 4.
4. **Critério do AUD-15 (obrigatório, sem exceção):** `pesada` 96k/32 com `excesso_sobre_ruido ≤ 0` na mediana de 3 execuções.

**Testes:** os dourados do item 2 e `cor_07_zero_alloc` para todo componente otimizado no caminho de tempo real.

**Pronto quando:**
- (a) o critério do item 4 é atingido;
- (b) a média de `padrao` 48k/128 e de `pesada` 48k/128 fica ≤ 1,10 × a de `8cfd49a`, medidas na mesma sessão, ou cada excesso tem objeção com medição conforme o item 3.

### COR-08 · *Red* retroativo, relatório e `AGENTS.md`

**Evidência:**
- No relatório, o *red* de quase todos os itens é só "`FAILED`", sem o trecho exigido pela §0.3.
- Faltam a saída das 20 repetições do AUD-03 e a tabela de ganho de pequeno sinal por modelo, antes e depois, exigida pelo AUD-08.
- O `git log` da seção 6 está desatualizado (falta o `2a4cc42`).
- 46 dos 55 commits não têm corpo. Quatro commits de portão misturam itens: `9505d67`, `2ab27d2`, `c095198` e `e11912c`.
- O `c095198` acrescentou `cab.set_hicut_khz(18.0)` à preparação do `aud_15_saida_igual_a_convolucao_direta` sem registrar isso.

**Decisão:**
1. **Red retroativo** para todo item AUD que não seja [DOC] e cujo *red* no relatório não tenha trecho:
   - crie um *worktree* no commit pai do commit de correção (`<commit>^`);
   - copie para ele **só** os arquivos de teste daquele commit;
   - rode o comando de teste do item e cole o trecho da falha, com o comando.
   - Se o teste passar no pai, o teste não prova a correção: reescreva-o até falhar pelo motivo do item e registre isso.
2. **AUD-03:** cole a saída das 20 + 20 repetições.
3. **AUD-08:** tabela de ganho de pequeno sinal por modelo nos valores padrão, em dB, medida com `regua::ganho_db` a −80 dBFS. "Antes" é medido num *worktree* de `4b6ea3d`; "depois", na HEAD final.
4. **`aud_15`:** explique em Objeções por que o `set_hicut_khz(18.0)` foi acrescentado e mostre, com medição, que com 18 kHz o hi-cut não altera a saída além da tolerância de 1e-5 do teste.
5. **Relatório:** reescreva `RELATORIO-auditoria-2026-11-20-execucao.md` com as seis seções da §6 da ordem original, mais:
   - **seção 7, "Desvios de processo registrados":** a lista de commits sem corpo e de commits que misturam itens, com hash e título. Não reescreva o histórico;
   - **seção 8, "Correções COR":** a mesma tabela da seção 1, para COR-01 a COR-07.
   - A seção 2 traz **cada comando literal da §2.3** com a contagem de aprovados, reprovados e ignorados. A seção 3 traz os 5 cenários do COR-06.
6. **`AGENTS.md`:** reescreva a linha "Estado atual" com os números do portão final desta rodada. Sem "bloqueio aceito 2" para o `contract_013` (COR-02).

**Pronto quando:** todo item não [DOC] tem *red* com trecho e comando; as seções 1 a 8 existem; os números do relatório se reproduzem com a tolerância da §0.10.

---

## 3. Fora desta ordem
- Qualquer item que a §1.4 dá como confirmado.
- Qualquer coisa das specs 053–092.
- Bloqueios aceitos da §0.7 da ordem original.
