// tests/contract/015-endurecimento-servidor-local-validacao-csrf.test.js
// Contrato de Aceite Automatizado - Spec 015 (Endurecimento do Servidor Local e Proteção CSRF/NaN)
// Cobre Critérios de Aceite CA-1 a CA-9

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

describe('Spec 015: Endurecimento do Servidor Local e Proteção CSRF/NaN', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-9)', () => {
    try {
      const mingwPath = 'C:\\ProgramData\\mingw64\\mingw64\\bin';
      const pathEnv = process.env.PATH ? `${mingwPath};${process.env.PATH}` : mingwPath;
      const output = execSync('cargo test -p sonix-desktop --test contract_015_server_hardening -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: pathEnv }
      });
      console.log(output);
      assert.match(output, /test_ca_01_cross_site_origin_rejected_with_403 \.\.\. ok/);
      assert.match(output, /test_ca_02_dns_rebinding_rejected_with_421 \.\.\. ok/);
      assert.match(output, /test_ca_03_no_cors_headers_present \.\.\. ok/);
      assert.match(output, /test_ca_04_session_token_validation_and_uniqueness \.\.\. ok/);
      assert.match(output, /test_ca_05_payload_too_large_rejected_with_413 \.\.\. ok/);
      assert.match(output, /test_ca_06_non_finite_values_ignored_and_output_remains_finite \.\.\. ok/);
      assert.match(output, /test_ca_07_nan_internal_state_recovery_within_one_block \.\.\. ok/);
      assert.match(output, /test_ca_08_telemetry_low_latency_under_concurrency \.\.\. ok/);
      assert.match(output, /test_ca_09_legitimate_post_with_token_succeeds \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });

  test('CA-4 e CA-9: Injeção do token no index.html e envio do cabeçalho X-Sonix-Token no app.js', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    // Cabeçalho X-Sonix-Token incluído nas chamadas fetch POST para /api/params
    assert.ok(
      appJs.includes("'X-Sonix-Token'") || appJs.includes('"X-Sonix-Token"'),
      'app.js deve enviar o cabeçalho X-Sonix-Token nas requisições POST para o servidor local'
    );

    // Token capturado de window.__SONIX_TOKEN__ ou meta tag
    assert.ok(
      appJs.includes('__SONIX_TOKEN__') || appJs.includes('sonix-token'),
      'app.js deve recuperar o token de sessão injetado pelo servidor'
    );
  });
});
