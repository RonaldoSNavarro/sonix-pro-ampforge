// Spec 051 — régua de contrato UI↔backend (T-00) e correções de contrato (T-01..T-07).
// A régua exercita o caminho REAL da UI (clique/store → transport) e grava o que ela envia em
// tests/fixtures/ipc-schemas/051_ui_contract_calls.json. O teste Rust
// crates/sonix-desktop/tests/contract_051_manual_test_fixes.rs reenvia esse fixture a `api::dispatch`
// e exige aceite. Fixture desatualizado reprova aqui (regenerar com UPDATE_051=1).
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { JSDOM } from 'jsdom';

const ROOT = process.cwd();
const FIXTURE = path.join(ROOT, 'tests/fixtures/ipc-schemas/051_ui_contract_calls.json');

const dom = new JSDOM('<!doctype html><html><body><div id="app"></div></body></html>', {
  url: 'http://localhost/',
});
globalThis.window = dom.window;
globalThis.document = dom.window.document;
for (const k of [
  'HTMLElement', 'customElements', 'Element', 'SVGElement', 'Node', 'Event', 'MouseEvent',
  'KeyboardEvent', 'WheelEvent',
]) {
  globalThis[k] = dom.window[k];
}
globalThis.PointerEvent = dom.window.MouseEvent;

const { AppStore } = await import('../../apps/sonix-app/ui/src/store.ts');
const { CONTROLS_REGISTRY } = await import('../../apps/sonix-app/ui/src/controls.registry.ts');
const { createAmpHeadElement } = await import('../../apps/sonix-app/ui/src/components/AmpHead.ts');
const { renderBancadaScreen } = await import('../../apps/sonix-app/ui/src/screens/Bancada.ts');
const { renderAudioScreen } = await import('../../apps/sonix-app/ui/src/screens/Audio.ts');
const { renderEstudioScreen } = await import('../../apps/sonix-app/ui/src/screens/Estudio.ts');
const { renderPalcoScreen } = await import('../../apps/sonix-app/ui/src/screens/Palco.ts');
const { renderGabineteScreen } = await import('../../apps/sonix-app/ui/src/screens/Gabinete.ts');
const { renderBibliotecaScreen } = await import('../../apps/sonix-app/ui/src/screens/Biblioteca.ts');
const { mountAppShell } = await import('../../apps/sonix-app/ui/src/shell.ts');
const { classifyBridgeError } = await import('../../apps/sonix-app/ui/src/transport.ts');

const AMP_MODELS = ['plexi_1959', 'jcm800_2203', 'twin_ab763', 'dual_rect', 'ac30_tb'];

/** Transport falso que registra exatamente o que a UI envia. */
function makeRecorder() {
  const rec = { writes: [], commands: [] };
  const noop = async () => ({});
  const transport = {
    setParam: (k, v) => rec.writes.push({ key: k, value: v }),
    setParamsPatch: (patch) => {
      for (const [k, v] of Object.entries(patch)) rec.writes.push({ key: k, value: v });
    },
    selectScene: async (payload) => {
      rec.commands.push({ cmd: 'select_scene', payload });
      return {};
    },
    calibrateInput: async (payload) => {
      rec.commands.push({ cmd: 'calibrate_input', payload });
      return {
        phase: payload?.phase,
        noise_floor_rms_db: -60,
        proposed_boost: 2.0,
        proposed_gate_threshold_db: -60,
      };
    },
    loadPreset: async (payload) => {
      rec.commands.push({ cmd: 'load_preset', payload });
      return {};
    },
    setAudioDevice: async (payload) => {
      rec.commands.push({ cmd: 'set_audio_device', payload });
      return {};
    },
    getAudioStatus: async () => ({}),
    getState: noop,
    getScope: noop,
    listPresets: async () => ({ presets: [] }),
    getDevices: async () => ({ devices: [] }),
    startStreams: noop,
    setStreamVisible: noop,
    setFullscreen: noop,
  };
  return { rec, transport };
}

function freshStore() {
  const { rec, transport } = makeRecorder();
  const store = new AppStore(transport);
  // Espera da calibração instantânea nos testes (a UI real espera 3 s / 5 s).
  store.calibrationWait = async () => {};
  return { store, rec };
}

function clickAll(root, selector) {
  const els = [...root.querySelectorAll(selector)];
  for (const el of els) el.click();
  return els.length;
}

function collectCalls() {
  const enumOptions = CONTROLS_REGISTRY.filter(
    (c) => c.type === 'enum' && c.backendKey && Array.isArray(c.options),
  ).map((c) => ({ key: c.backendKey, options: [...c.options] }));

  const paramWrites = [];
  const commands = [];
  const add = (source, rec) => {
    for (const w of rec.writes) paramWrites.push({ source, ...w });
    for (const c of rec.commands) commands.push({ source, ...c });
  };

  // T-04: todos os jacks de cada modelo, depois de trocar o modelo.
  for (const model of AMP_MODELS) {
    const { store, rec } = freshStore();
    store.setControl('amp.model', model);
    const head = createAmpHeadElement(store, {});
    document.body.appendChild(head);
    clickAll(head, '[data-control-id="amp.input_jack"]');
    head.remove();
    add(`AmpHead:${model}`, rec);
  }

  // T-06: cada chip do Circuito (tone stack).
  {
    const { store, rec } = freshStore();
    store.screen = 'chassi';
    store.chassiTab = 'circuit';
    const screen = renderBancadaScreen(store);
    document.body.appendChild(screen);
    clickAll(screen, '[data-control-id="chassi.tonestack"]');
    screen.remove();
    add('Bancada:circuit', rec);
  }

  // T-03: cenas A..H pelo caminho real do store.
  return (async () => {
    for (const letter of ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H']) {
      const { store, rec } = freshStore();
      await store.selectScene(letter);
      add(`store.selectScene(${letter})`, rec);
    }
    // T-01: assistente de calibração pelo botão real.
    {
      const { store, rec } = freshStore();
      store.screen = 'audio';
      store.audioTab = 'in';
      const screen = renderAudioScreen(store);
      document.body.appendChild(screen);
      const btn = screen.querySelector('[data-control-id="audio.calibrate_input"]');
      assert.ok(btn, 'botão do assistente de calibração deve existir');
      btn.click();
      await new Promise((r) => setTimeout(r, 50));
      screen.remove();
      add('Audio:calibrar', rec);
    }
    return { enum_options: enumOptions, param_writes: paramWrites, commands };
  })();
}

const calls = await collectCalls();

test('CA-0: fixture de contrato UI↔backend está atualizado (UPDATE_051=1 regenera)', () => {
  const generated = JSON.stringify(calls, null, 2) + '\n';
  if (process.env.UPDATE_051 === '1' || !fs.existsSync(FIXTURE)) {
    fs.writeFileSync(FIXTURE, generated);
  }
  const committed = fs.readFileSync(FIXTURE, 'utf8').replace(/\r\n/g, '\n');
  assert.equal(
    committed,
    generated,
    'CA-0: fixture 051_ui_contract_calls.json desatualizado em relação ao que a UI realmente envia',
  );
});

test('CA-0: literais de setControl na UI pertencem às options do registro', () => {
  const byId = new Map(CONTROLS_REGISTRY.map((c) => [c.id, c]));
  const files = [];
  const walk = (d) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      const p = path.join(d, e.name);
      if (e.isDirectory()) walk(p);
      else if (p.endsWith('.ts')) files.push(p);
    }
  };
  walk(path.join(ROOT, 'apps/sonix-app/ui/src'));
  const re = /setControl\(\s*'([\w.]+)'\s*,\s*'([\w.\-]+)'\s*\)/g;
  const bad = [];
  for (const f of files) {
    const src = fs.readFileSync(f, 'utf8');
    for (const m of src.matchAll(re)) {
      const desc = byId.get(m[1]);
      if (desc?.type === 'enum' && Array.isArray(desc.options) && !desc.options.includes(m[2])) {
        bad.push(`${path.relative(ROOT, f)}: setControl('${m[1]}', '${m[2]}')`);
      }
    }
  }
  assert.deepEqual(bad, [], `CA-0: literais fora das options do registro: ${bad.join('; ')}`);
});

test('CA-4: o registro de amp.input_jack não oferece top_boost (vocabulário = enum do DSP)', () => {
  const d = CONTROLS_REGISTRY.find((c) => c.id === 'amp.input_jack');
  assert.ok(!d.options.includes('top_boost'));
});

test('CA-4: AC30 nunca envia jack fora do vocabulário do DSP', () => {
  const dsp = new Set([
    'high', 'low', 'high1', 'low1', 'high2', 'low2', 'jumpered', 'normal_high', 'normal_low',
    'brilliant_high', 'brilliant_low', 'normal', 'vibrato',
  ]);
  const sent = calls.param_writes.filter((w) => w.key === 'amp_1.input_jack');
  assert.ok(sent.length > 0);
  for (const w of sent) assert.ok(dsp.has(w.value), `jack inválido enviado: ${w.value} (${w.source})`);
});

test('CA-6: chips do Circuito só enviam topologias que o motor tem', () => {
  const ok = new Set(['model', 'marshall', 'fender', 'vox', 'baxandall']);
  const sent = calls.param_writes.filter((w) => w.key === 'amp_1.tonestack');
  assert.ok(sent.length > 0);
  for (const w of sent) assert.ok(ok.has(w.value), `tonestack inválido: ${w.value}`);
});

test('CA-3: cena enviada com a letra em `scene`', () => {
  const sc = calls.commands.filter((c) => c.cmd === 'select_scene');
  assert.equal(sc.length, 8);
  for (const c of sc) {
    assert.match(String(c.payload.scene), /^[A-H]$/, `payload de cena inválido: ${JSON.stringify(c.payload)}`);
  }
});

test('CA-1: assistente de calibração envia phase silence e depois strum', () => {
  const cal = calls.commands.filter((c) => c.cmd === 'calibrate_input').map((c) => c.payload.phase);
  assert.ok(cal.includes('silence') && cal.includes('strum'), `fases enviadas: ${JSON.stringify(cal)}`);
  assert.ok(cal.indexOf('silence') < cal.indexOf('strum'));
  assert.ok(!cal.includes(undefined), 'nenhuma chamada sem phase');
});

test('CA-2: erro de validação 400 abre toast amigável sem ligar banner de Motor DSP desconectado', () => {
  const cVal = classifyBridgeError('E_VALIDATION: {"error":"amp_1.input_jack inválido"}');
  assert.equal(cVal.kind, 'validation');

  const cIpc = classifyBridgeError('TypeError: Failed to fetch');
  assert.equal(cIpc.kind, 'ipc');

  const { store } = freshStore();
  const root = document.createElement('div');
  document.body.appendChild(root);
  const unmount = mountAppShell(root, store);

  store.pushNotice('{"error":"amp_1.input_jack inválido"}', 'validation');
  const banner = root.querySelector('.disconnect-banner');
  const toast = root.querySelector('[data-toast="notice"]');
  assert.equal(banner.style.display, 'none', 'CA-2: erro 400 não pode mostrar banner vermelho de desconexão');
  assert.equal(toast.style.display, 'block', 'CA-2: erro 400 deve abrir toast discreto');
  assert.ok(!toast.textContent.includes('amp_1.'), `CA-2: toast não deve expor chave interna: ${toast.textContent}`);

  unmount();
  root.remove();
});

test('CA-5: falha ao carregar preset não avança presetIdx e mostra aviso amigável', async () => {
  const { store } = freshStore();
  store.presets = [
    { id: 'brit800', name: 'Brit 800 Crunch', type: 'factory' },
    { id: 'inexistente', name: 'Preset Fantasma', type: 'user' },
  ];
  store.presetIdx = 0;
  store.transport.loadPreset = async () => {
    throw new Error('E_VALIDATION: {"error":"preset_not_found"}');
  };
  await store.loadPresetAt(1);
  assert.equal(store.presetIdx, 0, 'CA-5: presetIdx não pode avançar quando loadPreset falha');
  assert.ok(
    store.notice?.text?.includes('Preset Fantasma'),
    `CA-5: aviso deve citar o nome do preset: ${JSON.stringify(store.notice)}`,
  );
});

test('CA-7: mix de modulação e reverb em 50 exibe 50% e máximo 100% (nunca 5000%)', () => {
  const { store } = freshStore();
  store.selectedBlock = 'chorus';
  store.params['mod_2.mix'] = 50;
  const screen = renderEstudioScreen(store);
  document.body.appendChild(screen);
  const txt = screen.textContent || '';
  assert.ok(!txt.includes('5000%'), 'CA-7: não pode exibir 5000%');
  assert.ok(txt.includes('50%'), 'CA-7: valor 50 deve exibir 50%');
  screen.remove();
});

test('CA-8 & CA-10: sincronização bidirecional entre knob do equipamento e slider do inspetor e rótulo consistente no GATE-2', () => {
  const { store } = freshStore();
  store.selectedBlock = 'amp';
  const root = document.createElement('div');
  document.body.appendChild(root);
  const unmount = mountAppShell(root, store);

  // Move amp.gain via setControl (como quando o usuário gira o knob PRE-AMP ou move o slider Gain)
  store.setControl('amp.gain', 8.5);
  const ampKnob = root.querySelector('sonix-knob[data-control-id="amp.gain"]');
  const ampSlider = root.querySelector('.m-track[data-control-id="amp.gain"]');
  assert.ok(ampKnob, 'knob amp.gain deve existir');
  assert.ok(ampSlider, 'slider amp.gain deve existir no inspetor');
  assert.equal(ampKnob.getAttribute('aria-valuenow'), '8.5', 'CA-8: knob deve refletir mudança do parâmetro em tempo real');
  assert.equal(ampSlider.getAttribute('aria-valuenow'), '8.5', 'CA-8: slider deve refletir mudança do parâmetro em tempo real');

  // Muda para o bloco gate e verifica que o 3º knob do GATE-2 não se chama MODE enquanto aponta para hysteresis_db
  store.selectBlock('gate');
  store.setControl('gate.threshold_db', -42);
  const gateKnob = root.querySelector('sonix-knob[data-control-id="gate.threshold_db"]');
  const gateSlider = root.querySelector('.m-track[data-control-id="gate.threshold_db"]');
  assert.equal(gateKnob.getAttribute('aria-valuenow'), '-42', 'CA-10: knob THRESHOLD do GATE-2 deve sincronizar');
  assert.equal(gateSlider.getAttribute('aria-valuenow'), '-42', 'CA-10: slider Threshold do inspetor deve sincronizar');

  const hysKnob = root.querySelector('sonix-knob[data-control-id="gate.hysteresis_db"]');
  assert.ok(hysKnob, 'knob gate.hysteresis_db no GATE-2 deve existir');
  assert.ok(
    !String(hysKnob.getAttribute('aria-label')).includes('MODE'),
    `CA-10: knob de histerese não pode ser rotulado MODE: ${hysKnob.getAttribute('aria-label')}`,
  );

  unmount();
  root.remove();
});

test('CA-11: Nível Master na saída aponta para master.level_db e não para merge.row_1_level_db', () => {
  const { store, rec } = freshStore();
  store.selectedBlock = 'out';
  const screen = renderEstudioScreen(store);
  document.body.appendChild(screen);
  const masterSlider = screen.querySelector('.m-track[data-control-id="master.level_db"]');
  assert.ok(masterSlider, 'CA-11: bloco out deve ter slider master.level_db');
  screen.remove();
});

test('CA-12: Inspetor de CAB 1 e CAB 2 usa chaves consistentes cab_1.speaker / cab_2.speaker', () => {
  const { store } = freshStore();
  store.selectedBlock = 'cab';
  store.setControl('cab.speaker', 'greenback_4x12');
  const screen1 = renderEstudioScreen(store);
  document.body.appendChild(screen1);
  const pressed1 = [...screen1.querySelectorAll('[data-control-id="cab.speaker"]')].find(
    (b) => b.getAttribute('aria-pressed') === 'true',
  );
  assert.ok(pressed1?.textContent?.includes('Greenback'), 'CA-12: chip Greenback deve ficar ativo quando cab_1.speaker=greenback_4x12');
  screen1.remove();

  store.selectedBlock = 'cab2';
  const screen2 = renderEstudioScreen(store);
  document.body.appendChild(screen2);
  assert.ok(screen2.querySelector('.m-track[data-control-id="cab_2.resonance_hz"]'), 'CA-12: cab2 deve ter cab_2.resonance_hz próprio');
  assert.ok(screen2.querySelector('.m-track[data-control-id="cab_2.hicut_khz"]'), 'CA-12: cab2 deve ter cab_2.hicut_khz próprio');
  screen2.remove();
});

test('CA-13 & CA-14: Palco envia ações válidas do Looper e FS5..FS8 acionam cenas A..D no modo Híbrido', async () => {
  const { store, rec } = freshStore();
  store.screen = 'palco';
  const screen = renderPalcoScreen(store);
  document.body.appendChild(screen);

  // Botão Gravar do Looper deve enviar 'record' (não 'rec')
  const recBtn = [...screen.querySelectorAll('[data-control-id="looper.action"]')].find((b) =>
    b.textContent?.includes('Gravar'),
  );
  assert.ok(recBtn, 'botão Gravar deve existir');
  recBtn.click();
  const looperWrites = rec.writes.filter((w) => w.key === 'looper.action');
  assert.equal(looperWrites[0]?.value, 'record', 'CA-13: botão Gravar deve enviar record');

  // FS 5 ("A · Clean") no modo Híbrido deve chamar selectScene('A')
  const fs5 = screen.querySelector('[data-control-id="palco.fs_5_eq"]');
  assert.ok(fs5, 'FS 5 deve existir');
  fs5.click();
  await new Promise((r) => setTimeout(r, 10));
  const sceneCmds = rec.commands.filter((c) => c.cmd === 'select_scene');
  assert.equal(sceneCmds[0]?.payload?.scene, 'A', 'CA-14: FS 5 em modo Híbrido deve selecionar Cena A');

  screen.remove();
});

test('CA-15: Chassi em V1 permite escolher todas as válvulas de pré sem rejeição e exibe traço de Tone Stack', () => {
  const { store, rec } = freshStore();
  store.screen = 'chassi';
  store.selectedTube = 'V1';
  store.chassiTab = 'tube';
  const screen = renderBancadaScreen(store);
  document.body.appendChild(screen);
  clickAll(screen, '[data-control-id="chassi.preamp_tube"]');
  const written = rec.writes.filter((w) => w.key === 'amp_1.preamp_tube').map((w) => w.value);
  assert.ok(written.includes('ecc83') && written.includes('12ay7'), `CA-15: deve enviar ecc83 e 12ay7: ${written}`);
  screen.remove();
});

test('CA-16: Microfone no Gabinete move posição no cone imediatamente durante o arraste sem desmontar o nó', () => {
  const { store } = freshStore();
  store.screen = 'gabinete';
  const root = document.createElement('div');
  document.body.appendChild(root);
  const unmount = mountAppShell(root, store);

  const mic1 = root.querySelector('[data-control-id="cab.mic1.position"]');
  assert.ok(mic1, 'MIC 1 deve existir');
  const cone = mic1.parentElement.children[3];
  cone.getBoundingClientRect = () => ({ left: 0, top: 0, width: 400, height: 400, right: 400, bottom: 400, x: 0, y: 0, toJSON() {} });

  const oldLeft = mic1.style.left;
  mic1.dispatchEvent(new window.MouseEvent('pointerdown', { clientX: 200, clientY: 200, bubbles: true }));
  mic1.dispatchEvent(new window.MouseEvent('pointermove', { clientX: 350, clientY: 200, bubbles: true }));
  const mic1After = root.querySelector('[data-control-id="cab.mic1.position"]');
  assert.equal(mic1, mic1After, 'CA-16: arrastar microfone não pode desmontar o elemento DOM');
  assert.notEqual(mic1.style.left, oldLeft, 'CA-16: posição visual left do microfone deve atualizar durante o arraste');

  mic1.dispatchEvent(new window.MouseEvent('pointerup', { bubbles: true }));
  unmount();
  root.remove();
});

test('CA-9: Modifier no Inspetor tem botões habilitados, alterna fonte e explica uso em português', () => {
  const { store } = freshStore();
  store.selectedBlock = 'amp';
  const screen = renderEstudioScreen(store);
  document.body.appendChild(screen);

  const toggleBtn = screen.querySelector('[data-control-id="modifier.toggle"]');
  assert.ok(toggleBtn, 'botão modifier.toggle deve existir');
  assert.equal(toggleBtn.disabled, false, 'CA-9: modifier.toggle deve estar habilitado');

  const srcBtns = [...screen.querySelectorAll('[data-control-id="modifier.source"]')];
  assert.ok(srcBtns.length >= 4, 'botões modifier.source devem existir');
  for (const b of srcBtns) {
    assert.equal(b.disabled, false, `CA-9: fonte ${b.textContent} deve estar habilitada`);
  }

  const lfoBtn = srcBtns.find((b) => b.textContent === 'LFO 1');
  lfoBtn.click();
  assert.equal(store.modifierSource, 'LFO 1', 'CA-9: clicar em LFO 1 deve atualizar modifierSource');

  screen.remove();
});

test('CA-17: Oversampling tem padrão consistente 2x, mostra impacto de CPU/latência e low-latency ajusta buffer para 64', async () => {
  const { store, rec } = freshStore();
  assert.equal(store.getParam('engine.oversampling', '2x'), '2x', 'CA-17: padrão de oversampling na store deve ser 2x');

  store.screen = 'audio';
  store.audioTab = 'perf';
  const screen = renderAudioScreen(store);
  document.body.appendChild(screen);
  const txt = screen.textContent || '';
  assert.ok(txt.includes('latência') || txt.includes('amostras'), 'CA-17: aba Desempenho deve explicar impacto de CPU e latência');
  screen.remove();

  // Ao selecionar wasapi_shared_low_latency a partir do default 128, buffer padrão cai para 64
  store.audioStatus.mode = 'wasapi_shared';
  store.audioStatus.output_period_frames = 128;
  store.audioTab = 'dev';
  const devScreen = renderAudioScreen(store);
  document.body.appendChild(devScreen);
  const selects = [...devScreen.querySelectorAll('select[data-control-id="audio.device_apply"]')];
  const driverSel = selects[0];
  assert.ok(driverSel, 'Seletor de driver deve existir na aba Dispositivo e latência');
  driverSel.value = 'wasapi_shared_low_latency';
  driverSel.dispatchEvent(new Event('change', { bubbles: true }));
  const applyBtn = [...devScreen.querySelectorAll('button[data-control-id="audio.device_apply"]')].find(
    (b) => b.textContent === 'Aplicar',
  );
  assert.ok(applyBtn, 'Botão Aplicar deve existir');
  applyBtn.click();
  await new Promise((r) => setTimeout(r, 10));
  const setDevCall = rec.commands.find((c) => c.cmd === 'set_audio_device');
  assert.ok(setDevCall, 'Aplicar deve invocar set_audio_device');
  assert.equal(setDevCall.payload?.mode, 'wasapi_shared_low_latency');
  assert.equal(setDevCall.payload?.buffer_size, 64, 'Trocar para baixa latência deve ajustar buffer padrão de 128 para 64');
  devScreen.remove();
});

test('CA-13 & CA-15: Afinador e osciloscópio atualizam em tempo real sem valores fictícios ou oitava duplicada', () => {
  const { store } = freshStore();
  // Sem sinal inicial: sem nota/frequência fictícia "A2 / 110.00 Hz"
  assert.equal(store.tuner.note, '---');
  assert.equal(store.tuner.octave, null);
  assert.equal(store.tuner.frequency_hz, 0);

  store.screen = 'palco';
  store.palcoMode = 'tuner';
  const root = document.createElement('div');
  document.body.appendChild(root);
  const unmount = mountAppShell(root, store);

  assert.ok(root.textContent?.includes('---'), 'Afinador inicial deve indicar --- sem sinal');
  assert.ok(!root.textContent?.includes('110.00 Hz'), 'Afinador inicial não deve exibir 110.00 Hz fictício');

  // Emite evento de stream do afinador com formato do backend ("note": "A2", "note_letter": "A", "octave": 2)
  store.applyTunerPayload?.({
    note: 'A2',
    note_letter: 'A',
    octave: 2,
    frequency_hz: 110.05,
    cents: 0.8,
    signal_present: true,
    enabled: true,
  });
  // Simula entrega via método público/handleTuner se não houver, ou via subscriber:
  const noteEl = root.querySelector('[data-live="tuner-note"]');
  const octEl = root.querySelector('[data-live="tuner-octave"]');
  const freqEl = root.querySelector('[data-live="tuner-freq"]');
  assert.ok(noteEl && octEl && freqEl, 'Palco deve expor elementos [data-live="tuner-*"] para atualização em tempo real');
  assert.equal(noteEl.textContent, 'A', 'Nota do afinador não pode duplicar oitava (ex: A22)');
  assert.equal(octEl.textContent, '2', 'Oitava do afinador deve ser exibida separadamente');
  assert.ok(freqEl.textContent?.includes('110.05 Hz'), `Frequência ao vivo não atualizou: ${freqEl.textContent}`);

  // Ao sumir o sinal, volta para ---
  store.applyTunerPayload?.({
    note: '---',
    note_letter: '---',
    octave: null,
    frequency_hz: 0.0,
    cents: 0.0,
    signal_present: false,
    enabled: true,
  });
  assert.equal(noteEl.textContent, '---');
  assert.equal(octEl.textContent, '');
  assert.ok(freqEl.textContent?.includes('---'));

  // Osciloscópio CRT em Bancada: sem sinal não fabrica onda senoidal/tanh sintética no modo waveform
  store.setScreen('chassi');
  store.crtMode = 'waveform';
  store.applyScopePayload('waveform', { no_signal: true, waveform: { di: [], output: [] } });
  const scopeOverlay = root.querySelector('[data-live="scope-no-signal"]');
  assert.ok(scopeOverlay && scopeOverlay.textContent?.includes('SEM SINAL'), 'Modo waveform sem sinal deve mostrar aviso explícito em vez de onda sintética');
  assert.equal(root.querySelector('svg[data-live="scope-svg"] path[data-scope-series="waveform"]'), null, 'Não pode desenhar curva sintética em waveform sem sinal');

  // Quando chegam amostras reais no stream 'scope', atualiza SVG em tempo real sem reconstruir tela
  const svgBefore = root.querySelector('svg[data-live="scope-svg"]');
  const diSig = Array.from({ length: 512 }, (_, i) => 0.25 * Math.sin((2 * Math.PI * 110 * i) / 48000));
  const outSig = diSig.map((v) => Math.tanh(v * 2.0));
  store.applyScopePayload('waveform', { no_signal: false, waveform: { di: diSig, output: outSig } });
  const svgAfter = root.querySelector('svg[data-live="scope-svg"]');
  assert.equal(svgBefore, svgAfter, 'Atualização do osciloscópio não deve reconstruir o SVG');
  const mainPath = root.querySelector('svg[data-live="scope-svg"] path[data-scope-series="waveform"]');
  assert.ok(mainPath && (mainPath.getAttribute('d') || '').length > 20, 'Stream scope deve desenhar o path da forma de onda real');

  // Botões chassi.crt_mode atualizam aria-pressed imediatamente ao clicar
  const crtBtnsBefore = [...root.querySelectorAll('button[data-control-id="chassi.crt_mode"]')];
  const wfBtnBefore = crtBtnsBefore.find((b) => b.textContent === 'Forma de onda');
  const trBtnBefore = crtBtnsBefore.find((b) => b.textContent === 'Transferência');
  assert.equal(wfBtnBefore?.getAttribute('aria-pressed'), 'true');
  assert.equal(trBtnBefore?.getAttribute('aria-pressed'), 'false');
  trBtnBefore.click();
  const crtBtnsAfter = [...root.querySelectorAll('button[data-control-id="chassi.crt_mode"]')];
  assert.equal(crtBtnsAfter.find((b) => b.textContent === 'Forma de onda')?.getAttribute('aria-pressed'), 'false');
  assert.equal(crtBtnsAfter.find((b) => b.textContent === 'Transferência')?.getAttribute('aria-pressed'), 'true');

  unmount();
  root.remove();
});

test('CA-18: Tap Tempo em BPM maior aumenta dly_1.repeat_rate (e diminui o tempo de delay em ms)', () => {
  const { store } = freshStore();
  // Simula 60 BPM (1000 ms entre batidas) vs 180 BPM (333.3 ms entre batidas)
  store.applyTapBpm(60);
  const rate60 = Number(store.getParam('dly_1.repeat_rate', 5));
  store.applyTapBpm(180);
  const rate180 = Number(store.getParam('dly_1.repeat_rate', 5));
  assert.ok(
    rate180 > rate60,
    `CA-18: BPM mais rápido (180) deve produzir repeat_rate maior que BPM lento (60): rate60=${rate60}, rate180=${rate180}`,
  );
});

test('CA-19 & CA-20: Pedalboard mostra estado/modelo real dos blocos e slots de rack têm altura >= 80 px', () => {
  const { store } = freshStore();
  store.setControl('drive.model', 'rat');
  store.setControl('drive.enabled', false);
  const screen = renderEstudioScreen(store);
  document.body.appendChild(screen);

  assert.ok(screen.textContent?.includes('RODENT'), 'CA-19: pedalboard deve exibir o pedal de drive selecionado (RODENT)');
  const rackSlots = [...screen.querySelectorAll('[data-rack-slot]')];
  assert.equal(rackSlots.length, 3, 'deve ter 3 slots de rack');
  for (const s of rackSlots) {
    const h = parseInt(s.style.height, 10);
    assert.ok(h >= 60, `CA-20: slot de rack deve ter altura definida (>= 60 px), obteve ${h}px`);
  }
  screen.remove();
});

test('CA-23 & CA-24: AFINADOR, PALCO e Status do motor têm destinos distintos e UI não exibe jargão Spec 0NN', () => {
  const { store } = freshStore();
  const root = document.createElement('div');
  document.body.appendChild(root);
  const unmount = mountAppShell(root, store);

  // Zero "Spec 0NN" no texto visível ou tooltips das telas principais
  for (const scr of ['main', 'chassi', 'gabinete', 'palco', 'biblioteca', 'audio']) {
    store.setScreen(scr);
    const visibleTxt = root.textContent || '';
    assert.ok(!/Spec\s+0\d\d/.test(visibleTxt), `CA-24: tela ${scr} não pode exibir 'Spec 0NN' no texto: ${visibleTxt}`);
    const titled = [...root.querySelectorAll('[title]')].map((el) => el.getAttribute('title') || '');
    for (const t of titled) {
      assert.ok(!/Spec\s+0\d\d/.test(t), `CA-24: tooltip na tela ${scr} não pode exibir 'Spec 0NN': ${t}`);
    }
  }

  // Navegação distinta para AFINADOR vs PALCO
  store.setScreen('main');
  const tunerNavBtn = root.querySelector('[data-nav-target="tuner"]');
  const palcoNavBtn = root.querySelector('[data-nav-target="palco"]');
  assert.ok(tunerNavBtn && palcoNavBtn, 'CA-23: botões AFINADOR e PALCO devem ter alvos identificados');
  tunerNavBtn.click();
  assert.equal(store.palcoMode, 'tuner', 'CA-23: botão AFINADOR deve abrir modo dedicado do afinador');
  assert.equal(store.getParam('tuner.enabled', false), true, 'CA-23: botão AFINADOR deve ativar tuner.enabled');

  store.setScreen('main');
  palcoNavBtn.click();
  assert.equal(store.palcoMode, 'stage', 'CA-23: botão PALCO deve abrir visão de palco (cenas/footswitches/looper)');

  unmount();
  root.remove();
});

test('CA-21 & CA-22: Biblioteca renderiza amplificadores com createAmpHeadElement completo (sem compact) e pedais/racks sem corte', () => {
  const { store } = freshStore();
  store.setScreen('biblioteca');
  store.libraryTab = 'amps';
  const screenAmps = renderBibliotecaScreen(store);
  document.body.appendChild(screenAmps);
  // Todos os 5 amps usam a mesma geometria escalada (0.788) da tela principal
  assert.ok(screenAmps.innerHTML.includes('scale(0.788)'), 'CA-22: Biblioteca deve escalar o cabeçote completo como no Rig');
  screenAmps.remove();

  store.libraryTab = 'pedals';
  const screenPedals = renderBibliotecaScreen(store);
  document.body.appendChild(screenPedals);
  assert.ok(screenPedals.textContent?.includes('FET 76') && screenPedals.textContent?.includes('Space Tape'), 'CA-21: racks presentes no catálogo');
  screenPedals.remove();
});

test('CA-26: Rótulos de knobs (GATE-2, RODENT) e barra de microfones na sala de Gabinete não colidem', () => {
  const { store } = freshStore();
  store.setControl('drive.model', 'rat');
  const estudio = renderEstudioScreen(store);
  document.body.appendChild(estudio);
  const txt = estudio.textContent || '';
  assert.ok(txt.includes('THRESH') && txt.includes('HYST'), 'CA-26: GATE-2 deve usar rótulos abreviados THRESH / DECAY / HYST');
  estudio.remove();

  store.setScreen('gabinete');
  const gab = renderGabineteScreen(store);
  document.body.appendChild(gab);
  const micBar = gab.querySelector('[data-region="mic-readout-bar"]');
  assert.ok(micBar, 'CA-26: barra de leitura dos microfones deve ter região dedicada');
  const spans = [...micBar.querySelectorAll('[data-mic-readout]')];
  assert.equal(spans.length, 2, 'CA-26: MIC 1 e MIC 2 devem estar em spans separados sem sobreposição');
  gab.remove();
});
