import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { JSDOM } from 'jsdom';

const ROOT = process.cwd();

let activeDom = null;
function setupDom() {
  if (!activeDom) {
    activeDom = new JSDOM('<!doctype html><html><body><div id="app"></div></body></html>', {
      url: 'http://localhost/',
    });
    globalThis.window = activeDom.window;
    globalThis.document = activeDom.window.document;
    globalThis.HTMLElement = activeDom.window.HTMLElement;
    globalThis.customElements = activeDom.window.customElements;
    globalThis.Element = activeDom.window.Element;
    globalThis.SVGElement = activeDom.window.SVGElement;
    globalThis.Node = activeDom.window.Node;
    globalThis.Event = activeDom.window.Event;
    globalThis.PointerEvent = activeDom.window.MouseEvent;
    globalThis.MouseEvent = activeDom.window.MouseEvent;
    globalThis.KeyboardEvent = activeDom.window.KeyboardEvent;
    globalThis.WheelEvent = activeDom.window.WheelEvent;
  } else {
    activeDom.window.document.body.innerHTML = '<div id="app"></div>';
  }
  return activeDom;
}

setupDom();

const { AppStore } = await import('../../apps/sonix-app/ui/src/store.ts');
const { findControl } = await import('../../apps/sonix-app/ui/src/controls.registry.ts');
const { ParamCoalescer } = await import('../../apps/sonix-app/ui/src/transport.ts');
const { renderAudioScreen } = await import('../../apps/sonix-app/ui/src/screens/Audio.ts');
const { renderBancadaScreen } = await import('../../apps/sonix-app/ui/src/screens/Bancada.ts');
const { renderEstudioScreen } = await import('../../apps/sonix-app/ui/src/screens/Estudio.ts');
const { mountAppShell } = await import('../../apps/sonix-app/ui/src/shell.ts');

function makeMockTransport(overrides = {}) {
  const base = {
    getStatus: async () => ({ status: 'online', engine: 'AmpForge RealTime', backend: 'wasapi' }),
    getAudioStatus: async () => ({
      backend: 'wasapi',
      mode: 'wasapi_shared',
      raw_processing: false,
      sample_rate: 48000,
      input_period_frames: 128,
      output_period_frames: 128,
      input_device: 'Focusrite USB',
      output_device: 'Focusrite USB',
      driver_input_latency_ms: 2.67,
      driver_output_latency_ms: 2.67,
      measured_roundtrip_ms: null,
      estimated_roundtrip_ms: 5.34,
      underruns: 0,
      adc_clips: 0,
      mode_capabilities: {
        wasapi_shared: true,
        wasapi_shared_low_latency: true,
        wasapi_exclusive: false,
        asio: false,
      },
    }),
    getDevices: async () => ({
      devices: [{ id: 'f1', name: 'Focusrite USB', is_input: true, is_output: true }],
    }),
    getState: async () => ({ 'amp_1.gain': 5.0, 'amp_1.model': 'jcm800_2203' }),
    listPresets: async () => ({
      presets: [{ id: 'custom_rock', name: 'Custom Rock', setlist: 'LIVE' }],
    }),
    loadPreset: async (payload) => {
      const id = typeof payload === 'string' ? payload : String(payload.id ?? '');
      return {
        status: 'ok',
        loaded: id,
        state: { 'amp_1.model': id, 'amp_1.gain': 7.5 },
      };
    },
    setParams: async (patch) => ({ status: 'ok', applied: Object.keys(patch).length }),
    setParam: (k, v) => {
      void base.setParams({ [k]: v });
    },
    setParamsPatch: (p) => {
      void base.setParams(p);
    },
    minimizeWindow: async () => {},
    toggleMaximizeWindow: async () => {},
    closeWindow: async () => {},
    setFullscreen: async () => {},
    getScope: async (mode) => {
      if (mode === 'transfer') {
        return { transfer: [{ x: -0.5, y: -0.25 }, { x: 0.5, y: 0.42 }] };
      }
      if (mode === 'waveform') {
        return { waveform: { di: [0, 0.5, 0, -0.5], output: [0, 0.4, 0, -0.4] } };
      }
      return { tonestack: [{ f: 100, db: -2 }, { f: 1000, db: 1 }] };
    },
    startStreams: async () => {},
    setStreamVisible: async () => {},
    measureLatency: async () => ({ measured_roundtrip_ms: null, estimated_roundtrip_ms: 5.34 }),
    setAudioDevice: async (p) => p,
    ...overrides,
  };
  return base;
}

test('Spec 035 CA-1: arraste contínuo de knob em 120 px com telemetria a 45 Hz não desmonta o nó DOM e atinge valor máximo', async () => {
  const dom = setupDom();
  let telemetryCb = null;
  const setCalls = [];
  const transport = makeMockTransport({
    startStreams: async (handlers) => {
      telemetryCb = handlers.onTelemetry;
    },
    setParams: async (p) => {
      setCalls.push(p);
      return { status: 'ok', applied: 1 };
    },
    setParam: (k, v) => {
      setCalls.push({ [k]: v });
    },
  });

  const store = new AppStore(transport);
  const appRoot = document.getElementById('app');
  mountAppShell(appRoot, store);
  await store.bootstrap();

  // Encontra o knob amp_1.gain na tela principal (Estúdio)
  const knobEl = appRoot.querySelector('sonix-knob[data-control-id="amp.gain"]');
  assert.ok(knobEl, 'Knob amp.gain deve existir na DOM');
  knobEl.setPointerCapture = () => {};
  knobEl.releasePointerCapture = () => {};

  const timer = setInterval(() => {
    telemetryCb?.({
      seq: Math.floor(Math.random() * 10000),
      payload: {
        input_peak_db: -24,
        output_peak_db: -12,
        cpu_usage_pct: 22.3,
        gate_state: 'open',
        gate_gr_db: -4.0,
        waveform: [0.1, -0.1],
      },
    });
  }, 22); // ~45 Hz

  try {
    knobEl.dispatchEvent(new dom.window.MouseEvent('pointerdown', { bubbles: true, clientY: 200, pointerId: 1 }));
    for (let i = 1; i <= 10; i++) {
      knobEl.dispatchEvent(
        new dom.window.MouseEvent('pointermove', { bubbles: true, clientY: 200 - i * 12, pointerId: 1 })
      );
      await new Promise((r) => setTimeout(r, 25));
      assert.equal(document.contains(knobEl), true, `Knob desmontado durante arraste no passo ${i}`);
    }
    knobEl.dispatchEvent(new dom.window.MouseEvent('pointerup', { bubbles: true, clientY: 80, pointerId: 1 }));
  } finally {
    clearInterval(timer);
  }

  const finalGain = Number(store.params['amp_1.gain']);
  assert.ok(finalGain >= 9.5, `CA-1: arraste de 120 px deve levar ganho perto de 10.0, obtido ${finalGain}`);
  assert.ok(setCalls.length >= 1, 'CA-1: ParamCoalescer deve enviar atualizações ao backend');
});

test('Spec 035 CA-2: telemetria usa cpu_usage_pct e gate_gr_db do backend na barra e UI', async () => {
  setupDom();
  const store = new AppStore(makeMockTransport());
  store.applyTelemetry({
    input_peak_db: -20,
    output_peak_db: -10,
    cpu_usage_pct: 37.5,
    gate_state: 'closed',
    gate_gr_db: -18.2,
    waveform: [],
  });

  const appRoot = document.getElementById('app');
  mountAppShell(appRoot, store);
  assert.ok(appRoot.textContent.includes('37.5%'), 'CA-2: Statusbar deve exibir cpu_usage_pct = 37.5%');

  store.selectedBlock = 'gate';
  const est = renderEstudioScreen(store);
  assert.ok(est.textContent.includes('-18.2'), 'CA-2: Painel de Gate deve exibir gate_gr_db = -18.2 dB');
});

test('Spec 035 CA-3, CA-4 e CA-9: osciloscópio plota pontos do backend, modos batem com backend e erro mostra sem dados do motor', async () => {
  setupDom();
  const store = new AppStore(makeMockTransport());
  const regScope = findControl('chassi.crt_mode');
  assert.deepEqual(regScope.options, ['waveform', 'transfer', 'tonestack']);

  // CA-3: transfer points vêm exatamente do backend sem curva sintética
  store.crtMode = 'transfer';
  store.scopeError = null;
  store.scope = { transfer: [{ x: -0.5, y: -0.25 }, { x: 0.5, y: 0.42 }] };
  const bancada = renderBancadaScreen(store);
  const pathEl = bancada.querySelector('svg path[data-scope-series="transfer"]');
  assert.ok(pathEl, 'CA-3: SVG do osciloscópio deve marcar path com data-scope-series="transfer"');
  const pointsCount = Number(pathEl.getAttribute('data-point-count'));
  assert.equal(pointsCount, 2, 'CA-3: Deve plotar exatamente os 2 pontos retornados pelo backend, não 49 sintéticos');

  // CA-9: erro HTTP 400 / falha no getScope mostra "sem dados do motor" e não desenha curva sintética
  const errStore = new AppStore(
    makeMockTransport({
      getScope: async () => {
        throw new Error('IPC status 400');
      },
    })
  );
  errStore.crtMode = 'transfer';
  await errStore.refreshScope();
  assert.ok(errStore.scopeError, 'CA-9: scopeError deve ser marcado quando backend falha');
  const errBancada = renderBancadaScreen(errStore);
  assert.ok(
    errBancada.textContent.toLowerCase().includes('sem dados do motor'),
    'CA-9: CRT deve exibir "sem dados do motor" em falha'
  );
  assert.equal(
    errBancada.querySelector('svg path[data-scope-series]'),
    null,
    'CA-9: Não deve desenhar curva sintética quando scopeError estiver ativo'
  );
});

test('Spec 035 CA-5 e CA-6: zero presets hardcoded na UI e loadPreset consome { status, loaded, state }', async () => {
  setupDom();
  const storeSrc = fs.readFileSync(path.join(ROOT, 'apps/sonix-app/ui/src/store.ts'), 'utf8');
  const transportSrc = fs.readFileSync(path.join(ROOT, 'apps/sonix-app/ui/src/transport.ts'), 'utf8');
  for (const forbidden of ['DEFAULT_PRESETS', 'plexi_jumped', 'cali_modern', 'twin_clean', 'ac30_chime']) {
    assert.ok(!storeSrc.includes(forbidden), `CA-5: store.ts não pode conter preset hardcoded '${forbidden}'`);
    assert.ok(!transportSrc.includes(forbidden), `CA-5: transport.ts não pode conter preset hardcoded '${forbidden}'`);
  }

  const store = new AppStore(makeMockTransport());
  await store.bootstrap();
  assert.deepEqual(
    store.presets.map((p) => p.id),
    ['custom_rock'],
    'CA-5: lista de presets na store deve vir unicamente do backend'
  );

  await store.loadPreset('recto');
  assert.equal(store.params['amp_1.model'], 'recto', 'CA-6: loadPreset deve aplicar state retornado');
  assert.equal(store.params['amp_1.gain'], 7.5, 'CA-6: loadPreset deve aplicar parâmetros de state');
});

test('Spec 035 CA-7 & Spec 036 CA-4, CA-7, CA-9: Audio.ts com lista vazia, modos desabilitados e latência estimada', () => {
  setupDom();
  const store = new AppStore(makeMockTransport());
  store.devices = [];
  store.audioStatus = {
    backend: 'wasapi',
    mode: 'wasapi_shared',
    raw_processing: false,
    sample_rate: 48000,
    input_period_frames: 128,
    output_period_frames: 128,
    input_device: '',
    output_device: '',
    driver_input_latency_ms: 2.67,
    driver_output_latency_ms: 2.67,
    measured_roundtrip_ms: null,
    estimated_roundtrip_ms: 5.34,
    underruns: 0,
    adc_clips: 0,
    mode_capabilities: {
      wasapi_shared: true,
      wasapi_shared_low_latency: true,
      wasapi_exclusive: false,
      asio: false,
    },
  };

  const screen = renderAudioScreen(store);
  const text = screen.textContent.toLowerCase();
  assert.ok(
    text.includes('nenhum dispositivo de entrada') || text.includes('nenhum dispositivo encontrado'),
    'Spec 035 CA-7 / 036 CA-9: deve informar explicitamente que nenhum dispositivo foi encontrado'
  );
  assert.ok(
    text.includes('estimada'),
    'Spec 036 CA-4: sem medição por loopback real, UI deve rotular latência como estimada'
  );

  const asioOpt = screen.querySelector('option[value="asio"]');
  assert.ok(asioOpt, 'Opção ASIO deve existir no seletor de modo/driver');
  assert.equal(asioOpt.disabled, true, 'Spec 036 CA-7: modo ASIO não suportado deve estar disabled');
  assert.ok(
    (asioOpt.title || '').includes('indisponível') || (asioOpt.title || '').includes('driver ASIO'),
    'Spec 036 CA-7 / Spec 051 CA-24: opção desabilitada deve conter dica clara em português sem jargão Spec 0NN'
  );
});

test('Spec 035 CA-8: fixtures JSON geradas pelo Rust existem e batem com o parsing TypeScript da store', () => {
  setupDom();
  const fixturesDir = path.join(ROOT, 'tests/fixtures/ipc-schemas');
  const required = [
    'telemetry.json',
    'audio_status.json',
    'devices.json',
    'scope_transfer.json',
    'scope_waveform.json',
    'scope_tonestack.json',
    'presets_list.json',
    'presets_load.json',
  ];
  for (const file of required) {
    assert.ok(fs.existsSync(path.join(fixturesDir, file)), `CA-8: Fixture ausente: ${file}`);
  }

  const readJson = (f) => JSON.parse(fs.readFileSync(path.join(fixturesDir, f), 'utf8'));
  const tel = readJson('telemetry.json');
  assert.equal(typeof tel.cpu_usage_pct, 'number');
  assert.equal(typeof tel.gate_gr_db, 'number');

  const aud = readJson('audio_status.json');
  assert.ok('estimated_roundtrip_ms' in aud);
  assert.ok('raw_processing' in aud);

  const devs = readJson('devices.json');
  assert.ok(Array.isArray(devs.devices));

  const presetsLoad = readJson('presets_load.json');
  assert.equal(presetsLoad.status, 'ok');
  assert.ok(presetsLoad.loaded);
  assert.equal(typeof presetsLoad.state, 'object');
});

test('Spec 035 CA-10: não-regressão no ParamCoalescer (latest-wins e no máximo 1 envio em voo)', async () => {
  const sent = [];
  let releaseFirst;
  let inFlight = 0;
  let maxInFlight = 0;

  const coalescer = new ParamCoalescer({
    rafFn: (cb) => Number(setTimeout(() => cb(Date.now()), 0)),
    invokeFn: async (_cmd, args) => {
      inFlight++;
      maxInFlight = Math.max(maxInFlight, inFlight);
      sent.push(args.params);
      if (sent.length === 1) {
        await new Promise((r) => {
          releaseFirst = r;
        });
      }
      inFlight--;
      return { status: 'ok', applied: Object.keys(args.params).length };
    },
  });

  coalescer.enqueue('amp_1.gain', 1.0);
  const p1 = coalescer.flushNow();
  coalescer.enqueue('amp_1.gain', 2.0);
  coalescer.enqueue('amp_1.gain', 9.0);
  const p2 = coalescer.flushNow();
  releaseFirst();
  await Promise.all([p1, p2]);

  assert.equal(maxInFlight, 1);
  assert.equal(sent.length, 2);
  assert.equal(sent[1]['amp_1.gain'], 9.0);
});
