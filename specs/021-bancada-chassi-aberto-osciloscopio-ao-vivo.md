---
spec: 021
titulo: Válvulas, bias, sag, realimentação e osciloscópio da bancada só mudam a tela, não o som
status: implementada
atualizado: 2026-11-10
---
# 021 — Bancada do Chassi Aberto (Mod Shop) e Osciloscópio CRT ao Vivo

## Objetivo
Fazer cada ajuste da bancada do chassi aberto mudar o som do mesmo jeito que a modificação
correspondente muda um amplificador real — de forma explicável pelo circuito — e fazer o
osciloscópio mostrar o que o motor está realmente fazendo.

## Contexto
Origem: pedido do usuário em 2026-11-09 (controles reais e parecidos com os reais, seguindo a
skill). Evidência:

- Os 6 sliders da bancada (`sliderBias` 20–55 mA, `sliderSag` 0–10, `sliderNfb` 0–10,
  `sliderXformer` 0–10, `sliderPlateV` 180–340 V, `sliderAsym` 0–0,6 — `index.html:271-311`) só
  gravam em `state.circuitParams` e atualizam o brilho das válvulas (`app.js:716-733`).
- `#preampTubeSelect`, `#toneStackSelect` e `#rectifierSelect` não têm handler (0 ocorrências em
  `app.js`); `#powerTubeSelect` só troca dois rótulos (`app.js:737-740`).
- O osciloscópio desenha uma curva calculada **em JavaScript** (`nonLinearTubeStage`,
  `app.js:300-308`; `drawCrtScope`, `app.js:310+`), não dados do motor.
- No motor, `ChassisWorkbench` (`modshop.rs:81-130`) guarda todos esses valores, mas não está na
  cadeia ao vivo (`audio.rs:408-421`), e `transient_sag_compression` (`modshop.rs:278-286`) é um
  produto de constantes sem dinâmica de carga/recuperação.
- Rótulos com afirmação perceptiva ou de marca sem medição (skill §2.3/§2.6): "JJ ECC83S Warm
  Asymmetric", "Mullard EL34 Pentode (Brit Crunch)", "RCA 6L6GC Beam Tetrode (US Punch)",
  "Genalex KT88 High Headroom", "Vox EL84 Class-A Chime" (`index.html:240-252`).

Base técnica (skill §5.1, §5.3-5.5): o tipo de válvula importa pelo ponto de operação e pelo
circuito em volta, não como "seletor de caráter"; 12AX7 e ECC83 são o mesmo tipo elétrico; EF86
(pentodo) e EL84 (noval) não são trocas diretas de soquete num chassi de 12AX7/EL34 — escolhê-las
significa outra **variante de circuito**; bias frio em push-pull classe AB aumenta distorção de
cruzamento em sinais baixos; retificador a válvula tem resistência interna maior que diodos e
produz mais queda de B+ sob carga; realimentação negativa global reduz distorção e impedância de
saída (e, com ela, o realce na ressonância do alto-falante); o núcleo do transformador satura
primeiro em frequências baixas; clipping simétrico gera só harmônicos ímpares.

## Comportamento esperado
1. **Bancada na cadeia ao vivo.** Os parâmetros abaixo entram no modelo do amp ativo (Spec 020)
   e são aplicados com suavização (números) ou crossfade de 20 ms (enums). Registro da Spec 014 §2:

   | Chave | Tipo | Faixa / valores | Unidade | Efeito modelado |
   |---|---|---|---|---|
   | `amp_1.preamp_tube` | enum | `12ax7`, `12at7`, `ef86_variant` | — | ganho e curva do 1º estágio (EF86 = estágio de entrada pentodo, variante de circuito) |
   | `amp_1.power_tube` | enum | `el34`, `6l6gc`, `kt88`, `el84_variant` | — | curva de transferência e headroom do estágio de potência |
   | `amp_1.bias_ma` | número | 20–55 | mA de repouso por válvula | distorção de cruzamento e ponto de operação |
   | `amp_1.sag` | número | 0–10 | posição | profundidade da queda de B+ sob carga |
   | `amp_1.rectifier` | enum | `diode`, `gz34`, `5u4gb` | — | resistência interna da fonte e tempo de recuperação |
   | `amp_1.nfb` | número | 0–10 | posição | quantidade de realimentação negativa global |
   | `amp_1.xformer_sat` | número | 0–10 | posição | saturação do núcleo, dependente da frequência |
   | `amp_1.preamp_plate_v` | número | 180–340 | V | headroom e assimetria dos estágios do pré |
   | `amp_1.triode_asym` | número | 0–0,6 | — | assimetria do clipping (harmônicos pares) |
   | `amp_1.tonestack` | enum | `model`, `marshall`, `fender`, `vox`, `baxandall` | — | `model` = topologia do modelo (Spec 020); demais substituem |

   Valor no boot e ao trocar de modelo: o valor nominal do modelo (tabela nas Notas, com fonte).
2. **Válvulas como tipos elétricos.** `12ax7` cobre 12AX7/ECC83/7025: diferenças entre fabricantes
   não são modeladas sem dados medidos. Os rótulos da UI passam a mostrar tipo e dado técnico
   (ex.: "12AX7 / ECC83 — μ ≈ 100"), sem adjetivos perceptivos nem marcas.
3. **Fonte com dinâmica.** Sag é um modelo com estado: a queda de B+ segue a corrente média
   exigida com constante de carga e de recuperação; o retificador define resistência interna e
   constantes (tubo > diodo). Não é compressor fixo nem queda de volume (skill §5.4).
4. **Indicador "MOD".** Quando qualquer valor difere do nominal do modelo, a bancada mostra o
   selo "MOD"; trocar de modelo volta aos nominais do novo modelo.
5. **Osciloscópio com dados do motor.** `GET /api/scope?mode=waveform|transfer|tonestack` devolve:
   formas de onda de entrada (DI) e saída do amp; a curva entrada×saída do estágio não linear com
   os ajustes atuais (varredura estática calculada pelo motor fora do callback); e as curvas de
   magnitude do tone stack e da impedância do alto-falante (Spec 022). A UI só desenha.

## Critérios de aceite
- **CA-1:** `12at7` reduz o ganho de pequeno sinal do 1º estágio em relação a `12ax7` em ≥ 4 dB;
  com a mesma chave `12ax7` não há variante por fabricante (o enum não oferece marcas).
- **CA-2:** Com seno de 1 kHz a −30 dBFS na entrada do estágio de potência, THD com
  `bias_ma`=20 é ≥ 2× a THD com `bias_ma`=45 (distorção de cruzamento).
- **CA-3:** Rajada de 0,5 s com pico 0,5 seguida de seno de pico 0,05: a redução de ganho durante
  a rajada com `gz34` é ≥ 2× (em dB) a com `diode`; com `sag`=0 a redução é ≤ 0,1 dB; após a
  rajada o ganho volta a 0,5 dB do repouso com constante de tempo igual à nominal do retificador
  ± 20 %.
- **CA-4:** Com a mesma potência de saída RMS, THD com `nfb`=10 < THD com `nfb`=0; e o realce na
  frequência de ressonância da curva de impedância padrão (ganho em f_res − ganho em 1 kHz) é
  ≥ 2 dB maior com `nfb`=0 do que com `nfb`=10.
- **CA-5:** Com `xformer_sat`=10 e mesma amplitude de saída, THD(60 Hz) − THD(1 kHz) é ≥ 2× essa
  diferença com `xformer_sat`=0.
- **CA-6:** O nível de entrada que produz 1 % de THD na saída do pré é ≥ 3 dB menor com
  `preamp_plate_v`=180 do que com 340.
- **CA-7:** Com `triode_asym`=0 o 2º harmônico fica ≤ −60 dBc; com 0,6 ele sobe ≥ 10 dB.
- **CA-8:** `amp_1.tonestack=fender` no modelo `jcm800_2203` produz a resposta do tone stack Fender
  da Spec 020 CA-1 (±1 dB); `model` volta à do 2203.
- **CA-9:** Trocar `amp_1.model` restaura os nominais do novo modelo em `GET /api/state` e remove o
  selo "MOD"; alterar `bias_ma` faz o selo aparecer (teste JS).
- **CA-10:** `GET /api/scope?mode=transfer` com `triode_asym` 0 e 0,6 devolve curvas diferentes e
  a de 0 é simétrica (|f(x) + f(−x)| ≤ 1e-4); a UI desenha esses pontos e `nonLinearTubeStage`
  deixa de existir em `app.js` (teste JS).
- **CA-11:** Mudanças de enum (`preamp_tube`, `power_tube`, `rectifier`, `tonestack`) não geram
  descontinuidade (régua da Spec 020 CA-2); nenhuma alocação no callback.
- **CA-12:** Os rótulos de `#preampTubeSelect` e `#powerTubeSelect` não contêm marcas nem os termos
  "Warm", "Crunch", "Punch", "Chime", "High Headroom" (teste JS).
- **CA-13:** Não-regressão — testes das Specs 004, 016 e 020 continuam verdes.

## Fora de escopo
- Indicador de dissipação de placa ("red plate"): exige tensão de placa do estágio de potência, que
  não é controle da UI; spec futura.
- 6V6 e 12AY7 (existem em `modshop.rs`, não na UI); posição do tone stack (`ToneStackPosition`),
  `phase_inverter_drive`, `ghost_note_filtering_pct`: sem controle na UI.
- Circuit-level completo (WDF/MNA) dos estágios: spec própria após medir CPU (Spec 018).

## Invariantes tocadas
- Áudio em tempo real (Spec 001/018): curvas do osciloscópio calculadas fora do callback; estados de
  sag/retificador pré-alocados.
- Spec 004 (Mod Shop/CRT): passa a ter efeito sonoro; contratos existentes mantidos.
- Skill §2.3/§2.6: rótulos sem afirmação perceptiva não medida.

## Testes que cobrem esta spec
Planejados (red → green): `crates/ampforge-dsp/tests/contract_021_workbench_live.rs` (CA-1…CA-8,
CA-11, CA-13), `crates/sonix-desktop/tests/contract_021_scope_api.rs` (CA-10 lado Rust) e
`tests/contract/021-bancada-chassi-aberto-osciloscopio-ao-vivo.test.js` (CA-9, CA-10 lado UI, CA-12).

## Notas
- Limites numéricos (4 dB, 2×, 3 dB, 10 dB, −60 dBc, 20 %) são hipóteses de projeto, editáveis com
  medição; nenhum é medição de válvula real.
- Valores nominais por modelo (válvula, bias, retificador, tone stack) vêm da mesma tabela de fontes
  da Spec 020; sem fonte, marcados como hipótese.
- Depende das Specs 016 e 020; a curva de impedância usada no CA-4 é a padrão da Spec 022.
