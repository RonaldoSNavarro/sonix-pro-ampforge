import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import http from 'node:http';
import { chromium } from 'playwright-core';
import { PNG } from 'pngjs';
import { ssim } from 'ssim.js';

const ROOT = process.cwd();
const REF_DIR = path.join(ROOT, 'docs', 'ui-desktop', 'referencia');
const CANVAS_JSON_PATH = path.join(ROOT, 'docs', 'ui-desktop', 'canvas', 'canvas.json');
const DIST_DIR = path.join(ROOT, 'apps', 'sonix-app', 'ui', 'dist');
const STYLES_PATH = path.join(ROOT, 'apps', 'sonix-app', 'ui', 'src', 'styles.css');
const DIFF_DIR = path.join(os.tmpdir(), 'sonix-048-diffs');

function findBrowserExecutable() {
  const candidates = [
    process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE,
    'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
    'C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe',
    'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
  ].filter(Boolean);
  for (const p of candidates) {
    if (fs.existsSync(p)) return p;
  }
  return null;
}

function startStaticServer(rootDir) {
  const mime = {
    '.html': 'text/html; charset=utf-8',
    '.js': 'application/javascript; charset=utf-8',
    '.css': 'text/css; charset=utf-8',
    '.svg': 'image/svg+xml',
    '.png': 'image/png',
    '.woff2': 'font/woff2',
    '.woff': 'font/woff',
    '.json': 'application/json',
  };
  const server = http.createServer((req, res) => {
    const cleanUrl = (req.url || '/').split('?')[0];
    const relPath = cleanUrl === '/' ? 'index.html' : cleanUrl.replace(/^\/+/, '');
    const filePath = path.join(rootDir, relPath);
    if (!filePath.startsWith(rootDir) || !fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) {
      res.writeHead(404);
      res.end('Not found');
      return;
    }
    const ext = path.extname(filePath).toLowerCase();
    res.writeHead(200, { 'Content-Type': mime[ext] || 'application/octet-stream' });
    fs.createReadStream(filePath).pipe(res);
  });
  return new Promise((resolve) => {
    server.listen(0, '127.0.0.1', () => {
      const addr = server.address();
      resolve({ server, url: `http://127.0.0.1:${addr.port}` });
    });
  });
}

function cropPng(src, x, y, w, h) {
  const out = new PNG({ width: w, height: h });
  for (let row = 0; row < h; row++) {
    const sy = Math.min(src.height - 1, y + row);
    for (let col = 0; col < w; col++) {
      const sx = Math.min(src.width - 1, x + col);
      const si = (sy * src.width + sx) * 4;
      const di = (row * w + col) * 4;
      out.data[di] = src.data[si];
      out.data[di + 1] = src.data[si + 1];
      out.data[di + 2] = src.data[si + 2];
      out.data[di + 3] = 255;
    }
  }
  return out;
}

function computeSsim(pngA, pngB) {
  const a = { data: pngA.data, width: pngA.width, height: pngA.height };
  const b = { data: pngB.data, width: pngB.width, height: pngB.height };
  return ssim(a, b, { ssim: 'fast' }).mssim;
}

function writeDiffPng(actual, ref, outPath) {
  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  const w = Math.min(actual.width, ref.width);
  const h = Math.min(actual.height, ref.height);
  const diff = new PNG({ width: w, height: h });
  for (let i = 0; i < w * h; i++) {
    const idx = i * 4;
    diff.data[idx] = Math.min(255, Math.abs(actual.data[idx] - ref.data[idx]) * 3);
    diff.data[idx + 1] = Math.min(255, Math.abs(actual.data[idx + 1] - ref.data[idx + 1]) * 3);
    diff.data[idx + 2] = Math.min(255, Math.abs(actual.data[idx + 2] - ref.data[idx + 2]) * 3);
    diff.data[idx + 3] = 255;
  }
  fs.writeFileSync(outPath, PNG.sync.write(diff));
}

test('Spec 048 CA-1: docs/ui-desktop/referencia/ contém todos os PNGs nas dimensões de canvas.json', () => {
  const canvas = JSON.parse(fs.readFileSync(CANVAS_JSON_PATH, 'utf8'));
  const boards = Object.entries(canvas.boards);
  assert.equal(boards.length, 10, 'Devem existir 10 boards em canvas.json');
  for (const [dcFile, meta] of boards) {
    const pngName = dcFile.replace('.dc.html', '.png');
    const pngPath = path.join(REF_DIR, pngName);
    assert.ok(fs.existsSync(pngPath), `PNG de referência ausente: ${pngName}`);
    const parsed = PNG.sync.read(fs.readFileSync(pngPath));
    assert.equal(parsed.width, meta.w, `${pngName}: largura ${parsed.width} !== ${meta.w}`);
    assert.equal(parsed.height, meta.h, `${pngName}: altura ${parsed.height} !== ${meta.h}`);
  }
});

test('Spec 048 CA-11: styles.css mantém tokens de cor e fontes do design canvas', () => {
  const css = fs.readFileSync(STYLES_PATH, 'utf8');
  for (const token of ['#08090a', '#ff8a3d', '#3dd8ff', '#ffb347', 'Inter', 'Oswald', 'JetBrains Mono']) {
    assert.ok(css.includes(token), `CA-11: Token '${token}' ausente em styles.css`);
  }
});

test('Spec 048 CA-2..CA-10 & CA-12: fidelidade visual E2E em 1440x900 e estrutura DOM do design', async () => {
  assert.ok(fs.existsSync(path.join(DIST_DIR, 'index.html')), 'apps/sonix-app/ui/dist precisa estar compilado');
  const exe = findBrowserExecutable();
  assert.ok(exe, 'Edge ou Chrome precisa estar instalado para o teste E2E Playwright');

  const { server, url } = await startStaticServer(DIST_DIR);
  const browser = await chromium.launch({ executablePath: exe, headless: true });

  try {
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
    const page = await context.newPage();

    // Injeta fixture determinística no transport offline para comparação visual justa (CA-10)
    await page.addInitScript(() => {
      window.__SONIX_048_FIXTURE__ = {
        setlist: 'Setlist Show 2026',
        presetNo: 'P03',
        presetId: 'recto',
        presetName: 'Cali Dual Recto Modern Metal',
        sceneIdx: 5,
        calibrationDone: true,
        presets: [
          { id: 'usdeluxe', name: 'P01 Twin Clean Sparkle', setlist: 'Show 2026' },
          { id: 'plexi', name: 'P02 Plexi Jumped Crunch', setlist: 'Show 2026' },
          { id: 'recto', name: 'P03 Cali Dual Recto Modern Metal', setlist: 'Show 2026' },
          { id: 'ac30', name: 'P04 AC30 Top Boost Chime', setlist: 'Show 2026' },
          { id: 'brit800', name: 'P05 Brit 800 Studio', setlist: 'Show 2026' },
        ],
        state: {
          'amp_1.model': 'dual_rect',
          'amp_1.power': true,
          'amp_1.standby': false,
          'amp_1.channel_mode': 'modern',
          'amp_1.power_tube': '6l6gc',
          'amp_1.preamp_tube': '12ax7',
          'amp_1.preamp_plate_v': 458,
          'amp_1.gain': 7.0,
          'amp_1.treble': 6.0,
          'amp_1.middle': 4.0,
          'amp_1.bass': 6.0,
          'amp_1.presence': 5.0,
          'amp_1.master': 3.5,
          'gate.enabled': true,
          'drive.enabled': true,
          'drive.model': 'ts808',
          'split.enabled': true,
          'amp_1.enabled': true,
          'cab_1.enabled': true,
          'cab_1.model': 'v30_4x12',
          'cab_1.speaker': 'v30_4x12',
          'cab_1.resonance_hz': 95,
          'cab_1.hicut_khz': 10.5,
          'cab_1.mic1.enabled': true,
          'cab_1.mic1.model': 'sm57',
          'cab_1.mic1.position': 0.15,
          'cab_1.mic1.distance_cm': 2.0,
          'cab_1.mic1.level_db': 0,
          'cab_1.mic1.angle_deg': '0',
          'cab_1.mic2.enabled': true,
          'cab_1.mic2.model': 'r121',
          'cab_1.mic2.position': 0.6,
          'cab_1.mic2.distance_cm': 8.0,
          'cab_1.mic2.level_db': -6,
          'cab_1.mic2.angle_deg': '45',
          'eq_1.enabled': true,
          'comp_2.enabled': false,
          'amp_2.enabled': true,
          'cab_2.enabled': true,
          'mod_2.enabled': true,
          'dly_1.enabled': true,
          'rev_1.enabled': true,
          'rev_1.decay_s': 2.7,
          'engine.oversampling': '4x',
          'looper.state': 'idle',
          'tuner.enabled': true,
        },
        telemetry: {
          input_peak_db: -34.0,
          output_peak_db: -24.0,
          cpu_usage_pct: 32.8,
          gate_state: 'open',
          gate_gr_db: -3.2,
          waveform: Array.from({ length: 64 }, (_, i) => Math.sin(i * 0.25) * 0.6),
        },
        tuner: {
          enabled: true,
          frequency_hz: 110.0,
          note: 'A',
          octave: 2,
          cents: 1.5,
          confidence: 0.96,
        },
        audioStatus: {
          backend: 'wasapi',
          mode: 'wasapi_exclusive',
          raw_processing: false,
          sample_rate: 48000,
          input_period_frames: 64,
          output_period_frames: 64,
          driver_input_latency_ms: 1.33,
          driver_output_latency_ms: 1.33,
          measured_roundtrip_ms: null,
          estimated_roundtrip_ms: 3.0,
          underruns: 0,
          overruns: 0,
          adc_clips: 0,
          cpu_avg_pct: 24.5,
          input_device: 'Focusrite 2i2',
          output_device: 'Focusrite 2i2',
          connected: true,
          mode_capabilities: {
            wasapi_shared: true,
            wasapi_shared_low_latency: true,
            wasapi_exclusive: false,
            asio: false,
          },
        },
        devices: [
          { name: 'Focusrite 2i2', is_input: true, is_output: true },
        ],
      };
    });

    await page.goto(url, { waitUntil: 'networkidle' });
    await page.waitForTimeout(200);

    // CA-2..CA-7 DOM checks na tela principal
    const mainMetrics = await page.evaluate(() => {
      const titlebar = document.querySelector('.sonix-titlebar');
      const menubar = document.querySelector('.sonix-menubar');
      const toolbar = document.querySelector('[data-region="toolbar"]');
      const grid = document.querySelector('[data-region="grid"]');
      const stage = document.querySelector('[data-region="stage"]');
      const inspector = document.querySelector('[data-region="inspector"]');
      const statusbar = document.querySelector('.sonix-statusbar');
      const ampSlot = document.querySelector('[data-region="amp-slot"]');
      return {
        titleText: titlebar?.textContent?.trim() ?? '',
        menuText: menubar?.textContent?.trim() ?? '',
        toolbarRect: toolbar?.getBoundingClientRect()?.toJSON() ?? null,
        toolbarHasSelect: Boolean(toolbar?.querySelector('select')),
        toolbarText: toolbar?.textContent ?? '',
        sceneButtonsCount: toolbar?.querySelectorAll('[data-control-id="topbar.scene"]').length ?? 0,
        meterSegsCount: toolbar?.querySelectorAll('.m-seg').length ?? 0,
        gridText: grid?.textContent ?? '',
        gridCellsCount: grid?.querySelectorAll('.m-cell').length ?? 0,
        ampRect: ampSlot?.getBoundingClientRect()?.toJSON() ?? null,
        stageText: stage?.textContent ?? '',
        inspectorText: inspector?.textContent ?? '',
        inspectorTracksCount: inspector?.querySelectorAll('.m-track').length ?? 0,
        statusRect: statusbar?.getBoundingClientRect()?.toJSON() ?? null,
        statusText: statusbar?.textContent ?? '',
        scrollOverflow:
          document.documentElement.scrollHeight > document.documentElement.clientHeight ||
          document.documentElement.scrollWidth > document.documentElement.clientWidth,
      };
    });

    assert.ok(
      mainMetrics.titleText.includes('SONIX PRO — AMPFORGE') && mainMetrics.titleText.includes('›'),
      `CA-2: título deve seguir "SONIX PRO — AMPFORGE · {setlist} › {preset}", obtido: ${mainMetrics.titleText}`
    );
    assert.ok(
      mainMetrics.menuText.includes('ponte UI→DSP'),
      `CA-2: barra de menu deve exibir indicador "ponte UI→DSP", obtido: ${mainMetrics.menuText}`
    );
    assert.ok(mainMetrics.toolbarRect, 'CA-3: toolbar [data-region="toolbar"] obrigatória');
    assert.equal(Math.round(mainMetrics.toolbarRect.height), 68, 'CA-3: toolbar deve ter 68 px de altura');
    assert.equal(mainMetrics.toolbarHasSelect, false, 'CA-3: não deve haver <select> na toolbar');
    assert.equal(mainMetrics.sceneButtonsCount, 8, 'CA-3: toolbar deve ter 8 botões de cena');
    assert.equal(mainMetrics.meterSegsCount, 40, 'CA-3: IN/OUT devem ter 20 segmentos cada (40 total)');
    for (const lbl of ['SALVAR', 'TAP', 'AFINADOR', 'PALCO']) {
      assert.ok(mainMetrics.toolbarText.includes(lbl), `CA-3: toolbar deve conter "${lbl}"`);
    }
    assert.ok(mainMetrics.gridText.includes('Matriz de sinal'), 'CA-4: cabeçalho deve conter "Matriz de sinal"');
    assert.equal(mainMetrics.gridCellsCount, 16, 'CA-4: matriz 11x2 exibe linha A + wah + linha B (16 células)');
    assert.ok(mainMetrics.ampRect, 'CA-5: AmpHead deve ficar em [data-region="amp-slot"]');
    assert.equal(Math.round(mainMetrics.ampRect.width), 788, 'CA-5: AmpHead no palco deve ter largura 788 px');
    assert.equal(Math.round(mainMetrics.ampRect.height), 226, 'CA-5: AmpHead no palco deve ter altura 226 px');
    assert.ok(mainMetrics.stageText.includes('LINHA B / PÓS'), 'CA-5: Pedalboard deve mostrar "LINHA B / PÓS"');
    assert.ok(mainMetrics.stageText.includes('Rack · loop / pós-amp'), 'CA-5: palco deve mostrar "Rack · loop / pós-amp"');
    assert.equal(mainMetrics.scrollOverflow, false, 'CA-5: zero scroll em 1440x900');
    assert.ok(mainMetrics.inspectorText.includes('Modifier ·'), 'CA-6: inspetor deve conter seção "Modifier · {alvo}"');
    assert.ok(mainMetrics.inspectorTracksCount >= 1, 'CA-6: inspetor deve usar trilhos horizontais .m-track');
    assert.ok(mainMetrics.statusRect, 'CA-7: barra de status obrigatória');
    assert.equal(Math.round(mainMetrics.statusRect.height), 26, 'CA-7: barra de status deve ter 26 px');
    assert.ok(mainMetrics.statusText.toLowerCase().includes('estimada'), 'CA-7: statusbar deve rotular latência estimada');

    // CA-10: SSIM visual na Main
    const mainShot = PNG.sync.read(await page.screenshot());
    const mainRef = PNG.sync.read(fs.readFileSync(path.join(REF_DIR, 'Main.png')));
    const mainFullSsim = computeSsim(mainShot, mainRef);
    const regions = [
      ['Main_title_menu', 0, 0, 1440, 60],
      ['Main_toolbar', 0, 60, 1440, 68],
      ['Main_grid', 0, 128, 1440, 136],
      ['Main_stage', 0, 264, 1140, 610],
      ['Main_inspector', 1140, 264, 300, 610],
      ['Main_statusbar', 0, 874, 1440, 26],
    ];
    const mainFailures = [];
    for (const [rName, x, y, w, h] of regions) {
      const regScore = computeSsim(cropPng(mainShot, x, y, w, h), cropPng(mainRef, x, y, w, h));
      console.log(`CA-10 Main ${rName}: SSIM=${regScore.toFixed(4)}`);
      if (regScore < 0.85) {
        mainFailures.push(`${rName}=${regScore.toFixed(4)}`);
      }
    }
    console.log(`CA-10 Main FULL: SSIM=${mainFullSsim.toFixed(4)}`);
    if (mainFailures.length > 0 || mainFullSsim < 0.80) {
      fs.mkdirSync(DIFF_DIR, { recursive: true });
      fs.writeFileSync(path.join(DIFF_DIR, 'Main-actual.png'), PNG.sync.write(mainShot));
      writeDiffPng(mainShot, mainRef, path.join(DIFF_DIR, 'Main-diff.png'));
    }
    assert.equal(mainFailures.length, 0, `CA-10: Main regiões abaixo de 0.85: ${mainFailures.join(', ')}`);
    assert.ok(mainFullSsim >= 0.80, `CA-10: Main full SSIM=${mainFullSsim.toFixed(4)} < 0.80`);

    // CA-8 & CA-10: Chassi
    await page.keyboard.press('Control+2');
    await page.waitForTimeout(150);
    const chassiHasElements = await page.evaluate(() => {
      const t = document.body.textContent ?? '';
      return t.includes('TRANSF. FORÇA') && t.includes('TRANSF. SAÍDA') && t.includes('CHOKE');
    });
    assert.ok(chassiHasElements, 'CA-8: Chassi deve renderizar transformadores e choke');
    const chassiShot = PNG.sync.read(await page.screenshot());
    const chassiRef = PNG.sync.read(fs.readFileSync(path.join(REF_DIR, 'Chassi.png')));
    const chassiSsim = computeSsim(chassiShot, chassiRef);
    if (chassiSsim < 0.80) writeDiffPng(chassiShot, chassiRef, path.join(DIFF_DIR, 'Chassi-diff.png'));
    assert.ok(chassiSsim >= 0.80, `CA-10: Chassi full SSIM=${chassiSsim.toFixed(4)} < 0.80`);

    // CA-8 & CA-10: Gabinete
    await page.keyboard.press('Control+3');
    await page.waitForTimeout(150);
    const gabHasElements = await page.evaluate(() => {
      const mics = document.querySelectorAll('[data-mic-id], .g-mic');
      return mics.length >= 2;
    });
    assert.ok(gabHasElements, 'CA-8: Gabinete deve renderizar os 2 microfones arrastáveis sobre o falante');
    const gabShot = PNG.sync.read(await page.screenshot());
    const gabRef = PNG.sync.read(fs.readFileSync(path.join(REF_DIR, 'Gabinete.png')));
    const gabSsim = computeSsim(gabShot, gabRef);
    if (gabSsim < 0.80) writeDiffPng(gabShot, gabRef, path.join(DIFF_DIR, 'Gabinete-diff.png'));
    assert.ok(gabSsim >= 0.80, `CA-10: Gabinete full SSIM=${gabSsim.toFixed(4)} < 0.80`);

    // CA-9 & CA-10: Palco
    await page.keyboard.press('Control+4');
    await page.waitForTimeout(150);
    const palcoOk = await page.evaluate(() => {
      const fsDock = document.querySelector('[data-region="footswitch-dock"]');
      const btns = document.querySelectorAll('[data-region="footswitch-dock"] [data-control-id^="palco.fs"]');
      return Boolean(fsDock) && btns.length >= 8;
    });
    assert.ok(palcoOk, 'CA-9: Palco deve ter dock de 8 footswitches');
    const palcoShot = PNG.sync.read(await page.screenshot());
    const palcoRef = PNG.sync.read(fs.readFileSync(path.join(REF_DIR, 'Palco.png')));
    const palcoSsim = computeSsim(palcoShot, palcoRef);
    if (palcoSsim < 0.80) writeDiffPng(palcoShot, palcoRef, path.join(DIFF_DIR, 'Palco-diff.png'));
    assert.ok(palcoSsim >= 0.80, `CA-10: Palco full SSIM=${palcoSsim.toFixed(4)} < 0.80`);
  } finally {
    await browser.close();
    server.close();
  }
});
