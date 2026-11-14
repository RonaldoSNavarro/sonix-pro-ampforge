/* ============================================================================
   SONIX PRO // AMPFORGE™ — Interactive Prototype & White-Box DSP / Design System
   app.js v1.0.0
   ============================================================================ */

const state = {
  activeScene: 2, // 0..7 (A..H)
  selectedBlockId: 'blk_amp_1',
  selectedChannel: 'C',
  oversampling: '4x',
  chassisOpen: false,
  hardwareShellMode: false,
  tunerVisible: false,
  tunerOutputMode: 'mute',
  calibRunning: false,
  calibTimer: null,
  testRiffPlaying: false,
  scopeMode: 'waveform', // 'waveform' | 'transfer' | 'tonestack'

  // Front Panel Amp Parameters (0.0 to 10.0)
  ampParams: {
    inputGain: 7.4,
    overdrive: 6.8,
    bass: 6.2,
    middle: 7.8,
    treble: 6.5,
    presence: 5.9,
    depth: 6.4,
    masterVol: 4.5,
    brightSwitch: true,
  },

  // Sincronização e parâmetros completos do motor DSP (Specs 014, 022, 023, 024)
  params: {
    'amp_1.gain': 7.4,
    'drive.enabled': false,
    'drive.drive': 5.0,
    'drive.tone': 5.0,
    'drive.level': 7.0,
    'amp_1.bright': true,
    'cab_1.ir_enabled': false,
    'mod_2.enabled': false,
    'dly_1.enabled': false,
    'rev_1.enabled': false,
  },

  // Open Chassis White-Box Circuit Parameters
  circuitParams: {
    biasMa: 38.5,
    sagDb: 3.4,
    nfb: 4.8,
    xformerSat: 5.2,
    plateVolts: 295,
    triodeAsym: 0.24,
    cabResHz: 98,
    cabHiCutKhz: 8.2,
  },

  // Signal Grid Blocks (4 rows x 8 visible slots per row)
  blocks: [
    { id: 'blk_gate', row: 0, col: 0, cat: 'GATE', name: 'Natural Gate', color: '#00e676', ch: 'A', bypassed: false, cpu: 1.8 },
    { id: 'blk_drive', row: 0, col: 1, cat: 'DRIVE', name: 'Screamer 808', color: '#ff9100', ch: 'B', bypassed: false, cpu: 4.6 },
    { id: 'blk_split', row: 0, col: 2, cat: 'SPLIT', name: 'A/B Crossover', color: '#00e5ff', ch: 'A', bypassed: false, cpu: 0.5 },
    { id: 'blk_amp_1', row: 0, col: 3, cat: 'AMP', name: 'Brit 800 2203', color: '#ff3d00', ch: 'C', bypassed: false, cpu: 11.4 },
    { id: 'blk_cab_1', row: 0, col: 4, cat: 'CAB', name: '4x12 Greenback', color: '#ffc400', ch: 'A', bypassed: false, cpu: 5.2 },
    { id: 'blk_eq_1', row: 0, col: 5, cat: 'EQ', name: '8-Band Studio', color: '#1de9b6', ch: 'A', bypassed: false, cpu: 2.1 },
    { id: 'blk_dly_1', row: 0, col: 6, cat: 'DELAY', name: 'Tape Echo 201', color: '#00e5ff', ch: 'B', bypassed: false, cpu: 4.3 },
    { id: 'blk_rev_1', row: 0, col: 7, cat: 'REVERB', name: 'Vintage Plate', color: '#651fff', ch: 'A', bypassed: false, cpu: 4.3 },

    // Parallel Row 2 (Stereo Dual-Amp / Parallel Clean Blend)
    { id: 'blk_comp_2', row: 1, col: 2, cat: 'COMP', name: 'FET 1176 Comp', color: '#00b0ff', ch: 'A', bypassed: false, cpu: 2.2 },
    { id: 'blk_amp_2', row: 1, col: 3, cat: 'AMP', name: 'US Twin Clean', color: '#ff3d00', ch: 'A', bypassed: true, cpu: 9.8 },
    { id: 'blk_cab_2', row: 1, col: 4, cat: 'CAB', name: '2x12 Alnico', color: '#ffc400', ch: 'A', bypassed: true, cpu: 4.5 },
    { id: 'blk_mod_2', row: 1, col: 5, cat: 'MOD', name: 'Tri-Chorus', color: '#7c4dff', ch: 'A', bypassed: false, cpu: 3.1 },
  ],

  // 8 Scenes Preset Snapshot Configurations
  scenes: [
    { id: 'A', name: 'CLEAN SPARKLE', ampCh: 'A', gain: 3.2, od: 2.0, driveBypass: true, dlyBypass: false, faceplate: 'chrome-theme', logo: 'US Twin Clean' },
    { id: 'B', name: 'PLEXI CRUNCH', ampCh: 'B', gain: 5.8, od: 4.5, driveBypass: true, dlyBypass: true, faceplate: '', logo: '1959 Super Plexi' },
    { id: 'C', name: 'MODDED 800', ampCh: 'C', gain: 7.4, od: 6.8, driveBypass: false, dlyBypass: false, faceplate: '', logo: 'Britannia 800' },
    { id: 'D', name: 'SOLO LEAD +3dB', ampCh: 'D', gain: 8.8, od: 8.2, driveBypass: false, dlyBypass: false, faceplate: 'blackout-theme', logo: 'Britannia Custom' },
    { id: 'E', name: 'AMBIENT CLOUD', ampCh: 'A', gain: 3.8, od: 2.5, driveBypass: true, dlyBypass: false, faceplate: 'chrome-theme', logo: 'US Twin Clean' },
    { id: 'F', name: 'HEAVY RHYTHM', ampCh: 'C', gain: 8.1, od: 7.5, driveBypass: false, dlyBypass: true, faceplate: 'blackout-theme', logo: 'Cali Recto Rev-F' },
    { id: 'G', name: 'VINTAGE BLUES', ampCh: 'B', gain: 5.1, od: 5.0, driveBypass: false, dlyBypass: true, faceplate: '', logo: '1959 Super Plexi' },
    { id: 'H', name: 'DRY REAMP DI', ampCh: 'A', gain: 5.0, od: 0.0, driveBypass: true, dlyBypass: true, faceplate: 'chrome-theme', logo: 'Direct Bypass' },
  ],
};

const KNOB_DEFINITIONS = [
  { key: 'inputGain', param: 'amp_1.gain', label: 'INPUT GAIN', min: 0, max: 10, defaultVal: 7.4, hasModifier: true },
  { key: 'overdrive', param: 'amp_1.overdrive', label: 'OVERDRIVE', min: 0, max: 10, defaultVal: 6.8, hasModifier: false },
  { key: 'bass', param: 'amp_1.bass', label: 'BASS', min: 0, max: 10, defaultVal: 6.2, hasModifier: false },
  { key: 'middle', param: 'amp_1.middle', label: 'MIDDLE', min: 0, max: 10, defaultVal: 7.8, hasModifier: false },
  { key: 'treble', param: 'amp_1.treble', label: 'TREBLE', min: 0, max: 10, defaultVal: 6.5, hasModifier: false },
  { key: 'presence', param: 'amp_1.presence', label: 'PRESENCE', min: 0, max: 10, defaultVal: 5.9, hasModifier: false },
  { key: 'depth', param: 'amp_1.depth', label: 'DEPTH / RES', min: 0, max: 10, defaultVal: 6.4, hasModifier: false },
  { key: 'masterVol', param: 'amp_1.master', label: 'MASTER VOL', min: 0, max: 10, defaultVal: 4.5, hasModifier: false },
];

const AMP_MODEL_KNOBS = {
  jcm800_2203: [
    { key: 'inputGain', param: 'amp_1.gain', label: 'PREAMP GAIN', min: 0, max: 10, defaultVal: 7.4, hasModifier: true },
    { key: 'overdrive', param: 'amp_1.overdrive', label: 'MASTER BOOST', min: 0, max: 10, defaultVal: 6.8, hasModifier: false },
    { key: 'bass', param: 'amp_1.bass', label: 'BASS', min: 0, max: 10, defaultVal: 6.2, hasModifier: false },
    { key: 'middle', param: 'amp_1.middle', label: 'MIDDLE', min: 0, max: 10, defaultVal: 7.8, hasModifier: false },
    { key: 'treble', param: 'amp_1.treble', label: 'TREBLE', min: 0, max: 10, defaultVal: 6.5, hasModifier: false },
    { key: 'presence', param: 'amp_1.presence', label: 'PRESENCE', min: 0, max: 10, defaultVal: 5.9, hasModifier: false },
    { key: 'depth', param: 'amp_1.depth', label: 'RESONANCE', min: 0, max: 10, defaultVal: 6.4, hasModifier: false },
    { key: 'masterVol', param: 'amp_1.master', label: 'MASTER VOL', min: 0, max: 10, defaultVal: 4.5, hasModifier: false },
  ],
  plexi_1959: [
    { key: 'inputGain', param: 'amp_1.gain', label: 'VOL 1 (HIGH)', min: 0, max: 10, defaultVal: 7.0, hasModifier: true },
    { key: 'volume_2', param: 'amp_1.volume_2', label: 'VOL 2 (NORM)', min: 0, max: 10, defaultVal: 5.0, hasModifier: false },
    { key: 'bass', param: 'amp_1.bass', label: 'BASS', min: 0, max: 10, defaultVal: 6.0, hasModifier: false },
    { key: 'middle', param: 'amp_1.middle', label: 'MIDDLE', min: 0, max: 10, defaultVal: 7.0, hasModifier: false },
    { key: 'treble', param: 'amp_1.treble', label: 'TREBLE', min: 0, max: 10, defaultVal: 6.5, hasModifier: false },
    { key: 'presence', param: 'amp_1.presence', label: 'PRESENCE', min: 0, max: 10, defaultVal: 6.0, hasModifier: false },
    { key: 'depth', param: 'amp_1.depth', label: 'DEPTH / RES', min: 0, max: 10, defaultVal: 5.0, hasModifier: false },
  ],
  ac30_tb: [
    { key: 'inputGain', param: 'amp_1.gain', label: 'BRILLIANT VOL', min: 0, max: 10, defaultVal: 6.5, hasModifier: true },
    { key: 'treble', param: 'amp_1.treble', label: 'TREBLE', min: 0, max: 10, defaultVal: 6.0, hasModifier: false },
    { key: 'bass', param: 'amp_1.bass', label: 'BASS', min: 0, max: 10, defaultVal: 5.5, hasModifier: false },
    { key: 'cut', param: 'amp_1.cut', label: 'TONE CUT', min: 0, max: 10, defaultVal: 4.5, hasModifier: false },
    { key: 'presence', param: 'amp_1.presence', label: 'TOP BOOST', min: 0, max: 10, defaultVal: 6.0, hasModifier: false },
    { key: 'masterVol', param: 'amp_1.master', label: 'MASTER VOL', min: 0, max: 10, defaultVal: 7.0, hasModifier: false },
  ],
  twin_ab763: [
    { key: 'inputGain', param: 'amp_1.gain', label: 'VOLUME', min: 0, max: 10, defaultVal: 5.0, hasModifier: true },
    { key: 'treble', param: 'amp_1.treble', label: 'TREBLE', min: 0, max: 10, defaultVal: 6.0, hasModifier: false },
    { key: 'middle', param: 'amp_1.middle', label: 'MIDDLE', min: 0, max: 10, defaultVal: 6.5, hasModifier: false },
    { key: 'bass', param: 'amp_1.bass', label: 'BASS', min: 0, max: 10, defaultVal: 5.5, hasModifier: false },
    { key: 'presence', param: 'amp_1.presence', label: 'PRESENCE', min: 0, max: 10, defaultVal: 5.0, hasModifier: false },
    { key: 'depth', param: 'amp_1.depth', label: 'RESONANCE', min: 0, max: 10, defaultVal: 4.0, hasModifier: false },
  ],
  dual_rect: [
    { key: 'inputGain', param: 'amp_1.gain', label: 'GAIN', min: 0, max: 10, defaultVal: 8.0, hasModifier: true },
    { key: 'overdrive', param: 'amp_1.overdrive', label: 'LEAD DRIVE', min: 0, max: 10, defaultVal: 7.5, hasModifier: false },
    { key: 'bass', param: 'amp_1.bass', label: 'BASS', min: 0, max: 10, defaultVal: 7.0, hasModifier: false },
    { key: 'middle', param: 'amp_1.middle', label: 'MIDDLE', min: 0, max: 10, defaultVal: 4.5, hasModifier: false },
    { key: 'treble', param: 'amp_1.treble', label: 'TREBLE', min: 0, max: 10, defaultVal: 6.8, hasModifier: false },
    { key: 'presence', param: 'amp_1.presence', label: 'PRESENCE', min: 0, max: 10, defaultVal: 6.0, hasModifier: false },
    { key: 'depth', param: 'amp_1.depth', label: 'RESONANCE', min: 0, max: 10, defaultVal: 7.2, hasModifier: false },
    { key: 'masterVol', param: 'amp_1.master', label: 'MASTER VOL', min: 0, max: 10, defaultVal: 5.0, hasModifier: false },
  ],
};

function getActiveKnobDefinitions() {
  const model = state.currentAmpModel || document.getElementById('ampModelSelect')?.value || 'jcm800_2203';
  return AMP_MODEL_KNOBS[model] || KNOB_DEFINITIONS;
}

const KNOB_PARAM_MAP = {
  inputGain: 'amp_1.gain',
  overdrive: 'amp_1.overdrive',
  bass: 'amp_1.bass',
  middle: 'amp_1.middle',
  treble: 'amp_1.treble',
  presence: 'amp_1.presence',
  depth: 'amp_1.depth',
  masterVol: 'amp_1.master',
  volume_2: 'amp_1.volume_2',
  cut: 'amp_1.cut',
};

function setApiError(isError, msg = 'OFFLINE') {
  const badge = document.getElementById('engineLinkBadge');
  if (badge) {
    if (isError) {
      badge.textContent = msg;
      badge.style.color = '#ff3d00';
    } else {
      badge.textContent = 'ONLINE';
      badge.style.color = '#00e5ff';
    }
  }
}

let pendingParams = {};
let syncTimeout = null;
function syncParamsToBackend(params, immediate = false) {
  if (!state.params) state.params = {};
  Object.assign(state.params, params);
  Object.assign(pendingParams, params);
  if (immediate) {
    if (syncTimeout) { clearTimeout(syncTimeout); syncTimeout = null; }
    flushParams();
  } else if (!syncTimeout) {
    syncTimeout = setTimeout(flushParams, 25);
  }
}
async function flushParams() {
  if (Object.keys(pendingParams).length === 0) return;
  const payload = { ...pendingParams };
  pendingParams = {};
  syncTimeout = null;
  const token = window.__SONIX_TOKEN__ || document.querySelector('meta[name="sonix-token"]')?.content || '';
  try {
    const res = await fetch('/api/params', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Sonix-Token': token,
      },
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      setApiError(true, `HTTP ${res.status}`);
    } else {
      setApiError(false);
    }
  } catch (_) {
    setApiError(true, 'OFFLINE');
  }
}

async function loadInitialState() {
  try {
    const res = await fetch('/api/state');
    if (res.ok) {
      const st = await res.json();
      if (!state.params) state.params = {};
      Object.assign(state.params, st);

      // Atualiza textos de nível, delay e ressonância lidos de /api/state (Spec 022, CA-7)
      const mic1Lvl = document.getElementById('mic1LevelText');
      if (mic1Lvl && st['cab_1.mic1.level_db'] !== undefined) {
        const lvl = st['cab_1.mic1.level_db'];
        mic1Lvl.textContent = `Nível: ${lvl >= 0 ? '+' : ''}${Number(lvl).toFixed(1)} dB | Fase: 0° Normal`;
      }
      const cabRes = document.getElementById('valCabRes');
      if (cabRes && (st['cab_1.resonance_hz'] !== undefined || st['cabResHz'] !== undefined)) {
        cabRes.textContent = `${st['cab_1.resonance_hz'] || st['cabResHz']} Hz`;
      }
      if (st['cab_1.ir_enabled']) {
        document.querySelectorAll('.mic-control').forEach(el => {
          el.disabled = true;
          el.classList.add('disabled');
        });
      }
    }
  } catch (_) {}
}

function setExpressionPedal(normVal) {
  // Mapeia ganho na faixa 3.0..9.5 (Spec 014, CA-1)
  const gain = 3.0 + normVal * (9.5 - 3.0);
  syncParamsToBackend({ 'amp_1.gain': gain }, false);
}

// ============================================================================
// 1. RENDER SIGNAL ROUTING MATRIX GRID (4 ROWS x 8 SLOTS)
// ============================================================================
function renderSignalGrid() {
  const container = document.getElementById('gridMatrixContainer');
  container.innerHTML = '';

  const rowInputs = ['IN 1 (INST)', 'SPLIT PATH B', 'AUX / LOOP', 'USB REAMP'];
  const rowOutputs = ['OUT 1/2 XLR', 'MIX MERGE →', 'FX SEND', 'USB REC'];

  for (let r = 0; r < 4; r++) {
    const rowEl = document.createElement('div');
    rowEl.className = `grid-row ${r <= 1 ? 'active-signal-path' : ''}`;

    const splitBlock = state.blocks.find((b) => b.id === 'blk_split');
    const isSplitDisabled = splitBlock ? splitBlock.bypassed : false;
    if (r === 1 && isSplitDisabled) {
      rowEl.classList.add('split-disabled');
      rowEl.style.opacity = '0.35';
    }

    const inTerm = document.createElement('div');
    inTerm.className = `io-terminal ${r <= 1 ? 'active-io' : ''}`;
    inTerm.textContent = rowInputs[r];
    rowEl.appendChild(inTerm);

    for (let c = 0; c < 8; c++) {
      const slotEl = document.createElement('div');
      const block = state.blocks.find((b) => b.row === r && b.col === c);

      if (block) {
        slotEl.className = `grid-slot populated ${state.selectedBlockId === block.id ? 'selected' : ''} ${block.bypassed ? 'bypassed' : ''}`;
        slotEl.style.setProperty('--slot-color', block.color);
        slotEl.style.borderColor = block.color;
        slotEl.title = 'Clique para inspecionar | Duplo clique para Ativar/Bypass';

        slotEl.innerHTML = `
          <div class="slot-left">
            <span class="slot-led"></span>
            <div class="slot-info">
              <span class="slot-cat">${block.cat}</span>
              <span class="slot-name">${block.name}</span>
            </div>
          </div>
          <span class="slot-ch-badge">${block.ch}</span>
        `;

        slotEl.addEventListener('click', () => {
          state.selectedBlockId = block.id;
          state.selectedChannel = block.ch;
          updateSelectedBlockHeader(block);
          renderSignalGrid();
          renderFootswitches();
          openBlockEditor(block);

          // Switch view automatically if user clicks CAB or AMP block
          if (block.cat === 'CAB' || block.id === 'blk_cab_1' || block.id === 'blk_cab_2') {
            switchMainWorkspaceView('cab-room');
          } else if (block.cat === 'AMP' || block.id === 'blk_amp_1' || block.id === 'blk_amp_2') {
            switchMainWorkspaceView('amp');
          }
        });

        slotEl.addEventListener('dblclick', (e) => {
          e.stopPropagation();
          block.bypassed = !block.bypassed;
          updateCpuBudget();
          renderSignalGrid();
          renderFootswitches();
          if (block.id === 'blk_drive' || block.cat === 'DRIVE' || block.category === 'DRIVE' || (block.id && block.id.includes('drive'))) {
            syncParamsToBackend({ 'drive.enabled': !block.bypassed }, false);
          }
        });
      } else {
        slotEl.className = 'grid-slot';
        slotEl.innerHTML = `<span class="shunt-node">${r <= 1 ? '── SHUNT ──' : '· · ·'}</span>`;
      }

      rowEl.appendChild(slotEl);
    }

    const outTerm = document.createElement('div');
    outTerm.className = `io-terminal ${r <= 1 ? 'active-io' : ''}`;
    outTerm.textContent = rowOutputs[r];
    rowEl.appendChild(outTerm);

    container.appendChild(rowEl);
  }

  // Aviso de sinal direto no caminho B quando sem amp nem cab ativos (Spec 024, CA-11)
  const splitBlock = state.blocks.find((b) => b.id === 'blk_split');
  const isSplitActive = splitBlock ? !splitBlock.bypassed : false;
  const amp2Block = state.blocks.find((b) => b.id === 'blk_amp_2');
  const cab2Block = state.blocks.find((b) => b.id === 'blk_cab_2');
  const isDirectB = isSplitActive && (!amp2Block || amp2Block.bypassed) && (!cab2Block || cab2Block.bypassed);

  let directWarn = document.getElementById('gridDirectWarning');
  if (!directWarn) {
    directWarn = document.createElement('div');
    directWarn.id = 'gridDirectWarning';
    directWarn.className = 'grid-direct-warning';
    container.parentNode.appendChild(directWarn);
  }
  directWarn.textContent = 'caminho B sem amp/cab: sinal direto';
  directWarn.style.display = isDirectB ? 'block' : 'none';
}

function openBlockEditor(block) {
  const panel = document.getElementById('blockEditor');
  const title = document.getElementById('blockEditorTitle');
  const badge = document.getElementById('blockEditorStatus');
  const body = document.getElementById('blockEditorBody');
  if (!panel || !body) return;

  panel.style.display = 'block';

  if (!block || !block.id) {
    if (title) title.textContent = 'BLOCO SELECIONADO';
    if (badge) badge.textContent = 'DESCONHECIDO';
    body.innerHTML = '<div class="block-editor-empty">sem controles nesta versão</div>';
    return;
  }

  if (block.id === 'blk_drive' || block.cat === 'DRIVE' || block.category === 'DRIVE' || (block.id && block.id.includes('drive'))) {
    if (title) title.textContent = `PEDAL: ${block.name.toUpperCase()} (DRIVE)`;
    if (badge) badge.textContent = 'INSPIRADO NO CIRCUITO 808';

    const driveVal = state.params['drive.drive'] !== undefined ? state.params['drive.drive'] : 5.0;
    const toneVal = state.params['drive.tone'] !== undefined ? state.params['drive.tone'] : 5.0;
    const levelVal = state.params['drive.level'] !== undefined ? state.params['drive.level'] : 7.0;

    body.innerHTML = `
      <div class="pedal-controls-grid">
        <div class="pedal-knob-box">
          <label>DRIVE</label>
          <input type="range" id="pedalDriveSlider" min="0" max="10" step="0.1" value="${driveVal}">
          <span class="pedal-val" id="pedalDriveVal">${Number(driveVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>TONE</label>
          <input type="range" id="pedalToneSlider" min="0" max="10" step="0.1" value="${toneVal}">
          <span class="pedal-val" id="pedalToneVal">${Number(toneVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>LEVEL</label>
          <input type="range" id="pedalLevelSlider" min="0" max="10" step="0.1" value="${levelVal}">
          <span class="pedal-val" id="pedalLevelVal">${Number(levelVal).toFixed(1)}</span>
        </div>
      </div>
    `;

    document.getElementById('pedalDriveSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('pedalDriveVal').textContent = v.toFixed(1);
      state.params['drive.drive'] = v;
      syncParamsToBackend({ 'drive.drive': v }, false);
    });
    document.getElementById('pedalToneSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('pedalToneVal').textContent = v.toFixed(1);
      state.params['drive.tone'] = v;
      syncParamsToBackend({ 'drive.tone': v }, false);
    });
    document.getElementById('pedalLevelSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('pedalLevelVal').textContent = v.toFixed(1);
      state.params['drive.level'] = v;
      syncParamsToBackend({ 'drive.level': v }, false);
    });
  } else if (block.id === 'blk_gate' || block.cat === 'GATE') {
    if (title) title.textContent = `NOISE GATE: ${block.name.toUpperCase()}`;
    if (badge) badge.textContent = 'DYNAMICS // HYSTERESIS + KEY FILTER';

    const thresh = state.params['gate.threshold_db'] !== undefined ? state.params['gate.threshold_db'] : -65.0;
    const enabled = state.params['gate.enabled'] !== undefined ? state.params['gate.enabled'] : !block.bypassed;
    const mode = state.params['gate.mode'] || 'natural';
    const hyst = state.params['gate.hysteresis_db'] !== undefined ? state.params['gate.hysteresis_db'] : 4.0;
    const att = state.params['gate.attack_ms'] !== undefined ? state.params['gate.attack_ms'] : 1.0;
    const hold = state.params['gate.hold_ms'] !== undefined ? state.params['gate.hold_ms'] : 35.0;
    const rel = state.params['gate.release_ms'] !== undefined ? state.params['gate.release_ms'] : 80.0;
    const range = state.params['gate.range_db'] !== undefined ? state.params['gate.range_db'] : -70.0;
    const hpf = state.params['gate.key_hpf_hz'] !== undefined ? state.params['gate.key_hpf_hz'] : 80.0;
    const lpf = state.params['gate.key_lpf_hz'] !== undefined ? state.params['gate.key_lpf_hz'] : 6000.0;
    const gateStatusLabel = `${state.liveGateState || 'CLOSED'} // GR ${(state.liveGateGrDb ?? -70.0).toFixed(1)} dB`;

    body.innerHTML = `
      <div class="pedal-controls-grid">
        <div class="pedal-knob-box">
          <label>THRESHOLD</label>
          <input type="range" id="gateThreshSlider" min="-90" max="0" step="1" value="${thresh}">
          <span class="pedal-val" id="gateThreshVal">${Number(thresh).toFixed(0)} dB</span>
        </div>
        <div class="pedal-knob-box">
          <label>HYSTERESIS</label>
          <input type="range" id="gateHysteresisSlider" min="0" max="12" step="0.5" value="${hyst}">
          <span class="pedal-val" id="gateHysteresisVal">${Number(hyst).toFixed(1)} dB</span>
        </div>
        <div class="pedal-knob-box">
          <label>ATTACK</label>
          <input type="range" id="gateAttackSlider" min="0.1" max="50" step="0.1" value="${att}">
          <span class="pedal-val" id="gateAttackVal">${Number(att).toFixed(1)} ms</span>
        </div>
        <div class="pedal-knob-box">
          <label>HOLD</label>
          <input type="range" id="gateHoldSlider" min="0" max="500" step="5" value="${hold}">
          <span class="pedal-val" id="gateHoldVal">${Number(hold).toFixed(0)} ms</span>
        </div>
        <div class="pedal-knob-box">
          <label>RELEASE / DECAY</label>
          <input type="range" id="gateReleaseSlider" min="5" max="1000" step="5" value="${rel}">
          <span class="pedal-val" id="gateReleaseVal">${Number(rel).toFixed(0)} ms</span>
        </div>
        <div class="pedal-knob-box">
          <label>RANGE</label>
          <input type="range" id="gateRangeSlider" min="-90" max="0" step="1" value="${range}">
          <span class="pedal-val" id="gateRangeVal">${Number(range).toFixed(0)} dB</span>
        </div>
        <div class="pedal-knob-box">
          <label>KEY HPF</label>
          <input type="range" id="gateKeyHpfSlider" min="20" max="500" step="5" value="${hpf}">
          <span class="pedal-val" id="gateKeyHpfVal">${Number(hpf).toFixed(0)} Hz</span>
        </div>
        <div class="pedal-knob-box">
          <label>KEY LPF</label>
          <input type="range" id="gateKeyLpfSlider" min="1000" max="12000" step="100" value="${lpf}">
          <span class="pedal-val" id="gateKeyLpfVal">${Number(lpf).toFixed(0)} Hz</span>
        </div>
        <div class="pedal-knob-box">
          <label>MODE</label>
          <select id="gateModeSelect" class="hw-select" style="margin-top:6px;">
            <option value="natural" ${mode === 'natural' ? 'selected' : ''}>Natural DI Sidechain</option>
            <option value="classic" ${mode === 'classic' ? 'selected' : ''}>Classic Expander</option>
          </select>
        </div>
        <div class="pedal-knob-box">
          <label>BYPASS / TELEMETRIA</label>
          <button class="hw-btn" id="btnGateToggle" style="margin-top:6px;">${enabled ? 'ATIVO' : 'BYPASS'}</button>
          <span class="pedal-val" id="gateStatusBadge">${gateStatusLabel}</span>
        </div>
      </div>
    `;

    const bindGateSlider = (sliderId, valId, paramKey, formatFn) => {
      document.getElementById(sliderId)?.addEventListener('input', (e) => {
        const v = Number(e.target.value);
        const valEl = document.getElementById(valId);
        if (valEl) valEl.textContent = formatFn(v);
        state.params[paramKey] = v;
        syncParamsToBackend({ [paramKey]: v }, false);
      });
    };

    bindGateSlider('gateThreshSlider', 'gateThreshVal', 'gate.threshold_db', (v) => `${v.toFixed(0)} dB`);
    bindGateSlider('gateHysteresisSlider', 'gateHysteresisVal', 'gate.hysteresis_db', (v) => `${v.toFixed(1)} dB`);
    bindGateSlider('gateAttackSlider', 'gateAttackVal', 'gate.attack_ms', (v) => `${v.toFixed(1)} ms`);
    bindGateSlider('gateHoldSlider', 'gateHoldVal', 'gate.hold_ms', (v) => `${v.toFixed(0)} ms`);
    bindGateSlider('gateReleaseSlider', 'gateReleaseVal', 'gate.release_ms', (v) => `${v.toFixed(0)} ms`);
    bindGateSlider('gateRangeSlider', 'gateRangeVal', 'gate.range_db', (v) => `${v.toFixed(0)} dB`);
    bindGateSlider('gateKeyHpfSlider', 'gateKeyHpfVal', 'gate.key_hpf_hz', (v) => `${v.toFixed(0)} Hz`);
    bindGateSlider('gateKeyLpfSlider', 'gateKeyLpfVal', 'gate.key_lpf_hz', (v) => `${v.toFixed(0)} Hz`);

    document.getElementById('gateModeSelect')?.addEventListener('change', (e) => {
      const m = e.target.value === 'classic' ? 'classic' : 'natural';
      state.params['gate.mode'] = m;
      syncParamsToBackend({ 'gate.mode': m }, true);
    });
    document.getElementById('btnGateToggle')?.addEventListener('click', (e) => {
      const next = !(state.params['gate.enabled'] !== undefined ? state.params['gate.enabled'] : true);
      state.params['gate.enabled'] = next;
      e.target.textContent = next ? 'ATIVO' : 'BYPASS';
      syncParamsToBackend({ 'gate.enabled': next }, true);
    });
  } else if (block.id === 'blk_comp_2' || block.cat === 'COMP') {
    if (title) title.textContent = 'COMPRESSOR: FET 1176 (comp_2)';
    if (badge) badge.textContent = 'STUDIO LIMITER';

    const inVal = state.params['comp_2.input'] !== undefined ? state.params['comp_2.input'] : 5.0;
    const outVal = state.params['comp_2.output'] !== undefined ? state.params['comp_2.output'] : 5.0;
    const attVal = state.params['comp_2.attack'] !== undefined ? state.params['comp_2.attack'] : 4.0;
    const relVal = state.params['comp_2.release'] !== undefined ? state.params['comp_2.release'] : 4.0;
    const ratioVal = state.params['comp_2.ratio'] || '4:1';

    body.innerHTML = `
      <div class="pedal-controls-grid" style="grid-template-columns: repeat(5, 1fr);">
        <div class="pedal-knob-box">
          <label>INPUT</label>
          <input type="range" id="compInSlider" min="0" max="10" step="0.1" value="${inVal}">
          <span class="pedal-val" id="compInVal">${Number(inVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>OUTPUT</label>
          <input type="range" id="compOutSlider" min="0" max="10" step="0.1" value="${outVal}">
          <span class="pedal-val" id="compOutVal">${Number(outVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>ATTACK</label>
          <input type="range" id="compAttSlider" min="1" max="7" step="0.1" value="${attVal}">
          <span class="pedal-val" id="compAttVal">${Number(attVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>RELEASE</label>
          <input type="range" id="compRelSlider" min="1" max="7" step="0.1" value="${relVal}">
          <span class="pedal-val" id="compRelVal">${Number(relVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>RATIO</label>
          <select id="compRatioSelect" class="tech-select" style="margin-top:6px; font-size:11px;">
            <option value="4:1" ${ratioVal === '4:1' ? 'selected' : ''}>4:1</option>
            <option value="8:1" ${ratioVal === '8:1' ? 'selected' : ''}>8:1</option>
            <option value="12:1" ${ratioVal === '12:1' ? 'selected' : ''}>12:1</option>
            <option value="20:1" ${ratioVal === '20:1' ? 'selected' : ''}>20:1</option>
            <option value="all" ${ratioVal === 'all' ? 'selected' : ''}>ALL (BRITISH)</option>
          </select>
        </div>
      </div>
    `;

    document.getElementById('compInSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('compInVal').textContent = v.toFixed(1);
      state.params['comp_2.input'] = v;
      syncParamsToBackend({ 'comp_2.input': v }, false);
    });
    document.getElementById('compOutSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('compOutVal').textContent = v.toFixed(1);
      state.params['comp_2.output'] = v;
      syncParamsToBackend({ 'comp_2.output': v }, false);
    });
    document.getElementById('compAttSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('compAttVal').textContent = v.toFixed(1);
      state.params['comp_2.attack'] = v;
      syncParamsToBackend({ 'comp_2.attack': v }, false);
    });
    document.getElementById('compRelSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('compRelVal').textContent = v.toFixed(1);
      state.params['comp_2.release'] = v;
      syncParamsToBackend({ 'comp_2.release': v }, false);
    });
    document.getElementById('compRatioSelect')?.addEventListener('change', (e) => {
      state.params['comp_2.ratio'] = e.target.value;
      syncParamsToBackend({ 'comp_2.ratio': e.target.value }, true);
    });
  } else if (block.id === 'blk_eq_1' || block.cat === 'EQ') {
    if (title) title.textContent = 'EQUALIZADOR: 8-BAND (eq_1)';
    if (badge) badge.textContent = 'PARAMETRIC EQ';

    const defaultFreqs = [80, 160, 320, 640, 1250, 2500, 5000, 10000];
    let bandsHtml = '<div class="pedal-controls-grid" style="grid-template-columns: repeat(8, 1fr); gap:6px;">';
    for (let b = 1; b <= 8; b++) {
      const g = state.params[`eq_1.band${b}.gain_db`] !== undefined ? state.params[`eq_1.band${b}.gain_db`] : 0.0;
      bandsHtml += `
        <div class="pedal-knob-box" style="padding:4px;">
          <label style="font-size:9px;">B${b} (${defaultFreqs[b-1]}Hz)</label>
          <input type="range" id="eqBand${b}Slider" min="-15" max="15" step="0.5" value="${g}" style="width:100%;">
          <span class="pedal-val" id="eqBand${b}Val" style="font-size:10px;">${Number(g).toFixed(1)} dB</span>
        </div>
      `;
    }
    bandsHtml += '</div>';
    body.innerHTML = bandsHtml;

    for (let b = 1; b <= 8; b++) {
      document.getElementById(`eqBand${b}Slider`)?.addEventListener('input', (e) => {
        const v = Number(e.target.value);
        document.getElementById(`eqBand${b}Val`).textContent = `${v.toFixed(1)} dB`;
        state.params[`eq_1.band${b}.gain_db`] = v;
        syncParamsToBackend({ [`eq_1.band${b}.gain_db`]: v }, false);
      });
    }
  } else if (block.id === 'blk_mod_2' || block.cat === 'MOD') {
    if (title) title.textContent = 'MODULAÇÃO: TRI-CHORUS (mod_2)';
    if (badge) badge.textContent = 'CHORUS BBD';

    const rateVal = state.params['mod_2.rate_hz'] !== undefined ? state.params['mod_2.rate_hz'] : 1.2;
    const depthVal = state.params['mod_2.depth'] !== undefined ? state.params['mod_2.depth'] : 5.0;
    const mixVal = state.params['mod_2.mix'] !== undefined ? state.params['mod_2.mix'] : 50.0;

    body.innerHTML = `
      <div class="pedal-controls-grid">
        <div class="pedal-knob-box">
          <label>RATE (SPEED)</label>
          <input type="range" id="modRateSlider" min="0.1" max="10.0" step="0.1" value="${rateVal}">
          <span class="pedal-val" id="modRateVal">${Number(rateVal).toFixed(1)} Hz</span>
        </div>
        <div class="pedal-knob-box">
          <label>DEPTH</label>
          <input type="range" id="modDepthSlider" min="0" max="10" step="0.1" value="${depthVal}">
          <span class="pedal-val" id="modDepthVal">${Number(depthVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>MIX</label>
          <input type="range" id="modMixSlider" min="0" max="100" step="1" value="${mixVal}">
          <span class="pedal-val" id="modMixVal">${Number(mixVal).toFixed(0)}%</span>
        </div>
      </div>
    `;

    document.getElementById('modRateSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('modRateVal').textContent = `${v.toFixed(1)} Hz`;
      state.params['mod_2.rate_hz'] = v;
      syncParamsToBackend({ 'mod_2.rate_hz': v }, false);
    });
    document.getElementById('modDepthSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('modDepthVal').textContent = v.toFixed(1);
      state.params['mod_2.depth'] = v;
      syncParamsToBackend({ 'mod_2.depth': v }, false);
    });
    document.getElementById('modMixSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('modMixVal').textContent = `${v.toFixed(0)}%`;
      state.params['mod_2.mix'] = v;
      syncParamsToBackend({ 'mod_2.mix': v }, false);
    });
  } else if (block.id === 'blk_dly_1' || block.cat === 'DELAY') {
    if (title) title.textContent = 'DELAY: TAPE ECHO (dly_1)';
    if (badge) badge.textContent = 'TAPE 201';

    const repVal = state.params['dly_1.repeat_rate'] !== undefined ? state.params['dly_1.repeat_rate'] : 5.0;
    const intVal = state.params['dly_1.intensity'] !== undefined ? state.params['dly_1.intensity'] : 5.0;
    const volVal = state.params['dly_1.echo_volume'] !== undefined ? state.params['dly_1.echo_volume'] : 5.0;
    const headsVal = state.params['dly_1.heads'] || 'mode_1';
    const spillVal = state.params['dly_1.spillover'] !== undefined ? state.params['dly_1.spillover'] : true;

    body.innerHTML = `
      <div class="pedal-controls-grid" style="grid-template-columns: repeat(5, 1fr);">
        <div class="pedal-knob-box">
          <label>REPEAT RATE</label>
          <input type="range" id="dlyRateSlider" min="0" max="10" step="0.1" value="${repVal}">
          <span class="pedal-val" id="dlyRateVal">${Number(repVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>INTENSITY</label>
          <input type="range" id="dlyIntSlider" min="0" max="10" step="0.1" value="${intVal}">
          <span class="pedal-val" id="dlyIntVal">${Number(intVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>ECHO VOL</label>
          <input type="range" id="dlyVolSlider" min="0" max="10" step="0.1" value="${volVal}">
          <span class="pedal-val" id="dlyVolVal">${Number(volVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>HEADS</label>
          <select id="dlyHeadsSelect" class="tech-select" style="margin-top:6px; font-size:11px;">
            <option value="mode_1" ${headsVal === 'mode_1' ? 'selected' : ''}>Head 1 (Curto)</option>
            <option value="mode_2" ${headsVal === 'mode_2' ? 'selected' : ''}>Head 2 (Médio)</option>
            <option value="mode_3" ${headsVal === 'mode_3' ? 'selected' : ''}>Head 3 (Longo)</option>
            <option value="mode_4" ${headsVal === 'mode_4' ? 'selected' : ''}>Multi (1+2+3)</option>
          </select>
        </div>
        <div class="pedal-knob-box">
          <label>SPILLOVER</label>
          <input type="checkbox" id="dlySpillCheck" ${spillVal ? 'checked' : ''} style="margin-top:10px;">
        </div>
      </div>
    `;

    document.getElementById('dlyRateSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('dlyRateVal').textContent = v.toFixed(1);
      state.params['dly_1.repeat_rate'] = v;
      syncParamsToBackend({ 'dly_1.repeat_rate': v }, false);
    });
    document.getElementById('dlyIntSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('dlyIntVal').textContent = v.toFixed(1);
      state.params['dly_1.intensity'] = v;
      syncParamsToBackend({ 'dly_1.intensity': v }, false);
    });
    document.getElementById('dlyVolSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('dlyVolVal').textContent = v.toFixed(1);
      state.params['dly_1.echo_volume'] = v;
      syncParamsToBackend({ 'dly_1.echo_volume': v }, false);
    });
    document.getElementById('dlyHeadsSelect')?.addEventListener('change', (e) => {
      state.params['dly_1.heads'] = e.target.value;
      syncParamsToBackend({ 'dly_1.heads': e.target.value }, true);
    });
    document.getElementById('dlySpillCheck')?.addEventListener('change', (e) => {
      state.params['dly_1.spillover'] = e.target.checked;
      syncParamsToBackend({ 'dly_1.spillover': e.target.checked }, true);
    });
  } else if (block.id === 'blk_rev_1' || block.cat === 'REVERB') {
    if (title) title.textContent = 'REVERB: VINTAGE PLATE (rev_1)';
    if (badge) badge.textContent = 'STUDIO PLATE';

    const decayVal = state.params['rev_1.decay_s'] !== undefined ? state.params['rev_1.decay_s'] : 2.8;
    const preVal = state.params['rev_1.predelay_ms'] !== undefined ? state.params['rev_1.predelay_ms'] : 25.0;
    const dampVal = state.params['rev_1.damping'] !== undefined ? state.params['rev_1.damping'] : 5.0;
    const mixVal = state.params['rev_1.mix'] !== undefined ? state.params['rev_1.mix'] : 40.0;
    const spillVal = state.params['rev_1.spillover'] !== undefined ? state.params['rev_1.spillover'] : true;

    body.innerHTML = `
      <div class="pedal-controls-grid" style="grid-template-columns: repeat(5, 1fr);">
        <div class="pedal-knob-box">
          <label>DECAY</label>
          <input type="range" id="revDecaySlider" min="0.2" max="15.0" step="0.1" value="${decayVal}">
          <span class="pedal-val" id="revDecayVal">${Number(decayVal).toFixed(1)} s</span>
        </div>
        <div class="pedal-knob-box">
          <label>PRE-DELAY</label>
          <input type="range" id="revPreSlider" min="0" max="250" step="1" value="${preVal}">
          <span class="pedal-val" id="revPreVal">${Number(preVal).toFixed(0)} ms</span>
        </div>
        <div class="pedal-knob-box">
          <label>DAMPING</label>
          <input type="range" id="revDampSlider" min="0" max="10" step="0.1" value="${dampVal}">
          <span class="pedal-val" id="revDampVal">${Number(dampVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>MIX</label>
          <input type="range" id="revMixSlider" min="0" max="100" step="1" value="${mixVal}">
          <span class="pedal-val" id="revMixVal">${Number(mixVal).toFixed(0)}%</span>
        </div>
        <div class="pedal-knob-box">
          <label>SPILLOVER</label>
          <input type="checkbox" id="revSpillCheck" ${spillVal ? 'checked' : ''} style="margin-top:10px;">
        </div>
      </div>
    `;

    document.getElementById('revDecaySlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('revDecayVal').textContent = `${v.toFixed(1)} s`;
      state.params['rev_1.decay_s'] = v;
      syncParamsToBackend({ 'rev_1.decay_s': v }, false);
    });
    document.getElementById('revPreSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('revPreVal').textContent = `${v.toFixed(0)} ms`;
      state.params['rev_1.predelay_ms'] = v;
      syncParamsToBackend({ 'rev_1.predelay_ms': v }, false);
    });
    document.getElementById('revDampSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('revDampVal').textContent = v.toFixed(1);
      state.params['rev_1.damping'] = v;
      syncParamsToBackend({ 'rev_1.damping': v }, false);
    });
    document.getElementById('revMixSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('revMixVal').textContent = `${v.toFixed(0)}%`;
      state.params['rev_1.mix'] = v;
      syncParamsToBackend({ 'rev_1.mix': v }, false);
    });
    document.getElementById('revSpillCheck')?.addEventListener('change', (e) => {
      state.params['rev_1.spillover'] = e.target.checked;
      syncParamsToBackend({ 'rev_1.spillover': e.target.checked }, true);
    });
  } else if (block.id === 'blk_split' || block.cat === 'SPLIT') {
    if (title) title.textContent = 'ROUTING: SPLIT / MERGE';
    if (badge) badge.textContent = 'A/B CROSSOVER';

    const modeVal = state.params['split.mode'] || 'ab_split';
    const xoverVal = state.params['split.crossover_hz'] !== undefined ? state.params['split.crossover_hz'] : 650;
    const mergeLvl = state.params['merge.row_1_level_db'] !== undefined ? state.params['merge.row_1_level_db'] : 0.0;
    const phaseInv = state.params['merge.row_1_phase_invert'] !== undefined ? state.params['merge.row_1_phase_invert'] : false;

    body.innerHTML = `
      <div class="pedal-controls-grid" style="grid-template-columns: repeat(4, 1fr);">
        <div class="pedal-knob-box">
          <label>SPLIT MODE</label>
          <select id="splitModeSelect" class="tech-select" style="margin-top:6px; font-size:11px;">
            <option value="ab_split" ${modeVal === 'ab_split' ? 'selected' : ''}>A/B Dual Mono</option>
            <option value="crossover" ${modeVal === 'crossover' ? 'selected' : ''}>Crossover Split</option>
          </select>
        </div>
        <div class="pedal-knob-box">
          <label>CROSSOVER FREQ</label>
          <input type="range" id="splitXoverSlider" min="100" max="4000" step="50" value="${xoverVal}">
          <span class="pedal-val" id="splitXoverVal">${Number(xoverVal).toFixed(0)} Hz</span>
        </div>
        <div class="pedal-knob-box">
          <label>PATH B LEVEL</label>
          <input type="range" id="mergeLevelSlider" min="-24" max="6" step="0.5" value="${mergeLvl}">
          <span class="pedal-val" id="mergeLevelVal">${Number(mergeLvl).toFixed(1)} dB</span>
        </div>
        <div class="pedal-knob-box">
          <label>PHASE INVERT</label>
          <input type="checkbox" id="mergePhaseCheck" ${phaseInv ? 'checked' : ''} style="margin-top:10px;">
        </div>
      </div>
    `;

    document.getElementById('splitModeSelect')?.addEventListener('change', (e) => {
      state.params['split.mode'] = e.target.value;
      syncParamsToBackend({ 'split.mode': e.target.value }, true);
    });
    document.getElementById('splitXoverSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('splitXoverVal').textContent = `${v.toFixed(0)} Hz`;
      state.params['split.crossover_hz'] = v;
      syncParamsToBackend({ 'split.crossover_hz': v }, false);
    });
    document.getElementById('mergeLevelSlider')?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById('mergeLevelVal').textContent = `${v.toFixed(1)} dB`;
      state.params['merge.row_1_level_db'] = v;
      syncParamsToBackend({ 'merge.row_1_level_db': v }, false);
    });
    document.getElementById('mergePhaseCheck')?.addEventListener('change', (e) => {
      state.params['merge.row_1_phase_invert'] = e.target.checked;
      syncParamsToBackend({ 'merge.row_1_phase_invert': e.target.checked }, true);
    });
  } else if (block.cat === 'AMP' || block.id.startsWith('blk_amp')) {
    const isAmp2 = block.id === 'blk_amp_2';
    const prefix = isAmp2 ? 'amp_2' : 'amp_1';
    if (title) title.textContent = `AMPLIFICADOR: ${block.name.toUpperCase()}`;
    if (badge) badge.textContent = 'VALVULADO WHITE-BOX';

    const gainVal = isAmp2
      ? (state.params['amp_2.gain'] !== undefined ? state.params['amp_2.gain'] : 5.0)
      : (state.ampParams.inputGain || 7.4);
    const masterVal = isAmp2
      ? (state.params['amp_2.master'] !== undefined ? state.params['amp_2.master'] : 7.0)
      : (state.ampParams.masterVol || 4.5);
    const bassVal = isAmp2
      ? (state.params['amp_2.bass'] !== undefined ? state.params['amp_2.bass'] : 5.0)
      : (state.ampParams.bass || 6.2);
    const midVal = isAmp2
      ? (state.params['amp_2.middle'] !== undefined ? state.params['amp_2.middle'] : 5.0)
      : (state.ampParams.middle || 7.8);
    const trebVal = isAmp2
      ? (state.params['amp_2.treble'] !== undefined ? state.params['amp_2.treble'] : 5.0)
      : (state.ampParams.treble || 6.5);

    body.innerHTML = `
      <div class="pedal-controls-grid" style="grid-template-columns: repeat(5, 1fr);">
        <div class="pedal-knob-box">
          <label>GAIN</label>
          <input type="range" id="${prefix}EditorGain" min="0" max="10" step="0.1" value="${gainVal}">
          <span class="pedal-val" id="${prefix}EditorGainVal">${Number(gainVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>MASTER</label>
          <input type="range" id="${prefix}EditorMaster" min="0" max="10" step="0.1" value="${masterVal}">
          <span class="pedal-val" id="${prefix}EditorMasterVal">${Number(masterVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>BASS</label>
          <input type="range" id="${prefix}EditorBass" min="0" max="10" step="0.1" value="${bassVal}">
          <span class="pedal-val" id="${prefix}EditorBassVal">${Number(bassVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>MIDDLE</label>
          <input type="range" id="${prefix}EditorMid" min="0" max="10" step="0.1" value="${midVal}">
          <span class="pedal-val" id="${prefix}EditorMidVal">${Number(midVal).toFixed(1)}</span>
        </div>
        <div class="pedal-knob-box">
          <label>TREBLE</label>
          <input type="range" id="${prefix}EditorTreb" min="0" max="10" step="0.1" value="${trebVal}">
          <span class="pedal-val" id="${prefix}EditorTrebVal">${Number(trebVal).toFixed(1)}</span>
        </div>
      </div>
    `;

    document.getElementById(`${prefix}EditorGain`)?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById(`${prefix}EditorGainVal`).textContent = v.toFixed(1);
      if (isAmp2) {
        state.params['amp_2.gain'] = v;
        syncParamsToBackend({ 'amp_2.gain': v }, false);
      } else {
        updateKnobValue('inputGain', v);
      }
    });
    document.getElementById(`${prefix}EditorMaster`)?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById(`${prefix}EditorMasterVal`).textContent = v.toFixed(1);
      if (isAmp2) {
        state.params['amp_2.master'] = v;
        syncParamsToBackend({ 'amp_2.master': v }, false);
      } else {
        updateKnobValue('masterVol', v);
      }
    });
    document.getElementById(`${prefix}EditorBass`)?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById(`${prefix}EditorBassVal`).textContent = v.toFixed(1);
      if (isAmp2) {
        state.params['amp_2.bass'] = v;
        syncParamsToBackend({ 'amp_2.bass': v }, false);
      } else {
        updateKnobValue('bass', v);
      }
    });
    document.getElementById(`${prefix}EditorMid`)?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById(`${prefix}EditorMidVal`).textContent = v.toFixed(1);
      if (isAmp2) {
        state.params['amp_2.middle'] = v;
        syncParamsToBackend({ 'amp_2.middle': v }, false);
      } else {
        updateKnobValue('middle', v);
      }
    });
    document.getElementById(`${prefix}EditorTreb`)?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById(`${prefix}EditorTrebVal`).textContent = v.toFixed(1);
      if (isAmp2) {
        state.params['amp_2.treble'] = v;
        syncParamsToBackend({ 'amp_2.treble': v }, false);
      } else {
        updateKnobValue('treble', v);
      }
    });
  } else if (block.cat === 'CAB' || block.id.startsWith('blk_cab')) {
    const isCab2 = block.id === 'blk_cab_2';
    const prefix = isCab2 ? 'cab_2' : 'cab_1';
    if (title) title.textContent = `GABINETE: ${block.name.toUpperCase()}`;
    if (badge) badge.textContent = 'RES & IR MIXER';

    const resVal = isCab2
      ? (state.params['cab_2.resonance_hz'] !== undefined ? state.params['cab_2.resonance_hz'] : 98)
      : (state.circuitParams.cabResHz || 98);
    const hiCutVal = isCab2
      ? (state.params['cab_2.hicut_khz'] !== undefined ? state.params['cab_2.hicut_khz'] : 8.2)
      : (state.circuitParams.cabHiCutKhz || 8.2);
    const mic1Dist = state.params[`${prefix}.mic1.distance_cm`] !== undefined
      ? state.params[`${prefix}.mic1.distance_cm`]
      : 2.0;

    body.innerHTML = `
      <div class="pedal-controls-grid" style="grid-template-columns: repeat(3, 1fr);">
        <div class="pedal-knob-box">
          <label>RESONANCE</label>
          <input type="range" id="${prefix}EditorRes" min="65" max="135" step="1" value="${resVal}">
          <span class="pedal-val" id="${prefix}EditorResVal">${Number(resVal).toFixed(0)} Hz</span>
        </div>
        <div class="pedal-knob-box">
          <label>HI-CUT FILTER</label>
          <input type="range" id="${prefix}EditorHiCut" min="3.5" max="18.0" step="0.1" value="${hiCutVal}">
          <span class="pedal-val" id="${prefix}EditorHiCutVal">${Number(hiCutVal).toFixed(1)} kHz</span>
        </div>
        <div class="pedal-knob-box">
          <label>MIC DISTANCE</label>
          <input type="range" id="${prefix}EditorMicDist" min="0" max="30" step="0.5" value="${mic1Dist}">
          <span class="pedal-val" id="${prefix}EditorMicDistVal">${Number(mic1Dist).toFixed(1)} cm</span>
        </div>
      </div>
    `;

    document.getElementById(`${prefix}EditorRes`)?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById(`${prefix}EditorResVal`).textContent = `${v.toFixed(0)} Hz`;
      if (!isCab2) state.circuitParams.cabResHz = v;
      state.params[`${prefix}.resonance_hz`] = v;
      syncParamsToBackend({ [`${prefix}.resonance_hz`]: v }, false);
    });
    document.getElementById(`${prefix}EditorHiCut`)?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById(`${prefix}EditorHiCutVal`).textContent = `${v.toFixed(1)} kHz`;
      if (!isCab2) state.circuitParams.cabHiCutKhz = v;
      state.params[`${prefix}.hicut_khz`] = v;
      syncParamsToBackend({ [`${prefix}.hicut_khz`]: v }, false);
    });
    document.getElementById(`${prefix}EditorMicDist`)?.addEventListener('input', (e) => {
      const v = Number(e.target.value);
      document.getElementById(`${prefix}EditorMicDistVal`).textContent = `${v.toFixed(1)} cm`;
      state.params[`${prefix}.mic1.distance_cm`] = v;
      syncParamsToBackend({ [`${prefix}.mic1.distance_cm`]: v }, false);
    });
  } else {
    if (title) title.textContent = `${block.cat}: ${block.name.toUpperCase()}`;
    if (badge) badge.textContent = 'BLOCO SELECIONADO';
    body.innerHTML = '<div class="block-editor-empty">sem controles nesta versão</div>';
  }
}

function updateSelectedBlockHeader(block) {
  const badge = document.getElementById('selectedBlockBadge');
  badge.textContent = `${block.cat} // CANAL ${block.ch}`;
  badge.style.borderColor = block.color;
  badge.style.color = block.color;

  document.querySelectorAll('.ch-btn').forEach((btn) => {
    btn.classList.toggle('active', btn.dataset.ch === block.ch);
  });
}

// ============================================================================
// 2. RENDER 3D ROTARY KNOBS ON AMP FACEPLATE
// ============================================================================
function valueToAngle(val, min = 0, max = 10) {
  const norm = (val - min) / (max - min);
  return -135 + norm * 270;
}

function renderFrontPanelKnobs() {
  const row = document.getElementById('frontPanelKnobsRow') || document.getElementById('ampKnobsRow');
  if (!row) return;
  row.innerHTML = '';

  const defs = getActiveKnobDefinitions();
  defs.forEach((def) => {
    const val = state.ampParams[def.key] !== undefined ? state.ampParams[def.key] : def.defaultVal;
    state.ampParams[def.key] = val;
    const deg = valueToAngle(val, def.min, def.max);

    const unit = document.createElement('div');
    unit.className = `knob-unit ${def.hasModifier ? 'has-modifier' : ''}`;
    unit.dataset.paramKey = def.key;

    unit.innerHTML = `
      <div class="knob-dial-wrapper" title="Arraste verticalmente | Shift = Ajuste Fino | Duplo clique = Reset (${def.defaultVal})">
        <svg class="knob-scale-svg" viewBox="0 0 66 66">
          <circle cx="33" cy="33" r="29" fill="none" stroke="rgba(0,0,0,0.35)" stroke-width="2.5" stroke-dasharray="138 60" stroke-dashoffset="-15" />
          ${def.hasModifier ? `<circle cx="33" cy="33" r="31" fill="none" stroke="#00e5ff" stroke-width="2" stroke-dasharray="70 140" stroke-dashoffset="-35" opacity="0.85" />` : ''}
        </svg>
        <div class="knob-shadow-base">
          <div class="knob-rotor" id="rotor_${def.key}" style="--knob-deg: ${deg}deg;">
            <div class="knob-cap"></div>
            <div class="knob-indicator"></div>
          </div>
        </div>
      </div>
      <span class="faceplate-label">${def.label}</span>
      <span class="knob-value-readout" id="readout_${def.key}">${val.toFixed(1)}${def.hasModifier ? ' [EXP1]' : ''}</span>
    `;

    const dial = unit.querySelector('.knob-dial-wrapper');
    attachKnobDragHandler(dial, def);

    row.appendChild(unit);
  });
}

function attachKnobDragHandler(dialEl, def) {
  let startY = 0;
  let startVal = 0;

  const onMouseMove = (e) => {
    const deltaY = startY - e.clientY;
    const sensitivity = e.shiftKey ? 0.012 : 0.055;
    let nextVal = startVal + deltaY * sensitivity;
    nextVal = Math.max(def.min, Math.min(def.max, nextVal));
    updateKnobValue(def.key, nextVal);
  };

  const onMouseUp = () => {
    window.removeEventListener('mousemove', onMouseMove);
    window.removeEventListener('mouseup', onMouseUp);
  };

  dialEl.addEventListener('mousedown', (e) => {
    e.preventDefault();
    startY = e.clientY;
    startVal = state.ampParams[def.key];
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  });

  dialEl.addEventListener('dblclick', () => {
    updateKnobValue(def.key, def.defaultVal);
  });
}

function updateKnobValue(key, val) {
  if (typeof val !== 'number' || Number.isNaN(val) || !Number.isFinite(val)) {
    return;
  }
  state.ampParams[key] = val;
  const defs = getActiveKnobDefinitions();
  const def = defs.find((d) => d.key === key) || KNOB_DEFINITIONS.find((d) => d.key === key);
  const rotor = document.getElementById(`rotor_${key}`);
  const readout = document.getElementById(`readout_${key}`);

  if (rotor && def) {
    rotor.style.setProperty('--knob-deg', `${valueToAngle(val, def.min, def.max)}deg`);
  }
  if (readout && def) {
    readout.textContent = `${val.toFixed(1)}${def.hasModifier ? ' [EXP1]' : ''}`;
  }

  updateTubeIncandescence();

  // Spec 027 CA-1: Conecta giro de qualquer knob a syncParamsToBackend
  const apiKey = (def && def.param) ? def.param : (KNOB_PARAM_MAP[key] || `amp_1.${key}`);
  syncParamsToBackend({ [apiKey]: val }, false);
}

// ============================================================================
// 3. VACUUM TUBE INCANDESCENCE & WHITE-BOX CIRCUIT TRIMPOTS
// ============================================================================
function updateTubeIncandescence() {
  const biasNorm = (state.circuitParams.biasMa - 20) / 35; // 0..1
  const gainNorm = state.ampParams.inputGain / 10;
  const heatIntensity = Math.min(1, 0.45 + biasNorm * 0.4 + gainNorm * 0.25);

  const glowRadius = Math.round(16 + heatIntensity * 26);
  const glowAlpha = (0.45 + heatIntensity * 0.48).toFixed(2);

  document.querySelectorAll('.tube-glass-envelope').forEach((env) => {
    env.style.setProperty(
      '--tube-dynamic-glow',
      `0 0 ${glowRadius}px rgba(255, 105, 0, ${glowAlpha}), inset 0 0 14px rgba(255, 180, 60, ${glowAlpha})`
    );
  });

  document.querySelectorAll('.tube-heater-filament').forEach((fil) => {
    fil.style.setProperty('--heater-opacity', (0.65 + heatIntensity * 0.35).toFixed(2));
  });

  const grilleGlow = document.getElementById('grilleGlow');
  if (grilleGlow) {
    grilleGlow.style.opacity = (0.35 + heatIntensity * 0.65).toFixed(2);
  }

  const plateCurrentReadout = document.getElementById('plateCurrentReadout');
  if (plateCurrentReadout) {
    plateCurrentReadout.textContent = `Ip: ${state.circuitParams.biasMa.toFixed(1)} mA / ${state.circuitParams.plateVolts}V B+`;
  }
}

// ============================================================================
// 4. TEKTRONIX-STYLE REAL-TIME CRT OSCILLOSCOPE & TRANSFER CURVE (/api/scope)
// ============================================================================
let scopePhase = 0;

async function pollScopeFromBackend() {
  try {
    const res = await fetch('/api/scope');
    if (res.ok) {
      const data = await res.json();
      if (data.transfer_curve && Array.isArray(data.transfer_curve)) {
        state.scopeTransferCurve = data.transfer_curve;
      }
      if (data.waveform && Array.isArray(data.waveform)) {
        state.scopeWaveform = data.waveform;
      }
    }
  } catch (_) {}
}

function drawCrtScope() {
  const canvas = document.getElementById('crtCanvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  const w = canvas.width;
  const h = canvas.height;

  ctx.clearRect(0, 0, w, h);

  // Center axes
  ctx.strokeStyle = 'rgba(0, 230, 118, 0.22)';
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(0, h / 2);
  ctx.lineTo(w, h / 2);
  ctx.moveTo(w / 2, 0);
  ctx.lineTo(w / 2, h);
  ctx.stroke();

  const drive = (state.ampParams.inputGain + state.ampParams.overdrive * 0.8) * 0.6;
  const asym = state.circuitParams.triodeAsym;
  const sag = state.circuitParams.sagDb;

  if (state.scopeMode === 'waveform') {
    // 1. Clean Input DI Signal (Dashed Green)
    ctx.strokeStyle = 'rgba(0, 230, 118, 0.55)';
    ctx.setLineDash([4, 4]);
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    for (let x = 0; x < w; x++) {
      const t = (x / w) * Math.PI * 4 + scopePhase;
      const inSig = Math.sin(t) * 0.55 + Math.sin(t * 2) * 0.18;
      const y = h / 2 - inSig * (h * 0.36);
      if (x === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();
    ctx.setLineDash([]);

    // 2. White-Box Tube Saturated Output Signal (Glowing Cyan)
    ctx.strokeStyle = '#00e5ff';
    ctx.shadowColor = '#00e5ff';
    ctx.shadowBlur = 10;
    ctx.lineWidth = 2.4;
    ctx.beginPath();
    if (state.liveWaveform && state.liveWaveform.length > 0 && (state.liveOutputDb > -58 || state.testRiffPlaying)) {
      const wave = state.liveWaveform;
      const step = wave.length / w;
      for (let x = 0; x < w; x++) {
        const idx = Math.floor(x * step);
        const s = wave[idx] || 0;
        const y = h / 2 - s * (h * 0.42);
        if (x === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
    } else {
      for (let x = 0; x < w; x++) {
        const t = (x / w) * Math.PI * 4 + scopePhase;
        const inSig = Math.sin(t) * 0.65 + Math.sin(t * 2) * 0.22;
        const outSig = Math.tanh(inSig * 1.8);
        const y = h / 2 - outSig * (h * 0.38);
        if (x === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
    }
    ctx.stroke();
    ctx.shadowBlur = 0;
  } else if (state.scopeMode === 'transfer') {
    // Curva de transferência real obtida do motor DSP (/api/scope)
    ctx.strokeStyle = '#ffca28';
    ctx.shadowColor = '#ff9100';
    ctx.shadowBlur = 10;
    ctx.lineWidth = 2.5;
    ctx.beginPath();
    const curve = state.scopeTransferCurve || [];
    if (curve.length > 0) {
      const step = curve.length / w;
      for (let px = 0; px < w; px++) {
        const idx = Math.min(curve.length - 1, Math.floor(px * step));
        const yNorm = curve[idx] || 0;
        const py = h / 2 - yNorm * (h * 0.4);
        if (px === 0) ctx.moveTo(px, py);
        else ctx.lineTo(px, py);
      }
    } else {
      for (let px = 0; px < w; px++) {
        const xNorm = (px / w) * 2 - 1;
        const yNorm = Math.tanh(xNorm * 1.6);
        const py = h / 2 - yNorm * (h * 0.4);
        if (px === 0) ctx.moveTo(px, py);
        else ctx.lineTo(px, py);
      }
    }
    ctx.stroke();
    ctx.shadowBlur = 0;
  } else if (state.scopeMode === 'tonestack') {
    // Tone Stack Frequency Curve + Speaker Impedance Resonance Curve
    const bass = state.ampParams.bass / 10;
    const mid = state.ampParams.middle / 10;
    const treb = state.ampParams.treble / 10;
    const resPeakNorm = (state.circuitParams.cabResHz - 65) / 70;

    // Speaker Impedance Curve (Orange)
    ctx.strokeStyle = '#ff9100';
    ctx.lineWidth = 2;
    ctx.beginPath();
    for (let px = 0; px < w; px++) {
      const f = px / w;
      const lfPeak = Math.exp(-Math.pow((f - (0.18 + resPeakNorm * 0.12)) / 0.06, 2)) * 0.45;
      const hfRise = Math.pow(f, 2.2) * 0.38;
      const y = h * 0.72 - (lfPeak + hfRise) * (h * 0.55);
      if (px === 0) ctx.moveTo(px, y);
      else ctx.lineTo(px, y);
    }
    ctx.stroke();

    // Tone Stack Curve (Bright Green)
    ctx.strokeStyle = '#00e676';
    ctx.shadowColor = '#00e676';
    ctx.shadowBlur = 8;
    ctx.lineWidth = 2.4;
    ctx.beginPath();
    for (let px = 0; px < w; px++) {
      const f = px / w;
      const lowShelf = (1 - f) * bass * 0.45;
      const midDip = -Math.exp(-Math.pow((f - 0.45) / 0.16, 2)) * (1 - mid) * 0.42;
      const highShelf = f * treb * 0.48;
      const y = h * 0.62 - (lowShelf + midDip + highShelf) * (h * 0.45);
      if (px === 0) ctx.moveTo(px, y);
      else ctx.lineTo(px, y);
    }
    ctx.stroke();
    ctx.shadowBlur = 0;
  }

  scopePhase += state.testRiffPlaying ? 0.14 : 0.035;

  // Update Top VU Meters dynamically from real WASAPI signal or simulation
  const vuIn = document.getElementById('vuInFill');
  const vuOut = document.getElementById('vuOutFill');
  if (vuIn && vuOut) {
    if (state.liveInputDb !== undefined && state.liveInputDb > -100) {
      const inPct = Math.min(100, Math.max(0, (state.liveInputDb + 60) * 1.666));
      const outPct = Math.min(100, Math.max(0, (state.liveOutputDb + 60) * 1.666));
      vuIn.style.width = `${inPct}%`;
      vuOut.style.width = `${outPct}%`;
    } else {
      const jitter = state.testRiffPlaying ? Math.sin(scopePhase * 3) * 14 : Math.sin(scopePhase) * 3;
      vuIn.style.width = `${Math.min(96, Math.max(18, 58 + jitter))}%`;
      vuOut.style.width = `${Math.min(98, Math.max(22, 48 + state.ampParams.masterVol * 4.2 + jitter))}%`;
    }
  }

  requestAnimationFrame(drawCrtScope);
}

// ============================================================================
// 5. RENDER 8 HARDWARE FOOTSWITCHES WITH RGB LED RINGS & OLED SCRIBBLE STRIPS
// ============================================================================
function renderFootswitches() {
  const container = document.getElementById('footswitchArray');
  container.innerHTML = '';

  // Hybrid Mode: Switches 1-4 = Stomp Blocks | Switches 5-8 = Scenes A-D
  const stompBlocks = [
    state.blocks.find((b) => b.id === 'blk_drive'),
    state.blocks.find((b) => b.id === 'blk_mod_2'),
    state.blocks.find((b) => b.id === 'blk_dly_1'),
    state.blocks.find((b) => b.id === 'blk_rev_1'),
  ];

  stompBlocks.forEach((blk, idx) => {
    if (!blk) return;
    const unit = document.createElement('div');
    unit.className = `footswitch-unit ${blk.bypassed ? 'dimmed' : ''}`;
    unit.style.setProperty('--fs-color', blk.color);

    unit.innerHTML = `
      <div class="scribble-strip-oled">
        <div class="oled-top-line"><span>FS ${idx + 1}</span><span>STOMP</span></div>
        <div class="oled-main-text">${blk.name}</div>
      </div>
      <div class="actuator-ring-rgb">
        <div class="actuator-metal-plunger"></div>
      </div>
    `;

    unit.addEventListener('click', () => {
      blk.bypassed = !blk.bypassed;
      const keyMap = {
        'blk_drive': 'drive.enabled',
        'blk_mod_2': 'mod_2.enabled',
        'blk_dly_1': 'dly_1.enabled',
        'blk_rev_1': 'rev_1.enabled',
      };
      if (keyMap[blk.id]) {
        syncParamsToBackend({ [keyMap[blk.id]]: !blk.bypassed }, true);
      }
      updateCpuBudget();
      renderSignalGrid();
      renderFootswitches();
    });

    container.appendChild(unit);
  });

  // Switches 5-8: Scenes A, B, C, D (Gapless Switching)
  const sceneColors = ['#00e5ff', '#ffb300', '#ff3d00', '#e040fb'];
  for (let s = 0; s < 4; s++) {
    const sc = state.scenes[s];
    const isActive = state.activeScene === s;
    const unit = document.createElement('div');
    unit.className = `footswitch-unit ${isActive ? '' : 'dimmed'}`;
    unit.style.setProperty('--fs-color', sceneColors[s]);

    unit.innerHTML = `
      <div class="scribble-strip-oled">
        <div class="oled-top-line"><span>FS ${s + 5}</span><span>SCENE ${sc.id}</span></div>
        <div class="oled-main-text">${sc.name}</div>
      </div>
      <div class="actuator-ring-rgb">
        <div class="actuator-metal-plunger"></div>
      </div>
    `;

    unit.addEventListener('click', () => activateSceneGapless(s));
    container.appendChild(unit);
  }
}

// ============================================================================
// 6. GAPLESS SCENE SWITCHING (< 1ms Crossfade State Transition)
// ============================================================================
async function activateSceneGapless(sceneIdx) {
  state.activeScene = sceneIdx;
  const sc = state.scenes[sceneIdx];
  if (!sc) return;

  // Update Top Scene Buttons
  document.querySelectorAll('.scene-btn').forEach((btn) => {
    btn.classList.toggle('active', Number(btn.dataset.scene) === sceneIdx);
  });

  // Update Amp Channel & Knobs without audio dropout
  const ampBlock = state.blocks.find((b) => b.id === 'blk_amp_1');
  if (ampBlock) {
    ampBlock.ch = sc.ampCh;
    state.selectedChannel = sc.ampCh;
    updateSelectedBlockHeader(ampBlock);
  }

  const driveBlock = state.blocks.find((b) => b.id === 'blk_drive');
  if (driveBlock) driveBlock.bypassed = sc.driveBypass;

  const dlyBlock = state.blocks.find((b) => b.id === 'blk_dly_1');
  if (dlyBlock) dlyBlock.bypassed = sc.dlyBypass;

  updateKnobValue('inputGain', sc.gain);
  updateKnobValue('overdrive', sc.od);

  // Sincroniza parâmetros de cena para o motor DSP (Spec 014, CA-2; Spec 024, CA-10; Spec 025, CA-6)
  const delta = {
    'amp_1.gain': sc.gain,
    'amp_1.overdrive': sc.od,
    'drive.enabled': !sc.driveBypass,
    'dly_1.enabled': !sc.dlyBypass,
    'scene.active': sc.id,
  };
  syncParamsToBackend(delta, true);

  let transitionMs = 1.2;
  try {
    const res = await fetch('/api/scenes/select', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Sonix-Token': window.__SONIX_TOKEN__ || '',
      },
      body: JSON.stringify({ scene: sc.id }),
    });
    if (res.ok) {
      const data = await res.json();
      if (data.transition_ms !== undefined) {
        transitionMs = data.transition_ms;
      }
    }
  } catch (_) {}

  // Update Faceplate Material Theme (Brushed Brass, Chrome, or Blackout)
  const faceplate = document.getElementById('ampFaceplate');
  faceplate.className = `amp-faceplate ${sc.faceplate}`;
  document.getElementById('ampBrandLogoText').textContent = sc.logo;

  const badge = document.getElementById('gaplessStatusBadge');
  if (badge) {
    badge.textContent = `⚡ SCENE ${sc.id} (${sc.name}) — GAPLESS CROSSFADE ${transitionMs}ms + SPILLOVER`;
  }

  updateCpuBudget();
  renderSignalGrid();
  renderFootswitches();
}

async function saveCurrentScene() {
  const sc = state.scenes[state.activeScene];
  if (!sc) return;
  if (state.ampParams && state.ampParams.gain !== undefined) {
    sc.gain = state.ampParams.gain;
  }
  const delta = {
    'amp_1.gain': sc.gain,
    'scene.active': sc.id,
  };
  await syncParamsToBackend(delta, true);
}

// ============================================================================
// 7. DSP CPU BUDGET CALCULATOR
// ============================================================================
function updateCpuBudget() {
  const osMultiplier = state.oversampling === '8x' ? 1.45 : state.oversampling === '4x' ? 1.15 : 0.9;
  let total = 4.2; // Base engine overhead
  state.blocks.forEach((b) => {
    if (!b.bypassed) {
      const factor = b.cat === 'AMP' || b.cat === 'DRIVE' ? osMultiplier : 1.0;
      total += b.cpu * factor;
    }
  });
  state.estimatedCpu = total;
}

// ============================================================================
// 8. INTERACTIVE 3D SPEAKER CONE MICROPHONE PLACEMENT (CAB VIEW)
// ============================================================================
function initDraggableCabMics() {
  const stage = document.getElementById('speakerConeStage');
  const mic1 = document.getElementById('mic1Handle');
  const mic2 = document.getElementById('mic2Handle');

  function makeDraggable(handle, readoutId, labelPrefix) {
    let dragging = false;

    handle.addEventListener('mousedown', (e) => {
      e.preventDefault();
      dragging = true;
    });

    window.addEventListener('mousemove', (e) => {
      if (!dragging) return;
      const rect = stage.getBoundingClientRect();
      let xPct = ((e.clientX - rect.left) / rect.width) * 100;
      let yPct = ((e.clientY - rect.top) / rect.height) * 100;
      xPct = Math.max(12, Math.min(88, xPct));
      yPct = Math.max(15, Math.min(85, yPct));

      handle.style.left = `${xPct}%`;
      handle.style.top = `${yPct}%`;

      // Distance from center (50%, 50%) = Cap-to-Edge % (Spec 022, CA-7: não altera distance_cm)
      const distNorm = Math.min(1.0, Math.max(0.0, Math.hypot(xPct - 50, yPct - 50) * 1.8 / 100));
      const distFromCenter = Math.round(distNorm * 100);
      const micIdx = handle.id === 'mic1Handle' ? 1 : 2;
      syncParamsToBackend({ [`cab_1.mic${micIdx}.position`]: distNorm });

      const readout = document.getElementById(readoutId);
      if (readout) {
        readout.textContent = `${labelPrefix} ${distFromCenter}%`;
      }
    });

    window.addEventListener('mouseup', () => {
      dragging = false;
    });
  }

  if (stage && mic1 && mic2) {
    makeDraggable(mic1, 'mic1CoordText', 'Cap-Edge');
    makeDraggable(mic2, 'mic2CoordText', 'Cap-Edge');
  }
}

// ============================================================================
// 9. WORKSPACE VIEW SWITCHING & EVENT LISTENERS
// ============================================================================
function switchMainWorkspaceView(viewName) {
  document.querySelectorAll('.subnav-tab').forEach((t) => {
    t.classList.toggle('active', t.dataset.view === viewName);
  });

  const gridSec = document.getElementById('signalGridSection');
  const mainStage = document.getElementById('mainStageSection');
  const ampShell = document.getElementById('ampHeadShell');
  const chassisBench = document.getElementById('openChassisWorkbench');
  const cabBench = document.getElementById('cabRoomWorkbench');
  const dsCatalog = document.getElementById('dsCatalogSection');

  if (viewName === 'ds-catalog') {
    gridSec.style.display = 'none';
    mainStage.style.display = 'none';
    dsCatalog.classList.add('visible');
  } else if (viewName === 'cab-room') {
    gridSec.style.display = 'block';
    mainStage.style.display = 'flex';
    dsCatalog.classList.remove('visible');
    ampShell.style.display = 'none';
    chassisBench.classList.remove('visible');
    cabBench.classList.add('visible');
  } else {
    gridSec.style.display = 'block';
    mainStage.style.display = 'flex';
    dsCatalog.classList.remove('visible');
    ampShell.style.display = 'block';
    cabBench.classList.remove('visible');
    if (state.chassisOpen) chassisBench.classList.add('visible');
  }
}

function initEventListeners() {
  // Subnav Tabs
  document.querySelectorAll('.subnav-tab').forEach((tab) => {
    tab.addEventListener('click', () => switchMainWorkspaceView(tab.dataset.view));
  });

  // 8 Scenes Strip
  document.querySelectorAll('.scene-btn').forEach((btn) => {
    btn.addEventListener('click', () => activateSceneGapless(Number(btn.dataset.scene)));
  });

  // Channel A/B/C/D Buttons
  document.querySelectorAll('.ch-btn').forEach((btn) => {
    btn.addEventListener('click', () => {
      const ch = btn.dataset.ch;
      state.selectedChannel = ch;
      const blk = state.blocks.find((b) => b.id === state.selectedBlockId);
      if (blk) {
        blk.ch = ch;
        updateSelectedBlockHeader(blk);
        renderSignalGrid();
      }
    });
  });

  // Oversampling Select
  const oversamplingSel = document.getElementById('oversamplingSelect');
  if (oversamplingSel) {
    oversamplingSel.addEventListener('change', (e) => {
      state.oversampling = e.target.value;
      updateCpuBudget();
      syncParamsToBackend({ 'engine.oversampling': e.target.value }, true);
    });
  }

  // Open Chassis / Mod Shop Switch
  const btnChassis = document.getElementById('btnToggleChassis');
  btnChassis.addEventListener('click', () => {
    state.chassisOpen = !state.chassisOpen;
    btnChassis.classList.toggle('open-active', state.chassisOpen);
    btnChassis.innerHTML = state.chassisOpen
      ? '🔒 FECHAR AMPFORGE™ CHASSI (FRONT PANEL)'
      : '🔧 AMPFORGE™ CHASSIS / MOD SHOP (CIRCUIT VIEW)';
    document.getElementById('openChassisWorkbench').classList.toggle('visible', state.chassisOpen);
  });

  // Scope Mode Tabs
  document.querySelectorAll('.scope-tab').forEach((tab) => {
    tab.addEventListener('click', () => {
      state.scopeMode = tab.dataset.scope;
      document.querySelectorAll('.scope-tab').forEach((t) => t.classList.remove('active'));
      tab.classList.add('active');
    });
  });

  // White-Box Circuit Trimpot Sliders
  const bindSlider = (id, valId, key, paramKey, unitSuffix) => {
    const el = document.getElementById(id);
    const valEl = document.getElementById(valId);
    if (!el) return;
    el.addEventListener('input', () => {
      const val = parseFloat(el.value);
      state.circuitParams[key] = val;
      if (valEl) valEl.textContent = `${el.value}${unitSuffix}`;
      updateTubeIncandescence();
      if (paramKey) {
        syncParamsToBackend({ [paramKey]: val }, false);
      }
    });
  };

  bindSlider('sliderBias', 'valBias', 'biasMa', 'amp_1.bias_ma', ' mA');
  bindSlider('sliderSag', 'valSag', 'sagDb', 'amp_1.sag', ' dB');
  bindSlider('sliderNfb', 'valNfb', 'nfb', 'amp_1.nfb', '');
  bindSlider('sliderXformer', 'valXformer', 'xformerSat', 'amp_1.xformer_sat', '');
  bindSlider('sliderPlateV', 'valPlateV', 'plateVolts', 'amp_1.preamp_plate_v', ' V');
  bindSlider('sliderAsym', 'valAsym', 'triodeAsym', 'amp_1.triode_asym', '');
  bindSlider('sliderCabRes', 'valCabRes', 'cabResHz', 'cab_1.resonance_hz', ' Hz (+6.8dB)');
  bindSlider('sliderCabHiCut', 'valCabHiCut', 'cabHiCutKhz', 'cab_1.hicut_khz', ' kHz');

  // Power Tube Model Select
  const powerTubeSel = document.getElementById('powerTubeSelect');
  if (powerTubeSel) {
    powerTubeSel.addEventListener('change', (e) => {
      const v = e.target.value;
      const lbl1 = document.getElementById('pwrTubeLabel1');
      const lbl2 = document.getElementById('pwrTubeLabel2');
      if (lbl1) lbl1.textContent = v;
      if (lbl2) lbl2.textContent = v;
      syncParamsToBackend({ 'amp_1.power_tube': v }, true);
    });
  }

  // Preamp Tube, Tone Stack, Rectifier Selects
  const preampTubeSel = document.getElementById('preampTubeSelect');
  if (preampTubeSel) {
    preampTubeSel.addEventListener('change', (e) => {
      syncParamsToBackend({ 'amp_1.preamp_tube': e.target.value }, true);
    });
  }

  const toneStackSel = document.getElementById('toneStackSelect');
  if (toneStackSel) {
    toneStackSel.addEventListener('change', (e) => {
      syncParamsToBackend({ 'amp_1.tonestack': e.target.value }, true);
    });
  }

  const rectifierSel = document.getElementById('rectifierSelect');
  if (rectifierSel) {
    rectifierSel.addEventListener('change', (e) => {
      syncParamsToBackend({ 'amp_1.rectifier': e.target.value }, true);
    });
  }

  // Bright Switch on Amp Faceplate
  const brightBtn = document.getElementById('brightSwitchBtn');
  if (brightBtn) {
    brightBtn.addEventListener('click', () => {
      state.ampParams.brightSwitch = !state.ampParams.brightSwitch;
      const lbl = document.getElementById('brightSwitchLabel');
      if (lbl) lbl.textContent = `BRIGHT: ${state.ampParams.brightSwitch ? 'ON' : 'OFF'}`;
      syncParamsToBackend({ 'amp_1.bright': state.ampParams.brightSwitch }, true);
    });
  }

  // Expression Pedal 1 -> Modulates Input Gain in Real Time
  const expSlider = document.getElementById('expPedalSlider');
  if (expSlider) {
    expSlider.addEventListener('input', () => {
      const pct = parseFloat(expSlider.value);
      const valTxt = document.getElementById('expPedalValueText');
      if (valTxt) valTxt.textContent = `${pct}%`;
      // Modifier maps 0..100% to 3.0..9.5 Input Gain
      const mappedGain = 3.0 + (pct / 100) * 6.5;
      updateKnobValue('inputGain', mappedGain);
    });
  }

  // Cab Room Controls (Distance, IR, Speaker)
  const mic1DistEl = document.getElementById('sliderMic1Dist');
  if (mic1DistEl) {
    mic1DistEl.addEventListener('input', (e) => {
      const val = parseFloat(e.target.value);
      const readout = document.getElementById('valMic1Dist');
      if (readout) readout.textContent = `${val.toFixed(1)} cm`;
      syncParamsToBackend({ 'cab_1.mic1.distance_cm': val }, false);
    });
  }

  const mic2DistEl = document.getElementById('sliderMic2Dist');
  if (mic2DistEl) {
    mic2DistEl.addEventListener('input', (e) => {
      const val = parseFloat(e.target.value);
      const readout = document.getElementById('valMic2Dist');
      if (readout) readout.textContent = `${val.toFixed(1)} cm`;
      syncParamsToBackend({ 'cab_1.mic2.distance_cm': val }, false);
    });
  }

  const cabIrCheck = document.getElementById('checkCabIrEnabled');
  if (cabIrCheck) {
    cabIrCheck.addEventListener('change', (e) => {
      const en = e.target.checked;
      syncParamsToBackend({ 'cab_1.ir_enabled': en }, true);
    });
  }

  const impCurveSel = document.getElementById('impedanceCurveSelect');
  if (impCurveSel) {
    impCurveSel.addEventListener('change', (e) => {
      syncParamsToBackend({ 'cab_1.speaker': e.target.value }, true);
    });
  }

  // Toggle Test Signal DSP Generator (Spec 019, CA-5)
  const btnRiff = document.getElementById('testSignalBtn') || document.getElementById('btnToggleRiff');
  if (btnRiff) {
    btnRiff.addEventListener('click', () => {
      state.testRiffPlaying = !state.testRiffPlaying;
      btnRiff.classList.toggle('active-riff', state.testRiffPlaying);
      btnRiff.textContent = state.testRiffPlaying ? '■ Stop Test Signal' : '▶ Test Signal DSP';
      toggleTestRiffAudio(state.testRiffPlaying);
    });
  }

  // Model Selection handler (Spec 020, CA-3, CA-7 & Spec 027 CA-6)
  const ampModelSel = document.getElementById('ampModelSelect');
  if (ampModelSel) {
    ampModelSel.addEventListener('change', (e) => {
      const model = e.target.value;
      state.currentAmpModel = model;
      state.ampParams.model = model;
      syncParamsToBackend({ 'amp_1.model': model }, false);
      renderFrontPanelKnobs();
    });
  }

  // Audio Input Channel Selection handler (Spec 018, CA-8)
  const audioInputChannelSelect = document.getElementById('audioInputChannelSelect');
  if (audioInputChannelSelect) {
    audioInputChannelSelect.addEventListener('change', (e) => {
      syncParamsToBackend({ 'audio.input_channel': e.target.value }, false);
    });
  }

  // Toggle Tuner Bar (Spec 026, CA-4, CA-5; Spec 029, CA-1)
  const btnTuner = document.getElementById('btnToggleTuner');
  if (btnTuner) {
    btnTuner.addEventListener('click', () => {
      state.tunerVisible = !state.tunerVisible;
      btnTuner.classList.toggle('active-mode', state.tunerVisible);
      const tunerBar = document.getElementById('tunerBar');
      if (tunerBar) tunerBar.classList.toggle('visible', state.tunerVisible);
      syncParamsToBackend({ 'tuner.enabled': state.tunerVisible }, false);
    });
  }

  const btnTunerOutputMode = document.getElementById('btnTunerOutputMode');
  if (btnTunerOutputMode) {
    btnTunerOutputMode.addEventListener('click', () => {
      state.tunerOutputMode = state.tunerOutputMode === 'mute' ? 'bypass' : 'mute';
      btnTunerOutputMode.textContent = `OUT: ${state.tunerOutputMode.toUpperCase()}`;
      syncParamsToBackend({ 'tuner.output': state.tunerOutputMode }, false);
    });
  }

  // Input Calibration Wizard (Spec 019 & Spec 029, CA-3 a CA-5)
  const btnCalibrateInput = document.getElementById('btnCalibrateInput');
  const wizard = document.getElementById('inputCalibrationWizard');
  const step1 = document.getElementById('calibStep1');
  const step2 = document.getElementById('calibStep2');
  const step3 = document.getElementById('calibStep3');
  const btnStartCalib = document.getElementById('btnStartCalib');
  const btnCloseCalib = document.getElementById('btnCloseCalib');

  const resetCalibWizardSteps = () => {
    if (state.calibTimer) {
      clearTimeout(state.calibTimer);
      state.calibTimer = null;
    }
    state.calibRunning = false;
    if (step1) step1.style.display = 'block';
    if (step2) step2.style.display = 'none';
    if (step3) step3.style.display = 'none';
    if (btnStartCalib) {
      btnStartCalib.disabled = false;
      btnStartCalib.textContent = 'Iniciar Calibração';
    }
  };

  if (btnCalibrateInput && wizard) {
    btnCalibrateInput.addEventListener('click', () => {
      resetCalibWizardSteps();
      wizard.style.display = 'block';
    });
  }

  if (btnCloseCalib && wizard) {
    btnCloseCalib.addEventListener('click', () => {
      resetCalibWizardSteps();
      wizard.style.display = 'none';
    });
  }

  if (btnStartCalib) {
    const waitMs = (ms) => new Promise((resolve) => {
      state.calibTimer = setTimeout(resolve, ms);
    });
    btnStartCalib.addEventListener('click', async () => {
      if (state.calibRunning) return;
      state.calibRunning = true;
      btnStartCalib.disabled = true;
      btnStartCalib.textContent = 'Medindo silêncio (3s)...';
      if (step1) step1.style.display = 'block';
      if (step2) step2.style.display = 'none';
      if (step3) step3.style.display = 'none';

      try {
        await waitMs(3000);
        if (!state.calibRunning) return;
        await fetch('/api/input/calibrate', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'X-Sonix-Token': window.__SONIX_TOKEN__ || '',
          },
          body: JSON.stringify({ phase: 'silence' }),
        });

        if (!state.calibRunning) return;
        if (step2) step2.style.display = 'block';
        btnStartCalib.textContent = 'Toque acordes agora (5s)...';

        await waitMs(5000);
        if (!state.calibRunning) return;
        const res = await fetch('/api/input/calibrate', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'X-Sonix-Token': window.__SONIX_TOKEN__ || '',
          },
          body: JSON.stringify({ phase: 'strum' }),
        });

        if (res && res.ok) {
          const data = await res.json();
          if (step3) step3.style.display = 'block';
          const resultEl = document.getElementById('calibResultText');
          const diagEl = document.getElementById('calibDiagnosticText');
          if (resultEl) {
            resultEl.textContent =
              `Boost: ${Number(data.proposed_boost).toFixed(2)}x | ` +
              `Gate: ${Number(data.proposed_gate_threshold_db).toFixed(1)} dB | ` +
              `Ruído: ${Number(data.noise_floor_rms_db).toFixed(1)} dBFS | ` +
              `Pico: ${Number(data.raw_peak_db).toFixed(1)} dBFS`;
          }
          if (diagEl) {
            diagEl.textContent = `Diagnóstico: ${data.diagnostic}`;
          }
          await loadInitialState();
        }
      } catch (_) {
        // Falha tratada silenciosamente mantendo interface responsiva
      } finally {
        state.calibRunning = false;
        btnStartCalib.disabled = false;
        btnStartCalib.textContent = 'Iniciar Calibração';
      }
    });
  }

  // Toggle Hardware Floorboard Shell Mode
  const btnShell = document.getElementById('btnHardwareShell');
  btnShell.addEventListener('click', () => {
    state.hardwareShellMode = !state.hardwareShellMode;
    btnShell.classList.toggle('active-mode', state.hardwareShellMode);
    document.body.classList.toggle('hardware-shell-mode', state.hardwareShellMode);
  });

  // Preset selection handler (Spec 025, CA-1, CA-2)
  const presetSel = document.getElementById('presetSelect');
  if (presetSel) {
    presetSel.addEventListener('change', async (e) => {
      const presetId = e.target.value;
      try {
        const res = await fetch('/api/presets/load', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'X-Sonix-Token': window.__SONIX_TOKEN__ || '',
          },
          body: JSON.stringify({ id: presetId }),
        });
        if (res.ok) {
          const badge = document.getElementById('presetBankBadge');
          if (badge) {
            const idx = presetSel.selectedIndex + 1;
            badge.textContent = `P0${idx}`;
          }
          await loadInitialState();
        }
      } catch (_) {}
    });
  }

  // Export Preset via backend engine (Spec 025, CA-4)
  const btnExport = document.getElementById('btnExportPreset');
  if (btnExport) {
    btnExport.addEventListener('click', async () => {
      const presetId = document.getElementById('presetSelect')?.value || 'brit800';
      try {
        const res = await fetch(`/api/presets/${presetId}/export`);
        if (res.ok) {
          const data = await res.json();
          const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `${presetId}.sonixpreset`;
          a.click();
          URL.revokeObjectURL(url);
        }
      } catch (_) {}
    });
  }
}

// ============================================================================
// DSP MOTOR GUITAR TEST RIFF (Spec 019, CA-5)
// ============================================================================
function toggleTestRiffAudio(start) {
  syncParamsToBackend({ "input.source": start ? "test_riff" : "device" }, false);
}

// ============================================================================
// REAL-TIME TELEMETRY & STROBE TUNER (Spec 026, CA-5, CA-6, CA-7)
// ============================================================================
async function updateStrobeTunerFromSignal() {
  const tunerEl = document.getElementById('tunerNoteText');
  const needle = document.querySelector('.strobe-center-needle');
  if (!tunerEl || !needle || !state.tunerVisible) return;

  try {
    const res = await fetch('/api/tuner');
    if (res.ok) {
      const data = await res.json();
      if (!data.note || data.note === '---' || data.frequency_hz <= 0) {
        tunerEl.textContent = '—';
        needle.style.transform = 'translateX(0px)';
      } else {
        const sign = data.cents > 0 ? '+' : '';
        const cents = Number(data.cents).toFixed(1);
        tunerEl.textContent = `${data.note} (${sign}${cents}¢)`;
        const offset = Math.min(48, Math.max(-48, data.cents * 0.5));
        needle.style.transform = `translateX(${offset}px)`;
      }
    }
  } catch (_) {
    tunerEl.textContent = '—';
  }
}

async function pollBackendTelemetry() {
  try {
    const res = await fetch('/api/telemetry');
    if (res.ok) {
      const data = await res.json();
      state.liveInputDb = data.input_peak_db;
      state.liveOutputDb = data.output_peak_db;
      if (typeof data.cpu_usage_pct === 'number') {
        const cpuEl = document.getElementById('cpuMeterText');
        if (cpuEl) cpuEl.textContent = `${data.cpu_usage_pct.toFixed(1)}%`;
      }
      if (typeof data.adc_clips === 'number' && data.adc_clips > 0) {
        state.adcClips = data.adc_clips;
        const clipEl = document.getElementById('vuInClip');
        if (clipEl) {
          clipEl.classList.add('clipping');
          setTimeout(() => clipEl.classList.remove('clipping'), 1000);
        }
      }
      if (typeof data.gate_state === 'string') {
        state.liveGateState = data.gate_state;
        state.liveGateGrDb = typeof data.gate_gr_db === 'number' ? data.gate_gr_db : 0.0;
        const gateEl = document.getElementById('gateStatusBadge');
        if (gateEl) {
          gateEl.textContent = `${state.liveGateState} // GR ${state.liveGateGrDb.toFixed(1)} dB`;
        }
      }
      if (data.waveform && data.waveform.length > 0) {
        state.liveWaveform = data.waveform;
      }
      updateStrobeTunerFromSignal();
      pollScopeFromBackend();
      setApiError(false);
    } else {
      setApiError(true, `HTTP ${res.status}`);
    }
  } catch (_) {
    setApiError(true, 'OFFLINE');
  }

  // Spec 027 CA-4: Atualiza badges com telemetria real de /api/audio/status
  try {
    const audioRes = await fetch('/api/audio/status');
    if (audioRes.ok) {
      const audioData = await audioRes.json();
      const modeEl = document.getElementById('audioModeText');
      if (modeEl && (audioData.mode || audioData.backend)) {
        modeEl.textContent = audioData.mode || audioData.backend;
      }
      const srEl = document.getElementById('audioSampleRateText');
      if (srEl && audioData.sample_rate) {
        srEl.textContent = `${audioData.sample_rate} Hz`;
      }
      const periodsEl = document.getElementById('audioPeriodsText');
      if (periodsEl && audioData.input_period_frames !== undefined && audioData.output_period_frames !== undefined) {
        periodsEl.textContent = `${audioData.input_period_frames}/${audioData.output_period_frames}`;
      }
      const latEl = document.getElementById('audioMeasuredRoundtripText');
      if (latEl && audioData.driver_input_latency_ms !== undefined && audioData.driver_output_latency_ms !== undefined) {
        const totalLat = audioData.driver_input_latency_ms + audioData.driver_output_latency_ms;
        latEl.textContent = `${totalLat.toFixed(1)} ms`;
      }
    }
  } catch (_) {}
}

// ============================================================================
// INITIALIZE APPLICATION
// ============================================================================
window.addEventListener('DOMContentLoaded', () => {
  renderSignalGrid();
  renderFrontPanelKnobs();
  renderFootswitches();
  updateTubeIncandescence();
  updateCpuBudget();
  initDraggableCabMics();
  initEventListeners();
  setInterval(pollBackendTelemetry, 45);
  requestAnimationFrame(drawCrtScope);
});
