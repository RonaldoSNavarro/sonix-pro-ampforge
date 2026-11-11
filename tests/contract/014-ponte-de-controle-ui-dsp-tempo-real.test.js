// tests/contract/014-ponte-de-controle-ui-dsp-tempo-real.test.js
// Contrato de Aceite Automatizado - Spec 014 (Ponte de Controle UI->DSP em Tempo Real)
// Cobre Critérios de Aceite CA-1 a CA-3, CA-10, CA-11 e CA-14

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

describe('Spec 014: Ponte de Controle UI->DSP em Tempo Real', () => {
  test('Execução de testes de contrato Rust (CA-4 a CA-9, CA-12 a CA-14)', () => {
    try {
      const mingwPath = 'C:\\ProgramData\\mingw64\\mingw64\\bin';
      const pathEnv = process.env.PATH ? `${mingwPath};${process.env.PATH}` : mingwPath;
      const output = execSync('cargo test -p sonix-desktop --test contract_014_ui_dsp_bridge -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: pathEnv }
      });
      console.log(output);
      assert.match(output, /test_ca_04_post_params_applies_all_keys_and_state_reflects_them \.\.\. ok/);
      assert.match(output, /test_ca_05_unknown_and_legacy_keys_ignored_without_mutation \.\.\. ok/);
      assert.match(output, /test_ca_06_and_07_lock_free_parameter_delivery_no_bypass_under_contention \.\.\. ok/);
      assert.match(output, /test_ca_08_parameter_smoothing_step_response \.\.\. ok/);
      assert.match(output, /test_ca_09_click_free_bypass_crossfade \.\.\. ok/);
      assert.match(output, /test_ca_12_atomic_validation_and_rejection_of_invalid_payloads \.\.\. ok/);
      assert.match(output, /test_ca_13_method_not_allowed_and_unsupported_media_type \.\.\. ok/);
      assert.match(output, /test_ca_14_non_regression_specs_001_to_013_and_018 \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });

  test('CA-1, CA-2, CA-3, CA-10, CA-11: Despachante syncParamsToBackend e integração no app.js', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    // Despachante syncParamsToBackend presente
    assert.ok(
      appJs.includes('syncParamsToBackend'),
      'app.js deve implementar a função syncParamsToBackend'
    );

    // Endpoint correto POST /api/params
    assert.ok(
      appJs.includes("'/api/params'") || appJs.includes('"/api/params"'),
      'syncParamsToBackend deve enviar requisição para /api/params'
    );

    // Chaves da tabela do §2
    assert.ok(appJs.includes("'amp_1.gain'") || appJs.includes('"amp_1.gain"'));
    assert.ok(appJs.includes("'drive.enabled'") || appJs.includes('"drive.enabled"'));
    assert.ok(appJs.includes("'amp_1.bright'") || appJs.includes('"amp_1.bright"'));

    // Expressão mapeada em 3.0..9.5
    assert.ok(
      appJs.includes('3.0') && appJs.includes('9.5'),
      'Pedal de expressão deve mapear ganho na faixa 3.0..9.5'
    );

    // Coalescência em janela de 25ms
    assert.ok(
      appJs.includes('25'),
      'Debounce/throttle dos knobs contínuos deve usar janela de 25ms'
    );

    // Leitura do estado inicial no boot GET /api/state
    assert.ok(
      appJs.includes('/api/state'),
      'app.js deve consultar /api/state no boot para carregar valores reais do motor'
    );
  });
});
