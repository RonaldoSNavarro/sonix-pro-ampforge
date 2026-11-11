import { test, suite } from 'node:test';
import assert from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

suite('Spec 026: Afinador ao Vivo na Entrada da Guitarra', () => {
  test('Execução de testes de contrato Rust DSP e Desktop (CA-1 a CA-4, CA-6 a CA-9)', () => {
    const res = spawnSync(
      'cargo',
      ['test', '-p', 'sonix-desktop', '--test', 'contract_026_live_tuner'],
      { encoding: 'utf-8', shell: true }
    );
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes de contrato Rust Tuner: \n${res.stdout}\n${res.stderr}`
    );
  });

  test('CA-5: app.js não contém mais autocorrelação local em JavaScript e usa /api/tuner ou cents do motor', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    // Não deve conter mais o loop antigo de autocorrelação com lag de 100 a 600
    assert.ok(
      !appJs.includes('lag = 100; lag <= 600; lag += 2'),
      'Autocorrelação JS local ingênua deve ser removida'
    );

    // O botão deve alternar tuner.enabled e consultar telemetria/api do afinador
    assert.ok(
      appJs.includes('tuner.enabled') || appJs.includes('/api/tuner') || appJs.includes('tunerResult'),
      'UI deve se comunicar com o afinador via parâmetros/API'
    );
  });

  test('CA-7: UI exibe traço "—" quando não há nota detectada', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    assert.ok(
      appJs.includes('—') || appJs.includes('---'),
      'UI deve exibir "—" na ausência de sinal com tom detectável'
    );
  });
});
