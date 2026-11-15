---
spec: 038
titulo: Convolução de IR é O(N) por amostra e estoura o buffer, e modelo de microfone e ressonância não mudam o som
status: implementada
atualizado: 2026-11-14
---
# 038 — Gabinete: convolução eficiente de IR, microfones e ressonância com efeito real

## Objetivo
IRs de até 500 ms tocam sem estourar a CPU nem corromper o som, e todo controle do gabinete muda a saída no sentido do equipamento real.

## Contexto
Review manual da Spec 005/022 (2026-11-14), por leitura de `crates/ampforge-dsp/src/cab.rs`.

1. **Convolução direta O(N).** `process_ir_convolution` (`cab.rs:836-885`) faz N multiplicações por amostra. Com IR de 500 ms a 48 kHz (24 000 taps), são cerca de 1,15 G MAC/s. No crossfade, o custo dobra, e com `cab_2` ativo, também.
2. **Buffer circular menor que a IR.** `ir_circ_buf` tem 32 768 posições (`cab.rs:554`). O limite é 500 ms na taxa **original**, e o resample para 96 kHz pode gerar até 48 000 taps. O índice `(pos + len − k) % len` com `k ≥ 32 768` lê amostras erradas: a resposta dobra no tempo.
3. **Modelo de microfone sem efeito.** `MicSlotConfig.model` é gravado (`audio.rs:2161,2191`), mas nenhum filtro o usa (`cab.rs:742-824`). SM57, MD421, R121, U87… soam iguais. Isso viola a regra "todo controle visível funciona", e o inventário (`docs/ui-desktop/inventario-controles.md`) marca como OK.
4. **Ressonância sem efeito no áudio.** `set_resonance_hz` só altera `impedance_curve` (`cab.rs:587-589`), que é usada no badge e no `calculate_impedance_peak`, nunca no processamento. O amp usa uma ressonância fixa em 98 Hz (`amp.rs:1201-1202`), independente do gabinete.
5. **Microfones ignorados com IR.** Com a IR ativa, `process` só convolui (`cab.rs:733-736`); posição, distância, nível, fase e mute dos microfones são ignorados. A API já devolve `ignored_reasons: ir_active`, mas a UI mostra os controles como ativos.
6. **IR estéreo.** Só o primeiro canal é lido (`cab.rs:443-463`), sem aviso.
7. **Taxa do loader fixa.** O upload valida com `WavIrLoader::new(48000.0)` fixo (`api.rs:2689`).

## Comportamento esperado
- Convolução particionada (FFT uniforme ou não uniforme) com a primeira partição direta, para não acrescentar latência, e custo por bloco sub-linear no tamanho da IR.
- Cada modelo de microfone tem uma resposta (filtro mínimo-fase ou FIR curto) derivada de curva publicada pelo fabricante, citada em `docs/`.
- `cab_1.resonance_hz` desloca a ressonância de baixa frequência que o amp enxerga como carga. Ou seja, a curva de impedância do gabinete alimenta o filtro de ressonância do amp (acoplamento real).
- Com a IR ativa, os controles de microfone aparecem desabilitados, com a dica "IR ativa — microfonação vem da IR".
- IR estéreo: a resposta da API informa `channels_used`, e o comportamento escolhido (canal 1 ou soma) é documentado.
- O loader usa a taxa atual do motor.

## Critérios de aceite
- **CA-1:** Uma IR de 48 000 taps (500 ms a 96 kHz) processada em blocos de 128 produz saída igual à convolução de referência (offline, f64), com erro ≤ −80 dB em relação ao pico.
- **CA-2:** Um impulso na entrada gera a primeira amostra não nula da saída no mesmo índice do primeiro tap não nulo da IR (latência adicional = 0).
- **CA-3:** No benchmark do repositório, o tempo por bloco de 128 com IR de 24 000 taps é ≤ 15% do tempo com o FIR direto atual, nas mesmas condições. O valor medido é registrado no PR.
- **CA-4:** Na mesma posição e distância, a energia de 6–10 kHz da saída com `r121` é pelo menos 3 dB menor que com `sm57` (ribbon mais escuro), e a resposta com `md421` difere da do `sm57` em ≥ 1 dB em alguma banda de 1/3 de oitava entre 100 Hz e 8 kHz. As curvas usadas têm fonte citada.
- **CA-5:** Mover `cab_1.resonance_hz` de 70 para 130 Hz desloca o pico de baixa frequência da resposta (sweep pela cadeia amp + gab) para cima em pelo menos 30 Hz.
- **CA-6:** Com `cab_1.ir_enabled = true`, os controles `cab_1.mic*` ficam `disabled` na UI, com a dica do registro. Ao desligar a IR, eles voltam a ser habilitados.
- **CA-7:** Um upload de WAV estéreo devolve `channels_used` na resposta, e a saída corresponde ao comportamento documentado.
- **CA-8:** Com o motor a 44,1 kHz, uma IR enviada a 48 kHz é reamostrada para 44,1 kHz (o comprimento em amostras escala por 44 100/48 000 ± 1).
- **CA-9 (erro):** Um WAV truncado ou corrompido devolve 400, e a IR anterior continua tocando sem interrupção.
- **CA-10 (não-regressão):** Os contratos das Specs 005, 017 e 022 continuam passando.

## Fora de escopo
- Modelo não linear do alto-falante (excursão, compressão térmica): a IR é linear por definição; vira spec própria se for pedido.
- Mixagem multi-mic com pan estéreo: a saída do motor continua mono.

## Invariantes tocadas
- **Todo controle visível funciona:** CA-4, CA-5 e CA-6.
- **Zero-alloc no callback:** as partições FFT e os buffers são pré-alocados na troca de IR, fora da thread de áudio (Spec 037).

## Testes que cobrem esta spec
- `crates/ampforge-dsp/tests/contract_038_cab_ir_mics_resonance.rs` → CA-1 (`ca1_48000_taps_96khz_ir_matches_f64_reference_under_minus_80db`), CA-2 (`ca2_zero_additional_latency_on_ir_convolution`), CA-3 (`ca3_partitioned_convolution_is_under_15_percent_of_direct_fir`, medido: `80603 ns/blk` vs `4714001 ns/blk`, `1.71%`), CA-4 (`ca4_mic_models_have_distinct_frequency_responses`), CA-5 (`ca5_cab_resonance_hz_shifts_amp_cab_low_frequency_peak`), CA-8/CA-9 (`ca8_and_ca9_resample_48k_to_44k1_and_reject_truncated_wav`)
- `crates/sonix-desktop/tests/contract_038_cab_api.rs` → CA-7/CA-9 (`ca7_and_ca9_stereo_wav_returns_channels_used_and_truncated_returns_400`)
- `tests/contract/038-cab-ir-ui.test.js` → CA-6 (`cab_1.mic*` desabilitados com dica quando `cab_1.ir_enabled = true`)
- `crates/ampforge-dsp/tests/contract_005_virtual_cab.rs` & `crates/sonix-desktop/tests/contract_022_cab_api.rs` → CA-10 (não-regressão)
