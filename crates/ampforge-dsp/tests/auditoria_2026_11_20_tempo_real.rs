//! Testes de auditoria 2026-11-20 — Tempo real e subnormais em `ampforge-dsp` (AUD-02)

use ampforge_dsp::denormal::{mxcsr_atual, DenormalGuard, FTZ_DAZ};
use ampforge_dsp::fx::delay::{DelayModel, DelayProcessor};
use ampforge_dsp::fx::modulation::{ModModel, ModulationProcessor};
use ampforge_dsp::fx::reverb::{ReverbModel, ReverbProcessor};
use ampforge_dsp::regua::contar_subnormais;

#[test]
fn aud_02_guard_liga_e_restaura() {
    let antes = mxcsr_atual();
    {
        let _guard = DenormalGuard::new();
        #[cfg(target_arch = "x86_64")]
        assert_eq!(
            mxcsr_atual() & FTZ_DAZ,
            FTZ_DAZ,
            "AUD-02: FTZ/DAZ deve estar ligado dentro do DenormalGuard"
        );
    }
    let depois = mxcsr_atual();
    assert_eq!(
        depois, antes,
        "AUD-02: MXCSR deve voltar ao valor original após drop do DenormalGuard"
    );
}

const SR: f32 = 48_000.0;
const BLOCK: usize = 128;
const TOTAL_SAMPLES: usize = 20 * 48_000 + 1; // impulso 1e-30 + 20 s de zeros

fn gerar_impulso_subnormal_para_testes() -> Vec<f32> {
    let mut buf = vec![0.0_f32; TOTAL_SAMPLES];
    buf[0] = 1e-30_f32;
    buf
}

fn rodar_em_blocos(entrada: &[f32], mut f: impl FnMut(&[f32], &mut [f32])) -> Vec<f32> {
    let mut saida = vec![0.0_f32; entrada.len()];
    let mut pos = 0usize;
    while pos < entrada.len() {
        let fim = (pos + BLOCK).min(entrada.len());
        f(&entrada[pos..fim], &mut saida[pos..fim]);
        pos = fim;
    }
    saida
}

fn checar_reverb_sem_subnormais(model: ReverbModel, nome: &str) {
    let mut rev = ReverbProcessor::new(model);
    rev.set_sample_rate(SR);
    rev.set_bypassed(false);
    rev.set_decay_time_s(2.0);
    rev.set_predelay_ms(0.0);
    rev.set_damping(5.0);
    rev.set_mix_pct(100.0);

    let entrada = gerar_impulso_subnormal_para_testes();
    let saida = rodar_em_blocos(&entrada, |i, o| rev.process(i, o));
    let sub = contar_subnormais(&saida);
    assert_eq!(
        sub, 0,
        "AUD-02 ({}): detectadas {} amostras subnormais no reverb sem guard",
        nome, sub
    );
}

fn checar_delay_sem_subnormais(model: DelayModel, re201_mode: Option<u8>, nome: &str) {
    let mut dly = DelayProcessor::new(model);
    dly.set_sample_rate(SR);
    dly.set_bypassed(false);
    dly.set_delay_time_ms(20.0);
    dly.set_feedback_pct(65.0);
    dly.set_mix_pct(100.0);
    if let Some(mode) = re201_mode {
        dly.set_re201_mode(mode);
        dly.set_reverb_volume(70.0);
    }

    let entrada = gerar_impulso_subnormal_para_testes();
    let saida = rodar_em_blocos(&entrada, |i, o| dly.process(i, o));
    let sub = contar_subnormais(&saida);
    assert_eq!(
        sub, 0,
        "AUD-02 ({}): detectadas {} amostras subnormais no delay sem guard",
        nome, sub
    );
}

fn checar_mod_sem_subnormais(model: ModModel, nome: &str) {
    let mut m = ModulationProcessor::new(model);
    m.set_sample_rate(SR);
    m.set_rate_hz(1.5);
    m.set_depth(0.8);
    m.set_feedback(0.75);
    m.set_mix_pct(100.0);

    let entrada = gerar_impulso_subnormal_para_testes();
    let saida = rodar_em_blocos(&entrada, |i, o| m.process(i, o));
    let sub = contar_subnormais(&saida);
    assert_eq!(
        sub, 0,
        "AUD-02 ({}): detectadas {} amostras subnormais na modulação sem guard",
        nome, sub
    );
}

#[test]
fn aud_02_reverb_tube_spring_sem_subnormais() {
    checar_reverb_sem_subnormais(ReverbModel::TubeSpringReverb, "TubeSpringReverb");
}

#[test]
fn aud_02_reverb_studio_plate_sem_subnormais() {
    checar_reverb_sem_subnormais(ReverbModel::StudioVintagePlate, "StudioVintagePlate");
}

#[test]
fn aud_02_reverb_concert_hall_sem_subnormais() {
    checar_reverb_sem_subnormais(ReverbModel::ConcertHall, "ConcertHall");
}

#[test]
fn aud_02_reverb_chamber_sem_subnormais() {
    checar_reverb_sem_subnormais(ReverbModel::Chamber, "Chamber");
}

#[test]
fn aud_02_reverb_shimmer_cloud_sem_subnormais() {
    checar_reverb_sem_subnormais(ReverbModel::ShimmerCloudAmbient, "ShimmerCloudAmbient");
}

#[test]
fn aud_02_delay_multi_head_tape_sem_subnormais() {
    checar_delay_sem_subnormais(DelayModel::MultiHeadTapeEcho, None, "MultiHeadTapeEcho");
}

#[test]
fn aud_02_delay_re201_modos_5_a_12_sem_subnormais() {
    for mode in 5..=12 {
        checar_delay_sem_subnormais(
            DelayModel::MultiHeadTapeEcho,
            Some(mode),
            &format!("RE201_Mode_{}", mode),
        );
    }
}

#[test]
fn aud_02_delay_analog_bbd_sem_subnormais() {
    checar_delay_sem_subnormais(DelayModel::AnalogBbdDelay, None, "AnalogBbdDelay");
}

#[test]
fn aud_02_delay_pristine_digital_sem_subnormais() {
    checar_delay_sem_subnormais(DelayModel::PristineDigitalDelay, None, "PristineDigitalDelay");
}

#[test]
fn aud_02_delay_dual_parallel_sem_subnormais() {
    checar_delay_sem_subnormais(DelayModel::DualParallelDelay, None, "DualParallelDelay");
}

#[test]
fn aud_02_delay_reverse_sem_subnormais() {
    checar_delay_sem_subnormais(DelayModel::ReverseDelay, None, "ReverseDelay");
}

#[test]
fn aud_02_delay_ping_pong_sem_subnormais() {
    checar_delay_sem_subnormais(DelayModel::PingPongStereo, None, "PingPongStereo");
}

#[test]
fn aud_02_mod_analog_bbd_chorus_sem_subnormais() {
    checar_mod_sem_subnormais(ModModel::AnalogBbdChorus, "AnalogBbdChorus");
}

#[test]
fn aud_02_mod_stereo_tri_chorus_sem_subnormais() {
    checar_mod_sem_subnormais(ModModel::StereoTriChorus, "StereoTriChorus");
}

#[test]
fn aud_02_mod_through_zero_flanger_sem_subnormais() {
    checar_mod_sem_subnormais(ModModel::ThroughZeroFlanger, "ThroughZeroFlanger");
}

#[test]
fn aud_02_mod_orange90_phaser_sem_subnormais() {
    checar_mod_sem_subnormais(ModModel::Orange90Phaser, "Orange90Phaser");
}

#[test]
fn aud_02_mod_optical_bias_tremolo_sem_subnormais() {
    checar_mod_sem_subnormais(ModModel::OpticalBiasTremolo, "OpticalBiasTremolo");
}

#[test]
fn aud_02_mod_photocell_univibe_sem_subnormais() {
    checar_mod_sem_subnormais(ModModel::PhotocellUnivibe, "PhotocellUnivibe");
}

#[test]
fn aud_02_mod_rotary_speaker_sem_subnormais() {
    checar_mod_sem_subnormais(ModModel::RotarySpeaker, "RotarySpeaker");
}
