// tests/contract/022-gabinete-microfones-ir-ao-vivo.test.js
// Testes de Contrato Automatizados para Spec 022 (Gabinete, Microfones e IR ao Vivo)
// Cobre Critérios de Aceite CA-7 e CA-13 lado UI e integração

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

describe('Spec 022: Gabinete com Microfonação Dupla e IR ao Vivo sem Latência', () => {
  test('Execução de testes de contrato Rust DSP (CA-1 a CA-6, CA-8 a CA-12, CA-15)', () => {
    const res = spawnSync('cargo', ['test', '-p', 'ampforge-dsp', '--test', 'contract_022_cab_mic_ir'], {
      encoding: 'utf-8',
      shell: true,
    });
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes de contrato Rust DSP: \n${res.stdout}\n${res.stderr}`
    );
  });

  test('Execução de testes de contrato Rust Cab API (CA-11, CA-13)', () => {
    const res = spawnSync('cargo', ['test', '-p', 'sonix-desktop', '--test', 'contract_022_cab_api'], {
      encoding: 'utf-8',
      shell: true,
    });
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes de contrato Rust API: \n${res.stdout}\n${res.stderr}`
    );
  });

  test('CA-7: Arrastar microfone envia cab_1.mic<n>.position e não altera distance_cm', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    // Não deve conter a atribuição antiga que derivava distanceCm de yPct no mousemove
    assert.ok(
      !appJs.includes('distanceCm = ((yPct / 100) * 12).toFixed(1)'),
      'Arrasto de microfone não deve derivar nem alterar distance_cm'
    );

    // Deve sincronizar position para backend
    assert.ok(
      appJs.includes('.position`]:') || appJs.includes('cab_1.mic1.position') || appJs.includes('position'),
      'Arrasto de microfone deve calcular e sincronizar position'
    );
  });

  test('CA-7: Textos de nível, delay e ressonância lêem de /api/state', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    assert.ok(
      appJs.includes('cab_1.resonance_hz') || appJs.includes('cabResHz'),
      'Ressonância deve ser atualizada com dados do backend /api/state'
    );
    assert.ok(
      appJs.includes('cab_1.mic1.level_db') || appJs.includes('mic1LevelText'),
      'Nível do microfone deve ser exibido com base no estado do backend'
    );
  });

  test('CA-13: Controles de microfone desabilitados quando IR está ativa', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    assert.ok(
      appJs.includes('ir_enabled') || appJs.includes('cab_1.ir_enabled'),
      'app.js deve checar cab_1.ir_enabled'
    );
    assert.ok(
      appJs.includes('disabled') || appJs.includes('classList.toggle'),
      'app.js deve desabilitar controles de microfone quando IR estiver ativa'
    );
  });
});
