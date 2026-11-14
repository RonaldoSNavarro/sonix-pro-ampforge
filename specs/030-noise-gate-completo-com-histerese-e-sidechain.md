---
spec: 030
titulo: Noise gate sem controles na UI e sem histerese, hold funcional ou filtro de sidechain no DSP
status: implementada
atualizado: 2026-11-13
---
# 030 — Noise Gate Completo (Histerese, Hold, Range e Key Filter) com Controles na UI

## Objetivo
Entregar um bloco de Noise Gate utilizável para guitarra high-gain e captação em placa onboard, com histerese anti-chatter, tempo de hold funcional, atenuação configurável (range), filtro de sidechain (key filter) e controles completos no editor da UI.

## Contexto
Origem: relato do usuário ("não tem um noise gate") e inspeção do código:

1. **Invisível/inoperável na UI (`design-system/app.js:349-353`):** Ao clicar no bloco `blk_gate` (`Natural Gate`) no grid, `openBlockEditor` cai no `else` e exibe `"sem controles nesta versão"`. Não há sliders para threshold, decay/release, attack, hold ou modo, nem indicador de redução de ganho (GR).
2. **Deficiências no DSP (`crates/ampforge-dsp/src/fx/dynamics.rs:10-107`):**
   - `hold_ms` existe como campo da struct (`dynamics.rs:15`) mas **nunca é lido em `process()`** (`dynamics.rs:69-106`), fazendo o gate fechar imediatamente quando a envoltória cruza o limiar.
   - Não há **histerese** (limiar de abertura vs limiar de fechamento), causando tremolação audível (*chatter*) na cauda de notas sustentadas.
   - Não há **filtro de sidechain / key filter** (HPF + LPF) no detector, permitindo que hum de 50/60 Hz abra o gate ou que ruído agudo de captador single-coil mantenha o gate aberto.
   - Não há parâmetro **`range_db`** (atenuação máxima quando fechado, como no ISP Decimator / Boss NS-2), nem suavização livre de cliques na transição.

## Comportamento esperado
1. **Máquina de estados com Histerese + Hold no `NoiseGateProcessor`:**
   - Estados: `Open` → `Hold` → `Closing/Closed`.
   - O gate **abre** quando a envoltória do detector (filtrada pelo key filter) ultrapassa `threshold_db`.
   - Enquanto abaixo de `threshold_db - hysteresis_db`, inicia a contagem de `hold_ms`; só após expirar `hold_ms` o ganho decai suavemente segundo `release_ms` até `range_db` (ex.: -80 dB = mute total, -40 dB = redução suave).
   - Se a envoltória subir acima de `threshold_db` durante o `Hold` ou `Closing`, reabre com `attack_ms`.
2. **Key Filter (Sidechain):**
   - Passa-altas (`key_hpf_hz`, padrão 80 Hz, faixa 20–500 Hz) e passa-baixas (`key_lpf_hz`, padrão 6000 Hz, faixa 1000–12000 Hz) aplicados exclusivamente ao sinal do detector (nunca ao áudio principal), evitando disparo falso por sub-grave/hum 60 Hz ou chiado agudo.
   - Suporte aos dois modos: `natural` (`IntelligentNaturalGate` usando sinal DI limpo pós-DC-blocker como sidechain) e `classic` (`ClassicExpanderGate` usando a própria entrada do bloco).
3. **Parâmetros expostos no registro da Spec 014 e `/api/params`:**
   - `gate.enabled` (bool, padrão `true`)
   - `gate.mode` (enum `"natural"` | `"classic"`, padrão `"natural"`)
   - `gate.threshold_db` (-90.0..0.0 dB, padrão `-55.0`)
   - `gate.hysteresis_db` (0.0..12.0 dB, padrão `4.0`)
   - `gate.attack_ms` (0.1..50.0 ms, padrão `1.0`)
   - `gate.hold_ms` (0.0..500.0 ms, padrão `35.0`)
   - `gate.release_ms` (5.0..1000.0 ms, padrão `80.0`)
   - `gate.range_db` (-90.0..0.0 dB, padrão `-70.0`)
   - `gate.key_hpf_hz` (20.0..500.0 Hz, padrão `80.0`)
   - `gate.key_lpf_hz` (1000.0..12000.0 Hz, padrão `6000.0`)
4. **UI no Editor de Bloco (`blk_gate`):**
   - Exibe toggle Ativo/Bypass, seletor de modo (`Natural DI Sidechain` / `Classic Expander`), knobs/sliders para `Threshold`, `Hysteresis`, `Attack`, `Hold`, `Release`, `Range` e `Key HPF/LPF`, mais LED/barra de status (`OPEN` / `HOLD` / `CLOSED` e `GR dB`).

## Critérios de aceite
- **CA-1:** Com sinal senoidal caindo abaixo de `threshold_db - hysteresis_db`, o gate mantém ganho `1.0` durante `hold_ms ± 2 ms` antes de iniciar a atenuação; com sinal oscilando dentro da banda `[threshold_db - hysteresis_db, threshold_db]`, não há alternância rápida (0 chatter).
- **CA-2:** Com `gate.range_db = -20.0` e entrada em silêncio (`-90 dBFS`), a atenuação estacionária do gate é `-20.0 dB ± 0.5 dB` (não silencia para `-80 dB`).
- **CA-3:** Com `gate.key_hpf_hz = 120.0` e `gate.threshold_db = -50.0`, um hum de `60 Hz` a `-35 dBFS` na entrada não abre o gate (atenuação mantida), enquanto uma nota de `220 Hz` a `-35 dBFS` abre o gate normalmente.
- **CA-4:** `POST /api/params` aceita e valida todas as chaves `gate.*` dentro de suas faixas; valores fora de faixa retornam `400`.
- **CA-5:** Na UI, clicar em `blk_gate` renderiza os controles interativos de `gate.*` (sem exibir `"sem controles nesta versão"`), e alterar `Threshold` atualiza o DSP em tempo real.
- **CA-6:** Zero alocações de heap e zero locks no `NoiseGateProcessor::process`.
- **CA-7:** Não-regressão — testes da Spec 006 (CA-1/CA-2) e Spec 019 continuam verdes.

## Fora de escopo
- Noise gate de 4 cabos com loop externo físico (hardware send/return): o modo `natural` já faz o equivalente interno lendo a entrada DI limpa antes do drive/amp.

## Invariantes tocadas
- Spec 001/028 (Realtime safety): filtros de sidechain e máquina de estado usam apenas escalares pré-alocados.
- Spec 014 (Tabela única de parâmetros): todas as chaves `gate.*` registradas e validadas no servidor.

## Testes que cobrem esta spec
Implementados (*red → green*): `crates/ampforge-dsp/tests/contract_030_noise_gate.rs` (CA-1, CA-2, CA-3, CA-6, CA-7 — 5/5 ok), `crates/sonix-desktop/tests/contract_030_gate_api.rs` (CA-4 — ok) e `tests/contract/030-noise-gate-ui.test.js` (CA-5 + suítes Rust — 3/3 ok).
