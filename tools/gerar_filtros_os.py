#!/usr/bin/env python3
"""Gera tabelas de biquads elipticos de 8a ordem (4 secoes SOS) para oversampling 2x/4x/8x.

Baseado nas formulas exatas de Orfanidis (Lecture Notes / "Elliptic Filter Design") usando
transformacao de Landen para funcoes elipticas de Jacobi, sem dependencias externas.
"""

from __future__ import annotations

import cmath
import math
from pathlib import Path


def ellip_landen_k_kprime(k: float) -> tuple[float, float]:
    """Calcula K(k) e K'(k) via iteracao AGM/Landen."""
    kp = math.sqrt(max(0.0, 1.0 - k * k))

    def complete_elliptic_k(m_k: float) -> float:
        a = 1.0
        b = math.sqrt(max(1e-30, 1.0 - m_k * m_k))
        for _ in range(30):
            a_next = 0.5 * (a + b)
            b_next = math.sqrt(a * b)
            a, b = a_next, b_next
            if abs(a - b) < 1e-15:
                break
        return math.pi / (2.0 * a)

    return complete_elliptic_k(k), complete_elliptic_k(kp)


def jacobi_sn_cd(u_complex: complex, k: float) -> tuple[complex, complex]:
    """Calcula sn(u * K, k) e cd(u * K, k) onde u esta normalizado por K(k)."""
    # Sequencia de Landen descendente: k_0 = k, k_{m} = (k_{m-1} / (1 + k'_{m-1}))^2
    ks = [k]
    for _ in range(12):
        ki = ks[-1]
        if ki < 1e-15:
            break
        kpi = math.sqrt(max(0.0, 1.0 - ki * ki))
        ks.append((ki / (1.0 + kpi)) ** 2)

    # No limite k_M -> 0: sn(z, 0) = sin(z), cd(z, 0) = cos(z)
    z = u_complex * (math.pi / 2.0)
    sn = cmath.sin(z)
    cd = cmath.cos(z)
    for ki in reversed(ks[1:]):
        # Transformacao de Landen ascendente para cd e sn
        # Usando recursao de Landen para sn: sn_prev = (1 + ki) * sn / (1 + ki * sn^2)
        pass

    # Recursao exata de Landen ascendente usando ks[1..] onde ks[i] = k_{i}:
    # Na verdade, para u normalizado por K(k):
    # w_M = (pi/2) * u; sn_M = sin(w_M); cd_M = cos(w_M)
    # Para i = M-1 .. 0:
    #   sn_i = (1 + k_{i+1}) * sn_{i+1} / (1 + k_{i+1} * sn_{i+1}^2)
    #   cn_i^2 = 1 - sn_i^2 ... melhor fazer com cd direto:
    #   cd_i = ((1 + k_{i+1}) * cd_{i+1}?) -> nao, a relacao para cd e:
    #   cd(u, k_i) = ( (1 + k_{i+1}) * cd_? ) -> usamos sn(u + 1, k) = cd(u, k)!
    def eval_sn(u_norm: complex) -> complex:
        s = cmath.sin(u_norm * (math.pi / 2.0))
        for k_next in reversed(ks[1:]):
            s = (1.0 + k_next) * s / (1.0 + k_next * s * s)
        return s

    return eval_sn(u_complex), eval_sn(u_complex + 1.0)


def ellip_analog_poles_zeros_8th(rp_db: float, rs_db: float) -> tuple[list[complex], list[complex], float]:
    """Retorna (polos, zeros, ganho_dc) de um filtro eliptico passa-baixas analogico normalizado (wp=1)."""
    n = 8
    ep = math.sqrt(10.0 ** (rp_db / 10.0) - 1.0)
    es = math.sqrt(10.0 ** (rs_db / 10.0) - 1.0)
    k1 = ep / es
    k1p = math.sqrt(1.0 - k1 * k1)

    K1, K1p = ellip_landen_k_kprime(k1)
    # Grau modular via nome q1 = exp(-pi * K1' / K1), q = q1^(1/N)
    # Ou inversao de Landen para achar k a partir de k1 e N
    q1 = math.exp(-math.pi * K1p / K1)
    q = q1 ** (1.0 / n)
    # Serie teta de Jacobi para k = (theta_2(q) / theta_3(q))^2
    t2 = 0.0
    t3 = 1.0
    for m in range(1, 20):
        t2 += 2.0 * (q ** ((m - 0.5) ** 2))
        t3 += 2.0 * (q ** (m * m))
    # Nota: theta_2 = 2 * sum_{m=0..} q^((m+0.5)^2)
    t2 = 0.0
    for m in range(20):
        t2 += 2.0 * (q ** ((m + 0.5) ** 2))
    k = (t2 / t3) ** 2

    # v0 = -j / (N * K1) * sn^{-1}(j / ep, k1)  -> via jacobi ou formula de Orfanidis:
    # Na verdade, |v0| = (1 / N) * F(atan(1/ep), k1') / K1' -> mas podemos calcular u0 = (1/N) * (K1 / K1')?
    # Mais direto: sn(j * v0 * K1, k1) = j / ep => sc(v0 * K1, k1') = 1 / ep => sn(v0 * K1, k1') = 1 / sqrt(1 + ep^2)
    # Calculo de u1 = F(phi, k1') / K1' onde sin(phi) = 1 / sqrt(1 + ep^2) via Landen descendente:
    phi = math.asin(1.0 / math.sqrt(1.0 + ep * ep))
    # Landen ascendente/descendente para F(phi, m):
    k_cur = k1p
    phi_cur = phi
    for _ in range(15):
        if k_cur < 1e-15:
            break
        kp_cur = math.sqrt(1.0 - k_cur * k_cur)
        k_next = (1.0 - kp_cur) / (1.0 + kp_cur)
        phi_next = phi_cur + math.atan(kp_cur * math.tan(phi_cur))
        # Desenrola quadrante se necessario (phi in [0, pi/2] -> tan >= 0)
        if phi_cur > math.pi / 2.0:
            phi_next += math.pi
        k_cur = k_next
        phi_cur = phi_next
    # F(phi, k1') / K1' = phi_final / (pi/2) * ... espera, K1' tambem escala!
    # Mais simples e 100% exato: integracao de Gauss-Legendre ou Simpson em [0, phi] / [0, pi/2]
    def ellip_f(upper: float, m_k: float) -> float:
        steps = 2000
        h = upper / steps
        acc = 0.0
        for i in range(steps + 1):
            t = i * h
            val = 1.0 / math.sqrt(1.0 - (m_k * math.sin(t)) ** 2)
            w = 1.0 if (i == 0 or i == steps) else (4.0 if i % 2 == 1 else 2.0)
            acc += w * val
        return acc * h / 3.0

    v0_norm = (ellip_f(phi, k1p) / K1p) / n  # v0 / K(k), pois K'/K = N * K1'/K1 => v0_real / K = (F/K1') / N * (K'/K)?
    # Conferindo: v0 = K / (N * K1) * F(phi, k1') ... e K' / K = K1' / (N * K1) => K / (N * K1) = K' / K1'
    # Logo v0 / K' = F(phi, k1') / K1' / N ... espera:
    # No argumento de cd(u_i - j * v0, k): u_i esta em [0, K], enquanto j*v0 e escalado por K'!
    # Entao na normalizacao por K(k), j * v0 / K = j * (v0 / K') * (K' / K) = j * (F(phi, k1') / K1') * (K' / K)?
    # Vamos checar: K / (N * K1) * F(phi, k1') / K = F(phi, k1') / (N * K1) = (F(phi, k1') / K1') * (K1' / (N * K1)) = (F(phi, k1') / K1') * (K' / K)!
    # Exato! E K' / K = (K1' / K1) / N!
    K_k, Kp_k = ellip_landen_k_kprime(k)
    jv0_norm = 1j * (ellip_f(phi, k1p) / K1p) * (Kp_k / K_k)

    poles: list[complex] = []
    zeros: list[complex] = []
    for i in range(1, n // 2 + 1):
        u_i = (2 * i - 1) / n  # normalizado por K(k)
        _, cd_z = jacobi_sn_cd(complex(u_i, 0.0), k)
        wz = 1.0 / (k * cd_z.real)
        zeros.append(1j * wz)
        zeros.append(-1j * wz)

        _, cd_p = jacobi_sn_cd(u_i - jv0_norm, k)
        p = 1j * cd_p
        if p.real > 0:
            p = complex(-p.real, p.imag)
        poles.append(p)
        poles.append(p.conjugate())

    # Normalizacao de ganho em DC para ordem par (N=8): H(0) = 1 / sqrt(1 + ep^2)
    h0 = 1.0 / math.sqrt(1.0 + ep * ep)
    return poles, zeros, h0


def bilinear_sos(
    poles: list[complex], zeros: list[complex], h0: float, fp_hz: float, fs_os_hz: float
) -> list[list[float]]:
    """Converte polos/zeros analogicos em 4 secoes biquad digitais [b0, b1, b2, a1, a2] via bilinear."""
    w_warp = math.tan(math.pi * fp_hz / fs_os_hz)

    # Pareia polos e zeros por proximidade de frequencia (maior Q por ultimo)
    p_pairs = [(poles[2 * i], poles[2 * i + 1]) for i in range(4)]
    z_pairs = [(zeros[2 * i], zeros[2 * i + 1]) for i in range(4)]

    # Ordena polos por |Im(p)| / |Re(p)| crescente (Q crescente)
    p_pairs.sort(key=lambda pair: abs(pair[0].imag) / max(1e-12, abs(pair[0].real)))
    # Pareia cada par de polos com o par de zeros mais proximo
    remaining_z = list(z_pairs)
    paired = []
    for p1, p2 in p_pairs:
        best_idx = min(
            range(len(remaining_z)),
            key=lambda idx: abs(abs(remaining_z[idx][0]) - abs(p1)),
        )
        z1, z2 = remaining_z.pop(best_idx)
        paired.append(((p1, p2), (z1, z2)))

    sos_list: list[list[float]] = []
    dc_gain_acc = 1.0
    for (p1, p2), (z1, z2) in paired:
        # Escala frequencia analogica por w_warp (ja que z_dig = (1 + s_scaled)/(1 - s_scaled))
        p1s = p1 * w_warp
        p2s = p2 * w_warp
        z1s = z1 * w_warp
        z2s = z2 * w_warp

        # Transformacao bilinear: s -> (z - 1) / (z + 1)
        zd_p1 = (1.0 + p1s) / (1.0 - p1s)
        zd_p2 = (1.0 + p2s) / (1.0 - p2s)
        zd_z1 = (1.0 + z1s) / (1.0 - z1s)
        zd_z2 = (1.0 + z2s) / (1.0 - z2s)

        a1 = -(zd_p1 + zd_p2).real
        a2 = (zd_p1 * zd_p2).real
        b1 = -(zd_z1 + zd_z2).real
        b2 = (zd_z1 * zd_z2).real

        # Ganho da secao normalizado para 1 em DC (z=1) antes de aplicar h0^(1/4)
        dc_sec = (1.0 + b1 + b2) / (1.0 + a1 + a2)
        b0 = 1.0 / dc_sec
        b1_n = b1 / dc_sec
        b2_n = b2 / dc_sec
        sos_list.append([b0, b1_n, b2_n, a1, a2])
        dc_gain_acc *= (b0 + b1_n + b2_n) / (1.0 + a1 + a2)

    # Aplica ganho H(0) distribuido igualmente nas 4 secoes
    sec_gain = (h0 / dc_gain_acc) ** 0.25
    for sec in sos_list:
        sec[0] *= sec_gain
        sec[1] *= sec_gain
        sec[2] *= sec_gain
    return sos_list


def eval_sos_db(sos_list: list[list[float]], freq_hz: float, fs_os_hz: float) -> float:
    w = 2.0 * math.pi * freq_hz / fs_os_hz
    z_inv = cmath.exp(-1j * w)
    z_inv2 = z_inv * z_inv
    mag = 1.0
    for b0, b1, b2, a1, a2 in sos_list:
        num = b0 + b1 * z_inv + b2 * z_inv2
        den = 1.0 + a1 * z_inv + a2 * z_inv2
        mag *= abs(num) / abs(den)
    return 20.0 * math.log10(max(1e-18, mag))


def main() -> None:
    poles, zeros, h0 = ellip_analog_poles_zeros_8th(rp_db=0.1, rs_db=70.0)
    rates = [
        (44100, 0.408 * 44100.0),
        (48000, 0.390 * 48000.0),
        (88200, 0.408 * 88200.0),
        (96000, 0.390 * 96000.0),
    ]
    os_factors = [2, 4, 8]

    lines = [
        "//! Filtros elipticos de 8a ordem (4 secoes biquad SOS) pre-calculados para oversampling.",
        "//! Gerado automaticamente por `tools/gerar_filtros_os.py`.",
        "",
        "/// Coeficientes `[b0, b1, b2, a1, a2]` para as 4 secoes biquad de um filtro eliptico de 8a ordem.",
        "pub(crate) type Elliptic8Sos = [[f64; 5]; 4];",
        "",
    ]

    for sr, fp in rates:
        for os in os_factors:
            fs_os = float(sr) * float(os)
            sos = bilinear_sos(poles, zeros, h0, fp, fs_os)
            g16 = 2.0 * eval_sos_db(sos, 16000.0, fs_os)
            g18 = 2.0 * eval_sos_db(sos, 18000.0, fs_os)
            s06 = eval_sos_db(sos, 0.60 * sr, fs_os)
            s075 = eval_sos_db(sos, 0.75 * sr, fs_os)
            s09 = eval_sos_db(sos, 0.90 * sr, fs_os)
            print(
                f"sr={sr} os={os}x fp={fp:.1f}: pair(16k)={g16:.3}dB pair(18k)={g18:.3}dB "
                f"stop(0.6)={s06:.2}dB stop(0.75)={s075:.2}dB stop(0.9)={s09:.2}dB"
            )
            const_name = f"ELLIP8_{sr}_{os}X"
            lines.append(f"pub const {const_name}: Elliptic8Sos = [")
            for sec in sos:
                lines.append(
                    f"    [{sec[0]:.16e}, {sec[1]:.16e}, {sec[2]:.16e}, {sec[3]:.16e}, {sec[4]:.16e}],"
                )
            lines.append("];")
            lines.append("")

    lines.extend(
        [
            "pub(crate) fn elliptic8_sos(sample_rate: f32, os_factor: usize) -> &'static Elliptic8Sos {",
            "    let sr_key = match sample_rate.round() as u32 {",
            "        0..=46000 => 44100,",
            "        46001..=68000 => 48000,",
            "        68001..=92000 => 88200,",
            "        _ => 96000,",
            "    };",
            "    match (sr_key, os_factor) {",
            "        (44100, 2) => &ELLIP8_44100_2X,",
            "        (44100, 4) => &ELLIP8_44100_4X,",
            "        (44100, _) => &ELLIP8_44100_8X,",
            "        (48000, 2) => &ELLIP8_48000_2X,",
            "        (48000, 4) => &ELLIP8_48000_4X,",
            "        (48000, _) => &ELLIP8_48000_8X,",
            "        (88200, 2) => &ELLIP8_88200_2X,",
            "        (88200, 4) => &ELLIP8_88200_4X,",
            "        (88200, _) => &ELLIP8_88200_8X,",
            "        (96000, 2) => &ELLIP8_96000_2X,",
            "        (96000, 4) => &ELLIP8_96000_4X,",
            "        (96000, _) => &ELLIP8_96000_8X,",
            "        _ => &ELLIP8_48000_2X,",
            "    }",
            "}",
            "",
        ]
    )

    out_path = Path("crates/ampforge-dsp/src/os_filters.rs")
    out_path.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {out_path}")


if __name__ == "__main__":
    main()
