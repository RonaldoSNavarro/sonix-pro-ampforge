---
spec: 023
titulo: O pedal "Screamer 808" não tem controles na interface e o drive não reproduz o circuito do pedal
status: rascunho
atualizado: 2026-11-09
---
# 023 — Pedal de Drive 808 Fiel ao Circuito e Editor do Bloco Selecionado

## Objetivo
Dar ao pedal de drive do grid os controles do pedal real (Drive, Tone, Level) e fazer o som
reproduzir os comportamentos que caracterizam o circuito de referência — e criar o editor de bloco
que os demais pedais e efeitos (Spec 024) vão usar.

## Contexto
Origem: pedido do usuário em 2026-11-09 ("pedais … devem funcionar realmente e serem parecidos com
os reais", seguindo a skill). Evidência:

- O grid mostra `blk_drive` "Screamer 808" (`app.js:45`), mas clicar no bloco só atualiza o
  cabeçalho (`app.js:124-135, 161-173`): não há knobs de pedal em lugar nenhum da UI.
- `DriveProcessor::process` (`fx/drive.rs:76-99`) aplica ganho fixo `1 + gain/10·12` e `tanh` por
  tensão de diodo; **ignora** `model`, `tone` e `level` (Spec 016 Contexto). A Spec 016 coloca o
  bloco na cadeia e exige que `gain/tone/level` alterem o sinal, mas não que o resultado se pareça
  com o pedal.

Base técnica (skill §7.1, a confirmar com o esquema da revisão escolhida): o 808 é um estágio
não inversor com amplificador operacional e diodos de silício em antiparalelo **na
realimentação** (clipping suave e simétrico); a rede RC em série com o resistor de terra da
realimentação forma um passa-altas no **ganho**: graves passam com ganho perto de 1 e quase não
distorcem, médios e agudos recebem o ganho do potenciômetro Drive — daí o "médio pronunciado" e o
grave firme; um capacitor em paralelo com a realimentação limita o ganho em agudos. Depois do
clipping, um controle de tonalidade ativo e o Level. Há buffers de entrada e saída, então o pedal é
pouco sensível à impedância do captador (ao contrário de um fuzz).

## Comportamento esperado
1. **Editor do bloco selecionado** (`#blockEditor`): clicar num bloco do grid abre um painel com
   os controles do equipamento que o bloco representa, no estilo de pedal (knobs 0–10 com o rótulo
   do pedal real). Duplo clique continua alternando o bypass (Spec 014). Blocos sem editor
   definido mostram "sem controles nesta versão".
2. **Controles do 808:** Drive, Tone, Level. Registro da Spec 014 §2:

   | Chave | Tipo | Faixa | Valor no boot | Suavização |
   |---|---|---|---|---|
   | `drive.drive` | número | 0–10 | 5 | `OnePoleSmoother` |
   | `drive.tone` | número | 0–10 | 5 | `OnePoleSmoother` |
   | `drive.level` | número | 0–10 | 5 | `OnePoleSmoother` |

   O taper de cada potenciômetro segue o do esquema citado (ex.: logarítmico), não dB linear.
3. **Modelo do circuito** (abordagem híbrida, skill §3 etapa D): partes lineares pelas funções de
   transferência do esquema citado (passa-altas do ganho, limite de agudos da realimentação, tone
   ativo, acoplamentos), discretizadas pela taxa da sessão; não linearidade dos diodos na
   realimentação resolvida por modelo de diodo (equação de Shockley ou aproximação documentada),
   com oversampling (`engine.oversampling`, Spec 020). Valores de componentes só do esquema
   citado nas Notas.
4. **Modelos do enum.** `DriveModel` tem outros 5 modelos (`fx/drive.rs:5-12`) sem bloco na UI;
   eles continuam com a implementação genérica e ficam fora desta spec.

## Critérios de aceite
- **CA-1:** Clicar em `blk_drive` abre `#blockEditor` com exatamente Drive, Tone e Level; arrastar
  cada knob envia `drive.drive`, `drive.tone`, `drive.level` (coalescido como na Spec 014 CA-1);
  duplo clique continua enviando `drive.enabled` (teste JS).
- **CA-2:** Pequeno sinal (−40 dBFS), `drive`=10: na saída do estágio de clipping, o ganho em 80 Hz
  fica a 0 ± 1,5 dB e o ganho em 1 kHz fica ≥ 15 dB acima do de 80 Hz.
- **CA-3:** `drive`=10, seno de pico 0,1: THD com 100 Hz ≤ 0,5 × THD com 1 kHz (graves distorcem
  menos).
- **CA-4:** `drive`=10, 1 kHz, pico 0,1: 2º harmônico ≤ −50 dBc e 3º harmônico ≥ −30 dBc (clipping
  simétrico).
- **CA-5:** O ganho de pequeno sinal em 1 kHz cresce monotonicamente em `drive` 0, 2,5, 5, 7,5, 10,
  e com `drive`=0 é ≥ +6 dB (o estágio nunca é unitário em médios).
- **CA-6:** `tone` 10 vs 0: energia acima de 3 kHz ≥ 6 dB maior; abaixo de 500 Hz muda ≤ 1 dB.
- **CA-7:** `level`=0 deixa a saída ≥ 60 dB abaixo de `level`=10; `level` não muda a THD
  (diferença ≤ 0,01 ponto percentual entre `level` 3 e 10).
- **CA-8:** Com o oversampling escolhido, aliasing do bloco pelo método da Spec 016 CA-11 cumpre o
  mesmo limite.
- **CA-9:** Para a mesma entrada, o 808 desta spec e a implementação genérica atual diferem (são
  modelos distintos) e os outros 5 `DriveModel` produzem saída bit-idêntica à de antes desta spec.
- **CA-10:** Não-regressão — testes das Specs 006, 014 e 016 continuam verdes.

## Fora de escopo
- Os outros 5 modelos de drive (Klon, RAT, DS, Fuzz Face, Big Muff): cada um exige revisão de
  circuito citada; o Fuzz Face ainda exige modelo de impedância de fonte (skill §7.1, §8.2). Spec
  futura por modelo.
- Escolha de modelo de pedal pela UI: o grid tem só o 808.
- Pedal de expressão como wah: o slider de expressão modula ganho (Spec 014).

## Invariantes tocadas
- Áudio em tempo real (Spec 001/018): filtros e estados pré-alocados; nada aloca no callback.
- Spec 016 CA-6: mantido (bypass bit-idêntico; drive/tone/level alteram o sinal), agora com metas de
  fidelidade.
- Skill §2.5/§2.8: valores só com esquema citado; a UI diz "inspirado no circuito 808".

## Testes que cobrem esta spec
Planejados (red → green): `crates/ampforge-dsp/tests/contract_023_drive_808.rs` (CA-2…CA-10) e
`tests/contract/023-pedal-drive-808-editor-de-bloco.test.js` (CA-1).

## Notas
- Fonte a anexar no PR: esquema do pedal de referência com revisão (ex.: TS808 original) e, se
  usada, análise de circuito publicada; o passa-altas do ganho e o capacitor de realimentação são
  calculados dos valores do esquema, não arbitrados.
- Limites (1,5 dB, 15 dB, 0,5×, −50/−30 dBc, 6 dB, 60 dB) são hipóteses de projeto derivadas do
  comportamento esperado do circuito; a spec é editada com o cálculo do esquema antes do merge se
  algum se mostrar incompatível com ele.
