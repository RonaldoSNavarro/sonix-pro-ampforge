//! Bancada de auditoria de desempenho em modo standalone offline (§2.4).
//!
//! Execução:
//! `cargo run --release -p sonix-desktop --example bancada_auditoria -- <cenario> <sr> <bloco>`

use std::hint::black_box;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use ampforge_dsp::regua::contar_subnormais;
use sonix_desktop::api::{self, ApiContext, AppState};
use sonix_desktop::audio::{RealtimeDspChain, TelemetryBuffer};

fn build_pcm16_ir_wav(sr: u32, seconds: f32) -> Vec<u8> {
    let n = ((sr as f32) * seconds).round().max(1.0) as usize;
    let data_bytes = (n * 2) as u32;
    let riff_size = 36 + data_bytes;
    let mut wav = Vec::with_capacity(44 + data_bytes as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // Mono
    wav.extend_from_slice(&sr.to_le_bytes());
    wav.extend_from_slice(&(sr * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_bytes.to_le_bytes());
    for i in 0..n {
        let t = (i as f32) / (sr as f32);
        let env = (-6.0 * t).exp();
        let val = if i == 0 {
            0.9
        } else {
            env * (0.4 * (2.0 * std::f32::consts::PI * 190.0 * t).sin()
                + 0.25 * (2.0 * std::f32::consts::PI * 730.0 * t).cos())
        };
        let q = (val.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        wav.extend_from_slice(&q.to_le_bytes());
    }
    wav
}

fn riff_sample(sample_idx: usize, sr: f32) -> f32 {
    let t = (sample_idx as f32) / sr;
    let env = 0.25_f32;
    env * (0.50 * (2.0 * std::f32::consts::PI * 110.0 * t).sin()
        + 0.30 * (2.0 * std::f32::consts::PI * 220.0 * t).sin()
        + 0.20 * (2.0 * std::f32::consts::PI * 330.0 * t).sin()
        + 0.12 * (2.0 * std::f32::consts::PI * 880.0 * t).sin())
}

fn configurar_cadeia(cenario: &str, sr: u32, bloco: usize) -> Arc<Mutex<RealtimeDspChain>> {
    let mut chain = RealtimeDspChain::new(sr as f32);
    chain.prepare(sr as f32, bloco);
    let pq = chain.param_queue();
    let dsp = Arc::new(Mutex::new(chain));
    let ctx = ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(512)),
        Some(Arc::clone(&dsp)),
        Some(pq),
        None,
        None,
    );

    match cenario {
        "pesada" => {
            let wav = build_pcm16_ir_wav(sr, 0.49);
            let (ir_st, _, ir_body) = api::dispatch(&ctx, "/api/cab/ir", "POST", &wav);
            assert_eq!(
                ir_st,
                200,
                "Falha ao carregar IR 0.49 s: {}",
                String::from_utf8_lossy(&ir_body)
            );
            let params = serde_json::json!({
                "drive.enabled": true,
                "drive.model": "ts808",
                "drive.drive": 6.5,
                "mod_2.enabled": true,
                "mod_2.model": "tz_flanger",
                "mod_2.mix": 50.0,
                "eq_1.enabled": true,
                "dly_1.enabled": true,
                "dly_1.intensity": 6.5,
                "rev_1.enabled": true,
                "rev_1.decay_s": 2.5,
                "rev_1.mix": 45.0,
                "split.enabled": true,
                "split.mode": "ab",
                "amp_2.enabled": true,
                "amp_2.model": "twin_ab763",
                "cab_2.enabled": true,
                "cab_1.ir_enabled": true
            })
            .to_string();
            let (st, _, body) = api::dispatch(&ctx, "/api/params", "POST", params.as_bytes());
            assert_eq!(
                st,
                200,
                "Falha ao configurar cenário pesada: {}",
                String::from_utf8_lossy(&body)
            );
        }
        "silencio" => {
            let params = serde_json::json!({
                "gate.enabled": false,
                "drive.enabled": true,
                "drive.model": "ts808",
                "mod_2.enabled": true,
                "mod_2.model": "phaser_90",
                "mod_2.mix": 60.0,
                "dly_1.enabled": true,
                "dly_1.intensity": 7.0,
                "dly_1.echo_volume": 6.0,
                "dly_1.reverb_volume": 5.0,
                "rev_1.enabled": true,
                "rev_1.decay_s": 2.5,
                "rev_1.mix": 60.0
            })
            .to_string();
            let (st, _, body) = api::dispatch(&ctx, "/api/params", "POST", params.as_bytes());
            assert_eq!(
                st,
                200,
                "Falha ao configurar cenário silencio: {}",
                String::from_utf8_lossy(&body)
            );
        }
        "pitch" => {
            let params = serde_json::json!({
                "pitch_1.enabled": true,
                "pitch_1.model": "harmonizer",
                "pitch_1.mix": 50.0
            })
            .to_string();
            let (st, _, body) = api::dispatch(&ctx, "/api/params", "POST", params.as_bytes());
            assert_eq!(
                st,
                200,
                "Falha ao configurar cenário pitch: {}",
                String::from_utf8_lossy(&body)
            );
        }
        _ => {}
    }

    dsp
}

fn rodar_cenario(cenario: &str, sr: u32, bloco: usize) {
    let budget_us = (bloco as f64) / (sr as f64) * 1_000_000.0;
    let total_secs: f64 = if cenario == "silencio" { 242.0 } else { 10.0 };
    let total_blocks = ((total_secs * sr as f64) / (bloco as f64)).ceil() as usize;
    let signal_end_sample = if cenario == "silencio" {
        (2.0 * sr as f64).round() as usize
    } else {
        usize::MAX
    };

    let mut timings_us = Vec::with_capacity(total_blocks);
    let mut subnormais: usize = 0;

    if cenario == "ruido" {
        for _ in 0..total_blocks {
            let t0 = Instant::now();
            while t0.elapsed().as_secs_f64() * 1_000_000.0 < 45.0 {
                black_box(0usize);
            }
            let elapsed_us = t0.elapsed().as_secs_f64() * 1_000_000.0;
            timings_us.push(elapsed_us);
        }
    } else {
        let dsp = configurar_cadeia(cenario, sr, bloco);
        let mut chain = dsp.lock().expect("dsp lock");
        let mut in_buf = vec![0.0_f32; bloco];
        let mut out_buf = vec![0.0_f32; bloco];

        // 1 bloco de aquecimento para drenar filas de parâmetros/recursos fora da medição
        chain.process(&in_buf, &mut out_buf);

        let mut sample_cursor: usize = 0;
        for _ in 0..total_blocks {
            for i in 0..bloco {
                let idx = sample_cursor + i;
                in_buf[i] = if idx < signal_end_sample {
                    riff_sample(idx, sr as f32)
                } else {
                    0.0
                };
            }
            sample_cursor += bloco;

            let t0 = Instant::now();
            chain.process(black_box(&in_buf), black_box(&mut out_buf));
            let elapsed_us = t0.elapsed().as_secs_f64() * 1_000_000.0;
            black_box(out_buf[0]);
            timings_us.push(elapsed_us);
            subnormais += contar_subnormais(&out_buf);
        }
    }

    let n = timings_us.len().max(1);
    let mean_us = timings_us.iter().sum::<f64>() / (n as f64);
    let max_us = timings_us.iter().copied().fold(0.0_f64, f64::max);

    let mut sorted = timings_us.clone();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let p99_idx = ((sorted.len() as f64) * 0.99).ceil() as usize;
    let p99_us = sorted[p99_idx.min(sorted.len() - 1)];

    let avg_pct = (mean_us / budget_us) * 100.0;
    let p99_pct = (p99_us / budget_us) * 100.0;
    let max_pct = (max_us / budget_us) * 100.0;

    let mut report = serde_json::json!({
        "cenario": cenario,
        "sr": sr,
        "bloco": bloco,
        "blocos": total_blocks,
        "orcamento_us": (budget_us * 1000.0).round() / 1000.0,
        "media_us": (mean_us * 1000.0).round() / 1000.0,
        "p99_us": (p99_us * 1000.0).round() / 1000.0,
        "max_us": (max_us * 1000.0).round() / 1000.0,
        "media_pct": (avg_pct * 1000.0).round() / 1000.0,
        "p99_pct": (p99_pct * 1000.0).round() / 1000.0,
        "max_pct": (max_pct * 1000.0).round() / 1000.0,
        "subnormais": subnormais,
    });

    if cenario == "silencio" {
        let blocks_per_10s = ((10.0 * sr as f64) / (bloco as f64)).round() as usize;
        let blocks_2s = ((2.0 * sr as f64) / (bloco as f64)).round() as usize;
        let first_end = blocks_per_10s.min(n);
        let first_10s_mean = timings_us[..first_end].iter().sum::<f64>() / (first_end.max(1) as f64);

        let tail_start = blocks_2s.min(n.saturating_sub(1));
        let tail_end = (tail_start + blocks_per_10s).min(n);
        let tail_first_10s_mean = timings_us[tail_start..tail_end]
            .iter()
            .sum::<f64>()
            / ((tail_end - tail_start).max(1) as f64);

        let last_start = n.saturating_sub(blocks_per_10s);
        let last_10s_mean =
            timings_us[last_start..].iter().sum::<f64>() / ((n - last_start).max(1) as f64);

        let ratio = if first_10s_mean > 0.0 {
            last_10s_mean / first_10s_mean
        } else {
            1.0
        };
        let tail_ratio = if tail_first_10s_mean > 0.0 {
            last_10s_mean / tail_first_10s_mean
        } else {
            1.0
        };

        if let Some(obj) = report.as_object_mut() {
            obj.insert(
                "primeira_janela_10s_us".to_string(),
                serde_json::json!((first_10s_mean * 1000.0).round() / 1000.0),
            );
            obj.insert(
                "primeira_janela_zeros_10s_us".to_string(),
                serde_json::json!((tail_first_10s_mean * 1000.0).round() / 1000.0),
            );
            obj.insert(
                "ultima_janela_10s_us".to_string(),
                serde_json::json!((last_10s_mean * 1000.0).round() / 1000.0),
            );
            obj.insert(
                "razao_ultima_sobre_primeira".to_string(),
                serde_json::json!((ratio * 1000.0).round() / 1000.0),
            );
            obj.insert(
                "razao_ultima_sobre_zeros_iniciais".to_string(),
                serde_json::json!((tail_ratio * 1000.0).round() / 1000.0),
            );
        }
    }

    println!("{}", report);
    let _ = std::io::stdout().flush();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cenario = args.get(1).map(|s| s.as_str()).unwrap_or("padrao");
    let sr: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(48_000);
    let bloco: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(128);

    match cenario {
        "padrao" | "pesada" | "silencio" | "pitch" | "ruido" => {}
        other => {
            eprintln!("Cenário desconhecido '{}'. Use: padrao | pesada | silencio | pitch | ruido", other);
            std::process::exit(2);
        }
    }

    rodar_cenario(cenario, sr, bloco);
    if cenario != "ruido" {
        rodar_cenario("ruido", sr, bloco);
    }
}
