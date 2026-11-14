export interface TubeVisualSpec {
  id: string;
  typeLabel?: string;
  width: number;
  height: number;
  radius: string;
  baseWidth: number;
  baseHeight: number;
  baseBg: string;
  glowOp: number;
  glowPx: number;
  selected?: boolean;
  onClick?: () => void;
  controlId?: string;
}

export function createTubeElement(spec: TubeVisualSpec): HTMLElement {
  const wrap = document.createElement(spec.onClick ? 'button' : 'div');
  if (spec.onClick) {
    wrap.className = 'tubebtn';
    wrap.style.cssText =
      'background:transparent;border:0;padding:4px 4px 0;border-radius:10px;display:flex;flex-direction:column;align-items:center;color:inherit';
    if (spec.controlId) {
      wrap.setAttribute('data-control-id', spec.controlId);
    }
    wrap.setAttribute('aria-pressed', String(Boolean(spec.selected)));
    wrap.setAttribute('aria-label', `${spec.id} ${spec.typeLabel ?? ''}`.trim());
    wrap.addEventListener('click', spec.onClick);
  } else {
    wrap.style.cssText = 'display:flex;flex-direction:column;align-items:center;flex:none';
  }

  const glass = document.createElement('div');
  glass.className = 'm-glass';
  glass.style.cssText = `width:${spec.width}px;height:${spec.height}px;border-radius:${spec.radius};box-shadow:0 0 18px rgba(255,100,20,${(spec.glowOp * 0.45).toFixed(2)}),0 0 0 2px ${spec.selected ? '#3dd8ff' : 'transparent'}`;

  const getter = document.createElement('span');
  getter.className = 'm-getter';
  const plate = document.createElement('span');
  plate.className = 'm-plate';
  const glow = document.createElement('span');
  glow.className = 'm-glow';
  glow.style.opacity = spec.glowOp.toFixed(2);
  glow.style.filter = `drop-shadow(0 0 ${spec.glowPx.toFixed(1)}px rgba(255,140,40,.9))`;

  glass.append(getter, plate, glow);

  const base = document.createElement('div');
  base.style.cssText = `width:${spec.baseWidth}px;height:${spec.baseHeight}px;border-radius:0 0 4px 4px;background:${spec.baseBg};box-shadow:inset 0 1px 0 rgba(255,255,255,.15)`;

  const lbl = document.createElement('span');
  lbl.className = 'm-mono';
  lbl.textContent = spec.id;
  lbl.style.cssText = `font-size:${spec.onClick ? 10 : 8}px;margin-top:2px;font-weight:700;color:${spec.selected ? '#3dd8ff' : '#a3abb4'}`;

  wrap.append(glass, base, lbl);
  return wrap;
}
