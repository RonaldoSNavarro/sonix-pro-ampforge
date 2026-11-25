import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '../..');

test('Spec 093 CA-5: Audio.ts expõe 3 modos explícitos e seletor de canal sem corromper pending', () => {
  const audioTs = fs.readFileSync(
    path.join(root, 'apps/sonix-app/ui/src/screens/Audio.ts'),
    'utf8',
  );
  assert.match(audioTs, /'instrument'/, 'Modo instrument deve existir na UI Áudio');
  assert.match(audioTs, /'pc_line_in'/, 'Modo pc_line_in deve existir na UI Áudio');
  assert.match(audioTs, /'line'/, 'Modo line (+4 dBu) deve existir na UI Áudio');
  assert.match(audioTs, /input\.channel/, 'Seletor input.channel (left/right/sum) deve existir na UI Áudio');
  assert.match(audioTs, /pc_line_in/, 'Banner contextual para entrada Line In de PC deve existir');
  assert.match(audioTs, /num\(store,\s*'input\.boost'/, 'Audio.ts deve ler input.boost via backendKey');
  assert.doesNotMatch(audioTs, /'guitar'|"guitar"/, "Não deve enviar 'guitar' inválido para input.mode");
  assert.doesNotMatch(audioTs, /num\(store,\s*'main\.input_boost'/, 'Não deve ler main.input_boost diretamente de store.params');
});

test('Spec 093 CA-5: store.ts restaura input_device e output_device; registry e shell sem chaves obsoletas', () => {
  const storeTs = fs.readFileSync(
    path.join(root, 'apps/sonix-app/ui/src/store.ts'),
    'utf8',
  );
  assert.match(storeTs, /saved\.input_device/, 'bootstrap deve restaurar input_device salvo');
  assert.match(storeTs, /saved\.output_device/, 'bootstrap deve restaurar output_device salvo');

  const registryTs = fs.readFileSync(
    path.join(root, 'apps/sonix-app/ui/src/controls.registry.ts'),
    'utf8',
  );
  assert.match(registryTs, /paramMeta\('input\.boost'\)/, 'controls.registry.ts deve mapear input.boost');
  assert.match(registryTs, /paramMeta\('input\.channel'\)/, 'controls.registry.ts deve mapear input.channel');
  assert.doesNotMatch(registryTs, /'guitar'/, "controls.registry.ts não pode usar 'guitar'");

  const shellTs = fs.readFileSync(
    path.join(root, 'apps/sonix-app/ui/src/shell.ts'),
    'utf8',
  );
  assert.match(shellTs, /amp_1\.standby|amp_standby/, 'shell.ts deve exibir alerta dinâmico quando amp_1.standby estiver ativo');
});
