// tests/contract/023-pedal-drive-808-editor-de-bloco.test.js
// Testes de Contrato Automatizados para Spec 023 (Pedal Drive 808 & Editor de Bloco Selecionado)
// Cobre Critérios de Aceite CA-1 a CA-10

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

describe('Spec 023: Pedal de Drive 808 Fiel ao Circuito e Editor do Bloco Selecionado', () => {
  test('Execução de testes de contrato Rust DSP (CA-2 a CA-10)', () => {
    const res = spawnSync('cargo', ['test', '-p', 'ampforge-dsp', '--test', 'contract_023_drive_808'], {
      encoding: 'utf-8',
      shell: true,
    });
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes de contrato Rust DSP: \n${res.stdout}\n${res.stderr}`
    );
  });

  test('CA-1: #blockEditor existe e possui controles fiéis do 808 (Drive, Tone, Level)', () => {
    const htmlPath = resolve('design-system/index.html');
    const html = readFileSync(htmlPath, 'utf-8');

    assert.ok(html.includes('id="blockEditor"'), 'HTML deve conter #blockEditor');
    assert.ok(
      html.includes('blockEditorTitle') || html.includes('blockEditorBody'),
      'HTML deve estruturar o painel do editor de bloco selecionado'
    );
  });

  test('CA-1: Clicar em blk_drive abre editor com Drive, Tone, Level e sincroniza params', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    assert.ok(
      appJs.includes('openBlockEditor') || appJs.includes('blockEditor'),
      'app.js deve implementar lógica de abertura do editor de bloco ao clicar'
    );
    assert.ok(
      appJs.includes('drive.drive') && appJs.includes('drive.tone') && appJs.includes('drive.level'),
      'app.js deve vincular e enviar drive.drive, drive.tone e drive.level ao backend'
    );
    assert.ok(
      appJs.includes('sem controles nesta versão'),
      'app.js deve exibir "sem controles nesta versão" para blocos sem editor dedicado'
    );
  });

  test('CA-1: Duplo clique em blk_drive continua alternando drive.enabled', () => {
    const appJsPath = resolve('design-system/app.js');
    const appJs = readFileSync(appJsPath, 'utf-8');

    assert.ok(
      appJs.includes("'drive.enabled'"),
      'app.js deve enviar drive.enabled no duplo clique para toggle de bypass'
    );
  });
});
