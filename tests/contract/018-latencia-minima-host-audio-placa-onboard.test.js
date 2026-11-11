// tests/contract/018-latencia-minima-host-audio-placa-onboard.test.js
// Contrato de Aceite Automatizado - Spec 018 (Host de Áudio Robusto e Latência Mínima em Placa Onboard)
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

describe('Spec 018: Latência Mínima e Host de Áudio Robusto em Placa Onboard', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-7, CA-9, CA-11 a CA-13)', () => {
    try {
      const mingwPath = 'C:\\ProgramData\\mingw64\\mingw64\\bin';
      const pathEnv = process.env.PATH ? `${mingwPath};${process.env.PATH}` : mingwPath;
      const output = execSync('cargo test -p sonix-desktop --test contract_018_low_latency_host -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: pathEnv }
      });
      console.log(output);
      assert.match(output, /test_ca_01_queue_occupancy_within_half_to_two_periods \.\.\. ok/);
      assert.match(output, /test_ca_02_drift_correction_and_waveform_continuity \.\.\. ok/);
      assert.match(output, /test_ca_03_zero_alloc_and_zero_lock_during_callbacks \.\.\. ok/);
      assert.match(output, /test_ca_04_negotiated_sample_rate_and_graceful_mismatch_fallback \.\.\. ok/);
      assert.match(output, /test_ca_05_stereo_to_mono_channel_selection_accuracy \.\.\. ok/);
      assert.match(output, /test_ca_06_backend_fallback_cascade_without_panic \.\.\. ok/);
      assert.match(output, /test_ca_07_api_audio_status_telemetry_schema \.\.\. ok/);
      assert.match(output, /test_ca_09_offline_loopback_latency_measurement \.\.\. ok/);
      assert.match(output, /test_ca_11_dsp_panic_catch_and_stream_resilience \.\.\. ok/);
      assert.match(output, /test_ca_12_device_lost_and_automatic_recovery \.\.\. ok/);
      assert.match(output, /test_ca_13_non_regression_specs_001_and_013 \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });

  test('CA-8: Interface expõe painel de áudio e telemetria real (#cpuMeterText e #audioSettingsPanel)', () => {
    const htmlPath = resolve('design-system/index.html');
    const html = readFileSync(htmlPath, 'utf-8');

    assert.ok(html.includes('id="cpuMeterText"'), 'Elemento #cpuMeterText deve existir no HTML');
    assert.ok(html.includes('id="audioSettingsPanel"'), 'Elemento #audioSettingsPanel deve existir no HTML');
    assert.ok(html.includes('id="audioInputChannelSelect"'), 'Elemento #audioInputChannelSelect deve existir no HTML');
    assert.ok(html.includes('id="audioModeText"'), 'Elemento #audioModeText deve existir no HTML');
    assert.ok(html.includes('id="audioSampleRateText"'), 'Elemento #audioSampleRateText deve existir no HTML');
    assert.ok(html.includes('id="audioPeriodsText"'), 'Elemento #audioPeriodsText deve existir no HTML');
    assert.ok(html.includes('id="audioMeasuredRoundtripText"'), 'Elemento #audioMeasuredRoundtripText deve existir no HTML');

    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    // Verifica que updateCpuBudget não escreve mais em cpuMeterText
    assert.ok(
      !appJs.includes('cpuText.textContent = `${total.toFixed(1)}%`'),
      'updateCpuBudget não deve mais sobrescrever #cpuMeterText com estimativa estática'
    );
    assert.ok(
      appJs.includes('audioInputChannelSelect'),
      'app.js deve escutar evento de seleção de canal #audioInputChannelSelect'
    );
  });
});
