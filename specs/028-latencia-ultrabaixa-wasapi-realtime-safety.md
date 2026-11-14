---
spec: 028
titulo: Atraso audível no áudio por streams separados com Mutex, fila de 50ms e falta de modo exclusivo/IAudioClient3
status: implementada
atualizado: 2026-11-13
---
# 028 — Latência Ultrabaixa Real (WASAPI Exclusive / IAudioClient3) e Callback 100% Lock-Free / Zero-Alloc

## Objetivo
Eliminar o atraso audível percebido pelo guitarrista na entrada/saída de áudio do Windows, removendo alocações, locks e acúmulo de buffer nos callbacks de áudio e ativando modos nativos de baixa latência (WASAPI Exclusive / IAudioClient3 Shared Low Latency / ASIO).

## Contexto
Origem: relato do usuário ("o audio ainda tem atrazo") e inspeção do host em `crates/sonix-desktop/src/audio.rs` e `main.rs`:

1. **Configuração padrão de alta latência (`audio.rs:581-594`):** `DesktopAudioHost::start_realtime_wasapi_duplex` usa `cpal::default_host()` com `in_device.default_input_config()` e `out_device.default_output_config()`, sem definir `BufferSize::Fixed(...)`, sem tentar WASAPI Exclusive e sem usar `IAudioClient3` (`InitializeSharedAudioStream`). No Windows, o modo Shared padrão entrega buffers de 10 ms a 20 ms+ por direção (480–960+ frames).
2. **Acúmulo de até 50 ms no ring buffer duplex (`audio.rs:818-825`):** `RealtimeDuplexEngine::pop_samples_for_processing` só drena o excesso quando `occupied > 2400` amostras (50 ms a 48 kHz), e drena apenas até `960` amostras (20 ms). Somado aos buffers do driver de entrada e saída, o round-trip ultrapassa facilmente 40–80 ms.
3. **Violações de Realtime Safety nos callbacks (`audio.rs:607-752`):**
   - Alocação de heap em todo callback: `Vec::with_capacity(num_frames)` (`audio.rs:609, 640`) e `vec![0.0f32; num_frames]` (`audio.rs:680-681, 718-719`).
   - `Mutex::lock()` dentro dos callbacks de entrada e saída via `RealtimeDuplexEngine` (`audio.rs:773-774, 801, 817`).
   - `dsp_chain_out.try_lock()` (`audio.rs:685, 723`) sobre `Arc<Mutex<RealtimeDspChain>>`: sempre que o servidor HTTP trava o mutex (ex.: `/api/tuner`, `/api/scope`, `/api/params`), o callback de saída pula o processamento DSP e copia o sinal seco (`mono_out.copy_from_slice(&mono_in)`), causando estalos/vazamento de sinal limpo.
   - `tele_out.push_waveform(&mono_out)` (`audio.rs:702, 741`) adquire `Mutex` (`audio.rs:1937`).
   - `eprintln!` síncrono nos callbacks de erro (`audio.rs:629, 659, 707, 747`).
4. **Telemetria desconectada (`main.rs:71`, `audio.rs:700-702`):** `main.rs` chama `bind_port_with_telemetry` sem passar `audio_telemetry`, fazendo `/api/audio/status` retornar valores hardcoded falsos (`"wasapi_shared"`, `128/128`, `2.5 ms`). Além disso, `cpu_usage_pct` nunca é medido no callback de saída.

## Comportamento esperado
1. **Seleção de modo de baixa latência no Windows:**
   - Ordem de prioridade no modo `auto`: (1) ASIO (se selecionado/disponível com interface); (2) WASAPI Exclusive event-driven (`AUDCLNT_SHAREMODE_EXCLUSIVE`, `RAW`) na menor taxa comum (preferencialmente 48 kHz) e menor período suportado pelo hardware (64 ou 128 frames); (3) WASAPI Shared Low Latency via `IAudioClient3` com `FundamentalPeriodInFrames` / menor período (≤ 128 frames) e `AUDCLNT_STREAMOPTIONS_RAW`; (4) WASAPI Shared padrão com registro explícito em `fallback_reason`.
2. **Callbacks 100% Zero-Alloc e Lock-Free:**
   - Pré-alocação de todos os buffers de trabalho (`mono_buf`, `mono_in`, `mono_out`) fora dos callbacks (capacidade ≥ 4096 frames).
   - Substituição de `Mutex<HeapRb>` no `RealtimeDuplexEngine` por `LockFreeSpscRingBuffer` atômico (ou anel SPSC `ringbuf` sem `Mutex`), mantendo ocupação alvo de **1 período** (teto de drenagem = `2 * period_frames`, ex.: 256 frames = 5.3 ms a 48 kHz, com interpolação suave anti-deriva).
   - Substituição de `Arc<Mutex<RealtimeDspChain>>` no caminho de áudio por **fila SPSC lock-free de comandos/parâmetros** (`CommandQueue` da Spec 001): o servidor HTTP enfileira atualizações de parâmetros e o callback de áudio consome no início do bloco sem nunca usar `Mutex`/`try_lock`.
   - Ring buffer SPSC lock-free para waveform e métricas atômicas (`cpu_avg_pct`, `cpu_peak_pct`, `underruns`, `overruns`) medidas no callback de saída.
3. **Prioridade de thread de áudio:** Registrar as threads de captura e reprodução no Windows com MMCSS (`AvSetMmThreadCharacteristicsW("Pro Audio")`).
4. **Telemetria real exposta:** `main.rs` conecta `AudioStatusTelemetry` real ao `LocalAppServer` para que `GET /api/audio/status` informe o modo real, taxa real, período real (`input_period_frames` / `output_period_frames`), latências do driver e carga real de CPU.

## Critérios de aceite
- **CA-1:** Em execução com `RealtimeDspChain` ativo e carga HTTP concorrente (`POST /api/params`, `GET /api/tuner`, `GET /api/scope` a 60 Hz), o callback de áudio executa **0 alocações de heap**, **0 locks (`Mutex`)** e **0 chamadas de bypass por contenção** (0 glitches em 1000 blocos consecutivos).
- **CA-2:** O `RealtimeDuplexEngine` mantém ocupação média em regime estacionário entre `0.5 * period_frames` e `2.0 * period_frames` (nunca acumulando ≥ 2400 amostras / 50 ms como antes).
- **CA-3:** A latência algorítmica da cadeia + buffering duplex com `period_frames = 128` a `48000 Hz` em modo `wasapi_exclusive` ou `wasapi_shared_low_latency` é `<= 10.0 ms` no caminho digital (driver + ring buffer + DSP), e o valor real é reportado em `GET /api/audio/status`.
- **CA-4:** `cpu_avg_pct` e `cpu_peak_pct` em `/api/audio/status` e `cpu_usage_pct` em `/api/telemetry` refletem o tempo real medido por bloco dividido pelo orçamento do bloco (`period_frames / sample_rate`), sendo `> 0.0%` quando a cadeia DSP está ativa.
- **CA-5:** Se o hardware recusar WASAPI Exclusive (ex.: dispositivo em uso ou taxas in/out diferentes), o host recua automaticamente para `wasapi_shared_low_latency` (ou `wasapi_shared`) sem panic e preenche `fallback_reason` com texto explícito.
- **CA-6:** Caso de erro — se o dispositivo de entrada ou saída desconectar durante a reprodução, o callback preenche o buffer de saída com silêncio (`0.0`), incrementa contador de erro sem `eprintln!` na thread de áudio e sinaliza reconexão.
- **CA-7:** Não-regressão — todos os testes de contrato da Spec 001 e Spec 018 continuam verdes.

## Fora de escopo
- Integração de SDK proprietário Steinberg ASIO que exija redistribuição de headers sem licença: se o usuário selecionar ASIO, depende de driver ASIO instalado no sistema (via `cpal` feature `asio` quando habilitada no build).
- Resampling assíncrono de alta ordem (sinc polifásico) para placas com taxas travadas diferentes entre mic e saída: usa-se a interpolação anti-deriva existente até spec dedicada caso necessário.

## Invariantes tocadas
- **Invariante Zero-Alloc / Zero-Lock no Callback de Áudio (AGENTS.md / Spec 001 / Spec 018 / Spec 026):** reforçada e finalmente cumprida integralmente no caminho real de `sonix-desktop/src/audio.rs`.

## Testes que cobrem esta spec
Implementados (green): `crates/sonix-desktop/tests/contract_028_low_latency_realtime_safety.rs` (CA-1…CA-7, 7/7 ok).
