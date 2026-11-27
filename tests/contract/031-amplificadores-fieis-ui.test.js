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

  it('Cali Recto: alternar RECTIFIER (TUBE/DIODE) e POWER (SPONGY/BOLD) sincroniza store, aria-pressed e B+ ao vivo', async () => {
    const { JSDOM } = await import('jsdom');
    const dom = new JSDOM('<!doctype html><html><body><div id="app"></div></body></html>', {
      url: 'http://localhost/',
    });
    const prevGlobals = {};
    for (const k of [
      'window', 'document', 'HTMLElement', 'customElements', 'Element', 'SVGElement', 'Node',
      'Event', 'MouseEvent', 'KeyboardEvent', 'WheelEvent', 'PointerEvent',
    ]) {
      prevGlobals[k] = globalThis[k];
      if (k === 'PointerEvent') globalThis[k] = dom.window.MouseEvent;
      else globalThis[k] = dom.window[k];
    }

    try {
      const { AppStore } = await import('../../apps/sonix-app/ui/src/store.ts');
      const { mountAppShell } = await import('../../apps/sonix-app/ui/src/shell.ts');

      const writes = [];
      const noop = async () => ({});
      const fakeTransport = {
        setParam: (k, v) => writes.push({ key: k, value: v }),
        setParamsPatch: (patch) => {
          for (const [k, v] of Object.entries(patch)) writes.push({ key: k, value: v });
        },
        getState: async () => ({ 'amp_1.model': 'jcm800_2203', 'amp_1.rectifier': 'diode', 'amp_1.spongy': false }),
        listPresets: async () => ({ presets: [{ id: 'recto', name: 'Recto High Gain' }] }),
        getDevices: async () => ({ devices: [] }),
        listMidiPorts: async () => ({ ports: [], connected_port: null }),
        getAudioStatus: async () => ({ backend: 'wasapi', mode: 'wasapi_shared', sample_rate: 48000, input_period_frames: 128, output_period_frames: 128, connected: true, output_device: 'Sim' }),
        startStreams: noop,
        getScope: noop,
      };
      const store = new AppStore(fakeTransport);
      const appRoot = dom.window.document.getElementById('app');
      const unmount = mountAppShell(appRoot, store);

      try {
        // Trocar para dual_rect deve sincronizar rectifier nominal (5u4gb = TUBE) no store
        store.setControl('amp.model', 'dual_rect');
        assert.equal(store.params['amp_1.model'], 'dual_rect');
        assert.equal(
          store.params['amp_1.rectifier'],
          '5u4gb',
          'Ao selecionar dual_rect, amp_1.rectifier deve sincronizar para 5u4gb (TUBE)'
        );

        let rectBtn = appRoot.querySelector('button[data-control-id="chassi.rectifier"]');
        let spongyBtn = appRoot.querySelector('button[data-control-id="amp.spongy"]');
        assert.ok(rectBtn, 'Botão RECTIFIER deve existir no AmpHead do Cali Recto');
        assert.ok(spongyBtn, 'Botão POWER SPONGY/BOLD deve existir no AmpHead do Cali Recto');
        assert.equal(rectBtn.getAttribute('aria-pressed'), 'true', 'Nominal do dual_rect deve iniciar em TUBE (true)');
        assert.equal(spongyBtn.getAttribute('aria-pressed'), 'false', 'Nominal do dual_rect deve iniciar em BOLD (false)');

        // Clicar RECTIFIER (1º clique deve ir direto para DIODE)
        rectBtn.click();
        assert.equal(store.params['amp_1.rectifier'], 'diode');
        assert.equal(writes.at(-1)?.key, 'amp_1.rectifier');
        assert.equal(writes.at(-1)?.value, 'diode');
        rectBtn = appRoot.querySelector('button[data-control-id="chassi.rectifier"]');
        assert.equal(rectBtn.getAttribute('aria-pressed'), 'false');

        // Clicar novamente no label TUBE/DIODE -> volta para 5u4gb
        const labelTop = rectBtn.parentElement?.querySelector('span.m-lbl');
        assert.ok(labelTop, 'Rótulo superior do toggle RECTIFIER deve existir');
        labelTop.click();
        assert.equal(store.params['amp_1.rectifier'], '5u4gb');
        assert.equal(writes.at(-1)?.value, '5u4gb');

        // Clicar SPONGY -> true e BOLD -> false
        spongyBtn = appRoot.querySelector('button[data-control-id="amp.spongy"]');
        spongyBtn.click();
        assert.equal(store.params['amp_1.spongy'], true);
        assert.equal(writes.at(-1)?.key, 'amp_1.spongy');
        assert.equal(writes.at(-1)?.value, true);
        spongyBtn = appRoot.querySelector('button[data-control-id="amp.spongy"]');
        assert.equal(spongyBtn.getAttribute('aria-pressed'), 'true');

        // Telemetria b_plus_v atualiza readout ao vivo
        store.applyTelemetry({ input_peak_db: -20, output_peak_db: -18, cpu_usage_pct: 5.0, b_plus_v: 342.0 });
        const bPlusEl = appRoot.querySelector('[data-live="b-plus"]');
        assert.ok(bPlusEl, 'Elemento [data-live="b-plus"] deve existir na tela Estudio');
        assert.match(bPlusEl.textContent ?? '', /342\s*V/);
      } finally {
        unmount();
      }
    } finally {
      for (const [k, v] of Object.entries(prevGlobals)) {
        if (v === undefined) delete globalThis[k];
        else globalThis[k] = v;
      }
    }
  });
});
