import './styles.css';
import { AppStore, type ScreenName } from './store';
import { renderEstudioScreen } from './screens/Estudio';
import { renderBancadaScreen } from './screens/Bancada';
import { renderGabineteScreen } from './screens/Gabinete';
import { renderPalcoScreen } from './screens/Palco';
import { renderBibliotecaScreen } from './screens/Biblioteca';
import { renderAudioScreen } from './screens/Audio';

const NAV_TABS: { id: ScreenName; label: string; shortcut: string }[] = [
  { id: 'main', label: 'Estúdio', shortcut: 'Ctrl+1' },
  { id: 'chassi', label: 'Bancada', shortcut: 'Ctrl+2' },
  { id: 'gabinete', label: 'Gabinete', shortcut: 'Ctrl+3' },
  { id: 'palco', label: 'Palco', shortcut: 'Ctrl+4' },
  { id: 'biblioteca', label: 'Biblioteca', shortcut: 'Ctrl+5' },
  { id: 'audio', label: 'Áudio', shortcut: 'Ctrl+,' },
];

const store = new AppStore();

function buildShell(appRoot: HTMLElement): {
  menubarTabs: Map<ScreenName, HTMLButtonElement>;
  disconnectBanner: HTMLElement;
  disconnectAction: HTMLButtonElement;
  screenHost: HTMLElement;
  statusLeft: HTMLElement;
  statusRight: HTMLElement;
} {
  appRoot.innerHTML = '';

  const titlebar = document.createElement('header');
  titlebar.className = 'sonix-titlebar';

  const drag = document.createElement('div');
  drag.className = 'sonix-titlebar-drag';
  drag.setAttribute('data-tauri-drag-region', '');
  drag.innerHTML = `
    <span class="m-osw" style="font-size:13px;font-weight:700;letter-spacing:0.12em;color:#ff9b4a;">SONIX PRO</span>
    <span style="font-size:10.5px;color:#7f8791;font-weight:600;letter-spacing:0.08em;">// AMPFORGE™ DESKTOP</span>
  `;

  const winActions = document.createElement('div');
  winActions.className = 'sonix-titlebar-actions';

  const minBtn = document.createElement('button');
  minBtn.type = 'button';
  minBtn.className = 'win-btn';
  minBtn.dataset.controlId = 'win.minimize';
  minBtn.title = 'Minimizar';
  minBtn.textContent = '—';
  minBtn.addEventListener('click', () => void store.transport.minimizeWindow());

  const maxBtn = document.createElement('button');
  maxBtn.type = 'button';
  maxBtn.className = 'win-btn';
  maxBtn.dataset.controlId = 'win.maximize';
  maxBtn.title = 'Maximizar / Restaurar';
  maxBtn.textContent = '▢';
  maxBtn.addEventListener('click', () => void store.transport.toggleMaximizeWindow());

  const closeBtn = document.createElement('button');
  closeBtn.type = 'button';
  closeBtn.className = 'win-btn win-close';
  closeBtn.dataset.controlId = 'win.close';
  closeBtn.title = 'Fechar';
  closeBtn.textContent = '✕';
  closeBtn.addEventListener('click', () => void store.transport.closeWindow());

  winActions.append(minBtn, maxBtn, closeBtn);
  titlebar.append(drag, winActions);

  const menubar = document.createElement('nav');
  menubar.className = 'sonix-menubar';
  menubar.setAttribute('aria-label', 'Telas do Estúdio');
  const menubarTabs = new Map<ScreenName, HTMLButtonElement>();
  for (const tab of NAV_TABS) {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'nav-tab';
    btn.dataset.controlId = 'ui.nav_screen';
    btn.dataset.screenTarget = tab.id;
    btn.title = `${tab.label} (${tab.shortcut})`;
    btn.textContent = tab.label;
    btn.addEventListener('click', () => store.setScreen(tab.id));
    menubar.appendChild(btn);
    menubarTabs.set(tab.id, btn);
  }

  const disconnectBanner = document.createElement('div');
  disconnectBanner.className = 'disconnect-banner';
  disconnectBanner.style.display = 'none';
  const disconnectMsg = document.createElement('span');
  disconnectMsg.textContent = 'Interface de áudio desconectada ou inativa. Verifique cabo USB/ASIO.';
  const disconnectAction = document.createElement('button');
  disconnectAction.type = 'button';
  disconnectAction.className = 'm-chip';
  disconnectAction.dataset.controlId = 'ui.audio_reconnect';
  disconnectAction.textContent = 'Reconectar Dispositivo';
  disconnectAction.addEventListener('click', () => void store.reconnectAudioDevice());
  disconnectBanner.append(disconnectMsg, disconnectAction);

  const screenHost = document.createElement('main');
  screenHost.id = 'screen-host';
  screenHost.style.cssText = 'min-height:0;overflow:hidden;position:relative;';

  const statusbar = document.createElement('footer');
  statusbar.className = 'sonix-statusbar';
  const statusLeft = document.createElement('span');
  const statusRight = document.createElement('span');
  statusbar.append(statusLeft, statusRight);

  appRoot.append(titlebar, menubar, disconnectBanner, screenHost, statusbar);
  return { menubarTabs, disconnectBanner, disconnectAction, screenHost, statusLeft, statusRight };
}

function renderActiveScreen(screenHost: HTMLElement): void {
  let node: HTMLElement;
  switch (store.screen) {
    case 'chassi':
      node = renderBancadaScreen(store);
      break;
    case 'gabinete':
      node = renderGabineteScreen(store);
      break;
    case 'palco':
      node = renderPalcoScreen(store);
      break;
    case 'biblioteca':
      node = renderBibliotecaScreen(store);
      break;
    case 'audio':
      node = renderAudioScreen(store);
      break;
    default:
      node = renderEstudioScreen(store);
      break;
  }
  screenHost.replaceChildren(node);
}

function installGlobalShortcuts(): void {
  window.addEventListener('keydown', (e: KeyboardEvent) => {
    const ctrl = e.ctrlKey || e.metaKey;
    if (ctrl && e.key === '1') {
      e.preventDefault();
      store.setScreen('main');
    } else if (ctrl && e.key === '2') {
      e.preventDefault();
      store.setScreen('chassi');
    } else if (ctrl && e.key === '3') {
      e.preventDefault();
      store.setScreen('gabinete');
    } else if (ctrl && e.key === '4') {
      e.preventDefault();
      store.setScreen('palco');
    } else if (ctrl && e.key === '5') {
      e.preventDefault();
      store.setScreen('biblioteca');
    } else if (ctrl && e.key === ',') {
      e.preventDefault();
      store.setScreen('audio');
    } else if (e.key === 'F11') {
      e.preventDefault();
      if (store.screen !== 'palco') {
        store.setScreen('palco');
      }
      void store.toggleFullscreen();
    } else if (e.key === 'Escape' && store.fullscreen) {
      e.preventDefault();
      void store.toggleFullscreen();
    }
  });

  const syncVisibility = () => {
    const visible = !document.hidden;
    void store.transport.setStreamVisible(visible, store.crtMode);
  };
  document.addEventListener('visibilitychange', syncVisibility);
  window.addEventListener('focus', syncVisibility);
  window.addEventListener('blur', () => {
    if (document.hidden) {
      void store.transport.setStreamVisible(false, store.crtMode);
    }
  });
}

function mount(): void {
  const appRoot = document.getElementById('app');
  if (!appRoot) return;

  const ui = buildShell(appRoot);
  installGlobalShortcuts();

  const update = () => {
    for (const [id, btn] of ui.menubarTabs.entries()) {
      btn.setAttribute('aria-current', store.screen === id ? 'page' : 'false');
    }
    ui.disconnectBanner.style.display = store.deviceDisconnected ? 'flex' : 'none';
    renderActiveScreen(ui.screenHost);

    const errText = store.bridgeError ? `ERRO IPC: ${store.bridgeError}` : 'IPC NATIVO TAURI v2 OK';
    ui.statusLeft.textContent = `${errText} · ${store.audioStatus.backend} (${store.audioStatus.sample_rate} Hz / ${store.audioStatus.input_period_frames})`;
    ui.statusLeft.style.color = store.bridgeError ? '#ff6b63' : '#9aa2ad';

    const cpu = (store.telemetry.cpu_avg_pct ?? 0).toFixed(1);
    const lat = (
      store.audioStatus.measured_roundtrip_ms ??
      store.audioStatus.driver_input_latency_ms + store.audioStatus.driver_output_latency_ms
    ).toFixed(1);
    const clip = (store.audioStatus.adc_clips ?? 0) > 0 ? 'CLIP!' : 'SAFE';
    ui.statusRight.textContent = `CPU ${cpu}% · LAT ${lat} ms · ${clip}`;
  };

  store.subscribe(update);
  update();
  void store.bootstrap();
}

mount();
