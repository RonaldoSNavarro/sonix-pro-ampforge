// tests/contract/003-modelagem-amplificadores-white-box-painel-frontal.test.js
// Contrato de Aceite Automatizado - Spec 003
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 003: Modelagem White-Box de Amplificadores Valvulados e Painel Frontal Autêntico', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-8)', () => {
    try {
      const output = execSync('cargo test -p ampforge-dsp --test contract_003_amp_modeling -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_front_panel_exposes_11_controls_and_clamps \.\.\. ok/);
      assert.match(output, /test_ca_02_amp_model_selection_updates_topology_and_theme \.\.\. ok/);
      assert.match(output, /test_ca_03_bright_switch_behavior_dependent_on_input_gain \.\.\. ok/);
      assert.match(output, /test_ca_04_master_volume_increases_thd_vs_output_level_linear_scaling \.\.\. ok/);
      assert.match(output, /test_ca_05_presence_and_depth_in_negative_feedback_loop \.\.\. ok/);
      assert.match(output, /test_ca_06_channel_toggle_voicing_reconfiguration \.\.\. ok/);
      assert.match(output, /test_ca_07_invalid_model_id_and_out_of_bounds_handling \.\.\. ok/);
      assert.match(output, /test_ca_08_non_regression_front_panel_preserves_chassis_mod_settings \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
