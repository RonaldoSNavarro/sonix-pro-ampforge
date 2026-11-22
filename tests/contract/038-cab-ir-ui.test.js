import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const GABINETE_TS = path.resolve(process.cwd(), 'apps/sonix-app/ui/src/screens/Gabinete.ts');

test('Spec 038 CA-6: cab_1.mic* ficam disabled com dica quando cab_1.ir_enabled = true', () => {
  const src = fs.readFileSync(GABINETE_TS, 'utf8');
  assert.match(src, /IR ativa — microfonação vem da IR/, 'Deve exibir dica de IR ativa');
  assert.match(src, /node\.disabled\s*=\s*irOn/, 'Nó do mic na arena deve ficar disabled quando IR ativa');
  assert.match(src, /muteBtn\.disabled\s*=\s*irActive/, 'Botão mute do mic deve ficar disabled quando IR ativa');
  assert.match(src, /inp\.disabled\s*=\s*true/, 'Sliders do mic devem ficar disabled quando IR ativa');
});

test('AUD-32: seletor de alto-falante fica desabilitado com dica quando cab_1.ir_enabled = true', () => {
  const src = fs.readFileSync(GABINETE_TS, 'utf8');
  assert.match(
    src,
    /IR carregada substitui o alto-falante/,
    'Gabinete.ts deve exibir a dica "IR carregada substitui o alto-falante" quando IR estiver ativa'
  );
  assert.match(
    src,
    /b\.disabled\s*=\s*irActive/,
    'Botões de seleção de alto-falante devem ficar disabled quando cab_1.ir_enabled = true'
  );
});
