---
spec: 031
titulo: Amplificadores usam o mesmo pipeline genérico com tone stack de 3 biquads e faltam controles dos esquemas originais
status: Implementada
atualizado: 2026-11-14
---
# 031 — Amplificadores Fiéis aos Esquemas Originais (Topologia, Tone Stacks e Controles Completos)

## Objetivo
Substituir o pipeline genérico único de `WhiteBoxAmpProcessor` por arquiteturas distintas por modelo, com tone stacks discretizados a partir dos esquemas originais, múltiplos estágios de ganho em cascata e exposição completa dos controles originais de cada amplificador.

## Contexto
Origem: relato do usuário ("examine os esquemas dos amplificadores, gabinetes, valvulas... pois em testes muitos não foram muito uteis... os pedais e amplificadores devem seguir os esquemas eletricos das versões originais e possuir todos os controles") e análise de `crates/ampforge-dsp/src/amp.rs`:

1. **Todos os 5 amplificadores executam o mesmo circuito (`amp.rs:689-915`):**
   - Existe apenas **1 estágio** de pré-amp (`v1_sat = pre_in / (1.0 + pre_in.abs())`, `amp.rs:855-863`), **3 biquads independentes** (`low_shelf(120)`, `peaking(650)`, `high_shelf(3500)`, `amp.rs:699-701`) e **1 `tanh()`** de power amp (`amp.rs:901`), diferenciados apenas por 2 biquads estáticos de voicing (`amp.rs:515-538`).
   - O tone stack **não é** a função de transferência passiva interativa FMV nem Top Boost (Yeh & Smith, DAFx-06): `Bass`, `Middle` e `Treble` não interagem entre si e não reproduzem o mid-scoop nem os valores R/C dos esquemas.
2. **Controles mortos ou ausentes:**
   - `AmpFrontPanel::volume_2` e `AmpFrontPanel::cut` (`amp.rs:61-62`) **nunca são lidos em `process()`** (`amp.rs:689-1015`).
   - `oversampling_factor` (`amp.rs:628-634`) não altera o processamento real porque o loop está fixo em `for step in 0..8` (`amp.rs:832`).
   - Controles originais ausentes por modelo:
     - **Marshall 1959 Super Lead Plexi (1968–69):** falta seletor das **4 entradas** (`High`/`Low` do Canal I e do Canal II, `Low` −6 dB com impedância menor) e o modo `Jumpered` (guitarra em I-High, cabo de I-Low para II-High), além de `Volume II (Normal)` funcional; `master` não existe no original. Painel conferido no *1959HW Owners Manual* (itens 1–13).
     - **Marshall JCM800 2203 (1981):** falta seletor de entrada (`High` / `Low` com atenuação de 6 dB e impedância distinta) e cascata real de 3 estágios com *cold clipper* assimétrico (V1b, Rk=10k) + *cathode follower* (V2b).
     - **Fender Twin Reverb AB763 (1965):** falta seletor de canal (`Normal` sem knob Middle / `Vibrato` com Middle 10k), `Bright` (120pF) por canal, `Reverb` (mola) e `Vibrato` (`Speed`, `Intensity` — tremolo por variação de bias).
     - **Mesa/Boogie Dual Rectifier (Rev F 2ch / 3ch):** falta seletor de canal/modo (`Clean` / `Pushed` / `Vintage` / `Modern`), chave `Rectifier` (`Silicon Diode` vs `Dual 5U4GB Tube`) e chave `Power` (`Bold` vs `Spongy` variac).
     - **Vox AC30/6 Top Boost (1960s):** falta `Tone Cut` funcional pós-Phase Inverter (shunt passa-baixas antes das EL84), ausência real de NFB global (`nfb = 0`), e seleção de canal (`Normal` / `Top Boost` / `Vib-Trem`).

## Comportamento esperado
1. **Topologia multi-estágio por modelo (híbrida: circuit-level/discretização de rede + ADAA):**
   - **Marshall 1959 Super Lead (1968–69):** Dois canais paralelos (High Treble com bright cap 5000pF/100pF + Normal com acoplamento 0.022µF/0.22µF) somados antes do V2a + V2b cathode follower → Tone Stack Marshall (`R_slope=33k`, `C1=470pF`, `C2=0.022µF`, `C3=0.022µF`, `Treble=250k/1M`, `Bass=1M`, `Mid=22k/25k`) → LTP Phase Inverter → 4×EL34 Push-Pull + NFB com `Presence` (sem Master Volume). Controles: `input_jack` (`high1`, `low1`, `high2`, `low2`, `jumpered`), `gain` (Volume I), `volume_2` (Volume II), `treble`, `middle`, `bass`, `presence`, `mains`, `standby`. Ordem do painel, da esquerda para a direita (manual 1959HW): POWER · STANDBY · INDICATOR · PRESENCE · BASS · MIDDLE · TREBLE · VOLUME I · VOLUME II · entradas 2×2 (High em cima, Low embaixo; Canal I à esquerda).
   - **Marshall JCM800 2203 (1981):** V1a (Rk=2.7k, Ck=0.68µF) → `Pre-Amp` → V1b Cold Clipper (Rk=10k sem bypass, clipping assimétrico pronunciado) → divisor 470k/470pF → V2a (Rk=820Ω) → V2b Cathode Follower → Tone Stack Marshall (33k/470pF/22nF/22nF) → `Master` → LTP PI → 4×EL34 + NFB `Presence`. Controles: `input_jack` (`high`, `low`), `gain` (Pre-Amp Volume), `master` (Master Volume), `treble`, `middle`, `bass`, `presence`, `mains`, `standby`. Ordem do painel (manual JCM800 2203, BOOK-00067-00): POWER · STANDBY · PRESENCE · BASS · MIDDLE · TREBLE · MASTER VOLUME · PRE-AMP VOLUME · entradas HIGH/LOW empilhadas.
   - **Fender Twin Reverb AB763 (1965):** Canal `Normal` (2 estágios, Middle fixo 6.8k) e Canal `Vibrato` (2 estágios + mix reverb + estágio recuperador, Middle 10k) selecionáveis; Tone Stack Fender AB763 (`R_slope=100k`, `C1=250pF`, `C2=0.047µF`, `C3=0.047µF`); `Bright` (120pF dependente do volume); `Reverb` e `Vibrato` (`Speed`, `Intensity` modulando o bias do estágio de saída); LTP PI → 4×6L6GC fixed-bias (alto headroom). Controles: `channel` (`normal`, `vibrato`), `gain` (Volume), `bright`, `treble`, `middle` (ativo no Vibrato), `bass`, `reverb`, `vib_speed`, `vib_intensity`.
   - **Mesa/Boogie Dual Rectifier (Rev F / 3ch):** Até 4 estágios de ganho em cascata com filtros passa-altas inter-estágio (acoplamento apertado nos graves para evitar *flub* no modo `Modern`, mais aberto no `Vintage`); Tone Stack pós-ganho; modos `clean`, `pushed`, `vintage`, `modern` alterando o número de estágios ativos e presença/NFB (no modo `Modern` o NFB global é desconectado, conforme esquema original); chaves `rectifier` (`diode` vs `5u4gb`) e `power_mode` (`bold` vs `spongy`). Controles: `channel_mode` (`clean`, `pushed`, `vintage`, `modern`), `gain`, `master`, `treble`, `middle`, `bass`, `presence`, `rectifier`, `power_mode`.
   - **Vox AC30/6 Top Boost (1960s):** Canais `normal` e `top_boost`; rede de Tone Stack Top Boost (3ª ordem interativa Treble 50pF / Bass, alimentada por cathode follower); **ausência de NFB global**; **`Tone Cut` real** (filtro RC variável entre os dois lados do Phase Inverter LTP antes das 4×EL84 cathode-biased 50Ω + retificação GZ34). Controles: `channel` (`normal`, `top_boost`), `gain` (Volume), `treble`, `bass`, `cut`, `vib_speed`, `vib_depth` (Vib/Trem opcional).
2. **Oversampling real selecionável (`2x`, `4x`, `8x`):** `WhiteBoxAmpProcessor::process` executa o fator configurado (2, 4 ou 8 sub-amostras por amostra) usando os filtros anti-aliasing apropriados ao fator, impactando realmente o aliasing e a CPU.
3. **Mod Shop conectado ao circuito:** Alterar `preamp_tube`, `power_tube`, `bias_ma`, `sag`, `rectifier`, `nfb`, `xformer_sat`, `preamp_plate_v`, `triode_asym` e `tonestack` modifica os parâmetros físicos reais dos estágios (ganho µ, headroom de placa, resistência interna da retificação, profundidade do NFB, assimetria de cátodo/grade e escolha do tone stack).

## Critérios de aceite
- **CA-1:** Cada um dos 5 modelos possui curva de resposta do Tone Stack (50 Hz–10 kHz) dentro de `±1.0 dB` da função de transferência contínua do circuito FMV/Top Boost com os componentes R/C documentados, exibindo interação mensurável entre `Bass`, `Middle` e `Treble`.
- **CA-2:** `amp_1.volume_2` altera perceptivelmente o ganho e o equilíbrio grave/médio do `plexi_1959` quando `input_jack = "jumpered"` (diferença RMS `>= 3 dB` entre `volume_2 = 0` e `volume_2 = 10` com `gain = 2`).
- **CA-3:** `amp_1.cut` no `ac30_tb` atenua progressivamente as frequências acima de `2 kHz` em `>= 6 dB` entre `cut = 0` e `cut = 10`.
- **CA-4:** No `jcm800_2203`, com `gain = 8`, o espectro harmônico após o pré-amp apresenta harmônicos pares significativos (assimetria do cold clipper V1b) e `input_jack = "low"` reduz o ganho de entrada em `~6 dB ± 1 dB` vs `"high"`.
- **CA-5:** No `dual_rect`, alternar `channel_mode` entre `clean` e `modern` altera drasticamente o número de estágios/THD (`THD_modern > 3 * THD_clean`), e alternar `power_mode = "spongy"` reduz a tensão B+ efetiva e antecipa a compressão/sag em relação a `"bold"`.
- **CA-6:** No `twin_ab763`, alternar `channel = "normal"` vs `"vibrato"` ativa/desativa o controle `middle` (mid-scoop fixo no Normal) e `vib_intensity > 0` modula a amplitude do sinal na frequência de `vib_speed`.
- **CA-7:** Alterar `engine.oversampling` entre `2x`, `4x` e `8x` altera efetivamente o número de iterações de superamostragem no `WhiteBoxAmpProcessor::process` (mensurável por redução de aliasing em teste de seno a 5 kHz + drive alto e variação proporcional de CPU).
- **CA-8:** O painel frontal na UI exibe exatamente os controles originais do modelo selecionado (knobs, chaves de canal/modo, `Bright`, `Rectifier`, `Bold/Spongy`, `Input High/Low`).
- **CA-9:** Zero alocações de heap e zero locks durante `WhiteBoxAmpProcessor::process` e durante a troca de modelo/canal (crossfade ≤ 20 ms).
- **CA-10:** Não-regressão — saída sempre limitada a `<= 0 dBFS` (`[-1.0, 1.0]`), defesa contra `NaN/Inf`, e testes das Specs 003, 016 e 020 atualizados/compatíveis.

## Fora de escopo
- Simulação WDF/MNA completa de 100+ nós em tempo real: usa-se arquitetura híbrida (estágios não-lineares com memória + funções de transferência exatas discretizadas via bilinear transform para os tone stacks) para manter o orçamento de CPU compatível com latência < 10 ms.

## Invariantes tocadas
- Skill `sonix-pro-ampforge-expert` (§2.4, §2.5, §5.1–5.4): valores de componentes citados por revisão histórica; sem afirmação de equivalência perfeita.
- Spec 001/028 (Realtime safety) e Spec 016 (Saída segura ≤ 0 dBFS).

## Testes que cobrem esta spec
Implementados (*red → green*, todos passando):
- `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs` (`CA-1`…`CA-7`, `CA-9`, `CA-10` — 9/9 passando);
- `crates/sonix-desktop/tests/contract_031_amp_controls_api.rs` (`CA-2`…`CA-7` — 1/1 passando);
- `tests/contract/031-amplificadores-fieis-ui.test.js` (`CA-8` — 1/1 passando).

## Notas
- **2026-11-13 — painéis Marshall conferidos em documentação do fabricante** (cópias em `docs/ui-desktop/fontes/`):
  - *Marshall 1959HW Owners Manual* (reedição handwired do circuito 1969, SL/A pré-julho de 1969): ordem frontal POWER, STANDBY, INDICATOR, PRESENCE, BASS, MIDDLE, TREBLE, VOLUME I, VOLUME II e quatro entradas (High/Low por canal; Low é 6 dB mais baixa e tem impedância menor). O manual também descreve o *jumper* (fig. 1: guitarra em I-High, cabo de I-Low para II-High; fig. 2: o inverso), o Middle como pot 10% log, o bright cap de 0,005 µF em Volume I (100 pF em unidades 1965–67) e a NFB tirada da saída do falante (não do tap de 8 Ω).
  - *Marshall JCM800 2203 Owners Manual* (BOOK-00067-00, reedição): ordem frontal POWER, STANDBY, PRESENCE, BASS, MIDDLE, TREBLE, MASTER VOLUME, PRE-AMP VOLUME, HIGH e LOW sensitivity. O loop de efeitos do painel traseiro é **da reedição**; o 2203 de 1981 não tinha loop.
  - Rótulos variam por época (ex.: unidades antigas com outra serigrafia). Adotamos os rótulos dos manuais acima e mantemos nomes fictícios na UI (sem logotipos nem assinaturas).
  - **Verificação AB763 concluída (2026-11-14):** no Fender Twin Reverb AB763 (esquema 1965), *ambos* os canais Normal e Vibrato possuem potenciômetro de médios **Middle (10k Lin)** (diferentemente do Deluxe Reverb AB763, que usa resistor fixo de 6,8kΩ no canal Normal). O canal Normal continua distinto do Vibrato por não passar pelo tanque de Spring Reverb nem pelo tremolo de variação de bias; ambos os controles de `middle` são mantidos e a diferença de rota/reverb/tremolo foi implementada e testada (`CA-6`).
