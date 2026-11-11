---
spec: 024
titulo: Compressor, EQ, chorus, delay, reverb, split e o segundo amp do grid não fazem parte do som
status: implementada
atualizado: 2026-11-11
---
# 024 — Efeitos e Caminho Paralelo do Grid ao Vivo

## Objetivo
Fazer todos os blocos do grid — compressor, EQ, chorus, delay, reverb, split, segundo amp e
segundo gabinete — processarem o som na ordem mostrada, com os controles dos equipamentos que eles
representam, e os footswitches FS2–FS4 funcionarem.

## Contexto
Origem: pedido do usuário em 2026-11-09 (todos os controles e pedais funcionando e parecidos com os
reais). Evidência:

- O grid (`app.js:43-57`) mostra: linha 0 `gate → drive → split → amp_1 → cab_1 → eq_1 → dly_1 →
  rev_1`; linha 1 `comp_2 → amp_2 → cab_2 → mod_2`, com entrada "SPLIT PATH B" e saída
  "MIX MERGE →" (`app.js:91-92`). `amp_2` e `cab_2` nascem em bypass.
- A cadeia ao vivo é fixa: Gate → Amp → Cab (`audio.rs:408-421`; drive entra pela Spec 016). O
  grafo da Spec 002 (`dag.rs`, `GridMatrix4x12`) e o `DspEngine` (`engine.rs`) existem, mas o
  desktop não os usa.
- Os processadores existem e têm contrato próprio (Specs 006/007): `CompressorProcessor`
  (`StudioFet1176`), `EqProcessor` (`Parametric8Band`), `ModulationProcessor` (`StereoTriChorus`),
  `DelayProcessor` (`MultiHeadTapeEcho`, com ducking e spillover), reverb (`StudioVintagePlate`).
- FS2–FS4 alternam `blk_mod_2`, `blk_dly_1`, `blk_rev_1` só na tela (`app.js:462-490`); as cenas
  gravam `dlyBypass` sem efeito sonoro (`app.js:545-546`).
- Não existe editor de parâmetros para nenhum desses blocos (o editor nasce na Spec 023).

Base técnica (skill §7.2-7.3, a confirmar com manual/esquema de cada referência): o compressor
FET de referência não tem controle de threshold — o Input empurra o sinal contra um limiar fixo,
Output compensa, Attack/Release têm faixas curtas e o sentido do knob é invertido (número maior =
mais rápido), e a razão é escolhida por botões; o eco de fita multicabeça tem cabeças em posições
fixas, velocidade da fita (mudar a velocidade com repetições soando muda a altura delas),
realimentação que pode auto-oscilar limitada pela saturação da fita, e filtros de grave/agudo nas
repetições; o reverb de placa tem tempo de decaimento ajustável e pré-atraso; chorus é linha de
atraso modulada misturada ao sinal seco. Delay e reverb normalmente vêm depois do amp; ao desligar,
caudas podem continuar (spillover — skill §10.3).

## Comportamento esperado
1. **Grafo ao vivo igual ao grid.** O motor compila o grid em um grafo (Spec 002): linha 0 em
   série; `split` envia o sinal para a linha 1; a linha 1 volta à linha 0 no ponto de merge, na
   coluna seguinte ao último bloco da linha 1 (coluna 6, antes de `dly_1`), de modo que delay e
   reverb atuam sobre a soma. `GET /api/graph` devolve o grafo efetivo. Linhas 2 e 3 vazias não
   custam processamento.
2. **Split e merge.** `split.mode` = `ab` (as duas linhas recebem o sinal inteiro) ou `crossover`
   (graves para a linha 0, agudos para a linha 1, soma plana — Spec 002 CA-3) com
   `split.crossover_hz`. No merge, `merge.row_1_level_db` e `merge.row_1_phase_invert`
   (Spec 002 CA-5/CA-6). `split.enabled=false` corta a linha 1 e a cadeia volta a ser a série da
   linha 0.
3. **Segundo amp e gabinete.** `amp_2` e `cab_2` usam os contratos das Specs 020-022 com prefixos
   `amp_2.`/`cab_2.`; clicar neles abre o painel/sala da instância correspondente.
4. **Padrão do caminho B.** Para o caminho paralelo soar como um rig real (amp limpo + chorus em
   paralelo com o amp principal), os valores nominais passam a ser `amp_2` (`twin_ab763`) e `cab_2`
   (`alnico_blue_2x12_open`) ligados e `merge.row_1_level_db` em −6 dB. Se o split fica ligado no
   boot é decisão do preset de boot (Spec 025: `brit800` mantém o split desligado, então o som do
   boot não muda em relação às Specs 013-016). Uma linha que chega ao merge sem amp nem cab ativos
   leva sinal direto e a UI avisa "caminho B sem amp/cab: sinal direto".
5. **Controles dos equipamentos** no editor de bloco (Spec 023), com taper e faixas do manual citado:

   | Bloco | Controles (chave `<bloco>.<controle>`) |
   |---|---|
   | `comp_2` (FET) | `input`, `output` 0–10; `attack`, `release` 1–7 (7 = mais rápido); `ratio` enum `4`, `8`, `12`, `20`, `all` |
   | `eq_1` (paramétrico 8 bandas) | `band<k>.freq_hz` 20–20.000, `band<k>.gain_db` −15–+15, `band<k>.q` 0,3–10, k = 1…8 |
   | `mod_2` (chorus) | `rate_hz` 0,05–10, `depth` 0–10, `mix` 0–100 % |
   | `dly_1` (eco de fita) | `heads` enum (combinações das 3 cabeças), `repeat_rate` 0–10 (velocidade da fita), `intensity` 0–10, `echo_volume` 0–10, `bass` 0–10, `treble` 0–10, `spillover` bool |
   | `rev_1` (placa) | `decay_s` 0,5–6, `predelay_ms` 0–150, `damping` 0–10, `mix` 0–100 %, `spillover` bool |
   | `split` / `merge` | `mode`, `crossover_hz` 100–2.000; `row_1_level_db` −60–+6, `row_1_phase_invert` |

   Todos têm `<bloco>.enabled` (crossfade de 10 ms). Valores no boot: os do preset de boot
   (Spec 025); até lá, o nominal de cada processador.
6. **Footswitches e cenas.** FS2, FS3 e FS4 enviam `mod_2.enabled`, `dly_1.enabled`,
   `rev_1.enabled`; as cenas enviam também `dly_1.enabled = !dlyBypass`.
7. **Spillover.** Com `spillover` ligado, desligar delay/reverb para a entrada no bloco e deixa a
   cauda decair; desligado, a cauda some com fade de 10 ms.

## Critérios de aceite
- **CA-1:** `GET /api/graph` lista os blocos de `app.js:43-57` nas linhas e colunas do grid, com o
  merge na coluna 6; com `split.enabled=false` e `eq_1`, `dly_1`, `rev_1` desligados, a saída é
  bit-idêntica à cadeia em série da Spec 016.
- **CA-2:** `split.mode=crossover`: a soma das duas bandas (sem blocos nas linhas) é plana
  ±0,1 dB de 20 Hz a 20 kHz e o ponto de cruzamento fica em `crossover_hz` ± 5 %;
  `ab`: as duas linhas recebem amostras idênticas.
- **CA-3:** Linhas idênticas com `row_1_phase_invert=true` e mesmo nível cancelam ≥ 40 dB;
  `row_1_level_db=-60` deixa a contribuição da linha 1 ≤ −60 dB.
- **CA-4:** `comp_2`: acima do limiar, a inclinação da curva estática bate com `ratio` 4, 8, 12 e
  20 a ±10 %; subir `input` aumenta a redução de ganho para a mesma entrada; o tempo de ataque
  medido com `attack`=7 é menor que com `attack`=1.
- **CA-5:** `eq_1` com todas as bandas em 0 dB tem resposta plana ±0,01 dB; uma banda em
  +12 dB, 1 kHz, Q 1 dá +12 ± 0,5 dB em 1 kHz.
- **CA-6:** `mod_2`: a modulação do atraso medida tem frequência `rate_hz` ± 5 %; `mix`=0 é
  bit-idêntico à entrada.
- **CA-7:** `dly_1`: o tempo entre repetições segue a velocidade de `repeat_rate` (tabela da fonte)
  ± 1 ms; com `intensity`=10 por 30 s a saída fica finita e ≤ 1,0 (Spec 016 CA-9); mudar
  `repeat_rate` com repetições de um tom de 1 kHz soando desloca a frequência das repetições na
  razão da mudança de velocidade ± 2 %, sem descontinuidade (régua da Spec 020 CA-2); `bass`/`treble`
  alteram só as repetições (sinal seco bit-idêntico).
- **CA-8:** `rev_1`: RT60 medido = `decay_s` ± 15 %; pré-atraso = `predelay_ms` ± 1 ms; `mix`=0 é
  bit-idêntico à entrada.
- **CA-9:** Com `spillover` ligado, desligar `dly_1` ou `rev_1` mantém a cauda (energia nos 500 ms
  seguintes ≥ −40 dB relativa à de antes) sem descontinuidade; desligado, a cauda cai ≥ 60 dB em
  10 ms ± 1 bloco.
- **CA-10:** FS2–FS4 e as cenas enviam as chaves do §6 sem debounce (teste JS); cada bloco abre seu
  editor com os controles da tabela do §5; `amp_2`/`cab_2` abrem painel e sala da instância 2.
- **CA-11:** Com `amp_2` e `cab_2` desligados e `split.enabled=true`, a UI mostra o aviso de sinal
  direto; com `split.enabled=false`, a linha 1 aparece esmaecida (teste JS).
- **CA-12 (manual, registrado no PR):** No PC do usuário, grafo completo (dois amps, dois cabs,
  efeitos, IR de 500 ms em cada cab) no menor período estável da Spec 018: `cpu_avg_pct`,
  `cpu_peak_pct` e `underruns` registrados nas Notas; aceite: 0 underruns em 5 min — se falhar, o
  PR registra o período mínimo em que passa.
- **CA-13:** Não-regressão — testes das Specs 002, 006, 007, 016, 020 e 022 continuam verdes.

## Fora de escopo
- Saída estéreo (tri-chorus estéreo, ping-pong, pan por linha): saída mono atual (`audio.rs:258-263`).
- Reverb de mola do eco de fita, ducking e tap tempo: sem controle na UI.
- Mover/adicionar blocos no grid e linhas 2-3 (AUX, USB reamp): a UI não permite hoje.
- Pitch e looper no grid: sem bloco na UI (looper → Spec 017/026).

## Invariantes tocadas
- Áudio em tempo real (Spec 001/018): grafo compilado fora do callback e trocado sem destruir
  objetos na thread de áudio (skill §3.1); buffers de delay/reverb pré-alocados pelo máximo.
- Spec 002 (DAG/split/mixer): passa a ser o motor do caminho ao vivo.
- Spec 007 (spillover/ducking): mantidos.

## Testes que cobrem esta spec
Planejados (red → green): `crates/ampforge-dsp/tests/contract_024_live_graph_fx.rs` (CA-1…CA-9,
CA-13), `crates/sonix-desktop/tests/contract_024_graph_api.rs` (CA-1 lado API) e
`tests/contract/024-efeitos-e-caminho-paralelo-ao-vivo.test.js` (CA-10, CA-11). CA-12 manual.

## Notas
- Fontes a anexar no PR: manual do compressor FET de referência (faixas de attack/release e razões),
  manual do eco de fita de referência (velocidades, posições das cabeças) e documentação do reverb
  de placa de referência. Sem fonte, faixas marcadas como hipótese.
- Com o split desligado no boot (preset `brit800`), FS2 (chorus, caminho B) só é audível depois de
  ligar o split ou carregar um preset que o use (ex.: `usdeluxe`); a UI esmaece a linha 1 quando o
  split está desligado. Decisão a confirmar pelo usuário antes de `aprovada`.
- Depende das Specs 014, 016, 020, 022 e 023.
