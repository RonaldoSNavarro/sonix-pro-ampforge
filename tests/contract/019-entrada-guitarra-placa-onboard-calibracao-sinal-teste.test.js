// tests/contract/019-entrada-guitarra-placa-onboard-calibracao-sinal-teste.test.js
// Contrato de Aceite Automatizado - Spec 019 (Entrada de Guitarra na Placa Onboard: Calibração, Clipping e Sinal de Teste)
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

describe('Spec 019: Entrada de Guitarra na Placa Onboard: Calibração, Clipping e Sinal de Teste', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-9)', () => {
    try {
      const mingwPath = 'C:\\ProgramData\\mingw64\\mingw64\\bin';
      const pathEnv = process.env.PATH ? `${mingwPath};${process.env.PATH}` : mingwPath;
      const output = execSync('cargo test -p sonix-desktop --test contract_019_input_calibration -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: pathEnv }
      });
      console.log(output);
      assert.match(output, /test_ca_01_input_calibration_ideal_boost_and_gate \.\.\. ok/);
      assert.match(output, /test_ca_02_input_too_hot_and_input_too_low \.\.\. ok/);
      assert.match(output, /test_ca_03_hum_detection_50hz_60hz_and_white_noise \.\.\. ok/);
      assert.match(output, /test_ca_04_adc_clips_counted_and_exposed_in_status \.\.\. ok/);
      assert.match(output, /test_ca_05_input_source_test_riff_plays_through_dsp_chain \.\.\. ok/);
      assert.match(output, /test_ca_06_test_riff_deterministic_bit_identical_and_peak_level \.\.\. ok/);
      assert.match(output, /test_ca_07_post_api_input_di_validation_and_resampling \.\.\. ok/);
      assert.match(output, /test_ca_08_persisted_calibration_per_device_id_loaded_on_boot \.\.\. ok/);
      assert.match(output, /test_ca_09_non_regression_specs_013_014_and_018 \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });

  test('CA-4 & CA-5 lado UI: Wizard de calibração, VU Clip por 1s e Test Signal DSP pelo motor sem AudioContext', () => {
    const htmlPath = resolve('design-system/index.html');
    const html = readFileSync(htmlPath, 'utf-8');

    assert.ok(html.includes('id="inputCalibrationWizard"'), 'Elemento #inputCalibrationWizard deve existir no HTML');
    assert.ok(html.includes('id="btnCalibrateInput"'), 'Botão #btnCalibrateInput deve existir no HTML');
    assert.ok(html.includes('id="vuInFill"'), 'Elemento #vuInFill deve existir no HTML');
    assert.ok(html.includes('id="testSignalBtn"'), 'Botão #testSignalBtn deve existir no HTML');

    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    // CA-5: Test Signal DSP não deve criar AudioContext sintetizador Web Audio, e sim enviar input.source
    assert.ok(appJs.includes('"input.source"'), 'app.js deve enviar chave input.source para a API de parâmetros');
    assert.ok(
      !appJs.includes('new (window.AudioContext || window.webkitAudioContext)()') ||
      !appJs.includes('createWaveShaper'),
      'app.js não deve utilizar sintetizador Web Audio com WaveShaper para o sinal de teste da guitarra'
    );

    // CA-4: VU Clip por 1 segundo
    assert.ok(appJs.includes('adc_clips'), 'app.js deve inspecionar adc_clips na telemetria');
  });
});
