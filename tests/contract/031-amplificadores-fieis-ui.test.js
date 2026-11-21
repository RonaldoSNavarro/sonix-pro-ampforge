import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { CONTROLS_REGISTRY } from '../../apps/sonix-app/ui/src/controls.registry.ts';

const ROOT = path.resolve(import.meta.dirname, '..', '..');

describe('Spec 031 — CA-8: Painel frontal fiel por modelo na UI e registro único', () => {
  it('CA-8: controls.registry.ts cobre todos os controles originais de amp_1 e engine.oversampling sem pendência 031', () => {
    const regPath = path.join(ROOT, 'apps', 'sonix-app', 'ui', 'src', 'controls.registry.ts');
    assert.ok(fs.existsSync(regPath), 'controls.registry.ts deve existir');
    const src = fs.readFileSync(regPath, 'utf8');

    const requiredKeys = [
      'engine.oversampling',
      'amp_1.model',
      'amp_1.power',
      'amp_1.standby',
      'amp_1.bright',
      'amp_1.input_jack',
      'amp_1.jumpered',
      'amp_1.channel_mode',
      'amp_1.spongy',
      'amp_1.volume_2',
      'amp_1.cut',
      'amp_1.reverb',
      'amp_1.vibrato_speed',
      'amp_1.vibrato_depth',
      'amp_1.impedance_ohms',
    ];
    for (const k of requiredKeys) {
      assert.ok(
        CONTROLS_REGISTRY.some((c) => c.backendKey === k),
        `Falta ${k} no registro`,
      );
    }

    // Depois da implementação da Spec 031, nenhum item pode permanecer marcado com pendingSpec: '031'
    const pending031Matches = src.match(/pendingSpec:\s*'031'/g) || [];
    assert.equal(
      pending031Matches.length,
      0,
      `Ainda existem ${pending031Matches.length} controles marcados com pendingSpec: '031'`,
    );
  });
});
