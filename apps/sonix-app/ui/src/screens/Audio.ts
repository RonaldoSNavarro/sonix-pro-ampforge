import { createKnob } from '../components/Knob.ts';
import type { AppStore } from '../store.ts';
import { makeChipButton, num, str } from './_helpers.ts';

export function renderAudioScreen(store: AppStore): HTMLElement {
  const root = document.createElement('div');
  root.style.cssText = 'flex:1;min-height:0;display:flex;flex-direction:column;background:#0e1013;overflow:hidden;';

  const aud = store.audioStatus;
  const caps = aud.mode_capabilities ?? {
    wasapi_shared: true,
    wasapi_shared_low_latency: true,
    wasapi_exclusive: false,
    asio: false,
  };
  const srKhz = ((aud.sample_rate || 48000) / 1000).toFixed(0);
  const frames = aud.output_period_frames || 64;
  const driverMs = Number(aud.driver_output_latency_ms || (frames / (aud.sample_rate || 48000)) * 1000).toFixed(2);
  const rtLabel =
    typeof aud.measured_roundtrip_ms === 'number'
      ? `${aud.measured_roundtrip_ms.toFixed(2)} ms medida`
      : `${Number(aud.estimated_roundtrip_ms || 3.0).toFixed(2)} ms (estimada)`;

  // Local selection refs shared with Aplicar footer
  const pending = {
    backend: aud.mode || aud.backend || 'wasapi_shared',
    inputDevice: aud.input_device || '',
    outputDevice: aud.output_device || '',
    sampleRate: aud.sample_rate || 48000,
    bufferSize: frames,
  };

  // ── 1. Modal Title Bar ─────────────────────────────────────────────────────
  const titleBar = document.createElement('div');
  titleBar.style.cssText =
    'height:32px;flex:none;display:flex;align-items:center;background:#07080a;border-bottom:1px solid #1d2127;padding-left:12px;';
  const tLbl = document.createElement('span');
  tLbl.style.cssText = 'font-size:12px;color:#c9ced4;flex:1;';
  tLbl.textContent = 'Configurações de áudio — SONIX PRO AMPFORGE';
  const closeBtn = document.createElement('button');
  closeBtn.type = 'button';
  closeBtn.setAttribute('data-control-id', 'ui.nav_screen');
  closeBtn.setAttribute('aria-label', 'Fechar');
  closeBtn.style.cssText =
    'width:46px;height:32px;background:transparent;border:0;color:#c9ced4;display:flex;align-items:center;justify-content:center;cursor:pointer;';
  closeBtn.innerHTML =
    '<svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><path d="M0 0l10 10M10 0L0 10" stroke="currentColor"/></svg>';
  closeBtn.addEventListener('click', () => store.setScreen('main'));
  titleBar.append(tLbl, closeBtn);
  root.appendChild(titleBar);

  // ── 2. Body Split: Sidebar + Content Pane ──────────────────────────────────
  const body = document.createElement('div');
  body.style.cssText = 'flex:1;min-height:0;display:flex;overflow:hidden;';

  // Left Sidebar (232px)
  const sidebar = document.createElement('aside');
  sidebar.style.cssText =
    'width:232px;flex:none;background:linear-gradient(180deg,#14171b,#0d0f12);border-right:1px solid #000;padding:14px 10px;display:flex;flex-direction:column;gap:4px;';

  for (const [tabId, lbl] of [
    ['dev', 'Dispositivo e latência'],
    ['in', 'Entrada e calibração'],
    ['perf', 'Desempenho'],
  ] as const) {
    const active = store.audioTab === tabId;
    const b = makeChipButton('ui.subtab', lbl, active, () => {
      store.audioTab = tabId;
      store.emit();
    });
    b.style.cssText = `height:40px;width:100%;justify-content:flex-start;text-align:left;padding:0 12px;border-radius:6px;border:0;border-left:3px solid ${active ? '#ff8a3d' : 'transparent'};background:${active ? 'linear-gradient(90deg,rgba(255,138,61,.14),transparent)' : 'transparent'};color:${active ? '#ffd2b0' : '#c9ced4'};font-size:12.5px;font-weight:600;cursor:pointer;`;
    sidebar.appendChild(b);
  }

  const crtMotor = document.createElement('div');
  crtMotor.className = 'm-mono';
  crtMotor.style.cssText =
    'margin-top:auto;padding:10px 12px;border-radius:4px;border:1px solid #000;background:radial-gradient(ellipse at 50% 0%,#0d2a20 0%,#05140f 75%);box-shadow:inset 0 2px 8px rgba(0,0,0,.9);display:flex;flex-direction:column;gap:4px;color:#57ffb0;text-shadow:0 0 6px rgba(87,255,176,.6);';
  crtMotor.innerHTML = `
    <span style="font-size:10px">MOTOR</span>
    <span style="font-size:12px">${aud.mode === 'wasapi_exclusive' ? 'WASAPI exclusivo' : aud.backend.toUpperCase()} · ${srKhz} kHz</span>
    <span style="font-size:12px">período ${frames} / ${driverMs} ms</span>
    <span style="font-size:12px">xruns ${aud.underruns ?? 0}</span>
  `;
  sidebar.appendChild(crtMotor);

  // Right Content Pane
  const pane = document.createElement('div');
  pane.style.cssText =
    'flex:1;min-width:0;padding:20px 22px;display:flex;flex-direction:column;gap:16px;overflow:hidden;background:radial-gradient(ellipse at 60% 0%,rgba(61,216,255,.05),transparent 50%);';
  const aPanelCss =
    'border-radius:8px;background:repeating-linear-gradient(90deg,rgba(255,255,255,.02) 0 1px,transparent 1px 3px),linear-gradient(180deg,#1d2025,#121418);box-shadow:inset 0 1px 0 rgba(255,255,255,.07),0 4px 10px rgba(0,0,0,.5),0 0 0 1px #000;';

  if (store.audioTab === 'dev') {
    const devCard = document.createElement('div');
    devCard.style.cssText =
      'border-radius:8px;background:#121417;border:1px solid #1f242b;padding:14px 16px;display:flex;flex-direction:column;gap:12px;';
    devCard.innerHTML = '<span class="m-lbl" style="font-size:10px;letter-spacing:.12em">DISPOSITIVO DE ÁUDIO &amp; BUFFER</span>';

    const makeRow = (label: string, el: HTMLElement) => {
      const r = document.createElement('div');
      r.style.cssText = 'display:grid;grid-template-columns:160px 1fr;align-items:center;gap:12px;';
      const l = document.createElement('span');
      l.style.cssText = 'font-size:11px;color:#aeb4bc;';
      l.textContent = label;
      r.append(l, el);
      return r;
    };

    const selStyle =
      'height:30px;border-radius:5px;border:1px solid #2a2f37;background:#090b0d;color:#e6eaef;padding:0 10px;font-size:12px;';

    const driverSel = document.createElement('select');
    driverSel.style.cssText = selStyle;
    driverSel.dataset.controlId = 'audio.device_apply';
    const drivers: Array<[string, string, boolean, string]> = [
      ['wasapi_shared', 'WASAPI Shared', Boolean(caps.wasapi_shared), ''],
      ['wasapi_shared_low_latency', 'WASAPI Shared Low-Latency', Boolean(caps.wasapi_shared_low_latency), ''],
      ['wasapi_exclusive', 'WASAPI Exclusive (Spec 036)', Boolean(caps.wasapi_exclusive), 'Indisponível (Spec 036)'],
      ['asio', 'ASIO (Spec 036)', Boolean(caps.asio), 'Indisponível (Spec 036)'],
    ];
    for (const [val, text, supported, titleHint] of drivers) {
      const opt = document.createElement('option');
      opt.value = val;
      opt.textContent = text;
      opt.disabled = !supported;
      if (titleHint) opt.title = titleHint;
      if (pending.backend === val) opt.selected = true;
      driverSel.appendChild(opt);
    }
    driverSel.addEventListener('change', () => {
      pending.backend = driverSel.value;
    });
    devCard.appendChild(makeRow('Driver / Backend', driverSel));

    const inDevs = store.devices.filter((d) => d.is_input);
    const outDevs = store.devices.filter((d) => d.is_output);

    const buildDeviceSel = (devs: typeof store.devices, current: string, onPick: (v: string) => void) => {
      const sel = document.createElement('select');
      sel.style.cssText = selStyle;
      sel.dataset.controlId = 'audio.device_apply';
      if (devs.length === 0) {
        const opt = document.createElement('option');
        opt.value = '';
        opt.textContent = 'nenhum dispositivo de entrada encontrado';
        sel.appendChild(opt);
      } else {
        for (const d of devs) {
          const opt = document.createElement('option');
          opt.value = d.name;
          opt.textContent = d.name;
          if (current === d.name) opt.selected = true;
          sel.appendChild(opt);
        }
      }
      sel.addEventListener('change', () => onPick(sel.value));
      return sel;
    };

    devCard.appendChild(
      makeRow(
        'Dispositivo Entrada',
        buildDeviceSel(inDevs, pending.inputDevice, (v) => {
          pending.inputDevice = v;
        })
      )
    );
    devCard.appendChild(
      makeRow(
        'Dispositivo Saída',
        buildDeviceSel(outDevs, pending.outputDevice, (v) => {
          pending.outputDevice = v;
        })
      )
    );

    const srSel = document.createElement('select');
    srSel.style.cssText = selStyle;
    srSel.dataset.controlId = 'audio.device_apply';
    for (const sr of [44100, 48000, 88200, 96000]) {
      const o = document.createElement('option');
      o.value = String(sr);
      o.textContent = `${sr} Hz`;
      if (pending.sampleRate === sr) o.selected = true;
      srSel.appendChild(o);
    }
    srSel.addEventListener('change', () => {
      pending.sampleRate = Number(srSel.value);
    });
    devCard.appendChild(makeRow('Sample Rate', srSel));

    const bufSel = document.createElement('select');
    bufSel.style.cssText = selStyle;
    bufSel.dataset.controlId = 'audio.device_apply';
    for (const sz of [32, 64, 128, 256, 512]) {
      const ms = ((sz / pending.sampleRate) * 1000).toFixed(2);
      const o = document.createElement('option');
      o.value = String(sz);
      o.textContent = `${sz} amostras (${ms} ms)`;
      if (pending.bufferSize === sz) o.selected = true;
      bufSel.appendChild(o);
    }
    bufSel.addEventListener('change', () => {
      pending.bufferSize = Number(bufSel.value);
    });
    devCard.appendChild(makeRow('Tamanho de Buffer', bufSel));

    if (store.devices.length === 0) {
      const emptyBanner = document.createElement('div');
      emptyBanner.style.cssText =
        'padding:8px 12px;border-radius:6px;background:rgba(255,106,74,.12);border:1px solid rgba(255,106,74,.35);color:#ff9e8a;font-size:11px;';
      emptyBanner.textContent =
        'Nenhum dispositivo encontrado no host. Verifique cabo USB/driver ou clique em Reconectar Dispositivo.';
      devCard.appendChild(emptyBanner);
    }

    const actions = document.createElement('div');
    actions.style.cssText = 'display:flex;gap:8px;margin-top:4px;';
    const refreshAudio = async () => {
      const st = await store.transport.getAudioStatus();
      if (st && typeof st === 'object') Object.assign(store.audioStatus, st);
      store.emit('status');
    };
    actions.append(
      makeChipButton('audio.measure_latency', 'Medir Latência Round-Trip', false, async () => {
        try {
          await store.transport.measureLatency();
          await refreshAudio();
        } catch {
          store.emit('status');
        }
      }),
      makeChipButton('ui.audio_reconnect', 'Reconectar Dispositivo', false, () => void store.reconnectAudioDevice())
    );
    const rtlRow = document.createElement('div');
    rtlRow.className = 'm-mono';
    rtlRow.style.cssText = 'font-size:11px;color:#9aa3ad;';
    rtlRow.textContent = `Ida-e-volta: ${rtLabel}`;
    devCard.append(rtlRow, actions);
    pane.appendChild(devCard);
  } else if (store.audioTab === 'in') {
    const src = str(store, 'main.input_source', 'guitar');
    const boost = num(store, 'main.input_boost', 2.5);
    const inDb = store.telemetry.input_peak_db ?? -18.2;
    const litCount = Math.max(0, Math.min(30, Math.round((inDb + 60) / 2)));

    // Top Card: Fonte do Motor + Boost + Nivel de Entrada
    const topCard = document.createElement('div');
    topCard.style.cssText = `${aPanelCss}height:126px;flex:none;box-sizing:border-box;padding:14px 18px;display:flex;align-items:center;gap:22px;`;

    const srcCol = document.createElement('div');
    srcCol.style.cssText = 'display:flex;flex-direction:column;gap:6px;';
    srcCol.innerHTML = '<span class="m-lbl" style="font-size:10px">Fonte do motor</span>';
    const srcBtns = document.createElement('div');
    srcBtns.style.cssText = 'display:flex;gap:4px;';
    for (const [val, lbl] of [
      ['guitar', 'Guitarra'],
      ['test_riff', 'Riff de teste'],
      ['di_file', 'Arquivo DI'],
    ] as const) {
      const b = makeChipButton('main.input_source', lbl, src === val, () => store.setControl('main.input_source', val));
      b.style.cssText = `height:34px;padding:0 12px;border-radius:5px;font-size:11.5px;font-weight:600;border:1px solid ${src === val ? '#ff8a3d' : '#000'};background:${src === val ? 'linear-gradient(180deg,#3a2414,#22140a)' : 'linear-gradient(180deg,#30353c,#1a1d21)'};color:${src === val ? '#ffd2b0' : '#d6dade'};box-shadow:${src === val ? 'inset 0 1px 0 rgba(255,255,255,.1),0 0 10px rgba(255,138,61,.35)' : 'inset 0 1px 0 rgba(255,255,255,.12)'};`;
      srcBtns.appendChild(b);
    }
    srcCol.appendChild(srcBtns);

    const boostCol = document.createElement('div');
    boostCol.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:2px;';
    const knob = createKnob({
      controlId: 'main.input_boost',
      label: 'Boost de Entrada',
      value: boost,
      min: 0.1,
      max: 8.0,
      step: 0.1,
      defaultValue: 1.0,
      skinClass: 'k-audio',
      sizePx: 60,
      onChange: (v) => store.setControl('main.input_boost', v),
      onDragState: (d) => store.setDraggingKnob(d),
    });
    const bLbl = document.createElement('span');
    bLbl.className = 'm-lbl';
    bLbl.style.cssText = 'font-size:9px;';
    bLbl.textContent = 'Boost';
    const bVal = document.createElement('span');
    bVal.className = 'm-mono';
    bVal.style.cssText = 'font-size:12px;font-weight:700;color:#e8eaed;';
    bVal.textContent = `${boost.toFixed(1)}×`;
    boostCol.append(knob, bLbl, bVal);

    const lvlCol = document.createElement('div');
    lvlCol.style.cssText = 'flex:1;min-width:0;display:flex;flex-direction:column;gap:6px;';
    lvlCol.innerHTML = `<div style="display:flex;align-items:center;gap:8px"><span class="m-lbl" style="font-size:10px;flex:1">Nível de entrada</span><span style="display:flex;align-items:center;gap:6px;font-size:10px;font-weight:700;color:#a3abb4"><span style="width:9px;height:9px;border-radius:50%;background:${(aud.adc_clips ?? 0) > 0 ? '#ff3b2f' : '#2a1816'};box-shadow:${(aud.adc_clips ?? 0) > 0 ? '0 0 6px #ff3b2f' : 'none'}"></span>CLIP ADC</span></div>`;
    const ledStrip = document.createElement('div');
    ledStrip.style.cssText = 'display:flex;gap:2px;padding:4px;border-radius:4px;background:#050505;position:relative;';
    for (let i = 0; i < 30; i++) {
      const db = -60 + i * 2;
      const on = i < litCount;
      const c = db >= -3 ? '#ff3b2f' : db >= -12 ? '#ffc400' : '#29c46a';
      const target = db >= -18 && db <= -6;
      const s = document.createElement('span');
      s.style.cssText = `width:7px;height:16px;border-radius:1px;flex:1;background:${on ? c : target ? '#14261b' : '#16181b'};box-shadow:${on ? `0 0 4px ${c}` : 'none'};`;
      ledStrip.appendChild(s);
    }
    const scaleRow = document.createElement('div');
    scaleRow.className = 'm-mono';
    scaleRow.style.cssText = 'display:flex;justify-content:space-between;font-size:9.5px;color:#9aa3ad;';
    scaleRow.innerHTML = '<span>−60</span><span style="color:#57ffb0">alvo −12 dBFS</span><span>0 dBFS</span>';
    lvlCol.append(ledStrip, scaleRow);

    topCard.append(srcCol, boostCol, lvlCol);
    pane.appendChild(topCard);

    // Bottom Card: Assistente de Calibracao
    const calCard = document.createElement('div');
    calCard.style.cssText = `${aPanelCss}padding:16px 18px;display:flex;flex-direction:column;gap:14px;`;
    calCard.innerHTML = `
      <div style="display:flex;align-items:center;gap:10px">
        <span class="m-osw" style="font-size:16px;font-weight:600;letter-spacing:.06em;flex:1">ASSISTENTE DE CALIBRAÇÃO</span>
        <span class="m-mono" style="font-size:11px;color:#a3abb4">passo 1 de 3</span>
      </div>
      <div style="display:flex;gap:8px">
        <div style="flex:1;display:flex;flex-direction:column;gap:6px"><div style="height:5px;border-radius:3px;background:#3a404a"></div><span style="font-size:11px;font-weight:600;color:#e8eaed">1 · Silêncio (3 s)</span></div>
        <div style="flex:1;display:flex;flex-direction:column;gap:6px"><div style="height:5px;border-radius:3px;background:#1a1d22"></div><span style="font-size:11px;font-weight:600;color:#a3abb4">2 · Tocar forte (5 s)</span></div>
        <div style="flex:1;display:flex;flex-direction:column;gap:6px"><div style="height:5px;border-radius:3px;background:#1a1d22"></div><span style="font-size:11px;font-weight:600;color:#a3abb4">3 · Resultado</span></div>
      </div>
    `;

    const calStepBox = document.createElement('div');
    calStepBox.style.cssText = 'display:flex;align-items:center;gap:18px;';
    calStepBox.innerHTML = `
      <div style="width:92px;height:92px;border-radius:50%;background:conic-gradient(#57ffb0 0deg,#1a1d22 0);display:flex;align-items:center;justify-content:center;flex-shrink:0">
        <div style="width:74px;height:74px;border-radius:50%;background:#0d0f12;display:flex;align-items:center;justify-content:center">
          <span class="m-mono" style="font-size:22px;font-weight:700;color:#57ffb0">—</span>
        </div>
      </div>
      <div style="display:flex;flex-direction:column;gap:6px;flex:1">
        <span style="font-size:15px;font-weight:600">Silêncio: abafe as cordas</span>
        <span style="font-size:12px;color:#a3abb4;line-height:1.5">Guitarra ligada, volume no máximo, sem tocar — medindo piso de ruído e hum (50/60 Hz).</span>
      </div>
    `;
    const calBtn = makeChipButton('audio.calibrate_input', 'Iniciar', false, async () => {
      try {
        await store.transport.calibrateInput({ action: 'start' });
        const st = await store.transport.getAudioStatus();
        if (st && typeof st === 'object') Object.assign(store.audioStatus, st);
        store.emit('status');
      } catch {
        store.emit('status');
      }
    });
    calBtn.style.cssText =
      'height:40px;padding:0 18px;border-radius:6px;background:linear-gradient(180deg,#ffa25c,#e46d1f);border:1px solid #6b2c05;color:#1a0b01;font-weight:700;font-size:12px;cursor:pointer;';
    calStepBox.appendChild(calBtn);
    calCard.appendChild(calStepBox);

    const tips = document.createElement('div');
    tips.style.cssText = 'font-size:11px;color:#a3abb4;line-height:1.6;padding:0 4px;';
    tips.textContent =
      'Dicas: desligue aprimoramentos, AGC e supressão de ruído do microfone no Windows (ou use WASAPI exclusivo/RAW); prefira a entrada de linha quando houver; um pedal com buffer antes do PC reduz a perda de agudos.';

    pane.append(calCard, tips);
  } else {
    const perfCard = document.createElement('div');
    perfCard.style.cssText =
      'border-radius:8px;background:#121417;border:1px solid #1f242b;padding:16px;display:flex;flex-direction:column;gap:12px;';
    perfCard.innerHTML = '<span class="m-lbl" style="font-size:10px;letter-spacing:.12em">DESEMPENHO &amp; OVERSAMPLING DSP</span>';
    const curOs = str(store, 'engine.oversampling', '4x');
    const osRow = document.createElement('div');
    osRow.style.cssText = 'display:flex;gap:6px;';
    for (const os of ['2x', '4x', '8x']) {
      osRow.appendChild(makeChipButton('topbar.oversampling', os.toUpperCase(), curOs === os, () => store.setControl('topbar.oversampling', os)));
    }
    const loadIrDi = makeChipButton('cab.ir_load', 'Abrir Arquivo WAV DI...', false, async () => {
      const inp = document.createElement('input');
      inp.type = 'file';
      inp.accept = '.wav';
      inp.click();
    });
    perfCard.append(osRow, loadIrDi);
    pane.appendChild(perfCard);
  }

  body.append(sidebar, pane);
  root.appendChild(body);

  // ── 3. Footer Bar (Cancelar / Aplicar) ─────────────────────────────────────
  const foot = document.createElement('div');
  foot.style.cssText =
    'height:54px;flex:none;display:flex;align-items:center;justify-content:flex-end;gap:8px;padding:0 18px;border-top:1px solid #000;background:#0a0c0e;';

  const cancelBtn = makeChipButton('ui.nav_screen', 'Cancelar', false, () => store.setScreen('main'));
  cancelBtn.style.cssText =
    'height:34px;padding:0 16px;border-radius:5px;border:1px solid #000;background:linear-gradient(180deg,#30353c,#1a1d21);color:#d6dade;box-shadow:inset 0 1px 0 rgba(255,255,255,.12);font-size:11.5px;font-weight:600;display:flex;align-items:center;';

  const applyBtn = makeChipButton('audio.device_apply', 'Aplicar', false, async () => {
    try {
      await store.transport.setAudioDevice({
        backend: pending.backend.startsWith('wasapi') ? 'wasapi' : pending.backend,
        sample_rate: pending.sampleRate,
        buffer_size: pending.bufferSize,
        input_device: pending.inputDevice || undefined,
        output_device: pending.outputDevice || undefined,
      });
      const st = await store.transport.getAudioStatus();
      if (st && typeof st === 'object') Object.assign(store.audioStatus, st);
      store.emit('status');
    } catch {
      // keep diagnostic state
    }
    store.setScreen('main');
  });
  applyBtn.style.cssText =
    'height:32px;padding:0 18px;border-radius:6px;border:0;background:#ff7d26;color:#111;font-size:12px;font-weight:700;';

  foot.append(cancelBtn, applyBtn);
  root.appendChild(foot);
  return root;
}
