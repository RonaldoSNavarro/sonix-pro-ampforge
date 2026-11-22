---
spec: 026
titulo: O afinador lê o sinal já distorcido, assume 48 kHz, não alcança notas abaixo de 80 Hz e não silencia a saída
status: implementada
atualizado: 2026-11-09
---
# 026 — Afinador ao Vivo na Entrada da Guitarra

## Objetivo
Fazer o afinador da barra superior afinar a guitarra de verdade: ler o sinal limpo da entrada, com
a precisão do afinador do motor, e silenciar a saída enquanto aberto, como um afinador de pedaleira.

## Contexto
Origem: pedido do usuário em 2026-11-09 (todos os controles funcionando, parecidos com os reais).
Evidência:

- `#btnToggleTuner` só mostra a barra (`app.js:769-774`). A nota vem de autocorrelação **em
  JavaScript** (`updateStrobeTunerFromSignal`, `app.js:876-907`) sobre a forma de onda de
  **saída** (`audio.rs:266`) — já distorcida pelo amp, o que enche o sinal de harmônicos.
- O código assume `sampleRate = 48000` (`app.js:883`): com a placa em 44,1 kHz toda nota sai
  ≈ +147 cents errada (fator 48.000/44.100).
- Atrasos testados de 100 a 600 amostras, de 2 em 2 (`app.js:886-892`): só cobre 80–480 Hz, não
  alcança Dó grave de afinações baixas e a resolução de atraso limita a precisão a vários cents.
- Nada silencia a saída com o afinador aberto.
- O motor já tem `TunerProcessor` (Spec 008: ±0,1 cent, referência A4 430–450 Hz, modos
  `MUTE`/`BYPASS` com rampa de 5 ms — `tuner.rs:36-259`), fora da cadeia ao vivo.

## Comportamento esperado
1. **Afinador no motor, sobre a entrada.** O callback copia a entrada (depois do DC blocker, antes
   de boost, gate e amp) para um buffer lock-free; uma thread de análise roda `TunerProcessor` na
   taxa efetiva da sessão (Spec 018) e publica nota, cents, frequência e confiança.
2. **Faixa de guitarra.** Detecta de 38 Hz a 1,4 kHz (cobre afinações graves até Dó2 e o Mi6 da
   corda aguda no 24º traste).
3. **Saída durante a afinação.** `tuner.output` = `mute` (padrão, como afinador de pedaleira) ou
   `bypass`; rampa de 5 ms ao abrir e fechar (Spec 008 CA-4).
4. **UI só exibe.** Abrir/fechar envia `tuner.enabled`; a barra mostra a nota e move a agulha
   estroboscópica pelos cents do motor (`GET /api/tuner` ou campo na telemetria). Sem confiança
   suficiente, mostra "—".
5. **Parâmetros** (registro da Spec 014 §2): `tuner.enabled` (booleano, boot `false`),
   `tuner.output` (enum `mute`, `bypass`, boot `mute`).

## Critérios de aceite
- **CA-1:** Seno de 110 Hz na entrada com `amp_1.gain`=10: o afinador reporta Lá2 a ±0,5 cent (lê a
  entrada, não a saída distorcida).
- **CA-2:** Com a sessão em 44,1 kHz, seno de 440 Hz → Lá4 a ±0,1 cent.
- **CA-3:** Senos de 65,41 Hz (Dó2) e 1.318,5 Hz (Mi6) → notas corretas a ±1 cent; abaixo de 38 Hz
  não reporta nota.
- **CA-4:** Com `tuner.enabled=true` e `tuner.output=mute`, a saída chega a 0 em 5 ms ± 1 bloco e
  volta com a mesma rampa ao fechar; com `bypass`, a saída não muda.
- **CA-5:** Na UI, o botão do afinador envia `tuner.enabled`; a agulha é posicionada pelos cents do
  motor; `app.js` não contém mais a autocorrelação local (teste JS).
- **CA-6:** Com o `test_riff` (Spec 019) tocando uma nota, a primeira leitura estável (±1 cent da
  final) sai em ≤ 200 ms após o ataque e as leituras se atualizam ≥ 15 vezes por segundo.
- **CA-7:** Só ruído na entrada (−60 dBFS): nenhuma nota é reportada (a UI mostra "—").
- **CA-8:** O callback só copia amostras: 0 alocações e 0 locks com o afinador ligado (régua da
  Spec 018 CA-3).
- **CA-9:** Não-regressão — testes da Spec 008 continuam verdes.

## Fora de escopo
- Ajuste da referência A4, modo polifônico e temperamento na UI: o motor suporta (Spec 008), mas a
  barra não tem esses controles; spec futura.
- Looper ao vivo: não há controles de looper na UI; a correção do motor está na Spec 017.
- Afinador acionado por footswitch longo (Spec 008 CA-1): os footswitches da UI são cliques.

## Invariantes tocadas
- Áudio em tempo real (Spec 001/018): análise fora do callback; o callback só copia.
- Spec 008: o afinador passa a operar no caminho ao vivo sem mudar seu contrato.

## Testes que cobrem esta spec
Planejados (red → green): `crates/sonix-desktop/tests/contract_026_live_tuner.rs` (CA-1…CA-4,
CA-6…CA-9) e `tests/contract/026-afinador-ao-vivo-na-entrada.test.js` (CA-5, CA-7 lado UI).

## Notas
- 200 ms e 15 leituras/s são metas de usabilidade (hipóteses), editáveis com medição.
- Depende das Specs 018 (taxa efetiva, telemetria) e 019 (`test_riff`).
