// tests/contract/001-nucleo-dsp-tempo-real-ipc-lock-free.test.js
// Contrato de Aceite Automatizado - Spec 001
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 001: Núcleo DSP de Tempo Real, IPC Lock-Free e Orçamento de CPU', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-11)', () => {
    try {
      const output = execSync('cargo test -p ampforge-dsp --test contract_001_nucleo_dsp -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_internal_algorithmic_latency_under_1_5ms \.\.\. ok/);
      assert.match(output, /test_ca_02_supported_sample_rates_and_buffer_sizes \.\.\. ok/);
      assert.match(output, /test_ca_03_zero_alloc_and_zero_lock_during_process_block \.\.\. ok/);
      assert.match(output, /test_ca_04_mixed_precision_32bit_bus_and_64bit_solvers \.\.\. ok/);
      assert.match(output, /test_ca_05_polyphase_half_band_oversampling_rejection_over_96db \.\.\. ok/);
      assert.match(output, /test_ca_06_one_pole_parameter_smoother_step_response \.\.\. ok/);
      assert.match(output, /test_ca_07_atomic_graph_pointer_swap_rcu \.\.\. ok/);
      assert.match(output, /test_ca_08_telemetry_extraction_spsc_queue_non_blocking \.\.\. ok/);
      assert.match(output, /test_ca_09_cpu_overload_protection_at_88_percent \.\.\. ok/);
      assert.match(output, /test_ca_10_invalid_audio_config_rejected_with_structured_error \.\.\. ok/);
      assert.match(output, /test_ca_11_bit_exact_deterministic_output_when_idle \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
