// tests/contract/006-blocos-dinamica-drive-equalizacao.test.js
// Contrato de Aceite Automatizado - Spec 006
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 006: Blocos de Dinâmica (Gate, Comp), Drive e Equalização (RTA)', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-8)', () => {
    try {
      const output = execSync('cargo test -p ampforge-dsp --test contract_006_dynamics_drive_eq -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_intelligent_natural_gate_sidechain_di_tracking \.\.\. ok/);
      assert.match(output, /test_ca_02_compressor_4_topologies_and_parallel_mix \.\.\. ok/);
      assert.match(output, /test_ca_03_drive_6_models_and_5_clipping_diodes \.\.\. ok/);
      assert.match(output, /test_ca_04_drive_germanium_vs_asymmetrical_si_led_headroom_and_harmonics \.\.\. ok/);
      assert.match(output, /test_ca_05_parametric_eq_8_bands_64bit_filters_and_rta \.\.\. ok/);
      assert.match(output, /test_ca_06_graphic_eq_boogie_5_band_and_pedal_10_band \.\.\. ok/);
      assert.match(output, /test_ca_07_invalid_parameters_clamped_safely \.\.\. ok/);
      assert.match(output, /test_ca_08_non_regression_flat_response_at_zero_gain \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
