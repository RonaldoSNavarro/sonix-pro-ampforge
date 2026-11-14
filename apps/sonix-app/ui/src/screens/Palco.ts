import type { AppStore } from '../store';
import {
  bool,
  makeChipButton,
  makeControlSlider,
  makeLedMeter,
  makePanelCard,
  num,
  str,
} from './_helpers';

const FOOTSWITCHES: {
  numLabel: string;
  controlId: string;
  paramKey: string;
  blockName: string;
  color: string;
  sceneId: string;
}[] = [
  { numLabel: 'FS 1', controlId: 'palco.fs_1_gate', paramKey: 'gate.enabled', blockName: 'GATE-2', color: '#ff8a3d', sceneId: 'A' },
  { numLabel: 'FS 2', controlId: 'palco.fs_2_drive', paramKey: 'drive.enabled', blockName: 'DRIVE', color: '#4ee680', sceneId: 'B' },
  { numLabel: 'FS 3', controlId: 'palco.fs_3_comp', paramKey: 'comp_2.enabled', blockName: 'FET 76', color: '#ffd166', sceneId: 'C' },
  { numLabel: 'FS 4', controlId: 'palco.fs_4_chorus', paramKey: 'mod_2.enabled', blockName: 'CHORUS', color: '#3dd8ff', sceneId: 'D' },
  { numLabel: 'FS 5', controlId: 'palco.fs_5_eq', paramKey: 'eq_1.enabled', blockName: '10B EQ', color: '#ff6e91', sceneId: 'E' },
  { numLabel: 'FS 6', controlId: 'palco.fs_6_delay', paramKey: 'dly_1.enabled', blockName: 'RE-201', color: '#b48eff', sceneId: 'F' },
  { numLabel: 'FS 7', controlId: 'palco.fs_7_reverb', paramKey: 'rev_1.enabled', blockName: 'PLATE', color: '#78d2ff', sceneId: 'G' },
  { numLabel: 'FS 8', controlId: 'palco.fs_8_split', paramKey: 'split.enabled', blockName: 'SPLIT B', color: '#ff9b4a', sceneId: 'H' },
];

export function renderPalcoScreen(store: AppStore): HTMLElement {
  const root = document.createElement('div');
  root.dataset.screen = 'palco';
  root.className = 'p-alu';
  root.style.cssText =
    'height:100%;display:grid;grid-template-rows:auto 1fr auto;gap:10px;padding:12px;min-height:0;overflow:hidden;';

  // Top Stage LCD + Preset / Scene + Fullscreen toggle
  const top = document.createElement('div');
  top.style.cssText = 'display:grid;grid-template-columns:1fr auto;gap:10px;align-items:center;';

  const lcd = document.createElement('div');
  lcd.className = 'a-lcd';
  lcd.style.cssText =
    'padding:8px 14px;border-radius:8px;display:flex;align-items:center;justify-content:space-between;gap:14px;';
  const presetName = store.presets[store.presetIdx]?.name ?? 'Factory Crunch 800';
  const activeScene = str(store, 'scene.active', 'A');
  const ampModel = str(store, 'amp_1.model', 'jcm800_2203').toUpperCase();
  const latMs =
    store.audioStatus.measured_roundtrip_ms ??
    store.audioStatus.driver_input_latency_ms + store.audioStatus.driver_output_latency_ms;
  lcd.innerHTML = `
    <div>
      <div class="a-lcdtxt m-osw" style="font-size:20px;font-weight:700;letter-spacing:0.06em;">${presetName.toUpperCase()}</div>
      <div class="a-lcdtxt m-mono" style="font-size:11px;opacity:.85;">SCENE ${activeScene} · ${ampModel} · TAP ${Math.round(store.bpm)} BPM</div>
    </div>
    <div class="a-lcdtxt m-mono" style="font-size:12px;text-align:right;">
      <div>OUT ${(store.telemetry.output_peak_db ?? -60).toFixed(1)} dB</div>
      <div>LATENCY ${latMs.toFixed(1)} ms</div>
    </div>
  `;

  const topActions = document.createElement('div');
  topActions.style.cssText = 'display:flex;align-items:center;gap:6px;';
  topActions.append(
    makeChipButton('topbar.preset_nav', '< PRESET', false, () => {
      const len = Math.max(1, store.presets.length);
      void store.loadPresetAt((store.presetIdx - 1 + len) % len);
    }),
    makeChipButton('topbar.preset_nav', 'PRESET >', false, () => {
      const len = Math.max(1, store.presets.length);
      void store.loadPresetAt((store.presetIdx + 1) % len);
    }),
    makeChipButton('ui.subtab', store.palcoFsMode === 'stomp' ? 'MODO: STOMP' : 'MODO: CENAS', true, () => {
      store.palcoFsMode = store.palcoFsMode === 'stomp' ? 'scene' : 'stomp';
      store.emit();
    }),
    makeChipButton('ui.nav_screen', store.fullscreen ? 'SAIR TELA CHEIA (ESC)' : 'TELA CHEIA (F11)', store.fullscreen, () => {
      void store.toggleFullscreen();
    })
  );
  top.append(lcd, topActions);

  // Center Stage: Tuner + Looper + EXP-1 Treadle
  const center = document.createElement('div');
  center.style.cssText = 'display:grid;grid-template-columns:1.1fr 0.9fr 210px;gap:10px;min-height:0;';

  // Tuner Card
  const { root: tunerCard, body: tunerBody } = makePanelCard('AFINADOR DE PRECISÃO (YIN LOCK-FREE)');
  const tunerOn = bool(store, 'tuner.enabled', false);
  const tunerMute = str(store, 'tuner.output', 'mute') === 'mute';
  const cents = store.tuner.cents ?? 0;
  const note = store.tuner.note || '--';
  const freq = store.tuner.frequency_hz ? `${store.tuner.frequency_hz.toFixed(1)} Hz` : '---.- Hz';

  const tunerBar = document.createElement('div');
  tunerBar.style.cssText = 'display:flex;align-items:center;gap:6px;flex-wrap:wrap;';
  tunerBar.append(
    makeChipButton('tuner.enabled', tunerOn ? 'AFINADOR ON' : 'AFINADOR OFF', tunerOn, () => {
      void store.setControl('tuner.enabled', !tunerOn);
    }),
    makeChipButton('tuner.output', tunerMute ? 'SAÍDA: MUDO' : 'SAÍDA: BYPASS', tunerMute, () => {
      void store.setControl('tuner.output', tunerMute ? 'bypass' : 'mute');
    }),
    makeChipButton('tuner.view', store.palcoTunerView === 'strobe' ? 'STROBE' : 'POLIFÔNICO', true, () => {
      store.palcoTunerView = store.palcoTunerView === 'strobe' ? 'poly' : 'strobe';
      store.emit();
    })
  );

  const noteDisplay = document.createElement('div');
  noteDisplay.className = 'm-lcd';
  noteDisplay.style.cssText =
    'flex:1;min-height:84px;border-radius:8px;padding:10px 14px;display:flex;flex-direction:column;justify-content:center;align-items:center;gap:6px;';
  const inTune = tunerOn && Math.abs(cents) <= 3;
  noteDisplay.innerHTML = `
    <div style="display:flex;align-items:baseline;gap:14px;">
      <span class="m-osw" style="font-size:38px;font-weight:700;color:${inTune ? '#4ee680' : '#ffb463'};">${note}</span>
      <span class="m-mono" style="font-size:13px;color:#ffd08a;">${cents >= 0 ? '+' : ''}${cents.toFixed(1)} cents · ${freq}</span>
    </div>
    <div style="width:100%;height:8px;background:#101214;border-radius:4px;position:relative;overflow:hidden;">
      <div style="position:absolute;left:50%;top:0;bottom:0;width:2px;background:#4ee680;"></div>
      <div style="position:absolute;top:1px;bottom:1px;width:12px;border-radius:3px;left:calc(50% + ${Math.max(-46, Math.min(46, cents))}%);transform:translateX(-50%);background:${inTune ? '#4ee680' : '#ff8a3d'};"></div>
    </div>
  `;

  tunerBody.append(
    tunerBar,
    noteDisplay,
    makeControlSlider(store, 'tuner.ref_hz', 'tuner.ref_hz', 'Referência A4', 440, (v) => `${v.toFixed(0)} Hz`)
  );

  // Looper Card
  const { root: looperCard, body: looperBody } = makePanelCard('LOOPER DE PALCO · 60s ESTÉREO');
  const looperPos = str(store, 'looper.position', 'post');
  const looperAction = str(store, 'looper.action', 'stop');
  const looperActions = [
    ['record', 'REC'],
    ['play', 'PLAY'],
    ['overdub', 'OVERDUB'],
    ['stop', 'STOP'],
    ['undo', 'UNDO'],
    ['clear', 'CLEAR'],
  ] as const;

  const actRow = document.createElement('div');
  actRow.style.cssText = 'display:grid;grid-template-columns:repeat(3,1fr);gap:6px;';
  for (const [act, lbl] of looperActions) {
    actRow.appendChild(
      makeChipButton('looper.action', lbl, looperAction === act, () => {
        void store.setControl('looper.action', act);
      })
    );
  }

  const posRow = document.createElement('div');
  posRow.style.cssText = 'display:flex;align-items:center;justify-content:space-between;gap:8px;';
  posRow.innerHTML = '<span class="m-lbl">POSIÇÃO NA CADEIA</span>';
  const posBtns = document.createElement('div');
  posBtns.style.cssText = 'display:flex;gap:4px;';
  posBtns.append(
    makeChipButton('looper.position', 'PRE-AMP (REAMP)', looperPos === 'pre', () => {
      void store.setControl('looper.position', 'pre');
    }),
    makeChipButton('looper.position', 'POST-FX (PALCO)', looperPos === 'post', () => {
      void store.setControl('looper.position', 'post');
    })
  );
  posRow.appendChild(posBtns);

  const looperMeter = document.createElement('div');
  looperMeter.className = 'm-lcd m-mono';
  looperMeter.style.cssText = 'padding:10px;border-radius:6px;font-size:11px;color:#ffd08a;display:flex;justify-content:space-between;';
  looperMeter.innerHTML = `<span>ESTADO: ${looperAction.toUpperCase()}</span><span>ROTA: ${looperPos.toUpperCase()}</span>`;

  const meterRow = document.createElement('div');
  meterRow.style.cssText = 'display:flex;align-items:center;gap:8px;';
  meterRow.innerHTML = '<span class="m-lbl">NÍVEL SAÍDA</span>';
  const outNorm = Math.max(0, Math.min(1, ((store.telemetry.output_peak_db ?? -60) + 60) / 60));
  meterRow.appendChild(makeLedMeter(20, outNorm));

  looperBody.append(actRow, posRow, looperMeter, meterRow);

  // EXP-1 Pedal Card
  const { root: expCard, body: expBody } = makePanelCard('PEDAL EXP-1', 'GAIN / WAH');
  const gainVal = num(store, 'amp_1.gain', 6.5);
  const expPct = Math.round((gainVal / 10) * 100);
  const treadleVisual = document.createElement('div');
  treadleVisual.style.cssText =
    'flex:1;min-height:90px;border-radius:8px;background:linear-gradient(180deg,#1a1d22,#0d0f12);border:1px solid #2d333c;padding:10px;display:flex;flex-direction:column;justify-content:space-between;align-items:center;';
  treadleVisual.innerHTML = `
    <span class="m-mono" style="font-size:18px;font-weight:700;color:#ff8a3d;">${expPct}%</span>
    <div style="width:36px;flex:1;margin:6px 0;border-radius:4px;background:#07080a;position:relative;overflow:hidden;">
      <div style="position:absolute;left:0;right:0;bottom:0;height:${expPct}%;background:linear-gradient(180deg,#ff8a3d,#b84510);"></div>
    </div>
    <span class="m-lbl">AMP GAIN: ${gainVal.toFixed(1)}</span>
  `;
  expBody.append(treadleVisual, makeControlSlider(store, 'palco.exp_1', 'amp_1.gain', 'EXP-1 -> Gain', 6.5));

  center.append(tunerCard, looperCard, expCard);

  // Bottom: 8 OLED Footswitches
  const fsStrip = document.createElement('div');
  fsStrip.className = 'p-well';
  fsStrip.style.cssText = 'display:grid;grid-template-columns:repeat(8,1fr);gap:8px;padding:10px;border-radius:10px;';

  for (const fs of FOOTSWITCHES) {
    const active = store.palcoFsMode === 'stomp' ? bool(store, fs.paramKey, false) : activeScene === fs.sceneId;
    const cell = document.createElement('div');
    cell.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:6px;';

    const oled = document.createElement('div');
    oled.className = 'p-oled';
    oled.style.cssText = 'width:100%;padding:5px 6px;border-radius:5px;text-align:center;';
    oled.innerHTML = `
      <div class="m-mono" style="font-size:10.5px;font-weight:700;color:${active ? fs.color : '#69727c'};">${
        store.palcoFsMode === 'stomp' ? fs.blockName : `SCENE ${fs.sceneId}`
      }</div>
      <div class="m-mono" style="font-size:8.5px;color:#929aa4;">${active ? 'ATIVO' : 'OFF'}</div>
    `;

    const led = document.createElement('span');
    led.style.cssText = `width:10px;height:10px;border-radius:50%;background:${active ? fs.color : '#1b1e23'};box-shadow:${
      active ? `0 0 10px ${fs.color}` : 'none'
    };`;

    const sw = document.createElement('button');
    sw.type = 'button';
    sw.className = 'p-fs';
    sw.dataset.controlId = fs.controlId;
    sw.setAttribute('aria-pressed', active ? 'true' : 'false');
    sw.textContent = fs.numLabel;
    sw.style.cssText =
      'width:48px;height:48px;border-radius:50%;font-size:10px;font-weight:700;color:#e8eaed;cursor:pointer;';
    sw.addEventListener('click', () => {
      if (store.palcoFsMode === 'stomp') {
        void store.setControl(fs.controlId, !bool(store, fs.paramKey, false));
      } else {
        void store.selectScene(fs.sceneId);
      }
    });

    cell.append(oled, led, sw);
    fsStrip.appendChild(cell);
  }

  root.append(top, center, fsStrip);
  return root;
}
