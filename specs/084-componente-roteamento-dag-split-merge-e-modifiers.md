---
spec: 084
titulo: Componente Compartilhado de Roteamento DAG 4×12, Split/Merge, Modifiers, Smoothers e Filas Lock-Free
status: rascunho
tipo: componente-compartilhado
atualizado: 2026-11-19
dependencias: [053]
---
# 084 — Componente de Roteamento DAG 4×12, Split/Merge, Modifiers e Filas SPSC Lock-Free

## 1. Identificação e objetivo
- **Identificador estável (`module_id`):** `component.core.dag_routing_modifiers_rt`
- **Revisão (`model_revision`):** `v1-dag4x12-spsc-reaper-modifiers`
- **Categoria:** Infraestrutura compartilhada de roteamento, controle em tempo real e segurança de memória
- **Objetivo:** Especificar os componentes estruturais que conectam todos os módulos DSP no engine — **Matriz DAG 4×12** com validação acíclica e compilação off-RT, **SplitNode** (`BalanceAb`, `Crossover`, `DynamicSplit`), **MixerNode** (nível, pan, fase Ø), **OnePoleSmoother**, **Filas SPSC Lock-Free** (`SpscBuffer`, `SpscOwnedQueue`) + **RetiredReaper** (liberação de memória off-RT), **Matriz de Modifiers** (`EXP 1/2`, `LFO 1/2`, `ENV`, `MIDI CC`, `Scene`) e **Crossfade Gapless de Cenas/Presets (≤ 20 ms)**.

## 2. Escopo
- `GridMatrix4x12` + `DagGraph` (validação de ciclos `DagCycleError`, ordem topológica determinística, múltiplos taps de saída `MAIN_XLR_LR`, `AMP_OUT_P10_NO_CAB`, `FX_SEND`, `USB_REC_OUT`).
- `SplitNode`: `BalanceAb` (equal-power), `Crossover` (par complementar soma plana `H_lp + H_hp = 1.0`, `100–2000 Hz`) e `DynamicSplit` (envelope attack/release roteando sinais leves→A e fortes→B).
- `MixerNode`: soma de até 4 linhas com `level_db` (`-60..+12 dB`), `pan` (`-1..+1`) e `phase_invert` (`180°`).
- `OnePoleSmoother`: interpolação exponencial `5–20 ms` invariante à taxa eliminando zipper noise.
- `SpscBuffer<T, CAP>` e `SpscOwnedQueue<T, CAP>`: comunicação lock-free UI/Control ↔ RT com padding anti-false-sharing (`[u8; 56]`), invariantes `// SAFETY:` documentadas e coleta de recursos descartados na thread de controle (`RetiredReaper`).
- `ModifierMatrix` compilada (`CompiledModifierTable`, zero-alloc em RT) + banco de 4 canais (`A/B/C/D`, processando apenas o canal ativo) + 8 cenas gapless com crossfade equal-power de 20 ms (mantendo `Delay`, `Reverb` e `Looper` em instância única fora do crossfade).

## 3. Fora de escopo
- Serialização em disco `.sonixpreset` / `.sforgepkg` e verificação SHA-256 (camada de persistência `sonix-core`, Specs 010 e 046).
- Transporte IPC Tauri v2 (`apps/sonix-app/src-tauri`, Specs 033 e 035).

## 4. Equipamento ou componente de referência
- **Referência funcional:** Arquiteturas profissionais de roteamento de modeladores flagship (matriz 4×12 com caminhos paralelos, cenas gapless, 4 canais por bloco e matriz universal de modifiers — Specs 002 e 009).

## 5. Revisão e variantes
- **Revisão:** `v1-dag4x12-spsc-reaper-modifiers`.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | `crates/ampforge-dsp/src/{dag.rs, queue.rs, smoother.rs}` | Workspace v1.0 | `GridMatrix4x12`, `SplitNode`, `MixerNode`, `SpscBuffer`, `SpscOwnedQueue`, `OnePoleSmoother` | `crates/ampforge-dsp/src/` | 2026-11-19 | `[confirmado em documentação]` | Implementação atual lock-free SPSC e nós de roteamento |
| F2 | `crates/sonix-core/src/{preset.rs, modifiers.rs, footswitch.rs}` | Workspace v1.0 | Hierarquia Setlist/Preset/Scenes/Channels/Modifiers | `crates/sonix-core/src/` | 2026-11-19 | `[confirmado em documentação]` | Modelo de domínio de cenas, canais e modifiers |
| F3 | Specs 001, 002, 009, 024, 025, 037, 045, 049 | Contratos existentes | RT safety, DAG ao vivo, cenas gapless e modifiers | `specs/` | 2026-11-19 | `[confirmado em documentação]` | Invariantes já verificadas em produção |

## 7. Fundamentação algorítmica e de concorrência
- **Crossover complementar (`SplitMode::Crossover`):** `H_lp(z) + H_hp(z) = 1.0` garante reconstrução de magnitude perfeitamente plana (`0,0 dB`) quando os dois ramos são somados sem efeitos não lineares, evitando coloração espúria no split paralelo.
- **SPSC Lock-Free (`Acquire`/`Release`):** produtor modifica exclusivamente `head` (`Release` após escrita do slot); consumidor modifica exclusivamente `tail` (`Release` após leitura). Padding de 56 bytes separa `head` e `tail` em cache lines distintas (64 bytes), evitando *false sharing* entre cores de CPU `[confirmado em documentação: F1]`.
- **Reclamation Off-RT (`RetiredReaper`):** recursos de heap (`ImpulseResponse`, `CompiledDagTopology`, `CompiledModifierTable`, `Vec<f32>` de DI) nunca sofrem `drop()` na thread de áudio; são devolvidos por `SpscOwnedQueue` para a thread de controle fazer a liberação fora do caminho crítico.

## 8. Comportamento esperado
- **Detecção de ciclos no DAG:** tentar conectar um ciclo na matriz 4×12 retorna `Err(DagCycleError)` na compilação off-RT sem afetar o grafo ativo em execução.
- **Reconstrução plana no Split/Merge:** com `SplitMode::Crossover` e `Merge` neutro (`0 dB`, fase normal), `out_a + out_b` reconstrói a entrada com erro `<= 0,05 dB` de `20 Hz` a `20 kHz`.
- **Inversão de fase real no Merge:** ativar `merge.row_1_phase_invert = true` sobre dois ramos idênticos cancela o sinal (`< -60 dBFS`).
- **Modifiers em tempo real sem alocação:** movimentar `EXP 1`, `LFO 1/2`, `ENV` ou `MIDI CC` modula suavemente os parâmetros-alvo configurados dentro da faixa `[min, max]` com `0` alocações RT.
- **Troca de cena/preset gapless:** alternar cenas `A..H` ocorre com crossfade equal-power de `20 ms` sem buraco de silêncio, preservando caudas de Delay/Reverb e o loop ativo do Looper (Spec 045 / 049 CA-1).
- **Custo zero de canais inativos:** apenas 1 dos 4 canais (`A/B/C/D`) de um bloco é processado por bloco (`channel_process_counts` dos canais inativos não incrementa — Spec 049 CA-30).

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Split Ativo | `split.enabled` | boolean | bool | `false/true` | `false` | discreto | Habilita caminho paralelo Linha B | Ativa ramo B + merge |
| Modo Split | `split.mode` | enum | id | `ab, crossover, dynamic_split` | `ab` | discreto | Seleciona algoritmo de divisão | `SplitNode` |
| Freq. Crossover | `split.crossover_hz` | number | Hz | `100.0..=2000.0` | `650.0` | log | Corte entre Linha A (LP) e B (HP) | Filtro complementar 1ª/LR |
| Nível Linha B | `merge.row_1_level_db` | number | dB | `-60.0..=12.0` | `-6.0` | linear em dB | Ganho da Linha B na soma | `MixerNode` |
| Fase Linha B | `merge.row_1_phase_invert` | boolean | bool | `false/true` | `false` | discreto | Inverte polaridade 180° da Linha B | Multiplicação `-1.0` |
| Cena Ativa | `scene.active` | enum | cena | `A..H` | `A` | discreto | Troca cena gapless | Crossfade equal-power 20 ms |
| Pedal EXP 1 | `palco.exp_1` | number | % | `0.0..=100.0` | `62.0` | linear | Fonte `EXP_PEDAL_1` dos modifiers | `modifier_exp1` suavizado |

## 10. Contratos de integração
- Fornece a espinha dorsal lock-free para a `Spec 053`: `SpscBuffer`, `SpscOwnedQueue`, `RetiredReaper`, `OnePoleSmoother`, `SplitNode`, `MixerNode`, `CompiledDagTopology` e `CompiledModifierTable`.
- Na evolução da `Spec 053`, a execução do grafo compilado (`CompiledDagTopology`) e o `RetiredReaper` serão movidos de `crates/sonix-desktop/src/audio.rs` para `crates/ampforge-dsp/src/graph/`, desacoplando `sonix-plugin` de `sonix-desktop`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/{dag.rs, queue.rs, smoother.rs}`, `crates/sonix-core/src/{preset.rs, modifiers.rs, footswitch.rs}`, `crates/sonix-desktop/src/audio.rs` (linhas 2648–2675, `RetiredReaper`, `CompiledDagTopology`, `CompiledModifierTable`).
- **Dívidas registradas:**
  1. `dag.rs:288–290` (`MixerNode::configure_channel`) usa `Vec::resize` — garantir que seja chamado **exclusivamente** fora da RT (`prepare`/compilação) e usar array fixo `[MixerChannelConfig; 4]` no executor RT.
  2. `queue.rs:58–59, 84, 120, 238, 254` contém os únicos blocos `unsafe` do núcleo DSP: adicionar comentários `// SAFETY:` exaustivos provando invariante single-producer/single-consumer e validar com testes de estresse concorrente (e Miri quando disponível).
  3. `SplitMode::Crossover` atual é de 1ª ordem (`6 dB/oit`); avaliar opção Linkwitz-Riley 2ª/4ª ordem (com soma all-pass plana) para separação mais nítida entre linha de graves e agudos.
- **Estrutura alvo:** `crates/ampforge-dsp/src/graph/{dag.rs, split.rs, mixer.rs, modifiers.rs, rt_queues.rs}`.

## 12. Estratégia DSP
- Compilação topológica determinística off-RT em lista plana de operações (`CompiledGraph`), execução linear cache-friendly sem recursão nem alocação em RT, smoothers `f64` (5–20 ms), crossfade equal-power `cos/sin` em troca de cena/preset e compensação PDC automática em `MixerNode`.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap em `process()`, avaliação de modifiers, troca de cena e drenagem de filas SPSC; zero locks; zero panic; filas cheias aplicam política explícita (nunca bloqueiam a thread RT).

## 14. Gestão de estado
- `active_topology`, `active_modifiers`, `shadow` pré-alocada para crossfade, `modifier_smoothed`, `lfo1/2_phase`, `env_follower` e estados de `SplitNode` encapsulados no executor do grafo.

## 15. Latência
- **`0` amostras** intrínsecas nos nós de roteamento/split/merge; `MixerNode` introduz atraso PDC apenas no ramo mais rápido para alinhar ramos paralelos de latências diferentes (Spec 053 CA-5).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 1,5%`** para Split + Merge + avaliação da tabela de 16 Modifiers + filas SPSC @ 128 frames / 48 kHz em build `--release`.

## 17. Testes
- Detecção de ciclo no DAG (`DagCycleError`); reconstrução plana do `SplitMode::Crossover` (`±0,05 dB`); cancelamento `phase_invert` (`< -60 dBFS`); teste concorrente multi-thread de `SpscBuffer` e `SpscOwnedQueue` (1 milhão de itens sem perda/corrupção); verificação de zero-dealloc com `RetiredReaper`; troca de cena sem buraco nem degrau; verificação de `0` chamadas em canais inativos; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (DAG Acíclico e Determinístico):** Grafos com ciclo são rejeitados com `Err(DagCycleError)` sem alterar a topologia ativa; grafos válidos executam em ordem topológica determinística.
- **CA-2 (Split/Merge Fiel):** `Crossover` reconstrói `out_a + out_b` dentro de `±0,05 dB` de `20 Hz` a `20 kHz`; `phase_invert` cancela sinais idênticos para `< -60 dBFS`.
- **CA-3 (Filas SPSC Lock-Free e RetiredReaper):** Transferência concorrente de parâmetros e recursos pesados entre 2 threads ocorre sem perda, sem corrupção e com `alloc_count == 0` e `dealloc_count == 0` na thread RT.
- **CA-4 (Cenas Gapless e Canais Econômicos):** Trocar cena `A → B` realiza crossfade de `20 ms` sem queda a zero, preserva caudas de Delay/Reverb/Looper e processa estritamente `1` canal ativo por bloco.
- **CA-5 (Modifiers Suaves e Zero-Alloc):** Variação em degrau de `EXP 1` (`0 → 100%`) produz transição suavizada sem zipper noise nem alocação em RT.
- **CA-6 (Não-regressão):** Testes das Specs 001, 002, 009, 024, 025, 037, 045 e 049 (`contract_049_live_dag.rs`, `contract_049_scene_state_and_queues.rs`, `contract_049_setlists_modifiers_footswitch.rs`) continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Dependentes:** Todo o grafo executável do engine (Specs `054–083`).
- **Relacionadas / Legadas:** `Specs 001, 002, 009, 024, 025, 037, 045, 049`.

## 20. Limitações conhecidas
- Limite de 16 bindings simultâneos de modifier na tabela compilada `CompiledModifierTable` (suficiente para rigs de palco; expansível em `prepare()` se necessário).

## 21. Riscos técnicos
- Qualquer erro de `Ordering` nos blocos `unsafe` de `SpscBuffer`/`SpscOwnedQueue` pode gerar UB sutil em arquiteturas fracamente ordenadas (ARM64) — manter invariantes `// SAFETY:` auditadas e encapsuladas.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `001`, `002`, `009`, `024`, `025`, `037`, `045`, `049`.
- **Código atual:** `crates/ampforge-dsp/src/{dag.rs, queue.rs, smoother.rs}`, `crates/sonix-core/src/{preset.rs, modifiers.rs, footswitch.rs}`, `crates/sonix-desktop/src/audio.rs`.
- **Testes existentes / futuros:** `contract_002_matriz_dag.rs`, `contract_037_realtime_safety.rs`, `contract_045_scenes_presets_graph.rs`, `contract_049_live_dag.rs`, futuro `contract_084_dag_routing_modifiers.rs`.
