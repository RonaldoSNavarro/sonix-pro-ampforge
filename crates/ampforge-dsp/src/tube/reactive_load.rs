//! Sub-modelo 10 (Spec 077): `ReactiveLoadAndNfb`
//!
//! Modela a curva de impedância complexa do alto-falante $Z_{\text{spk}}(s)$
//! (pico de ressonância grave $f_r \in [75, 110\text{ Hz}]$ + elevação indutiva
//! acima de `1.5 kHz`), modulada pela malha NFB e pelos controles `Presence`,
//! `Depth/Resonance` e seletor `4/8/16 Ω`.

use crate::biquad::BiquadFilter;

#[derive(Debug, Clone, Copy)]
pub struct ReactiveLoadAndNfb {
    pub speaker_res_hz: f64,
    pub impedance_ohms: u8,
    pub nfb_enabled: bool,
    pub nfb_amount: f64, // 0..10
    pub presence: f64,   // 0..10
    pub depth: f64,      // 0..10
    res_filter: BiquadFilter,
    ind_filter: BiquadFilter,
    pres_filter: BiquadFilter,
    depth_filter: BiquadFilter,
    sr: f64,
    dirty: bool,
}

impl Default for ReactiveLoadAndNfb {
    fn default() -> Self {
        Self::new(48_000.0)
    }
}

impl ReactiveLoadAndNfb {
    pub fn new(sample_rate_hz: f64) -> Self {
        let mut s = Self {
            speaker_res_hz: 95.0,
            impedance_ohms: 16,
            nfb_enabled: true,
            nfb_amount: 6.0,
            presence: 5.0,
            depth: 0.0,
            res_filter: BiquadFilter::identity(),
            ind_filter: BiquadFilter::identity(),
            pres_filter: BiquadFilter::identity(),
            depth_filter: BiquadFilter::identity(),
            sr: sample_rate_hz.max(8000.0),
            dirty: true,
        };
        s.update_coeffs();
        s
    }

    pub fn set_sample_rate(&mut self, sample_rate_hz: f64) {
        let fs = sample_rate_hz.max(8000.0);
        if (fs - self.sr).abs() > 1.0 {
            self.sr = fs;
            self.dirty = true;
            self.update_coeffs();
        }
    }

    pub fn configure(
        &mut self,
        speaker_res_hz: f64,
        impedance_ohms: u8,
        nfb_enabled: bool,
        nfb_amount: f64,
        presence: f64,
        depth: f64,
    ) {
        let res = speaker_res_hz.clamp(50.0, 200.0);
        let nfb = nfb_amount.clamp(0.0, 10.0);
        let pres = presence.clamp(0.0, 10.0);
        let dep = depth.clamp(0.0, 10.0);
        if (res - self.speaker_res_hz).abs() > 0.1
            || impedance_ohms != self.impedance_ohms
            || nfb_enabled != self.nfb_enabled
            || (nfb - self.nfb_amount).abs() > 1e-3
            || (pres - self.presence).abs() > 1e-3
            || (dep - self.depth).abs() > 1e-3
        {
            self.speaker_res_hz = res;
            self.impedance_ohms = impedance_ohms;
            self.nfb_enabled = nfb_enabled;
            self.nfb_amount = nfb;
            self.presence = pres;
            self.depth = dep;
            self.dirty = true;
        }
        if self.dirty {
            self.update_coeffs();
        }
    }

    pub fn update_coeffs(&mut self) {
        self.dirty = false;
        let eff_nfb = if self.nfb_enabled { self.nfb_amount } else { 0.0 };

        // Tonal Note 3 (Spec 054): 4 Ohm -> menor fator NFB / mais solto e ressonante;
        // 16 Ohm -> maior fator NFB / mais firme e amortecido.
        let imp_damping = match self.impedance_ohms {
            4 => 0.72,
            8 => 1.0,
            _ => 1.28,
        };
        let damping = (eff_nfb / 10.0) * imp_damping;

        // Pico de impedância em f_r (~+6.5 dB sem NFB, amortecido até ~+1.2 dB com NFB máximo)
        let res_boost_db = (6.5 / (1.0 + 3.2 * damping)).max(0.4);
        self.res_filter
            .update_coeffs(BiquadFilter::peaking(self.speaker_res_hz, res_boost_db, 2.2, self.sr));

        // Elevação indutiva acima de 2 kHz também amortecida pelo NFB
        let ind_boost_db = (4.0 / (1.0 + 2.5 * damping)).max(0.3);
        self.ind_filter
            .update_coeffs(BiquadFilter::high_shelf(2_800.0, ind_boost_db, self.sr));

        // Presence e Depth dentro da malha NFB: quando nfb_enabled == false, efeito estrito = 0.0 dB
        if self.nfb_enabled && eff_nfb > 0.01 {
            let nfb_scale = (eff_nfb / 10.0).clamp(0.0, 1.0);
            let pres_db = ((self.presence - 5.0) / 5.0) * 4.5 * nfb_scale + 2.25 * nfb_scale;
            self.pres_filter
                .update_coeffs(BiquadFilter::high_shelf(3_200.0, pres_db, self.sr));
            let depth_db = (self.depth / 10.0) * 6.0 * nfb_scale;
            self.depth_filter
                .update_coeffs(BiquadFilter::low_shelf(110.0, depth_db, self.sr));
        } else {
            self.pres_filter.update_coeffs(BiquadFilter::identity());
            self.depth_filter.update_coeffs(BiquadFilter::identity());
        }
    }

    pub fn reset(&mut self) {
        for f in [
            &mut self.res_filter,
            &mut self.ind_filter,
            &mut self.pres_filter,
            &mut self.depth_filter,
        ] {
            f.x1 = 0.0;
            f.x2 = 0.0;
            f.y1 = 0.0;
            f.y2 = 0.0;
        }
    }

    #[inline]
    pub fn process(&mut self, x: f64) -> f64 {
        let mut y = self.res_filter.process(x);
        y = self.ind_filter.process(y);
        if self.nfb_enabled && self.nfb_amount > 0.01 {
            y = self.pres_filter.process(y);
            y = self.depth_filter.process(y);
        }
        y
    }
}
