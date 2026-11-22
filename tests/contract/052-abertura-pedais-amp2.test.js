// Spec 052 — CA-1 (referência visual), CA-2 (abertura animada SVG offline),
// CA-3 (designs skeuomórficos de modulação, pitch e wah) e CA-4 (mini-cabeça AMP 2 no Estúdio).
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { JSDOM } from 'jsdom';

const ROOT = process.cwd();

function setupDom(reducedMotion = false) {
  const dom = new JSDOM('<!doctype html><html><body><div id="app"></div></body></html>', {
    url: 'http://localhost/',
  });
  globalThis.window = dom.window;
  globalThis.document = dom.window.document;
  for (const k of [
    'HTMLElement',
    'customElements',
    'Element',
    'SVGElement',
    'Node',
    'Event',
    'MouseEvent',
    'KeyboardEvent',
    'WheelEvent',
  ]) {
    globalThis[k] = dom.window[k];
  }
  globalThis.PointerEvent = dom.window.MouseEvent;
  dom.window.matchMedia = (q) => ({
    matches: reducedMotion && String(q).includes('prefers-reduced-motion'),
    media: q,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  });
  globalThis.matchMedia = dom.window.matchMedia;
  return dom;
}

setupDom(false);

const { AppStore } = await import('../../apps/sonix-app/ui/src/store.ts');
const { CONTROLS_REGISTRY, findControl } = await import('../../apps/sonix-app/ui/src/controls.registry.ts');
const { createPedalElement } = await import('../../apps/sonix-app/ui/src/components/Pedal.ts');
const { renderBibliotecaScreen } = await import('../../apps/sonix-app/ui/src/screens/Biblioteca.ts');
const { renderEstudioScreen } = await import('../../apps/sonix-app/ui/src/screens/Estudio.ts');
const { mountAppShell } = await import('../../apps/sonix-app/ui/src/shell.ts');

function makeDelayedTransport() {
  let releaseGetState;
  const writes = [];
  const stateReady = new Promise((r) => {
    releaseGetState = r;
  });
  return {
    writes,
    releaseBootstrap: () => releaseGetState({}),
    transport: {
      setParam: (k, v) => writes.push({ key: k, value: v }),
      setParamsPatch: (patch) => {
        for (const [k, v] of Object.entries(patch)) writes.push({ key: k, value: v });
      },
      getState: () => stateReady,
      listPresets: async () => ({ presets: [{ id: 'brit800', name: 'Brit 800 Crunch' }] }),
      getDevices: async () => ({ devices: [] }),
      getAudioStatus: async () => ({ connected: true, output_device: 'Monitor' }),
      startStreams: async () => {},
      setStreamVisible: async () => {},
      setFullscreen: async () => {},
      getScope: async () => ({}),
    },
  };
}

test('CA-1: docs/ui-desktop/canvas/ e docs/ui-desktop/referencia/ têm referências atualizadas para pedais faltantes e AMP 2', () => {
  const pedalDc = fs.readFileSync(path.join(ROOT, 'docs/ui-desktop/canvas/Pedal.dc.html'), 'utf8');
  const pedaisDc = fs.readFileSync(path.join(ROOT, 'docs/ui-desktop/canvas/Pedais.dc.html'), 'utf8');
  const mainDc = fs.readFileSync(path.join(ROOT, 'docs/ui-desktop/canvas/Main.dc.html'), 'utf8');

  for (const id of ['trichorus', 'flanger', 'phaser90', 'tremolo', 'univibe', 'rotary', 'whammy', 'drop', 'harmonizer', 'detune', 'wah']) {
    assert.ok(pedalDc.includes(`${id}:`), `CA-1: Pedal.dc.html deve definir '${id}'`);
    assert.ok(pedaisDc.includes(`'${id}'`), `CA-1: Pedais.dc.html deve exibir '${id}'`);
  }
  assert.ok(mainDc.includes('AMP 2'), 'CA-1: Main.dc.html deve incluir o mini-cabeçote AMP 2');

  assert.ok(fs.existsSync(path.join(ROOT, 'docs/ui-desktop/referencia/Pedais.png')), 'CA-1: Pedais.png atualizado deve existir');
  assert.ok(fs.existsSync(path.join(ROOT, 'docs/ui-desktop/referencia/Main.png')), 'CA-1: Main.png atualizado deve existir');
});

test('CA-2: splash screen SVG SONIX PRO // AMPFORGE™ permanece durante IPC atrasado, fecha após bootstrap e respeita reduced-motion', async () => {
  setupDom(false);
  const { transport, releaseBootstrap } = makeDelayedTransport();
  const store = new AppStore(transport);
  const root = document.getElementById('app');
  const unmount = mountAppShell(root, store);

  const bootPromise = store.bootstrap();
  // Splash deve estar visível imediatamente antes da resolução do IPC
  await new Promise((r) => setTimeout(r, 25));
  const splash = root.querySelector('[data-region="splash"]');
  assert.ok(splash, 'CA-2: splash screen [data-region="splash"] deve existir no shell');
  assert.equal(splash.getAttribute('data-splash-active'), 'true', 'CA-2: splash deve permanecer ativo enquanto bootstrap() aguarda IPC');
  assert.notEqual(splash.style.display, 'none', 'CA-2: splash não pode ser ocultado antes do bootstrap concluir');
  assert.ok(splash.querySelector('svg'), 'CA-2: splash deve usar emblema SVG vetorial embutido');
  assert.ok(splash.textContent?.includes('SONIX PRO') && splash.textContent?.includes('AMPFORGE'), 'CA-2: splash deve exibir marca SONIX PRO // AMPFORGE™');
  assert.equal(splash.getAttribute('data-reduced-motion'), 'false');

  // Conclui bootstrap -> splash encerra
  releaseBootstrap();
  await bootPromise;
  assert.equal(store.bootstrapped, true, 'CA-2: store.bootstrapped deve ser true após bootstrap()');
  assert.equal(splash.getAttribute('data-splash-active'), 'false', 'CA-2: splash deve fechar ao fim do bootstrap()');
  assert.equal(splash.style.display, 'none', 'CA-2: splash oculto via display:none após bootstrap()');
  unmount();

  // Reduced motion
  setupDom(true);
  const rec2 = makeDelayedTransport();
  const storeRm = new AppStore(rec2.transport);
  const rootRm = document.getElementById('app');
  const unmountRm = mountAppShell(rootRm, storeRm);
  const p2 = storeRm.bootstrap();
  const splashRm = rootRm.querySelector('[data-region="splash"]');
  assert.equal(splashRm?.getAttribute('data-reduced-motion'), 'true', 'CA-2: deve detectar prefers-reduced-motion: reduce');
  assert.equal(splashRm?.style.animation, 'none', 'CA-2: animações devem estar desativadas com prefers-reduced-motion');
  rec2.releaseBootstrap();
  await p2;
  assert.equal(splashRm?.getAttribute('data-splash-active'), 'false');
  unmountRm();
});

test('CA-3: Pedal.ts e Biblioteca.ts renderizam todos os modelos de Modulação, Pitch e Wah com controles registrados', () => {
  setupDom(false);
  const { transport, writes } = makeDelayedTransport();
  const store = new AppStore(transport);

  const requiredPedals = [
    'ce2', 'trichorus', 'flanger', 'phaser90', 'tremolo', 'univibe', 'rotary',
    'whammy', 'drop', 'harmonizer', 'detune',
    'wah',
  ];

  for (const model of requiredPedals) {
    const el = createPedalElement(model, store);
    assert.ok(el instanceof HTMLElement, `CA-3: createPedalElement('${model}') deve retornar HTMLElement`);
    const w = parseInt(el.style.width, 10);
    const h = parseInt(el.style.height, 10);
    assert.ok(w >= 90 && h >= 140, `CA-3: pedal '${model}' precisa ter dimensões válidas, obteve ${w}x${h}`);
    assert.ok(el.querySelector('button[data-control-id]'), `CA-3: pedal '${model}' deve ter footswitch/treadle com data-control-id`);
  }

  // Controles do Wah registrados e ligados ao store -> backendKey
  for (const cid of ['wah_1.enabled', 'wah_1.position', 'wah_1.q', 'wah_1.mode']) {
    const desc = findControl(cid);
    assert.ok(desc, `CA-3: '${cid}' deve estar em controls.registry.ts`);
  }
  store.setControl('wah_1.enabled', true);
  store.setControl('wah_1.position', 0.85);
  assert.ok(writes.some((w) => w.key === 'wah_1.enabled' && w.value === true));
  assert.ok(writes.some((w) => w.key === 'wah_1.position' && Math.abs(w.value - 0.85) < 1e-4));

  // Biblioteca renderiza todos sem texto "design pendente"
  store.libraryTab = 'pedals';
  const bib = renderBibliotecaScreen(store);
  document.body.appendChild(bib);
  const bibTxt = bib.textContent || '';
  for (const title of ['CHORUS 3 VOZES (MONO)', 'TZ FLANGER', 'PHASER 90', 'TREMOLO', 'UNI-VIBE', 'ROTARY 147', 'WHAMMY', 'DROP CAPO', 'HARMONIZER', 'MICRO DETUNE', 'CRY WAH']) {
    assert.ok(bibTxt.toUpperCase().includes(title), `CA-3: Biblioteca deve listar '${title}'`);
  }
  bib.remove();
});

test('CA-4: palco do Estúdio exibe mini-cabeça AMP 2 refletindo amp_2.model e controles gain/bass/middle/treble/master', () => {
  setupDom(false);
  const { transport, writes } = makeDelayedTransport();
  const store = new AppStore(transport);
  store.setControl('split.enabled', true);
  store.setControl('amp_2.enabled', true);
  store.setControl('amp_2.model', 'ac30_tb');

  const estudio = renderEstudioScreen(store);
  document.body.appendChild(estudio);

  const miniAmp2 = estudio.querySelector('[data-region="amp2-mini"]');
  assert.ok(miniAmp2, 'CA-4: palco do Estúdio deve exibir [data-region="amp2-mini"]');
  assert.ok(miniAmp2.textContent?.toUpperCase().includes('AC30'), `CA-4: mini-AMP 2 deve refletir amp_2.model=ac30_tb, obteve: ${miniAmp2.textContent}`);

  for (const cid of ['amp_2.gain', 'amp_2.bass', 'amp_2.middle', 'amp_2.treble', 'amp_2.master']) {
    assert.ok(findControl(cid), `CA-4: '${cid}' deve estar registrado em controls.registry.ts`);
    const knob = miniAmp2.querySelector(`[data-control-id="${cid}"]`);
    assert.ok(knob, `CA-4: mini-AMP 2 deve renderizar knob '${cid}'`);
  }

  store.setControl('amp_2.gain', 8.2);
  store.setControl('amp_2.master', 4.5);
  assert.ok(writes.some((w) => w.key === 'amp_2.gain' && Math.abs(w.value - 8.2) < 1e-4));
  assert.ok(writes.some((w) => w.key === 'amp_2.master' && Math.abs(w.value - 4.5) < 1e-4));

  // Registro completo tem 0 pendências
  const pending = CONTROLS_REGISTRY.filter((c) => c.pendingSpec !== null);
  assert.equal(pending.length, 0, 'Spec 052 concluída não deve deixar controles com pendingSpec != null');
  estudio.remove();
});
