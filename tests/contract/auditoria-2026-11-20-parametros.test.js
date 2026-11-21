// Auditoria 2026-11-20 — AUD-08: controles de amp declaram models[] e tela Main filtra pelo modelo ativo.
import test from 'node:test';
import assert from 'node:assert/strict';
import { JSDOM } from 'jsdom';

function setupDom() {
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
  return dom;
}

setupDom();

const { AppStore } = await import('../../apps/sonix-app/ui/src/store.ts');
const { CONTROLS_REGISTRY } = await import('../../apps/sonix-app/ui/src/controls.registry.ts');
const { renderEstudioScreen } = await import('../../apps/sonix-app/ui/src/screens/Estudio.ts');

const VALID_MODELS = ['plexi_1959', 'jcm800_2203', 'twin_ab763', 'dual_rect', 'ac30_tb'];

function makeStubTransport() {
  return {
    setParam: () => {},
    setParamsPatch: () => {},
    getState: async () => ({}),
    listPresets: async () => ({ presets: [{ id: 'brit800', name: 'Brit 800 Crunch' }] }),
  };
}

test('AUD-08: todo controle de amp declara models; a tela Main só renderiza os controles do modelo ativo', () => {
  // 1. overdrive e depth foram removidos de CONTROLS_REGISTRY
  for (const forbidden of ['amp.overdrive', 'amp.depth', 'amp_1.overdrive', 'amp_1.depth']) {
    const hit = CONTROLS_REGISTRY.find((c) => c.id === forbidden || c.backendKey === forbidden);
    assert.equal(hit, undefined, `AUD-08: '${forbidden}' não pode existir em CONTROLS_REGISTRY`);
  }

  // 2. Todo controle amp.* declara models não-vazio com IDs válidos
  const ampControls = CONTROLS_REGISTRY.filter((c) => c.id.startsWith('amp.'));
  assert.ok(ampControls.length > 0);
  for (const desc of ampControls) {
    assert.ok(
      Array.isArray(desc.models) && desc.models.length > 0,
      `AUD-08: controle '${desc.id}' deve declarar models[] não-vazio`,
    );
    for (const m of desc.models) {
      assert.ok(
        VALID_MODELS.includes(m),
        `AUD-08: controle '${desc.id}' cita modelo inválido '${m}'`,
      );
    }
  }

  // Regras específicas do circuito real por modelo
  const byId = Object.fromEntries(ampControls.map((c) => [c.id, c]));
  assert.deepEqual([...byId['amp.presence'].models].sort(), ['dual_rect', 'jcm800_2203', 'plexi_1959']);
  assert.deepEqual([...byId['amp.master'].models].sort(), ['dual_rect', 'jcm800_2203']);
  assert.equal(byId['amp.middle'].models.includes('ac30_tb'), false, 'AC30 não tem controle Middle');

  // 3. Para cada modelo, a tela Main só renderiza controles cujo models inclui o modelo ativo
  for (const model of VALID_MODELS) {
    setupDom();
    const store = new AppStore(makeStubTransport());
    store.selectBlock('amp');
    store.setControl('amp.model', model);

    const screen = renderEstudioScreen(store);
    document.body.appendChild(screen);

    const renderedIds = Array.from(screen.querySelectorAll('[data-control-id]'))
      .map((el) => el.getAttribute('data-control-id'))
      .filter((id) => id && id.startsWith('amp.'));

    assert.ok(renderedIds.length > 0, `AUD-08: tela Main deve renderizar controles amp.* para '${model}'`);

    for (const cid of renderedIds) {
      const desc = byId[cid];
      assert.ok(desc, `AUD-08: controle renderizado '${cid}' não registrado`);
      assert.ok(
        desc.models.includes(model),
        `AUD-08: modelo '${model}' renderizou '${cid}' na tela Main, mas models=${JSON.stringify(desc.models)}`,
      );
    }

    if (model === 'ac30_tb') {
      assert.equal(renderedIds.includes('amp.master'), false, 'AC30 não pode renderizar MASTER na Main');
      assert.equal(renderedIds.includes('amp.presence'), false, 'AC30 não pode renderizar PRESENCE na Main');
      assert.equal(renderedIds.includes('amp.middle'), false, 'AC30 não pode renderizar MIDDLE na Main');
    }
    if (model === 'twin_ab763') {
      assert.equal(renderedIds.includes('amp.master'), false, 'Twin não pode renderizar MASTER na Main');
      assert.equal(renderedIds.includes('amp.presence'), false, 'Twin não pode renderizar PRESENCE na Main');
    }
    if (model === 'plexi_1959') {
      assert.equal(renderedIds.includes('amp.master'), false, 'Plexi não pode renderizar MASTER na Main');
    }

    screen.remove();
  }
});
