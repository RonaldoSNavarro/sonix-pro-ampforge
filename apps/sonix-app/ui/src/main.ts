import './styles.css';
import { listen } from '@tauri-apps/api/event';
import { AppStore } from './store.ts';
import { mountAppShell } from './shell.ts';
import { isTauriRuntime } from './transport.ts';

function mount(): void {
  const appRoot = document.getElementById('app');
  if (!appRoot) return;
  const store = new AppStore();
  mountAppShell(appRoot, store);
  if (isTauriRuntime()) {
    void listen<string>('audio_device_state', (ev) => {
      if (ev.payload === 'lost' || ev.payload === 'ok') {
        store.audioDeviceState = ev.payload;
        store.deviceDisconnected = ev.payload === 'lost';
        store.emit('status');
      }
    });
  }
  void store.bootstrap();
}

mount();
