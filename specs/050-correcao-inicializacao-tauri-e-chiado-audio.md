---
spec: 050
titulo: Correção da inicialização do shell Tauri release (frontend embutido) e eliminação de chiado ao abrir
status: implementada
atualizado: 2026-11-15
---
# 050 — Correção da inicialização do shell Tauri release e eliminação de chiado ao abrir

## Objetivo
Garantir que o executável desktop `sonix-app.exe` abra exibindo a interface embutida sem depender de servidor `localhost:1420` e sem emitir chiado/microfonia imediata ao iniciar em estações sem calibração ou com dispositivos de áudio em taxas distintas.

## Contexto
Ao abrir `G:\Dev\.cargo-target\simulador-de-efeitos\release\sonix-app.exe` manualmente após o build da Spec 049:
1. A janela abriu exibindo erro do WebView2: *"Hum... Não consigo chegar a esta página — localhost se recusou a se conectar (`ERR_CONNECTION_REFUSED`)"*. A inspeção do binário mostrou `Contains localhost:1420 = True` e `Contains assets/index = False`, porque `cargo build --release -p sonix-app` não habilitava `tauri/custom-protocol` (feature necessária no Tauri v2 para embutir `frontendDist` no executável em vez de apontar para `devUrl`).
2. Ao mesmo tempo, o áudio iniciou com chiado contínuo nos alto-falantes: `build_runtime()` em `apps/sonix-app/src-tauri/src/lib.rs:60-86` inicia imediatamente o duplex WASAPI no dispositivo de entrada padrão do Windows (tipicamente microfone aberto) através do `BritJcm800_2203` com `input_gain = 8.0`, `master_volume = 7.0`, `input_boost = 2.5`, `gate = -65 dB` e `amp_standby = false`, além de `start_realtime_duplex_on_host` (`crates/sonix-desktop/src/audio.rs:1188-1208`) não alinhar `in_stream_config.sample_rate` e `out_stream_config.sample_rate` quando `requested_sample_rate` é `None` e entrada/saída padrão operam em taxas diferentes.

## Comportamento esperado
1. O crate `apps/sonix-app/src-tauri` expõe a feature `custom-protocol = ["tauri/custom-protocol"]` e o build release do aplicativo desktop (`npm --prefix apps/sonix-app/ui run tauri build -- --no-bundle` ou `cargo build --release -p sonix-app --features custom-protocol`) gera `sonix-app.exe` com `apps/sonix-app/ui/dist` embutido e sem referência de navegação a `http://localhost:1420`.
2. Quando o aplicativo desktop (`build_runtime`) inicia sem calibração prévia registrada para o perfil `"default"`, o amplificador inicia em **Standby** (`amp_standby = true` no estado inicial e no `RealtimeDspChain`), silenciando a saída até que o usuário desative `STANDBY` ou calibre a entrada. Quando já existe calibração salva ou quando `amp_1.standby = false` é enviado pelo usuário, o áudio é processado normalmente.
3. Ao iniciar streaming duplex sem `requested_sample_rate` explícito, se a taxa padrão do dispositivo de entrada diferir da taxa padrão do dispositivo de saída, o host escolhe uma taxa comum suportada pela entrada e pela saída (preferindo a taxa da saída quando suportada pela entrada), evitando deriva de taxa e underruns/overruns contínuos.

## Critérios de aceite
- **CA-1:** `apps/sonix-app/src-tauri/Cargo.toml` define `custom-protocol = ["tauri/custom-protocol"]`, e o executável release `sonix-app.exe` gerado com `--features custom-protocol` embute os assets JS/CSS e o HTML de `apps/sonix-app/ui/dist` (sem depender de `http://localhost:1420` em runtime).
- **CA-2:** Na inicialização do runtime desktop sem calibração salva (`calib_store.get("default").is_none()`), `initial_state.amp_standby == true` e `dsp_chain.process(...)` com sinal na entrada produz `RMS < 1e-5` (silêncio seguro); ao enviar `amp_1.standby = false`, o sinal volta a ser amplificado normalmente.
- **CA-3:** `resolve_duplex_sample_rate` em `crates/sonix-desktop/src/audio.rs` alinha entrada e saída para a mesma taxa de amostragem quando `requested_sample_rate` é `None` e os padrões diferem (ex.: `44100` e `48000` com suporte a `48000` -> `(48000, 48000)`), preservando fallback seguro quando a entrada não suporta a taxa da saída.
- **CA-4:** Não-regressão: `RealtimeDspChain::new()` direto e `CalibrationStore` com calibração salva continuam operando normalmente, sem quebrar os contratos existentes (`contract_033`, `contract_049`).
- **CA-5:** Calibração concluída com sucesso (`POST /api/input/calibrate` com `phase: "strum"`) desativa o standby inicial (`amp_standby = false`) tanto no `AppState` (`GET /api/state`) quanto no `RealtimeDspChain` (o sinal volta a ser amplificado na saída).

## Fora de escopo
- Resampler assíncrono completo (SRC) caso hardware exótico não compartilhe nenhuma taxa comum entre entrada e saída.
- Empacotamento de instalador NSIS/MSI.

## Invariantes tocadas
- **Zero-alloc no callback de áudio:** nenhuma alocação adicionada ao callback de áudio; o alinhamento de taxa ocorre antes de abrir os streams e `standby` já opera por flag atômica/booleana pré-existente.
- **Todo controle visível funciona no backend:** `amp_1.standby` já é o controle real do estágio de saída no painel do amplificador.

## Testes que cobrem esta spec
- `tests/contract/050-release-embedded-ui.test.js` → `CA-1` (`1 pass, 0 fail`)
- `apps/sonix-app/src-tauri/tests/contract_050_startup_safety.rs` → `CA-2`, `CA-4`, `CA-5` (`3 passed; 0 failed`)
- `crates/sonix-desktop/tests/contract_050_duplex_rate_alignment.rs` → `CA-3` (`3 passed; 0 failed`)
