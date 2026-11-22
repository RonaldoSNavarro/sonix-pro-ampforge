import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

function collectRustTestFiles() {
  const dirs = [];
  const cratesDir = resolve('crates');
  for (const entry of readdirSync(cratesDir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      const testsDir = join(cratesDir, entry.name, 'tests');
      try {
        for (const f of readdirSync(testsDir, { withFileTypes: true })) {
          if (f.isFile() && f.name.endsWith('.rs')) {
            dirs.push(join(testsDir, f.name));
          }
        }
      } catch {
        // crate may not have a tests/ directory
      }
    }
  }
  const tauriTestsDir = resolve('apps/sonix-app/src-tauri/tests');
  try {
    for (const f of readdirSync(tauriTestsDir, { withFileTypes: true })) {
      if (f.isFile() && f.name.endsWith('.rs')) {
        dirs.push(join(tauriTestsDir, f.name));
      }
    }
  } catch {
    // ignore if missing
  }
  return dirs;
}

test('COR-01: todo teste Rust exigido por um wrapper existe', () => {
  const rustFiles = collectRustTestFiles();
  const definedFns = new Set();
  const fnRegex = /\bfn\s+((?:test_|ca\d+_)\w+)\s*\(/g;

  for (const file of rustFiles) {
    const content = readFileSync(file, 'utf-8');
    let match;
    while ((match = fnRegex.exec(content)) !== null) {
      definedFns.add(match[1]);
    }
  }

  const contractDir = resolve('tests/contract');
  const jsFiles = readdirSync(contractDir)
    .filter((f) => f.endsWith('.test.js'))
    .sort();

  const missing = [];

  for (const jsFile of jsFiles) {
    const fullPath = join(contractDir, jsFile);
    const lines = readFileSync(fullPath, 'utf-8').split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const mRegex = /\b((?:test_|ca\d+_)\w+)\s+(?:\\\.\\\.\\\.|\.\.\.)\s+ok\b/g;
      let m;
      while ((m = mRegex.exec(line)) !== null) {
        const name = m[1];
        if (!definedFns.has(name)) {
          missing.push(`tests/contract/${jsFile}:${i + 1}: ${name}`);
        }
      }
    }
  }

  assert.equal(
    missing.length,
    0,
    `Testes Rust exigidos por wrappers não encontrados:\n${missing.join('\n')}`
  );
});
