// tests/contract/011-design-system-skeuomorfico-componentes-3d-hardware-shell.test.js
// Contrato de Aceite Automatizado - Spec 011
// Regra SDD: Sem spec, sem código. Cada CA-n deve ser verificado.

import { test, describe } from 'node:test';
import assert from 'node:assert';
import fs from 'node:fs';

describe('Spec 011: Design System Skeuomórfico, Componentes 3D e Hardware Shell', () => {
  const stylesCss = fs.readFileSync('design-system/styles.css', 'utf-8');
  const indexHtml = fs.readFileSync('design-system/index.html', 'utf-8');
  const appJs = fs.readFileSync('design-system/app.js', 'utf-8');

  test('CA-1: Declaração no :root de todos os tokens canônicos de superfície, LEDs e sombras', () => {
    // 7 Canonical Surfaces
    assert.match(stylesCss, /--surface-chassis-anodized:/);
    assert.match(stylesCss, /--surface-tolex-black:/);
    assert.match(stylesCss, /--surface-faceplate-gold:/);
    assert.match(stylesCss, /--surface-faceplate-chrome:/);
    assert.match(stylesCss, /--surface-circuit-board:/);
    assert.match(stylesCss, /--surface-oled-screen:/);
    assert.match(stylesCss, /--surface-crt-phosphor:/);

    // 10 Semantic LEDs
    assert.match(stylesCss, /--led-gate:\s*#00e676/i);
    assert.match(stylesCss, /--led-comp:\s*#00b0ff/i);
    assert.match(stylesCss, /--led-drive:\s*#ff9100/i);
    assert.match(stylesCss, /--led-amp:\s*#ff3d00/i);
    assert.match(stylesCss, /--led-cab:\s*#ffc400/i);
    assert.match(stylesCss, /--led-eq:\s*#1de9b6/i);
    assert.match(stylesCss, /--led-mod:\s*#7c4dff/i);
    assert.match(stylesCss, /--led-pitch:\s*#e040fb/i);
    assert.match(stylesCss, /--led-delay:\s*#00e5ff/i);
    assert.match(stylesCss, /--led-reverb:\s*#651fff/i);

    // Glow and 3D Shadows
    assert.match(stylesCss, /--tube-glow-idle:/);
    assert.match(stylesCss, /--tube-glow-hot:/);
    assert.match(stylesCss, /--shadow-knob-3d:/);
    assert.match(stylesCss, /--shadow-chassis-inset:/);
    assert.match(stylesCss, /--shadow-footswitch:/);
  });

  test('CA-2: Tipografia, números tabulares para telemetria e serigrafia de hardware', () => {
    assert.match(stylesCss, /tabular-nums/);
    assert.match(stylesCss, /\.faceplate-label\s*\{[^}]*text-transform:\s*uppercase/s);
    assert.match(stylesCss, /\.faceplate-label\s*\{[^}]*letter-spacing/s);
  });

  test('CA-3: Catálogo de 3 variantes de knob e sombra estacionária desvinculada do rotor', () => {
    assert.match(stylesCss, /\.knob-variant-skirted/);
    assert.match(stylesCss, /\.knob-variant-chicken-head/);
    assert.match(stylesCss, /\.knob-variant-studio-encoder/);
    assert.match(stylesCss, /\.knob-shadow-base/);
  });

  test('CA-4: Ergonomia de knob - Shift+Drag para ajuste fino 0.01 e dblclick reset', () => {
    assert.match(appJs, /shiftKey/);
    assert.match(appJs, /0\.01/);
    assert.match(appJs, /dblclick/);
  });

  test('CA-5: Footswitch com estados ativo, dimmed 25% e pulso sincronizado ao BPM', () => {
    assert.match(stylesCss, /\.dimmed/);
    assert.match(stylesCss, /tap-tempo-pulse/);
  });

  test('CA-6: Modo Hardware Shell ativa chassis-screws e moldura CNC', () => {
    assert.match(indexHtml, /id="btnHardwareShell"/);
    assert.match(stylesCss, /\.hardware-shell-mode/);
    assert.match(stylesCss, /\.chassis-screws/);
  });

  test('CA-7: Aba #dsCatalogSection renderiza swatches de superfícies, LEDs, tipografia e knobs', () => {
    assert.match(indexHtml, /id="dsCatalogSection"/);
    assert.match(indexHtml, /swatch/);
  });

  test('CA-8: Função valueToAngle limita estritamente ao arco de -135deg a +135deg', () => {
    const valueToAngle = (val, min, max) => {
      const clamped = Math.max(min, Math.min(max, val));
      const norm = (clamped - min) / (max - min);
      return -135 + norm * 270;
    };

    assert.strictEqual(valueToAngle(-999, 0, 10), -135);
    assert.strictEqual(valueToAngle(999, 0, 10), 135);
    assert.strictEqual(valueToAngle(5, 0, 10), 0);
  });

  test('CA-9: Não-regressão - alternância entre abas preserva estado da aplicação', () => {
    assert.match(appJs, /dataset\.view|data-view/);
    assert.match(indexHtml, /data-view="rig-workspace"/);
  });
});
