// AUD-39 + Spec 053: Arquitetura DSP Modular e Contratos de Processadores
// Cobre:
// CA-1: Contratos e Descritores de Processador no ProcessorRegistry
// CA-2: Independência do Executor do Grafo (&mut [Box<dyn AudioProcessor>])
// CA-3: Validação Atômica por Modelo
// CA-4: Invariância a Tamanho de Bloco (AUD-39)
// CA-5: Compensação de Latência Paralela (PDC)
// CA-6: Fronteira Off-RT vs Zero-Alloc RT Medida
// CA-9: Headroom Interno > 0 dBFS e Sanitização Não-Finita
use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::amps::register_amp_processors;
use ampforge_dsp::cab::VirtualCabProcessor;
use ampforge_dsp::dag::ModularParallelExecutor;
use ampforge_dsp::fx::delay::{DelayModel, DelayProcessor};
use ampforge_dsp::fx::drive::{DriveModel, DriveProcessor};
use ampforge_dsp::fx::reverb::{ReverbModel, ReverbProcessor};
use ampforge_dsp::processor::{
    AudioBlockMut, AudioBlockRef, AudioProcessor, AudioSpec, ChannelLayout, DspError,
    ModelDescriptor, ParamDescriptor, ParamError, ParamId, ParamTaper, ParamValue, ProcessContext,
    ProcessorRegistry,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct CountingAlloc;

thread_local! {
    static COUNTING_ACTIVE: Cell<bool> = const { Cell::new(false) };
    static ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
    static DEALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        COUNTING_ACTIVE.with(|a| {
            if a.get() {
                ALLOC_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        COUNTING_ACTIVE.with(|a| {
            if a.get() {
                DEALLOC_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn run_chain(heavy: bool, block_size: usize, input: &[f32]) -> Vec<f32> {
    let sr = 48_000.0;
    let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive.set_sample_rate(sr);
    drive.set_gain(5.5);

    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp.set_sample_rate(sr);
    amp.set_oversampling_factor(4);
    amp.panel_mut().set_input_gain(6.0);

    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);

    let mut delay = DelayProcessor::new(DelayModel::PristineDigitalDelay);
    delay.set_sample_rate(sr);
    delay.set_delay_time_ms(250.0);
    delay.set_mix_pct(25.0);

    let mut reverb = ReverbProcessor::new(ReverbModel::StudioVintagePlate);
    reverb.set_sample_rate(sr);
    reverb.set_mix_pct(20.0);

    let mut out = vec![0.0_f32; input.len()];
    let mut tmp_a = vec![0.0_f32; block_size];
    let mut tmp_b = vec![0.0_f32; block_size];

    for (chunk_in, chunk_out) in input.chunks(block_size).zip(out.chunks_mut(block_size)) {
        let n = chunk_in.len();
        if heavy {
            drive.process(chunk_in, &mut tmp_a[..n]);
            amp.process(&tmp_a[..n], &mut tmp_b[..n]);
            cab.process(&tmp_b[..n], &mut tmp_a[..n]);
            delay.process(&tmp_a[..n], &mut tmp_b[..n]);
            reverb.process(&tmp_b[..n], chunk_out);
        } else {
            amp.process(chunk_in, chunk_out);
        }
    }
    out
}

#[test]
fn ca4_invariancia_ao_tamanho_de_bloco() {
    // AUD-39
    let sr = 48_000.0_f32;
    let total = 4800usize; // 100 ms
    let input: Vec<f32> = (0..total)
        .map(|i| 0.25 * (2.0 * std::f32::consts::PI * 220.0 * (i as f32) / sr).sin())
        .collect();

    for heavy in [false, true] {
        let ref_128 = run_chain(heavy, 128, &input);
        for blk in [32usize, 37, 64, 256, 480, 512, 1024] {
            let got = run_chain(heavy, blk, &input);
            assert_eq!(got.len(), ref_128.len());
            for i in 0..total {
                assert_eq!(
                    got[i].to_bits(),
                    ref_128[i].to_bits(),
                    "AUD-39 (heavy={heavy}): bloco {blk} divergiu de 128 no sample {i}: got={} ref={}",
                    got[i],
                    ref_128[i]
                );
            }
        }
    }
}

struct FixedDelayTestProc {
    delay: usize,
    ring: Vec<f32>,
    pos: usize,
}

impl FixedDelayTestProc {
    const PARAMS: &[ParamDescriptor] = &[ParamDescriptor {
        id: 1,
        key: "test.gain",
        label: "Gain",
        unit: "lin",
        min: 0.0,
        max: 4.0,
        default: 1.0,
        taper: ParamTaper::Linear,
        enum_variants: &[],
    }];
    const DESC: ModelDescriptor = ModelDescriptor {
        module_id: "test.fixed_delay",
        model_revision: "v1",
        display_name: "Fixed Delay Test",
        layout: ChannelLayout::Mono,
        params: Self::PARAMS,
    };

    fn new(delay: usize) -> Self {
        Self {
            delay,
            ring: vec![0.0; delay.max(1) + 4],
            pos: 0,
        }
    }
}

impl AudioProcessor for FixedDelayTestProc {
    fn descriptor(&self) -> &ModelDescriptor {
        &Self::DESC
    }
    fn prepare(&mut self, _spec: AudioSpec) -> Result<(), DspError> {
        Ok(())
    }
    fn reset(&mut self) {
        self.ring.fill(0.0);
        self.pos = 0;
    }
    fn apply_param_rt(&mut self, id: ParamId, _val: ParamValue) -> Result<(), ParamError> {
        if id == 1 { Ok(()) } else { Err(ParamError::UnknownParamId(id)) }
    }
    fn process(&mut self, _ctx: &ProcessContext, input: &AudioBlockRef<'_>, output: &mut AudioBlockMut<'_>) {
        let inp = input.channel(0);
        let out = output.channel_mut(0);
        if self.delay == 0 {
            out.copy_from_slice(inp);
            return;
        }
        let n = self.ring.len();
        for i in 0..inp.len() {
            let r = (self.pos + n - self.delay) % n;
            out[i] = self.ring[r];
            self.ring[self.pos] = inp[i];
            self.pos = (self.pos + 1) % n;
        }
    }
    fn latency_samples(&self) -> usize {
        self.delay
    }
}

#[test]
fn ca1_and_ca2_registry_descriptors_and_generic_execution() {
    let mut reg = ProcessorRegistry::new();
    register_amp_processors(&mut reg);
    assert_eq!(reg.len(), 5);

    let spec = AudioSpec::new(48_000, 128, ChannelLayout::Mono).unwrap();
    for id in ["plexi_1959", "jcm800_2203", "twin_ab763", "dual_rect", "ac30_tb"] {
        let mut proc = reg.create(id, spec).expect("modelo registrado");
        let desc = proc.descriptor();
        assert!(!desc.module_id.is_empty());
        assert!(!desc.model_revision.is_empty());
        assert!(!desc.params.is_empty());

        let ctx = ProcessContext::default();
        let inp = [0.1f32; 128];
        let mut out = [0.0f32; 128];
        let ib = AudioBlockRef::from_mono(&inp);
        let mut ob = AudioBlockMut::from_mono(&mut out);
        proc.process(&ctx, &ib, &mut ob);
        assert!(out.iter().all(|v| v.is_finite()));
    }
}

#[test]
fn ca3_atomic_batch_validation_against_descriptor() {
    let mut reg = ProcessorRegistry::new();
    register_amp_processors(&mut reg);
    let desc = reg.descriptor("ac30_tb").unwrap();

    // Rejeita amp_1.master no AC30
    let err = desc.validate_batch(&[
        ("amp_1.gain", ParamValue::Float(5.0)),
        ("amp_1.master", ParamValue::Float(5.0)),
    ]);
    assert!(matches!(err, Err(ParamError::UnknownParamKey(_))));

    // Aceita amp_1.volume_2 no AC30 (conciliado Spec 058)
    let ok = desc.validate_batch(&[
        ("amp_1.gain", ParamValue::Float(5.5)),
        ("amp_1.volume_2", ParamValue::Float(4.5)),
    ]);
    assert!(ok.is_ok());
}

#[test]
fn ca5_parallel_latency_compensation_zero_sample_error() {
    let mut exec = ModularParallelExecutor::new(128, 256, 48_000.0);
    let mut branch_a: Vec<Box<dyn AudioProcessor>> = vec![Box::new(FixedDelayTestProc::new(64))];
    let mut branch_b: Vec<Box<dyn AudioProcessor>> = vec![Box::new(FixedDelayTestProc::new(0))];

    let mut inp = [0.0f32; 128];
    inp[0] = 1.0;
    let mut out = [0.0f32; 128];
    let ctx = ProcessContext::default();

    exec.process_parallel_mono(&ctx, &mut branch_a, &mut branch_b, &inp, &mut out);
    assert_eq!(exec.total_algorithmic_latency_samples(), 64);

    // Único pico alinhado em n = 64 (amplitude 1.0), zero pré-eco ou pico duplo!
    assert!((out[64] - 1.0).abs() < 1e-5, "out[64] = {}", out[64]);
    for (i, &v) in out.iter().enumerate() {
        if i != 64 {
            assert!(v.abs() < 1e-5, "vazamento temporal em i={}: {}", i, v);
        }
    }
}

#[test]
fn ca6_zero_alloc_rt_boundary_with_counting_allocator() {
    let mut reg = ProcessorRegistry::new();
    register_amp_processors(&mut reg);
    let spec = AudioSpec::new(48_000, 128, ChannelLayout::Mono).unwrap();
    let mut proc = reg.create("jcm800_2203", spec).unwrap();
    let ctx = ProcessContext::default();
    let inp = [0.05f32; 128];
    let mut out = [0.0f32; 128];

    ALLOC_COUNT.with(|c| c.set(0));
    DEALLOC_COUNT.with(|c| c.set(0));
    COUNTING_ACTIVE.with(|a| a.set(true));

    for _ in 0..200 {
        let _ = proc.apply_param_rt(3, ParamValue::Float(7.0));
        let ib = AudioBlockRef::from_mono(&inp);
        let mut ob = AudioBlockMut::from_mono(&mut out);
        proc.process(&ctx, &ib, &mut ob);
    }

    COUNTING_ACTIVE.with(|a| a.set(false));
    let allocs = ALLOC_COUNT.with(|c| c.get());
    let deallocs = DEALLOC_COUNT.with(|c| c.get());
    assert_eq!(allocs, 0, "alocação RT detectada: {}", allocs);
    assert_eq!(deallocs, 0, "desalocação RT detectada: {}", deallocs);
}

#[test]
fn ca9_internal_headroom_above_zero_dbfs_and_nan_sanitization() {
    let mut reg = ProcessorRegistry::new();
    register_amp_processors(&mut reg);
    let spec = AudioSpec::new(48_000, 512, ChannelLayout::Mono).unwrap();
    let mut amp = reg.create("twin_ab763", spec).unwrap();
    amp.apply_param_rt(3, ParamValue::Float(9.5)).unwrap(); // PID_GAIN
    amp.apply_param_rt(20, ParamValue::Float(12.0)).unwrap(); // PID_OUTPUT_LEVEL_DB

    let mut inp = [0.0f32; 512];
    for (i, s) in inp.iter_mut().enumerate() {
        *s = 0.7079 * (2.0 * std::f32::consts::PI * 1000.0 * (i as f32) / 48_000.0).sin();
    }
    let mut out = [0.0f32; 512];
    let ctx = ProcessContext::default();
    amp.process(
        &ctx,
        &AudioBlockRef::from_mono(&inp),
        &mut AudioBlockMut::from_mono(&mut out),
    );

    let max_abs = out.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
    assert!(max_abs > 1.0, "esperado headroom interno > 1.0 (0 dBFS), obtido {}", max_abs);

    // Sanitização NaN / Inf
    let bad = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1e6, -1e6, 0.1, 0.1, 0.1];
    let mut bad_out = [0.0f32; 8];
    amp.process(
        &ctx,
        &AudioBlockRef::from_mono(&bad),
        &mut AudioBlockMut::from_mono(&mut bad_out),
    );
    assert!(bad_out.iter().all(|v| v.is_finite()));
}

#[test]
fn modular_amps_register_and_honor_oversampling_and_latency() {
    let mut reg = ProcessorRegistry::new();
    register_amp_processors(&mut reg);

    let mut inp = vec![0.0f32; 512];
    for (i, s) in inp.iter_mut().enumerate() {
        *s = 0.05 * (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / 48_000.0).sin();
    }

    let spec = AudioSpec::new(48_000, 128, ChannelLayout::Mono).unwrap();
    for id in ["plexi_1959", "jcm800_2203", "twin_ab763", "dual_rect", "ac30_tb"] {
        let mut proc = reg.create(id, spec).expect("registrar amp modular");
        let mut prev_lat = None;
        for os in [1usize, 2, 4, 8] {
            let ctx = ProcessContext {
                oversampling_factor: os,
                ..ProcessContext::default()
            };
            let mut rms_acc = 0.0f64;
            for chunk in inp.chunks(128) {
                let mut out_chunk = [0.0f32; 128];
                let ib = AudioBlockRef::from_mono(chunk);
                let mut ob = AudioBlockMut::from_mono(&mut out_chunk);
                proc.process(&ctx, &ib, &mut ob);
                assert!(out_chunk.iter().all(|v| v.is_finite()), "{id} os={os} gerou não-finito");
                rms_acc += out_chunk.iter().map(|&v| (v as f64) * (v as f64)).sum::<f64>();
            }
            assert!(rms_acc > 1e-6, "{id} os={os} produziu silêncio inesperado");
            let lat = proc.latency_samples();
            if os == 1 {
                assert_eq!(lat, 0, "{id} 1x OS deve ter latência 0");
            }
            if let Some(p) = prev_lat {
                assert!(lat >= p, "{id} latência deve ser monotônica com OS ({p} -> {lat})");
            }
            prev_lat = Some(lat);
        }
    }
}
