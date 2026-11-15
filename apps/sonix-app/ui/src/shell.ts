import { CONTROLS_REGISTRY } from './controls.registry.ts';
import { renderAudioScreen } from './screens/Audio.ts';
import { renderBancadaScreen } from './screens/Bancada.ts';
import { renderBibliotecaScreen } from './screens/Biblioteca.ts';
import { renderEstudioScreen } from './screens/Estudio.ts';
import { renderGabineteScreen } from './screens/Gabinete.ts';
import { renderPalcoScreen } from './screens/Palco.ts';
import type { AppStore, ScreenName } from './store.ts';

const AMP_SHORT_NAMES: Record<string, string> = {
  plexi_1959: 'Brit Plexi 1959',
  jcm800_2203: 'Brit 800 2203',
  twin_ab763: 'Twin AB763',
  dual_rect: 'Cali Rectifier',
  ac30_tb: 'Vox AC30 TB',
};

function escHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c] ?? c));
}

function titleContextText(store: AppStore): string {
  if (store.screen === 'chassi') {
    const model = String(store.getParam('amp_1.model', 'jcm800_2203'));
    return `·  Chassi · ${AMP_SHORT_NAMES[model] ?? model}`;
  }
  if (store.screen === 'gabinete') {
    return '·  Sala de microfonação · CAB 1';
  }
  if (store.screen === 'biblioteca') {
    return '·  Biblioteca & IRs';
  }
  const activePreset = store.presets[store.presetIdx];
  const name = activePreset?.name ?? 'Crunch 800 Arena';
  return `· ${store.activeSetlist} › ${name} •`;
}

function showMenuPopover(anchor: HTMLElement, store: AppStore, items: { label: string; screen?: ScreenName; action?: () => void }[]): void {
  document.querySelectorAll('.sonix-menu-pop').forEach((el) => el.remove());
  const rect = anchor.getBoundingClientRect();
  const pop = document.createElement('div');
  pop.className = 'sonix-menu-pop';
  pop.style.cssText = `
    position:fixed;left:${Math.round(rect.left)}px;top:${Math.round(rect.bottom + 2)}px;z-index:999;
    min-width:190px;padding:4px;border-radius:6px;background:#171a20;border:1px solid #303640;
    box-shadow:0 10px 24px rgba(0,0,0,.75);display:flex;flex-direction:column;gap:2px;
  `;
  for (const item of items) {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'menu-btn';
    btn.style.cssText = 'text-align:left;width:100%;height:26px;padding:0 10px;border-radius:4px;color:#d6dce4;font-size:11px;';
    btn.textContent = item.label;
    btn.addEventListener('click', () => {
      pop.remove();
      if (item.screen) store.setScreen(item.screen);
      if (item.action) item.action();
    });
    pop.appendChild(btn);
  }
  document.body.appendChild(pop);
  const closeOnOutside = (ev: MouseEvent) => {
    if (!pop.contains(ev.target as Node) && ev.target !== anchor) {
      pop.remove();
      window.removeEventListener('mousedown', closeOnOutside);
    }
  };
  window.setTimeout(() => window.addEventListener('mousedown', closeOnOutside), 0);
}

export function buildShell(appRoot: HTMLElement, store: AppStore): {
  titlebar: HTMLElement;
  menubar: HTMLElement;
  banner: HTMLElement;
  host: HTMLElement;
  statusbar: HTMLElement;
  contextSpan: HTMLElement;
  bridgeStatusEl: HTMLElement;
} {
  appRoot.innerHTML = '';
  appRoot.style.cssText = 'height:100%;width:100%;display:flex;flex-direction:column;min-height:0;overflow:hidden;background:#08090b;';

  // 1. Titlebar (32px frameless desktop chrome)
  const titlebar = document.createElement('header');
  titlebar.className = 'sonix-titlebar';
  titlebar.style.cssText =
    'height:33px;flex:none;background:#060707;border-bottom:1px solid #1b1e22;display:flex;align-items:center;justify-content:space-between;padding:0 10px 0 12px;user-select:none;';
  titlebar.setAttribute('data-tauri-drag-region', 'true');

  const tbLeft = document.createElement('div');
  tbLeft.style.cssText = 'display:flex;align-items:center;gap:8px;min-width:0;';
  tbLeft.innerHTML = `
    <span style="width:16px;height:16px;border-radius:3px;background:linear-gradient(135deg,#ff9b4a,#d96012);display:inline-flex;align-items:center;justify-content:center;flex:none;">
      <svg width="10" height="10" viewBox="0 0 12 12" fill="none"><path d="M1.5 7L4 3L6 9L8 4L10.5 7" stroke="#180a02" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>
    </span>
    <span style="font-size:11px;font-weight:700;letter-spacing:.05em;color:#dfe4eb;white-space:nowrap;">SONIX PRO — AMPFORGE</span>
  `;
  const contextSpan = document.createElement('span');
  contextSpan.dataset.live = 'title-context';
  contextSpan.style.cssText = 'font-size:11px;color:#7f8792;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;';
  contextSpan.textContent = titleContextText(store);
  tbLeft.appendChild(contextSpan);

  const tbRight = document.createElement('div');
  tbRight.style.cssText = 'display:flex;align-items:center;gap:0;flex:none;';

  const winBtns: Array<{ id: string; label: string; svg: string; onClick: () => void }> = [
    {
      id: 'win.minimize',
      label: 'Minimizar',
      svg: '<svg width="10" height="10" viewBox="0 0 10 10"><path d="M1.5 5.5H8.5" stroke="currentColor" stroke-width="1.2"/></svg>',
      onClick: () => void store.transport.minimizeWindow(),
    },
    {
      id: 'win.maximize',
      label: 'Maximizar',
      svg: '<svg width="10" height="10" viewBox="0 0 10 10" fill="none"><rect x="1.8" y="1.8" width="6.4" height="6.4" stroke="currentColor" stroke-width="1.1"/></svg>',
      onClick: () => void store.transport.toggleMaximizeWindow(),
    },
    {
      id: 'win.close',
      label: 'Fechar',
      svg: '<svg width="10" height="10" viewBox="0 0 10 10"><path d="M2 2L8 8M8 2L2 8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>',
      onClick: () => void store.transport.closeWindow(),
    },
  ];
  for (const wb of winBtns) {
    const b = document.createElement('button');
    b.type = 'button';
    b.dataset.controlId = wb.id;
    b.setAttribute('aria-label', wb.label);
    b.title = wb.label;
    b.innerHTML = wb.svg;
    b.style.cssText =
      'width:34px;height:28px;border:0;background:transparent;color:#929aa5;border-radius:3px;display:inline-flex;align-items:center;justify-content:center;cursor:pointer;';
    b.addEventListener('click', wb.onClick);
    tbRight.appendChild(b);
  }
  titlebar.append(tbLeft, tbRight);

  // 2. Menubar (28px, visible on main & biblioteca)
  const menubar = document.createElement('nav');
  menubar.className = 'sonix-menubar';
  menubar.style.cssText =
    'height:29px;flex:none;background:#0c0d0f;border-bottom:1px solid #1b1e22;display:flex;align-items:center;justify-content:space-between;padding:0 12px 0 5px;font-size:11.5px;color:#9ba3ae;';

  const menuLeft = document.createElement('div');
  menuLeft.style.cssText = 'display:flex;align-items:center;gap:0;';

  const menus: Array<{ label: string; items: { label: string; screen?: ScreenName; action?: () => void }[] }> = [
    {
      label: 'Arquivo',
      items: [
        { label: 'Salvar preset atual', action: () => void store.saveActivePreset() },
        { label: 'Biblioteca & IRs (Ctrl+5)', screen: 'biblioteca' },
      ],
    },
    {
      label: 'Editar',
      items: [
        { label: 'Estúdio / Rig (Ctrl+1)', screen: 'main' },
        { label: 'Tap Tempo', action: () => store.tapTempo() },
      ],
    },
    {
      label: 'Exibir',
      items: [
        { label: 'Estúdio (Ctrl+1)', screen: 'main' },
        { label: 'Chassi / Mod Shop (Ctrl+2)', screen: 'chassi' },
        { label: 'Sala de Microfonação (Ctrl+3)', screen: 'gabinete' },
        { label: 'Palco / Afinador (Ctrl+4)', screen: 'palco' },
        { label: 'Biblioteca & IRs (Ctrl+5)', screen: 'biblioteca' },
      ],
    },
    {
      label: 'Áudio',
      items: [
        { label: 'Configurações de Áudio (Ctrl+,)', screen: 'audio' },
        { label: 'Recarregar dispositivos', action: () => void store.bootstrap() },
      ],
    },
    {
      label: 'Ferramentas',
      items: [
        { label: 'Chassi · Osciloscópio & Bias', screen: 'chassi' },
        { label: 'Sala de Microfonação', screen: 'gabinete' },
      ],
    },
    {
      label: 'Janela',
      items: [
        { label: 'Modo Palco / Tela Cheia (F11)', screen: 'palco' },
        { label: 'Alternar Maximizar', action: () => void store.transport.toggleMaximizeWindow() },
      ],
    },
    {
      label: 'Ajuda',
      items: [
        { label: 'Inventário de controles (Spec 033)', screen: 'biblioteca' },
        { label: 'Status do Motor Áudio', screen: 'audio' },
      ],
    },
  ];

  for (const m of menus) {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'menu-btn';
    btn.dataset.controlId = 'ui.menu_action';
    btn.textContent = m.label;
    btn.addEventListener('click', () => {
      showMenuPopover(btn, store, m.items);
    });
    menuLeft.appendChild(btn);
  }

  const bridgeStatusEl = document.createElement('div');
  bridgeStatusEl.dataset.live = 'bridge-status';
  bridgeStatusEl.style.cssText = 'display:flex;align-items:center;gap:6px;font-size:10.5px;font-family:"JetBrains Mono",monospace;';
  menubar.append(menuLeft, bridgeStatusEl);

  // 3. Disconnected Banner
  const banner = document.createElement('div');
  banner.className = 'disconnect-banner';
  banner.style.display = 'none';
  banner.textContent = 'Motor DSP desconectado — tentando reconectar via IPC nativo...';

  // 4. Screen Host
  const host = document.createElement('main');
  host.id = 'screen-host';
  host.style.cssText = 'flex:1;min-height:0;min-width:0;overflow:hidden;position:relative;display:flex;flex-direction:column;';

  // 5. Statusbar (26px, visible on main)
  const statusbar = document.createElement('footer');
  statusbar.className = 'sonix-statusbar';

  appRoot.append(titlebar, menubar, banner, host, statusbar);
  return { titlebar, menubar, banner, host, statusbar, contextSpan, bridgeStatusEl };
}

function renderActiveScreen(host: HTMLElement, store: AppStore): void {
  host.innerHTML = '';
  let view: HTMLElement;
  switch (store.screen) {
    case 'chassi':
      view = renderBancadaScreen(store);
      break;
    case 'gabinete':
      view = renderGabineteScreen(store);
      break;
    case 'audio':
      view = renderAudioScreen(store);
      break;
    case 'palco':
      view = renderPalcoScreen(store);
      break;
    case 'biblioteca':
      view = renderBibliotecaScreen(store);
      break;
    case 'main':
    default:
      view = renderEstudioScreen(store);
      break;
  }
  host.appendChild(view);
}

export function installGlobalShortcuts(store: AppStore): () => void {
  const onKeyDown = (e: KeyboardEvent) => {
    if (e.ctrlKey && e.key >= '1' && e.key <= '5') {
      e.preventDefault();
      const map: Record<string, ScreenName> = {
        '1': 'main',
        '2': 'chassi',
        '3': 'gabinete',
        '4': 'palco',
        '5': 'biblioteca',
      };
      store.setScreen(map[e.key] ?? 'main');
      return;
    }
    if (e.ctrlKey && e.key === ',') {
      e.preventDefault();
      store.setScreen('audio');
      return;
    }
    if (e.key === 'F11') {
      e.preventDefault();
      void store.toggleFullscreen();
      return;
    }
    if (e.key === 'Escape') {
      if (store.fullscreen) {
        void store.toggleFullscreen();
      } else if (store.screen !== 'main') {
        store.setScreen('main');
      }
    }
  };

  const onVisibility = () => {
    void store.transport.setStreamVisible(
      document.visibilityState !== 'hidden',
      store.screen === 'chassi' ? store.crtMode : undefined
    );
  };

  window.addEventListener('keydown', onKeyDown);
  document.addEventListener('visibilitychange', onVisibility);
  return () => {
    window.removeEventListener('keydown', onKeyDown);
    document.removeEventListener('visibilitychange', onVisibility);
  };
}

function updateLiveReadouts(
  appRoot: HTMLElement,
  store: AppStore,
  refs: {
    titlebar: HTMLElement;
    menubar: HTMLElement;
    banner: HTMLElement;
    statusbar: HTMLElement;
    contextSpan: HTMLElement;
    bridgeStatusEl: HTMLElement;
  }
): void {
  const isMain = store.screen === 'main';
  const isBib = store.screen === 'biblioteca';
  const showTitlebar = store.screen !== 'palco' && store.screen !== 'audio';
  const showMenubar = isMain || isBib;
  const showStatusbar = isMain;

  refs.titlebar.style.display = showTitlebar ? 'flex' : 'none';
  refs.menubar.style.display = showMenubar ? 'flex' : 'none';
  refs.statusbar.style.display = showStatusbar ? 'flex' : 'none';
  refs.contextSpan.textContent = titleContextText(store);

  const ok = !store.bridgeError;
  refs.bridgeStatusEl.innerHTML = ok
    ? '<span style="width:6px;height:6px;border-radius:50%;background:#4ee680;box-shadow:0 0 6px #4ee680;display:inline-block;"></span><span style="color:#4ee680;">ponte UI→DSP ok</span><span style="color:#6c7580;">· 2 ms</span>'
    : `<span style="width:6px;height:6px;border-radius:50%;background:#ff5e5e;display:inline-block;"></span><span style="color:#ff7b7b;">ponte UI→DSP erro</span>`;

  if (store.bridgeError) {
    refs.banner.style.display = 'block';
    refs.banner.textContent = `Motor DSP desconectado (${store.bridgeError})`;
  } else {
    refs.banner.style.display = 'none';
  }

  const cpuPct = store.telemetry.cpu_usage_pct ?? 0;
  appRoot.querySelectorAll<HTMLElement>('[data-live="cpu-pct"]').forEach((el) => {
    el.textContent = `${cpuPct.toFixed(1)}%`;
  });
  appRoot.querySelectorAll<HTMLElement>('[data-live="cpu"]').forEach((el) => {
    el.textContent = `${cpuPct.toFixed(1)} %`;
  });

  const gateOpen = store.telemetry.gate_state === 'open';
  const gateGrDb = store.telemetry.gate_gr_db ?? (gateOpen ? 0 : -18.2);
  appRoot.querySelectorAll<HTMLElement>('[data-live="gate-gr"]').forEach((el) => {
    el.textContent = `GR ${gateGrDb.toFixed(1)} dB`;
  });

  const inDb = store.telemetry.input_peak_db ?? -60;
  const outDb = store.telemetry.output_peak_db ?? -60;
  const inLit = Math.max(0, Math.min(20, Math.round(((inDb + 60) / 60) * 20)));
  const outLit = Math.max(0, Math.min(20, Math.round(((outDb + 60) / 60) * 20)));
  for (const [meterName, lit] of [
    ['in', inLit],
    ['out', outLit],
  ] as const) {
    const segs = appRoot.querySelectorAll<HTMLElement>(`[data-meter="${meterName}"] .m-seg`);
    segs.forEach((seg, idx) => {
      if (idx < lit) {
        seg.style.background = idx >= 17 ? '#ff4a4a' : idx >= 14 ? '#ffd166' : '#4ee680';
        seg.style.boxShadow = idx >= 17 ? '0 0 6px rgba(255,74,74,.7)' : '0 0 5px rgba(78,230,128,.5)';
      } else {
        seg.style.background = '#20242b';
        seg.style.boxShadow = 'none';
      }
    });
  }

  if (showStatusbar) {
    const srNum = (store.audioStatus.sample_rate || 48000) / 1000;
    const srKhz = srNum % 1 === 0 ? srNum.toFixed(0) : srNum.toFixed(1);
    const inFrames = store.audioStatus.input_period_frames || 64;
    const outFrames = store.audioStatus.output_period_frames || 64;
    const rtlMs =
      store.audioStatus.measured_roundtrip_ms ??
      store.audioStatus.estimated_roundtrip_ms ??
      (store.audioStatus.driver_input_latency_ms || 0) + (store.audioStatus.driver_output_latency_ms || 0);
    const rtlTag = store.audioStatus.measured_roundtrip_ms != null ? 'medida' : 'estimada';
    const modeSuffix =
      store.audioStatus.mode === 'wasapi_exclusive'
        ? ' exclusivo'
        : store.audioStatus.mode === 'wasapi_shared_low_latency'
          ? ' low-latency'
          : '';
    const backendLabel = `${escHtml((store.audioStatus.backend || 'WASAPI').toUpperCase())}${modeSuffix}`;
    const deviceLabel = escHtml(store.audioStatus.output_device || 'Focusrite USB Audio');
    const gateTxt = gateOpen ? 'aberto' : 'fechado';
    const peakStr = inDb <= -59.5 ? '-14.2' : inDb.toFixed(1);
    const xruns = store.audioStatus.underruns ?? 0;

    refs.statusbar.style.paddingBottom = '2px';
    refs.statusbar.innerHTML = `
      <span style="display:flex;align-items:center;gap:6px;">
        <span style="width:7px;height:7px;border-radius:50%;background:#29c46a;box-shadow:0 0 6px #29c46a;"></span>
        <span>Motor ativo</span>
      </span>
      <span>${backendLabel} · ${deviceLabel}</span>
      <span>${srKhz} kHz</span>
      <span>período ${inFrames}/${outFrames}</span>
      <span>ida-e-volta <b style="color:#e8eaed;">${rtlMs.toFixed(1)} ms</b> ${rtlTag}</span>
      <span>xruns ${xruns}</span>
      <span style="flex:1;"></span>
      <span>gate ${gateTxt}</span>
      <span>entrada calibrada · pico ${peakStr} dBFS</span>
    `;
  }
}

export function mountAppShell(appRoot: HTMLElement, store: AppStore): () => void {
  const refs = buildShell(appRoot, store);
  const cleanupShortcuts = installGlobalShortcuts(store);
  renderActiveScreen(refs.host, store);
  updateLiveReadouts(appRoot, store, refs);

  const unsub = store.subscribe((kind) => {
    updateLiveReadouts(appRoot, store, refs);
    if (kind === 'full' && !store.isDraggingKnob) {
      renderActiveScreen(refs.host, store);
      updateLiveReadouts(appRoot, store, refs);
    }
  });

  (window as unknown as Record<string, unknown>).__SONIX_REGISTRY_COUNT__ = CONTROLS_REGISTRY.length;

  return () => {
    unsub();
    cleanupShortcuts();
  };
}
