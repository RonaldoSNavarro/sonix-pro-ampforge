import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '../..');

function resolveTargetDir() {
  const cfgPath = path.join(root, '.cargo/config.toml');
  if (fs.existsSync(cfgPath)) {
    const cfg = fs.readFileSync(cfgPath, 'utf8');
    const m = cfg.match(/target-dir\s*=\s*"([^"]+)"/);
    if (m) return m[1];
  }
  return path.join(root, 'target');
}

test('Spec 050 CA-1: Cargo.toml define custom-protocol e sonix-app.exe release embute ui/dist sem localhost:1420', () => {
  const cargoToml = fs.readFileSync(
    path.join(root, 'apps/sonix-app/src-tauri/Cargo.toml'),
    'utf8',
  );
  assert.match(
    cargoToml,
    /custom-protocol\s*=\s*\[\s*"tauri\/custom-protocol"\s*\]/,
    'apps/sonix-app/src-tauri/Cargo.toml deve definir feature custom-protocol = ["tauri/custom-protocol"]',
  );

  const distHtmlPath = path.join(root, 'apps/sonix-app/ui/dist/index.html');
  const distHtml = fs.readFileSync(distHtmlPath, 'utf8');
  const assetMatches = [...distHtml.matchAll(/\/(assets\/index-[^"']+)/g)].map((m) => m[1]);
  assert.ok(assetMatches.length >= 2, 'ui/dist/index.html deve referenciar assets JS e CSS gerados pelo Vite');

  const targetDir = resolveTargetDir();
  const exePath = path.join(targetDir, 'release', 'sonix-app.exe');
  assert.ok(
    fs.existsSync(exePath),
    `Executável release deve existir em ${exePath} (rode: cargo build --release -p sonix-app --features custom-protocol)`,
  );
  const distMtime = fs.statSync(distHtmlPath).mtimeMs;
  const exeMtime = fs.statSync(exePath).mtimeMs;
  assert.ok(
    exeMtime >= distMtime,
    `sonix-app.exe está desatualizado em relação a ui/dist/index.html — rode rebuild release com: cargo build --release -p sonix-app --features custom-protocol`,
  );

  const exeBuf = fs.readFileSync(exePath);
  for (const asset of assetMatches) {
    assert.equal(
      exeBuf.includes(Buffer.from(asset)),
      true,
      `sonix-app.exe release deve embutir o asset ${asset} de ui/dist`,
    );
  }
});
