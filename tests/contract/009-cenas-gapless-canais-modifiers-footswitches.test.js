// tests/contract/009-cenas-gapless-canais-modifiers-footswitches.test.js
// Contrato de Aceite Automatizado - Spec 009
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 009: Hierarquia Setlist/Preset, 8 Scenes Gapless, 4 Canais, Modifiers e Footswitches', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-9)', () => {
    try {
      const output = execSync('cargo test -p sonix-core --test contract_009_scenes_modifiers -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_setlist_preset_hierarchy_and_8_scenes \.\.\. ok/);
      assert.match(output, /test_ca_02_block_4_channels_a_b_c_d_independence \.\.\. ok/);
      assert.match(output, /test_ca_03_inactive_channels_consume_zero_cpu \.\.\. ok/);
      assert.match(output, /test_ca_05_modifier_matrix_creation_and_binding \.\.\. ok/);
      assert.match(output, /test_ca_06_modifier_transfer_function_and_linear_interpolation \.\.\. ok/);
      assert.match(output, /test_ca_07_footswitch_4_modes_reconfiguration \.\.\. ok/);
      assert.match(output, /test_ca_08_error_handling_invalid_scene_or_block \.\.\. ok/);
      assert.match(output, /test_ca_09_non_regression_unbound_parameters_unaffected \.\.\. ok/);

      const outputCa3 = execSync('cargo test -p sonix-desktop --test contract_049_scene_state_and_queues ca3_009', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(outputCa3);
      assert.match(outputCa3, /ca3_009_troca_de_cena_sem_clique_razao_transiente_le_2 \.\.\. ok/);

      const outputCa4 = execSync('cargo test -p sonix-desktop --test contract_049_scene_state_and_queues ca4_009', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(outputCa4);
      assert.match(outputCa4, /ca4_009_quatro_canais_por_bloco_comutam \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
