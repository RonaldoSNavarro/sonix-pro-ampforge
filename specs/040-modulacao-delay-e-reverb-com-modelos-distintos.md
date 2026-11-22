---
spec: 040
titulo: Os 7 modelos de modulação, 6 de delay e 5 de reverb soam quase iguais e há controles sem efeito
status: implementada (AUD-30: TZF 1ms dry + inversão, Rotary 2 rotores LR4 800Hz, Optical Tremolo 5/60ms LDR, AC30 Vibrato/Tremolo no canal Vibrato)
atualizado: 2026-11-20
---
# 040 — Modulação, delay e reverb: cada modelo com o mecanismo do equipamento real

## Objetivo
Cada modelo de modulação, delay e reverb produz o fenômeno que dá nome a ele (flanger com realimentação, phaser com all-pass, tremolo com modulação de amplitude, mola com dispersão…), e todo controle exposto tem efeito.

## Contexto
Review manual das Specs 007, 024 e 032 (2026-11-14), por leitura de `crates/ampforge-dsp/src/fx/`.

| Bloco | Achado | Evidência |
|---|---|---|
| Modulação | `ModulationProcessor::process` ignora `model`: todos os 7 modelos (BBD Chorus, Tri-Chorus, Flanger, Phase 90, Tremolo, Univibe, Rotary) rodam o mesmo delay modulado de 8 ± 4 ms, sem realimentação, sem all-pass e sem modulação de amplitude. | `fx/modulation.rs:128-165` |
| Delay | Só `MultiHeadTapeEcho` tem comportamento próprio. BBD, Digital, Dual, Reverse e Ping-Pong caem no mesmo caminho de 1 cabeça. Reverse não inverte, e Ping-Pong é mono. | `fx/delay.rs:299-317` |
| Delay | A mudança do tempo salta o ponteiro de leitura: estala, e não há "glide" de fita. | `fx/delay.rs:294, 375-389` |
| Delay | O ducking liga e desliga o ganho em degrau (`duck_mult` 1 ↔ atenuação). | `fx/delay.rs:345-367` |
| Delay | As combs da mola interna têm comprimentos fixos em amostras (não escalam com a taxa). | `fx/delay.rs:99, 275-287` |
| Reverb | Os 5 modelos são 4 combs paralelas sem all-pass de difusão. Só o Shimmer difere, por um `tanh` no laço, que **não** gera oitava. `spring_tension` é armazenado e nunca usado. | `fx/reverb.rs:553-614, 20, 549-551` |
| Reverb | Os comprimentos das combs são fixos em amostras de 48 kHz, e a densidade de ecos é baixa (som metálico). | `fx/reverb.rs:461` |

## Comportamento esperado
- **Modulação:** cada modelo implementa seu mecanismo:
  - Chorus: delay modulado, 1 ou 3 vozes.
  - Flanger: delay curto com realimentação, varrendo os entalhes.
  - Phaser: cascata de all-pass de 1ª ordem com realimentação, com quantidade de estágios citada para o modelo.
  - Tremolo: modulação de amplitude, sem delay.
  - Univibe: all-pass com LFO não senoidal de fotocélula.
  - Rotary: modulação de amplitude + Doppler.
- **Delay:**
  - Fita: o tempo muda com glide.
  - BBD: perda de agudos por repetição.
  - Digital: repetição plana.
  - Reverse: trechos invertidos.
  - Ping-Pong: só fica disponível quando houver saída estéreo; até lá, aparece desabilitado com dica (regra do projeto).
- **Ducking:** o ganho é suavizado com ataque/release.
- **Reverb:**
  - Mola: dispersão com all-pass (chirp), e `spring_tension` muda o tempo da dispersão.
  - Plate e Hall: difusão por all-pass em série + combs ou FDN, com densidade crescente.
  - Shimmer: realimentação com pitch shift de +12 semitons, ou o modelo é removido.
- Todas as linhas de atraso escalam com a taxa.

## Critérios de aceite
- **CA-1:** O seletor de modelo da modulação existe na tabela única (`mod_2.model`), no registro da UI e no backend.
- **CA-2:** Tremolo com depth 10 e rate 4 Hz sobre uma senoide de 1 kHz produz envelope modulado a 4 ± 0,1 Hz, e o espectro da saída **não** tem as raias laterais de modulação de fase típicas de delay (energia fora de 1 kHz ± 10 Hz < −40 dB do pico).
- **CA-3:** Num flanger com feedback 70%, os picos da resposta a ruído branco ficam ≥ 10 dB acima dos vales; com feedback 0%, a diferença cai em pelo menos 6 dB.
- **CA-4:** O phaser com N estágios citados produz N/2 entalhes na resposta medida com o LFO congelado.
- **CA-5:** No delay digital, a 1ª repetição de uma senoide de 5 kHz tem o mesmo nível da entrada × feedback (±0,1 dB). No BBD, a mesma repetição fica pelo menos 3 dB abaixo.
- **CA-6:** O reverse delay com tempo de 500 ms, alimentado com uma rampa crescente, produz um trecho decrescente.
- **CA-7:** Mudar o tempo do delay de 300 para 400 ms com feedback 50% não gera degrau: a derivada máxima fica ≤ 1,5× a máxima antes da mudança.
- **CA-8:** No ducking, a variação de ganho entre amostras consecutivas nunca passa de 0,5 dB.
- **CA-9:** O RT60 medido de cada reverb fica dentro de ±15% do `decay_s` pedido, a 44,1, 48 e 96 kHz.
- **CA-10:** A 100 ms da cauda, a densidade de ecos (picos/s acima de −40 dB) do Plate é pelo menos 4× a da implementação atual. Mover `spring_tension` de 0 a 10 muda o atraso de grupo da dispersão em pelo menos 20%.
- **CA-11:** O Shimmer produz energia crescente em 2·f0 na cauda de uma senoide f0, ou o modelo é removido do enum, do registro e da UI.
- **CA-12 (não-regressão):** O spillover (Spec 007/024) mantém as caudas no bypass, e os 12 modos do RE-201 (Spec 032) continuam passando.

## Fora de escopo
- Saída estéreo do motor: Ping-Pong, pan e Tri-Chorus estéreo dependem dela, que vira spec própria.
- Reverb por convolução de salas reais.

## Invariantes tocadas
- **Todo controle visível funciona:** CA-1, CA-10 e CA-11 (o seletor e `spring_tension`).
- **Zero-alloc no callback:** todas as linhas são dimensionadas em `prepare` para a taxa máxima (96 kHz).

## Testes que cobrem esta spec
<Preenchido no fechamento.>
