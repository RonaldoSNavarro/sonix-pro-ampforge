import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { JSDOM } from 'jsdom';

const ROOT = process.cwd();
const UI_SRC = path.join(ROOT, 'apps', 'sonix-app', 'ui', 'src');

function setupDom() {
  const dom = new JSDOM('<!doctype html><html><body><div id="app"></div></body></html>', {
    url: 'http://localhost/',
  });
  globalThis.window = dom.window;
  globalThis.document = dom.window.document;
  globalThis.HTMLElement = dom.window.HTMLElement;
  globalThis.customElements = dom.window.customElements;
  globalThis.Element = dom.window.Element;
  globalThis.SVGElement = dom.window.SVGElement;
  globalThis.Node = dom.window.Node;
  globalThis.Event = dom.window.Event;
  globalThis.PointerEvent = dom.window.MouseEvent;
  globalThis.MouseEvent = dom.window.MouseEvent;
  globalThis.KeyboardEvent = dom.window.KeyboardEvent;
  globalThis.WheelEvent = dom.window.WheelEvent;
  return dom;
}

setupDom();

const { AppStore } = await import('../../apps/sonix-app/ui/src/store.ts');
const { mountAppShell } = await import('../../apps/sonix-app/ui/src/shell.ts');

function walkTsFiles(dir) {
  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walkTsFiles(full));
    else if (entry.name.endsWith('.ts') || entry.name.endsWith('.tsx')) out.push(full);
  }
  return out;
}

test('CA-10: nenhum arquivo em apps/sonix-app/ui/src contém literais inventados Focusrite, -14.2 ou · 2 ms', () => {
  const forbidden = ['Focusrite', '-14.2', '· 2 ms'];
  for (const file of walkTsFiles(UI_SRC)) {
    const content = fs.readFileSync(file, 'utf8');
    for (const lit of forbidden) {
      assert.equal(
        content.includes(lit),
        false,
        `CA-10: literal proibido "${lit}" encontrado em ${path.relative(ROOT, file)}`,
      );
    }
  }
});

test('CA-10: barra de status reflete dispositivo vazio, entrada em silêncio, stream parado e não calibrada', () => {
  setupDom();
  const root = document.getElementById('app');
  const store = new AppStore({
    getStatus: async () => ({ status: 'online' }),
    getState: async () => ({}),
    getAudioStatus: async () => ({}),
    getTelemetry: async () => ({}),
  });

  store.audioStatus.output_device = '';
  store.audioStatus.connected = false;
  store.telemetry.input_peak_db = -120;
  store.calibrationWizard.phase = 'idle';

  const unmount = mountAppShell(root, store);
  const text = root.textContent || '';

  assert.match(text, /sem dispositivo/i, 'CA-10: output_device vazio deve mostrar "sem dispositivo"');
  assert.equal(text.includes('-14.2'), false, 'CA-10: nunca pode mostrar "-14.2"');
  assert.equal(text.includes('Motor ativo'), false, 'CA-10: "Motor ativo" não pode aparecer com stream parado');
  assert.equal(text.includes('entrada calibrada'), false, 'CA-10: "entrada calibrada" não pode aparecer sem calibração concluída');
  assert.match(text, /não calibrada/i, 'CA-10: deve mostrar "não calibrada" antes da calibração');
  assert.equal(text.includes('· 2 ms'), false, 'CA-10: latência fixa da ponte não pode aparecer');

  // Agora simula stream ativo e calibração concluída
  store.audioStatus.output_device = 'Minha Interface';
  store.audioStatus.connected = true;
  store.calibrationWizard.phase = 'done';
  store.emit('telemetry');

  const textActive = root.textContent || '';
  assert.match(textActive, /Motor ativo/, 'CA-10: "Motor ativo" deve aparecer com stream rodando');
  assert.match(textActive, /entrada calibrada/, 'CA-10: "entrada calibrada" deve aparecer após calibração concluída');

  unmount();
});
