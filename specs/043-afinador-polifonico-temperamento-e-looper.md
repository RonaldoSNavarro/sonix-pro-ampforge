---
spec: 043
titulo: Afinador polifônico devolve zeros fixos, temperamento não tem efeito e overdub em meia velocidade dobra o nível
status: implementada
atualizado: 2026-11-14
---
# 043 — Afinador (polifônico, temperamento, precisão) e looper (meia velocidade, undo, capacidade)

## Objetivo
O afinador e o looper fazem o que a Spec 008 promete, ou deixam de prometer o que não fazem.

## Contexto
Review manual das Specs 008, 017 e 026 (2026-11-14), por leitura de `crates/ampforge-dsp/src/tuner.rs` e `looper.rs`.

| # | Achado | Evidência |
|---|---|---|
| 1 | **Polifônico fixo.** `detect_polyphonic_status` devolve as 6 cordas com `cents_deviation: 0.0` fixo, sem analisar sinal. | `tuner.rs:261-279` |
| 2 | **Temperamento e strobe sem efeito.** `TunerTemperament::SweetenedGuitar` e `TunerMode::Strobe` são armazenados e nunca usados no cálculo. | `tuner.rs:10-14, 95-105, 244-258` |
| 3 | **Precisão sem medição.** O cabeçalho promete "±0,1 cent", mas não há teste nem medição que sustente o número. | `tuner.rs:2` |
| 4 | **Overdub em meia velocidade.** Com `half_speed`, o índice fica igual por 2 amostras e `overdub_buffer[idx] += input` soma duas amostras por posição (nível dobrado, sem filtro). | `looper.rs:520-541` |
| 5 | **Overdub apertado duas vezes.** `overdub()` em `Overdub` copia o buffer atual (já com a camada) para o `undo_buffer`, e o ponto de undo anterior se perde. | `looper.rs:445-453` |
| 6 | **Capacidade presa a 48 kHz.** A capacidade é fixa em `60 × 48000` amostras: a 96 kHz o looper tem 30 s, não 60. A UI anuncia "60s ESTÉREO", mas o motor é mono. | `looper.rs:353-355`; `ui/src/screens/Palco.ts` |

## Comportamento esperado
- O modo polifônico mostra o desvio real de cada corda, ou sai da API e da UI até ser implementado.
- O temperamento "sweetened" aplica deslocamentos por corda documentados com fonte, ou é removido.
- A precisão anunciada é a medida em teste.
- O overdub em meia velocidade escreve cada posição uma vez por passagem, com média das duas amostras de entrada.
- Apertar overdub de novo durante o overdub mantém o ponto de undo de antes da primeira camada da sessão.
- A capacidade é definida em segundos e dimensionada em `prepare` para a taxa atual. O rótulo da UI diz o que o motor faz (mono ou estéreo).

## Critérios de aceite
- **CA-1:** Num acorde sintético E2 A2 D3 G3 B3 E4 com a corda G desafinada em +20 cents, o modo polifônico relata a corda 4 com +20 ± 3 cents e as demais com |cents| ≤ 3. Se o modo não for implementado, `tuner.mode = "polyphonic"` é rejeitado com 400 e não aparece na UI.
- **CA-2:** Com o temperamento `sweetened`, a referência de cada corda desloca exatamente o valor da tabela documentada (fonte citada); com `equal`, o deslocamento é 0. Sem tabela com fonte, a opção é removida.
- **CA-3:** Numa senoide sintética de 110 Hz deslocada em +0,5 cent, a 48 kHz, o erro medido do afinador é ≤ X cent, onde X é o valor publicado no código e na UI (o número sai do teste).
- **CA-4:** Em meia velocidade, um overdub de uma passagem com entrada constante 0,1 sobre loop vazio deixa cada posição com 0,1 ± 1e-6 (não 0,2).
- **CA-5:** Gravar a base, fazer overdub (camada 1), apertar overdub de novo (camada 2) e depois undo + undo deixam o loop igual à base. O primeiro undo remove a camada 2 e o segundo, a camada 1, ou o comportamento de undo único fica documentado e testado.
- **CA-6:** Com `prepare(96000, …)`, o looper grava 60 s antes de passar sozinho para Play.
- **CA-7 (não-regressão):** Os contratos das Specs 008, 017, 026 e 029 continuam passando, exceto os CAs substituídos (listados nas Notas no fechamento).

## Fora de escopo
- Looper estéreo: depende da saída estéreo do motor (ver Spec 040, Fora de escopo).
- Quantização do looper ao tempo (BPM).

## Invariantes tocadas
- **Pitch detection desacoplada (Spec 026):** o modo polifônico, se implementado, roda fora do callback, como o YIN.
- **Zero-alloc:** o redimensionamento do looper acontece em `prepare`, nunca no callback.

## Testes que cobrem esta spec
<Preenchido no fechamento.>
