# Specs — Spec Driven Development (SDD)

Cada mudança de comportamento deste projeto nasce de uma **spec leve** aqui, **antes** do código.
A spec é a fonte dos **critérios de aceite** (`CA-n`), que orientam a implementação e a verificação.

Este arquivo é, além das regras, o **índice** — e o índice é o changelog do projeto.

## Fluxo

1. **Spec** — crie `specs/NNN-nome-da-feature.md` a partir do `_template.md`, com objetivo,
   contexto (com evidência), comportamento esperado e **critérios de aceite numerados**.
2. **Índice** — registre a spec na tabela abaixo. Spec fora do índice é spec que ninguém acha.
3. **Código** — implemente o comportamento descrito. Comportamento não previsto que aparecer
   durante a implementação volta para a spec, antes do merge.
4. **Verificação** — cada `CA-n` é verificado antes do merge: teste automatizado quando o critério
   toca invariante ou contrato público; verificação manual **documentada no PR** quando não valer
   o custo do teste.
5. **Fechamento** — status → `implementada`, seção *Testes que cobrem esta spec* preenchida,
   índice atualizado.

## Regras

- **Sem spec, sem código.** Mudança de comportamento começa por editar ou criar a spec.
- Critérios de aceite são **verificáveis** — afirmações que alguém pode reprovar, não desejos.
- CAs que tocam **invariantes** do projeto exigem teste automatizado; os demais podem ser
  verificados manualmente, desde que o PR registre como.
- Specs são versionadas com o código e revisadas **no mesmo PR** da implementação.
- Numeração é sequencial e **nunca reutilizada** — nem quando a spec é descartada.
- Toda spec tem *Fora de escopo* preenchido.

> **Regime de teste: SDD + TDD estrito.** Cada `CA-n` vira um teste que **falha antes** da
> implementação (red → green). Quando o projeto relaxar esse ciclo, registre aqui em qual spec
> a transição aconteceu — senão o histórico fica incoerente para quem lê de trás para frente.

## Convenção de nomes

- Spec: `specs/001-captura-de-evento.md`
- Teste de contrato: arquivo que espelha o número da spec (`.../contract/001-captura-de-evento.*`).
- Cada caso cita o critério no nome: `ca4_<comportamento>` ou `CA-4: <afirmação>`.
- Deve existir um jeito de rodar **só** os testes de uma spec (marcador, filtro ou diretório).
- O cabeçalho do arquivo de teste lista, em uma linha cada, todos os CAs que ele cobre.
- Um CA pode ter mais de um teste (os dois sentidos de uma precedência, por exemplo); todos citam
  o mesmo número.

## Status

`rascunho` → `aprovada` → `implementada`. Use estados honestos quando for o caso:
`implementada (CA-N revertida)`, `descartada` (com motivo, arquivo mantido, número não reaproveitado).

## Índice de specs

| Spec | Título | Status | Testes |
|---|---|---|---|
| [001](001-nucleo-dsp-tempo-real-ipc-lock-free.md) | Isolamento em tempo real do motor DSP, comunicação lock-free e orçamento de CPU | Implementada | `crates/ampforge-dsp/tests/contract_001_nucleo_dsp.rs` (11 CAs ok) |
| [002](002-matriz-roteamento-sinal-dag-4x12.md) | Matriz de roteamento de sinal 4x12 (DAG), nós de Split/Mixer e múltiplos I/Os | Implementada | `crates/ampforge-dsp/tests/contract_002_matriz_dag.rs` (10 CAs ok) |
| [003](003-modelagem-amplificadores-white-box-painel-frontal.md) | Modelagem White-Box de amplificadores valvulados e painel frontal autêntico | Implementada | `crates/ampforge-dsp/tests/contract_003_amp_modeling.rs` (8 CAs ok) |
| [004](004-bancada-chassi-aberto-mod-shop-osciloscopio-crt.md) | Bancada Chassi Aberto (Mod Shop) e osciloscópio CRT de transferência em tempo real | Implementada | `crates/ampforge-dsp/tests/contract_004_modshop_crt.rs` (9 CAs ok) |
| [005](005-gabinete-sala-microfonacao-ir-acoplamento-impedancia.md) | Bloco de gabinete (Cab), sala de microfonação dupla, loader de IR e acoplamento de impedância | Implementada | `crates/ampforge-dsp/tests/contract_005_virtual_cab.rs` (8 CAs ok) |
| [006](006-blocos-dinamica-drive-equalizacao.md) | Blocos de dinâmica (Noise Gate, Compressor), Overdrive/Fuzz White-Box e Equalizadores (RTA) | Implementada | `crates/ampforge-dsp/tests/contract_006_dynamics_drive_eq.rs` (8 CAs ok) |
| [007](007-blocos-modulacao-pitch-delay-reverb.md) | Blocos de modulação, pitch polifônico, delay com ducking/spillover e reverb | Implementada | `crates/ampforge-dsp/tests/contract_007_mod_pitch_delay_reverb.rs` (9 CAs ok) |
| [008](008-afinador-estroboscopico-polifonico-looper.md) | Afinador estroboscópico/polifônico de alta precisão e Looper de performance | Implementada | `crates/ampforge-dsp/tests/contract_008_tuner_looper.rs` (9 CAs ok) |
| [009](009-cenas-gapless-canais-modifiers-footswitches.md) | Hierarquia Setlist/Preset, 8 Scenes gapless, 4 canais por bloco, Modifiers e modos de footswitch | Implementada | `crates/sonix-core/tests/contract_009_scenes_modifiers.rs` (9 CAs ok) |
| [010](010-persistencia-offline-schema-sonixpreset-pacotes-sforgepkg.md) | Persistência 100% offline-first, validação de schema .sonixpreset e pacotes .sforgepkg | Implementada | `crates/sonix-core/tests/contract_010_persistence_offline.rs` (8 CAs ok) |
| [011](011-design-system-skeuomorfico-componentes-3d-hardware-shell.md) | Design System skeuomórfico, tokens de materiais, controles táteis 3D e modo Hardware Shell | Implementada | `tests/contract/011-design-system-skeuomorfico-componentes-3d-hardware-shell.test.js` (9 CAs ok) |
| [012](012-host-desktop-standalone-tauri-plugins-vst3-clap.md) | Arquitetura Desktop nativa (Tauri v2 + Rust) e plugins de DAW (VST3 e CLAP) | Implementada | `crates/sonix-desktop/tests/contract_012_desktop_plugin.rs` (8 CAs ok) |
| [013](013-execucao-desktop-windows-captura-fisica-p2-duplo-clique.md) | Execução desktop nativa no Windows com captura física P2/P3 e inicialização por duplo clique | Implementada | `crates/sonix-desktop/tests/contract_013_desktop_realtime_audio.rs` (8 CAs ok) |
| [014](014-ponte-de-controle-ui-dsp-tempo-real.md) | Ponte de controle UI→DSP em tempo real (tabela única de parâmetros, validação atômica, SPSC lock-free, bypass sem estalo) — implementar 2ª | Rascunho | Planejado: `crates/sonix-desktop/tests/contract_014_ui_dsp_bridge.rs` |
| [015](015-endurecimento-servidor-local-validacao-csrf.md) | Endurecimento do servidor local (token CSRF, Host/Origin, sem CORS, limite de corpo, defesa NaN/Inf no DSP) — implementar 3ª, mesma release da 014 | Rascunho | Planejado: `crates/sonix-desktop/tests/contract_015_server_hardening.rs` |
| [016](016-cadeia-dsp-guitarra-tonestack-drive-gabinete-saida-segura.md) | Cadeia DSP de guitarra funcional (tone stack ativo, DriveProcessor, ganho 1x cab, saída segura 0 dBFS) | Rascunho | Planejado: `crates/ampforge-dsp/tests/contract_016_guitar_chain.rs` |
| [017](017-resiliencia-carregador-ir-e-estado-looper.md) | Resiliência do carregador de IR (pânico com channels/sample_rate = 0) e undo/redo fiel do looper (camadas perdidas) | Rascunho | Planejado: `crates/ampforge-dsp/tests/contract_017_resilience_ir_looper.rs` |
| [018](018-latencia-minima-host-audio-placa-onboard.md) | Latência mínima em placa onboard (WASAPI exclusivo/baixa latência/RAW, fila de 1 período, deriva, sem lock/alocação, medição por loopback) — implementar 1ª | Rascunho | Planejado: `crates/sonix-desktop/tests/contract_018_low_latency_host.rs` |
| [019](019-entrada-guitarra-placa-onboard-calibracao-sinal-teste.md) | Entrada de guitarra na placa onboard (calibração de nível e gate, clipping do conversor, sinal de teste pelo motor) | Rascunho | Planejado: `crates/sonix-desktop/tests/contract_019_input_calibration.rs` |
| [020](020-amplificadores-por-modelo-painel-fiel.md) | Amplificadores por modelo (5 modelos, painel com os controles reais, tone stack pelo circuito citado, oversampling selecionável) | Rascunho | Planejado: `crates/ampforge-dsp/tests/contract_020_amp_models.rs` |
| [021](021-bancada-chassi-aberto-osciloscopio-ao-vivo.md) | Bancada do chassi aberto e osciloscópio ao vivo (válvulas, bias, sag, NFB, transformador, retificador, tone stack) | Rascunho | Planejado: `crates/ampforge-dsp/tests/contract_021_workbench_live.rs` |
| [022](022-gabinete-microfones-ir-ao-vivo.md) | Gabinete, dois microfones e IR ao vivo (acoplamento de impedância, proximidade, filtro pente, convolução sem latência adicional) | Rascunho | Planejado: `crates/ampforge-dsp/tests/contract_022_cab_mic_ir.rs` |
| [023](023-pedal-drive-808-editor-de-bloco.md) | Pedal de drive 808 fiel ao circuito e editor do bloco selecionado | Rascunho | Planejado: `crates/ampforge-dsp/tests/contract_023_drive_808.rs` |
| [024](024-efeitos-e-caminho-paralelo-ao-vivo.md) | Efeitos e caminho paralelo do grid ao vivo (compressor FET, EQ, chorus, eco de fita, placa, split/merge, amp 2/cab 2, FS2–FS4) | Rascunho | Planejado: `crates/ampforge-dsp/tests/contract_024_live_graph_fx.rs` |
| [025](025-presets-cenas-canais-de-bloco-reais.md) | Presets, cenas e canais de bloco reais (estado completo, schema 1.1.0, import/export pelo motor, troca sem buraco) | Rascunho | Planejado: `crates/sonix-core/tests/contract_025_presets_scenes.rs` |
| [026](026-afinador-ao-vivo-na-entrada.md) | Afinador ao vivo na entrada (taxa efetiva, 60 Hz–1,4 kHz, mute com rampa) | Rascunho | Planejado: `crates/sonix-desktop/tests/contract_026_live_tuner.rs` |

**Ordem de implementação recomendada (specs em rascunho):** 018 → 014 → 015 → 016 → 017 → 019 →
020 → 021 → 022 → 023 → 024 → 025 → 026. A latência (018) vem primeiro porque todo controle novo
é sentido através do caminho de áudio.
