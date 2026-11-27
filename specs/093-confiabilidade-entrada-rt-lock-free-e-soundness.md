---
spec: 093
titulo: Confiabilidade de Entrada, Propriedade RT Lock-Free, Soundness de Fila e Limitador de Saída
status: implementada
atualizado: 2026-11-25
---
# 093 — Confiabilidade de Entrada, Propriedade RT Lock-Free, Soundness de Fila e Limitador de Saída

## Objetivo
Eliminar as causas raiz de áudio cru/sem efeitos na entrada de linha do PC, contenção e *dry-bypass* no callback de tempo real, comportamento indefinido (*UB*) na fila de telemetria, clipping digital duro na saída e dessincronização entre UI, `Standby` e seleção de dispositivo de áudio.

## Contexto
Durante testes reais com guitarra conectada na entrada de linha (*Line In*) do computador e na auditoria completa do sistema (`2026-11-24`), foram observados:
1. **Guitarra crua sem efeitos:** `crates/sonix-desktop/src/audio.rs` (`process_output_callback_*`) usa `dsp_chain.try_lock()` sobre `Arc<Mutex<RealtimeDspChain>>`; em contenção, executa `mono_out[..frames].copy_from_slice(&mono_in[..frames])` (regras da Spec 034 CA-6), vazando sinal seco. Além disso, `apps/sonix-app/src-tauri/src/lib.rs` inicia em `amp_standby = true` sem sair de Standby ao aplicar um dispositivo em `commands.rs::set_audio_device`, e `Audio.ts` possui estado local `pending` resetado em `emit()`, além de `store.ts` restaurar apenas `output_device` e `controls.registry.ts` referenciar `'guitar'` / `'main.input_boost'`.
2. **Entrada de linha vs instrumento:** `params.rs` e `audio.rs` (`effective_input_gain`) tratam `input.mode` de forma binária (`instrument` vs `line` com pad `-11.78 dB`), sem distinguir a entrada de linha de placa-mãe/PC (`pc_line_in`, ganho `0 dB` sem pad) de linha profissional balanceada (`+4 dBu`, pad `-11.78 dB`), nem exibir o seletor explícito de canal (`input.channel`: `left | right | sum`) na tela Áudio.
3. **Soundness em `queue.rs`:** `crates/ampforge-dsp/src/queue.rs::SpscBuffer::push_overwrite` avança `tail` via `compare_exchange_weak` pelo produtor e reescreve slots sem sincronização exclusiva com o consumidor (`pop`), configurando *data race* / UB sob concorrência real SPSC.
4. **Estabilidade de saída e duplex:** A saída final em `audio.rs` aplica `val.clamp(-1.0, 1.0)` (clipping digital duro) e `RealtimeDuplexEngine` zera/descarta amostras em underrun/drift sem suavização de borda, causando estalos audíveis. `write_atomic_file` em `api.rs` remove o arquivo destino antes do `rename`.

> **Revogação formal (Spec 034 CA-6):** Esta spec revoga expressamente a permissão de fallback em *dry-bypass* (`copy_from_slice(&mono_in)`) sob contenção da `Spec 034 CA-6`. O callback de saída passa a deter propriedade exclusiva de `&mut RealtimeDspChain`, tornando impossível qualquer contenção ou vazamento de sinal cru.

## Comportamento esperado
1. **Fila Lock-Free Sound (`queue.rs`):** `SpscBuffer<T, CAP>` mantém apenas semântica FIFO estrita (`push` / `pop`) sem `push_overwrite`. A telemetria de último estado (`SpscTelemetryQueue`) utiliza um `TripleBuffer<T>` lock-free (`O(1)`, zero-alloc, sem UB e com `Drop` seguro).
2. **Propriedade Exclusiva RT (`audio.rs`, `api.rs`, `lib.rs`):** A thread de áudio detém propriedade exclusiva de `RealtimeDspChain` via `AudioThreadChainOwner` (sem código morto duplicado e sem `Mutex`/`try_lock` ou `copy_from_slice(&mono_in)` no callback). Ao transferir a instância ativa para a thread RT, o slot na thread de controle preserva os mesmos handles compartilhados (`param_queue`, `resource_queue`, `clean_input_ring`, `scope_output_ring`, `shared_*`, `rt_owned`) para que `ApiContext`, calibração persistida no boot e recuperação de *fault* (via `DspResourceUpdate::ReplaceChain` e `RetiredResource::Chain` off-RT) operem sempre sobre a cadeia real na thread de áudio.
3. **Standby e Aplicação de Dispositivo (`commands.rs`, `audio.rs`, `AmpHead.ts`):** O boot frio sem calibração continua seguro em `Standby` (`Spec 050`). Ao existir calibração salva no boot, `build_runtime_with_data_dir` propaga `InputBoost`, `GateThreshold` e `AmpStandby(false)` também via `param_queue` e sincroniza a telemetria sem sobrescrevê-la com instância dummy em `ApiContext::new`. Ao aplicar um dispositivo de áudio via `set_audio_device`, o backend atualiza `active_device_id`, carrega calibração salva (se houver) e **desativa `amp_standby` / `output_muted` automaticamente**. `swap_dsp_state` preserva o estado `amp_standby` do `AppState`. A chave `STANDBY` no `AmpHead.ts` exibe polaridade visual inequívoca (`ON` = amp ativo, `STANDBY` = mudo/âmbar) nos 5 modelos.
4. **3 Modos Explícitos de Entrada e Canal (`params.rs`, `audio.rs`, `Audio.ts`, `store.ts`):**
   - `input.mode` aceita `"instrument"` (`0 dB`), `"pc_line_in"` (`0 dB` sem pad, para entrada Line In de PC/notebook) e `"line"` (`-11,78 dB` pad, para Linha Profissional `+4 dBu`).
   - `input.channel` aceita `"left"`, `"right"` e `"sum"` e é exposto na tela `Audio.ts`.
   - `Audio.ts` não descarta seleções pendentes em eventos de telemetria, limpa `store.audioPending = null` ao fechar pelo botão `×` e persiste `backend`, `mode`, `sampleRate`, `bufferSize`, `inputDevice`, `outputDevice`, `inputMode` e `inputChannel` no `localStorage`. `store.ts` restaura a preferência salva sempre que qualquer desses campos diferir do estado atual (com `backend` normalizado) e atualiza `amp_1.standby = false` após `setAudioDevice`. `controls.registry.ts` e `shell.ts` não usam mais `'guitar'` ou `'main.input_boost'`.
   - Banners de alerta dinâmicos aparecem **apenas** quando `amp_standby === true` (com botão direto de ativação) ou `input.mode === 'pc_line_in'` (dica para desativar "Escutar este dispositivo" no Windows), mantendo o layout de fixture da `Spec 048` intacto.
5. **Limitador Suave Zero-Latency, Duplex Suave e Escrita Atômica (`audio.rs`, `api.rs`, `calibration.rs`):**
   - A saída final passa por `OutputSoftLimiter` zero-latency: transparente (`0,00 dB`) abaixo de `-1,0 dBFS`, joelho suave entre `-1,0 dBFS` e `-0,1 dBFS` e teto rígido em `[-1.0, 1.0]`, zerando `NaN`/`Inf`.
   - `RealtimeDuplexEngine` aplica micro-fade (`1–2 ms`) em underrun (incluindo o caso de borda `rem == 1`), fade-in de retomada no bloco seguinte ao underrun e crossfade suave na correção de drift, sem adicionar latência algorítmica.
   - `write_atomic_file` usa nome temporário único por chamada (`tmp.<pid>.<seq>`) e `std::fs::rename` atômico sem apagar o destino antes de o novo conteúdo estar completo e sincronizado; `handle_post_presets_save` e `CalibrationStore::save` também utilizam `write_atomic_file`.

## Critérios de aceite
> Cada item é uma afirmação verificável — algo que alguém pode reprovar.

- **CA-1 (Soundness Lock-Free `TripleBuffer`):** `SpscBuffer` não expõe `push_overwrite`; `TripleBuffer<T>` garante entrega do último snapshot publicado sob concorrência multi-thread (produtor/consumidor contínuos), zero alocações e contagem exata de `Drop` (sem vazamento ou double-drop).
- **CA-2 (Propriedade Exclusiva RT e Fim do Dry-Bypass):** `process_output_callback_f32` e `process_output_callback_i16` recebem `&mut RealtimeDspChain` diretamente sem `Mutex`/`try_lock`; sob carga simultânea de mutação de parâmetros na thread de controle, 100% dos blocos processados passam pela cadeia DSP (nunca retornam `mono_in` cru quando ativo) com 0 alocações na heap.
- **CA-3 (Standby Seguro + Saída Automática ao Aplicar Dispositivo):** No boot frio sem calibração (`Spec 050`), `amp_standby == true` é preservado; após `set_audio_device` bem-sucedido, `active_device_id` é atualizado e `amp_standby` passa a `false` (`output_muted == false`), enquanto `swap_dsp_state` preserva `amp_standby` consistente com `AppState`.
- **CA-4 (3 Modos Explícitos de Entrada e Seleção de Canal):** `params.rs` e `RealtimeDspChain` aceitam `input.mode` em `{"instrument", "pc_line_in", "line"}` e `input.channel` em `{"left", "right", "sum"}`: `instrument` e `pc_line_in` aplicam ganho nominal `0,0 dB` (`±0,05 dB`), `line` aplica pad `-11,78 dB` (`±0,1 dB`), e `left`/`right`/`sum` roteiam o canal correspondente do buffer estéreo.
- **CA-5 (Integridade da UI Áudio, Registro e Banners Dinâmicos):** `Audio.ts` expõe os 3 modos e seletor de canal, não perde edições pendentes durante `emit()`, `store.ts` restaura `input_device` e `output_device`, `controls.registry.ts` não contém `'guitar'` nem `'main.input_boost'`, `AmpHead.ts` mostra `STANDBY` com polaridade clara e os banners dinâmicos aparecem apenas em estado ativo (`amp_standby === true` ou `input.mode === 'pc_line_in'`), preservando o SSIM da `Spec 048`.
- **CA-6 (Limitador Suave Zero-Latency na Saída):** `OutputSoftLimiter` mantém ganho unitário (`|out/in - 1.0| < 1e-4`) para sinal em `-12 dBFS`, comprime suavemente acima de `-1.0 dBFS` com THD menor que `clamp(-1.0, 1.0)` em `+3 dBFS`, mantém saída estritamente dentro de `[-1.0, 1.0]` em `+12 dBFS` e substitui `NaN`/`Inf` por `0.0`.
- **CA-7 (Suavização de Underrun e Drift no Duplex):** Em `RealtimeDuplexEngine`, eventos simulados de underrun e ajuste de drift não produzem salto amostral instantâneo (`|y[n] - y[n-1]| > 0.15` em sinal contínuo de `-12 dBFS`), aplicando micro-fade / crossfade sem latência adicional.
- **CA-8 (Escrita Atômica Segura):** `write_atomic_file` preserva o arquivo de destino íntegro caso a escrita ou etapa anterior falhe, sem remoção prévia destrutiva.

## Fora de escopo
- Modelagem física RLC de captadores, `input.gain_db` contínuo e `input.pad` dedicado (chaves planejadas — permanecem na **Spec 076**).
- Calibração espectral dos amplificadores (`Specs 054–058`), pedais (`Specs 059–075`) e gate (`Spec 065`), que serão tratados nas Fases 2–4.

## Invariantes tocadas
- **Zero-Alloc / Zero-Lock RT (`Spec 037`, `Spec 044`):** reforçada com remoção de `Mutex::try_lock` do callback de áudio.
- **Segurança na Inicialização (`Spec 050`):** boot frio sem calibração continua iniciando mudo/Standby até ação explícita do usuário.
- **Fidelidade Visual (`Spec 048`):** banners de alerta são condicionais e não alteram o layout base capturado pelos testes visuais.

## Testes que cobrem esta spec
- `crates/ampforge-dsp/tests/contract_093_lockfree_and_limiter.rs` → CA-1, CA-6
- `crates/sonix-desktop/tests/contract_093_rt_ownership_and_input.rs` → CA-2, CA-4, CA-7, CA-8
- `apps/sonix-app/src-tauri/tests/contract_093_device_apply_and_standby.rs` → CA-3
- `tests/contract/093-audio-input-standby-ui.test.js` → CA-5
