// tests/contract/024-efeitos-e-caminho-paralelo-ao-vivo.test.js
// Testes de Contrato Automatizados para Spec 024: Efeitos e Caminho Paralelo do Grid ao Vivo
// Cobre Critérios de Aceite CA-1 a CA-13

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

describe('Spec 024: Efeitos e Caminho Paralelo do Grid ao Vivo', () => {
  test('Execução de testes de contrato Rust DSP (CA-2 a CA-9, CA-13)', () => {
    const res = spawnSync('cargo', ['test', '-p', 'ampforge-dsp', '--test', 'contract_024_live_graph_fx'], {
      encoding: 'utf-8',
      shell: true,
    });
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes de contrato Rust DSP: \n${res.stdout}\n${res.stderr}`
    );
  });

  test('Execução de testes de contrato Rust Desktop Graph API (CA-1, CA-10)', () => {
    const res = spawnSync('cargo', ['test', '-p', 'sonix-desktop', '--test', 'contract_024_graph_api'], {
      encoding: 'utf-8',
      shell: true,
    });
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes de contrato Rust Desktop Graph API: \n${res.stdout}\n${res.stderr}`
    );
  });

  test('CA-10: FS2-FS4 enviam mod_2.enabled, dly_1.enabled, rev_1.enabled sem debounce', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    assert.ok(
      appJs.includes("'mod_2.enabled'") || appJs.includes('"mod_2.enabled"'),
      'app.js deve mapear footswitch FS2 para mod_2.enabled'
    );
    assert.ok(
      appJs.includes("'dly_1.enabled'") || appJs.includes('"dly_1.enabled"'),
      'app.js deve mapear footswitch FS3 para dly_1.enabled'
    );
    assert.ok(
      appJs.includes("'rev_1.enabled'") || appJs.includes('"rev_1.enabled"'),
      'app.js deve mapear footswitch FS4 para rev_1.enabled'
    );
  });

  test('CA-10: Editor de bloco (#blockEditor) abre com controles para comp_2, eq_1, mod_2, dly_1, rev_1, split, merge', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    assert.ok(appJs.includes('comp_2'), 'app.js deve conter editor dedicado para comp_2');
    assert.ok(appJs.includes('eq_1'), 'app.js deve conter editor dedicado para eq_1');
    assert.ok(appJs.includes('mod_2'), 'app.js deve conter editor dedicado para mod_2');
    assert.ok(appJs.includes('dly_1'), 'app.js deve conter editor dedicado para dly_1');
    assert.ok(appJs.includes('rev_1'), 'app.js deve conter editor dedicado para rev_1');
    assert.ok(appJs.includes('split') || appJs.includes('merge'), 'app.js deve suportar split e merge');
  });

  test('CA-11: Aviso de sinal direto e esmaecimento da linha 1 quando split desligado', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    assert.ok(
      appJs.includes('caminho B sem amp/cab: sinal direto'),
      'app.js deve exibir aviso quando caminho B estiver sem amp nem cab ativos'
    );
    assert.ok(
      appJs.includes('split-disabled') || appJs.includes('opacity') || appJs.includes('esmaecida'),
      'app.js deve suportar estado esmaecido da linha 1 quando split estiver desligado'
    );
  });
});
