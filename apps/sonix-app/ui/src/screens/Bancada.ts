import { createKnob } from '../components/Knob.ts';
import type { AppStore } from '../store.ts';
import { makeChipButton, num, str } from './_helpers.ts';

function buildScopePaths(store: AppStore): {
  mainPath: string | null;
  refPath: string | null;
  pointCount: number;
  errMsg: string | null;
  caption: string;
} {
  const mode = store.crtMode;
  const labels: Record<typeof mode, string> = {
    waveform: 'SAÍDA DO ESTÁGIO DE POTÊNCIA · tracejado = entrada DI',
    transfer: 'CURVA DE TRANSFERÊNCIA X-Y · ponto de operação e joelho de clipping',
    tonestack: 'TONE STACK · tracejado = impedância do falante',
  };

  if (store.scopeError) {
    return {
      mainPath: null,
      refPath: null,
      pointCount: 0,
      errMsg: `sem dados do motor (${store.scopeError})`,
      caption: labels[mode],
    };
  }

  let mainPts: Array<{ x: number; y: number }> = [];
  let refPts: Array<{ x: number; y: number }> = [];
  if (mode === 'waveform') {
    const raw = store.scope.waveform as unknown;
    const outArr = Array.isArray(raw)
      ? (raw as number[])
      : raw && typeof raw === 'object' && Array.isArray((raw as { output?: unknown }).output)
        ? ((raw as { output: number[] }).output)
        : [];
    const diArr =
      raw && typeof raw === 'object' && Array.isArray((raw as { di?: unknown }).di)
        ? ((raw as { di: number[] }).di)
        : [];
    if (outArr.length >= 2) {
      mainPts = outArr.map((v, i) => ({ x: (i / (outArr.length - 1)) * 600, y: 80 - v * 62 }));
    }
    if (diArr.length >= 2) {
      refPts = diArr.map((v, i) => ({ x: (i / (diArr.length - 1)) * 600, y: 80 - v * 62 }));
    }
  } else if (mode === 'transfer' && Array.isArray(store.scope.transfer) && store.scope.transfer.length >= 2) {
    mainPts = store.scope.transfer.map((p) => ({
      x: ((p.x + 1) / 2) * 600,
      y: 80 - p.y * 62,
    }));
  } else if (mode === 'tonestack' && Array.isArray(store.scope.tonestack) && store.scope.tonestack.length >= 2) {
    const ts = store.scope.tonestack;
    mainPts = ts.map((p, i) => ({
      x: (i / (ts.length - 1)) * 600,
      y: 60 - p.db * 4,
    }));
  }

  if (mainPts.length === 0) {
    return { mainPath: null, refPath: null, pointCount: 0, errMsg: null, caption: labels[mode] };
  }

  const d = mainPts.map((p, i) => `${i === 0 ? 'M' : 'L'}${p.x.toFixed(1)} ${p.y.toFixed(1)}`).join(' ');
  const refD =
    refPts.length >= 2
      ? refPts.map((p, i) => `${i === 0 ? 'M' : 'L'}${p.x.toFixed(1)} ${p.y.toFixed(1)}`).join(' ')
      : null;
  return {
    mainPath: d,
    refPath: refD,
    pointCount: mainPts.length,
    errMsg: null,
    caption: labels[mode],
  };
}

function makeTrackControl(
  store: AppStore,
  controlId: string,
  paramKey: string,
  label: string,
  min: number,
  max: number,
  step: number,
  def: number,
  formatVal: (v: number) => string,
  compact = false
): HTMLElement {
  const val = num(store, paramKey, def);
  const pct = Math.max(0, Math.min(100, ((val - min) / (max - min)) * 100));

  const wrap = document.createElement('div');
  wrap.style.cssText = 'display:flex;flex-direction:column;gap:4px;';

  const top = document.createElement('div');
  top.style.cssText = 'display:flex;align-items:center;justify-content:space-between;gap:8px;';
  const lbl = document.createElement('span');
  lbl.style.cssText = `font-size:${compact ? '10' : '11'}px;color:#c9ced4;`;
  lbl.textContent = label;
  const txt = document.createElement('span');
  txt.className = 'm-mono';
  txt.style.cssText = 'font-size:10.5px;color:#aeb4bc;';
  txt.textContent = formatVal(val);
  top.append(lbl, txt);

  const track = document.createElement('div');
  track.className = 'm-track';
  track.setAttribute('data-control-id', controlId);
  track.setAttribute('role', 'slider');
  track.tabIndex = 0;
  track.setAttribute('aria-label', label);
  track.setAttribute('aria-valuemin', String(min));
  track.setAttribute('aria-valuemax', String(max));
  track.setAttribute('aria-valuenow', String(val));

  const fill = document.createElement('div');
  fill.className = 'm-fill';
  fill.style.width = `${pct.toFixed(1)}%`;
  const thumb = document.createElement('div');
  thumb.className = 'm-thumb';
  thumb.style.left = `${pct.toFixed(1)}%`;
  track.append(fill, thumb);

  const commit = (raw: number) => {
    const snapped = Math.round(Math.max(min, Math.min(max, raw)) / step) * step;
    const clean = Number(snapped.toFixed(4));
    const p = Math.max(0, Math.min(100, ((clean - min) / (max - min)) * 100));
    fill.style.width = `${p.toFixed(1)}%`;
    thumb.style.left = `${p.toFixed(1)}%`;
    txt.textContent = formatVal(clean);
    track.setAttribute('aria-valuenow', String(clean));
    void store.setControl(controlId, clean);
  };

  let dragging = false;
  const fromPtr = (e: PointerEvent) => {
    const r = track.getBoundingClientRect();
    if (r.width <= 0) return;
    const u = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width));
    commit(min + u * (max - min));
  };
  track.addEventListener('pointerdown', (e) => {
    dragging = true;
    try {
      track.setPointerCapture(e.pointerId);
    } catch {
      // ignore
    }
    fromPtr(e);
  });
  track.addEventListener('pointermove', (e) => {
    if (dragging) fromPtr(e);
  });
  track.addEventListener('pointerup', () => {
    dragging = false;
  });
  track.addEventListener('keydown', (e) => {
    const dir = e.key === 'ArrowRight' || e.key === 'ArrowUp' ? 1 : e.key === 'ArrowLeft' || e.key === 'ArrowDown' ? -1 : 0;
    if (!dir) return;
    e.preventDefault();
    commit(Number(track.getAttribute('aria-valuenow') ?? def) + dir * step);
  });

  wrap.append(top, track);
  return wrap;
}

export function renderBancadaScreen(store: AppStore): HTMLElement {
  const root = document.createElement('div');
  root.style.cssText = 'flex:1;min-height:0;display:flex;flex-direction:column;background:#0b0c0e;overflow:hidden;';

  const ampModel = str(store, 'amp_1.model', 'dual_rect');
  const ampNames: Record<string, string> = {
    plexi: 'PLEXI 1959',
    jcm800: 'BRIT 800 · 2203',
    dual_rect: 'CALI RECTO · 3CH',
    twin: 'TWIN REVERB',
    ac30: 'AC30 TOP BOOST',
  };
  const preTubeRaw = str(store, 'amp_1.preamp_tube', '12ax7');
  const pwrTubeRaw = str(store, 'amp_1.power_tube', 'el34');
  const rectRaw = str(store, 'amp_1.rectifier', 'diode');
  const impRaw = str(store, 'amp_1.impedance_ohms', '8');
  const biasMa = num(store, 'amp_1.bias_ma', 38);
  const sagVal = num(store, 'amp_1.sag', 3.0);
  const bplus = rectRaw === 'diode' ? 464 : rectRaw === 'gz34' ? 440 : 426;

  // ── Top Bar ────────────────────────────────────────────────────────────────
  const topBar = document.createElement('div');
  topBar.className = 'm-metal';
  topBar.style.cssText =
    'height:56px;flex:none;display:flex;align-items:center;gap:16px;padding:0 18px;border-bottom:1px solid #000;box-shadow:inset 0 1px 0 rgba(255,255,255,.06);';

  const backBtn = document.createElement('button');
  backBtn.className = 'm-btn';
  backBtn.style.cssText =
    'height:34px;padding:0 14px;border-radius:6px;background:linear-gradient(180deg,#30353c,#1a1d21);border:1px solid #000;color:#e8eaed;font-size:12px;display:flex;align-items:center;gap:8px;box-shadow:inset 0 1px 0 rgba(255,255,255,.12);';
  backBtn.setAttribute('data-control-id', 'ui.nav_screen');
  backBtn.textContent = '‹ Rig';
  backBtn.addEventListener('click', () => store.setScreen('main'));

  const dot = document.createElement('span');
  dot.style.cssText = 'width:9px;height:9px;border-radius:50%;background:#ff5a1f;box-shadow:0 0 8px #ff5a1f;flex:none;';

  const title = document.createElement('span');
  title.className = 'm-osw';
  title.style.cssText = 'font-size:20px;font-weight:600;letter-spacing:.05em;color:#e8eaed;text-transform:uppercase;white-space:nowrap;';
  title.textContent = `${ampNames[ampModel] ?? ampModel.toUpperCase()} — CHASSI`;

  const sub = document.createElement('span');
  sub.style.cssText = 'font-size:11px;color:#9aa3ad;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;';
  sub.textContent = 'clique numa válvula para inspecionar · valores simulados pelo modelo, não medições';

  const testWrap = document.createElement('div');
  testWrap.style.cssText = 'margin-left:auto;display:flex;align-items:center;gap:8px;flex:none;';
  const testLbl = document.createElement('span');
  testLbl.className = 'm-lbl';
  testLbl.style.cssText = 'font-size:9px;white-space:nowrap;';
  testLbl.textContent = 'SINAL DE TESTE';
  const boostVal = num(store, 'main.input_boost', 1.0);
  const boostDb = Math.round(20 * Math.log10(Math.max(0.1, boostVal)) - 15);
  const boostCtrl = makeTrackControl(store, 'main.input_boost', 'main.input_boost', '', 0.1, 8.0, 0.05, 1.0, () => '', true);
  boostCtrl.style.cssText = 'width:150px;flex:none;';
  const boostFirst = boostCtrl.firstElementChild as HTMLElement | null;
  if (boostFirst) boostFirst.style.display = 'none';
  const boostTxt = document.createElement('span');
  boostTxt.className = 'm-mono';
  boostTxt.style.cssText = 'font-size:11px;width:56px;color:#e8eaed;white-space:nowrap;';
  boostTxt.textContent = `${boostDb} dBFS`;
  testWrap.append(testLbl, boostCtrl, boostTxt);

  topBar.append(backBtn, dot, title, sub, testWrap);
  root.appendChild(topBar);

  // ── Main Content Split ─────────────────────────────────────────────────────
  const main = document.createElement('div');
  main.style.cssText = 'flex:1;min-height:0;display:grid;grid-template-columns:minmax(0,1fr) 380px;overflow:hidden;';

  // Left: Workbench containing 470px Chassis Bay + Scope Row
  const left = document.createElement('div');
  left.style.cssText =
    'display:flex;flex-direction:column;gap:16px;padding:20px 22px;min-height:0;background:radial-gradient(ellipse at 45% 30%,rgba(255,150,70,0.08) 0%,transparent 55%),repeating-linear-gradient(90deg,rgba(255,255,255,.012) 0 1px,transparent 1px 46px),linear-gradient(180deg,#141518,#0b0c0d);overflow:hidden;';

  const bay = document.createElement('div');
  bay.style.cssText = 'height:470px;flex:none;position:relative;display:flex;flex-direction:column;justify-content:flex-end;';

  // Brushed aluminum chassis apron (150px at bottom of 470px chassis box) + phenolic turret board
  const apron = document.createElement('div');
  apron.className = 'alu';
  apron.style.cssText =
    'position:absolute;left:0;right:0;bottom:0;height:150px;border-radius:6px;box-shadow:0 30px 40px rgba(0,0,0,.7),0 0 0 1px #000;';
  for (const pos of ['left:10px;top:10px;', 'right:10px;top:10px;', 'left:10px;bottom:10px;', 'right:10px;bottom:10px;']) {
    const sc = document.createElement('span');
    sc.className = 'm-screw';
    sc.style.cssText = `position:absolute;${pos}`;
    apron.appendChild(sc);
  }

  const board = document.createElement('div');
  board.className = 'turret';
  board.style.cssText =
    'position:absolute;left:150px;right:150px;bottom:14px;height:58px;border-radius:3px;box-shadow:0 4px 6px rgba(0,0,0,.45),inset 0 1px 0 rgba(255,255,255,.12);display:flex;align-items:center;justify-content:space-around;padding:0 14px;';
  const bandStyles = [
    'linear-gradient(180deg,#d9c79a 0 18%,#8b4513 18% 30%,#111 30% 42%,#e53935 42% 54%,#d9c79a 54% 76%,#c9a227 76% 88%,#d9c79a 88%)',
    'linear-gradient(180deg,#d9c79a 0 18%,#e53935 18% 30%,#7e57c2 30% 42%,#ff9800 42% 54%,#d9c79a 54% 76%,#c9a227 76% 88%,#d9c79a 88%)',
    'linear-gradient(180deg,#d9c79a 0 18%,#ffeb3b 18% 30%,#7e57c2 30% 42%,#8b4513 42% 54%,#d9c79a 54% 76%,#c9a227 76% 88%,#d9c79a 88%)',
  ];
  for (let i = 0; i < 22; i++) {
    const colWrap = document.createElement('div');
    colWrap.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:2px;';
    const isCap = i % 4 === 2 || i % 7 === 5;
    colWrap.innerHTML = `
      <span style="width:6px;height:6px;border-radius:50%;background:radial-gradient(circle at 35% 30%,#fff,#9a9a9a 60%,#555)"></span>
      <span style="width:${isCap ? 14 : 8}px;height:26px;border-radius:${isCap ? '6px' : '4px'};background:${
        isCap ? 'linear-gradient(90deg,#b98a1e,#f0c64a 40%,#c4931f)' : bandStyles[i % 3]
      }"></span>
      <span style="width:6px;height:6px;border-radius:50%;background:radial-gradient(circle at 35% 30%,#fff,#9a9a9a 60%,#555)"></span>
    `;
    board.appendChild(colWrap);
  }
  apron.appendChild(board);
  bay.appendChild(apron);

  const hwRow = document.createElement('div');
  hwRow.style.cssText = 'position:relative;display:flex;align-items:flex-end;justify-content:center;gap:14px;padding:0 24px 112px;';

  const bellCss =
    'background:radial-gradient(ellipse at 40% 20%,#45474b 0%,#1b1c1e 60%,#0b0b0c 100%);box-shadow:inset 0 2px 0 rgba(255,255,255,.12);';
  const makeXfmr = (w: number, h: number, lblText: string, subText: string, withBells: boolean) => {
    const wrap = document.createElement('div');
    wrap.style.cssText = 'display:flex;flex-direction:column;align-items:center;flex:none;';
    if (withBells) {
      const topBell = document.createElement('div');
      topBell.style.cssText = `width:120px;height:34px;border-radius:12px 12px 0 0;${bellCss}`;
      wrap.appendChild(topBell);
    }
    const box = document.createElement('div');
    box.className = 'xfmr';
    box.style.cssText = `width:${w}px;height:${h}px;border-radius:3px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:6px;`;
    box.innerHTML = `<div style="padding:6px 10px;border-radius:2px;background:#d8d2bf;box-shadow:0 1px 2px rgba(0,0,0,.6)"><span class="m-mono" style="font-size:9px;color:#222;font-weight:700">${lblText}</span></div>${
      subText ? `<span class="m-mono" style="font-size:9px;color:#9aa3ad">${subText}</span>` : ''
    }`;
    wrap.appendChild(box);
    if (withBells) {
      const botBell = document.createElement('div');
      botBell.style.cssText = `width:120px;height:14px;border-radius:0 0 4px 4px;${bellCss}`;
      wrap.appendChild(botBell);
    }
    return wrap;
  };

  hwRow.appendChild(makeXfmr(130, 170, 'TRANSF. FORÇA', `B+ ${bplus} V`, true));

  // Dual blue electrolytic capacitors
  const capPair = document.createElement('div');
  capPair.style.cssText = 'display:flex;gap:6px;align-items:flex-end;flex:none;';
  for (let i = 0; i < 2; i++) {
    const cap = document.createElement('div');
    cap.style.cssText =
      'width:40px;height:112px;border-radius:6px 6px 3px 3px;background:linear-gradient(90deg,#0e2a55 0%,#2d5ea8 30%,#1b417d 60%,#0b2148 100%);box-shadow:0 12px 18px rgba(0,0,0,.55);display:flex;align-items:center;justify-content:center;position:relative;';
    cap.innerHTML =
      '<span style="position:absolute;top:0;left:0;right:0;height:8px;border-radius:6px 6px 0 0;background:linear-gradient(180deg,#d9dde1,#8f969c)"></span>' +
      '<span class="m-mono" style="writing-mode:vertical-rl;transform:rotate(180deg);font-size:8px;color:#dfe8f5;letter-spacing:.06em">50µF 500V</span>';
    capPair.appendChild(cap);
  }
  hwRow.appendChild(capPair);

  // Tubes row (4 preamp + 4 power)
  const tubesWrap = document.createElement('div');
  tubesWrap.style.cssText = 'display:flex;align-items:flex-end;gap:6px;margin:0 6px;';
  const selTube = store.selectedTube || 'V7';
  const preNames = ['V1', 'V2', 'V3', 'PI'];
  const pwrNames = ['V5', 'V6', 'V7', 'V8'];
  const pwrLabelMap: Record<string, string> = { el34: 'EL34', '6l6gc': '6L6GC', kt88: 'KT88', el84_variant: 'EL84' };
  const pwrDims: Record<string, [number, number]> = { el34: [56, 196], '6l6gc': [56, 186], kt88: [62, 206], el84_variant: [40, 138] };
  const [pwW, pwH] = pwrDims[pwrTubeRaw] ?? [56, 196];

  for (const tName of [...preNames, ...pwrNames]) {
    const isPwr = pwrNames.includes(tName);
    const selected = selTube === tName;
    const w = isPwr ? pwW : 34;
    const h = isPwr ? pwH : 104;
    const bw = isPwr ? w - 4 : 30;
    const bh = isPwr ? 22 : 12;
    const sw = isPwr ? w + 6 : 38;
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.setAttribute('aria-pressed', String(selected));
    btn.setAttribute('aria-label', `Válvula ${tName}`);
    btn.style.cssText = `
      background:transparent;border:0;padding:6px 4px 0;border-radius:10px;outline:2px solid ${selected ? '#3dd8ff' : 'transparent'};outline-offset:2px;
      display:flex;flex-direction:column;align-items:center;cursor:pointer;color:inherit;
    `;
    const glowPx = selected ? 20 : 10;
    const glowOp = selected ? 1 : 0.7;
    btn.innerHTML = `
      <div style="filter:drop-shadow(0 0 ${glowPx}px rgba(255,120,30,.65));display:flex;flex-direction:column;align-items:center">
        <div style="position:relative;overflow:hidden;width:${w}px;height:${h}px;border-radius:${isPwr ? '26px 26px 8px 8px' : '16px 16px 6px 6px'};border:1px solid rgba(255,255,255,.28);background:linear-gradient(90deg,rgba(255,255,255,.05) 0%,rgba(255,255,255,.3) 14%,rgba(255,255,255,.06) 30%,rgba(255,255,255,.02) 72%,rgba(255,255,255,.16) 88%,rgba(255,255,255,.04) 100%)">
          <div style="position:absolute;left:0;right:0;top:0;height:20%;background:radial-gradient(ellipse at 50% 0%,rgba(215,222,230,.8) 0%,rgba(120,128,138,.4) 55%,transparent 80%)"></div>
          <div style="position:absolute;left:22%;right:22%;top:24%;bottom:18%;border-radius:2px;background:repeating-linear-gradient(0deg,rgba(0,0,0,.18) 0 1px,transparent 1px 5px),linear-gradient(90deg,#2b2b2b,#6c6c6c 45%,#8f8f8f 50%,#3a3a3a);box-shadow:inset 0 0 0 1px rgba(0,0,0,.4)"></div>
          <span style="position:absolute;left:12%;right:12%;top:22%;height:3px;background:rgba(230,230,220,.55);border-radius:1px"></span>
          <span style="position:absolute;left:12%;right:12%;bottom:16%;height:3px;background:rgba(230,230,220,.55);border-radius:1px"></span>
          <div style="position:absolute;left:0;right:0;bottom:4%;height:46%;background:radial-gradient(ellipse at 50% 72%,rgba(255,200,110,.95) 0%,rgba(255,110,25,.62) 30%,rgba(255,80,10,.16) 58%,transparent 74%);mix-blend-mode:screen;opacity:${glowOp}"></div>
        </div>
        <div style="width:${bw}px;height:${bh}px;border-radius:3px 3px 5px 5px;background:${isPwr ? 'linear-gradient(180deg,#523c2a,#241910)' : 'linear-gradient(180deg,#3d3025,#1c150f)'};display:flex;align-items:center;justify-content:center;box-shadow:0 3px 4px rgba(0,0,0,.6)">
          <span class="m-mono" style="font-size:7px;color:#e9dccb">${isPwr ? (pwrLabelMap[pwrTubeRaw] ?? 'EL34') : preTubeRaw.toUpperCase()}</span>
        </div>
        <div style="width:${sw}px;height:8px;background:linear-gradient(180deg,#bfc4c9,#6d7379);border-radius:0 0 3px 3px"></div>
      </div>
      <span class="m-mono" style="font-size:10px;font-weight:700;color:${selected ? '#3dd8ff' : '#c9ced4'};margin:6px 0 4px">${tName}</span>
    `;
    btn.addEventListener('click', () => {
      store.selectedTube = tName;
      store.emit();
    });
    tubesWrap.appendChild(btn);
  }
  hwRow.appendChild(tubesWrap);

  const outWrap = document.createElement('div');
  outWrap.style.cssText = 'display:flex;gap:10px;align-items:flex-end;flex:none;';
  outWrap.appendChild(makeXfmr(120, 150, 'TRANSF. SAÍDA', `${impRaw} Ω`, false));
  const choke = document.createElement('div');
  choke.className = 'xfmr';
  choke.style.cssText = 'width:68px;height:88px;border-radius:3px;display:flex;align-items:center;justify-content:center;';
  choke.innerHTML = '<span class="m-mono" style="font-size:8px;color:#9aa3ad">CHOKE</span>';
  outWrap.appendChild(choke);
  hwRow.appendChild(outWrap);
  bay.appendChild(hwRow);
  left.appendChild(bay);

  // Lower Split: CRT Oscilloscope + Fonte Card (minmax(0,1fr) 260px)
  const lower = document.createElement('div');
  lower.style.cssText = 'flex:1;display:grid;grid-template-columns:minmax(0,1fr) 260px;gap:16px;min-height:0;';

  const crtCard = document.createElement('div');
  crtCard.style.cssText =
    'border-radius:10px;padding:10px;background:linear-gradient(180deg,#2a2d31,#121315);box-shadow:0 10px 20px rgba(0,0,0,.6),inset 0 1px 0 rgba(255,255,255,.1);display:flex;gap:12px;min-width:0;';

  const scopeCol = document.createElement('div');
  scopeCol.className = 'crt';
  scopeCol.style.cssText = 'flex:1;border-radius:14px;position:relative;overflow:hidden;min-width:0;';

  const gridOverlay = document.createElement('div');
  gridOverlay.style.cssText =
    'position:absolute;inset:0;background:repeating-linear-gradient(0deg,rgba(80,255,170,.09) 0 1px,transparent 1px 25%),repeating-linear-gradient(90deg,rgba(80,255,170,.09) 0 1px,transparent 1px 10%);pointer-events:none;';
  const midLine = document.createElement('div');
  midLine.style.cssText = 'position:absolute;left:0;right:0;top:50%;height:1px;background:rgba(80,255,170,.22);pointer-events:none;';
  scopeCol.append(gridOverlay, midLine);

  const scopeData = buildScopePaths(store);
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 600 160');
  svg.setAttribute('preserveAspectRatio', 'none');
  svg.style.cssText = 'position:absolute;inset:0;width:100%;height:100%;';
  let svgHtml = '';
  if (scopeData.refPath) {
    svgHtml += `<path d="${scopeData.refPath}" fill="none" stroke="rgba(87,255,176,0.28)" stroke-width="1.2" stroke-dasharray="4 4"/>`;
  }
  if (scopeData.mainPath) {
    svgHtml += `<path data-scope-series="${store.crtMode}" data-point-count="${scopeData.pointCount}" d="${scopeData.mainPath}" fill="none" stroke="#57ffb0" stroke-width="2.2" style="filter:drop-shadow(0 0 4px #57ffb0)"/>`;
  }
  svg.innerHTML = svgHtml;
  scopeCol.appendChild(svg);

  const scopeHdr = document.createElement('span');
  scopeHdr.className = 'm-mono';
  scopeHdr.style.cssText = 'position:absolute;left:12px;top:8px;font-size:10px;color:#57ffb0;opacity:.8;';
  scopeHdr.textContent = scopeData.caption;
  scopeCol.appendChild(scopeHdr);

  if (scopeData.errMsg) {
    const errDiv = document.createElement('div');
    errDiv.className = 'm-mono';
    errDiv.style.cssText =
      'position:absolute;inset:24px 0 32px;display:flex;align-items:center;justify-content:center;color:#ff6a4a;font-size:12px;font-weight:700;text-align:center;';
    errDiv.textContent = scopeData.errMsg;
    scopeCol.appendChild(errDiv);
  }

  const crtFoot = document.createElement('div');
  crtFoot.style.cssText = 'position:absolute;left:12px;bottom:6px;display:flex;gap:4px;';
  for (const [m, lbl] of [
    ['waveform', 'Forma de onda'],
    ['transfer', 'Transferência'],
    ['tonestack', 'Tone stack'],
  ] as const) {
    const btn = makeChipButton('chassi.crt_mode', lbl, store.crtMode === m, () => {
      store.crtMode = m;
      void store.refreshScope();
    });
    btn.className = 'm-mono';
    btn.style.cssText = `height:20px;padding:0 8px;border-radius:3px;border:1px solid rgba(87,255,176,.35);background:${
      store.crtMode === m ? 'rgba(87,255,176,.22)' : 'rgba(8,24,18,.6)'
    };color:${store.crtMode === m ? '#57ffb0' : '#7cb099'};font-size:10px;cursor:pointer;`;
    crtFoot.appendChild(btn);
  }
  const clipLbl = document.createElement('span');
  clipLbl.className = 'm-mono';
  clipLbl.style.cssText = 'position:absolute;right:12px;bottom:8px;font-size:10px;color:#57ffb0;opacity:.8;';
  clipLbl.textContent = 'saturação suave';
  scopeCol.append(crtFoot, clipLbl);

  // Headroom column inside CRT bezel (130px)
  const headCol = document.createElement('div');
  headCol.style.cssText = 'width:130px;flex:none;display:flex;flex-direction:column;gap:8px;justify-content:center;';
  const hTop = document.createElement('span');
  hTop.className = 'm-lbl';
  hTop.style.cssText = 'font-size:9px;';
  hTop.textContent = 'Headroom';
  const hBars = document.createElement('div');
  hBars.style.cssText =
    'height:120px;width:22px;border-radius:4px;background:#050505;padding:3px;box-sizing:border-box;display:flex;flex-direction:column-reverse;gap:2px;';
  const litSegs = Math.max(2, Math.min(14, Math.round(14 - sagVal * 0.7)));
  for (let i = 0; i < 14; i++) {
    const lit = i < litSegs;
    const col = i < 4 ? '#ff3b2f' : i < 7 ? '#ffc400' : '#29c46a';
    const seg = document.createElement('span');
    seg.style.cssText = `flex:1;border-radius:1px;background:${lit ? col : '#1a1c1f'};box-shadow:${lit ? `0 0 4px ${col}` : 'none'};`;
    hBars.appendChild(seg);
  }
  const hBot = document.createElement('span');
  hBot.className = 'm-mono';
  hBot.style.cssText = 'font-size:10px;color:#c9ced4;';
  hBot.textContent = litSegs > 7 ? 'folga confortável' : 'perto do limite';
  headCol.append(hTop, hBars, hBot);

  crtCard.append(scopeCol, headCol);

  // FONTE Card (260px grid column)
  const fonteCard = document.createElement('div');
  fonteCard.style.cssText =
    'border-radius:10px;padding:14px;background:linear-gradient(180deg,#1d2024,#121315);box-shadow:0 10px 20px rgba(0,0,0,.6),inset 0 1px 0 rgba(255,255,255,.08);display:flex;flex-direction:column;gap:10px;';
  fonteCard.innerHTML = '<span class="m-lbl" style="font-size:9px;">FONTE</span>';

  const rectGrp = document.createElement('div');
  rectGrp.style.cssText = 'display:flex;background:#0b0c0e;border:1px solid #000;border-radius:6px;padding:2px;';
  for (const [val, lbl] of [
    ['diode', 'Diodo'],
    ['gz34', 'GZ34'],
    ['5u4gb', '2× 5U4GB'],
  ] as const) {
    const b = makeChipButton('chassi.rectifier', lbl, rectRaw === val, () => store.setControl('chassi.rectifier', val));
    b.style.cssText = `flex:1;height:28px;border-radius:4px;border:0;font-size:11px;cursor:pointer;justify-content:center;background:${
      rectRaw === val ? '#232830' : 'transparent'
    };color:${rectRaw === val ? '#f0f3f7' : '#8a939e'};font-weight:${rectRaw === val ? '600' : '500'};`;
    rectGrp.appendChild(b);
  }
  fonteCard.appendChild(rectGrp);

  const sagCtrl = makeTrackControl(store, 'chassi.sag', 'amp_1.sag', 'Sag', 0, 10, 0.1, 3.0, (v) => v.toFixed(1), true);
  sagCtrl.style.cssText = 'display:flex;align-items:center;gap:8px;';
  const sagTop = sagCtrl.firstElementChild as HTMLElement | null;
  const sagTrack = sagCtrl.children[1] as HTMLElement | null;
  if (sagTop && sagTrack && sagTop.children.length === 2) {
    const sagName = sagTop.children[0] as HTMLElement;
    const sagReadout = sagTop.children[1] as HTMLElement;
    sagCtrl.innerHTML = '';
    sagName.style.cssText = 'font-size:11px;width:40px;color:#c9ced4;';
    sagReadout.style.cssText = 'font-size:10px;width:28px;text-align:right;';
    sagCtrl.append(sagName, sagTrack, sagReadout);
  }
  fonteCard.appendChild(sagCtrl);

  const impTitle = document.createElement('span');
  impTitle.className = 'm-lbl';
  impTitle.style.cssText = 'font-size:9px;margin-top:4px;';
  impTitle.textContent = 'CARGA (SECUNDÁRIO)';
  const impGrp = document.createElement('div');
  impGrp.style.cssText = 'display:flex;background:#0b0c0e;border:1px solid #000;border-radius:6px;padding:2px;';
  for (const val of ['4', '8', '16']) {
    const b = makeChipButton('chassi.impedance_ohms', `${val} Ω`, impRaw === val, () => store.setControl('chassi.impedance_ohms', val));
    b.className = 'm-mono';
    b.style.cssText = `flex:1;height:28px;border-radius:4px;border:0;font-size:11px;cursor:pointer;justify-content:center;background:${
      impRaw === val ? '#232830' : 'transparent'
    };color:${impRaw === val ? '#f0f3f7' : '#8a939e'};font-weight:${impRaw === val ? '600' : '500'};`;
    impGrp.appendChild(b);
  }
  fonteCard.append(impTitle, impGrp);

  lower.append(crtCard, fonteCard);
  left.appendChild(lower);

  // ── Right Sidebar (380px) ──────────────────────────────────────────────────
  const side = document.createElement('aside');
  side.className = 'm-metal';
  side.style.cssText = 'border-left:1px solid #000;display:flex;flex-direction:column;min-height:0;box-shadow:inset 1px 0 0 rgba(255,255,255,.05);overflow:hidden;';

  const tabsRow = document.createElement('div');
  tabsRow.style.cssText = 'height:36px;flex:none;display:grid;grid-template-columns:1fr 1fr;border-bottom:1px solid #000;';
  for (const [tabId, lbl] of [
    ['tube', 'Válvula'],
    ['circuit', 'Circuito'],
  ] as const) {
    const on = store.chassiTab === tabId;
    const b = makeChipButton('ui.subtab', lbl, on, () => {
      store.chassiTab = tabId;
      store.emit();
    });
    b.style.cssText = `height:36px;padding:0;justify-content:center;background:transparent;border:0;border-bottom:2px solid ${on ? '#ff8a3d' : 'transparent'};border-radius:0;color:${on ? '#e8eaed' : '#9aa3ad'};font-size:12px;font-weight:600;cursor:pointer;`;
    tabsRow.appendChild(b);
  }
  side.appendChild(tabsRow);

  const sideBody = document.createElement('div');
  sideBody.style.cssText = 'flex:1;min-height:0;display:flex;flex-direction:column;overflow:hidden;';

  if (store.chassiTab === 'tube') {
    const sec1 = document.createElement('div');
    sec1.style.cssText = 'padding:14px 16px;border-bottom:1px solid #000;display:flex;flex-direction:column;gap:6px;';
    const isSelPwr = pwrNames.includes(selTube);
    const titleBox = document.createElement('div');
    titleBox.style.cssText = 'display:flex;align-items:baseline;gap:10px;';
    const activeLabel = isSelPwr ? (pwrLabelMap[pwrTubeRaw] ?? 'EL34') : preTubeRaw.toUpperCase();
    titleBox.innerHTML = `<span class="m-osw" style="font-size:26px;font-weight:600;color:#e8eaed">${selTube}</span><span class="m-mono" style="font-size:14px;color:#ffb347">${activeLabel}</span>`;
    const roleSub = document.createElement('span');
    roleSub.style.cssText = 'font-size:11px;color:#9aa3ad;';
    roleSub.textContent = isSelPwr
      ? `Potência · push-pull, par ${selTube === 'V5' || selTube === 'V6' ? 'A' : 'B'} · fixed bias`
      : 'Pré-amplificador · estágio de ganho triodo';

    const chipsRow = document.createElement('div');
    chipsRow.style.cssText = 'display:flex;flex-wrap:wrap;gap:4px;margin-top:4px;';
    if (isSelPwr) {
      for (const [v, lbl] of [
        ['el34', 'EL34'],
        ['6l6gc', '6L6GC'],
        ['kt88', 'KT88'],
        ['el84_variant', 'EL84'],
      ] as const) {
        const b = makeChipButton('chassi.power_tube', lbl, pwrTubeRaw === v, () => store.setControl('chassi.power_tube', v));
        b.style.cssText += ';height:28px;padding:0 10px;border-radius:5px;font-size:11px;font-weight:600;';
        chipsRow.appendChild(b);
      }
    } else {
      for (const v of ['12ax7', 'ecc83', '12at7', '12ay7', 'ef86'] as const) {
        const b = makeChipButton('chassi.preamp_tube', v.toUpperCase(), preTubeRaw === v, () => store.setControl('chassi.preamp_tube', v));
        b.style.cssText += ';height:28px;padding:0 10px;border-radius:5px;font-size:11px;font-weight:600;';
        chipsRow.appendChild(b);
      }
    }
    sec1.append(titleBox, roleSub, chipsRow);
    sideBody.appendChild(sec1);

    // Dual analog cream VU meters (84px tall) with bezel
    const metersGrid = document.createElement('div');
    metersGrid.style.cssText =
      'padding:14px 16px;border-bottom:1px solid #000;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px;';
    const plateV = isSelPwr ? bplus - 12 : Math.round(bplus * 0.44);
    const curDeg = -62 + Math.min(1, biasMa / 80) * 124;
    const vDeg = -62 + Math.min(1, plateV / 600) * 124;
    for (const [mTitle, mTxt, mCol, mUnit, deg] of [
      ['CORRENTE DE PLACA', `${Math.round(biasMa)} mA`, '#29c46a', 'AMPERAGEM', curDeg],
      ['TENSÃO DE PLACA', `${plateV} V`, '#e8eaed', 'TENSÃO', vDeg],
    ] as const) {
      const mBox = document.createElement('div');
      mBox.style.cssText = 'display:flex;flex-direction:column;gap:6px;';
      mBox.innerHTML = `
        <span class="m-lbl" style="font-size:9px">${mTitle}</span>
        <div style="border-radius:5px;padding:5px;background:linear-gradient(180deg,#2a2d31,#121315);box-shadow:inset 0 1px 0 rgba(255,255,255,.1),0 3px 6px rgba(0,0,0,.6)">
          <div class="vuface" style="height:84px;border-radius:3px;position:relative;overflow:hidden;box-shadow:inset 0 2px 8px rgba(0,0,0,.5)">
            <svg viewBox="0 0 160 84" style="position:absolute;inset:0;width:100%;height:100%" aria-hidden="true">
              <path d="M22 70 A62 62 0 0 1 138 70" stroke="#2a2214" stroke-width="1.2" fill="none"/>
              <path d="M112 31 A62 62 0 0 1 138 70" stroke="#c0251a" stroke-width="3.5" fill="none"/>
              <g stroke="#2a2214" stroke-width="1.2"><path d="M22 70 l-5 -3"/><path d="M37 41 l-4 -4"/><path d="M60 24 l-2 -5"/><path d="M80 20 v-6"/><path d="M100 24 l2 -5"/><path d="M123 41 l4 -4"/><path d="M138 70 l5 -3"/></g>
              <text x="80" y="58" font-family="Oswald, sans-serif" font-size="9" fill="#2a2214" text-anchor="middle" letter-spacing="1.5">${mUnit}</text>
              <g transform="rotate(${deg.toFixed(1)} 80 82)"><path d="M80 82 L80 22" stroke="#141414" stroke-width="1.5" stroke-linecap="round"/></g>
              <circle cx="80" cy="82" r="6" fill="#1a1a1a"/>
            </svg>
          </div>
        </div>
        <span class="m-mono" style="font-size:12px;font-weight:700;color:${mCol}">${mTxt}</span>
      `;
      metersGrid.appendChild(mBox);
    }
    sideBody.appendChild(metersGrid);

    const sec3 = document.createElement('div');
    sec3.style.cssText = 'padding:14px 16px;display:flex;flex-direction:column;gap:12px;flex:1;min-height:0;';

    const adjHdr = document.createElement('span');
    adjHdr.className = 'm-lbl';
    adjHdr.style.cssText = 'font-size:9px;';
    adjHdr.textContent = 'AJUSTES DA VÁLVULA';
    sec3.appendChild(adjHdr);

    const biasRow = document.createElement('div');
    biasRow.style.cssText = 'display:flex;align-items:center;gap:16px;';
    const biasKnob = createKnob({
      controlId: 'chassi.bias_ma',
      label: 'Bias de Grade',
      value: biasMa,
      min: 20,
      max: 55,
      step: 0.5,
      defaultValue: 38.0,
      skinClass: 'k-marshall',
      sizePx: 70,
      onChange: (v) => store.setControl('chassi.bias_ma', v),
      onDragState: (d) => store.setDraggingKnob(d),
    });
    const biasNegV = -(52 - (biasMa - 20) * 0.45);
    const biasMeta = document.createElement('div');
    biasMeta.style.cssText = 'display:flex;flex-direction:column;gap:2px;';
    biasMeta.innerHTML = `<span class="m-osw" style="font-size:14px;font-weight:600;letter-spacing:.06em;color:#e8eaed">BIAS</span><span class="m-mono" style="font-size:16px;font-weight:700;color:#ffb347">${biasNegV.toFixed(1)} V</span><span style="font-size:10px;color:#9aa3ad">${biasMa > 45 ? 'quente' : biasMa < 30 ? 'frio' : 'nominal'} (${biasMa.toFixed(1)} mA)</span>`;
    biasRow.append(biasKnob, biasMeta);
    sec3.appendChild(biasRow);

    // Note box
    const dissW = ((plateV * biasMa) / 1000).toFixed(1);
    const dissMax = pwrTubeRaw === 'kt88' ? 42 : pwrTubeRaw === '6l6gc' ? 30 : 25;
    const dissPct = Math.round((Number(dissW) / dissMax) * 100);
    const noteBox = document.createElement('div');
    noteBox.style.cssText =
      'margin-top:auto;padding:10px 12px;border-radius:6px;background:rgba(61,216,255,.06);border:1px solid rgba(61,216,255,.2);font-size:11px;color:#c9ced4;line-height:1.5;';
    noteBox.textContent = isSelPwr
      ? `Dissipação estimada ${dissW} W de ${dissMax} W (${dissPct} %). Bias mais quente = mais corrente e compressão mais cedo; valores do modelo, não de medição.`
      : 'Trocar para 12AT7/12AY7 reduz o ganho deste estágio (fator de amplificação menor). Valores simulados pelo modelo WDF.';
    sec3.appendChild(noteBox);
    sideBody.appendChild(sec3);
  } else {
    sideBody.style.cssText = 'flex:1;min-height:0;padding:14px 16px;display:flex;flex-direction:column;gap:11px;overflow:hidden;';
    sideBody.appendChild(makeTrackControl(store, 'chassi.preamp_plate_v', 'amp_1.preamp_plate_v', 'Tensão de placa do pré', 180, 320, 2, 260, (v) => `${Math.round(v)} V`));
    sideBody.appendChild(makeTrackControl(store, 'chassi.triode_asym', 'amp_1.triode_asym', 'Assimetria triodo', 0, 10, 0.1, 3.0, (v) => v.toFixed(1)));
    sideBody.appendChild(makeTrackControl(store, 'chassi.nfb', 'amp_1.nfb', 'Realimentação (NFB)', 0, 10, 0.1, 5.0, (v) => v.toFixed(1)));
    sideBody.appendChild(makeTrackControl(store, 'chassi.xformer_sat', 'amp_1.xformer_sat', 'Saturação do trafo', 0, 10, 0.1, 4.0, (v) => v.toFixed(1)));

    const tsLbl = document.createElement('span');
    tsLbl.className = 'm-lbl';
    tsLbl.style.cssText = 'font-size:9.5px;margin-top:6px;';
    tsLbl.textContent = 'TOPOLOGIA TONE STACK';
    const tsRow = document.createElement('div');
    tsRow.style.cssText = 'display:flex;flex-wrap:wrap;gap:5px;';
    const curTs = str(store, 'amp_1.tonestack', 'marshall');
    for (const [k, l] of [
      ['baxandall', 'Baxandall'],
      ['fender', 'Fender TMB'],
      ['marshall', 'Marshall'],
      ['vox', 'Vox Top Boost'],
      ['mesa', 'Mesa Pré'],
    ] as const) {
      const b = makeChipButton('chassi.tonestack', l, curTs === k, () => store.setControl('chassi.tonestack', k));
      b.style.cssText += ';padding:4px 8px;font-size:10px;';
      tsRow.appendChild(b);
    }
    sideBody.append(tsLbl, tsRow);
  }

  side.appendChild(sideBody);
  main.append(left, side);
  root.appendChild(main);
  return root;
}
