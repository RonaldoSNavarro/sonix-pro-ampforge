import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const registryPath = path.resolve('apps/sonix-app/ui/src/controls.registry.ts');

test('Spec 032 — CA-1/CA-7: controls.registry.ts cobre controles originais de Drive, EQ 10-band, 1176LN, RE-201 e Cab sem pendência 032', () => {
  const src = fs.readFileSync(registryPath, 'utf8');

  const requiredKeys = [
    'drive.model',
    'drive.drive',
    'drive.tone',
    'drive.level',
    'drive.bias',
    'eq_1.mode',
    'eq_1.band${idx + 1}.gain_db',
    'eq_1.gain_db',
    'eq_1.volume_db',
    'comp_2.output',
    'comp_2.ratio',
    'dly_1.mode',
    'dly_1.reverb_volume',
    'dly_1.bass',
    'dly_1.treble',
    'cab_1.speaker',
    'cab_1.mic${m}.angle_deg',
    'cab_1.mic${m}.phase_invert',
  ];

  for (const key of requiredKeys) {
    assert.ok(
      src.includes(key),
      `controls.registry.ts deve registrar a chave Spec 032 "${key}"`,
    );
  }

  for (const option of ['ts808', 'klon', 'rat', 'muff', 'fuzzface', 'ds1', 'twin_2x12_open']) {
    assert.ok(src.includes(`'${option}'`), `Opção obrigatória '${option}' ausente`);
  }

  const pending032 = src.match(/pendingSpec:\s*'032'/g) || [];
  assert.equal(
    pending032.length,
    0,
    `Spec 032 não deve deixar controles pendentes em controls.registry.ts (encontrados ${pending032.length})`,
  );
});
