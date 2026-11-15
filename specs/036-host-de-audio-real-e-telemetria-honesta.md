---
spec: 036
titulo: Dispositivos, modo de áudio, latência "medida" e portas MIDI exibidos são fixos no código
status: implementada
atualizado: 2026-11-14
---
# 036 — Host de áudio real e telemetria honesta (dispositivos, modo, latência, canal de entrada, MIDI)

## Objetivo
O usuário vê na tela o dispositivo, o modo, a latência e as portas MIDI que estão de fato em uso, e não valores escritos no código.

## Contexto
Review manual das Specs 012, 013, 018 e 028 (2026-11-14), feito por leitura de código.

| O que a UI/API mostra | De onde vem hoje |
|---|---|
| Lista de dispositivos | `enumerate_physical_devices` devolve dois dispositivos "Realtek" literais (`crates/sonix-desktop/src/audio.rs:514-541`). `/api/devices` devolve um JSON literal com "Realtek Microphone" (`api.rs:581-583`). |
| Modo `wasapi_shared_low_latency`, `raw_processing: true` | Literais (`audio.rs:874-879`). O stream é aberto com `cpal::default_host()` em modo compartilhado padrão; RAW, exclusivo e IAudioClient3 nunca são pedidos. |
| `measured_roundtrip_ms` | Soma calculada de períodos + 16 amostras (`audio.rs:866-872`). Nenhuma medição por loopback acontece. |
| `underruns`, `overruns`, `panics` | Sempre 0 (`audio.rs:888-890`). Nenhum contador é ligado ao ring. |
| Canal de entrada L/R/Soma | O stream real escolhe, **amostra a amostra**, o canal de maior módulo (`audio.rs:620-623`). Essa comutação não linear distorce quando os dois canais têm sinal. `InputChannel::convert_interleaved` existe, mas não é usado. |
| `/api/status` → `"backend":"WASAPI"` | Literal (`api.rs:545`). |
| Portas MIDI | `enumerate_input_ports` devolve duas strings fixas (`midi.rs:30-35`). As mensagens vão para uma `SpscCommandQueue` que a cadeia real não consome. |
| ASIO | Aparece no enum `AudioMode` e na UI, mas nenhum caminho abre ASIO. |

Os testes das Specs 013/018 passam porque exercitam `SimulatedAudioHost`, e não o host real.

## Comportamento esperado
- Dispositivos, taxa, período, modo e backend vêm do que o `cpal` (ou o backend escolhido) enumerou e abriu.
- `measured_roundtrip_ms` fica `null` até o usuário rodar a medição por loopback (Spec 018 CA-9). Estimativas aparecem em `estimated_roundtrip_ms`, rotuladas como estimativa na UI.
- Os contadores de underrun/overrun refletem ring vazio ou cheio no host real.
- O canal de entrada segue a escolha do usuário (`Left`, `Right`, `Sum`) com conversão linear.
- O modo exclusivo, o RAW e o ASIO funcionam de verdade ou aparecem **desabilitados** com a dica "em implementação — Spec 036" (regra "todo controle visível funciona").
- As portas MIDI vêm do sistema. CC e Program Change chegam ao motor pelo mesmo validador da tabela única (Spec 014).

## Critérios de aceite
- **CA-1:** `list_devices` devolve os nomes enumerados pelo backend de áudio. Um teste com host injetado contendo `["Focusrite USB"]` devolve exatamente esse nome; a string "Realtek" não aparece em nenhum literal de produção (teste de grep em `crates/` e `apps/`).
- **CA-2:** `audio_status.mode`, `backend`, `sample_rate`, `input_period_frames` e `output_period_frames` são lidos do stream aberto. Com o host simulado abrindo 44 100 Hz/480 frames em modo compartilhado, o status relata exatamente isso.
- **CA-3:** `raw_processing` só vale `true` se o modo RAW foi pedido e aceito. No modo compartilhado padrão, vale `false`.
- **CA-4:** Antes de qualquer medição por loopback, `measured_roundtrip_ms` é `null`, e a UI mostra a estimativa com o rótulo "estimada".
- **CA-5:** Ao forçar o esvaziamento do ring de entrada por 3 callbacks no host simulado, `underruns` vale 3.
- **CA-6:** Com L = senoide e R = −senoide: `Sum` produz silêncio (< −90 dBFS), `Left` produz a senoide com o THD da entrada (diferença < 0,1%), e `Right` produz a senoide invertida.
- **CA-7:** Se o backend não suporta o modo exclusivo ou ASIO, esses itens aparecem `disabled` na tela Áudio, com a dica do registro, e `set_audio_device` com esses modos devolve erro `mode_unavailable`.
- **CA-8:** `list_midi_ports` devolve as portas do sistema (host MIDI injetado em teste). Um CC 7 = 64 mapeado para `amp_1.master` faz o motor receber `amp_1.master ≈ 5,04` pela tabela única.
- **CA-9 (erro):** Sem nenhum dispositivo de entrada, o app inicia, mostra "nenhum dispositivo de entrada" e mantém a fonte `test_riff` disponível; `list_devices` devolve `[]`.
- **CA-10 (não-regressão):** Os contratos de `SimulatedAudioHost` (Specs 013 e 018) continuam passando.

## Fora de escopo
- Pedir IAudioClient3 / período mínimo do WASAPI direto pela API do Windows, se o `cpal` não expuser: entra como CA desabilitado (CA-7) e vira spec própria se for decidido implementar.
- MIDI Learn com UI: aqui só o caminho CC → tabela única.

## Invariantes tocadas
- **Todo controle visível funciona (regra do projeto):** os itens sem suporte passam a aparecer desabilitados, em vez de fingirem funcionar.
- **Zero-alloc no callback:** a conversão de canal usa buffer pré-alocado.

## Testes que cobrem esta spec
- `crates/sonix-desktop/tests/contract_036_audio_host_midi_telemetry.rs` (`CA-1` a `CA-10`)
- `tests/contract/035-036-host-ui-contracts.test.js` (`CA-4`, `CA-7`, `CA-9`)
