export type ModifierSourceType =
  | 'EXP_PEDAL_1'
  | 'EXP_PEDAL_2'
  | 'MIDI_CC_1'
  | 'MIDI_CC_7'
  | 'MIDI_CC_11'
  | 'INTERNAL_LFO_1'
  | 'INTERNAL_LFO_2'
  | 'ENVELOPE_FOLLOWER'
  | 'SCENE_CONTROLLER';

export interface ModifierBindingView {
  id: string;
  target_param: string;
  source: string;
  min_val: number;
  max_val: number;
  curve_slope: number;
  damping_ms: number;
}

export interface ModifierMatrixOptions {
  targetParam: string;
  label: string;
  paramMin: number;
  paramMax: number;
  existing?: ModifierBindingView | null;
  onSave: (binding: ModifierBindingView) => void;
  onRemove?: (id: string) => void;
}

const MODIFIER_SOURCES: Array<{ id: string; label: string }> = [
  { id: 'EXP_PEDAL_1', label: 'Pedal de Expressão 1 (EXP_PEDAL_1)' },
  { id: 'EXP_PEDAL_2', label: 'Pedal de Expressão 2 (EXP_PEDAL_2)' },
  { id: 'MIDI_CC_1', label: 'MIDI CC 1 (MIDI_CC_n)' },
  { id: 'MIDI_CC_7', label: 'MIDI CC 7 (Volume)' },
  { id: 'MIDI_CC_11', label: 'MIDI CC 11 (Expression)' },
  { id: 'INTERNAL_LFO_1', label: 'LFO Interno 1 (1 Hz)' },
  { id: 'INTERNAL_LFO_2', label: 'LFO Interno 2 (0.5 Hz)' },
  { id: 'ENVELOPE_FOLLOWER', label: 'Envelope Follower' },
  { id: 'SCENE_CONTROLLER', label: 'Scene Controller' },
];

export function openModifierMatrixModal(opts: ModifierMatrixOptions): HTMLElement {
  const existingModal = document.getElementById('sonix-modifier-matrix-modal');
  if (existingModal) existingModal.remove();

  const overlay = document.createElement('div');
  overlay.id = 'sonix-modifier-matrix-modal';
  overlay.style.cssText =
    'position:fixed;inset:0;z-index:9999;background:rgba(0,0,0,0.72);display:flex;align-items:center;justify-content:center;';

  const card = document.createElement('div');
  card.style.cssText =
    'width:380px;background:#16191f;border:1px solid #2d3442;border-radius:10px;padding:16px;color:#e6ebf2;display:flex;flex-direction:column;gap:10px;';

  const hdr = document.createElement('div');
  hdr.style.cssText = 'display:flex;justify-content:space-between;align-items:center;';
  hdr.innerHTML = `<strong style="font-size:13px;">Modifier Matrix — ${opts.label}</strong><span style="font-size:11px;color:#9aa3ad;">${opts.targetParam}</span>`;

  const srcSelect = document.createElement('select');
  srcSelect.style.cssText = 'background:#0f1216;color:#e6ebf2;border:1px solid #2d3442;border-radius:6px;padding:6px;';
  for (const s of MODIFIER_SOURCES) {
    const opt = document.createElement('option');
    opt.value = s.id;
    opt.textContent = s.label;
    if (opts.existing?.source === s.id) opt.selected = true;
    srcSelect.appendChild(opt);
  }

  const minInp = document.createElement('input');
  minInp.type = 'number';
  minInp.step = '0.1';
  minInp.value = String(opts.existing?.min_val ?? opts.paramMin);
  minInp.style.cssText = 'background:#0f1216;color:#e6ebf2;border:1px solid #2d3442;border-radius:6px;padding:6px;';

  const maxInp = document.createElement('input');
  maxInp.type = 'number';
  maxInp.step = '0.1';
  maxInp.value = String(opts.existing?.max_val ?? opts.paramMax);
  maxInp.style.cssText = 'background:#0f1216;color:#e6ebf2;border:1px solid #2d3442;border-radius:6px;padding:6px;';

  const slopeInp = document.createElement('input');
  slopeInp.type = 'number';
  slopeInp.step = '0.05';
  slopeInp.min = '0.05';
  slopeInp.max = '0.95';
  slopeInp.value = String(opts.existing?.curve_slope ?? 0.5);
  slopeInp.style.cssText = 'background:#0f1216;color:#e6ebf2;border:1px solid #2d3442;border-radius:6px;padding:6px;';

  const dampInp = document.createElement('input');
  dampInp.type = 'number';
  dampInp.step = '5';
  dampInp.min = '0';
  dampInp.max = '1000';
  dampInp.value = String(opts.existing?.damping_ms ?? 20);
  dampInp.style.cssText = 'background:#0f1216;color:#e6ebf2;border:1px solid #2d3442;border-radius:6px;padding:6px;';

  const actions = document.createElement('div');
  actions.style.cssText = 'display:flex;gap:8px;justify-content:flex-end;margin-top:6px;';

  const saveBtn = document.createElement('button');
  saveBtn.type = 'button';
  saveBtn.textContent = 'Salvar Vínculo';
  saveBtn.style.cssText = 'background:#2e89ff;color:#fff;border:none;border-radius:6px;padding:6px 12px;cursor:pointer;';
  saveBtn.addEventListener('click', () => {
    opts.onSave({
      id: opts.existing?.id ?? `mod_${opts.targetParam}_${Date.now()}`,
      target_param: opts.targetParam,
      source: srcSelect.value,
      min_val: Number(minInp.value),
      max_val: Number(maxInp.value),
      curve_slope: Number(slopeInp.value),
      damping_ms: Number(dampInp.value),
    });
    overlay.remove();
  });

  const cancelBtn = document.createElement('button');
  cancelBtn.type = 'button';
  cancelBtn.textContent = 'Fechar';
  cancelBtn.style.cssText = 'background:#252a34;color:#e6ebf2;border:none;border-radius:6px;padding:6px 12px;cursor:pointer;';
  cancelBtn.addEventListener('click', () => overlay.remove());

  if (opts.existing && opts.onRemove) {
    const rmBtn = document.createElement('button');
    rmBtn.type = 'button';
    rmBtn.textContent = 'Remover';
    rmBtn.style.cssText = 'background:#a62b2b;color:#fff;border:none;border-radius:6px;padding:6px 12px;cursor:pointer;margin-right:auto;';
    rmBtn.addEventListener('click', () => {
      if (opts.existing) opts.onRemove?.(opts.existing.id);
      overlay.remove();
    });
    actions.appendChild(rmBtn);
  }

  actions.append(cancelBtn, saveBtn);
  card.append(hdr, srcSelect, minInp, maxInp, slopeInp, dampInp, actions);
  overlay.appendChild(card);
  document.body?.appendChild(overlay);
  return overlay;
}
