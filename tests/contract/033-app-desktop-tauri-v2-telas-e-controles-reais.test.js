import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { CONTROLS_REGISTRY, findControl, listPendingControls } from '../../apps/sonix-app/ui/src/controls.registry.ts';
import { ParamCoalescer } from '../../apps/sonix-app/ui/src/transport.ts';

const ROOT = process.cwd();
const APP_DIR = path.join(ROOT, 'apps', 'sonix-app');
const UI_DIR = path.join(APP_DIR, 'ui');
const TAURI_DIR = path.join(APP_DIR, 'src-tauri');

function collectTsFiles(dir) {
  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...collectTsFiles(full));
    else if (entry.name.endsWith('.ts') && !entry.name.endsWith('.d.ts')) out.push(full);
  }
  return out;
}

test('CA-4: controls.registry.ts e inventario existem sem controles pendentes e sem controles sem registro na UI', () => {
  const regPath = path.join(UI_DIR, 'src', 'controls.registry.ts');
  const invPath = path.join(ROOT, 'docs', 'ui-desktop', 'inventario-controles.md');
  assert.ok(fs.existsSync(regPath), 'controls.registry.ts deve existir');
  assert.ok(fs.existsSync(invPath), 'docs/ui-desktop/inventario-controles.md deve existir');

  const regText = fs.readFileSync(regPath, 'utf8');
  assert.ok(regText.includes('gate.threshold_db'), 'Deve registrar gate.threshold_db');
  assert.ok(regText.includes('amp_1.standby'), 'Deve registrar amp_1.standby');
  assert.ok(regText.includes('comp_2.output'), 'Deve registrar comp_2.output');
  assert.ok(regText.includes('looper.action'), 'Deve registrar looper.action');

  const pendingControls = listPendingControls();
  assert.equal(
    pendingControls.length,
    0,
    `Ainda existem ${pendingControls.length} controle(s) pendente(s) em controls.registry.ts: ${pendingControls.map((c) => c.id).join(', ')}`,
  );

  const invText = fs.readFileSync(invPath, 'utf8');
  assert.ok(!/Pendente\s+\*\*Spec/i.test(invText), 'inventario-controles.md não pode manter status Pendente após Spec 033');

  // Verifica que todo data-control-id literal nos fontes UI existe em CONTROLS_REGISTRY
  const uiFiles = collectTsFiles(path.join(UI_DIR, 'src'));
  for (const file of uiFiles) {
    const src = fs.readFileSync(file, 'utf8');
    for (const m of src.matchAll(/dataset\.controlId\s*=\s*'([^']+)'/g)) {
      assert.ok(findControl(m[1]), `Controle '${m[1]}' em ${path.relative(ROOT, file)} ausente de controls.registry.ts`);
    }
    for (const m of src.matchAll(/setAttribute\(\s*'data-control-id'\s*,\s*'([^']+)'\s*\)/g)) {
      assert.ok(findControl(m[1]), `Controle '${m[1]}' em ${path.relative(ROOT, file)} ausente de controls.registry.ts`);
    }
    for (const m of src.matchAll(/(?:makeChipButton|makeControlSlider|makeControlSelect|makeToggleRow|buildKnobControl)\(\s*(?:store\s*,\s*)?'([^']+)'/g)) {
      assert.ok(findControl(m[1]), `Controle '${m[1]}' em ${path.relative(ROOT, file)} ausente de controls.registry.ts`);
    }
  }
  assert.ok(CONTROLS_REGISTRY.length >= 100, 'Registro deve cobrir todos os parâmetros reais da UI');

  // Todo backendKey não-ação do registro precisa estar implementado no roteador único api.rs
  const apiText = fs.readFileSync(path.join(ROOT, 'crates', 'sonix-desktop', 'src', 'api.rs'), 'utf8');
  const unmappedKeys = CONTROLS_REGISTRY
    .filter((c) => c.backendKey && !c.backendKey.startsWith('tauri:') && c.backendKey !== 'preset.active')
    .filter((c) => {
      if (/^eq_1\.band([1-9]|10)\.gain_db$/.test(c.backendKey)) {
        return !apiText.includes('starts_with("eq_1.band")');
      }
      return !apiText.includes(`"${c.backendKey}"`);
    })
    .map((c) => `${c.id} (${c.backendKey})`);
  assert.deepEqual(unmappedKeys, [], `Chaves do registro ausentes no api.rs: ${unmappedKeys.join(', ')}`);
});

test('CA-6 e CA-7: Telas Tauri, painel Marshall fiel e jumper Plexi implementados', () => {
  const transportPath = path.join(UI_DIR, 'src', 'transport.ts');
  assert.ok(fs.existsSync(transportPath), 'transport.ts deve existir');
  const transport = fs.readFileSync(transportPath, 'utf8');
  assert.ok(transport.includes('requestAnimationFrame'), 'Deve coalescer comandos a 1 por rAF');
  assert.ok(transport.includes('invoke'), 'Deve usar Tauri invoke');

  const componentsDir = path.join(UI_DIR, 'src', 'components');
  for (const name of ['AmpHead.ts', 'Pedal.ts', 'Rack.ts', 'Knob.ts', 'Tube.ts']) {
    assert.ok(fs.existsSync(path.join(componentsDir, name)), `Componente ${name} deve existir`);
  }

  const screensDir = path.join(UI_DIR, 'src', 'screens');
  for (const scr of ['Estudio.ts', 'Bancada.ts', 'Gabinete.ts', 'Palco.ts', 'Biblioteca.ts', 'Audio.ts']) {
    assert.ok(fs.existsSync(path.join(screensDir, scr)), `Tela ${scr} deve existir`);
  }

  // CA-6: ordem esquerda -> direita dos painéis Plexi 1959 e JCM800 2203 em AmpHead.ts
  const ampSrc = fs.readFileSync(path.join(componentsDir, 'AmpHead.ts'), 'utf8');
  const plexiBlock = ampSrc.slice(ampSrc.indexOf("model === 'plexi_1959'"), ampSrc.indexOf("model === 'jcm800_2203'"));
  const plexiTokens = ['POWER', 'STANDBY', 'createJewelLight', 'PRESENCE', 'BASS', 'MIDDLE', 'TREBLE', 'VOLUME I', 'VOLUME II', 'I-HIGH', 'JUMPER'];
  let prev = -1;
  for (const tok of plexiTokens) {
    const idx = plexiBlock.indexOf(tok, prev + 1);
    assert.ok(idx > prev, `CA-6 Plexi: '${tok}' deve aparecer na ordem esquerda->direita`);
    prev = idx;
  }

  const jcmBlock = ampSrc.slice(ampSrc.indexOf("model === 'jcm800_2203'"), ampSrc.indexOf("model === 'twin_ab763'"));
  const jcmTokens = ['POWER', 'STANDBY', 'BRIT 800', 'PRESENCE', 'BASS', 'MIDDLE', 'TREBLE', 'MASTER', 'PRE-AMP', 'HIGH', 'LOW'];
  prev = -1;
  for (const tok of jcmTokens) {
    const idx = jcmBlock.indexOf(tok, prev + 1);
    assert.ok(idx > prev, `CA-6 JCM800: '${tok}' deve aparecer na ordem esquerda->direita`);
    prev = idx;
  }

  // CA-7: cabo jumper SVG e retorno ao II-Low
  assert.ok(plexiBlock.includes('plexi-jumper-cable'), 'CA-7: cabo patch SVG do Plexi deve existir');
  assert.ok(plexiBlock.includes("'low2'"), 'CA-7: jack II-Low deve enviar low2');
  assert.ok(plexiBlock.includes("'jumpered'"), 'CA-7: botão JUMPER deve enviar jumpered');
});

test('CA-8: ParamCoalescer reduz 100 atualizações num único frame rAF para 1 invoke com último valor', async () => {
  const calls = [];
  let rafQueue = [];
  const coalescer = new ParamCoalescer({
    invokeFn: async (cmd, args) => {
      calls.push({ cmd, args });
      return { ok: true };
    },
    rafFn: (cb) => {
      rafQueue.push(cb);
      return rafQueue.length;
    },
  });

  for (let i = 0; i < 100; i++) {
    coalescer.enqueue('amp_1.gain', Number((i * 0.1).toFixed(1)));
  }
  coalescer.enqueue('amp_1.bass', 7.5);
  assert.equal(calls.length, 0, 'Não deve invocar antes do próximo rAF');
  assert.equal(rafQueue.length, 1, 'Deve agendar apenas 1 callback de rAF');

  const flushCb = rafQueue.shift();
  flushCb(16);
  await Promise.resolve();
  await Promise.resolve();

  assert.equal(calls.length, 1, 'Deve coalescer 100 alterações para exatamente 1 invoke set_params');
  assert.equal(calls[0].cmd, 'set_params');
  assert.equal(calls[0].args.params['amp_1.gain'], 9.9);
  assert.equal(calls[0].args.params['amp_1.bass'], 7.5);
});

test('CA-8 e CA-10: Fontes offline e CSP/bundle Tauri sem origens externas ou wildcard', () => {
  const indexHtml = fs.readFileSync(path.join(UI_DIR, 'index.html'), 'utf8');
  assert.ok(!/https?:\/\//i.test(indexHtml), 'index.html não pode referenciar URLs externas');

  const distDir = path.join(UI_DIR, 'dist');
  assert.ok(fs.existsSync(path.join(distDir, 'index.html')), 'dist/index.html compilado deve existir');
  const distAssetsDir = path.join(distDir, 'assets');
  const distAssets = fs.readdirSync(distAssetsDir);
  assert.ok(distAssets.some((f) => f.endsWith('.woff2')), 'Deve empacotar fontes locais .woff2 para uso 100% offline');

  for (const file of [path.join(distDir, 'index.html'), ...distAssets.filter((f) => /\.(css|html|js)$/i.test(f)).map((f) => path.join(distAssetsDir, f))]) {
    const content = fs.readFileSync(file, 'utf8');
    assert.ok(!/fonts\.googleapis\.com|fonts\.gstatic\.com|unpkg\.com|cdn\.jsdelivr\.net/i.test(content), `Asset ${path.basename(file)} não pode depender de CDN externa`);
  }

  const conf = JSON.parse(fs.readFileSync(path.join(TAURI_DIR, 'tauri.conf.json'), 'utf8'));
  const csp = conf.app.security.csp;
  assert.ok(!csp.includes('*'), 'CSP não pode ter wildcard *');
  assert.equal(conf.bundle.windows.webviewInstallMode.type, 'offlineInstaller');
});

test('CA-12: faixa de dispositivo de audio desconectado aparece com lost e some com ok', async () => {
  // AUD-05
  const { JSDOM } = await import('jsdom');
  const dom = new JSDOM('<!doctype html><html><body><div id="app"></div></body></html>', {
    url: 'http://localhost/',
  });
  for (const k of [
    'window', 'document', 'HTMLElement', 'customElements', 'Element', 'SVGElement',
    'Node', 'Event', 'MouseEvent', 'KeyboardEvent', 'WheelEvent',
  ]) {
    globalThis[k] = k === 'window' ? dom.window : k === 'document' ? dom.window.document : dom.window[k];
  }
  globalThis.PointerEvent = dom.window.MouseEvent;

  const { AppStore } = await import('../../apps/sonix-app/ui/src/store.ts');
  const { mountAppShell, syncConnectionBanner } = await import('../../apps/sonix-app/ui/src/shell.ts');

  const store = new AppStore();
  const root = document.createElement('div');
  document.body.appendChild(root);
  const unmount = mountAppShell(root, store);
  const bannerEl = root.querySelector('.disconnect-banner');
  assert.ok(bannerEl, 'Faixa disconnect-banner deve existir no shell');

  syncConnectionBanner(bannerEl, store);
  assert.equal(bannerEl.style.display, 'none');

  store.applyTelemetry({ device_state: 'lost', connected: false });
  assert.equal(store.audioDeviceState, 'lost');
  assert.equal(store.deviceDisconnected, true);
  assert.equal(bannerEl.style.display, 'block');
  assert.equal(
    bannerEl.textContent,
    'Dispositivo de áudio desconectado — tentando reconectar…',
  );

  store.applyTelemetry({ device_state: 'ok', connected: true });
  assert.equal(store.audioDeviceState, 'ok');
  assert.equal(store.deviceDisconnected, false);
  assert.equal(bannerEl.style.display, 'none');

  unmount();
  root.remove();
});
