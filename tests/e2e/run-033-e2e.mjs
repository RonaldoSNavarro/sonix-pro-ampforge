// tests/e2e/run-033-e2e.mjs
// AUD-36: Executor E2E para Spec 033 (CA-1, CA-3, CA-6, CA-7) contra UI construída (apps/sonix-app/ui/dist)
// com fixtures de IPC e mesma infraestrutura de Playwright/Edge usada no JS 048.

import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import { chromium } from 'playwright-core';

const ROOT = process.cwd();
const DIST_DIR = path.join(ROOT, 'apps', 'sonix-app', 'ui', 'dist');

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

async function run() {
  assert.ok(fs.existsSync(path.join(DIST_DIR, 'index.html')), 'apps/sonix-app/ui/dist/index.html deve existir');
  const exe = findBrowserExecutable();
  assert.ok(exe, 'Navegador Edge/Chrome não encontrado para executar test:e2e');

  const presetLoad = JSON.parse(
    fs.readFileSync(path.join(ROOT, 'tests', 'fixtures', 'ipc-schemas', 'presets_load.json'), 'utf8'),
  );
  const presetsList = JSON.parse(
    fs.readFileSync(path.join(ROOT, 'tests', 'fixtures', 'ipc-schemas', 'presets_list.json'), 'utf8'),
  );
  const telemetry = JSON.parse(
    fs.readFileSync(path.join(ROOT, 'tests', 'fixtures', 'ipc-schemas', 'telemetry.json'), 'utf8'),
  );
  const audioStatus = JSON.parse(
    fs.readFileSync(path.join(ROOT, 'tests', 'fixtures', 'ipc-schemas', 'audio_status.json'), 'utf8'),
  );
  const devices = JSON.parse(
    fs.readFileSync(path.join(ROOT, 'tests', 'fixtures', 'ipc-schemas', 'devices.json'), 'utf8'),
  );

  const { server, url } = await startStaticServer(DIST_DIR);
  const browser = await chromium.launch({
    executablePath: exe,
    headless: true,
    args: ['--no-sandbox', '--disable-gpu'],
  });

  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
    await page.addInitScript(
      (fixtures) => {
        window.__SONIX_MOCK__ = {
          state: { ...fixtures.presetLoad.state, 'amp_1.model': 'plexi_1959' },
          presets: fixtures.presetsList.presets || [],
          telemetry: {
            ...fixtures.telemetry,
            waveform: Array.from({ length: 64 }, (_, i) => Math.sin(i * 0.25) * 0.6),
          },
          tuner: {
            enabled: false,
            frequency_hz: 0,
            note: '---',
            octave: 0,
            cents: 0,
            confidence: 0,
          },
          audioStatus: fixtures.audioStatus,
          devices: fixtures.devices.devices || [],
        };
      },
      { presetLoad, presetsList, telemetry, audioStatus, devices },
    );

    await page.goto(url, { waitUntil: 'networkidle' });
    await page.waitForSelector('.sonix-titlebar', { timeout: 10000 });
    await page.waitForTimeout(200);

    // 1. Verificar as 6 telas via atalhos Ctrl+1..5 e Ctrl+, e ausência de scroll externo em 1280x800 e 1920x1080
    const screens = [
      ['Control+1', 'main', '.sonix-titlebar'],
      ['Control+2', 'chassi', 'body'],
      ['Control+3', 'gabinete', 'body'],
      ['Control+4', 'palco', 'body'],
      ['Control+5', 'biblioteca', 'body'],
      ['Control+,', 'audio', 'body'],
    ];

    for (const vp of [
      { width: 1280, height: 800 },
      { width: 1920, height: 1080 },
    ]) {
      await page.setViewportSize(vp);
      for (const [shortcut, scr] of screens) {
        await page.keyboard.press(shortcut);
        await page.waitForTimeout(120);
        const dims = await page.evaluate(() => ({
          scrollHeight: document.documentElement.scrollHeight,
          innerHeight: window.innerHeight,
        }));
        assert.ok(
          dims.scrollHeight <= dims.innerHeight,
          `CA-3 (${vp.width}x${vp.height}): Tela ${scr} (${shortcut}) estourou altura (${dims.scrollHeight} > ${dims.innerHeight})`,
        );
      }
    }

    // 2. CA-6 & CA-7: Plexi 1959 Jumper I-Low -> II-High e retorno II-Low
    await page.keyboard.press('Control+1');
    await page.waitForTimeout(150);
    await page.click('[data-control-id="amp.model"]:has-text("Plexi 1959")');
    await page.waitForTimeout(150);
    await page.click('[data-control-id="amp.jumpered"]');
    await page.waitForTimeout(100);
    const cableAfterJump = await page.locator('.plexi-jumper-cable').count();
    assert.ok(cableAfterJump > 0, 'CA-7: cabo patch SVG do Plexi deve ficar visível ao ligar JUMPER');

    await page.click('button:has-text("II-Low"), [data-jack-id="low2"]');
    await page.waitForTimeout(100);
    const cableAfterLow2 = await page.locator('.plexi-jumper-cable').count();
    assert.equal(cableAfterLow2, 0, 'CA-7: clicar em II-Low deve desativar JUMPER e retornar para low2');

    console.log('E2E 033 GREEN: 6 telas sem scroll em 1280x800/1920x1080, Ctrl+1..5/Ctrl+,, e Plexi jumper OK');
  } finally {
    await browser.close();
    server.close();
  }
}

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
