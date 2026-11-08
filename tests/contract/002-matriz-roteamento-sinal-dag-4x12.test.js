// tests/contract/002-matriz-roteamento-sinal-dag-4x12.test.js
// Contrato de Aceite Automatizado - Spec 002
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 002: Matriz de Roteamento de Sinal 4x12 (DAG), Split/Mixer e Múltiplos I/Os', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-10)', () => {
    try {
      const output = execSync('cargo test -p ampforge-dsp --test contract_002_matriz_dag -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_grid_4x12_addressable_48_slots \.\.\. ok/);
      assert.match(output, /test_ca_02_split_node_modes_instantiation \.\.\. ok/);
      assert.match(output, /test_ca_03_split_crossover_flat_sum_reconstruction \.\.\. ok/);
      assert.match(output, /test_ca_04_split_dynamic_envelope_routing \.\.\. ok/);
      assert.match(output, /test_ca_05_mixer_node_controls_gain_pan_phase_master \.\.\. ok/);
      assert.match(output, /test_ca_06_mixer_complete_phase_cancellation_under_minus_90dbfs \.\.\. ok/);
      assert.match(output, /test_ca_07_independent_io_routing_per_row \.\.\. ok/);
      assert.match(output, /test_ca_08_amp_out_p10_no_cab_bypasses_cab_convolution \.\.\. ok/);
      assert.match(output, /test_ca_09_dag_cycle_prevention_rejects_feedback_loop \.\.\. ok/);
      assert.match(output, /test_ca_10_non_regression_parallel_line_state_isolation \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
