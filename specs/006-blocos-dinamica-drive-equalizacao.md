---
spec: 006
titulo: Blocos de dinâmica (Noise Gate, Compressor), Overdrive/Fuzz White-Box e Equalizadores (RTA)
status: aprovada
atualizado: 2026-11-08
---
# 006 — Blocos de dinâmica (Noise Gate, Compressor), Overdrive/Fuzz White-Box e Equalizadores (RTA)

## Objetivo
Fornecer ao músico o tratamento completo de pré e pós-amplificação para controle de ruído, sustentação dinâmica, saturação de pedais de overdrive/distortion/fuzz com troca de diodos no Mod Shop e escultura espectral com analisador RTA em tempo real.

## Contexto
Pedaleiras profissionais dependem de blocos de condicionamento antes do amplificador (Noise Gate inteligente referenciado na entrada DI limpa, compressores de pedal e overdrives clássicos empurrando o primeiro triodo) e blocos de precisão após o gabinete (compressores de estúdio FET/Optical/VCA e equalizadores paramétricos/gráficos).

Fontes da verdade e evidências no repositório:
- `docs/01-SRS-Levantamento-de-Requisitos.md:139-147` — Define **RF-018** (Bloco Noise Gate / Input Gate), **RF-019** (Bloco Compressor em 4 topologias), **RF-020** (Bloco Overdrive / Distortion / Fuzz com seleção de diodo de clipping no Mod Shop e 6 modelos base) e **RF-021** (Bloco Equalizador em 3 modos com RTA em tempo real).
- `docs/02-Design-System-Specification.md:45-51` — Define os tokens semânticos `--led-gate` (`#00E676`), `--led-comp` (`#00B0FF`), `--led-drive` (`#FF9100`) e `--led-eq` (`#1DE9B6`).
- `design-system/app.js:44-45,49,54` — O protótipo instancia os slots `blk_gate` (`Natural Gate`), `blk_drive` (`Screamer 808`), `blk_eq_1` (`8-Band Studio`) e `blk_comp_2` (`FET 1176 Comp`) no Grid, mas ainda não implementa seus painéis de controle dedicados nem seus algoritmos de processamento no motor DSP.

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Bloco `GATE` (**RF-018**) | Card `blk_gate` no Grid (`design-system/app.js:44`) | Modos `Intelligent Natural Gate` (sidechain no DI limpo) e `Classic Expander Gate`, controles `Threshold`, `Ratio`, `Attack`, `Hold`, `Release` e medidor de Gain Reduction (dB) |
| Bloco `COMP` (**RF-019**) | Card `blk_comp_2` no Grid (`design-system/app.js:54`) | 4 topologias (`Pedal Comp Ross/Dyna`, `Studio FET 1176`, `Optical LA-2A`, `VCA Master Bus`), controle `Parallel Mix (Dry/Wet)` e medidor VU de redução de ganho |
| Bloco `DRIVE` (**RF-020**) | Card `blk_drive` no Grid (`design-system/app.js:45`) | 6 modelos base (`Green Screamer 808`, `Golden Klone Centaur`, `Black Rodent RAT`, `Orange Boss DS`, `Silicon/Germanium Fuzz Face`, `Big Muff Pi`) + seleção de diodo no Mod Shop (`Silicon Symmetrical`, `Asymmetrical Si+LED`, `Germanium 1N34A`, `MOSFET`, `Hard Op-Amp`) |
| Bloco `EQ` (**RF-021**) | Card `blk_eq_1` no Grid (`design-system/app.js:49`) | 3 modos (`8-Band Parametric EQ` com RTA em tempo real, `5-Band Boogie Graphic EQ` em 80/240/750/2200/6600 Hz e `10-Band Pedal Graphic EQ`) |

**Alternativas descartadas:**
1. *Noise Gate rastreando apenas o sinal na posição onde o bloco está inserido:* descartada porque, quando posicionado após pedais de alto ganho ou antes da etapa de potência, o piso de ruído elevado impede o fechamento preciso; o modo `Intelligent Natural Gate` usa o barramento DI limpo da entrada física como sidechain.
2. *Waveshaper genérico polinomial para todos os pedais de Drive:* descartada porque não captura a diferença de tensão direta ($V_f$) entre Germânio `1N34A` ($\approx 0,3\text{ V}$), Silício ($\approx 0,65\text{ V}$) e LED ($\approx 1,8\text{ V}$) exigida pelo Mod Shop (**RF-020**).

## Comportamento esperado
1. **Bloco Noise Gate / Input Gate (`GATE` — RF-018):**
   - Opera em dois modos selecionáveis: `INTELLIGENT_NATURAL_GATE` (detector de envelope acoplado ao sinal DI limpo da entrada física para abrir/fechar com precisão mesmo após distorções pesadas) e `CLASSIC_EXPANDER_GATE` (detector local).
   - Controles: `Threshold` (`-90.0 dBFS` a `0.0 dBFS`), `Ratio` (`1.2:1` a `100:1`), `Attack` (`0.1 ms` a `50.0 ms`), `Hold` (`1.0 ms` a `500.0 ms`), `Release` (`5.0 ms` a `1000.0 ms`) e telemetria contínua de atenuação `gainReductionDb`.
2. **Bloco Compressor (`COMP` — RF-019):**
   - Oferece 4 topologias selecionáveis: `PEDAL_ROSS_DYNA`, `STUDIO_FET_1176`, `OPTICAL_LA2A` (constante de liberação eletroluminescente dependente de programa) e `VCA_MASTER_BUS`.
   - Inclui controle `Parallel Mix (Dry/Wet)` de `0%` a `100%`, `Makeup Gain` e medidor VU de redução de ganho em dB.
3. **Bloco Overdrive / Distortion / Fuzz (`DRIVE` — RF-020):**
   - Catálogo de 6 modelos base de circuito: `GREEN_SCREAMER_808`, `GOLDEN_KLONE_CENTAUR` (com mistura de sinal limpo + buffer/charge-pump interno), `BLACK_RODENT_RAT`, `ORANGE_BOSS_DS`, `FUZZ_FACE_SI_GE` (sensível à impedância de entrada) e `BIG_MUFF_PI` (4 estágios em cascata + mid-scoop).
   - Na visão Mod Shop do pedal, permite substituir o par de clipping por: `SILICON_SYMMETRICAL`, `ASYMMETRICAL_SI_LED`, `GERMANIUM_1N34A`, `MOSFET_CLIP` ou `HARD_OPAMP`.
4. **Bloco Equalizador (`EQ` — RF-021):**
   - `PARAMETRIC_8_BAND`: 8 bandas paramétricas independentes (Frequência `20 Hz–20 kHz`, Ganho `-18 dB a +18 dB`, Fator `Q` `0.1 a 20.0`, tipos Bell/Shelf/Cut) renderizadas como curva interativa sobreposta ao analisador de espectro **RTA (Real-Time Analyzer)** em tempo real.
   - `GRAPHIC_5_BAND_BOOGIE`: 5 faders indutivos clássicos centrados exatamente em `80 Hz`, `240 Hz`, `750 Hz`, `2200 Hz` e `6600 Hz` (`-12 dB` a `+12 dB`, ideal para a curva em "V" pós-preamp).
   - `GRAPHIC_10_BAND_PEDAL`: 10 oitavas ISO de `31.25 Hz` a `16 kHz` (`-15 dB` a `+15 dB`) + Slider de `Output Level`.

## Critérios de aceite
- **CA-1:** No modo `INTELLIGENT_NATURAL_GATE`, mesmo quando o bloco `GATE` é posicionado após um bloco `DRIVE` com alto ruído de fundo, a abertura e o fechamento do gate obedecem ao envelope do sinal DI limpo de entrada conforme `Threshold`, `Attack`, `Hold` e `Release`, reportando a atenuação exata em `gainReductionDb`.
- **CA-2:** O bloco `COMP` disponibiliza as 4 topologias (`PEDAL_ROSS_DYNA`, `STUDIO_FET_1176`, `OPTICAL_LA2A`, `VCA_MASTER_BUS`), aplica compressão proporcional à curva da topologia selecionada, publica a redução de ganho no medidor VU e mistura o sinal seco e comprimido segundo `Parallel Mix (0–100%)`.
- **CA-3:** O bloco `DRIVE` disponibiliza os 6 modelos base (`GREEN_SCREAMER_808`, `GOLDEN_KLONE_CENTAUR`, `BLACK_RODENT_RAT`, `ORANGE_BOSS_DS`, `FUZZ_FACE_SI_GE`, `BIG_MUFF_PI`) e permite trocar o tipo de diodo no Mod Shop entre as 5 opções (`SILICON_SYMMETRICAL`, `ASYMMETRICAL_SI_LED`, `GERMANIUM_1N34A`, `MOSFET_CLIP`, `HARD_OPAMP`).
- **CA-4:** No bloco `DRIVE`, alterar o diodo de `GERMANIUM_1N34A` para `ASYMMETRICAL_SI_LED` eleva o limiar de tensão de corte ($V_f$) aumentando o headroom antes da saturação e introduzindo harmônicos pares por assimetria de semiciclo.
- **CA-5:** No modo `PARAMETRIC_8_BAND` do bloco `EQ`, ajustar `Frequency`, `Gain` e `Q` de qualquer uma das 8 bandas modifica a resposta em frequência do filtro biquad em 64-bit e atualiza a curva gráfica sobreposta ao espectro RTA em tempo real.
- **CA-6:** No modo `GRAPHIC_5_BAND_BOOGIE` do bloco `EQ`, os 5 faders atuam exatamente nas frequências centrais `80 Hz`, `240 Hz`, `750 Hz`, `2200 Hz` e `6600 Hz`; no modo `GRAPHIC_10_BAND_PEDAL`, atuam nas 10 bandas ISO de `31.25 Hz` a `16 kHz`.
- **CA-7:** Caso de erro — configurar um fator `Q <= 0` ou frequência fora de `[20 Hz, 20000 Hz]` no `EQ`, ou `Threshold > 0 dBFS` no `GATE`, é recusado ou limitado (*clamped*) aos limites físicos válidos sem gerar coeficientes `NaN` ou auto-oscilação nos filtros IIR.
- **CA-8:** Não-regressão — com todas as bandas de qualquer modo do bloco `EQ` em `0.0 dB` (ou `Parallel Mix = 0%` no `COMP`), a resposta em frequência de magnitude entre `20 Hz` e `20 kHz` permanece plana dentro de `±0,05 dB`.

## Fora de escopo
- Efeitos baseados em linha de atraso temporal, modulação LFO, pitch shifting e reverberação (cobertos pela **Spec 007**).
- A automação externa de parâmetros via Pedal de Expressão ou LFO (coberta pela **Spec 009**).

## Invariantes tocadas
- **RNF-003 (Acumuladores 64-bit abaixo de 100 Hz):** As bandas graves do EQ (`31.25 Hz`, `62.5 Hz`, `80 Hz`) e os filtros de detector de envelope operam em precisão `float64` para evitar ruído de quantização e instabilidade de polos próximos ao círculo unitário.
- **RNF-004 (Oversampling Dinâmico):** O solver de diodos do bloco `DRIVE` roda sob o fator de oversampling configurado (`1x` a `8x`).
- **Tokens Semânticos de LED (Design System Seção 2.2):** Os blocos usam obrigatoriamente `--led-gate` (`#00E676`), `--led-comp` (`#00B0FF`), `--led-drive` (`#FF9100`) e `--led-eq` (`#1DE9B6`).

## Testes que cobrem esta spec
- `tests/contract/006-blocos-dinamica-drive-equalizacao.test.js` → CA-1, CA-2, CA-3, CA-4, CA-5, CA-6, CA-7, CA-8 *(pendente — ciclo TDD red → green)*.

## Notas
- Rastreabilidade SRS: **RF-018**, **RF-019**, **RF-020**, **RF-021**; Design System Seção 2.2.
