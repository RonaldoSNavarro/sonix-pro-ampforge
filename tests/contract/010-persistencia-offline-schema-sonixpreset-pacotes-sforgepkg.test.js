// tests/contract/010-persistencia-offline-schema-sonixpreset-pacotes-sforgepkg.test.js
// Contrato de Aceite Automatizado - Spec 010
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { execSync } from 'node:child_process';

describe('Spec 010: Persistência 100% Offline, Schema .sonixpreset e Pacotes .sforgepkg', () => {
  test('Execução de testes de contrato Rust (CA-1 a CA-8)', () => {
    try {
      const output = execSync('cargo test -p sonix-core --test contract_010_persistence_offline -- --nocapture', {
        encoding: 'utf-8',
        env: { ...process.env, PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` }
      });
      console.log(output);
      assert.match(output, /test_ca_01_json_preset_schema_v1_top_level_fields_and_grid_4x12 \.\.\. ok/);
      assert.match(output, /test_ca_02_lossless_round_trip_export_and_import \.\.\. ok/);
      assert.match(output, /test_ca_03_sforgepkg_export_manifest_sha256_and_relative_ir_paths \.\.\. ok/);
      assert.match(output, /test_ca_04_sforgepkg_import_verifies_sha256_and_loads_completely \.\.\. ok/);
      assert.match(output, /test_ca_05_zero_external_cdn_requests_in_design_system_html \.\.\. ok/);
      assert.match(output, /test_ca_06_malformed_json_or_missing_schema_rejected \.\.\. ok/);
      assert.match(output, /test_ca_07_tampered_package_or_path_traversal_rejected_with_package_integrity_error \.\.\. ok/);
      assert.match(output, /test_ca_08_non_regression_export_is_non_blocking \.\.\. ok/);
    } catch (err) {
      assert.fail(`Falha na suíte de testes de contrato Rust: ${err.message}\n${err.stdout || ''}`);
    }
  });
});
