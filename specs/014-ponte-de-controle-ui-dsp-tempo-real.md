---
spec: 014
titulo: Controles da interface não alteram o som — ponte de controle UI→DSP em tempo real
status: rascunho
atualizado: 2026-11-09
---
# 014 — Ponte de Controle UI→DSP em Tempo Real

## Objetivo
Fazer cada controle visível da interface que corresponde a um bloco da cadeia ao vivo (knobs do
amp, BRIGHT, FS1, cenas, bypass de bloco, pedal de expressão) chegar ao motor de áudio e alterar
o som em tempo real, sem estalos e sem comprometer a thread de áudio.

## Contexto
Origem: relato de campo de 2026-11-09 ("agora é possível ouvir a guitarra, com distorção, porém
nenhum controle funciona na interface") e code review de 2026-11-09 (achados F-01, F-04).
Evidência:

- `design-system/app.js:246-260` — `updateKnobValue` só atualiza `state`, o rotor CSS e o
  readout; **nenhuma** chamada de rede. A única requisição do frontend é
  `fetch('/api/telemetry')` (`app.js:912`). Footswitches (`app.js:485-490`), cenas
  (`app.js:522-560`), BRIGHT (`app.js:743`), expressão (`app.js:749-756`) e bypass por duplo
  clique no grid (`app.js:137-139`) idem.
- `crates/sonix-desktop/src/server.rs:87-110` — mutação por `GET /api/params?k=v`, com 5 chaves
  (`gain`, `master`, `gate`, `boost`, `cab`) e `v.parse::<f32>()` sem validação.
- `crates/sonix-desktop/src/audio.rs:249-253` — o callback usa `dsp_chain_out.try_lock()`;
  quando o servidor segura o `Mutex` (`server.rs:90` usa `lock()` bloqueante) o callback
  **emite o sinal de entrada sem processar** (`mono_out.copy_from_slice(&mono_in)`). Hoje é raro
  (ninguém chama `/api/params`); com a ponte ativa vira estalo/timbre seco a cada movimento de knob.
- `crates/sonix-desktop/src/audio.rs:463-485` — `RealtimeDspChain` só expõe 6 setters (gate
  threshold/enabled, boost, gain, master, cab enabled).
- Os parâmetros são aplicados em degrau (`amp.rs:243-269` relê o painel a cada bloco, sem
  suavização) → *zipper noise* ao arrastar knobs (Spec 001 CA-6).
- O que a UI tem e o que é ao vivo (`app.js:18-28, 43-68, 462-467`): 8 knobs do amp
  (`inputGain`, `overdrive`, `bass`, `middle`, `treble`, `presence`, `depth`, `masterVol`),
  BRIGHT, FS1 = `blk_drive`, FS2–FS4 = mod/delay/reverb, FS5–FS8 + barra de cenas A–H (cada cena
  tem `gain`, `od`, `driveBypass`, `dlyBypass`), botões de canal A–D do bloco selecionado
  (Spec 009, `BlockChannel`). A cadeia ao vivo é Gate → [Drive, Spec 016] → Amp → Cab
  (`audio.rs:408-421`): mod, delay e reverb não estão nela.
- Spec 013 (Notas) já registrava a lacuna; esta spec a fecha. A **audibilidade** de
  `bass/middle/treble/overdrive` e do pedal depende da Spec 016; aqui se garante que o valor
  **chega e é lido** pelo motor.

## Comportamento esperado
1. **Despachante no frontend.** Todo controle mapeado na tabela do §2 chama
   `syncParamsToBackend(patch, immediate)`. Arrastos contínuos (knobs, expressão) são coalescidos
   em janela de 25 ms (último valor vence); cliques discretos (FS1, cena, BRIGHT, bypass por duplo
   clique) são enviados imediatamente. Uma cena envia, em uma requisição, `amp_1.gain`, `amp_1.overdrive` e
   `drive.enabled` da cena.
2. **Contrato do endpoint — registro de chaves, tipos e faixas.** `POST /api/params`,
   `Content-Type: application/json`, corpo objeto JSON `{"<chave>": <valor>, ...}`. Esta tabela e as
   tabelas das specs posteriores (mesmo formato) formam o registro de parâmetros do projeto; a
   Spec 015 o protege e não o redefine.

   | Chave | Tipo | Unidade | Faixa | Valor no boot | Suavização | Origem na UI |
   |---|---|---|---|---|---|---|
   | `amp_1.gain` | número | posição 0–10 | 0–10 | 8,0 (`main.rs:43`) | `OnePoleSmoother` | knob `inputGain`, expressão, cena |
   | `amp_1.overdrive` | número | posição 0–10 | 0–10 | 5,0 | `OnePoleSmoother` | knob `overdrive`, cena |
   | `amp_1.bass` | número | posição 0–10 | 0–10 | 5,0 | `OnePoleSmoother` | knob |
   | `amp_1.middle` | número | posição 0–10 | 0–10 | 5,0 | `OnePoleSmoother` | knob |
   | `amp_1.treble` | número | posição 0–10 | 0–10 | 5,0 | `OnePoleSmoother` | knob |
   | `amp_1.presence` | número | posição 0–10 | 0–10 | 5,0 | `OnePoleSmoother` | knob |
   | `amp_1.depth` | número | posição 0–10 | 0–10 | 5,0 | `OnePoleSmoother` | knob |
   | `amp_1.master` | número | posição 0–10 | 0–10 | 7,0 (`main.rs:44`) | `OnePoleSmoother` | knob `masterVol` |
   | `input.boost` | número | ganho linear (×) | 0,1–8,0 | 2,5 (`main.rs:42`) | `OnePoleSmoother` | sem controle na UI (calibração) |
   | `gate.threshold_db` | número | dBFS | −90–0 | −65 (`main.rs:41`) | `OnePoleSmoother` | sem controle na UI (calibração) |
   | `amp_1.bright` | booleano | — | — | `false` | crossfade | BRIGHT |
   | `drive.enabled` | booleano | — | — | `false` | crossfade | FS1, duplo clique em `blk_drive`, cena (`!driveBypass`) |
   | `gate.enabled` | booleano | — | — | `true` | crossfade | duplo clique em `blk_gate` |
   | `amp_1.enabled` | booleano | — | — | `true` | crossfade | duplo clique em `blk_amp_1` |
   | `cab_1.enabled` | booleano | — | — | `true` (`main.rs:45`) | crossfade | duplo clique em `blk_cab_1` |

   "Posição 0–10" é a posição do knob, não uma grandeza física: a curva que a liga ao circuito
   (taper do potenciômetro, ganho em dB, frequência) é definida no bloco DSP (Spec 016) e nunca
   convertida linearmente em dB aqui. "Valor no boot" é o estado do motor ao iniciar
   (`AmpFrontPanel::default`, `amp.rs:48-62`, sobrescrito por `main.rs:41-45`); a UI o lê do
   motor (§7) em vez de impor os próprios defaults (`app.js:18-28`). `drive.enabled` nasce
   `false` para que a entrada do pedal na cadeia (Spec 016) não mude o som do boot.

   Chaves usam o formato `<bloco>.<parâmetro>`, em que `<bloco>` é o id do bloco no grid sem o
   prefixo `blk_` (`app.js:43-57`: `gate`, `drive`, `amp_1`, `cab_1`, …) e `input` é a entrada
   física. As chaves desta spec são número JSON ou booleano JSON. Specs posteriores acrescentam
   chaves com tabela no mesmo formato e podem usar o tipo **enum** (string de um conjunto fechado,
   ex.: modelo de amp); valem para elas as mesmas regras do §3 (string fora do conjunto → `400`).
   Nenhuma spec redefine uma chave existente.
3. **Validação atômica.** O corpo inteiro é validado antes de aplicar qualquer chave: se uma chave
   conhecida tem tipo errado, valor fora da faixa ou o corpo não é um objeto JSON válido, a
   resposta é `400 {"error":"..."}` e **nenhum** parâmetro muda. Chaves desconhecidas — inclusive
   os nomes antigos `boost`, `gate` e `cab` — não alteram estado e voltam em `ignored`. Sucesso:
   `200 {"status":"ok","applied":[...],"ignored":[...]}`. `Content-Type` diferente de
   `application/json` → `415`.
4. **Rota antiga removida.** `GET /api/params` deixa de mutar estado e responde `405`.
5. **Entrega sem bloqueio.** O servidor publica os valores por estrutura lock-free (átomos por
   parâmetro ou a `SpscCommandQueue` existente em `ampforge-dsp::queue`); a thread de áudio os lê
   no início do bloco. A thread de áudio **nunca** deixa de processar por contenção com o servidor.
6. **Suavização e bypass sem estalos.** Cada parâmetro numérico é aplicado através de
   `OnePoleSmoother` (τ entre 5 e 20 ms, Spec 001 CA-6). Cada booleano da tabela troca por
   crossfade linear de 10 ms entre o caminho anterior e o novo (estados dos dois caminhos já
   pré-alocados), nunca por troca no meio do bloco.
7. **Fonte da verdade.** O motor é a fonte; `GET /api/state` devolve todas as chaves da tabela com
   os valores correntes. Ao carregar, a UI posiciona knobs/readouts a partir dele. Se o servidor
   cair, a UI mantém o estado visual, exibe `#engineLinkBadge` = `OFFLINE` e reenvia o estado
   completo ao reconectar.

## Critérios de aceite
- **CA-1:** Arrastar qualquer um dos 8 knobs do painel produz **no máximo 1** requisição
  `POST /api/params` por janela de 25 ms, e a última requisição do arrasto carrega o valor igual
  ao exibido no readout (teste JS com `fetch` simulado e relógio falso).
- **CA-2:** Clicar em FS1, em um botão de cena, no BRIGHT ou dar duplo clique em `blk_gate`,
  `blk_drive`, `blk_amp_1` ou `blk_cab_1` dispara a requisição **sem espera de debounce**
  (≤ 1 tick), com a chave da tabela do §2 (`drive.enabled` / `amp_1.gain`+`amp_1.overdrive`+`drive.enabled` /
  `amp_1.bright` / `gate.enabled`, `drive.enabled`, `amp_1.enabled`, `cab_1.enabled`).
- **CA-3:** Mover o slider de expressão (`#expPedalSlider`) envia `amp_1.gain` com o valor mapeado em
  `3.0..9.5`, coalescido como no CA-1.
- **CA-4:** `POST /api/params` aplica **cada** chave da tabela do §2: após o POST,
  `GET /api/state` devolve o valor enviado (teste de contrato Rust sobre `LocalAppServer` +
  `RealtimeDspChain`, uma asserção por chave).
- **CA-5:** Chave desconhecida e os nomes antigos `boost`, `gate`, `cab` não alteram estado, são
  listados em `ignored` e a resposta é `200` (nunca `5xx`).
- **CA-6:** Com 1.000 atualizações/s de parâmetros em outra thread durante 2 s, **nenhum** bloco
  processado pelo `RealtimeDspChain` é idêntico ao bloco de entrada (sem bypass por contenção) —
  prova de que o fallback `copy_from_slice(&mono_in)` de `audio.rs:252` deixou de existir.
- **CA-7:** Com a thread do servidor segurando o estado compartilhado por 200 ms (contenção
  simulada), a thread de áudio continua produzindo saída processada em todos os blocos do intervalo.
- **CA-8:** Um degrau de `amp_1.gain` de `2.0` para `9.0` converge pelo suavizador: o ganho efetivo
  atinge ≥ 63 % da variação em ≤ 20 ms e < 63 % em < 5 ms (mesma régua da Spec 001 CA-6).
- **CA-9:** Bypass sem estalo — com seno de 1 kHz e pico 0,3 na entrada, alternar
  `cab_1.enabled`, `amp_1.enabled`, `drive.enabled`, `gate.enabled` e `amp_1.bright` (um por vez) resulta
  em `max |y[n] − y[n−1]|` durante a transição ≤ 1,1 × o maior valor de `max |y[n] − y[n−1]|`
  medido em regime nos dois estados; a transição dura 10 ms ± 1 bloco.
- **CA-10:** `GET /api/state` devolve JSON com todas as chaves da tabela do §2 e, no boot, os
  valores da coluna "Valor no boot"; ao carregar
  `index.html` com um estado diferente dos defaults, knobs e readouts refletem esse estado (teste
  JS com `fetch` simulado).
- **CA-11:** Caso de erro — com o servidor indisponível (fetch rejeita), a UI mantém os valores
  visuais, mostra `#engineLinkBadge` = `OFFLINE` e, ao voltar, reenvia o estado completo em uma
  única requisição.
- **CA-12:** Caso de erro — cada corpo a seguir retorna `400` e **nenhum** parâmetro muda
  (comparação de `GET /api/state` antes/depois): `{"input.boost":1e30}`, `{"input.boost":-1}`,
  `{"input.boost":0}`, `{"input.boost":9}`, `{"gate.threshold_db":-91}`,
  `{"gate.threshold_db":1}`, `{"amp_1.gain":-0.1}`, `{"amp_1.gain":10.1}`, `{"amp_1.gain":"9"}`, `{"amp_1.gain":null}`,
  `{"amp_1.gain":[9]}`, `{"amp_1.bright":1}`, `{"amp_1.gain":9,"amp_1.master":11}` (atômico: `amp_1.gain` também não muda),
  `[1,2]`, JSON truncado e `{"input.boost":NaN}` (não é JSON válido).
- **CA-13:** Caso de erro — `GET /api/params?gain=9` retorna `405`, e `POST` com
  `Content-Type: text/plain` retorna `415`; em ambos o estado não muda.
- **CA-14:** Não-regressão — sem interação do usuário nenhuma requisição `POST /api/params` é
  feita; clicar nos botões de canal A–D, em FS2–FS4 ou mudar `#oversamplingSelect` não faz
  requisição; as 8 CAs da Spec 013 e as 106 asserções das Specs 001–012 seguem verdes.

## Fora de escopo
- Tornar audíveis tone stack, overdrive e pedal de drive → **Spec 016** (aqui só se verifica que
  o valor chega ao motor).
- Proteção contra clientes hostis (Host/Origin/token, CORS, limite de corpo, concorrência) e
  defesa em profundidade contra `NaN` dentro do DSP → **Spec 015**.
- Canais de bloco A–D (Spec 009): a UI só guarda a letra em `block.ch`, sem parâmetros por canal;
  ligar canal de bloco ao motor exige estado por canal na UI → **Spec 025**.
- FS2–FS4 (mod, delay, reverb) e `dlyBypass` das cenas: os blocos não estão na cadeia ao vivo →
  **Spec 024**.
- Voicing `Norm`/`HighLead` do amp (`AmpChannelVoicing`), parâmetros do pedal de drive
  (`gain/tone/level`), `outputLevel` e oversampling: sem controle na UI ligado a eles hoje
  (pedal → **Spec 023**; oversampling → **Spec 020**).
- Correções do host de áudio (taxa, formatos, reconexão de dispositivo) → **Spec 018**;
  persistência de presets da UI → **Spec 025**.

## Invariantes tocadas
- Áudio em tempo real (Spec 001/013): nenhuma alocação nem trava na thread de áudio — a entrega
  por átomos/SPSC é o que permite cumpri-la (CA-6, CA-7).
- Loopback only (Spec 012/013): o endpoint continua restrito a `127.0.0.1`.
- Spec 003 CA-7 (clamp/NaN no painel): mantida; a validação do §3 rejeita antes de chegar ao DSP.

## Testes que cobrem esta spec
Planejados (red → green): `crates/sonix-desktop/tests/contract_014_ui_dsp_bridge.rs` (CA-4…CA-9,
CA-12…CA-14) e `tests/contract/014-ponte-de-controle-ui-dsp-tempo-real.test.js` (CA-1…CA-3,
CA-10, CA-11, CA-14).

## Notas
- Ordem de implementação: **018 → 014 → 015 → 016** (ordem completa no `specs/README.md`). A 014
  define e valida o contrato; a 015 o protege.
  Entre o merge da 014 e o da 015 o servidor ainda envia `Access-Control-Allow-Origin: *`; exigir
  `application/json` (CA-13) força preflight em `POST` de outra origem, mas a 015 deve entrar na
  mesma release.
- Esta versão substitui a anterior em três pontos, para eliminar contradições com a Spec 015:
  (a) a chave `channel` (`norm`/`lead`) foi removida — os botões A–D são canais de bloco com 4
  valores, não o voicing do amp, e nenhum controle da UI produz `norm`/`lead`; (b) os nomes
  `input.boost` e `gate.threshold_db` substituem `boost` e `gate`; (c) validação de tipo/faixa e
  `GET → 405` passaram para cá; `output_db`, `drive_gain/tone/level` e `delay_enabled` saíram da
  tabela por não terem controle na UI nem bloco ao vivo.
- Revisão de 2026-11-09 (antes de qualquer implementação): as chaves passaram ao formato
  `<bloco>.<parâmetro>` e o contrato admite enums, porque as Specs 020–025 acrescentam parâmetros
  de vários blocos (amp 2, cab 2, efeitos) e nomes planos colidiriam (skill do projeto §10.5:
  identificador estável).
- Origem: code review de 2026-11-09 (achados F-01 e F-04), registrado no Cortex
  (`simulador-de-efeitos`).
