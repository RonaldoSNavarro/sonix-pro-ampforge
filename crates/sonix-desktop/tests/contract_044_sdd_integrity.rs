//! Testes de contrato da Spec 044 — Integridade do SDD, Código Morto e Testes Tautológicos.
//! Verifica CA-1, CA-2, CA-3, CA-5, CA-6, CA-7, CA-8 com réguas medidas em tempo real e estáticas.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use ampforge_dsp::oversampling::{HalfBandOversampler, OversamplingFactor};
use sonix_desktop::audio::{DspParamUpdate, RealtimeDspChain};

struct CountingAllocator;

thread_local! {
    static COUNTING_ENABLED: Cell<bool> = const { Cell::new(false) };
}

static RT_ALLOCS: AtomicUsize = AtomicUsize::new(0);
static RT_DEALLOCS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING_ENABLED.with(|c| c.get()) {
            RT_ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if COUNTING_ENABLED.with(|c| c.get()) {
            RT_DEALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

struct CountingGuard;
impl Drop for CountingGuard {
    fn drop(&mut self) {
        COUNTING_ENABLED.with(|c| c.set(false));
    }
}

fn with_counting_alloc<F: FnOnce()>(f: F) -> (usize, usize) {
    RT_ALLOCS.store(0, Ordering::SeqCst);
    RT_DEALLOCS.store(0, Ordering::SeqCst);
    COUNTING_ENABLED.with(|c| c.set(true));
    let _guard = CountingGuard;
    f();
    drop(_guard);
    (RT_ALLOCS.load(Ordering::SeqCst), RT_DEALLOCS.load(Ordering::SeqCst))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if !dir.exists() {
        return;
    }
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn contains_word(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let nbytes = needle.as_bytes();
    if nbytes.is_empty() || bytes.len() < nbytes.len() {
        return false;
    }
    let mut start = 0usize;
    while let Some(pos) = haystack[start..].find(needle) {
        let abs = start + pos;
        let left_ok = abs == 0
            || !(bytes[abs - 1].is_ascii_alphanumeric() || bytes[abs - 1] == b'_');
        let end = abs + nbytes.len();
        let right_ok = end == bytes.len()
            || !(bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_');
        if left_ok && right_ok {
            return true;
        }
        start = abs + 1;
    }
    false
}

fn extract_pub_types(src: &str) -> Vec<String> {
    let mut items = Vec::new();
    for line in src.lines() {
        let trimmed = line.trim_start();
        for kw in ["pub struct ", "pub enum ", "pub trait ", "pub type "] {
            if let Some(rest) = trimmed.strip_prefix(kw) {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    items.push(name);
                }
            }
        }
    }
    items
}

#[test]
fn test_ca_01_index_and_specs_report_honest_status_and_notes() {
    let root = workspace_root();
    let index = fs::read_to_string(root.join("specs/README.md")).unwrap();

    for spec_num in ["001", "002", "007", "008", "009", "012", "014"] {
        let prefix = format!("| [{}](", spec_num);
        let row = index
            .lines()
            .find(|l| l.starts_with(&prefix) || l.starts_with(&format!("| {} ", spec_num)))
            .unwrap_or_else(|| panic!("CA-1: Spec {} não encontrada no índice specs/README.md", spec_num));
        assert!(
            row.to_lowercase().contains("revertida") || row.to_lowercase().contains("parcial"),
            "CA-1: Linha da Spec {} em specs/README.md ainda omite status revertido/parcial: {}",
            spec_num,
            row
        );
    }

    let spec_files = [
        ("001", "specs/001-nucleo-dsp-tempo-real-ipc-lock-free.md", "044"),
        ("002", "specs/002-matriz-roteamento-sinal-dag-4x12.md", "045"),
        ("007", "specs/007-blocos-modulacao-pitch-delay-reverb.md", "042"),
        ("008", "specs/008-afinador-estroboscopico-polifonico-looper.md", "043"),
        ("009", "specs/009-cenas-gapless-canais-modifiers-footswitches.md", "045"),
        ("012", "specs/012-host-desktop-standalone-tauri-plugins-vst3-clap.md", "036"),
        ("014", "specs/014-ponte-de-controle-ui-dsp-tempo-real.md", "044"),
    ];
    for (num, rel, resume_spec) in spec_files {
        let txt = fs::read_to_string(root.join(rel)).unwrap();
        assert!(
            txt.contains("2026-11-14") && txt.contains(resume_spec),
            "CA-1: Spec {} ({}) deve registrar nota de revisão (2026-11-14) citando retomada pela Spec {}",
            num,
            rel,
            resume_spec
        );
    }
}

#[test]
fn test_ca_02_every_pub_item_in_core_crates_is_used_in_prod_or_documented() {
    let root = workspace_root();
    let doc_path = root.join("docs/biblioteca-sem-uso-em-producao.md");
    assert!(
        doc_path.exists(),
        "CA-2: docs/biblioteca-sem-uso-em-producao.md deve existir para mapear código ainda não ligado à produção"
    );
    let doc_txt = fs::read_to_string(&doc_path).unwrap();

    let mut prod_files = Vec::new();
    collect_rs_files(&root.join("crates/sonix-desktop/src"), &mut prod_files);
    collect_rs_files(&root.join("apps/sonix-app/src-tauri/src"), &mut prod_files);
    let mut prod_blob = String::new();
    for f in prod_files {
        prod_blob.push_str(&fs::read_to_string(f).unwrap());
        prod_blob.push('\n');
    }

    let mut core_files = Vec::new();
    collect_rs_files(&root.join("crates/ampforge-dsp/src"), &mut core_files);
    collect_rs_files(&root.join("crates/sonix-core/src"), &mut core_files);

    let mut undocumented = BTreeSet::new();
    for file in core_files {
        let src = fs::read_to_string(&file).unwrap();
        for name in extract_pub_types(&src) {
            if !contains_word(&prod_blob, &name) && !contains_word(&doc_txt, &name) {
                undocumented.insert(format!(
                    "{} ({})",
                    name,
                    file.strip_prefix(&root).unwrap().display()
                ));
            }
        }
    }

    assert!(
        undocumented.is_empty(),
        "CA-2: Itens pub ausentes tanto da produção quanto de docs/biblioteca-sem-uso-em-producao.md:\n{}",
        undocumented.into_iter().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn test_ca_03_counting_allocator_enforces_zero_alloc_on_process() {
    let mut chain = RealtimeDspChain::new(48_000.0);
    chain.prepare(48_000.0, 2048);

    // Garante que o alocador detecta uma alocação intencional
    let (injected_allocs, _) = with_counting_alloc(|| {
        let mut v: Vec<f32> = Vec::with_capacity(32);
        v.push(1.0);
        std::hint::black_box(&v);
    });
    assert!(
        injected_allocs >= 1,
        "CA-3: CountingAllocator precisa detectar alocações reais"
    );

    let queue = chain.param_queue();
    let _ = queue.push(DspParamUpdate::AmpGain(7.5));
    let input = [0.05f32; 256];
    let mut output = [0.0f32; 256];

    let (rt_allocs, rt_deallocs) = with_counting_alloc(|| {
        for _ in 0..200 {
            chain.process(&input, &mut output);
        }
    });
    assert_eq!(rt_allocs, 0, "CA-3: process() alocou {} vez(es)", rt_allocs);
    assert_eq!(rt_deallocs, 0, "CA-3: process() desalocou {} vez(es)", rt_deallocs);
}

#[test]
fn test_ca_05_oversampler_stopband_attenuation_matches_measured_dft() {
    for factor in [
        OversamplingFactor::X2,
        OversamplingFactor::X4,
        OversamplingFactor::X8,
    ] {
        let reported_db = HalfBandOversampler::new(factor).stopband_attenuation_db();
        assert!(
            reported_db >= 96.0,
            "CA-5: Rejeição medida para {:?} ({:.2} dB) deve ser >= 96 dB",
            factor,
            reported_db
        );

        // Medição DFT independente sobre a resposta ao impulso real
        let m = factor.multiplier();
        let mut dut = HalfBandOversampler::new(factor);
        let base_len = 256usize;
        let mut ir = vec![0.0f64; base_len * m];
        let mut tmp = [0.0f32; 8];
        for n in 0..base_len {
            dut.upsample_sample(if n == 0 { 1.0 } else { 0.0 }, &mut tmp[..m]);
            for k in 0..m {
                ir[n * m + k] = tmp[k] as f64;
            }
        }
        let dc_mag = ir.iter().sum::<f64>().abs();
        let start_w = 1.20 * std::f64::consts::PI / (m as f64);
        let end_w = std::f64::consts::PI;
        let mut max_stop = 1e-15f64;
        for b in 0..=256 {
            let w = start_w + (end_w - start_w) * (b as f64 / 256.0);
            let mut re = 0.0f64;
            let mut im = 0.0f64;
            for (n, &h) in ir.iter().enumerate() {
                if h == 0.0 {
                    continue;
                }
                let ang = w * (n as f64);
                re += h * ang.cos();
                im -= h * ang.sin();
            }
            max_stop = max_stop.max((re * re + im * im).sqrt());
        }
        let independent_db = 20.0 * (dc_mag / max_stop).log10();
        assert!(
            (reported_db - independent_db).abs() < 0.5,
            "CA-5: Valor reportado ({:.2} dB) difere da DFT independente ({:.2} dB) para {:?}",
            reported_db,
            independent_db,
            factor
        );
    }
}

#[test]
fn test_ca_06_cpu_monitor_receives_real_callback_time() {
    let mut chain = RealtimeDspChain::new(48_000.0);
    chain.prepare(48_000.0, 256);
    assert_eq!(chain.cpu_usage_pct(), 0.0);

    let input = [0.05f32; 256];
    let mut output = [0.0f32; 256];
    for _ in 0..16 {
        chain.process(&input, &mut output);
    }

    assert!(
        chain.cpu_usage_pct() > 0.0 && chain.cpu_monitor().cpu_usage_pct() > 0.0,
        "CA-6: CpuMonitor da cadeia ao vivo deve receber tempo real de execução em process() (obtido {:.4}%)",
        chain.cpu_usage_pct()
    );
}

#[test]
fn test_ca_07_control_inventory_cites_existing_tests_and_marks_pending() {
    let root = workspace_root();
    let inv = fs::read_to_string(root.join("docs/ui-desktop/inventario-controles.md")).unwrap();
    let index = fs::read_to_string(root.join("specs/README.md")).unwrap();

    // Mapa Spec NNN -> coluna de status em specs/README.md (Spec 049 CA-15)
    let mut implemented_specs: BTreeSet<String> = BTreeSet::new();
    for line in index.lines() {
        if !line.starts_with("| [") {
            continue;
        }
        let cols: Vec<&str> = line.split('|').collect();
        if cols.len() < 4 {
            continue;
        }
        let spec_num = cols[1]
            .trim()
            .trim_start_matches('[')
            .split(']')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        let status = cols[3].trim().to_ascii_lowercase();
        if status.starts_with("implementada") {
            implemented_specs.insert(spec_num);
        }
    }

    // Spec 049 CA-15: nenhum `pendente — Spec NNN` pode apontar para uma spec já marcada `implementada` no índice.
    for line in inv.lines() {
        if !line.starts_with('|') {
            continue;
        }
        let mut search_from = 0usize;
        while let Some(pos) = line[search_from..].find("pendente — Spec ") {
            let abs = search_from + pos + "pendente — Spec ".len();
            let digits: String = line[abs..].chars().take_while(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                assert!(
                    !implemented_specs.contains(&digits),
                    "CA-7 / Spec 049 CA-15: inventário marca `pendente — Spec {}` mas a Spec {} já está Implementada em specs/README.md (linha: {})",
                    digits,
                    digits,
                    line
                );
            }
            search_from = abs;
        }
    }

    // Spec 049 CA-15: Notas das Specs 002, 009 e 012 apontam para Spec 049 onde havia pendência anterior.
    for spec_rel in [
        "specs/002-matriz-roteamento-sinal-dag-4x12.md",
        "specs/009-cenas-gapless-canais-modifiers-footswitches.md",
        "specs/012-host-desktop-standalone-tauri-plugins-vst3-clap.md",
    ] {
        let content = fs::read_to_string(root.join(spec_rel)).unwrap();
        assert!(
            content.contains("Spec 049"),
            "Spec 049 CA-15: {} deve referenciar Spec 049 nas Notas",
            spec_rel
        );
    }

    // Toda linha marcada "OK" precisa citar pelo menos um arquivo de teste que existe em disco
    for line in inv.lines() {
        if !line.starts_with('|') || !line.contains("**OK") {
            continue;
        }
        let mut found_test_path = false;
        for token in line.split(|c: char| matches!(c, '`' | ' ' | ',' | ';' | '(' | ')' | '|')) {
            if (token.starts_with("crates/")
                || token.starts_with("apps/")
                || token.starts_with("tests/"))
                && (token.ends_with(".rs") || token.ends_with(".js"))
            {
                assert!(
                    root.join(token).exists(),
                    "CA-7: Caminho de teste citado no inventário não existe em disco: {} (linha: {})",
                    token,
                    line
                );
                found_test_path = true;
            }
        }
        assert!(
            found_test_path,
            "CA-7: Linha marcada OK no inventário não cita caminho de teste de efeito verificável: {}",
            line
        );
    }
}

#[test]
fn test_ca_08_spec_012_and_readme_do_not_claim_delivered_vst3_clap() {
    let root = workspace_root();
    let readme = fs::read_to_string(root.join("README.md")).unwrap();
    let index = fs::read_to_string(root.join("specs/README.md")).unwrap();

    assert!(
        !readme.contains("Plugins de DAW 64-bit (`VST3` e `CLAP`)**: Integração direta"),
        "CA-8: README.md ainda afirma plugins VST3 e CLAP entregues em produção"
    );
    let spec12_row = index
        .lines()
        .find(|l| l.starts_with("| [012](") || l.starts_with("| 012 "))
        .expect("Linha da Spec 012 deve existir em specs/README.md");
    assert!(
        spec12_row.contains("passthrough") || spec12_row.contains("sem binário VST3/CLAP") || spec12_row.contains("revertida"),
        "CA-8: Linha da Spec 012 em specs/README.md precisa informar honestamente o estado passthrough/sem binário VST3/CLAP: {}",
        spec12_row
    );
}

#[test]
fn ca4_suavizacao_014_atinge_63_porcento_entre_5_e_20ms() {
    // AUD-36: Spec 044 CA-4
    let sr = 48_000.0_f32;
    let mut chain = RealtimeDspChain::new(sr);
    chain.prepare(sr, 64);
    chain.set_gate_enabled(false);
    chain.set_cab_enabled(false);
    chain.set_amp_enabled(true);
    chain.set_amp_model("twin_ab763");
    chain.set_amp_gain(2.0);

    let blk = 64usize;
    let mut phase = 0usize;
    let mut next_sine = |buf: &mut [f32]| {
        for s in buf.iter_mut() {
            *s = 0.02 * (2.0 * std::f32::consts::PI * 1000.0 * (phase as f32) / sr).sin();
            phase += 1;
        }
    };
    let mut inp = vec![0.0f32; blk];
    let mut out = vec![0.0f32; blk];
    let rms_of = |s: &[f32]| -> f64 {
        let sum: f64 = s.iter().map(|&v| (v as f64) * (v as f64)).sum();
        (sum / s.len().max(1) as f64).sqrt()
    };

    for _ in 0..40 {
        next_sine(&mut inp);
        chain.process(&inp, &mut out);
    }
    let level_low = rms_of(&out);

    let q = chain.param_queue();
    let _ = q.push(DspParamUpdate::AmpGain(8.0));

    let mut env_at_3ms = 0.0_f64;
    let mut env_at_18ms = 0.0_f64;
    for b in 0..80 {
        next_sine(&mut inp);
        chain.process(&inp, &mut out);
        let lvl = rms_of(&out);
        if b == 2 {
            // ~4 ms
            env_at_3ms = lvl;
        }
        if b == 13 {
            // ~18.7 ms
            env_at_18ms = lvl;
        }
    }
    let level_high = rms_of(&out);
    let span = (level_high - level_low).abs().max(1e-6);
    let pct_4ms = (env_at_3ms - level_low) / span;
    let pct_18ms = (env_at_18ms - level_low) / span;

    assert!(
        pct_4ms < 0.63 && pct_18ms >= 0.63,
        "CA-4 (AUD-36): Suavização deve estar < 63% em <5ms ({:.2}%) e >= 63% em <=20ms ({:.2}%)",
        pct_4ms * 100.0,
        pct_18ms * 100.0
    );
}

#[test]
fn ca15_inventario_sem_pendencia_de_spec_implementada() {
    // AUD-36: Spec 049 CA-15
    let root = workspace_root();
    let index = fs::read_to_string(root.join("specs/README.md")).unwrap();
    let inv = fs::read_to_string(root.join("docs/ui-desktop/inventario-controles.md")).unwrap();

    let mut implemented_specs = BTreeSet::new();
    for line in index.lines() {
        if line.starts_with("| [") && line.to_lowercase().contains("implementada") {
            if let Some(id) = line.split('[').nth(1).and_then(|s| s.split(']').next()) {
                implemented_specs.insert(id.trim().to_string());
            }
        }
    }
    for line in inv.lines() {
        if !line.starts_with('|') {
            continue;
        }
        if line.to_lowercase().contains("pendente") {
            for spec_id in &implemented_specs {
                assert!(
                    !line.contains(&format!("Spec {spec_id}")),
                    "CA-15 (AUD-36): Inventário marca como pendente item da spec implementada {spec_id}: {line}"
                );
            }
        }
    }
}
