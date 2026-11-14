// tests/contract/027-ui-controles-reais-e-layout-responsivo.test.js
// Contrato de Aceite Automatizado - Spec 027 (Ponte UI->DSP Completa e Layout Responsivo Desktop sem Scroll)
// Cobre Critérios de Aceite CA-1 a CA-8

import { test, describe } from 'node:test';
import assert from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

describe('Spec 027: Ponte UI->DSP Completa e Layout Responsivo Desktop sem Scroll', () => {
  const appJsPath = resolve('design-system/app.js');
  const stylesCssPath = resolve('design-system/styles.css');
  const indexHtmlPath = resolve('design-system/index.html');

  const appJs = readFileSync(appJsPath, 'utf-8');
  const stylesCss = readFileSync(stylesCssPath, 'utf-8');
  const indexHtml = readFileSync(indexHtmlPath, 'utf-8');

  test('test_ca_01_knob_changes_dispatch_api_params', () => {
    // CA-1: Girar qualquer knob do painel frontal dispara POST /api/params com chave correspondente
    // updateKnobValue deve invocar syncParamsToBackend
    const updateKnobFnMatch = appJs.match(/function\s+updateKnobValue\s*\([^)]*\)\s*\{([\s\S]*?)\n\}/);
    assert.ok(updateKnobFnMatch, 'updateKnobValue deve ser declarada em app.js');
    const updateKnobBody = updateKnobFnMatch[1];
    assert.ok(
      updateKnobBody.includes('syncParamsToBackend'),
      'updateKnobValue deve invocar syncParamsToBackend ao alterar qualquer knob'
    );

    // Mapeamento de chaves canônicas para os knobs do amplificador
    assert.ok(
      appJs.includes("'amp_1.gain'") || appJs.includes('"amp_1.gain"'),
      'app.js deve mapear ganho para amp_1.gain'
    );
    assert.ok(
      appJs.includes("'amp_1.bass'") || appJs.includes('"amp_1.bass"'),
      'app.js deve mapear graves para amp_1.bass'
    );
    assert.ok(
      appJs.includes("'amp_1.treble'") || appJs.includes('"amp_1.treble"'),
      'app.js deve mapear agudos para amp_1.treble'
    );
  });

  test('test_ca_02_mod_shop_and_cab_controls_dispatch_api_params', () => {
    // CA-2: Alterar oversampling, bright switch, exp pedal, Mod Shop e Cab Room envia chaves em POST /api/params
    // Mod Shop sliders e selects
    const expectedModShopKeys = [
      'amp_1.bias_ma',
      'amp_1.sag',
      'amp_1.nfb',
      'amp_1.xformer_sat',
      'amp_1.preamp_plate_v',
      'amp_1.triode_asym',
      'amp_1.preamp_tube',
      'amp_1.power_tube',
      'amp_1.tonestack',
      'amp_1.rectifier',
    ];
    for (const key of expectedModShopKeys) {
      assert.ok(
        appJs.includes(`'${key}'`) || appJs.includes(`"${key}"`),
        `app.js deve sincronizar ${key} ao alterar controles do Mod Shop`
      );
    }

    // Cab Room controls
    const expectedCabKeys = [
      'cab_1.resonance_hz',
      'cab_1.hicut_khz',
      'cab_1.ir_enabled',
      'cab_1.mic1.distance_cm',
      'cab_1.mic2.distance_cm',
      'cab_1.speaker',
    ];
    for (const key of expectedCabKeys) {
      assert.ok(
        appJs.includes(`'${key}'`) || appJs.includes(`"${key}"`),
        `app.js deve sincronizar ${key} ao alterar controles da sala de gabinetes`
      );
    }

    // Oversampling e bright switch
    assert.ok(
      appJs.includes("'engine.oversampling'") || appJs.includes('"engine.oversampling"'),
      'app.js deve enviar engine.oversampling ao alterar #oversamplingSelect'
    );
    assert.ok(
      appJs.includes("'amp_1.bright'") || appJs.includes('"amp_1.bright"'),
      'app.js deve enviar amp_1.bright ao alternar #brightSwitchBtn'
    );
  });

  test('test_ca_03_all_grid_blocks_render_interactive_controls', () => {
    // CA-3: Selecionar no grid qualquer bloco populado renderiza controles interativos (<input>, <select>, <button>)
    // Nunca deve exibir "sem controles nesta versão" para blocos populados
    const requiredBlocks = [
      'blk_gate',
      'blk_comp_2',
      'blk_eq_1',
      'blk_mod_2',
      'blk_dly_1',
      'blk_rev_1',
      'blk_split',
      'blk_amp_1',
      'blk_cab_1',
    ];

    for (const blk of requiredBlocks) {
      assert.ok(
        appJs.includes(blk) || appJs.includes(blk.replace('blk_', '')),
        `openBlockEditor deve possuir seção para ${blk}`
      );
    }

    // openBlockEditor deve conter inputs para comp_2, eq_1, mod_2, dly_1, rev_1, split, gate, amp, cab
    assert.ok(appJs.includes('gate.threshold_db'), 'Editor do blk_gate deve conter controle para gate.threshold_db');
    assert.ok(appJs.includes('comp_2.input') || appJs.includes('comp_2.ratio'), 'Editor do blk_comp_2 deve conter inputs interativos');
    assert.ok(appJs.includes('eq_1.band') || appJs.includes('eq_1_band'), 'Editor do blk_eq_1 deve conter controles interativos de bandas');
    assert.ok(appJs.includes('mod_2.rate_hz') || appJs.includes('mod_2.depth'), 'Editor do blk_mod_2 deve conter inputs interativos');
    assert.ok(appJs.includes('dly_1.repeat_rate') || appJs.includes('dly_1.intensity'), 'Editor do blk_dly_1 deve conter inputs interativos');
    assert.ok(appJs.includes('rev_1.decay_s') || appJs.includes('rev_1.mix'), 'Editor do blk_rev_1 deve conter inputs interativos');
    assert.ok(appJs.includes('split.crossover_hz') || appJs.includes('split.mode'), 'Editor do blk_split deve conter inputs interativos');
    assert.ok(
      appJs.includes("block.cat === 'DRIVE'") || appJs.includes("block.category === 'DRIVE'") || appJs.includes("includes('drive')"),
      'openBlockEditor deve contemplar blocos da categoria DRIVE e IDs com drive'
    );
    assert.ok(
      appJs.includes("'amp_2.gain'") && appJs.includes("'amp_2.master'"),
      'Editor do blk_amp_2 deve sincronizar parâmetros isolados de amp_2 sem sobrescrever amp_1'
    );

    // Validação que blocos populados não recebem 'sem controles nesta versão'
    const openBlockFn = appJs.match(/function\s+openBlockEditor\s*\([^)]*\)\s*\{([\s\S]*?)\n\}/);
    assert.ok(openBlockFn, 'openBlockEditor deve existir');
    const openBlockContent = openBlockFn[1];
    // Garantir que comp_2, eq_1, mod_2, dly_1, rev_1, split não exibem texto estático "Controles <bloco>:"
    assert.ok(
      !openBlockContent.includes('Controles comp_2: input, output'),
      'comp_2 não deve exibir string estática informativa'
    );
    assert.ok(
      !openBlockContent.includes('Controles eq_1: 8 bandas'),
      'eq_1 não deve exibir string estática informativa'
    );
  });

  test('test_ca_04_cpu_meter_and_audio_status_reflect_live_telemetry', () => {
    // CA-4: #cpuMeterText exibe cpu_usage_pct e badges de áudio refletem /api/audio/status
    assert.ok(
      appJs.includes('cpuMeterText') && appJs.includes('cpu_usage_pct'),
      'app.js deve atualizar #cpuMeterText com dados da telemetria'
    );
    assert.ok(
      appJs.includes('/api/audio/status'),
      'app.js deve consultar /api/audio/status para atualizar badges de áudio'
    );
    assert.ok(
      appJs.includes('audioModeText') && appJs.includes('audioSampleRateText') && appJs.includes('audioPeriodsText'),
      'app.js deve atualizar os elementos de telemetria de áudio (#audioModeText, #audioSampleRateText, #audioPeriodsText)'
    );
  });

  test('test_ca_05_desktop_viewports_fit_without_scroll', () => {
    // CA-5: Layout desktop em 1280x820 e 1920x1080 cabe dentro de 100vh sem overflow/scroll
    // styles.css deve restringir .app-shell para height: 100vh e overflow: hidden
    assert.match(
      stylesCss,
      /\.app-shell\s*\{[^}]*height:\s*100vh;[^}]*overflow:\s*hidden;/s,
      '.app-shell deve ter height: 100vh e overflow: hidden para evitar scrollbars na janela'
    );
    assert.match(
      stylesCss,
      /html,\s*body\s*\{[^}]*height:\s*100%/s,
      'html, body devem ter height: 100% e overflow: hidden'
    );
    assert.ok(
      stylesCss.includes('min-height: 0') && stylesCss.includes('clamp('),
      'styles.css deve utilizar min-height: 0 e clamp() para dimensionamento adaptativo proporcional em 1280x820'
    );
  });

  test('test_ca_06_amp_model_switch_adapts_front_panel_knobs', () => {
    // CA-6: Trocar ampModelSelect reconstrói dinamicamente os knobs do faceplate
    assert.ok(
      appJs.includes('ampModelSelect'),
      'app.js deve monitorar #ampModelSelect'
    );
    assert.ok(
      appJs.includes('plexi_1959') && appJs.includes('ac30_tb') && appJs.includes('twin_ab763'),
      'app.js deve conter definições ou adaptações específicas para plexi_1959, ac30_tb e twin_ab763'
    );
    assert.ok(
      appJs.includes('amp_1.volume_2') || appJs.includes('volume_2'),
      'Plexi 1959 deve expor controle de Volume 2 / Loudness II'
    );
    assert.ok(
      appJs.includes('amp_1.cut') || appJs.includes('cut'),
      'AC30 Top Boost deve expor controle Cut'
    );
  });

  test('test_ca_07_api_error_handling_graceful_recovery', () => {
    // CA-7: Falha de rede ou HTTP 4xx/5xx emite indicador visual discreto sem quebrar renderização
    assert.ok(
      appJs.includes('engineLinkBadge'),
      'app.js deve gerenciar indicador de estado de conexão #engineLinkBadge'
    );
    assert.ok(
      appJs.includes('setApiError') || appJs.includes('offline') || appJs.includes('OFFLINE'),
      'app.js deve atualizar badge em caso de erro na API'
    );
    // requestAnimationFrame não deve ser interrompido por exceções não tratadas em fetch
    assert.ok(
      appJs.includes('requestAnimationFrame'),
      'app.js deve manter loop de animação'
    );
  });

  test('test_ca_08_non_regression_existing_ui_contracts', () => {
    // CA-8: Não-regressão das suítes de UI 011, 014 e 020
    const res = spawnSync('node', [
      '--test',
      'tests/contract/011-design-system-skeuomorfico-componentes-3d-hardware-shell.test.js',
      'tests/contract/014-ponte-de-controle-ui-dsp-tempo-real.test.js',
      'tests/contract/020-amplificadores-por-modelo-painel-fiel.test.js',
    ], {
      encoding: 'utf-8',
    });
    assert.strictEqual(
      res.status,
      0,
      `Regressão detectada em contratos de UI existentes:\n${res.stdout}\n${res.stderr}`
    );
  });
});
