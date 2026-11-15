import { createKnob } from './Knob.ts';
import type { AppStore } from '../store.ts';

export type RackModelId = 'comp1176' | 're201' | 'plate';

export function createRackElement(model: RackModelId, store: AppStore): HTMLElement {
  const heights: Record<RackModelId, number> = { comp1176: 130, re201: 190, plate: 110 };
  const root = document.createElement('div');
  root.style.cssText = `position:relative;width:560px;height:${heights[model]}px;user-select:none`;

  const addEars = (customBg?: string, holeInset = 18) => {
    for (const side of ['left', 'right'] as const) {
      const ear = document.createElement('div');
      ear.className = 'ear';
      ear.style.cssText = `${side}:0;${customBg ? `background:${customBg}` : ''}`;
      ear.innerHTML = `<span class="hole" style="top:${holeInset}px"></span><span class="hole" style="bottom:${holeInset}px"></span>`;
      root.appendChild(ear);
    }
  };

  if (model === 'comp1176') {
    const on = Boolean(store.getParam('comp_2.enabled', false));
    root.style.cssText +=
      ';border-radius:3px;background:repeating-linear-gradient(90deg,rgba(255,255,255,.02) 0 1px,transparent 1px 3px),linear-gradient(180deg,#1c1c1c,#0b0b0b);box-shadow:0 12px 18px rgba(0,0,0,.6),inset 0 1px 0 rgba(255,255,255,.12)';
    addEars(undefined, 18);
    const title = document.createElement('span');
    title.className = 'osw';
    title.textContent = 'FET 76 · LIMITING AMPLIFIER';
    title.style.cssText = 'position:absolute;left:36px;top:8px;font-size:10px;font-weight:600;letter-spacing:.18em;color:#d9d9d9';
    root.appendChild(title);

    const knobsRow = document.createElement('div');
    knobsRow.style.cssText = 'position:absolute;left:36px;top:26px;display:flex;gap:18px;align-items:flex-start';

    const bigDefs = [
      { id: 'comp_2.input', lbl: 'INPUT', min: 0, max: 10, def: 5 },
      { id: 'comp_2.output', lbl: 'OUTPUT', min: 0, max: 10, def: 5 },
    ];
    for (const k of bigDefs) {
      const col = document.createElement('div');
      col.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:4px';
      const knob = createKnob({
        controlId: k.id,
        label: `1176 ${k.lbl}`,
        value: Number(store.getParam(k.id, k.def)),
        min: k.min,
        max: k.max,
        step: 0.1,
        defaultValue: k.def,
        skinClass: 'r-big',
        sizePx: 66,
        onChange: (v) => store.setControl(k.id, v),
        onDragState: (d) => store.setDraggingKnob(d),
      });
      const lbl = document.createElement('span');
      lbl.textContent = k.lbl;
      lbl.style.cssText = 'font-size:8px;font-weight:800;letter-spacing:.14em;color:#e6e6e6';
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
        sizePx: 32,
        onChange: (v) => store.setControl(k.id, v),
        onDragState: (d) => store.setDraggingKnob(d),
      });
      const lbl = document.createElement('span');
      lbl.textContent = k.lbl;
      lbl.style.cssText = 'font-size:7.5px;font-weight:800;letter-spacing:.12em;color:#e6e6e6';
      r.append(knob, lbl);
      smallCol.appendChild(r);
    }
    knobsRow.appendChild(smallCol);
    root.appendChild(knobsRow);

    // Ratio buttons (20, 12, 8, 4, ALL)
    const ratioCol = document.createElement('div');
    ratioCol.style.cssText = 'position:absolute;left:286px;top:24px;display:flex;flex-direction:column;gap:3px;align-items:center';
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
      const lit = pressed || curRatio === 'all';
      b.setAttribute('aria-pressed', String(pressed));
      b.style.cssText = `width:34px;height:15px;background:${lit ? 'linear-gradient(180deg,#fffef6,#d6d2c2)' : 'linear-gradient(180deg,#e8e8e8,#a9a9a9)'};color:#111;box-shadow:${lit ? '0 0 6px rgba(255,240,180,.8)' : 'none'}`;
      b.addEventListener('click', () => store.setControl('comp_2.ratio', r));
      ratioCol.appendChild(b);
    }
    root.appendChild(ratioCol);

    // Meter selector buttons (visualOnly)
    const meterCol = document.createElement('div');
    meterCol.style.cssText = 'position:absolute;left:334px;top:24px;display:flex;flex-direction:column;gap:3px;align-items:center';
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
      b.style.cssText = `width:34px;height:15px;background:${pressed ? 'linear-gradient(180deg,#fffef6,#d6d2c2)' : 'linear-gradient(180deg,#e8e8e8,#a9a9a9)'};color:#111`;
      b.addEventListener('click', () => {
        store.vuMode = m;
        store.emit();
      });
      meterCol.appendChild(b);
    }
    root.appendChild(meterCol);

    // VU Meter
    const grDb = on ? Math.abs(store.telemetry.gate_gr_db ?? 0) : 0;
    const needleDeg = store.vuMode === 'OFF' ? -50 : Math.max(-50, Math.min(45, 45 - grDb * 4.5));
    const meterUnit = store.vuMode === 'GR' ? 'GAIN REDUCTION' : 'VU';
    const vu = document.createElement('div');
    vu.className = 'vu';
    vu.style.cssText = `position:absolute;left:384px;top:20px;width:138px;height:86px;filter:${on ? 'none' : 'brightness(.55)'}`;
    vu.innerHTML = `<svg viewBox="0 0 138 86" style="position:absolute;inset:0;width:100%;height:100%" aria-hidden="true">
      <path d="M18 66 A58 58 0 0 1 120 66" stroke="#2a2214" stroke-width="1.1" fill="none"/>
      <path d="M95 30 A58 58 0 0 1 120 66" stroke="#b5241a" stroke-width="3" fill="none"/>
      <g stroke="#2a2214" stroke-width="1.1"><path d="M18 66 l-4 -3"/><path d="M33 39 l-3 -4"/><path d="M53 24 l-2 -5"/><path d="M69 20 v-6"/><path d="M86 24 l2 -5"/><path d="M105 39 l3 -4"/><path d="M120 66 l4 -3"/></g>
      <text x="69" y="52" font-family="Oswald, sans-serif" font-size="8" fill="#2a2214" text-anchor="middle" letter-spacing="1.5">${meterUnit}</text>
      <g transform="rotate(${needleDeg.toFixed(1)} 69 84)"><path d="M69 84 L69 18" stroke="#111" stroke-width="1.4" stroke-linecap="round"/></g>
      <circle cx="69" cy="84" r="6" fill="#1a1a1a"/>
    </svg>`;
    root.appendChild(vu);

    const inBtn = document.createElement('button');
    inBtn.className = 'rbtn';
    inBtn.setAttribute('data-control-id', 'comp_2.enabled');
    inBtn.setAttribute('aria-pressed', String(on));
    inBtn.style.cssText =
      'position:absolute;left:36px;bottom:8px;height:16px;padding:0 6px;display:flex;align-items:center;gap:5px;background:#1d1d1d;color:#ddd';
    inBtn.innerHTML = `<span style="width:6px;height:6px;border-radius:50%;background:${on ? '#ff3b1f' : '#3a1210'};box-shadow:${on ? '0 0 7px #ff3b1f' : 'none'}"></span>IN`;
    inBtn.addEventListener('click', () => store.setControl('comp_2.enabled', !on));
    root.appendChild(inBtn);

    const grSpan = document.createElement('span');
    grSpan.className = 'mono';
    grSpan.textContent = `GR -${grDb.toFixed(1)} dB`;
    grSpan.style.cssText = 'position:absolute;left:96px;bottom:10px;font-size:9px;color:#b9b9b9';
    root.appendChild(grSpan);
    return root;
  }

  if (model === 're201') {
    const on = Boolean(store.getParam('dly_1.enabled', false));
    root.style.cssText +=
      ';border-radius:8px;background:radial-gradient(circle at 1px 1px,rgba(255,255,255,.05) 1px,transparent 1.6px) 0 0/4px 4px,linear-gradient(180deg,#3d5243,#25352a);box-shadow:0 14px 20px rgba(0,0,0,.6),inset 0 2px 0 rgba(255,255,255,.18);padding:10px;box-sizing:border-box';
    const inner = document.createElement('div');
    inner.style.cssText =
      'position:relative;width:100%;height:100%;border-radius:4px;background:repeating-linear-gradient(90deg,rgba(255,255,255,.025) 0 1px,transparent 1px 3px),linear-gradient(180deg,#1b1b1b,#090909);box-shadow:inset 0 0 0 2px #a9aeb3,inset 0 0 0 3px #222';

    const curMode = String(store.getParam('dly_1.mode', '4'));
    const vuNeedle = on ? -10 : -50;
    inner.innerHTML = `
      <span class="osw" style="position:absolute;left:14px;top:8px;font-size:15px;font-weight:700;letter-spacing:.1em;color:#ececec">TAPE ECHO 201</span>
      <span style="position:absolute;left:14px;top:28px;font-size:7px;font-weight:700;letter-spacing:.2em;color:#9da3a8">ECHO · CHORUS · REVERB</span>
      <div class="vu" style="position:absolute;left:14px;top:46px;width:104px;height:66px;background:radial-gradient(ellipse at 50% 120%,#eaffd8 0%,#cde9a8 55%,#a9c97c 100%);filter:${on ? 'none' : 'brightness(.55)'}">
        <svg viewBox="0 0 104 66" style="position:absolute;inset:0;width:100%;height:100%" aria-hidden="true">
          <path d="M14 52 A44 44 0 0 1 90 52" fill="none" stroke="#1d2a14" stroke-width="1"/>
          <path d="M70 22 A44 44 0 0 1 90 52" fill="none" stroke="#b5241a" stroke-width="3"/>
          <text x="52" y="40" font-family="Oswald,sans-serif" font-size="7" fill="#1d2a14" text-anchor="middle">INPUT VU</text>
          <g transform="rotate(${vuNeedle} 52 64)"><path d="M52 64 L52 14" stroke="#111" stroke-width="1.3"/></g>
        </svg>
      </div>
      <span style="position:absolute;left:14px;top:120px;display:flex;align-items:center;gap:5px;font-size:7px;font-weight:800;letter-spacing:.14em;color:#ddd">
        <span style="width:8px;height:8px;border-radius:50%;background:${on ? '#3a1210' : '#3a1210'}"></span>PEAK
      </span>
    `;

    // Mode Selector 1..12
    const modeBox = document.createElement('div');
    modeBox.style.cssText = 'position:absolute;left:138px;top:6px;width:150px;height:150px';
    const mHdr = document.createElement('span');
    mHdr.textContent = 'MODE SELECTOR';
    mHdr.style.cssText = 'position:absolute;left:0;right:0;top:0;text-align:center;font-size:7px;font-weight:800;letter-spacing:.2em;color:#ddd';
    modeBox.appendChild(mHdr);

    const modeDial = document.createElement('div');
    modeDial.className = 'knob r-chick';
    const modeAng = -135 + ((Number(curMode) || 4) - 1) * (270 / 11);
    modeDial.style.cssText = 'position:absolute;left:47px;top:52px;width:56px;height:56px;pointer-events:none';
    modeDial.innerHTML = `<span class="sh"></span><span class="rotor" style="transform:rotate(${modeAng.toFixed(1)}deg)"><span class="a"></span><span class="b"></span><span class="p"></span></span>`;
    modeBox.appendChild(modeDial);

    const modeDescs = ['', 'cabeça 1', 'cabeça 2', 'cabeça 3', 'cabeças 2+3', 'cabeças 1+2', 'cabeças 1+3', 'cabeças 1+2+3', '1 + reverb', '2 + reverb', '3 + reverb', 'só reverb', '1+2+3 + reverb'];
    const modeSub = document.createElement('span');
    modeSub.className = 'mono';
    modeSub.style.cssText = 'position:absolute;left:0;right:0;bottom:2px;text-align:center;font-size:8px;color:#9da3a8;';
    modeSub.textContent = `MODO ${curMode} · ${modeDescs[Number(curMode)] ?? 'cabeças 2+3'}`;
    modeBox.appendChild(modeSub);

    for (let i = 0; i < 12; i++) {
      const n = String(i + 1);
      const a = ((-135 + i * (270 / 11)) * Math.PI) / 180;
      const R = 60;
      const x = Math.round(75 + R * Math.sin(a));
      const y = Math.round(80 - R * Math.cos(a));
      const mBtn = document.createElement('button');
      mBtn.className = 'rbtn';
      mBtn.textContent = i + 1 === 11 ? 'R' : n;
      mBtn.setAttribute('data-control-id', 'dly_1.mode');
      mBtn.setAttribute('aria-pressed', String(curMode === n));
      mBtn.style.cssText = `position:absolute;left:${x}px;top:${y}px;width:18px;height:14px;margin:-7px 0 0 -9px;padding:0;font-size:8px;border-color:transparent;background:${curMode === n ? '#e8e8e8' : 'transparent'};color:${curMode === n ? '#111' : '#d6d6d6'}`;
      mBtn.addEventListener('click', () => store.setControl('dly_1.mode', n));
      modeBox.appendChild(mBtn);
    }
    inner.appendChild(modeBox);

    // 6 RE-201 Knobs in 2x3 grid
    const kGrid = document.createElement('div');
    kGrid.style.cssText = 'position:absolute;left:300px;top:14px;right:12px;display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:10px 6px';
    const kDefs = [
      { id: 'dly_1.repeat_rate', lbl: 'REPEAT RATE', def: 5 },
      { id: 'dly_1.intensity', lbl: 'INTENSITY', def: 5 },
      { id: 'dly_1.echo_volume', lbl: 'ECHO VOL', def: 5 },
      { id: 'dly_1.bass', lbl: 'BASS', def: 5 },
      { id: 'dly_1.treble', lbl: 'TREBLE', def: 5 },
      { id: 'dly_1.reverb_volume', lbl: 'REVERB VOL', def: 2.5 },
    ];
    for (const k of kDefs) {
      const c = document.createElement('div');
      c.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:4px';
      const lbl = document.createElement('span');
      lbl.textContent = k.lbl;
      lbl.style.cssText = 'font-size:7px;font-weight:800;color:#e6e6e6;letter-spacing:.1em;white-space:nowrap';
      const knob = createKnob({
        controlId: k.id,
        label: `RE-201 ${k.lbl}`,
        value: Number(store.getParam(k.id, k.def)),
        min: 0,
        max: 10,
        step: 0.1,
        defaultValue: k.def,
        skinClass: 'r-chick',
        sizePx: 40,
        onChange: (v) => store.setControl(k.id, v),
        onDragState: (d) => store.setDraggingKnob(d),
      });
      c.append(lbl, knob);
      kGrid.appendChild(c);
    }
    inner.appendChild(kGrid);

    const onBtn = document.createElement('button');
    onBtn.className = 'rbtn';
    onBtn.setAttribute('data-control-id', 'dly_1.enabled');
    onBtn.setAttribute('aria-pressed', String(on));
    onBtn.style.cssText =
      'position:absolute;left:62px;top:116px;height:16px;padding:0 6px;display:flex;align-items:center;gap:5px;background:#1d1d1d;color:#ddd';
    onBtn.innerHTML = `<span style="width:6px;height:6px;border-radius:50%;background:${on ? '#ff3b1f' : '#3a1210'};box-shadow:${on ? '0 0 7px #ff3b1f' : 'none'}"></span>ON`;
    onBtn.addEventListener('click', () => store.setControl('dly_1.enabled', !on));

    inner.appendChild(onBtn);
    root.appendChild(inner);
    return root;
  }

  // Plate Reverb
  const on = Boolean(store.getParam('rev_1.enabled', false));
  const spill = Boolean(store.getParam('rev_1.spillover', true));
  const decay = Number(store.getParam('rev_1.decay_s', 2.5));
  root.style.cssText +=
    ';border-radius:3px;background:repeating-linear-gradient(90deg,rgba(255,255,255,.12) 0 1px,rgba(0,0,0,.03) 1px 2px,transparent 2px 4px),linear-gradient(180deg,#d7dbdf,#a9afb5 55%,#c6cbd0);box-shadow:0 12px 18px rgba(0,0,0,.6),inset 0 1px 0 rgba(255,255,255,.6)';
  addEars('linear-gradient(90deg,#7d838a,#b9bec3 40%,#737980)', 14);

  const plateHdr = document.createElement('span');
  plateHdr.className = 'osw';
  plateHdr.style.cssText = 'position:absolute;left:36px;top:8px;font-size:12px;font-weight:700;letter-spacing:.16em;color:#1b1d20';
  plateHdr.textContent = 'STUDIO PLATE · PR-140';
  root.appendChild(plateHdr);

  const lcd = document.createElement('div');
  lcd.style.cssText =
    'position:absolute;left:36px;top:30px;width:112px;height:52px;border-radius:3px;background:#140600;box-shadow:inset 0 2px 6px #000,0 0 0 2px #2a2d31;display:flex;align-items:center;justify-content:center';
  lcd.innerHTML = `<span class="mono" style="font-size:28px;font-weight:700;color:#ff7a1a;text-shadow:0 0 8px rgba(255,110,20,.8);letter-spacing:.04em">${on ? decay.toFixed(1) : '--'}</span>`;
  root.appendChild(lcd);

  const decaySub = document.createElement('span');
  decaySub.style.cssText = 'position:absolute;left:36px;top:86px;font-size:7px;font-weight:800;letter-spacing:.16em;color:#2a2d31';
  decaySub.textContent = 'DECAY TIME (s)';
  root.appendChild(decaySub);

  const pRow = document.createElement('div');
  pRow.style.cssText = 'position:absolute;left:170px;top:22px;display:flex;gap:20px';
  for (const k of [
    { id: 'rev_1.decay_s', lbl: 'DECAY', min: 0.1, max: 20, step: 0.1, def: 2.5, fmt: (v: number) => `${v.toFixed(1)} s` },
    { id: 'rev_1.predelay_ms', lbl: 'PRE-DELAY', min: 0, max: 250, step: 1, def: 20, fmt: (v: number) => `${v.toFixed(0)} ms` },
    { id: 'rev_1.damping', lbl: 'DAMPING', min: 0, max: 10, step: 0.1, def: 4, fmt: (v: number) => v.toFixed(1) },
    { id: 'rev_1.mix', lbl: 'MIX', min: 0, max: 100, step: 1, def: 30, fmt: (v: number) => `${Math.round(v > 1 ? v : v * 100)} %` },
  ]) {
    const valNum = Number(store.getParam(k.id, k.def));
    const col = document.createElement('div');
    col.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:4px';
    const knob = createKnob({
      controlId: k.id,
      label: `Plate ${k.lbl}`,
      value: valNum,
      min: k.min,
      max: k.max,
      step: k.step,
      defaultValue: k.def,
      skinClass: 'r-alu',
      sizePx: 44,
      onChange: (v) => store.setControl(k.id, v),
      onDragState: (d) => store.setDraggingKnob(d),
    });
    const lbl = document.createElement('span');
    lbl.textContent = k.lbl;
    lbl.style.cssText = 'font-size:7.5px;font-weight:800;letter-spacing:.12em;color:#1b1d20';
    const valSpan = document.createElement('span');
    valSpan.className = 'mono';
    valSpan.textContent = k.fmt(valNum);
    valSpan.style.cssText = 'font-size:8px;color:#3a3d42;font-weight:600';
    col.append(knob, lbl, valSpan);
    pRow.appendChild(col);
  }
  root.appendChild(pRow);

  const btnCol = document.createElement('div');
  btnCol.style.cssText = 'position:absolute;right:36px;top:26px;display:flex;flex-direction:column;gap:8px;align-items:flex-end';

  const spBtn = document.createElement('button');
  spBtn.className = 'rbtn';
  spBtn.setAttribute('data-control-id', 'rev_1.spillover');
  spBtn.setAttribute('aria-pressed', String(spill));
  spBtn.style.cssText = 'height:20px;padding:0 8px;display:flex;align-items:center;gap:6px;background:#2a2d31;color:#eee';
  spBtn.innerHTML = `<span style="width:7px;height:7px;border-radius:50%;background:${spill ? '#29e07a' : '#123a22'};box-shadow:${spill ? '0 0 6px #29e07a' : 'none'}"></span>SPILLOVER`;
  spBtn.addEventListener('click', () => store.setControl('rev_1.spillover', !spill));

  const enBtn = document.createElement('button');
  enBtn.className = 'rbtn';
  enBtn.setAttribute('data-control-id', 'rev_1.enabled');
  enBtn.setAttribute('aria-pressed', String(on));
  enBtn.style.cssText = 'height:20px;padding:0 8px;display:flex;align-items:center;gap:6px;background:#2a2d31;color:#eee';
  enBtn.innerHTML = `<span style="width:7px;height:7px;border-radius:50%;background:${on ? '#ff3b1f' : '#3a1210'};box-shadow:${on ? '0 0 7px #ff3b1f' : 'none'}"></span>ENGAGE`;
  enBtn.addEventListener('click', () => store.setControl('rev_1.enabled', !on));

  btnCol.append(spBtn, enBtn);
  root.appendChild(btnCol);
  return root;
}
