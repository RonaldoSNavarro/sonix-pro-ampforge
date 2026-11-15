---
spec: 045
titulo: Trocar de cena não muda o som, a troca de preset é mute + fade-in e salvar perde cenas e canais
status: implementada
atualizado: 2026-11-14
---
# 045 — Cenas, presets e grafo: troca audível, sem buraco e com estado completo

## Objetivo
Trocar de cena ou preset muda o som na hora, sem silêncio, e salvar/abrir um preset preserva cenas e canais.

## Contexto
Review manual das Specs 009, 010 e 025 (2026-11-14), por leitura de `crates/sonix-desktop/src/api.rs` e `audio.rs`:

| # | Achado | Evidência |
|---|---|---|
| 1 | **Cena sem efeito.** `/api/scenes/select` só grava `st.scene_active` e mede o próprio tempo de execução como `transition_ms`. Nada muda no motor. | `api.rs:2971-3002` |
| 2 | **Troca de preset com buraco.** Não há crossfade: a cadeia faz uma rampa a partir de **0** (`ramp = 0,5·(1−cos(π·fade))`), ou seja, silencia e volta. Além disso, a carga segura o lock da cadeia (`api.rs:3141`), e o callback emite silêncio enquanto isso. | `audio.rs:2684-2692` |
| 3 | **Salvar perde estado.** `presets/save` grava `scenes: vec![]`, `channels: None` e o grid vazio. Cenas e canais A–D se perdem ao salvar. | `api.rs:2874-2896` |
| 4 | **Data fixa.** `created_at` é a literal `"2026-11-11T00:00:00Z"`. | `api.rs:2883` |
| 5 | **Grafo fixo.** `/api/graph` devolve nomes fixos ("Brit 800 2203", "Screamer 808", "4x12 Greenback"…) e `enabled: true` fixo para gate, drive, amp e cab, independente do estado. | `api.rs:684-710` |
| 6 | **Carga não atômica.** `apply_preset_params` aplica as chaves uma a uma; não há validação prévia do documento inteiro. | `api.rs:3005-3140` |

## Comportamento esperado
- Uma cena define, por bloco, o canal ativo (A–D) e o bypass. Selecionar a cena aplica essas diferenças no motor dentro de um bloco de áudio.
- A troca de cena ou de preset faz **crossfade entre o estado anterior e o novo**, com as duas cadeias ativas durante a transição (≤ 20 ms), ou troca só parâmetros suavizados quando a topologia é a mesma. Nunca há silêncio.
- Salvar grava todos os parâmetros, as 8 cenas, os canais A–D por bloco e o grid. Abrir restaura tudo.
- `created_at` é o instante real do salvamento.
- `/api/graph` reflete os modelos e o bypass atuais.
- A carga de preset valida o documento inteiro antes de aplicar qualquer coisa.

## Critérios de aceite
- **CA-1:** Num preset em que a cena B tem o drive em bypass e a cena A não, selecionar B durante uma senoide muda o THD da saída (cai em pelo menos 50%) em até 1 bloco + 20 ms.
- **CA-2:** Trocar de cena ou de preset durante uma senoide estável não produz nenhuma janela de 1 ms com RMS < 50% do menor RMS entre os estados antes e depois (sem buraco).
- **CA-3:** `save → load` de um preset com cenas e canais personalizados devolve um `GET /api/state` idêntico, incluindo cenas, canais por bloco e cena ativa.
- **CA-4:** `created_at` de um preset salvo está a menos de 5 s do relógio do sistema no momento do salvamento.
- **CA-5:** Depois de `amp_1.model = "ac30_tb"` e `drive.enabled = false`, `/api/graph` mostra o nome do AC30 no bloco `amp_1` e `enabled: false` no `drive`.
- **CA-6 (erro):** Um preset com uma chave inválida no meio do documento responde 400, e nenhum parâmetro do estado atual muda.
- **CA-7 (não-regressão):** A migração de schema 1.0.0 → 1.1.0 (Spec 025) continua funcionando, e os presets de fábrica abrem.

## Fora de escopo
- O DAG 4×12 configurável pelo usuário (Spec 002): aqui a topologia continua a cadeia fixa atual.
- Modifiers (pedal de expressão, LFO) atrelados à cena.

## Invariantes tocadas
- **Zero-alloc / zero-lock no callback:** o crossfade usa uma segunda instância pré-alocada da cadeia, ou parâmetros suavizados. Nunca uma alocação na troca (Spec 037).
- **Persistência offline-first (Spec 010):** o formato continua sendo `.sonixpreset` 1.1.0; esta spec só passa a preencher campos que já existem no schema.

## Testes que cobrem esta spec
- `crates/sonix-desktop/tests/contract_045_scenes_presets_graph.rs` → CA-1 (`ca1_scene_switch_changes_output_thd`), CA-2 (`ca2_scene_and_preset_switch_have_no_silence_hole`), CA-3 (`ca3_save_load_preserves_scenes_channels_and_active_scene`), CA-4 (`ca4_saved_preset_created_at_is_within_5s_of_system_clock`), CA-5 (`ca5_graph_reflects_live_models_and_bypass`), CA-6 (`ca6_invalid_key_in_preset_returns_400_and_leaves_state_unchanged`), CA-7 (`ca7_schema_migration_and_factory_presets_still_work`)
- `crates/sonix-desktop/tests/contract_037_realtime_safety.rs` → zero-alloc/zero-dealloc no crossfade e troca de IR com cadeia shadow (`10/10` ok)
