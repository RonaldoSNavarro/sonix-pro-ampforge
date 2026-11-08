---
spec: 007
titulo: Blocos de modulação, pitch polifônico, delay com ducking/spillover e reverb
status: implementada
atualizado: 2026-11-08
---
# 007 — Blocos de modulação, pitch polifônico, delay com ducking/spillover e reverb

## Objetivo
Prover processamento espacial, temporal, de modulação e de transposição polifônica de estúdio com sincronia rítmica ao BPM do preset e preservação natural das caudas de ambiência (*spillover*) durante a performance.

## Contexto
Guitarristas e baixistas utilizam efeitos de modulação clássicos (Chorus BBD, Flanger Through-Zero, Phaser, Univibe, Rotary), transposição de afinação instantânea (`Virtual Capo` / `Whammy` / `Intelligent Harmonizer`) e ambiências ricas (`Tape Echo`, `BBD Delay`, `Spring/Plate/Hall/Shimmer Reverb`). Em apresentações ao vivo, cortar abruptamente a cauda de um Delay ou Reverb ao trocar da cena de Solo para a cena de Base destrói a naturalidade sonora.

Fontes da verdade e evidências no repositório:
- `docs/01-SRS-Levantamento-de-Requisitos.md:147-151` — Define **RF-022** (Bloco de Modulação `MOD` com 7 modelos e sincronia BPM `Note Division`), **RF-023** (Bloco de Pitch & Harmonizer `PITCH` com 4 modelos), **RF-024** (Bloco de Delay `DELAY` com 6 modelos, `Ducking` e `Spillover`) e **RF-025** (Bloco de Reverb `REVERB` com 5 modelos, incluindo tensão de molas / *"drip"* e `Shimmer`).
- `docs/02-Design-System-Specification.md:51-55` — Define os tokens `--led-mod` (`#7C4DFF`), `--led-pitch` (`#E040FB`), `--led-delay` (`#00E5FF`) e `--led-reverb` (`#651FFF`).
- `design-system/app.js:50-51,57` — O protótipo instancia os cards `blk_dly_1` (`Tape Echo 201`), `blk_rev_1` (`Vintage Plate`) e `blk_mod_2` (`Tri-Chorus`) no Grid, mas ainda não implementa seus algoritmos DSP, sincronia BPM nem buffers de *spillover*.

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Bloco `MOD` (**RF-022**) | Card `blk_mod_2` estático (`design-system/app.js:57`) | 7 modelos (`Analog BBD Chorus`, `Tri-Chorus Estéreo`, `Through-Zero Flanger`, `Orange 90 Phaser 4/8/12 stages`, `Optical/Bias Tremolo`, `Photocell Univibe`, `Rotary Speaker`) + sincronia por `Note Division` ao `tempoBpm` |
| Bloco `PITCH` (**RF-023**) | Não instanciado no Grid inicial | 4 modelos (`Pedal Whammy`, `Polyphonic Drop / Virtual Capo` `-12..+12 st`, `Intelligent Dual Harmonizer` com tonalidade/escala e intervalos `3ª/5ª/6ª/8ª`, `Micro Pitch Detune`) |
| Bloco `DELAY` (**RF-024**) | Card `blk_dly_1` estático (`design-system/app.js:50`) | 6 modelos (`Multi-Head Magnetic Tape Echo` com `Wow/Flutter/Tape Age` e auto-oscilação, `Analog BBD`, `Pristine Studio Digital`, `Dual Parallel`, `Reverse`, `Ping-Pong Stereo`) + `Ducking` + `Spillover` |
| Bloco `REVERB` (**RF-025**) | Card `blk_rev_1` estático (`design-system/app.js:51`) | 5 modelos (`Tube Driven Spring Reverb` com tensão de mola e *"drip"*, `Studio Vintage Plate`, `Concert Hall`, `Chamber`, `Shimmer / Cloud Ambient`) + `Spillover` |

**Alternativas descartadas:**
1. *Zerar os buffers de linha de atraso quando o bloco `DELAY` ou `REVERB` entra em Bypass:* descartada porque interrompe abruptamente a cauda sonora; em vez disso, o bypass com `Spillover` ativo apenas corta a alimentação de entrada (`input mute`) enquanto mantém a leitura do buffer de cauda até decair abaixo de `-80 dBFS`.
2. *Alocar memória das linhas de atraso sob demanda durante a execução:* descartada por violar o **RNF-005**; os buffers máximos de delay/reverb são pré-alocados na inicialização da instância do grafo.

## Comportamento esperado
1. **Bloco de Modulação (`MOD` — RF-022):**
   - Disponibiliza os modelos: `ANALOG_BBD_CHORUS`, `STEREO_TRI_CHORUS`, `THROUGH_ZERO_FLANGER`, `ORANGE_90_PHASER` (com chave de `4`, `8` ou `12` estágios all-pass), `OPTICAL_BIAS_TREMOLO`, `PHOTOCELL_UNIVIBE` e `ROTARY_SPEAKER` (com velocidades independentes de corneta `Horn` e tambor `Drum`, mais transição acelerada *Slow/Fast*).
   - Permite operar com taxa livre em Hz (`0.05 Hz` a `20.0 Hz`) ou sincronizado ao `tempoBpm` do preset via divisões rítmicas (`1/1`, `1/2`, `1/4`, `1/4T`, `1/8`, `1/8D`, `1/16`).
2. **Bloco de Pitch & Harmonizer (`PITCH` — RF-023):**
   - `PEDAL_WHAMMY`: pitch bending contínuo controlado por posição de pedal/modifier (`-24` a `+24` semitons);
   - `POLYPHONIC_DROP_CAPO`: transposição polifônica de acordes inteiros de `-12` a `+12` semitons em incrementos de `1` semitom;
   - `INTELLIGENT_DUAL_HARMONIZER`: gera até 2 vozes harmônicas diatônicas respeitando a Tonalidade (`C` a `B`), Escala (`Maior`, `Menor Natural/Harmônica/Melódica` e os 7 `Modos Gregos`) e intervalos (`3ª`, `5ª`, `6ª`, `8ª`);
   - `MICRO_PITCH_DETUNE`: desafinação fina estéreo (`-50.0` a `+50.0 cents`) com atraso curto L/R.
3. **Bloco de Delay (`DELAY` — RF-024):**
   - Modelos: `MULTI_HEAD_TAPE_ECHO` (controles de `Wow`, `Flutter`, `Tape Age` e realimentação capaz de auto-oscilação orgânica saturada sem explodir digitalmente), `ANALOG_BBD_DELAY`, `PRISTINE_DIGITAL_DELAY`, `DUAL_PARALLEL_DELAY`, `REVERSE_DELAY` e `PING_PONG_STEREO`.
   - `Ducking`: atenua automaticamente o volume das repetições (em `dB` configurável) enquanto o envelope do sinal de entrada supera o `Ducking Threshold`, restaurando o nível da cauda nas pausas da frase.
   - `Spillover`: mantém as repetições existentes soando naturalmente ao colocar o bloco em bypass ou ao trocar de Scene.
4. **Bloco de Reverb (`REVERB` — RF-025):**
   - Modelos: `TUBE_SPRING_REVERB` (com parâmetro de tensão das molas `Spring Tension` e resposta percussiva de dispersão `"Drip"`), `STUDIO_VINTAGE_PLATE`, `CONCERT_HALL`, `CHAMBER` e `SHIMMER_CLOUD_AMBIENT` (realimentação com pitch-shift de `+12` e `+24` semitons filtrados na cauda).

## Critérios de aceite
- **CA-1:** O bloco `MOD` disponibiliza os 7 modelos especificados no **RF-022** e, quando `tempoSync = true`, calcula a frequência do LFO exatamente a partir de `tempoBpm` e `noteDivision` (ex.: a `120 BPM`, `1/4` = `2.0 Hz`, `1/8D` = `2.667 Hz`).
- **CA-2:** No bloco `PITCH` em modo `POLYPHONIC_DROP_CAPO`, configurar `-2` semitons transpõe a frequência fundamental e harmônicos de sinais monofônicos e polifônicos pela razão $2^{-2/12} \approx 0,890899$ mantendo o timbre preservado.
- **CA-3:** No bloco `PITCH` em modo `INTELLIGENT_DUAL_HARMONIZER` configurado em `Dó Maior (C Major)` com intervalo de `3ª Acima`, tocar a nota `C4` (`261.63 Hz`) sintetiza `E4` (`329.63 Hz`, terça maior), enquanto tocar `D4` (`293.66 Hz`) sintetiza `F4` (`349.23 Hz`, terça menor diatônica).
- **CA-4:** O bloco `DELAY` disponibiliza os 6 modelos do **RF-024**; no modelo `MULTI_HEAD_TAPE_ECHO` com `Feedback > 100%`, o sinal entra em auto-oscilação limitada por saturação magnética suave sem ultrapassar `0.0 dBFS` nem gerar `Infinity`/`NaN`.
- **CA-5:** Com `Ducking` ativo no bloco `DELAY`, enquanto o sinal de entrada está acima do `duckingThreshold`, o nível da cauda de delay é atenuado pelo valor de `duckingAttenuationDb`, retornando ao nível nominal após o tempo de `duckingReleaseMs` quando o músico para de tocar.
- **CA-6:** O bloco `REVERB` disponibiliza os 5 modelos do **RF-025**, incluindo o controle de `"drip"` / tensão de mola em `TUBE_SPRING_REVERB` e a geração de harmônicos de oitava (`+12 st`) na cauda do modelo `SHIMMER_CLOUD_AMBIENT`.
- **CA-7:** Com `spillover = true` nos blocos `DELAY` e `REVERB`, comutar o bloco de ativo para bypass (ou trocar para uma Scene onde o bloco está em bypass) silencia imediatamente a entrada do efeito mas preserva a cauda remanescente no barramento de saída até decair abaixo de `-80 dBFS`.
- **CA-8:** Caso de erro — configurar `tempoBpm <= 0` ou `delayTimeMs` acima da capacidade máxima do buffer pré-alocado (ex.: `> 4000 ms`) faz o bloco limitar o valor ao intervalo suportado `[20.0, 300.0] BPM` e `[1.0, 4000.0] ms` sem realocar memória na thread de áudio.
- **CA-9:** Não-regressão — quando `mix = 0%` (100% Dry) em qualquer bloco `MOD`, `PITCH`, `DELAY` ou `REVERB` (em topologia dry-through), o sinal de saída preserva amplitude e fase idênticas ao sinal de entrada.

## Fora de escopo
- Utilitários de palco que não são efeitos de timbre: Afinador e Looper (cobertos pela **Spec 008**).
- A matriz de automação externa (Pedal de Expressão / MIDI CC) que controla o `Pedal Whammy` (coberta pela **Spec 009**).

## Invariantes tocadas
- **RNF-005 (Real-Time Safe — Buffers Pré-Alocados):** Todas as linhas de atraso de `MOD`, `PITCH`, `DELAY` e `REVERB` (incluindo os buffers de `Spillover`) são pré-alocadas fora do callback de tempo real.
- **Tokens Semânticos de LED (Design System Seção 2.2):** Usa `--led-mod` (`#7C4DFF`), `--led-pitch` (`#E040FB`), `--led-delay` (`#00E5FF`) e `--led-reverb` (`#651FFF`).

## Testes que cobrem esta spec
- `crates/ampforge-dsp/tests/contract_007_mod_pitch_delay_reverb.rs` → CA-1 a CA-9 (9 testes unitários de contrato em Rust, 100% ok).
- `tests/contract/007-blocos-modulacao-pitch-delay-reverb.test.js` → Suíte de integração e validação de critérios CA-1 a CA-9.

## Notas
- Rastreabilidade SRS: **RF-022**, **RF-023**, **RF-024**, **RF-025**; Design System Seção 2.2.
