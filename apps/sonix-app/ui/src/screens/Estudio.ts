import type { AppStore, BlockId } from '../store';
import { createAmpHeadElement } from '../components/AmpHead';
import { createPedalElement, type PedalModelId } from '../components/Pedal';
import { createRackElement } from '../components/Rack';
import { createTubeElement } from '../components/Tube';
import {
  bool,
  makeChipButton,
  makeControlSelect,
  makeControlSlider,
  makeLedMeter,
  makePanelCard,
  makeToggleRow,
  num,
  str,
} from './_helpers';

const BLOCK_ORDER_ROW_A: { id: BlockId; code: string; label: string; toggleControl?: string; paramKey?: string }[] = [
  { id: 'in', code: 'IN', label: 'Entrada' },
  { id: 'gate', code: 'NG', label: 'Gate-2', toggleControl: 'gate.enabled', paramKey: 'gate.enabled' },
  { id: 'drive', code: 'DRV', label: 'Drive', toggleControl: 'drive.enabled', paramKey: 'drive.enabled' },
  { id: 'split', code: 'SPL', label: 'Split' },
  { id: 'amp', code: 'AMP', label: 'Amp 1', toggleControl: 'amp.enabled', paramKey: 'amp_1.enabled' },
  { id: 'cab', code: 'CAB', label: 'Cab 1', toggleControl: 'cab.enabled', paramKey: 'cab_1.enabled' },
  { id: 'eq', code: 'EQ', label: '10B EQ', toggleControl: 'eq_1.enabled', paramKey: 'eq_1.enabled' },
  { id: 'merge', code: 'MRG', label: 'Merge' },
  { id: 'delay', code: 'DLY', label: 'RE-201', toggleControl: 'dly_1.enabled', paramKey: 'dly_1.enabled' },
  { id: 'reverb', code: 'REV', label: 'Plate', toggleControl: 'rev_1.enabled', paramKey: 'rev_1.enabled' },
  { id: 'out', code: 'OUT', label: 'Saída' },
];

const BLOCK_ORDER_ROW_B: { id: BlockId; code: string; label: string; toggleControl?: string; paramKey?: string }[] = [
  { id: 'comp', code: '1176', label: 'FET 76', toggleControl: 'comp_2.enabled', paramKey: 'comp_2.enabled' },
  { id: 'amp2', code: 'AMP B', label: 'Amp 2', toggleControl: 'amp_2.enabled', paramKey: 'amp_2.enabled' },
  { id: 'cab2', code: 'CAB B', label: 'Cab 2', toggleControl: 'cab_2.enabled', paramKey: 'cab_2.enabled' },
  { id: 'chorus', code: 'CE-2', label: 'Chorus', toggleControl: 'mod_2.enabled', paramKey: 'mod_2.enabled' },
];

function renderSignalBlockCell(
  store: AppStore,
  def: { id: BlockId; code: string; label: string; toggleControl?: string; paramKey?: string }
): HTMLElement {
  const selected = store.selectedBlock === def.id;
  const active = def.paramKey ? bool(store, def.paramKey, true) : true;
  const cell = document.createElement('button');
  cell.type = 'button';
  cell.className = 'm-cell';
  cell.dataset.controlId = 'ui.inspector_select';
  cell.dataset.blockId = def.id;
  cell.dataset.selected = selected ? 'true' : 'false';
  cell.style.minWidth = '72px';
  cell.style.flex = '1 1 0';
  cell.title = `${def.label} — clique para inspecionar`;

  const top = document.createElement('div');
  top.style.cssText = 'display:flex;align-items:center;justify-content:space-between;width:100%;gap:4px;';
  const codeEl = document.createElement('span');
  codeEl.className = 'm-mono';
  codeEl.textContent = def.code;
  codeEl.style.cssText = 'font-size:10px;font-weight:700;color:#e8eaed;';

  const dot = document.createElement('span');
  dot.style.cssText = `width:6px;height:6px;border-radius:50%;background:${active ? '#ff8a3d' : '#3a3f46'};box-shadow:${active ? '0 0 6px #ff8a3d' : 'none'};`;
  top.append(codeEl, dot);

  const sub = document.createElement('span');
  sub.style.cssText = 'font-size:9px;color:#9aa2ad;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;';
  sub.textContent = def.label;
  cell.append(top, sub);

  cell.addEventListener('click', () => store.selectBlock(def.id));
  cell.addEventListener('dblclick', () => {
    if (def.toggleControl && def.paramKey) {
      void store.setControl(def.toggleControl, !bool(store, def.paramKey, true));
    }
  });
  return cell;
}

function renderInspectorBody(store: AppStore, blk: BlockId): HTMLElement {
  const body = document.createElement('div');
  body.style.cssText = 'display:flex;flex-direction:column;gap:8px;overflow:auto;min-height:0;flex:1;padding-right:2px;';

  const add = (...els: HTMLElement[]) => els.forEach((e) => body.appendChild(e));

  if (blk === 'in') {
    add(
      makeControlSelect(
        store,
        'main.input_source',
        'input.source',
        [
          { value: 'device', label: 'Interface Áudio (DI)' },
          { value: 'test_riff', label: 'Riff Interno Teste' },
          { value: 'di_file', label: 'Arquivo DI (.wav)' },
        ],
        'device'
      ),
      makeControlSlider(store, 'main.input_boost', 'input.boost', 'Ganho de Entrada', 1.0, (v) => `${v.toFixed(2)}x`)
    );
  } else if (blk === 'gate') {
    add(
      makeToggleRow(store, 'gate.enabled', 'gate.enabled', 'Bypass Gate-2', true),
      makeControlSelect(
        store,
        'gate.mode',
        'gate.mode',
        [
          { value: 'natural', label: 'Natural (Expansor)' },
          { value: 'classic', label: 'Classic (Hard Gate)' },
        ],
        'natural'
      ),
      makeControlSlider(store, 'gate.threshold_db', 'gate.threshold_db', 'Threshold', -65),
      makeControlSlider(store, 'gate.release_ms', 'gate.release_ms', 'Release / Decay', 80),
      makeControlSlider(store, 'gate.hysteresis_db', 'gate.hysteresis_db', 'Histerese', 4),
      makeControlSlider(store, 'gate.attack_ms', 'gate.attack_ms', 'Attack', 1),
      makeControlSlider(store, 'gate.hold_ms', 'gate.hold_ms', 'Hold', 35),
      makeControlSlider(store, 'gate.range_db', 'gate.range_db', 'Range', -70),
      makeControlSlider(store, 'gate.key_hpf_hz', 'gate.key_hpf_hz', 'Sidechain HPF', 80),
      makeControlSlider(store, 'gate.key_lpf_hz', 'gate.key_lpf_hz', 'Sidechain LPF', 4500)
    );
  } else if (blk === 'drive') {
    add(
      makeToggleRow(store, 'drive.enabled', 'drive.enabled', 'Bypass Drive', false),
      makeControlSelect(
        store,
        'drive.model',
        'drive.model',
        [
          { value: 'ts808', label: 'TS-808 Overdrive' },
          { value: 'klon', label: 'Golden Klone' },
          { value: 'rat', label: 'Rodent Pro' },
          { value: 'muff', label: 'Big Fuzz Pi' },
          { value: 'fuzzface', label: 'Face Fuzz Ge' },
          { value: 'ds1', label: 'DS-One Dist' },
        ],
        'ts808'
      ),
      makeControlSlider(store, 'drive.drive', 'drive.drive', 'Drive / Gain', 5),
      makeControlSlider(store, 'drive.tone', 'drive.tone', 'Tone / Filter', 5),
      makeControlSlider(store, 'drive.level', 'drive.level', 'Level', 5),
      makeControlSlider(store, 'drive.bias', 'drive.bias', 'Bias / Sag', 5)
    );
  } else if (blk === 'chorus') {
    add(
      makeToggleRow(store, 'mod_2.enabled', 'mod_2.enabled', 'Bypass Chorus-2', false),
      makeControlSlider(store, 'mod_2.rate_hz', 'mod_2.rate_hz', 'Rate', 0.8),
      makeControlSlider(store, 'mod_2.depth', 'mod_2.depth', 'Depth', 0.55),
      makeControlSlider(store, 'mod_2.mix', 'mod_2.mix', 'Mix Wet', 0.5)
    );
  } else if (blk === 'eq') {
    add(
      makeToggleRow(store, 'eq_1.enabled', 'eq_1.enabled', 'Bypass 10-Band EQ', false),
      makeControlSelect(
        store,
        'eq_1.mode',
        'eq_1.mode',
        [
          { value: 'mxr_10band', label: '10 Bandas (±12 dB)' },
          { value: 'eight_band', label: '8 Bandas Studio' },
          { value: 'graphic_5_boogie', label: '5 Bandas Classic' },
        ],
        'mxr_10band'
      ),
      makeControlSlider(store, 'eq_1.volume_db', 'eq_1.volume_db', 'Output Volume', 0),
      makeControlSlider(store, 'eq_1.gain_db', 'eq_1.gain_db', 'Input Gain', 0),
      makeControlSlider(store, 'eq_1.band1', 'eq_1.band1.gain_db', '31.25 Hz', 0),
      makeControlSlider(store, 'eq_1.band3', 'eq_1.band3.gain_db', '125 Hz', 0),
      makeControlSlider(store, 'eq_1.band5', 'eq_1.band5.gain_db', '500 Hz', 0),
      makeControlSlider(store, 'eq_1.band7', 'eq_1.band7.gain_db', '2 kHz', 0),
      makeControlSlider(store, 'eq_1.band9', 'eq_1.band9.gain_db', '8 kHz', 0)
    );
  } else if (blk === 'comp') {
    add(
      makeToggleRow(store, 'comp_2.enabled', 'comp_2.enabled', 'Bypass FET 76', false),
      makeControlSelect(
        store,
        'comp_2.ratio',
        'comp_2.ratio',
        [
          { value: '4', label: '4:1 Smooth' },
          { value: '8', label: '8:1 Tight' },
          { value: '12', label: '12:1 Limit' },
          { value: '20', label: '20:1 Brick' },
          { value: 'all', label: 'ALL BUTTONS IN' },
        ],
        '4'
      ),
      makeControlSlider(store, 'comp_2.input', 'comp_2.input', 'Input Drive', 5),
      makeControlSlider(store, 'comp_2.output', 'comp_2.output', 'Output Level', 5),
      makeControlSlider(store, 'comp_2.attack', 'comp_2.attack', 'Attack', 4),
      makeControlSlider(store, 'comp_2.release', 'comp_2.release', 'Release', 5)
    );
  } else if (blk === 'delay') {
    add(
      makeToggleRow(store, 'dly_1.enabled', 'dly_1.enabled', 'Bypass RE-201', false),
      makeControlSlider(store, 'dly_1.repeat_rate', 'dly_1.repeat_rate', 'Repeat Rate', 4.5),
      makeControlSlider(store, 'dly_1.intensity', 'dly_1.intensity', 'Intensity', 4.0),
      makeControlSlider(store, 'dly_1.echo_volume', 'dly_1.echo_volume', 'Echo Volume', 5.0),
      makeControlSlider(store, 'dly_1.reverb_volume', 'dly_1.reverb_volume', 'Spring Mix', 2.0),
      makeControlSlider(store, 'dly_1.bass', 'dly_1.bass', 'Echo Bass', 5.0),
      makeControlSlider(store, 'dly_1.treble', 'dly_1.treble', 'Echo Treble', 5.0),
      makeToggleRow(store, 'dly_1.spillover', 'dly_1.spillover', 'Spillover Cauda', true, 'ON', 'OFF')
    );
  } else if (blk === 'reverb') {
    add(
      makeToggleRow(store, 'rev_1.enabled', 'rev_1.enabled', 'Bypass Plate Reverb', false),
      makeControlSlider(store, 'rev_1.decay_s', 'rev_1.decay_s', 'Decay', 2.5),
      makeControlSlider(store, 'rev_1.predelay_ms', 'rev_1.predelay_ms', 'Pre-Delay', 20),
      makeControlSlider(store, 'rev_1.damping', 'rev_1.damping', 'Damping', 5.0),
      makeControlSlider(store, 'rev_1.mix', 'rev_1.mix', 'Mix', 40),
      makeToggleRow(store, 'rev_1.spillover', 'rev_1.spillover', 'Spillover', true, 'ON', 'OFF')
    );
  } else if (blk === 'split' || blk === 'merge' || blk === 'amp2' || blk === 'cab2') {
    add(
      makeToggleRow(store, 'split.enabled', 'split.enabled', 'Roteamento Paralelo (B)', false, 'ATIVO', 'OFF'),
      makeControlSelect(
        store,
        'split.mode',
        'split.mode',
        [
          { value: 'ab', label: 'A/B Paralelo' },
          { value: 'crossover', label: 'Crossover LR4' },
        ],
        'ab'
      ),
      makeControlSlider(store, 'split.crossover_hz', 'split.crossover_hz', 'Crossover Hz', 650),
      makeControlSlider(store, 'merge.row_1_level_db', 'merge.row_1_level_db', 'Nível Linha B', -6),
      makeToggleRow(store, 'merge.row_1_phase_invert', 'merge.row_1_phase_invert', 'Fase 180° B', false, 'INV', '0°'),
      makeToggleRow(store, 'amp_2.enabled', 'amp_2.enabled', 'Amp 2 Ativo', true),
      makeControlSelect(
        store,
        'amp_2.model',
        'amp_2.model',
        [
          { value: 'twin_ab763', label: 'US Twin AB763' },
          { value: 'ac30_tb', label: 'AC30 Top Boost' },
          { value: 'plexi_1959', label: 'Plexi 1959' },
          { value: 'jcm800_2203', label: 'JCM 800 2203' },
          { value: 'dual_rect', label: 'Dual Rectifier' },
        ],
        'twin_ab763'
      ),
      makeToggleRow(store, 'cab_2.enabled', 'cab_2.enabled', 'Cab 2 Ativo', true),
      makeControlSelect(
        store,
        'cab_2.speaker',
        'cab_2.speaker',
        [
          { value: 'alnico_blue_2x12', label: '2x12 Alnico Blue' },
          { value: 'twin_2x12_open', label: '2x12 Open Back' },
          { value: 'greenback_4x12', label: '4x12 Greenback' },
          { value: 'v30_4x12', label: '4x12 V30' },
        ],
        'alnico_blue_2x12'
      )
    );
  } else if (blk === 'cab') {
    add(
      makeToggleRow(store, 'cab.enabled', 'cab_1.enabled', 'Bypass Gabinete', true),
      makeControlSelect(
        store,
        'cab.speaker',
        'cab_1.speaker',
        [
          { value: 'greenback_4x12', label: '4x12 G12M Greenback' },
          { value: 'v30_4x12', label: '4x12 V30 Heavy' },
          { value: 'alnico_blue_2x12', label: '2x12 Alnico Blue' },
          { value: 'twin_2x12_open', label: '2x12 Open Back Jensen' },
        ],
        'greenback_4x12'
      ),
      makeControlSlider(store, 'cab.resonance_hz', 'cab_1.resonance_hz', 'Ressonância Caixa', 110),
      makeControlSlider(store, 'cab.hicut_khz', 'cab_1.hicut_khz', 'High Cut', 6.5),
      makeToggleRow(store, 'cab.ir_enabled', 'cab_1.ir_enabled', 'IR Custom Ativo', false, 'ON', 'OFF')
    );
  } else {
    // Amp / Out default inspector
    add(
      makeControlSlider(store, 'amp.gain', 'amp_1.gain', 'Gain / Pre-Amp', 6.5),
      makeControlSlider(store, 'amp.overdrive', 'amp_1.overdrive', 'Overdrive Trim', 5.0),
      makeControlSlider(store, 'amp.bass', 'amp_1.bass', 'Bass', 5.5),
      makeControlSlider(store, 'amp.middle', 'amp_1.middle', 'Middle', 5.0),
      makeControlSlider(store, 'amp.treble', 'amp_1.treble', 'Treble', 6.0),
      makeControlSlider(store, 'amp.presence', 'amp_1.presence', 'Presence', 5.0),
      makeControlSlider(store, 'amp.depth', 'amp_1.depth', 'Depth / Resonance', 5.0),
      makeControlSlider(store, 'amp.master', 'amp_1.master', 'Master Volume', 5.0)
    );
  }

  return body;
}

export function renderEstudioScreen(store: AppStore): HTMLElement {
  const root = document.createElement('div');
  root.dataset.screen = 'main';
  root.style.cssText =
    'height:100%;display:grid;grid-template-rows:auto auto 1fr;gap:8px;padding:8px 10px;min-height:0;overflow:hidden;';

  // 1. Top rig toolbar (Presets, Amber LCD, Scenes A-H, IN/OUT meters, Oversampling, Tap Tempo)
  const toolbar = document.createElement('div');
  toolbar.className = 'm-metal';
  toolbar.style.cssText =
    'display:flex;align-items:center;justify-content:space-between;gap:10px;padding:6px 10px;border-radius:8px;flex:none;';

  const leftGrp = document.createElement('div');
  leftGrp.style.cssText = 'display:flex;align-items:center;gap:6px;min-width:0;';

  const prevBtn = makeChipButton('topbar.preset_nav', '<', false, () => {
    const len = Math.max(1, store.presets.length);
    void store.loadPresetAt((store.presetIdx - 1 + len) % len);
  });
  const nextBtn = makeChipButton('topbar.preset_nav', '>', false, () => {
    const len = Math.max(1, store.presets.length);
    void store.loadPresetAt((store.presetIdx + 1) % len);
  });

  const presetSel = document.createElement('select');
  presetSel.dataset.controlId = 'topbar.preset_load';
  presetSel.style.cssText =
    'height:24px;max-width:190px;padding:0 7px;border-radius:4px;background:#0a0b0e;border:1px solid #2b313a;color:#ffd08a;font-size:11px;font-weight:600;';
  const presetItems =
    store.presets.length > 0 ? store.presets : [{ id: 'factory', name: 'Factory Crunch 800' }];
  presetItems.forEach((item, idx) => {
    const o = document.createElement('option');
    o.value = String(idx);
    o.textContent = item.name;
    if (idx === store.presetIdx) o.selected = true;
    presetSel.appendChild(o);
  });
  presetSel.addEventListener('change', () => {
    void store.loadPresetAt(Number(presetSel.value));
  });

  const saveBtn = makeChipButton('topbar.preset_save', store.lastSavedStatus || 'Salvar', false, () => {
    void store.saveActivePreset();
  });

  const lcd = document.createElement('div');
  lcd.className = 'm-lcd';
  lcd.style.cssText = 'padding:3px 10px;border-radius:4px;min-width:195px;display:flex;align-items:center;gap:8px;';
  const activeModelName = str(store, 'amp_1.model', 'jcm800_2203').toUpperCase();
  const activeScene = str(store, 'scene.active', 'A');
  lcd.innerHTML = `
    <span class="m-lcdtxt m-mono" style="font-size:11px;font-weight:700;">SCENE ${activeScene}</span>
    <span class="m-lcdtxt m-mono" style="font-size:10.5px;opacity:.9;">${activeModelName}</span>
  `;

  leftGrp.append(prevBtn, nextBtn, presetSel, saveBtn, lcd);

  // Scenes A-H
  const sceneGrp = document.createElement('div');
  sceneGrp.style.cssText = 'display:flex;align-items:center;gap:3px;';
  ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'].forEach((sc) => {
    sceneGrp.appendChild(
      makeChipButton('topbar.scene', sc, activeScene === sc, () => void store.selectScene(sc), { scene: sc })
    );
  });

  // Meters, OS & Tap Tempo
  const rightGrp = document.createElement('div');
  rightGrp.style.cssText = 'display:flex;align-items:center;gap:10px;flex:none;';

  const dbToLinear = (db: number) => Math.max(0, Math.min(1, (db + 60) / 60));
  const meters = document.createElement('div');
  meters.style.cssText = 'display:flex;flex-direction:column;gap:3px;';
  const inRow = document.createElement('div');
  inRow.style.cssText = 'display:flex;align-items:center;gap:5px;';
  inRow.innerHTML = '<span class="m-lbl" style="width:22px;">IN</span>';
  inRow.appendChild(makeLedMeter(16, dbToLinear(store.telemetry.input_peak_db ?? -60)));
  const outRow = document.createElement('div');
  outRow.style.cssText = 'display:flex;align-items:center;gap:5px;';
  outRow.innerHTML = '<span class="m-lbl" style="width:22px;">OUT</span>';
  outRow.appendChild(makeLedMeter(16, dbToLinear(store.telemetry.output_peak_db ?? -60)));
  meters.append(inRow, outRow);

  const osSel = makeControlSelect(
    store,
    'topbar.oversampling',
    'engine.oversampling',
    [
      { value: '2x', label: 'OS 2x' },
      { value: '4x', label: 'OS 4x' },
      { value: '8x', label: 'OS 8x' },
    ],
    '4x'
  );
  const tapBtn = makeChipButton('topbar.tap_tempo', `TAP ${Math.round(store.bpm)}`, false, () => {
    store.tapTempo();
  });
  const stageBtn = makeChipButton('ui.nav_screen', 'PALCO', false, () => {
    store.setScreen('palco');
  });

  rightGrp.append(meters, osSel, tapBtn, stageBtn);
  toolbar.append(leftGrp, sceneGrp, rightGrp);

  // 2. Signal Chain Grid (Row A + optional Parallel Row B)
  const grid = document.createElement('div');
  grid.className = 'm-metal';
  grid.style.cssText = 'padding:6px 10px;border-radius:8px;display:flex;flex-direction:column;gap:5px;flex:none;';

  const rowA = document.createElement('div');
  rowA.style.cssText = 'display:flex;align-items:center;gap:5px;min-width:0;';
  BLOCK_ORDER_ROW_A.forEach((b, i) => {
    if (i > 0) {
      const cable = document.createElement('span');
      cable.className = 'm-cable';
      cable.style.width = '10px';
      rowA.appendChild(cable);
    }
    rowA.appendChild(renderSignalBlockCell(store, b));
  });
  grid.appendChild(rowA);

  if (bool(store, 'split.enabled', false)) {
    const rowB = document.createElement('div');
    rowB.style.cssText = 'display:flex;align-items:center;gap:5px;padding-left:200px;';
    const tag = document.createElement('span');
    tag.className = 'm-lbl';
    tag.textContent = 'LINHA B';
    tag.style.color = '#3dd8ff';
    rowB.appendChild(tag);
    BLOCK_ORDER_ROW_B.forEach((b) => rowB.appendChild(renderSignalBlockCell(store, b)));
    grid.appendChild(rowB);
  }

  // 3. Main Split: Hardware Stage (Left) + Inspector (Right)
  const mainArea = document.createElement('div');
  mainArea.style.cssText =
    'display:grid;grid-template-columns:1fr 310px;gap:10px;min-height:0;overflow:hidden;';

  // Left: Stage (AmpHead + Tube window + Pedals + Racks)
  const stage = document.createElement('div');
  stage.style.cssText =
    'display:flex;flex-direction:column;gap:8px;min-height:0;min-width:0;overflow-y:auto;padding-right:4px;';

  const ampSection = document.createElement('div');
  ampSection.style.cssText = 'display:grid;grid-template-columns:1fr auto;gap:8px;align-items:stretch;';
  ampSection.appendChild(createAmpHeadElement(store));

  // Tube window alongside AmpHead
  const tubeBox = document.createElement('div');
  tubeBox.className = 'm-metal';
  tubeBox.style.cssText =
    'border-radius:10px;padding:6px 8px;display:flex;flex-direction:column;justify-content:space-between;align-items:center;min-width:170px;';
  const tubeHdr = document.createElement('div');
  tubeHdr.style.cssText = 'display:flex;align-items:center;justify-content:space-between;width:100%;';
  const bPlusV = Math.max(180, num(store, 'amp_1.preamp_plate_v', 280) * 1.5 - (store.telemetry.gain_reduction_db ?? 0) * 2).toFixed(0);
  tubeHdr.innerHTML = `<span class="m-lbl">VÁLVULAS</span><span class="m-mono" style="font-size:10px;color:#ffb463;">B+ ${bPlusV}V</span>`;
  const tubeRow = document.createElement('div');
  tubeRow.style.cssText = 'display:flex;align-items:flex-end;gap:4px;';
  const isLit = bool(store, 'amp_1.power', true) && !bool(store, 'amp_1.standby', false);
  const glow = isLit ? 0.45 + Math.min(0.55, Math.abs(store.telemetry.gain_reduction_db ?? 0) * 0.08 + 0.3) : 0.05;
  ['V1', 'V2', 'V3', 'V4'].forEach((id) => {
    tubeRow.appendChild(
      createTubeElement({
        id,
        typeLabel: '12AX7',
        width: 14,
        height: 36,
        radius: '7px 7px 2px 2px',
        baseWidth: 16,
        baseHeight: 7,
        baseBg: '#23262b',
        glowOp: glow,
        glowPx: isLit ? 5 : 0,
        selected: store.selectedTube === id,
        controlId: 'ui.inspector_select',
        onClick: () => {
          store.selectedTube = id;
          store.setScreen('chassi');
        },
      })
    );
  });
  const chassiQuickBtn = makeChipButton('ui.nav_screen', 'Bancada ->', false, () => store.setScreen('chassi'));
  tubeBox.append(tubeHdr, tubeRow, chassiQuickBtn);
  ampSection.appendChild(tubeBox);

  // Pedalboard strip
  const driveModelRaw = str(store, 'drive.model', 'ts808');
  const drivePedal: PedalModelId =
    driveModelRaw === 'klon' ||
    driveModelRaw === 'rat' ||
    driveModelRaw === 'muff' ||
    driveModelRaw === 'fuzzface' ||
    driveModelRaw === 'ds1'
      ? driveModelRaw
      : 'ts808';

  const pedalboard = document.createElement('div');
  pedalboard.className = 'm-grip';
  pedalboard.style.cssText =
    'border-radius:10px;border:1px solid #23272e;padding:8px 10px;display:flex;align-items:center;justify-content:center;gap:12px;flex-wrap:wrap;';
  (['ns2', drivePedal, 'ce2', 'mxr10'] as PedalModelId[]).forEach((pid) => {
    pedalboard.appendChild(createPedalElement(pid, store));
  });

  // Rack strip
  const rackStrip = document.createElement('div');
  rackStrip.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:6px;';
  rackStrip.append(
    createRackElement('comp1176', store),
    createRackElement('re201', store),
    createRackElement('plate', store)
  );

  stage.append(ampSection, pedalboard, rackStrip);

  // Right: Block Inspector Panel
  const { root: inspRoot, body: inspCardBody } = makePanelCard(
    `INSPETOR · ${store.selectedBlock.toUpperCase()}`,
    `Canal ${store.activeInspectorCh}`
  );
  inspRoot.style.minHeight = '0';

  const chTabs = document.createElement('div');
  chTabs.style.cssText = 'display:flex;gap:4px;align-items:center;flex:none;';
  (['A', 'B', 'C', 'D'] as const).forEach((ch) => {
    chTabs.appendChild(
      makeChipButton('ui.inspector_channel', `CH ${ch}`, store.activeInspectorCh === ch, () => {
        store.activeInspectorCh = ch;
        store.emit();
      })
    );
  });
  inspCardBody.appendChild(chTabs);
  inspCardBody.appendChild(renderInspectorBody(store, store.selectedBlock));

  // Gate Telemetry Readout when Gate selected
  if (store.selectedBlock === 'gate') {
    const gState = document.createElement('div');
    gState.className = 'm-lcd m-mono';
    gState.style.cssText = 'padding:5px 8px;border-radius:4px;font-size:10px;color:#ffb463;';
    gState.textContent = `GATE: ${(store.telemetry.gate_state ?? 'open').toUpperCase()} · GR ${(store.telemetry.gain_reduction_db ?? 0).toFixed(1)} dB`;
    inspCardBody.appendChild(gState);
  }

  mainArea.append(stage, inspRoot);
  root.append(toolbar, grid, mainArea);
  return root;
}
