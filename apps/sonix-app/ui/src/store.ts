import { CONTROLS_REGISTRY, type ControlDescriptor } from './controls.registry.ts';
import {
  SonixTransport,
  classifyBridgeError,
  type BridgeFaultKind,
  type ScopeFrame,
  type TelemetryFrame,
  type TunerFrame,
} from './transport.ts';

export type ScreenName =
  | 'main'
  | 'chassi'
  | 'gabinete'
  | 'palco'
  | 'biblioteca'
  | 'audio';

export type BlockId =
  | 'in'
  | 'gate'
  | 'drive'
  | 'split'
  | 'amp'
  | 'cab'
  | 'eq'
  | 'merge'
  | 'delay'
  | 'reverb'
  | 'out'
  | 'comp'
  | 'amp2'
  | 'cab2'
  | 'chorus'
  | 'pitch'
  | 'wah';

export interface PresetItem {
  id: string;
  name: string;
  setlist?: string;
}

export interface AudioStatusData {
  backend: string;
  mode: string;
  raw_processing?: boolean;
  sample_rate: number;
  input_period_frames: number;
  output_period_frames: number;
  driver_input_latency_ms: number;
  driver_output_latency_ms: number;
  measured_roundtrip_ms?: number | null;
  estimated_roundtrip_ms?: number;
  underruns: number;
  overruns: number;
  adc_clips: number;
  cpu_avg_pct: number;
  input_device: string;
  output_device: string;
  fallback_reason?: string | null;
  device_state?: 'ok' | 'lost' | string;
  connected?: boolean;
  mode_capabilities?: {
    wasapi_shared?: boolean;
    wasapi_shared_low_latency?: boolean;
    wasapi_exclusive?: boolean;
    asio?: boolean;
  };
}

export interface TunerData {
  enabled: boolean;
  frequency_hz: number;
  note: string;
  octave: number | null;
  cents: number;
  confidence: number;
  signal_present?: boolean;
  muted?: boolean;
  ref_hz?: number;
  strings?: Array<{ name: string; cents: number; ok: boolean }>;
}

export interface TelemetryData {
  input_peak_db: number;
  output_peak_db: number;
  cpu_usage_pct: number;
  gate_gr_db?: number;
  gate_state?: 'open' | 'hold' | 'closed';
  looper?: {
    state: string;
    layers: number;
    position_pct: number;
    reversed?: boolean;
    half_speed?: boolean;
  };
  waveform?: number[];
}

export interface ScopeData {
  mode: 'waveform' | 'transfer' | 'tonestack' | string;
  no_signal?: boolean;
  di_active?: boolean;
  output_active?: boolean;
  waveform?: {
    di: number[];
    output: number[];
  };
  transfer?: Array<{ x: number; y: number }>;
  tonestack?: Array<{ f: number; db: number }>;
  speaker_z?: Array<{ f: number; z: number }>;
}

export type EmitKind = 'full' | 'telemetry' | 'tuner' | 'scope' | 'param' | 'status';
export type StoreListener = (kind: EmitKind) => void;

export class AppStore {
  readonly transport: SonixTransport;
  readonly params: Record<string, unknown> = {};

  screen: ScreenName = 'main';
  selectedBlock: BlockId = 'amp';
  activeInspectorCh: 'A' | 'B' | 'C' | 'D' = 'D';
  ampChannelModels: Record<'A' | 'B' | 'C' | 'D', string> = {
    A: 'twin_ab763',
    B: 'ac30_tb',
    C: 'jcm800_2203',
    D: 'dual_rect',
  };
  selectedTube = 'V7';
  crtMode: 'waveform' | 'transfer' | 'tonestack' = 'waveform';
  scopeError: string | null = null;
  chassiTab: 'tube' | 'circuit' = 'tube';
  cabTab: 'mics' | 'cab' | 'ir' = 'mics';
  cabSpeakerIdx = 0;
  palcoFsMode: 'preset' | 'scene' | 'stomp' | 'hybrid' = 'hybrid';
  palcoTunerView: 'strobe' | 'poly' = 'strobe';
  palcoMode: 'stage' | 'tuner' = 'stage';
  engineStatusModalOpen = false;
  audioTab: 'dev' | 'in' | 'perf' = 'dev';
  libraryTab: 'amps' | 'pedals' | 'presets' = 'amps';
  vuMode: 'GR' | '+8' | '+4' | 'OFF' = 'GR';
  fullscreen = false;
  modifierOn = false;
  modifierSource = 'EXP 1';
  lastSavedStatus = '';
  bridgeError: string | null = null;
  deviceDisconnected = false;
  audioDeviceState: 'ok' | 'lost' = 'ok';
  isDraggingKnob = false;
  bootstrapped = false;
  reducedMotion =
    typeof window !== 'undefined' && typeof window.matchMedia === 'function'
      ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
      : false;

  presets: PresetItem[] = [];
  presetIdx = 0;
  activeSetlist = 'Setlist Show 2026';
  bpm = 120;
  private tapTimestamps: number[] = [];

  audioStatus: AudioStatusData = {
    backend: 'wasapi',
    mode: 'wasapi_shared',
    raw_processing: false,
    sample_rate: 48000,
    input_period_frames: 128,
    output_period_frames: 128,
    driver_input_latency_ms: 2.67,
    driver_output_latency_ms: 2.67,
    measured_roundtrip_ms: null,
    estimated_roundtrip_ms: 5.34,
    underruns: 0,
    overruns: 0,
    adc_clips: 0,
    cpu_avg_pct: 0,
    input_device: '',
    output_device: '',
    connected: true,
    mode_capabilities: {
      wasapi_shared: true,
      wasapi_shared_low_latency: true,
      wasapi_exclusive: false,
      asio: false,
    },
  };

  devices: Array<{ name: string; is_input?: boolean; is_output?: boolean; is_default?: boolean }> = [];
  midiPorts: string[] = [];
  midiConnectedPort: string | null = null;
  midiError: string | null = null;

  telemetry: TelemetryData = {
    input_peak_db: -60,
    output_peak_db: -60,
    cpu_usage_pct: 0,
    gate_gr_db: 0,
    gate_state: 'open',
    looper: { state: 'idle', layers: 0, position_pct: 0 },
  };

  tuner: TunerData = {
    enabled: false,
    frequency_hz: 0.0,
    note: '---',
    octave: null,
    cents: 0.0,
    confidence: 0.0,
    signal_present: false,
  };

  scope: ScopeData = {
    mode: 'waveform',
    no_signal: true,
  };

  irInfo = {
    fileName: 'Nenhum arquivo WAV carregado (usando sala de microfones virtual)',
    sampleRate: 48000,
    samples: 0,
  };

  calibrationWizard: {
    phase: 'idle' | 'running' | 'done';
    result: Record<string, unknown> | null;
    /** 1 = silêncio, 2 = tocar forte, 3 = resultado (Spec 051, CA-1). */
    step: 1 | 2 | 3;
    noiseFloorDb: number | null;
    error: string | null;
    /** Instante (ms) em que a captura do passo atual termina, para a barra de progresso. */
    stepEndsAt: number | null;
  } = { phase: 'idle', result: null, step: 1, noiseFloorDb: null, error: null, stepEndsAt: null };

  /** Espera do assistente de calibração; substituível nos testes (a UI real espera 3 s / 5 s). */
  calibrationWait: (ms: number) => Promise<void> = (ms) =>
    new Promise((resolve) => {
      setTimeout(resolve, ms);
    });

  /** Aviso não bloqueante (rejeição do motor), diferente de `bridgeError` (queda do IPC). */
  notice: { text: string; kind: BridgeFaultKind | 'info'; at: number } | null = null;
  noticeTtlMs = 6000;
  private noticeTimer: ReturnType<typeof setTimeout> | null = null;

  private listeners = new Set<StoreListener>();

  constructor(transport?: SonixTransport) {
    this.params['preset.active'] = 'brit800';
    for (const desc of CONTROLS_REGISTRY) {
      if (desc.backendKey && !(desc.backendKey in this.params)) {
        this.params[desc.backendKey] = desc.defaultValue;
      }
    }
    this.transport =
      transport ??
      new SonixTransport({
        onBridgeError: (err) => {
          if (this.bridgeError !== err) {
            this.bridgeError = err;
            this.emit('status');
          }
        },
        onNotice: (msg, kind) => this.pushNotice(msg, kind),
      });
  }

  /** Mostra um aviso em português claro, sem nome de chave interna; some sozinho (Spec 051, CA-2). */
  pushNotice(raw: string, kind: BridgeFaultKind | 'info' = 'validation'): void {
    this.notice = { text: this.friendlyMessage(raw), kind, at: Date.now() };
    if (this.noticeTimer) clearTimeout(this.noticeTimer);
    this.noticeTimer = setTimeout(() => {
      this.notice = null;
      this.noticeTimer = null;
      this.emit('status');
    }, this.noticeTtlMs);
    (this.noticeTimer as { unref?: () => void }).unref?.();
    this.emit('status');
  }

  /** Troca chaves internas (`amp_1.input_jack`) pelo rótulo do controle e traduz códigos do backend. */
  friendlyMessage(raw: string): string {
    const known: Record<string, string> = {
      missing_scene: 'Cena não informada.',
      invalid_scene: 'Cena inválida.',
      preset_not_found: 'Preset não encontrado.',
    };
    if (known[raw]) return known[raw];
    return raw.replace(/\b[a-z]+(?:_\d+)?(?:\.[a-z0-9_]+)+\b/g, (key) => {
      const desc = CONTROLS_REGISTRY.find((c) => c.backendKey === key);
      return desc ? `“${desc.label}”` : 'um controle';
    });
  }

  subscribe(fn: StoreListener): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  emit(kind: EmitKind = 'full'): void {
    for (const fn of this.listeners) fn(kind);
  }

  setDraggingKnob(dragging: boolean): void {
    this.isDraggingKnob = dragging;
  }

  getDescriptor(id: string): ControlDescriptor | undefined {
    return CONTROLS_REGISTRY.find((c) => c.id === id);
  }

  getParam<T>(key: string, fallback: T): T {
    const val = this.params[key];
    return val === undefined || val === null ? fallback : (val as T);
  }

  setParamByKey(key: string, value: unknown): void {
    this.params[key] = value;
    this.transport.setParam(key, value);
    this.emit(this.isDraggingKnob ? 'param' : 'full');
  }

  setControl(id: string, value: unknown): void {
    const desc = this.getDescriptor(id);
    if (!desc) return;
    if (desc.backendKey) {
      let normalized = value;
      if (desc.type === 'number' && typeof value === 'number' && desc.range) {
        const [lo, hi] = desc.range;
        normalized = Math.max(lo, Math.min(hi, value));
      }
      this.params[desc.backendKey] = normalized;
      if (desc.backendKey === 'amp_1.model' && typeof normalized === 'string') {
        const defaultPwr: Record<string, string> = {
          jcm800_2203: 'el34',
          plexi_1959: 'el34',
          twin_ab763: '6l6gc',
          dual_rect: '6l6gc',
          ac30_tb: 'el84_variant',
        };
        if (defaultPwr[normalized]) {
          this.params['amp_1.power_tube'] = defaultPwr[normalized];
        }
      }
      this.transport.setParam(desc.backendKey, normalized);
      const isContinuous = desc.kind === 'knob' || desc.kind === 'slider' || desc.kind === 'pad' || this.isDraggingKnob;
      this.emit(isContinuous ? 'param' : 'full');
    }
  }

  setScreen(screen: ScreenName): void {
    if (this.screen === screen) return;
    this.screen = screen;
    if (screen === 'chassi') {
      void this.transport.setStreamVisible?.(true, this.crtMode);
      void this.refreshScope();
    }
    this.emit();
  }

  selectBlock(blk: BlockId): void {
    this.selectedBlock = blk;
    this.emit();
  }

  async toggleFullscreen(): Promise<void> {
    this.fullscreen = !this.fullscreen;
    await this.transport.setFullscreen(this.fullscreen);
    this.emit();
  }

  async bootstrap(): Promise<void> {
    try {
      const state = await this.transport.getState();
      if (state && typeof state === 'object') {
        Object.assign(this.params, state);
      }
    } catch {
      // keep defaults
    }

    try {
      const pres = await this.transport.listPresets();
      const list = pres?.presets;
      if (Array.isArray(list)) {
        this.presets = list as PresetItem[];
      }
    } catch {
      this.presets = [];
    }

    try {
      const devs = await this.transport.getDevices();
      if (Array.isArray(devs?.devices)) {
        this.devices = devs.devices as Array<{
          name: string;
          is_input?: boolean;
          is_output?: boolean;
          is_default?: boolean;
        }>;
      }
    } catch {
      // offline mock or unavailable
    }

    await this.refreshMidiPorts();

    try {
      const st = await this.transport.getAudioStatus();
      if (st && typeof st === 'object') {
        Object.assign(this.audioStatus, st);
        this.deviceDisconnected = st.connected === false;
        this.audioDeviceState = st.device_state === 'lost' ? 'lost' : 'ok';
      }
    } catch {
      // ignore
    }

    try {
      const saved = typeof localStorage !== 'undefined' ? localStorage.getItem('sonix:preferred_audio_device') : null;
      if (saved) {
        const pref = JSON.parse(saved);
        if (pref && typeof pref === 'object' && pref.outputDevice) {
          const hasOut = this.devices.some((d) => d.name === pref.outputDevice);
          if (hasOut && this.audioStatus.output_device !== pref.outputDevice) {
            await this.transport.setAudioDevice({
              backend: pref.backend || 'wasapi',
              mode: pref.backend,
              sample_rate: pref.sampleRate || 48000,
              buffer_size: pref.bufferSize || 64,
              input_device: pref.inputDevice || undefined,
              output_device: pref.outputDevice,
            });
            const updatedSt = await this.transport.getAudioStatus();
            if (updatedSt && typeof updatedSt === 'object') Object.assign(this.audioStatus, updatedSt);
          }
        }
      } else {
        const curOut = this.audioStatus.output_device || '';
        if (/digital output|spdif|s\/pdif/i.test(curOut)) {
          const analogDev = this.devices.find(
            (d) => d.is_output && /alto-falantes|speakers|alto falantes|headphone|fone/i.test(d.name)
          );
          if (analogDev) {
            await this.transport.setAudioDevice({
              backend: this.audioStatus.backend || 'wasapi',
              mode: this.audioStatus.mode || 'wasapi_shared',
              sample_rate: this.audioStatus.sample_rate || 48000,
              buffer_size: this.audioStatus.output_period_frames || 64,
              input_device: this.audioStatus.input_device || undefined,
              output_device: analogDev.name,
            });
            const updatedSt = await this.transport.getAudioStatus();
            if (updatedSt && typeof updatedSt === 'object') Object.assign(this.audioStatus, updatedSt);
          }
        }
      }
    } catch {
      // ignore
    }

    try {
      await this.transport.startStreams({
        onTelemetry: (f: TelemetryFrame) => this.handleTelemetry(f),
        onTuner: (f: TunerFrame) => this.handleTuner(f),
        onScope: (f: ScopeFrame) => this.handleScope(f),
      });

      const fixture = (
        globalThis as {
          __SONIX_048_FIXTURE__?: {
            setlist?: string;
            presetId?: string;
            sceneIdx?: number;
            telemetry?: Record<string, unknown>;
            tuner?: Record<string, unknown>;
            scope?: Record<string, Record<string, unknown>>;
            calibrationDone?: boolean;
          };
        }
      ).__SONIX_048_FIXTURE__;
      if (fixture) {
        if (fixture.calibrationDone === true) {
          this.calibrationWizard.phase = 'done';
        }
        if (typeof fixture.setlist === 'string') {
          this.activeSetlist = fixture.setlist;
        }
        if (typeof fixture.presetId === 'string') {
          const idx = this.presets.findIndex((p) => p.id === fixture.presetId);
          if (idx >= 0) this.presetIdx = idx;
        }
        if (typeof fixture.sceneIdx === 'number') {
          const sc = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'][fixture.sceneIdx];
          if (sc) this.params['scene.active'] = sc;
        }
        if (fixture.telemetry) {
          this.applyTelemetry(fixture.telemetry);
        }
        if (fixture.tuner) {
          this.handleTuner({ seq: 1, payload: fixture.tuner });
        }
        if (fixture.scope && fixture.scope[this.crtMode]) {
          this.applyScopePayload(this.crtMode, fixture.scope[this.crtMode]);
        }
      }

      this.bootstrapped = true;
      this.emit();
    } finally {
      if (!this.bootstrapped) {
        this.bootstrapped = true;
        this.emit('status');
      }
    }
  }

  applyTelemetry(p: Record<string, unknown>): void {
    if (!p) return;
    if (typeof p.input_peak_db === 'number') this.telemetry.input_peak_db = p.input_peak_db;
    if (typeof p.output_peak_db === 'number') this.telemetry.output_peak_db = p.output_peak_db;
    const cpu =
      typeof p.cpu_usage_pct === 'number'
        ? p.cpu_usage_pct
        : typeof p.cpu_avg_pct === 'number'
          ? p.cpu_avg_pct
          : undefined;
    if (cpu !== undefined) {
      this.telemetry.cpu_usage_pct = cpu;
      this.audioStatus.cpu_avg_pct = cpu;
    }
    const gr =
      typeof p.gate_gr_db === 'number'
        ? p.gate_gr_db
        : typeof p.gain_reduction_db === 'number'
          ? p.gain_reduction_db
          : undefined;
    if (gr !== undefined) {
      this.telemetry.gate_gr_db = gr;
    }
    if (typeof p.gate_state === 'string') {
      this.telemetry.gate_state = p.gate_state as 'open' | 'hold' | 'closed';
    }
    if (Array.isArray(p.waveform)) {
      this.telemetry.waveform = p.waveform as number[];
    }
    if (p.looper && typeof p.looper === 'object') {
      this.telemetry.looper = p.looper as TelemetryData['looper'];
    }
    if (typeof p.underruns === 'number') {
      this.audioStatus.underruns = p.underruns;
    }
    if (p.device_state === 'lost' || p.connected === false) {
      this.audioDeviceState = 'lost';
      this.deviceDisconnected = true;
    } else if (p.device_state === 'ok' || p.connected === true) {
      this.audioDeviceState = 'ok';
      this.deviceDisconnected = false;
    }
    this.emit('telemetry');
  }

  private handleTelemetry(frame: TelemetryFrame): void {
    this.applyTelemetry(frame.payload);
  }

  applyTunerPayload(p: Record<string, unknown>): void {
    const enabled = Boolean(p.enabled ?? p.active ?? this.getParam('tuner.enabled', false));
    const freqRaw = Number(p.frequency_hz ?? 0);
    const freq = Number.isFinite(freqRaw) && freqRaw > 0 ? freqRaw : 0;

    let note = String(p.note_letter ?? p.note ?? '---').trim();
    let octave: number | null =
      p.octave === null || p.octave === undefined
        ? null
        : Number.isFinite(Number(p.octave))
          ? Number(p.octave)
          : null;

    if (note !== '---' && octave === null) {
      const m = /^([A-G]#?)(-?\d+)$/.exec(note);
      if (m) {
        note = m[1];
        octave = Number(m[2]);
      }
    } else if (note !== '---') {
      const m = /^([A-G]#?)(-?\d+)$/.exec(note);
      if (m) {
        note = m[1];
      }
    }

    const hasSignal =
      p.signal_present !== undefined
        ? Boolean(p.signal_present) && freq > 0 && note !== '---'
        : freq > 0 && note !== '---';

    const centsRaw = Number(p.cents ?? 0);
    this.tuner = {
      enabled,
      signal_present: hasSignal,
      muted: p.muted !== undefined ? Boolean(p.muted) : undefined,
      ref_hz: typeof p.ref_hz === 'number' && Number.isFinite(p.ref_hz) ? p.ref_hz : undefined,
      frequency_hz: hasSignal ? freq : 0,
      note: hasSignal ? note : '---',
      octave: hasSignal ? octave : null,
      cents: hasSignal && Number.isFinite(centsRaw) ? centsRaw : 0,
      confidence: hasSignal ? Number(p.confidence ?? 1) : 0,
      strings: Array.isArray(p.strings) ? (p.strings as TunerData['strings']) : undefined,
    };
    this.emit('tuner');
  }

  private handleTuner(frame: TunerFrame): void {
    const p = frame.payload;
    if (!p) return;
    this.applyTunerPayload(p);
  }

  applyScopePayload(mode: string, p: Record<string, unknown>): void {
    this.scopeError = null;
    const wf =
      p.waveform && typeof p.waveform === 'object'
        ? (p.waveform as { di?: unknown; output?: unknown })
        : undefined;
    const diArr = wf && Array.isArray(wf.di) ? (wf.di as number[]) : [];
    const outArr = wf && Array.isArray(wf.output) ? (wf.output as number[]) : [];
    const diPeak = diArr.reduce((acc, v) => Math.max(acc, Math.abs(Number(v) || 0)), 0);
    const outPeak = outArr.reduce((acc, v) => Math.max(acc, Math.abs(Number(v) || 0)), 0);
    const explicitNoSignal = Boolean(p.no_signal);
    const wfHasSignal = mode === 'waveform' ? !explicitNoSignal && (diPeak > 1e-4 || outPeak > 1e-4) : true;

    this.scope = {
      mode,
      no_signal: mode === 'waveform' ? !wfHasSignal : explicitNoSignal,
      di_active: typeof p.di_active === 'boolean' ? p.di_active : diPeak > 1e-4,
      output_active: typeof p.output_active === 'boolean' ? p.output_active : outPeak > 1e-4,
      waveform: wf
        ? {
            di: wfHasSignal ? diArr : [],
            output: wfHasSignal ? outArr : [],
          }
        : undefined,
      transfer: Array.isArray(p.transfer)
        ? (p.transfer as Array<{ x: number; y: number }>)
        : undefined,
      tonestack: Array.isArray(p.tonestack)
        ? (p.tonestack as Array<{ f: number; db: number }>)
        : undefined,
      speaker_z: Array.isArray(p.speaker_z)
        ? (p.speaker_z as Array<{ f: number; z: number }>)
        : undefined,
    };
    this.emit('scope');
  }

  private handleScope(frame: ScopeFrame): void {
    const p = frame.payload;
    if (!p) return;
    this.applyScopePayload(frame.mode, p);
  }

  async refreshScope(): Promise<void> {
    try {
      const payload = await this.transport.getScope(this.crtMode);
      this.applyScopePayload(this.crtMode, payload);
    } catch (err) {
      this.scopeError = err instanceof Error ? err.message : String(err);
      this.scope = { mode: this.crtMode };
      this.emit('scope');
    }
  }

  /** Carrega um preset; devolve `true` só quando o backend aceitou (Spec 051, CA-5). */
  async loadPreset(id: string): Promise<boolean> {
    let ok = false;
    try {
      const res = await this.transport.loadPreset({ id });
      ok = true;
      this.params['preset.active'] = id;
      if (res && typeof res.state === 'object' && res.state !== null) {
        Object.assign(this.params, res.state);
      }
    } catch {
      const name = this.presets.find((p) => p.id === id)?.name ?? id;
      this.pushNotice(`Não foi possível carregar o preset “${name}”.`, 'validation');
    }
    this.emit('full');
    return ok;
  }

  async selectScene(sceneId: string): Promise<void> {
    try {
      // O backend lê `scene` (letra A–H) ou `index` (Spec 051, CA-3).
      const res = await this.transport.selectScene({ scene: sceneId });
      this.params['scene.active'] = sceneId;
      if (res && typeof res.state === 'object' && res.state !== null) {
        Object.assign(this.params, res.state);
      }
    } catch {
      // A cena ativa só muda quando o backend aceita; o aviso já foi emitido pelo transporte.
    }
    this.emit('full');
  }

  async loadPresetAt(index: number): Promise<void> {
    if (this.presets.length === 0) return;
    const target = ((index % this.presets.length) + this.presets.length) % this.presets.length;
    const item = this.presets[target];
    // O índice (e o nome exibido) só avança depois do carregamento bem-sucedido.
    if (await this.loadPreset(item.id)) {
      this.presetIdx = target;
      this.emit('full');
    }
  }

  /** Assistente de calibração: begin → silêncio (3 s) → palhetada (5 s) → resultado (Spec 051, CA-1). */
  async runCalibrationWizard(): Promise<void> {
    const w = this.calibrationWizard;
    if (w.phase === 'running') return;
    w.phase = 'running';
    w.step = 1;
    w.error = null;
    w.result = null;
    w.noiseFloorDb = null;
    w.stepEndsAt = Date.now() + 3000;
    this.emit('full');
    try {
      await this.transport.calibrateInput({ phase: 'begin' });
      await this.calibrationWait(3000);
      const sil = await this.transport.calibrateInput({ phase: 'silence' });
      if (typeof sil?.noise_floor_rms_db === 'number') w.noiseFloorDb = sil.noise_floor_rms_db;
      w.step = 2;
      w.stepEndsAt = Date.now() + 5000;
      this.emit('full');
      await this.calibrationWait(5000);
      const res = await this.transport.calibrateInput({ phase: 'strum' });
      w.result = res;
      w.step = 3;
      w.phase = 'done';
      w.stepEndsAt = null;
      // O backend já aplicou boost e gate ao DSP; refletimos o estado real na tela.
      try {
        const st = await this.transport.getState();
        if (st && typeof st === 'object') Object.assign(this.params, st);
        const aud = await this.transport.getAudioStatus();
        if (aud && typeof aud === 'object') Object.assign(this.audioStatus, aud);
      } catch {
        // atualização opcional
      }
    } catch (err) {
      w.phase = 'idle';
      w.stepEndsAt = null;
      const raw = err instanceof Error ? err.message : String(err);
      w.error = this.friendlyMessage(classifyBridgeError(raw).message);
    }
    this.emit('full');
  }

  async saveActivePreset(): Promise<void> {
    const cur = this.presets[this.presetIdx];
    if (!cur) {
      this.lastSavedStatus = 'Nenhum preset selecionado';
      this.emit('full');
      return;
    }
    try {
      await this.transport.savePreset({
        id: cur.id,
        name: cur.name,
        state: this.params,
      });
      this.lastSavedStatus = `Salvo: ${cur.name}`;
    } catch {
      this.lastSavedStatus = 'Erro ao salvar preset';
    }
    this.emit('full');
  }

  applyTapBpm(bpmRaw: number): void {
    const bpm = Math.max(40, Math.min(240, Math.round(bpmRaw)));
    this.bpm = bpm;
    // BPM maior = delay mais curto = velocidade da fita (repeat_rate) maior (Spec 051, CA-18).
    const rate = Math.max(0, Math.min(10, Number((((bpm - 40) / 200) * 10).toFixed(1))));
    this.setParamByKey('dly_1.repeat_rate', rate);
  }

  tapTempo(): void {
    const now = performance.now();
    this.tapTimestamps = this.tapTimestamps.filter((t) => now - t < 2500);
    this.tapTimestamps.push(now);
    if (this.tapTimestamps.length >= 2) {
      let sum = 0;
      for (let i = 1; i < this.tapTimestamps.length; i++) {
        sum += this.tapTimestamps[i] - this.tapTimestamps[i - 1];
      }
      const avgMs = sum / (this.tapTimestamps.length - 1);
      const bpm = Math.max(40, Math.min(240, Math.round(60000 / avgMs)));
      this.applyTapBpm(bpm);
    } else {
      this.emit();
    }
  }

  async reconnectAudioDevice(): Promise<void> {
    try {
      await this.transport.setAudioDevice({
        backend: this.audioStatus.backend || 'wasapi',
        sample_rate: this.audioStatus.sample_rate || 48000,
        buffer_size: this.audioStatus.input_period_frames || 64,
        input_device: this.audioStatus.input_device,
        output_device: this.audioStatus.output_device,
      });
      this.deviceDisconnected = false;
      this.audioDeviceState = 'ok';
    } catch {
      this.deviceDisconnected = true;
      this.audioDeviceState = 'lost';
    }
    this.emit();
  }

  async refreshMidiPorts(): Promise<void> {
    try {
      const res = await this.transport.listMidiPorts();
      if (Array.isArray(res?.ports)) {
        this.midiPorts = res.ports.map((p) => String(p));
      }
      this.midiConnectedPort =
        typeof res?.connected_port === 'string' ? res.connected_port : null;
    } catch {
      // offline mock or unavailable
    }
  }

  async connectMidiPort(portName: string): Promise<void> {
    this.midiError = null;
    try {
      if (!portName) {
        await this.transport.disconnectMidiPort();
        this.midiConnectedPort = null;
      } else {
        const res = await this.transport.connectMidiPort(portName);
        this.midiConnectedPort =
          typeof res?.connected_port === 'string' ? res.connected_port : portName;
      }
    } catch (err) {
      this.midiError = err instanceof Error ? err.message : String(err);
    }
    this.emit('status');
  }

  async exportPackageDialog(presetIds?: string[]): Promise<boolean> {
    const cur = this.presets[this.presetIdx];
    const ids = presetIds && presetIds.length > 0 ? presetIds : cur ? [cur.id] : [];
    if (ids.length === 0) {
      this.pushNotice('Selecione ao menos um preset para exportar o pacote.', 'validation');
      return false;
    }
    try {
      const targetPath = await this.transport.saveFileDialog(
        [{ name: 'Pacote SONIX PRO (.sforgepkg)', extensions: ['sforgepkg'] }],
        `${ids[0]}.sforgepkg`,
      );
      if (!targetPath) return false;
      const res = await this.transport.exportPackage(ids, targetPath);
      this.lastSavedStatus = `Pacote exportado (${Number(res?.presets_exported ?? ids.length)} preset(s))`;
      this.emit('full');
      return true;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.pushNotice(this.friendlyMessage(classifyBridgeError(msg).message), 'validation');
      return false;
    }
  }

  async importPackageDialog(): Promise<boolean> {
    try {
      const srcPath = await this.transport.openFileDialog([
        { name: 'Pacote SONIX PRO (.sforgepkg)', extensions: ['sforgepkg'] },
      ]);
      if (!srcPath) return false;
      const res = await this.transport.importPackage(srcPath);
      const list = await this.transport.listPresets();
      if (Array.isArray(list?.presets)) {
        this.presets = list.presets as PresetItem[];
      }
      const importedIds = Array.isArray(res?.imported_presets)
        ? res.imported_presets.map((x) => String(x))
        : [];
      if (importedIds.length > 0) {
        const idx = this.presets.findIndex((p) => p.id === importedIds[0]);
        if (idx >= 0) this.presetIdx = idx;
      }
      this.lastSavedStatus = `Pacote importado (${importedIds.length} preset(s))`;
      this.emit('full');
      return true;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.pushNotice(this.friendlyMessage(classifyBridgeError(msg).message), 'validation');
      return false;
    }
  }
}
