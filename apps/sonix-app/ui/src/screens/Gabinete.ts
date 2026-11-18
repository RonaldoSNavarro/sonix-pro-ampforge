import type { AppStore } from '../store.ts';
import { bool, makeChipButton, num, str } from './_helpers.ts';

function makeCabSlider(
  store: AppStore,
  controlId: string,
  paramKey: string,
  label: string,
  min: number,
  max: number,
  step: number,
  def: number,
  color: string,
  fmt: (v: number) => string
): HTMLElement {
  const val = num(store, paramKey, def);
  const pct = Math.max(0, Math.min(100, ((val - min) / (max - min)) * 100));

  const row = document.createElement('div');
  row.style.cssText = 'display:grid;grid-template-columns:56px 1fr 56px;align-items:center;gap:8px;';

  const lbl = document.createElement('span');
  lbl.style.cssText = 'font-size:10px;color:#aeb4bc;';
  lbl.textContent = label;

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
  fill.style.cssText = `width:${pct.toFixed(1)}%;background:${color};`;
  const thumb = document.createElement('div');
  thumb.className = 'm-thumb';
  thumb.style.left = `${pct.toFixed(1)}%`;
  track.append(fill, thumb);

  const valTxt = document.createElement('span');
  valTxt.className = 'm-mono';
  valTxt.style.cssText = 'font-size:10px;color:#e6eaef;text-align:right;';
  valTxt.textContent = fmt(val);

  const commit = (raw: number) => {
    const snapped = Math.round(Math.max(min, Math.min(max, raw)) / step) * step;
    const clean = Number(snapped.toFixed(4));
    const p = Math.max(0, Math.min(100, ((clean - min) / (max - min)) * 100));
    fill.style.width = `${p.toFixed(1)}%`;
    thumb.style.left = `${p.toFixed(1)}%`;
    valTxt.textContent = fmt(clean);
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

  row.append(lbl, track, valTxt);
  return row;
}

export function renderGabineteScreen(store: AppStore): HTMLElement {
  const root = document.createElement('div');
  root.style.cssText = 'flex:1;min-height:0;display:flex;flex-direction:column;background:#0b0c0e;overflow:hidden;';

  const cabSpecs: Record<
    string,
    { title: string; cones: number; f0: number; badge: string; note: string }
  > = {
    greenback_4x12: {
      title: '4×12 Greenback',
      cones: 4,
      f0: 75,
      badge: 'GREENBACK 25 W · 16 Ω',
      note: 'Caixa fechada 4×12, falantes de 25 W com breakup cedo nos médios.',
    },
    v30_4x12: {
      title: '4×12 V30',
      cones: 4,
      f0: 85,
      badge: 'V30 · 60 W · 16 Ω',
      note: 'Caixa fechada 4×12 com pico de presença característico em 2–4 kHz.',
    },
    alnico_blue_2x12: {
      title: '2×12 Alnico Blue',
      cones: 2,
      f0: 80,
      badge: 'ALNICO BLUE · 15 W · 8 Ω',
      note: 'Ímã de alnico, agudos brilhantes, comprime cedo em volume alto.',
    },
    twin_2x12_open: {
      title: '2×12 aberto (Twin)',
      cones: 2,
      f0: 90,
      badge: 'C12N · 50 W · 8 Ω',
      note: 'Caixa aberta atrás: menos graves profundos, som mais espalhado.',
    },
  };

  const activeSpeaker = str(store, 'cab_1.speaker', 'v30_4x12');
  const cab = cabSpecs[activeSpeaker] ?? cabSpecs.v30_4x12;

  // ── Top Bar ────────────────────────────────────────────────────────────────
  const topBar = document.createElement('div');
  topBar.style.cssText =
    'height:56px;flex:none;display:flex;align-items:center;gap:14px;padding:0 18px;background:linear-gradient(180deg,#24282e 0%,#181b20 45%,#101215 100%);border-bottom:1px solid #000;';

  const backBtn = document.createElement('button');
  backBtn.className = 'm-hw';
  backBtn.style.cssText = 'height:34px;padding:0 14px;font-size:12px;display:flex;align-items:center;cursor:pointer;';
  backBtn.setAttribute('data-control-id', 'ui.nav_screen');
  backBtn.textContent = '‹ Rig';
  backBtn.addEventListener('click', () => store.setScreen('main'));

  const dot = document.createElement('span');
  dot.style.cssText = 'width:9px;height:9px;border-radius:50%;background:#ffc400;box-shadow:0 0 8px #ffc400;flex:none;';

  const title = document.createElement('span');
  title.className = 'm-osw';
  title.style.cssText = 'font-size:20px;font-weight:600;letter-spacing:.05em;color:#e8eaed;text-transform:uppercase;white-space:nowrap;';
  title.textContent = `GABINETE · ${cab.title.toUpperCase()}`;

  const spacer = document.createElement('span');
  spacer.style.flex = '1';

  const cabPickRow = document.createElement('div');
  cabPickRow.style.cssText = 'display:flex;align-items:center;gap:6px;';
  const cabLbl = document.createElement('span');
  cabLbl.className = 'm-lbl';
  cabLbl.style.cssText = 'font-size:9px;margin-right:4px;';
  cabLbl.textContent = 'Caixa';
  cabPickRow.appendChild(cabLbl);
  for (const [id, info] of Object.entries(cabSpecs)) {
    const b = makeChipButton('cab.speaker', info.title, activeSpeaker === id, () => store.setControl('cab.speaker', id));
    cabPickRow.appendChild(b);
  }
  topBar.append(backBtn, dot, title, spacer, cabPickRow);
  root.appendChild(topBar);

  // ── Main Content Split ─────────────────────────────────────────────────────
  const main = document.createElement('div');
  main.style.cssText = 'flex:1;min-height:0;display:grid;grid-template-columns:minmax(0,1fr) 420px;overflow:hidden;';

  // Left Visual Speaker Arena (matches Gabinete.dc.html)
  const left = document.createElement('div');
  left.style.cssText =
    'position:relative;display:flex;gap:18px;padding:18px;min-height:0;background:radial-gradient(ellipse at 40% 20%,rgba(255,220,160,.08) 0%,transparent 55%),linear-gradient(180deg,#121315,#0a0b0c);overflow:hidden;';

  // Left subcolumn (150px): Falante 2x2/2x1 + Legenda at bottom
  const leftCol = document.createElement('div');
  leftCol.style.cssText = 'display:flex;flex-direction:column;gap:12px;width:150px;flex:none;';
  leftCol.innerHTML = '<span class="m-lbl" style="font-size:9px">Falante</span>';

  const gridCones = document.createElement('div');
  gridCones.style.cssText = `width:150px;height:${cab.cones === 4 ? 150 : 75}px;border-radius:8px;padding:9px;box-sizing:border-box;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:8px;background:radial-gradient(circle at 1px 1px,rgba(255,255,255,.07) 1px,transparent 1.6px) 0 0/4px 4px,linear-gradient(180deg,#1d1d1d,#0f0f0f);box-shadow:0 14px 22px rgba(0,0,0,.6),0 0 0 1px #000;`;
  for (let i = 0; i < cab.cones; i++) {
    const c = document.createElement('div');
    c.style.cssText = `aspect-ratio:1;border-radius:50%;border:2px solid ${i === 0 ? '#ffc400' : '#2a2e35'};background:radial-gradient(circle,#5a554c 0 16%,#24211d 18%,#2e2a25 55%,#111 66%,#3a3a3a 70%,#161616 100%);`;
    gridCones.appendChild(c);
  }
  const spkNote = document.createElement('span');
  spkNote.style.cssText = 'font-size:11px;color:#a3abb4;line-height:1.5;';
  spkNote.textContent = cab.note;
  const spFlex = document.createElement('div');
  spFlex.style.flex = '1';
  const legend = document.createElement('div');
  legend.style.cssText = 'display:flex;flex-direction:column;gap:6px;';
  legend.innerHTML = `
    <span class="m-lbl" style="font-size:9px">Legenda</span>
    <span style="font-size:11px;color:#c9ced4;display:flex;align-items:center;gap:6px"><span style="width:10px;height:10px;border-radius:50%;background:#3dd8ff"></span>MIC 1</span>
    <span style="font-size:11px;color:#c9ced4;display:flex;align-items:center;gap:6px"><span style="width:10px;height:10px;border-radius:50%;background:#ff8a3d"></span>MIC 2</span>
    <span style="font-size:11px;color:#a3abb4;line-height:1.5">Arraste o microfone sobre o cone · roda do mouse = distância · duplo clique = 0°/45°</span>
  `;
  leftCol.append(gridCones, spkNote, spFlex, legend);
  left.appendChild(leftCol);

  // Center Speaker Col (exact 560x560 geometry at top)
  const spkCenterCol = document.createElement('div');
  spkCenterCol.style.cssText = 'flex:1;display:flex;flex-direction:column;align-items:center;justify-content:space-between;gap:4px;min-width:0;min-height:0;';

  const arenaScaleWrap = document.createElement('div');
  arenaScaleWrap.style.cssText = 'position:relative;width:516px;height:516px;flex-shrink:0;';

  const arenaBox = document.createElement('div');
  arenaBox.style.cssText = 'position:absolute;left:0;top:0;width:560px;height:560px;transform:scale(0.92);transform-origin:top left;';

  const baffle = document.createElement('div');
  baffle.style.cssText =
    'position:absolute;inset:-14px;border-radius:12px;background:radial-gradient(circle at 1px 1px,rgba(255,255,255,.04) 1px,transparent 1.5px) 0 0/3px 3px,#0c0c0c;box-shadow:inset 0 4px 14px rgba(0,0,0,.9);';

  const basket = document.createElement('div');
  basket.style.cssText =
    'position:absolute;inset:0;border-radius:50%;background:radial-gradient(circle,#202020 0 66%,#5d5d5d 67.5%,#2a2a2a 69.5%,#454545 73%,#1a1a1a 100%);box-shadow:0 20px 40px rgba(0,0,0,.8);';

  for (let i = 0; i < 8; i++) {
    const a = (i * Math.PI) / 4 + Math.PI / 8;
    const bx = 280 + 266 * Math.cos(a);
    const by = 280 + 266 * Math.sin(a);
    const bolt = document.createElement('span');
    bolt.style.cssText = `position:absolute;left:${bx.toFixed(1)}px;top:${by.toFixed(1)}px;width:14px;height:14px;margin:-7px 0 0 -7px;border-radius:50%;background:radial-gradient(circle at 35% 30%,#e4e4e4,#7a7a7a 55%,#2a2a2a);`;
    basket.appendChild(bolt);
  }

  const badge = document.createElement('span');
  badge.className = 'm-mono';
  badge.style.cssText =
    'position:absolute;left:50%;bottom:20px;transform:translateX(-50%);font-size:10px;letter-spacing:.2em;color:#b8b2a2;background:#1a1a1a;padding:2px 8px;border-radius:3px;z-index:2;';
  badge.textContent = cab.badge;
  basket.appendChild(badge);

  const surround = document.createElement('div');
  surround.style.cssText =
    'position:absolute;inset:44px;border-radius:50%;background:radial-gradient(circle,transparent 0 85%,#2c2a27 86%,#57524a 90%,#2b2824 94%,#0e0d0b 100%);';

  const cone = document.createElement('div');
  cone.style.cssText =
    'position:absolute;inset:62px;border-radius:50%;background:conic-gradient(from 210deg,rgba(255,255,255,.07),transparent 22%,rgba(0,0,0,.25) 50%,transparent 78%,rgba(255,255,255,.07)),repeating-radial-gradient(circle,rgba(255,255,255,.025) 0 2px,transparent 2px 10px),radial-gradient(circle at 50% 45%,#47423a 0%,#2a2620 55%,#161410 100%);box-shadow:inset 0 0 30px rgba(0,0,0,.7);';

  const dustCap = document.createElement('div');
  dustCap.style.cssText =
    'position:absolute;inset:214px;border-radius:50%;background:radial-gradient(circle at 40% 34%,#6b655a 0%,#36322b 55%,#16140f 100%);box-shadow:0 6px 12px rgba(0,0,0,.6),inset 0 2px 3px rgba(255,255,255,.12);';

  arenaBox.append(baffle, basket, surround, cone);
  for (const ins of ['206px', '138px', '80px']) {
    const r = document.createElement('div');
    r.style.cssText = `position:absolute;inset:${ins};border-radius:50%;border:1px dashed rgba(255,255,255,${ins === '206px' ? '.18' : ins === '138px' ? '.12' : '.1'});pointer-events:none;`;
    arenaBox.appendChild(r);
  }
  arenaBox.appendChild(dustCap);
  arenaBox.innerHTML +=
    '<span style="position:absolute;left:286px;top:262px;font-size:9px;color:rgba(255,255,255,.35);pointer-events:none">calota</span>' +
    '<span style="position:absolute;left:356px;top:262px;font-size:9px;color:rgba(255,255,255,.3);pointer-events:none">borda calota</span>' +
    '<span style="position:absolute;left:426px;top:262px;font-size:9px;color:rgba(255,255,255,.28);pointer-events:none">cone</span>';

  const coneEl = arenaBox.children[3] as HTMLElement;

  // Draggable Microphones 1 & 2 matching Gabinete.dc.html coords
  const micStyles: Record<string, { type: string; w: number; h: number; r: string; bg: string; mesh: string }> = {
    sm57: {
      type: 'dinâmico',
      w: 50,
      h: 50,
      r: '50%',
      bg: 'linear-gradient(145deg,#3b4046,#181a1d)',
      mesh: 'repeating-linear-gradient(90deg,rgba(255,255,255,.12) 0 2px,transparent 2px 4px)',
    },
    md421: { type: 'dinâmico', w: 54, h: 36, r: '8px', bg: 'linear-gradient(180deg,#2a2a2a,#0e0e0e)', mesh: '' },
    m88: { type: 'dinâmico', w: 46, h: 46, r: '50%', bg: 'radial-gradient(circle at 40% 35%,#c9cdd1,#7d838a 70%)', mesh: '' },
    r121: {
      type: 'fita',
      w: 36,
      h: 64,
      r: '18px',
      bg: 'linear-gradient(90deg,#7b8189,#dfe4ea 45%,#8d949c)',
      mesh: 'repeating-linear-gradient(0deg,rgba(0,0,0,.35) 0 2px,transparent 2px 4px)',
    },
    m160: { type: 'fita', w: 40, h: 40, r: '50%', bg: 'radial-gradient(circle at 40% 35%,#d9dde1,#80868d 70%)', mesh: '' },
    u87: { type: 'condensador', w: 52, h: 52, r: '50%', bg: 'radial-gradient(circle at 40% 35%,#eceef0,#9aa0a6 70%)', mesh: '' },
    c414: { type: 'condensador', w: 48, h: 48, r: '10px', bg: 'linear-gradient(180deg,#2a2a2a,#111)', mesh: '' },
  };

  const micMeta = [1, 2].map((idx) => {
    const pos = num(store, `cab_1.mic${idx}.position`, idx === 1 ? 0.16 : 0.68);
    const dist = num(store, `cab_1.mic${idx}.distance_cm`, idx === 1 ? 2.0 : 8.0);
    const angle = Number(str(store, `cab_1.mic${idx}.angle_deg`, idx === 1 ? '0' : '45'));
    const model = str(store, `cab_1.mic${idx}.model`, idx === 1 ? 'sm57' : 'r121');
    const en = bool(store, `cab_1.mic${idx}.enabled`, true);
    const inv = bool(store, `cab_1.mic${idx}.phase_invert`, false);
    const col = idx === 1 ? '#3dd8ff' : '#ff8a3d';
    const ms = micStyles[model] ?? micStyles.sm57;

    const unitX = idx === 1 ? 0.86 : 0.88;
    const unitY = idx === 1 ? -0.5 : 0.47;
    const mx = 280 + pos * 220 * unitX;
    const my = 280 + pos * 220 * unitY;
    const hw = ms.w / 2;
    const hh = ms.h / 2;
    const sc = 1 - Math.min(dist, 30) * 0.008;

    const sh = document.createElement('div');
    sh.style.cssText = `position:absolute;left:${(mx + dist * 0.9).toFixed(1)}px;top:${(my + dist * 0.9).toFixed(1)}px;width:${ms.w}px;height:${ms.h}px;margin:-${hh}px 0 0 -${hw}px;border-radius:${ms.r};background:rgba(0,0,0,.55);filter:blur(${Math.round(3 + dist * 0.4)}px);pointer-events:none;`;

    const irOn = bool(store, 'cab_1.ir_enabled', false);
    const irMicHint = 'IR ativa — microfonação vem da IR';
    const node = document.createElement('button');
    node.type = 'button';
    node.disabled = irOn;
    if (irOn) node.title = irMicHint;
    node.setAttribute('data-mic-id', String(idx));
    node.setAttribute('data-control-id', `cab.mic${idx}.position`);
    node.setAttribute('role', 'slider');
    node.tabIndex = 0;
    node.setAttribute('aria-label', `MIC ${idx} ${model.toUpperCase()} — posição ${Math.round(pos * 100)}%, distância ${dist.toFixed(1)} cm`);
    node.setAttribute('aria-valuemin', '0');
    node.setAttribute('aria-valuemax', '1');
    node.setAttribute('aria-valuenow', String(pos));
    node.style.cssText = `position:absolute;border:0;padding:0;cursor:grab;touch-action:none;display:flex;align-items:center;justify-content:center;left:${mx.toFixed(1)}px;top:${my.toFixed(1)}px;width:${ms.w}px;height:${ms.h}px;margin:-${hh}px 0 0 -${hw}px;border-radius:${ms.r};background:${ms.bg};box-shadow:0 0 0 3px ${col},0 0 18px ${col};transform:rotate(${angle}deg) scale(${sc.toFixed(3)});opacity:${en ? 1 : 0.45};z-index:4;`;
    if (ms.mesh) {
      node.innerHTML = `<span style="position:absolute;inset:5px;border-radius:${ms.r};background:${ms.mesh};"></span>`;
    }

    const tagSpan = document.createElement('span');
    tagSpan.className = 'm-mono';
    tagSpan.style.cssText = `position:absolute;left:${mx.toFixed(1)}px;top:${(my + hh + 6).toFixed(1)}px;transform:translateX(-50%);font-size:10px;font-weight:700;color:#111;background:${col};padding:1px 6px;border-radius:3px;white-space:nowrap;pointer-events:none;z-index:5;`;
    tagSpan.textContent = `MIC ${idx} · ${model.toUpperCase()}`;

    let dragging = false;
    node.addEventListener('pointerdown', (e) => {
      dragging = true;
      store.setDraggingKnob(true);
      try {
        node.setPointerCapture(e.pointerId);
      } catch {
        // ignore
      }
    });
    node.addEventListener('pointermove', (e) => {
      if (!dragging) return;
      const r = coneEl.getBoundingClientRect();
      if (r.width <= 0) return;
      const nx = (e.clientX - (r.left + r.width / 2)) / (r.width / 2);
      const ny = (e.clientY - (r.top + r.height / 2)) / (r.height / 2);
      const norm = Number(Math.max(0, Math.min(1, Math.hypot(nx, ny))).toFixed(2));
      const newMx = 280 + norm * 220 * unitX;
      const newMy = 280 + norm * 220 * unitY;
      node.style.left = `${newMx.toFixed(1)}px`;
      node.style.top = `${newMy.toFixed(1)}px`;
      node.setAttribute('aria-valuenow', String(norm));
      node.setAttribute('aria-label', `MIC ${idx} ${model.toUpperCase()} — posição ${Math.round(norm * 100)}%, distância ${dist.toFixed(1)} cm`);
      sh.style.left = `${(newMx + dist * 0.9).toFixed(1)}px`;
      sh.style.top = `${(newMy + dist * 0.9).toFixed(1)}px`;
      tagSpan.style.left = `${newMx.toFixed(1)}px`;
      tagSpan.style.top = `${(newMy + hh + 6).toFixed(1)}px`;
      const spEl = root.querySelector(`[data-mic-readout="${idx}"]`);
      if (spEl) {
        spEl.textContent = `MIC ${idx}: ${Math.round(norm * 100)}% da calota · ${dist.toFixed(1)} cm · ${angle}° · ${(dist / 34.3).toFixed(2)} ms`;
      }
      void store.setControl(`cab.mic${idx}.position`, norm);
    });
    const endMicDrag = () => {
      if (!dragging) return;
      dragging = false;
      store.setDraggingKnob(false);
    };
    node.addEventListener('pointerup', endMicDrag);
    node.addEventListener('pointercancel', endMicDrag);
    node.addEventListener(
      'wheel',
      (e) => {
        e.preventDefault();
        const next = Math.max(0, Math.min(30, dist + (e.deltaY < 0 ? -0.5 : 0.5)));
        void store.setControl(`cab.mic${idx}.distance_cm`, Number(next.toFixed(1)));
      },
      { passive: false }
    );
    node.addEventListener('dblclick', () => {
      void store.setControl(`cab.mic${idx}.angle_deg`, angle === 45 ? '0' : '45');
    });

    arenaBox.append(sh, node, tagSpan);
    return { idx, pos, dist, angle, model, type: ms.type, en, inv, col };
  });

  arenaScaleWrap.appendChild(arenaBox);
  spkCenterCol.appendChild(arenaScaleWrap);

  const micLine = document.createElement('div');
  micLine.className = 'm-mono';
  micLine.dataset.region = 'mic-readout-bar';
  micLine.style.cssText = 'display:flex;flex-wrap:wrap;justify-content:center;gap:6px 18px;font-size:11px;line-height:1.35;z-index:6;padding:2px 8px;border-radius:4px;background:rgba(10,11,12,.85);';
  for (const m of micMeta) {
    const sp = document.createElement('span');
    sp.dataset.micReadout = String(m.idx);
    sp.style.color = m.col;
    sp.style.whiteSpace = 'nowrap';
    sp.textContent = `MIC ${m.idx}: ${Math.round(m.pos * 100)}% da calota · ${m.dist.toFixed(1)} cm · ${m.angle}° · ${(m.dist / 34.3).toFixed(2)} ms`;
    micLine.appendChild(sp);
  }
  spkCenterCol.appendChild(micLine);
  left.appendChild(spkCenterCol);

  // ── Right Panel (420px) ────────────────────────────────────────────────────
  const side = document.createElement('aside');
  side.style.cssText =
    'background:linear-gradient(180deg,#24282e 0%,#181b20 45%,#101215 100%);border-left:1px solid #000;display:flex;flex-direction:column;min-height:0;overflow:hidden;';

  const tabs = document.createElement('div');
  tabs.style.cssText = 'display:flex;border-bottom:1px solid #000;';
  for (const [id, lbl] of [
    ['mics', 'Microfones'],
    ['cab', 'Caixa e filtros'],
    ['ir', 'IR'],
  ] as const) {
    const on = store.cabTab === id;
    const b = makeChipButton('ui.subtab', lbl, on, () => {
      store.cabTab = id;
      store.emit();
    });
    b.style.cssText = `flex:1;height:38px;border:0;border-bottom:2px solid ${on ? '#ff8a3d' : 'transparent'};background:transparent;color:${on ? '#e8eaed' : '#9aa3ad'};font-size:12px;font-weight:600;cursor:pointer;justify-content:center;`;
    tabs.appendChild(b);
  }
  side.appendChild(tabs);

  const sideBody = document.createElement('div');
  sideBody.style.cssText = 'flex:1;min-height:0;padding:12px 16px;display:flex;flex-direction:column;gap:14px;overflow:hidden;';

  const irActive = bool(store, 'cab_1.ir_enabled', false);
  const irMicHint = 'IR ativa — microfonação vem da IR';

  if (store.cabTab === 'mics') {
    if (irActive) {
      const banner = document.createElement('div');
      banner.style.cssText = 'padding:6px 10px;border-radius:6px;background:rgba(255,138,61,.12);border:1px solid rgba(255,138,61,.4);color:#ffb074;font-size:11px;';
      banner.textContent = irMicHint;
      sideBody.appendChild(banner);
    }
    for (const m of micMeta) {
      const card = document.createElement('div');
      card.style.cssText = `border-radius:8px;padding:10px 12px;background:#0b0c0e;border:1px solid #1f242b;border-left:3px solid ${m.col};display:flex;flex-direction:column;gap:8px;opacity:${irActive ? 0.55 : 1};`;

      const hdr = document.createElement('div');
      hdr.style.cssText = 'display:flex;align-items:center;gap:8px;';
      hdr.innerHTML = `<span class="m-osw" style="font-size:14px;font-weight:600;color:${m.col}">MIC ${m.idx}</span><span style="font-size:11px;color:#a3abb4">${m.model.toUpperCase()} · ${m.type}</span>`;

      const utilBtns = document.createElement('div');
      utilBtns.style.cssText = 'margin-left:auto;display:flex;gap:4px;';
      const muteBtn = makeChipButton(`cab.mic${m.idx}.enabled`, 'M', !m.en, () => store.setControl(`cab.mic${m.idx}.enabled`, !m.en));
      muteBtn.style.cssText += ';height:20px;padding:0 6px;font-size:10px;';
      muteBtn.disabled = irActive;
      if (irActive) muteBtn.title = irMicHint;
      const phBtn = makeChipButton(`cab.mic${m.idx}.phase_invert`, 'Ø', m.inv, () => store.setControl(`cab.mic${m.idx}.phase_invert`, !m.inv));
      phBtn.style.cssText += ';height:20px;padding:0 6px;font-size:10px;';
      phBtn.disabled = irActive;
      if (irActive) phBtn.title = irMicHint;
      utilBtns.append(muteBtn, phBtn);
      hdr.appendChild(utilBtns);
      card.appendChild(hdr);

      const modRow = document.createElement('div');
      modRow.style.cssText = 'display:flex;flex-wrap:wrap;gap:3px;';
      for (const mk of ['sm57', 'md421', 'm88', 'r121', 'm160', 'u87', 'c414']) {
        const b = makeChipButton(`cab.mic${m.idx}.model`, mk.toUpperCase(), m.model === mk, () => store.setControl(`cab.mic${m.idx}.model`, mk));
        b.style.cssText += ';height:20px;padding:0 5px;font-size:9.5px;';
        b.disabled = irActive;
        if (irActive) b.title = irMicHint;
        modRow.appendChild(b);
      }
      card.appendChild(modRow);

      const lvlSlider = makeCabSlider(store, `cab.mic${m.idx}.level_db`, `cab_1.mic${m.idx}.level_db`, 'Nível', -60, 12, 0.5, m.idx === 1 ? 0 : -6, m.col, (v) => `${v.toFixed(1)} dB`);
      const distSlider = makeCabSlider(store, `cab.mic${m.idx}.distance_cm`, `cab_1.mic${m.idx}.distance_cm`, 'Distância', 0, 30, 0.5, m.idx === 1 ? 2 : 8, m.col, (v) => `${v.toFixed(1)} cm`);
      if (irActive) {
        lvlSlider.querySelectorAll('input').forEach((inp) => { inp.disabled = true; inp.title = irMicHint; });
        distSlider.querySelectorAll('input').forEach((inp) => { inp.disabled = true; inp.title = irMicHint; });
      }
      card.appendChild(lvlSlider);
      card.appendChild(distSlider);

      const axRow = document.createElement('div');
      axRow.style.cssText = 'display:flex;gap:4px;align-items:center;';
      axRow.innerHTML = '<span class="m-lbl" style="font-size:8.5px;width:70px">Eixo</span>';
      for (const [degVal, lbl] of [
        ['0', '0° on-axis'],
        ['45', '45° off-axis'],
      ] as const) {
        const b = makeChipButton(`cab.mic${m.idx}.angle_deg`, lbl, String(m.angle) === degVal, () => store.setControl(`cab.mic${m.idx}.angle_deg`, degVal));
        b.style.cssText += ';height:20px;padding:0 6px;font-size:10px;';
        b.disabled = irActive;
        if (irActive) b.title = irMicHint;
        axRow.appendChild(b);
      }
      card.appendChild(axRow);
      sideBody.appendChild(card);
    }
  } else if (store.cabTab === 'cab') {
    const cabOn = bool(store, 'cab_1.enabled', true);
    const tog = makeChipButton('cab.enabled', cabOn ? 'GABINETE ATIVO' : 'GABINETE BYPASS', cabOn, () =>
      store.setControl('cab.enabled', !cabOn)
    );
    sideBody.appendChild(tog);
    sideBody.appendChild(
      makeCabSlider(store, 'cab.resonance_hz', 'cab_1.resonance_hz', 'Ressonância', 55, 160, 1, 95, '#ff8a3d', (v) => `${Math.round(v)} Hz`)
    );
    sideBody.appendChild(
      makeCabSlider(store, 'cab.hicut_khz', 'cab_1.hicut_khz', 'High cut', 2.0, 20.0, 0.2, 10.5, '#ff8a3d', (v) => `${v.toFixed(1)} kHz`)
    );
  } else {
    const irOn = bool(store, 'cab_1.ir_enabled', false);
    const irCard = document.createElement('div');
    irCard.style.cssText =
      'border-radius:8px;padding:12px;background:#0b0c0e;border:1px solid #1f242b;display:flex;flex-direction:column;gap:8px;';
    irCard.innerHTML = `<span style="font-size:12px;font-weight:600;color:#e6eaef">${store.irInfo.fileName}</span><span class="m-mono" style="font-size:10px;color:#8a939e">${store.irInfo.sampleRate} Hz · mono · sem normalização</span>`;
    const irRow = document.createElement('div');
    irRow.style.cssText = 'display:flex;gap:8px;';
    const toggleIr = makeChipButton('cab.ir_enabled', irOn ? 'IR ATIVA' : 'IR DESLIGADA', irOn, () =>
      store.setControl('cab.ir_enabled', !irOn)
    );
    const loadIr = makeChipButton('cab.ir_load', 'Carregar IR (.wav)...', false, async () => {
      const inp = document.createElement('input');
      inp.type = 'file';
      inp.accept = '.wav';
      inp.onchange = async () => {
        const f = inp.files?.[0];
        if (!f) return;
        try {
          await store.transport.loadIr({ name: f.name });
          store.irInfo.fileName = f.name;
          await store.setControl('cab.ir_enabled', true);
          store.emit();
        } catch {
          store.emit('status');
        }
      };
      inp.click();
    });
    irRow.append(toggleIr, loadIr);
    irCard.appendChild(irRow);
    sideBody.appendChild(irCard);
  }

  side.appendChild(sideBody);

  // Bottom summed frequency curve
  const respSec = document.createElement('div');
  respSec.style.cssText = 'margin-top:auto;padding:10px 16px 14px;border-top:1px solid #000;display:flex;flex-direction:column;gap:6px;';
  const deltaCm = Math.abs(micMeta[0].dist - micMeta[1].dist);
  respSec.innerHTML = `
    <div style="display:flex;align-items:center">
      <span class="m-lbl" style="font-size:9px;flex:1">RESPOSTA RESULTANTE (MICS SOMADOS)</span>
      <span class="m-mono" style="font-size:9px;color:${deltaCm > 0.5 ? '#ffc400' : '#29c46a'}">${deltaCm > 0.5 ? `filtro pente: Δ ${deltaCm.toFixed(1)} cm entre mics` : 'mics alinhados'}</span>
    </div>
  `;
  const respBox = document.createElement('div');
  respBox.style.cssText = 'height:120px;border-radius:6px;background:#050607;border:1px solid #1f242b;position:relative;';
  const pts: string[] = [];
  for (let i = 0; i <= 76; i++) {
    const f = 50 * Math.pow(200, i / 76);
    const res = 5 * Math.exp(-Math.pow(Math.log2(f / cab.f0), 2) * 3);
    const pres = 3.5 * Math.exp(-Math.pow(Math.log2(f / 3000), 2) * 4);
    const phase = (2 * Math.PI * f * (deltaCm / 100)) / 343 + (micMeta[0].inv !== micMeta[1].inv ? Math.PI : 0);
    const comb = 4 * Math.log10(Math.max(0.08, Math.abs(Math.cos(phase / 2))) * 2);
    const hc = f > 8000 ? -12 * Math.log2(f / 8000) : 0;
    const lc = f < 80 ? -12 * Math.log2(80 / f) : 0;
    const y = Math.max(6, Math.min(114, 62 - (res + pres + comb + hc + lc) * 3));
    pts.push(`${i === 0 ? 'M' : 'L'}${i * 5} ${y.toFixed(1)}`);
  }
  respBox.innerHTML = `
    <svg viewBox="0 0 380 120" preserveAspectRatio="none" style="position:absolute;inset:0;width:100%;height:100%" aria-hidden="true">
      <g stroke="rgba(255,255,255,.06)"><path d="M0 30h380M0 60h380M0 90h380M95 0v120M190 0v120M285 0v120"/></g>
      <path d="${pts.join(' ')}" stroke="#1de9b6" stroke-width="2" fill="none" style="filter:drop-shadow(0 0 3px #1de9b6)"/>
    </svg>
    <span class="m-mono" style="position:absolute;left:8px;bottom:4px;font-size:9px;color:#8a939e">50 Hz</span>
    <span class="m-mono" style="position:absolute;right:8px;bottom:4px;font-size:9px;color:#8a939e">10 kHz</span>
  `;
  respSec.appendChild(respBox);
  side.appendChild(respSec);

  main.append(left, side);
  root.appendChild(main);
  return root;
}
