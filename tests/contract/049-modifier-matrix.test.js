import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, '../..');

describe('Spec 049 CA-31: Modifier Matrix na UI (clique direito, fontes, .has-modifier, arco duplo)', () => {
  it('CA-31: knob/controle expõe clique direito para abrir Modifier Matrix com todas as fontes e classe .has-modifier', () => {
    const knobPath = path.join(rootDir, 'apps/sonix-app/ui/src/components/Knob.ts');
    const modUiPath = path.join(rootDir, 'apps/sonix-app/ui/src/components/ModifierMatrixModal.ts');

    assert.equal(fs.existsSync(knobPath), true, 'Knob.ts deve existir');
    assert.equal(fs.existsSync(modUiPath), true, 'ModifierMatrixModal.ts deve existir para CA-31');

    const knobSrc = fs.readFileSync(knobPath, 'utf8');
    const modSrc = fs.readFileSync(modUiPath, 'utf8');

    assert.ok(knobSrc.includes('contextmenu'), 'Knob.ts deve abrir Modifier Matrix no evento contextmenu (clique direito)');
    assert.ok(knobSrc.includes('has-modifier'), 'Knob.ts deve aplicar classe .has-modifier quando houver vínculo');
    assert.ok(knobSrc.includes('modifier-range-arc') || knobSrc.includes('min') && knobSrc.includes('max'), 'Knob.ts deve renderizar arco duplo min↔max');

    for (const src of [
      'EXP_PEDAL_1',
      'EXP_PEDAL_2',
      'MIDI_CC_',
      'INTERNAL_LFO_1',
      'INTERNAL_LFO_2',
      'ENVELOPE_FOLLOWER',
      'SCENE_CONTROLLER',
    ]) {
      assert.ok(modSrc.includes(src), `ModifierMatrixModal.ts deve suportar fonte ${src}`);
    }
  });
});
