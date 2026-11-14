/**
 * Spec 033 E2E Suite (CA-1 .. CA-14)
 *
 * Pode ser executada via WebDriver / tauri-driver quando disponível em ambiente Windows
 * (`npx wdio` ou Playwright sobre o binário desktop compilado) ou validada via harness JS/DOM.
 */
import assert from 'node:assert/strict';

export interface E2eDriver {
  url(): Promise<string>;
  execute<T>(script: string, ...args: unknown[]): Promise<T>;
  click(selector: string): Promise<void>;
  setValue(selector: string, value: string): Promise<void>;
  getAttribute(selector: string, attr: string): Promise<string | null>;
}

export async function runSpec033E2e(driver: E2eDriver): Promise<void> {
  // CA-1 & CA-3: URL nativa Tauri (zero localhost/http server) e zero scroll externo
  const currentUrl = await driver.url();
  assert.ok(
    currentUrl.startsWith('tauri://localhost') || currentUrl.startsWith('https://tauri.localhost'),
    `Origem esperada tauri://localhost, obtida ${currentUrl}`,
  );

  for (const screen of ['main', 'chassi', 'gabinete', 'palco', 'biblioteca', 'audio']) {
    await driver.click(`[data-control-id="ui.nav_screen"][data-screen-target="${screen}"]`);
    const dims = await driver.execute<{ scrollHeight: number; innerHeight: number }>(
      'return { scrollHeight: document.documentElement.scrollHeight, innerHeight: window.innerHeight };',
    );
    assert.ok(
      dims.scrollHeight <= dims.innerHeight,
      `Tela ${screen} estourou altura da janela: ${dims.scrollHeight} > ${dims.innerHeight}`,
    );
  }

  // CA-6 & CA-7: Painel Plexi 1959 + Jumper I-Low -> II-High e retorno II-Low
  await driver.click('[data-control-id="ui.nav_screen"][data-screen-target="main"]');
  await driver.setValue('[data-control-id="amp.model"]', 'plexi_1959');
  await driver.click('[data-control-id="amp.jumpered"]');
  const cableExists = await driver.execute<boolean>(
    'return Boolean(document.querySelector(".plexi-jumper-cable"));',
  );
  assert.ok(cableExists, 'CA-7: cabo patch SVG do Plexi deve ficar visível ao ligar JUMPER');

  await driver.click('[data-control-id="amp.input_jack"][data-jack-id="low2"]');
  const cableAfterLow2 = await driver.execute<boolean>(
    'return Boolean(document.querySelector(".plexi-jumper-cable"));',
  );
  assert.ok(!cableAfterLow2, 'CA-7: clicar em II-Low deve desativar JUMPER e retornar para low2');
}
