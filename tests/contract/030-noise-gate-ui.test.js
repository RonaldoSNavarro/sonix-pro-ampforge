import { test, suite } from 'node:test';
import assert from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import vm from 'node:vm';

suite('Spec 030: Noise Gate Completo com Histerese, Hold, Range e Sidechain', () => {
  test('Execução de testes de contrato Rust DSP (CA-1, CA-2, CA-3, CA-6, CA-7)', () => {
    const res = spawnSync(
      'cargo',
      ['test', '-p', 'ampforge-dsp', '--test', 'contract_030_noise_gate'],
      { encoding: 'utf-8', shell: true }
    );
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes Rust DSP Spec 030:\n${res.stdout}\n${res.stderr}`
    );
  });

  test('Execução de testes de contrato Rust API (CA-4)', () => {
    const res = spawnSync(
      'cargo',
      ['test', '-p', 'sonix-desktop', '--test', 'contract_030_gate_api'],
      { encoding: 'utf-8', shell: true }
    );
    assert.strictEqual(
      res.status,
      0,
      `Falha nos testes Rust API Spec 030:\n${res.stdout}\n${res.stderr}`
    );
  });

  test('CA-5 UI: Clicar em blk_gate renderiza controles interativos completos de gate.* e sincroniza threshold', async () => {
    const appJs = readFileSync(resolve('design-system/app.js'), 'utf-8');
    const elements = new Map();
    const fetchCalls = [];

    const makeEl = (id) => {
      const listeners = {};
      const classes = new Set();
      return {
        id,
        value: '-65',
        innerHTML: '',
        textContent: '',
        style: { setProperty: () => {}, getPropertyValue: () => '' },
        dataset: {},
        classList: {
          add: (...c) => c.forEach((x) => classes.add(x)),
          remove: (...c) => c.forEach((x) => classes.delete(x)),
          toggle: (c, f) => (f === undefined ? !classes.delete(c) && classes.add(c) : f ? classes.add(c) : classes.delete(c)),
          contains: (c) => classes.has(c),
        },
        addEventListener: (e, fn) => {
          listeners[e] = listeners[e] || [];
          listeners[e].push(fn);
        },
        dispatchEvent: (evt) => (listeners[evt.type] || []).forEach((fn) => fn(evt)),
        appendChild: (c) => c,
        parentNode: { appendChild: (c) => c },
        querySelector: (s) => makeEl(`${id}_${s}`),
        querySelectorAll: () => [],
      };
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
      setTimeout,
      clearTimeout,
      setInterval: () => 1,
      clearInterval: () => {},
      requestAnimationFrame: () => 1,
      document: {
        body: makeEl('body'),
        getElementById: (id) => {
          if (!elements.has(id)) elements.set(id, makeEl(id));
          return elements.get(id);
        },
        querySelector: (s) => makeEl(`q_${s}`),
        querySelectorAll: () => [],
        createElement: (tag) => makeEl(`tag_${tag}_${Math.random()}`),
      },
      window: {
        __SONIX_TOKEN__: 'tok',
        addEventListener: () => {},
      },
      fetch: async (url, opts = {}) => {
        fetchCalls.push({ url, method: opts.method || 'GET', body: opts.body });
        return { ok: true, json: async () => ({}) };
      },
    };
    sandbox.globalThis = sandbox;

    vm.runInNewContext(appJs, sandbox);
    const gateBlk = { id: 'blk_gate', row: 0, col: 0, cat: 'GATE', name: 'Natural Gate', bypassed: false };

    sandbox.openBlockEditor(gateBlk);
    const bodyHtml = elements.get('blockEditorBody').innerHTML;

    // Altera Threshold e valida POST /api/params com gate.threshold_db (após debounce de 25ms)
    const threshEl = elements.get('gateThreshSlider');
    assert.ok(threshEl, 'Slider gateThreshSlider deve estar registrado');
    threshEl.dispatchEvent({ type: 'input', target: { value: '-48' } });
    await new Promise((r) => setTimeout(r, 40));
    assert.ok(
      fetchCalls.some((c) => c.url === '/api/params' && c.body && c.body.includes('gate.threshold_db') && c.body.includes('-48')),
      'Alterar gateThreshSlider deve enviar gate.threshold_db ao backend'
    );

    assert.ok(
      !bodyHtml.includes('sem controles nesta versão'),
      'Editor do Noise Gate não pode exibir mensagem de controles ausentes'
    );
    for (const key of [
      'gate.mode',
      'gate.threshold_db',
      'gate.hysteresis_db',
      'gate.attack_ms',
      'gate.hold_ms',
      'gate.release_ms',
      'gate.range_db',
      'gate.key_hpf_hz',
      'gate.key_lpf_hz',
      'gateStatusBadge',
    ]) {
      assert.ok(
        appJs.includes(key),
        `app.js deve implementar controle/indicador para ${key} no editor do gate`
      );
    }
    assert.ok(
      bodyHtml.includes('gateHysteresisSlider') &&
        bodyHtml.includes('gateHoldSlider') &&
        bodyHtml.includes('gateRangeSlider') &&
        bodyHtml.includes('gateKeyHpfSlider') &&
        bodyHtml.includes('gateKeyLpfSlider') &&
        bodyHtml.includes('gateModeSelect'),
      'HTML gerado em #blockEditorBody para blk_gate deve conter todos os controles completos do Noise Gate'
    );
  });
});
