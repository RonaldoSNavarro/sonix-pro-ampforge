# Relatório de Execução — Auditoria 2026-11-20 (`fix/auditoria-2026-11-20`)

## 1. Tabela por item

| AUD | Estado | Commits | Testes (nomes) | Red (comando + trecho) | Green (comando + trecho) | Medição antes → depois |
|---|---|---|---|---|---|---|
| AUD-01 | FEITO | `2c2ac38` | `auditoria_2026_11_20_seguranca::aud_01_ipc_reamp_recusa_session_path` | `cargo test -p sonix-app --test auditoria_2026_11_20_seguranca` (em `2c2ac38^`) → `error[E0432]: unresolved import sonix_app_lib::commands::run_reamp_ipc \| error[E0599]: no method named set_data_dir found for struct ApiContext` | `cargo test --locked -p sonix-app --test auditoria_2026_11_20_seguranca` → `1 passed; 0 failed` | CSRF/Host/reamp sem validação → validação estrita + recusa `session_path` |
| AUD-02 | FEITO | `8cfd49a` | `auditoria_2026_11_20_tempo_real::aud_02_*` (20 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_tempo_real` (em `8cfd49a^`) → `error[E0432]: unresolved import ampforge_dsp::denormal \| unresolved import ampforge_dsp::regua` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_tempo_real` → `20 passed; 0 failed` | subnormais > 0 → `subnormais: 0` com `DenormalGuard` e `flush_f32`/`flush_f64` |
| AUD-03 | FEITO | `813120f` | `contract_015_server_hardening::test_ca_05`, `contract_019_input_calibration::test_ca_07` | Flaky intermitente em `8f499ac` (`contract_015::test_ca_05` e `contract_019::test_ca_07` fechavam socket antes de drenar corpo sem `411`) | `20× contract_015::test_ca_05` → `20/20`; `20× contract_019::test_ca_07` → `20/20` | fechamento abrupto sem `411` → dreno + `411 Length Required` |
| AUD-04 | FEITO | `c079a4c` | `contract_018_low_latency_host::test_ca_11_dsp_panic_catch_and_stream_resilience` | `cargo test -p sonix-desktop --test contract_018_low_latency_host test_ca_11` (em `c079a4c^`) → `error[E0432]: unresolved import sonix_desktop::audio::recover_dsp_chain_if_faulted \| error[E0599]: no method named panic_count` | `cargo test --locked -p sonix-desktop --test contract_018_low_latency_host --features fault-injection` → `10 passed; 0 failed` | `panic=abort` / `unwrap()` → `panic=unwind` + `catch_unwind` no callback |
| AUD-05 | FEITO | `5e01959`, `2f9c699` | `auditoria_2026_11_20_tempo_real::aud_05_*`, `033::CA-12` | `cargo test -p sonix-app --test auditoria_2026_11_20_tempo_real` (em `5e01959^`) → `error[E0432]: unresolved imports AudioDeviceConfig, AudioDeviceSupervisor \| no method named stream_error_code_handle` | `cargo test --locked -p sonix-app --test auditoria_2026_11_20_tempo_real` → `2 passed; 0 failed` | erro `cpal` ignorado → `device_state=lost` + banner na UI + reabertura |
| AUD-06 | FEITO | `5922ecc` | `contract_049_scene_state_and_queues::ca14_*` | `cargo test -p sonix-desktop --test contract_049_scene_state_and_queues ca14` (em `5922ecc^`) → `try_lock` entregava DI cru e mute do afinador vazava quando em contenção | `cargo test --locked -p sonix-desktop --test contract_049_scene_state_and_queues` → `12 passed; 0 failed` | contenção no mutex da cadeia → zero contenção + mute/standby respeitados |
| AUD-07 | FEITO | `949560d` | `contract_037_realtime_safety` | `cargo test -p sonix-desktop --test contract_037_realtime_safety` (em `949560d^`) → recurso aposentado usava `mem::forget` e `unpark()` no thread de tempo real | `cargo test --locked -p sonix-desktop --test contract_037_realtime_safety` → `10 passed; 0 failed` | `mem::forget` + `unpark` → fila pendentes + backpressure + `park_timeout` |
| AUD-08 | FEITO | `f6a0d50` | `auditoria_2026_11_20_parametros::aud_08_*`, `auditoria-2026-11-20-parametros.test.js::AUD-08` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_parametros` (em `f6a0d50^`) → `error[E0599]: no variant ALL found for enum AmpModelId \| no method named has_presence` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_parametros` → `2 passed; 0 failed` | Ver tabela de ganho de pequeno sinal abaixo (`4b6ea3d` → HEAD) |
| AUD-09 | FEITO | `b150165` | `auditoria-2026-11-20-parametros.test.js::AUD-09` | `node --test tests/contract/auditoria-2026-11-20-parametros.test.js` (em `b150165^`) → `✖ tests\contract\auditoria-2026-11-20-parametros.test.js (497.77ms) 'test failed'` | `node --test tests/contract/auditoria-2026-11-20-parametros.test.js` → `4 passed; 0 failed` | tabela duplicada TS/Rust → `PARAMS_TABLE` única em `params.rs` gerando `params.generated.ts` |
| AUD-10 | FEITO | `bfc41dc` | `auditoria_2026_11_20_artefatos::aud_10_*` (3 testes) | `aud_10_troca_com_bloco_32` falhou em 17/20 pares (`max razao_transiente=69.895`); `aud_10_amp2_troca_de_modelo_sem_clique` falhou em 5/8 pares (`max=56.814`) | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_10` → `3 passed; 0 failed` | troca de modelo com clique → crossfade constante com 2 instâncias completas |
| AUD-11 | FEITO | `bab7c5a`, `909d49c` | `auditoria_2026_11_20_artefatos::aud_11_*` (3 testes) | `aud_11_degrau_sem_zipper` falhou com zipper de até `797.57×` (`amp.bass=797.57×`, `middle=37.75×`, `eq=51.29×`, `cab.hicut=10.69×`, `wah.position=3.44×`) | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_11` → `3 passed; 0 failed` (COR-04: `0↔10` em `master`/`volume_2`) | degrau com zipper → rampa linear de 20 ms com sub-blocos |
| AUD-12 | FEITO | `b53d833`, `a9c28f9` | `auditoria_2026_11_20_artefatos::aud_12_*` (4 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_12` (em `b53d833^`) → `error[E0599]: no method named wet_latency_ms \| no field ops_ultima_chamada` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_12` → `4 passed; 0 failed` | seco com atraso de 1024 amostras → seco sem atraso, wet 512 amostras |
| AUD-13 | FEITO | `f75e73d` | `contract_035_ui_backend_contracts`, `contract_039_amps_fidelity` | `contract_035_ui_backend_contracts` reportava latência fixa sem medição por impulso em `prepare()` | `cargo test --locked -p sonix-desktop --test contract_035_ui_backend_contracts` → `5 passed; 0 failed` | latência fixa → medição por impulso em `prepare()` + suporte 1x |
| AUD-14 | FEITO | `3508261` | `auditoria_2026_11_20_artefatos::aud_14_*` (2 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_14` (em `3508261^`) → `error[E0599]: no method named measure_ai_filter_magnitude_db` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_14` → `2 passed; 0 failed` | aliasing alto ganho → elíptico IIR 8ª ordem 2x/4x/8x (`>= 60 dB` rejeição) |
| AUD-15 | **NÃO ATENDIDO** | `485d19b`, `c095198`, `22520c7`, `f14f23c`..`5d936cc` | `auditoria_2026_11_20_artefatos::aud_15_*` (3 testes), `bancada_auditoria` (`pesada` 96k/32) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_15` (em `485d19b^`) → `error[E0599]: no method named ultima_chamada_macs found for struct VirtualCabProcessor` | `aud_15_*` → `3 passed; 0 failed`; `pesada` 96k/32 (5×3, `tempo_real`): mediana `excesso_sobre_ruido = 196` (`media_pct = 38.787%`, melhor rodada `118.105 µs/bloco` / `excesso = 23` em `724b837`); critério `<= 0` não atingido | convolução complexa → Real FFT 256 + SoA SSE2 + otimizações COR-12 (`141.274` → `118.105 µs/bloco` em `724b837`, `129.289 µs` na suite 5×3); `excesso_sobre_ruido = 196 > 0` |
| AUD-16 | FEITO | `7242281` | `contract_049_plugins` | `cargo test -p sonix-plugin` (em `7242281^`) → `error[E0432]: unresolved import sonix_plugin::PluginParamValues \| no method named process_block` | `cargo test --locked -p sonix-plugin` → `15 passed; 0 failed` | `sonix-plugin` passthrough → `SonixPluginProcessor::process_block` real |
| AUD-17 | FEITO | `35ab95d` | `contract_036_audio_host_midi_telemetry`, `contract_049_midi_ranges` | `cargo test -p sonix-desktop --test contract_036_audio_host_midi_telemetry` (em `35ab95d^`) → `error[E0433]: cannot find MidiRouter in midi` | `cargo test --locked -p sonix-desktop --test contract_036_audio_host_midi_telemetry` → `12 passed; 0 failed` | MIDI mock → `midir` + `MidiRouter` em tempo real |
| AUD-18 | FEITO | `8311536` | `auditoria_2026_11_20_persistencia::aud_18_*` (3 testes) | `cargo test -p sonix-app --test auditoria_2026_11_20_persistencia -- aud_18` (em `8311536^`) → `error[E0432]: unresolved import sonix_app_lib::build_runtime_with_data_dir \| struct AppRuntime is private` | `cargo test --locked -p sonix-app --test auditoria_2026_11_20_persistencia -- aud_18` → `3 passed; 0 failed` | calibração em memória → persistência atômica por dispositivo |
| AUD-19 | FEITO | `a3f00dc` | `auditoria_2026_11_20_persistencia::aud_19_*`, `auditoria_2026_11_20_sforgepkg`, `auditoria-2026-11-20-parametros.test.js::AUD-19` | `cargo test -p sonix-core --test auditoria_2026_11_20_sforgepkg` (em `a3f00dc^`) → `error[E0432]: unresolved imports MAX_PACKAGE_BYTES, MAX_PACKAGE_RESOURCES, MAX_RESOURCE_BYTES` | `cargo test --locked -p sonix-core --test auditoria_2026_11_20_sforgepkg` → `3 passed; 0 failed` | export/import incompleto → `.sforgepkg` v2 base64 com limites |
| AUD-20 | FEITO | `c85e8cf` | `auditoria_2026_11_20_split_dag` | `cargo test -p sonix-desktop --test auditoria_2026_11_20_split_dag` (em `c85e8cf^`) → `error[E0599]: no method named set_ratio found for struct SplitNode` | `cargo test --locked -p sonix-desktop --test auditoria_2026_11_20_split_dag` → `4 passed; 0 failed` | split balance resetava → crossover LR4 + split dinâmico parametrizado |
| AUD-21 | FEITO | `37283fb` | `auditoria_2026_11_20_split_dag`, `auditoria-2026-11-20-parametros.test.js::AUD-21` | `node --test tests/contract/auditoria-2026-11-20-parametros.test.js` (em `37283fb^`) → `✖ tests\contract\auditoria-2026-11-20-parametros.test.js (492.21ms) 'test failed'` | `node --test tests/contract/auditoria-2026-11-20-parametros.test.js` → `4 passed; 0 failed` | wah implícito → bloco `wah_1` explícito na grade + Linha 2 esmaecida |
| AUD-22 | FEITO | `48e26fb` | `contract_039_amps_fidelity::aud_22_stack_igual_a_yeh` | `cargo test -p ampforge-dsp --test contract_039_amps_fidelity -- aud_22` (em `48e26fb^`) → `error[E0432]: unresolved import analog_tonestack_magnitude_db_for_components \| no method named update_with_components` | `cargo test --locked -p ampforge-dsp --test contract_039_amps_fidelity -- aud_22` → `1 passed` (`err <= 0.05 dB` analógico, `<= 0.10 dB` digital) | matriz FMV/AB763 divergente → validada contra Yeh & Smith |
| AUD-23 | FEITO | `08e6d3c` | `auditoria_2026_11_20_amps::aud_23*` (4 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_23` (em `08e6d3c^`) → `error[E0599]: no variant RectoPostGain found for enum ToneStackTopology` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_23` → `4 passed; 0 failed` | scoop duplo Twin / Presence Modern → corrigidos por modelo |
| AUD-24 | FEITO | `6871ac8` | `auditoria_2026_11_20_amps::aud_24*` (2 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_24` (em `6871ac8^`) → `error[E0599]: no method named ganho_estagio1_pequeno_sinal found for struct WhiteBoxAmpProcessor` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_24` → `2 passed; 0 failed` | LOW JCM800 só atenuava → pula V1a; Plexi LOW `-6 dB` |
| AUD-25 | FEITO | `5a5992d` | `auditoria_2026_11_20_amps::aud_25_*` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_25` (em `5a5992d^`) → `test aud_25_impedancia_menor_mais_ressonante ... FAILED (panicked at auditoria_2026_11_20_amps.rs:415:9)` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_25` → `1 passed; 0 failed` | sentido invertido → `4 > 8 > 16 Ohm` conforme Tonal Note 3 |
| AUD-26 | FEITO | `d90db79` | `auditoria_2026_11_20_amps::aud_26_*` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_26` (em `d90db79^`) → `test aud_26_middle_10pct_log ... FAILED (panicked at auditoria_2026_11_20_amps.rs:486:9)` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_26` → `1 passed; 0 failed` | `map_pot_mid_10pct` duplicado → `audio_taper` 10% log direto |
| AUD-27 | FEITO | `24a92cb`, `c095198` | `auditoria_2026_11_20_amps::aud_27*` (2 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_27` (em `24a92cb^`) → `error[E0432]: unresolved imports Jcm800PeakingNetwork, PlexiBrightCapFilter` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_27` → `2 passed; 0 failed` | bright cap e rede 470k\|\|470pF ausentes → 1ª ordem fiel |
| AUD-28 | FEITO | `bf80361` | `auditoria_2026_11_20_efeitos::aud_28_*` (4 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_28` (em `bf80361^`) → `test aud_28_cabecas_por_modo ... FAILED \| test aud_28_mola_tem_cauda ... FAILED \| test aud_28_wow_audivel ... FAILED (panicked at :77:17)` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_28` → `4 passed; 0 failed` | modos 5–12 RE-201 sem mola/invertidos → tabela única 12 modos + mola 5..12 + wow 3 cents |
| AUD-29 | FEITO | `6913cd5` | `auditoria_2026_11_20_efeitos::aud_29_*` (4 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_29` (em `6913cd5^`) → `error[E0432]: unresolved import DECISAO_SONIX_LM308_GBW \| unresolved import ampforge_dsp::unidades` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_29` → `4 passed; 0 failed` | ganhos/limiares arbitrários → `VOLTS_POR_FS`, ganho TS808/RAT (GBW LM308), DS-1 simétrico |
| AUD-30 | FEITO | `942fee8` | `auditoria_2026_11_20_efeitos::aud_30_*` (4 testes), `contract_040_mod_delay_reverb` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_30` (em `942fee8^`) → `error[E0432]: unresolved import Ac30VibTremMode \| no method named set_tzf_inverted_polarity` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_30` → `4 passed; 0 failed` | TZF/Rotary/Optical genéricos → TZF 1ms dry + inversão, Rotary 2 rotores LR4 800Hz, LDR 5/60ms, AC30 Vib/Trem |
| AUD-31 | FEITO | `1d6dcb8` | `auditoria_2026_11_20_efeitos::aud_31b_dispersao`, `aud_31c_rt60_twin` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_31` (em `1d6dcb8^`) → `test aud_31c_rt60_twin ... FAILED (panicked at auditoria_2026_11_20_efeitos.rs:885:5)` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_31` → `2 passed; 0 failed` | dispersão mola < 5 ms / nomes não documentados → dispersão >= 5 ms, Twin RT60 3.0s, nomes Freeverb (Jezar) |
| AUD-32 | FEITO | `9265cc4` | `auditoria_2026_11_20_efeitos::aud_32_*` (2 testes), `038-cab-ir-ui.test.js` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_32` (em `9265cc4^`) → `error[E0432]: unresolved import speaker_curve_points_hz_db \| no method named speaker_fir_taps` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_32` → `2 passed; 0 failed` | hi-cut ignorado com IR → hi-cut pós-IR + FIR fase mínima 384 taps + seletor desabilitado com IR |
| AUD-33 | FEITO | `b6f99f1` | `auditoria_2026_11_20_afinador::aud_33_*` (2 testes), `contract_026_live_tuner`, `contract_043_tuner_looper` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_afinador -- aud_33` (em `b6f99f1^`) → `test aud_33_detecta_de_41_a_1400 ... FAILED (panicked at auditoria_2026_11_20_afinador.rs:52:17)` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_afinador -- aud_33` → `2 passed; 0 failed` | faixa 60 Hz–1.4 kHz / Peterson → 38–1405 Hz, `k*tau` > 330 Hz `+-0.1 cent`, `tuner_scratch = 8192` |
| AUD-34 | FEITO | `340acfa`, `e11912c` | `auditoria_2026_11_20_afinador::aud_34_poli_com_harmonicos`, `contract_051_manual_test_fixes` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_afinador -- aud_34` (em `340acfa^`) → `test aud_34_poli_com_harmonicos ... FAILED (panicked at auditoria_2026_11_20_afinador.rs:136:17)` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_afinador -- aud_34` → `1 passed; 0 failed` | polifônico sem filtro de harmônicos → FFT Hann 4x zero-padding + exclusão de parciais `+-15 cents` |
| AUD-35 | FEITO | `3218d31` | `contract_039::ca2_amp_resposta_linear_igual_entre_taxas`, `contract_035::test_ca_08`, `contract_049::ca3_009`, `ca4_009`, `auditoria-2026-11-20-ui-legado.test.js` | `ca2_amp_resposta_linear_igual_entre_taxas` → `FAILED` (`1.105 dB` @ 16 kHz) | `cargo test --locked -p ampforge-dsp --test contract_039_amps_fidelity` → `10 passed`; `contract_035` → `5 passed`; `ui-legado` → `4 passed` | `1.105 dB` @ 16 kHz (sinc + elliptic) → `<= 0.5 dB` (zero-stuffing + elliptic); `048-diffs` em `os.tmpdir()` |
| AUD-36 | FEITO | `5d949b7` | `033::ca3_*`, `contract_039::ca5_*`, `contract_044::ca4_*`, `ca15_*`, `contract_047::ca5_*`, `npm run test:e2e` | `ca5_stack_fender_ab763_coincide_com_yeh` → `FAILED` (taper vs linear); `run-033-e2e` timeout → `Plexi 1959` chip | `npm run test:e2e` → `E2E 033 GREEN`; `044` → `9 passed`; `047` → `6 passed`; `033` → `6 passed` | E2E sem executor + 5 CAs sem teste → `npm run test:e2e` + 5 testes `caN_*` + docs manuais `018-CA-10.md`, `022-CA-14.md` |
| AUD-37 | FEITO | `c20cc65` | `048-ui-desktop-fidelidade-visual.test.js` (Main + Audio DOM & SSIM) | `048` Audio_body → `SSIM=0.6878` (aba `dev` vs `Audio.png`) | `048` → `Audio_title=0.8769`, `Audio_body=0.8609`, `Audio FULL=0.8643`, `Main_stage=0.8512`, `Main FULL=0.8999` | Audio sem SSIM (`0.6878`) → `Audio_body=0.8609`, `Audio FULL=0.8643` |
| AUD-38 | FEITO | `3e8c340` | `tools/verificar_specs.py` | `verificar_specs.py` (pré-limpeza) → identificadores inexistentes/trocados | `python tools/verificar_specs.py` → `92 specs, 173 citações válidas, 0 inválidas, 0 front-matter ausente, 0 IDs inexistentes` | 17 IDs fabricados/trocados e 059..092 sem YAML `status: rascunho` → 0 erros |
| AUD-39 | FEITO | `9087fe4` | `contract_053_auditoria_039::ca4_invariancia_ao_tamanho_de_bloco`, `contract_018`, `contract_028`, `contract_044` | `contract_018`/`028` mantinham testes simulados de modo exclusivo (`test_ca_06`); `ca4_invariancia_ao_tamanho_de_bloco` já passava em `9087fe4^` pois a propriedade já valia (§1.2) | `contract_053_auditoria_039` → `1 passed` (bit-idêntico `32,37,64,256,480,512,1024` vs `128`); `018` `10 passed`; `028` `6 passed` | 028 inflada como implementada → `parcial`, item UI desabilitado (`não implementado`), `053 CA-4=TESTED`, `CA-9=NÃO ATENDIDO` |
| AUD-40 | FEITO | `4f9eed5` | `contract_001_nucleo_dsp`, `contract_044_sdd_integrity` | N/A (`[DOC]` + remoção `MixedPrecisionSolver`) | `contract_001` → `11 passed`; `contract_044` → `9 passed` | frases contraditas (`Newton-Raphson`, `FIR > 96 dB`, `< 1,5 ms`, `100%`) → medições e fórmulas reais |
| AUD-41 | FEITO | `573344b` | `tools/verificar_specs.py` | `[DOC]` auditoria manual de fontes | `python tools/verificar_specs.py` → `0 erros` (`19 arquivos atualizados`) | 7 `PRIMARY_OFFICIAL` sem link / 404 / homepages / reissues → `UNKNOWN` / `SECONDARY` / `[link 404 removido]` |
| AUD-42 | FEITO | `4f1c3f9` | `tools/verificar_specs.py` | `verificar_specs.py` → `2 citações inválidas` (`server.rs:2992`, `main.ts:138`) | `python tools/verificar_specs.py` → `92 specs, 173 citações, 0 inválidas` | 62 links `file:///g:/` → relativos; `083`/`026`/`029`/`README` alinhados |
| AUD-43 | FEITO | `c339bf5` | Inspeção `README.md` | N/A (`[DOC]`) | `README.md` atualizado com CC BY-NC 4.0 + GPLv3 VST3 não distribuídos | aviso genérico → texto exato de licença |
| AUD-44 | FEITO | `b5b6e52` | `cargo build -p sonix-plugin --locked` | `cargo build -p sonix-plugin --locked` → erro sem `rev` fixado | `cargo build -p sonix-plugin --locked` → `Finished dev profile` | `nih_plug` sem `rev` → `rev = "de421011f41a6d10fc8c7a6084e4f4dee0143683"` |
| AUD-45 | FEITO | `f36153e`, `2a4cc42` | `contract_053_aud45_biquad_golden::aud_45_biquad_unificado_bit_a_bit_identico`, `contract_044`, `cargo tree -p sonix-plugin \| Select-String cpal` | `sonix-app` sem feature `host-audio` falhou build (`E0432`/`E0599`) antes de `2a4cc42`; `aud_45` é teste dourado bit-a-bit idêntico (passou em `f36153e^` e `HEAD`) | `aud_45_biquad_unificado_bit_a_bit_identico` → `1 passed` (bit-idêntico); `cargo tree -p sonix-plugin` sem `cpal`; `sonix-app` release ok | `cpal` acoplado + 6 `Biquad` duplicados → `host-audio` opcional + `ampforge_dsp::biquad` |
| AUD-46 | FEITO | `ecc3246` | `contract_053_aud45_biquad_golden`, `contract_039_amps_fidelity`, `bancada_auditoria` | `aa_filters` possuía 10 slots identidade ociosos (`[BiquadFilter; 14]`) | `aud_45` → `1 passed` (bit-idêntico); `contract_039` → `10 passed` | `aa_filters: [BiquadFilter; 14]` → `[BiquadFilter; 4]` |
| AUD-47 | FEITO | `a95f6b0` | Inspeção `docs/ui-desktop/inventario-controles.md` | N/A (`[DOC]`) | `docs/ui-desktop/inventario-controles.md:15` atualizado | descrição 1176LN sem ressalva → aproximação comportamental + link manual UA |
| Bloqueios aceitos | BLOQUEIO ACEITO 1/2/3 | — | `018-CA-10.md`, `022-CA-14.md` | — | — | `CA-36`/ASIO (`CPAL_ASIO_DIR`), controlador MIDI/interface USB para desconexão, medição física loopback (`T-25`) e `pluginval`/`clap-validator` |

### AUD-08 — Ganho de pequeno sinal por modelo (`regua::ganho_db`, −80 dBFS @ 1 kHz, valores padrão)

| Modelo | Antes (`4b6ea3d`, dB) | Depois (HEAD, dB) |
|---|---:|---:|
| `plexi_1959` (`BritPlexi1959`) | `7.3837` | `26.9073` |
| `jcm800_2203` (`BritJcm800_2203`) | `7.7173` | `-1.4121` |
| `twin_ab763` (`UsTwinBlackface65`) | `-20.5707` | `-19.7690` |
| `dual_rect` (`CaliDualRectoRevF`) | `30.3326` | `7.2944` |
| `ac30_tb` (`UkAc30TopBoost`) | `-0.7704` | `-5.0063` |

## 2. Execução final do portão (§2.3 literal + COR-13)

| Comando literal (§2.3) | Aprovados | Reprovados | Ignorados | Resultado |
|---|---:|---:|---:|---|
| `cargo test --workspace --locked --no-fail-fast` (execução 1/3) | 598 | 0 | 0 | **exit 0** (inclui `contract_013::test_ca_02` sem `--skip`) |
| `cargo test --workspace --locked --no-fail-fast` (execução 2/3) | 598 | 0 | 0 | **exit 0** |
| `cargo test --workspace --locked --no-fail-fast` (execução 3/3) | 598 | 0 | 0 | **exit 0** |
| `cargo test -p sonix-desktop --locked --no-fail-fast --features fault-injection` | 236 | 0 | 0 | **exit 0** |
| `npm test` (`node --test tests/contract/`) | 133 | 0 | 0 | **exit 0** (`30 suites, 133 tests, 133 pass, 0 fail`) |
| `node --test tests/contract/048-*.test.js` | 3 | 0 | 0 | **exit 0** (`Main FULL=0.8999`, `Main_stage=0.8512`, `Audio_title=0.8769`, `Audio_body=0.8609`, `Audio FULL=0.8643`) |
| `npm run test:e2e` | — | 0 | 0 | **exit 0** (`E2E 033 GREEN: 6 telas sem scroll em 1280x800/1920x1080, Ctrl+1..5/Ctrl+,, e Plexi jumper OK`) |
| `npm --prefix apps/sonix-app/ui run build` | — | 0 | 0 | **exit 0** (`built in 1.38s`) |
| `python tools/verificar_specs.py` | 92 | 0 | 0 | **exit 0** (`92 specs, 173 citações válidas, 0 inválidas`) |
| `cargo build --release -p sonix-app --features custom-protocol` | — | 0 | 0 | **exit 0** |
| `cargo build --release -p sonix-desktop` | — | 0 | 0 | **exit 0** |
| `cargo build --release -p sonix-plugin` | — | 0 | 0 | **exit 0** |
| `git status --porcelain` | — | — | — | Limpo (antes do commit documental `COR-13`) |

- **AUD-03 (20 + 20):** `contract_015_server_hardening::test_ca_05` = `20/20`; `contract_019_input_calibration::test_ca_07` = `20/20`.
- **COR-03 (20× sob `cargo build --release -p sonix-desktop`):** `auditoria_2026_11_20_tempo_real` = `20/20`.
- **COR-04 (`aud_11_rampa_termina_em_25ms`):** no `aud_11_rampa_termina_em_25ms` os valores `0,5 <-> 9,5` **ficam**: com `0`, a saída é silêncio e o ganho em dB não é definido.

## 3. Bancada (§2.4 + COR-06 + COR-11 + COR-12)

Executada via `cargo run --release --locked -p sonix-desktop --example bancada_auditoria -- <cenario> <sr> <bloco>` em 3 execuções por cenário (todas com `"prioridade": "tempo_real"`, COR-11):

### 3.1 Medições dos 5 cenários (3 execuções + mediana)

| Cenário (`sr`/`bloco`) | Run | `media_pct` | `p99_pct` | `p999_pct` | `max_pct` | `blocos_acima_50` | `blocos_acima_100` | `ruido` `blocos_acima_100` (`media_pct` / `p99_pct`) | `excesso_sobre_ruido` | `subnormais` | `prioridade` |
|---|---|---:|---:|---:|---:|---:|---:|---|---:|---:|---|
| `padrao` `48000`/`128` | 1 | `6.855%` | `16.253%` | `87.161%` | `103.290%` | 17 | 1 | 0 (`1.765%` / `1.819%`) | 1 | 0 | `tempo_real` |
| `padrao` `48000`/`128` | 2 | `6.992%` | `19.545%` | `85.459%` | `86.449%` | 23 | 0 | 0 (`1.690%` / `1.691%`) | 0 | 0 | `tempo_real` |
| `padrao` `48000`/`128` | 3 | `6.900%` | `18.030%` | `84.675%` | `84.994%` | 19 | 0 | 0 (`1.712%` / `1.691%`) | 0 | 0 | `tempo_real` |
| **`padrao` `48000`/`128` (mediana)** | **med** | **`6.900%`** | **`18.030%`** | **`85.459%`** | **`86.449%`** | **19** | **0** | **0 (`1.712%` / `1.691%`)** | **0** | **0** | **`tempo_real`** |
| `pesada` `48000`/`128` | 1 | `17.334%` | `72.098%` | `102.476%` | `110.085%` | 51 | 3 | 0 (`1.771%` / `2.453%`) | 3 | 0 | `tempo_real` |
| `pesada` `48000`/`128` | 2 | `17.444%` | `65.704%` | `95.513%` | `96.698%` | 50 | 0 | 0 (`1.723%` / `1.954%`) | 0 | 0 | `tempo_real` |
| `pesada` `48000`/`128` | 3 | `17.760%` | `88.834%` | `100.073%` | `120.296%` | 61 | 3 | 0 (`2.103%` / `1.763%`) | 3 | 0 | `tempo_real` |
| **`pesada` `48000`/`128` (mediana)** | **med** | **`17.444%`** | **`72.098%`** | **`100.073%`** | **`110.085%`** | **51** | **3** | **0 (`1.771%` / `1.954%`)** | **3** | **0** | **`tempo_real`** |
| `pesada` `96000`/`32` | 1 | `42.975%` | `582.330%` | `718.140%` | `866.820%` | 978 | 405 | 53 (`14.271%` / `16.380%`) | 352 | 0 | `tempo_real` |
| `pesada` `96000`/`32` | 2 | `38.787%` | `90.600%` | `673.380%` | `850.290%` | 871 | 283 | 163 (`16.567%` / `18.540%`) | 120 | 0 | `tempo_real` |
| `pesada` `96000`/`32` | 3 | `37.942%` | `78.540%` | `665.550%` | `792.210%` | 898 | 254 | 58 (`14.448%` / `15.300%`) | 196 | 0 | `tempo_real` |
| **`pesada` `96000`/`32` (mediana)** | **med** | **`38.787%`** | **`90.600%`** | **`673.380%`** | **`850.290%`** | **898** | **283** | **58 (`14.448%` / `16.380%`)** | **196** | **0** | **`tempo_real`** |
| `pitch` `48000`/`64` | 1 | `16.751%` | `169.313%` | `190.950%` | `225.390%` | 156 | 148 | 35 (`4.119%` / `4.223%`) | 113 | 0 | `tempo_real` |
| `pitch` `48000`/`64` | 2 | `15.162%` | `63.578%` | `173.648%` | `206.888%` | 92 | 70 | 3 (`3.464%` / `3.383%`) | 67 | 0 | `tempo_real` |
| `pitch` `48000`/`64` | 3 | `15.389%` | `155.108%` | `186.533%` | `215.715%` | 100 | 90 | 5 (`3.554%` / `3.953%`) | 85 | 0 | `tempo_real` |
| **`pitch` `48000`/`64` (mediana)** | **med** | **`15.389%`** | **`155.108%`** | **`186.533%`** | **`215.715%`** | **100** | **90** | **5 (`3.554%` / `3.953%`)** | **85** | **0** | **`tempo_real`** |
| `silencio` `48000`/`128` | 1 | `11.277%` | `85.024%` | `92.925%` | `116.861%` | 1286 | 21 | 0 (`1.807%` / `2.205%`) | 21 | 0 | `tempo_real` |
| `silencio` `48000`/`128` | 2 | `11.436%` | `85.770%` | `93.915%` | `112.050%` | 1319 | 16 | 0 (`1.834%` / `2.033%`) | 16 | 0 | `tempo_real` |
| `silencio` `48000`/`128` | 3 | `12.056%` | `87.780%` | `95.351%` | `115.714%` | 2162 | 25 | 0 (`2.020%` / `2.318%`) | 25 | 0 | `tempo_real` |
| **`silencio` `48000`/`128` (mediana)** | **med** | **`11.436%`** | **`85.770%`** | **`93.915%`** | **`115.714%`** | **1319** | **21** | **0 (`1.834%` / `2.205%`)** | **21** | **0** | **`tempo_real`** |

- **AUD-02 (`silencio`):** `razao_ultima_sobre_primeira` nas 3 execuções: `1.031`, `1.043`, `1.100` (**mediana `1.043 <= 1.5`**), `subnormais: 0`.
- **AUD-12 (`pitch` 48k/64):** `p99_pct` mediana = `155.108%` (`2068.1 µs`, vs `38.378%` antes da promoção de prioridade / sob carga).

### 3.2 Medição de regressão de CPU por fase e commit a commit (COR-07)

Mediana de 3 execuções em `worktree` separado (`sr = 48000`, `bloco = 128`):

| Commit | Fase | `padrao` (mediana) | Variação vs `8cfd49a` | `pesada` (mediana) | Variação vs `8cfd49a` |
|---|---|---:|---:|---:|---:|
| `8cfd49a` | Base AUD-02 | `7.225%` | `0.0%` | `23.243%` | `0.0%` |
| `949560d` | Fase 1 | `6.968%` | `-3.6%` | `20.360%` | `-12.4%` |
| `b150165` | Fase 2 | `7.588%` | `+5.0%` | `19.779%` | `-14.9%` |
| `9505d67` | Fase 3 | `6.268%` | `-13.2%` | `20.078%` | `-13.6%` |
| `2ab27d2` | Fase 4 | `7.329%` | `+1.4%` | `18.021%` | `-22.5%` |
| `c095198` | Fase 5 (**salto >10%**) | `9.121%` | `+26.2%` | `25.438%` | `+9.4%` (`+41.2%` vs `2ab27d2`) |
| `e11912c` | Fase 6 | `9.297%` | `+28.7%` | `25.623%` | `+10.2%` |
| `c20cc65` | Fase 7 | `8.731%` | `+20.8%` | `25.710%` | `+10.6%` |
| `ecc3246` | Fase 9 | `8.982%` | `+24.3%` | `25.833%` | `+11.1%` |
| `72fe513` | Pós COR-07 | **`8.203%`** | **`+13.5%`** | **`17.976%`** | **`-22.7%` (`0.77×`)** |

**Commit a commit dentro da Fase 5 (`2ab27d2..c095198`):**
- `48e26fb`: `padrao = 6.625%`, `pesada = 17.986%`
- `08e6d3c`: `padrao = 6.215%`, `pesada = 18.403%`
- `6871ac8`: `padrao = 6.408%`, `pesada = 18.884%`
- `5a5992d`: `padrao = 6.484%`, `pesada = 18.251%`
- `d90db79`: `padrao = 6.076%`, `pesada = 18.303%`
- `24a92cb`: `padrao = 6.776%`, `pesada = 18.965%`
- `bf80361`: `padrao = 7.619%`, `pesada = 19.138%`
- **`6913cd5` (AUD-29 — responsável em `pesada`):** `padrao = 7.060%`, `pesada = 24.067%` (`+4.93 pp`, `powf(0.125)` e `flush_f64` 16× em `DriveBiquad64`)
- `942fee8`: `padrao = 6.734%`, `pesada = 23.599%`
- `1d6dcb8`: `padrao = 6.798%`, `pesada = 24.190%`
- **`9265cc4` (AUD-32 — responsável em `padrao` e `pesada`):** `padrao = 9.342%` (`+2.54 pp`, FIR 384 taps com desvio circular por tap), `pesada = 25.752%` (`+1.56 pp`)
- `c095198`: `padrao = 8.809%`, `pesada = 26.594%`

## 4. Decisões `DECISAO_SONIX_` criadas

- `DECISAO_SONIX_TWIN_RT60 = 3.0` (`crates/ampforge-dsp/src/amp.rs:552`) — calibração de RT60 da mola do Twin AB763; confirmável por medição acústica de tanque Accutronics 4AB3C1B real.
- `DECISAO_SONIX_PREAMP_STAGE_NORM = 10.0` (`crates/ampforge-dsp/src/amp.rs:729`) — normalização grade-placa entre estágios; confirmável por simulação nodal SPICE/WDF completa.
- `DECISAO_SONIX_RECTO_STAGE_GAIN` / `DECISAO_SONIX_RECTO_MODE_GAIN` / `DECISAO_SONIX_AC30_BRILL_GAIN` (`crates/ampforge-dsp/src/amp.rs:734-757`) — ganhos nominais por estágio; confirmáveis por esquema + medição em bancada.
- `DECISAO_SONIX_RECTO_*` (`crates/ampforge-dsp/src/tonestack.rs:36`) — parâmetros do tone stack Rectifier Rev F; confirmáveis pelo esquema oficial Mesa Rev F.
- `DECISAO_SONIX_MICS_CURVE` / `DECISAO_SONIX_SPEAKER_*` (`crates/ampforge-dsp/src/cab.rs`) — pontos de 1/3 oitava de falantes e microfones; confirmáveis por medição anecoica ou datasheet vetorial completo.
- `DECISAO_SONIX_TEMPERAMENTO_*` (`crates/ampforge-dsp/src/tuner.rs:407-414`) — offsets de temperamento do afinador; confirmáveis por tabela de referência de luthieria.
- `DECISAO_SONIX_RE201_*` (`crates/ampforge-dsp/src/fx/delay.rs:6-14`) — tabela de pares de cabeças (modos 8..11) e profundidade de wow (3 cents); confirmáveis por medição em Roland RE-201 físico.
- `DECISAO_SONIX_LM308_GBW` / `DECISAO_SONIX_VOLTS_POR_FS` (`crates/ampforge-dsp/src/fx/drive.rs:7`) — conversão `FS <-> Volts` e largura de banda/slew LM308; confirmáveis por calibração com pedais físicos TS808/RAT/DS-1.
- `DECISAO_SONIX_LDR_ATTACK_MS = 5.0`, `DECISAO_SONIX_LDR_RELEASE_MS = 60.0`, `DECISAO_SONIX_ROTARY_*` (`crates/ampforge-dsp/src/fx/modulation.rs:4-11`) — constantes temporais LDR e velocidades Horn/Drum Leslie 147; confirmáveis por medição em LDR CdS e Leslie 147 físicos.
- `DECISAO_SONIX_CC_*` (`crates/sonix-desktop/src/midi.rs:8`) — mapeamento padrão de MIDI CC; confirmável por definição de mapa de fábrica.

## 5. Objeções

- **`Main_stage` (Spec 048):** `SSIM = 0.8512` frente ao limiar `0.85` (margem de `+0.0012`), sem violar o critério.
- **`aud_15_saida_igual_a_convolucao_direta` (`cab.set_hicut_khz(18.0)`):** o `c095198` acrescentou `cab.set_hicut_khz(18.0)` porque o `AUD-32` (`9265cc4`) passou a aplicar `apply_hicut_output` após a convolução de IR, e o padrão do `VirtualCabProcessor::new()` (`hicut_khz = 12.0`) filtrava a saída contra a convolução direta sem hi-cut. Em `crates/ampforge-dsp/src/cab.rs:1244`, `if fc >= 17990.0 { return; }` faz bypass total do filtro hi-cut quando `hicut_khz = 18.0` (`18000 Hz >= 17990 Hz`), logo a alteração introduz diferença exata `0.0` (`< 1e-5`).
- **`aud_11_rampa_termina_em_25ms`:** no `aud_11_rampa_termina_em_25ms` os valores `0,5 <-> 9,5` **ficam**: com `0`, a saída é silêncio e o ganho em dB não é definido.
- **COR-07 (`padrao` 48k/128):**
  - Em `padrao` 48k/128, a mediana após o COR-07 era `8.203%` (`218.7 µs`) contra `7.225%` (`192.7 µs`) em `8cfd49a` (`1.135×`, `+26.0 µs/bloco`). O custo decorre da FIR de fase mínima de 384 taps do falante exigida pelo `AUD-32` (`9265cc4`), mesmo após janela contígua de histórico duplo (`step_speaker_fir`). Após `COR-11` (`tempo_real`) + `COR-12`, a mediana de `padrao` 48k/128 caiu para `6.900%` (`183.991 µs/bloco`, `excesso_sobre_ruido = 0`).

## 6. `git log --oneline main..fix/auditoria-2026-11-20`

```text
5d936cc perf(COR-12): remove dead interp_hist shift and gate model-specific amp.rs setup
724b837 perf(COR-12): SSE2 intrinsics in cab.rs accumulate_partition and skip identity voicing filters in amp.rs
b312628 perf(COR-12): 4-way y_head accumulator in cab.rs and coeff caching in delay/modulation
21387cd perf(COR-12): SoA spectrum layout in cab.rs, AdaaTriodeState cache in amp.rs, 0dB skip in eq.rs
e9487f8 perf(COR-12): cache VirtualCabProcessor no-IR mic/filter coeffs and use mask in delay line
c701948 perf(COR-12): cache amp.rs subblock coefficients and reuse adaa_power_f_prev
f14f23c perf(COR-12): hoist amp.rs constant calculations out of sample loop and skip settled gain_smoother sub-blocks
22520c7 feat(COR-11): enable cpal/realtime in host-audio and promote bancada thread to real-time
66473d3 fix(COR-10): revert drive 96k oversampling and DriveBiquad64 guards; real golden fixtures
61d79ec test(COR-09): cor_07_zero_alloc counts allocations per thread
4e44d14 docs(handoff): ordem corretiva 2 da auditoria 2026-11-20
746943e docs(COR-08): rewrite execution report with retroactive red, 5-scenario bancada, process deviations, and update AGENTS.md
72fe513 perf(COR-07): optimize cab FIR/FFT, drive soft_clip_8/96k OS, and amp hoisted exp
9ab9bb8 feat(COR-06): add blocos_acima_50, blocos_acima_100, p999_pct and excesso_sobre_ruido to bancada_auditoria
ac321d3 docs(COR-05): replace unverified White-Box/fidelity claims in README with test-backed descriptions
909d49c test(COR-04): use 0..10 steps for amp.master and amp.volume_2 in aud_11_degrau_sem_zipper
f18c676 test(COR-02): include enumerated devices details in contract_013::test_ca_02 failure message
f59927e fix(COR-01): update wrappers 009/018, spec 018 CA-6 status, and add wrapper lint
2becedd docs(handoff): ordem corretiva da entrega da auditoria 2026-11-20
e26c7e9 docs(auditoria): update AGENTS.md current state and add final execution report
2a4cc42 fix(aud-45): enable host-audio feature on sonix-desktop in sonix-app
a95f6b0 docs(aud-47): document 1176LN behavioral detector/input note and UA manual link in inventory
ecc3246 perf(aud-46): shrink aa_filters from 14 to 4 slots (0 dead identity biquads)
f36153e refactor(aud-45): gate cpal behind host-audio feature and unify BiquadFilter in ampforge_dsp::biquad
b5b6e52 build(aud-44): pin nih_plug git rev in sonix-plugin/Cargo.toml and Cargo.lock
c339bf5 docs(aud-43): add concise CC BY-NC 4.0 and GPLv3 VST3 license notice in README
4f1c3f9 docs(aud-42): convert file:///g:/ links to relative, fix 083/026/029/README and citations
573344b docs(aud-41): audit source classifications, mark inaccessible/image-only/homepage links, remove 404
4f9eed5 docs(aud-40): remove MixedPrecisionSolver and align public claims in README, AGENTS.md, and inventory
9087fe4 fix(aud-39): align spec 028/040/053 statuses, disable exclusive mode in UI, add block-size invariance test
3e8c340 docs(aud-38): fix fabricated identifiers, add front-matter and citations in specs 053-092, add verificar_specs.py
c20cc65 test(aud-37): add Audio screen DOM & SSIM fidelity checks to 048
5d949b7 test(aud-36): add missing CA tests, manual verification docs, and npm run test:e2e
3218d31 fix(aud-35): replace tautological tests, decouple IPC fixture generator, and align legacy UI coverage
e11912c fix(fase-6-gate): prioriza janela recente de 4096 em transicoes de nota >=65 Hz
340acfa fix(AUD-34): afinador polifonico FFT Hann 4x zero-padding e exclusao de parciais colidentes
b6f99f1 fix(AUD-33): afinador 38-1400 Hz, k*tau >330 Hz +-0.1 cent, tuner_scratch 8192 e nomes SONIX
c095198 fix(fase-5-gate): visibilidade interna AUD-27, hicut 18k em aud_15 e headroom de valvula/bias
9265cc4 fix(AUD-32): hi-cut pos-IR, seletor de falante desabilitado com IR e curvas 1/3 oitava FIR fase minima
1d6dcb8 fix(AUD-31): nomes Freeverb (Jezar), dispersao Tube Spring >= 5 ms e RT60 Twin 3.0s
942fee8 fix(AUD-30): TZF 1ms, Rotary dois rotores LR4, optico LDR 5/60ms, Chorus 3 vozes mono e AC30 Vib-Trem
6913cd5 fix(AUD-29): escala VOLTS_POR_FS, ganho do esquema no TS808 e RAT com GBW LM308 e clip simetrico no DS-1
bf80361 fix(AUD-28): tabela unica de 12 modos do RE-201, mola TubeSpringReverb em paralelo e wow de 3 cents
24a92cb fix(AUD-27): rede 470k||470pF de 1a ordem no JCM800 e bright cap 5000pF no Volume I do Plexi
d90db79 fix(AUD-26): Middle Marshall usa audio_taper 10pct log direto e remove map_pot_mid_10pct
5a5992d fix(AUD-25): corrige sentido do seletor de impedancia (4 > 8 > 16 Ohm) conforme Tonal Note 3
6871ac8 fix(AUD-24): entrada LOW do JCM800 pula V1a e expoe ganho_estagio1_pequeno_sinal
08e6d3c fix(AUD-23): fidelidade dos amps (Twin sem scoop duplo, Vox Top Boost nodal, Recto pos-ganho, Presence Modern, sem Pushed)
48e26fb fix(AUD-22): corrige matriz nodal do tone stack FMV/AB763 e valida contra Yeh & Smith
2ab27d2 chore(fase-4): atualiza biblioteca-sem-uso-em-producao e fixture presets_load no portao da Fase 4
37283fb fix(AUD-21): wah explicito no grafo/grid e linha 2 inativa com split desligado
c85e8cf fix(AUD-20): split balance sem reset, parametros dinamicos e crossover LR4
a3f00dc fix(AUD-19): pacote .sforgepkg v2 em base64 com limites e comandos Tauri/UI
8311536 fix(AUD-18): persistencia atomica de calibracao em disco por dispositivo
35ab95d fix(AUD-17): host MIDI real via midir e MidiRouter ligado ao runtime Tauri
7242281 fix(AUD-16): plugin VST3/CLAP real via SonixPluginProcessor::process_block
9505d67 chore(gate-3): ajustes de visibilidade e fixtures pos-Fase 3
485d19b fix(AUD-15): Real FFT 256 (129 bins) e MACs distribuidos na convolucao particionada
3508261 fix(AUD-14): filtros elipticos de 8a ordem para oversampling 2x/4x/8x
f75e73d fix(AUD-13): medicao real de latencia por impulso em prepare() e suporte a oversampling 1x
a9c28f9 chore(AUD-12): atualizar schema fixture audio_status.json com pitch_wet_latency_ms
b53d833 fix(AUD-12): pitch com seco sem atraso, wet centralizado em 512 amostras e autocorrelacao amortizada
bab7c5a fix(AUD-11): suavizacao linear de 20ms com sub-blocos em controles de tom, cab, eq e wah
bfc41dc fix(AUD-10): crossfade de potencia constante com duas instancias completas na troca de modelo de amp
b150165 fix(AUD-09): centralizar tabela unica de parametros em params.rs e gerar params.generated.ts
f6a0d50 fix(AUD-08): remover controles ocultos por modelo, limpar presets e corrigir estagios reais
2f9c699 test(AUD-05): atualizar fixtures de schema IPC com campos de falha e estado de dispositivo
949560d fix(AUD-07): substituir forget e unpark por pendentes, backpressure e park_timeout no reaper
5922ecc fix(AUD-06): eliminar contencao no mutex da cadeia DSP e respeitar mute/standby no fallback
5e01959 fix(AUD-05): tratar erros de stream cpal com recuperacao automatica e faixa na UI
c079a4c fix(AUD-04): capturar panicos no callback DSP, remover unwraps da API e ativar panic=unwind
813120f fix(AUD-03): drenar socket antes de fechar em erros e retornar 411 sem Content-Length
8cfd49a fix(AUD-02): eliminar subnormais com DenormalGuard, flush DSP, regua e bancada
2c2ac38 fix(AUD-01): endurecer CSRF, Host, rotas e persistencia de reamp
```

## 7. Desvios de processo registrados

Sem reescrever o histórico (`§0.3`), registram-se os desvios da rodada `8f499ac..e26c7e9`:
1. **4 commits de portão que misturam itens:** `9505d67` (`chore(gate-3)`), `2ab27d2` (`chore(fase-4)`), `c095198` (`fix(fase-5-gate)`) e `e11912c` (`fix(fase-6-gate)`).
2. **46 commits sem corpo (sem comando e resultado numérico):** `5e01959`, `5922ecc`, `949560d`, `2f9c699`, `a9c28f9`, `f75e73d`, `3508261`, `485d19b`, `9505d67`, `7242281`, `35ab95d`, `8311536`, `a3f00dc`, `c85e8cf`, `37283fb`, `2ab27d2`, `48e26fb`, `08e6d3c`, `6871ac8`, `5a5992d`, `d90db79`, `24a92cb`, `bf80361`, `6913cd5`, `942fee8`, `1d6dcb8`, `9265cc4`, `c095198`, `b6f99f1`, `340acfa`, `e11912c`, `3218d31`, `5d949b7`, `c20cc65`, `3e8c340`, `9087fe4`, `4f9eed5`, `573344b`, `4f1c3f9`, `c339bf5`, `b5b6e52`, `f36153e`, `ecc3246`, `a95f6b0`, `2a4cc42`, `e26c7e9`.

## 8. Correções COR

| COR | Estado | Commits | Testes (nomes) | Red (comando + trecho) | Green (comando + trecho) | Medição antes → depois |
|---|---|---|---|---|---|---|
| COR-01 | FEITO | `f59927e` | `auditoria-2026-11-20-wrappers.test.js` (`'COR-01: todo teste Rust exigido por um wrapper existe'`) | `node --test tests/contract/auditoria-2026-11-20-wrappers.test.js` → `3 !== 0` (`009:20 test_ca_04_gapless_scene_transition_cosine_crossfade_1_5ms`, `018:24 test_ca_04_negotiated_sample_rate_and_graceful_mismatch_fallback`, `018:26 test_ca_06_backend_fallback_cascade_without_panic`) | `npm test` → `133 tests, 30 suites, 133 pass, 0 fail` | `130/132` → `133/133` + Spec 018 CA-6 `NÃO ATENDIDO` |
| COR-02 | FEITO | `f18c676` | `contract_013_desktop_realtime_audio::test_ca_02_physical_audio_device_enumeration_wasapi_default_duplex` | Mensagem sem listagem de dispositivos | `cargo test --workspace --locked --no-fail-fast` → `576 passed; 0 failed` (`contract_013::test_ca_02` aprovado sem `--skip`) | "bloqueio aceito 2" removido; falha agora lista `nome`, `is_input`, `supported_sample_rates`, `supported_buffer_sizes` |
| COR-03 | FEITO | — | `auditoria_2026_11_20_tempo_real::aud_05_erro_do_stream_marca_perda` | Falha prévia registrada (`354.25 ms` > `200 ms`) | 20× seguidas com `cargo build --release -p sonix-desktop` em paralelo → `PASS: 20/20` | `354.25 ms` intermitente → `20/20` dentro de `200 ms` |
| COR-04 | FEITO | `909d49c` | `auditoria_2026_11_20_artefatos::aud_11_degrau_sem_zipper` | `0.5 <-> 9.5` em `amp.master` e `amp.volume_2` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_artefatos aud_11` → `3 passed; 0 failed` | `0.5 <-> 9.5` → `0 <-> 10` em `aud_11_degrau_sem_zipper` (`0.5 <-> 9.5` mantidos em `aud_11_rampa_termina_em_25ms`) |
| COR-05 | FEITO | `ac321d3` | Inspeção `README.md` + `tools/verificar_specs.py` | `README.md:9` afirmava "modelagem White-Box (100% algorítmica em nível de circuito analógico discreto)" | `README.md` atualizado com testes exatos (`aud_22`, `aud_23..29`, `contract_039`), link para seção 4 (`DECISAO_SONIX_*`) e nota comportamental do 1176LN | adjetivos sem medição → afirmações verificáveis |
| COR-06 | FEITO | `9ab9bb8` | `bancada_auditoria` (5 cenários × 3 execuções) | `bancada_auditoria` só emitia média/p99/máx e rodava `padrao`/`ruido` | JSON emite `blocos_acima_50`, `blocos_acima_100`, `p999_pct`, `excesso_sobre_ruido` com `ruido` pareado no mesmo nº de blocos | 2 cenários → 5 cenários × 3 execuções + mediana |
| COR-07 | FEITO (custo), substituído pelo COR-10 (fidelidade) | `72fe513` | `cor_07_regressao_cpu` (no `746943e`, antes do `COR-09`), `contract_053_aud45_biquad_golden` | `padrao 48k/128 = 8.8–9.1%`, `pesada 48k/128 = 25.4–26.0%`, `pesada 96k/32 = 54.491%` (`p99 = 222.69%`, `excesso = 639`) | Na revisão de `746943e`: `cargo test --workspace` → `575 passed; 1 failed` (`cor_07_zero_alloc` `left: 8, right: 0`, intermitente `6/10`); `padrao 48k/128 = 8.203%`; `pesada 48k/128 = 17.976%` (`0.77×`); `pesada 96k/32 = 35.064%` (`excesso = 163`) | Ver tabela §3.2 e §5 (`COR-09` corrigiu `cor_07_zero_alloc`; `COR-10` substituiu os dourados de drive/amp/cab) |
| COR-08 | FEITO | `746943e` | Inspeção `RELATORIO-auditoria-2026-11-20-execucao.md`, `AGENTS.md` | Relatório inicial sem red retroativo, sem 5 cenários e sem seção 7/8 | Reescrita das seções 1–8 + atualização de `AGENTS.md` | — |
| COR-09 | FEITO | `61d79ec` | `cor_07_regressao_cpu::cor_07_zero_alloc` | 10 execuções de `cor_07_regressao_cpu` em `746943e` → `6/10` (`4` falhas `left: 8, right: 0` por contador global) | 50/50 execuções seguidas de `cor_07_regressao_cpu` (`4 passed; 0 failed` cada); injeção de `Vec::with_capacity(1)` em `VirtualCabProcessor::process` → `FAILED` (`left: 2000, right: 0`, revertida) | contador global → `thread_local! Cell` (alocações e desalocações separadas, sem `TEST_SERIAL`) |
| COR-10 | FEITO | `66473d3` | `cor_10_dourados::cor_10_*` (24 testes), `cor_07_zero_alloc`, `contract_053_aud45` | `cargo test -p ampforge-dsp --test cor_10_dourados` sobre `746943e` → `18 passed; 6 failed` (todos os 6 drives a 96 kHz: `ts808` `2.70e-3 > 2.74e-7`, `muff` `2.13e-2 > 3.50e-7`, etc.) | `cargo test -p ampforge-dsp --locked` → `cor_10_dourados` `24 passed; 0 failed` (`max\|y-y_dourado\| <= 1e-6 * max\|y_dourado\|`); zero ocorrências de `sample_rate > 64_000` em `drive.rs` | drives 96 kHz revertidos para 8× OS + `flush_f64`/`is_finite` restaurados em `DriveBiquad64`; 24 fixtures em `tests/fixtures/cor-10/` |
| COR-11 | FEITO | `22520c7` | `cor_11_realtime_priority::cor_11_host_audio_ativa_realtime`, `cargo tree -p sonix-plugin \| Select-String cpal` | `cor_11_host_audio_ativa_realtime` → `FAILED` (`host-audio = ["dep:cpal"]`) | `cor_11` → `1 passed; 0 failed`; `cargo tree -p sonix-plugin` sem `cpal` (`0`); `bancada_auditoria` emite `"prioridade":"tempo_real"` em todos os cenários; `sonix-desktop` release `exit 0` | prioridade normal → `cpal/realtime` em `host-audio` e `promote_current_thread_to_real_time` na bancada |
| COR-12 | **NÃO ATENDIDO** | `f14f23c`, `c701948`, `e9487f8`, `21387cd`, `b312628`, `724b837`, `5d936cc` | `cor_10_dourados` (24/24), `cor_07_zero_alloc` (1/1), `bancada_auditoria pesada 96000 32` | Baseline pós COR-10 + COR-11 (`22520c7`): `media_us = 141.274` (`42.382%`), `excesso_sobre_ruido = 206` | `cor_10_dourados` `24/24` e `cor_07_zero_alloc` `1/1` verdes após cada commit; melhor mediana (`724b837`): `118.105 µs/bloco` (`35.432%`), `excesso_sobre_ruido = 23` (`26, 23, -17`); suite 5×3 final (`5d936cc`): mediana `129.289 µs/bloco` (`38.787%`), `excesso_sobre_ruido = 196` (`352, 120, 196`) | `141.274` → `118.105`–`129.289 µs/bloco`; `excesso_sobre_ruido <= 0` não atingido na mediana |
