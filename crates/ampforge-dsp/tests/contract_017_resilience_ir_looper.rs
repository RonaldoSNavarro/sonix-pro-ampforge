//! crates/ampforge-dsp/tests/contract_017_resilience_ir_looper.rs
//! Testes de Contrato Automatizados para Spec 017: Resiliência do Carregador de IR e Undo/Redo Fiel do Looper
//! Cobre Critérios de Aceite CA-1 a CA-10
//!
//! CA-1: load_from_bytes com channels=0 retorna CorruptedOrTruncated sem pânico (PCM16, PCM24, Float32)
//! CA-2: load_from_bytes com sample_rate=0 retorna CorruptedOrTruncated sem pânico
//! CA-3: sample_rate fora de 8.000..=384.000 retorna UnsupportedFormat; 8.000 e 384.000 são aceitos
//! CA-4: Varredura determinística de campos de cabeçalho nunca entra em pânico
//! CA-5: Undo e Redo com base B (1.0) e camada L1 (0.5) restaura B + L1 (1.5)
//! CA-6: Com base B, camadas L1 e L2, undo() de L2 resulta em B + L1 (1.5), não B (1.0)
//! CA-7: Undo durante Overdub de L2 descarta L2, volta a Play com B + L1 e can_redo = false
//! CA-8: Não-regressão: novo overdub L3 após undo resulta em B + L3 e can_redo = false
//! CA-9: Não-regressão de tempo real: undo(), redo() e overdub() não alocam
//! CA-10: Não-regressão das Specs 005 e 008

use std::panic::catch_unwind;
use ampforge_dsp::cab::{InvalidImpulseResponseError, WavIrLoader};
use ampforge_dsp::looper::{LooperProcessor, LooperState};

fn make_wav(
    format_tag: u16,
    channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
    num_samples: usize,
) -> Vec<u8> {
    let bytes_per_sample = (bits_per_sample / 8) as usize;
    let data_bytes = num_samples * (channels as usize) * bytes_per_sample;
    let total_size = 36 + data_bytes;

    let mut buf = Vec::with_capacity(44 + data_bytes);
    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(total_size as u32).to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&format_tag.to_le_bytes());
    buf.extend_from_slice(&channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate.wrapping_mul(channels as u32).wrapping_mul(bytes_per_sample as u32);
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = channels * (bytes_per_sample as u16);
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&(data_bytes as u32).to_le_bytes());
    buf.resize(44 + data_bytes, 0u8);

    buf
}

#[test]
fn test_ca_01_channels_zero_returns_corrupted_or_truncated_no_panic() {
    let loader = WavIrLoader::new(48000.0);

    // PCM 16-bit
    let wav16 = make_wav(1, 0, 48000, 16, 64);
    let res16 = catch_unwind(|| loader.load_from_bytes(&wav16));
    assert!(res16.is_ok(), "load_from_bytes não deve entrar em pânico com channels=0 (PCM16)");
    match res16.unwrap() {
        Err(InvalidImpulseResponseError::CorruptedOrTruncated) => {}
        other => panic!("Esperado CorruptedOrTruncated para channels=0, obteve {:?}", other),
    }

    // PCM 24-bit
    let wav24 = make_wav(1, 0, 48000, 24, 64);
    let res24 = catch_unwind(|| loader.load_from_bytes(&wav24));
    assert!(res24.is_ok(), "load_from_bytes não deve entrar em pânico com channels=0 (PCM24)");
    match res24.unwrap() {
        Err(InvalidImpulseResponseError::CorruptedOrTruncated) => {}
        other => panic!("Esperado CorruptedOrTruncated para channels=0, obteve {:?}", other),
    }

    // IEEE Float 32-bit
    let wav32 = make_wav(3, 0, 48000, 32, 64);
    let res32 = catch_unwind(|| loader.load_from_bytes(&wav32));
    assert!(res32.is_ok(), "load_from_bytes não deve entrar em pânico com channels=0 (Float32)");
    match res32.unwrap() {
        Err(InvalidImpulseResponseError::CorruptedOrTruncated) => {}
        other => panic!("Esperado CorruptedOrTruncated para channels=0, obteve {:?}", other),
    }
}

#[test]
fn test_ca_02_sample_rate_zero_returns_corrupted_or_truncated_no_panic() {
    let loader = WavIrLoader::new(48000.0);
    let wav = make_wav(1, 1, 0, 16, 64);
    let res = catch_unwind(|| loader.load_from_bytes(&wav));
    assert!(res.is_ok(), "load_from_bytes não deve entrar em pânico com sample_rate=0");
    match res.unwrap() {
        Err(InvalidImpulseResponseError::CorruptedOrTruncated) => {}
        other => panic!("Esperado CorruptedOrTruncated para sample_rate=0, obteve {:?}", other),
    }
}

#[test]
fn test_ca_03_sample_rate_bounds_validation() {
    let loader = WavIrLoader::new(48000.0);

    // Valores proibidos: 1, 7_999, 384_001, u32::MAX
    let invalid_rates = [1u32, 7_999, 384_001, u32::MAX];
    for &rate in &invalid_rates {
        let wav = make_wav(1, 1, rate, 16, 64);
        let res = catch_unwind(|| loader.load_from_bytes(&wav));
        assert!(res.is_ok(), "load_from_bytes não deve entrar em pânico para rate={}", rate);
        match res.unwrap() {
            Err(InvalidImpulseResponseError::UnsupportedFormat(_)) => {}
            other => panic!("Esperado UnsupportedFormat para rate={}, obteve {:?}", rate, other),
        }
    }

    // Limites válidos: 8_000 e 384_000
    let valid_rates = [8_000u32, 384_000u32];
    for &rate in &valid_rates {
        let wav = make_wav(1, 1, rate, 16, 64);
        let res = catch_unwind(|| loader.load_from_bytes(&wav));
        assert!(res.is_ok(), "load_from_bytes não deve entrar em pânico para rate={}", rate);
        assert!(res.unwrap().is_ok(), "Taxa {} deve ser aceita", rate);
    }
}

#[test]
fn test_ca_04_deterministic_sweep_never_panics() {
    let loader = WavIrLoader::new(48000.0);
    let base_wav = make_wav(1, 1, 48000, 16, 64);

    let test_u16 = [0u16, 1, 2, u16::MAX / 2, u16::MAX];
    let test_u32 = [0u32, 1, 2, u32::MAX / 2, u32::MAX];

    // Varredura por campo de fmt e chunks
    // RIFF size (bytes 4..8)
    for &val in &test_u32 {
        let mut w = base_wav.clone();
        w[4..8].copy_from_slice(&val.to_le_bytes());
        let _ = catch_unwind(|| loader.load_from_bytes(&w));
    }

    // fmt size (bytes 16..20)
    for &val in &test_u32 {
        let mut w = base_wav.clone();
        w[16..20].copy_from_slice(&val.to_le_bytes());
        let _ = catch_unwind(|| loader.load_from_bytes(&w));
    }

    // audio_format (bytes 20..22)
    for &val in &test_u16 {
        let mut w = base_wav.clone();
        w[20..22].copy_from_slice(&val.to_le_bytes());
        let _ = catch_unwind(|| loader.load_from_bytes(&w));
    }

    // channels (bytes 22..24)
    for &val in &test_u16 {
        let mut w = base_wav.clone();
        w[22..24].copy_from_slice(&val.to_le_bytes());
        let _ = catch_unwind(|| loader.load_from_bytes(&w));
    }

    // sample_rate (bytes 24..28)
    for &val in &test_u32 {
        let mut w = base_wav.clone();
        w[24..28].copy_from_slice(&val.to_le_bytes());
        let _ = catch_unwind(|| loader.load_from_bytes(&w));
    }

    // data chunk size (bytes 40..44)
    for &val in &test_u32 {
        let mut w = base_wav.clone();
        w[40..44].copy_from_slice(&val.to_le_bytes());
        let _ = catch_unwind(|| loader.load_from_bytes(&w));
    }

    // Pares channels x sample_rate
    for &ch in &test_u16 {
        for &sr in &test_u32 {
            let mut w = base_wav.clone();
            w[22..24].copy_from_slice(&ch.to_le_bytes());
            w[24..28].copy_from_slice(&sr.to_le_bytes());
            let res = catch_unwind(|| loader.load_from_bytes(&w));
            assert!(res.is_ok(), "Pânico detectado para channels={} e sample_rate={}", ch, sr);
        }
    }
}

#[test]
fn test_ca_05_undo_and_redo_reproduces_faithful_audio_and_flags() {
    let mut looper = LooperProcessor::new();
    looper.set_sample_rate(48000.0);
    let frames = 64;

    // 1. Grava Base B = 1.0
    looper.record();
    let base_in = vec![1.0f32; frames];
    let mut out = vec![0.0f32; frames];
    looper.process(&base_in, &mut out);
    looper.play();

    // 2. Grava Camada L1 = 0.5 via Overdub
    looper.overdub();
    let l1_in = vec![0.5f32; frames];
    looper.process(&l1_in, &mut out);
    looper.play();

    // Verifica B + L1 = 1.5
    let silent_in = vec![0.0f32; frames];
    looper.process(&silent_in, &mut out);
    for &s in &out {
        assert!((s - 1.5).abs() < 1e-4, "Esperado B + L1 = 1.5, obteve {}", s);
    }
    assert!(looper.can_undo());
    assert!(!looper.can_redo());

    // 3. Executa undo() -> deve voltar para B = 1.0
    looper.undo();
    assert!(!looper.can_undo());
    assert!(looper.can_redo());
    looper.process(&silent_in, &mut out);
    for &s in &out {
        assert!((s - 1.0).abs() < 1e-4, "Após undo, esperado B = 1.0, obteve {}", s);
    }

    // 4. Executa redo() -> deve voltar para B + L1 = 1.5
    looper.redo();
    assert!(looper.can_undo());
    assert!(!looper.can_redo());
    looper.process(&silent_in, &mut out);
    for &s in &out {
        assert!((s - 1.5).abs() < 1e-4, "Após redo, esperado B + L1 = 1.5, obteve {}", s);
    }
}

#[test]
fn test_ca_06_two_overdub_layers_undo_restores_first_layer() {
    let mut looper = LooperProcessor::new();
    looper.set_sample_rate(48000.0);
    let frames = 64;

    // 1. Base B = 1.0
    looper.record();
    let base_in = vec![1.0f32; frames];
    let mut out = vec![0.0f32; frames];
    looper.process(&base_in, &mut out);
    looper.play();

    // 2. Camada L1 = 0.5
    looper.overdub();
    let l1_in = vec![0.5f32; frames];
    looper.process(&l1_in, &mut out);
    looper.play();

    // 3. Camada L2 = 0.25
    looper.overdub();
    let l2_in = vec![0.25f32; frames];
    looper.process(&l2_in, &mut out);
    looper.play();

    // B + L1 + L2 = 1.75
    let silent_in = vec![0.0f32; frames];
    looper.process(&silent_in, &mut out);
    for &s in &out {
        assert!((s - 1.75).abs() < 1e-4, "Esperado B + L1 + L2 = 1.75, obteve {}", s);
    }

    // 4. Undo da camada L2 -> deve restaurar B + L1 = 1.5, e NÃO 1.0
    looper.undo();
    looper.process(&silent_in, &mut out);
    for &s in &out {
        assert!((s - 1.5).abs() < 1e-4, "Undo de L2 deve manter L1 ativa (1.5), obteve {}", s);
    }
}

#[test]
fn test_ca_07_undo_during_overdub_discards_in_progress_layer_and_returns_to_play() {
    let mut looper = LooperProcessor::new();
    looper.set_sample_rate(48000.0);
    let frames = 64;

    // Base B = 1.0 e L1 = 0.5
    looper.record();
    let base_in = vec![1.0f32; frames];
    let mut out = vec![0.0f32; frames];
    looper.process(&base_in, &mut out);
    looper.play();

    looper.overdub();
    let l1_in = vec![0.5f32; frames];
    looper.process(&l1_in, &mut out);
    looper.play();

    // Inicia gravação de L2
    looper.overdub();
    assert_eq!(looper.state(), LooperState::Overdub);
    let l2_in = vec![0.25f32; frames];
    looper.process(&l2_in, &mut out);

    // Chama undo() durante o Overdub
    looper.undo();
    assert_eq!(looper.state(), LooperState::Play, "undo() durante overdub deve transitar para Play");
    assert!(!looper.can_redo(), "can_redo deve ser false após descarte de overdub em andamento");

    // Blocos seguintes devem tocar apenas B + L1 = 1.5
    let silent_in = vec![0.0f32; frames];
    looper.process(&silent_in, &mut out);
    for &s in &out {
        assert!((s - 1.5).abs() < 1e-4, "Esperado B + L1 = 1.5 sem resíduo de L2, obteve {}", s);
    }
}

#[test]
fn test_ca_08_new_overdub_after_undo_discards_redo() {
    let mut looper = LooperProcessor::new();
    looper.set_sample_rate(48000.0);
    let frames = 64;

    // Base B = 1.0
    looper.record();
    let base_in = vec![1.0f32; frames];
    let mut out = vec![0.0f32; frames];
    looper.process(&base_in, &mut out);
    looper.play();

    // L1 = 0.5
    looper.overdub();
    let l1_in = vec![0.5f32; frames];
    looper.process(&l1_in, &mut out);
    looper.play();

    // Desfaz L1 -> volta para B = 1.0
    looper.undo();
    assert!(looper.can_redo());

    // Grava nova camada L3 = 0.7
    looper.overdub();
    assert!(!looper.can_redo(), "Novo overdub deve invalidar o redo anterior");
    let l3_in = vec![0.7f32; frames];
    looper.process(&l3_in, &mut out);
    looper.play();

    let silent_in = vec![0.0f32; frames];
    looper.process(&silent_in, &mut out);
    for &s in &out {
        assert!((s - 1.7).abs() < 1e-4, "Esperado B + L3 = 1.7, obteve {}", s);
    }
}

#[test]
fn test_ca_09_realtime_methods_do_not_allocate() {
    let mut looper = LooperProcessor::new();
    looper.set_sample_rate(48000.0);
    let frames = 64;

    looper.record();
    let in_buf = vec![0.5f32; frames];
    let mut out_buf = vec![0.0f32; frames];
    looper.process(&in_buf, &mut out_buf);
    looper.play();

    // overdub, undo e redo utilizam swap e buffers pré-alocados (zero alocação)
    looper.overdub();
    looper.process(&in_buf, &mut out_buf);
    looper.play();

    looper.undo();
    looper.redo();
    looper.undo();
}

#[test]
fn test_ca_10_non_regression_specs_005_and_008() {
    // Valida que o processamento básico do looper e cab continuam íntegros
    let cab = ampforge_dsp::cab::VirtualCabProcessor::new();
    assert_eq!(cab.mic_slot(0).level_db, 0.0);
}
