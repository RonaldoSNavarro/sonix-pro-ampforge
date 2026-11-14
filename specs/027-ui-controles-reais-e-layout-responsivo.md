---
spec: 027
titulo: Controles inertes na UI, editor de blocos incompleto e layout com barra de rolagem vertical
status: implementada
atualizado: 2026-11-13
---
# 027 — UI 100% Conectada ao Motor DSP, Editor de Todos os Blocos e Layout Responsivo sem Scroll

## Objetivo
Conectar todos os controles visíveis da interface ao motor DSP em tempo real, fornecer editores completos para cada bloco do grid 4×12 e eliminar a barra de rolagem vertical em janelas desktop padrão (1280×820 e 1920×1080).

## Contexto
Origem: code review total solicitado pelo usuário em 2026-11-12 com captura de tela anexada. Evidências no código atual:

1. **Controles desconectados do backend (`design-system/app.js`):**
   - `updateKnobValue` (`app.js:441-455`) atualiza apenas `state.ampParams[key]` e a rotação CSS; **não chama** `syncParamsToBackend`, então girar qualquer knob do amplificador na UI não altera o som.
   - `KNOB_DEFINITIONS` (`app.js:87-96`) renderiza sempre os mesmos 8 knobs (`inputGain`, `overdrive`, `bass`, `middle`, `treble`, `presence`, `depth`, `masterVol`), ignorando o modelo selecionado em `#ampModelSelect` (contrariando a Spec 020 CA-3).
   - `#oversamplingSelect` (`app.js:953-956`) só chama `updateCpuBudget()` local, sem enviar `engine.oversampling`.
   - `#brightSwitchBtn` (`app.js:1006-1009`) não envia `amp_1.bright`.
   - `#expPedalSlider` (`app.js:1012-1019`) chama `updateKnobValue` (que não sincroniza com o backend).
   - Sliders e selects do Mod Shop (`sliderBias`, `sliderSag`, `sliderNfb`, `sliderXformer`, `sliderPlateV`, `sliderAsym`, `powerTubeSelect`, `preampTubeSelect`, `toneStackSelect`, `rectifierSelect` em `app.js:979-1003`) não enviam nenhum parâmetro `amp_1.*` ao backend.
   - Controles da sala de gabinete (`sliderMic1Dist`, `sliderMic2Dist`, `sliderCabRes`, `sliderCabHiCut`, `checkCabIrEnabled`, `irFileInput`, `impedanceCurveSelect` em `index.html:444-499`) estão sem listeners de envio completo em `app.js`.
   - `#cpuMeterText` (`index.html:64`) permanece travado em `0.0%` porque `pollBackendTelemetry()` (`app.js:1153-1175`) não lê `data.cpu_usage_pct`.
   - Os indicadores de áudio (`#audioModeText`, `#audioSampleRateText`, `#audioPeriodsText`, `#audioMeasuredRoundtripText` em `index.html:70-76`) são texto estático no HTML; a UI nunca consulta `GET /api/audio/status`.

2. **Editor do bloco selecionado incompleto (`app.js:270-354`):**
   - Apenas `blk_drive` renderiza sliders (`DRIVE`, `TONE`, `LEVEL`).
   - `blk_comp_2`, `blk_eq_1`, `blk_mod_2`, `blk_dly_1`, `blk_rev_1` e `blk_split` exibem apenas uma linha de texto estático (`"Controles comp_2: input, output, attack, release, ratio"` etc.), sem nenhum input interativo.
   - `blk_gate`, `blk_amp_1`, `blk_amp_2`, `blk_cab_1`, `blk_cab_2` caem no `else` e exibem `"sem controles nesta versão"` (confirmado no print do usuário ao selecionar `Natural Gate`).

3. **Overflow vertical e falta de responsividade (`design-system/styles.css` e `index.html`):**
   - O `INICIAR_SONIX_PRO.bat:62` e `main.rs:81` abrem a janela com `--window-size=1280,820`.
   - `.app-shell` (`styles.css:86-94`) empilha `top-master-bar` + `workspace-subnav` + `signal-grid-section` (4 linhas × 50px + paddings + `blockEditor`) + `main-stage-section` (`amp-head-outer-shell`) + `hardware-footswitch-dock` sem restrição de altura de viewport, ultrapassando 950 px e forçando barra de rolagem vertical na lateral direita.

## Comportamento esperado
1. **Sincronização bidirecional real:** Todo controle visível na UI envia seu respectivo parâmetro via `POST /api/params` (ou endpoint dedicado) e reflete o estado retornado por `GET /api/state` e `GET /api/audio/status`.
2. **Telemetria real no cabeçalho:** `cpuMeterText` exibe `cpu_usage_pct` real vindo do motor; os badges de modo WASAPI/ASIO, taxa de amostragem, frames de período (`in/out`) e latência medida são atualizados a partir de `GET /api/audio/status`.
3. **Painel do amplificador por modelo:** Selecionar qualquer modelo em `#ampModelSelect` (ou carregar preset) reconstrói dinamicamente os knobs e chaves do painel frontal para corresponder exatamente aos controles do modelo ativo.
4. **Editor interativo para todo bloco do grid:** Clicar em qualquer bloco populado do grid 4×12 abre controles reais (knobs/sliders/selects/toggles) para todos os parâmetros daquele bloco, nunca exibindo texto estático nem `"sem controles nesta versão"`.
5. **Layout responsivo sem barra de rolagem vertical:** Em desktop (1280×820 e 1920×1080), a aplicação preenche 100% da largura útil e se ajusta dentro de `100vh` sem barra de rolagem vertical no `body`/janela principal. Quando o `blockEditor` ou o `Mod Shop` estiver aberto, a área central adapta sua proporção (`flex: 1; min-height: 0`) com escalamento fluido de alturas e paddings (`clamp()`).

## Critérios de aceite
- **CA-1:** Girar qualquer knob do painel frontal do amplificador dispara `POST /api/params` com a chave correspondente (`amp_1.gain`, `amp_1.bass`, etc.) dentro de ≤ 50 ms e altera o áudio processado.
- **CA-2:** Alterar `#oversamplingSelect`, `#brightSwitchBtn`, `#expPedalSlider`, qualquer slider/select do Mod Shop (`sliderBias`, `sliderSag`, `sliderNfb`, `sliderXformer`, `sliderPlateV`, `sliderAsym`, `preampTubeSelect`, `powerTubeSelect`, `toneStackSelect`, `rectifierSelect`) ou da sala de gabinete (`sliderMic1Dist`, `sliderMic2Dist`, `sliderCabRes`, `sliderCabHiCut`, `checkCabIrEnabled`, `impedanceCurveSelect`) envia a chave correspondente em `POST /api/params`.
- **CA-3:** Selecionar no grid os blocos `blk_gate`, `blk_drive`, `blk_split`, `blk_comp_2`, `blk_eq_1`, `blk_mod_2`, `blk_dly_1`, `blk_rev_1`, `blk_amp_1`/`2` ou `blk_cab_1`/`2` renderiza controles interativos (`<input>` ou `<select>` ou `<button>`) para cada parâmetro exposto pelo bloco; a string `"sem controles nesta versão"` não é exibida para nenhum bloco populado.
- **CA-4:** `#cpuMeterText` exibe o valor de `cpu_usage_pct` recebido de `/api/telemetry` (formatado com 1 casa decimal) e `#audioModeText`, `#audioSampleRateText`, `#audioPeriodsText` e `#audioMeasuredRoundtripText` refletem os campos reais de `/api/audio/status`.
- **CA-5:** Em viewport `1280×820` e `1920×1080`, com a aba principal aberta e um bloco selecionado no editor, `document.documentElement.scrollHeight <= window.innerHeight` (sem barra de rolagem vertical na janela principal) e `document.documentElement.scrollWidth <= window.innerWidth` (sem barra de rolagem horizontal).
- **CA-6:** Trocar `#ampModelSelect` entre os 5 modelos altera imediatamente o conjunto de knobs/chaves exibidos no faceplate conforme a tabela da Spec 020 / Spec 031.
- **CA-7:** Em caso de falha de rede ou resposta `4xx/5xx` de `/api/params`, a UI exibe indicador de erro discreto sem travar o loop de renderização (`requestAnimationFrame`).
- **CA-8:** Não-regressão — `tests/contract/011-*.test.js`, `014-*.test.js` e `020-*.test.js` permanecem compatíveis com os novos seletores/handlers.

## Fora de escopo
- Suporte a telas mobile/portrait (< 900 px de largura): o foco do standalone Windows é desktop/notebook ≥ 1280×720.
- Arrastar e soltar blocos para reordenar a matriz 4×12 (drag-and-drop de topologia do DAG): fica para spec futura.

## Invariantes tocadas
- Spec 014 (tabela única de parâmetros UI→DSP): todas as novas ligações devem usar exclusivamente as chaves canônicas da tabela de parâmetros.
- Spec 015 (CSRF/Token): toda requisição `POST` da UI deve incluir o header `X-Sonix-Token`.

## Testes que cobrem esta spec
Implementados (green): `tests/contract/027-ui-controles-reais-e-layout-responsivo.test.js` (CA-1…CA-8, 8/8 ok).
