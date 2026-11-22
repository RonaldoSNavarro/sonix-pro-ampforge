//! Testes de Contrato Automatizados - Spec 001
//! SDD: specs/001-nucleo-dsp-tempo-real-ipc-lock-free.md
//! Regime TDD Estrito: Cada CA-n possui teste dedicado que falha antes da implementação (Red -> Green).

use ampforge_dsp::config::{AudioConfig, SampleRate, BufferSize};
use ampforge_dsp::errors::InvalidAudioConfigError;
use ampforge_dsp::queue::DspCommand;
use ampforge_dsp::smoother::OnePoleSmoother;
use ampforge_dsp::oversampling::{HalfBandOversampler, OversamplingFactor};
use ampforge_dsp::precision::IirBiquad64;
use ampforge_dsp::cpu::CpuMonitor;
use ampforge_dsp::engine::DspEngine;

#[test]
fn test_ca_01_internal_algorithmic_latency_under_1_5ms() {
    let config_48k = AudioConfig::new(SampleRate::Rate48000, BufferSize::Size64).unwrap();
    let mut engine_48k = DspEngine::new(config_48k).unwrap();
    engine_48k.set_oversampling(OversamplingFactor::X4);
    
    // CA-1: Latência algorítmica < 1.5 ms (< 72 samples @ 48kHz, < 144 samples @ 96kHz)
    let latency_samples_48k = engine_48k.algorithmic_latency_samples();
    let latency_ms_48k = engine_48k.algorithmic_latency_ms();
    assert!(latency_samples_48k <= 72, "Latência a 48kHz ({latency_samples_48k} samples) deve ser <= 72 samples");
    assert!(latency_ms_48k < 1.5, "Latência a 48kHz ({latency_ms_48k} ms) deve ser < 1.5 ms");

    let config_96k = AudioConfig::new(SampleRate::Rate96000, BufferSize::Size128).unwrap();
    let mut engine_96k = DspEngine::new(config_96k).unwrap();
    engine_96k.set_oversampling(OversamplingFactor::X4);
    let latency_samples_96k = engine_96k.algorithmic_latency_samples();
    let latency_ms_96k = engine_96k.algorithmic_latency_ms();
    assert!(latency_samples_96k <= 144, "Latência a 96kHz ({latency_samples_96k} samples) deve ser <= 144 samples");
    assert!(latency_ms_96k < 1.5, "Latência a 96kHz ({latency_ms_96k} ms) deve ser < 1.5 ms");
}

#[test]
fn test_ca_02_supported_sample_rates_and_buffer_sizes() {
    let valid_sample_rates = [
        SampleRate::Rate44100,
        SampleRate::Rate48000,
        SampleRate::Rate88200,
        SampleRate::Rate96000,
    ];
    let valid_buffer_sizes = [
        BufferSize::Size32,
        BufferSize::Size64,
        BufferSize::Size128,
        BufferSize::Size256,
        BufferSize::Size512,
    ];

    for &sr in &valid_sample_rates {
        for &bs in &valid_buffer_sizes {
            let config = AudioConfig::new(sr, bs);
            assert!(config.is_ok(), "Config deve ser válida para SR {:?} e BS {:?}", sr, bs);
            let mut engine = DspEngine::new(config.unwrap()).unwrap();
            
            let in_l = vec![0.1f32; bs.as_usize()];
            let in_r = vec![-0.1f32; bs.as_usize()];
            let mut out_l = vec![0.0f32; bs.as_usize()];
            let mut out_r = vec![0.0f32; bs.as_usize()];

            let res = engine.process_block(&in_l, &in_r, &mut out_l, &mut out_r);
            assert!(res.is_ok(), "Process block deve rodar sem falha para {:?} / {:?}", sr, bs);
        }
    }
}

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct CountingAlloc;

thread_local! {
    static ALLOC_COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOC_EVENTS: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNTING.with(|on| {
            if on.get() {
                ALLOC_EVENTS.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOC_COUNTING.with(|on| {
            if on.get() {
                ALLOC_EVENTS.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOC_COUNTING.with(|on| {
            if on.get() {
                ALLOC_EVENTS.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL_TEST_ALLOCATOR: CountingAlloc = CountingAlloc;

#[test]
fn test_ca_03_zero_alloc_and_zero_lock_during_process_block() {
    let config = AudioConfig::new(SampleRate::Rate48000, BufferSize::Size128).unwrap();
    let mut engine = DspEngine::new(config).unwrap();
    
    // Inundação de 1000 mensagens/s na SPSC Command Queue
    for i in 0..1000 {
        let _ = engine.command_sender().push(DspCommand::SetParameter {
            block_id: 1,
            param_id: 0,
            value: (i as f32) / 1000.0,
        });
    }

    let in_l = [0.05f32; 128];
    let in_r = [-0.05f32; 128];
    let mut out_l = [0.0f32; 128];
    let mut out_r = [0.0f32; 128];

    // Medição real com alocador global de contagem na thread do callback
    ALLOC_EVENTS.with(|c| c.set(0));
    ALLOC_COUNTING.with(|on| on.set(true));
    for _ in 0..100 {
        engine.process_block(&in_l, &in_r, &mut out_l, &mut out_r).unwrap();
    }
    ALLOC_COUNTING.with(|on| on.set(false));
    let events = ALLOC_EVENTS.with(|c| c.get());

    assert_eq!(events, 0, "Alocações/desalocações no process_block devem ser estritamente ZERO (RNF-005)");
    assert_eq!(engine.rt_lock_count(), 0, "Locks ou contenções no process_block devem ser ZERO");
}

#[test]
fn test_ca_04_mixed_precision_32bit_bus_and_64bit_solvers() {
    // Verificação de barramento 32-bit e solver/IIR com erro DC < -120 dBFS
    let mut iir_sub100hz = IirBiquad64::high_pass(48000.0, 40.0, 0.707); // corte em 40 Hz (< 100 Hz)
    
    // Aplica impulso unitário e cauda longa para acumulação de ruído numérico DC
    let mut dc_offset_accumulator = 0.0f64;
    for sample_idx in 0..96000 {
        let input = if sample_idx == 0 { 1.0f64 } else { 0.0f64 };
        let out = iir_sub100hz.process_sample(input);
        dc_offset_accumulator += out;
    }

    // Erro de quantização DC deve ser < -120 dBFS (amplitude < 10^(-120/20) = 1e-6)
    let dc_error = dc_offset_accumulator.abs();
    assert!(dc_error < 1e-6, "Erro de quantização DC {dc_error} deve ser < 1e-6 (-120 dBFS)");
}

#[test]
fn test_ca_05_polyphase_half_band_oversampling_rejection_over_96db() {
    let factors = [OversamplingFactor::X2, OversamplingFactor::X4, OversamplingFactor::X8];
    for &factor in &factors {
        let oversampler = HalfBandOversampler::new(factor);
        let stopband_attenuation_db = oversampler.stopband_attenuation_db();
        assert!(
            stopband_attenuation_db >= 96.0,
            "Atenuação na banda de corte para {:?} ({stopband_attenuation_db} dB) deve ser >= 96 dB",
            factor
        );
    }
}

#[test]
fn test_ca_06_one_pole_parameter_smoother_step_response() {
    // Variação em degrau de 0.0 para 10.0 a 48 kHz com tau = 10 ms (0.010 s)
    let sample_rate = 48000.0;
    let time_constant_sec = 0.010; // 10 ms
    let mut smoother = OnePoleSmoother::new(sample_rate, time_constant_sec, 0.0);

    smoother.set_target(10.0);

    let mut prev_val = 0.0;
    let mut max_derivative = 0.0;

    for _ in 0..480 { // 10 ms = 480 amostras
        let current_val = smoother.next_sample();
        let delta = (current_val - prev_val).abs();
        if delta > max_derivative {
            max_derivative = delta;
        }
        prev_val = current_val;
    }

    // Não pode haver salto degrau (máxima derivada deve ser suave e finita)
    assert!(max_derivative < 0.5, "Derivada ({max_derivative}) deve ser contínua e sem cliques de zipper noise");
    // Após 1 constante de tempo (1 tau), o valor deve ter alcançado ~63.2% do alvo (aprox. 6.32)
    assert!(prev_val > 5.5 && prev_val < 7.5, "Após 1 tau, valor atingiu {prev_val}, esperado ~6.32");
}

#[test]
fn test_ca_07_atomic_graph_pointer_swap_rcu() {
    let config = AudioConfig::new(SampleRate::Rate48000, BufferSize::Size64).unwrap();
    let mut engine = DspEngine::new(config).unwrap();

    let in_l = [0.1f32; 64];
    let in_r = [0.1f32; 64];
    let mut out_l = [0.0f32; 64];
    let mut out_r = [0.0f32; 64];

    // Ciclo 1 com topologia A
    engine.process_block(&in_l, &in_r, &mut out_l, &mut out_r).unwrap();
    let out_sample_before = out_l[63];

    // Troca atômica de grafo para topologia B via RCU
    engine.swap_graph_topology(1).unwrap();

    // Ciclo 2 logo após swap
    engine.process_block(&in_l, &in_r, &mut out_l, &mut out_r).unwrap();
    let out_sample_after = out_l[0];

    // Transição sem zerar buffers ou produzir NaN
    assert!(!out_sample_after.is_nan(), "Saída pós-RCU swap não deve conter NaN");
    let diff = (out_sample_after - out_sample_before).abs();
    assert!(diff < 0.5, "Diferença ({diff}) entre blocos na troca RCU deve ser suave e O(1)");
}

#[test]
fn test_ca_08_telemetry_extraction_spsc_queue_non_blocking() {
    let config = AudioConfig::new(SampleRate::Rate48000, BufferSize::Size128).unwrap();
    let mut engine = DspEngine::new(config).unwrap();

    let in_l = [0.2f32; 128];
    let in_r = [0.2f32; 128];
    let mut out_l = [0.0f32; 128];
    let mut out_r = [0.0f32; 128];

    // Inunda o extrator de telemetria sem ler da UI para encher a fila
    for _ in 0..300 {
        engine.process_block(&in_l, &in_r, &mut out_l, &mut out_r).unwrap();
    }

    // Mesmo com a fila cheia, o process_block não bloqueia e descarta o mais antigo
    let snapshot = engine.telemetry_receiver().pop();
    assert!(snapshot.is_some(), "Snapshot de telemetria deve ser consumível");
    let snap = snapshot.unwrap();
    assert!(snap.cpu_usage_pct >= 0.0 && snap.cpu_usage_pct <= 100.0, "CPU % deve ser válida");
}

#[test]
fn test_ca_09_cpu_overload_protection_at_88_percent() {
    let mut monitor = CpuMonitor::new(SampleRate::Rate48000, BufferSize::Size128);
    
    // Simula tempo de execução abaixo de 88%
    // Deadline = 128 / 48000 = 2.666 ms
    // 50% = 1.333 ms
    monitor.record_execution_time_micros(1333);
    assert!(!monitor.is_overloaded(), "50% de CPU não deve acionar proteção");
    assert!(!monitor.warning_flag());

    // Simula tempo de execução em 89% (2373 micros)
    monitor.record_execution_time_micros(2373);
    assert!(monitor.is_overloaded(), "89% de CPU DEVE acionar estado de sobrecarga (>= 88.0%)");
    assert!(monitor.warning_flag(), "Flag cpuOverloadWarning deve ser true");

    // Recusa inserção de bloco pesado
    let can_add_heavy_block = monitor.can_admit_block(5.0); // requer 5% adicionais
    assert!(!can_add_heavy_block, "Deve recusar novo bloco pesado quando CPU >= 88%");
}

#[test]
fn test_ca_10_invalid_audio_config_rejected_with_structured_error() {
    // Taxa de amostragem inválida (8000 Hz)
    let err_sr = AudioConfig::from_raw(8000, 128);
    assert!(err_sr.is_err(), "8000 Hz deve ser recusado");
    match err_sr.err().unwrap() {
        InvalidAudioConfigError::UnsupportedSampleRate(sr) => assert_eq!(sr, 8000),
        other => panic!("Esperado UnsupportedSampleRate, obtido: {:?}", other),
    }

    // Buffer fora de 32..512 (0 ou 4096)
    let err_bs_low = AudioConfig::from_raw(48000, 0);
    assert!(err_bs_low.is_err(), "Buffer 0 deve ser recusado");
    
    let err_bs_high = AudioConfig::from_raw(48000, 4096);
    assert!(err_bs_high.is_err(), "Buffer 4096 deve ser recusado");
    match err_bs_high.err().unwrap() {
        InvalidAudioConfigError::InvalidBufferSize(bs) => assert_eq!(bs, 4096),
        other => panic!("Esperado InvalidBufferSize, obtido: {:?}", other),
    }
}

#[test]
fn test_ca_11_bit_exact_deterministic_output_when_idle() {
    let config = AudioConfig::new(SampleRate::Rate48000, BufferSize::Size64).unwrap();
    let mut engine1 = DspEngine::new(config).unwrap();
    let mut engine2 = DspEngine::new(config).unwrap();

    let input_signal: Vec<f32> = (0..64).map(|i| (i as f32 * 0.1).sin()).collect();
    
    let mut out1_l = [0.0f32; 64];
    let mut out1_r = [0.0f32; 64];
    let mut out2_l = [0.0f32; 64];
    let mut out2_r = [0.0f32; 64];

    engine1.process_block(&input_signal, &input_signal, &mut out1_l, &mut out1_r).unwrap();
    engine2.process_block(&input_signal, &input_signal, &mut out2_l, &mut out2_r).unwrap();

    // Determinismo bit-a-bit (saídas idênticas sem variação)
    for i in 0..64 {
        assert_eq!(
            out1_l[i].to_bits(),
            out2_l[i].to_bits(),
            "Saída no sample {i} deve ser bit-a-bit idêntica entre duas instâncias sem comandos"
        );
        assert_eq!(
            out1_r[i].to_bits(),
            out2_r[i].to_bits(),
            "Saída R no sample {i} deve ser bit-a-bit idêntica"
        );
    }
}
