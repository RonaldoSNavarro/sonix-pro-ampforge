---
spec: 041
titulo: All-Buttons-In inverte o sinal, ratio e modo do gate não fazem nada e drives geram aliasing
status: implementada
atualizado: 2026-11-14
---
# 041 — Dinâmica (gate, 1176) e pedais de drive com o comportamento do circuito

## Objetivo
O compressor, o gate e os pedais de drive reagem como os equipamentos de referência, sem inversão de sinal, sem controles sem efeito e sem aliasing audível.

## Contexto
Review do commit `e1ffff3` e review manual das Specs 006, 023, 030 e 032 (2026-11-14), por leitura de código.

| # | Achado | Evidência |
|---|---|---|
| 1 | **All Buttons In não monotônico.** O shaper `d + 0,65d² − 0,2d³` com `d = 7,5x` não é monotônico: uma amostra comprimida de −0,2 vira +0,64; −0,5 vira ≈ +1 (retificação); +1,0 vira −1. O resultado são polaridade invertida, onda dobrada e DC. | `fx/dynamics.rs:475-480` |
| 2 | **Topologias sem efeito.** `CompressorTopology` (Dyna, 1176, LA-2A, VCA) só muda os tempos iniciais; `process` é idêntico para todas. | `fx/dynamics.rs:360-364, 426-488` |
| 3 | **Ratio do gate sem efeito.** `NoiseGateProcessor.ratio` é ajustável (`set_ratio`), mas `process` nunca o lê. O modo "Classic Expander" é um gate duro igual ao Natural. | `fx/dynamics.rs:189-193, 249-334` |
| 4 | **Modo Natural sem DI.** O modo "Intelligent Natural" deveria ler a DI limpa como sidechain, mas a cadeia chama `gate.process(boost, None, …)`, então o detector usa sempre a entrada do bloco. | `sonix-desktop/src/audio.rs:2549`, `dynamics.rs:264-267` |
| 5 | **Drives sem anti-aliasing.** Nenhum pedal tem oversampling ou ADAA. O RAT chega a ganho 40 e o DS-1 a 43 antes do clipping; a 48 kHz, os harmônicos acima de Nyquist dobram de volta. | `fx/drive.rs:387-553` |
| 6 | **TS808 sem caminho limpo.** No TS808 real, o estágio não inversor com diodos na realimentação soma o sinal limpo (ganho unitário) ao componente limitado, e o HP de ~720 Hz fica na realimentação. O modelo é um high-shelf em 480 Hz seguido de `tanh`, sem o caminho limpo. | `fx/drive.rs:336-340, 387-409` |
| 7 | **Filtro do RAT de 2ª ordem.** O filtro do RAT é um passa-baixa de 1ª ordem (resistor + pot com capacitor), mas o modelo usa um biquad de 2ª ordem (12 dB/oit). | `fx/drive.rs:346-348` |

## Comportamento esperado
- O All Buttons In aumenta a compressão e acrescenta distorção **monotônica**, sem inverter a polaridade e com DC desprezível.
- Só há topologias de compressor cujo processamento seja realmente distinto. As que não forem implementadas saem do enum.
- O gate em modo expander aplica `ratio` abaixo do limiar. O modo Natural detecta pela DI limpa (antes do drive) quando houver DI.
- Os pedais de drive usam oversampling (ou ADAA) no estágio não linear.
- O TS808 tem o caminho limpo de ganho unitário somado ao limitado, e o RAT tem filtro de 1ª ordem. As referências de circuito ficam citadas em `docs/`.

## Critérios de aceite
- **CA-1:** Com `comp_2.ratio = "all"`, a curva estática de transferência (entrada −1…+1) é monotônica não decrescente, `sign(saída) = sign(entrada)` para toda entrada com |x| > 1e-3, e o DC de uma senoide de 1 kHz na saída fica abaixo de −60 dBFS.
- **CA-2:** Toda variante de `CompressorTopology` produz, para o mesmo burst de teste, envelopes de ganho distintos (diferença RMS ≥ 1 dB). Uma variante sem processamento próprio não compila (é removida).
- **CA-3:** No modo expander com ratio 1:4 e limiar −40 dB, um sinal estável a −50 dB sai a −80 ± 2 dB, e a −45 dB sai a −60 ± 2 dB.
- **CA-4:** No modo Natural com o drive ligado, o estado do gate é decidido pela DI limpa. Com um sinal cuja DI está 3 dB abaixo do limiar, mas cuja saída do drive está acima, o gate fica fechado; no modo Classic, nas mesmas condições, fica aberto.
- **CA-5:** Para cada pedal, com ganho 10 e uma senoide de 5 kHz a −6 dBFS, a 48 kHz, os componentes fora dos harmônicos de 5 kHz ficam abaixo de −50 dBFS.
- **CA-6:** No TS808 com Drive em 0 e Tone em 5, uma senoide de 200 Hz a −30 dBFS sai com THD < 1% e ganho entre 0 e +3 dB (caminho limpo presente). Com Drive em 10, o nível do caminho limpo continua presente (a correlação com a entrada é > 0,3).
- **CA-7:** O filtro do RAT, medido duas oitavas acima do corte, cai 12 ± 1,5 dB (1ª ordem: 6 dB/oit).
- **CA-8 (não-regressão):** Os contratos das Specs 023, 030 e 032 passam, exceto os CAs substituídos por esta spec (listados nas Notas no fechamento).

## Fora de escopo
- A interação do Fuzz Face com a impedância da guitarra (exige um modelo de entrada de instrumento): candidata a spec própria.
- LA-2A com fotocélula T4 completa: se não for implementado, sai do enum (CA-2).

## Invariantes tocadas
- **Todo controle visível funciona:** CA-3 (ratio) e CA-4 (modo).
- **Zero-alloc no callback:** os buffers de oversampling dos pedais são pré-alocados.

## Testes que cobrem esta spec
- `crates/ampforge-dsp/tests/contract_041_dynamics_and_drives.rs` (`CA-1`..`CA-3`, `CA-5`..`CA-7`)
- `crates/sonix-desktop/tests/contract_041_gate_di.rs` (`CA-4`)
- `crates/ampforge-dsp/tests/contract_006_dynamics_drive_eq.rs` (8/8), `contract_023_drive_808.rs` (7/7), `contract_030_noise_gate.rs` (5/5), `contract_032_pedals_fx_cabs.rs` (8/8)
