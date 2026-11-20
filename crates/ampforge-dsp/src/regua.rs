//! Réguas de medição para testes e auditoria de DSP (`ampforge_dsp::regua`).
//!
//! **AVISO:** Módulo de medição para testes e bancada; **proibido no caminho de tempo real**
//! (realiza acumuladores offline em `f64` sobre buffers completos).

use std::f64::consts::PI;

/// Retorna o ganho em dB na frequência `f` Hz usando DFT exata de um bin com janela contendo
/// número inteiro de ciclos, descartando os primeiros 0,5 s de transitório.
///
/// Retorna `20 * log10(|Y| / |X|)`. Em sinais silenciosos retorna `f64::NEG_INFINITY` (ou `0.0`
/// quando entrada e saída são idênticas).
pub fn ganho_db(entrada: &[f32], saida: &[f32], sr: f32, f: f32) -> f64 {
    let n_total = entrada.len().min(saida.len());
    if n_total == 0 || sr <= 0.0 || f <= 0.0 {
        return f64::NEG_INFINITY;
    }

    let skip = (0.5_f64 * sr as f64).round() as usize;
    let start = if skip < n_total { skip } else { 0 };
    let avail = n_total - start;
    if avail < 4 {
        return f64::NEG_INFINITY;
    }

    let period = (sr as f64) / (f as f64);
    let win_len = if period >= 1.0 {
        let cycles = ((avail as f64) / period).floor();
        if cycles >= 1.0 {
            ((cycles * period).round() as usize).clamp(1, avail)
        } else {
            avail
        }
    } else {
        avail
    };

    let omega = 2.0 * PI * (f as f64) / (sr as f64);
    let mut rx = 0.0_f64;
    let mut ix = 0.0_f64;
    let mut ry = 0.0_f64;
    let mut iy = 0.0_f64;

    for (k, (&x_s, &y_s)) in entrada[start..start + win_len]
        .iter()
        .zip(&saida[start..start + win_len])
        .enumerate()
    {
        let theta = omega * (k as f64);
        let c = theta.cos();
        let s = theta.sin();
        let xv = x_s as f64;
        let yv = y_s as f64;
        rx += xv * c;
        ix -= xv * s;
        ry += yv * c;
        iy -= yv * s;
    }

    let mag_x = rx.hypot(ix);
    let mag_y = ry.hypot(iy);
    if mag_x <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if mag_y <= 0.0 {
        return f64::NEG_INFINITY;
    }
    20.0 * (mag_y / mag_x).log10()
}

/// Retorna `max |x[n] - 2*x[n-1] + x[n-2]|` no intervalo `[ini, fim)`.
pub fn pico_segunda_diferenca(x: &[f32], ini: usize, fim: usize) -> f32 {
    let start = ini.max(2);
    let end = fim.min(x.len());
    if start >= end {
        return 0.0;
    }
    let mut max_d2 = 0.0_f32;
    for n in start..end {
        let d2 = (x[n] - 2.0 * x[n - 1] + x[n - 2]).abs();
        if d2 > max_d2 {
            max_d2 = d2;
        }
    }
    max_d2
}

/// Razão de transiente em torno da amostra `evento`:
/// pico da 2ª diferença em `[evento - 128, evento + 0.030*sr)` dividido pelo maior pico em
/// `[evento - 0.040*sr, evento - 0.010*sr)` e em `[evento + 0.100*sr, evento + 0.130*sr)`.
pub fn razao_transiente(x: &[f32], evento: usize, sr: f32) -> f32 {
    if x.is_empty() || sr <= 0.0 {
        return 0.0;
    }
    let s = sr as f64;
    let win_evt_ini = evento.saturating_sub(128);
    let win_evt_fim = evento.saturating_add((0.030 * s).round() as usize);
    let pico_evt = pico_segunda_diferenca(x, win_evt_ini, win_evt_fim);

    let pre_ini = evento.saturating_sub((0.040 * s).round() as usize);
    let pre_fim = evento.saturating_sub((0.010 * s).round() as usize);
    let pico_pre = pico_segunda_diferenca(x, pre_ini, pre_fim);

    let pos_ini = evento.saturating_add((0.100 * s).round() as usize);
    let pos_fim = evento.saturating_add((0.130 * s).round() as usize);
    let pico_pos = pico_segunda_diferenca(x, pos_ini, pos_fim);

    let ref_pico = pico_pre.max(pico_pos);
    if ref_pico <= 1e-12 {
        if pico_evt <= 1e-12 {
            1.0
        } else {
            pico_evt / 1e-12
        }
    } else {
        pico_evt / ref_pico
    }
}

/// Conta o número de amostras subnormais em `x` (`v.is_subnormal()`).
pub fn contar_subnormais(x: &[f32]) -> usize {
    x.iter().filter(|v| v.is_subnormal()).count()
}

/// Retorna o índice do pico absoluto da resposta a um impulso unitário em `saida`.
pub fn atraso_por_impulso(saida: &[f32]) -> usize {
    let mut best_idx = 0usize;
    let mut best_abs = -1.0_f32;
    for (idx, &v) in saida.iter().enumerate() {
        let a = v.abs();
        if a > best_abs {
            best_abs = a;
            best_idx = idx;
        }
    }
    best_idx
}

/// Calcula o RT60 em segundos pela integral reversa de Schroeder, ajustando uma reta entre
/// `-5 dB` e `-35 dB` e extrapolando para `-60 dB`.
pub fn rt60_schroeder(ir: &[f32], sr: f32) -> f64 {
    if ir.len() < 4 || sr <= 0.0 {
        return 0.0;
    }
    let n = ir.len();
    let mut rev = vec![0.0_f64; n];
    let mut acc = 0.0_f64;
    for i in (0..n).rev() {
        let v = ir[i] as f64;
        acc += v * v;
        rev[i] = acc;
    }
    let e0 = rev[0];
    if e0 <= 0.0 {
        return 0.0;
    }

    let mut sum_t = 0.0_f64;
    let mut sum_y = 0.0_f64;
    let mut sum_tt = 0.0_f64;
    let mut sum_ty = 0.0_f64;
    let mut count = 0usize;

    for (i, &e) in rev.iter().enumerate() {
        if e <= 0.0 {
            continue;
        }
        let db = 10.0 * (e / e0).log10();
        if (-35.0..=-5.0).contains(&db) {
            let t = (i as f64) / (sr as f64);
            sum_t += t;
            sum_y += db;
            sum_tt += t * t;
            sum_ty += t * db;
            count += 1;
        }
    }

    if count < 2 {
        return 0.0;
    }
    let cf = count as f64;
    let denom = cf * sum_tt - sum_t * sum_t;
    if denom.abs() <= 1e-18 {
        return 0.0;
    }
    let slope = (cf * sum_ty - sum_t * sum_y) / denom;
    if slope >= -1e-9 {
        return 0.0;
    }
    (-60.0 / slope).max(0.0)
}

/// Diferença em cents entre `f` e `f_ref`: `1200 * log2(f / f_ref)`.
pub fn cents(f: f64, f_ref: f64) -> f64 {
    if f <= 0.0 || f_ref <= 0.0 {
        return 0.0;
    }
    1200.0 * (f / f_ref).log2()
}
