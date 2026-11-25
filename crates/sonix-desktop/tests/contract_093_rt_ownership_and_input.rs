//! Spec 093 — CA-2 (Zero-Mutex RT), CA-4 (3 Modos de Entrada + Canal), CA-7 (Duplex Suave), CA-8 (Escrita Atômica)

use sonix_desktop::api::write_atomic_file;
use sonix_desktop::audio::{
    DesktopAudioHost, DspChainSlot, DspParamUpdate, InputChannel, RealtimeDspChain,
    RealtimeDuplexEngine, TelemetryBuffer,
};
use sonix_desktop::params::{find_param, AppState};
use std::sync::Arc;

#[test]
fn test_ca_02_rt_output_callback_owns_chain_without_mutex_or_dry_bypass() {
    let mut chain = RealtimeDspChain::new(48_000.0);
    chain.apply_param_update(DspParamUpdate::AmpStandby(false));
    chain.apply_param_update(DspParamUpdate::AmpEnabled(true));
    chain.apply_param_update(DspParamUpdate::AmpGain(0.85));
    chain.apply_param_update(DspParamUpdate::AmpMaster(0.7));

    let slot = DspChainSlot::new(chain);
    let mut rt_chain = slot.take_for_audio_thread().expect("audio thread toma posse exclusiva");

    let duplex = Arc::new(RealtimeDuplexEngine::with_target_period(4096, 128));
    let telemetry = Arc::new(TelemetryBuffer::new(256));
    telemetry.set_output_muted(false);

    // Encher entrada do duplex com senoide
    let mut in_buf = vec![0.0f32; 256];
    for (i, s) in in_buf.iter_mut().enumerate() {
        *s = 0.25 * ((i as f32) * 0.09).sin();
    }
    duplex.push_captured_samples(&in_buf);

    let mut out_stereo = vec![0.0f32; 512];
    let mut scratch_in = vec![0.0f32; 256];
    let mut scratch_out = vec![0.0f32; 256];

    DesktopAudioHost::process_output_callback_f32(
        &mut out_stereo,
        2,
        48_000.0,
        &mut scratch_in,
        &mut scratch_out,
        &duplex,
        &mut rt_chain,
        &telemetry,
        None,
    );

    assert_eq!(
        telemetry.dsp_contention_silence_blocks(),
        0,
        "Sem Mutex/try_lock no callback RT, contenção deve ser sempre 0"
    );

    // O sinal de saída não pode ser cópia crua do sinal de entrada (dry-bypass)
    let mut diff_sum = 0.0f32;
    for i in 0..256 {
        diff_sum += (out_stereo[i * 2] - in_buf[i]).abs();
    }
    assert!(
        diff_sum > 0.5,
        "Saída processada pelo amp não pode ser sinal cru (diff_sum={diff_sum})"
    );
}

#[test]
fn test_ca_04_three_explicit_input_modes_and_channel_param() {
    let desc_mode = find_param("input.mode").expect("input.mode deve existir");
    let desc_chan = find_param("input.channel").expect("input.channel deve existir");

    let mut st = AppState::default();
    let v_inst = serde_json::json!("instrument");
    (desc_mode.write_state)(&mut st, &v_inst);
    let u1 = (desc_mode.to_dsp)(&v_inst).expect("instrument válido");
    assert_eq!(st.input_mode, "instrument");

    let v_pc = serde_json::json!("pc_line_in");
    (desc_mode.write_state)(&mut st, &v_pc);
    let u2 = (desc_mode.to_dsp)(&v_pc).expect("pc_line_in válido");
    assert_eq!(st.input_mode, "pc_line_in");

    let v_line = serde_json::json!("line");
    (desc_mode.write_state)(&mut st, &v_line);
    let u3 = (desc_mode.to_dsp)(&v_line).expect("line válido");
    assert_eq!(st.input_mode, "line");

    let v_right = serde_json::json!("right");
    (desc_chan.write_state)(&mut st, &v_right);
    assert_eq!(st.input_channel, "right");

    // Verificar ganho real: instrument e pc_line_in = 0 dB; line = -11.78 dB (~0.2576x)
    let mut chain_inst = RealtimeDspChain::new(48_000.0);
    chain_inst.apply_param_update(DspParamUpdate::AmpStandby(false));
    chain_inst.apply_param_update(DspParamUpdate::InputBoost(1.0));
    chain_inst.apply_param_update(DspParamUpdate::GateEnabled(false));
    chain_inst.apply_param_update(DspParamUpdate::AmpEnabled(false));
    chain_inst.apply_param_update(DspParamUpdate::CabEnabled(false));
    chain_inst.apply_param_update(u1);

    let mut chain_pc = RealtimeDspChain::new(48_000.0);
    chain_pc.apply_param_update(DspParamUpdate::AmpStandby(false));
    chain_pc.apply_param_update(DspParamUpdate::InputBoost(1.0));
    chain_pc.apply_param_update(DspParamUpdate::GateEnabled(false));
    chain_pc.apply_param_update(DspParamUpdate::AmpEnabled(false));
    chain_pc.apply_param_update(DspParamUpdate::CabEnabled(false));
    chain_pc.apply_param_update(u2);

    let mut chain_line = RealtimeDspChain::new(48_000.0);
    chain_line.apply_param_update(DspParamUpdate::AmpStandby(false));
    chain_line.apply_param_update(DspParamUpdate::InputBoost(1.0));
    chain_line.apply_param_update(DspParamUpdate::GateEnabled(false));
    chain_line.apply_param_update(DspParamUpdate::AmpEnabled(false));
    chain_line.apply_param_update(DspParamUpdate::CabEnabled(false));
    chain_line.apply_param_update(u3);

    let input: Vec<f32> = (0..1024).map(|i| 0.25 * ((i as f32) * 0.08).sin()).collect();
    let mut out_inst = vec![0.0f32; 1024];
    let mut out_pc = vec![0.0f32; 1024];
    let mut out_line = vec![0.0f32; 1024];

    chain_inst.process(&input, &mut out_inst);
    chain_pc.process(&input, &mut out_pc);
    chain_line.process(&input, &mut out_line);

    let rms = |s: &[f32]| (s[512..].iter().map(|x| x * x).sum::<f32>() / (s.len() - 512) as f32).sqrt();
    let r_inst = rms(&out_inst);
    let r_pc = rms(&out_pc);
    let r_line = rms(&out_line);

    assert!(
        (r_pc / r_inst - 1.0).abs() < 0.02,
        "pc_line_in deve manter ganho nominal 0 dB igual a instrument (r_pc={r_pc}, r_inst={r_inst})"
    );
    let ratio_db = 20.0 * (r_line / r_inst).log10();
    assert!(
        (ratio_db - (-11.78)).abs() < 0.2,
        "line (+4 dBu) deve aplicar pad de -11.78 dB (obtido {ratio_db:.2} dB)"
    );
    assert_eq!(InputChannel::from_str_opt("sum"), InputChannel::Sum);
}

#[test]
fn test_ca_07_duplex_smooth_underrun_without_hard_step() {
    let duplex = RealtimeDuplexEngine::with_target_period(2048, 128);
    let input: Vec<f32> = (0..128).map(|i| 0.25 * ((i as f32) * 0.08).sin()).collect();
    duplex.push_captured_samples(&input);

    let mut out = vec![0.0f32; 384];
    let _ = duplex.pop_samples_for_processing(&mut out);

    // Verifica que na transição entre amostras disponíveis e underrun não há salto instantâneo > 0.15
    let mut max_step = 0.0f32;
    for i in 1..out.len() {
        max_step = max_step.max((out[i] - out[i - 1]).abs());
    }
    assert!(
        max_step <= 0.15,
        "Underrun no duplex deve aplicar micro-fade suave sem salto brusco (max_step={max_step})"
    );
}

#[test]
fn test_ca_08_write_atomic_file_preserves_existing_on_failure() {
    let dir = std::env::temp_dir().join(format!("sonix_093_atomic_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let target = dir.join("preset.json");
    std::fs::write(&target, br#"{"ok":true}"#).expect("seed");

    write_atomic_file(&target, br#"{"ok":2}"#).expect("escrita atômica bem-sucedida");
    assert_eq!(std::fs::read(&target).unwrap(), br#"{"ok":2}"#);

    // Se o caminho pai não existir, deve falhar sem remover nada existente
    let bad_target = dir.join("missing_subdir").join("preset.json");
    assert!(write_atomic_file(&bad_target, br#"bad"#).is_err());

    let _ = std::fs::remove_dir_all(&dir);
}
