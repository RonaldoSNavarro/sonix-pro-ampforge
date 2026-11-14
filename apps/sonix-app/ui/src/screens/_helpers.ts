import type { AppStore } from '../store';
import { createHSliderRow } from '../components/Knob';

export function num(store: AppStore, key: string, fallback: number): number {
  const v = store.getParam(key, fallback);
  return typeof v === 'number' && Number.isFinite(v) ? v : fallback;
}

export function bool(store: AppStore, key: string, fallback: boolean): boolean {
  const v = store.getParam(key, fallback);
  return typeof v === 'boolean' ? v : Boolean(v);
}

export function str(store: AppStore, key: string, fallback: string): string {
  const v = store.getParam(key, fallback);
  return typeof v === 'string' && v.length > 0 ? v : fallback;
}

export function makePanelCard(title: string, subtitle?: string): { root: HTMLElement; body: HTMLElement } {
  const root = document.createElement('section');
  root.className = 'm-metal';
  root.style.cssText =
    'border-radius:8px;padding:8px 10px;display:flex;flex-direction:column;gap:7px;min-height:0;min-width:0;';

  const hdr = document.createElement('div');
  hdr.style.cssText =
    'display:flex;align-items:center;justify-content:space-between;gap:8px;padding-bottom:5px;border-bottom:1px solid #22262c;';
  const t = document.createElement('span');
  t.className = 'm-lbl';
  t.textContent = title;
  t.style.color = '#d5dbe2';
  hdr.appendChild(t);

  if (subtitle) {
    const s = document.createElement('span');
    s.className = 'm-mono';
    s.textContent = subtitle;
    s.style.cssText = 'font-size:9.5px;color:#7f8791;';
    hdr.appendChild(s);
  }

  const body = document.createElement('div');
  body.style.cssText = 'display:flex;flex-direction:column;gap:6px;min-height:0;min-width:0;';
  root.append(hdr, body);
  return { root, body };
}

export function makeControlSlider(
  store: AppStore,
  controlId: string,
  paramKey: string,
  label: string,
  fallback: number,
  format?: (val: number) => string,
  fillGradient?: string
): HTMLElement {
  const desc = store.getDescriptor(controlId);
  const [min, max] = desc?.range ?? [0, 10];
  const step = desc?.step ?? 0.1;
  const def = typeof desc?.defaultValue === 'number' ? desc.defaultValue : fallback;
  return createHSliderRow({
    controlId,
    label,
    value: num(store, paramKey, def),
    min,
    max,
    step,
    defaultValue: def,
    unit: desc?.unit,
    format,
    fillGradient,
    onChange: (val) => {
      void store.setControl(controlId, val);
    },
  });
}

export function makeChipButton(
  controlId: string,
  label: string,
  pressed: boolean,
  onClick: () => void,
  extraData?: Record<string, string>
): HTMLButtonElement {
  const btn = document.createElement('button');
  btn.type = 'button';
  btn.className = 'm-chip';
  btn.dataset.controlId = controlId;
  btn.setAttribute('aria-pressed', pressed ? 'true' : 'false');
  btn.textContent = label;
  if (extraData) {
    for (const [k, v] of Object.entries(extraData)) {
      btn.dataset[k] = v;
    }
  }
  btn.addEventListener('click', onClick);
  return btn;
}

export function makeControlSelect(
  store: AppStore,
  controlId: string,
  paramKey: string,
  choices: { value: string; label: string }[],
  fallback: string
): HTMLSelectElement {
  const sel = document.createElement('select');
  sel.dataset.controlId = controlId;
  sel.style.cssText =
    'height:24px;padding:0 7px;border-radius:4px;background:#0b0d10;border:1px solid #2b313a;color:#e8eaed;font-size:11px;font-weight:600;min-width:0;';
  const cur = str(store, paramKey, fallback);
  for (const ch of choices) {
    const opt = document.createElement('option');
    opt.value = ch.value;
    opt.textContent = ch.label;
    if (ch.value === cur) opt.selected = true;
    sel.appendChild(opt);
  }
  sel.addEventListener('change', () => {
    void store.setControl(controlId, sel.value);
  });
  return sel;
}

export function makeToggleRow(
  store: AppStore,
  controlId: string,
  paramKey: string,
  label: string,
  fallback: boolean,
  onText = 'ATIVO',
  offText = 'BYPASS'
): HTMLElement {
  const row = document.createElement('div');
  row.style.cssText = 'display:flex;align-items:center;justify-content:space-between;gap:8px;';
  const l = document.createElement('span');
  l.className = 'm-lbl';
  l.textContent = label;
  const active = bool(store, paramKey, fallback);
  const btn = makeChipButton(controlId, active ? onText : offText, active, () => {
    void store.setControl(controlId, !active);
  });
  row.append(l, btn);
  return row;
}

export function makeLedMeter(segments: number, levelLinear: number, vertical = false): HTMLElement {
  const meter = document.createElement('div');
  meter.style.cssText = vertical
    ? 'display:flex;flex-direction:column-reverse;gap:2px;height:100%;'
    : 'display:flex;align-items:center;gap:2px;';
  const litCount = Math.round(Math.max(0, Math.min(1, levelLinear)) * segments);
  for (let i = 0; i < segments; i += 1) {
    const seg = document.createElement('span');
    seg.className = 'm-seg';
    seg.style.cssText = vertical ? 'width:7px;flex:1;' : 'width:4px;height:12px;';
    const active = i < litCount;
    const ratio = i / segments;
    if (active) {
      seg.style.background = ratio > 0.88 ? '#ff4238' : ratio > 0.7 ? '#ffc247' : '#3be67c';
      seg.style.boxShadow = `0 0 6px ${ratio > 0.88 ? '#ff4238' : '#3be67c'}`;
    } else {
      seg.style.background = '#13161b';
    }
    meter.appendChild(seg);
  }
  return meter;
}
