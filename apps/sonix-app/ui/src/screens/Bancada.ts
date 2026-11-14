import type { AppStore } from '../store';
import { createTubeElement } from '../components/Tube';
import {
  bool,
  makeChipButton,
  makeControlSelect,
  makeControlSlider,
  makePanelCard,
  num,
  str,
} from './_helpers';

const CHASSIS_TUBES: { id: string; stage: string; isPower: boolean }[] = [
  { id: 'V1', stage: 'Entrada / Triodo 1', isPower: false },
  { id: 'V2', stage: 'Ganho 2 / Cátodo', isPower: false },
  { id: 'V3', stage: 'Inversor Fase LTP', isPower: false },
  { id: 'V4', stage: 'Buffer / Loop FX', isPower: false },
  { id: 'V5', stage: 'Power Push A', isPower: true },
  { id: 'V6', stage: 'Power Push B', isPower: true },
  { id: 'V7', stage: 'Power Pull A', isPower: true },
  { id: 'V8', stage: 'Power Pull B', isPower: true },
];

function buildScopeSvg(store: AppStore): SVGSVGElement {
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 360 170');
  svg.style.cssText = 'width:100%;height:170px;display:block;';

  // Grid lines
  for (let x = 36; x < 360; x += 36) {
    const l = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    l.setAttribute('x1', String(x));
    l.setAttribute('y1', '0');
    l.setAttribute('x2', String(x));
    l.setAttribute('y2', '170');
    l.setAttribute('stroke', 'rgba(80,255,140,0.12)');
    svg.appendChild(l);
  }
  for (let y = 34; y < 170; y += 34) {
    const l = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    l.setAttribute('x1', '0');
    l.setAttribute('y1', String(y));
    l.setAttribute('x2', '360');
    l.setAttribute('y2', String(y));
    l.setAttribute('stroke', 'rgba(80,255,140,0.12)');
    svg.appendChild(l);
  }

  const pts: string[] = [];
  const samples = store.scope.waveform_out ?? store.telemetry.waveform ?? [];
  if (store.crtMode === 'wave') {
    if (samples.length > 4) {
      samples.slice(0, 128).forEach((s: number, i: number, arr: number[]) => {
        const x = (i / Math.max(1, arr.length - 1)) * 360;
        const y = 85 - Math.max(-1, Math.min(1, s)) * 68;
        pts.push(`${x.toFixed(1)},${y.toFixed(1)}`);
      });
    } else {
      const asym = num(store, 'amp_1.triode_asym', 0.2);
      const gain = num(store, 'amp_1.gain', 6.5) / 4;
      for (let i = 0; i <= 120; i += 1) {
        const ph = (i / 120) * Math.PI * 4;
        const raw = Math.sin(ph) * gain + asym * 0.25;
        const clipped = Math.tanh(raw) - asym * 0.1;
        pts.push(`${(i * 3).toFixed(1)},${(85 - clipped * 60).toFixed(1)}`);
      }
    }
  } else if (store.crtMode === 'transfer') {
    const xs = store.scope.transfer_x ?? [];
    const ys = store.scope.transfer_y ?? [];
    if (xs.length > 4 && xs.length === ys.length) {
      xs.forEach((xv, i) => {
        const px = ((xv + 1) * 0.5) * 360;
        const py = 85 - Math.max(-1, Math.min(1, ys[i])) * 62;
        pts.push(`${px.toFixed(1)},${py.toFixed(1)}`);
      });
    } else {
      const asym = num(store, 'amp_1.triode_asym', 0.2);
      const sat = num(store, 'amp_1.xformer_sat', 4) / 8;
      for (let i = 0; i <= 90; i += 1) {
        const u = (i / 90) * 2 - 1;
        const yVal = Math.tanh((u + asym * 0.35) * (1.8 + sat * 2)) - Math.tanh(asym * 0.35);
        pts.push(`${(i * 4).toFixed(1)},${(85 - yVal * 62).toFixed(1)}`);
      }
    }
  } else {
    const tsDb = store.scope.tonestack_db ?? [];
    if (tsDb.length > 4) {
      tsDb.forEach((db, i, arr) => {
        const x = (i / Math.max(1, arr.length - 1)) * 360;
        const y = 85 - Math.max(-24, Math.min(24, db)) * 2.4;
        pts.push(`${x.toFixed(1)},${y.toFixed(1)}`);
      });
    } else {
      const bass = num(store, 'amp_1.bass', 5.5) / 10;
      const mid = num(store, 'amp_1.middle', 5.0) / 10;
      const treb = num(store, 'amp_1.treble', 6.0) / 10;
      for (let i = 0; i <= 90; i += 1) {
        const f = i / 90;
        const curve =
          bass * Math.exp(-Math.pow((f - 0.16) * 4, 2)) +
          mid * Math.exp(-Math.pow((f - 0.45) * 4.2, 2)) * 0.8 +
          treb * Math.exp(-Math.pow((f - 0.82) * 3.5, 2));
        pts.push(`${(i * 4).toFixed(1)},${(148 - curve * 112).toFixed(1)}`);
      }
    }
  }

  const poly = document.createElementNS('http://www.w3.org/2000/svg', 'polyline');
  poly.setAttribute('fill', 'none');
  poly.setAttribute('stroke', '#5cff96');
  poly.setAttribute('stroke-width', '2.1');
  poly.setAttribute('points', pts.join(' '));
  svg.appendChild(poly);
  return svg;
}

export function renderBancadaScreen(store: AppStore): HTMLElement {
  const root = document.createElement('div');
  root.dataset.screen = 'chassi';
  root.style.cssText =
    'height:100%;display:grid;grid-template-columns:1.15fr 0.85fr;gap:10px;padding:10px;min-height:0;overflow:hidden;';

  // Left: Open Chassis + Turret Board + Transformers + V1..V8
  const chassis = document.createElement('section');
  chassis.className = 'alu';
  chassis.style.cssText =
    'border-radius:10px;border:1px solid #8a9099;padding:12px;display:flex;flex-direction:column;justify-content:space-between;gap:10px;min-height:0;overflow:hidden;';

  const hdr = document.createElement('div');
  hdr.style.cssText = 'display:flex;align-items:center;justify-content:space-between;color:#111418;';
  const preampTube = str(store, 'amp_1.preamp_tube', '12ax7').toUpperCase();
  const powerTube = str(store, 'amp_1.power_tube', 'el34').toUpperCase();
  const rect = str(store, 'amp_1.rectifier', 'diode').toUpperCase();
  const bPlus = Math.max(180, num(store, 'amp_1.preamp_plate_v', 280) * 1.5 - (store.telemetry.gain_reduction_db ?? 0) * 2).toFixed(0);
  hdr.innerHTML = `
    <div>
      <div class="m-osw" style="font-size:15px;font-weight:700;letter-spacing:0.08em;">BANCADA MOD SHOP · CHASSI ABERTO</div>
      <div class="m-mono" style="font-size:10.5px;color:#2b313a;">${preampTube} PRÉ · ${powerTube} POWER · RETIF. ${rect}</div>
    </div>
    <div class="m-lcd" style="padding:4px 10px;border-radius:5px;">
      <span class="m-lcdtxt m-mono" style="font-size:11px;font-weight:700;">B+ ${bPlus}V · SAG ${num(store, 'amp_1.sag', 3).toFixed(1)}</span>
    </div>
  `;

  // Physical Transformers + Caps + Tubes stage
  const hwStage = document.createElement('div');
  hwStage.style.cssText =
    'background:linear-gradient(180deg,#1a1d22,#0f1114);border-radius:8px;padding:12px;border:1px solid #30353e;display:flex;flex-direction:column;gap:12px;';

  const xfRow = document.createElement('div');
  xfRow.style.cssText = 'display:flex;align-items:flex-end;justify-content:space-between;gap:12px;';

  const pt = document.createElement('div');
  pt.className = 'xfmr';
  pt.style.cssText = 'width:86px;height:66px;padding:6px;display:flex;flex-direction:column;justify-content:space-between;color:#c2c7cf;';
  pt.innerHTML = '<div class="bell" style="height:10px;border-radius:2px;"></div><span class="m-mono" style="font-size:9px;text-align:center;">POWER PT</span>';

  const caps = document.createElement('div');
  caps.style.cssText = 'display:flex;gap:8px;align-items:flex-end;';
  for (const capLbl of ['100µF', '100µF', '47µF']) {
    const c = document.createElement('div');
    c.className = 'cap-electrolytic';
    c.style.cssText = 'width:26px;height:44px;display:flex;align-items:flex-end;justify-content:center;padding-bottom:3px;font-size:7px;color:#fff;';
    c.textContent = capLbl;
    caps.appendChild(c);
  }

  const tubesRow = document.createElement('div');
  tubesRow.style.cssText = 'display:flex;align-items:flex-end;gap:8px;flex:1;justify-content:center;';
  const isLit = bool(store, 'amp_1.power', true) && !bool(store, 'amp_1.standby', false);
  const glowOp = isLit ? 0.55 + Math.min(0.45, num(store, 'amp_1.bias_ma', 38) / 80) : 0.04;
  for (const t of CHASSIS_TUBES) {
    tubesRow.appendChild(
      createTubeElement({
        id: t.id,
        typeLabel: t.isPower ? powerTube.slice(0, 5) : preampTube.slice(0, 5),
        width: t.isPower ? 24 : 16,
        height: t.isPower ? 64 : 44,
        radius: t.isPower ? '11px 11px 3px 3px' : '8px 8px 2px 2px',
        baseWidth: t.isPower ? 28 : 18,
        baseHeight: t.isPower ? 14 : 8,
        baseBg: t.isPower ? 'linear-gradient(90deg,#1a1a1a,#3a3a3a,#151515)' : '#2b2e34',
        glowOp,
        glowPx: isLit ? 8 : 0,
        selected: store.selectedTube === t.id,
        controlId: 'ui.inspector_select',
        onClick: () => {
          store.selectedTube = t.id;
          store.emit();
        },
      })
    );
  }

  const ot = document.createElement('div');
  ot.className = 'xfmr';
  ot.style.cssText = 'width:86px;height:66px;padding:6px;display:flex;flex-direction:column;justify-content:space-between;color:#c2c7cf;';
  ot.innerHTML = `<div class="bell" style="height:10px;border-radius:2px;"></div><span class="m-mono" style="font-size:9px;text-align:center;">OT ${str(store, 'amp_1.impedance_ohms', '16')}Ω</span>`;

  xfRow.append(pt, caps, tubesRow, ot);

  // Turret Board schematic preview
  const turret = document.createElement('div');
  turret.className = 'turret';
  turret.style.cssText =
    'border-radius:6px;padding:10px 14px;display:flex;align-items:center;justify-content:space-between;color:#f3dec0;';
  const selectedTubeInfo = CHASSIS_TUBES.find((t) => t.id === store.selectedTube) ?? CHASSIS_TUBES[0];
  turret.innerHTML = `
    <span class="m-mono" style="font-size:10.5px;">VÁLVULA SELECIONADA: <strong>${selectedTubeInfo.id}</strong> (${selectedTubeInfo.stage})</span>
    <span class="m-mono" style="font-size:10px;color:#ffd093;">B+ PRÉ: ${num(store, 'amp_1.preamp_plate_v', 280).toFixed(0)}V · BIAS: ${num(store, 'amp_1.bias_ma', 38).toFixed(1)}mA</span>
  `;

  hwStage.append(xfRow, turret);

  // CRT Oscilloscope box
  const crtWrap = document.createElement('div');
  crtWrap.style.cssText = 'display:flex;flex-direction:column;gap:6px;';
  const crtBar = document.createElement('div');
  crtBar.style.cssText = 'display:flex;align-items:center;justify-content:space-between;';
  crtBar.innerHTML = '<span class="m-lbl" style="color:#1b1f25;">OSCILOSCÓPIO DE BANCADA (TELEMETRIA EM TEMPO REAL)</span>';
  const crtModes = document.createElement('div');
  crtModes.style.cssText = 'display:flex;gap:4px;';
  (
    [
      ['wave', 'Forma Onda'],
      ['transfer', 'Curva Transf.'],
      ['tonestack', 'Tone Stack'],
    ] as const
  ).forEach(([m, lbl]) => {
    crtModes.appendChild(
      makeChipButton('chassi.crt_mode', lbl, store.crtMode === m, () => {
        store.crtMode = m;
        void store.transport.setStreamVisible(true, m);
        store.emit();
      })
    );
  });
  crtBar.appendChild(crtModes);

  const crt = document.createElement('div');
  crt.className = 'crt';
  crt.style.cssText = 'border-radius:8px;padding:6px 10px;';
  crt.appendChild(buildScopeSvg(store));
  crtWrap.append(crtBar, crt);

  chassis.append(hdr, hwStage, crtWrap);

  // Right: Tabbed Circuit & Tube Mods Panel
  const { root: modCard, body: modBody } = makePanelCard('MODIFICAÇÕES DE CIRCUITO E VÁLVULAS', 'Spec 031 Real DSP');
  const tabs = document.createElement('div');
  tabs.style.cssText = 'display:flex;gap:6px;flex:none;';
  tabs.append(
    makeChipButton('ui.subtab', 'Válvulas & Fonte', store.chassiTab === 'tube', () => {
      store.chassiTab = 'tube';
      store.emit();
    }),
    makeChipButton('ui.subtab', 'Bias, NFB & Tone Stack', store.chassiTab === 'circuit', () => {
      store.chassiTab = 'circuit';
      store.emit();
    })
  );
  modBody.appendChild(tabs);

  const controlsBox = document.createElement('div');
  controlsBox.style.cssText = 'display:flex;flex-direction:column;gap:9px;overflow-y:auto;flex:1;min-height:0;';

  if (store.chassiTab === 'tube') {
    const addSelectRow = (label: string, controlId: string, key: string, choices: { value: string; label: string }[], def: string) => {
      const row = document.createElement('div');
      row.style.cssText = 'display:flex;align-items:center;justify-content:space-between;gap:8px;';
      const l = document.createElement('span');
      l.className = 'm-lbl';
      l.textContent = label;
      row.append(l, makeControlSelect(store, controlId, key, choices, def));
      controlsBox.appendChild(row);
    };
    addSelectRow(
      'Válvula de Pré',
      'chassi.preamp_tube',
      'amp_1.preamp_tube',
      [
        { value: '12ax7', label: '12AX7 (ECC83 High Gain)' },
        { value: '12at7', label: '12AT7 (ECC81 Lower Mu / Clean)' },
        { value: 'ef86_variant', label: 'EF86 Pentode (Rich Harmonic)' },
      ],
      '12ax7'
    );
    addSelectRow(
      'Válvula de Potência',
      'chassi.power_tube',
      'amp_1.power_tube',
      [
        { value: 'el34', label: 'EL34 (British Mid Crunch)' },
        { value: '6l6gc', label: '6L6GC (US Deep Headroom)' },
        { value: 'kt88', label: 'KT88 (Tight Modern Punch)' },
        { value: 'el84_variant', label: 'EL84 Class-A Quad (Chime)' },
      ],
      'el34'
    );
    addSelectRow(
      'Retificador da Fonte',
      'chassi.rectifier',
      'amp_1.rectifier',
      [
        { value: 'diode', label: 'Silício Solid-State (Rígido)' },
        { value: 'gz34', label: 'GZ34 / 5AR4 (Sag Médio)' },
        { value: '5u4gb', label: '5U4GB Dual Valve (Sag Vintage)' },
      ],
      'diode'
    );
    addSelectRow(
      'Impedância OT',
      'chassi.impedance_ohms',
      'amp_1.impedance_ohms',
      [
        { value: '4', label: '4 Ohms' },
        { value: '8', label: '8 Ohms' },
        { value: '16', label: '16 Ohms' },
      ],
      '16'
    );
    controlsBox.append(
      makeControlSlider(store, 'chassi.sag', 'amp_1.sag', 'Sag da Fonte B+', 3.0),
      makeControlSlider(store, 'chassi.preamp_plate_v', 'amp_1.preamp_plate_v', 'Tensão de Placa (B+ Pré)', 280)
    );
  } else {
    const tsRow = document.createElement('div');
    tsRow.style.cssText = 'display:flex;align-items:center;justify-content:space-between;gap:8px;';
    tsRow.innerHTML = '<span class="m-lbl">Topologia Tone Stack</span>';
    tsRow.appendChild(
      makeControlSelect(
        store,
        'chassi.tonestack',
        'amp_1.tonestack',
        [
          { value: 'model', label: 'Original do Amplificador' },
          { value: 'marshall', label: 'FMV Marshall Stack' },
          { value: 'fender', label: 'Blackface AB763 Stack' },
          { value: 'vox', label: 'Top Boost Passivo' },
          { value: 'baxandall', label: 'Baxandall Ativo / Studio' },
        ],
        'model'
      )
    );
    controlsBox.append(
      tsRow,
      makeControlSlider(store, 'chassi.bias_ma', 'amp_1.bias_ma', 'Bias de Grade (Class AB)', 38),
      makeControlSlider(store, 'chassi.nfb', 'amp_1.nfb', 'Realimentação Negativa (NFB)', 5.0),
      makeControlSlider(store, 'chassi.xformer_sat', 'amp_1.xformer_sat', 'Saturação Trafo Saída', 4.0),
      makeControlSlider(store, 'chassi.triode_asym', 'amp_1.triode_asym', 'Assimetria de Tríodos', 0.2)
    );
  }

  modBody.appendChild(controlsBox);
  root.append(chassis, modCard);
  return root;
}
