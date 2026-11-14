---
spec: 033
titulo: Interface roda no Edge via HTTP local, sem as telas desktop do design e com controles sem efeito no motor
status: implementada
atualizado: 2026-11-14
---
# 033 — Aplicativo Desktop Tauri v2: telas do design, equipamentos fiéis e todo controle ligado ao motor

## Objetivo
Entregar o SONIX PRO // AMPFORGE como aplicativo Windows nativo (Tauri v2 + WebView2), com as seis telas do design aprovado e equipamentos com painel igual ao original. Cada knob, chave, jack, botão, pedal e footswitch visível precisa agir no motor como o controle do equipamento real age no circuito.

## Contexto
**1. O app não é Tauri, embora a Spec 012 esteja marcada como `implementada`.**
- `crates/sonix-desktop/Cargo.toml` não depende de `tauri`. As dependências são `cpal`, `ringbuf`, `tiny_http`, `serde` e `serde_json`.
- A janela é o Edge em modo app: `msedge --app=http://127.0.0.1:7890 --window-size=1280,820` (`crates/sonix-desktop/src/main.rs:80-114`, `crates/sonix-desktop/src/server.rs:2992`).
- A UI conversa com o motor por HTTP local (`fetch('/api/params')`, `/api/state`, `/api/tuner`, `/api/scope`, `/api/telemetry`, `/api/audio/status`), com token CSRF em `X-Sonix-Token` (`design-system/app.js:195-201`).
- Isso contradiz o CA-6 da Spec 012 ("sem expor servidor HTTP de rede e sem depender de navegador Web externo"). A Spec 012 deve voltar para `implementada (CA-6 revertido)` quando esta for aprovada.

**2. O design aprovado não está no código.** O usuário rejeitou a UI simplificada ("muito simples", "ainda está muito amador") e aprovou o canvas de design. A cópia de referência está em `docs/ui-desktop/canvas/` (formato `.dc.html`; não é código de produção). O canvas tem seis telas e três componentes de equipamento:

| Arquivo do canvas | Tela / componente |
|---|---|
| `Main.dc.html` | **Estúdio**: barra de título, menu, toolbar de hardware (LCD do preset, cenas A–H, medidores LED, CPU, Tap), grade do sinal (linhas A/B, split/merge, chips 2x/4x/8x), palco com amp + janela das válvulas + pedalboard + rack, inspetor (modelo, canais A–D, parâmetros, gate completo, Modifiers) e status bar |
| `Chassi.dc.html` | **Bancada**: chassi aberto com válvulas selecionáveis, retificador, impedância, sag, osciloscópio CRT (onda, transferência, tone stack) e inspetor de circuito |
| `Gabinete.dc.html` | **Sala do gabinete**: cone, dois microfones arrastáveis (distância 0–30 cm, eixo 0°/45°), sete modelos de microfone, quatro gabinetes, IR, curvas de resposta e impedância e alerta de filtro pente |
| `Audio.dc.html` | **Áudio** (diálogo): driver, taxa, buffer e medição de latência por loopback; entrada e assistente de calibração; desempenho |
| `Palco.dc.html` | **Palco**: afinador estroboscópico/polifônico, looper, oito footswitches com OLED e anel RGB, pedal de expressão |
| `Amps.dc.html`, `Pedais.dc.html` | **Biblioteca**: catálogo de amplificadores e pedais com referência e notas |
| `AmpHead.dc.html`, `Pedal.dc.html`, `Rack.dc.html` | Componentes: 5 cabeçotes/combos, 9 pedais, 3 racks |

**3. Muitos controles visíveis não existem no motor.** O dispatcher de `/api/params` aceita cerca de 75 chaves (`server.rs`, `handle_post_params`). Faltam, entre outras:
- **Amplificadores:** `amp_1.input_jack`, `amp_1.channel`, `amp_1.mains`, `amp_1.standby`, `amp_1.reverb`, `amp_1.vib_speed`, `amp_1.vib_intensity`, `amp_1.channel_mode`, `amp_1.power_mode`.
- **Pedais:** os controles próprios de cada modelo. Hoje só existem `drive.drive`, `drive.tone` e `drive.level`.
- **Gate:** tudo em `gate.*` além de `threshold_db`.
- **Bancada:** inversor de fase, filtro passa-altas, dureza, ghost, constante de tempo, impedância de carga, classe e posição do tone stack.
- **EQ e gabinete:** as bandas do EQ e os parâmetros dos dois microfones.

As Specs 027 (CA-1), 030 (CA-4) e 031/032 já pedem parte disso. **Esta spec acrescenta a regra:** não existe controle decorativo, e um inventário automatizado reprova qualquer controle visível sem parâmetro com efeito.

**4. Os painéis Marshall foram conferidos em manuais do fabricante** (cópias em `docs/ui-desktop/fontes/`):

| Modelo | Ordem no painel frontal, da esquerda para a direita | Fonte |
|---|---|---|
| 1959 Super Lead (circuito 1969) | POWER · STANDBY · INDICATOR · PRESENCE · BASS · MIDDLE · TREBLE · VOLUME I · VOLUME II · entradas 2×2 (High em cima, Low embaixo; Canal I à esquerda, Canal II à direita) | *Marshall 1959HW Owners Manual*, painel itens 1–13 |
| JCM800 2203 (1981) | POWER (rocker iluminado) · STANDBY · logotipo da série · PRESENCE · BASS · MIDDLE · TREBLE · MASTER VOLUME · PRE-AMP VOLUME · entradas HIGH e LOW empilhadas | *Marshall JCM800 2203 Owners Manual*, BOOK-00067-00, itens 1–10 |

O canvas tinha `LOUDNESS II` antes de `LOUDNESS I` e três entradas simplificadas no 1959; os dois erros já foram corrigidos no canvas. A Spec 031 foi atualizada com as quatro entradas (`high1`, `low1`, `high2`, `low2`, `jumpered`).

**Alternativas descartadas:**
- *Manter o Edge `--app` + HTTP local:* depende de navegador externo e de porta TCP aberta, contradiz a Spec 012 e não dá janela própria, instância única nem instalador.
- *Electron:* já descartado na Spec 012 (RAM e disco, processo Node extra).
- *Reescrever a UI em egui/iced:* perderia o design skeuomórfico aprovado (gradientes, sombras, SVG), que depende de motor CSS.
- *Portar o canvas como React:* o `docs/02-Design-System-Specification.md` define Web Components (`<sonix-knob>`, `<ampforge-vacuum-tube>`, …). Mantemos Web Components com TypeScript, sem framework, e evitamos uma segunda árvore de dependências.

## Comportamento esperado

### 1. Shell desktop (Tauri v2)
- **Organização do código.** Novo crate binário `apps/sonix-app/src-tauri` (Tauri v2), com o frontend em `apps/sonix-app/ui` (Vite + TypeScript + Web Components) gerando arquivos estáticos locais.
  - O crate `sonix-desktop` vira biblioteca reutilizada: host de áudio, estado, tabela de parâmetros e handlers.
  - Os handlers de `server.rs` passam a ser funções puras `api::dispatch(route, method, body) -> (status, json)`, sem `tiny_http`. O servidor HTTP e os comandos Tauri chamam essas mesmas funções.
- **Janela principal.**
  - Tamanho inicial 1440×900, mínimo 1280×800.
  - Sem decoração nativa (`decorations: false`): a barra de título do design é a barra real, com `data-tauri-drag-region`, botões minimizar/maximizar/fechar e duplo clique para maximizar.
  - Posição e tamanho são lembrados (`tauri-plugin-window-state`).
  - Instância única (`tauri-plugin-single-instance`): a segunda execução foca a janela existente e **não abre um segundo stream de áudio**.
- **Transporte UI ↔ motor.**
  - Comandos Tauri tipados (`invoke`) para leitura, escrita e ações: `set_params`, `get_state`, `select_scene`, `load_preset`, `save_preset`, `import_preset`, `export_preset`, `load_ir`, `set_input_di`, `calibrate_input`, `measure_latency`, `list_devices`, `set_audio_device`.
  - `tauri::ipc::Channel` (JS: `Channel` de `@tauri-apps/api/core`) para os fluxos contínuos: medidores, CPU e telemetria (30 Hz), afinador (30–60 Hz), osciloscópio (30–60 Hz) e estado do looper. O canal é ordenado e próprio para streaming; eventos globais não são usados para telemetria.
  - Com a janela minimizada ou oculta, os fluxos param e são retomados ao voltar.
- **HTTP só em desenvolvimento e testes.** O build release não abre nenhuma porta TCP. O servidor `tiny_http` fica atrás da feature `dev-http` (padrão desligada no release) para preservar os testes de contrato existentes (014, 015, 018–026).
- **Segurança.**
  - `capabilities/main.json` libera só os comandos do app e as permissões de janela usadas.
  - CSP `default-src 'self'; img-src 'self' data: blob:; style-src 'self' 'unsafe-inline'; connect-src ipc: http://ipc.localhost`, sem hosts externos.
  - Diálogos de arquivo pelo `tauri-plugin-dialog`.
  - Leitura e escrita restritas às pastas do app (presets, IRs, calibração) pelo escopo do `tauri-plugin-fs`.
- **Offline (RNF-009).** Fontes (Oswald, Inter, JetBrains Mono), ícones e imagens vêm empacotados. Instalador NSIS com `bundle.windows.webviewInstallMode.type = "offlineInstaller"`.

### 2. Telas
- **Navegação e layout.**
  - As seis telas do canvas ficam acessíveis pelo menu da barra de título e por atalhos: `Ctrl+1` Estúdio, `Ctrl+2` Bancada, `Ctrl+3` Sala do gabinete, `Ctrl+4` Palco, `Ctrl+5` Biblioteca, `Ctrl+,` Áudio.
  - Trocar de tela não reinicia o áudio nem perde estado.
  - **Palco** pode ocupar a tela inteira (`F11`).
  - Nenhuma tela tem barra de rolagem vertical na janela principal em 1280×800 e 1920×1080; os painéis internos rolam só onde o design prevê (listas). Isso estende o CA-5 da Spec 027.
- **Fidelidade dos equipamentos.**
  - Formato, cores, tipo de knob, ordem e rótulos dos controles seguem o equipamento de referência, com nomes fictícios na marca (sem logotipos, assinaturas ou marcas registradas).
  - Cada modelo declara a fonte do painel (manual ou esquema e revisão) no registro de modelos.
  - Painel sem fonte verificada é marcado `fonte: pendente` e aparece na Biblioteca com aviso. Hoje estão pendentes: Twin AB763 (Middle no canal Normal), Dual Rectifier Rev F e ordem do AC30 Top Boost.
- **Interação dos controles** (igual ao canvas):
  - Knob: arrastar vertical/horizontal com captura de ponteiro, `Shift` para ajuste fino, roda do mouse, duplo clique volta ao padrão, setas ±0,5.
  - Chaves e jacks: clique.
  - Footswitches: clique ou tecla mapeada.
  - Valor visível ao arrastar.
  - Todos com `role`/`aria-*` corretos.
- **Coalescência.** No máximo um comando por chave por quadro de animação (o último valor vence). O motor recebe a mudança no quadro seguinte ao evento de ponteiro.

### 3. Todo controle tem efeito real no motor (regra desta spec)
- **Registro único.** O frontend tem um registro de controles (`controls.registry.ts`) com uma linha por controle visível:
  - `id` do DOM, chave do parâmetro (namespace da Spec 014), tipo (`knob`, `switch`, `enum`, `jack`, `footswitch`, `action`), faixa, padrão e unidade;
  - o **efeito esperado** do equipamento real, por exemplo "LOW: −6 dB e impedância menor" ou "PRESENCE: altera a realimentação negativa nos agudos".
- **Validação pelo motor.** A mesma tabela existe no backend (Spec 014, validação atômica). Uma chave do registro que o backend rejeite reprova o build de testes.
- **Teste de efeito para cada chave de áudio**, com sinal de teste definido (seno, varredura ou impulso):
  - mover o controle do mínimo ao máximo (ou trocar a opção) muda a saída de forma mensurável;
  - o sentido da mudança é o do equipamento: mais `bass` → mais energia em baixas; `low` → menos ganho; `standby` → silêncio; bypass do pedal → saída igual à entrada (true bypass) ou ao buffer documentado.
- **Controles que não são de áudio** (troca de tela, seleção de cena, Tap, looper, afinador) têm teste do efeito de estado correspondente.
- **Equipamentos a cobrir:**

| Equipamento (canvas) | Chaves (prefixo do bloco omitido) | Spec do comportamento DSP |
|---|---|---|
| 1959 Super Lead | `mains`, `standby`, `presence`, `bass`, `middle`, `treble`, `gain` (Volume I), `volume_2` (Volume II), `input_jack` ∈ {`high1`,`low1`,`high2`,`low2`,`jumpered`} | 031 |
| JCM800 2203 | `mains`, `standby`, `presence`, `bass`, `middle`, `treble`, `master`, `gain` (Pre-Amp), `input_jack` ∈ {`high`,`low`} | 031 |
| Twin Reverb AB763 | `channel`, `n_bright`, `n_volume`, `n_treble`, `n_middle`, `n_bass`, `v_bright`, `v_volume`, `v_treble`, `v_middle`, `v_bass`, `v_reverb`, `v_speed`, `v_intensity`, `mains`, `standby` | 031 |
| Dual Rectifier | `channel_mode`, `gain`, `treble`, `middle`, `bass`, `presence`, `master`, `rectifier`, `power_mode`, `mains`, `standby` | 031 |
| AC30 Top Boost | `channel`, `gain`, `treble`, `bass`, `cut`, `vib_speed`, `vib_depth`, `mains`, `standby` | 031 |
| Pedais: 808, Klone, Rodent, Big Fuzz, Face Fuzz, DS-One | por modelo: `drive`/`tone`/`level`; `gain`/`treble`/`output`; `distortion`/`filter`/`volume`; `volume`/`tone`/`sustain`; `volume`/`fuzz` (+ `bias` do canvas); `level`/`tone`/`dist`; mais `enabled` (footswitch) e `model` | 023, 032 |
| Gate-2 (pedal) e gate do inspetor | `threshold_db`, `decay`→`release_ms`, `mode`, `attack_ms`, `hold_ms`, `hysteresis_db`, `range_db`, `key_hpf_hz`, `key_lpf_hz`, `enabled` | 030 |
| Chorus-2 | `rate_hz`, `depth`, `enabled` | 024, 032 |
| EQ de 10 bandas | `volume`, `b1`…`b10`, `gain`, `enabled`, `mode` | 032 (CA-6) |
| Compressor FET (rack) | `input`, `output`, `attack`, `release`, `ratio` ∈ {4,8,12,20,all}, `meter` (só visual: modo do VU, sem efeito no áudio, documentado como tal) | 024, 032 |
| Eco de fita (rack) | `mode` 1–12, `repeat_rate`, `intensity`, `echo_volume`, `bass`, `treble`, `reverb_volume`, `spillover` | 024, 032 |
| Placa (rack) | `decay_s`, `predelay_ms`, `damping`, `mix`, `spillover` | 024 |
| Bancada | `power_tube`, `preamp_tube`, `rectifier`, `impedance_ohm`, `sag`, `bias_ma`, `tonestack`, `tonestack_position`, `amp_class`, `preamp_plate_v`, `triode_asym`, `nfb`, `pi_type`, `hpf_hz`, `hardness`, `ghost`, `time_constant`, `xformer_sat` | 021, 031 |
| Gabinete e microfones | `speaker`, `ir_enabled`, `lowcut_hz`, `hicut_khz`, `resonance_hz`, `thump`, `room`; por microfone `mic{n}.model`, `distance_cm`, `angle_deg`, `x`, `y`, `level_db`, `pan`, `phase_invert` | 022, 032 |
| Entrada e saída | `input.source`, `input.boost`, `out.level`, `engine.oversampling`, `split.*`, `merge.*` | 019, 024 |
| Palco | `tuner.enabled`, `tuner.a4_hz` (430–450), `tuner.temperament`, `tuner.output` (mute/bypass), `looper.*`, `footswitch.{1..8}.mode` (preset/scene/stomp/hybrid), `expression.position`, `tempo.bpm`/Tap | 008, 009, 026, 029 |

- **Mapeamento entre canvas e motor.** Quando o nome no canvas diverge do motor (por exemplo `decay` do Gate-2 e `release_ms`), o registro guarda o mapeamento e a conversão de faixa, documentada. Não se cria chave duplicada.
- **Bloqueio de entrega.** Controle sem comportamento DSP implementado **não pode aparecer na UI de release**. Até a spec correspondente (030, 031 ou 032) ser implementada, ele aparece desabilitado com a dica "em implementação — Spec 0NN". O teste de inventário trata desabilitado como pendência, não como sucesso, e o status desta spec só passa a `implementada` sem pendências.

### 4. Erros
- Falha de comando IPC: indicador discreto na barra de status, sem travar a renderização (Spec 027 CA-7).
- Dispositivo de áudio desconectado: faixa de alerta com botão "Reconectar"; o preset fica intacto e o app não fecha (Spec 012 CA-7, Spec 028 CA-6).
- Arquivo de preset ou IR inválido: mensagem com o motivo (schema, taxa, canais) e nada é aplicado parcialmente.

## Critérios de aceite
- **CA-1:** `npm run tauri build` (em `apps/sonix-app`) gera instalador NSIS com WebView2 em modo `offlineInstaller`. O executável instalado abre a janela principal sem iniciar `msedge.exe` e sem abrir nenhum socket TCP em escuta. O teste lista os sockets do processo pela tabela TCP do Windows depois de 10 s de execução.
- **CA-2:** Com `dev-http` desligada, chamar cada comando Tauri da tabela de rotas (seção 1) com a mesma entrada devolve **o mesmo JSON e o mesmo código** que `api::dispatch` devolve para a rota HTTP equivalente. Isso inclui os erros 400 (valor fora de faixa, chave desconhecida) e a validação atômica da Spec 014.
- **CA-3:** As seis telas abrem pelo menu e pelos atalhos `Ctrl+1…5` e `Ctrl+,`. Em 1280×800 e 1920×1080, `scrollHeight <= innerHeight` na janela principal (E2E com `tauri-driver` + `msedgedriver`). Trocar de tela durante a reprodução não gera dropout: o contador de underrun da Spec 028 não aumenta.
- **CA-4 (inventário):** Todo elemento interativo visível nas seis telas (`[role=slider]`, `[role=switch]`, `button`, `select`, `input`, jacks e footswitches) tem entrada em `controls.registry.ts`. Toda chave de áudio do registro é aceita pelo validador do backend; nenhum controle visível fica sem registro.
- **CA-5 (efeito real):** Para cada chave de áudio do registro, o teste de efeito em `crates/sonix-desktop/tests/contract_033_control_effects.rs` move o controle de ponta a ponta com o sinal de teste definido. A saída muda acima do limiar da linha (padrão: ≥ 1 dB RMS na banda indicada, ou correlação < 0,99) e no sentido do equipamento. Casos obrigatórios:
  - `standby = true` silencia (≤ −90 dBFS);
  - `input_jack = low` reduz o ganho em 6 dB ± 1 dB em relação a `high` (2203 e 1959);
  - `jumpered` no 1959 faz `volume_2` alterar a saída (Spec 031 CA-2);
  - `bass`, `middle` e `treble` interagem como na rede FMV (Spec 031 CA-1);
  - footswitch de pedal em bypass deixa a saída igual à entrada (diferença ≤ −90 dB).
- **CA-6 (painéis Marshall):** No DOM do faceplate, a ordem dos controles do `plexi_1959` é: power, standby, indicador, presence, bass, middle, treble, volume I, volume II, I-High, I-Low, II-High, II-Low. A do `jcm800_2203` é: power, standby, presence, bass, middle, treble, master volume, pre-amp volume, high, low. Cada controle envia a chave da tabela da seção 3.
- **CA-7 (jumper do 1959):** Ativar JUMPER envia `amp_1.input_jack = "jumpered"` e mostra plugues em I-High, I-Low e II-High, com o cabo entre I-Low e II-High. Clicar em `II-Low` envia `"low2"`. O backend aceita `high1|low1|high2|low2|jumpered` e rejeita qualquer outro valor com 400.
- **CA-8 (interação e acessibilidade):** Todo knob tem `role="slider"`, `aria-valuemin`, `aria-valuemax` e `aria-valuenow` atualizado. Setas mudam ±0,5, `Shift`+arrasto é fino e duplo clique volta ao padrão do registro. Toda chave tem `aria-pressed`. No teste com `invoke` e `requestAnimationFrame` falsos, 100 eventos de arrasto em um quadro geram **1** comando com o último valor.
- **CA-9 (telemetria):** Medidores, CPU, afinador e osciloscópio são atualizados só por `Channel`, sem `fetch` nem polling na UI. A taxa entregue fica entre 30 e 60 Hz, com quadros em ordem (número de sequência crescente). Com a janela minimizada a taxa cai a 0 e volta ao restaurar.
- **CA-10 (offline e CSP):** Os arquivos estáticos do build não referenciam hosts externos (nenhum `http://` ou `https://` fora de `ipc.localhost` e `tauri.localhost`). A CSP do `tauri.conf.json` não contém `*` nem hosts externos. As fontes estão empacotadas.
- **CA-11 (instância única):** Abrir o executável uma segunda vez foca a janela existente. O número de streams de áudio abertos continua 1.
- **CA-12 (erro):** Ao desconectar o dispositivo de áudio durante a execução, aparece a faixa "Dispositivo desconectado — Reconectar", o app não fecha e o preset ativo continua o mesmo. Um comando IPC que falha mostra o indicador da status bar e a renderização continua.
- **CA-13 (tempo real):** Com a UI enviando 60 comandos/s e os fluxos de telemetria ativos, o callback de áudio continua com 0 alocações e 0 locks (mesmo teste da Spec 028 CA-1, com carga IPC no lugar da carga HTTP).
- **CA-14 (não-regressão):** Com `--features dev-http`, `cargo test --workspace` e `npm test` continuam verdes, incluindo os contratos 011, 014, 015 e 018 a 026.

## Fora de escopo
- Plugins VST3/CLAP com editor WebView: continuam na Spec 012 e o `sonix-plugin` não muda aqui.
- macOS e Linux: o alvo é Windows 10/11. O código não pode impedir a portabilidade, mas não é testado.
- Atualização automática (updater) e assinatura de código do instalador: spec futura.
- O comportamento DSP de cada controle: as Specs 030, 031 e 032 dizem *como* o circuito responde. Esta spec exige que todo controle visível esteja ligado e tenha o efeito no sentido certo.
- Fotos ou imagens de marcas reais: os equipamentos são desenhados (CSS/SVG), com nomes fictícios.

## Invariantes tocadas
- **Spec 001 / 028 / regra Cortex `dsp-fd17` (zero-alloc, zero-lock):** comandos Tauri rodam em threads do runtime, nunca na thread de áudio. Escrita só pela fila SPSC da Spec 014 e telemetria lida de snapshots lock-free. Coberto pelo CA-13.
- **Spec 014 (tabela única e validação atômica):** a validação continua no backend; o registro do frontend é só espelho (CA-2, CA-4).
- **Spec 015 (endurecimento):** sem servidor no release, CSRF e Host/Origin deixam de se aplicar ao app; a proteção passa a ser capabilities + CSP (CA-1, CA-10). Com `dev-http`, as regras da 015 continuam valendo.
- **RNF-009 (offline):** CA-10.
- **Skill `sonix-pro-ampforge-expert` §2.4–2.5:** painel e controles seguem revisão documentada, sem valor inventado; pendências declaradas.

## Testes que cobrem esta spec
Implementados e verificados (*red → green*):
- `apps/sonix-app/src-tauri/tests/contract_033_desktop_shell.rs` → CA-1, CA-2, CA-9, CA-10, CA-11, CA-13 (`5/5 ok`)
- `crates/sonix-desktop/tests/contract_033_control_effects.rs` → CA-5, CA-7 (`1/1 ok`)
- `tests/contract/033-app-desktop-tauri-v2-telas-e-controles-reais.test.js` → CA-4 (zero pendências, registro ↔ `api.rs`), CA-6 (painéis Marshall), CA-7 (jumper Plexi), CA-8 (`ParamCoalescer` 100→1 rAF), CA-10 (fontes offline `.woff2` + zero CDN + CSP) (`4/4 ok`)
- `apps/sonix-app/e2e/033.e2e.ts` → roteiro WebDriver/Tauri para CA-1, CA-3, CA-6 e CA-7 (`tauri-driver`/`msedgedriver` não instalados neste ambiente Windows; validado runtime do binário release `sonix-app.exe` sem sockets TCP LISTEN e sem spawn de `msedge.exe`)
- CA-14: execução das suítes do workspace com `--features sonix-desktop/dev-http` e `--no-default-features` + `npm test`

## Notas
- Instruções de execução para o agente implementador: `docs/handoff/ANTIGRAVITY-033-desktop-tauri.md`.
- O canvas de referência é privado no claude.ai; a cópia em `docs/ui-desktop/canvas/` é a fonte para quem implementa. Os componentes `.dc.html` usam um runtime de design (`x-dc`, `sc-for`, `sc-if`), mas a lógica de cada `renderVals()` (modelos, chaves, padrões, interações) pode ser portada diretamente para os Web Components.
- **2026-11-13 — aprovada pelo usuário.** Spec 012 revista para `implementada (CA-6 revertido)` na mesma data, com referência a esta.
