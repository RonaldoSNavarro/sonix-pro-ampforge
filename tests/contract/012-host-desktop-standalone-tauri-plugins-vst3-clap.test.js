// tests/contract/012-host-desktop-standalone-tauri-plugins-vst3-clap.test.js
// Contrato de Aceite Automatizado - Spec 012
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 012: Host Desktop Standalone (Tauri v2) e Plugins DAW (VST3/CLAP)', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-8)', () => {
    try {
      const output = execSync('cargo test -p sonix-desktop --test contract_012_desktop_plugin -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_desktop_audio_host_device_enumeration_and_stream_init \.\.\. ok/);
      assert.match(output, /test_ca_02_desktop_midi_input_enumeration_and_cc_queue_forwarding \.\.\. ok/);
      assert.match(output, /test_ca_03_vst3_and_clap_plugin_descriptors_and_64bit_processing \.\.\. ok/);
      assert.match(output, /test_ca_04_daw_state_save_and_restore_lossless_matching_sonixpreset \.\.\. ok/);
      assert.match(output, /test_ca_05_daw_transport_tempo_sync \.\.\. ok/);
      assert.match(output, /test_ca_06_desktop_and_plugin_native_webview_ipc_zero_network_server \.\.\. ok/);
      assert.match(output, /test_ca_07_audio_device_disconnect_safe_handling_without_crash \.\.\. ok/);
      assert.match(output, /test_ca_08_non_regression_preset_interoperability_between_standalone_and_plugins \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
