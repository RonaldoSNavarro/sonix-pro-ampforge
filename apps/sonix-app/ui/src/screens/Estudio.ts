import type { AppStore, BlockId } from '../store.ts';
import { createAmpHeadElement, type AmpModelId } from '../components/AmpHead.ts';
import { createKnob } from '../components/Knob.ts';
import { createPedalElement, PEDAL_SIZES, type PedalModelId } from '../components/Pedal.ts';
import { createRackElement } from '../components/Rack.ts';
import { createTubeElement } from '../components/Tube.ts';
import {
  bool,
  makeChipButton,
  makeControlSlider,
  makeToggleRow,
  str,
} from './_helpers.ts';

interface GridBlockDef {
  id: BlockId;
  row: 1 | 2;
  col: number;
  tag: string;
  title: string;
  dotColor: string;
  chTag: string;
  enableKey?: string;
  bypassControlId?: string;
  subText: (store: AppStore) => string;
}

const AMP_LABELS: Record<string, { short: string; full: string; ref: string }> = {
  plexi_1959: { short: 'Plexi 1959', full: 'BRIT PLEXI · 1959', ref: 'Ref.: Marshall Super Lead 100W · 4× EL34' },
  jcm800_2203: { short: '2203', full: 'BRIT 800 · 2203', ref: 'Ref.: Marshall JCM800 2203 · 4× EL34' },
  twin_ab763: { short: 'Twin', full: 'TWIN REVERB · AB763', ref: 'Ref.: Fender Blackface AB763 · 4× 6L6GC' },
  dual_rect: { short: 'Recto', full: 'CALI RECTO · DUAL RECT', ref: 'Ref.: Mesa/Boogie Dual Rectifier Rev F · 4× 6L6GC' },
  ac30_tb: { short: 'AC30 TB', full: 'VOX · AC30/6 TB', ref: 'Ref.: Vox AC30 Top Boost · 4× EL84' },
};

const AMP2_MINI_TITLES: Record<string, string> = {
  twin_ab763: 'US TWIN',
  plexi_1959: 'BRIT PLEXI 1959',
  jcm800_2203: 'BRIT 800 2203',
  dual_rect: 'CALI DUAL RECT',
  ac30_tb: 'AC30 TOP BOOST',
};

const MOD_MODEL_MAP: Record<string, { pedal: PedalModelId; label: string }> = {
  AnalogBbdChorus: { pedal: 'ce2', label: 'Chorus-2' },
  StereoTriChorus: { pedal: 'trichorus', label: 'Chorus 3 vozes (mono)' },
  ThroughZeroFlanger: { pedal: 'flanger', label: 'TZ Flanger' },
  Orange90Phaser: { pedal: 'phaser90', label: 'Phaser 90' },
  OpticalBiasTremolo: { pedal: 'tremolo', label: 'Tremolo' },
  PhotocellUnivibe: { pedal: 'univibe', label: 'Uni-Vibe' },
  RotarySpeaker: { pedal: 'rotary', label: 'Rotary 147' },
};

const PITCH_MODEL_MAP: Record<string, { pedal: PedalModelId; label: string }> = {
  whammy: { pedal: 'whammy', label: 'Whammy' },
  drop_capo: { pedal: 'drop', label: 'Drop Capo' },
  harmonizer: { pedal: 'harmonizer', label: 'Harmonizer' },
  micro_detune: { pedal: 'detune', label: 'Detune' },
};

const CAB_LABELS: Record<string, string> = {
  v30_4x12: '4×12 V30',
  greenback_4x12: '4×12 Greenback',
  alnico_blue_2x12: '2×12 Alnico',
  twin_2x12_open: '2×12 Open',
};

const DRIVE_LABELS: Record<string, string> = {
  ts808: 'Screamer 808',
  klon: 'Golden Klone',
  rat: 'Rodent',
  muff: 'Big Fuzz Pi',
  fuzzface: 'Fuzz Face',
  ds1: 'DS-One',
};

const GRID_BLOCKS: GridBlockDef[] = [
  { id: 'in', row: 1, col: 1, tag: 'IN', title: 'ENTRADA · HI-Z', dotColor: '#3dd8ff', chTag: '-', subText: () => 'Guitarra Hi-Z' },
  { id: 'gate', row: 1, col: 2, tag: 'GATE', title: 'NOISE GATE · NS-2', dotColor: '#4ee680', chTag: 'A', enableKey: 'gate.enabled', bypassControlId: 'gate.enabled', subText: () => 'Noise Gate' },
  { id: 'wah', row: 2, col: 2, tag: 'WAH', title: 'CRY WAH · 500mH', dotColor: '#3dd8ff', chTag: 'A', enableKey: 'wah_1.enabled', bypassControlId: 'wah_1.enabled', subText: () => 'Cry Wah' },
  { id: 'drive', row: 1, col: 3, tag: 'DRIVE', title: 'PEDAL DRIVE', dotColor: '#ff8a3d', chTag: 'B', enableKey: 'drive.enabled', bypassControlId: 'drive.enabled', subText: (s) => DRIVE_LABELS[str(s, 'drive.model', 'ts808')] ?? 'Screamer 808' },
  { id: 'split', row: 1, col: 4, tag: 'SPLIT', title: 'SPLIT A / B', dotColor: '#3dd8ff', chTag: '-', enableKey: 'split.enabled', bypassControlId: 'split.enabled', subText: () => 'Split A/B' },
  { id: 'amp', row: 1, col: 5, tag: 'AMP 1', title: 'AMPLIFICADOR 1', dotColor: '#ff8a3d', chTag: 'D', enableKey: 'amp_1.enabled', bypassControlId: 'amp.enabled', subText: (s) => (AMP_LABELS[str(s, 'amp_1.model', 'dual_rect')]?.full.split(' · ')[0] ?? 'CALI RECTO') },
  { id: 'cab', row: 1, col: 6, tag: 'CAB 1', title: 'GABINETE 1', dotColor: '#ffd166', chTag: 'A', enableKey: 'cab_1.enabled', bypassControlId: 'cab.enabled', subText: (s) => CAB_LABELS[str(s, 'cab_1.speaker', str(s, 'cab_1.model', 'v30_4x12'))] ?? '4×12 V30' },
  { id: 'eq', row: 1, col: 7, tag: 'EQ', title: '10 BAND EQ', dotColor: '#4ee680', chTag: 'A', enableKey: 'eq_1.enabled', bypassControlId: 'eq_1.enabled', subText: () => '10 Band EQ' },
  { id: 'merge', row: 1, col: 8, tag: 'MERGE', title: 'MERGE A + B', dotColor: '#3dd8ff', chTag: '-', subText: () => 'Mix A+B' },
  { id: 'delay', row: 1, col: 9, tag: 'DELAY', title: 'TAPE ECHO RE-201', dotColor: '#3dd8ff', chTag: 'B', enableKey: 'dly_1.enabled', bypassControlId: 'dly_1.enabled', subText: () => 'Tape Echo 201' },
  { id: 'reverb', row: 1, col: 10, tag: 'REVERB', title: 'STUDIO PLATE', dotColor: '#9c7cff', chTag: 'A', enableKey: 'rev_1.enabled', bypassControlId: 'rev_1.enabled', subText: () => 'Studio Plate' },
  { id: 'out', row: 1, col: 11, tag: 'OUT', title: 'SAÍDA 1 / 2', dotColor: '#3dd8ff', chTag: '-', subText: () => 'Saída 1/2' },
  { id: 'comp', row: 2, col: 4, tag: 'COMP', title: 'FET 76 COMPRESSOR', dotColor: '#6f7785', chTag: 'A', enableKey: 'comp_2.enabled', bypassControlId: 'comp_2.enabled', subText: () => 'FET 76' },
  { id: 'amp2', row: 2, col: 5, tag: 'AMP 2', title: 'AMP PARALELO 2', dotColor: '#ff8a3d', chTag: 'A', enableKey: 'amp_2.enabled', bypassControlId: 'amp_2.enabled', subText: (s) => (str(s, 'amp_2.model', 'twin_ab763') === 'twin_ab763' ? 'US Twin' : (AMP_LABELS[str(s, 'amp_2.model', 'twin_ab763')]?.short ?? 'US Twin')) },
  { id: 'cab2', row: 2, col: 6, tag: 'CAB 2', title: 'CAB PARALELO 2', dotColor: '#ffd166', chTag: 'A', enableKey: 'cab_2.enabled', bypassControlId: 'cab_2.enabled', subText: (s) => CAB_LABELS[str(s, 'cab_2.speaker', str(s, 'cab_2.model', 'alnico_blue_2x12'))] ?? '2×12 Alnico' },
  { id: 'chorus', row: 2, col: 7, tag: 'MOD', title: 'MODULAÇÃO', dotColor: '#9c7cff', chTag: 'A', enableKey: 'mod_2.enabled', bypassControlId: 'mod_2.enabled', subText: (s) => MOD_MODEL_MAP[str(s, 'mod_2.model', 'AnalogBbdChorus')]?.label ?? 'Chorus-2' },
];

function tubeComplementForModel(model: AmpModelId): string[] {
  switch (model) {
    case 'twin_ab763':
      return ['12AX7', '12AX7', '12AT7', '12AX7', '6L6GC', '6L6GC', '6L6GC', '6L6GC'];
    case 'dual_rect':
      return ['12AX7', '12AX7', '12AX7', '12AX7', '6L6GC', '6L6GC', '6L6GC', '6L6GC'];
    case 'ac30_tb':
      return ['ECC83', 'ECC83', 'ECC83', 'ECC82', 'EL84', 'EL84', 'EL84', 'EL84'];
    default:
      return ['12AX7', '12AX7', '12AX7', '12AT7', 'EL34', 'EL34', 'EL34', 'EL34'];
  }
}

function showPresetMenuPopover(anchor: HTMLElement, store: AppStore): void {
  document.querySelectorAll('.sonix-preset-pop').forEach((el) => el.remove());
  const rect = anchor.getBoundingClientRect();
  const pop = document.createElement('div');
  pop.className = 'sonix-preset-pop';
  pop.style.cssText = `
    position:fixed;left:${Math.round(rect.left)}px;top:${Math.round(rect.bottom + 4)}px;z-index:999;
    width:${Math.round(rect.width)}px;max-height:280px;overflow:auto;padding:4px;border-radius:6px;
    background:#171a20;border:1px solid #353c48;box-shadow:0 12px 28px rgba(0,0,0,.8);
    display:flex;flex-direction:column;gap:2px;
  `;
  store.presets.forEach((p, idx) => {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.dataset.controlId = 'topbar.preset_load';
    btn.style.cssText = `
      height:26px;padding:0 10px;border:0;border-radius:4px;text-align:left;font-size:11px;cursor:pointer;
      background:${idx === store.presetIdx ? '#2a3340' : 'transparent'};
      color:${idx === store.presetIdx ? '#ff9f4d' : '#dbe1e9'};
    `;
    btn.textContent = p.name;
    btn.addEventListener('click', () => {
      pop.remove();
      void store.loadPresetAt(idx);
    });
    pop.appendChild(btn);
  });
  const manageBtn = document.createElement('button');
  manageBtn.type = 'button';
  manageBtn.dataset.controlId = 'ui.nav_screen';
  manageBtn.style.cssText =
    'height:26px;padding:0 10px;border:0;border-top:1px solid #2b313c;margin-top:4px;border-radius:4px;text-align:left;font-size:11px;cursor:pointer;background:transparent;color:#93c5fd;';
  manageBtn.textContent = 'Gerenciar pacotes (.sforgepkg)…';
  manageBtn.addEventListener('click', () => {
    pop.remove();
    store.libraryTab = 'presets';
    store.setScreen('biblioteca');
  });
  pop.appendChild(manageBtn);
  document.body.appendChild(pop);
  const closeFn = (ev: MouseEvent) => {
    if (!pop.contains(ev.target as Node) && !anchor.contains(ev.target as Node)) {
      pop.remove();
      window.removeEventListener('mousedown', closeFn);
    }
  };
  window.setTimeout(() => window.addEventListener('mousedown', closeFn), 0);
}

function renderInspectorBody(store: AppStore, container: HTMLElement): void {
  const blk = store.selectedBlock;
  if (blk === 'in') {
    const chips = document.createElement('div');
    chips.style.cssText = 'display:flex;gap:4px;flex-wrap:wrap;';
    const curSrc = str(store, 'input.source', 'device');
    for (const [val, lbl] of [
      ['device', 'Guitarra Hi-Z'],
      ['test_riff', 'Riff Interno'],
      ['di_file', 'Arquivo DI'],
    ] as const) {
      chips.appendChild(makeChipButton('main.input_source', lbl, curSrc === val, () => void store.setControl('main.input_source', val)));
    }
    container.append(chips, makeControlSlider(store, 'main.input_boost', 'input.boost', 'Ganho de Entrada', 1.0, (v) => `${v.toFixed(2)}x`));
    return;
  }

  if (blk === 'gate') {
    const gr = document.createElement('div');
    gr.dataset.live = 'gate-gr';
    gr.className = 'm-mono';
    gr.style.cssText = 'font-size:11px;color:#8be8a2;background:#101418;border:1px solid #222a32;border-radius:4px;padding:4px 8px;';
    const gateOpen = store.telemetry.gate_state === 'open';
    const gateGr = store.telemetry.gate_gr_db ?? (gateOpen ? 0 : -18.2);
    gr.textContent = `GR ${gateGr.toFixed(1)} dB`;
    container.append(
      gr,
      makeControlSlider(store, 'gate.threshold_db', 'gate.threshold_db', 'Threshold', -65, (v) => `${v.toFixed(0)} dB`),
      makeControlSlider(store, 'gate.release_ms', 'gate.release_ms', 'Decay / Release', 80, (v) => `${v.toFixed(0)} ms`),
      makeControlSlider(store, 'gate.hysteresis_db', 'gate.hysteresis_db', 'Hysteresis', 4.0, (v) => `${v.toFixed(1)} dB`),
      makeControlSlider(store, 'gate.hold_ms', 'gate.hold_ms', 'Hold', 35, (v) => `${v.toFixed(0)} ms`),
      makeControlSlider(store, 'gate.range_db', 'gate.range_db', 'Range', -70, (v) => `${v.toFixed(0)} dB`)
    );
    return;
  }

  if (blk === 'drive') {
    const chips = document.createElement('div');
    chips.style.cssText = 'display:flex;gap:4px;flex-wrap:wrap;';
    const cur = str(store, 'drive.model', 'ts808');
    for (const [val, lbl] of Object.entries(DRIVE_LABELS)) {
      chips.appendChild(makeChipButton('drive.model', lbl, cur === val, () => void store.setControl('drive.model', val)));
    }
    container.append(
      chips,
      makeControlSlider(store, 'drive.drive', 'drive.drive', 'Drive', 4.5),
      makeControlSlider(store, 'drive.tone', 'drive.tone', 'Tone', 5.5),
      makeControlSlider(store, 'drive.level', 'drive.level', 'Level', 6.0)
    );
    return;
  }

  if (blk === 'comp') {
    container.append(
      makeControlSlider(store, 'comp_2.input', 'comp_2.input', 'Input', 5.0),
      makeControlSlider(store, 'comp_2.output', 'comp_2.output', 'Output', 5.0),
      makeControlSlider(store, 'comp_2.attack', 'comp_2.attack', 'Attack', 4.0),
      makeControlSlider(store, 'comp_2.release', 'comp_2.release', 'Release', 4.0)
    );
    return;
  }

  if (blk === 'amp' || blk === 'amp2') {
    if (blk === 'amp') {
      const activeModel = str(store, 'amp_1.model', 'dual_rect') as AmpModelId;
      const ampSliders: Array<[string, string, string, number]> = [
        ['amp.gain', 'amp_1.gain', 'Gain', 7.0],
        ['amp.volume_2', 'amp_1.volume_2', 'Vol II', 4.5],
        ['amp.treble', 'amp_1.treble', 'Treble', 6.0],
        ['amp.middle', 'amp_1.middle', 'Mid', 4.0],
        ['amp.bass', 'amp_1.bass', 'Bass', 6.0],
        ['amp.presence', 'amp_1.presence', 'Presence', 5.0],
        ['amp.cut', 'amp_1.cut', 'Tone Cut', 5.0],
        ['amp.master', 'amp_1.master', 'Master', 3.5],
        ['amp.reverb', 'amp_1.reverb', 'Reverb', 3.0],
      ];
      const slidersToRender = ampSliders
        .filter(([cid]) => {
          const models = store.getDescriptor(cid)?.models;
          return !models || models.includes(activeModel);
        })
        .map(([cid, key, label, def]) => makeControlSlider(store, cid, key, label, def));
      slidersToRender.push(
        makeControlSlider(store, 'chassi.sag', 'amp_1.sag', 'Sag da fonte', 3.0),
        makeControlSlider(store, 'chassi.nfb', 'amp_1.nfb', 'Realim. (NFB)', 5.0)
      );
      container.append(...slidersToRender);
    } else {
      const chips = document.createElement('div');
      chips.style.cssText = 'display:flex;gap:4px;flex-wrap:wrap;';
      const cur = str(store, 'amp_2.model', 'twin_ab763');
      for (const [val, meta] of Object.entries(AMP_LABELS)) {
        chips.appendChild(makeChipButton('amp_2.model', meta.short, cur === val, () => void store.setControl('amp_2.model', val)));
      }
      container.append(
        chips,
        makeControlSlider(store, 'amp_2.gain', 'amp_2.gain', 'Gain (AMP 2)', 5.0),
        makeControlSlider(store, 'amp_2.bass', 'amp_2.bass', 'Bass', 5.0),
        makeControlSlider(store, 'amp_2.middle', 'amp_2.middle', 'Mid', 5.0),
        makeControlSlider(store, 'amp_2.treble', 'amp_2.treble', 'Treble', 5.0),
        makeControlSlider(store, 'amp_2.master', 'amp_2.master', 'Master', 7.0),
        makeControlSlider(store, 'merge.row_1_level_db', 'merge.row_1_level_db', 'Nível Merge', -6.0, (v) => `${v.toFixed(1)} dB`)
      );
    }
    return;
  }

  if (blk === 'cab' || blk === 'cab2') {
    const isB = blk === 'cab2';
    const spkKey = isB ? 'cab_2.speaker' : 'cab_1.speaker';
    const ctrlId = isB ? 'cab_2.speaker' : 'cab.speaker';
    const resId = isB ? 'cab_2.resonance_hz' : 'cab.resonance_hz';
    const resKey = isB ? 'cab_2.resonance_hz' : 'cab_1.resonance_hz';
    const hcId = isB ? 'cab_2.hicut_khz' : 'cab.hicut_khz';
    const hcKey = isB ? 'cab_2.hicut_khz' : 'cab_1.hicut_khz';
    const chips = document.createElement('div');
    chips.style.cssText = 'display:flex;gap:4px;flex-wrap:wrap;';
    const cur = str(store, spkKey, isB ? 'alnico_blue_2x12' : 'v30_4x12');
    const irActive = !isB && bool(store, 'cab_1.ir_enabled', false);
    for (const [val, lbl] of Object.entries(CAB_LABELS)) {
      const b = makeChipButton(ctrlId, lbl, cur === val, () => void store.setControl(ctrlId, val));
      b.disabled = irActive;
      if (irActive) b.title = 'IR carregada substitui o alto-falante';
      chips.appendChild(b);
    }
    container.append(
      chips,
      makeControlSlider(store, resId, resKey, 'Ressonância', 92, (v) => `${v.toFixed(0)} Hz`),
      makeControlSlider(store, hcId, hcKey, 'Hi-Cut', 7.5, (v) => `${v.toFixed(1)} kHz`)
    );
    const navCab = makeChipButton('ui.nav_screen', 'Sala de microfonação ›', false, () => store.setScreen('gabinete'));
    container.appendChild(navCab);
    return;
  }

  if (blk === 'eq') {
    container.append(
      makeControlSlider(store, 'eq_1.gain_db', 'eq_1.gain_db', 'Gain Pré-EQ', 0, (v) => `${v.toFixed(1)} dB`),
      makeControlSlider(store, 'eq_1.volume_db', 'eq_1.volume_db', 'Output Volume', 0, (v) => `${v.toFixed(1)} dB`)
    );
    return;
  }

  if (blk === 'chorus') {
    const modModels: [string, string][] = [
      ['AnalogBbdChorus', 'CE-2 Chorus'],
      ['StereoTriChorus', 'Chorus 3 vozes (mono)'],
      ['ThroughZeroFlanger', 'TZ Flanger'],
      ['Orange90Phaser', 'Phaser 90'],
      ['OpticalBiasTremolo', 'Tremolo'],
      ['PhotocellUnivibe', 'Uni-Vibe'],
      ['RotarySpeaker', 'Rotary'],
    ];
    const chips = document.createElement('div');
    chips.style.cssText = 'display:flex;gap:4px;flex-wrap:wrap;';
    const cur = str(store, 'mod_2.model', 'AnalogBbdChorus');
    for (const [val, lbl] of modModels) {
      chips.appendChild(makeChipButton('mod_2.model', lbl, cur === val, () => void store.setControl('mod_2.model', val)));
    }
    container.append(
      chips,
      makeControlSlider(store, 'mod_2.rate_hz', 'mod_2.rate_hz', 'Rate', 1.0, (v) => `${v.toFixed(2)} Hz`),
      makeControlSlider(store, 'mod_2.depth', 'mod_2.depth', 'Depth', 5.0),
      makeControlSlider(store, 'mod_2.mix', 'mod_2.mix', 'Mix', 50, (v) => `${Math.round(v)}%`)
    );
    return;
  }

  if (blk === 'pitch') {
    const pitchModels: [string, string][] = [
      ['whammy', 'Whammy'],
      ['drop_capo', 'Drop Capo'],
      ['harmonizer', 'Harmonizer'],
      ['micro_detune', 'Detune'],
    ];
    const chips = document.createElement('div');
    chips.style.cssText = 'display:flex;gap:4px;flex-wrap:wrap;';
    const cur = str(store, 'pitch_1.model', 'whammy');
    for (const [val, lbl] of pitchModels) {
      chips.appendChild(makeChipButton('pitch_1.model', lbl, cur === val, () => void store.setControl('pitch_1.model', val)));
    }
    container.append(
      makeToggleRow(store, 'pitch_1.enabled', 'pitch_1.enabled', 'Ativo', false),
      chips,
      makeControlSlider(store, 'pitch_1.semitones', 'pitch_1.semitones', 'Semitons', 0, (v) => `${v >= 0 ? '+' : ''}${Math.round(v)} st`),
      makeControlSlider(store, 'pitch_1.mix', 'pitch_1.mix', 'Mix', 100, (v) => `${Math.round(v)}%`)
    );
    return;
  }

  if (blk === 'wah') {
    const wahModes: [string, string][] = [
      ['crybaby_gcb95', 'CryBaby GCB95'],
      ['clyde_mccoy', 'Clyde McCoy'],
      ['auto_envelope', 'Auto-Wah'],
    ];
    const chips = document.createElement('div');
    chips.style.cssText = 'display:flex;gap:4px;flex-wrap:wrap;';
    const cur = str(store, 'wah_1.mode', 'crybaby_gcb95');
    for (const [val, lbl] of wahModes) {
      chips.appendChild(makeChipButton('wah_1.mode', lbl, cur === val, () => void store.setControl('wah_1.mode', val)));
    }
    container.append(
      makeToggleRow(store, 'wah_1.enabled', 'wah_1.enabled', 'Wah Ativo', false),
      chips,
      makeControlSlider(
        store,
        'wah_1.position',
        'wah_1.position',
        'Varredura (Pos)',
        0.5,
        (v) => `${Math.round(350 * (2200 / 350) ** v)} Hz`
      ),
      makeControlSlider(store, 'wah_1.q', 'wah_1.q', 'Ressonância Q', 5.0, (v) => v.toFixed(1))
    );
    return;
  }

  if (blk === 'delay') {
    container.append(
      makeControlSlider(store, 'dly_1.repeat_rate', 'dly_1.repeat_rate', 'Repeat Rate', 5.0),
      makeControlSlider(store, 'dly_1.intensity', 'dly_1.intensity', 'Intensity', 4.5),
      makeControlSlider(store, 'dly_1.echo_volume', 'dly_1.echo_volume', 'Echo Volume', 5.0),
      makeToggleRow(store, 'dly_1.spillover', 'dly_1.spillover', 'Spillover', true)
    );
    return;
  }

  if (blk === 'reverb') {
    container.append(
      makeControlSlider(store, 'rev_1.decay_s', 'rev_1.decay_s', 'Decay', 2.4, (v) => `${v.toFixed(1)} s`),
      makeControlSlider(store, 'rev_1.predelay_ms', 'rev_1.predelay_ms', 'Pre-Delay', 18, (v) => `${v.toFixed(0)} ms`),
      makeControlSlider(store, 'rev_1.damping', 'rev_1.damping', 'Damping', 0.45),
      makeControlSlider(store, 'rev_1.mix', 'rev_1.mix', 'Mix', 28, (v) => `${Math.round(v)}%`)
    );
    return;
  }

  if (blk === 'split') {
    const modeRow = document.createElement('div');
    modeRow.style.cssText = 'display:flex;gap:4px;flex-wrap:wrap;';
    const curMode = str(store, 'split.mode', 'ab').toLowerCase();
    for (const [val, lbl] of [
      ['ab', 'A/B'],
      ['crossover', 'Crossover'],
      ['dynamic_split', 'Dinâmico'],
    ] as const) {
      modeRow.appendChild(
        makeChipButton('split.mode', lbl, curMode === val, () => void store.setControl('split.mode', val))
      );
    }
    container.append(
      modeRow,
      makeControlSlider(store, 'split.balance', 'split.balance', 'Balanço A/B', 0.5, (v) => `${Math.round(v * 100)}%`),
      makeControlSlider(store, 'split.crossover_hz', 'split.crossover_hz', 'Crossover LR4', 650, (v) => `${v.toFixed(0)} Hz`),
      makeControlSlider(store, 'split.dyn_threshold_db', 'split.dyn_threshold_db', 'Threshold Dinâmico', -30, (v) => `${v.toFixed(1)} dB`),
      makeControlSlider(store, 'split.dyn_attack_ms', 'split.dyn_attack_ms', 'Ataque Dinâmico', 5, (v) => `${v.toFixed(1)} ms`),
      makeControlSlider(store, 'split.dyn_release_ms', 'split.dyn_release_ms', 'Release Dinâmico', 80, (v) => `${v.toFixed(0)} ms`),
      makeControlSlider(store, 'merge.row_1_level_db', 'merge.row_1_level_db', 'Balanço Linha B', -6, (v) => `${v.toFixed(1)} dB`)
    );
    return;
  }

  if (blk === 'out') {
    const outChips = document.createElement('div');
    outChips.style.cssText = 'display:flex;gap:4px;flex-wrap:wrap;';
    const supportsMultiCh = Number(store.params['audio.channels'] ?? 2) > 2;
    const taps: Array<{ id: string; label: string; multiOnly?: boolean }> = [
      { id: 'MAIN_XLR_LR', label: 'MAIN XLR L/R' },
      { id: 'AMP_OUT_P10_NO_CAB', label: 'AMP OUT P10 (Sem Cab)' },
      { id: 'FX_SEND', label: 'FX SEND', multiOnly: true },
      { id: 'USB_REC_OUT', label: 'USB REC OUT', multiOnly: true },
    ];
    const activeTap = str(store, 'ui.output_tap', 'MAIN_XLR_LR');
    for (const tap of taps) {
      const btn = makeChipButton('ui.output_tap', tap.label, activeTap === tap.id, () => {
        if (tap.multiOnly && !supportsMultiCh) return;
        store.params['ui.output_tap'] = tap.id;
        store.selectBlock('out');
      });
      if (tap.multiOnly && !supportsMultiCh) {
        btn.disabled = true;
        btn.title = 'Requer interface de áudio com >2 canais de saída (dispositivo estéreo 2ch atual).';
        btn.style.opacity = '0.45';
      }
      outChips.appendChild(btn);
    }
    const warn = document.createElement('div');
    warn.style.cssText = 'font-size:11px;color:#ffd166;';
    warn.textContent =
      activeTap === 'AMP_OUT_P10_NO_CAB'
        ? 'Aviso: AMP_OUT_P10_NO_CAB envia sinal pré-gabinete direto para power amp + caixa real.'
        : 'Saída principal L/R com simulação de gabinete ativa.';
    container.append(
      outChips,
      warn,
      makeControlSlider(store, 'master.level_db', 'master.level_db', 'Nível Master', 0, (v) => `${v.toFixed(1)} dB`),
      makeToggleRow(store, 'merge.row_1_phase_invert', 'merge.row_1_phase_invert', 'Inverter Fase B', false, '180°', '0°')
    );
    return;
  }

  // Bloco MERGE A + B
  container.append(
    makeControlSlider(store, 'merge.row_1_level_db', 'merge.row_1_level_db', 'Nível Linha B (Merge)', -6, (v) => `${v.toFixed(1)} dB`),
    makeToggleRow(store, 'merge.row_1_phase_invert', 'merge.row_1_phase_invert', 'Inverter Fase B', false, '180°', '0°')
  );
}

export function renderEstudioScreen(store: AppStore): HTMLElement {
  const root = document.createElement('div');
  root.dataset.screen = 'main';
  root.style.cssText = 'flex:1;min-height:0;display:flex;flex-direction:column;overflow:hidden;background:#0b0c0e;';

  // ── 1. BRUSHED TOOLBAR (68 px, zero <select>) ────────────────────────────
  const toolbar = document.createElement('section');
  toolbar.dataset.region = 'toolbar';
  toolbar.className = 'm-brushed';
  toolbar.style.cssText = `
    height:68px;flex:none;border-bottom:1px solid #000;padding:0 16px;position:relative;
    display:flex;align-items:center;gap:16px;
    box-shadow:inset 0 1px 0 rgba(255,255,255,.07);
  `;
  for (const pos of ['top:5px;left:5px', 'top:5px;right:5px', 'bottom:5px;left:5px', 'bottom:5px;right:5px']) {
    const sc = document.createElement('span');
    sc.className = 'm-screw';
    sc.style.cssText = `position:absolute;${pos};`;
    toolbar.appendChild(sc);
  }

  const navGrp = document.createElement('div');
  navGrp.style.cssText = 'display:flex;align-items:center;gap:6px;margin-left:10px;flex:none;';

  const prevBtn = document.createElement('button');
  prevBtn.type = 'button';
  prevBtn.className = 'm-btn';
  prevBtn.dataset.controlId = 'topbar.preset_nav';
  prevBtn.style.cssText = 'width:30px;height:44px;font-size:16px;';
  prevBtn.textContent = '‹';
  prevBtn.addEventListener('click', () => {
    const n = Math.max(1, store.presets.length);
    void store.loadPresetAt((store.presetIdx - 1 + n) % n);
  });

  const lcd = document.createElement('button');
  lcd.type = 'button';
  lcd.className = 'm-lcd';
  lcd.dataset.controlId = 'topbar.preset_load';
  lcd.style.cssText =
    'width:330px;height:46px;border-radius:5px;padding:0 14px;box-sizing:border-box;display:flex;flex-direction:column;justify-content:center;gap:2px;text-align:left;cursor:pointer;';
  const activePreset = store.presets[store.presetIdx];
  const ampId = str(store, 'amp_1.model', 'dual_rect') as AmpModelId;
  const activeScene = str(store, 'scene.active', 'A');
  const cleanName = (activePreset?.name ?? 'CALI DUAL RECTO MODERN METAL').replace(/^P0\d\s+/, '').toUpperCase();
  lcd.innerHTML = `
    <div style="width:100%;display:flex;gap:10px;align-items:center;font-family:'JetBrains Mono',monospace;font-size:10px;opacity:.85;">
      <span class="m-lcd-s">SHOW 2026</span>
      <span class="m-lcd-s">P0${store.presetIdx + 1}</span>
      <span style="flex:1;"></span>
      <span class="m-lcd-s">CENA ${activeScene} · HEAVY</span>
    </div>
    <div class="m-lcd-t m-mono" style="width:100%;font-size:15px;font-weight:700;letter-spacing:.04em;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
      ${cleanName}
    </div>
  `;
  lcd.addEventListener('click', () => showPresetMenuPopover(lcd, store));

  const nextBtn = document.createElement('button');
  nextBtn.type = 'button';
  nextBtn.className = 'm-btn';
  nextBtn.dataset.controlId = 'topbar.preset_nav';
  nextBtn.style.cssText = 'width:30px;height:44px;font-size:16px;';
  nextBtn.textContent = '›';
  nextBtn.addEventListener('click', () => {
    const n = Math.max(1, store.presets.length);
    void store.loadPresetAt((store.presetIdx + 1) % n);
  });

  const saveBtn = document.createElement('button');
  saveBtn.type = 'button';
  saveBtn.className = 'm-btn m-pri';
  saveBtn.dataset.controlId = 'topbar.preset_save';
  saveBtn.style.cssText = 'height:44px;padding:0 12px;font-size:11px;font-weight:700;';
  saveBtn.textContent = 'SALVAR';
  saveBtn.addEventListener('click', () => void store.saveActivePreset());

  navGrp.append(prevBtn, lcd, nextBtn, saveBtn);
  toolbar.appendChild(navGrp);

  const sceneBox = document.createElement('div');
  sceneBox.style.cssText = 'display:flex;flex-direction:column;gap:4px;flex:none;';
  const sLbl = document.createElement('span');
  sLbl.className = 'm-lbl';
  sLbl.style.cssText = 'font-size:9px;';
  sLbl.textContent = 'Cenas';
  const sRow = document.createElement('div');
  sRow.style.cssText = 'display:flex;gap:4px;';
  for (const sc of ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H']) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'm-btn m-osw';
    b.dataset.controlId = 'topbar.scene';
    b.setAttribute('aria-pressed', activeScene === sc ? 'true' : 'false');
    b.style.cssText = 'width:32px;height:30px;font-size:12px;font-weight:700;';
    b.textContent = sc;
    b.addEventListener('click', () => void store.selectScene(sc));
    sRow.appendChild(b);
  }
  sceneBox.append(sLbl, sRow);
  toolbar.appendChild(sceneBox);

  const metersBox = document.createElement('div');
  metersBox.style.cssText = 'display:flex;align-items:center;gap:22px;padding-top:1px;margin-left:auto;flex:none;';

  const ledStack = document.createElement('div');
  ledStack.style.cssText = 'display:flex;flex-direction:column;gap:5px;';
  for (const [mKey, mLbl] of [
    ['in', 'IN'],
    ['out', 'OUT'],
  ] as const) {
    const line = document.createElement('div');
    line.style.cssText = 'display:flex;align-items:center;gap:6px;';
    const lbl = document.createElement('span');
    lbl.className = 'm-lbl';
    lbl.style.cssText = 'width:26px;font-size:9px;';
    lbl.textContent = mLbl;
    const bar = document.createElement('div');
    bar.dataset.meter = mKey;
    bar.style.cssText = 'display:flex;gap:2px;padding:3px;background:#050505;border-radius:3px;';
    for (let i = 0; i < 20; i += 1) {
      const seg = document.createElement('span');
      seg.className = 'm-seg';
      seg.style.background = '#20242b';
      bar.appendChild(seg);
    }
    line.append(lbl, bar);
    ledStack.appendChild(line);
  }

  const cpuBox = document.createElement('div');
  cpuBox.className = 'm-mono';
  cpuBox.style.cssText = 'display:flex;flex-direction:column;align-items:flex-end;gap:2px;font-size:10px;white-space:nowrap;color:#a3abb4;';
  const cpuPct = store.telemetry.cpu_usage_pct ?? 0;
  const osVal = str(store, 'engine.oversampling', '4x');
  cpuBox.innerHTML = `
    <div>CPU <b data-live="cpu-pct" style="color:#3dd8ff;">${cpuPct.toFixed(1)}%</b></div>
    <div>OS ${osVal}</div>
  `;
  metersBox.append(ledStack, cpuBox);
  toolbar.appendChild(metersBox);

  const rightBtns = document.createElement('div');
  rightBtns.style.cssText = 'display:flex;gap:6px;flex:none;';

  const tapBtn = document.createElement('button');
  tapBtn.type = 'button';
  tapBtn.className = 'm-btn';
  tapBtn.dataset.controlId = 'topbar.tap_tempo';
  tapBtn.style.cssText = 'width:54px;height:44px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:1px;';
  tapBtn.innerHTML = `<span class="m-mono" style="font-size:12px;font-weight:700;color:#c49a6c;">${Math.round(store.bpm)}</span><span style="font-size:8.5px;color:#8e96a0;letter-spacing:.08em;">TAP</span>`;
  tapBtn.addEventListener('click', () => store.tapTempo());

  const tunerBtn = document.createElement('button');
  tunerBtn.type = 'button';
  tunerBtn.className = 'm-btn';
  tunerBtn.dataset.controlId = 'ui.nav_screen';
  tunerBtn.dataset.navTarget = 'tuner';
  tunerBtn.style.cssText = 'width:64px;height:44px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:2px;';
  tunerBtn.innerHTML =
    '<svg width="18" height="12" viewBox="0 0 18 12" fill="none"><path d="M2 11A7 7 0 0 1 16 11" stroke="#3dd8ff" stroke-width="1.6"/><path d="M9 11L11.5 4" stroke="#29c46a" stroke-width="2" stroke-linecap="round"/></svg><span style="font-size:9px;color:#a3abb4;">AFINADOR</span>';
  tunerBtn.addEventListener('click', () => {
    store.palcoMode = 'tuner';
    store.setControl('tuner.enabled', true);
    store.setScreen('palco');
  });

  const palcoBtn = document.createElement('button');
  palcoBtn.type = 'button';
  palcoBtn.className = 'm-btn';
  palcoBtn.dataset.controlId = 'ui.nav_screen';
  palcoBtn.dataset.navTarget = 'palco';
  palcoBtn.style.cssText = 'width:64px;height:44px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:2px;';
  palcoBtn.innerHTML =
    '<svg width="16" height="12" viewBox="0 0 16 12" fill="none"><rect x="1" y="1" width="14" height="10" rx="1.5" stroke="#ff8a3d" stroke-width="1.4"/><path d="M4 8h8" stroke="#ff8a3d" stroke-width="1.4"/></svg><span style="font-size:9px;color:#a3abb4;">PALCO</span>';
  palcoBtn.addEventListener('click', () => {
    store.palcoMode = 'stage';
    store.setScreen('palco');
  });

  const gearBtn = document.createElement('button');
  gearBtn.type = 'button';
  gearBtn.className = 'm-btn';
  gearBtn.dataset.controlId = 'ui.nav_screen';
  gearBtn.setAttribute('aria-label', 'Configurações de Áudio');
  gearBtn.style.cssText = 'width:44px;height:44px;display:inline-flex;align-items:center;justify-content:center;';
  gearBtn.innerHTML =
    '<svg width="16" height="16" viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="2.3" stroke="currentColor" stroke-width="1.4"/><path d="M8 1.5V3M8 13V14.5M1.5 8H3M13 8H14.5M3.4 3.4L4.5 4.5M11.5 11.5L12.6 12.6M12.6 3.4L11.5 4.5M4.5 11.5L3.4 12.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/></svg>';
  gearBtn.addEventListener('click', () => store.setScreen('audio'));

  rightBtns.append(tapBtn, tunerBtn, palcoBtn, gearBtn);
  toolbar.appendChild(rightBtns);

  // ── 2. SIGNAL GRID (136 px, 11x2 fixed matrix) ───────────────────────────
  const gridSec = document.createElement('section');
  gridSec.dataset.region = 'grid';
  gridSec.style.cssText = `
    height:136px;flex:none;background:linear-gradient(180deg,#0e1013,#0a0b0d);border-bottom:1px solid #000;
    padding:9px 16px 6px;box-sizing:border-box;display:flex;flex-direction:column;gap:6px;
  `;

  const gHdr = document.createElement('div');
  gHdr.style.cssText = 'height:20px;flex:none;display:flex;align-items:center;gap:12px;';
  const gTitle = document.createElement('span');
  gTitle.className = 'm-lbl';
  gTitle.textContent = 'Matriz de sinal';
  const gHint = document.createElement('span');
  gHint.style.cssText = 'font-size:11px;color:#9aa3ad;';
  gHint.textContent = 'clique = editar · duplo clique = bypass · linha B em paralelo (split → merge)';

  const osGrp = document.createElement('div');
  osGrp.style.cssText = 'margin-left:auto;display:flex;align-items:center;gap:4px;';
  const osLbl = document.createElement('span');
  osLbl.className = 'm-lbl';
  osLbl.style.cssText = 'font-size:9px;';
  osLbl.textContent = 'Oversampling';
  osGrp.appendChild(osLbl);
  for (const factor of ['2x', '4x', '8x']) {
    const b = makeChipButton('topbar.oversampling', factor, osVal === factor, () => {
      void store.setControl('topbar.oversampling', factor);
    });
    b.className = 'm-chip m-mono';
    b.style.cssText = 'height:20px;padding:0 6px;';
    osGrp.appendChild(b);
  }
  gHdr.append(gTitle, gHint, osGrp);

  const matrix = document.createElement('div');
  matrix.style.cssText = `
    flex:1;min-height:0;display:grid;grid-template-columns:repeat(11,minmax(0,1fr));
    grid-template-rows:52px 44px;gap:6px 6px;align-items:stretch;
  `;

  const splitEnabled = bool(store, 'split.enabled', false);
  for (const blk of GRID_BLOCKS) {
    const cell = document.createElement('button');
    cell.type = 'button';
    cell.className = 'm-cell';
    cell.dataset.controlId = 'ui.inspector_select';
    cell.dataset.blockId = blk.id;
    const isSel = store.selectedBlock === blk.id;
    const enabled = blk.enableKey ? bool(store, blk.enableKey, true) : true;
    const isLine2Parallel = blk.row === 2 && blk.col >= 4;
    const line2Inactive = isLine2Parallel && !splitEnabled;
    if (line2Inactive) {
      cell.title = 'Linha 2 inativa (split desligado)';
      cell.dataset.line2Inactive = 'true';
    } else {
      cell.title = blk.title;
    }
    const effectiveOpacity = line2Inactive ? '0.35' : !enabled ? '0.46' : '1';
    cell.setAttribute('aria-pressed', isSel ? 'true' : 'false');
    cell.style.cssText = `
      grid-column:${blk.col};grid-row:${blk.row};
      height:${blk.row === 1 ? 52 : 44}px;
      padding:5px 8px;display:flex;flex-direction:column;justify-content:center;align-items:flex-start;gap:4px;
      opacity:${effectiveOpacity};color:#dbe1e8;text-align:left;cursor:pointer;
    `;

    const topRow = document.createElement('div');
    topRow.style.cssText = 'width:100%;display:flex;align-items:center;gap:5px;';
    const dot = document.createElement('span');
    dot.style.cssText = `width:6px;height:6px;border-radius:50%;flex:none;background:${
      enabled ? blk.dotColor : '#444b54'
    };box-shadow:${enabled ? `0 0 6px ${blk.dotColor}aa` : 'none'};`;
    const tag = document.createElement('span');
    tag.style.cssText = `font-size:9px;font-weight:700;letter-spacing:.08em;color:${blk.dotColor};`;
    tag.textContent = blk.tag;
    const ch = document.createElement('span');
    ch.className = 'm-mono';
    ch.style.cssText = 'margin-left:auto;font-size:9px;color:#6d7580;';
    ch.textContent = blk.chTag;
    topRow.append(dot, tag, ch);

    const sub = document.createElement('span');
    sub.style.cssText = 'font-size:11px;font-weight:500;color:#dbe1e8;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:100%;';
    sub.textContent = blk.subText(store);

    cell.append(topRow, sub);
    cell.addEventListener('click', () => store.selectBlock(blk.id));
    cell.addEventListener('dblclick', () => {
      if (blk.bypassControlId && blk.enableKey) {
        void store.setControl(blk.bypassControlId, !bool(store, blk.enableKey, true));
      }
    });
    matrix.appendChild(cell);
  }
  gridSec.append(gHdr, matrix);

  // ── 3. STAGE + INSPECTOR ────────────────────────────────────────────────
  const bodyArea = document.createElement('div');
  bodyArea.style.cssText = 'flex:1;min-height:0;display:grid;grid-template-columns:minmax(0,1fr) 300px;overflow:hidden;';

  const stage = document.createElement('section');
  stage.dataset.region = 'stage';
  stage.style.cssText =
    'min-width:0;min-height:0;background:radial-gradient(ellipse at 35% -10%,rgba(255,170,90,0.06) 0%,transparent 55%),repeating-linear-gradient(90deg,rgba(255,255,255,.012) 0 1px,transparent 1px 46px),linear-gradient(180deg,#141518 0%,#0d0e10 62%,#090a0b 100%);padding:14px 18px;display:flex;flex-direction:column;gap:14px;overflow:hidden;';

  const stageTop = document.createElement('div');
  stageTop.style.cssText = 'display:flex;gap:16px;height:226px;flex-shrink:0;';

  const ampSlot = document.createElement('div');
  ampSlot.dataset.region = 'amp-slot';
  ampSlot.style.cssText = 'width:788px;height:226px;flex-shrink:0;position:relative;';
  const ampScaleWrap = document.createElement('div');
  ampScaleWrap.style.cssText = 'position:absolute;left:0;top:5px;width:1000px;transform:scale(0.788);transform-origin:0 0;';
  ampScaleWrap.appendChild(createAmpHeadElement(store));
  ampSlot.appendChild(ampScaleWrap);

  const tubesCard = document.createElement('div');
  tubesCard.style.cssText =
    'flex:1;height:226px;border-radius:12px;background:#050505;box-shadow:0 20px 30px rgba(0,0,0,.7),0 0 0 1px #000;padding:8px;box-sizing:border-box;display:flex;flex-direction:column;gap:4px;min-width:0;';
  const tHdr = document.createElement('div');
  tHdr.style.cssText = 'display:flex;align-items:center;';
  const liveBPlus = store.telemetry.b_plus_v ?? numSafe(store, 'amp_1.preamp_plate_v', 280);
  tHdr.innerHTML = `<span class="m-lbl" style="flex:1;font-size:9px;">VÁLVULAS · ${str(store, 'amp_1.preamp_tube', '12ax7').toUpperCase()} · 4× ${str(store, 'amp_1.power_tube', '6l6gc').toUpperCase()}</span><span class="m-mono" data-live="b-plus" style="font-size:9px;color:#ffb347;">B+ ${Math.round(liveBPlus)} V</span>`;

  const tubesRow = document.createElement('div');
  tubesRow.style.cssText =
    'flex:1;border-radius:8px;position:relative;overflow:hidden;background:radial-gradient(ellipse at 50% 100%,rgba(255,120,30,.12) 0%,transparent 60%),linear-gradient(180deg,#0b0b0c,#17181a);display:flex;align-items:flex-end;justify-content:center;gap:6px;';
  const tubeNames = tubeComplementForModel(ampId);
  const ampIsHot = bool(store, 'amp_1.power', true) && !bool(store, 'amp_1.standby', false);
  tubeNames.forEach((tLabel, i) => {
    const isPwr = i >= 4;
    const tubeId = i === 3 ? 'PI' : `V${i + 1}`;
    const tubeEl = createTubeElement({
      id: tubeId,
      typeLabel: tLabel,
      width: isPwr ? 34 : 22,
      height: isPwr ? 108 : 58,
      radius: isPwr ? '12px 12px 6px 6px' : '10px 10px 4px 4px',
      baseWidth: isPwr ? 30 : 20,
      baseHeight: isPwr ? 14 : 8,
      baseBg: 'linear-gradient(180deg,#2a2a2a,#0a0a0a)',
      glowOp: ampIsHot ? (isPwr ? 1.0 : 0.78) : 0.15,
      glowPx: ampIsHot ? 10 : 1,
      selected: false,
      controlId: 'ui.nav_screen',
      onClick: () => {
        store.selectedTube = `V${i + 1}`;
        store.setScreen('chassi');
      },
    });
    tubeEl.style.padding = '0';
    tubeEl.style.marginBottom = '16px';
    const tLbl = tubeEl.querySelector('span.m-mono') as HTMLElement | null;
    if (tLbl) {
      tLbl.style.fontSize = '7.5px';
      tLbl.style.fontWeight = '400';
      tLbl.style.color = '#c9ced4';
    }
    tubesRow.appendChild(tubeEl);
  });
  const tubesDeck = document.createElement('div');
  tubesDeck.style.cssText = 'position:absolute;left:0;right:0;bottom:0;height:16px;background:linear-gradient(180deg,#8f949a,#5b6066 40%,#2c2f33);';
  tubesRow.appendChild(tubesDeck);

  const chassiLink = document.createElement('button');
  chassiLink.type = 'button';
  chassiLink.dataset.controlId = 'ui.nav_screen';
  chassiLink.style.cssText = 'border:0;background:transparent;color:#3dd8ff;font-size:10px;font-weight:600;cursor:pointer;padding:0;text-align:right;align-self:flex-end;';
  chassiLink.textContent = 'Abrir chassi / Mod Shop ›';
  chassiLink.addEventListener('click', () => store.setScreen('chassi'));
  tubesCard.append(tHdr, tubesRow, chassiLink);

  stageTop.append(ampSlot, tubesCard);

  const stageBottom = document.createElement('div');
  stageBottom.style.cssText = 'flex:1;min-height:0;display:flex;gap:16px;overflow:hidden;';

  const board = document.createElement('div');
  board.className = 'm-grip';
  board.style.cssText =
    'flex:1;min-width:0;border-radius:10px;position:relative;box-shadow:0 0 0 1px #000;padding:8px 16px 9px;box-sizing:border-box;display:flex;flex-direction:column;overflow:hidden;';
  const boardRail = document.createElement('div');
  boardRail.style.cssText =
    'position:absolute;left:0;right:0;bottom:0;height:10px;border-radius:0 0 10px 10px;background:linear-gradient(180deg,#5d6268,#2b2e32);pointer-events:none;';

  const boardTopLabels = document.createElement('div');
  boardTopLabels.style.cssText = 'display:flex;align-items:center;gap:8px;margin-bottom:8px;flex:none;position:relative;z-index:3;';
  boardTopLabels.innerHTML =
    '<span class="m-lbl" style="font-size:9px;">Pedalboard</span><span style="font-size:10px;color:#9aa3ad;">PRÉ-AMP</span><span style="flex:1;height:1px;background:#24282d;"></span><span style="font-size:10px;color:#9aa3ad;">LINHA B / PÓS</span><span style="flex:1;height:1px;background:#24282d;"></span>';

  const makeStageQuickPill = (blkTarget: 'wah' | 'pitch', text: string, activeKey: string) => {
    const isOn = bool(store, activeKey, false);
    const isSel = store.selectedBlock === blkTarget;
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'm-chip m-mono';
    b.dataset.controlId = 'ui.inspector_select';
    b.dataset.blockSelect = blkTarget;
    b.textContent = text;
    b.setAttribute('aria-pressed', String(isSel || isOn));
    b.style.cssText = `height:16px;padding:0 5px;font-size:8.5px;line-height:1;opacity:${isSel || isOn ? '1' : '0.72'};border-color:${isOn ? '#ff9f43' : '#2a2e35'};color:${isOn ? '#ffb86c' : '#9aa3ad'};`;
    b.addEventListener('click', () => store.selectBlock(blkTarget));
    return b;
  };
  boardTopLabels.insertBefore(
    makeStageQuickPill('wah', 'WAH', 'wah_1.enabled'),
    boardTopLabels.children[2] ?? null
  );
  boardTopLabels.insertBefore(
    makeStageQuickPill('pitch', 'PITCH', 'pitch_1.enabled'),
    boardTopLabels.children[3] ?? null
  );

  // Mini-cabeçote AMP 2 na seção LINHA B / PÓS (T-29 / CA-4)
  const amp2ModelId = str(store, 'amp_2.model', 'twin_ab763');
  const amp2MiniTitle = AMP2_MINI_TITLES[amp2ModelId] ?? 'US TWIN';
  const splitActive = bool(store, 'split.enabled', false) && bool(store, 'amp_2.enabled', true);
  const amp2Mini = document.createElement('div');
  amp2Mini.dataset.region = 'amp2-mini';
  amp2Mini.style.cssText =
    `position:absolute;right:16px;top:28px;width:352px;height:78px;border-radius:8px;background:linear-gradient(180deg,#1a1d22,#0b0d10);border:1px solid ${splitActive ? '#ff8a3d' : '#2e333c'};box-shadow:0 8px 16px rgba(0,0,0,.65),inset 0 1px 0 rgba(255,255,255,.12);padding:6px 12px;box-sizing:border-box;display:flex;flex-direction:column;justify-content:space-between;z-index:2;opacity:${splitActive ? '1' : '0.72'};`;

  const amp2Hdr = document.createElement('button');
  amp2Hdr.type = 'button';
  amp2Hdr.dataset.controlId = 'ui.inspector_select';
  amp2Hdr.dataset.blockSelect = 'amp2';
  amp2Hdr.style.cssText = 'display:flex;align-items:center;gap:8px;width:100%;border:0;background:transparent;padding:0;cursor:pointer;text-align:left;';
  amp2Hdr.innerHTML = `<span style="width:7px;height:7px;border-radius:50%;background:${splitActive ? '#ff8a3d' : '#6f7785'};box-shadow:${splitActive ? '0 0 6px #ff8a3d' : 'none'};flex:none;"></span><span class="m-osw" style="font-size:11px;font-weight:700;letter-spacing:.08em;color:#f1f3f5;">AMP 2 · ${amp2MiniTitle} (PARALELO)</span><span style="flex:1;"></span><span class="m-mono" style="font-size:9px;color:${splitActive ? '#4ee680' : '#9aa3ad'};">${splitActive ? 'LINHA B ATIVA' : 'LINHA B BYPASS'}</span>`;
  amp2Hdr.addEventListener('click', () => {
    if (store.selectedBlock !== 'amp2') store.selectBlock('amp2');
  });

  const amp2KnobsRow = document.createElement('div');
  amp2KnobsRow.style.cssText = 'display:flex;align-items:center;justify-content:space-between;gap:8px;';
  for (const k of [
    { id: 'amp_2.gain', lbl: 'GAIN', def: 5.0 },
    { id: 'amp_2.bass', lbl: 'BASS', def: 5.0 },
    { id: 'amp_2.middle', lbl: 'MID', def: 5.0 },
    { id: 'amp_2.treble', lbl: 'TREBLE', def: 5.0 },
    { id: 'amp_2.master', lbl: 'MASTER', def: 7.0 },
  ]) {
    const col = document.createElement('div');
    col.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:2px;';
    const knob = createKnob({
      controlId: k.id,
      label: `AMP 2 ${k.lbl}`,
      value: numSafe(store, k.id, k.def),
      min: 0,
      max: 10,
      step: 0.1,
      defaultValue: k.def,
      skinClass: 'p-black',
      sizePx: 24,
      onChange: (v) => store.setControl(k.id, v),
      onDragState: (dr) => store.setDraggingKnob(dr),
    });
    const kl = document.createElement('span');
    kl.textContent = k.lbl;
    kl.style.cssText = 'font-size:7px;font-weight:700;color:#c9ced4;line-height:1;';
    col.append(knob, kl);
    amp2KnobsRow.appendChild(col);
  }
  amp2Mini.append(amp2Hdr, amp2KnobsRow);

  const pedalsRow = document.createElement('div');
  pedalsRow.style.cssText = 'flex:1;min-height:0;display:flex;align-items:flex-end;justify-content:center;gap:0;padding:0;overflow:hidden;';

  const wrapScaledPedal = (
    el: HTMLElement,
    scale: number,
    origW: number,
    origH: number,
    marginBottom = 0,
    blkId?: BlockId,
    enabledKey?: string
  ) => {
    const slot = document.createElement('div');
    const isOn = enabledKey ? bool(store, enabledKey, true) : true;
    const isSelected = blkId ? store.selectedBlock === blkId : false;
    slot.style.cssText = `width:${Math.round(origW * scale)}px;height:${Math.round(origH * scale)}px;position:relative;flex:none;opacity:${isOn ? 1 : 0.62};outline:${isSelected ? '1px solid rgba(255,138,61,.65)' : 'none'};border-radius:10px;${marginBottom ? `margin-bottom:${marginBottom}px;` : ''}`;
    if (blkId) {
      slot.addEventListener('click', () => {
        if (store.selectedBlock !== blkId) store.selectBlock(blkId);
      });
    }
    const inner = document.createElement('div');
    inner.style.cssText = `transform:scale(${scale});transform-origin:top left;`;
    inner.appendChild(el);
    slot.appendChild(inner);
    return slot;
  };
  const makeCable = (w = 16, mb = 150) => {
    const c = document.createElement('div');
    c.className = 'm-cable';
    c.style.cssText = `width:${w}px;margin-bottom:${mb}px;`;
    return c;
  };

  const driveModel = str(store, 'drive.model', 'ts808') as PedalModelId;
  const [dw, dh] = PEDAL_SIZES[driveModel] ?? [130, 210];
  const modKey = str(store, 'mod_2.model', 'AnalogBbdChorus');
  const modPedal = MOD_MODEL_MAP[modKey]?.pedal ?? 'ce2';
  const [mw, mh] = PEDAL_SIZES[modPedal] ?? [104, 184];
  const showWah = bool(store, 'wah_1.enabled', false) || store.selectedBlock === 'wah';
  const showPitch = bool(store, 'pitch_1.enabled', false) || store.selectedBlock === 'pitch';
  const pitchKey = str(store, 'pitch_1.model', 'whammy');
  const pitchPedal = PITCH_MODEL_MAP[pitchKey]?.pedal ?? 'whammy';
  const [pw, ph] = PEDAL_SIZES[pitchPedal] ?? [148, 210];
  const rowScale = showWah && showPitch ? 0.78 : showWah || showPitch ? 0.86 : 1.0;

  const splitJoin = document.createElement('div');
  splitJoin.style.cssText = 'display:flex;align-items:flex-end;margin-bottom:150px;';
  const cLeft = document.createElement('div');
  cLeft.className = 'm-cable';
  cLeft.style.width = '12px';
  const divider = document.createElement('div');
  divider.style.cssText =
    'width:2px;height:190px;margin:0 8px -40px;background:repeating-linear-gradient(180deg,#3dd8ff 0 6px,transparent 6px 11px);opacity:.5;';
  const cRight = document.createElement('div');
  cRight.className = 'm-cable';
  cRight.style.width = '12px';
  splitJoin.append(cLeft, divider, cRight);

  if (showWah) {
    const [ww, wh] = PEDAL_SIZES.wah;
    pedalsRow.append(
      wrapScaledPedal(createPedalElement('wah', store), rowScale, ww, wh, 0, 'wah', 'wah_1.enabled'),
      makeCable(12, 130)
    );
  }
  pedalsRow.append(
    wrapScaledPedal(createPedalElement('ns2', store), rowScale, 104, 184, 0, 'gate', 'gate.enabled'),
    makeCable(showWah || showPitch ? 12 : 16, 150),
    wrapScaledPedal(createPedalElement(driveModel, store), rowScale, dw, dh, 0, 'drive', 'drive.enabled')
  );
  if (showPitch) {
    pedalsRow.append(
      makeCable(12, 130),
      wrapScaledPedal(createPedalElement(pitchPedal, store), rowScale, pw, ph, 0, 'pitch', 'pitch_1.enabled')
    );
  }
  pedalsRow.append(
    splitJoin,
    wrapScaledPedal(createPedalElement(modPedal, store), rowScale, mw, mh, 0, 'chorus', 'mod_2.enabled'),
    makeCable(showWah || showPitch ? 12 : 16, 110),
    wrapScaledPedal(createPedalElement('mxr10', store), rowScale, 232, 150, 0, 'eq', 'eq_1.enabled')
  );
  board.append(boardRail, boardTopLabels, amp2Mini, pedalsRow);

  const rack = document.createElement('div');
  rack.style.cssText =
    'width:316px;flex-shrink:0;padding:8px 0 0;display:flex;flex-direction:column;gap:6px;overflow:hidden;';
  const rackHdr = document.createElement('span');
  rackHdr.className = 'm-lbl';
  rackHdr.style.fontSize = '9px';
  rackHdr.textContent = 'Rack · loop / pós-amp';
  rack.appendChild(rackHdr);

  for (const [rmodel, origH, slotH, rBlk] of [
    ['comp1176', 130, 74, 'comp'],
    ['re201', 190, 107, 'delay'],
    ['plate', 110, 62, 'reverb'],
  ] as const) {
    const scale = 0.56;
    const slot = document.createElement('div');
    slot.dataset.rackSlot = rmodel;
    slot.style.cssText = `width:314px;height:${slotH}px;position:relative;flex:none;cursor:pointer;`;
    slot.title = `Clique para selecionar e editar o bloco ${rBlk.toUpperCase()} no Inspetor`;
    slot.addEventListener('click', () => {
      if (store.selectedBlock !== rBlk) store.selectBlock(rBlk);
    });
    const inner = document.createElement('div');
    inner.style.cssText = `position:absolute;left:0;top:0;width:560px;height:${origH}px;transform:scale(${scale});transform-origin:0 0;`;
    inner.appendChild(createRackElement(rmodel, store));
    slot.appendChild(inner);
    rack.appendChild(slot);
  }

  stageBottom.append(board, rack);
  stage.append(stageTop, stageBottom);

  const inspector = document.createElement('aside');
  inspector.dataset.region = 'inspector';
  inspector.className = 'm-metal';
  inspector.style.cssText =
    'border-left:1px solid #000;display:flex;flex-direction:column;min-height:0;box-shadow:inset 1px 0 0 rgba(255,255,255,.05);overflow:hidden;';

  const extraBlockDefs: Partial<Record<BlockId, GridBlockDef>> = {
    wah: {
      id: 'wah',
      row: 1,
      col: 2,
      tag: 'WAH',
      title: 'CRY WAH GCB95',
      dotColor: '#ffb347',
      chTag: 'A',
      enableKey: 'wah_1.enabled',
      bypassControlId: 'wah_1.enabled',
      subText: () => 'Wah 500 mH',
    },
    pitch: {
      id: 'pitch',
      row: 1,
      col: 3,
      tag: 'PITCH',
      title: 'PITCH SHIFTER',
      dotColor: '#ff5263',
      chTag: 'A',
      enableKey: 'pitch_1.enabled',
      bypassControlId: 'pitch_1.enabled',
      subText: (s) => PITCH_MODEL_MAP[str(s, 'pitch_1.model', 'whammy')]?.label ?? 'Whammy',
    },
  };
  const curDef =
    GRID_BLOCKS.find((g) => g.id === store.selectedBlock) ??
    extraBlockDefs[store.selectedBlock] ??
    GRID_BLOCKS[4];
  const ampMeta = AMP_LABELS[ampId] ?? AMP_LABELS.dual_rect;

  const inspTopSec = document.createElement('div');
  inspTopSec.style.cssText = 'padding:13px 14px 10px;border-bottom:1px solid #000;display:flex;flex-direction:column;gap:8px;flex:none;';

  const inspHdr = document.createElement('div');
  inspHdr.style.cssText = 'display:flex;align-items:center;gap:8px;';
  const inspTitle = document.createElement('div');
  inspTitle.style.cssText = 'display:flex;align-items:center;gap:8px;min-width:0;flex:1;';
  inspTitle.innerHTML = `<span style="width:9px;height:9px;border-radius:50%;background:${curDef.dotColor};box-shadow:0 0 8px ${curDef.dotColor};flex:none;"></span><span class="m-osw" style="font-size:16px;font-weight:600;letter-spacing:.04em;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${
    blkAmpTitle(store.selectedBlock, ampMeta.full, curDef.title)
  }</span>`;
  inspHdr.appendChild(inspTitle);
  if (curDef.enableKey && curDef.bypassControlId) {
    const en = bool(store, curDef.enableKey, true);
    const stBtn = makeChipButton(curDef.bypassControlId, en ? 'ATIVO' : 'BYPASS', en, () => {
      void store.setControl(curDef.bypassControlId!, !en);
    });
    stBtn.className = 'm-chip m-mono';
    stBtn.style.height = '20px';
    inspHdr.appendChild(stBtn);
  }

  const refSub = document.createElement('div');
  refSub.style.cssText = 'font-size:11px;color:#a3abb4;line-height:1.4;';
  refSub.textContent = store.selectedBlock === 'amp' ? ampMeta.ref : curDef.tag;

  const ampChips = document.createElement('div');
  ampChips.style.cssText = 'display:flex;flex-wrap:wrap;gap:4px;';
  if (store.selectedBlock === 'amp') {
    for (const [val, meta] of Object.entries(AMP_LABELS)) {
      const b = makeChipButton('amp.model', meta.short, ampId === val, () => void store.setControl('amp.model', val));
      b.style.height = '24px';
      ampChips.appendChild(b);
    }
  }

  const chRow = document.createElement('div');
  chRow.style.cssText = 'display:flex;align-items:center;gap:6px;';
  const chLbl = document.createElement('span');
  chLbl.className = 'm-lbl';
  chLbl.style.fontSize = '9px';
  chLbl.textContent = 'Canal';
  chRow.appendChild(chLbl);
  for (const ch of ['A', 'B', 'C', 'D'] as const) {
    const cb = makeChipButton('ui.inspector_channel', ch, store.activeInspectorCh === ch, () => {
      store.activeInspectorCh = ch;
      if (store.selectedBlock === 'amp' && store.ampChannelModels[ch]) {
        void store.setControl('amp.model', store.ampChannelModels[ch]);
      }
      store.emit('full');
    });
    cb.className = 'm-chip m-osw';
    cb.style.cssText = 'width:28px;height:24px;padding:0;justify-content:center;';
    chRow.appendChild(cb);
  }
  const chHint = document.createElement('span');
  chHint.style.cssText = 'font-size:10px;color:#9aa3ad;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;';
  chHint.textContent = 'cada canal guarda um modelo';
  chHint.title = 'Quatro ajustes independentes para este bloco (trocáveis por Cena A–H).';
  chRow.appendChild(chHint);

  inspTopSec.append(inspHdr, refSub);
  if (ampChips.childElementCount > 0) inspTopSec.appendChild(ampChips);
  inspTopSec.appendChild(chRow);

  const inspBotSec = document.createElement('div');
  inspBotSec.style.cssText = 'padding:10px 14px;display:flex;flex-direction:column;gap:8px;flex:1;min-height:0;overflow:hidden;';

  const paramsHdr = document.createElement('div');
  paramsHdr.style.cssText = 'display:flex;align-items:center;';
  paramsHdr.innerHTML =
    '<span class="m-lbl" style="font-size:9px;flex:1;">Parâmetros</span><span class="m-mono" style="font-size:9px;color:#29c46a;">TOCANDO</span>';

  const paramsBody = document.createElement('div');
  paramsBody.style.cssText = `display:flex;flex-direction:column;gap:${
    store.selectedBlock === 'amp2' ? 5 : 8
  }px;overflow:hidden;`;
  renderInspectorBody(store, paramsBody);

  const modBox = document.createElement('div');
  modBox.style.cssText =
    'margin-top:auto;border-radius:6px;padding:9px 10px;background:#0b0c0e;border:1px solid #1f242b;display:flex;flex-direction:column;gap:6px;';
  const modTop = document.createElement('div');
  modTop.style.cssText = 'display:flex;align-items:center;gap:6px;';
  const modTargetLabel =
    store.selectedBlock === 'amp'
      ? 'GAIN'
      : store.selectedBlock === 'drive'
        ? 'DRIVE'
        : store.selectedBlock === 'delay'
          ? 'ECHO VOL'
          : store.selectedBlock === 'reverb'
            ? 'MIX'
            : store.selectedBlock === 'chorus'
              ? 'DEPTH'
              : curDef.tag;
  const modTitle = document.createElement('span');
  modTitle.className = 'm-lbl';
  modTitle.style.cssText = 'font-size:9px;flex:1;';
  modTitle.textContent = `Modifier · ${modTargetLabel}`;
  const modBtn = makeChipButton('modifier.toggle', store.modifierOn ? 'ATIVO' : 'OFF', store.modifierOn, () => {
    store.modifierOn = !store.modifierOn;
    store.emit('full');
  });
  modBtn.title = 'Ativa ou desativa a modulação contínua sobre o parâmetro principal do bloco';
  modBtn.style.height = '20px';
  modTop.append(modTitle, modBtn);

  const modSources = document.createElement('div');
  modSources.style.cssText = 'display:flex;gap:4px;flex-wrap:wrap;';
  for (const sLblText of ['EXP 1', 'EXP 2', 'LFO 1', 'ENV', 'MIDI CC']) {
    const activeSrc = store.modifierSource === sLblText;
    const sb = makeChipButton('modifier.source', sLblText, activeSrc, () => {
      store.modifierSource = sLblText;
      store.modifierOn = true;
      store.emit('full');
    });
    sb.className = 'm-chip m-mono';
    sb.title = `Controlar ${modTargetLabel} via ${sLblText}`;
    sb.style.cssText = 'height:20px;padding:0 5px;font-size:9.5px;';
    modSources.appendChild(sb);
  }
  const modFoot = document.createElement('div');
  modFoot.className = 'm-mono';
  modFoot.style.cssText = 'font-size:9.5px;color:#9aa3ad;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;';
  modFoot.textContent = store.modifierOn
    ? `${store.modifierSource} → faixa 20–80 %`
    : 'EXP 1 → faixa 20–80 %';
  modFoot.title = store.modifierOn
    ? `${store.modifierSource} controla ${modTargetLabel} (faixa 20–80%) · clique direito num knob para configurar`
    : 'Controle em tempo real por pedal de expressão, LFO, envelope ou MIDI CC';
  modBox.append(modTop, modSources, modFoot);

  inspBotSec.append(paramsHdr, paramsBody, modBox);
  inspector.append(inspTopSec, inspBotSec);

  bodyArea.append(stage, inspector);
  root.append(toolbar, gridSec, bodyArea);
  return root;
}

function numSafe(store: AppStore, key: string, fallback: number): number {
  const v = store.getParam(key, fallback);
  return typeof v === 'number' && Number.isFinite(v) ? v : fallback;
}

function blkAmpTitle(blk: BlockId, ampTitle: string, defTitle: string): string {
  return blk === 'amp' ? ampTitle : defTitle;
}
