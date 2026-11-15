---
spec: 034
titulo: Parâmetros somem sob contenção, ações disparam duas vezes e o host processa no período/taxa errados
status: implementada
atualizado: 2026-11-14
---
# 034 — Ponte de controle e host de áudio: correções pós-review do commit e1ffff3

## Objetivo
Garantir que todo parâmetro aceito pela API chegue ao motor exatamente uma vez, e que o host de áudio processe no período e na taxa de amostragem que o dispositivo realmente abriu.

## Contexto
Origem: code review do commit `e1ffff3` (2026-11-14), verificado por leitura de código, sem execução.

1. **Parâmetro descartado sob contenção.** `crates/sonix-desktop/src/api.rs:2462` aplica as chaves que não passam pela fila SPSC com `dsp_ref.try_lock()`. Se o callback está com o mutex, o bloco `if let Ok` é pulado, nada é refeito, mas o `AppState` já foi atualizado e a resposta é 200. Afeta `amp_1.model`, `amp_1.bright`, `split.*`, `merge.*`, `cab_1.mic*`, `amp_2.*`, `cab_2.*`, `rev_1.predelay_ms`, `input.source` e todo o Mod Shop.
2. **Ação de uma vez executada duas vezes.** No mesmo POST, quando há uma chave fora da fila, todas as chaves aplicadas são reaplicadas direto sob o lock (api.rs:2460–2640) depois de já terem sido enfileiradas. Como o `ParamCoalescer` junta as escritas de um frame, `{"amp_1.bright":true,"looper.action":"undo"}` desfaz duas camadas.
3. **Período fixo de 128.** `RealtimeDuplexEngine::new` fixa `target_period = 128` (`audio.rs:1084`). `pop_samples_for_processing` (`audio.rs:1124`) drena até 128 amostras. No fallback sem `BufferSize::Fixed(128)` e nos caminhos I16, o período real é o padrão do dispositivo (ex.: 480). Resultado: 128 amostras reais e 352 zeros por callback, com estalo constante, enquanto a telemetria diz 128.
4. **Taxa fixa em 48 kHz.** `apps/sonix-app/src-tauri/src/lib.rs:61` cria a cadeia e o ring limpo a 48 kHz e nunca chama `prepare()` com a taxa de `out_config.sample_rate()`. Num dispositivo a 44,1 kHz, o afinador lê cerca de +147 cents, e delays, filtros e tempos do gate ficam todos desviados.
5. **Callback grande.** `mono_buf`, `mono_in_f32` e `mono_out_f32` têm 4096 posições (`audio.rs:610,709`). Um callback com mais de 4096 frames deixa o resto do buffer de saída sem escrever.
6. **Silêncio por contenção.** Quando o callback não consegue o lock, a saída vira silêncio (`audio.rs:725–731`). Antes deste commit, a entrada passava direto.
7. **Fila com vários produtores.** `LockFreeParamQueue` é SPSC (`ampforge_dsp::queue::SpscBuffer`), mas é `Clone + Sync` e recebe `push` de várias threads (IPC, HTTP, MIDI) sem exclusão mútua. Isso quebra o contrato do SPSC.
8. **Modo do delay divergente.** `AppState` começa com `dly_1.mode = "4"` (`api.rs:46`); o `DelayProcessor` começa no modo 1 (`fx/delay.rs:144`).
9. **Troca de dispositivo decorativa.** `set_audio_device` (`apps/sonix-app/src-tauri/src/commands.rs:216`) só reescreve a telemetria e nunca reabre os streams.

## Comportamento esperado
- Toda chave validada segue **um único caminho** até o motor: a fila de parâmetros. Mudanças que exigem preparação (modelo de amp, IR, topologia) são preparadas fora da thread de áudio e trocadas por ponteiro ou mensagem, sem `try_lock` silencioso.
- A API só responde 200 quando cada chave aplicada foi enfileirada. Se a fila estiver cheia, responde 503 com `retry_after_ms` e não altera o `AppState`.
- O host lê o período e a taxa efetivos do stream aberto e configura `target_period`, `prepare(sample_rate, max_block)` e o ring limpo com esses valores. Callbacks maiores que o bloco preparado são processados em fatias.
- O callback de áudio nunca espera por lock e nunca emite silêncio por contenção.
- `set_audio_device` reabre os streams com a configuração pedida. Se o dispositivo recusar, devolve erro e mantém a configuração anterior funcionando.

## Critérios de aceite
- **CA-1:** Com uma thread segurando o lock da cadeia por 10 ms, um POST de `{"amp_1.model":"twin_ab763"}` faz `chain.amp().model_id` virar `UsTwinBlackface65` em até 50 ms depois de o lock ser liberado. Nenhuma chave com resposta 200 deixa de ser aplicada.
- **CA-2:** Com duas camadas de overdub gravadas, o POST `{"amp_1.bright":true,"looper.action":"undo"}` desfaz **exatamente uma** camada. O comando `LooperAction(Undo)` é consumido uma única vez.
- **CA-3:** No host simulado com callbacks de 480 frames e entrada senoidal de 1 kHz, depois do aquecimento, 100% das amostras de saída vêm do ring (contador `zero_fill_samples == 0`) por 10 s.
- **CA-4:** Com o dispositivo de saída simulado a 44 100 Hz, uma senoide de 110,00 Hz na entrada aparece no afinador como `A2` com |cents| ≤ 1. `clean_input_ring.sample_rate()` vale 44 100.
- **CA-5:** Um callback de 8192 frames escreve todos os 8192 frames de saída com o sinal processado; nenhum frame fica com conteúdo anterior.
- **CA-6:** Com uma thread enviando 1000 POSTs/s por 5 s durante a reprodução, o contador `dsp_contention_silence_blocks` da telemetria fica em 0.
- **CA-7:** Duas threads enviando 10 000 parâmetros cada pela API fazem o motor receber os 20 000, sem perda nem duplicação (a fila tem um único produtor real ou é MPSC comprovada).
- **CA-8:** Na inicialização, `GET /api/state` → `dly_1.mode` é igual a `DelayProcessor::re201_mode()`.
- **CA-9:** `set_audio_device({sample_rate:44100, buffer:256})` no host simulado faz o status relatar 44 100 Hz e 256 frames, **lidos do stream aberto**. Uma configuração recusada devolve erro, e o status e o áudio continuam na configuração anterior.
- **CA-10 (erro):** Com a fila cheia, o POST responde 503, e `GET /api/state` não mostra o valor rejeitado.
- **CA-11 (não-regressão):** Os testes de contrato das Specs 014, 018 e 028 continuam passando.

## Fora de escopo
- Mostrar dispositivos reais e telemetria honesta do host. Isso fica na Spec 036.
- Remover as alocações restantes no callback (strings, `resize`, drop de IR). Isso fica na Spec 037.
- O shaper do "All Buttons In" do 1176 (achado do mesmo review) fica na Spec 041, e o pad duplo do Plexi na Spec 039.

## Invariantes tocadas
- **Zero-alloc / zero-lock no callback (regra do projeto, Spec 026):** este trabalho remove o `try_lock` do caminho de controle e elimina o silêncio por contenção. O callback não ganha nenhum lock novo.
- **Tabela única de parâmetros (Spec 014):** continua sendo a única porta de entrada. Muda só o transporte: fila em vez de aplicação direta.

## Testes que cobrem esta spec
- `crates/sonix-desktop/tests/contract_034_control_bridge_host.rs` (CA-1, CA-2, CA-3, CA-4, CA-5, CA-6, CA-7, CA-8, CA-9, CA-10, CA-11 — 12 testes passando)
- `apps/sonix-app/src-tauri/tests/contract_034_audio_device.rs` (CA-4, CA-9 — abertura/rollback de dispositivo e propagação de sample_rate)
- Não-regressão: `contract_014_ui_dsp_bridge.rs`, `contract_018_low_latency_host.rs`, `contract_028_low_latency_realtime_safety.rs`

## Notas
Os achados 1 e 2 têm a mesma raiz: dois caminhos de aplicação. Unificar na fila resolve os dois, desde que as mudanças "pesadas" (modelo de amp) venham pré-preparadas.
