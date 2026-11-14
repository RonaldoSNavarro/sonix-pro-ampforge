import type { AppStore, AudioStatusData } from '../store';
import {
  makeChipButton,
  makeControlSelect,
  makeControlSlider,
  makeLedMeter,
  makePanelCard,
  num,
} from './_helpers';

export function renderAudioScreen(store: AppStore): HTMLElement {
  const root = document.createElement('div');
  root.dataset.screen = 'audio';
  root.style.cssText = 'height:100%;display:flex;flex-direction:column;gap:10px;padding:10px;min-height:0;overflow:hidden;';

  const hdr = document.createElement('div');
  hdr.className = 'm-metal';
  hdr.style.cssText =
    'display:flex;align-items:center;justify-content:space-between;padding:8px 12px;border-radius:8px;flex:none;';
  hdr.innerHTML =
    '<div><div class="m-osw" style="font-size:15px;font-weight:700;letter-spacing:0.08em;">CONFIGURAÇÃO DE ÁUDIO, CALIBRAÇÃO & DESEMPENHO</div><div style="font-size:11px;color:#8e96a0;">Interface WASAPI/ASIO nativa via IPC Tauri · Zero HTTP local</div></div>';

  const tabs = document.createElement('div');
  tabs.style.cssText = 'display:flex;gap:6px;';
  (
    [
      ['dev', 'Interface & Latência'],
      ['in', 'Entrada & Calibração'],
      ['perf', 'Desempenho DSP'],
    ] as const
  ).forEach(([tab, lbl]) => {
    tabs.appendChild(
      makeChipButton('ui.subtab', lbl, store.audioTab === tab, () => {
        store.audioTab = tab;
        store.emit();
      })
    );
  });
  hdr.appendChild(tabs);

  const content = document.createElement('div');
  content.style.cssText = 'flex:1;min-height:0;display:grid;grid-template-columns:1.1fr 0.9fr;gap:10px;overflow:hidden;';

  if (store.audioTab === 'dev') {
    const { root: leftCard, body: leftBody } = makePanelCard('DISPOSITIVO DE ÁUDIO E BUFFER');

    const driverSel = document.createElement('select');
    driverSel.dataset.controlId = 'audio.device_apply';
    driverSel.style.cssText =
      'height:26px;padding:0 8px;border-radius:4px;background:#0b0d10;border:1px solid #2b313a;color:#e8eaed;font-size:11px;';
    [
      ['wasapi', 'WASAPI Low-Latency'],
      ['asio', 'ASIO Studio'],
    ].forEach(([val, lbl]) => {
      const o = document.createElement('option');
      o.value = val;
      o.textContent = lbl;
      if (store.audioStatus.backend?.toLowerCase().includes(val)) {
        o.selected = true;
      }
      driverSel.appendChild(o);
    });

    const inputList = store.devices.filter((d) => d.is_input !== false).map((d) => d.name);
    if (inputList.length === 0) inputList.push(store.audioStatus.input_device || 'Entrada Padrão');
    const outputList = store.devices.filter((d) => d.is_output !== false).map((d) => d.name);
    if (outputList.length === 0) outputList.push(store.audioStatus.output_device || 'Saída Padrão');

    const inputSel = document.createElement('select');
    inputSel.dataset.controlId = 'audio.device_apply';
    inputSel.style.cssText = driverSel.style.cssText;
    inputList.forEach((d) => {
      const o = document.createElement('option');
      o.value = d;
      o.textContent = d;
      if (d === store.audioStatus.input_device) o.selected = true;
      inputSel.appendChild(o);
    });

    const outputSel = document.createElement('select');
    outputSel.dataset.controlId = 'audio.device_apply';
    outputSel.style.cssText = driverSel.style.cssText;
    outputList.forEach((d) => {
      const o = document.createElement('option');
      o.value = d;
      o.textContent = d;
      if (d === store.audioStatus.output_device) o.selected = true;
      outputSel.appendChild(o);
    });

    const srSel = document.createElement('select');
    srSel.dataset.controlId = 'audio.device_apply';
    srSel.style.cssText = driverSel.style.cssText;
    [44100, 48000, 88200, 96000].forEach((sr) => {
      const o = document.createElement('option');
      o.value = String(sr);
      o.textContent = `${sr} Hz`;
      if (sr === (store.audioStatus.sample_rate ?? 48000)) o.selected = true;
      srSel.appendChild(o);
    });

    const bufSel = document.createElement('select');
    bufSel.dataset.controlId = 'audio.device_apply';
    bufSel.style.cssText = driverSel.style.cssText;
    [64, 128, 256, 512, 1024].forEach((b) => {
      const o = document.createElement('option');
      o.value = String(b);
      o.textContent = `${b} amostras (${((b / (store.audioStatus.sample_rate || 48000)) * 1000).toFixed(2)} ms)`;
      if (b === (store.audioStatus.input_period_frames ?? 64)) o.selected = true;
      bufSel.appendChild(o);
    });

    const addLabeled = (lbl: string, el: HTMLElement) => {
      const row = document.createElement('div');
      row.style.cssText = 'display:grid;grid-template-columns:150px 1fr;align-items:center;gap:8px;';
      const l = document.createElement('span');
      l.className = 'm-lbl';
      l.textContent = lbl;
      row.append(l, el);
      leftBody.appendChild(row);
    };
    addLabeled('Driver / Backend', driverSel);
    addLabeled('Dispositivo Entrada', inputSel);
    addLabeled('Dispositivo Saída', outputSel);
    addLabeled('Sample Rate', srSel);
    addLabeled('Tamanho de Buffer', bufSel);

    const actions = document.createElement('div');
    actions.style.cssText = 'display:flex;gap:8px;padding-top:6px;flex-wrap:wrap;';
    actions.append(
      makeChipButton('audio.device_apply', 'Aplicar Interface', false, async () => {
        const res = await store.transport.setAudioDevice({
          backend: driverSel.value,
          input_device: inputSel.value,
          output_device: outputSel.value,
          sample_rate: Number(srSel.value),
          buffer_size: Number(bufSel.value),
        });
        Object.assign(store.audioStatus, res as Partial<AudioStatusData>);
        store.emit();
      }),
      makeChipButton('audio.measure_latency', 'Medir Latência Round-Trip', false, async () => {
        const lat = await store.transport.measureLatency();
        const rtl = Number(lat.measured_roundtrip_ms ?? NaN);
        if (Number.isFinite(rtl)) {
          store.audioStatus.measured_roundtrip_ms = rtl;
        }
        store.emit();
      }),
      makeChipButton('ui.audio_reconnect', 'Reconectar Dispositivo', false, () => {
        void store.reconnectAudioDevice();
      })
    );
    leftBody.appendChild(actions);

    const { root: rightCard, body: rightBody } = makePanelCard('STATUS ATUAL DE TELEMETRIA HARDWARE');
    const lcd = document.createElement('div');
    lcd.className = 'm-lcd m-mono';
    lcd.style.cssText = 'padding:12px;border-radius:6px;display:flex;flex-direction:column;gap:6px;font-size:11.5px;color:#ffb463;';
    const rtl =
      store.audioStatus.measured_roundtrip_ms ??
      store.audioStatus.driver_input_latency_ms + store.audioStatus.driver_output_latency_ms;
    lcd.innerHTML = `
      <div>CONECTADO: ${store.audioStatus.connected !== false ? 'SIM' : 'STANDBY'} · ${store.audioStatus.backend} (${store.audioStatus.mode})</div>
      <div>ENTRADA: ${store.audioStatus.input_device}</div>
      <div>SAÍDA: ${store.audioStatus.output_device}</div>
      <div>TAXA / BUFFER: ${store.audioStatus.sample_rate} Hz / ${store.audioStatus.input_period_frames} samples</div>
      <div>LATÊNCIA ROUND-TRIP: ${rtl.toFixed(2)} ms</div>
      <div>UNDERRUNS / CLIPS: ${store.audioStatus.underruns} / ${store.audioStatus.adc_clips}</div>
    `;
    rightBody.appendChild(lcd);
    content.append(leftCard, rightCard);
  } else if (store.audioTab === 'in') {
    const { root: leftCard, body: leftBody } = makePanelCard('FONTE DE ENTRADA E ASSISTENTE DE CALIBRAÇÃO');
    leftBody.append(
      makeControlSelect(
        store,
        'main.input_source',
        'input.source',
        [
          { value: 'device', label: 'Interface Guitarra (Tempo Real)' },
          { value: 'test_riff', label: 'Riff DI Interno' },
          { value: 'di_file', label: 'Arquivo WAV DI (.wav)' },
        ],
        'device'
      ),
      makeControlSlider(store, 'main.input_boost', 'input.boost', 'Input Trim / Boost', 1.0, (v) => `${v.toFixed(2)}x`)
    );

    const loadDiBtn = makeChipButton('cab.ir_load', 'Abrir Arquivo WAV DI...', false, async () => {
      const path = await store.transport.openFileDialog([{ name: 'Audio WAV DI', extensions: ['wav'] }]);
      if (!path) return;
      const bytes = await store.transport.readFileBytes(path);
      const name = path.split(/[/\\]/).pop() ?? 'di.wav';
      await store.transport.setInputDi({ name, wavBytes: Array.from(bytes) });
      void store.setControl('main.input_source', 'di_file');
      store.emit();
    });
    leftBody.appendChild(loadDiBtn);

    const calBtn = makeChipButton('audio.calibrate_input', 'Calibrar Ruído & Ganho Automático', false, async () => {
      store.calibrationWizard = { phase: 'running', result: null };
      store.emit();
      try {
        const res = await store.transport.calibrateInput({
          phase: 'auto',
          currentInputBoost: num(store, 'input.boost', 1.0),
        });
        store.calibrationWizard = { phase: 'done', result: res };
      } catch (e) {
        store.calibrationWizard = { phase: 'done', result: { error: String(e) } };
      }
      store.emit();
    });
    leftBody.appendChild(calBtn);

    const { root: rightCard, body: rightBody } = makePanelCard('MEDIÇÃO DE ENTRADA E RECOMENDAÇÃO');
    const meterIn = document.createElement('div');
    meterIn.style.cssText = 'display:flex;align-items:center;gap:8px;';
    meterIn.innerHTML = '<span class="m-lbl">NÍVEL ENTRADA</span>';
    const inNorm = Math.max(0, Math.min(1, ((store.telemetry.input_peak_db ?? -60) + 60) / 60));
    meterIn.appendChild(makeLedMeter(24, inNorm));

    const wizLcd = document.createElement('div');
    wizLcd.className = 'm-lcd m-mono';
    wizLcd.style.cssText = 'padding:10px;border-radius:6px;font-size:11px;color:#ffb463;';
    const r = store.calibrationWizard.result;
    if (store.calibrationWizard.phase === 'running') {
      wizLcd.textContent = 'Medindo piso de ruído e pico da guitarra via IPC...';
    } else if (store.calibrationWizard.phase === 'done' && r) {
      wizLcd.textContent =
        typeof r.error === 'string'
          ? `Erro calibração: ${r.error}`
          : `Calibrado: Boost sugerido ${Number(r.recommended_boost ?? 1).toFixed(2)}x · Gate ${Number(r.recommended_threshold_db ?? -62).toFixed(1)} dB`;
    } else {
      wizLcd.textContent = 'Clique em "Calibrar Ruído & Ganho Automático" com a guitarra plugada.';
    }
    rightBody.append(meterIn, wizLcd);
    content.append(leftCard, rightCard);
  } else {
    const { root: leftCard, body: leftBody } = makePanelCard('OVERSAMPLING & ORÇAMENTO DE CPU EM TEMPO REAL');
    leftBody.appendChild(
      makeControlSelect(
        store,
        'topbar.oversampling',
        'engine.oversampling',
        [
          { value: '2x', label: '2x Oversampling (Menor CPU)' },
          { value: '4x', label: '4x Oversampling (Padrão Studio)' },
          { value: '8x', label: '8x Oversampling (Máxima Pureza)' },
        ],
        '4x'
      )
    );
    const { root: rightCard, body: rightBody } = makePanelCard('TELEMETRIA DE CALLBACK ZERO-ALLOC');
    const cpu = store.telemetry.cpu_avg_pct ?? 0;
    const cpuLcd = document.createElement('div');
    cpuLcd.className = 'm-lcd m-mono';
    cpuLcd.style.cssText = 'padding:12px;border-radius:6px;font-size:12px;color:#ffb463;display:flex;flex-direction:column;gap:6px;';
    cpuLcd.innerHTML = `
      <div>CARGA CALLBACK DSP: ${cpu.toFixed(1)}%</div>
      <div>ADC CLIPS / UNDERRUNS: ${store.audioStatus.adc_clips} / ${store.audioStatus.underruns} · ZERO HEAP ALLOCS</div>
      <div>MODO STREAMS: ${document.hidden ? 'PAUSADO (OCULTO)' : 'ATIVO 30 Hz'}</div>
    `;
    rightBody.appendChild(cpuLcd);
    content.append(leftCard, rightCard);
  }

  root.append(hdr, content);
  return root;
}
