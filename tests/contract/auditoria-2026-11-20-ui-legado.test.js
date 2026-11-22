// tests/contract/auditoria-2026-11-20-ui-legado.test.js
// AUD-35: Cobertura equivalente na UI Tauri (apps/sonix-app/ui) para os 15 testes legados
// que liam design-system/ (011, 014, 015, 018, 019, 020, 021, 022, 023, 024, 025, 026, 027, 029, 030).
//
// Tabela de Mapeamento CA -> teste legado -> teste UI Tauri:
// - 011 (CA-1..9): Tokens/tipografia/knobs/footswitch/hardware-shell -> 048-ui-desktop-fidelidade-visual.test.js + este arquivo (Knob.ts)
// - 014 (CA-1..3,10,11): Coalescência/debounce UI->DSP -> 033-app-desktop-tauri-v2-telas-e-controles-reais.test.js (CA-8 ParamCoalescer)
// - 015 (CA-4,9): Segurança IPC Tauri (CSP/sem origem externa) -> 033-app-desktop-tauri-v2-telas-e-controles-reais.test.js (CA-8/CA-10)
// - 018 (CA-8): Painel de áudio e telemetria real -> 049-status-bar-honesta.test.js + 051-correcao-contratos-e-controles-ui.test.js (CA-17)
// - 019 (CA-4,5): Assistente de calibração e sinal de teste -> 051-correcao-contratos-e-controles-ui.test.js (CA-1)
// - 020 (CA-3,7): 5 modelos de amp e controles adaptativos -> auditoria-2026-11-20-parametros.test.js (AUD-08) + 052 (CA-4)
// - 021 (CA-10,12): Osciloscópio do motor e rótulos técnicos de válvulas -> 051 (CA-15) + este arquivo
// - 022 (CA-7,13): Microfones no cone e bloqueio com IR ativa -> 038-cab-ir-ui.test.js + 051 (CA-16)
// - 023 (CA-1): Pedal Drive 808 e controles -> 051 (CA-19) + 048
// - 024 (CA-10,11): FS2-FS4, editores e esmaecimento da linha 2 -> 051 (CA-13,14) + 052 (CA-3) + auditoria-2026-11-20-parametros.test.js (AUD-21)
// - 025 (CA-1,6,7,9): Presets de fábrica, cenas e medição real -> 051 (CA-3,5) + este arquivo
// - 026 (CA-5,7): Afinador via motor e traço "—" -> 051 (CA-13,15) + este arquivo
// - 027 (CA-1..8): Ponte UI->DSP completa e layout 1440x900 sem scroll -> 033 + 048 + 049 + 051
// - 029 (CA-1,3): Afinador note/cents e calibração silence->strum -> 051 (CA-1,13,15)
// - 030 (CA-5): Noise Gate controles gate.* -> 051 (CA-8,10,26) + 048

import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const uiSrc = path.resolve('apps/sonix-app/ui/src');

describe('AUD-35: Cobertura UI Tauri equivalente aos testes legados 011..030', () => {
  test('011 CA-4 & CA-8: Knob.ts limita rotação a [-135, +135] deg, tem ajuste fino com Shift e reset com dblclick', () => {
    const knobTs = fs.readFileSync(path.join(uiSrc, 'components/Knob.ts'), 'utf8');
    assert.match(knobTs, /-135\s*\+\s*norm\s*\*\s*270/, 'valueToRotationDeg deve mapear [min,max] para [-135,+135]');
    assert.match(knobTs, /e\.shiftKey\s*\?\s*520\s*:\s*140/, 'Shift+Drag deve aplicar ajuste fino');
    assert.match(knobTs, /addEventListener\('dblclick'[\s\S]*defaultValue/, 'dblclick deve resetar para defaultValue');
  });

  test('021 CA-12: UI Tauri usa nomenclatura técnica de válvulas sem termos comerciais/hype', () => {
    const bancadaTs = fs.readFileSync(path.join(uiSrc, 'screens/Bancada.ts'), 'utf8');
    const paramsGenTs = fs.readFileSync(path.join(uiSrc, 'params.generated.ts'), 'utf8');
    assert.match(paramsGenTs, /12ax7/i);
    assert.match(paramsGenTs, /6l6/i);
    assert.doesNotMatch(bancadaTs, /\b(Mullard|Telefunken|Groove Tubes|magical|warmth)\b/i);
  });

  test('025 CA-1: Presets de fábrica existem em presets/factory em JSON válido', () => {
    const factoryDir = path.resolve('presets/factory');
    const files = fs.readdirSync(factoryDir).filter((f) => f.endsWith('.sonixpreset'));
    assert.ok(files.length >= 4, `Esperado >= 4 presets de fábrica, encontrado ${files.length}`);
    for (const f of files) {
      const parsed = JSON.parse(fs.readFileSync(path.join(factoryDir, f), 'utf8'));
      assert.strictEqual(parsed.schemaVersion, '1.1.0', `Preset ${f} deve seguir schemaVersion 1.1.0`);
      assert.ok(parsed.metadata?.name || parsed.metadata?.id, `Preset ${f} deve declarar metadata.name ou metadata.id`);
    }
  });

  test('026 CA-7: Afinador Tauri exibe marcador "---" quando não há nota detectada', () => {
    const shellTs = fs.readFileSync(path.join(uiSrc, 'shell.ts'), 'utf8');
    assert.match(shellTs, /tuner-note[\s\S]*'---'/, 'shell.ts deve exibir "---" quando sem nota');
  });
});
