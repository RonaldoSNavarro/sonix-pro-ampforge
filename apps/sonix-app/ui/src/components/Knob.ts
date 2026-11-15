export interface KnobOptions {
  controlId: string;
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  defaultValue: number;
  skinClass: string;
  sizePx: number;
  unit?: string;
  onChange: (val: number) => void;
  onDragState?: (dragging: boolean) => void;
}

export function clampStep(val: number, min: number, max: number, step: number): number {
  const clamped = Math.max(min, Math.min(max, val));
  const rounded = Math.round((clamped - min) / step) * step + min;
  const fixed = Number(rounded.toFixed(4));
  return Math.max(min, Math.min(max, fixed));
}

export function valueToRotationDeg(val: number, min: number, max: number): number {
  if (max <= min) return 0;
  const norm = (val - min) / (max - min);
  return Math.round(-135 + norm * 270);
}

export class SonixKnobElement extends HTMLElement {
  private opts: KnobOptions | null = null;
  private dragState: { x: number; y: number; startVal: number } | null = null;

  configure(opts: KnobOptions): void {
    this.opts = opts;
    this.className = `knob ${opts.skinClass}`;
    this.style.width = `${opts.sizePx}px`;
    this.style.height = `${opts.sizePx}px`;
    this.setAttribute('data-control-id', opts.controlId);
    this.setAttribute('role', 'slider');
    this.setAttribute('tabindex', '0');
    this.setAttribute('aria-label', opts.label);
    this.setAttribute('aria-valuemin', String(opts.min));
    this.setAttribute('aria-valuemax', String(opts.max));
    this.setValueDisplay(opts.value);

    if (!this.firstChild) {
      this.innerHTML =
        '<span class="sh"></span><span class="rotor"><span class="a"></span><span class="b"></span><span class="p"></span></span>';
      this.attachListeners();
    }
    this.updateRotor();
  }

  private setValueDisplay(v: number): void {
    if (!this.opts) return;
    const txt = `${v.toFixed(this.opts.step && this.opts.step >= 1 ? 0 : 1)}${this.opts.unit ? ' ' + this.opts.unit : ''}`;
    this.setAttribute('aria-valuenow', String(v));
    this.setAttribute('aria-valuetext', txt);
    this.title = `${this.opts.label}: ${txt}`;
  }

  private updateRotor(): void {
    if (!this.opts) return;
    const rotor = this.querySelector('.rotor') as HTMLElement | null;
    if (rotor) {
      const deg = valueToRotationDeg(this.opts.value, this.opts.min, this.opts.max);
      rotor.style.transform = `rotate(${deg}deg)`;
    }
  }

  private commit(next: number): void {
    if (!this.opts) return;
    const step = this.opts.step ?? 0.1;
    const v = clampStep(next, this.opts.min, this.opts.max, step);
    this.opts.value = v;
    this.setValueDisplay(v);
    this.updateRotor();
    this.opts.onChange(v);
  }

  private attachListeners(): void {
    this.addEventListener('pointerdown', (e: PointerEvent) => {
      if (!this.opts) return;
      try {
        this.setPointerCapture(e.pointerId);
      } catch {
        // ignore if unsupported in JSDOM
      }
      this.dragState = {
        x: e.clientX,
        y: e.clientY,
        startVal: this.opts.value,
      };
      this.opts.onDragState?.(true);
    });

    this.addEventListener('pointermove', (e: PointerEvent) => {
      if (!this.opts || !this.dragState) return;
      const span = this.opts.max - this.opts.min;
      const deltaPx = e.clientX - this.dragState.x + (this.dragState.y - e.clientY);
      const sensitivity = e.shiftKey ? 520 : 140;
      const next = this.dragState.startVal + (deltaPx / sensitivity) * span;
      this.commit(next);
    });

    const endDrag = () => {
      if (!this.dragState) return;
      this.dragState = null;
      this.opts?.onDragState?.(false);
    };
    this.addEventListener('pointerup', endDrag);
    this.addEventListener('pointercancel', endDrag);

    this.addEventListener('wheel', (e: WheelEvent) => {
      if (!this.opts) return;
      e.preventDefault();
      const span = this.opts.max - this.opts.min;
      const coarse = span >= 50 ? span / 40 : 0.2;
      const delta = (e.deltaY < 0 ? 1 : -1) * (e.shiftKey ? coarse * 0.25 : coarse);
      this.commit(this.opts.value + delta);
    }, { passive: false });

    this.addEventListener('dblclick', () => {
      if (!this.opts) return;
      this.commit(this.opts.defaultValue);
    });

    this.addEventListener('keydown', (e: KeyboardEvent) => {
      if (!this.opts) return;
      const dir =
        e.key === 'ArrowUp' || e.key === 'ArrowRight'
          ? 1
          : e.key === 'ArrowDown' || e.key === 'ArrowLeft'
            ? -1
            : 0;
      if (!dir) return;
      e.preventDefault();
      const span = this.opts.max - this.opts.min;
      const delta = span >= 2 ? 0.5 : (this.opts.step ?? 0.05);
      this.commit(this.opts.value + dir * delta);
    });
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('sonix-knob')) {
  customElements.define('sonix-knob', SonixKnobElement);
}

export function createKnob(opts: KnobOptions): SonixKnobElement {
  const el = document.createElement('sonix-knob') as SonixKnobElement;
  if (typeof el.configure === 'function') {
    el.configure(opts);
  }
  return el;
}

export interface HSliderOptions {
  controlId: string;
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  defaultValue: number;
  unit?: string;
  format?: (v: number) => string;
  fillGradient?: string;
  onChange: (val: number) => void;
  onDragState?: (dragging: boolean) => void;
}

export function createHSliderRow(opts: HSliderOptions): HTMLElement {
  const row = document.createElement('div');
  row.style.cssText = 'display:flex;align-items:center;gap:8px;min-width:0';

  const lbl = document.createElement('span');
  lbl.textContent = opts.label;
  lbl.style.cssText =
    'font-size:11px;width:84px;color:#c9ced4;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;flex-shrink:0';

  const track = document.createElement('div');
  track.className = 'm-track';
  track.setAttribute('data-control-id', opts.controlId);
  track.setAttribute('role', 'slider');
  track.tabIndex = 0;
  track.setAttribute('aria-label', opts.label);
  track.setAttribute('aria-valuemin', String(opts.min));
  track.setAttribute('aria-valuemax', String(opts.max));
  track.setAttribute('aria-valuenow', String(opts.value));

  const pct = Math.max(0, Math.min(100, ((opts.value - opts.min) / (opts.max - opts.min)) * 100));
  const fill = document.createElement('div');
  fill.style.cssText = `width:${pct.toFixed(1)}%;height:100%;border-radius:3px;background:${opts.fillGradient ?? 'linear-gradient(90deg,#b8501a,#ff8a3d)'}`;

  const thumb = document.createElement('div');
  thumb.style.cssText = `position:absolute;top:-4px;left:${pct.toFixed(1)}%;margin-left:-6px;width:12px;height:14px;border-radius:3px;background:linear-gradient(180deg,#e8e8e8,#8a8a8a);box-shadow:0 1px 3px rgba(0,0,0,.8);pointer-events:none`;

  const readout = document.createElement('span');
  readout.className = 'm-mono';
  readout.style.cssText = 'font-size:10px;width:54px;text-align:right;flex-shrink:0;color:#d6dce2';
  const formatVal = (v: number) =>
    opts.format ? opts.format(v) : `${v.toFixed(opts.step >= 1 ? 0 : 1)}${opts.unit ? ' ' + opts.unit : ''}`;
  readout.textContent = formatVal(opts.value);

  const updateUi = (v: number) => {
    const clamped = clampStep(v, opts.min, opts.max, opts.step);
    const nextPct = Math.max(0, Math.min(100, ((clamped - opts.min) / (opts.max - opts.min)) * 100));
    fill.style.width = `${nextPct.toFixed(1)}%`;
    thumb.style.left = `${nextPct.toFixed(1)}%`;
    track.setAttribute('aria-valuenow', String(clamped));
    readout.textContent = formatVal(clamped);
    opts.onChange(clamped);
  };

  let dragging = false;
  const applyClientX = (clientX: number) => {
    const r = track.getBoundingClientRect();
    if (r.width <= 0) return;
    const ratio = Math.max(0, Math.min(1, (clientX - r.left) / r.width));
    updateUi(opts.min + ratio * (opts.max - opts.min));
  };

  track.addEventListener('pointerdown', (e) => {
    try {
      track.setPointerCapture(e.pointerId);
    } catch {
      // ignore
    }
    dragging = true;
    opts.onDragState?.(true);
    applyClientX(e.clientX);
  });
  track.addEventListener('pointermove', (e) => {
    if (!dragging) return;
    applyClientX(e.clientX);
  });
  const endSliderDrag = () => {
    if (!dragging) return;
    dragging = false;
    opts.onDragState?.(false);
  };
  track.addEventListener('pointerup', endSliderDrag);
  track.addEventListener('pointercancel', endSliderDrag);
  track.addEventListener('wheel', (e) => {
    e.preventDefault();
    const cur = Number(track.getAttribute('aria-valuenow') ?? opts.value);
    const delta = (e.deltaY < 0 ? 1 : -1) * Math.max(opts.step, (opts.max - opts.min) / 40);
    updateUi(cur + (e.shiftKey ? delta * 0.25 : delta));
  }, { passive: false });
  track.addEventListener('dblclick', () => {
    updateUi(opts.defaultValue);
  });
  track.addEventListener('keydown', (e) => {
    const dir =
      e.key === 'ArrowUp' || e.key === 'ArrowRight'
        ? 1
        : e.key === 'ArrowDown' || e.key === 'ArrowLeft'
          ? -1
          : 0;
    if (!dir) return;
    e.preventDefault();
    const cur = Number(track.getAttribute('aria-valuenow') ?? opts.value);
    const span = opts.max - opts.min;
    const step = span >= 2 && opts.step <= 0.5 ? 0.5 : Math.max(opts.step, span / 20);
    updateUi(cur + dir * step);
  });

  track.append(fill, thumb);
  row.append(lbl, track, readout);
  return row;
}
