import type { AppStore } from '../store.ts';
import { bool, makeChipButton, num, str } from './_helpers.ts';

export function renderPalcoScreen(store: AppStore): HTMLElement {
  const root = document.createElement('div');
  root.style.cssText =
    'flex:1;min-height:0;width:100%;height:100%;position:relative;box-sizing:border-box;padding:22px;background:repeating-linear-gradient(90deg,rgba(255,255,255,.06) 0 1px,rgba(0,0,0,.03) 1px 2px,transparent 2px 4px),linear-gradient(180deg,#5b6168,#3a3f45 50%,#2a2e33);overflow:hidden;user-select:none;color:#e8eaed;';

  for (const pos of [
    'left:8px;top:8px;',
    'right:8px;top:8px;',
    'left:8px;bottom:8px;',
    'right:8px;bottom:8px;',
  ]) {
    const screw = document.createElement('span');
    screw.style.cssText = `position:absolute;${pos}width:12px;height:12px;border-radius:50%;background:radial-gradient(circle at 35% 30%,#f0f0f0,#8a8a8a 55%,#333);pointer-events:none;`;
    root.appendChild(screw);
  }

  const chassisInner = document.createElement('div');
  chassisInner.style.cssText =
    'position:absolute;inset:14px;border-radius:10px;background:linear-gradient(180deg,#16181b,#0c0d0f);box-shadow:inset 0 2px 0 rgba(255,255,255,.06),0 0 0 2px #000;pointer-events:none;';
  root.appendChild(chassisInner);

  const stageWrap = document.createElement('div');
  stageWrap.style.cssText = 'position:relative;height:100%;display:flex;flex-direction:column;gap:16px;padding:14px 18px;box-sizing:border-box;';

  const tunerOn = bool(store, 'tuner.enabled', true);
  const tunerMute = str(store, 'tuner.output', 'mute') === 'mute';
  const refHz = num(store, 'tuner.ref_hz', 440);
  const looperPos = str(store, 'looper.position', 'post');
  const looperAction = str(store, 'looper.action', 'stop');
  const expVal = num(store, 'palco.exp_1', 62);
  const sceneLetter = str(store, 'scene.active', 'F').toUpperCase();
  const sceneNames: Record<string, string> = {
    A: 'CLEAN',
    B: 'CRUNCH',
    C: 'RHYTHM',
    D: 'LEAD',
    E: 'AMBIENT',
    F: 'HEAVY',
    G: 'SOLO',
    H: 'OCTAVE',
  };
  const bpm = store.bpm ? Math.round(store.bpm) : 120;
  const activePreset = store.presets[store.presetIdx];
  const presetMatch = activePreset?.name?.match(/^(P\d+)\s+(.+)$/);
  const presetNo = presetMatch?.[1] ?? 'P03';
  const presetTitle = presetMatch?.[2] ?? activePreset?.name ?? 'CALI DUAL RECTO MODERN METAL';
  const setlistTitle = (activePreset?.setlist || 'Show 2026').toUpperCase();

  const pBtnCss = (pressed = false) =>
    `border-radius:6px;border:1px solid ${pressed ? '#ff8a3d' : '#000'};background:${
      pressed ? 'linear-gradient(180deg,#3a2414,#22140a)' : 'linear-gradient(180deg,#30353c,#1a1d21)'
    };color:${pressed ? '#ffd2b0' : '#e0e4e8'};box-shadow:${
      pressed ? '0 0 12px rgba(255,138,61,.4)' : 'inset 0 1px 0 rgba(255,255,255,.12),0 2px 3px rgba(0,0,0,.5)'
    };font-weight:700;cursor:pointer;`;
  const pWellCss =
    'border-radius:10px;background:linear-gradient(180deg,#101214,#08090a);box-shadow:inset 0 3px 10px rgba(0,0,0,.9),0 1px 0 rgba(255,255,255,.08);';

  // ── 1. Header LCD ──────────────────────────────────────────────────────────
  const hdr = document.createElement('div');
  hdr.style.cssText = 'height:128px;flex:none;display:flex;gap:16px;';

  const lcd = document.createElement('div');
  lcd.style.cssText =
    'flex:1;min-width:0;padding:14px 20px;border-radius:4px;border:1px solid #000;box-shadow:inset 0 1px 4px #000;background:radial-gradient(ellipse at 30% 0%,#2a1a06 0%,#0d0702 70%);display:flex;flex-direction:column;justify-content:space-between;';
  lcd.innerHTML = `
    <div class="m-mono" style="display:flex;gap:18px;font-size:13px;color:#ffb347;text-shadow:0 0 6px rgba(255,150,40,.7)">
      <span>SETLIST · ${setlistTitle}</span><span>${presetNo}</span><span style="flex:1"></span><span>CENA ${sceneLetter} · ${sceneNames[sceneLetter] ?? 'HEAVY'}</span>
    </div>
    <div class="m-osw" style="font-size:48px;font-weight:600;letter-spacing:.03em;color:#ffcf8a;text-shadow:0 0 12px rgba(255,150,40,.6);white-space:nowrap;overflow:hidden;line-height:1;text-transform:uppercase">${presetTitle}</div>
  `;

  const tempoBox = document.createElement('div');
  tempoBox.style.cssText =
    'width:170px;flex:none;border-radius:4px;background:#020304;border:1px solid #000;box-shadow:inset 0 1px 4px #000;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:6px;cursor:pointer;';
  tempoBox.setAttribute('data-control-id', 'topbar.tap_tempo');
  tempoBox.innerHTML = `
    <span class="m-lbl" style="font-size:9px">TEMPO</span>
    <span class="m-mono" style="font-size:36px;font-weight:700;color:#57ffb0;text-shadow:0 0 8px rgba(87,255,176,.6);line-height:1">${bpm}</span>
    <span style="width:14px;height:14px;border-radius:50%;background:#57ffb0;box-shadow:0 0 10px #57ffb0"></span>
  `;
  tempoBox.addEventListener('click', () => store.tapTempo());

  const navCol = document.createElement('div');
  navCol.style.cssText = 'width:130px;flex:none;display:flex;flex-direction:column;gap:8px;';
  const exitBtn = makeChipButton('ui.nav_screen', 'Sair (Esc)', false, () => store.setScreen('main'));
  exitBtn.style.cssText = `${pBtnCss(false)}flex:1;display:flex;align-items:center;justify-content:center;font-size:13px;`;
  const tunBtn = makeChipButton('tuner.enabled', 'Afinador', tunerOn, () => store.setControl('tuner.enabled', !tunerOn));
  tunBtn.style.cssText = `${pBtnCss(tunerOn)}flex:1;display:flex;align-items:center;justify-content:center;font-size:13px;`;
  navCol.append(exitBtn, tunBtn);

  hdr.append(lcd, tempoBox, navCol);
  stageWrap.appendChild(hdr);

  // ── 2. Middle Stage: Afinador + Looper ─────────────────────────────────────
  const mid = document.createElement('div');
  mid.style.cssText = 'flex:1;display:flex;gap:16px;min-height:0;';

  // Afinador Card (flex: 1.5)
  const tunerCard = document.createElement('section');
  tunerCard.style.cssText = `${pWellCss}flex:1.5;padding:16px 20px;display:flex;flex-direction:column;gap:12px;min-width:0;opacity:${tunerOn ? 1 : 0.45};`;

  const tHdr = document.createElement('div');
  tHdr.style.cssText = 'display:flex;align-items:center;gap:8px;';
  tHdr.innerHTML = '<span class="m-lbl" style="flex:1;font-size:10px;letter-spacing:.12em">Afinador</span>';

  const mkPBtn = (cid: string, lbl: string, active: boolean, onClick: () => void, extra = 'height:28px;padding:0 10px;font-size:11px;') => {
    const b = makeChipButton(cid, lbl, active, onClick);
    b.style.cssText = `${pBtnCss(active)}${extra}`;
    return b;
  };
  const strobeBtn = mkPBtn('tuner.view', 'Strobe', store.palcoTunerView === 'strobe', () => {
    store.palcoTunerView = 'strobe';
    store.emit();
  });
  const polyBtn = mkPBtn('tuner.view', 'Polifônico', store.palcoTunerView === 'poly', () => {
    store.palcoTunerView = 'poly';
    store.emit();
  });
  const sp = document.createElement('span');
  sp.style.width = '12px';
  const muteBtn = mkPBtn('tuner.output', 'Mute', tunerMute, () => store.setControl('tuner.output', 'mute'));
  const bypBtn = mkPBtn('tuner.output', 'Bypass', !tunerMute, () => store.setControl('tuner.output', 'bypass'));
  tHdr.append(strobeBtn, polyBtn, sp, muteBtn, bypBtn);
  tunerCard.appendChild(tHdr);

  const tBody = document.createElement('div');
  tBody.style.cssText = 'flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;';
  const note = store.tuner.note ?? 'A';
  const oct = store.tuner.octave ?? 2;
  const cents = store.tuner.cents ?? 1.5;
  const hz = store.tuner.frequency_hz ?? 110.0;
  const needlePct = Math.max(4, Math.min(96, 50 + cents * 1.1));

  tBody.innerHTML = `
    <div style="display:flex;align-items:baseline;gap:6px">
      <span class="m-osw" style="font-size:150px;font-weight:700;line-height:.9;color:#29c46a;text-shadow:0 0 30px rgba(41,196,106,.5)">${note}</span>
      <span class="m-osw" style="font-size:44px;color:#a3abb4">${oct}</span>
    </div>
    <div style="width:100%;max-width:760px;height:46px;border-radius:6px;overflow:hidden;position:relative;box-shadow:inset 0 2px 6px #000;background:repeating-linear-gradient(90deg,#29c46a 0 22px,#0a0b0c 22px 44px)">
      <div style="position:absolute;left:50%;top:0;bottom:0;width:3px;margin-left:-1.5px;background:#fff;box-shadow:0 0 8px #fff"></div>
    </div>
    <div style="width:100%;max-width:760px;position:relative;height:30px">
      <div style="position:absolute;left:0;right:0;top:14px;height:2px;background:#2a2f36"></div>
      <div style="position:absolute;left:46%;width:8%;top:8px;height:14px;border-radius:3px;background:rgba(41,196,106,.2)"></div>
      <div style="position:absolute;top:2px;left:${needlePct.toFixed(1)}%;width:6px;height:26px;margin-left:-3px;border-radius:3px;background:#29c46a;box-shadow:0 0 10px #29c46a"></div>
    </div>
    <div class="m-mono" style="display:flex;gap:28px;font-size:15px">
      <span style="color:#c9ced4">${hz.toFixed(2)} Hz</span>
      <span style="color:#29c46a">${cents >= 0 ? '+' : ''}${cents.toFixed(1)} cents</span>
      <span style="color:#a3abb4">temperamento igual</span>
    </div>
  `;
  tunerCard.appendChild(tBody);

  const tFoot = document.createElement('div');
  tFoot.style.cssText = 'display:flex;align-items:center;gap:8px;';
  const refLbl = document.createElement('span');
  refLbl.className = 'm-lbl';
  refLbl.style.fontSize = '9px';
  refLbl.textContent = 'Referência A4';
  const refDown = mkPBtn('tuner.ref_hz', '−', false, () => store.setControl('tuner.ref_hz', Math.max(430, refHz - 1)), 'width:30px;height:28px;font-size:13px;');
  const refTxt = document.createElement('span');
  refTxt.className = 'm-mono';
  refTxt.style.cssText = 'font-size:14px;font-weight:700;width:76px;text-align:center;';
  refTxt.textContent = `${refHz.toFixed(1)} Hz`;
  const refUp = mkPBtn('tuner.ref_hz', '+', false, () => store.setControl('tuner.ref_hz', Math.min(450, refHz + 1)), 'width:30px;height:28px;font-size:13px;');
  const sp2 = document.createElement('span');
  sp2.style.width = '16px';
  const eqBtn = mkPBtn('ui.subtab', 'Igual', true, () => {});
  const compBtn = mkPBtn('ui.subtab', 'Compensado', false, () => {});
  tFoot.append(refLbl, refDown, refTxt, refUp, sp2, eqBtn, compBtn);
  tunerCard.appendChild(tFoot);

  // Looper Card (flex: 1)
  const looperCard = document.createElement('section');
  looperCard.style.cssText = `${pWellCss}flex:1;padding:16px 20px;display:flex;flex-direction:column;gap:12px;min-width:0;`;

  const lHdr = document.createElement('div');
  lHdr.style.cssText = 'display:flex;align-items:center;gap:8px;';
  lHdr.innerHTML = '<span class="m-lbl" style="flex:1;font-size:10px;letter-spacing:.12em">Looper</span>';
  lHdr.append(
    mkPBtn('looper.position', 'PRE', looperPos === 'pre', () => store.setControl('looper.position', 'pre')),
    mkPBtn('looper.position', 'POST', looperPos === 'post', () => store.setControl('looper.position', 'post'))
  );
  looperCard.appendChild(lHdr);

  const lBody = document.createElement('div');
  lBody.style.cssText = 'flex:1;display:flex;align-items:center;gap:22px;';

  const ring = document.createElement('div');
  ring.style.cssText =
    'width:190px;height:190px;border-radius:50%;background:conic-gradient(#7e868f 0deg,#1a1d22 0);display:flex;align-items:center;justify-content:center;flex-shrink:0;';
  const innerRing = document.createElement('div');
  innerRing.style.cssText =
    'width:160px;height:160px;border-radius:50%;background:#0a0b0c;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:4px;';
  const stateLabel =
    looperAction === 'rec' ? 'GRAVANDO' : looperAction === 'play' ? 'TOCANDO' : looperAction === 'overdub' ? 'OVERDUB' : 'VAZIO';
  innerRing.innerHTML = `
    <span class="m-osw" style="font-size:26px;font-weight:700;color:${looperAction === 'rec' ? '#ff3b2f' : '#7e868f'}">${stateLabel}</span>
    <span class="m-mono" style="font-size:13px;color:#a3abb4">0 %</span>
    <span class="m-mono" style="font-size:11px;color:#a3abb4">0 camada(s)</span>
  `;
  ring.appendChild(innerRing);

  const btnsGrid = document.createElement('div');
  btnsGrid.style.cssText = 'flex:1;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:8px;';
  for (const [act, lbl, dotCol, round] of [
    ['rec', 'Gravar', '#ff3b2f', true],
    ['play', 'Tocar', '#29c46a', false],
    ['undo', 'Desfazer', '#7e868f', false],
    ['redo', 'Refazer', '#7e868f', false],
    ['reverse', 'Reverso', '#3dd8ff', false],
    ['half_speed', '½ velocidade', '#3dd8ff', false],
    ['clear', 'Limpar', '#7e868f', false],
  ] as const) {
    const active = looperAction === act;
    const b = makeChipButton('looper.action', '', active, () => store.setControl('looper.action', act));
    b.style.cssText = `${pBtnCss(active)}height:40px;font-size:12px;display:flex;align-items:center;justify-content:center;gap:8px;`;
    b.innerHTML = `<span style="width:9px;height:9px;border-radius:${round ? '50%' : '2px'};background:${dotCol}"></span>${lbl}`;
    btnsGrid.appendChild(b);
  }
  lBody.append(ring, btnsGrid);
  looperCard.appendChild(lBody);

  mid.append(tunerCard, looperCard);
  stageWrap.appendChild(mid);

  // ── 3. Bottom Footswitch Dock (250 px) ─────────────────────────────────────
  const dock = document.createElement('section');
  dock.dataset.region = 'footswitch-dock';
  dock.style.cssText = `${pWellCss}height:250px;padding:14px 20px;display:flex;flex-direction:column;gap:12px;flex-shrink:0;box-sizing:border-box;`;

  const dHdr = document.createElement('div');
  dHdr.style.cssText = 'display:flex;align-items:center;gap:8px;';
  dHdr.innerHTML = '<span class="m-lbl" style="flex:1;font-size:10px;letter-spacing:.12em">Footswitches · modo</span>';
  for (const [m, lbl] of [
    ['preset', 'Preset'],
    ['scene', 'Scene'],
    ['stomp', 'Stomp'],
    ['hybrid', 'Híbrido'],
  ] as const) {
    dHdr.appendChild(
      mkPBtn(
        'ui.subtab',
        lbl,
        store.palcoFsMode === m,
        () => {
          store.palcoFsMode = m;
          store.emit();
        },
        'height:28px;padding:0 12px;font-size:11px;'
      )
    );
  }
  dock.appendChild(dHdr);

  const fsOuter = document.createElement('div');
  fsOuter.style.cssText = 'flex:1;display:flex;gap:14px;align-items:stretch;';
  const fsGrid = document.createElement('div');
  fsGrid.style.cssText = 'flex:1;display:grid;grid-template-columns:repeat(8,minmax(0,1fr));gap:12px;';

  const fsDefs = [
    { controlId: 'palco.fs_1_gate', paramKey: 'gate.enabled', tag: 'STOMP', lbl: 'Screamer 808', fs: 'FS 1' },
    { controlId: 'palco.fs_2_drive', paramKey: 'drive.enabled', tag: 'STOMP', lbl: 'Chorus-2', fs: 'FS 2' },
    { controlId: 'palco.fs_3_comp', paramKey: 'comp_2.enabled', tag: 'STOMP', lbl: 'Tape Echo', fs: 'FS 3' },
    { controlId: 'palco.fs_4_chorus', paramKey: 'mod_2.enabled', tag: 'STOMP', lbl: 'Plate', fs: 'FS 4' },
    { controlId: 'palco.fs_5_eq', paramKey: 'eq_1.enabled', tag: 'SCENE', lbl: 'A · Clean', fs: 'FS 5' },
    { controlId: 'palco.fs_6_delay', paramKey: 'dly_1.enabled', tag: 'SCENE', lbl: 'B · Crunch', fs: 'FS 6' },
    { controlId: 'palco.fs_7_reverb', paramKey: 'rev_1.enabled', tag: 'SCENE', lbl: 'C · Rhythm', fs: 'FS 7' },
    { controlId: 'palco.fs_8_split', paramKey: 'split.enabled', tag: 'SCENE', lbl: 'D · Solo', fs: 'FS 8' },
  ];

  for (const fs of fsDefs) {
    const active = bool(store, fs.paramKey, false);
    const col = document.createElement('div');
    col.style.cssText = `display:flex;flex-direction:column;align-items:center;gap:12px;opacity:${active ? 1 : 0.72};`;

    const oled = document.createElement('div');
    oled.style.cssText =
      'width:100%;height:58px;padding:5px 7px;box-sizing:border-box;background:#020304;border:1px solid #000;border-radius:4px;box-shadow:inset 0 1px 4px #000;display:flex;flex-direction:column;justify-content:space-between;';
    oled.innerHTML = `
      <div class="m-mono" style="display:flex;font-size:9px;color:#7fd8ff"><span>${fs.fs}</span><span style="flex:1"></span><span>${fs.tag}</span></div>
      <span style="font-size:13px;font-weight:700;color:#e8f6ff;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">${fs.lbl}</span>
    `;

    const ringWrap = document.createElement('div');
    ringWrap.style.cssText = `position:relative;width:90px;height:90px;display:flex;align-items:center;justify-content:center;border-radius:50%;box-shadow:0 0 0 4px ${
      active ? '#29c46a' : '#2a2e33'
    },0 0 22px ${active ? 'rgba(41,196,106,.7)' : 'transparent'};`;

    const btn = document.createElement('button');
    btn.type = 'button';
    btn.setAttribute('data-control-id', fs.controlId);
    btn.setAttribute('aria-pressed', String(active));
    btn.setAttribute('aria-label', `Footswitch ${fs.fs}: ${fs.lbl}`);
    btn.style.cssText =
      'width:66px;height:66px;border-radius:50%;border:0;padding:0;cursor:pointer;background:radial-gradient(circle at 38% 32%,#ffffff 0%,#d6d6d6 22%,#909090 55%,#4a4a4a 82%,#2a2a2a 100%);box-shadow:0 0 0 6px #8f949a,0 0 0 7px #2a2a2a,0 8px 14px rgba(0,0,0,.7);';
    btn.addEventListener('click', () => void store.setControl(fs.controlId, !active));

    ringWrap.appendChild(btn);
    col.append(oled, ringWrap);
    fsGrid.appendChild(col);
  }

  // EXP 1 ribbed treadle
  const expBox = document.createElement('div');
  expBox.style.cssText = 'width:120px;display:flex;flex-direction:column;align-items:center;gap:8px;';
  expBox.innerHTML = '<span class="m-lbl" style="font-size:9px;text-align:center;line-height:1.15">EXP 1 · WAH /<br>VOLUME</span>';

  const treadleWrap = document.createElement('div');
  treadleWrap.setAttribute('data-control-id', 'palco.exp_1');
  treadleWrap.setAttribute('role', 'slider');
  treadleWrap.tabIndex = 0;
  treadleWrap.setAttribute('aria-label', 'Pedal de Expressão EXP-1');
  treadleWrap.setAttribute('aria-valuemin', '0');
  treadleWrap.setAttribute('aria-valuemax', '100');
  treadleWrap.setAttribute('aria-valuenow', String(expVal));
  treadleWrap.style.cssText = 'flex:1;width:92px;position:relative;cursor:ns-resize;touch-action:none;perspective:300px;';
  const expTilt = Math.round(((expVal - 50) / 50) * 8);
  treadleWrap.innerHTML = `
    <div style="position:absolute;inset:0;border-radius:10px;background:linear-gradient(180deg,#2c3035,#121417);box-shadow:inset 0 2px 6px #000"></div>
    <div style="position:absolute;left:8px;right:8px;top:10px;bottom:10px;border-radius:8px;background:repeating-linear-gradient(0deg,#0d0d0d 0 3px,#2a2a2a 3px 5px);box-shadow:0 6px 10px rgba(0,0,0,.7);transform:rotateX(${expTilt}deg);transform-origin:50% 100%"></div>
  `;

  let expDrag = false;
  let startY = 0;
  let startExp = expVal;
  treadleWrap.addEventListener('pointerdown', (e) => {
    expDrag = true;
    startY = e.clientY;
    startExp = num(store, 'palco.exp_1', 62);
    try {
      treadleWrap.setPointerCapture(e.pointerId);
    } catch {
      // ignore
    }
  });
  treadleWrap.addEventListener('pointermove', (e) => {
    if (!expDrag) return;
    const next = Math.round(Math.max(0, Math.min(100, startExp + (startY - e.clientY) * 0.7)));
    void store.setControl('palco.exp_1', next);
  });
  treadleWrap.addEventListener('pointerup', () => {
    expDrag = false;
  });

  const expTxt = document.createElement('span');
  expTxt.className = 'm-mono';
  expTxt.style.cssText = 'font-size:12px;font-weight:700;color:#e8eaed;';
  expTxt.textContent = `${Math.round(expVal)} %`;
  expBox.append(treadleWrap, expTxt);

  fsOuter.append(fsGrid, expBox);
  dock.appendChild(fsOuter);
  stageWrap.appendChild(dock);
  root.appendChild(stageWrap);
  return root;
}
