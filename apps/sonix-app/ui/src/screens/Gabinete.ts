import type { AppStore } from '../store';
import {
  bool,
  makeChipButton,
  makeControlSelect,
  makeControlSlider,
  makePanelCard,
  makeToggleRow,
  num,
  str,
} from './_helpers';

const CAB_SPEAKERS = [
  { id: 'greenback_4x12', title: '4x12 G12M Greenback', detail: '25W Celestion · Fechado' },
  { id: 'v30_4x12', title: '4x12 Vintage 30', detail: '60W High-Mid Punch · Fechado' },
  { id: 'alnico_blue_2x12', title: '2x12 Alnico Blue', detail: '15W Bell Chime · Semi-Aberto' },
  { id: 'twin_2x12_open', title: '2x12 Open Blackface', detail: 'Jensen C12N · Open Back' },
];

const MIC_CHOICES = [
  { value: 'sm57', label: 'SM57 Dynamic' },
  { value: 'md421', label: 'MD421-II' },
  { value: 'm88', label: 'M88 TG' },
  { value: 'r121', label: 'R121 Ribbon' },
  { value: 'm160', label: 'M160 Hyper' },
  { value: 'u87', label: 'U87 Condenser' },
  { value: 'c414', label: 'C414 XLII' },
];

function createDraggableMic(
  store: AppStore,
  micIdx: 1 | 2,
  stageEl: HTMLElement
): HTMLElement {
  const posKey = `cab_1.mic${micIdx}.position`;
  const distKey = `cab_1.mic${micIdx}.distance_cm`;
  const angleKey = `cab_1.mic${micIdx}.angle_deg`;
  const enKey = `cab_1.mic${micIdx}.enabled`;

  const pos = num(store, posKey, micIdx === 1 ? 0.25 : 0.6);
  const dist = num(store, distKey, micIdx === 1 ? 4 : 12);
  const angle = num(store, angleKey, micIdx === 1 ? 0 : 45);
  const enabled = bool(store, enKey, micIdx === 1);

  const marker = document.createElement('button');
  marker.type = 'button';
  marker.className = 'g-mic';
  marker.dataset.controlId = `cab.mic${micIdx}.position`;
  marker.setAttribute('aria-label', `Microfone ${micIdx} posição radial ${pos.toFixed(2)}`);
  const leftPct = 50 + pos * 34;
  const topPct = 36 + Math.min(28, dist * 0.8);
  marker.style.cssText = `
    position:absolute;
    left:${leftPct.toFixed(1)}%;
    top:${topPct.toFixed(1)}%;
    width:32px;
    height:32px;
    border-radius:50%;
    transform:translate(-50%,-50%) rotate(${angle === 45 ? -22 : 0}deg);
    border:2px solid ${micIdx === 1 ? '#ff8a3d' : '#3dd8ff'};
    background:rgba(12,14,18,0.92);
    color:#fff;
    font-size:10px;
    font-weight:700;
    cursor:grab;
    opacity:${enabled ? '1' : '0.4'};
    display:flex;
    align-items:center;
    justify-content:center;
    box-shadow:0 4px 12px rgba(0,0,0,0.75);
  `;
  marker.textContent = `M${micIdx}`;
  marker.title = `Arraste para mover eixo · Scroll distância (${dist.toFixed(1)}cm) · Duplo clique ângulo (${angle}°)`;

  marker.addEventListener('pointerdown', (e: PointerEvent) => {
    e.preventDefault();
    marker.setPointerCapture(e.pointerId);
    const onMove = (ev: PointerEvent) => {
      const rect = stageEl.getBoundingClientRect();
      const norm = Math.max(0, Math.min(1, ((ev.clientX - rect.left) / Math.max(1, rect.width) - 0.5) / 0.34));
      void store.setControl(`cab.mic${micIdx}.position`, Number(norm.toFixed(2)));
    };
    const onUp = () => {
      marker.removeEventListener('pointermove', onMove);
      marker.removeEventListener('pointerup', onUp);
      marker.removeEventListener('pointercancel', onUp);
    };
    marker.addEventListener('pointermove', onMove);
    marker.addEventListener('pointerup', onUp);
    marker.addEventListener('pointercancel', onUp);
  });

  marker.addEventListener(
    'wheel',
    (e: WheelEvent) => {
      e.preventDefault();
      const next = Math.max(1, Math.min(35, dist + (e.deltaY > 0 ? 1 : -1)));
      void store.setControl(`cab.mic${micIdx}.distance_cm`, Number(next.toFixed(1)));
    },
    { passive: false }
  );

  marker.addEventListener('dblclick', () => {
    void store.setControl(`cab.mic${micIdx}.angle_deg`, angle === 45 ? '0' : '45');
  });

  return marker;
}

function buildCabResponseSvg(store: AppStore): SVGSVGElement {
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 360 110');
  svg.style.cssText = 'width:100%;height:110px;background:#070a08;border:1px solid #1e2a22;border-radius:6px;';
  const resHz = num(store, 'cab_1.resonance_hz', 110);
  const hiCutKhz = num(store, 'cab_1.hicut_khz', 6.5);
  const pts: string[] = [];
  for (let i = 0; i <= 90; i += 1) {
    const f = 40 * Math.pow(12000 / 40, i / 90);
    const resBoost = Math.exp(-Math.pow(Math.log2(f / resHz), 2) * 3) * 18;
    const rolloff = f > hiCutKhz * 1000 ? 32 * Math.log2(f / (hiCutKhz * 1000)) : 0;
    const y = 55 - resBoost + Math.min(48, rolloff);
    pts.push(`${(i * 4).toFixed(1)},${Math.max(8, Math.min(102, y)).toFixed(1)}`);
  }
  const line = document.createElementNS('http://www.w3.org/2000/svg', 'polyline');
  line.setAttribute('fill', 'none');
  line.setAttribute('stroke', '#3dd8ff');
  line.setAttribute('stroke-width', '2');
  line.setAttribute('points', pts.join(' '));
  svg.appendChild(line);
  return svg;
}

export function renderGabineteScreen(store: AppStore): HTMLElement {
  const root = document.createElement('div');
  root.dataset.screen = 'gabinete';
  root.style.cssText =
    'height:100%;display:grid;grid-template-columns:1.1fr 0.9fr;gap:10px;padding:10px;min-height:0;overflow:hidden;';

  // Left: Cab Model Cards + 12" Speaker Mic Stage + Response Curve
  const left = document.createElement('div');
  left.className = 'm-metal';
  left.style.cssText =
    'border-radius:10px;padding:10px;display:flex;flex-direction:column;gap:10px;min-height:0;overflow:hidden;';

  const cabGrid = document.createElement('div');
  cabGrid.style.cssText = 'display:grid;grid-template-columns:repeat(4,1fr);gap:6px;flex:none;';
  const activeSpeaker = str(store, 'cab_1.speaker', 'greenback_4x12');
  for (const cab of CAB_SPEAKERS) {
    const btn = makeChipButton('cab.speaker', cab.title, activeSpeaker === cab.id, () => {
      void store.setControl('cab.speaker', cab.id);
    });
    btn.style.cssText = `height:auto;padding:7px 8px;text-align:left;display:flex;flex-direction:column;gap:2px;${
      activeSpeaker === cab.id ? 'border-color:#ff8a3d;background:rgba(255,138,61,0.16);color:#ffd2b0;' : ''
    }`;
    const sub = document.createElement('span');
    sub.style.cssText = 'font-size:9px;opacity:0.75;font-weight:500;';
    sub.textContent = cab.detail;
    btn.appendChild(sub);
    cabGrid.appendChild(btn);
  }

  // Interactive 12" Speaker Cone Stage
  const stage = document.createElement('div');
  stage.style.cssText =
    'position:relative;flex:1;min-height:170px;border-radius:10px;background:radial-gradient(circle at 50% 50%, #1d2025 0%, #0d0f12 68%, #07080a 100%);border:1px solid #2b3038;overflow:hidden;display:flex;align-items:center;justify-content:center;';
  const cone = document.createElement('div');
  cone.style.cssText =
    'width:220px;height:220px;border-radius:50%;border:6px solid #25282e;background:radial-gradient(circle, #060708 0 24%, #15171b 25% 55%, #1d2025 80%, #111316 100%);box-shadow:inset 0 0 28px rgba(0,0,0,0.9);';
  stage.appendChild(cone);
  stage.appendChild(createDraggableMic(store, 1, stage));
  stage.appendChild(createDraggableMic(store, 2, stage));

  const curveBox = document.createElement('div');
  curveBox.style.cssText = 'display:flex;flex-direction:column;gap:4px;flex:none;';
  curveBox.innerHTML = `<span class="m-lbl">CURVA DO GABINETE · RESSONÂNCIA ${num(store, 'cab_1.resonance_hz', 110).toFixed(0)} Hz · CORTE ${num(store, 'cab_1.hicut_khz', 6.5).toFixed(1)} kHz</span>`;
  curveBox.appendChild(buildCabResponseSvg(store));

  left.append(cabGrid, stage, curveBox);

  // Right: Tabs (Microfones | Caixa | IR Custom)
  const { root: rightCard, body: rightBody } = makePanelCard('GABINETE, MICROFONES & IMPULSE RESPONSE');
  const tabs = document.createElement('div');
  tabs.style.cssText = 'display:flex;gap:6px;flex:none;';
  (
    [
      ['mics', 'Microfones Dual'],
      ['cab', 'Caixa & Impedância'],
      ['ir', 'IR Custom (.wav)'],
    ] as const
  ).forEach(([tab, lbl]) => {
    tabs.appendChild(
      makeChipButton('ui.subtab', lbl, store.cabTab === tab, () => {
        store.cabTab = tab;
        store.emit();
      })
    );
  });
  rightBody.appendChild(tabs);

  const panel = document.createElement('div');
  panel.style.cssText = 'display:flex;flex-direction:column;gap:8px;overflow-y:auto;flex:1;min-height:0;';

  if (store.cabTab === 'mics') {
    for (const idx of [1, 2] as const) {
      const grp = makePanelCard(`MICROFONE ${idx}`, idx === 1 ? 'Close Mic' : 'Room / Off-Axis');
      grp.body.append(
        makeToggleRow(store, `cab.mic${idx}.enabled`, `cab_1.mic${idx}.enabled`, `Ativar Mic ${idx}`, idx === 1),
        makeControlSelect(store, `cab.mic${idx}.model`, `cab_1.mic${idx}.model`, MIC_CHOICES, idx === 1 ? 'sm57' : 'r121'),
        makeControlSlider(store, `cab.mic${idx}.position`, `cab_1.mic${idx}.position`, 'Eixo Centro->Borda', idx === 1 ? 0.25 : 0.6),
        makeControlSlider(store, `cab.mic${idx}.distance_cm`, `cab_1.mic${idx}.distance_cm`, 'Distância', idx === 1 ? 4 : 12),
        makeControlSlider(store, `cab.mic${idx}.level_db`, `cab_1.mic${idx}.level_db`, 'Nível Mix', idx === 1 ? 0 : -6),
        makeToggleRow(store, `cab.mic${idx}.phase_invert`, `cab_1.mic${idx}.phase_invert`, 'Inverter Fase 180°', false, '180°', '0°'),
        makeControlSelect(
          store,
          `cab.mic${idx}.angle_deg`,
          `cab_1.mic${idx}.angle_deg`,
          [
            { value: '0', label: 'On-Axis (0°)' },
            { value: '45', label: 'Off-Axis (45°)' },
          ],
          idx === 1 ? '0' : '45'
        )
      );
      panel.appendChild(grp.root);
    }
  } else if (store.cabTab === 'cab') {
    panel.append(
      makeToggleRow(store, 'cab.enabled', 'cab_1.enabled', 'Simulação de Gabinete', true),
      makeControlSelect(
        store,
        'cab.speaker',
        'cab_1.speaker',
        CAB_SPEAKERS.map((s) => ({ value: s.id, label: s.title })),
        'greenback_4x12'
      ),
      makeControlSlider(store, 'cab.resonance_hz', 'cab_1.resonance_hz', 'Ressonância Baixa', 110),
      makeControlSlider(store, 'cab.hicut_khz', 'cab_1.hicut_khz', 'High Cut Térmico', 6.5)
    );
  } else {
    panel.appendChild(makeToggleRow(store, 'cab.ir_enabled', 'cab_1.ir_enabled', 'Usar IR Custom Carregado', false, 'ON', 'OFF'));
    const irBtn = makeChipButton('cab.ir_load', 'Carregar Arquivo IR (.wav)...', false, async () => {
      const path = await store.transport.openFileDialog([{ name: 'Impulse Response WAV', extensions: ['wav'] }]);
      if (!path) return;
      const bytes = await store.transport.readFileBytes(path);
      const name = path.split(/[/\\]/).pop() ?? 'custom.wav';
      const info = await store.transport.loadIr({
        name,
        wavBytes: Array.from(bytes),
        normalize: true,
      });
      store.irInfo = {
        fileName: name,
        sampleRate: Number(info.sampleRate ?? info.sample_rate ?? 48000),
        samples: Number(info.lengthSamples ?? info.samples ?? 0),
      };
      void store.setControl('cab.ir_enabled', true);
      store.emit();
    });
    panel.appendChild(irBtn);

    const lcd = document.createElement('div');
    lcd.className = 'm-lcd m-mono';
    lcd.style.cssText = 'padding:8px 10px;border-radius:6px;font-size:11px;color:#ffb463;';
    lcd.textContent =
      store.irInfo.samples > 0
        ? `IR: ${store.irInfo.fileName} · ${store.irInfo.sampleRate} Hz · ${store.irInfo.samples} amostras`
        : store.irInfo.fileName;
    panel.appendChild(lcd);
  }

  rightBody.appendChild(panel);
  root.append(left, rightCard);
  return root;
}
