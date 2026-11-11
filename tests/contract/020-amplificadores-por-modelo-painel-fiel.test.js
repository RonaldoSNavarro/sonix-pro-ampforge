// tests/contract/020-amplificadores-por-modelo-painel-fiel.test.js
// Contrato de Aceite Automatizado - Spec 020 (Amplificadores por Modelo com Painel e Resposta Fiéis)
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

describe('Spec 020: Amplificadores por Modelo com Painel e Resposta Fiéis à Referência', () => {
  test('Execução de testes de contrato Rust DSP (CA-1, CA-2, CA-5, CA-6, CA-8)', () => {
    try {
      const mingwPath = 'C:\\ProgramData\\mingw64\\mingw64\\bin';
      const pathEnv = process.env.PATH ? `${mingwPath};${process.env.PATH}` : mingwPath;
      const output = execSync('cargo test -p ampforge-dsp --test contract_020_amp_models -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: pathEnv }
      });
      console.log(output);
      assert.match(output, /test_ca_01_tone_stack_response_across_models_and_sample_rates \.\.\. ok/);
      assert.match(output, /test_ca_02_model_switching_crossfade_continuity \.\.\. ok/);
      assert.match(output, /test_ca_05_twin_bright_switch_volume_dependency \.\.\. ok/);
      assert.match(output, /test_ca_06_non_master_amps_power_stage_saturation_by_volume \.\.\. ok/);
      assert.match(output, /test_ca_08_selectable_oversampling_modes_and_latency \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte Rust DSP: ${err.message}\n${err.stdout || ''}`);
    }
  });

  test('Execução de testes de contrato Rust API (CA-4, CA-9)', () => {
    try {
      const mingwPath = 'C:\\ProgramData\\mingw64\\mingw64\\bin';
      const pathEnv = process.env.PATH ? `${mingwPath};${process.env.PATH}` : mingwPath;
      const output = execSync('cargo test -p sonix-desktop --test contract_020_amp_models_api -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: pathEnv }
      });
      console.log(output);
      assert.match(output, /test_ca_04_controls_per_model_validation_and_model_switching \.\.\. ok/);
      assert.match(output, /test_ca_09_non_regression_specs_003_014_016 \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte Rust API: ${err.message}\n${err.stdout || ''}`);
    }
  });

  test('CA-3 & CA-7: UI expõe os 5 modelos em #ampModelSelect e adapta controles por modelo', () => {
    const htmlPath = resolve('design-system/index.html');
    const html = readFileSync(htmlPath, 'utf-8');

    assert.ok(html.includes('id="ampModelSelect"'), 'Elemento #ampModelSelect deve existir no HTML');
    assert.ok(html.includes('jcm800_2203'), 'Opção jcm800_2203 deve existir');
    assert.ok(html.includes('plexi_1959'), 'Opção plexi_1959 deve existir');
    assert.ok(html.includes('twin_ab763'), 'Opção twin_ab763 deve existir');
    assert.ok(html.includes('dual_rect'), 'Opção dual_rect deve existir');
    assert.ok(html.includes('ac30_tb'), 'Opção ac30_tb deve existir');

    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    // Handler de alteração do modelo de amplificador
    assert.ok(appJs.includes('ampModelSelect'), 'app.js deve ter listener para #ampModelSelect');
    assert.ok(appJs.includes('"amp_1.model"') || appJs.includes("'amp_1.model'"), 'app.js deve enviar amp_1.model');
  });
});
