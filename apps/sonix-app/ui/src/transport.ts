import { Channel, invoke as tauriInvoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

export type InvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
export type RafFn = (cb: (time: number) => void) => number;

export interface TelemetryFrame {
  seq: number;
  payload: Record<string, unknown>;
}

export interface TunerFrame {
  seq: number;
  payload: Record<string, unknown>;
}

export interface ScopeFrame {
  seq: number;
  mode: string;
  payload: Record<string, unknown>;
}

export function isTauriRuntime(): boolean {
  return (
    typeof window !== 'undefined' &&
    ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)
  );
}

/**
 * Coalesce UI parameter writes to at most one `set_params` IPC invocation
 * per animation frame, retaining the latest value per key (Spec 033, CA-8).
 */
export class ParamCoalescer {
  private pending: Record<string, unknown> = {};
  private scheduled = false;
  private inflight: Promise<void> | null = null;
  private readonly invokeFn: InvokeFn;
  private readonly rafFn: RafFn;
  private readonly onResult?: (res: Record<string, unknown>) => void;
  private readonly onError?: (err: string) => void;

  constructor(opts: {
    invokeFn?: InvokeFn;
    rafFn?: RafFn;
    onResult?: (res: Record<string, unknown>) => void;
    onError?: (err: string) => void;
  } = {}) {
    this.invokeFn = opts.invokeFn ?? defaultInvoke;
    this.rafFn =
      opts.rafFn ??
      ((cb) => {
        if (typeof globalThis.requestAnimationFrame === 'function') {
          return globalThis.requestAnimationFrame(cb);
        }
        return Number(globalThis.setTimeout(() => cb(Date.now()), 16));
      });
    this.onResult = opts.onResult;
    this.onError = opts.onError;
  }

  enqueue(key: string, value: unknown): void {
    this.pending[key] = value;
    this.scheduleFlush();
  }

  enqueueBatch(patch: Record<string, unknown>): void {
    Object.assign(this.pending, patch);
    this.scheduleFlush();
  }

  private scheduleFlush(): void {
    if (this.scheduled || this.inflight) return;
    this.scheduled = true;
    this.rafFn(() => {
      void this.flushNow();
    });
  }

  async flushNow(): Promise<void> {
    while (this.inflight) {
      await this.inflight;
    }
    this.scheduled = false;
    const keys = Object.keys(this.pending);
    if (keys.length === 0) return;

    const batch = this.pending;
    this.pending = {};
    const run = async () => {
      try {
        const res = (await this.invokeFn('set_params', { params: batch })) as
          | Record<string, unknown>
          | undefined;
        if (res && this.onResult) this.onResult(res);
      } catch (err) {
        this.onError?.(err instanceof Error ? err.message : String(err));
      }
    };
    this.inflight = run();
    try {
      await this.inflight;
    } finally {
      this.inflight = null;
    }

    if (Object.keys(this.pending).length > 0 && !this.scheduled) {
      this.scheduleFlush();
    }
  }
}

async function defaultInvoke(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<unknown> {
  if (isTauriRuntime()) {
    return tauriInvoke(cmd, args);
  }
  return offlinePreviewStub(cmd, args);
}

function offlinePreviewStub(
  cmd: string,
  args?: Record<string, unknown>,
): Record<string, unknown> {
  const fixture = (globalThis as { __SONIX_048_FIXTURE__?: Record<string, unknown> })
    .__SONIX_048_FIXTURE__;
  switch (cmd) {
    case 'set_params':
      return {
        applied: Object.keys((args?.params as Record<string, unknown>) ?? {}),
        ignored: [],
      };
    case 'get_state':
      return (fixture?.state as Record<string, unknown>) ?? {};
    case 'get_audio_status':
      if (fixture?.audioStatus) {
        return fixture.audioStatus as Record<string, unknown>;
      }
      return {
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
        mode_capabilities: {
          wasapi_shared: true,
          wasapi_shared_low_latency: true,
          wasapi_exclusive: false,
          asio: false,
        },
        connected: false,
      };
    case 'get_devices':
      return {
        devices: (fixture?.devices as unknown[]) ?? [],
      };
    case 'list_presets':
      return {
        presets: (fixture?.presets as unknown[]) ?? [],
      };
    case 'get_scope': {
      const mode = String(args?.mode ?? 'waveform');
      const scopeFix = fixture?.scope as Record<string, unknown> | undefined;
      if (scopeFix && scopeFix[mode]) {
        return scopeFix[mode] as Record<string, unknown>;
      }
      return {};
    }
    case 'measure_latency':
      return {
        ok: true,
        driver_input_latency_ms: 2.67,
        driver_output_latency_ms: 2.67,
        algorithmic_latency_ms: 0.33,
        estimated_roundtrip_ms: 5.67,
        measured_roundtrip_ms: null,
        sample_rate: 48000,
      };
    case 'calibrate_input':
      return {
        ok: true,
        recommended_boost: 2.5,
        recommended_threshold_db: -62,
        noise_floor_db: -74,
        peak_raw_db: -18,
        diagnosis: 'ok',
      };
    default:
      return { ok: true };
  }
}

export class SonixTransport {
  readonly coalescer: ParamCoalescer;
  private readonly invokeFn: InvokeFn;
  private onBridgeError?: (msg: string | null) => void;
  private lastSeq = { telemetry: 0, tuner: 0, scope: 0 };

  constructor(opts: {
    invokeFn?: InvokeFn;
    rafFn?: RafFn;
    onBridgeError?: (msg: string | null) => void;
  } = {}) {
    this.invokeFn = opts.invokeFn ?? defaultInvoke;
    this.onBridgeError = opts.onBridgeError;
    this.coalescer = new ParamCoalescer({
      invokeFn: this.invokeFn,
      rafFn: opts.rafFn,
      onResult: () => this.onBridgeError?.(null),
      onError: (err) => this.onBridgeError?.(err),
    });
  }

  private async call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    try {
      const res = (await this.invokeFn(cmd, args)) as T;
      this.onBridgeError?.(null);
      return res;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.onBridgeError?.(msg);
      throw err;
    }
  }

  setParam(key: string, value: unknown): void {
    this.coalescer.enqueue(key, value);
  }

  setParamsPatch(patch: Record<string, unknown>): void {
    this.coalescer.enqueueBatch(patch);
  }

  getStatus(): Promise<Record<string, unknown>> {
    return this.call('get_status');
  }

  getAudioStatus(): Promise<Record<string, unknown>> {
    return this.call('get_audio_status');
  }

  getTelemetry(): Promise<Record<string, unknown>> {
    return this.call('get_telemetry');
  }

  getDevices(): Promise<Record<string, unknown>> {
    return this.call('get_devices');
  }

  getState(): Promise<Record<string, unknown>> {
    return this.call('get_state');
  }

  getTuner(): Promise<Record<string, unknown>> {
    return this.call('get_tuner');
  }

  getGraph(): Promise<Record<string, unknown>> {
    return this.call('get_graph');
  }

  getScope(mode?: string): Promise<Record<string, unknown>> {
    return this.call('get_scope', { mode: mode ?? null });
  }

  loadPreset(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return this.call('load_preset', { payload });
  }

  savePreset(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return this.call('save_preset', { payload });
  }

  importPreset(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return this.call('import_preset', { payload });
  }

  exportPreset(id: string): Promise<Record<string, unknown>> {
    return this.call('export_preset', { id });
  }

  listPresets(): Promise<Record<string, unknown>> {
    return this.call('list_presets');
  }

  selectScene(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return this.call('select_scene', { payload });
  }

  loadIr(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return this.call('load_ir', { payload });
  }

  setInputDi(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return this.call('set_input_di', { payload });
  }

  calibrateInput(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return this.call('calibrate_input', { payload });
  }

  measureLatency(): Promise<Record<string, unknown>> {
    return this.call('measure_latency');
  }

  setAudioDevice(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return this.call('set_audio_device', { payload });
  }

  setStreamVisible(visible: boolean, scopeMode?: string): Promise<Record<string, unknown>> {
    return this.call('set_stream_visible', {
      visible,
      scopeMode: scopeMode ?? null,
    });
  }

  async openFileDialog(filters: Array<{ name: string; extensions: string[] }>): Promise<string | null> {
    if (!isTauriRuntime()) return null;
    const selected = await this.invokeFn('plugin:dialog|open', {
      options: { multiple: false, filters },
    });
    if (typeof selected === 'string') return selected;
    return null;
  }

  async readFileBytes(path: string): Promise<number[]> {
    if (!isTauriRuntime()) return [];
    const bytes = await this.invokeFn('plugin:fs|read_file', { path });
    return Array.isArray(bytes) ? (bytes as number[]) : [];
  }

  async startStreams(handlers: {
    onTelemetry: (f: TelemetryFrame) => void;
    onTuner: (f: TunerFrame) => void;
    onScope: (f: ScopeFrame) => void;
  }): Promise<void> {
    if (!isTauriRuntime()) {
      return;
    }

    const chTelemetry = new Channel<TelemetryFrame>();
    chTelemetry.onmessage = (frame) => {
      if (frame.seq > this.lastSeq.telemetry) {
        this.lastSeq.telemetry = frame.seq;
        handlers.onTelemetry(frame);
      }
    };

    const chTuner = new Channel<TunerFrame>();
    chTuner.onmessage = (frame) => {
      if (frame.seq > this.lastSeq.tuner) {
        this.lastSeq.tuner = frame.seq;
        handlers.onTuner(frame);
      }
    };

    const chScope = new Channel<ScopeFrame>();
    chScope.onmessage = (frame) => {
      if (frame.seq > this.lastSeq.scope) {
        this.lastSeq.scope = frame.seq;
        handlers.onScope(frame);
      }
    };

    await this.call('start_streams', {
      onTelemetry: chTelemetry,
      onTuner: chTuner,
      onScope: chScope,
    });
  }

  async minimizeWindow(): Promise<void> {
    if (!isTauriRuntime()) return;
    await getCurrentWindow().minimize();
  }

  async toggleMaximizeWindow(): Promise<void> {
    if (!isTauriRuntime()) return;
    await getCurrentWindow().toggleMaximize();
  }

  async closeWindow(): Promise<void> {
    if (!isTauriRuntime()) return;
    await getCurrentWindow().close();
  }

  async setFullscreen(fullscreen: boolean): Promise<void> {
    if (!isTauriRuntime()) return;
    await getCurrentWindow().setFullscreen(fullscreen);
  }
}
