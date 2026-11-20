import { clampStep, createKnob, valueToRotationDeg } from './Knob.ts';
import type { AppStore } from '../store.ts';

export type PedalModelId =
  | 'ts808'
  | 'klon'
  | 'rat'
  | 'muff'
  | 'fuzzface'
  | 'ds1'
  | 'ns2'
  | 'ce2'
  | 'trichorus'
  | 'flanger'
  | 'phaser90'
  | 'tremolo'
  | 'univibe'
  | 'rotary'
  | 'whammy'
  | 'drop'
  | 'harmonizer'
  | 'detune'
  | 'wah'
  | 'mxr10';

export const PEDAL_SIZES: Record<PedalModelId, [number, number]> = {
  ts808: [130, 210],
  klon: [128, 200],
  rat: [116, 180],
  muff: [160, 200],
  fuzzface: [180, 180],
  ds1: [104, 184],
  ns2: [104, 184],
  ce2: [104, 184],
  trichorus: [140, 196],
  flanger: [116, 190],
  phaser90: [104, 184],
  tremolo: [104, 184],
  univibe: [148, 196],
  rotary: [148, 196],
  whammy: [148, 210],
  drop: [104, 184],
  harmonizer: [128, 196],
  detune: [116, 184],
  wah: [118, 214],
  mxr10: [232, 150],
};

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

const MOD_MODELS = new Set<PedalModelId>([
  'ce2',
  'trichorus',
  'flanger',
  'phaser90',
  'tremolo',
  'univibe',
  'rotary',
]);

const PITCH_MODELS = new Set<PedalModelId>([
  'whammy',
  'drop',
  'harmonizer',
  'detune',
]);

export function createPedalElement(model: PedalModelId, store: AppStore): HTMLElement {
  const dims: Record<PedalModelId, { w: number; h: number; bg: string; rad: string; name: string; sub?: string }> = {
    ts808: { w: 130, h: 210, bg: 'linear-gradient(180deg,#5aa86b 0%,#3d8a4e 45%,#2c6a3a 100%)', rad: '10px', name: 'SCREAMER 808', sub: 'OVERDRIVE' },
    klon: { w: 128, h: 200, bg: 'repeating-linear-gradient(90deg,rgba(255,255,255,.08) 0 1px,transparent 1px 3px),linear-gradient(180deg,#efd590 0%,#caa652 50%,#a8862f 100%)', rad: '10px', name: 'GOLDEN KLONE', sub: 'PROFESSIONAL OVERDRIVE' },
    rat: { w: 116, h: 180, bg: 'radial-gradient(circle at 1px 1px,rgba(255,255,255,.04) 1px,transparent 1.5px) 0 0/3px 3px,linear-gradient(180deg,#232323,#0d0d0d)', rad: '8px', name: 'RODENT', sub: 'DISTORTION' },
    muff: { w: 160, h: 200, bg: 'repeating-linear-gradient(90deg,rgba(255,255,255,.12) 0 1px,transparent 1px 3px),linear-gradient(180deg,#eef0f2 0%,#b8bcc0 50%,#d5d8db 100%)', rad: '8px', name: 'BIG FUZZ π', sub: 'SUSTAINER' },
    fuzzface: { w: 180, h: 180, bg: 'radial-gradient(circle at 38% 30%,#e8564a 0%,#b5261d 55%,#741109 100%)', rad: '50%', name: 'FACE FUZZ' },
    ds1: { w: 104, h: 184, bg: 'linear-gradient(180deg,#f47a2a 0%,#d65a10 100%)', rad: '8px', name: 'DS-ONE', sub: 'DISTORTION' },
    ns2: { w: 104, h: 184, bg: 'linear-gradient(180deg,#f1f0ea 0%,#cfcec7 100%)', rad: '8px', name: 'GATE-2', sub: 'Noise Gate' },
    ce2: { w: 104, h: 184, bg: 'linear-gradient(180deg,#86c3e6 0%,#5ba3cf 100%)', rad: '8px', name: 'CHORUS-2', sub: 'Chorus' },
    trichorus: { w: 140, h: 196, bg: 'repeating-linear-gradient(90deg,rgba(255,255,255,.06) 0 1px,transparent 1px 3px),linear-gradient(180deg,#2d5ca8 0%,#1d3d78 52%,#112650 100%)', rad: '9px', name: 'TRI-CHORUS', sub: 'STEREO 3-VOICE BBD' },
    flanger: { w: 116, h: 190, bg: 'linear-gradient(180deg,#594078 0%,#3a2654 55%,#231538 100%)', rad: '8px', name: 'TZ FLANGER', sub: 'THROUGH-ZERO' },
    phaser90: { w: 104, h: 184, bg: 'linear-gradient(180deg,#f57e20 0%,#d96008 60%,#b04802 100%)', rad: '8px', name: 'PHASER 90', sub: '4-STAGE FET' },
    tremolo: { w: 104, h: 184, bg: 'linear-gradient(180deg,#4f7c58 0%,#33573b 100%)', rad: '8px', name: 'TREMOLO', sub: 'Optical/Bias' },
    univibe: { w: 148, h: 196, bg: 'repeating-linear-gradient(90deg,rgba(255,255,255,.03) 0 1px,transparent 1px 3px),linear-gradient(180deg,#25282e 0%,#121418 65%,#0a0b0d 100%)', rad: '9px', name: 'UNI-VIBE', sub: 'PHOTOCELL MODULATOR' },
    rotary: { w: 148, h: 196, bg: 'linear-gradient(180deg,#6e4326 0%,#4e2d18 55%,#341c0d 100%)', rad: '9px', name: 'ROTARY 147', sub: 'HORN & DRUM DOPPLER' },
    whammy: { w: 148, h: 210, bg: 'linear-gradient(180deg,#d62424 0%,#a61414 55%,#750a0a 100%)', rad: '9px', name: 'WHAMMY', sub: 'PITCH SHIFT' },
    drop: { w: 104, h: 184, bg: 'linear-gradient(180deg,#8f1922 0%,#5e0e14 60%,#3b060a 100%)', rad: '8px', name: 'DROP CAPO', sub: 'POLY DOWNTUNE' },
    harmonizer: { w: 128, h: 196, bg: 'linear-gradient(180deg,#1e4f8c 0%,#123461 60%,#0a203e 100%)', rad: '9px', name: 'HARMONIZER', sub: 'DIATONIC DUAL PITCH' },
    detune: { w: 116, h: 184, bg: 'repeating-linear-gradient(90deg,rgba(255,255,255,.12) 0 1px,transparent 1px 3px),linear-gradient(180deg,#d8dde3 0%,#a7aeb7 55%,#7e8690 100%)', rad: '8px', name: 'MICRO DETUNE', sub: 'DUAL-VOICE PITCH' },
    wah: { w: 118, h: 214, bg: 'linear-gradient(180deg,#25272b 0%,#121316 55%,#070809 100%)', rad: '10px', name: 'CRY WAH', sub: '500mH INDUCTOR' },
    mxr10: { w: 232, h: 150, bg: 'radial-gradient(circle at 1px 1px,rgba(255,255,255,.04) 1px,transparent 1.5px) 0 0/3px 3px,linear-gradient(180deg,#262626,#0c0c0c)', rad: '8px', name: '10 BAND EQ', sub: 'GRAPHIC EQUALIZER' },
  };

  const d = dims[model];
  const root = document.createElement('div');
  root.dataset.pedalModel = model;
  root.style.cssText = `position:relative;width:${d.w}px;height:${d.h}px;user-select:none;flex-shrink:0`;

  const body = document.createElement('div');
  body.style.cssText = `position:absolute;inset:0;border-radius:${d.rad};background:${d.bg};box-shadow:0 14px 18px rgba(0,0,0,.65),0 2px 4px rgba(0,0,0,.6),inset 0 2px 0 rgba(255,255,255,.35),inset 0 -3px 0 rgba(0,0,0,.35)`;
  root.appendChild(body);

  const isDrive =
    model === 'ts808' || model === 'klon' || model === 'rat' || model === 'muff' || model === 'fuzzface' || model === 'ds1';
  const bypassId =
    model === 'ns2'
      ? 'gate.enabled'
      : MOD_MODELS.has(model)
        ? 'mod_2.enabled'
        : PITCH_MODELS.has(model)
          ? 'pitch_1.enabled'
          : model === 'wah'
            ? 'wah_1.enabled'
            : model === 'mxr10'
              ? 'eq_1.enabled'
              : 'drive.enabled';
  const defaultOn = model === 'ns2' || isDrive || model === 'mxr10';
  const isOn = Boolean(store.getParam(bypassId, defaultOn));

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
      { controlId: 'drive.drive', label: 'DISTORTION', min: 0, max: 10, step: 0.1, def: 4.5, x: 22, y: 16, px: 28, cls: 'p-black', ink: '#ededed', fs: 5.5 },
      { controlId: 'drive.tone', label: 'FILTER', min: 0, max: 10, step: 0.1, def: 5.5, x: 58, y: 16, px: 28, cls: 'p-black', ink: '#ededed', fs: 5.5 },
      { controlId: 'drive.level', label: 'VOLUME', min: 0, max: 10, step: 0.1, def: 6.0, x: 94, y: 16, px: 28, cls: 'p-black', ink: '#ededed', fs: 5.5 },
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
      { controlId: 'gate.threshold_db', label: 'THRESH', min: -90, max: 0, step: 1, def: -65, x: 22, y: 12, px: 26, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6 },
      { controlId: 'gate.release_ms', label: 'DECAY', min: 5, max: 1000, step: 5, def: 80, x: 52, y: 12, px: 26, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6 },
      { controlId: 'gate.hysteresis_db', label: 'HYST', min: 0, max: 12, step: 0.5, def: 4, x: 82, y: 12, px: 26, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6 },
    ],
    ce2: [
      { controlId: 'mod_2.rate_hz', label: 'RATE', min: 0.05, max: 10, step: 0.05, def: 1.0, x: 34, y: 12, px: 28, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6.5 },
      { controlId: 'mod_2.depth', label: 'DEPTH', min: 0, max: 10, step: 0.1, def: 5.0, x: 70, y: 12, px: 28, cls: 'p-boss', ink: '#111', dir: 'column-reverse', fs: 6.5 },
    ],
    trichorus: [
      { controlId: 'mod_2.rate_hz', label: 'RATE', min: 0.05, max: 10, step: 0.05, def: 1.0, x: 28, y: 16, px: 28, cls: 'p-black', ink: '#e4f0ff' },
      { controlId: 'mod_2.depth', label: 'DEPTH', min: 0, max: 10, step: 0.1, def: 5.0, x: 70, y: 16, px: 28, cls: 'p-black', ink: '#e4f0ff' },
      { controlId: 'mod_2.mix', label: 'MIX', min: 0, max: 100, step: 1, def: 50, x: 112, y: 16, px: 28, cls: 'p-black', ink: '#e4f0ff' },
    ],
    flanger: [
      { controlId: 'mod_2.rate_hz', label: 'RATE', min: 0.05, max: 10, step: 0.05, def: 1.0, x: 24, y: 14, px: 26, cls: 'p-black', ink: '#f3ebff', fs: 6.5 },
      { controlId: 'mod_2.depth', label: 'DEPTH', min: 0, max: 10, step: 0.1, def: 5.0, x: 58, y: 14, px: 26, cls: 'p-black', ink: '#f3ebff', fs: 6.5 },
      { controlId: 'mod_2.mix', label: 'MIX', min: 0, max: 100, step: 1, def: 50, x: 92, y: 14, px: 26, cls: 'p-black', ink: '#f3ebff', fs: 6.5 },
    ],
    phaser90: [
      { controlId: 'mod_2.rate_hz', label: 'SPEED', min: 0.05, max: 10, step: 0.05, def: 1.0, x: 52, y: 16, px: 36, cls: 'p-black', ink: '#180d02' },
    ],
    tremolo: [
      { controlId: 'mod_2.rate_hz', label: 'RATE', min: 0.05, max: 10, step: 0.05, def: 1.0, x: 22, y: 12, px: 26, cls: 'p-boss', ink: '#eef5ef', dir: 'column-reverse', fs: 6 },
      { controlId: 'mod_2.depth', label: 'DEPTH', min: 0, max: 10, step: 0.1, def: 5.0, x: 52, y: 12, px: 26, cls: 'p-boss', ink: '#eef5ef', dir: 'column-reverse', fs: 6 },
      { controlId: 'mod_2.mix', label: 'MIX', min: 0, max: 100, step: 1, def: 50, x: 82, y: 12, px: 26, cls: 'p-boss', ink: '#eef5ef', dir: 'column-reverse', fs: 6 },
    ],
    univibe: [
      { controlId: 'mod_2.rate_hz', label: 'SPEED', min: 0.05, max: 10, step: 0.05, def: 1.0, x: 32, y: 14, px: 28, cls: 'p-chicken', ink: '#e8eaed', fs: 6 },
      { controlId: 'mod_2.depth', label: 'INTENSITY', min: 0, max: 10, step: 0.1, def: 5.0, x: 74, y: 14, px: 28, cls: 'p-chicken', ink: '#e8eaed', fs: 6 },
      { controlId: 'mod_2.mix', label: 'MIX', min: 0, max: 100, step: 1, def: 50, x: 116, y: 14, px: 28, cls: 'p-chicken', ink: '#e8eaed', fs: 6 },
    ],
    rotary: [
      { controlId: 'mod_2.rate_hz', label: 'SPEED', min: 0.05, max: 10, step: 0.05, def: 1.0, x: 32, y: 14, px: 28, cls: 'p-chicken', ink: '#f7e2b8', fs: 6.5 },
      { controlId: 'mod_2.depth', label: 'DEPTH', min: 0, max: 10, step: 0.1, def: 5.0, x: 74, y: 14, px: 28, cls: 'p-chicken', ink: '#f7e2b8', fs: 6.5 },
      { controlId: 'mod_2.mix', label: 'MIX', min: 0, max: 100, step: 1, def: 50, x: 116, y: 14, px: 28, cls: 'p-chicken', ink: '#f7e2b8', fs: 6.5 },
    ],
    whammy: [
      { controlId: 'pitch_1.semitones', label: 'SHIFT', min: -24, max: 24, step: 1, def: 0, x: 40, y: 14, px: 30, cls: 'p-black', ink: '#fff2f2' },
      { controlId: 'pitch_1.mix', label: 'MIX', min: 0, max: 100, step: 1, def: 100, x: 40, y: 60, px: 26, cls: 'p-black', ink: '#fff2f2' },
    ],
    drop: [
      { controlId: 'pitch_1.semitones', label: 'TUNE', min: -24, max: 24, step: 1, def: -2, x: 32, y: 14, px: 28, cls: 'p-black', ink: '#fff0f0', fs: 6.5 },
      { controlId: 'pitch_1.mix', label: 'MIX', min: 0, max: 100, step: 1, def: 100, x: 74, y: 14, px: 26, cls: 'p-black', ink: '#fff0f0', fs: 6.5 },
    ],
    harmonizer: [
      { controlId: 'pitch_1.semitones', label: 'INTERVAL', min: -24, max: 24, step: 1, def: 4, x: 38, y: 14, px: 30, cls: 'p-black', ink: '#e6f2ff', fs: 6 },
      { controlId: 'pitch_1.mix', label: 'BALANCE', min: 0, max: 100, step: 1, def: 50, x: 90, y: 14, px: 30, cls: 'p-black', ink: '#e6f2ff', fs: 6 },
    ],
    detune: [
      { controlId: 'pitch_1.semitones', label: 'DETUNE', min: -24, max: 24, step: 1, def: 0, x: 34, y: 14, px: 28, cls: 'p-black', ink: '#12151a', fs: 6.5 },
      { controlId: 'pitch_1.mix', label: 'MIX', min: 0, max: 100, step: 1, def: 50, x: 82, y: 14, px: 28, cls: 'p-black', ink: '#12151a', fs: 6.5 },
    ],
    wah: [
      { controlId: 'wah_1.position', label: 'SWEEP', min: 0, max: 1, step: 0.01, def: 0.5, x: 26, y: 125, px: 22, cls: 'p-black', ink: '#e8eaed', fs: 5.5 },
      { controlId: 'wah_1.q', label: 'RES Q', min: 1, max: 10, step: 0.1, def: 5.0, x: 92, y: 125, px: 22, cls: 'p-trim', ink: '#e8eaed', fs: 5.5 },
    ],
    mxr10: [],
  };

  // Decor / nameplates
  const addRect = (css: string) => {
    const dec = document.createElement('div');
    dec.style.cssText = `position:absolute;${css}`;
    root.appendChild(dec);
  };
  const addTxt = (x: number, y: number, w: number, text: string, css: string) => {
    const t = document.createElement('div');
    t.className = 'osw';
    t.textContent = text;
    t.style.cssText = `position:absolute;left:${x}px;top:${y}px;width:${w}px;text-align:center;${css}`;
    root.appendChild(t);
  };

  if (model === 'trichorus') {
    addRect('left:10px;right:10px;top:76px;height:40px;border-radius:4px;background:rgba(6,12,28,.78);border:1px solid #4c82d6');
    addTxt(10, 81, 120, d.name, 'font-size:17px;font-weight:700;letter-spacing:.06em;color:#e4f0ff;white-space:nowrap');
    addTxt(10, 102, 120, d.sub ?? '', 'font-size:6px;font-weight:800;letter-spacing:.12em;color:#7cb7ff;white-space:nowrap');
  } else if (model === 'flanger') {
    addRect('left:8px;right:8px;top:76px;height:2px;background:linear-gradient(90deg,transparent,#c79eff,transparent)');
    addTxt(0, 82, 116, d.name, 'font-size:17px;font-weight:700;letter-spacing:.05em;color:#f3ebff;white-space:nowrap');
    addTxt(0, 104, 116, d.sub ?? '', 'font-size:6px;font-weight:800;letter-spacing:.18em;color:#d0b2ff;white-space:nowrap');
  } else if (model === 'phaser90') {
    addRect('left:10px;right:10px;top:78px;height:28px;border:2px solid #180d02;border-radius:4px');
    addTxt(0, 82, 104, d.name, 'font-size:15px;font-weight:700;font-style:italic;letter-spacing:.04em;color:#180d02;white-space:nowrap');
    addTxt(0, 110, 104, d.sub ?? '', 'font-size:6px;font-weight:800;letter-spacing:.14em;color:#2b1504;white-space:nowrap');
  } else if (model === 'univibe') {
    addRect('left:12px;right:12px;top:76px;height:38px;border-radius:4px;background:linear-gradient(180deg,#d8dde2,#9ea4ab);box-shadow:inset 0 1px 0 #fff');
    addTxt(12, 80, 124, d.name, 'font-size:18px;font-weight:700;letter-spacing:.08em;color:#141619;white-space:nowrap');
    addTxt(12, 101, 124, d.sub ?? '', 'font-size:5.5px;font-weight:800;letter-spacing:.1em;color:#2c3138;white-space:nowrap');
  } else if (model === 'rotary') {
    addRect('left:12px;right:12px;top:76px;height:40px;border-radius:4px;background:repeating-linear-gradient(0deg,#1a120b 0 3px,#2f2014 3px 6px);border:1px solid #a98b5a');
    addTxt(12, 81, 124, d.name, 'font-size:17px;font-weight:700;letter-spacing:.06em;color:#f7e2b8;white-space:nowrap');
    addTxt(12, 101, 124, d.sub ?? '', 'font-size:5.5px;font-weight:800;letter-spacing:.12em;color:#dfc27c;white-space:nowrap');
  } else if (model === 'whammy') {
    addRect('left:76px;top:16px;width:58px;height:142px;border-radius:6px;background:linear-gradient(180deg,#202226,#0c0d0f);border:2px solid #8a8f96;box-shadow:inset 0 2px 6px #000');
    addRect('left:82px;top:24px;width:46px;height:126px;border-radius:4px;background:repeating-linear-gradient(0deg,#111 0 4px,#2b2e33 4px 7px)');
    addTxt(8, 102, 64, d.name, 'font-size:15px;font-weight:700;letter-spacing:.04em;color:#fff2f2;white-space:nowrap');
    addTxt(8, 121, 64, d.sub ?? '', 'font-size:5.5px;font-weight:800;letter-spacing:.12em;color:#ffd0d0;white-space:nowrap');
  } else if (model === 'drop') {
    addTxt(0, 76, 104, d.name, 'font-size:15px;font-weight:700;letter-spacing:.05em;color:#fff0f0;white-space:nowrap');
    addTxt(0, 98, 104, d.sub ?? '', 'font-size:5.5px;font-weight:800;letter-spacing:.12em;color:#ffb8bd;white-space:nowrap');
  } else if (model === 'harmonizer') {
    addTxt(0, 82, 128, d.name, 'font-size:16px;font-weight:700;letter-spacing:.06em;color:#e6f2ff;white-space:nowrap');
    addTxt(0, 105, 128, d.sub ?? '', 'font-size:5.8px;font-weight:800;letter-spacing:.12em;color:#8cc4ff;white-space:nowrap');
  } else if (model === 'detune') {
    addTxt(0, 76, 116, d.name, 'font-size:14px;font-weight:700;letter-spacing:.04em;color:#12151a;white-space:nowrap');
    addTxt(0, 96, 116, d.sub ?? '', 'font-size:5.5px;font-weight:800;letter-spacing:.12em;color:#2b313a;white-space:nowrap');
  } else if (model === 'wah') {
    addRect('left:14px;right:14px;top:12px;height:108px;border-radius:8px;background:repeating-linear-gradient(0deg,#0c0d0e 0 4px,#26282c 4px 7px);border:2px solid #595e66;box-shadow:inset 0 3px 8px rgba(0,0,0,.9)');
    addRect('left:22px;right:22px;top:48px;height:36px;border-radius:4px;background:rgba(8,9,10,.94);border:1px solid #c9ced4');
    addTxt(16, 52, 86, d.name, 'font-size:15px;font-weight:700;letter-spacing:.06em;color:#f5f7fa;white-space:nowrap');
    addTxt(16, 70, 86, d.sub ?? '', 'font-size:5.2px;font-weight:800;letter-spacing:.1em;color:#ffb347;white-space:nowrap');
  } else if (model !== 'ts808' && model !== 'ds1' && model !== 'ns2' && model !== 'ce2' && model !== 'tremolo') {
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
    if (model === 'mxr10' && d.sub) {
      const subEl = document.createElement('div');
      subEl.textContent = d.sub;
      subEl.style.cssText = 'position:absolute;left:10px;top:132px;font-size:6.5px;font-weight:700;letter-spacing:.12em;color:#8f969e';
      root.appendChild(subEl);
    }
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
      onDragState: (dr) => store.setDraggingKnob(dr),
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
      { id: 'eq_1.band1', lbl: '31', min: -12, max: 12 },
      { id: 'eq_1.band2', lbl: '62', min: -12, max: 12 },
      { id: 'eq_1.band3', lbl: '125', min: -12, max: 12 },
      { id: 'eq_1.band4', lbl: '250', min: -12, max: 12 },
      { id: 'eq_1.band5', lbl: '500', min: -12, max: 12 },
      { id: 'eq_1.band6', lbl: '1K', min: -12, max: 12 },
      { id: 'eq_1.band7', lbl: '2K', min: -12, max: 12 },
      { id: 'eq_1.band8', lbl: '4K', min: -12, max: 12 },
      { id: 'eq_1.band9', lbl: '8K', min: -12, max: 12 },
      { id: 'eq_1.band10', lbl: '16K', min: -12, max: 12 },
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
      cap.addEventListener('pointerdown', (e) => {
        active = true;
        startY = e.clientY;
        startVal = Number(store.getParam(bKey, val));
        store.setDraggingKnob(true);
        try {
          cap.setPointerCapture(e.pointerId);
        } catch {
          // ignore
        }
        e.preventDefault();
      });
      cap.addEventListener('pointermove', (e) => {
        if (!active) return;
        const delta = ((startY - e.clientY) / 67) * (b.max - b.min);
        const nv = clampStep(startVal + delta, b.min, b.max, 0.5);
        cap.style.top = `${Math.round(((b.max - nv) / (b.max - b.min)) * 67)}px`;
        cap.setAttribute('aria-valuenow', String(nv));
        store.setControl(b.id, nv);
      });
      const finish = () => {
        if (!active) return;
        active = false;
        store.setDraggingKnob(false);
      };
      cap.addEventListener('pointerup', finish);
      cap.addEventListener('pointercancel', finish);
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
    trichorus: [70, 130],
    flanger: [58, 66],
    phaser90: [52, 69],
    tremolo: [19, 68],
    univibe: [74, 129],
    rotary: [74, 130],
    whammy: [40, 142],
    drop: [52, 66],
    harmonizer: [64, 68],
    detune: [58, 66],
    wah: [59, 136],
    mxr10: [160, 124],
  };
  const [lx, ly] = ledPos[model];
  const ledCol = model === 'mxr10' ? '#3db7ff' : '#ff2a1a';
  const led = document.createElement('div');
  led.style.cssText = `position:absolute;left:${lx}px;top:${ly}px;width:9px;height:9px;margin:-4.5px 0 0 -4.5px;border-radius:50%;background:${isOn ? ledCol : '#3a1210'};box-shadow:${isOn ? `0 0 9px ${ledCol}` : 'inset 0 1px 2px rgba(0,0,0,.7)'}`;
  root.appendChild(led);

  const isBossTreadle = model === 'ns2' || model === 'ce2' || model === 'ds1' || model === 'tremolo';
  if (isBossTreadle) {
    const chk = document.createElement('span');
    chk.textContent = 'CHECK';
    chk.style.cssText = `position:absolute;left:${lx - 11}px;top:${ly - 13}px;font-size:5.5px;font-weight:800;letter-spacing:.06em;color:${model === 'tremolo' ? '#eef5ef' : '#111'}`;
    root.appendChild(chk);
  }

  const isTreadle = model === 'ts808' || isBossTreadle;
  const swBtn = document.createElement('button');
  swBtn.setAttribute('data-control-id', bypassId);
  swBtn.setAttribute('aria-pressed', String(isOn));
  swBtn.setAttribute('aria-label', `Bypass ${d.name}`);
  swBtn.addEventListener('click', () => {
    store.setControl(bypassId, !isOn);
    if (isDrive) store.selectBlock('drive');
    else if (MOD_MODELS.has(model)) store.selectBlock('chorus');
    else if (PITCH_MODELS.has(model)) store.selectBlock('pitch');
    else if (model === 'wah') store.selectBlock('wah');
  });

  if (isTreadle) {
    swBtn.className = 'treadle';
    swBtn.style.cssText =
      model === 'ts808'
        ? 'left:5px;top:100px;width:120px;height:104px;border-radius:6px;background:linear-gradient(180deg,#56a467,#357a45);box-shadow:0 -2px 0 rgba(0,0,0,.35),inset 0 1px 0 rgba(255,255,255,.4),0 3px 4px rgba(0,0,0,.4);overflow:hidden'
        : `left:4px;top:78px;width:96px;height:102px;border-radius:5px;background:rgba(0,0,0,.06);box-shadow:0 -2px 0 rgba(0,0,0,.3),inset 0 1px 0 rgba(255,255,255,.35),0 3px 4px rgba(0,0,0,.4);padding-top:6px`;
    const badge = document.createElement('span');
    badge.className = 'osw';
    const subSpan = document.createElement('span');
    subSpan.textContent = d.sub ?? '';
    if (model === 'ts808') {
      badge.textContent = d.name;
      badge.style.cssText =
        'width:100%;background:linear-gradient(180deg,#f4f4f4,#b9b9b9);font-size:13px;font-weight:700;letter-spacing:.06em;color:#151515;padding:2px 0;text-align:center';
      subSpan.style.cssText =
        'font-size:7px;font-weight:800;letter-spacing:.2em;color:#f4f4f4;margin:4px 0';
    } else {
      const inkBoss = model === 'tremolo' ? '#eef5ef' : '#111';
      badge.textContent = d.name;
      badge.style.cssText =
        `font-size:15px;font-weight:700;letter-spacing:.04em;color:${inkBoss};border:1.5px solid ${inkBoss};padding:0 5px;line-height:1.2`;
      subSpan.style.cssText =
        `font-size:7px;font-weight:700;color:${inkBoss};margin:3px 0 6px`;
    }
    const pad = document.createElement('span');
    pad.className = 'pad';
    pad.style.cssText =
      model === 'ts808' ? 'width:100px;height:52px;margin-top:auto;margin-bottom:8px' : 'width:80px;height:48px;margin-top:auto;margin-bottom:6px';
    swBtn.append(badge, subSpan, pad);
  } else {
    const swPos: Record<string, [number, number]> = {
      klon: [64, 164],
      rat: [58, 150],
      muff: [80, 172],
      fuzzface: [90, 104],
      trichorus: [70, 168],
      flanger: [58, 158],
      phaser90: [52, 156],
      univibe: [74, 168],
      rotary: [74, 168],
      whammy: [40, 178],
      drop: [52, 152],
      harmonizer: [64, 162],
      detune: [58, 152],
      wah: [59, 184],
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
