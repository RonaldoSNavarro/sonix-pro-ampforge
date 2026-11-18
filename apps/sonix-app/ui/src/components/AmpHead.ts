import type { AppStore } from '../store.ts';
import { createKnob } from './Knob.ts';

export type KnobSkin = 'k-marshall' | 'k-fender' | 'k-mesa' | 'k-vox';
export type AmpModelId = 'plexi_1959' | 'jcm800_2203' | 'twin_ab763' | 'dual_rect' | 'ac30_tb';

interface AmpTheme {
  chassisBg: string;
  grilleClass: string;
  panelBg: string;
  panelBorder: string;
  textColor: string;
  subTextColor: string;
  knobSkin: KnobSkin;
  brand: string;
  modelLabel: string;
  piping: string;
}

const AMP_THEMES: Record<AmpModelId, AmpTheme> = {
  plexi_1959: {
    chassisBg: 'linear-gradient(180deg, #1b1b1b, #0e0e0e)',
    grilleClass: 'g-plexi',
    panelBg: 'linear-gradient(180deg, #e5bf62 0%, #c69a33 52%, #8e6b1b 100%)',
    panelBorder: '#574110',
    textColor: '#181207',
    subTextColor: '#3a2b0d',
    knobSkin: 'k-marshall',
    brand: 'BRIT PLEXI',
    modelLabel: '1959 SUPER LEAD · MK II · 100W',
    piping: '#e4c97a',
  },
  jcm800_2203: {
    chassisBg: 'linear-gradient(180deg, #171717, #0b0b0b)',
    grilleClass: 'g-800',
    panelBg: 'linear-gradient(180deg, #e0bd65 0%, #bd9335 55%, #846218 100%)',
    panelBorder: '#4c380b',
    textColor: '#161106',
    subTextColor: '#2f2409',
    knobSkin: 'k-marshall',
    brand: 'BRIT 800',
    modelLabel: 'JCM 800 · 2203 LEAD SERIES · 100W',
    piping: '#f0d88b',
  },
  twin_ab763: {
    chassisBg: 'linear-gradient(180deg, #181a1d, #0d0e10)',
    grilleClass: 'g-fender',
    panelBg: 'linear-gradient(180deg, #1d2025 0%, #0d0f12 65%, #07080a 100%)',
    panelBorder: '#3f454d',
    textColor: '#f1f3f5',
    subTextColor: '#aeb5be',
    knobSkin: 'k-fender',
    brand: 'TWIN REVERB',
    modelLabel: 'BLACKFACE AB763 · DUAL 6L6GC · 85W',
    piping: '#cfd4da',
  },
  dual_rect: {
    chassisBg: 'linear-gradient(180deg, #15171a, #0a0b0d)',
    grilleClass: 'g-mesa',
    panelBg:
      'repeating-linear-gradient(90deg, rgba(255,255,255,0.025) 0 1px, transparent 1px 3px), linear-gradient(180deg, #242424, #0c0c0c)',
    panelBorder: '#4d535c',
    textColor: '#eef1f4',
    subTextColor: '#a2a9b2',
    knobSkin: 'k-mesa',
    brand: 'CALI RECTO',
    modelLabel: 'DUAL RECTIFIER · REV F',
    piping: '#9da3aa',
  },
  ac30_tb: {
    chassisBg: 'linear-gradient(180deg, #1c1714, #0f0c0a)',
    grilleClass: 'g-vox',
    panelBg: 'linear-gradient(180deg, #74161a 0%, #520e12 58%, #34080a 100%)',
    panelBorder: '#8d2b2f',
    textColor: '#f7ead2',
    subTextColor: '#d9c099',
    knobSkin: 'k-vox',
    brand: 'VOX TOP BOOST',
    modelLabel: 'AC30/6 TB · CLASS A QUAD EL84 · 30W',
    piping: '#dfc27c',
  },
};

function numParam(store: AppStore, key: string, fallback: number): number {
  const v = store.getParam(key, fallback);
  return typeof v === 'number' && Number.isFinite(v) ? v : fallback;
}

function boolParam(store: AppStore, key: string, fallback: boolean): boolean {
  const v = store.getParam(key, fallback);
  return typeof v === 'boolean' ? v : Boolean(v);
}

function strParam(store: AppStore, key: string, fallback: string): string {
  const v = store.getParam(key, fallback);
  return typeof v === 'string' && v.length > 0 ? v : String(v ?? fallback);
}

function createRockerSwitch(
  controlId: string,
  label: string,
  active: boolean,
  onColor: string,
  onToggle: () => void
): HTMLElement {
  const wrap = document.createElement('div');
  wrap.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:2px;flex:none;';

  const btn = document.createElement('button');
  btn.type = 'button';
  btn.className = 'rocker';
  btn.dataset.controlId = controlId;
  btn.setAttribute('aria-pressed', active ? 'true' : 'false');
  btn.title = label;

  const face = document.createElement('span');
  face.className = 'face';
  face.style.background = active
    ? `linear-gradient(180deg, ${onColor}, #4a0a07)`
    : 'linear-gradient(180deg, #2b2d31, #121315)';
  face.style.boxShadow = active
    ? `0 0 10px ${onColor}, inset 0 2px 3px rgba(255,255,255,0.35)`
    : 'inset 0 -3px 4px rgba(0,0,0,0.85)';
  btn.appendChild(face);
  btn.addEventListener('click', onToggle);

  const lbl = document.createElement('span');
  lbl.className = 'm-lbl';
  lbl.textContent = label;
  lbl.style.fontSize = '8px';
  lbl.style.color = 'currentColor';
  wrap.append(btn, lbl);
  return wrap;
}

function createBatToggle(
  controlId: string,
  label: string,
  active: boolean,
  upLabel: string,
  downLabel: string,
  onToggle: () => void,
  showBottomLabel = true
): HTMLElement {
  const wrap = document.createElement('div');
  wrap.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:1px;flex:none;';

  const top = document.createElement('span');
  top.className = 'm-lbl';
  top.textContent = upLabel;
  top.style.cssText = 'font-size:7px;line-height:1;opacity:0.85;color:currentColor;';

  const btn = document.createElement('button');
  btn.type = 'button';
  btn.className = 'bat';
  btn.dataset.controlId = controlId;
  btn.setAttribute('aria-pressed', active ? 'true' : 'false');
  btn.title = `${label}: ${active ? upLabel : downLabel}`;
  btn.style.height = '44px';

  const nut = document.createElement('span');
  nut.className = 'nut';
  nut.style.top = '13px';
  const lev = document.createElement('span');
  lev.className = 'lev';
  lev.style.top = active ? '2px' : '22px';
  btn.append(nut, lev);
  btn.addEventListener('click', onToggle);

  wrap.append(top, btn);
  if (showBottomLabel) {
    const bot = document.createElement('span');
    bot.className = 'm-lbl';
    bot.textContent = label || downLabel;
    bot.style.cssText = 'font-size:7.5px;line-height:1;color:currentColor;';
    wrap.appendChild(bot);
  }
  return wrap;
}

function createJewelLight(lit: boolean, titleText: string): HTMLElement {
  const wrap = document.createElement('div');
  wrap.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:2px;flex:none;';

  const jewel = document.createElement('div');
  jewel.className = 'jewel';
  jewel.title = titleText;
  if (!lit) {
    jewel.style.filter = 'brightness(0.26) saturate(0.45)';
    jewel.style.boxShadow = '0 0 0 3px #8a8a8a, 0 0 0 5px #3a3a3a';
  }
  wrap.appendChild(jewel);
  return wrap;
}

function createJackSocket(
  jackId: string,
  label: string,
  plugged: boolean,
  onClick: () => void
): HTMLElement {
  const wrap = document.createElement('div');
  wrap.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:2px;flex:none;position:relative;';

  const jack = document.createElement('button');
  jack.type = 'button';
  jack.className = 'jack';
  jack.dataset.controlId = 'amp.input_jack';
  jack.dataset.jackId = jackId;
  jack.setAttribute('aria-pressed', plugged ? 'true' : 'false');
  jack.title = `Entrada ${label}`;
  if (plugged) {
    const plug = document.createElement('span');
    plug.className = 'plug';
    jack.appendChild(plug);
  }
  jack.addEventListener('click', onClick);

  const lbl = document.createElement('span');
  lbl.className = 'm-lbl';
  lbl.textContent = label;
  lbl.style.cssText = 'font-size:7.5px;line-height:1;color:currentColor;white-space:nowrap;';

  wrap.append(jack, lbl);
  return wrap;
}

function buildKnobControl(
  store: AppStore,
  controlId: string,
  paramKey: string,
  label: string,
  skin: KnobSkin,
  sizePx: number,
  fallback: number,
  onDragState?: (dragging: boolean) => void
): HTMLElement {
  const desc = store.getDescriptor(controlId);
  const [min, max] = desc?.range ?? [0, 10];
  const step = desc?.step ?? 0.1;
  const def = typeof desc?.defaultValue === 'number' ? desc.defaultValue : fallback;

  const col = document.createElement('div');
  col.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:6px;flex:none;position:relative;';

  const lbl = document.createElement('span');
  lbl.className = 'm-osw';
  lbl.textContent = label;
  lbl.style.cssText = 'font-size:11px;font-weight:600;letter-spacing:.08em;color:currentColor;white-space:nowrap;min-height:15px;';

  const knobEl = createKnob({
    controlId,
    label,
    value: numParam(store, paramKey, def),
    min,
    max,
    step,
    defaultValue: def,
    skinClass: skin,
    sizePx,
    onChange: (val) => {
      void store.setControl(controlId, val);
    },
    onDragState: onDragState ?? ((d) => store.setDraggingKnob(d)),
  });
  const subSlot = document.createElement('div');
  subSlot.style.cssText = 'min-height:15px;display:flex;justify-content:center;';
  col.append(lbl, knobEl, subSlot);
  return col;
}

export interface AmpHeadOptions {
  modelOverride?: AmpModelId;
  compact?: boolean;
  onKnobDragState?: (dragging: boolean) => void;
  onSelectAmpModel?: (model: AmpModelId) => void;
}

export function createAmpHeadElement(store: AppStore, options: AmpHeadOptions = {}): HTMLElement {
  const modelRaw = options.modelOverride ?? strParam(store, 'amp_1.model', 'jcm800_2203');
  const model: AmpModelId =
    modelRaw === 'plexi_1959' ||
    modelRaw === 'jcm800_2203' ||
    modelRaw === 'twin_ab763' ||
    modelRaw === 'dual_rect' ||
    modelRaw === 'ac30_tb'
      ? modelRaw
      : 'jcm800_2203';
  const theme = AMP_THEMES[model];
  const power = boolParam(store, 'amp_1.power', true);
  const standby = boolParam(store, 'amp_1.standby', false);
  const bright = boolParam(store, 'amp_1.bright', false);
  const enabled = boolParam(store, 'amp_1.enabled', true);
  const inputJack = strParam(store, 'amp_1.input_jack', 'high');
  const jumpered = boolParam(store, 'amp_1.jumpered', false) || inputJack === 'jumpered';
  const isLit = power && !standby;
  const kSizeByModel: Record<AmpModelId, number> = {
    plexi_1959: 54,
    jcm800_2203: 54,
    twin_ab763: 40,
    dual_rect: 50,
    ac30_tb: 46,
  };
  const kSize = options.compact ? 32 : kSizeByModel[model];

  const head = document.createElement('section');
  head.className = 'amp-head tolex';
  head.dataset.ampModel = model;
  head.style.cssText = `
    position: relative;
    width: 1000px;
    height: 282px;
    display: flex;
    flex-direction: column;
    border-radius: 12px;
    padding: 12px;
    box-sizing: border-box;
    background: ${theme.chassisBg};
    box-shadow: 0 26px 40px rgba(0,0,0,0.75), 0 0 0 1px #000, inset 0 1px 0 rgba(255,255,255,0.08);
    gap: 8px;
    min-width: 0;
  `;
  if (!enabled) {
    head.style.opacity = '0.72';
  }

  const topStrap = document.createElement('div');
  topStrap.style.cssText =
    'position:absolute;left:50%;top:-7px;margin-left:-62px;width:124px;height:12px;border-radius:6px 6px 2px 2px;background:linear-gradient(180deg,#2a2a2a,#0a0a0a);box-shadow:0 2px 3px rgba(0,0,0,.7);z-index:2;';
  head.appendChild(topStrap);

  // Top grille / window header
  const grille = document.createElement('div');
  grille.className = `amp-grille ${theme.grilleClass}`;
  const grillePipingByModel: Partial<Record<AmpModelId, string>> = {
    dual_rect: 'inset 0 0 0 3px #1c1c1c, inset 0 0 0 5px #8f949a',
    twin_ab763: 'inset 0 0 0 3px #1a1a1a',
  };
  const grillePiping =
    grillePipingByModel[model] ?? `inset 0 0 0 2px ${theme.piping}, inset 0 0 0 3px rgba(0,0,0,0.6)`;
  grille.style.cssText = `
    position: relative;
    flex: 1;
    min-height: 0;
    border-radius: 6px;
    box-shadow: ${grillePiping};
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  `;

  const plaque = document.createElement('div');
  plaque.style.cssText =
    'position:relative;padding:7px 24px;border-radius:5px;background:linear-gradient(180deg,#f4f6f8,#aab0b6 50%,#e1e4e7);box-shadow:0 4px 8px rgba(0,0,0,.6),inset 0 1px 0 rgba(255,255,255,.55),inset 0 -1px 0 rgba(0,0,0,.3);';
  plaque.innerHTML = `<span class="m-osw" style="font-size:30px;font-weight:700;letter-spacing:0.12em;color:#141414;text-shadow:0 1px 0 rgba(255,255,255,.35);">${theme.brand}</span>`;

  const cornerLbl = document.createElement('span');
  cornerLbl.className = 'm-mono';
  cornerLbl.style.cssText = 'position:absolute;right:16px;bottom:9px;font-size:10px;letter-spacing:0.14em;color:rgba(255,240,210,0.6);';
  cornerLbl.textContent = theme.modelLabel;

  grille.append(plaque, cornerLbl);

  // Authentic Metal Control Panel
  const panel = document.createElement('div');
  panel.className = 'control-panel';
  const panelBorderByModel: Partial<Record<AmpModelId, string>> = {
    dual_rect: 'border: 2px solid #8f949a;',
    twin_ab763: 'border: 2px solid #a9aeb3;',
  };
  const panelBorderCss = panelBorderByModel[model] ?? 'border: none;';
  panel.style.cssText = `
    position: relative;
    flex: none;
    height: 130px;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 18px;
    padding: 10px 20px;
    border-radius: 5px;
    ${panelBorderCss}
    background: ${theme.panelBg};
    color: ${theme.textColor};
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.45), inset 0 -2px 0 rgba(0,0,0,0.3), 0 2px 3px rgba(0,0,0,0.5);
    min-width: 0;
  `;

  const pushSwitch = (el: HTMLElement) => panel.appendChild(el);
  const pushKnob = (cid: string, key: string, label: string, def = 5) =>
    panel.appendChild(
      buildKnobControl(store, cid, key, label, theme.knobSkin, kSize, def, options.onKnobDragState)
    );

  if (model === 'plexi_1959') {
    // CA-6 order: Power, Standby, Indicator, Presence, Bass, Middle, Treble, Volume I, Volume II, 4 Jacks + JUMPER
    pushSwitch(
      createRockerSwitch('amp.power', 'POWER', power, '#ff3a28', () => {
        void store.setControl('amp.power', !power);
      })
    );
    pushSwitch(
      createRockerSwitch('amp.standby', 'STANDBY', standby, '#ffb020', () => {
        void store.setControl('amp.standby', !standby);
      })
    );
    panel.appendChild(createJewelLight(isLit, 'Indicador Pilot Plexi 1959'));

    pushKnob('amp.presence', 'amp_1.presence', 'PRESENCE', 5.0);
    pushKnob('amp.bass', 'amp_1.bass', 'BASS', 5.5);
    pushKnob('amp.middle', 'amp_1.middle', 'MIDDLE', 5.0);
    pushKnob('amp.treble', 'amp_1.treble', 'TREBLE', 6.0);
    pushKnob('amp.gain', 'amp_1.gain', 'VOLUME I', 6.5);
    pushKnob('amp.volume_2', 'amp_1.volume_2', 'VOLUME II', 4.5);

    const jackGridWrap = document.createElement('div');
    jackGridWrap.style.cssText =
      'position:relative;display:flex;align-items:center;gap:7px;padding-left:4px;flex:none;';

    const grid2x2 = document.createElement('div');
    grid2x2.style.cssText =
      'position:relative;display:grid;grid-template-columns:repeat(2,auto);gap:4px 8px;align-items:center;';

    const jHigh1 = createJackSocket(
      'high1',
      'I-HIGH',
      jumpered || inputJack === 'high1' || inputJack === 'high',
      () => {
        void store.setControl('amp.jumpered', false);
        void store.setControl('amp.input_jack', 'high1');
      }
    );
    const jHigh2 = createJackSocket('high2', 'II-HIGH', jumpered || inputJack === 'high2', () => {
      void store.setControl('amp.jumpered', false);
      void store.setControl('amp.input_jack', 'high2');
    });
    const jLow1 = createJackSocket('low1', 'I-LOW', jumpered || inputJack === 'low1', () => {
      void store.setControl('amp.jumpered', false);
      void store.setControl('amp.input_jack', 'low1');
    });
    const jLow2 = createJackSocket('low2', 'II-Low', !jumpered && inputJack === 'low2', () => {
      // CA-7: clicking II-Low sends low2 and drops jumper
      void store.setControl('amp.jumpered', false);
      void store.setControl('amp.input_jack', 'low2');
    });

    grid2x2.append(jHigh1, jHigh2, jLow1, jLow2);

    if (jumpered) {
      const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
      svg.setAttribute('class', 'plexi-jumper-cable');
      svg.setAttribute('viewBox', '0 0 80 76');
      svg.style.cssText =
        'position:absolute;inset:0;width:100%;height:100%;pointer-events:none;overflow:visible;z-index:5;';
      const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
      path.setAttribute('d', 'M 14 54 C 14 74, 62 72, 58 14');
      path.setAttribute('fill', 'none');
      path.setAttribute('stroke', '#181818');
      path.setAttribute('stroke-width', '4.5');
      path.setAttribute('stroke-linecap', 'round');
      const hi = document.createElementNS('http://www.w3.org/2000/svg', 'path');
      hi.setAttribute('d', 'M 14 54 C 14 74, 62 72, 58 14');
      hi.setAttribute('fill', 'none');
      hi.setAttribute('stroke', '#ff9b4a');
      hi.setAttribute('stroke-width', '1.2');
      hi.setAttribute('stroke-opacity', '0.75');
      svg.append(path, hi);
      grid2x2.appendChild(svg);
    }

    const jumperBtn = document.createElement('button');
    jumperBtn.type = 'button';
    jumperBtn.className = 'selbtn';
    jumperBtn.dataset.controlId = 'amp.jumpered';
    jumperBtn.setAttribute('aria-pressed', jumpered ? 'true' : 'false');
    jumperBtn.textContent = 'JUMPER';
    jumperBtn.title = 'Conectar cabo jumper I-Low -> II-High';
    if (jumpered) {
      jumperBtn.style.borderColor = '#ff8a3d';
      jumperBtn.style.color = '#ffbe82';
    }
    jumperBtn.addEventListener('click', () => {
      const next = !jumpered;
      void store.setControl('amp.jumpered', next);
      void store.setControl('amp.input_jack', next ? 'jumpered' : 'high1');
    });

    jackGridWrap.append(grid2x2, jumperBtn);
    panel.appendChild(jackGridWrap);
  } else if (model === 'jcm800_2203') {
    // CA-6 order: Power, Standby, Indicator + BRIT 800, Presence, Bass, Middle, Treble, Master, Pre-Amp, High, Low
    pushSwitch(
      createRockerSwitch('amp.power', 'POWER', power, '#ff3a28', () => {
        void store.setControl('amp.power', !power);
      })
    );
    pushSwitch(
      createRockerSwitch('amp.standby', 'STANDBY', standby, '#ffb020', () => {
        void store.setControl('amp.standby', !standby);
      })
    );

    const indicatorBrand = document.createElement('div');
    indicatorBrand.style.cssText = 'display:flex;align-items:center;gap:6px;flex:none;';
    indicatorBrand.appendChild(createJewelLight(isLit, 'Indicador Pilot JCM800'));
    const b800 = document.createElement('span');
    b800.className = 'm-osw';
    b800.textContent = 'BRIT 800';
    b800.style.cssText =
      'font-size:12px;font-weight:700;padding:2px 6px;border:1px solid #1d1507;border-radius:3px;background:rgba(0,0,0,0.1);letter-spacing:0.08em;';
    indicatorBrand.appendChild(b800);
    panel.appendChild(indicatorBrand);

    pushKnob('amp.presence', 'amp_1.presence', 'PRESENCE', 5.0);
    pushKnob('amp.bass', 'amp_1.bass', 'BASS', 5.5);
    pushKnob('amp.middle', 'amp_1.middle', 'MIDDLE', 5.0);
    pushKnob('amp.treble', 'amp_1.treble', 'TREBLE', 6.0);
    pushKnob('amp.master', 'amp_1.master', 'MASTER', 5.0);
    pushKnob('amp.gain', 'amp_1.gain', 'PRE-AMP', 6.5);

    const jacks = document.createElement('div');
    jacks.style.cssText = 'display:flex;align-items:center;gap:8px;flex:none;';
    jacks.append(
      createJackSocket('high', 'HIGH', inputJack !== 'low', () => {
        void store.setControl('amp.input_jack', 'high');
      }),
      createJackSocket('low', 'LOW', inputJack === 'low', () => {
        void store.setControl('amp.input_jack', 'low');
      })
    );
    panel.appendChild(jacks);
  } else if (model === 'twin_ab763') {
    // Normal vs Vibrato Twin AB763 layout
    const channelGroup = document.createElement('div');
    channelGroup.style.cssText =
      'display:flex;align-items:center;gap:5px;padding:3px 6px;border-radius:4px;border:1px solid #353b44;background:rgba(255,255,255,0.03);flex:none;';

    const normalActive = inputJack === 'normal' || inputJack === 'normal_high' || inputJack === 'normal_low';
    const normBtn = document.createElement('button');
    normBtn.type = 'button';
    normBtn.className = 'm-chip';
    normBtn.dataset.controlId = 'amp.input_jack';
    normBtn.dataset.jackId = 'normal';
    normBtn.setAttribute('aria-pressed', normalActive ? 'true' : 'false');
    normBtn.textContent = 'NORMAL';
    normBtn.addEventListener('click', () => void store.setControl('amp.input_jack', 'normal'));

    const vibBtn = document.createElement('button');
    vibBtn.type = 'button';
    vibBtn.className = 'm-chip';
    vibBtn.dataset.controlId = 'amp.input_jack';
    vibBtn.dataset.jackId = 'vibrato';
    vibBtn.setAttribute('aria-pressed', !normalActive ? 'true' : 'false');
    vibBtn.textContent = 'VIBRATO';
    vibBtn.addEventListener('click', () => void store.setControl('amp.input_jack', 'vibrato'));

    channelGroup.append(normBtn, vibBtn);
    panel.appendChild(channelGroup);

    panel.appendChild(
      createBatToggle('amp.bright', 'BRIGHT', bright, 'ON', 'OFF', () => {
        void store.setControl('amp.bright', !bright);
      })
    );
    pushKnob('amp.gain', 'amp_1.gain', 'VOLUME', 5.0);
    pushKnob('amp.treble', 'amp_1.treble', 'TREBLE', 6.0);
    pushKnob('amp.middle', 'amp_1.middle', 'MIDDLE', 5.0);
    pushKnob('amp.bass', 'amp_1.bass', 'BASS', 5.5);
    pushKnob('amp.reverb', 'amp_1.reverb', 'REVERB', 3.0);
    pushKnob('amp.vibrato_speed', 'amp_1.vibrato_speed', 'SPEED', 4.0);
    pushKnob('amp.vibrato_depth', 'amp_1.vibrato_depth', 'INTENSITY', 3.5);

    panel.appendChild(createJewelLight(isLit, 'Jewel Blue/Red AB763'));
    panel.appendChild(
      createBatToggle('amp.power', 'POWER', power, 'ON', 'OFF', () => {
        void store.setControl('amp.power', !power);
      })
    );
    panel.appendChild(
      createBatToggle('amp.standby', 'STBY', standby, 'STBY', 'PLAY', () => {
        void store.setControl('amp.standby', !standby);
      })
    );
  } else if (model === 'dual_rect') {
    const rectMode = strParam(store, 'amp_1.channel_mode', 'modern');
    const spongy = boolParam(store, 'amp_1.spongy', false);
    const rectifier = strParam(store, 'amp_1.rectifier', 'diode');

    const modeBox = document.createElement('div');
    modeBox.style.cssText =
      'border-top:1px solid rgba(255,255,255,0.16);padding-top:4px;display:flex;flex-direction:column;gap:6px;flex:none;';
    modeBox.innerHTML = '<span class="m-osw" style="font-size:10px;font-weight:700;letter-spacing:.18em;color:#e8e8e8;text-align:center;">MODO</span>';
    const modeGrid = document.createElement('div');
    modeGrid.style.cssText = 'display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:5px;';
    (
      [
        ['clean', 'CLEAN'],
        ['pushed', 'PUSHED'],
        ['vintage', 'VINTAGE'],
        ['modern', 'MODERN'],
      ] as const
    ).forEach(([val, text]) => {
      const b = document.createElement('button');
      b.type = 'button';
      b.dataset.controlId = 'amp.channel_mode';
      b.setAttribute('aria-pressed', rectMode === val ? 'true' : 'false');
      b.style.cssText = `
        height:22px;padding:0 7px;border-radius:3px;border:1px solid #000;
        background:linear-gradient(180deg,#2c2c2c,#121212);color:#ddd;font-size:9px;font-weight:700;letter-spacing:.06em;
        display:flex;align-items:center;gap:5px;cursor:pointer;
      `;
      const ledColor = val === 'modern' || val === 'pushed' ? '#ff3b2f' : '#29e07a';
      b.innerHTML = `<span style="width:7px;height:7px;border-radius:50%;background:${
        rectMode === val ? ledColor : '#2a2a2a'
      };box-shadow:${rectMode === val ? `0 0 6px ${ledColor}` : 'none'};"></span><span>${text}</span>`;
      b.addEventListener('click', () => void store.setControl('amp.channel_mode', val));
      modeGrid.appendChild(b);
    });
    modeBox.appendChild(modeGrid);

    const knobCluster = document.createElement('div');
    knobCluster.style.cssText = 'display:flex;align-items:flex-end;gap:16px;flex:none;';
    knobCluster.append(
      buildKnobControl(store, 'amp.gain', 'amp_1.gain', 'GAIN', theme.knobSkin, kSize, 7.0, options.onKnobDragState),
      buildKnobControl(store, 'amp.treble', 'amp_1.treble', 'TREBLE', theme.knobSkin, kSize, 6.0, options.onKnobDragState),
      buildKnobControl(store, 'amp.middle', 'amp_1.middle', 'MID', theme.knobSkin, kSize, 4.0, options.onKnobDragState),
      buildKnobControl(store, 'amp.bass', 'amp_1.bass', 'BASS', theme.knobSkin, kSize, 6.0, options.onKnobDragState),
      buildKnobControl(store, 'amp.presence', 'amp_1.presence', 'PRESENCE', theme.knobSkin, kSize, 5.0, options.onKnobDragState),
      buildKnobControl(store, 'amp.master', 'amp_1.master', 'MASTER', theme.knobSkin, kSize, 3.5, options.onKnobDragState)
    );

    const rearBox = document.createElement('div');
    rearBox.style.cssText =
      'border-top:1px solid rgba(255,255,255,0.16);padding-top:4px;display:flex;flex-direction:column;gap:6px;flex:none;';
    rearBox.innerHTML = '<span class="m-osw" style="font-size:10px;font-weight:700;letter-spacing:.18em;color:#e8e8e8;text-align:center;">TRASEIRO</span>';
    const rearRow = document.createElement('div');
    rearRow.style.cssText = 'display:flex;align-items:flex-end;gap:16px;';
    const isTubeRect = rectifier === '5u4gb' || rectifier === 'gz34';
    const mkRear = (cid: string, title: string, on: boolean, up: string, dn: string, fn: () => void) => {
      const wrap = document.createElement('div');
      wrap.style.cssText = 'display:flex;flex-direction:column;align-items:center;gap:2px;';
      const t = document.createElement('span');
      t.className = 'm-osw';
      t.style.cssText = 'font-size:9px;font-weight:600;letter-spacing:.08em;color:#e8e8e8;white-space:nowrap;';
      t.textContent = title;
      wrap.append(t, createBatToggle(cid, dn, on, up, dn, fn));
      return wrap;
    };
    rearRow.append(
      mkRear('chassi.rectifier', 'RECTIFIER', isTubeRect, 'TUBE', 'DIODE', () => {
        void store.setControl('chassi.rectifier', isTubeRect ? 'diode' : '5u4gb');
      }),
      mkRear('amp.spongy', 'POWER', spongy, 'SPONGY', 'BOLD', () => {
        void store.setControl('amp.spongy', !spongy);
      })
    );
    rearBox.appendChild(rearRow);

    const groupsWrap = document.createElement('div');
    groupsWrap.style.cssText = 'flex:1;display:flex;align-items:center;justify-content:space-around;gap:14px;min-width:0;';
    groupsWrap.append(modeBox, knobCluster, rearBox);
    panel.appendChild(groupsWrap);

    const pwrSep = document.createElement('div');
    pwrSep.style.cssText = 'width:1px;align-self:stretch;background:rgba(255,255,255,0.16);';
    panel.appendChild(pwrSep);

    const pwrRow = document.createElement('div');
    pwrRow.style.cssText = 'display:flex;align-items:center;gap:12px;flex:none;';
    pwrRow.append(
      createBatToggle('amp.standby', 'STANDBY', standby, 'STANDBY', '', () => {
        void store.setControl('amp.standby', !standby);
      }, false),
      createBatToggle('amp.power', 'POWER', power, 'POWER', '', () => {
        void store.setControl('amp.power', !power);
      }, false),
      createJewelLight(isLit, 'Jewel Dual Rectifier')
    );
    panel.appendChild(pwrRow);
  } else {
    // AC30 Top Boost
    const isTopBoost = inputJack !== 'normal';
    const chanSel = document.createElement('div');
    chanSel.style.cssText = 'display:flex;gap:4px;align-items:center;flex:none;';
    const tbBtn = document.createElement('button');
    tbBtn.type = 'button';
    tbBtn.className = 'm-chip';
    tbBtn.dataset.controlId = 'amp.input_jack';
    tbBtn.dataset.jackId = 'vibrato';
    tbBtn.setAttribute('aria-pressed', isTopBoost ? 'true' : 'false');
    tbBtn.textContent = 'TOP BOOST';
    // O motor expõe o canal Top Boost do AC30 como `vibrato` (vocabulário único do DSP, Spec 051 CA-4).
    tbBtn.addEventListener('click', () => void store.setControl('amp.input_jack', 'vibrato'));

    const normBtn = document.createElement('button');
    normBtn.type = 'button';
    normBtn.className = 'm-chip';
    normBtn.dataset.controlId = 'amp.input_jack';
    normBtn.dataset.jackId = 'normal';
    normBtn.setAttribute('aria-pressed', !isTopBoost ? 'true' : 'false');
    normBtn.textContent = 'NORMAL';
    normBtn.addEventListener('click', () => void store.setControl('amp.input_jack', 'normal'));
    chanSel.append(tbBtn, normBtn);
    panel.appendChild(chanSel);

    pushKnob('amp.volume_2', 'amp_1.volume_2', 'NORMAL VOL', 4.5);
    pushKnob('amp.gain', 'amp_1.gain', 'BRILL VOL', 6.0);
    pushKnob('amp.treble', 'amp_1.treble', 'TREBLE', 6.5);
    pushKnob('amp.bass', 'amp_1.bass', 'BASS', 5.0);
    pushKnob('amp.cut', 'amp_1.cut', 'TONE CUT', 4.0);
    pushKnob('amp.vibrato_speed', 'amp_1.vibrato_speed', 'VIB SPEED', 4.0);
    pushKnob('amp.vibrato_depth', 'amp_1.vibrato_depth', 'VIB DEPTH', 3.5);
    pushKnob('amp.master', 'amp_1.master', 'MASTER', 7.0);

    panel.appendChild(createJewelLight(isLit, 'Pilot Lamp AC30'));
    panel.appendChild(
      createRockerSwitch('amp.power', 'POWER', power, '#ff3a28', () => {
        void store.setControl('amp.power', !power);
      })
    );
    panel.appendChild(
      createRockerSwitch('amp.standby', 'STANDBY', standby, '#ffb020', () => {
        void store.setControl('amp.standby', !standby);
      })
    );
  }

  head.append(grille, panel);
  return head;
}
