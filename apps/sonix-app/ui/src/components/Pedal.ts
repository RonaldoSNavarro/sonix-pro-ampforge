import { clampStep, createKnob, valueToRotationDeg } from './Knob';
import type { AppStore } from '../store';

export type PedalModelId =
  | 'ts808'
  | 'klon'
  | 'rat'
  | 'muff'
  | 'fuzzface'
  | 'ds1'
  | 'ns2'
  | 'ce2'
  | 'mxr10';

interface PedalKnobDef {
  controlId: string;
  label: string;
  min: number;
  max: number;
  step: number;
  def: number;
  x: number;
  y: number;
  px: number;
  cls: string;
  ink: string;
  dir?: 'column' | 'column-reverse';
  fs?: number;
}

export function createPedalElement(model: PedalModelId, store: AppStore): HTMLElement {
  const dims: Record<PedalModelId, { w: number; h: number; bg: string; rad: string; name: string }> = {
    ts808: { w: 130, h: 210, bg: 'linear-gradient(180deg,#5aa86b 0%,#3d8a4e 45%,#2c6a3a 100%)', rad: '10px', name: 'SCREAMER 808' },
    klon: { w: 128, h: 200, bg: 'repeating-linear-gradient(90deg,rgba(255,255,255,.08) 0 1px,transparent 1px 3px),linear-gradient(180deg,#efd590 0%,#caa652 50%,#a8862f 100%)', rad: '10px', name: 'GOLDEN KLONE' },
    rat: { w: 116, h: 180, bg: 'radial-gradient(circle at 1px 1px,rgba(255,255,255,.04) 1px,transparent 1.5px) 0 0/3px 3px,linear-gradient(180deg,#232323,#0d0d0d)', rad: '8px', name: 'RODENT' },
    muff: { w: 160, h: 200, bg: 'repeating-linear-gradient(90deg,rgba(255,255,255,.12) 0 1px,transparent 1px 3px),linear-gradient(180deg,#eef0f2 0%,#b8bcc0 50%,#d5d8db 100%)', rad: '8px', name: 'BIG FUZZ PI' },
    fuzzface: { w: 180, h: 180, bg: 'radial-gradient(circle at 38% 30%,#e8564a 0%,#b5261d 55%,#741109 100%)', rad: '50%', name: 'FACE FUZZ' },
    ds1: { w: 104, h: 184, bg: 'linear-gradient(180deg,#f47a2a 0%,#d65a10 100%)', rad: '8px', name: 'DS-ONE' },
    ns2: { w: 104, h: 184, bg: 'linear-gradient(180deg,#f1f0ea 0%,#cfcec7 100%)', rad: '8px', name: 'GATE-2' },
    ce2: { w: 104, h: 184, bg: 'linear-gradient(180deg,#86c3e6 0%,#5ba3cf 100%)', rad: '8px', name: 'CHORUS-2' },
    mxr10: { w: 232, h: 150, bg: 'radial-gradient(circle at 1px 1px,rgba(255,255,255,.04) 1px,transparent 1.5px) 0 0/3px 3px,linear-gradient(180deg,#262626,#0c0c0c)', rad: '8px', name: '10 BAND EQ' },
  };

  const d = dims[model];
  const root = document.createElement('div');
  root.style.cssText = `position:relative;width:${d.w}px;height:${d.h}px;user-select:none;flex-shrink:0`;

  const body = document.createElement('div');
  body.style.cssText = `position:absolute;inset:0;border-radius:${d.rad};background:${d.bg};box-shadow:0 14px 18px rgba(0,0,0,.65),0 2px 4px rgba(0,0,0,.6),inset 0 2px 0 rgba(255,255,255,.35),inset 0 -3px 0 rgba(0,0,0,.35)`;
  root.appendChild(body);

  const isDrive = model === 'ts808' || model === 'klon' || model === 'rat' || model === 'muff' || model === 'fuzzface' || model === 'ds1';
  const bypassId =
    model === 'ns2' ? 'gate.enabled' : model === 'ce2' ? 'mod_2.enabled' : model === 'mxr10' ? 'eq_1.enabled' : 'drive.enabled';
  const bypassKey = bypassId;
  const isOn = Boolean(store.getParam(bypassKey, true));

  const knobsByModel: Record<PedalModelId, PedalKnobDef[]> = {
    ts808: [
      { controlId: 'drive.drive', label: 'OVERDRIVE', min: 0, max: 10, step: 0.1, def: 4.5, x: 32, y: 16, px: 34, cls: 'p-boss', ink: '#f2f2f2' },
      { controlId: 'drive.level', label: 'LEVEL', min: 0, max: 10, step: 0.1, def: 6.0, x: 98, y: 16, px: 34, cls: 'p-boss', ink: '#f2f2f2' },
      { controlId: 'drive.tone', label: 'TONE', min: 0, max: 10, step: 0.1, def: 5.5, x: 65, y: 50, px: 26, cls: 'p-boss', ink: '#f2f2f2' },
    ],
    klon: [
      { controlId: 'drive.drive', label: 'GAIN', min: 0, max: 10, step: 0.1, def: 4.5, x: 32, y: 28, px: 30, cls: 'p-black', ink: '#3a2a0c' },
      { controlId: 'drive.tone', label: 'TREBLE', min: 0, max: 10, step: 0.1, def: 5.5, x: 64, y: 28, px: 30, cls: 'p-black', ink: '#3a2a0c' },
      { controlId: 'drive.level', label: 'OUTPUT', min: 0, max: 10, step: 0.1, def: 6.0, x: 96, y: 28, px: 30, cls: 'p-black', ink: '#3a2a0c' },
    ],
    rat: [
      { controlId: 'drive.drive', label: 'DISTORTION', min: 0, max: 10, step: 0.1, def: 4.5, x: 24, y: 16, px: 28, cls: 'p-black', ink: '#ededed' },
      { controlId: 'drive.tone', label: 'FILTER', min: 0, max: 10, step: 0.1, def: 5.5, x: 58, y: 16, px: 28, cls: 'p-black', ink: '#ededed' },
      { controlId: 'drive.level', label: 'VOLUME', min: 0, max: 10, step: 0.1, def: 6.0, x: 92, y: 16, px: 28, cls: 'p-black', ink: '#ededed' },
    ],
    muff: [
      { controlId: 'drive.level', label: 'VOLUME', min: 0, max: 10, step: 0.1, def: 6.0, x: 34, y: 14, px: 30, cls: 'p-black', ink: '#111' },
      { controlId: 'drive.tone', label: 'TONE', min: 0, max: 10, step: 0.1, def: 5.5, x: 80, y: 14, px: 30, cls: 'p-black', ink: '#111' },
      { controlId: 'drive.drive', label: 'SUSTAIN', min: 0, max: 10, step: 0.1, def: 4.5, x: 126, y: 14, px: 30, cls: 'p-black', ink: '#111' },
    ],
    fuzzface: [
      { controlId: 'drive.level', label: 'VOLUME', min: 0, max: 10, step: 0.1, def: 6.0, x: 56, y: 28, px: 32, cls: 'p-chicken', ink: '#fff3e8' },
      { controlId: 'drive.drive', label: 'FUZZ', min: 0, max: 10, step: 0.1, def: 4.5, x: 124, y: 28, px: 32, cls: 'p-chicken', ink: '#fff3e8' },
      { controlId: 'drive.bias', label: 'BIAS', min: 0, max: 10, step: 0.1, def: 5.0, x: 90, y: 18, px: 18, cls: 'p-trim', ink: '#fff3e8', fs: 6 },
    ],
    ds1: [
      { controlId: 'drive.level', label: 'LEVEL', min: 0, max: 10, step: 0.1, def: 6.0, x: 22, y: 12, px: 26, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6.5 },
      { controlId: 'drive.tone', label: 'TONE', min: 0, max: 10, step: 0.1, def: 5.5, x: 52, y: 12, px: 26, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6.5 },
      { controlId: 'drive.drive', label: 'DIST', min: 0, max: 10, step: 0.1, def: 4.5, x: 82, y: 12, px: 26, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6.5 },
    ],
    ns2: [
      { controlId: 'gate.threshold_db', label: 'THRESHOLD', min: -90, max: 0, step: 1, def: -65, x: 22, y: 12, px: 26, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6 },
      { controlId: 'gate.release_ms', label: 'DECAY', min: 5, max: 1000, step: 5, def: 80, x: 52, y: 12, px: 26, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6 },
      { controlId: 'gate.hysteresis_db', label: 'HYST', min: 0, max: 12, step: 0.5, def: 4, x: 82, y: 12, px: 26, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6 },
    ],
    ce2: [
      { controlId: 'mod_2.rate_hz', label: 'RATE', min: 0.05, max: 10, step: 0.05, def: 1.0, x: 34, y: 12, px: 28, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6.5 },
      { controlId: 'mod_2.depth', label: 'DEPTH', min: 0, max: 10, step: 0.1, def: 5.0, x: 70, y: 12, px: 28, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6.5 },
    ],
    mxr10: [],
  };

  // Nameplate artwork
  if (model !== 'ts808' && model !== 'ds1' && model !== 'ns2' && model !== 'ce2') {
    const titleEl = document.createElement('div');
    titleEl.className = 'osw';
    titleEl.textContent = d.name;
    titleEl.style.cssText =
      model === 'mxr10'
        ? 'position:absolute;left:10px;top:114px;font-size:14px;font-weight:700;color:#f2f2f2;letter-spacing:.04em'
        : model === 'fuzzface'
          ? 'position:absolute;left:0;right:0;top:138px;text-align:center;font-size:15px;font-weight:700;letter-spacing:.12em;color:#fff3e8'
          : model === 'muff'
            ? 'position:absolute;left:10px;right:10px;top:86px;height:46px;border-radius:4px;background:#111;color:#e3261c;display:flex;align-items:center;justify-content:center;font-size:24px;font-style:italic;font-weight:700'
            : model === 'klon'
              ? 'position:absolute;left:0;right:0;top:100px;text-align:center;font-size:14px;font-weight:700;color:#3a2a0c;letter-spacing:.08em'
              : 'position:absolute;left:0;right:0;top:86px;text-align:center;font-size:26px;font-style:italic;font-weight:700;color:#f4f4f4';
    root.appendChild(titleEl);
  }

  for (const k of knobsByModel[model]) {
    const holder = document.createElement('div');
    holder.style.cssText = `position:absolute;left:${k.x}px;top:${k.y}px;width:62px;margin-left:-31px;display:flex;flex-direction:${k.dir ?? 'column'};align-items:center;gap:2px`;
    const curVal = Number(store.getParam(k.controlId, k.def));
    const knob = createKnob({
      controlId: k.controlId,
      label: `${d.name} ${k.label}`,
      value: curVal,
      min: k.min,
      max: k.max,
      step: k.step,
      defaultValue: k.def,
      skinClass: k.cls,
      sizePx: k.px,
      onChange: (v) => store.setControl(k.controlId, v),
    });
    const lbl = document.createElement('span');
    lbl.textContent = k.label;
    lbl.style.cssText = `font-size:${k.fs ?? 7}px;font-weight:800;letter-spacing:.04em;color:${k.ink};line-height:1;white-space:nowrap`;
    holder.append(knob, lbl);
    root.appendChild(holder);
  }

  if (model === 'mxr10') {
    const bands = [
      { id: 'eq_1.volume_db', lbl: 'VOL', min: -12, max: 12 },
      { id: 'eq_1.band1', lbl: '31', min: -12, max: 15 },
      { id: 'eq_1.band2', lbl: '62', min: -12, max: 15 },
      { id: 'eq_1.band3', lbl: '125', min: -12, max: 15 },
      { id: 'eq_1.band4', lbl: '250', min: -12, max: 15 },
      { id: 'eq_1.band5', lbl: '500', min: -12, max: 15 },
      { id: 'eq_1.band6', lbl: '1K', min: -12, max: 15 },
      { id: 'eq_1.band7', lbl: '2K', min: -12, max: 15 },
      { id: 'eq_1.band8', lbl: '4K', min: -12, max: 15 },
      { id: 'eq_1.band9', lbl: '8K', min: -12, max: 15 },
      { id: 'eq_1.band10', lbl: '16K', min: -12, max: 15 },
      { id: 'eq_1.gain_db', lbl: 'GAIN', min: -12, max: 12 },
    ];
    bands.forEach((b, i) => {
      const desc = store.getDescriptor(b.id);
      const bKey = desc?.backendKey ?? b.id;
      const val = Number(store.getParam(bKey, 0));
      const x = 18 + i * 17.8;
      const slot = document.createElement('div');
      slot.style.cssText = `position:absolute;left:${x}px;top:14px;width:4px;height:76px;margin-left:-2px;border-radius:2px;background:#050505`;
      const cap = document.createElement('div');
      cap.className = 'slcap';
      cap.setAttribute('data-control-id', b.id);
      cap.setAttribute('role', 'slider');
      cap.tabIndex = 0;
      cap.setAttribute('aria-label', `EQ ${b.lbl}`);
      cap.setAttribute('aria-valuemin', String(b.min));
      cap.setAttribute('aria-valuemax', String(b.max));
      cap.setAttribute('aria-valuenow', String(val));
      const topPx = Math.round(((b.max - val) / (b.max - b.min)) * 67);
      cap.style.top = `${topPx}px`;

      let startY = 0;
      let startVal = val;
      let active = false;
      const commitEq = (v: number) => {
        const next = clampStep(v, b.min, b.max, 0.5);
        cap.style.top = `${Math.round(((b.max - next) / (b.max - b.min)) * 67)}px`;
        cap.setAttribute('aria-valuenow', String(next));
        store.setControl(b.id, next);
      };
      cap.addEventListener('pointerdown', (e) => {
        try {
          cap.setPointerCapture(e.pointerId);
        } catch {
          // ignore
        }
        active = true;
        startY = e.clientY;
        startVal = Number(cap.getAttribute('aria-valuenow') ?? 0);
      });
      cap.addEventListener('pointermove', (e) => {
        if (!active) return;
        const delta = ((startY - e.clientY) / 67) * (b.max - b.min);
        commitEq(startVal + (e.shiftKey ? delta * 0.25 : delta));
      });
      cap.addEventListener('pointerup', () => {
        active = false;
      });
      cap.addEventListener('dblclick', () => commitEq(0));
      cap.addEventListener('keydown', (e) => {
        const dir = e.key === 'ArrowUp' || e.key === 'ArrowRight' ? 1 : e.key === 'ArrowDown' || e.key === 'ArrowLeft' ? -1 : 0;
        if (!dir) return;
        e.preventDefault();
        commitEq(Number(cap.getAttribute('aria-valuenow') ?? 0) + dir * 0.5);
      });

      slot.appendChild(cap);
      const lbl = document.createElement('span');
      lbl.textContent = b.lbl;
      lbl.style.cssText = `position:absolute;left:${x}px;top:94px;width:28px;margin-left:-14px;text-align:center;font-size:6.5px;font-weight:700;color:#d8d8d8`;
      root.append(slot, lbl);
    });
  }

  // LED + Stomp / Treadle
  const ledPos: Record<PedalModelId, [number, number]> = {
    ts808: [65, 24],
    klon: [64, 15],
    rat: [58, 70],
    muff: [80, 148],
    fuzzface: [90, 66],
    ds1: [19, 68],
    ns2: [19, 68],
    ce2: [19, 68],
    mxr10: [160, 124],
  };
  const [lx, ly] = ledPos[model];
  const ledCol = model === 'mxr10' ? '#3db7ff' : '#ff2a1a';
  const led = document.createElement('div');
  led.style.cssText = `position:absolute;left:${lx}px;top:${ly}px;width:9px;height:9px;margin:-4.5px 0 0 -4.5px;border-radius:50%;background:${isOn ? ledCol : '#3a1210'};box-shadow:${isOn ? `0 0 9px ${ledCol}` : 'inset 0 1px 2px rgba(0,0,0,.7)'}`;
  root.appendChild(led);

  const isTreadle = model === 'ts808' || model === 'ds1' || model === 'ns2' || model === 'ce2';
  const swBtn = document.createElement('button');
  swBtn.setAttribute('data-control-id', bypassId);
  swBtn.setAttribute('aria-pressed', String(isOn));
  swBtn.setAttribute('aria-label', `Bypass ${d.name}`);
  swBtn.addEventListener('click', () => {
    store.setControl(bypassId, !isOn);
    if (isDrive) store.selectBlock('drive');
  });

  if (isTreadle) {
    swBtn.className = 'treadle';
    swBtn.style.cssText =
      model === 'ts808'
        ? 'left:5px;top:100px;width:120px;height:104px;border-radius:6px;background:linear-gradient(180deg,#56a467,#357a45);box-shadow:0 -2px 0 rgba(0,0,0,.35),inset 0 1px 0 rgba(255,255,255,.4);overflow:hidden'
        : `left:4px;top:78px;width:96px;height:102px;border-radius:5px;background:rgba(0,0,0,.06);box-shadow:0 -2px 0 rgba(0,0,0,.3),inset 0 1px 0 rgba(255,255,255,.35);padding-top:6px`;
    const badge = document.createElement('span');
    badge.className = 'osw';
    badge.textContent = d.name;
    badge.style.cssText =
      model === 'ts808'
        ? 'width:100%;background:linear-gradient(180deg,#f4f4f4,#b9b9b9);font-size:12px;font-weight:700;color:#151515;padding:2px 0'
        : 'font-size:13px;font-weight:700;color:#111;border:1.5px solid #111;padding:0 5px';
    const pad = document.createElement('span');
    pad.className = 'pad';
    pad.style.cssText =
      model === 'ts808' ? 'width:98px;height:52px;margin-top:auto;margin-bottom:8px' : 'width:80px;height:48px;margin-top:auto;margin-bottom:6px';
    swBtn.append(badge, pad);
  } else {
    const swPos: Record<string, [number, number]> = {
      klon: [64, 164],
      rat: [58, 150],
      muff: [80, 172],
      fuzzface: [90, 104],
      mxr10: [196, 124],
    };
    const [sx, sy] = swPos[model] ?? [64, 150];
    swBtn.className = 'stomp';
    swBtn.style.cssText = `left:${sx}px;top:${sy}px;margin:-20px 0 0 -20px`;
  }

  root.appendChild(swBtn);
  void valueToRotationDeg;
  return root;
}
