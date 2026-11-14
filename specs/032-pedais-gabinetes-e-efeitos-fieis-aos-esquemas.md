---
spec: 032
titulo: Pedais, compressor, tape echo, equalizadores e gabinetes genéricos ou com controles ausentes
status: implementada
atualizado: 2026-11-14
---
# 032 — Pedais, Dinâmica, Tape Echo, EQ e Gabinetes Fiéis aos Esquemas Originais

## Objetivo
Implementar modelos DSP distintos e fiéis aos circuitos originais para todos os pedais de drive, compressor 1176LN, Tape Echo RE-201, equalizadores gráficos/paramétricos e gabinetes, com todos os controles originais expostos na API e na UI.

## Contexto
Origem: relato do usuário ("examine os esquemas dos amplificadores, gabinetes, valvulas, pedais, equalização, pois em testes muitos não foram muito uteis... os pedais e amplificadores devem seguir os esquemas eletricos das versões originais e possuir todos os controles") e auditoria do código:

1. **Pedais de Drive (`crates/ampforge-dsp/src/fx/drive.rs:171-256`):**
   - Apenas `GreenScreamer808` tem função própria; os outros 5 modelos (`GoldenKloneCentaur`, `BlackRodentRat`, `OrangeBossDs`, `FuzzFaceSiGe`, `BigMuffPi`) caem no `process_generic` idêntico (um `tanh` + filtro RC de 1º ordem).
   - Não existe parâmetro `drive.model` na tabela de parâmetros de `server.rs` nem seletor de pedal na UI.
2. **Compressor FET 1176LN (`crates/sonix-desktop/src/audio.rs:1527` e `fx/dynamics.rs:117-211`):**
   - `set_comp_2_output` é um stub vazio (`// Output makeup`), então o controle `Output` não faz nada.
   - Falta o modo `"all"` (**All-Buttons In / British Mode**) distinto do `20:1`, com saturação FET e resposta de ataque/release explosiva característica, além de medição real de GR.
3. **Tape Echo RE-201 (`crates/ampforge-dsp/src/fx/delay.rs` e `audio.rs:1582-1603`):**
   - `dly_1.heads` existe no `AppState` (`server.rs:166`), mas não é aplicado no `RealtimeDspChain`; faltam os **12 modos do seletor do RE-201** (combinações de cabeçotes 1/2/3 + spring reverb), os controles `Bass` e `Treble` nas repetições (`dly_1.bass`, `dly_1.treble` estão no `AppState` mas sem efeito no áudio!), `Reverb Volume` e saturação/wow-flutter de fita.
4. **Equalizadores (`crates/ampforge-dsp/src/fx/eq.rs` e `audio.rs:1554-1564`):**
   - `EqProcessor` tem 3 modos (`Parametric8Band`, `Graphic5BandBoogie`, `Graphic10BandPedal`), mas `RealtimeDspChain` instancia fixo em `Parametric8Band` (`audio.rs:977`) e não expõe `eq_1.mode` nem `Gain`/`Volume` do MXR M108 na API/UI.
5. **Gabinetes (`crates/ampforge-dsp/src/cab.rs:136-177`):**
   - Faltam modelos específicos para acompanhar o Twin Reverb (`Twin2x12Open` — Jensen C12N / JBL D120F) e controle completo de `cab_1.speaker`, `cab_1.mic1.model`, `cab_1.mic2.model`, `cab_1.mic1.angle`, `cab_1.mic2.angle`, `cab_1.mic2.phase_invert`, `cab_1.low_cut_hz` na UI.

## Comportamento esperado
1. **Pedais de Drive (`drive.model` selecionável em tempo real sem estalos):**
   - `ts808` (Ibanez TS808): passa-altas pré-clipping (~720 Hz), op-amp com diodos em realimentação + dry clean bleed parcial, filtro de `Tone` passivo/ativo e `Level`.
   - `klon_centaur` (Klon Centaur): potenciômetro duplo de `Gain` (clean blend + clipping por diodos de germânio com headroom estendido), `Treble` (shelving ativo), `Output`.
   - `proco_rat` (ProCo RAT): op-amp LM308 (slew-rate limiting em sinais rápidos + ganho alto), hard clipping para terra, e `Filter` funcionando no sentido original (horário = corta mais agudos!). Controles: `Distortion`, `Filter`, `Volume`.
   - `big_muff_pi` (EHX Big Muff Pi): 4 estágios transistorizados em cascata com clipping entre estágios + tone stack passivo característico com notch de médios (`Tone`), `Sustain`, `Volume`.
   - `fuzz_face` (Dallas Arbiter Fuzz Face Ge): 2 estágios acoplados com assimetria de germânio, dependência da impedância/nível de entrada (limpa ao baixar o volume da guitarra), controles `Fuzz`, `Volume`, `Bias`.
   - `boss_ds1` (Boss DS-1): pré-ênfase + hard clipping op-amp + filtro de `Tone` tilt/V-curve característico, controles `Dist`, `Tone`, `Level`.
   - Controles comuns mapeados e rótulos exibidos na UI conforme o pedal escolhido, além de seletor de diodo no Mod Shop do pedal.
2. **Compressor 1176LN (`comp_2`):**
   - `comp_2.output` aplica make-up gain real (-24 a +24 dB).
   - `comp_2.ratio`: `4`, `8`, `12`, `20` e `all` (All-Buttons In: saturação harmônica FET + curva de release dependente do programa).
   - Sentido autêntico de `Attack` e `Release` documentado na UI (1 = mais lento, 7 = mais rápido).
3. **Roland RE-201 Space Echo (`dly_1`):**
   - `dly_1.mode` (1 a 12, conforme painel original do RE-201): modos 1–4 somente fita/cabeçotes (1=H1, 2=H2, 3=H3, 4=H2+H3), modos 5–11 multi-head + spring reverb interno (5=H1+H2+Rev, 6=H1+H3+Rev, 7=H2+H3+Rev, 8–11 combinações multi-head + Rev), e modo 12 apenas spring reverb interno (sem eco discreto de fita).
   - Controles funcionais: `dly_1.repeat_rate`, `dly_1.intensity` (permite auto-oscilação controlada acima de 8.5), `dly_1.echo_volume`, `dly_1.reverb_volume`, `dly_1.bass`, `dly_1.treble` (aplicados apenas no caminho molhado/feedback), `dly_1.spillover`.
4. **Equalizadores (`eq_1`):**
   - `eq_1.mode`: `eight_band` (`parametric_8`, 8 bandas Freq/Gain/Q), `graphic_5_boogie` (80, 240, 750, 2200, 6600 Hz, ±12 dB), `mxr_10band` (`graphic_10_mxr`, 31.25 Hz a 16 kHz, ±12/+15 dB + `eq_1.gain_db` / `eq_1.input_gain_db` + `eq_1.volume_db` / `eq_1.output_level_db`).
5. **Gabinetes (`cab_1` e `cab_2`):**
   - Adição de `twin_2x12_open` (ressonância ~90 Hz, resposta brilhante open-back) aos modelos existentes (`greenback_4x12`, `v30_4x12`, `alnico_blue_2x12`).
   - Exposição na UI e API de seleção de modelo de caixa, modelos de microfone 1 e 2 (`SM57`, `MD421`, `R121`, `M160`, `U87`, `C414`, `M88`), eixo (`cab_1.mic{1,2}.angle_deg`: `"0"` / `"45"`), inversão de fase (`cab_1.mic{1,2}.phase_invert`) e `High Cut`.

## Critérios de aceite
- **CA-1:** Cada um dos 6 modelos de `DriveProcessor` (`ts808`, `klon`, `rat`, `muff`, `fuzzface`, `ds1`) produz resposta em frequência e assinatura harmônica (THD / razão par-ímpar) mensuravelmente distintas com a mesma entrada senoidal e controles em posição neutra (`5.0`).
- **CA-2:** No `BlackRodentRat` (`rat`), aumentar `Filter` (`tone`) de `0` para `10` reduz a energia acima de `3 kHz` em `>= 6 dB` (comportamento fiel ao filtro invertido original).
- **CA-3:** No `BigMuffPi` (`muff`) com `Tone = 5.0`, a resposta em frequência apresenta notch de médios em torno de `800–1200 Hz` `>= 4 dB` abaixo das extremidades; no `GoldenKloneCentaur` (`klon`) com `Gain = 2.0`, o sinal limpo permanece presente na saída (`correlação > 0.7` com a entrada).
- **CA-4:** `comp_2.output` altera o nível de saída em dB conforme configurado (`±12 dB` verificável em teste) e `comp_2.ratio = "all"` produz maior THD e ganho de compressão mais agressivo que `"20"`.
- **CA-5:** No `dly_1`, `mode = 5` produz múltiplos taps rítmicos distintos em resposta a um impulso (ao contrário de `mode = 1`), `dly_1.bass` e `dly_1.treble` alteram apenas o espectro das repetições (não o sinal seco) e `mode = 12` produz apenas spring reverb sem eco discreto.
- **CA-6:** `eq_1.mode` alterna entre `eight_band` (`parametric_8`), `graphic_5_boogie` e `mxr_10band` (`graphic_10_mxr`), com todas as bandas (1..10) e `gain_db`/`volume_db` alterando a resposta conforme RBJ peaking filters.
- **CA-7:** `cab_1.speaker = "twin_2x12_open"` e os controles de modelo/ângulo (`0`/`45`)/fase (`phase_invert`) dos microfones funcionam via `api::dispatch` (`/api/params`) e estão acessíveis no `controls.registry.ts` e na aba `Gabinete` da UI.
- **CA-8:** Zero alocações de heap e zero locks nos métodos `process()` de todos os efeitos e gabinetes.
- **CA-9:** Não-regressão — testes das Specs 005, 006, 007, 022, 023 e 024 continuam verdes.

## Fora de escopo
- Simulação física de fita magnética por elementos finitos ou convolução estéreo de tanques de mola reais externos (usa-se modelagem comportamental + all-pass dispersivo / filtro de fita com zero latência adicional).

## Invariantes tocadas
- Spec 001/028 (Realtime safety): buffers de delay e reverb pré-alocados na construção/prepare.
- Spec 014 (Tabela única de parâmetros) e Spec 016 (Saída segura ≤ 0 dBFS).

## Testes que cobrem esta spec
Implementados e verificados (*red → green*):
- `crates/ampforge-dsp/tests/contract_032_pedals_fx_cabs.rs` (CA-1…CA-6, CA-8, CA-9 — 8/8 ok, zero alocações e não-regressão)
- `crates/sonix-desktop/tests/contract_032_fx_api.rs` (CA-4…CA-7 — API/SPSC e efeito real em áudio)
- `tests/contract/032-pedais-fx-cabs-ui.test.js` (CA-1, CA-7 — inventário em `controls.registry.ts` sem pendências `032`)
