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
| [001](001-nucleo-dsp-tempo-real-ipc-lock-free.md) | Isolamento em tempo real do motor DSP, comunicação lock-free e orçamento de CPU | Implementada (CA-7/CA-9 revertida: apenas biblioteca; CA-3/CA-5/CA-6 auditados pela Spec 044) | `crates/ampforge-dsp/tests/contract_001_nucleo_dsp.rs` + `contract_037_realtime_safety.rs` + `contract_044_sdd_integrity.rs` |
| [002](002-matriz-roteamento-sinal-dag-4x12.md) | Matriz de roteamento de sinal 4x12 (DAG), nós de Split/Mixer e múltiplos I/Os | Implementada (CA-1..CA-9 revertida: apenas biblioteca; produção aguarda Spec 045) | `crates/ampforge-dsp/tests/contract_002_matriz_dag.rs` (10 CAs biblioteca) |
| [003](003-modelagem-amplificadores-white-box-painel-frontal.md) | Modelagem White-Box de amplificadores valvulados e painel frontal autêntico | Implementada | `crates/ampforge-dsp/tests/contract_003_amp_modeling.rs` (8 CAs ok) |
| [004](004-bancada-chassi-aberto-mod-shop-osciloscopio-crt.md) | Bancada Chassi Aberto (Mod Shop) e osciloscópio CRT de transferência em tempo real | Implementada | `crates/ampforge-dsp/tests/contract_004_modshop_crt.rs` (9 CAs ok) |
| [005](005-gabinete-sala-microfonacao-ir-acoplamento-impedancia.md) | Bloco de gabinete (Cab), sala de microfonação dupla, loader de IR e acoplamento de impedância | Implementada | `crates/ampforge-dsp/tests/contract_005_virtual_cab.rs` (8 CAs ok) |
| [006](006-blocos-dinamica-drive-equalizacao.md) | Blocos de dinâmica (Noise Gate, Compressor), Overdrive/Fuzz White-Box e Equalizadores (RTA) | Implementada | `crates/ampforge-dsp/tests/contract_006_dynamics_drive_eq.rs` (8 CAs ok) |
| [007](007-blocos-modulacao-pitch-delay-reverb.md) | Blocos de modulação, pitch polifônico, delay com ducking/spillover e reverb | Implementada (CA-2/CA-3 revertida: pitch sem processamento até Spec 042; modelos FX na Spec 040) | `crates/ampforge-dsp/tests/contract_007_mod_pitch_delay_reverb.rs` |
| [008](008-afinador-estroboscopico-polifonico-looper.md) | Afinador estroboscópico/polifônico de alta precisão e Looper de performance | Implementada (CA-3/CA-6 revertida/parcial: polifônico e half-speed na Spec 043) | `crates/ampforge-dsp/tests/contract_008_tuner_looper.rs` |
| [009](009-cenas-gapless-canais-modifiers-footswitches.md) | Hierarquia Setlist/Preset, 8 Scenes gapless, 4 canais por bloco, Modifiers e modos de footswitch | Implementada (CA-1..CA-9 revertida: apenas biblioteca; produção aguarda Spec 045) | `crates/sonix-core/tests/contract_009_scenes_modifiers.rs` (biblioteca) |
| [010](010-persistencia-offline-schema-sonixpreset-pacotes-sforgepkg.md) | Persistência 100% offline-first, validação de schema .sonixpreset e pacotes .sforgepkg | Implementada | `crates/sonix-core/tests/contract_010_persistence_offline.rs` (8 CAs ok) |
| [011](011-design-system-skeuomorfico-componentes-3d-hardware-shell.md) | Design System skeuomórfico, tokens de materiais, controles táteis 3D e modo Hardware Shell | Implementada | `tests/contract/011-design-system-skeuomorfico-componentes-3d-hardware-shell.test.js` (9 CAs ok) |
| [012](012-host-desktop-standalone-tauri-plugins-vst3-clap.md) | Arquitetura Desktop nativa (Tauri v2 + Rust); módulo de DAW em passthrough sem binário VST3/CLAP entregue ainda | Implementada (CA-2/CA-3/CA-4/CA-5 revertida: `sonix-plugin` passthrough e sem binário VST3/CLAP; MIDI real na Spec 036) | `crates/sonix-desktop/tests/contract_012_desktop_plugin.rs` + `apps/sonix-app/src-tauri/tests/contract_033_desktop_shell.rs` |
| [013](013-execucao-desktop-windows-captura-fisica-p2-duplo-clique.md) | Execução desktop nativa no Windows com captura física P2/P3 e inicialização por duplo clique | Implementada | `crates/sonix-desktop/tests/contract_013_desktop_realtime_audio.rs` (8 CAs ok) |
| [014](014-ponte-de-controle-ui-dsp-tempo-real.md) | Ponte de controle UI→DSP em tempo real (tabela única de parâmetros, validação atômica, SPSC lock-free, bypass sem estalo) — implementar 2ª | Implementada (CA-8 revertida no review 2026-11-14 e corrigida com áudio real pela Spec 044) | `crates/sonix-desktop/tests/contract_014_ui_dsp_bridge.rs` (8 CAs ok) e `tests/contract/014-ponte-de-controle-ui-dsp-tempo-real.test.js` (CAs frontend ok) |
| [015](015-endurecimento-servidor-local-validacao-csrf.md) | Endurecimento do servidor local (token CSRF, Host/Origin, sem CORS, limite de corpo, defesa NaN/Inf no DSP) — implementar 3ª, mesma release da 014 | Implementada | `crates/sonix-desktop/tests/contract_015_server_hardening.rs` (9 CAs ok) e `tests/contract/015-endurecimento-servidor-local-validacao-csrf.test.js` (CAs frontend ok) |
| [016](016-cadeia-dsp-guitarra-tonestack-drive-gabinete-saida-segura.md) | Cadeia DSP de guitarra funcional (tone stack ativo, DriveProcessor, ganho 1x cab, saída segura 0 dBFS) | Implementada | `crates/ampforge-dsp/tests/contract_016_guitar_chain.rs` (13 CAs ok) |
| [017](017-resiliencia-carregador-ir-e-estado-looper.md) | Resiliência do carregador de IR (pânico com channels/sample_rate = 0) e undo/redo fiel do looper (camadas perdidas) | Implementada | `crates/ampforge-dsp/tests/contract_017_resilience_ir_looper.rs` (10 CAs ok) |
| [018](018-latencia-minima-host-audio-placa-onboard.md) | Latência mínima em placa onboard (WASAPI exclusivo/baixa latência/RAW, fila de 1 período, deriva, sem lock/alocação, medição por loopback) — implementar 1ª | Implementada | `crates/sonix-desktop/tests/contract_018_low_latency_host.rs` (11 CAs ok) e `tests/contract/018-latencia-minima-host-audio-placa-onboard.test.js` (CA-8 ok) |
| [019](019-entrada-guitarra-placa-onboard-calibracao-sinal-teste.md) | Entrada de guitarra na placa onboard (calibração de nível e gate, clipping do conversor, sinal de teste pelo motor) | Implementada (CA-2/CA-7 reabertas pela Spec 051: T-01/T-17) | `crates/sonix-desktop/tests/contract_019_input_calibration.rs` (9 CAs ok) e `tests/contract/019-entrada-guitarra-placa-onboard-calibracao-sinal-teste.test.js` (CAs frontend ok) |
| [020](020-amplificadores-por-modelo-painel-fiel.md) | Amplificadores por modelo (5 modelos, painel com os controles reais, tone stack pelo circuito citado, oversampling selecionável) | Implementada | `crates/ampforge-dsp/tests/contract_020_amp_models.rs`, `crates/sonix-desktop/tests/contract_020_amp_models_api.rs`, `tests/contract/020-amplificadores-por-modelo-painel-fiel.test.js` |
| [021](021-bancada-chassi-aberto-osciloscopio-ao-vivo.md) | Bancada do chassi aberto e osciloscópio ao vivo (válvulas, bias, sag, NFB, transformador, retificador, tone stack) | Implementada | `crates/ampforge-dsp/tests/contract_021_workbench_live.rs`, `crates/sonix-desktop/tests/contract_021_scope_api.rs`, `tests/contract/021-bancada-chassi-aberto-osciloscopio-ao-vivo.test.js` |
| [022](022-gabinete-microfones-ir-ao-vivo.md) | Gabinete, dois microfones e IR ao vivo (acoplamento de impedância, proximidade, filtro pente, convolução sem latência adicional) | Implementada | `crates/ampforge-dsp/tests/contract_022_cab_mic_ir.rs`, `crates/sonix-desktop/tests/contract_022_cab_api.rs`, `tests/contract/022-gabinete-microfones-ir-ao-vivo.test.js` |
| [023](023-pedal-drive-808-editor-de-bloco.md) | Pedal de drive 808 fiel ao circuito e editor do bloco selecionado | Implementada | `crates/ampforge-dsp/tests/contract_023_drive_808.rs`, `tests/contract/023-pedal-drive-808-editor-de-bloco.test.js` |
| [024](024-efeitos-e-caminho-paralelo-ao-vivo.md) | Efeitos e caminho paralelo do grid ao vivo (compressor FET, EQ, chorus, eco de fita, placa, split/merge, amp 2/cab 2, FS2–FS4) | Implementada (CA-5 reaberta pela Spec 051: T-11) | `crates/ampforge-dsp/tests/contract_024_live_graph_fx.rs`, `crates/sonix-desktop/tests/contract_024_graph_api.rs`, `tests/contract/024-efeitos-e-caminho-paralelo-ao-vivo.test.js` |
| [025](025-presets-cenas-canais-de-bloco-reais.md) | Presets, cenas e canais de bloco reais (estado completo, schema 1.1.0, import/export pelo motor, troca sem buraco) | Implementada (CA-1 reaberta pela Spec 051: T-05) | `crates/sonix-core/tests/contract_025_presets_scenes.rs`, `crates/sonix-desktop/tests/contract_025_preset_switch.rs`, `tests/contract/025-presets-cenas-canais-de-bloco-reais.test.js` |
| [026](026-afinador-ao-vivo-na-entrada.md) | Afinador ao vivo na entrada (taxa efetiva, 60 Hz–1,4 kHz, mute com rampa) | Implementada (CA-6 reaberta pela Spec 051: T-13) | `crates/sonix-desktop/tests/contract_026_live_tuner.rs`, `tests/contract/026-afinador-ao-vivo-na-entrada.test.js` |
| [027](027-ui-controles-reais-e-layout-responsivo.md) | UI com controles reais para todos os blocos/knobs/Mod Shop e layout responsivo sem barra de rolagem | Implementada | `tests/contract/027-ui-controles-reais-e-layout-responsivo.test.js` (8 CAs ok) |
| [028](028-latencia-ultrabaixa-wasapi-realtime-safety.md) | Latência ultrabaixa WASAPI Exclusivo/IAudioClient3, SPSC lock-free/zero-alloc e telemetria real | Implementada | `crates/sonix-desktop/tests/contract_028_low_latency_realtime_safety.rs` (7 CAs ok) |
| [029](029-afinador-e-calibracao-entrada-funcionais.md) | Afinador (YIN 40 Hz–1,4 kHz + contrato JSON UI/API) e Calibração de Entrada conectados ponta a ponta | Implementada (CA-5/CA-6 reabertas pela Spec 051: T-01) | `crates/sonix-desktop/tests/contract_029_tuner_and_calibration.rs` & `tests/contract/029-afinador-e-calibracao-entrada.test.js` (CA-1..CA-8 ok) |
| [030](030-noise-gate-completo-com-histerese-e-sidechain.md) | Noise Gate completo com histerese, hold, range e sidechain HPF/LPF (inspirado no Boss NS-2 / ISP Decimator) | Implementada | `crates/ampforge-dsp/tests/contract_030_noise_gate.rs`, `crates/sonix-desktop/tests/contract_030_gate_api.rs`, `tests/contract/030-noise-gate-ui.test.js` |
| [031](031-amplificadores-fieis-aos-esquemas-e-controles-originais.md) | Amplificadores White-Box fiéis aos esquemas originais (1959 SLP, JCM800 2203, Twin AB763, Dual Rectifier, AC30 TB), válvulas e oversampling real | Implementada (CA-5 reaberta pela Spec 051: T-04 `top_boost`) | `contract_031_authentic_amps.rs`, `contract_031_amp_controls_api.rs`, `031-amplificadores-fieis-ui.test.js` |
| [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md) | Pedais, EQs, compressores, delay RE-201, modulação, reverb e gabinetes fiéis aos esquemas elétricos originais | Implementada | `contract_032_pedals_fx_cabs.rs`, `contract_032_fx_api.rs`, `032-pedais-fx-cabs-ui.test.js` |
| [033](033-app-desktop-tauri-v2-telas-e-controles-reais.md) | App desktop Tauri v2 (sem HTTP/Edge no release), 6 telas do design, painéis fiéis (Marshall conferidos em manual) e todo controle visível com efeito real no motor | Implementada (CA-4/CA-5 reabertas pela Spec 051: T-00, T-23) | `apps/sonix-app/src-tauri/tests/contract_033_desktop_shell.rs`, `crates/sonix-desktop/tests/contract_033_control_effects.rs`, `tests/contract/033-app-desktop-tauri-v2-telas-e-controles-reais.test.js` |
| [034](034-ponte-de-controle-e-host-de-audio-pos-review.md) | Ponte de controle e host de áudio: parâmetro descartado sob contenção, ação dupla, período/taxa errados, troca de dispositivo decorativa (review de `e1ffff3`) | Implementada | `crates/sonix-desktop/tests/contract_034_control_bridge_host.rs`, `apps/sonix-app/src-tauri/tests/contract_034_audio_device.rs` |
| [035](035-contratos-ui-backend-e-interacao-dos-knobs.md) | Contratos UI↔backend (telemetria, osciloscópio, presets, dispositivos) e arraste dos knobs (review de `e1ffff3`) | Implementada (CA-6 reaberta pela Spec 051: T-00, T-02) | `crates/sonix-desktop/tests/contract_035_ui_backend_contracts.rs`, `tests/contract/035-036-host-ui-contracts.test.js` (`CA-1`..`CA-10`), `tests/fixtures/ipc-schemas/*.json` |
| [036](036-host-de-audio-real-e-telemetria-honesta.md) | Host de áudio real e telemetria honesta: dispositivos, modo, latência medida, canal de entrada, ASIO/exclusivo e MIDI | Implementada (CA-3 reaberta pela Spec 051: T-25) | `crates/sonix-desktop/tests/contract_036_audio_host_midi_telemetry.rs` (`CA-1`..`CA-10`), `tests/contract/035-036-host-ui-contracts.test.js` |
| [037](037-seguranca-de-tempo-real-na-cadeia-ao-vivo.md) | Segurança de tempo real na cadeia ao vivo: zero alocação/desalocação/lock no callback, carga de IR e preset sem buraco | Implementada | `crates/sonix-desktop/tests/contract_037_realtime_safety.rs`, `contract_026_live_tuner.rs`, `contract_001_nucleo_dsp.rs` |
| [038](038-gabinete-ir-convolucao-e-microfones-com-efeito.md) | Gabinete: convolução particionada de IR, modelo de microfone e ressonância com efeito real | Implementada (CA-6 reaberta pela Spec 051: T-12, T-16) | `crates/ampforge-dsp/tests/contract_038_cab_ir_mics_resonance.rs` (CA-1..CA-5, CA-8, CA-9), `contract_038_cab_api.rs` (CA-7, CA-9), `tests/contract/038-cab-ir-ui.test.js` (CA-6) |
| [039](039-amplificadores-fidelidade-e-robustez.md) | Amplificadores: troca de modelo sem estalo, coeficientes por taxa, master fantasma no Plexi, tone stacks AB763/Top Boost pelo esquema | Implementada | `crates/ampforge-dsp/tests/contract_039_amps_fidelity.rs` (`CA-1`..`CA-8`), `contract_003_amp_modeling.rs`, `contract_020_amp_models_api.rs` |
| [040](040-modulacao-delay-e-reverb-com-modelos-distintos.md) | Modulação, delay e reverb: cada modelo com o mecanismo do equipamento real e sem controles mortos | Implementada (CA-9 reaberta pela Spec 051: T-07 `mix`) | `crates/ampforge-dsp/tests/contract_040_mod_delay_reverb.rs` (`CA-2`..`CA-11` ok), `contract_007_mod_pitch_delay_reverb.rs`, `contract_024_live_graph_fx.rs`, `contract_032_pedals_fx_cabs.rs` |
| [041](041-dinamica-e-drives-comportamento-do-circuito.md) | Dinâmica e drives: All-Buttons-In monotônico, ratio/modo do gate com efeito, anti-aliasing e TS808/RAT pelo circuito | Implementada | `crates/ampforge-dsp/tests/contract_041_dynamics_and_drives.rs` (`CA-1`..`CA-3`, `CA-5`..`CA-7`), `crates/sonix-desktop/tests/contract_041_gate_di.rs` (`CA-4`) |
| [042](042-bloco-de-pitch-sem-processamento.md) | Bloco de pitch sem processamento (Spec 007): implementar pitch shifter e harmonizador diatônico (decisão do usuário: implementar) | Implementada | `crates/ampforge-dsp/tests/contract_042_pitch_shifter.rs` (`CA-1`..`CA-4`, `CA-7`), `crates/sonix-desktop/tests/contract_042_pitch_api.rs` (`CA-5`, `CA-6`), `contract_037_realtime_safety.rs` |
| [043](043-afinador-polifonico-temperamento-e-looper.md) | Afinador (polifônico, temperamento, precisão medida) e looper (meia velocidade, undo, capacidade por taxa) | Implementada | `crates/ampforge-dsp/tests/contract_043_tuner_looper.rs` (`CA-1`..`CA-6` ok), `contract_008_tuner_looper.rs`, `contract_017_resilience_ir_looper.rs` |
| [044](044-integridade-do-sdd-codigo-morto-e-testes-tautologicos.md) | Integridade do SDD: código morto entregue como implementado, testes tautológicos e inventário de controles honesto | Implementada | `crates/sonix-desktop/tests/contract_044_sdd_integrity.rs` (7 CAs ok), `contract_014_ui_dsp_bridge.rs` (`CA-8`), `contract_001_nucleo_dsp.rs` (`CA-5`) |
| [045](045-cenas-presets-e-grafo-reais.md) | Cenas audíveis, troca de preset/cena sem buraco, salvar com cenas/canais e grafo com estado real | Implementada (CA-1 reaberta pela Spec 051: T-03 `scene_id`) | `crates/sonix-desktop/tests/contract_045_scenes_presets_graph.rs` (CA-1..CA-7 ok), `contract_037_realtime_safety.rs` (10/10 ok) |
| [046](046-endurecimento-servidor-dev-e-persistencia.md) | Endurecimento do servidor dev-http (Host/Origin exatos, token CSPRNG) e da persistência (id de preset, caminhos de pacote) | Implementada | `crates/sonix-core/tests/contract_046_package_hardening.rs` (2 CAs ok), `crates/sonix-desktop/tests/contract_046_server_persistence_hardening.rs` (5 CAs ok) |
| [047](047-bancada-e-osciloscopio-com-dados-do-motor.md) | Bancada e osciloscópio com dados do motor (transferência, forma de onda, tone stack, impedância, válvulas por modelo) | Implementada (CA-1/CA-6 reabertas pela Spec 051: T-06, T-15) | `crates/sonix-desktop/tests/contract_047_workbench_scope.rs` (`CA-1`..`CA-4`, `CA-6`, `CA-7`), `contract_021_scope_api.rs`, `contract_037_realtime_safety.rs` |
| [048](048-ui-desktop-fiel-ao-design-aprovado.md) | UI desktop fiel ao design aprovado, com referência renderizada e régua de regressão visual | Implementada (CA-5/CA-6 reabertas pela Spec 051: T-19..T-26) | `tests/contract/048-ui-desktop-fidelidade-visual.test.js` (CA-1..CA-12 ok), `tests/contract/033-app-desktop-tauri-v2-telas-e-controles-reais.test.js` |
| [049](049-correcoes-pos-review-das-specs-034-048.md) | Correções do review da entrega 034–048 (estado na troca de cena, reconfiguração de dispositivo, double free na fila de recursos, API/UI honestas, oversampler) e pendências abertas: plugins VST3/CLAP (012), DAG 4×12 (002), Setlists/Modifiers/footswitches (009), ASIO, entrada instrumento/linha e reamp | Implementada (`CA-31` reaberta pela Spec 051: T-09; `CA-36` e validadores externos `CA-16` pendentes) | `crates/sonix-desktop/tests/contract_049_*.rs`, `crates/ampforge-dsp/tests/contract_049_oversampler.rs`, `crates/sonix-plugin/tests/contract_049_plugins.rs`, `tests/contract/049-*.test.js` |
| [050](050-correcao-inicializacao-tauri-e-chiado-audio.md) | Correção da inicialização do shell Tauri release (frontend embutido sem `localhost:1420`) e eliminação de chiado ao abrir (Standby inicial sem calibração e alinhamento de taxa duplex) | Implementada | `tests/contract/050-release-embedded-ui.test.js`, `apps/sonix-app/src-tauri/tests/contract_050_startup_safety.rs`, `crates/sonix-desktop/tests/contract_050_duplex_rate_alignment.rs` |
| [051](051-correcao-contratos-e-controles-teste-manual-2026-11-16.md) | Correção de contratos UI↔backend, controles sem efeito e usabilidade do teste manual de 2026-11-16 (itens T-00 a T-26; auditoria e revisão de fidelidade DSP/tempo real em 2026-11-18) | Implementada | `crates/sonix-desktop/tests/contract_051_manual_test_fixes.rs` (15/15 ok), `tests/contract/051-correcao-contratos-e-controles-ui.test.js` (24/24 ok) |
| [052](052-abertura-animada-pedais-faltantes-e-amp2-visivel.md) | Abertura animada com emblema SVG vetorial, designs skeuomórficos dos pedais faltantes e mini-cabeça AMP 2 no Estúdio (itens T-27 a T-29) | Aprovada | `crates/ampforge-dsp/tests/contract_052_wah_and_pedals.rs`, `tests/contract/052-abertura-pedais-amp2.test.js` |

**Ordem de implementação recomendada (specs anteriores já concluídas):**
1. **028** e **027** — implementadas.
2. **029** e **030** — implementadas.
3. **031** e **032** — implementadas.
4. **033** — implementada com inventário de controles zerado (`docs/ui-desktop/inventario-controles.md`).

*(Specs 001 a 033 implementadas. `apps/sonix-app` entrega o shell Tauri v2 nativo sem HTTP/Edge no release, restaurando o CA-6 da Spec 012.)*

**Review manual de 2026-11-14 (Specs 001–033) → Specs 034–048.** A Spec 044 corrige os estados do índice
acima onde a entrega não está no caminho de produção. Ordem recomendada:
1. **034** e **037**: ponte de controle e segurança de tempo real (base para todas as outras).
2. **046** (segurança do servidor local) e **044** (estados honestos e testes que reprovam).
3. **036** e **035**: host e telemetria reais, contratos UI↔backend.
4. **048**: UI fiel ao design (depende de 035/036 para os dados reais na tela).
5. **045**, **038**, **039**, **041**, **040**, **047** e **043**: comportamento dos equipamentos.
6. **042**: implementar o bloco de pitch (decidido pelo usuário em 2026-11-14).

**Review da entrega 034–048 (2026-11-15) → Spec 049**, spec única por decisão do usuário. Ordem das partes: A (CA-7 e CA-14 primeiro) → A′ → F (ASIO, instrumento/linha, reamp) → C (DAG) → D (Setlists/Modifiers/footswitches) → B (plugins CLAP + VST3, evolução futura; licença resolvida: projeto sob CC BY-NC 4.0 - Atribuição e Não-Comercial).
