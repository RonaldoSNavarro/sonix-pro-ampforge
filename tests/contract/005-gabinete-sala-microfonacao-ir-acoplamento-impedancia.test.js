// tests/contract/005-gabinete-sala-microfonacao-ir-acoplamento-impedancia.test.js
// Contrato de Aceite Automatizado - Spec 005
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 005: Bloco de Gabinete (Cab), Sala de Microfonação, IR Loader e Acoplamento de Impedância', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-8)', () => {
    try {
      const output = execSync('cargo test -p ampforge-dsp --test contract_005_virtual_cab -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_mic_positioning_and_spectral_response_changes \.\.\. ok/);
      assert.match(output, /test_ca_02_mic_catalog_and_channel_mixer_controls \.\.\. ok/);
      assert.match(output, /test_ca_03_fractional_delay_and_phase_inversion \.\.\. ok/);
      assert.match(output, /test_ca_04_ir_loader_wav_parsing_and_zero_latency_convolution \.\.\. ok/);
      assert.match(output, /test_ca_05_impedance_coupling_transmission_to_amp \.\.\. ok/);
      assert.match(output, /test_ca_06_acoustic_conditioning_filters_and_room_early_reflections \.\.\. ok/);
      assert.match(output, /test_ca_07_invalid_or_truncated_ir_file_rejected_safely \.\.\. ok/);
      assert.match(output, /test_ca_08_non_regression_slot2_disabled_spares_processing_and_preserves_slot1 \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
