// tests/contract/021-bancada-chassi-aberto-osciloscopio-ao-vivo.test.js
// Testes de Contrato Automatizados para Spec 021 (Bancada Chassi Aberto & Osciloscópio CRT)
// Cobre Critérios de Aceite CA-9, CA-10 e CA-12 lado UI

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

describe('Spec 021: Bancada do Chassi Aberto (Mod Shop) e Osciloscópio CRT ao Vivo', () => {
  test('Execução de testes de contrato Rust DSP (CA-1 a CA-8, CA-11, CA-13)', () => {
    const res = spawnSync('cargo', ['test', '-p', 'ampforge-dsp', '--test', 'contract_021_workbench_live'], {
      encoding: 'utf-8',
      shell: true,
    });
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes de contrato Rust DSP: \n${res.stdout}\n${res.stderr}`
    );
  });

  test('Execução de testes de contrato Rust Scope API (CA-9, CA-10)', () => {
    const res = spawnSync('cargo', ['test', '-p', 'sonix-desktop', '--test', 'contract_021_scope_api'], {
      encoding: 'utf-8',
      shell: true,
    });
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes de contrato Rust API: \n${res.stdout}\n${res.stderr}`
    );
  });

  test('CA-10: nonLinearTubeStage não existe mais em app.js e osciloscópio usa dados do motor', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    assert.ok(
      !appJs.includes('function nonLinearTubeStage'),
      'nonLinearTubeStage calculada localmente em JS deve ser removida'
    );
    assert.ok(
      appJs.includes('/api/scope'),
      'app.js deve consultar /api/scope para renderizar dados reais do motor'
    );
  });

  test('CA-12: Rótulos de válvulas são técnicos e não possuem termos perceptivos/marcas', () => {
    const htmlPath = resolve('design-system/index.html');
    const html = readFileSync(htmlPath, 'utf-8');

    const forbiddenTerms = ['Warm', 'Crunch', 'Punch', 'Chime', 'High Headroom', 'Mullard', 'RCA', 'Genalex'];
    for (const term of forbiddenTerms) {
      assert.ok(
        !html.includes(term),
        `HTML não deve conter termo perceptivo ou marca não medida: "${term}"`
      );
    }
  });
});
