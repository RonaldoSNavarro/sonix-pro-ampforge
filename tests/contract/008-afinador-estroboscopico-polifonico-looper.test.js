// tests/contract/008-afinador-estroboscopico-polifonico-looper.test.js
// Contrato de Aceite Automatizado - Spec 008
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 008: Afinador Estroboscópico/Polifônico e Looper de Performance', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-9)', () => {
    try {
      const output = execSync('cargo test -p ampforge-dsp --test contract_008_tuner_looper -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_tuner_toggle_and_long_press_activation \.\.\. ok/);
      assert.match(output, /test_ca_02_strobe_pitch_detection_accuracy_within_0_1_cent \.\.\. ok/);
      assert.match(output, /test_ca_03_tuner_a4_calibration_polyphonic_mode_and_temperament \.\.\. ok/);
      assert.match(output, /test_ca_04_tuner_mute_vs_bypass_audio_routing \.\.\. ok/);
      assert.match(output, /test_ca_05_looper_state_machine_undo_redo_and_progress_pct \.\.\. ok/);
      assert.match(output, /test_ca_06_looper_reverse_and_half_speed \.\.\. ok/);
      assert.match(output, /test_ca_07_looper_pre_chain_vs_post_chain_positioning \.\.\. ok/);
      assert.match(output, /test_ca_08_error_handling_and_auto_record_buffer_limit_switch \.\.\. ok/);
      assert.match(output, /test_ca_09_non_regression_idle_tuner_and_looper_transparency \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
