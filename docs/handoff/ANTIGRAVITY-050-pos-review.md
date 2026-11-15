# Handoff — correções do code review das Specs 049 e 050 (para o Antigravity)

> Origem: code review do working tree (entrega da Spec 049 + Spec 050), feito em 2026-11-15.
> Substitui `ANTIGRAVITY-049-pos-review.md` como trabalho em andamento.
> **Decisão do usuário: não criar spec nova.** Cada achado é corrigido dentro da spec a que pertence
> (049 ou 050). Quando o achado já é violação de um CA existente, o CA não muda — só o código e o
> teste que deveria tê-lo pego. Quando nenhum CA cobre o comportamento, **acrescente um CA à spec
> existente indicada** (próximo número livre, ex. `CA-40` na 049, `CA-5` na 050) antes do código.

## 0. Leia antes, nesta ordem
1. `AGENTS.md` (SDD + TDD estrito, áudio zero-alloc, "todo controle visível funciona no backend").
2. `CLAUDE.md` (comandos; o `target-dir` fica em `G:/Dev/.cargo-target/simulador-de-efeitos`).
3. `specs/049-correcoes-pos-review-das-specs-034-048.md` e `specs/050-correcao-inicializacao-tauri-e-chiado-audio.md`, inteiras.
4. `.agents/skills/sonix-pro-ampforge-expert/SKILL.md` para os itens de DSP (R1–R7, R9).

## 1. Regras inegociáveis
- **TDD estrito:** para cada item, primeiro um teste que reproduz o defeito e é **mostrado falhando** (red), depois a correção (green). Nome cita o CA (`ca32_...` / `'CA-3: ...'`) e fica no arquivo de contrato da spec (`contract_049_*.rs`, `contract_050_*.rs`, `tests/contract/050-*.test.js`).
- **Callback de áudio:** zero alocação, desalocação, lock, I/O e log — medido pelo alocador de contagem da Spec 037.
- **Não reescreva teste antigo para passar.** Exceção autorizada: o teste de S1 (abaixo) afirma o comportamento defeituoso; corrija a asserção e registre nas Notas da 050 como "revertido", com o motivo.
- **Git:** nunca rode `commit`, `push`, `rebase` ou `reset` sem autorização explícita.
- Números de linha abaixo são do momento do review; confira antes de editar.

## 2. Itens — Spec 049

### Alta prioridade
- **R1 — Mod 2 mudo com split desligado** (`crates/sonix-desktop/src/audio.rs:~4551`). Os blocos da linha 1 do DAG (Mod2, Amp2, Cab2, Comp2) só rodam com `split_active`, que depende de `split_enabled`. No layout padrão, ligar `mod_2` não altera a saída; só o Comp2 tem caminho alternativo. Viola a regra "todo controle visível funciona no backend".
  - CA: 049 CA-23/CA-26 (a grade é a topologia). Se nenhum cobrir "bloco na linha 1 com split desligado", acrescente um CA à 049.
  - Teste: topologia padrão, split desligado, `mod_2` ligado com chorus → saída difere mensuravelmente do `mod_2` desligado.
- **R2 — troca de canal corta caudas de delay/reverb** (`audio.rs:~2773`). `set_active_channel` chama `reset_state()` em `dly_1`, `rev_1` e `mod_2`. Delay e reverb agora são instância única pós-merge (Notas da 049), então a cauda é cortada mesmo com spillover ligado.
  - CA: 049 CA-3 (spillover). Atenção à interação com CA-30b: o "estado zerado" vale para blocos **por canal** (amp, drive, EQ, cab), não para as instâncias únicas de delay/reverb/looper. Esclareça isso nas Notas da 049.
  - Teste: spillover ligado, rajada no delay e no reverb, troca de canal → a cauda continua (mesma régua do CA-3).
- **R3 — Modifiers sobrescrevem o valor-base** (`audio.rs:~2862`). O valor modulado é gravado direto no parâmetro (alvo do smoother de ganho, painel do amp, mix de delay/reverb); o valor do usuário não é guardado.
  - Efeitos: remover o vínculo com o pedal no calcanhar deixa o parâmetro preso no `min_val`; com vínculo ativo, edições da UI e de preset são sobrescritas no bloco seguinte.
  - CA: 049 CA-32 (Modifier no motor). Correção esperada: manter valor-base separado do valor efetivo; remover o vínculo restaura o base.
  - Testes: (a) vincular `EXP_PEDAL_1 → amp_1.gain`, pedal em 0, remover vínculo → ganho volta ao base; (b) com vínculo ativo, `amp.gain` pela API muda o base.
- **R4 — Modifiers não chegam à cadeia-sombra do crossfade** (mesmo trecho de R3). Durante troca de cena/preset com LFO ativo, a sombra roda sem modulação e o parâmetro salta quando as cadeias trocam.
  - CA: 049 CA-32. Teste: LFO em `amp_1.treble`, troca de cena → sem descontinuidade (régua de janela de 1 ms do CA-24).
- **R5 — vazamento permanente no reaper** (`audio.rs:~1944`). Com a fila de 16 posições cheia, `retire_topology`/`retire_modifiers` fazem `mem::forget` no Box antigo, e nada esvazia a fila (ver R10). Arrastar blocos na grade vaza uma topologia ou tabela de modifiers por troca até o fim da sessão.
  - CA: 049 CA-24 (troca de topologia sem buraco e sem alocação). A desalocação tem de ocorrer fora do callback (thread de controle drenando a fila).
  - Teste: 1000 trocas de topologia → contador de `Drop` igual ao número de topologias retiradas; zero desalocações no callback.

### Média prioridade
- **R6 — `clear_tail()` dentro do callback** (`audio.rs:~3283`). Na troca de cena/preset com spillover desligado, zera de uma vez o buffer de delay (segundos de áudio), molas, combs/allpass do reverb e shimmer — centenas de KB num callback. Risco de xrun em 64–128 frames.
  - CA: 049 CA-2 + regra de tempo real. Correção: zerar de forma amortizada (fatias por bloco) ou trocar por buffer já limpo preparado fora do callback.
  - Teste: tempo de pior callback na troca de cena, buffer de 64 frames, abaixo do período do bloco (registre valor medido ou `[medida pendente]`).
- **R7 — reset incompleto do DC blocker** (`audio.rs:~2771`). A troca de canal zera `dc_x1`/`dc_y1`, mas deixa `dc_x2`/`dc_y2` (f64) com histórico antigo → transiente/clique que leva segundos para decair (r = 0,99999).
  - CA: 049 CA-30b. Teste: troca de canal com sinal → nenhum degrau acima do limiar na saída.

### Baixa prioridade
- **R8 — latência do oversampler 8x** (`crates/ampforge-dsp/src/oversampling.rs:~239`). Comentários dizem "Stage 0: 47", mas `STAGE0_CENTER` = 51. A fórmula trunca para 64 amostras; o atraso real é ≈65 (51 + 10,5 + 3,25). Compensação e telemetria ficam uma amostra abaixo.
  - CA: 049 CA-12 (±1 amostra). Verifique se o teste do CA-12 passa só pela tolerância; corrija fórmula e comentários.
- **R9 — `reclaim_retired()` não recupera nada** (`audio.rs:~1828`). Diz implementar o dreno cooperativo do CA-24, mas só chama `std::thread::yield_now()`. O teste do CA-24 pode estar passando sem testar o dreno. É a causa raiz de R5 — corrija junto.

## 3. Itens — Spec 050

- **S1 — `resolve_duplex_sample_rate` aceita taxa não suportada** (`audio.rs:~1079`). Com `requested_sr = Some(x)`, devolve `(x, x)` sem conferir `supported_in`/`supported_out`. O teste `test_ca_03_resolve_duplex_sample_rate_falls_back_to_common_or_requested` (`crates/sonix-desktop/tests/contract_050_duplex_rate_alignment.rs`) **afirma esse defeito** (96000 com entrada que só faz 44100 → `(96000, 96000)`).
  - CA: 050 CA-3. Corrija a asserção (exceção autorizada da §1) e o código: taxa pedida não suportada → erro claro ou fallback para taxa comum, nunca abrir em taxa que o dispositivo recusa.
- **S2 — sem taxa comum, volta ao desalinhamento** (`audio.rs:~1097`). O fallback final devolve `(default_in_sr, default_out_sr)` — exatamente o desalinhamento que causava o chiado. A 050 chama isso de "fallback seguro", mas não é.
  - CA: 050 CA-3. Com resampler fora de escopo, o seguro é recusar com erro (`mode_unavailable`/motivo) ou abrir em standby; registre a escolha nas Notas da 050.
- **S3 — calibrar não tira o amp do standby.** O "Comportamento esperado 2" da 050 diz que o silêncio dura "até que o usuário desative STANDBY ou calibre a entrada", mas `handle_post_input_calibrate` (`crates/sonix-desktop/src/api.rs`) não altera `amp_standby`. Nenhum CA cobre isso.
  - Acrescente um CA à 050 (ex. `CA-5`): calibração concluída com sucesso no perfil `default` → `amp_standby = false` no estado e no DSP, e a UI reflete. Teste red primeiro.
- **S4 — teste do CA-1 frágil e incompleto** (`tests/contract/050-release-embedded-ui.test.js`).
  - Caminho do executável fixo (`G:/Dev/.cargo-target/.../release/sonix-app.exe`): falha sem build release e passa contra um exe antigo. Derive o caminho de `.cargo/config.toml` e, se o exe for mais antigo que `ui/dist/index.html`, falhe com mensagem pedindo rebuild.
  - O CA-1 exige ausência de `localhost:1420` no executável, mas o teste não verifica. Acrescente a asserção.
- **S5 — release depende de lembrar a feature** (`apps/sonix-app/src-tauri/Cargo.toml:16`). Com `default = []`, `cargo build --release -p sonix-app` sem `--features custom-protocol` volta a gerar o exe que aponta para o localhost. Documente o comando correto no `CLAUDE.md` (seção Comandos) e/ou faça o build script/teste falhar quando o release não embutir a UI.

### Menores (050)
- **S6:** em `contract_050_startup_safety.rs`, `reset_audio_host_count_for_test()` não tem efeito (o teste usa `start_audio_host = false`). Remova ou justifique.
- **S7:** o teste do CA-4 só verifica `!amp_standby`. Acrescente a verificação de que boost e gate da calibração chegaram ao DSP.

## 4. Ordem sugerida
1. R5 + R9 (vazamento e dreno — mesma correção).
2. R3 + R4 (modifiers).
3. R2 + R7 (troca de canal) e R1 (split).
4. S1 + S2 + S3 (startup/duplex, risco de chiado).
5. R6, R8, S4, S5, S6, S7.

Gate entre etapas: `cargo test --workspace` e `npm test` verdes.

## 5. Fechamento
- Atualize as Notas e a seção "Testes que cobrem esta spec" da 049 e da 050 com os testes novos/alterados e os resultados (`N passed; 0 failed`).
- Atualize o "Estado atual" do `AGENTS.md` e o índice em `specs/README.md`.
- Relatório por etapa: item → CA → teste (red mostrado) → arquivo alterado → resultado.
- Não commite sem autorização.
