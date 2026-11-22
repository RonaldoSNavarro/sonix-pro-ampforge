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
    specRef: 'Esquemático 1959HW · 100 W',
    sourceStatus: 'fonte: verificada',
    notes: 'Painel dourado: Power, Standby, Jewel, Presence, Bass, Middle, Treble, Vol I, Vol II e 4 entradas + Jumper.',
  },
  {
    id: 'jcm800_2203',
    name: 'Brit 800 (2203 Lead Series)',
    specRef: 'Esquemático 2203 Master Volume · 100 W',
    sourceStatus: 'fonte: verificada',
    notes: 'Canal cascateado Master + Pre-Amp com entradas High/Low de impedância real e rede FMV passiva.',
  },
  {
    id: 'twin_ab763',
    name: 'US Twin AB763 Blackface',
    specRef: 'Circuito AB763 Blackface · 85 W',
    sourceStatus: 'fonte: pendente',
    notes: 'Canais Normal/Vibrato, chave Bright, Reverb de mola e Tremolo LFO.',
  },
  {
    id: 'dual_rect',
    name: 'Cali Dual Rectifier Solo Head',
    specRef: 'Solo Head Rev F · 100 W',
    sourceStatus: 'fonte: pendente',
    notes: 'Modos Clean/Vintage/Modern, Bold/Spongy Variac e chave Diodo vs Válvula 5U4GB.',
  },
  {
    id: 'ac30_tb',
    name: 'VOX AC30/6 Top Boost',
    specRef: 'Circuito AC30/6 Top Boost · 30 W',
    sourceStatus: 'fonte: pendente',
    notes: 'Classe A Quad EL84, canal Top Boost (Treble/Bass), Tone Cut invertido e Vib-Trem.',
  },
];

const PEDAL_CATALOG: CatalogEntry<PedalModelId | RackModelId>[] = [
  {
    id: 'ns2',
    name: 'GATE-2 (NS-2 Noise Suppressor)',
    specRef: 'Noise Gate Natural / Expander',
    sourceStatus: 'fonte: verificada',
    notes: 'Detector sidechain LR4 HPF/LPF, Threshold, Histerese, Attack, Hold, Release (Decay) e Range.',
  },
  {
    id: 'ts808',
    name: 'Screamer 808 Overdrive',
    specRef: 'Clipping Simétrico +720 Hz',
    sourceStatus: 'fonte: verificada',
    notes: 'Op-amp com clipagem suave por diodos em realimentação e ênfase média característica.',
  },
  {
    id: 'klon',
    name: 'Golden Klone',
    specRef: 'Dual-Gang Clean Blend + Ge',
    sourceStatus: 'fonte: verificada',
    notes: 'Mistura limpa paralela, diodos de germânio e buffer com alto headroom.',
  },
  {
    id: 'rat',
    name: 'Rodent Distortion',
    specRef: 'Slew-Rate LM308 + Filter',
    sourceStatus: 'fonte: verificada',
    notes: 'Hard clipping simétrico após estágio op-amp e controle Filter passa-baixas reverso.',
  },
  {
    id: 'muff',
    name: 'Big Fuzz Pi',
    specRef: '4 Estágios Cascata + Scoop Tone',
    sourceStatus: 'fonte: verificada',
    notes: 'Sustentação massiva em 4 estágios com filtro Tilt Bass/Treble em médios recuados.',
  },
  {
    id: 'fuzzface',
    name: 'Face Fuzz Ge/Si',
    specRef: 'Par Transistorizado com Bias',
    sourceStatus: 'fonte: verificada',
    notes: 'Sensibilidade direta ao volume da guitarra com controle Bias ajustável.',
  },
  {
    id: 'ds1',
    name: 'DS-One Classic',
    specRef: 'Hard Clip de Diodos + Scoop',
    sourceStatus: 'fonte: verificada',
    notes: 'Distorção pontuda com filtro scoop/tilt clássico.',
  },
  {
    id: 'ce2',
    name: 'Chorus-2 BBD Analógico',
    specRef: 'Linha de Atraso BBD Modulada',
    sourceStatus: 'fonte: verificada',
    notes: 'Chorus quente com modulação senoidal de linha de atraso analógica.',
  },
  {
    id: 'trichorus',
    name: 'Chorus 3 vozes (mono)',
    specRef: '3 Linhas BBD Defasadas (120°) em Mono',
    sourceStatus: 'fonte: verificada',
    notes: 'Modulação de 3 vozes somadas em mono no barramento atual do motor DSP.',
  },
  {
    id: 'flanger',
    name: 'TZ Flanger Through-Zero',
    specRef: 'Filtro Pente Through-Zero + Feedback',
    sourceStatus: 'fonte: verificada',
    notes: 'Cancelamento de fase nulo e varredura jet metalizada através do ponto zero.',
  },
  {
    id: 'phaser90',
    name: 'Phaser 90 FET 4 Estágios',
    specRef: '4 All-Pass FET + Controle Único Speed',
    sourceStatus: 'fonte: verificada',
    notes: 'Notches assimétricos quentes de phaser laranja com um único potenciômetro de velocidade.',
  },
  {
    id: 'tremolo',
    name: 'Tremolo Óptico & Bias',
    specRef: 'LFO Senoidal + Modulação de Bias',
    sourceStatus: 'fonte: verificada',
    notes: 'Pulsação de amplitude orgânica modelada por optoacoplador e variação de corrente de placa.',
  },
  {
    id: 'univibe',
    name: 'Uni-Vibe Fotocélula 4 Estágios',
    specRef: 'Filtros Assimétricos LDR + Lâmpada',
    sourceStatus: 'fonte: verificada',
    notes: 'Resposta latejante assimétrica clássica de quatro fotocélulas e capacitor assimétrico.',
  },
  {
    id: 'rotary',
    name: 'Rotary 147 Horn & Drum',
    specRef: 'Doppler Dual Rotor Agudos/Graves',
    sourceStatus: 'fonte: verificada',
    notes: 'Rotação separada para corneta e tambor com inércia mecânica e dispersão Doppler.',
  },
  {
    id: 'whammy',
    name: 'Whammy Pitch Shifter',
    specRef: 'Pitch Polifônico em Tempo Real (-24..+24 st)',
    sourceStatus: 'fonte: verificada',
    notes: 'Transposição contínua controlada por pedal de expressão EXP 1 e mix seco/úmido.',
  },
  {
    id: 'drop',
    name: 'Drop Capo Polyphonic',
    specRef: 'Reafinação Descendente Polifônica',
    sourceStatus: 'fonte: verificada',
    notes: 'Afinação instantânea semitom a semitom para baixo preservando ataque de acordes.',
  },
  {
    id: 'harmonizer',
    name: 'Dual Intelligent Harmonizer',
    specRef: 'Harmonização Diatônica por Tonalidade/Escala',
    sourceStatus: 'fonte: verificada',
    notes: 'Dobra inteligente diatônica ajustável por tonalidade (C..B) e modo grego/menor.',
  },
  {
    id: 'detune',
    name: 'Micro Pitch Detune',
    specRef: 'Duas Vozes em Micro-Desafinação Estéreo',
    sourceStatus: 'fonte: verificada',
    notes: 'Espessamento tipo rack de estúdio sem oscilação perceptível de LFO.',
  },
  {
    id: 'wah',
    name: 'Cry Wah GCB95 / Clyde McCoy',
    specRef: 'Indutor Ressonante 500 mH + Cap 0,01 µF',
    sourceStatus: 'fonte: verificada',
    notes: 'Varredura ressonante 350 Hz – 2,2 kHz com fator Q variável, voicing Clyde McCoy e Auto-Wah.',
  },
  {
    id: 'mxr10',
    name: '10-Band Graphic EQ',
    specRef: '10 Filtros Gráficos ISO',
    sourceStatus: 'fonte: verificada',
    notes: '10 bandas ±12 dB (31.25 Hz a 16 kHz) com Volume e Gain independentes.',
  },
  {
    id: 'comp1176',
    name: 'FET 76 Limiting Amplifier',
    specRef: 'Peak Limiter FET Ultrarrápido',
    sourceStatus: 'fonte: verificada',
    notes: 'Ratios 4:1, 8:1, 12:1, 20:1 e modo All-Buttons-In com ataque ultrarrápido.',
  },
  {
    id: 're201',
    name: 'Space Tape RE-201 Echo',
    specRef: '12 Modos Multi-Head + Mola',
    sourceStatus: 'fonte: verificada',
    notes: 'Cabeças de fita 1..12, flutter/wow orgânico, EQ úmido e mola integrada.',
  },
  {
    id: 'plate',
    name: 'Studio Plate Reverb',
    specRef: 'Tanque Metálico Estéreo',
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
    makeChipButton('ui.subtab', `Pedais & Racks (${PEDAL_CATALOG.length})`, store.libraryTab === 'pedals', () => {
      store.libraryTab = 'pedals';
      store.emit();
    }),
    makeChipButton('ui.subtab', `Presets & Pacotes (${store.presets.length})`, store.libraryTab === 'presets', () => {
      store.libraryTab = 'presets';
      store.emit();
    })
  );
  hdr.appendChild(tabs);

  const list = document.createElement('div');
  list.className = 'scroll-y';
  list.style.cssText =
    'flex:1;min-height:0;overflow-y:auto;display:flex;flex-direction:column;gap:10px;padding-right:4px;';

  if (store.libraryTab === 'presets') {
    const pkgCard = makePanelCard('Pacotes .sforgepkg (Presets + IRs)', 'Formato v2 Base64 · SHA-256 · Offline-First');
    pkgCard.root.style.flex = 'none';
    const selectedIds = new Set<string>();
    const active = store.presets[store.presetIdx];
    if (active) selectedIds.add(active.id);

    const actions = document.createElement('div');
    actions.style.cssText = 'display:flex;flex-wrap:wrap;gap:8px;align-items:center;';

    const exportBtn = document.createElement('button');
    exportBtn.type = 'button';
    exportBtn.className = 'm-btn m-pri';
    exportBtn.dataset.controlId = 'preset.export_package';
    exportBtn.style.cssText = 'height:32px;padding:0 14px;font-size:11.5px;font-weight:700;';
    exportBtn.textContent = 'Exportar pacote';
    exportBtn.addEventListener('click', () => {
      const ids = selectedIds.size > 0 ? [...selectedIds] : active ? [active.id] : [];
      void store.exportPackageDialog(ids);
    });

    const importBtn = document.createElement('button');
    importBtn.type = 'button';
    importBtn.className = 'm-btn';
    importBtn.dataset.controlId = 'preset.import_package';
    importBtn.style.cssText = 'height:32px;padding:0 14px;font-size:11.5px;font-weight:700;';
    importBtn.textContent = 'Importar pacote';
    importBtn.addEventListener('click', () => void store.importPackageDialog());

    const hint = document.createElement('span');
    hint.className = 'm-mono';
    hint.style.cssText = 'font-size:10.5px;color:#8e96a0;';
    hint.textContent = store.lastSavedStatus || 'Limites: arquivo <= 64 MiB · recurso <= 20 MiB · até 256 recursos';
    actions.append(exportBtn, importBtn, hint);

    const presetRows = document.createElement('div');
    presetRows.style.cssText = 'display:grid;grid-template-columns:repeat(auto-fill,minmax(260px,1fr));gap:8px;';
    store.presets.forEach((p, idx) => {
      const row = document.createElement('div');
      row.style.cssText =
        'display:flex;align-items:center;justify-content:space-between;gap:8px;padding:8px 10px;border-radius:6px;background:#12151a;border:1px solid #252b34;';
      const chkWrap = document.createElement('label');
      chkWrap.style.cssText = 'display:flex;align-items:center;gap:8px;cursor:pointer;font-size:12px;color:#dbe1e9;';
      const chk = document.createElement('input');
      chk.type = 'checkbox';
      chk.checked = selectedIds.has(p.id);
      chk.addEventListener('change', () => {
        if (chk.checked) selectedIds.add(p.id);
        else selectedIds.delete(p.id);
      });
      const lbl = document.createElement('span');
      lbl.textContent = `${p.name} (${p.id})`;
      chkWrap.append(chk, lbl);
      const loadBtn = makeChipButton('topbar.preset_load', idx === store.presetIdx ? 'Ativo' : 'Carregar', idx === store.presetIdx, () => {
        void store.loadPresetAt(idx);
      });
      row.append(chkWrap, loadBtn);
      presetRows.appendChild(row);
    });
    pkgCard.body.append(actions, presetRows);
    list.appendChild(pkgCard.root);
  } else if (store.libraryTab === 'amps') {
    for (const entry of AMP_CATALOG) {
      const card = makePanelCard(entry.name, `${entry.specRef} · ${entry.sourceStatus}`);
      card.root.style.flex = 'none';
      const note = document.createElement('div');
      note.style.cssText = 'display:flex;align-items:center;justify-content:space-between;gap:10px;font-size:11px;color:#b2b9c2;';
      const badgeColor = entry.sourceStatus === 'fonte: verificada' ? '#4ee680' : '#ffb84d';
      note.innerHTML = `<span>${entry.notes}</span><span class="m-mono" style="flex:none;padding:2px 6px;border-radius:4px;border:1px solid ${badgeColor};color:${badgeColor};font-size:9.5px;">${entry.sourceStatus}</span>`;
      const headWrap = document.createElement('div');
      headWrap.style.cssText = 'width:788px;height:226px;position:relative;overflow:hidden;border-radius:10px;';
      const headInner = document.createElement('div');
      headInner.style.cssText = 'position:absolute;left:0;top:0;width:1000px;transform:scale(0.788);transform-origin:0 0;';
      headInner.appendChild(createAmpHeadElement(store, { modelOverride: entry.id }));
      headWrap.appendChild(headInner);
      const loadBtn = makeChipButton('amp.model', 'Ativar no Estúdio', store.getParam('amp_1.model', '') === (entry.id as string), () => {
        void store.setControl('amp.model', entry.id);
        store.setScreen('main');
      });
      card.body.append(note, headWrap, loadBtn);
      list.appendChild(card.root);
    }
  } else {
    const grid = document.createElement('div');
    grid.style.cssText = 'display:grid;grid-template-columns:repeat(auto-fill,minmax(330px,1fr));gap:10px;';
    for (const item of PEDAL_CATALOG) {
      const card = makePanelCard(item.name, item.sourceStatus);
      const p = document.createElement('p');
      p.style.cssText = 'font-size:11px;color:#aeb5be;';
      p.textContent = `${item.specRef} — ${item.notes}`;
      const previewWrap = document.createElement('div');
      const isRack = item.id === 'comp1176' || item.id === 're201' || item.id === 'plate';
      previewWrap.style.cssText =
        `display:flex;justify-content:center;align-items:center;padding:8px;background:#0b0d10;border-radius:6px;overflow:hidden;min-height:${isRack ? 120 : 216}px;position:relative;`;
      if (isRack) {
        const origH = item.id === 're201' ? 190 : 130;
        const scale = 0.56;
        const slot = document.createElement('div');
        slot.style.cssText = `width:${Math.round(560 * scale)}px;height:${Math.round(origH * scale)}px;position:relative;flex:none;`;
        const inner = document.createElement('div');
        inner.style.cssText = `position:absolute;left:0;top:0;width:560px;height:${origH}px;transform:scale(${scale});transform-origin:0 0;`;
        inner.appendChild(createRackElement(item.id as RackModelId, store));
        slot.appendChild(inner);
        previewWrap.appendChild(slot);
      } else {
        const hw = createPedalElement(item.id as PedalModelId, store);
        previewWrap.appendChild(hw);
      }
      card.body.append(p, previewWrap);
      grid.appendChild(card.root);
    }
    list.appendChild(grid);
  }

  root.append(hdr, list);
  return root;
}
