import { CONTROLS_REGISTRY, type ControlDescriptor } from './controls.registry';
import {
  SonixTransport,
  type ScopeFrame,
  type TelemetryFrame,
  type TunerFrame,
} from './transport';

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
  | 'chorus';

export interface PresetItem {
  id: string;
  name: string;
  setlist?: string;
}

export interface AudioStatusData {
  backend: string;
  mode: string;
  sample_rate: number;
  input_period_frames: number;
  output_period_frames: number;
  driver_input_latency_ms: number;
  driver_output_latency_ms: number;
  measured_roundtrip_ms?: number | null;
  underruns: number;
  overruns: number;
  adc_clips: number;
  cpu_avg_pct: number;
  input_device: string;
  output_device: string;
  fallback_reason?: string | null;
  connected?: boolean;
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
  cpu_avg_pct: number;
  gain_reduction_db?: number;
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
  mode: string;
  waveform_in?: number[];
  waveform_out?: number[];
  transfer_x?: number[];
  transfer_y?: number[];
  tonestack_freqs?: number[];
  tonestack_db?: number[];
}

export type StoreListener = () => void;

const DEFAULT_PRESETS: PresetItem[] = [
  { id: 'brit800', name: 'P01 Brit 800 JCM2203 Crunch', setlist: 'SHOW 2026' },
  { id: 'plexi_jumped', name: 'P02 Plexi 1959 Jumped Lead', setlist: 'SHOW 2026' },
  { id: 'cali_modern', name: 'P03 Cali Dual Recto Modern Metal', setlist: 'SHOW 2026' },
  { id: 'twin_clean', name: 'P04 US Twin AB763 Clean Sparkle', setlist: 'SHOW 2026' },
  { id: 'ac30_chime', name: 'P05 UK 30 Top Boost Chime', setlist: 'SHOW 2026' },
];

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
  crtMode: 'wave' | 'transfer' | 'tonestack' = 'wave';
  chassiTab: 'tube' | 'circuit' = 'tube';
  cabTab: 'mics' | 'cab' | 'ir' = 'mics';
  cabSpeakerIdx = 0;
  palcoFsMode: 'preset' | 'scene' | 'stomp' | 'hybrid' = 'hybrid';
  palcoTunerView: 'strobe' | 'poly' = 'strobe';
  audioTab: 'dev' | 'in' | 'perf' = 'in';
  libraryTab: 'amps' | 'pedals' = 'amps';
  vuMode: 'GR' | '+8' | '+4' | 'OFF' = 'GR';
  fullscreen = false;
  modifierOn = true;
  modifierSource = 'EXP 1';
  lastSavedStatus = '';
  bridgeError: string | null = null;
  deviceDisconnected = false;

  presets: PresetItem[] = [...DEFAULT_PRESETS];
  presetIdx = 0;
  bpm = 120;
  private tapTimestamps: number[] = [];

  audioStatus: AudioStatusData = {
    backend: 'wasapi',
    mode: 'wasapi_exclusive',
    sample_rate: 48000,
    input_period_frames: 64,
    output_period_frames: 64,
    driver_input_latency_ms: 1.33,
    driver_output_latency_ms: 1.33,
    measured_roundtrip_ms: null,
    underruns: 0,
    overruns: 0,
    adc_clips: 0,
    cpu_avg_pct: 5.2,
    input_device: 'Audio Device',
    output_device: 'Audio Device',
    connected: true,
  };

  devices: Array<{ name: string; is_input?: boolean; is_output?: boolean; is_default?: boolean }> = [];

  telemetry: TelemetryData = {
    input_peak_db: -28,
    output_peak_db: -18,
    cpu_avg_pct: 5.2,
    gain_reduction_db: 0,
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
    mode: 'transfer',
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

  constructor() {
    for (const desc of CONTROLS_REGISTRY) {
      if (desc.backendKey && !(desc.backendKey in this.params)) {
        this.params[desc.backendKey] = desc.defaultValue;
      }
    }
    this.transport = new SonixTransport({
      onBridgeError: (err) => {
        if (this.bridgeError !== err) {
          this.bridgeError = err;
          this.emit();
        }
      },
    });
  }

  subscribe(fn: StoreListener): () => void {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  emit(): void {
    for (const fn of this.listeners) fn();
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
    this.emit();
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
      this.emit();
    }
  }

  setScreen(screen: ScreenName): void {
    if (this.screen === screen) return;
    this.screen = screen;
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
      if (Array.isArray(list) && list.length > 0) {
        this.presets = list as PresetItem[];
      }
    } catch {
      // keep default presets
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

    this.emit();
  }

  private handleTelemetry(frame: TelemetryFrame): void {
    const p = frame.payload;
    if (!p) return;
    if (typeof p.input_peak_db === 'number') this.telemetry.input_peak_db = p.input_peak_db;
    if (typeof p.output_peak_db === 'number') this.telemetry.output_peak_db = p.output_peak_db;
    if (typeof p.cpu_avg_pct === 'number') {
      this.telemetry.cpu_avg_pct = p.cpu_avg_pct;
      this.audioStatus.cpu_avg_pct = p.cpu_avg_pct;
    }
    if (typeof p.gain_reduction_db === 'number') {
      this.telemetry.gain_reduction_db = p.gain_reduction_db;
    }
    if (typeof p.gate_state === 'string') {
      this.telemetry.gate_state = p.gate_state as 'open' | 'hold' | 'closed';
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
    this.emit();
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
    this.emit();
  }

  private handleScope(frame: ScopeFrame): void {
    const p = frame.payload;
    if (!p) return;
    this.scope = {
      mode: frame.mode,
      waveform_in: Array.isArray(p.waveform_in) ? (p.waveform_in as number[]) : undefined,
      waveform_out: Array.isArray(p.waveform_out) ? (p.waveform_out as number[]) : undefined,
      transfer_x: Array.isArray(p.transfer_x) ? (p.transfer_x as number[]) : undefined,
      transfer_y: Array.isArray(p.transfer_y) ? (p.transfer_y as number[]) : undefined,
      tonestack_freqs: Array.isArray(p.tonestack_freqs) ? (p.tonestack_freqs as number[]) : undefined,
      tonestack_db: Array.isArray(p.tonestack_db) ? (p.tonestack_db as number[]) : undefined,
    };
    this.emit();
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
    this.emit();
  }

  async loadPresetAt(index: number): Promise<void> {
    if (this.presets.length === 0) return;
    this.presetIdx = ((index % this.presets.length) + this.presets.length) % this.presets.length;
    const item = this.presets[this.presetIdx];
    this.params['preset.active'] = item.id;
    try {
      const res = await this.transport.loadPreset({ id: item.id });
      if (res && typeof res.state === 'object' && res.state !== null) {
        Object.assign(this.params, res.state);
      }
    } catch {
      // error indicator handled by transport
    }
    this.emit();
  }

  async saveActivePreset(): Promise<void> {
    const cur = this.presets[this.presetIdx] ?? DEFAULT_PRESETS[0];
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
    this.emit();
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
