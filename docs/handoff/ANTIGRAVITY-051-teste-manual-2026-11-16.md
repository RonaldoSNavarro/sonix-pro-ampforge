# Handoff — achados do teste manual de 2026-11-16 (para o Antigravity)

> **Origem:** teste manual do usuário em 2026-11-16 — guitarra na entrada de microfone da placa onboard, WASAPI low-latency, 48 kHz, período 128/128. Relato original: `Testes_manuais_e_obervações_do_comportamento_sonix_ampforge.docx`. As 26 capturas estão em `evidencias-051/` (o número do arquivo = ordem das imagens no relato).
> **Base de código analisada:** `b7ec90f` (HEAD de `main` em 2026-11-16).
> **Método e limite:** leitura estática do repositório cruzada com as capturas. **O app e a suíte de testes não foram executados** nesta análise. Cada item tem uma marca:
> - **[CONFIRMADA]** — a causa está visível no código (arquivo e linha aproximada; confira antes de editar).
> - **[DIAGNOSTICAR]** — o sintoma é real (relato + captura), mas a causa não foi provada. Reproduza e meça **antes** de corrigir.

## 0. Leia antes, nesta ordem
1. `AGENTS.md` e `CLAUDE.md` (SDD + TDD estrito, zero-alloc, "todo controle visível funciona no backend", régua medida).
2. `specs/README.md` e as specs citadas em cada item.
3. `docs/handoff/ANTIGRAVITY-050-pos-review.md` (formato e disciplina de relatório).
4. `.agents/skills/sonix-pro-ampforge-expert/SKILL.md` (itens de DSP) e `cortex-conventions` (`query` no início, `capture` no fim).

## 1. Regras inegociáveis
- **TDD estrito:** cada item começa por um teste que reproduz o defeito e é **mostrado falhando** (red); depois a correção (green). O nome do teste cita o CA.
- **Régua medida** (regra `20261114`): teste que não falha quando o comportamento quebra é tautológico. Mostre o red com a quebra real.
- **Todo controle visível funciona no backend** (regra `20261113`). Controle sem DSP fica **desabilitado**, com dica em português, e conta como **pendência**; a spec só vai para `implementada` sem pendências.
- **Reproduzir antes de corrigir** nos itens [DIAGNOSTICAR]: registre o valor medido de hoje ("antes") e o de depois.
- **Não invente números.** Limiares de "efeito mensurável" saem do equipamento real documentado (ex.: `docs/ui-desktop/fontes/` e `docs/amps/voicing-fontes.md`); registre a fonte na spec. Sem fonte, use `[verificação pendente]`.
- **Não reescreva teste antigo para passar.** Se um teste antigo afirmar o defeito, corrija a asserção e registre nas Notas como "revertido", com o motivo.
- **Callback de áudio:** zero alocação, desalocação, lock, I/O e log (alocador de contagem da Spec 037).
- **Git:** nunca `commit`, `push`, `rebase` ou `reset` sem autorização explícita.

## 2. Por que a suíte (93/93 verde) convive com estes defeitos
Três causas sistêmicas, todas vistas no código:
- **S-A — testes de inventário leem registro e fontes, não exercitam o caminho real.** O registro (`controls.registry.ts`) lista `top_boost` como opção válida de `amp.input_jack`; o backend não aceita. A UI envia `scene_id`; o backend espera `scene`/`index`. A UI envia `{action:'start'}`; o backend espera `phase`. Nenhum teste percorreu "UI → transport → `api::dispatch`" com o payload real.
- **S-B — status do índice diz "Implementada", a UI diz "em implementação".** Os chips de Modifier (`Estudio.ts` ~909–923) nascem `disabled` com a dica "em implementação — Spec 045", embora as Specs 045 e 049 estejam `Implementada` no índice. Pela regra `20261113`, isso é pendência aberta, não spec fechada.
- **S-C — erro de validação aparece como "Motor DSP desconectado".** `shell.ts` ~346 usa um texto fixo para qualquer `bridgeError`. O motor está vivo (a barra de status segue "ponte UI→DSP ok"); o usuário lê "desconectado" e não sabe o que fazer.

## 3. Como tratar as specs (decisão proposta — o usuário pode alterar)
- **Spec 051 — Correção de contratos e controles sem efeito** (itens T-00 a T-26). Quando o achado já viola um CA existente, o CA **não muda**; só o código e o teste que deveria tê-lo pego. Cite o CA violado.
- **Spec 052 — Melhorias e novos componentes** (itens T-27 a T-29: abertura animada, pedais sem design, AMP 2 visível). É mudança de comportamento/design, então nasce de spec com **referência visual antes do código** (regra da Spec 048).
- Registre as duas em `specs/README.md`. Onde a entrega de uma spec anterior estiver incorreta, ajuste o status para o estado honesto (`Implementada (CA-N reaberta pela Spec 051)`), como a Spec 044 fez.
- *Alternativa:* spec única com partes A–F, como foi feito na 049. Se o usuário preferir, junte 051+052.

## 4. Resumo dos itens

| ID | Título | Prio | Marca | Spec de origem |
|---|---|---|---|---|
| T-00 | Régua de contrato UI↔backend (fazer primeiro) | P0 | CONFIRMADA | 033, 035, 044 |
| T-01 | Assistente de calibração falha (`phase`) | P0 | CONFIRMADA | 019, 029 |
| T-02 | Erro de validação mostrado como "Motor DSP desconectado" | P0 | CONFIRMADA | 035, 036 |
| T-03 | Botões de cena dão erro (`scene_id`) | P0 | CONFIRMADA | 045 |
| T-04 | Jack `top_boost` do AC30 rejeitado | P0 | CONFIRMADA (+caminho a reproduzir) | 031, 033 |
| T-05 | Setas de preset: `preset_not_found` | P0 | CONFIRMADA (causa provável) | 025, 045 |
| T-06 | Tone stack "Mesa Pré" rejeitado | P0 | CONFIRMADA | 004, 047 |
| T-07 | Mix de modulação exibido como 5000 % / 3300 % | P0 | CONFIRMADA | 040 |
| T-08 | Knobs do amp quase não mudam o som (Brit 800) | P0 | DIAGNOSTICAR | 031, 033, 039 |
| T-09 | Chips de Modifier desabilitados ("O que é isso?") | P1 | CONFIRMADA | 045, 049 |
| T-10 | Noise gate e pedais de drive/fuzz: efeito, sincronia e rótulos | P1 | DIAGNOSTICAR | 030, 033, 041 |
| T-11 | AMP 2 / caminho paralelo sem efeito; "Nível Master" ligado à chave errada | P0 | CONFIRMADA + DIAGNOSTICAR | 024, 049 |
| T-12 | Gabinetes 1 e 2: sem reação (chaves de leitura ≠ escrita; CAB 2 edita CAB 1) | P0 | CONFIRMADA + DIAGNOSTICAR | 022, 038 |
| T-13 | Afinador e Looper não funcionam | P0 | DIAGNOSTICAR | 026, 029, 043 |
| T-14 | Footswitches e wah/EXP não funcionam | P0 | DIAGNOSTICAR | 009, 049 |
| T-15 | Chassi: válvula, bias, osciloscópio | P0 | DIAGNOSTICAR | 021, 047 |
| T-16 | Sala de microfonação: arrastar, roda, controles | P0 | DIAGNOSTICAR | 022, 038 |
| T-17 | Oversampling/Desempenho sem efeito; botão WAV DI morto | P1 | CONFIRMADA (botão) + DIAGNOSTICAR | 020, 049 |
| T-18 | TAP sem efeito claro | P1 | DIAGNOSTICAR | 007, 040 |
| T-19 | Pedalboard fixo: só 4 pedais, escala 1:1, espaço vazio | P1 | CONFIRMADA | 048 |
| T-20 | Racks pequenos demais para ler e operar | P1 | CONFIRMADA | 048 |
| T-21 | Pedais/racks deformados na Biblioteca | P1 | DIAGNOSTICAR | 048 |
| T-22 | Amps da Biblioteca não se parecem com o rig | P2 | DIAGNOSTICAR | 048 |
| T-23 | Afinador × Palco × Config. de áudio × Status do motor | P1 | CONFIRMADA (roteamento) | 033, 036 |
| T-24 | Clareza: canais A–D, cenas, textos cortados, jargão "Spec NNN" | P2 | CONFIRMADA | 033, 048 |
| T-25 | Latência alta e xruns | P0 | DIAGNOSTICAR | 018, 028, 036, 050 |
| T-26 | Rótulos sobrepostos (pedais, legenda dos mics) | P2 | CONFIRMADA (visual) | 048 |
| T-27 | Abertura animada com a logomarca | P2 | feature | 052 |
| T-28 | Pedais sem design (flanger, wah, phaser, tremolo…) | P2 | feature | 052 |
| T-29 | AMP 2 visível no Estúdio em tamanho reduzido | P2 | feature | 052 |

---

## 5. Itens — Spec 051

### T-00 · Régua de contrato UI↔backend — P0 — fazer primeiro
- **Por quê:** os defeitos T-01 a T-06 são todos do mesmo tipo (UI e backend discordam do nome, do valor ou do formato) e nenhum teste os pegou (S-A).
- **Teste (red esperado nos quatro casos conhecidos):**
  1. Para cada controle `type: 'enum'` do registro, enviar **cada opção** pelo caminho real (`transport.ts` → comando Tauri → `api::dispatch`) e exigir aceite. Red: `top_boost`, `mesa`.
  2. Para cada chamada de `transport.ts`, validar o formato do payload contra o que o backend aceita (estender `tests/fixtures/ipc-schemas/`). Red: `select_scene {scene_id}`, `calibrate_input {action}`.
  3. Varrer `ui/src/` por `setControl('<id>', '<literal>')` cujo literal não esteja nas `options` do registro.
- **Aceite:** os três passam verdes **depois** de T-01…T-06; reverter qualquer correção faz o teste correspondente falhar.

### T-01 · Assistente de calibração falha — P0 — [CONFIRMADA]
- **Relato:** "Assistente de calibração não funciona e exibe mensagem de erro." **Evidência:** 18.
- **Causa:** `Audio.ts` ~372 chama `calibrateInput({ action: 'start' })`; `api.rs` ~4324 só aceita `phase` = `'silence'` ou `'strum'` e responde 400 `phase deve ser 'silence' ou 'strum'`.
- **Correção:** conduzir o assistente de 3 passos (silêncio 3 s → tocar forte 5 s → resultado) enviando `phase: 'silence'` e depois `phase: 'strum'`; consumir a resposta (`noise_floor_rms_db`, resultado proposto) e aplicar boost e gate propostos (Spec 029). Mostrar progresso real, não só "Iniciar".
- **Efeito colateral importante:** sem calibração, a barra de status mostra "não calibrada · pico −38,8 dBFS" (imagem 01), contra o alvo de −12 dBFS do próprio assistente (imagem 18). Isso pode explicar parte de T-08 e T-10. **Acrescente:** aviso visível "sinal baixo — calibre a entrada" quando o pico ficar muito abaixo do alvo.
- **Verifique:** o CA-5 da Spec 050 (calibração concluída tira o amp do standby) está coberto por teste.
- **Teste:** fluxo completo silêncio → strum → resultado com sinal de teste definido: backend retorna 200 em cada fase e o boost aplicado chega ao DSP.

### T-02 · Erro de validação aparece como "Motor DSP desconectado" — P0 — [CONFIRMADA]
- **Relato:** "mensagem de erro em vermelho" ao trocar de amp, clicar nas setas, nas cenas, no Circuito e na calibração. **Evidências:** 07, 08, 18, 25.
- **Causa:** `shell.ts` ~346 escreve `Motor DSP desconectado (${store.bridgeError})` para qualquer erro; `store.ts` ~219 grava o erro e ele permanece.
- **Correção:** separar *queda do IPC* de *erro de validação/engine*. Validação → aviso não bloqueante (toast), mensagem em português claro, some sozinha, sem nome de chave interna. O banner vermelho fica só para queda real do IPC e some ao reconectar.
- **Teste:** erro 400 simulado **não** mostra "desconectado"; queda real do IPC mostra e some ao reconectar.

### T-03 · Botões de cena dão erro — P0 — [CONFIRMADA]
- **Relato:** "Os botões de cena também exibem o erro ao clicar, também não ficou muito claro a funcionalidade deles." **Evidência:** 09.
- **Causa:** `store.ts` ~503 envia `selectScene({ scene_id })`; `api.rs` ~3795–3812 só lê `scene` (letra A–H) ou `index`; sem eles responde 400 `missing_scene`.
- **Correção:** alinhar o payload (preferir `scene` com a letra) e consumir `state` da resposta. Clareza da função: ver T-24.
- **Teste:** clicar B…H muda `scene.active` e o estado dos blocos (Spec 045); nenhuma cena retorna 400.

### T-04 · Jack `top_boost` do AC30 rejeitado — P0 — [CONFIRMADA] + caminho a reproduzir
- **Relato:** "Ao mudar de amplificador, uma mensagem de erro em vermelho é exibida" (`amp_1.input_jack inválido`). **Evidência:** 07.
- **Causa:** `AmpHead.ts` ~691–694 envia `'top_boost'`; `controls.registry.ts` ~289 o lista como opção; `AmpInputJack::from_str_name` (`amp.rs` ~46–60) não tem `top_boost`, então `api.rs` ~2254–2262 rejeita.
- **A reproduzir:** confirmar se **trocar de modelo** dispara esse envio (por exemplo via valor padrão/restauração do jack) ou se só o clique em "Top Boost" o faz. O relato indica erro na troca; o caminho exato não foi provado.
- **Correção:** decidir uma fonte única do vocabulário de jacks por modelo (enum do DSP) e fazer UI e registro derivarem dela. Trocar de modelo nunca deve enviar jack inválido para o novo modelo.
- **Teste:** para cada modelo, trocar `amp_1.model` e percorrer todos os jacks do painel: 200 em todos.

### T-05 · Setas de preset: `preset_not_found` — P0 — [CONFIRMADA] (causa provável)
- **Relato:** "Ao clicar nas setas para direita e esquerda, a mensagem de erro também aparece." **Evidência:** 08.
- **Causa provável:** `find_preset_file_in` (`api.rs` ~3411–3446) procura `presets/factory` e variantes **relativas ao diretório de trabalho**, além de `%APPDATA%\SonixPro\presets`. A lista exibida vem de `factory_defs` fixo em `handle_get_presets` (~3450), **desacoplada dos arquivos**. Se o app abrir fora da raiz do repo (atalho, instalador), a lista aparece e o arquivo não é achado.
- **Defeito de UI junto:** `store.ts` ~517 atualiza `presetIdx` **antes** do carregamento; o display troca de nome ("RECTO HIGH GAIN", imagens 09 e 16) mesmo quando o load falha.
- **Pergunte ao usuário:** qual executável e qual diretório de trabalho ele usou (o `SonixPro.exe` da raiz, o build release do Tauri ou outro).
- **Correção:** embutir os presets de fábrica no binário (`include_str!`) ou resolver pelo caminho do executável; listar e carregar da **mesma** fonte; só avançar o índice após sucesso; o erro cita o nome do preset.
- **Teste:** iniciar com diretório de trabalho diferente (temporário) e carregar cada preset de fábrica (red hoje se a causa se confirmar).

### T-06 · Tone stack "Mesa Pré" rejeitado — P0 — [CONFIRMADA]
- **Relato:** "Circuito exibe mensagem de erro." **Evidência:** 25 (`amp_1.tonestack inválido`).
- **Causa:** `Bancada.ts` ~702 oferece o chip `['mesa', 'Mesa Pré']`; o backend aceita só `model`, `marshall`, `fender`, `vox`, `baxandall` (`api.rs` ~2412; mesmas `options` no registro ~668). Também há divergência de padrão: `Bancada.ts` usa `'marshall'`, o registro usa `'model'`.
- **Correção:** não existe topologia Mesa no motor. Remova o chip ou mostre-o **desabilitado** ("em implementação") e registre a pendência. Não invente valores de componentes para uma topologia nova (skill do especialista: exige revisão e fonte). Unifique o padrão.
- **Teste:** cada chip do Circuito é aceito pelo backend (coberto por T-00).

### T-07 · Mix de modulação exibido como 5000 % / 3300 % — P0 — [CONFIRMADA]
- **Relato:** não citado, visível nas capturas 01 e 15 (Mix "5000%", "3300%").
- **Causa:** o registro define `mod_2.mix` com faixa 0–100 e padrão 50 (`controls.registry.ts` ~980–990); `Estudio.ts` ~250 formata com `Math.round(v * 100)%`, ou seja, multiplica por 100 um valor que já está em porcentagem. O padrão local `0.5` no mesmo trecho também está na escala errada.
- **Correção:** uma única escala (a do registro, 0–100 %). Revise os demais sliders de `Estudio.ts` por unidade/escala (rate, depth, decay, pre-delay).
- **Teste:** valor 50 exibe "50 %"; mover o slider ao máximo exibe "100 %" e o backend recebe o mesmo número.

### T-08 · Knobs do amp quase não mudam o som (Brit 800) — P0 — [DIAGNOSTICAR]
- **Relato:** "o sistema abre direto no brit 800, os botões parecem não funcionar pois quase não mudam o som, o jumper do lado direito não faz nada, não consegui aumentar ou diminuir o drive. Na guitarra ao abrir ou fechar o treble, o som não sofre grandes mudanças (deveria mudar)." **Evidências:** 01, 22.
- **O que já se sabe:** a ligação UI→chave existe (`AmpHead.ts` mapeia `amp.treble`→`amp_1.treble`, `amp.gain`→`amp_1.gain`, `amp.master`→`amp_1.master`; faixa 0–10 no registro). Portanto o problema não é de nome de chave.
- **Hipótese forte, não provada:** nível de entrada. A barra de status mostra pico ≈ −39 dBFS e "não calibrada" (imagens 01 e 02), cerca de 27 dB abaixo do alvo de −12 dBFS do assistente. Com sinal tão baixo, o drive do pré e a interação do tone stack ficam pouco audíveis. E o assistente que corrigiria isso está quebrado (T-01).
- **Protocolo:** (1) corrija T-01 e calibre; (2) com `input.source = test_riff` e a cadeia offline (como em `contract_033_control_effects.rs`), varra `amp_1.gain`, `treble`, `bass`, `middle`, `presence`, `master` de 0 a 10 e meça RMS e energia por banda; (3) registre a tabela "antes/depois"; (4) compare o sentido e a ordem de grandeza com o equipamento real (manual em `docs/ui-desktop/fontes/`).
- **Jack HIGH/LOW:** o registro diz que LOW atenua ~6 dB; meça o delta real. Se o painel do Brit 800 tiver chave que não alimenta nada, torne-a `visualOnly` declarada ou ligue-a ao DSP.
- **Aceite:** cada knob do amp tem um teste de efeito mensurável no sentido do equipamento real (regra `20261113`), com limiar e fonte registrados na spec.

### T-09 · Chips de Modifier desabilitados — P1 — [CONFIRMADA]
- **Relato:** "O que é isso: ao clicar, nada acontece." **Evidência:** 02.
- **Causa:** `Estudio.ts` ~909–923 cria os chips EXP 1 / EXP 2 / LFO 1 / ENV / MIDI CC com `disabled = true` e a dica "em implementação — Spec 045"; o texto "EXP 1 → faixa 20–80 % · Spec 045" é fixo. Já existem `components/ModifierMatrixModal.ts`, `sonix-core/src/modifiers.rs` e o Modifier no motor (Spec 049 CA-32).
- **Correção:** ligar os chips ao modal/atribuição real (fonte → parâmetro, com faixa), refletir o estado verdadeiro no texto do rodapé e explicar em uma linha o que é um Modifier ("fonte que move um parâmetro do bloco dentro de uma faixa"). Se algum DSP realmente não existir, o chip continua desabilitado **e** entra como pendência no índice.
- **Teste:** atribuir `EXP 1 → gate.threshold_db` e mover o pedal muda o parâmetro efetivo; remover o vínculo restaura o valor-base (Spec 049 CA-32).

### T-10 · Noise gate e pedais de drive/fuzz — P1 — [DIAGNOSTICAR]
- **Relato:** "Os controles do noise gate aparentam funcionar, reduzindo o chiado quando aumento o threshold, porém só faz um drástico efeito a partir da metade e não notei diferença nos demais controles. Ao girar um botão, as barras deveriam acompanhar e vice e versa… O mesmo vale para os pedais de distorção/fuzz." **Evidências:** 03, 04, 05, 06.
- **(a) Efeito só da metade em diante:** com pico de entrada ≈ −39 dBFS, o gate só age quando o threshold passa do nível do sinal. Pode ser comportamento correto com entrada baixa. Meça a curva threshold×atenuação com sinal calibrado antes de mexer no mapeamento. Mesma regra para decay/hysteresis/hold/range (Spec 030, 041).
- **(b) Knob ↔ barra sem sincronia:** `store.setControl` emite `'param'` (arraste) ou `'full'`; verifique se o painel lateral e o pedal reagem a `'param'`. Teste de UI: girar o knob do pedal move o slider no mesmo quadro e vice-versa.
- **(c) Conjunto de parâmetros diferente:** o pedal GATE-2 mostra Threshold / Decay / **Mode**; o painel mostra Threshold / Decay / **Hysteresis**. O mesmo bloco deve expor o mesmo conjunto nas duas representações, ou o rótulo explicar o mapeamento (Spec 033: divergência de nome fica no registro).
- **(d) Drives (Screamer, Klon, Rodent, Big Fuzz, Face Fuzz, DS-One):** o painel de drive mostra "Drive 10.0" (imagem 06) com o pedal Rodent; confirme efeito por modelo. Um teste de efeito por modelo e por parâmetro (Spec 041).

### T-11 · AMP 2 / caminho paralelo sem efeito — P0 — [CONFIRMADA] + [DIAGNOSTICAR]
- **Relato:** "O amp paralelo poderia aparecer na tela… e não está funcionando pois não altera o som." **Evidências:** 10, 11.
- **[CONFIRMADA] ligação errada:** em `Estudio.ts`, o slider "Nível Merge" (~198), "Balanço Linha B" (~312) e **"Nível Master" (~350 e ~357)** usam todos a mesma chave `merge.row_1_level_db`. O "Nível Master" do bloco de saída está movendo o nível da linha B.
- **[DIAGNOSTICAR] candidatos ao "não altera o som":**
  1. O painel mostra "Nível Merge −28,0 dB" (imagem 10), enquanto o padrão do backend é −6 dB (`api.rs` ~584) e a faixa é −60…12. Descubra quem grava −28 (preset? UI?). Em −28 dB o AMP 2 fica praticamente inaudível.
  2. Handoff 050, R1: blocos da linha 1 só rodam com `split_active`. Verifique se a correção entrou e se vale para o layout padrão.
  3. As chaves `amp_2.*` chegam ao DSP.
- **Teste:** split ativo, AMP 2 ligado, nível da linha B em 0 dB: trocar o modelo do AMP 2 muda a saída de forma mensurável; AMP 2 desligado ≠ ligado.

### T-12 · Gabinetes 1 e 2 sem reação — P0 — [CONFIRMADA] + [DIAGNOSTICAR]
- **Relato:** "Os parâmetros dos gabinetes 1 e 2 não funcionam, não tem nenhuma reação ao clicar." **Evidências:** 12, 13.
- **[CONFIRMADA] chaves de leitura ≠ escrita:** `Estudio.ts` ~207 lê o estado em `cab_1.model` e escreve por `cab.speaker` (`cab_1.speaker`). O chip não destaca após o clique, então parece que "nada acontece". O backend aceita as duas chaves na validação, mas o estado devolvido usa `cab_1.speaker`.
- **[CONFIRMADA] CAB 2 edita o CAB 1:** no mesmo bloco (`blk === 'cab' || 'cab2'`), os sliders Ressonância e Hi-Cut estão fixos em `cab_1.*`, mesmo no painel do CAB 2.
- **[DIAGNOSTICAR]:** confirmar se o efeito **audível** acontece (Spec 038: IR, microfones e ressonância com efeito real) quando o chip e os sliders mudam.
- **Teste:** trocar o gabinete destaca o chip certo e muda a resposta em frequência da saída; os sliders do CAB 2 alteram `cab_2.*` e não `cab_1.*`.

### T-13 · Afinador e Looper não funcionam — P0 — [DIAGNOSTICAR]
- **Relato:** "O afinador e Looper não funcionam." **Evidência:** 16 (Hz 0,00, +0,0 cents, nota vazia; looper "VAZIO 0 %").
- **A investigar:** (1) a nota "—" e 0,00 Hz com sinal presente: o botão **Mute** do afinador está ligado na captura; (2) o stream do afinador (`streams.rs`) chega à tela? (3) os botões do looper (Gravar/Tocar/Desfazer/Refazer/Reverso/½ velocidade/Limpar) enviam as chaves que o backend aceita (rodar T-00 neles)?
- **Teste:** com sinal de teste a 110 Hz, o afinador mostra A2 e ≈ 0 cent; gravar → tocar no looper reproduz a camada gravada (Specs 026, 043).

### T-14 · Footswitches e wah/EXP — P0 — [DIAGNOSTICAR]
- **Relato:** "Os footswitches e wah não funcionam." **Evidência:** 17.
- **O que se vê:** `Palco.ts` ~300 faz `setControl(fs.controlId, !active)`; o pedal EXP 1 "wah/volume" está em 62 %. **Não existe bloco/pedal de wah** no registro ou na lista de pedais (ver T-28), então EXP 1 só tem utilidade se for vinculado por Modifier (T-09).
- **Correção:** FS1–FS4 (stomp) ligam/desligam os blocos indicados no rig; FS5–FS8 (scene) trocam a cena (depende de T-03); o rig reflete o estado e vice-versa. EXP 1 sem alvo aparece desabilitado e explica por quê.
- **Teste:** footswitch em bypass deixa a saída igual à entrada (régua da regra `20261113`); a troca de cena por FS5–FS8 ecoa na tela.

### T-15 · Chassi: válvula, bias, osciloscópio — P0 — [DIAGNOSTICAR]
- **Relato:** "Chassi: Nenhum controle aqui funciona direito. Mudança de válvula não produz efeito, osciloscópio e bias não funcionam. Circuito exibe mensagem de erro." **Evidências:** 24, 25.
- **O que se vê:** o osciloscópio CRT está vazio na captura 24 (sem traço); o sinal de teste está em −15 dBFS; o painel mostra bias −43,9 V para a V6 com EL84 selecionada num JCM800 (que usa EL34 no original). O erro do Circuito é T-06.
- **Protocolo:** (1) o scope recebe dados (Spec 047: transferência, forma de onda, tone stack)? mostrar mensagem quando não houver; (2) trocar a válvula de potência muda corrente/tensão de placa e a saída (Spec 021/047); (3) mover o bias muda corrente de placa, dissipação e distorção no sentido do equipamento real; (4) tensão de placa, assimetria, NFB e saturação do trafo têm efeito medido.
- **Teste:** um teste de efeito por controle da bancada; o scope reprova se ficar sem traço com sinal presente.

### T-16 · Sala de microfonação — P0 — [DIAGNOSTICAR]
- **Relato:** "os controles não funcionam, não é possível clicar e arrastar os microfones e controle de distância com a roda do mouse não funciona." **Evidência:** 26.
- **A investigar:** os handlers de arraste/roda em `Gabinete.ts` (a legenda promete "arraste o microfone sobre o cone · roda do mouse = distância · duplo clique = 0°/45°"); se o evento chega (`pointerdown`/`wheel`, `touch-action`, `preventDefault` em listener passivo) e se `cab_1.mic1.*` / `mic2.*` chegam ao DSP.
- **Atenção a valores:** nível do MIC 1 em −48,5 dB e do MIC 2 em −30,5 dB são padrões estranhos; MIC 1 em −48,5 dB praticamente cala o microfone. Descubra de onde vêm e confirme se são padrões ou estado carregado.
- **Teste:** arrastar o MIC 1 muda `position`; roda muda `distance`; cada mudança altera a resposta resultante mostrada no gráfico e o áudio (Spec 038, 022).

### T-17 · Oversampling / Desempenho sem efeito; botão WAV DI morto — P1 — [CONFIRMADA] + [DIAGNOSTICAR]
- **Relato:** "Desempenho e Oversampling dsp não tem efeito algum." **Evidências:** 19, 20, 01.
- **[CONFIRMADA]** `Audio.ts` ~402–406: o botão "Abrir Arquivo WAV DI..." cria um `<input type="file">` e chama `click()` sem tratador de `change` e sem anexar ao DOM útil: não faz nada. O cartão aparece quase vazio (imagem 19).
- **[CONFIRMADA]** O padrão local do cartão é `'4x'` (`Audio.ts` ~396) e o topo exibe 8x (imagem 01): dois padrões diferentes para o mesmo controle.
- **[DIAGNOSTICAR]** Oversampling não muda o timbre de forma óbvia; o efeito esperado é menos aliasing, mais CPU e mais latência. Hoje a tela não mostra nenhum desses três. **Mostre** CPU e latência adicionada por fator e teste que a chave `engine.oversampling` chega ao DSP (aliasing medido cai de 2x para 8x; latência reportada sobe).
- **Teste:** `engine.oversampling` 2x/4x/8x muda a energia de aliasing medida e a latência reportada; o botão WAV DI abre o seletor e carrega o arquivo no motor (`input.source = di_file`).

### T-18 · TAP sem efeito claro — P1 — [DIAGNOSTICAR]
- **Relato:** "TAP não tem efeito algum." **Evidências:** 01 (120), 16 e 20 (40).
- **A investigar:** `store.ts` ~542–554 converte BPM em `rate` com `(240 − bpm)/200 × 10` — **mais BPM dá menos rate**, sem relação musical clara. Defina o que o TAP controla (tempo do delay? taxa da modulação?), mostre o BPM atual de forma consistente (120 no topo, 40 no Palco) e confirme que o valor chega ao DSP.
- **Teste:** 4 toques a 120 BPM definem o tempo do delay para 500 ms (ou o que a spec decidir), mensurável na saída.

### T-19 · Pedalboard fixo — P1 — [CONFIRMADA]
- **Relato:** "faltam os outros pedais. O pedalboard ficou com muito espaço vazio… Na modulação, apenas o pedal chorus-2 é exibido no pedalboard e os demais não aparecem mesmo ao ser selecionados." **Evidências:** 01, 14, 15.
- **Causa:** `Estudio.ts` ~770–776 monta o pedalboard **à mão**: `ns2`, o modelo de drive, `ce2` e `mxr10`, com escala 1,0 e alinhamento embaixo. Escolher TZ Flanger, Phaser, Tremolo etc. altera o modelo no backend, mas o pedal exibido continua sendo o CE-2.
- **Correção:** gerar o pedalboard a partir do **grid/estado** (blocos de pedal ativos, na ordem do sinal; linha B separada), com um pedal por modelo selecionado; escalar para ocupar a área; rolagem horizontal se não couber. Pedais ainda sem design aparecem como placa genérica rotulada, não somem (design real em T-28).
- **Teste:** para cada modelo de `mod_2`, `drive`, `eq`, `comp`, o pedal exibido corresponde ao modelo selecionado; régua visual da Spec 048 atualizada.

### T-20 · Racks pequenos demais — P1 — [CONFIRMADA]
- **Relato:** "os hacks ficaram muito pequenos, tornando difícil a leitura e controle dos botões." **Evidência:** 01.
- **Causa:** `Estudio.ts` ~790 define os racks com altura original 130 e slot 74 (escala ≈ 0,57).
- **Correção:** aumentar a área do rack (ou rolagem vertical) e/ou abrir o rack em tamanho de edição ao clicar. Defina na spec um **tamanho mínimo de alvo de clique** e teste.

### T-21 · Pedais e racks deformados na Biblioteca — P1 — [DIAGNOSTICAR]
- **Relato:** "Alguns pedais estão deformados, como o fet 76, space tape e studio plate reverb." **Evidências:** 23 (e o Tape Echo cortado à direita na 01).
- **Hipótese:** o `wrapScaledPedal`/escala usa largura e altura originais fixas que não batem com o tamanho real do componente de rack. Meça a caixa real do componente e derive o slot dela.
- **Teste:** régua visual (SSIM, Spec 048) de cada rack e pedal da Biblioteca contra a referência.

### T-22 · Amps da Biblioteca não se parecem com o rig — P2 — [DIAGNOSTICAR]
- **Relato:** "Em ajuda > inventário de controles os amplificadores não se parecem com exibido no rig." **Evidências:** 21, 22, 01.
- **Observação:** ambos usam `createAmpHeadElement`; a diferença deve estar em opções/escala. Compare e unifique, ou documente a diferença; teste visual entre as duas telas para o mesmo modelo.

### T-23 · Afinador × Palco × Config. de áudio × Status do motor — P1 — [CONFIRMADA] (roteamento)
- **Relato:** "afinador e palco abrem exatamente a mesma tela e o último botão do lado direito abre as configurações de áudio… Ajuda > status do motor áudio abre exatamente a mesma tela com as configurações de áudio." **Evidências:** 20, 21, 16.
- **Causa:** os menus apontam para a tela `palco` (`shell.ts` ~171 e ~192) e a tela Palco já contém o afinador e o looper. Confirme o handler dos botões AFINADOR/PALCO do topo (`Estudio.ts` ~521 e ~530), que não foi lido. "Status do Motor Áudio" abre a tela de configurações.
- **Decisão pendente do usuário** (se o agente não puder perguntar, implemente a recomendação e registre): **AFINADOR** = tela/overlay dedicada ao afinador (com mute); **PALCO** = visão de palco (nome do preset, cenas, footswitches, EXP) sem repetir o afinador; **⚙** = configurações de áudio; **Ajuda › Status do motor** = painel **somente leitura** (dispositivos de entrada e saída, taxa, período, latência medida × estimada, xruns, CPU, oversampling, estado das filas).
- **Teste:** cada entrada abre uma tela diferente; o status do motor mostra dados reais da telemetria (Spec 036: telemetria honesta).

### T-24 · Clareza dos textos — P2 — [CONFIRMADA]
- **Relato:** "Para quê serve o canal a, b, c, d? a frase logo na frente não aparece por completo… não ficou muito claro a funcionalidade [das cenas]."
- **Causa:** o texto "cada canal guarda um…" é cortado por `text-overflow: ellipsis` (imagens 04, 06, 10, 12, 15).
- **Correção:** mostrar o texto inteiro (quebra de linha ou dica) e explicar em uma linha o que são **Canal A–D** (conjunto de ajustes guardado por bloco) e **Cena A–H** (estado dos blocos e canais escolhidos, trocado sem buraco). Confirme a redação nas Specs 009, 025 e 045.
- **Jargão interno na interface:** remover "Spec NNN" dos textos do usuário (`shell.ts` ~199 "Inventário de controles (Spec 033)", `Audio.ts` ~119–120, `Estudio.ts` ~923, dicas "em implementação — Spec 0NN", `specRef` na Biblioteca). Cuidado: testes que leem os fontes podem depender desses textos.
- **Título do Chassi:** "JCM800_2203 — CHASSI" mostra o id interno; usar o nome de exibição ("BRIT 800 — CHASSI").

### T-25 · Latência alta e xruns — P0 — [DIAGNOSTICAR]
- **Relato:** "ainda existe uma considerável latência entre o tocar de uma nota e a saída do som." **Evidência:** 01 (barra de status).
- **O que a barra mostra:** `WASAPI low-latency · Smart M50C (Áudio Intel(R) para telas) · 48 kHz · período 128/128 · ida-e-volta 5,7 ms estimada · xruns 12`, CPU 23,2 % com oversampling 8x.
- **Pontos a verificar (nenhum provado):**
  1. **"Estimada" não é medida.** 5,7 ms parece derivar do buffer, sem a latência do driver/codec nem do dispositivo de saída. Meça por loopback (contrato de medição das Specs 018/036) e mostre **medida × estimada**.
  2. **O dispositivo ativo parece ser áudio de monitor** ("Áudio Intel(R) para telas", HDMI/DisplayPort), que costuma ter latência própria e relógio diferente do microfone onboard. **Pergunte ao usuário** qual é a saída em uso. A Spec 050 alinhou taxas, não resolveu latência de dispositivos distintos. Mostre entrada e saída separadas.
  3. **xruns = 12** com período 128/128 indica *underruns* acumulados. Descubra a origem (CPU, fila de 1 período, mudança de modo) e reporte por sessão.
  4. **Oversampling 8x acrescenta atraso** (≈65 amostras segundo o handoff 050, R8) e usa mais CPU. Reavalie o padrão para uso ao vivo (T-17).
- **Aceite:** valor medido registrado (ou `[medida pendente]` sem hardware); nenhuma promessa de latência sem medição.

### T-26 · Rótulos sobrepostos — P2 — [CONFIRMADA] (visual)
- **Evidências:** 03 ("THRESHOLD" colado em "DECAY"), 05 ("DISTORTION" colado em "FILTER"), 26 (textos "MIC 1: … · MIC 2: …" um sobre o outro abaixo do cone).
- **Correção:** espaçamento/abreviação nos knobs do pedal e quebra da legenda dos mics. Verifique contra a referência da Spec 048.

---

## 6. Itens — Spec 052 (melhorias, com referência visual antes do código)

### T-27 · Abertura animada com a logomarca — P2
- **Relato:** "Seria interessante exibir a logomarca Sonix-Pro // AmpForge como uma abertura animada enquanto a aplicação é carregada."
- **Requisitos propostos:** aparece antes do primeiro quadro do shell e some quando o `bootstrap()` termina (duração = carregamento real, com teto); respeita `prefers-reduced-motion`; funciona offline e embutida (sem `localhost`, Spec 050); não toca o callback de áudio. Pergunte ao usuário qual arquivo de logomarca usar (há ícones em `apps/sonix-app/src-tauri/icons/`).
- **CA exemplo:** com o IPC atrasado artificialmente, a abertura permanece; com `reduced-motion`, não anima.

### T-28 · Pedais sem design — P2
- **Relato:** "Pedais como flanger, wah e outros sequer tiveram o design feito."
- **Escopo mínimo (do que a UI já oferece):** modulação — Tri-Chorus, TZ Flanger, Phaser 90, Tremolo, Uni-Vibe, Rotary; pitch — Whammy, Drop Capo, Harmonizer, Detune; **Wah** (hoje não existe bloco). **Confirme a lista completa** varrendo os seletores de modelo de cada bloco contra a Biblioteca (hoje 12 pedais/racks).
- **Regra:** cada novo pedal precisa de design de referência renderizado **antes** do código (Spec 048), registro de controles com efeito real e entrada na Biblioteca. O wah exige DSP e spec próprios (fonte do circuito).

### T-29 · AMP 2 visível no Estúdio — P2
- **Relato:** "O amp paralelo poderia aparecer na tela mesmo que em um espaço menor."
- **Proposta:** mini-cabeça do AMP 2 (escala reduzida, com os knobs principais) quando o split estiver ativo. Altera o layout aprovado: atualize `docs/ui-desktop/canvas/` e a referência antes do código.

---

## 7. Ordem sugerida e gates
1. **T-00** (régua) → red nos quatro casos conhecidos.
2. **T-01, T-02, T-03, T-04, T-05, T-06, T-07** (contratos; cada um vira green na régua).
3. **T-25 (medição)** em paralelo: reunir dados reais antes de decidir mudanças de latência.
4. Recalibrar a entrada e **re-testar T-08 e T-10** com sinal calibrado; só então mexer no DSP.
5. **T-11, T-12, T-13, T-14, T-15, T-16** (controles sem efeito, com protocolo de medição).
6. **T-09, T-17, T-18, T-23** e depois **T-19, T-20, T-21, T-22, T-24, T-26** (UI).
7. **Spec 052** (T-27 a T-29) por último.

Gate entre etapas: `cargo test --workspace` e `npm test` verdes; build release `sonix-app --features custom-protocol` verificado.

## 8. Fechamento
- Atualize Notas e "Testes que cobrem esta spec" da 051 (e 052), `specs/README.md` e o "Estado atual" do `AGENTS.md` com os resultados (`N passed; 0 failed`).
- **Relatório por etapa:** item → CA → teste (red mostrado) → arquivo alterado → resultado → "antes/depois" medido quando aplicável.
- Capture no Cortex as decisões e armadilhas (por exemplo: "payload UI deve ser testado pelo caminho real, não só pelo registro").
- Não commite sem autorização.

## 9. Mapa de evidências

| Imagem | Conteúdo | Itens |
|---|---|---|
| 01 | Rig principal e barra de status | T-07, T-08, T-17, T-19, T-20, T-25 |
| 02 | Chips de Modifier desabilitados | T-09 |
| 03–06 | Pedal/painel do gate e dos drives | T-10, T-24, T-26 |
| 07 | Erro `input_jack` | T-02, T-04 |
| 08 | Erro `preset_not_found` | T-02, T-05 |
| 09 | Botões de cena | T-03, T-24 |
| 10–11 | AMP 2 e Merge −28 dB | T-11, T-29 |
| 12–13 | Gabinete 1 | T-12 |
| 14–15 | Pedalboard só com Chorus-2; Mix 3300 % | T-07, T-19 |
| 16 | Palco: afinador e looper | T-13, T-18, T-23 |
| 17 | Footswitches e EXP | T-14 |
| 18 | Calibração com erro | T-01, T-02 |
| 19 | Desempenho / Oversampling | T-17 |
| 20 | Topo: TAP, Afinador, Palco | T-18, T-23 |
| 21–22 | Menu Ajuda e inventário de amps | T-22, T-23 |
| 23 | Pedais/racks deformados | T-21, T-28 |
| 24–25 | Chassi e Circuito | T-06, T-15 |
| 26 | Sala de microfonação | T-16, T-26 |
