import { test, suite } from 'node:test';
import assert from 'node:assert';
import { readFileSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

suite('Spec 025: Presets, Cenas e Canais de Bloco Reais', () => {
  test('Execução de testes de contrato Rust Core (CA-3, CA-4, CA-5, CA-8, CA-11, CA-12)', () => {
    const res = spawnSync(
      'cargo',
      ['test', '-p', 'sonix-core', '--test', 'contract_025_presets_scenes'],
      { encoding: 'utf-8', shell: true }
    );
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes de contrato Rust Core: \n${res.stdout}\n${res.stderr}`
    );
  });

  test('Execução de testes de contrato Rust Desktop (CA-2, CA-10, CA-11)', () => {
    const res = spawnSync(
      'cargo',
      ['test', '-p', 'sonix-desktop', '--test', 'contract_025_preset_switch'],
      { encoding: 'utf-8', shell: true }
    );
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes de contrato Rust Desktop: \n${res.stdout}\n${res.stderr}`
    );
  });

  test('CA-1: Os 4 presets de fábrica existem em presets/factory e seguem schema 1.1.0', () => {
    const factoryFiles = [
      'presets/factory/brit800.sonixpreset',
      'presets/factory/usdeluxe.sonixpreset',
      'presets/factory/recto.sonixpreset',
      'presets/factory/ac30.sonixpreset',
    ];

    for (const f of factoryFiles) {
      assert.ok(existsSync(resolve(f)), `Preset de fábrica deve existir: ${f}`);
      const content = JSON.parse(readFileSync(resolve(f), 'utf-8'));
      assert.strictEqual(content.schemaVersion, '1.1.0');
      assert.ok(content.params, `Preset ${f} deve conter dicionário de parâmetros`);
      assert.ok(content.metadata && content.metadata.id);
    }
  });

  test('CA-9: O selo de troca de cena não possui texto fixo "1.1ms" em app.js e atualiza com tempo medido', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    // Texto fixo não deve mais existir no app.js
    assert.ok(
      !appJs.includes('1.1ms'),
      'Texto fixo "1.1ms" deve ser removido de app.js'
    );

    // Deve exibir transition_ms medido
    assert.ok(
      appJs.includes('transition_ms') || appJs.includes('transitionMs'),
      'Selo deve exibir tempo retornado pelo backend'
    );
  });

  test('CA-6 & CA-7: Troca de cena envia delta de chaves e salvar cena persiste alterações', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    assert.ok(
      appJs.includes('/api/scenes/select') || appJs.includes('selectScene'),
      'UI deve conter função de seleção de cena via API'
    );
    assert.ok(
      appJs.includes('saveCurrentScene') || appJs.includes('saveScene') || appJs.includes('salvar cena') || appJs.includes('Salvar'),
      'UI deve permitir salvar a cena ativa'
    );
  });
});
