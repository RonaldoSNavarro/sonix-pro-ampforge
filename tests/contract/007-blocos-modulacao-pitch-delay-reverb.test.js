// tests/contract/007-blocos-modulacao-pitch-delay-reverb.test.js
// Contrato de Aceite Automatizado - Spec 007
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 007: Blocos de Modulação, Pitch, Delay (Ducking/Spillover) e Reverb', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-9)', () => {
    try {
      const output = execSync('cargo test -p ampforge-dsp --test contract_007_mod_pitch_delay_reverb -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_mod_7_models_and_bpm_tempo_sync_frequency \.\.\. ok/);
      assert.match(output, /test_ca_02_polyphonic_drop_capo_transposition_ratio \.\.\. ok/);
      assert.match(output, /test_ca_03_intelligent_dual_harmonizer_c_major_diatonic_thirds \.\.\. ok/);
      assert.match(output, /test_ca_04_tape_echo_self_oscillation_soft_saturation \.\.\. ok/);
      assert.match(output, /test_ca_05_delay_ducking_attenuates_tail_during_input_presence \.\.\. ok/);
      assert.match(output, /test_ca_06_reverb_models_spring_drip_and_shimmer_octave \.\.\. ok/);
      assert.match(output, /test_ca_07_spillover_preserves_decaying_tail_when_bypassed \.\.\. ok/);
      assert.match(output, /test_ca_08_error_handling_and_safe_clamping_bpm_and_delay_time \.\.\. ok/);
      assert.match(output, /test_ca_09_non_regression_zero_mix_dry_through_identity \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
