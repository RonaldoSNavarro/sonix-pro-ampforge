# Inventário de Controles — SONIX PRO // AMPFORGE™ (Spec 033)

> **Regra zero controle decorativo:** Todo controle visível na UI desktop consta em [`apps/sonix-app/ui/src/controls.registry.ts`](../../apps/sonix-app/ui/src/controls.registry.ts), é aceito pelo despachante/backend (Spec 014) ou ação IPC e possui teste automatizado de efeito (CA-4 e CA-5 da Spec 033).

## Status Final Auditado (Zero Pendências — Specs 030, 031, 032 e 033 Implementadas)

| Grupo | Controle UI | Chave Backend / IPC | Comportamento Esperado (Equipamento Real) | Status / Verificação |
| :--- | :--- | :--- | :--- | :--- |
| Topbar / Rig & UI | Janela (Min/Max/Fechar), Cenas, Presets, Input Source/Gain, Tap, Navegação e Sub-abas | `scene.active`, `input.*`, `amp_2.*`, `cab_2.*`, IPC / Visual | Troca cenas, presets, telas e ganho de entrada atomicamente | **OK (Spec 033)** |
| Motor | `engine.oversampling` (`2x/4x/8x`) | `engine.oversampling` | Muda fator real de oversampling IIR/FIR | **OK (Spec 031)** |
| Amp (5 modelos) | `power`, `standby`, `bright`, `input_jack`, `jumpered`, `channel_mode`, `spongy`, `cut`, `reverb`, `vibrato_*`, `gain`, `volume_2`, `bass`, `middle`, `treble`, `presence`, `depth`, `master` | `amp_1.*` | Tone stacks passivos exatos, Power/Standby silenciam, Low -6 dB, Plexi Jumper, Twin Vibrato, AC30 Cut | **OK (Spec 031 / 033)** |
| Chassi / Mod Shop | `preamp_tube`, `power_tube`, `rectifier`, `impedance_ohms`, `bias_ma`, `sag`, `nfb`, `xformer_sat`, `preamp_plate_v`, `triode_asym`, `tonestack` | `amp_1.*` | Cada troca muda ganho, compressão, curva de impedância ou resposta harmônica | **OK (Spec 031)** |
| Noise Gate (Spec 030) | `enabled`, `mode`, `threshold_db`, `release_ms` (DECAY), `hysteresis_db`, `attack_ms`, `hold_ms`, `range_db`, `key_hpf_hz`, `key_lpf_hz` | `gate.*` | Histerese anti-chatter, hold/release, LR4 sidechain | **OK (Spec 030)** |
| Drives (6 modelos) | `drive.model` (`ts808/klon/rat/muff/fuzzface/ds1`), `drive.drive`, `drive.tone` (Filter invertido no RAT), `drive.level`, `drive.bias` | `drive.*` | Topologias distintas, blend limpo no Klon, notch no Muff, trim Ge no Fuzz Face | **OK (Spec 032)** |
| Chorus & EQ | `mod_2.*` (`enabled`, `rate_hz`, `depth`, `mix`), `eq_1.enabled`, `eq_1.band1..8` | `mod_2.*`, `eq_1.*` | Chorus BBD e EQ 8/10 bandas ativos | **OK (Spec 032)** |
| MXR 10-Band EQ | `eq_1.mode` (`mxr_10band`), `eq_1.band9..10`, `eq_1.gain_db`, `eq_1.volume_db` | `eq_1.*` | 10 bandas ISO + trims pré/pós EQ | **OK (Spec 032)** |
| Compressor 1176LN | `comp_2.enabled`, `input`, `attack`, `release`; `comp_2.output`, `comp_2.ratio` (`all`) | `comp_2.*` | `output` aplica makeup real; `"all"` ativa All-Buttons-In | **OK (Spec 032)** |
| Tape Echo RE-201 | `dly_1.enabled`, `repeat_rate`, `intensity`, `echo_volume`, `spillover`; `mode` (1–12), `bass`, `treble`, `reverb_volume` | `dly_1.*` | Modos 1–12 reais, EQ só no wet, mola nos modos 5–12 | **OK (Spec 032)** |
| Plate Reverb & Routing | `rev_1.*`, `split.*`, `merge.*` | `rev_1.*`, `split.*`, `merge.*` | Spillover, damping, crossover A/B, inversão de fase | **OK (Spec 024 / 033)** |
| Gabinete, Mics & IR | `cab_1.enabled`, `resonance_hz`, `hicut_khz`, `ir_enabled`, `mic{1,2}.{enabled,model,position,distance_cm,level_db}`, `twin_2x12_open`, `mic*.angle_deg`, `mic*.phase_invert` | `cab_1.*`, `load_ir` | `twin_2x12_open`, off-axis 45° e fase Ø com efeito físico + seleção nativa IR | **OK (Spec 032 / 033)** |
| Áudio & Calibração | Dispositivo WASAPI/ASIO, taxa, buffer, canal, calibração, round-trip | IPC (`set_audio_device`, `calibrate_input`, `measure_latency`) | Configura stream real e persiste calibração | **OK (Spec 029 / 033)** |
| Palco | `tuner.enabled`, `tuner.output`, `tuner.ref_hz`, Footswitches F1–F8, EXP 1, `looper.action`, `looper.position` | `tuner.*`, `looper.*`, `amp_1.gain`, switches | `tuner.ref_hz` e `looper.*` conectados ao `TunerProcessor`/`LooperProcessor` em ring buffer zero-alloc | **OK (Spec 033)** |
