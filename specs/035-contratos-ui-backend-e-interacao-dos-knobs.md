---
spec: 035
titulo: UI lê campos que o backend não envia e destrói o knob durante o arraste
status: implementada
atualizado: 2026-11-14
---
# 035 — Contratos UI↔backend (telemetria, osciloscópio, presets, dispositivos) e arraste dos knobs

## Objetivo
Fazer a UI mostrar os dados reais que o motor envia e permitir girar os controles com o mouse de forma contínua.

## Contexto
Origem: code review do commit `e1ffff3` (2026-11-14), verificado por leitura de código.

| Achado | Evidência |
|---|---|
| Arraste do knob quebra | No commit histórico `e1ffff3` (`main.ts:138`, hoje refatorado em `apps/sonix-app/ui/src/shell.ts:1-709 @ 573344b`): todo `store.emit()` chamava `renderActiveScreen` → `replaceChildren`. Isso acontecia a cada commit de knob e a cada frame de telemetria (~30–45 Hz). |
| Telemetria com nomes trocados | O backend envia `cpu_usage_pct` e `gate_gr_db` (`crates/sonix-desktop/src/audio.rs:2722 @ 573344b`); a UI lê `cpu_avg_pct` e `gain_reduction_db` (`ui/src/store.ts:72-73`). CPU fica em 5,2% e GR em 0,0 dB para sempre. |
| Osciloscópio com formato trocado | O backend envia `{transfer:[{x,y}]}`, `{waveform:{di,output}}` e `{tonestack:[{f,db}]}` (`api.rs:762,775,794`); a UI lê `transfer_x/transfer_y`, `waveform_in/out` e `tonestack_db` (`store.ts:87-92`). O modo `'wave'` da UI não existe no backend (`"waveform"`) e devolve 400. |
| Presets e dispositivos | `/api/presets` e `/api/devices` devolvem arrays puros; a UI espera `{presets}` e `{devices}` (`store.ts:279-289`). Os ids padrão da UI (plexi_jumped, cali_modern…) não existem no backend (brit800, usdeluxe, recto, ac30), então carregar dá 404. O load não devolve `state` e os knobs não se atualizam. |

Causa comum: os tipos TypeScript e as structs serde são mantidos à mão, em paralelo, sem teste que os confronte.

## Comportamento esperado
- Atualizações de telemetria, afinador e osciloscópio mexem só nos elementos que exibem esses dados. Elas não reconstroem a tela, e durante um arraste nenhum re-render remove o controle ativo.
- Existe uma **única fonte de verdade de schema** para cada payload IPC: fixtures JSON geradas a partir das structs Rust e validadas pelos tipos TypeScript.
- A UI usa exatamente os modos de osciloscópio que o backend aceita.
- A lista de presets e de dispositivos vem do backend. Carregar um preset devolve o estado completo, e a UI reflete esse estado.

## Critérios de aceite
- **CA-1:** Num teste DOM (harness da Spec 033), um arraste vertical de 120 px num knob de 0–10, com frames de telemetria a 45 Hz chegando durante o arraste, leva o valor de 5,0 até o limite na direção do arraste. O elemento que recebeu `pointerdown` é o mesmo nó do DOM no `pointerup`.
- **CA-2:** Um frame de telemetria do backend com `cpu_usage_pct: 37.5` e `gate_gr_db: -18.2` faz a barra de status mostrar `CPU 37.5%`, e o leitor do gate mostrar `GR -18.2 dB`.
- **CA-3:** Um payload `{transfer:[{x:-1,y:-0.5},{x:1,y:0.6}]}` é desenhado com esses pontos. A curva sintética local **não** é desenhada quando o backend envia dados.
- **CA-4:** Cada modo de osciloscópio que a UI oferece recebe 200 do backend; o teste percorre a lista de modos da UI contra `/api/scope`.
- **CA-5:** A lista de presets da UI é igual, em ids e ordem, à resposta de `list_presets`. Não há id de preset fixo no código da UI.
- **CA-6:** `load_preset("recto")` devolve `{status, loaded, state}`, e cada knob registrado mostra o valor de `state`.
- **CA-7:** A tela Áudio lista exatamente os dispositivos devolvidos pelo backend (lista vazia → mensagem "nenhum dispositivo encontrado").
- **CA-8 (contrato):** Um teste carrega as fixtures JSON geradas pelo Rust (`TelemetrySnapshot`, `AudioStatusTelemetry`, respostas de scope, presets, devices e load) e falha se um campo esperado pelos tipos TS estiver ausente, ou se um campo enviado não for consumido.
- **CA-9 (erro):** Se o backend responder 400 ao osciloscópio, a tela mostra o estado "sem dados do motor" e não desenha a curva sintética.
- **CA-10 (não-regressão):** O `ParamCoalescer` continua enviando no máximo 1 `set_params` por frame (Spec 033 CA-8).

## Fora de escopo
- Se os dados do osciloscópio são reais ou sintéticos no backend fica na Spec 047. Esta spec garante só que a UI desenha o que recebe.
- A fidelidade visual das telas ao design fica na Spec 048.

## Invariantes tocadas
- **Registro único de controles (regra do projeto):** sem mudança; os controles continuam vindo do `controls.registry.ts`.
- **Spec 033 CA-8 (coalescência):** preservada por CA-10.

## Testes que cobrem esta spec
- `crates/sonix-desktop/tests/contract_035_ui_backend_contracts.rs` (`CA-4`, `CA-5`, `CA-6`, `CA-7`, `CA-8` + gerador de fixtures em `tests/fixtures/ipc-schemas/*.json`)
- `tests/contract/035-036-host-ui-contracts.test.js` (`CA-1`, `CA-2`, `CA-3`, `CA-4`, `CA-5`, `CA-6`, `CA-7`, `CA-8`, `CA-9`, `CA-10`)
