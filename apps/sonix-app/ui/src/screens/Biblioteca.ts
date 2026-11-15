import type { AppStore } from '../store.ts';
import { createAmpHeadElement, type AmpModelId } from '../components/AmpHead.ts';
import { createPedalElement, type PedalModelId } from '../components/Pedal.ts';
import { createRackElement, type RackModelId } from '../components/Rack.ts';
import { makeChipButton, makePanelCard } from './_helpers.ts';

interface CatalogEntry<T extends string> {
  id: T;
  name: string;
  specRef: string;
  sourceStatus: 'fonte: verificada' | 'fonte: pendente';
  notes: string;
}

const AMP_CATALOG: CatalogEntry<AmpModelId>[] = [
  {
    id: 'plexi_1959',
    name: 'Brit 1959 Super Lead Plexi',
    specRef: 'Spec 029 · 1959HW Schematic',
    sourceStatus: 'fonte: verificada',
    notes: 'Painel dourado: Power, Standby, Jewel, Presence, Bass, Middle, Treble, Vol I, Vol II e 4 entradas + Jumper.',
  },
  {
    id: 'jcm800_2203',
    name: 'Brit 800 (2203 Lead Series)',
    specRef: 'Spec 029 · JCM800 2203',
    sourceStatus: 'fonte: verificada',
    notes: 'Canal cascateado Master + Pre-Amp com entradas High/Low de impedância real e rede FMV passiva.',
  },
  {
    id: 'twin_ab763',
    name: 'US Twin AB763 Blackface',
    specRef: 'Spec 029 · Fender AB763',
    sourceStatus: 'fonte: pendente',
    notes: 'Canais Normal/Vibrato, chave Bright, Reverb de mola e Tremolo LFO. Nota: inclusão de Middle no Normal pendente de conferência.',
  },
  {
    id: 'dual_rect',
    name: 'Cali Dual Rectifier Solo Head',
    specRef: 'Spec 029 · Solo Head 3Ch',
    sourceStatus: 'fonte: pendente',
    notes: 'Modos Clean/Pushed/Vintage/Modern, Bold/Spongy Variac e chave Diódo vs Válvula 5U4GB. Revisão Rev F vs Multi-Watt em auditoria.',
  },
  {
    id: 'ac30_tb',
    name: 'VOX AC30/6 Top Boost',
    specRef: 'Spec 029 · AC30 TB',
    sourceStatus: 'fonte: pendente',
    notes: 'Classe A Quad EL84, canal Top Boost (Treble/Bass), Tone Cut invertido e Vib-Trem. Ordem exacta de painel em conferência.',
  },
];

const PEDAL_CATALOG: CatalogEntry<PedalModelId | RackModelId>[] = [
  {
    id: 'ns2',
    name: 'GATE-2 (NS-2 Noise Suppressor)',
    specRef: 'Spec 030 · Noise Gate Hard/Natural',
    sourceStatus: 'fonte: verificada',
    notes: 'Detector sidechain LR4 HPF/LPF, Threshold, Histerese, Attack, Hold, Release (Decay) e Range.',
  },
  {
    id: 'ts808',
    name: 'Screamer 808 Overdrive',
    specRef: 'Spec 032 · Simétrico +720 Hz',
    sourceStatus: 'fonte: verificada',
    notes: 'Op-amp com clipagem suave por diodos em realimentação e ênfase média característica.',
  },
  {
    id: 'klon',
    name: 'Golden Klone',
    specRef: 'Spec 032 · Dual-Gang Clean Mix',
    sourceStatus: 'fonte: verificada',
    notes: 'Mistura limpa paralela, diodos de germânio e buffer com alto headroom.',
  },
  {
    id: 'rat',
    name: 'Rodent Distortion',
    specRef: 'Spec 032 · LM308 + Filter',
    sourceStatus: 'fonte: verificada',
    notes: 'Hard clipping simétrico após estágio op-amp e controle Filter passa-baixas reverso.',
  },
  {
    id: 'muff',
    name: 'Big Fuzz Pi',
    specRef: 'Spec 032 · 4 Estágios + Scoop Tone',
    sourceStatus: 'fonte: verificada',
    notes: 'Sustentação massiva em 4 estágios com filtro Tilt Bass/Treble em médios recuados.',
  },
  {
    id: 'fuzzface',
    name: 'Face Fuzz Ge/Si',
    specRef: 'Spec 032 · Transistores Bias',
    sourceStatus: 'fonte: verificada',
    notes: 'Sensibilidade direta ao volume da guitarra com controle Bias ajustável.',
  },
  {
    id: 'ds1',
    name: 'DS-One Classic',
    specRef: 'Spec 032 · Diode Hard Clip',
    sourceStatus: 'fonte: verificada',
    notes: 'Distorção pontuda com filtro scoop/tilt clássico.',
  },
  {
    id: 'ce2',
    name: 'Chorus-2 BBD Analógico',
    specRef: 'Spec 032 · Delay BBD Modulado',
    sourceStatus: 'fonte: verificada',
    notes: 'Chorus quente com modulação senoidal de linha de atraso analógica.',
  },
  {
    id: 'mxr10',
    name: '10-Band Graphic EQ',
    specRef: 'Spec 032 · 10 Filtros Pico',
    sourceStatus: 'fonte: verificada',
    notes: '10 bandas ±12 dB (31.25 Hz a 16 kHz) com Volume e Gain independentes.',
  },
  {
    id: 'comp1176',
    name: 'FET 76 Limiting Amplifier',
    specRef: 'Spec 032 · Peak Limiter',
    sourceStatus: 'fonte: verificada',
    notes: 'Ratios 4:1, 8:1, 12:1, 20:1 e modo All-Buttons-In com ataque ultrarrápido.',
  },
  {
    id: 're201',
    name: 'Space Tape RE-201 Echo',
    specRef: 'Spec 032 · 12 Modos Multi-Head',
    sourceStatus: 'fonte: verificada',
    notes: 'Cabeças de fita 1..12, flutter/wow orgânico, EQ úmido e mola integrada.',
  },
  {
    id: 'plate',
    name: 'Studio Plate Reverb',
    specRef: 'Spec 032 · Tanque Metálico Stereo',
    sourceStatus: 'fonte: verificada',
    notes: 'Pre-delay até 250 ms, damping de agudos e decaimento até 20 segundos.',
  },
];

export function renderBibliotecaScreen(store: AppStore): HTMLElement {
  const root = document.createElement('div');
  root.dataset.screen = 'biblioteca';
  root.style.cssText = 'height:100%;display:flex;flex-direction:column;gap:10px;padding:10px;min-height:0;overflow:hidden;';

  const hdr = document.createElement('div');
  hdr.className = 'm-metal';
  hdr.style.cssText =
    'display:flex;align-items:center;justify-content:space-between;padding:8px 12px;border-radius:8px;flex:none;';
  hdr.innerHTML =
    '<div><div class="m-osw" style="font-size:15px;font-weight:700;letter-spacing:0.08em;">BIBLIOTECA DE MODELOS E ESQUEMÁTICOS</div><div style="font-size:11px;color:#8e96a0;">Cada unidade abaixo usa controles reais ligados ao motor DSP em tempo real.</div></div>';

  const tabs = document.createElement('div');
  tabs.style.cssText = 'display:flex;gap:6px;';
  tabs.append(
    makeChipButton('ui.subtab', 'Amplificadores (5)', store.libraryTab === 'amps', () => {
      store.libraryTab = 'amps';
      store.emit();
    }),
    makeChipButton('ui.subtab', 'Pedais & Racks (12)', store.libraryTab === 'pedals', () => {
      store.libraryTab = 'pedals';
      store.emit();
    })
  );
  hdr.appendChild(tabs);

  const list = document.createElement('div');
  list.className = 'scroll-y';
  list.style.cssText =
    'flex:1;min-height:0;overflow-y:auto;display:flex;flex-direction:column;gap:10px;padding-right:4px;';

  if (store.libraryTab === 'amps') {
    for (const entry of AMP_CATALOG) {
      const card = makePanelCard(entry.name, `${entry.specRef} · ${entry.sourceStatus}`);
      card.root.style.flex = 'none';
      const note = document.createElement('div');
      note.style.cssText = 'display:flex;align-items:center;justify-content:space-between;gap:10px;font-size:11px;color:#b2b9c2;';
      const badgeColor = entry.sourceStatus === 'fonte: verificada' ? '#4ee680' : '#ffb84d';
      note.innerHTML = `<span>${entry.notes}</span><span class="m-mono" style="flex:none;padding:2px 6px;border-radius:4px;border:1px solid ${badgeColor};color:${badgeColor};font-size:9.5px;">${entry.sourceStatus}</span>`;
      const head = createAmpHeadElement(store, { modelOverride: entry.id, compact: true });
      const loadBtn = makeChipButton('amp.model', 'Ativar no Estúdio', store.getParam('amp_1.model', '') === (entry.id as string), () => {
        void store.setControl('amp.model', entry.id);
        store.setScreen('main');
      });
      card.body.append(note, head, loadBtn);
      list.appendChild(card.root);
    }
  } else {
    const grid = document.createElement('div');
    grid.style.cssText = 'display:grid;grid-template-columns:repeat(auto-fill,minmax(310px,1fr));gap:10px;';
    for (const item of PEDAL_CATALOG) {
      const card = makePanelCard(item.name, item.sourceStatus);
      const p = document.createElement('p');
      p.style.cssText = 'font-size:11px;color:#aeb5be;';
      p.textContent = `${item.specRef} — ${item.notes}`;
      const previewWrap = document.createElement('div');
      previewWrap.style.cssText =
        'display:flex;justify-content:center;align-items:center;padding:6px;background:#0b0d10;border-radius:6px;overflow:hidden;';
      const hw =
        item.id === 'comp1176' || item.id === 're201' || item.id === 'plate'
          ? createRackElement(item.id as RackModelId, store)
          : createPedalElement(item.id as PedalModelId, store);
      if (item.id === 'comp1176' || item.id === 're201' || item.id === 'plate') {
        hw.style.transform = 'scale(0.54)';
        hw.style.transformOrigin = 'center center';
        previewWrap.style.height = '112px';
      }
      previewWrap.appendChild(hw);
      card.body.append(p, previewWrap);
      grid.appendChild(card.root);
    }
    list.appendChild(grid);
  }

  root.append(hdr, list);
  return root;
}
