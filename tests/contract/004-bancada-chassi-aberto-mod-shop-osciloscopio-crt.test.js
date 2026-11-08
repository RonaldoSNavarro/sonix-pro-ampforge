// tests/contract/004-bancada-chassi-aberto-mod-shop-osciloscopio-crt.test.js
// Contrato de Aceite Automatizado - Spec 004
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 004: Bancada Chassi Aberto (Mod Shop) e Osciloscópio CRT', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-9)', () => {
    try {
      const output = execSync('cargo test -p ampforge-dsp --test contract_004_modshop_crt -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_chassis_toggle_visibility_without_interrupting_audio \.\.\. ok/);
      assert.match(output, /test_ca_02_preamp_tube_selection_and_gain_reduction \.\.\. ok/);
      assert.match(output, /test_ca_03_tone_stack_topologies_and_position_routing \.\.\. ok/);
      assert.match(output, /test_ca_04_power_amp_tubes_class_bias_and_saturation \.\.\. ok/);
      assert.match(output, /test_ca_05_power_supply_rectifiers_and_sag_compression \.\.\. ok/);
      assert.match(output, /test_ca_06_vacuum_tube_interactive_glow_scaling_with_bias \.\.\. ok/);
      assert.match(output, /test_ca_07_crt_scope_3_modes_telemetry_generation \.\.\. ok/);
      assert.match(output, /test_ca_08_error_handling_and_safe_clamping \.\.\. ok/);
      assert.match(output, /test_ca_09_non_regression_closed_chassis_preserves_mod_shop_parameters \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
