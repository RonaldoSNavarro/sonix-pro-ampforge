import './styles.css';
import { AppStore } from './store.ts';
import { mountAppShell } from './shell.ts';

function mount(): void {
  const appRoot = document.getElementById('app');
  if (!appRoot) return;
  const store = new AppStore();
  mountAppShell(appRoot, store);
  void store.bootstrap();
}

mount();
