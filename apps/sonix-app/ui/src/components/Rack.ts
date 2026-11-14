import { createKnob } from './Knob';
import type { AppStore } from '../store';

export type RackModelId = 'comp1176' | 're201' | 'plate';

export function createRackElement(model: RackModelId, store: AppStore): HTMLElement {
  const heights: Record<RackModelId, number> = { comp1176: 130, re201: 190, plate: 110 };
  const root = document.createElement('div');
  root.style.cssText = `position:relative;width:560px;height:${heights[model]}px;user-select:none`;

  if (model === 'comp1176') {
    const on = Boolean(store.getParam('comp_2.enabled', false));
    root.style.cssText +=
      ';border-radius:3px;background:repeating-linear-gradient(90deg,rgba(255,255,255,.02) 0 1px,transparent 1px 3px),linear-gradient(180deg,#1c1c1c,#0b0b0b);box-shadow:0 12px 18px rgba(0,0,0,.6),inset 0 1px 0 rgba(255,255,255,.12)';
    const title = document.createElement('span');
    title.className = 'osw';
    title.textContent = 'FET 76 · LIMITING AMPLIFIER';
    title.style.cssText = 'position:absolute;left:28px;top:7px;font-size:10px;font-weight:600;letter-spacing:.18em;color:#d9d9d9';
    root.appendChild(title);

    const knobsRow = document.createElement('div');
    knobsRow.style.cssText = 'position:absolute;left:28px;top:24px;display:flex;gap:14px;align-items:flex-start';

    const bigDefs = [
      { id: 'comp_2.input', lbl: 'INPUT', min: 0, max: 10, def: 5 },
      { id: 'comp_2.output', lbl: 'OUTPUT', min: 0, max: 10, def: 5 },
    ];
    for (const k of bigDefs) {
      const col = document.createElement('div');
      col.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:3px';
      const knob = createKnob({
        controlId: k.id,
        label: `1176 ${k.lbl}`,
        value: Number(store.getParam(k.id, k.def)),
        min: k.min,
        max: k.max,
        step: 0.1,
        defaultValue: k.def,
        skinClass: 'r-big',
        sizePx: 62,
        onChange: (v) => store.setControl(k.id, v),
      });
      const lbl = document.createElement('span');
      lbl.textContent = k.lbl;
      lbl.style.cssText = 'font-size:7.5px;font-weight:800;letter-spacing:.12em;color:#e6e6e6';
      col.append(knob, lbl);
      knobsRow.appendChild(col);
    }

    const smallCol = document.createElement('div');
    smallCol.style.cssText = 'display:flex;flex-direction:column;gap:8px';
    for (const k of [
      { id: 'comp_2.attack', lbl: 'ATTACK', def: 4 },
      { id: 'comp_2.release', lbl: 'RELEASE', def: 4 },
    ]) {
      const r = document.createElement('div');
      r.style.cssText = 'display:flex;align-items:center;gap:6px';
      const knob = createKnob({
        controlId: k.id,
        label: `1176 ${k.lbl}`,
        value: Number(store.getParam(k.id, k.def)),
        min: 1,
        max: 7,
        step: 0.1,
        defaultValue: k.def,
        skinClass: 'r-small',
        sizePx: 30,
        onChange: (v) => store.setControl(k.id, v),
      });
      const lbl = document.createElement('span');
      lbl.textContent = k.lbl;
      lbl.style.cssText = 'font-size:7px;font-weight:800;letter-spacing:.1em;color:#e6e6e6';
      r.append(knob, lbl);
      smallCol.appendChild(r);
    }
    knobsRow.appendChild(smallCol);
    root.appendChild(knobsRow);

    // Ratio buttons (20, 12, 8, 4, ALL)
    const ratioCol = document.createElement('div');
    ratioCol.style.cssText = 'position:absolute;left:282px;top:22px;display:flex;flex-direction:column;gap:2px;align-items:center';
    const rHdr = document.createElement('span');
    rHdr.textContent = 'RATIO';
    rHdr.style.cssText = 'font-size:7px;font-weight:800;letter-spacing:.14em;color:#e6e6e6';
    ratioCol.appendChild(rHdr);
    const curRatio = String(store.getParam('comp_2.ratio', '4'));
    for (const r of ['20', '12', '8', '4', 'all']) {
      const b = document.createElement('button');
      b.className = 'rbtn';
      b.textContent = r.toUpperCase();
      b.setAttribute('data-control-id', 'comp_2.ratio');
      const pressed = curRatio === r;
      b.setAttribute('aria-pressed', String(pressed));
      b.style.cssText = `width:34px;height:15px;background:${pressed ? '#fffef6' : '#c6c6c6'};color:#111`;
      b.addEventListener('click', () => store.setControl('comp_2.ratio', r));
      ratioCol.appendChild(b);
    }
    root.appendChild(ratioCol);

    // Meter selector buttons (visualOnly)
    const meterCol = document.createElement('div');
    meterCol.style.cssText = 'position:absolute;left:328px;top:22px;display:flex;flex-direction:column;gap:2px;align-items:center';
    const mHdr = document.createElement('span');
    mHdr.textContent = 'METER';
    mHdr.style.cssText = 'font-size:7px;font-weight:800;letter-spacing:.14em;color:#e6e6e6';
    meterCol.appendChild(mHdr);
    for (const m of ['GR', '+8', '+4', 'OFF'] as const) {
      const b = document.createElement('button');
      b.className = 'rbtn';
      b.textContent = m;
      b.setAttribute('data-control-id', 'comp_2.vu_mode');
      const pressed = store.vuMode === m;
      b.setAttribute('aria-pressed', String(pressed));
      b.style.cssText = `width:34px;height:15px;background:${pressed ? '#fffef6' : '#c6c6c6'};color:#111`;
      b.addEventListener('click', () => {
        store.vuMode = m;
        store.emit();
      });
      meterCol.appendChild(b);
    }
    root.appendChild(meterCol);

    // VU Meter
    const grDb = Math.abs(store.telemetry.gain_reduction_db ?? 0);
    const needleDeg = store.vuMode === 'OFF' ? -50 : Math.max(-50, Math.min(45, 45 - grDb * 4.5));
    const vu = document.createElement('div');
    vu.className = 'vu';
    vu.style.cssText = `position:absolute;left:380px;top:18px;width:144px;height:88px;filter:${on ? 'none' : 'brightness(.55)'}`;
    vu.innerHTML = `<svg viewBox="0 0 138 86" style="width:100%;height:100%" aria-hidden="true">
      <path d="M18 66 A58 58 0 0 1 120 66" stroke="#2a2214" stroke-width="1.1" fill="none"/>
      <path d="M95 30 A58 58 0 0 1 120 66" stroke="#b5241a" stroke-width="3" fill="none"/>
      <text x="69" y="52" font-family="Oswald,sans-serif" font-size="8" fill="#2a2214" text-anchor="middle">${store.vuMode === 'GR' ? 'GAIN REDUCTION' : 'VU'}</text>
      <g transform="rotate(${needleDeg.toFixed(1)} 69 84)"><path d="M69 84 L69 18" stroke="#111" stroke-width="1.4" stroke-linecap="round"/></g>
      <circle cx="69" cy="84" r="6" fill="#1a1a1a"/>
    </svg>`;
    root.appendChild(vu);

    const inBtn = document.createElement('button');
    inBtn.className = 'rbtn';
    inBtn.setAttribute('data-control-id', 'comp_2.enabled');
    inBtn.setAttribute('aria-pressed', String(on));
    inBtn.style.cssText =
      'position:absolute;left:28px;bottom:8px;height:18px;padding:0 7px;display:flex;align-items:center;gap:5px;background:#1d1d1d;color:#ddd';
    inBtn.innerHTML = `<span style="width:6px;height:6px;border-radius:50%;background:${on ? '#ff3b1f' : '#3a1210'};box-shadow:${on ? '0 0 7px #ff3b1f' : 'none'}"></span>IN`;
    inBtn.addEventListener('click', () => store.setControl('comp_2.enabled', !on));
    root.appendChild(inBtn);

    const grSpan = document.createElement('span');
    grSpan.className = 'mono';
    grSpan.textContent = `GR -${grDb.toFixed(1)} dB`;
    grSpan.style.cssText = 'position:absolute;left:88px;bottom:10px;font-size:9px;color:#b9b9b9';
    root.appendChild(grSpan);
    return root;
  }

  if (model === 're201') {
    const on = Boolean(store.getParam('dly_1.enabled', false));
    root.style.cssText +=
      ';border-radius:8px;background:linear-gradient(180deg,#3d5243,#25352a);box-shadow:0 14px 20px rgba(0,0,0,.6),inset 0 2px 0 rgba(255,255,255,.18);padding:9px';
    const inner = document.createElement('div');
    inner.style.cssText =
      'position:relative;width:100%;height:100%;border-radius:4px;background:linear-gradient(180deg,#1b1b1b,#090909);box-shadow:inset 0 0 0 2px #a9aeb3';

    inner.innerHTML = `
      <span class="osw" style="position:absolute;left:12px;top:6px;font-size:14px;font-weight:700;letter-spacing:.1em;color:#ececec">TAPE ECHO 201</span>
      <span style="position:absolute;left:12px;top:24px;font-size:7px;font-weight:700;letter-spacing:.18em;color:#9da3a8">ECHO · CHORUS · REVERB</span>
    `;

    // Mode Selector 1..12
    const curMode = String(store.getParam('dly_1.mode', '4'));
    const modeBox = document.createElement('div');
    modeBox.style.cssText = 'position:absolute;left:130px;top:10px;width:155px;height:150px';
    const mHdr = document.createElement('span');
    mHdr.textContent = `MODE ${curMode} · RE-201`;
    mHdr.style.cssText = 'position:absolute;left:0;right:0;top:0;text-align:center;font-size:7px;font-weight:800;letter-spacing:.14em;color:#ddd';
    modeBox.appendChild(mHdr);

    for (let i = 0; i < 12; i++) {
      const n = String(i + 1);
      const a = ((-135 + i * (270 / 11)) * Math.PI) / 180;
      const R = 54;
      const x = Math.round(77 + R * Math.sin(a));
      const y = Math.round(78 - R * Math.cos(a));
      const mBtn = document.createElement('button');
      mBtn.className = 'rbtn';
      mBtn.textContent = n;
      mBtn.setAttribute('data-control-id', 'dly_1.mode');
      mBtn.setAttribute('aria-pressed', String(curMode === n));
      mBtn.style.cssText = `position:absolute;left:${x - 10}px;top:${y - 8}px;width:20px;height:16px;font-size:8px;background:${curMode === n ? '#e8e8e8' : '#181818'};color:${curMode === n ? '#111' : '#d6d6d6'}`;
      mBtn.addEventListener('click', () => store.setControl('dly_1.mode', n));
      modeBox.appendChild(mBtn);
    }
    inner.appendChild(modeBox);

    // 6 RE-201 Knobs in 2 rows
    const kDefs = [
      { id: 'dly_1.repeat_rate', lbl: 'RATE', x: 300, y: 20, def: 5 },
      { id: 'dly_1.intensity', lbl: 'INTENSITY', x: 382, y: 20, def: 5 },
      { id: 'dly_1.echo_volume', lbl: 'ECHO VOL', x: 464, y: 20, def: 5 },
      { id: 'dly_1.bass', lbl: 'BASS', x: 300, y: 94, def: 5 },
      { id: 'dly_1.treble', lbl: 'TREBLE', x: 382, y: 94, def: 5 },
      { id: 'dly_1.reverb_volume', lbl: 'REV VOL', x: 464, y: 94, def: 2.5 },
    ];
    for (const k of kDefs) {
      const c = document.createElement('div');
      c.style.cssText = `position:absolute;left:${k.x}px;top:${k.y}px;width:62px;display:flex;flex-direction:column;align-items:center;gap:3px`;
      const knob = createKnob({
        controlId: k.id,
        label: `RE-201 ${k.lbl}`,
        value: Number(store.getParam(k.id, k.def)),
        min: 0,
        max: 10,
        step: 0.1,
        defaultValue: k.def,
        skinClass: 'r-small',
        sizePx: 36,
        onChange: (v) => store.setControl(k.id, v),
      });
      const lbl = document.createElement('span');
      lbl.textContent = k.lbl;
      lbl.style.cssText = 'font-size:7px;font-weight:800;color:#d8d8d8;letter-spacing:.08em';
      c.append(knob, lbl);
      inner.appendChild(c);
    }

    // VU & Power ON button
    const onBtn = document.createElement('button');
    onBtn.className = 'rbtn';
    onBtn.setAttribute('data-control-id', 'dly_1.enabled');
    onBtn.setAttribute('aria-pressed', String(on));
    onBtn.style.cssText =
      'position:absolute;left:14px;top:116px;height:18px;padding:0 7px;display:flex;align-items:center;gap:5px;background:#1d1d1d;color:#ddd';
    onBtn.innerHTML = `<span style="width:6px;height:6px;border-radius:50%;background:${on ? '#ff3b1f' : '#3a1210'};box-shadow:${on ? '0 0 7px #ff3b1f' : 'none'}"></span>ON`;
    onBtn.addEventListener('click', () => store.setControl('dly_1.enabled', !on));

    const spOn = Boolean(store.getParam('dly_1.spillover', true));
    const spBtn = document.createElement('button');
    spBtn.className = 'rbtn';
    spBtn.setAttribute('data-control-id', 'dly_1.spillover');
    spBtn.setAttribute('aria-pressed', String(spOn));
    spBtn.textContent = 'SPILL';
    spBtn.style.cssText = `position:absolute;left:64px;top:116px;height:18px;padding:0 7px;background:${spOn ? '#123a22' : '#1d1d1d'};color:${spOn ? '#29e07a' : '#888'}`;
    spBtn.addEventListener('click', () => store.setControl('dly_1.spillover', !spOn));

    inner.append(onBtn, spBtn);
    root.appendChild(inner);
    return root;
  }

  // Plate Reverb
  const on = Boolean(store.getParam('rev_1.enabled', false));
  const spill = Boolean(store.getParam('rev_1.spillover', true));
  const decay = Number(store.getParam('rev_1.decay_s', 2.5));
  root.style.cssText +=
    ';border-radius:3px;background:repeating-linear-gradient(90deg,rgba(255,255,255,.04) 0 1px,transparent 1px 3px),linear-gradient(180deg,#d8dce0 0%,#aeb4ba 50%,#8c9298 100%);box-shadow:0 12px 18px rgba(0,0,0,.6)';

  const lcd = document.createElement('div');
  lcd.style.cssText =
    'position:absolute;left:28px;top:24px;width:104px;height:52px;border-radius:3px;background:#140600;box-shadow:inset 0 2px 6px #000,0 0 0 2px #2a2d31;display:flex;align-items:center;justify-content:center';
  lcd.innerHTML = `<span class="mono" style="font-size:24px;font-weight:700;color:#ff7a1a;text-shadow:0 0 8px rgba(255,110,20,.8)">${on ? decay.toFixed(1) + 's' : '--'}</span>`;
  root.appendChild(lcd);

  const pRow = document.createElement('div');
  pRow.style.cssText = 'position:absolute;left:156px;top:18px;display:flex;gap:16px';
  for (const k of [
    { id: 'rev_1.decay_s', lbl: 'DECAY', min: 0.1, max: 20, step: 0.1, def: 2.5 },
    { id: 'rev_1.predelay_ms', lbl: 'PRE-DLY', min: 0, max: 250, step: 1, def: 20 },
    { id: 'rev_1.damping', lbl: 'DAMPING', min: 0, max: 10, step: 0.1, def: 5 },
    { id: 'rev_1.mix', lbl: 'MIX', min: 0, max: 100, step: 1, def: 40 },
  ]) {
    const col = document.createElement('div');
    col.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:4px';
    const knob = createKnob({
      controlId: k.id,
      label: `Plate ${k.lbl}`,
      value: Number(store.getParam(k.id, k.def)),
      min: k.min,
      max: k.max,
      step: k.step,
      defaultValue: k.def,
      skinClass: 'r-alu',
      sizePx: 42,
      onChange: (v) => store.setControl(k.id, v),
    });
    const lbl = document.createElement('span');
    lbl.textContent = k.lbl;
    lbl.style.cssText = 'font-size:7px;font-weight:800;letter-spacing:.1em;color:#1b1d20';
    col.append(knob, lbl);
    pRow.appendChild(col);
  }
  root.appendChild(pRow);

  const btnCol = document.createElement('div');
  btnCol.style.cssText = 'position:absolute;right:24px;top:24px;display:flex;flex-direction:column;gap:8px;align-items:flex-end';

  const spBtn = document.createElement('button');
  spBtn.className = 'rbtn';
  spBtn.setAttribute('data-control-id', 'rev_1.spillover');
  spBtn.setAttribute('aria-pressed', String(spill));
  spBtn.style.cssText = 'height:20px;padding:0 8px;display:flex;align-items:center;gap:5px;background:#2a2d31;color:#eee';
  spBtn.innerHTML = `<span style="width:7px;height:7px;border-radius:50%;background:${spill ? '#29e07a' : '#123a22'}"></span>SPILLOVER`;
  spBtn.addEventListener('click', () => store.setControl('rev_1.spillover', !spill));

  const enBtn = document.createElement('button');
  enBtn.className = 'rbtn';
  enBtn.setAttribute('data-control-id', 'rev_1.enabled');
  enBtn.setAttribute('aria-pressed', String(on));
  enBtn.style.cssText = 'height:20px;padding:0 8px;display:flex;align-items:center;gap:5px;background:#2a2d31;color:#eee';
  enBtn.innerHTML = `<span style="width:7px;height:7px;border-radius:50%;background:${on ? '#ff3b1f' : '#3a1210'}"></span>ENGAGE`;
  enBtn.addEventListener('click', () => store.setControl('rev_1.enabled', !on));

  btnCol.append(spBtn, enBtn);
  root.appendChild(btnCol);
  return root;
}
