import { test, suite } from 'node:test';
import assert from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import vm from 'node:vm';

function createDomHarness() {
  const elements = new Map();
  const makeEl = (id, tag = 'div') => {
    const classes = new Set();
    const listeners = {};
    const children = [];
    const el = {
      id,
      tagName: tag.toUpperCase(),
      value: '',
      checked: false,
      textContent: '',
      innerHTML: '',
      style: { setProperty: () => {}, getPropertyValue: () => '' },
      dataset: {},
      classList: {
        add: (...c) => c.forEach((x) => classes.add(x)),
        remove: (...c) => c.forEach((x) => classes.delete(x)),
        toggle: (c, force) => {
          if (force === undefined) {
            if (classes.has(c)) {
              classes.delete(c);
              return false;
            }
            classes.add(c);
            return true;
          }
          if (force) classes.add(c);
          else classes.delete(c);
          return force;
        },
        contains: (c) => classes.has(c),
      },
      addEventListener: (evt, fn) => {
        if (!listeners[evt]) listeners[evt] = [];
        listeners[evt].push(fn);
      },
      dispatchEvent: (evt) => {
        const list = listeners[evt.type] || [];
        for (const fn of list) fn(evt);
      },
      click: () => {
        const list = listeners['click'] || [];
        for (const fn of list) fn({ target: el, preventDefault: () => {} });
      },
      appendChild: (child) => {
        child.parentNode = el;
        children.push(child);
        return child;
      },
      parentNode: { appendChild: () => {} },
      querySelectorAll: () => [],
      querySelector: (sel) => makeEl(`${id}_${sel}`),
      getBoundingClientRect: () => ({ width: 200, height: 100, left: 0, top: 0 }),
      setAttribute: () => {},
      children,
      _listeners: listeners,
    };
    return el;
  };

  const trackedIds = [
    'btnCalibrateInput',
    'inputCalibrationWizard',
    'calibStep1',
    'calibStep2',
    'calibStep3',
    'calibResultText',
    'calibDiagnosticText',
    'btnStartCalib',
    'btnCloseCalib',
    'btnToggleTuner',
    'tunerBar',
    'tunerNoteText',
    'btnTunerOutputMode',
    'btnHardwareShell',
    'btnAmpPower',
    'ampPowerLamp',
    'ampChassisBanner',
    'crtScopeCanvas',
    'gridMatrixContainer',
    'ampKnobsContainer',
    'footswitchRow',
    'footswitchArray',
    'ampKnobsGrid',
    'cabMic1',
    'cabMic2',
    'cabCanvas',
  ];
  for (const id of trackedIds) {
    elements.set(id, makeEl(id));
  }
  const needleEl = makeEl('strobeNeedle');

  const fetchCalls = [];
  const doc = {
    body: makeEl('body', 'body'),
    getElementById: (id) => {
      if (!elements.has(id)) elements.set(id, makeEl(id));
      return elements.get(id);
    },
    querySelector: (sel) => {
      if (sel === '.strobe-center-needle') return needleEl;
      return null;
    },
    querySelectorAll: () => [],
    createElement: (tag) => makeEl(`anon_${Math.random()}`, tag),
    addEventListener: (evt, fn) => {
      doc._listeners[evt] = doc._listeners[evt] || [];
      doc._listeners[evt].push(fn);
    },
    _listeners: {},
  };

  const sandbox = {
    console,
    Math,
    Number,
    String,
    Boolean,
    Array,
    Object,
    JSON,
    Promise,
    setTimeout: (fn) => {
      if (typeof fn === 'function') fn();
      return 1;
    },
    clearTimeout: () => {},
    setInterval: () => 1,
    clearInterval: () => {},
    requestAnimationFrame: () => 1,
    document: doc,
    window: {
      __SONIX_TOKEN__: 'test-token',
      addEventListener: (evt, fn) => {
        sandbox.window._listeners[evt] = sandbox.window._listeners[evt] || [];
        sandbox.window._listeners[evt].push(fn);
      },
      _listeners: {},
    },
    fetch: async (url, opts = {}) => {
      fetchCalls.push({ url, method: opts.method || 'GET', body: opts.body, headers: opts.headers });
      if (url === '/api/tuner') {
        return {
          ok: true,
          json: async () => ({
            note: 'E2',
            frequency_hz: 82.41,
            cents: -0.4,
            in_tune: true,
            active: true,
            muted: true,
          }),
        };
      }
      if (url === '/api/input/calibrate') {
        const parsed = opts.body ? JSON.parse(opts.body) : {};
        if (parsed.phase === 'silence') {
          return {
            ok: true,
            json: async () => ({
              phase: 'silence',
              noise_floor_rms_db: -70.0,
            }),
          };
        }
        return {
          ok: true,
          json: async () => ({
            proposed_boost: 7.94,
            proposed_gate_threshold_db: -46.0,
            diagnostic: 'ok',
            noise_floor_rms_db: -70.0,
            raw_peak_db: -30.0,
          }),
        };
      }
      return {
        ok: true,
        json: async () => ({
          'input.boost': 7.94,
          'gate.threshold_db': -46.0,
        }),
      };
    },
  };
  sandbox.globalThis = sandbox;
  return { elements, needleEl, fetchCalls, sandbox };
}

suite('Spec 029: Afinador Funcional End-to-End e Calibração de Entrada Ao Vivo', () => {
  test('Execução de testes de contrato Rust DSP e Desktop (CA-1, CA-2, CA-4 a CA-8)', () => {
    const res = spawnSync(
      'cargo',
      ['test', '-p', 'sonix-desktop', '--test', 'contract_029_tuner_and_calibration'],
      { encoding: 'utf-8', shell: true }
    );
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes Rust da Spec 029:\n${res.stdout}\n${res.stderr}`
    );
  });

  test('CA-1 UI: app.js lê data.note e data.cents (sem note_name/cents_deviation) e atualiza #tunerNoteText + botão Mute/Bypass', async () => {
    const appJs = readFileSync(resolve('design-system/app.js'), 'utf-8');
    const html = readFileSync(resolve('design-system/index.html'), 'utf-8');

    assert.ok(
      !appJs.includes('data.note_name') && !appJs.includes('data.cents_deviation'),
      'app.js não deve mais referenciar data.note_name nem data.cents_deviation'
    );
    assert.ok(
      html.includes('id="btnTunerOutputMode"'),
      'Barra do afinador deve expor botão rápido #btnTunerOutputMode (Mute/Bypass)'
    );

    const { elements, needleEl, sandbox } = createDomHarness();
    vm.runInNewContext(appJs, sandbox);
    for (const fn of sandbox.window._listeners.DOMContentLoaded || []) {
      fn();
    }

    elements.get('btnToggleTuner').click();
    await sandbox.updateStrobeTunerFromSignal();

    assert.strictEqual(elements.get('tunerNoteText').textContent, 'E2 (-0.4¢)');
    assert.notStrictEqual(needleEl.style.transform, 'translateX(0px)');
  });

  test('CA-3 UI: #btnCalibrateInput abre assistente, #btnStartCalib executa fases silence->strum e #btnCloseCalib oculta', async () => {
    const appJs = readFileSync(resolve('design-system/app.js'), 'utf-8');
    const { elements, fetchCalls, sandbox } = createDomHarness();

    vm.runInNewContext(appJs, sandbox);
    for (const fn of sandbox.window._listeners.DOMContentLoaded || []) {
      fn();
    }

    const wizard = elements.get('inputCalibrationWizard');
    elements.get('btnCalibrateInput').click();
    assert.notStrictEqual(
      wizard.style.display,
      'none',
      'Clicar em #btnCalibrateInput deve tornar #inputCalibrationWizard visível'
    );

    await elements.get('btnStartCalib').click();
    await new Promise((r) => setTimeout(r, 10));

    const calibCalls = fetchCalls.filter((c) => c.url === '/api/input/calibrate');
    assert.strictEqual(calibCalls.length, 2, 'Deve chamar /api/input/calibrate para silence e strum');
    assert.strictEqual(JSON.parse(calibCalls[0].body).phase, 'silence');
    assert.strictEqual(JSON.parse(calibCalls[1].body).phase, 'strum');

    assert.ok(
      elements.get('calibResultText').textContent.includes('7.94'),
      'Resultado da calibração deve exibir boost proposto no passo 3'
    );

    elements.get('btnCloseCalib').click();
    assert.strictEqual(
      wizard.style.display,
      'none',
      'Clicar em #btnCloseCalib deve ocultar #inputCalibrationWizard'
    );
  });
});
