---
spec: 025
titulo: Seletor de preset não carrega nada, cenas só mudam dois knobs e o export gera arquivo que o próprio validador recusa
status: rascunho
atualizado: 2026-11-09
---
# 025 — Presets, Cenas e Canais de Bloco Reais

## Objetivo
Fazer presets, cenas e canais A–D guardarem e restaurarem o som inteiro — todos os parâmetros do
motor, o grafo e a IR — trocando sem buraco no áudio, com arquivos que o próprio app sabe importar.

## Contexto
Origem: pedido do usuário em 2026-11-09 (todos os controles funcionando). Evidência:

- `#presetSelect` (`index.html:32-37`, 4 presets) não tem handler: a única leitura é o nome no
  export (`app.js:787`). O selo `#presetBankBadge` é fixo ("P01").
- "Export" (`app.js:784-808`) gera um JSON próprio (`schemaVersion`, `presetName`, `ampParams`,
  `circuitParams`, `blocks`) que **não** segue o `SonixPresetDoc` da Spec 010
  (`persistence.rs:96-107`: `$schema`, `metadata`, `activeSceneIndex`, `scenes`, `grid`,
  `modifiers`) — `from_json_str` o recusa. Não há import na UI.
- Cena (`app.js:522-560`) aplica só `gain`, `od`, bypass de drive/delay, tema e logo; o selo diz
  "GAPLESS CROSSFADE 1.1ms + SPILLOVER" como texto fixo, sem medição.
- Canais A–D (Spec 009, `BlockChannel`/`BlockChannelState` em `preset.rs:22-112`) existem no
  `sonix-core`, mas na UI o botão só grava a letra em `block.ch` (`app.js:675-686`).
- Os valores no boot hoje vêm de constantes (`main.rs:41-45`) e da tabela da Spec 014.

## Comportamento esperado
1. **Preset = estado completo.** Um preset guarda: todos os valores do registro de parâmetros
   (Specs 014, 018-024), o grafo (Spec 024), as 8 cenas, os 4 canais de cada bloco e referências
   de IR/DI com hash. Formato: `SonixPresetDoc` com `schemaVersion` **1.1.0**, que acrescenta os
   parâmetros; documentos 1.0.0 continuam importáveis (migração com valores no boot para o que
   faltar).
2. **Carregar.** Escolher no `#presetSelect` aplica o preset inteiro no motor de uma vez: estado
   novo preparado fora do callback e trocado com crossfade de 20 ms; delay/reverb com spillover
   mantêm a cauda; a UI se redesenha a partir de `GET /api/state`. O selo mostra o número do preset.
3. **Presets de fábrica.** Os 4 do seletor viram arquivos `.sonixpreset` versionados no repositório
   (`presets/factory/`), descritos nas Notas, validados pelo mesmo validador do import.
4. **Salvar, exportar, importar.** "Salvar" e "Salvar como" gravam presets do usuário na pasta de
   dados do app (`%APPDATA%\SonixPro\presets`). "Export" gera `.sonixpreset` (ou `.sforgepkg` com
   IRs, Spec 010) **pelo motor**, não por JSON montado na UI. "Import" valida e mostra o erro sem
   alterar o estado atual.
5. **Cenas.** Cada cena guarda, por bloco, ligado/desligado, canal ativo e os parâmetros que o
   usuário marcou como "da cena". Trocar de cena aplica só as diferenças, numa operação atômica, sem
   buraco no áudio; "Salvar cena" grava o estado atual na cena ativa. O selo de troca mostra o tempo
   medido pelo motor.
6. **Canais A–D.** Cada bloco tem 4 conjuntos de parâmetros; o botão de canal troca o conjunto do
   bloco selecionado com crossfade; a cena lembra o canal de cada bloco.
7. **Boot.** O app abre com o último preset usado; na primeira execução, `brit800`. As tabelas
   "valor no boot" das outras specs passam a ser o conteúdo desse preset.
8. **Endpoints.** `GET /api/presets`, `POST /api/presets/load`, `POST /api/presets/save`,
   `POST /api/presets/import`, `GET /api/presets/<id>/export`; chaves `scene.active` (enum `A`–`H`)
   e `<bloco>.channel` (enum `A`–`D`) no registro da Spec 014.

## Critérios de aceite
- **CA-1:** Carregar cada um dos 4 presets de fábrica deixa `GET /api/state` igual ao conteúdo do
  arquivo (todas as chaves) e a UI mostra esses valores (teste JS com `fetch` simulado).
- **CA-2:** Troca de preset com seno de 1 kHz: descontinuidade ≤ 1,1 × a de regime (régua da
  Spec 020 CA-2), troca concluída em 20 ms ± 1 bloco, 0 alocações no callback; a cauda do delay
  continua audível depois da troca quando `spillover` está ligado.
- **CA-3:** Salvar um preset, reiniciar o app e carregá-lo restaura `GET /api/state` idêntico.
- **CA-4:** O arquivo exportado passa em `SonixPresetDoc::from_json_str`; importá-lo restaura o
  estado; JSON malformado ou `schemaVersion` desconhecido mostra o erro na UI e não altera nada
  (Spec 010 CA-6). Um documento 1.0.0 é importado com os valores no boot nas chaves ausentes.
- **CA-5:** Preset com IR carregada exportado como `.sforgepkg` inclui a IR com SHA-256 (Spec 010
  CA-3); importar restaura a IR ativa no cab.
- **CA-6:** Trocar da cena C para a F envia uma única operação com só as chaves que diferem (teste
  JS) e não gera descontinuidade de áudio.
- **CA-7:** Alterar `amp_1.gain` na cena C, clicar "Salvar cena", ir para a A e voltar à C
  restaura o valor alterado.
- **CA-8:** `amp_1` com canal A em `gain`=3 e canal D em `gain`=9: selecionar D aplica 9, voltar a
  A aplica 3; a cena A guarda o canal A de `amp_1`.
- **CA-9:** O selo de troca de cena mostra o tempo medido devolvido pelo motor; o texto fixo
  "1.1ms" deixa de existir em `app.js` (teste JS).
- **CA-10:** Na primeira execução o app abre com `brit800`; nas seguintes, com o último preset
  carregado.
- **CA-11:** `POST /api/presets/load` com id inexistente → `404`; `scene.active` fora de `A`–`H` →
  `400`; em ambos o estado não muda (Spec 009 CA-8).
- **CA-12:** Não-regressão — testes das Specs 009, 010 e 014 continuam verdes.

## Fora de escopo
- Modifiers por clique direito e modos de footswitch (Spec 009 CA-5/CA-7): não existem na UI atual;
  o pedal de expressão segue mapeado para `amp_1.gain` (Spec 014). Spec futura.
- Setlists e sincronização entre máquinas.

## Invariantes tocadas
- 100 % offline (Spec 010): presets em arquivo local; nada de rede.
- Áudio em tempo real (Spec 001/018): estado completo preparado fora do callback e trocado por
  referência.
- Spec 010: o schema evolui para 1.1.0 com migração; 1.0.0 continua aceito.

## Testes que cobrem esta spec
Planejados (red → green): `crates/sonix-core/tests/contract_025_presets_scenes.rs` (CA-3…CA-5,
CA-8, CA-11, CA-12), `crates/sonix-desktop/tests/contract_025_preset_switch.rs` (CA-2, CA-10) e
`tests/contract/025-presets-cenas-canais-de-bloco-reais.test.js` (CA-1, CA-6, CA-7, CA-9).

## Notas
- Conteúdo dos presets de fábrica (intenção; valores definidos e medidos no PR):
  `brit800` — `jcm800_2203` + 808 + `greenback_4x12_closed`, caminho B desligado;
  `usdeluxe` — `twin_ab763` limpo + chorus + placa, `alnico_blue_2x12_open`;
  `recto` — `dual_rect` + `v30_4x12_closed`, gate mais fechado;
  `ac30` — `ac30_tb` + `alnico_blue_2x12_open`.
- Este preset de boot decide a questão aberta da Spec 024 §4 (caminho B ligado ou não no boot).
- Depende das Specs 010, 014 e 018-024.
