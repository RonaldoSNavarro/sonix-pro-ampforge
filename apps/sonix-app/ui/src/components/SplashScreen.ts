import type { AppStore } from '../store';

export interface SplashRefs {
  root: HTMLElement;
  cleanup: () => void;
}

export function createSplashScreenElement(store: AppStore): SplashRefs {
  const el = document.createElement('div');
  el.className = 'sonix-splash';
  el.dataset.region = 'splash';
  el.setAttribute('role', 'status');
  el.setAttribute('aria-label', 'Carregando SONIX PRO // AMPFORGE');

  el.innerHTML = `
    <div class="sonix-splash-chassis">
      <div class="sonix-splash-screws" aria-hidden="true">
        <span></span><span></span><span></span><span></span>
      </div>
      <div class="sonix-splash-emblem">
        <svg viewBox="0 0 360 156" width="360" height="156" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
          <defs>
            <linearGradient id="sonixChassisGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#2c3138"/>
              <stop offset="52%" stop-color="#191d22"/>
              <stop offset="100%" stop-color="#0f1215"/>
            </linearGradient>
            <linearGradient id="sonixGoldTrim" x1="0" y1="0" x2="1" y2="0">
              <stop offset="0%" stop-color="#a87424"/>
              <stop offset="50%" stop-color="#f2c464"/>
              <stop offset="100%" stop-color="#a87424"/>
            </linearGradient>
            <radialGradient id="sonixTubeGlow" cx="50%" cy="46%" r="54%">
              <stop offset="0%" stop-color="#fff0bd" stop-opacity="0.98"/>
              <stop offset="42%" stop-color="#ff9922" stop-opacity="0.72"/>
              <stop offset="100%" stop-color="#ff6600" stop-opacity="0"/>
            </radialGradient>
          </defs>
          <rect x="10" y="10" width="340" height="136" rx="14" fill="url(#sonixChassisGrad)" stroke="url(#sonixGoldTrim)" stroke-width="2"/>
          <rect x="20" y="20" width="320" height="116" rx="9" fill="#0d1014" stroke="rgba(255,255,255,0.08)" stroke-width="1"/>
          <g class="sonix-splash-tubes">
            <circle cx="80" cy="60" r="26" fill="url(#sonixTubeGlow)"/>
            <rect x="70" y="34" width="20" height="44" rx="10" fill="rgba(255,212,138,0.12)" stroke="#e6a84c" stroke-width="1.3"/>
            <path d="M76 66 L76 48 L80 42 L84 48 L84 66" fill="none" stroke="#ffcf70" stroke-width="1.8" stroke-linecap="round"/>
            <circle cx="180" cy="56" r="32" fill="url(#sonixTubeGlow)"/>
            <rect x="168" y="26" width="24" height="52" rx="12" fill="rgba(255,212,138,0.16)" stroke="#f2c464" stroke-width="1.5"/>
            <path d="M175 66 L175 44 L180 35 L185 44 L185 66" fill="none" stroke="#ffe29a" stroke-width="2" stroke-linecap="round"/>
            <circle cx="280" cy="60" r="26" fill="url(#sonixTubeGlow)"/>
            <rect x="270" y="34" width="20" height="44" rx="10" fill="rgba(255,212,138,0.12)" stroke="#e6a84c" stroke-width="1.3"/>
            <path d="M276 66 L276 48 L280 42 L284 48 L284 66" fill="none" stroke="#ffcf70" stroke-width="1.8" stroke-linecap="round"/>
          </g>
          <path d="M40 94 L125 94 L132 86 L140 102 L148 84 L156 104 L164 88 L172 94 L320 94"
            fill="none" stroke="url(#sonixGoldTrim)" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" class="sonix-splash-wave"/>
          <line x1="36" y1="122" x2="324" y2="122" stroke="rgba(242,196,100,0.28)" stroke-width="1"/>
        </svg>
      </div>
      <div class="sonix-splash-brand">SONIX PRO <span class="sonix-splash-sep">//</span> <strong>AMPFORGE&trade;</strong></div>
      <div class="sonix-splash-sub">INICIALIZANDO MOTOR VALVULADO WHITE-BOX &amp; INTERFACE DE ÁUDIO…</div>
      <div class="sonix-splash-bar"><span class="sonix-splash-fill"></span></div>
    </div>
  `;

  let safetyTimer: ReturnType<typeof setTimeout> | null = setTimeout(() => {
    safetyTimer = null;
    if (!store.bootstrapped) {
      store.bootstrapped = true;
      updateSplashElement(el, store);
    }
  }, 8000);
  const timerUnknown = safetyTimer as unknown as { unref?: () => void };
  if (typeof timerUnknown?.unref === 'function') {
    timerUnknown.unref();
  }

  updateSplashElement(el, store);

  return {
    root: el,
    cleanup: () => {
      if (safetyTimer !== null) {
        clearTimeout(safetyTimer);
        safetyTimer = null;
      }
    },
  };
}

export function updateSplashElement(el: HTMLElement, store: AppStore): void {
  const reduced = Boolean(store.reducedMotion);
  el.setAttribute('data-reduced-motion', reduced ? 'true' : 'false');
  if (reduced) {
    el.style.animation = 'none';
    el.querySelectorAll<HTMLElement>('.sonix-splash-fill, .sonix-splash-tubes, .sonix-splash-wave').forEach((node) => {
      node.style.animation = 'none';
    });
  } else {
    el.style.animation = '';
  }

  const active = !store.bootstrapped;
  el.setAttribute('data-splash-active', active ? 'true' : 'false');
  el.setAttribute('aria-hidden', active ? 'false' : 'true');
  el.style.display = active ? 'flex' : 'none';
}
