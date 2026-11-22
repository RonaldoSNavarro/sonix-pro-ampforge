# Relatório de Execução — Auditoria 2026-11-20 (`fix/auditoria-2026-11-20`)

## 1. Tabela por item

| AUD | Estado | Commits | Testes (nomes) | Red (comando + trecho) | Green (comando + trecho) | Medição antes → depois |
|---|---|---|---|---|---|---|
| AUD-01 | FEITO | `2c2ac38` | `auditoria_2026_11_20_seguranca::aud_01_ipc_reamp_recusa_session_path` | `cargo test -p sonix-app --test auditoria_2026_11_20_seguranca` → `FAILED` (aceitava session path) | `cargo test --locked -p sonix-app --test auditoria_2026_11_20_seguranca` → `1 passed; 0 failed` | CSRF/Host/reamp sem validação → validação estrita + recusa session path |
| AUD-02 | FEITO | `8cfd49a` | `auditoria_2026_11_20_tempo_real::aud_02_*` (20 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_tempo_real` → `FAILED` (subnormais no decay/reverb) | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_tempo_real` → `20 passed; 0 failed` | subnormais > 0 → `subnormais: 0` com `DenormalGuard` e `flush_f32`/`flush_f64` |
| AUD-03 | FEITO | `813120f` | `contract_015_server_hardening` | `cargo test -p sonix-desktop --test contract_015_server_hardening` → `FAILED` | `cargo test --locked -p sonix-desktop --test contract_015_server_hardening` → `ok` | fechamento abrupto sem `411` → dreno + `411 Length Required` |
| AUD-04 | FEITO | `c079a4c` | `contract_018_low_latency_host::test_ca_11_dsp_panic_catch_and_stream_resilience` | `cargo test -p sonix-desktop --test contract_018_low_latency_host` → `FAILED` | `cargo test --locked -p sonix-desktop --test contract_018_low_latency_host` → `10 passed; 0 failed` | `panic=abort` / `unwrap()` → `panic=unwind` + `catch_unwind` no callback |
| AUD-05 | FEITO | `5e01959`, `2f9c699` | `auditoria_2026_11_20_tempo_real::aud_05_*`, `033::CA-12` | `cargo test -p sonix-app --test auditoria_2026_11_20_tempo_real` → `FAILED` | `cargo test --locked -p sonix-app --test auditoria_2026_11_20_tempo_real` → `2 passed; 0 failed` | erro `cpal` ignorado → `device_state=lost` + banner na UI + reabertura |
| AUD-06 | FEITO | `5922ecc` | `contract_049_scene_state_and_queues::ca14_*` | `cargo test -p sonix-desktop --test contract_049_scene_state_and_queues` → `FAILED` | `cargo test --locked -p sonix-desktop --test contract_049_scene_state_and_queues` → `12 passed; 0 failed` | contenção no mutex da cadeia → zero contenção + mute/standby respeitados |
| AUD-07 | FEITO | `949560d` | `contract_037_realtime_safety` | `cargo test -p sonix-desktop --test contract_037_realtime_safety` → `FAILED` | `cargo test --locked -p sonix-desktop --test contract_037_realtime_safety` → `ok` | `mem::forget` + `unpark` → fila pendentes + backpressure + `park_timeout` |
| AUD-08 | FEITO | `f6a0d50` | `auditoria_2026_11_20_parametros::aud_08_*`, `auditoria-2026-11-20-parametros.test.js::AUD-08` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_parametros` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_parametros` → `2 passed; 0 failed` | controles ocultos por modelo → `models` declarados e filtrados na tela Main |
| AUD-09 | FEITO | `b150165` | `auditoria-2026-11-20-parametros.test.js::AUD-09` | `node --test tests/contract/auditoria-2026-11-20-parametros.test.js` → `FAILED` | `node --test tests/contract/auditoria-2026-11-20-parametros.test.js` → `4 passed; 0 failed` | tabela duplicada TS/Rust → `PARAMS_TABLE` única em `params.rs` gerando `params.generated.ts` |
| AUD-10 | FEITO | `bfc41dc` | `auditoria_2026_11_20_artefatos::aud_10_*` (3 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_10` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_10` → `3 passed; 0 failed` | troca de modelo com clique → crossfade constante com 2 instâncias completas |
| AUD-11 | FEITO | `bab7c5a` | `auditoria_2026_11_20_artefatos::aud_11_*` (3 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_11` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_11` → `3 passed; 0 failed` | degrau com zipper → rampa linear de 20 ms com sub-blocos |
| AUD-12 | FEITO | `b53d833`, `a9c28f9` | `auditoria_2026_11_20_artefatos::aud_12_*` (4 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_12` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_12` → `4 passed; 0 failed` | seco com atraso no pitch → seco sem atraso, wet 512 amostras |
| AUD-13 | FEITO | `f75e73d` | `contract_035_ui_backend_contracts`, `contract_039_amps_fidelity` | `cargo test -p sonix-desktop --test contract_035_ui_backend_contracts` → `FAILED` | `cargo test --locked -p sonix-desktop --test contract_035_ui_backend_contracts` → `5 passed; 0 failed` | latência fixa → medição por impulso em `prepare()` + suporte 1x |
| AUD-14 | FEITO | `3508261` | `auditoria_2026_11_20_artefatos::aud_14_*` (2 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_14` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_14` → `2 passed; 0 failed` | aliasing alto ganho → elíptico IIR 8ª ordem 2x/4x/8x (`>= 60 dB` rejeição) |
| AUD-15 | FEITO | `485d19b`, `c095198` | `auditoria_2026_11_20_artefatos::aud_15_*` (3 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_15` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_artefatos -- aud_15` → `3 passed; 0 failed` | convolução complexa → Real FFT 256 (`fft_128`, 129 bins) + MACs distribuídos |
| AUD-16 | FEITO | `7242281` | `contract_049_plugins` | `cargo test -p sonix-plugin` → `FAILED` | `cargo test --locked -p sonix-plugin` → `ok` | `sonix-plugin` passthrough → `SonixPluginProcessor::process_block` real |
| AUD-17 | FEITO | `35ab95d` | `contract_036_audio_host_midi_telemetry`, `contract_049_midi_ranges` | `cargo test -p sonix-desktop --test contract_036_audio_host_midi_telemetry` → `FAILED` | `cargo test --locked -p sonix-desktop --test contract_036_audio_host_midi_telemetry` → `ok` | MIDI mock → `midir` + `MidiRouter` em tempo real |
| AUD-18 | FEITO | `8311536` | `auditoria_2026_11_20_persistencia::aud_18_*` (3 testes) | `cargo test -p sonix-app --test auditoria_2026_11_20_persistencia -- aud_18` → `FAILED` | `cargo test --locked -p sonix-app --test auditoria_2026_11_20_persistencia -- aud_18` → `3 passed; 0 failed` | calibração em memória → persistência atômica por dispositivo |
| AUD-19 | FEITO | `a3f00dc` | `auditoria_2026_11_20_persistencia::aud_19_*`, `auditoria_2026_11_20_sforgepkg`, `auditoria-2026-11-20-parametros.test.js::AUD-19` | `cargo test -p sonix-core --test auditoria_2026_11_20_sforgepkg` → `FAILED` | `cargo test --locked -p sonix-core --test auditoria_2026_11_20_sforgepkg` → `ok` | export/import incompleto → `.sforgepkg` v2 base64 com limites |
| AUD-20 | FEITO | `c85e8cf` | `auditoria_2026_11_20_split_dag` | `cargo test -p sonix-desktop --test auditoria_2026_11_20_split_dag` → `FAILED` | `cargo test --locked -p sonix-desktop --test auditoria_2026_11_20_split_dag` → `ok` | split balance resetava → crossover LR4 + split dinâmico parametrizado |
| AUD-21 | FEITO | `37283fb` | `auditoria_2026_11_20_split_dag`, `auditoria-2026-11-20-parametros.test.js::AUD-21` | `node --test tests/contract/auditoria-2026-11-20-parametros.test.js` → `FAILED` | `node --test tests/contract/auditoria-2026-11-20-parametros.test.js` → `4 passed; 0 failed` | wah implícito → bloco `wah_1` explícito na grade + Linha 2 esmaecida |
| AUD-22 | FEITO | `48e26fb` | `contract_039_amps_fidelity::aud_22_stack_igual_a_yeh` | `cargo test -p ampforge-dsp --test contract_039_amps_fidelity -- aud_22` → `FAILED` | `cargo test --locked -p ampforge-dsp --test contract_039_amps_fidelity -- aud_22` → `ok` (`err <= 0.05 dB` analógico, `<= 0.10 dB` digital) | matriz FMV/AB763 divergente → validada contra Yeh & Smith |
| AUD-23 | FEITO | `08e6d3c` | `auditoria_2026_11_20_amps::aud_23*` (4 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_23` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_23` → `4 passed; 0 failed` | scoop duplo Twin / Presence Modern → corrigidos por modelo |
| AUD-24 | FEITO | `6871ac8` | `auditoria_2026_11_20_amps::aud_24*` (2 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_24` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_24` → `2 passed; 0 failed` | LOW JCM800 só atenuava → pula V1a; Plexi LOW `-6 dB` |
| AUD-25 | FEITO | `5a5992d` | `auditoria_2026_11_20_amps::aud_25_*` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_25` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_25` → `ok` | sentido invertido → `4 > 8 > 16 Ohm` conforme Tonal Note 3 |
| AUD-26 | FEITO | `d90db79` | `auditoria_2026_11_20_amps::aud_26_*` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_26` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_26` → `ok` | `map_pot_mid_10pct` duplicado → `audio_taper` 10% log direto |
| AUD-27 | FEITO | `24a92cb`, `c095198` | `auditoria_2026_11_20_amps::aud_27*` (2 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_27` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_amps -- aud_27` → `2 passed; 0 failed` | bright cap e rede 470k\|\|470pF ausentes → 1ª ordem fiel |
| AUD-28 | FEITO | `bf80361` | `auditoria_2026_11_20_efeitos::aud_28_*` (4 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_28` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_28` → `4 passed; 0 failed` | modos 5–12 RE-201 sem mola/invertidos → tabela única 12 modos + mola 5..12 + wow 3 cents |
| AUD-29 | FEITO | `6913cd5` | `auditoria_2026_11_20_efeitos::aud_29_*` (4 testes) | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_29` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_29` → `4 passed; 0 failed` | ganhos/limiares arbitrários → `VOLTS_POR_FS`, ganho TS808/RAT (GBW LM308), DS-1 simétrico |
| AUD-30 | FEITO | `942fee8` | `auditoria_2026_11_20_efeitos::aud_30_*` (4 testes), `contract_040_mod_delay_reverb` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_30` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_30` → `4 passed; 0 failed` | TZF/Rotary/Optical genéricos → TZF 1ms dry + inversão, Rotary 2 rotores LR4 800Hz, LDR 5/60ms, AC30 Vib/Trem |
| AUD-31 | FEITO | `1d6dcb8` | `auditoria_2026_11_20_efeitos::aud_31b_dispersao`, `aud_31c_rt60_twin` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_31` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_31` → `2 passed; 0 failed` | dispersão mola < 5 ms / nomes não documentados → dispersão >= 5 ms, Twin RT60 3.0s, nomes Freeverb (Jezar) |
| AUD-32 | FEITO | `9265cc4` | `auditoria_2026_11_20_efeitos::aud_32_*` (2 testes), `038-cab-ir-ui.test.js` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_32` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_efeitos -- aud_32` → `2 passed; 0 failed` | hi-cut ignorado com IR → hi-cut pós-IR + FIR fase mínima 384 taps + seletor desabilitado com IR |
| AUD-33 | FEITO | `b6f99f1` | `auditoria_2026_11_20_afinador::aud_33_*` (2 testes), `contract_026_live_tuner`, `contract_043_tuner_looper` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_afinador -- aud_33` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_afinador -- aud_33` → `2 passed; 0 failed` | faixa 60 Hz–1.4 kHz / Peterson → 38–1405 Hz, `k*tau` > 330 Hz `+-0.1 cent`, `tuner_scratch = 8192` |
| AUD-34 | FEITO | `340acfa`, `e11912c` | `auditoria_2026_11_20_afinador::aud_34_poli_com_harmonicos`, `contract_051_manual_test_fixes` | `cargo test -p ampforge-dsp --test auditoria_2026_11_20_afinador -- aud_34` → `FAILED` | `cargo test --locked -p ampforge-dsp --test auditoria_2026_11_20_afinador -- aud_34` → `ok` | polifônico sem filtro de harmônicos → FFT Hann 4x zero-padding + exclusão de parciais `+-15 cents` |
| AUD-35 | FEITO | `3218d31` | `contract_039::ca2_amp_resposta_linear_igual_entre_taxas`, `contract_035::test_ca_08`, `contract_049::ca3_009`, `ca4_009`, `auditoria-2026-11-20-ui-legado.test.js` | `ca2_amp_resposta_linear_igual_entre_taxas` → `FAILED` (`1.105 dB` @ 16 kHz) | `cargo test --locked -p ampforge-dsp --test contract_039_amps_fidelity` → `10 passed`; `contract_035` → `5 passed`; `ui-legado` → `4 passed` | `1.105 dB` @ 16 kHz (sinc + elliptic) → `<= 0.5 dB` (zero-stuffing + elliptic); `048-diffs` em `os.tmpdir()` |
| AUD-36 | FEITO | `5d949b7` | `033::ca3_*`, `contract_039::ca5_*`, `contract_044::ca4_*`, `ca15_*`, `contract_047::ca5_*`, `npm run test:e2e` | `ca5_stack_fender_ab763_coincide_com_yeh` → `FAILED` (taper vs linear); `run-033-e2e` timeout → `Plexi 1959` chip | `npm run test:e2e` → `E2E 033 GREEN`; `044` → `9 passed`; `047` → `6 passed`; `033` → `6 passed` | E2E sem executor + 5 CAs sem teste → `npm run test:e2e` + 5 testes `caN_*` + docs manuais `018-CA-10.md`, `022-CA-14.md` |
| AUD-37 | FEITO | `c20cc65` | `048-ui-desktop-fidelidade-visual.test.js` (Main + Audio DOM & SSIM) | `048` Audio_body → `SSIM=0.6878` (aba `dev` vs `Audio.png`) | `048` → `Audio_title=0.8769`, `Audio_body=0.8609`, `Audio FULL=0.8643`, `Main_stage=0.8512` (margem `+0.0012`), `Main FULL=0.8999` | Audio sem SSIM (`0.6878`) → `Audio_body=0.8609`, `Audio FULL=0.8643` |
| AUD-38 | FEITO | `3e8c340` | `tools/verificar_specs.py` | `verificar_specs.py` (pré-limpeza) → identificadores inexistentes/trocados | `python tools/verificar_specs.py` → `92 specs, 173 citações válidas, 0 inválidas, 0 front-matter ausente, 0 IDs inexistentes` | 17 IDs fabricados/trocados e 059..092 sem YAML `status: rascunho` → 0 erros |
| AUD-39 | FEITO | `9087fe4` | `contract_053_auditoria_039::ca4_invariancia_ao_tamanho_de_bloco`, `contract_018`, `contract_028`, `contract_044` | `contract_018`/`028` mantinham testes simulados de modo exclusivo | `contract_053_auditoria_039` → `1 passed` (bit-idêntico `32,37,64,256,480,512,1024` vs `128`); `018` `10 passed`; `028` `6 passed` | 028 inflada como implementada → `parcial`, item UI desabilitado (`não implementado`), `053 CA-4=TESTED`, `CA-9=NÃO ATENDIDO` |
| AUD-40 | FEITO | `4f9eed5` | `contract_001_nucleo_dsp`, `contract_044_sdd_integrity` | N/A (`MixedPrecisionSolver` removido) | `contract_001` → `11 passed`; `contract_044` → `9 passed` | frases contraditas (`Newton-Raphson`, `FIR > 96 dB`, `< 1,5 ms`, `100%`) → medições e fórmulas reais |
| AUD-41 | FEITO | `573344b` | `tools/verificar_specs.py` | auditoria manual de fontes | `python tools/verificar_specs.py` → `0 erros` (`19 arquivos atualizados`) | 7 `PRIMARY_OFFICIAL` sem link / 404 / homepages / reissues → `UNKNOWN` / `SECONDARY` / `[link 404 removido]` |
| AUD-42 | FEITO | `4f1c3f9` | `tools/verificar_specs.py` | `verificar_specs.py` → `2 citações inválidas` (`server.rs:2992`, `main.ts:138`) | `python tools/verificar_specs.py` → `92 specs, 173 citações, 0 inválidas` | 62 links `file:///g:/` → relativos; `083`/`026`/`029`/`README` alinhados |
| AUD-43 | FEITO | `c339bf5` | Inspeção `README.md` | N/A (doc) | `README.md` atualizado com CC BY-NC 4.0 + GPLv3 VST3 não distribuídos | aviso genérico → texto exato de licença |
| AUD-44 | FEITO | `b5b6e52` | `cargo build -p sonix-plugin --locked` | `cargo build -p sonix-plugin --locked` → erro sem `rev` fixado | `cargo build -p sonix-plugin --locked` → `Finished dev profile` | `nih_plug` sem `rev` → `rev = "de421011f41a6d10fc8c7a6084e4f4dee0143683"` |
| AUD-45 | FEITO | `f36153e`, `2a4cc42` | `contract_053_aud45_biquad_golden::aud_45_biquad_unificado_bit_a_bit_identico`, `contract_044`, `cargo tree -p sonix-plugin \| Select-String cpal` | `sonix-app` sem feature `host-audio` falhou build (`E0432`/`E0599`) | `aud_45_biquad_unificado_bit_a_bit_identico` → `1 passed` (bit-idêntico); `cargo tree -p sonix-plugin` sem `cpal`; `sonix-app` release ok | `cpal` acoplado + 6 `Biquad` duplicados → `host-audio` opcional + `ampforge_dsp::biquad` |
| AUD-46 | FEITO | `ecc3246` | `contract_053_aud45_biquad_golden`, `contract_039_amps_fidelity`, `bancada_auditoria` | N/A | `aud_45` → `1 passed` (bit-idêntico); `contract_039` → `10 passed`; `bancada_auditoria` ok | `aa_filters: [BiquadFilter; 14]` (10 identidade ociosos) → `[BiquadFilter; 4]` (FFT real `fft_128` já em produção desde `AUD-15`) |
| AUD-47 | FEITO | `a95f6b0` | Inspeção `docs/ui-desktop/inventario-controles.md` | N/A (doc) | `docs/ui-desktop/inventario-controles.md:15` atualizado | descrição 1176LN sem ressalva → aproximação comportamental + link manual UA |
| Bloqueios aceitos | BLOQUEIO ACEITO 1/2/3 | — | `contract_013::test_ca_02`, `018-CA-10.md`, `022-CA-14.md` | — | — | `CA-36`/ASIO (`CPAL_ASIO_DIR`), WASAPI físico (`contract_013::test_ca_02`), medição física loopback (`T-25`) e `clap-validator`/VST3 SDK `validator` |

## 2. Execução final do portão

- `cargo test --locked -p ampforge-dsp -- --test-threads=1` → aprovado (0 reprovados, 0 ignorados)
- `cargo test --locked -p sonix-core -- --test-threads=1` → aprovado (0 reprovados, 0 ignorados)
- `cargo test --locked -p sonix-plugin -- --test-threads=1` → aprovado (0 reprovados, 0 ignorados)
- `cargo test --locked -p sonix-desktop -- --skip test_ca_02_physical_audio_device_enumeration_wasapi_default_duplex --test-threads=1` → aprovado (0 reprovados; `test_ca_02_physical_audio_device_enumeration_wasapi_default_duplex` pulado por **BLOQUEIO ACEITO 2** — ausência de placa WASAPI com entrada física 44.1k/48k/96k nesta estação)
- `cargo test --locked -p sonix-app -- --test-threads=1` → aprovado (20 aprovados, 0 reprovados, 0 ignorados)
- `cargo build --release -p sonix-app --features custom-protocol --locked` → aprovado (`7m 16s`)
- `cargo build --release -p sonix-desktop --locked` → aprovado (`40.07s`)
- `cargo build --release -p sonix-plugin --locked` → aprovado (`1m 35s`)
- `node --test tests/contract/033-... 038-... 048-... 049-... 050-... 051-... 052-... auditoria-2026-11-20-parametros.test.js auditoria-2026-11-20-ui-legado.test.js` → `51 passed; 0 failed` (`SSIM`: `Main FULL=0.8999`, `Main_stage=0.8512`, `Audio_title=0.8769`, `Audio_body=0.8609`, `Audio FULL=0.8643`)
- `npm run test:e2e` → `E2E 033 GREEN: 6 telas sem scroll em 1280x800/1920x1080, Ctrl+1..5/Ctrl+,, e Plexi jumper OK`
- `python tools/verificar_specs.py` → `Specs verificadas: 92 (001..092)`, `Citações arquivo:linha @ <hash> verificadas: 173`, `Citações inválidas: 0`, `Front-matter status: rascunho ausente (059..092): 0`, `Identificadores inexistentes/trocados como código atual (053..092): 0`
- `git status --porcelain` → limpo antes deste commit documental final.

## 3. Bancada (§2.4)

Executado via `cargo run --release --locked -p sonix-desktop --example bancada_auditoria` (`sr = 48000`, `bloco = 128`, `blocos = 3750`, `orcamento_us = 2666.667`):

| Cenário | `media_pct` | `media_us` | `p99_pct` | `p99_us` | `max_pct` | `max_us` | `subnormais` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `padrao` | `9.616%` | `256.425` | `87.266%` | `2327.1` | `503.276%` | `13420.7` | `0` |
| `ruido` | `2.319%` | `61.832` | `6.514%` | `173.7` | `278.216%` | `7419.1` | `0` |

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

- `Main_stage` em `tests/contract/048-ui-desktop-fidelidade-visual.test.js` obteve `SSIM = 0.8512` frente ao limiar `0.85` (margem de `+0.0012`), sem violar o critério.
- `auditoria_2026_11_20_tempo_real::aud_05_erro_do_stream_marca_perda` (`apps/sonix-app/src-tauri`) falhou uma vez sob carga paralela de compilação (`elapsed = 354.25 ms` > `200 ms`, `is_lost = true`) e passou na execução isolada subsequente (`2 passed; 0 failed`).

## 6. `git log --oneline main..fix/auditoria-2026-11-20`

```text
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
