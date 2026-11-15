import { CONTROLS_REGISTRY, type ControlDescriptor } from './controls.registry.ts';
import {
  SonixTransport,
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
  | 'pitch';

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
  octave: number;
  cents: number;
  confidence: number;
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
  audioTab: 'dev' | 'in' | 'perf' = 'dev';
  libraryTab: 'amps' | 'pedals' = 'amps';
  vuMode: 'GR' | '+8' | '+4' | 'OFF' = 'GR';
  fullscreen = false;
  modifierOn = true;
  modifierSource = 'EXP 1';
  lastSavedStatus = '';
  bridgeError: string | null = null;
  deviceDisconnected = false;
  isDraggingKnob = false;

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
    frequency_hz: 110.0,
    note: 'A',
    octave: 2,
    cents: 0.0,
    confidence: 0.0,
  };

  scope: ScopeData = {
    mode: 'waveform',
  };

  irInfo = {
    fileName: 'Nenhum arquivo WAV carregado (usando sala de microfones virtual)',
    sampleRate: 48000,
    samples: 0,
  };

  calibrationWizard: {
    phase: 'idle' | 'running' | 'done';
    result: Record<string, unknown> | null;
  } = { phase: 'idle', result: null };

  private listeners = new Set<StoreListener>();

  constructor(transport?: SonixTransport) {
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
      this.transport.setParam(desc.backendKey, normalized);
      const isContinuous = desc.kind === 'knob' || desc.kind === 'slider' || this.isDraggingKnob;
      this.emit(isContinuous ? 'param' : 'full');
    }
  }

  setScreen(screen: ScreenName): void {
    if (this.screen === screen) return;
    this.screen = screen;
    if (screen === 'chassi') {
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

    try {
      const st = await this.transport.getAudioStatus();
      if (st && typeof st === 'object') {
        Object.assign(this.audioStatus, st);
        this.deviceDisconnected = st.connected === false;
      }
    } catch {
      // ignore
    }

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

    this.emit();
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
    if (p.connected === false) {
      this.deviceDisconnected = true;
    }
    this.emit('telemetry');
  }

  private handleTelemetry(frame: TelemetryFrame): void {
    this.applyTelemetry(frame.payload);
  }

  private handleTuner(frame: TunerFrame): void {
    const p = frame.payload;
    if (!p) return;
    this.tuner = {
      enabled: Boolean(p.enabled ?? this.getParam('tuner.enabled', false)),
      frequency_hz: Number(p.frequency_hz ?? this.tuner.frequency_hz),
      note: String(p.note ?? this.tuner.note),
      octave: Number(p.octave ?? this.tuner.octave),
      cents: Number(p.cents ?? this.tuner.cents),
      confidence: Number(p.confidence ?? 0),
      strings: Array.isArray(p.strings) ? (p.strings as TunerData['strings']) : undefined,
    };
    this.emit('tuner');
  }

  applyScopePayload(mode: string, p: Record<string, unknown>): void {
    this.scopeError = null;
    const wf =
      p.waveform && typeof p.waveform === 'object'
        ? (p.waveform as { di?: unknown; output?: unknown })
        : undefined;
    this.scope = {
      mode,
      waveform: wf
        ? {
            di: Array.isArray(wf.di) ? (wf.di as number[]) : [],
            output: Array.isArray(wf.output) ? (wf.output as number[]) : [],
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

  async loadPreset(id: string): Promise<void> {
    this.params['preset.active'] = id;
    try {
      const res = await this.transport.loadPreset({ id });
      if (res && typeof res.state === 'object' && res.state !== null) {
        Object.assign(this.params, res.state);
      }
    } catch {
      // handled by transport bridgeError
    }
    this.emit('full');
  }

  async selectScene(sceneId: string): Promise<void> {
    this.params['scene.active'] = sceneId;
    try {
      const res = await this.transport.selectScene({ scene_id: sceneId });
      if (res && typeof res.state === 'object' && res.state !== null) {
        Object.assign(this.params, res.state);
      } else {
        this.transport.setParam('scene.active', sceneId);
      }
    } catch {
      this.transport.setParam('scene.active', sceneId);
    }
    this.emit('full');
  }

  async loadPresetAt(index: number): Promise<void> {
    if (this.presets.length === 0) return;
    this.presetIdx = ((index % this.presets.length) + this.presets.length) % this.presets.length;
    const item = this.presets[this.presetIdx];
    await this.loadPreset(item.id);
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
      this.bpm = bpm;
      const rate = Math.max(0, Math.min(10, Number((((240 - bpm) / 200) * 10).toFixed(1))));
      this.setParamByKey('dly_1.repeat_rate', rate);
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
    } catch {
      this.deviceDisconnected = true;
    }
    this.emit();
  }
}
