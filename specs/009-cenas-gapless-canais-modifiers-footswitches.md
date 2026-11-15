---
spec: 009
titulo: Hierarquia Setlist/Preset, 8 Scenes gapless, 4 canais por bloco, Modifiers e modos de footswitch
status: implementada
atualizado: 2026-11-08
---
# 009 — Hierarquia Setlist/Preset, 8 Scenes gapless, 4 canais por bloco, Modifiers e modos de footswitch

## Objetivo
Permitir ao músico organizar seu repertório de palco em Setlists e Presets com 8 cenas instantâneas sem corte de áudio (*gapless*), 4 canais independentes por bloco sem custo de CPU ociosa, automação universal de parâmetros por Modifiers e 4 modos operacionais para os 8 footswitches físicos.

## Contexto
Em performances ao vivo, carregar um Preset inteiro do zero pode reconfigurar o grafo completo, mas dentro de uma mesma música o guitarrista precisa alternar instantaneamente entre *Intro Clean*, *Verse Crunch*, *Chorus Rhythm* e *Lead Solo (+3 dB + Delay)* em menos de 1,5 ms, sem qualquer corte (*gapless*), trocando simultaneamente o modelo/canal do amplificador e o estado de múltiplos pedais.

Fontes da verdade e evidências no repositório:
- `docs/01-SRS-Levantamento-de-Requisitos.md:154-179` — Define o diagrama de estados `Setlist → Preset → 8 Scenes` e os requisitos **RF-028** (Hierarquia Setlist → Preset → 8 Scenes A–H), **RF-029** (4 Canais A/B/C/D independentes por bloco na memória, sem consumir CPU dos canais inativos), **RF-030** (Troca de Cena Gapless com crossfade cosseno de 1,5 ms e Spillover), **RF-031** (Sistema Universal de Modifiers via clique direito em qualquer knob: `Expression Pedal 1/2`, `MIDI CC 0–127`, `Internal LFO 1/2`, `Envelope Follower`, `Scene Controller` com `Min`, `Max`, `Slope/Midpoint`, `Damping ms`) e **RF-032** (4 modos de Footswitch: `Preset Mode`, `Scene Mode`, `Stomp Mode`, `Hybrid Mode`).
- `docs/02-Design-System-Specification.md:20-22,94,115-120` — Especifica o clique direito nos knobs para abrir a matriz de Modifiers, o segundo arco colorido (`Min ↔ Max`) nos knobs modulados e os estados dos 8 footswitches (`100% + Glow`, `Dimmed 25%`, `Piscando no BPM`).
- `design-system/index.html:32-54,121-127,505-523` e `design-system/app.js:60-71,438-541,658-668,729-737` — O protótipo implementa os 8 botões de Scene A–H, os botões de Canal A–D e os 8 footswitches fixos apenas em `Hybrid Mode` (`app.js:442`), porém: (a) trocar de Canal A/B/C/D em `app.js:658-668` altera apenas o rótulo `blk.ch` sem carregar parâmetros independentes por canal, (b) os outros 3 modos de footswitch (`PRESET`, `SCENE`, `STOMP`) não estão implementados, (c) a organização em `Setlist` não está implementada, e (d) a matriz de Modifiers por clique direito nos knobs não abre modal de edição (apenas o `EXP 1` está acoplado de forma fixa ao `inputGain` em `app.js:729-737`).

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Hierarquia `Setlist → Preset → 8 Scenes` (**RF-028**) | `<select id="presetSelect">` com 4 opções sem estado próprio e 8 Scenes (`design-system/index.html:34-54`) | Hierarquia completa `Setlist` contendo múltiplos `Presets`, cada um com seu Grid 4×12 e 8 `Scenes (A..H)` independentes |
| 4 Canais A/B/C/D por Bloco (**RF-029**) | Troca apenas a letra do badge `blk.ch` (`design-system/app.js:658-668`) | Cada bloco armazena 4 objetos de estado completos (`channels.A`, `B`, `C`, `D`) na memória; apenas o canal ativo é processado na thread de áudio (`0%` de CPU pelos 3 canais inativos) |
| Troca de Cena Gapless (**RF-030**) | Atualiza estado da UI e texto do badge (`design-system/app.js:503-541`) | Aplica `activeChannelByScene[s]` e `bypassByScene[s]` em todos os blocos com crossfade em janela cosseno de `1.5 ms` e preserva caudas de Delay/Reverb |
| Matriz Universal de Modifiers (**RF-031**) | `EXP 1` fixo em `inputGain` (`design-system/app.js:729-737`) | Clique direito (`contextmenu`) em qualquer knob abre o painel `Modifier Matrix` com fontes `EXP_PEDAL_1/2`, `MIDI_CC (0-127)`, `LFO_1/2`, `ENVELOPE_FOLLOWER`, `SCENE_CONTROLLER` e parâmetros `minVal`, `maxVal`, `curveSlope`, `dampingMs` |
| Modos de Footswitch (**RF-032**) | Fixo em `Hybrid Mode` (`design-system/app.js:442`) | Alternância completa entre `PRESET_MODE` (8 presets do banco), `SCENE_MODE` (Scenes A–H), `STOMP_MODE` (8 blocos de efeito) e `HYBRID_STOMP_SCENE` (4 Stomps + 4 Scenes) |

**Alternativas descartadas:**
1. *Manter os 4 canais A/B/C/D de cada bloco rodando simultaneamente em paralelo na CPU para trocar o áudio:* descartada por quadruplicar o consumo de CPU e violar explicitamente o **RF-029** ("sem consumir CPU dos canais inativos"); em vez disso, o estado dos 4 canais reside pré-alocado em memória e a troca de canal executa um crossfade curto de `1.5 ms` em janela cosseno entre o buffer de saída anterior e o novo canal ativado.
2. *Limitar automação apenas ao Pedal de Expressão:* descartada porque o **RF-031** exige fontes internas (`LFO 1/2`, `Envelope Follower`, `Scene Controller`) e externas (`MIDI CC 0–127`).

## Comportamento esperado
1. **Hierarquia Setlist → Preset → 8 Scenes (RF-028):**
   - O gerenciador de estado organiza coleções de **Setlists**; cada Setlist contém uma lista ordenada de **Presets** (`P01`, `P02`, ...);
   - Selecionar outro Preset em `#presetSelect` carrega sua topologia 4×12, seus modelos de amplificador/efeitos, seu `tempoBpm` e suas **8 Scenes (`A` a `H`, índices `0` a `7`)**.
2. **4 Canais Independentes por Bloco (RF-029):**
   - Todo bloco no Grid possui 4 slots de canal independentes (`A`, `B`, `C`, `D`), cada um podendo armazenar tanto ajustes distintos quanto um `modelId` diferente da mesma categoria (ex.: `AMP 1` com `us_twin_blackface_65` no Canal A, `uk_ac30_topboost` no Canal B, `brit_jcm800_2203` no Canal C e `cali_dual_recto_revf` no Canal D).
   - O agendador do DAG processa exclusivamente o canal ativo do bloco na cena corrente; os 3 canais inativos consomem `0.0%` de orçamento de CPU.
3. **Troca Gapless de Cenas (RF-030):**
   - Acionar uma Scene $s \in \{0..7\}$ lê os vetores `activeChannelByScene[s]` e `bypassByScene[s]` de cada bloco do Grid.
   - No motor de áudio, a transição aplica uma janela cosseno complementar ($w_{\text{out}}(t) = \cos^2(\frac{\pi t}{2 T})$, $w_{\text{in}}(t) = \sin^2(\frac{\pi t}{2 T})$) com duração $T = 1,5\text{ ms}$ (`72 samples` a `48 kHz`), eliminando qualquer degrau de fase ou clique, enquanto blocos de `DELAY` e `REVERB` mantêm suas caudas via `Spillover`.
4. **Sistema Universal de Modifiers (RF-031):**
   - Disparar clique direito (`contextmenu`) sobre qualquer `<sonix-knob>` abre o painel **Modifier Matrix** vinculado àquele `targetBlockId` e `targetParam`.
   - Permite vincular uma fonte `source` (`EXP_PEDAL_1`, `EXP_PEDAL_2`, `MIDI_CC_0..127`, `INTERNAL_LFO_1`, `INTERNAL_LFO_2`, `ENVELOPE_FOLLOWER`, `SCENE_CONTROLLER`) e configurar `minVal`, `maxVal`, `curveSlope` (`0.0` a `1.0`, onde `0.5` é linear) e `dampingMs` (`0` a `1000 ms`).
   - Enquanto um knob possui Modifier ativo, ele recebe a classe `.has-modifier`, renderiza o segundo arco colorido perimetral indicando o intervalo `minVal ↔ maxVal` e atualiza sua posição modulada em tempo real.
5. **Modos dos 8 Footswitches Físicos (RF-032):**
   - O usuário pode selecionar o modo de operação dos 8 footswitches (`FS 1` a `FS 8`):
     - `PRESET`: `FS 1..8` carregam os 8 presets do banco atual;
     - `SCENE`: `FS 1..8` selecionam diretamente as Scenes `A` até `H`;
     - `STOMP`: `FS 1..8` ligam/desligam até 8 blocos individuais do Grid como pedaleira analógica;
     - `HYBRID_STOMP_SCENE`: `FS 1..4` operam como Stomps e `FS 5..8` selecionam as 4 Scenes principais (`A..D`).

## Critérios de aceite
- **CA-1:** A estrutura de estado suporta múltiplos `Setlists`, múltiplos `Presets` por Setlist e exatamente `8 Scenes (A..H)` por Preset; trocar o Preset em `#presetSelect` reconstrói o Grid, os knobs, as 8 cenas e os *scribble strips* OLED de acordo com o Preset selecionado.
- **CA-2:** Cada bloco do Grid mantém configurações independentes nos 4 canais (`A`, `B`, `C`, `D`); editar um parâmetro ou trocar o `modelId` no Canal `C` não altera os parâmetros nem o `modelId` salvos nos Canais `A`, `B` ou `D` do mesmo bloco.
- **CA-3:** O cálculo e a medição de CPU contabilizam apenas o canal ativo de cada bloco não-bypassado; os 3 canais inativos de qualquer bloco contribuem com exatamente `0.0%` de carga de CPU.
- **CA-4:** A troca entre quaisquer duas Scenes (`A..H`) atualiza simultaneamente `activeChannelByScene` e `bypassByScene` de todos os blocos aplicando crossfade em janela cosseno de `1.5 ms` sem gerar amostras nulas (silêncio espúrio) nem descontinuidade de amplitude acima do limite de suavização.
- **CA-5:** Disparar o evento `contextmenu` (clique direito) em qualquer knob abre o painel `Modifier Matrix` para aquele parâmetro, permitindo criar, editar ou remover um vínculo com `EXP_PEDAL_1/2`, `MIDI_CC`, `INTERNAL_LFO_1/2`, `ENVELOPE_FOLLOWER` ou `SCENE_CONTROLLER` com `minVal`, `maxVal`, `curveSlope` e `dampingMs`.
- **CA-6:** Para um Modifier ativo com entrada normalizada $u \in [0, 1]$, o valor aplicado ao parâmetro segue a função de transferência parametrizada por `minVal`, `maxVal` e `curveSlope` (onde `u = 0` produz `minVal`, `u = 1` produz `maxVal`, e `curveSlope = 0.5` produz interpolação linear exata $\text{minVal} + u \cdot (\text{maxVal} - \text{minVal})$), suavizada pelo filtro de amortecimento `dampingMs`.
- **CA-7:** Alternar `footswitchMode` entre `PRESET`, `SCENE`, `STOMP` e `HYBRID_STOMP_SCENE` reconfigura os 8 elementos `.footswitch-unit` no `#footswitchArray` (rótulos OLED `.oled-top-line`, `.oled-main-text`, cores `--fs-color` e callbacks de clique).
- **CA-8:** Caso de erro — tentar ativar um índice de Scene fora de `0..7`, um identificador de canal fora de `{'A','B','C','D'}` ou um Modifier apontando para um `targetBlockId` inexistente é rejeitado com erro de validação explícito sem alterar a cena ou o canal em execução.
- **CA-9:** Não-regressão — parâmetros que não possuem Modifier vinculado continuam respondendo diretamente ao arraste vertical, `Shift+Drag` e duplo clique sem interferência do avaliador de automação.

## Fora de escopo
- O formato de empacotamento em disco `.sonixpreset` e `.sforgepkg` com verificação SHA-256 (coberto pela **Spec 010**).

## Invariantes tocadas
- **RNF-005 (Real-Time Safe):** Os 4 canais de cada bloco são pré-alocados na construção do Preset; trocar entre os canais A/B/C/D ou entre Scenes A–H na thread de áudio apenas alterna ponteiros/índices pré-existentes e executa o crossfader de `1.5 ms`.
- **RNF-006 (Zipper-Noise Free):** O amortecimento `dampingMs` do Modifier e o crossfade cosseno de `1.5 ms` garantem transições livres de cliques.

## Testes que cobrem esta spec
- `crates/sonix-core/tests/contract_009_scenes_modifiers.rs` → CA-1 a CA-9 (9 testes unitários de contrato em Rust, 100% ok).
- `tests/contract/009-cenas-gapless-canais-modifiers-footswitches.test.js` → Suíte de integração e validação de critérios CA-1 a CA-9.

## Notas
- Rastreabilidade SRS: Seção 3.5, **RF-028**, **RF-029**, **RF-030**, **RF-031**, **RF-032**, Seção 5 (`scenes`, `activeChannelByScene`, `bypassByScene`, `modifiers`).
- **2026-11-14 (Auditoria Spec 044):** `Preset`, `Scene`, `BlockNode`, `ModifierMatrix`, `FootswitchArray` e `GaplessCrossfader` em `sonix-core` ainda não dirigem a cadeia de áudio em produção; integração retomada pela **Spec 045**.
