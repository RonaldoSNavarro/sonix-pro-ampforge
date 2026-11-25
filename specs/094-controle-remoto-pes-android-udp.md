---
spec: 094
titulo: Controle Remoto pelos Pés via App Android (Câmera + IMU) e Receptor UDP Opt-In de Baixo Custo de CPU
status: rascunho
atualizado: 2026-11-25
---
# 094 — Controle Remoto pelos Pés via App Android (Câmera + IMU) e Receptor UDP Opt-In

> **Prioridade no roadmap:** `Rascunho — implementação planejada para o final do projeto, após a conclusão de todas as fases e specs pendentes (Specs 053–092 e Fases 2–4).`

## Objetivo
Permitir que o guitarrista controle pedais, Wah, Whammy, cenas, presets, Tap Tempo e Looper no Modo Palco usando os pés — sem adquirir pedaleira MIDI física e sem sobrecarregar a CPU do computador dedicado ao áudio em tempo real — descarregando 100% da captura de câmera e giroscópio para um aplicativo Android leve conectado por UDP local autenticado (Wi-Fi ou USB tethering).

## Contexto
Durante o planejamento da evolução do SONIX PRO // AMPFORGE™ (`2026-11-25`), o usuário relatou a necessidade prática de acionar efeitos contínuos (`Wah`, `Whammy`) e trocas discretas do Modo Palco (`Cenas`, `Presets`, liga/desliga de pedais) com os pés enquanto toca guitarra, sujeito a uma restrição severa de orçamento de CPU no computador Windows (que já opera no limite do orçamento RT de baixa latência em `48 kHz / 64` e `96 kHz / 32`, cf. `Spec 001` / `AUD-15`).

**O que já existe no repositório (reaproveitado integralmente):**
- Parâmetros de controle canônicos validados em `crates/sonix-desktop/src/params.rs`: `palco.exp_1`, `wah_1.enabled`, `wah_1.position`, `wah_1.mode`, `pitch_1.enabled`, `pitch_1.model`, `pitch_1.semitones`, `scene.active`, `footswitch.mode`, `looper.action`, `tuner.enabled`, `drive.enabled`, `dly_1.enabled`, `rev_1.enabled`, `mod_2.enabled`, `eq_1.enabled`, `comp_2.enabled`, `gate.enabled` e `master.level_db`.
- Roteamento centralizado `api::dispatch` + fila lock-free SPSC `LockFreeParamQueue` com propriedade exclusiva RT (`Spec 014`, `Spec 037`, `Spec 093`).
- Infraestrutura de `MidiRouter` (`crates/sonix-desktop/src/midi.rs`) e cenas/setlists (`Spec 045`, `Spec 049`).
- Invariante de segurança e rede local da `Spec 033 CA-1`: o binário release Tauri v2 não pode abrir nenhum socket TCP em escuta no boot nem depender de recursos externos (`Spec 033 CA-10`).

**Alternativas descartadas (com justificativa técnica):**
1. **Visão computacional / IA (MediaPipe / Pose Estimation) na webcam do PC:** consome `15–40%` de CPU no computador host, disputa banda do controlador USB com a interface de áudio e compromete diretamente o determinismo de tempo real.
2. **Web App / PWA servido no navegador do celular:** navegadores móveis (`Chrome` Android / `Safari` iOS) bloqueiam acesso à câmera (`getUserMedia`) e ao giroscópio (`DeviceOrientationEvent`) em rede local HTTP comum (exigindo certificado HTTPS autoassinado), não oferecem UDP puro de baixa latência e suspendem sensores ao escurecer a tela.
3. **Ponte exclusiva por cabo USB-MIDI:** embora aproveite `midi.rs`, elimina o uso sem fio em Wi-Fi local e impede retorno rico de estado (nome do preset ativo, cena, LEDs de stomp e nota do afinador) na tela do celular.
4. **Servidor TCP/HTTP aberto no boot:** violaria diretamente a `Spec 033 CA-1`.

## Comportamento esperado

### 1. Receptor UDP Opt-In no SONIX PRO (`crates/sonix-desktop` + `apps/sonix-app`)
- **Desligado por padrão (opt-in estrito):** no boot frio do aplicativo, nenhum socket UDP ou TCP de controle remoto é aberto (`0` portas em escuta). O receptor só é iniciado quando o usuário habilita explicitamente o modo *Pedaleira Remota* no Modo Palco (ou tela de Áudio) e é encerrado imediatamente ao desativar.
- **Pareamento offline por QR Code + Token Efêmero:**
  - Ao ativar o receptor, o backend abre um socket UDP local (`bind 0.0.0.0:<porta>` ou interface selecionada, suportando Wi-Fi LAN e *USB tethering* Android), gera um segredo de sessão CSPRNG de 256 bits (e PIN curto opcional) e gera um **QR Code 100% local (SVG/canvas, sem acesso à internet)** exibido em modal sob demanda contendo `{ ip, port, session_id, token }`.
  - Desativar o receptor fecha o socket e revoga imediatamente o segredo de sessão.
- **Protocolo UDP Compacto e Autenticado (compatível com frame binário enxuto e subconjunto OSC):**
  - Todo pacote `Celular → PC` trafega em datagrama curto (`<= 128 bytes`), assinado com MAC de sessão (`HMAC-SHA256` truncado para 64 ou 128 bits) e número de sequência monotônico `seq: u64` verificado com janela deslizante anti-replay de 64 pacotes.
  - Tipos de mensagem de controle:
    - `Expression { axis: u8, value_norm: f32 }` (`0.0..=1.0`, taxa de `30–100 Hz`);
    - `Footswitch { slot: u8, pressed: bool }` (`slot ∈ 1..=6`);
    - `TapTempo`;
    - `Heartbeat { client_ts_ms: u32 }`.
  - A thread receptora roda **fora da thread de áudio**, usando buffer fixo na stack (`[u8; 512]`, zero alocação por pacote), coalescendo eventos contínuos de expressão e despachando comandos exclusivamente pelo caminho `api::dispatch` / `LockFreeParamQueue` (`< 0,1%` de CPU típica a `100 Hz`, teto `< 0,5%` de 1 núcleo).
- **Retorno Leve de Estado (`PC → Celular`, `5–10 Hz` ou sob mudança):**
  - Envia ao cliente pareado datagramas compactos contendo: nome/ID do preset ativo, cena ativa (`A`–`H`), bitmask de estado `on/off` dos 6 footswitches mapeados, estado de `Auto-Engage` (Wah/Whammy) e, quando `tuner.enabled == true`, nota e desvio em *cents*.
- **Failsafe de Desconexão:**
  - Ausência de `Heartbeat` ou pacotes autenticados por `> 3,0 s` marca a pedaleira como `desconectada` na UI sem interromper o áudio, sem disparar estalos e sem alterar o preset ou posição atual dos controles.

### 2. Mapeamento Completo no Modo Palco e *Auto-Engage*
- **Expressão Contínua (`axis 0`):** roteável para:
  - `wah_1.position` (Pedal Wah `0.0..=1.0`);
  - `pitch_1.semitones` (Pedal Whammy, interpolado suavemente na faixa ativa do modelo);
  - `palco.exp_1` (Modifiers livres da matriz);
  - `master.level_db` (Pedal de Volume).
- ***Auto-Engage* com Histerese (Wah e Whammy):**
  - Quando habilitado, mantém o efeito desligado enquanto o pé repousa no calcanhar (`value_norm < 0,05`).
  - Ao avançar acima do limiar de ativação (`value_norm >= 0,08`), ativa imediatamente `wah_1.enabled = true` (ou `pitch_1.enabled = true` no modo Whammy).
  - Entre `0,05` e `0,08` (faixa de histerese), preserva o estado atual para evitar liga/desliga repetitivo (*chattering*).
  - Ao retornar abaixo de `0,05` e permanecer estável pelo tempo configurado (`disengage_delay_ms`, padrão `400 ms`, faixa `100..=2000 ms`), desativa suavemente o efeito sem estalo.
- **6 Footswitches Virtuais Mapeáveis (`FS1..FS6`):**
  - Cada slot mapeável para qualquer ação do Modo Palco: seleção direta de cena (`scene.active` `A`–`H`), `preset_prev` / `preset_next`, toggle de blocos (`drive.enabled`, `wah_1.enabled`, `pitch_1.enabled`, `dly_1.enabled`, `rev_1.enabled`, `mod_2.enabled`, `eq_1.enabled`, `comp_2.enabled`, `gate.enabled`), `tap_tempo`, `looper.action` (`record`, `play`, `overdub`, `stop`, `undo`) ou `tuner.enabled`.
  - Debounce configurável no receptor (`>= 120 ms` padrão) contra disparos duplicados.
- **Chaves de parâmetro (todas marcadas como `planejada` até implementação):**
  - `foot.enabled` (`bool`, padrão `false`, planejada)
  - `foot.exp_target` (`enum`: `wah | whammy | exp_1 | volume`, padrão `wah`, planejada)
  - `foot.auto_engage` (`bool`, padrão `true`, planejada)
  - `foot.auto_engage_delay_ms` (`number`: `100..=2000`, padrão `400`, planejada)
  - `foot.fs1_action` .. `foot.fs6_action` (`enum`/ação de palco, planejada)

### 3. Aplicativo Android Nativo Dedicado (`apps/sonix-foot-android/`)
- Aplicativo nativo leve em **Kotlin + Jetpack Compose + CameraX**, compilado como APK instalável diretamente no dispositivo Android (100% offline/LAN, sem Play Services).
- Mantém tela ativa (`FLAG_KEEP_SCREEN_ON`) durante a sessão pareada e oferece **3 modos integrados**:
  1. **Modo Câmera para os Pés (100% pés livres, `160×120` ou `320×240` YUV @ `30–60 fps`):**
     - Calibração do chão vazio em 1 toque (`1 s`), construindo mapa de referência de luminância/croma.
     - Até **6 zonas virtuais posicionáveis** no chão para Footswitches discretos, com limiar de ocupação de área, compensação de variação global de exposição/brilho, histerese e debounce contra sombras e passagens rápidas.
     - **1 zona opcional de Expressão Contínua** (calcanhar apoiado, elevação da ponta do pé `0–100%`), operando por padrão **sem marcador** (topo da silhueta do pé) e com opção de **Modo Marcador Colorido HSV** (fita/elástico fluorescente na ponta do calçado/meia) para máxima precisão em baixa luminosidade.
     - Suavização adaptativa com **Filtro One-Euro** (resposta instantânea em varreduras rápidas de Wah/Whammy e estabilidade sem tremor quando parado).
  2. **Modo Expressão IMU / Giroscópio (`TYPE_GAME_ROTATION_VECTOR` / `GYROSCOPE` @ `100 Hz`):**
     - Celular apoiado sobre o pé ou em prancha basculante no chão.
     - Calibração de 2 pontos (`Calcanhar = 0%`, `Ponta = 100%`) convertendo inclinação (`pitch`) em `0.0..=1.0` com zona morta configurável nas extremidades (`~3%`), filtro One-Euro e latência alvo ponta-a-ponta `<= 15 ms` em Wi-Fi 5 GHz ou USB tethering.
  3. **Modo Pedaleira Touch na Tela:**
     - 6 pads grandes com LEDs de estado ao vivo vindos do SONIX, visor de Preset/Cena/Afinador, slider de expressão e botão `Tap Tempo`, operável isoladamente ou em conjunto com Câmera/IMU.

## Critérios de aceite
> Cada item é uma afirmação verificável e reprovável sob a disciplina anti-superficialidade do SDD.

- **CA-1 (Opt-In Estrito e Zero Socket TCP — `Spec 033 CA-1`):** Com `foot.enabled = false` (padrão de boot), o processo `sonix-app` mantém `0` sockets TCP e `0` sockets UDP de controle remoto abertos; ao habilitar `foot.enabled = true`, abre exclusivamente `1` socket UDP (ainda `0` sockets TCP em escuta); ao desabilitar, fecha o socket UDP e recusa pacotes com o token antigo.
- **CA-2 (Autenticação MAC, Anti-Replay e Defesa contra Pacotes Malformados):** Enviar ao socket UDP ativo pacotes com (a) MAC inválido, (b) `seq` repetido/atrasado fora da janela de 64, (c) payload truncado/corrompido ou (d) `value_norm` em `NaN`/`±Inf`/`-0.5`/`1.5` resulta em descarte integral (`0` alterações em `AppState`, `0` comandos na `LockFreeParamQueue`, incremento do contador de pacotes rejeitados e zero panic).
- **CA-3 (Controle Contínuo de `Wah`, `Whammy` e `palco.exp_1` na Cadeia Real):** Enviar sequência autenticada UDP `Expression` (`0.0 → 0.5 → 1.0` a `100 Hz`) com `foot.exp_target = wah` altera `wah_1.position` e desloca o pico de frequência do filtro Wah na saída da `RealtimeDspChain` de forma monotônica (`>= 600 Hz` entre `0.0` e `1.0`); com `foot.exp_target = whammy`, desloca o pitch fundamental medido em `pitch_1` na direção configurada sem cliques ou `NaN`.
- **CA-4 (*Auto-Engage* Determinístico com Histerese):** Com `foot.auto_engage = true` e `foot.auto_engage_delay_ms = 400`:
  1. Subir `Expression` de `0.00` para `0.06` (`< 0.08`) mantém `wah_1.enabled == false`.
  2. Subir para `0.12` (`>= 0.08`) ativa `wah_1.enabled == true` em `<= 1` bloco de áudio.
  3. Oscilar repetidamente entre `0.055` e `0.075` (faixa de histerese `[0.05, 0.08]`) mantém `wah_1.enabled == true` sem nenhum desligamento espúrio.
  4. Retornar para `0.02` (`< 0.05`) mantém `wah_1.enabled == true` em `t = 300 ms` e desativa (`wah_1.enabled == false`) em `t >= 450 ms`.
- **CA-5 (6 Footswitches Mapeáveis no Modo Palco):** Eventos autenticados `Footswitch(1..6)` disparam com paridade a `api::dispatch` cada categoria de ação configurada (`scene.active` A–H, `preset_next`/`preset_prev`, toggle de `drive.enabled`/`dly_1.enabled`/`rev_1.enabled`, `tap_tempo`, `looper.action` e `tuner.enabled`), respeitando debounce `>= 120 ms` (dois acionamentos em `40 ms` executam apenas `1` toggle).
- **CA-6 (Orçamento de CPU no PC e Segurança RT Zero-Alloc Medida):** Sob rajada contínua de `100 pacotes/s` `Expression` + `10 pacotes/s` `Footswitch` simultâneos à execução da `RealtimeDspChain` (`48 kHz / 128` e `96 kHz / 32`), o `CountingAllocator` na thread RT registra `alloc_count == 0` e `dealloc_count == 0`, `0` novos underruns e custo médio de CPU da thread receptora UDP `<= 0,5%` de 1 núcleo em build `--release`.
- **CA-7 (Retorno de Estado PC → Celular e Failsafe de Desconexão):** Alterar preset, cena ou stomp no SONIX entrega um pacote de estado UDP atualizado ao cliente pareado em `<= 200 ms`; interromper pacotes do cliente por `> 3,0 s` muda o status da conexão para `disconnected` mantendo o áudio contínuo e o preset/controles intactos.
- **CA-8 (Pareamento 100% Offline via QR Code — `Spec 033 CA-10`):** O payload e a renderização SVG do QR Code de pareamento ocorrem 100% offline sem requisições externas e sem incluir segredos persistidos em disco.
- **CA-9 (App Android — Mapeamento IMU 2-Pontos e Filtro One-Euro):** No módulo Kotlin `apps/sonix-foot-android`, calibrar `heel = 5°` e `toe = 35°` mapeia `5° → 0.00`, `20° → 0.50 ± 0.02`, `35° → 1.00`, satura limpamente fora da faixa (`<= 0.0` e `>= 1.0`), aplica zona morta configurável e atenua ruído gaussiano estático (`σ = 0,5°`) para variação de saída `< 0,005`.
- **CA-10 (App Android — Zonas Virtuais de Câmera e Marcador Colorido HSV):** Sobre sequências de frames YUV sintéticos (testes unitários JVM): (a) variação uniforme de iluminação de `±10%` sobre o fundo calibrado gera `0` falsos disparos; (b) objeto entrando na zona `Z1..Z6` acima do limiar dispara exclusivamente o slot correspondente após `debounce_ms`; (c) modo marcador HSV estima a altura normalizada `0.0..1.0` com erro absoluto `<= 0,03`.
- **CA-11 (Inventário de Controles e Regressão Visual `Spec 048`):** Todos os controles visíveis do modal/botão de Pedaleira Remota estão registrados em `controls.registry.ts` com chaves aceitas pelo backend; o modal permanece fechado no estado de fixture da `Spec 048`, preservando `SSIM >= 0,85` por região.
- **CA-12 (Prova de Mutação e Não-Regressão):** Todos os testes de contrato da `Spec 094` falham em *red real* antes da implementação e toda a suíte existente (`Specs 001–093`) permanece 100% aprovada com `foot.enabled = false`.

## Fora de escopo
- Processamento de visão computacional, webcam ou modelos de IA/pose diretamente no computador PC (descartado pela restrição de CPU; caso desejado no futuro, uma webcam ultraleve no PC poderá ser especificada separadamente reaproveitando o mesmo barramento interno de `FootEvent`).
- Cliente nativo iOS / iPhone (o protocolo UDP/OSC desta spec é aberto para clientes futuros, mas o aplicativo entregue aqui é exclusivamente Android).
- Publicação na Google Play Store, nuvem ou operação pela internet (100% rede local LAN / USB tethering, offline-first).
- Transporte de áudio da guitarra pelo celular ou Wi-Fi (somente mensagens leves de controle e status).
- Implementação imediata: por decisão explícita do usuário (`2026-11-25`), a execução desta spec fica agendada para o final do roadmap, após todas as fases e specs pendentes.

## Invariantes tocadas
- **Zero-Alloc / Zero-Lock RT (`Spec 037`, `Spec 044`, `Spec 093`, Regras Cortex `fd17`, `d272`, `4d05`):** o receptor UDP executa em thread não-RT separada, usando buffer estático na stack e despachando exclusivamente pela `LockFreeParamQueue` SPSC; a thread de áudio nunca toca em sockets, mutexes ou heap.
- **Zero Socket TCP no Boot e 100% Offline (`Spec 033 CA-1, CA-10`, `Spec 015`):** usa apenas UDP sob opt-in explícito (`foot.enabled = false` no boot) e QR Code gerado localmente sem URLs externas.
- **Zero Controle Decorativo (`Spec 033 CA-4, CA-5`, Regra Cortex `0a2f`):** todo controle visível na tela Palco/modal de pareamento entra em `controls.registry.ts` com validação atômica e teste de efeito.
- **Fidelidade Visual (`Spec 048`):** painel de pareamento/configuração é aberto sob demanda sem alterar o layout base de fixture (`SSIM >= 0,85`).

## Testes que cobrem esta spec
> Preenchido no fechamento (caminhos planejados):
- `Planejado: crates/sonix-desktop/tests/contract_094_foot_udp_receiver.rs` → CA-1, CA-2, CA-3, CA-4, CA-5, CA-6, CA-7, CA-12
- `Planejado: apps/sonix-app/src-tauri/tests/contract_094_foot_optin_and_qr.rs` → CA-1, CA-8
- `Planejado: tests/contract/094-foot-controller-ui.test.js` → CA-8, CA-11
- `Planejado: apps/sonix-foot-android/app/src/test/java/com/sonixpro/foot/FootSensorContractTest.kt` → CA-9, CA-10

## Notas
- **Decisões fechadas na entrevista `/grill-me` (`2026-11-25`):**
  1. Processamento 100% no celular (`< 0,1%` CPU no PC) sobre receptor genérico no SONIX.
  2. Cliente Android nativo dedicado (`apps/sonix-foot-android/`, Kotlin + Jetpack Compose + CameraX, APK offline).
  3. Modo Duplo (Câmera para 6 zonas virtuais + zona de expressão sem marcador ou marcador colorido HSV; Modo IMU/Giroscópio `~10 ms`; Pedaleira Touch na tela).
  4. Transporte UDP opt-in (zero TCP no boot), pareamento por QR Code + token autenticado, Wi-Fi local ou USB tethering, com retorno leve de estado (preset/cena/LEDs/afinador).
  5. Escopo completo do Modo Palco com *Auto-Engage* no Wah/Whammy.
  6. **Agendamento:** apenas especificação registrada agora; implementação adiada para o final do projeto, após a conclusão de todas as fases e specs pendentes.
