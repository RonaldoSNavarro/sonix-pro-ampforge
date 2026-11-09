// tests/contract/013-execucao-desktop-windows-captura-fisica-p2-duplo-clique.test.js
// Contrato de Aceite Automatizado - Spec 013
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 013: Execução Desktop Nativa no Windows com Captura Física P2/P3 e Duplo Clique', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-8)', () => {
    try {
      const output = execSync('CARGO_TARGET_DIR=/tmp/target cargo test -p sonix-desktop --test contract_013_desktop_realtime_audio -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_desktop_windows_double_click_launcher_artifact_present \.\.\. ok/);
      assert.match(output, /test_ca_02_physical_audio_device_enumeration_wasapi_default_duplex \.\.\. ok/);
      assert.match(output, /test_ca_03_duplex_realtime_lock_free_spsc_ring_buffer \.\.\. ok/);
      assert.match(output, /test_ca_04_active_dsp_chain_realtime_processing \.\.\. ok/);
      assert.match(output, /test_ca_05_dedicated_desktop_window_and_local_isolated_server \.\.\. ok/);
      assert.match(output, /test_ca_06_realtime_telemetry_vu_crt_oscilloscope_tuner \.\.\. ok/);
      assert.match(output, /test_ca_07_graceful_fault_handling_and_zero_fill_silence \.\.\. ok/);
      assert.match(output, /test_ca_08_non_regression_specs_001_to_012_integrity \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
