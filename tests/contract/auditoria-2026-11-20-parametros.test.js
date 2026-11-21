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

test('AUD-09: toda backendKey do registro existe em params.generated.ts', async () => {
  const fs = await import('node:fs');
  const path = await import('node:path');
  const root = path.resolve(import.meta.dirname, '..', '..');
  const genPath = path.join(root, 'apps', 'sonix-app', 'ui', 'src', 'params.generated.ts');
  assert.ok(
    fs.existsSync(genPath),
    `AUD-09: ${genPath} não existe ainda`,
  );

  const regSrc = fs.readFileSync(
    path.join(root, 'apps', 'sonix-app', 'ui', 'src', 'controls.registry.ts'),
    'utf8',
  );
  assert.ok(
    regSrc.includes('./params.generated.ts'),
    'AUD-09: controls.registry.ts deve importar de ./params.generated.ts',
  );

  const { PARAMS_SCHEMA } = await import('../../apps/sonix-app/ui/src/params.generated.ts');
  assert.ok(PARAMS_SCHEMA && typeof PARAMS_SCHEMA === 'object');

  const withBackend = CONTROLS_REGISTRY.filter((c) => c.backendKey !== null);
  assert.ok(withBackend.length > 0);
  for (const desc of withBackend) {
    const schema = PARAMS_SCHEMA[desc.backendKey];
    assert.ok(
      schema,
      `AUD-09: controle '${desc.id}' referencia backendKey '${desc.backendKey}' ausente de params.generated.ts`,
    );
    assert.equal(
      desc.type,
      schema.type,
      `AUD-09: '${desc.id}' type=${desc.type} diverge de params.generated.ts (${schema.type})`,
    );
    assert.deepEqual(
      desc.defaultValue,
      schema.defaultValue,
      `AUD-09: '${desc.id}' defaultValue diverge de params.generated.ts`,
    );
    if (schema.type === 'number') {
      assert.deepEqual(desc.range, schema.range, `AUD-09: '${desc.id}' range diverge`);
      assert.equal(desc.step, schema.step, `AUD-09: '${desc.id}' step diverge`);
    }
    if (schema.type === 'enum') {
      assert.deepEqual(desc.options, schema.options, `AUD-09: '${desc.id}' options diverge`);
    }
  }
});

test('AUD-19: botões Exportar pacote e Importar pacote na tela de presets chamam comandos IPC', async () => {
  setupDom();
  const { SonixTransport } = await import('../../apps/sonix-app/ui/src/transport.ts');
  const { renderBibliotecaScreen } = await import('../../apps/sonix-app/ui/src/screens/Biblioteca.ts');

  const calls = [];
  const transport = new SonixTransport({
    invokeFn: async (cmd, args) => {
      calls.push({ cmd, args });
      if (cmd === 'plugin:dialog|save') return 'C:\\Temp\\rig_aud19.sforgepkg';
      if (cmd === 'plugin:dialog|open') return 'C:\\Temp\\rig_aud19.sforgepkg';
      if (cmd === 'export_package') return { ok: true, path: args?.path, presets_exported: 1, irs_exported: 0 };
      if (cmd === 'import_package') return { ok: true, path: args?.path, imported_presets: ['brit800'], imported_irs: [] };
      if (cmd === 'list_presets') return { presets: [{ id: 'brit800', name: 'Brit 800 Crunch', type: 'factory' }] };
      return { ok: true };
    },
  });

  const store = new AppStore(transport);
  store.presets = [{ id: 'brit800', name: 'Brit 800 Crunch', type: 'factory' }];
  store.presetIdx = 0;
  store.libraryTab = 'presets';

  const screen = renderBibliotecaScreen(store);
  document.body.appendChild(screen);

  const exportBtn = screen.querySelector('[data-control-id="preset.export_package"]');
  const importBtn = screen.querySelector('[data-control-id="preset.import_package"]');
  assert.ok(exportBtn, 'AUD-19: tela de presets deve renderizar botão Exportar pacote');
  assert.ok(importBtn, 'AUD-19: tela de presets deve renderizar botão Importar pacote');
  assert.ok(CONTROLS_REGISTRY.find((c) => c.id === 'preset.export_package'), 'AUD-19: preset.export_package deve estar no registro');
  assert.ok(CONTROLS_REGISTRY.find((c) => c.id === 'preset.import_package'), 'AUD-19: preset.import_package deve estar no registro');

  exportBtn.click();
  await new Promise((r) => setTimeout(r, 20));
  importBtn.click();
  await new Promise((r) => setTimeout(r, 20));

  const cmds = calls.map((c) => c.cmd);
  assert.ok(cmds.includes('export_package'), `AUD-19: Exportar pacote deve invocar export_package, recebeu ${JSON.stringify(calls)}`);
  assert.ok(cmds.includes('import_package'), `AUD-19: Importar pacote deve invocar import_package, recebeu ${JSON.stringify(calls)}`);
  const expCall = calls.find((c) => c.cmd === 'export_package');
  assert.deepEqual(expCall.args.presetIds, ['brit800']);
  assert.equal(expCall.args.path, 'C:\\Temp\\rig_aud19.sforgepkg');
  const impCall = calls.find((c) => c.cmd === 'import_package');
  assert.equal(impCall.args.path, 'C:\\Temp\\rig_aud19.sforgepkg');

  screen.remove();
});

test('AUD-21: grid renderiza bloco de wah explícito e esmaece Linha 2 com dica quando split desligado', async () => {
  setupDom();
  const { renderEstudioScreen } = await import('../../apps/sonix-app/ui/src/screens/Estudio.ts');
  const store = new AppStore(makeStubTransport());
  store.setControl('split.enabled', false);

  const screen = renderEstudioScreen(store);
  document.body.appendChild(screen);

  const wahCell = screen.querySelector('.m-cell[data-block-id="wah"]');
  assert.ok(wahCell, 'AUD-21: matriz de sinal deve renderizar célula [data-block-id="wah"]');

  const line2Ids = ['comp', 'amp2', 'cab2', 'chorus'];
  for (const bid of line2Ids) {
    const cell = screen.querySelector(`.m-cell[data-block-id="${bid}"]`);
    assert.ok(cell, `AUD-21: bloco paralelo '${bid}' deve existir na Linha 2`);
    assert.equal(
      cell.getAttribute('title'),
      'Linha 2 inativa (split desligado)',
      `AUD-21: '${bid}' deve trazer dica 'Linha 2 inativa (split desligado)' com split desligado`,
    );
  }

  screen.remove();
});
