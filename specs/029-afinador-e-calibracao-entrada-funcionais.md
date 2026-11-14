---
spec: 029
titulo: Afinador incompatível com JSON da UI e calibração de entrada sem endpoint nem handlers
status: implementada
atualizado: 2026-11-13
---
# 029 — Afinador Funcional End-to-End e Calibração de Entrada Ao Vivo

## Objetivo
Fazer o afinador (`◎ Tuner`) e o assistente de calibração de entrada (`⚖ Calibrar Entrada`) funcionarem de ponta a ponta entre a interface web, o servidor HTTP local e o motor de áudio em tempo real.

## Contexto
Origem: relato do usuário ("o afinador não funciona, calibrar entrada também não funciona") e evidências literais no código:

1. **Afinador quebrado por mismatch de contrato JSON (`server.rs` vs `app.js`):**
   - `GET /api/tuner` (`server.rs:569-576`) retorna:
     `{"note": res.note_name, "frequency_hz": res.frequency_hz, "cents": res.cents_deviation, "in_tune": ..., "active": ..., "muted": ...}`.
   - `updateStrobeTunerFromSignal()` (`app.js:1136-1145`) lê `data.note_name` e `data.cents_deviation`, que são sempre `undefined`, deixando o display preso e a agulha imóvel.
   - `TunerProcessor::detect_pitch` (`tuner.rs:134`) descarta qualquer sinal com `rms < 0.005` (-46 dBFS). Como `audio.rs:1728` copia o sinal **antes** do `input_boost`, captadores passivos ligados em entrada P2/onboard frequentemente entram abaixo de -46 dBFS na sustentação da nota.
   - `tuner_input_accum` (`audio.rs:1078`) tem apenas `2048` amostras: a 96 kHz, `max_lag = 1024` não alcança a corda Mi2 (82,41 Hz exige 1165 amostras de lag).

2. **Calibração de Entrada completamente ausente no servidor e na UI:**
   - `design-system/index.html:88-135` define `#btnCalibrateInput`, `#inputCalibrationWizard`, `#btnStartCalib` e `#btnCloseCalib`, mas **não existe nenhum `addEventListener`** para nenhum desses IDs em `design-system/app.js` (0 ocorrências de `Calib` em `app.js`).
   - `crates/sonix-desktop/src/server.rs` possui `CalibrationStore` no import (`server.rs:11`), porém **não existe nenhuma rota `/api/input/calibrate*`** no `match path.as_str()` (`server.rs:472-962`), e `main.rs:71` nem sequer passa um `CalibrationStore` para `LocalAppServer`.

## Comportamento esperado
1. **Afinador funcional em todas as taxas (44.1 / 48 / 96 kHz):**
   - Buffer circular lock-free SPSC dedicado de 4096 amostras alimentado no callback antes do boost/gate/amp (após DC blocker).
   - `TunerProcessor` reduz o limiar mínimo de RMS para `-64 dBFS` (`0.00063`) mantendo o critério de periodicidade YIN (`CMND <= 0.35`) para não disparar em ruído branco/hum puro.
   - `GET /api/tuner` retorna `note`, `cents`, `frequency_hz`, `in_tune`, `active`, `muted` e a UI (`app.js`) lê exatamente esses campos, além de exibir um botão rápido de modo de saída (`Mute` / `Bypass`) na barra do afinador.
2. **Assistente de Calibração de Entrada funcional:**
   - Clicar em `#btnCalibrateInput` abre `#inputCalibrationWizard` no passo 1 (silêncio, 3s).
   - Clicar em `#btnStartCalib` dispara a captura ao vivo via `POST /api/input/calibrate` (fase `"silence"` por 3s, depois avança automaticamente para passo 2 `"strum"` por 5s) usando amostras reais da entrada capturada (via ring buffer lock-free).
   - Ao concluir o passo 2, o servidor executa `calibration::calibrate_input(&noise_samples, &strum_samples, sample_rate)`, aplica imediatamente `proposed_boost` (`input_boost`) e `proposed_gate_threshold_db` (`gate.threshold_db`) na cadeia DSP, salva no `CalibrationStore` vinculado ao dispositivo ativo e exibe no passo 3 o resultado (`boost`, `gate_threshold_db`, `noise_floor_rms_db`, `raw_peak_db` e diagnóstico: `Ok`, `InputTooHot`, `InputTooLow`, `Hum50Hz`, `Hum60Hz`).
   - Clicar em `#btnCloseCalib` fecha o modal e restaura o estado normal do assistente.

## Critérios de aceite
- **CA-1:** `GET /api/tuner` retorna JSON com campos `"note"`, `"cents"`, `"frequency_hz"`, `"in_tune"`, `"active"`, `"muted"`, e `app.js` atualiza `#tunerNoteText` e `.strobe-center-needle` usando `data.note` e `data.cents` (sem referenciar `data.note_name` ou `data.cents_deviation`).
- **CA-2:** Com seno de `82.41 Hz` (Mi2) a `-55 dBFS` (RMS ≈ 0.00126) e taxa `96000 Hz`, o afinador detecta `E2` com erro `<= ±1.0 cent`; com ruído branco a `-60 dBFS`, retorna `"---"`.
- **CA-3:** Clicar em `#btnCalibrateInput` torna `#inputCalibrationWizard` visível; clicar em `#btnCloseCalib` o oculta.
- **CA-4:** `POST /api/input/calibrate` aceita `{ "phase": "silence" | "strum" }` (com `X-Sonix-Token` válido), coleta amostras reais do buffer de entrada da cadeia ao vivo e, ao finalizar `"strum"`, retorna `200` com `proposed_boost`, `proposed_gate_threshold_db`, `diagnostic`, `noise_floor_rms_db` e `raw_peak_db`.
- **CA-5:** Após `POST /api/input/calibrate` concluir, `RealtimeDspChain` e `/api/state` passam a usar o `proposed_boost` e `proposed_gate_threshold_db` calculados, e o `CalibrationStore` armazena o perfil para o dispositivo atual.
- **CA-6:** Caso de erro — requisição `POST /api/input/calibrate` com `phase` inválido retorna `400`; sem `X-Sonix-Token` retorna `401`; se não houver sinal na fase `"strum"` (`raw_peak_db < -42.0`), retorna diagnóstico `"input_too_low"` com `proposed_boost = 8.0`.
- **CA-7:** Zero alocações e zero locks no callback de áudio durante captura para afinador ou calibração.
- **CA-8:** Não-regressão — testes das Specs 008, 019 e 026 continuam verdes.

## Fora de escopo
- Afinador polifônico simultâneo de 6 cordas na UI (o motor tem stub `detect_polyphonic_status` na Spec 008, mas a barra atual é cromática/estroboscópica).
- Persistência em disco do perfil de calibração entre reinicializações (spec futura; nesta spec fica em `CalibrationStore` durante a sessão e visível em `/api/state`).

## Invariantes tocadas
- Spec 001/018/028 (Zero-Alloc / Zero-Lock no callback): amostras para afinador e calibração são copiadas apenas via SPSC lock-free pré-alocado.
- Spec 015 (Segurança local): endpoints `POST /api/input/calibrate` exigem validação de Origin/Host e `X-Sonix-Token`.

## Testes que cobrem esta spec
Implementados e verdes (red → green): `crates/sonix-desktop/tests/contract_029_tuner_and_calibration.rs` (CA-1, CA-2, CA-4…CA-8) e `tests/contract/029-afinador-e-calibracao-entrada.test.js` (CA-1, CA-3 + orquestração Rust). Não-regressão verificada em `contract_008_tuner_looper`, `contract_019_input_calibration`, `contract_026_live_tuner`, `contract_028_low_latency_realtime_safety` e `contract_033_phase1_api_dispatch`.
