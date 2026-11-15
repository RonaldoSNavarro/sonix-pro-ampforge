---
spec: 049
titulo: Correções do review da entrega 034–048 e fechamento das pendências abertas (plugins VST3/CLAP, DAG 4×12, Setlists/Modifiers/footswitches)
status: implementada (CA-36 e validadores externos VST3/CLAP pendentes)
atualizado: 2026-11-15
---
# 049 — Correções pós-review das Specs 034–048 e pendências abertas

## Objetivo
Garantir três coisas:
- trocar de cena ou de dispositivo não corrompe o áudio nem a memória;
- a API e a UI só mostram estado que existe;
- as promessas ainda abertas das Specs 002, 009 e 012 passam a funcionar no caminho de produção: plugins VST3/CLAP, grafo 4×12 configurável, Setlists, Modifiers e modos de footswitch;
- os requisitos do primeiro release ainda abertos (ASIO, entrada instrumento/linha e reamp) passam a funcionar.

## Contexto
Origem: code review do working tree entregue pelo Antigravity com as Specs 034–048 (handoff Cortex `20261114-235809-sonix-pro-9691`, 2026-11-15), feito em 2026-11-15 com verificação por leitura de código. Os testes da entrega estão verdes, mas **nenhum deles exercita** os achados A1, A2, A3 e A10: são defeitos de concorrência e de estado entre a cadeia ativa e a cadeia sombra.

Por decisão do usuário (2026-11-15), os achados e as pendências abertas ficam **numa única spec**, organizada em partes (A a E).

### Parte A — achados do review
1. **A troca de cena descarta o estado vivo.** `RealtimeDspChain::begin_preset_crossfade` (`crates/sonix-desktop/src/audio.rs:2661`) chama `swap_dsp_state` (`audio.rs:2610–2659`). Essa função troca **todo** o estado com a cadeia sombra, incluindo `looper`, `looper_position`, `dly_1` e `rev_1`. A sombra só processa áudio durante o fade de 20 ms (`audio.rs:3606–3613`); fora dele, ela recebe só parâmetros (`audio.rs:2424`, `3657–3664`). Consequências:
   - O loop gravado some. A cadeia ativa passa a usar o looper da sombra, que recebeu os mesmos comandos Record/Play mas tem `loop_length = 0`.
   - Ecos antigos voltam. Na troca seguinte, o delay e o reverb que voltam para a cadeia ativa carregam no buffer o áudio de antes da troca anterior.
   - O spillover não funciona. A cauda do delay e do reverb da cena anterior é cortada em 20 ms.
   - O teste `contract_045` CA-2 só mede buraco de RMS numa senoide, por isso não pega nenhum dos três.
2. **A reconfiguração do dispositivo deixa dois streams no mesmo ring SPSC.** `configure_audio_device_with_opener` (`apps/sonix-app/src-tauri/src/commands.rs:290`) abre e inicia os streams novos e só depois troca o guard (`commands.rs:311`). Entre esses dois momentos, há dois callbacks de entrada dando `push` e dois de saída dando `pop` no mesmo `LockFreeSpscRingBuffer`. Além disso:
   - A thread de fallback criada em `apps/sonix-app/src-tauri/src/lib.rs:105` quando o app sobe sem stream nunca é parada. Ela continua dando `pop` e chamando `dsp.process` em paralelo ao stream novo.
   - No modo exclusivo, o dispositivo recusa a segunda abertura. Por isso ele nunca pode ser reconfigurado.
3. **`LockFreeResourceQueue::push` não tem lock de produtor.** `audio.rs:1453` escreve direto num `SpscOwnedQueue`. A `LockFreeParamQueue` ganhou `producer_lock` neste mesmo diff (Spec 034 CA-7), e a fila de recursos ficou sem. Dois dispatches concorrentes (`/api/cab/ir` e `/api/input/di`) leem o mesmo `head`. Um `ImpulseResponse` ou `Vec` é sobrescrito sem drop e o outro é publicado duas vezes, o que é **double free (UB)** quando os dois são aposentados.
4. **A seleção de cena responde 200 quando falha.** `handle_post_scenes_select` (`crates/sonix-desktop/src/api.rs:3529`) só trata `PresetApplyError::QueueFull`. Um erro de validação aborta antes de aplicar qualquer chave, mas a resposta continua sendo `{"status":"ok"}`. Exemplos: `instanceId: "merge"` gera `merge.enabled`, que não existe na validação, e um canal com `amp_1.model: "foo"`.
5. **Chaves aceitas que não atualizam o `AppState`.** `handle_post_params` valida e enfileira `pitch_1.*` (`api.rs:1597–1605`) e os aliases `mod_2.mix_pct`, `rev_1.mix_pct`, `cab_1.model` e `cab_2.model` (`api.rs:1486, 1548, 1566, 1575`). Só que o `match` que muta o `AppState` (`api.rs:~2850–2990`) não tem braço para elas. O áudio muda, mas `GET /api/state` mostra o valor antigo, e salvar o preset ou recarregar a UI perde o ajuste. O caminho de preset (`api.rs:3803, 3836`) trata os aliases: os dois caminhos divergem.
6. **A barra de status mostra valores fixos.** `apps/sonix-app/ui/src/shell.ts` traz quatro textos sem dado real por trás:
   - `'Focusrite USB Audio'` quando `output_device` está vazio (`:402`);
   - `'-14.2'` dBFS com a entrada em silêncio (`:404`);
   - `"Motor ativo"` e `"entrada calibrada"` (`:411, 420`);
   - `"ponte UI→DSP ok · 2 ms"` (`:341`).

   Isso viola a Spec 036 e a regra "todo controle visível funciona no backend como o equipamento real".
7. **O MIDI CC14 sai da faixa aceita.** `map_cc_to_param` (`crates/sonix-desktop/src/midi.rs:135`) mapeia `gate.threshold_db` de -96 a 0 dB, mas a validação aceita só de -90 a 0 (`api.rs:1856`). Os valores 0 a 7 do CC dão erro 400 e o gate fica preso.
8. **A latência do oversampler é informada pela metade.** `HalfBandOversampler::latency_samples` (`crates/ampforge-dsp/src/oversampling.rs:222`) devolve `HB_CENTER * 7 / 8 = 55` amostras em 8x. Cada estágio FIR de 127 taps (`HB_CENTER = 63`) atrasa na interpolação e na decimação, e o atraso real ida e volta fica em ~110 amostras base (≈ 2,3 ms a 48 kHz). `estimated_roundtrip_ms` e a medição de latência herdam o erro.
9. **O FIR do oversampler é caro demais.** `push_and_fir_up` e o equivalente de decimação (`oversampling.rs:91`) deslocam 127 `f64` com um laço a cada push, e os três estágios usam o kernel completo. Em 8x, isso dá ~4,5 mil operações por amostra por instância de amp (`amp_1`, `amp_2` e a sombra no crossfade). Com buffers de 64 ou 128 frames, há risco de xrun.
10. **A telemetria disputa o lock da cadeia com o callback.** O handler de `/api/telemetry` (`api.rs:817`) faz `try_lock` na cadeia a cada ~33 ms só para ler `b_plus_v`. Quando o callback perde a disputa, sai sinal limpo, um blip audível. O `TelemetryBuffer::update_b_plus_v` (`audio.rs:3923`) existe, mas nada o chama.

### Parte A′ — higiene herdada
- `docs/ui-desktop/inventario-controles.md` tem 13 rótulos `pendente — Spec 0NN` que apontam para specs já `implementada` (por exemplo, `pendente — Spec 042` para `pitch_1.*`, `pendente — Spec 038` para o gabinete e `pendente — Spec 045` para Setlist/Modifiers/grafo).
- As Notas das Specs 002 e 009 dizem que a produção é "retomada pela Spec 045", mas a 045 está `implementada` e põe o DAG e os Modifiers no Fora de escopo. A Nota da 012 aponta para a 036, que não tratou de plugin. Nenhuma spec aberta cobre o que falta: esta passa a cobrir.

### Parte B — pendência da Spec 012 (plugins VST3/CLAP)
`crates/sonix-plugin/src/lib.rs` (119 linhas) é passthrough. O `Cargo.toml` não tem nenhuma dependência de formato de plugin, e o build não gera `.vst3` nem `.clap`. A Spec 012 CA-3, CA-4, CA-5 e CA-8 e o trecho de plugin do CA-6 estão revertidos desde a auditoria da Spec 044.

### Parte C — pendência da Spec 002 (DAG 4×12)
`DagGraph` e `GridMatrix4x12` existem e são testados como biblioteca (`contract_002_matriz_dag.rs`). `RealtimeDspChain::process` executa uma cadeia fixa com um único split/merge (`split`, `amp_2`, `cab_2`, `merge` em `audio.rs`). A grade da UI não muda a ordem nem a topologia do processamento.

### Parte D — pendência da Spec 009 (Setlists, Modifiers, footswitches)
Cenas A–H e canais A–D por bloco já funcionam em produção (Spec 045). Continuam só na biblioteca (`sonix-core`):
- a hierarquia Setlist → Preset (009 CA-1);
- o custo zero dos canais inativos (CA-3);
- a Modifier Matrix (CA-5, CA-6);
- os 4 modos de footswitch (CA-7);
- as validações de erro correspondentes (CA-8).

O único "modifier" de produção é o mapeamento MIDI fixo de `midi.rs:130–140`.

## Comportamento esperado
### A — troca de cena, dispositivo, filas, API e telemetria
- **Troca de cena ou de preset:**
  - O looper existe uma vez só e fica fora do crossfade. Trocar de cena não para, não esvazia e não troca o loop.
  - Delay e reverb seguem o `spillover`. Ligado: a cauda da cena anterior continua decaindo. Desligado: ela some com o fade.
  - Nenhum bloco com memória volta a tocar áudio que entrou antes da troca anterior.
- **Reconfiguração de dispositivo:**
  - Em nenhum momento há mais de um callback de entrada e um de saída no ring.
  - A thread de fallback termina quando um stream real abre.
  - Reconfigurar o mesmo dispositivo funciona também no modo exclusivo: o stream antigo para, o novo abre e, se o novo falhar, o antigo é reaberto com a configuração anterior.
- **Filas e API:**
  - Toda fila com mais de um produtor serializa os produtores. Cada recurso chega à thread de áudio exatamente uma vez e é desalocado exatamente uma vez, fora dela.
  - A API só responde 2xx quando o estado mudou. Um erro de validação responde 400 e não muda nada.
  - Toda chave aceita por `dispatch` aparece em `GET /api/state` na chave canônica.
- **UI e telemetria:**
  - A barra de status só mostra dado vindo do backend. Sem dado, mostra um marcador neutro ("—", "sem dispositivo", "não calibrada").
  - O MIDI só produz valores dentro da faixa validada.
  - O oversampler informa o atraso medido e cabe no orçamento de tempo real.
  - Nenhum leitor de telemetria toma o lock da cadeia.

### B — plugins
- O build gera um `.vst3` e um `.clap` de 64 bits para Windows. Eles rodam a **mesma** `RealtimeDspChain` do standalone, sem servidor HTTP.
- Os parâmetros da tabela única viram parâmetros automatizáveis do host.
- O estado salvo no projeto da DAW é o documento `.sonixpreset`.
- Com `hostTempoSync`, o tempo segue o transporte da DAW.
- O editor do plugin abre a mesma UI do app via WebView, por IPC.

### C — grafo 4×12
- A grade da UI define a topologia processada: ordem dos blocos, ramificações, splits e mixers em qualquer uma das 4 linhas e 12 colunas.
- Uma topologia nova é compilada fora da thread de áudio, trocada por ponteiro e aplicada com o mesmo crossfade sem buraco da Spec 045.
- Uma conexão que forme ciclo é recusada e a topologia em execução não muda.
- As entradas e saídas de cada linha só oferecem os canais que o dispositivo aberto tem; as demais aparecem desabilitadas com o motivo.

### D — Setlists, Modifiers e footswitches
- **Setlists:** o usuário organiza presets em Setlists persistidos offline e navega por eles pela UI, pela API e pelos footswitches.
- **Canais:** os canais inativos de um bloco não são processados.
- **Modifier Matrix:**
  - O clique direito em qualquer knob abre a Modifier Matrix daquele parâmetro.
  - Fontes disponíveis: EXP 1/2, MIDI CC, LFO 1/2, Envelope Follower e Scene Controller.
  - Cada vínculo tem `min`, `max`, `curveSlope` e `dampingMs`, é avaliado na thread de áudio sem alocação e é salvo no preset.
- **Footswitches:** os 8 footswitches operam nos modos PRESET, SCENE, STOMP e HYBRID, com efeito real no backend.

## Critérios de aceite
### Parte A — achados do review
- **CA-1 (looper na troca de cena):** Com um loop de 2 s gravado e tocando, selecionar outra cena mantém o loop tocando. O RMS da saída do looper nos 2 s depois da troca fica em pelo menos 90% do RMS dos 2 s anteriores, e `looper_position` continua avançando sem voltar a 0.
- **CA-2 (sem eco antigo):** Com o `dly_1` ligado e o `spillover` desligado: rajada de 1 kHz, troca para a cena B, rajada de 2 kHz, troca de volta para a cena A e entrada em silêncio. Depois da segunda troca, a energia em 1 kHz fica abaixo de -80 dBFS por 3 s.
- **CA-3 (spillover):** Com o `dly_1` em `intensity` de 50% e o `rev_1` com decay que dê RT60 ≥ 2 s (medido pela régua da Spec 040), e com o `spillover` ligado nos dois, a cauda de uma rajada de -6 dBFS da cena anterior continua acima de -60 dBFS 500 ms depois da troca e decai sem degrau. Com o `spillover` desligado, ela cai abaixo de -80 dBFS em até 30 ms.
- **CA-4 (um par de streams por vez):** No host simulado com contadores de callbacks concorrentes, 100 `set_audio_device` alternando entre 128 e 256 frames nunca registram mais de um produtor ou de um consumidor ativo no ring. Uma senoide de 1 kHz passa com `ring_index_errors == 0`.
- **CA-5 (fallback parado):** Com o app iniciado sem stream, o primeiro `set_audio_device` bem-sucedido encerra a thread de fallback em até 50 ms (`join` concluído). Depois disso, `dsp.process` só é chamado pelo callback do stream.
- **CA-6 (exclusivo e rollback):** No host simulado que recusa duas aberturas simultâneas do mesmo dispositivo, reconfigurar de 128 para 256 frames tem sucesso e o status relata 256. Se a nova abertura falhar, a resposta é erro, a configuração de 128 é reaberta, o status relata 128 e o áudio volta a sair.
- **CA-7 (fila de recursos com vários produtores):** Duas threads publicam 1000 `DiSamples` cada, com identificadores únicos, na `LockFreeResourceQueue`. O consumidor recebe os 2000 identificadores exatamente uma vez cada, e um contador de `Drop` registra exatamente 2000 desalocações, nenhuma na thread consumidora.
- **CA-8 (cena inválida responde 400):** `POST /api/scenes/select` num preset com um bloco de chave desconhecida, ou com um canal `amp_1.model: "foo"`, responde 400 com a chave ofensiva na mensagem. `GET /api/state` antes e depois é idêntico, inclusive a cena ativa.
- **CA-9 (estado reflete toda chave aceita):** Para **toda** chave da tabela única aceita por `dispatch`, inclusive `pitch_1.*` e os aliases `mod_2.mix_pct`, `rev_1.mix_pct`, `cab_1.model` e `cab_2.model`, um POST com valor válido diferente do padrão faz `GET /api/state` devolver esse valor na chave canônica, e `save → load` o preserva. O teste percorre a tabela, sem lista escrita à mão.
- **CA-10 (status sem valor inventado):**
  - Com `output_device` vazio, a barra mostra "sem dispositivo".
  - Com a entrada em silêncio, o pico mostra "—" ou o valor medido, nunca "-14.2".
  - "Motor ativo" só aparece com o stream rodando, e "entrada calibrada" só depois de uma calibração concluída na sessão.
  - A latência da ponte é medida ou não aparece.
  - Um teste JS reprova os literais `Focusrite`, `-14.2` e `· 2 ms` em `apps/sonix-app/ui/src`.
- **CA-11 (MIDI dentro da faixa):** Para todo controlador mapeado e todo valor de 0 a 127, o par de `map_cc_to_param` passa na validação de `dispatch`. O CC14 com valor 0 aplica `gate.threshold_db = -90`, e com 127 aplica 0.
- **CA-12 (latência medida):** Para 2x, 4x e 8x, um impulso passado por up + down tem o pico na amostra `latency_samples()` com tolerância de ±1. `estimated_roundtrip_ms` usa esse valor.
- **CA-13 (custo do oversampler):** Em 8x, 1 s de áudio a 48 kHz numa instância de amp é processado em no máximo 25% do tempo da implementação atual, na mesma máquina, em release. Os dois números são registrados nas Notas. `stopband_attenuation_db` não cai em relação ao valor atual. O aliasing medido na saída do amp também não piora mais de 1 dB: com uma senoide de 5 kHz a -6 dBFS em ganho alto, medimos a energia dos componentes inarmônicos (fora dos múltiplos de 5 kHz) abaixo de 20 kHz.
- **CA-14 (telemetria sem lock):** O handler de `/api/telemetry` não toma o lock da cadeia, nem com `try_lock`, o que é verificado por teste de fonte e por execução. `b_plus_v` vem do `TelemetryBuffer` publicado em `process()`. Com um leitor a 1 kHz por 5 s durante a reprodução, `dsp_contention_silence_blocks` fica em 0.

### Parte A′ — higiene
- **CA-15 (inventário e Notas honestos):** Nenhuma linha de `docs/ui-desktop/inventario-controles.md` marca como `pendente` um item de spec `implementada`. Toda pendência restante aponta para esta spec e para o CA que a resolve. As Notas das Specs 002, 009 e 012 apontam para a 049. O teste de integridade da Spec 044 passa a reprovar uma pendência que aponte para spec implementada.

### Parte B — plugins VST3/CLAP (retoma 012 CA-3, CA-4, CA-5, CA-6-plugin e CA-8)
- **CA-16 (artefatos válidos):** O build de release gera `sonix-pro.vst3` e `sonix-pro.clap` de 64 bits para Windows. O `.clap` passa no `clap-validator` sem falhas, e o `.vst3` passa no validador do VST3 SDK sem falhas. Os dois declaram estéreo de entrada e saída e entrada de MIDI.
- **CA-17 (mesmo núcleo):** Para o mesmo `.sonixpreset`, o mesmo arquivo de entrada, a **mesma taxa de amostragem e o mesmo tamanho de bloco**, a renderização offline pelo `.clap`, pelo `.vst3` e pela `RealtimeDspChain` do standalone difere em no máximo -100 dBFS de pico, depois de compensada a latência declarada. Com tamanhos de bloco diferentes (64 contra 512), a diferença fica abaixo de -60 dBFS: os smoothers são por amostra, não por bloco.
- **CA-18 (parâmetros automatizáveis):** Todo parâmetro contínuo ou discreto da tabela única aparece como parâmetro do plugin, com o mesmo id estável, a mesma faixa e a mesma unidade. Automatizar `amp_1.gain` de 0 a 10 no host muda a saída, com o THD crescendo de forma monotônica, sem degrau audível e com aplicação sample-accurate no CLAP.
- **CA-19 (estado na sessão):** `state save → state restore` no host devolve um documento `.sonixpreset` idêntico ao salvo: 8 cenas, 4 canais por bloco, parâmetros White-Box, microfones, Modifiers e topologia. Um preset exportado pelo standalone e importado no plugin, e vice-versa, produz a mesma saída de áudio (régua do CA-17).
- **CA-20 (tempo do host):** Com `hostTempoSync = true`, mudar o BPM do transporte de 120 para 140 atualiza o tempo alvo de um `dly_1` sincronizado em 1/4 de 500 ms para 428,6 ms já no próximo bloco processado. Depois de 1 s de acomodação, o intervalo medido entre ecos é 428,6 ms (±1 ms). No modelo de fita (RE-201), a transição é um glide de velocidade do motor, com variação de pitch nos ecos já gravados, como no equipamento real, e não um degrau. O RE-201 original não sincroniza com tempo; a sincronia é uma abstração do software, documentada na UI.
- **CA-21 (tempo real no plugin):** O alocador de contagem da Spec 037, aplicado ao `process` do plugin, registra zero alocações, desalocações e locks durante 10 s de reprodução com automação ativa.
- **CA-22 (editor sem HTTP):** O editor do plugin abre a UI do app numa WebView embutida e conversa com o núcleo por IPC. Nenhuma porta TCP fica em LISTEN durante a sessão, e o binário do plugin não contém o servidor `dev-http`.

### Parte C — grafo 4×12 em produção (retoma 002 CA-1..CA-10)
- **CA-23 (a grade é a topologia):** Mover o `drive` de antes para depois do `amp_1` na grade muda a saída de forma mensurável: com a mesma senoide e as saídas **com RMS igualado**, o espectro de harmônicos difere em mais de 6 dB em pelo menos um harmônico entre o 2º e o 7º. A diferença de nível sozinha não conta. `/api/graph` devolve as 48 coordenadas com o bloco de cada uma.
- **CA-24 (troca de topologia sem buraco e sem alocação):** Trocar de topologia durante uma senoide não produz janela de 1 ms com RMS abaixo de 50% do menor RMS entre os dois estados (régua da 045 CA-2). O alocador de contagem registra zero alocações no callback durante a troca, porque a topologia nova é compilada fora da thread de áudio.
- **CA-25 (ciclo recusado):** Uma conexão que forme ciclo, ou que aponte para a mesma coluna ou para uma coluna anterior, responde 400 com `DagCycleError`. A UI marca a rota com `.invalid-route`, e a topologia em execução (identidade do ponteiro na thread de áudio) não muda.
- **CA-26 (split e mixer pela API):** Por `/api/params`:
  - um split em `CROSSOVER` a 500 Hz soma plano em ±0,2 dB de 40 Hz a 4 kHz;
  - um mixer com dois ramos iguais a 0 dB e um deles com fase em 180° soma abaixo de -90 dBFS;
  - um `DYNAMIC_SPLIT` manda as notas abaixo do threshold para o ramo A e os transientes acima para o ramo B.
- **CA-27 (I/O por linha honesto):** O seletor de entrada e saída de cada linha só habilita destinos que o dispositivo aberto tem. Com uma placa estéreo, `FX_SEND` e `USB_REC_OUT` aparecem desabilitados com a dica do motivo. Num dispositivo com dois pares de saída, `AMP_OUT_P10_NO_CAB` entrega, em nível de linha, o sinal daquela linha tirado antes do gabinete: a subtração contra o tap pré-cab da mesma linha fica abaixo de -90 dBFS. A UI avisa que essa saída alimenta um power amp ou o retorno de um amp e não deve ser ligada direto a um alto-falante.
- **CA-28 (bypass isolado):** Ligar ou desligar um bloco, ou trocar o canal dele, numa linha paralela não altera bypass, canal nem parâmetros dos blocos das outras linhas (002 CA-10).

### Parte D — Setlists, Modifiers e footswitches em produção (retoma 009 CA-1, CA-3, CA-5..CA-9)
- **CA-29 (Setlists):**
  - É possível criar um Setlist com 3 presets numa ordem.
  - Ele é salvo offline e reabre igual depois de reiniciar o app.
  - `POST /api/setlist/next` e `/prev`, e os footswitches no modo PRESET, carregam o preset seguinte e o anterior na ordem do Setlist, sem buraco (régua da 045 CA-2).
- **CA-30 (canais inativos sem custo):** Com um bloco de amp com 4 canais de modelos diferentes configurados, o tempo médio de processamento por bloco de áudio fica a até 5% do tempo do mesmo bloco com 1 canal. Os processadores dos 3 canais inativos registram 0 chamadas de `process`.
- **CA-30b (canal reativado sem estado velho):** Um canal que ficou inativo e volta a ficar ativo começa com o estado dos filtros, delays e moduladores **zerado**, e não com o estado de quando foi desativado. A troca usa o crossfade da Spec 045. Régua: uma rajada de 1 kHz processada no canal B antes de trocar para o A não aparece, acima de -80 dBFS, quando o B volta. É a mesma classe de defeito do achado A1.
- **CA-31 (Modifier Matrix na UI):** O clique direito em qualquer knob do registro abre a Modifier Matrix daquele parâmetro. É possível criar, editar e remover um vínculo com `EXP_PEDAL_1/2`, `MIDI_CC_n`, `INTERNAL_LFO_1/2`, `ENVELOPE_FOLLOWER` ou `SCENE_CONTROLLER`. O knob com vínculo ganha `.has-modifier` e o segundo arco `min ↔ max`, e o vínculo é salvo no preset.
- **CA-32 (Modifier no motor):** Para um vínculo com entrada `u`:
  - `u = 0` aplica `minVal`, `u = 1` aplica `maxVal`, e `curveSlope = 0.5` interpola linearmente com erro de no máximo 1e-4;
  - com `dampingMs = 100`, um degrau de `u` chega a 63% do alvo em 100 ms (±10%);
  - um LFO de 1 Hz ligado ao `mod_2.depth` modula a saída a 1 Hz;
  - o alocador de contagem registra zero alocações no callback com 8 vínculos ativos;
  - Modifiers só podem apontar para parâmetros **contínuos**. Um vínculo para um parâmetro discreto (`amp_1.model`, `*.mode`, chaves liga/desliga) é recusado com 400, porque trocar modelo pela taxa de um LFO não tem equivalente físico e gera estalo.
- **CA-33 (modos de footswitch):** Alternar `footswitch.mode` entre `PRESET`, `SCENE`, `STOMP` e `HYBRID_STOMP_SCENE` muda os rótulos OLED, as cores e a ação dos FS1–FS8 com efeito no backend:
  - no SCENE, FS3 seleciona a cena C;
  - no STOMP, FS2 liga e desliga o bloco atribuído a ele;
  - no HYBRID, FS1–4 são stomps e FS5–8 selecionam as cenas A–D.

  O mapeamento MIDI fixo de `midi.rs` passa a ser um conjunto de Modifiers padrão editáveis.
- **CA-34 (erros):** Cena fora de 0..7, canal fora de `{A,B,C,D}`, Modifier com `targetParam` inexistente ou com `min/max` fora da faixa do parâmetro, e Setlist que referencia um preset inexistente respondem 400 com mensagem explícita. O estado em execução não muda.

### Parte F — requisitos do primeiro release ainda abertos: ASIO, entrada instrumento/linha e reamp
> Decisões de produto já confirmadas (`.agents/skills/sonix-pro-ampforge-expert`, `references/architecture-rust-windows.md`): WASAPI **e** ASIO no primeiro release, modos Instrument/Hi-Z e Line, e fluxo de DI e reamp. Hoje o ASIO aparece no enum `AudioMode` (`crates/sonix-desktop/src/audio.rs:40,53`), mas o `cpal` não é compilado com a feature `asio`, e a Spec 036 CA-7 o deixou `disabled`. Não há modo de entrada nem fluxo de reamp. Estes requisitos vêm **antes** da Parte B (o VST3 é evolução futura).

- **CA-36 (ASIO real):** O build de release inclui o backend ASIO. Com um driver ASIO instalado, a tela Áudio lista os dispositivos ASIO, e `set_audio_device({mode:"asio"})` abre o stream. O status relata o período e a taxa lidos do driver. A régua de tempo real (037 CA-7) vale também no callback ASIO. O hardware de referência é registrado nas Notas; sem hardware, o CA fica `[verificação pendente]`, nunca dado como ok.
- **CA-37 (falha de ASIO não derruba o app):** Sem driver ASIO, ou se ele falhar ao abrir, `set_audio_device({mode:"asio"})` responde `mode_unavailable` com o motivo, e o WASAPI continua funcionando com a configuração anterior.
- **CA-38 (modo instrumento/linha):** `input.mode = "instrument" | "line"` muda o alvo de calibração e o ganho de entrada, de modo que o mesmo sinal de -10 dBV num modo e de +4 dBu no outro chegue ao primeiro bloco no mesmo nível nominal (±1 dB). A UI informa que Hi-Z é uma propriedade elétrica da entrada da interface: o app não converte uma entrada de linha em Hi-Z por software.
- **CA-39 (reamp):** O usuário escolhe uma DI (arquivo ou gravação), uma saída física e uma entrada de retorno. O app toca a DI, grava o retorno e alinha a gravação pelo atraso de ida e volta **medido** por loopback, com o erro de alinhamento contra a DI em no máximo 1 amostra. O atraso medido, o backend, o driver, a taxa e o buffer ficam registrados no arquivo da sessão. A UI avisa que normalmente é preciso um reamp box entre a saída de linha e a entrada do amplificador.
- **CA-40 (Mod 2 ativo com split desligado):** Na topologia padrão com `split.enabled = false`, ligar `mod_2.enabled = true` (ex.: chorus) processa o sinal principal em série e altera a saída de forma mensurável em relação a `mod_2.enabled = false`.

### Parte E — não-regressão
- **CA-35:**
  - `cargo test --workspace` e `npm test` continuam verdes, inclusive os contratos das Specs 002, 009, 012, 034, 037 e 045.
  - O alocador de contagem (037 CA-7) continua registrando zero alocações, desalocações e locks no callback do standalone.
  - Knobs sem Modifier respondem ao arraste, ao `Shift+Drag` e ao duplo clique como hoje (009 CA-9).
  - O app desktop continua sem HTTP no release (Spec 033).

## Fora de escopo
- **Formatos AU, AAX e LV2 e builds para macOS e Linux:** o alvo do projeto hoje é Windows (Specs 013, 018 e 028). Os outros formatos e sistemas ficam para uma spec futura.
- **Assinatura de código e instalador dos plugins:** o CA-16 entrega os binários validados. A distribuição é outra decisão.
- **Modulação polifônica do CLAP:** o produto é monofônico na entrada (guitarra). Só a automação sample-accurate entra.
- **Trocar o crossfade de 20 ms pelo de 1,5 ms da Spec 009:** a duração continua a da Spec 045. Esta spec corrige o que o crossfade troca, não a duração.
- **Reamp por USB e FX Send/Return em placa que não tem esses canais:** o CA-27 desabilita esses destinos com honestidade, em vez de simulá-los.
- **Pitch polifônico:** continua fora, como na Spec 042.

## Invariantes tocadas
- **Zero alocação, desalocação, lock, I/O e log no callback (Specs 026 e 037):**
  - O CA-14 remove a disputa de lock vinda de um leitor.
  - O CA-7 elimina o double free.
  - Os CAs 21, 24 e 32 estendem a régua medida ao plugin, à troca de topologia e aos Modifiers.
  - O lock de produtor da fila de recursos fica do lado de controle; o `pop` do callback continua sem lock.
- **Tabela única de parâmetros (Spec 014):**
  - O CA-9 fecha a divergência entre `/api/params` e o caminho de preset.
  - O CA-18 torna a tabela a fonte dos parâmetros do plugin.
  - Os Modifiers (CA-32) só escrevem chaves da tabela, validadas nas faixas dela.
- **Release sem HTTP (Spec 033):** o CA-22 estende a regra ao plugin.
- **Host de áudio real e latência medida (Specs 018, 028 e 036):** o ASIO (CA-36) usa a mesma interface de backend do WASAPI, sem vazar tipos de backend para o `ampforge-dsp`. O atraso de reamp (CA-39) é medido, nunca estimado.
- **Todo controle visível funciona no backend (AGENTS.md) e telemetria honesta (Spec 036):** CA-10, CA-15, CA-27 e CA-31.
- **Persistência offline-first (Spec 010):**
  - Setlists, Modifiers e topologia são gravados no `.sonixpreset` e no `.sforgepkg`.
  - Campos novos exigem bump de schema com migração; a migração 1.0.0 → 1.1.0 continua funcionando (CA-35).
  - O estado do plugin é o mesmo documento (CA-19).

## Testes que cobrem esta spec
- `crates/sonix-desktop/tests/contract_049_scene_state_and_queues.rs` → CA-1, CA-2, CA-3, CA-7, CA-8, CA-9, CA-14 (`10 passed; 0 failed`)
- `apps/sonix-app/src-tauri/tests/contract_049_device_reconfigure.rs` → CA-4, CA-5, CA-6
- `crates/sonix-desktop/tests/contract_049_midi_ranges.rs` → CA-11 (`1 passed; 0 failed`)
- `crates/ampforge-dsp/tests/contract_049_oversampler.rs` → CA-12, CA-13 (`2 passed; 0 failed`)
- `tests/contract/049-status-bar-honesta.test.js` → CA-10
- `crates/sonix-desktop/tests/contract_044_sdd_integrity.rs` (estendido) → CA-15 (`7 passed; 0 failed`)
- `crates/sonix-plugin/tests/contract_049_plugins.rs` + script de validação (`clap-validator`, validador VST3) → CA-16..CA-22 (`7 passed; 0 failed`)
- `crates/sonix-desktop/tests/contract_049_live_dag.rs` → CA-23..CA-28, CA-40 (`8 passed; 0 failed`)
- `crates/sonix-desktop/tests/contract_049_setlists_modifiers_footswitch.rs` + `tests/contract/049-modifier-matrix.test.js` → CA-29..CA-34 (inclui CA-30b) (`9 passed; 0 failed`)
- `crates/sonix-desktop/tests/contract_049_asio_input_reamp.rs` → CA-36..CA-39 (CA-36 com hardware real registrado nas Notas) (`3 passed; 0 failed`)
- Não-regressão → CA-35

## Notas
- **Tamanho da spec:** são 40 CAs, acima da faixa de 5 a 15 do SDD do projeto. A spec única foi decisão explícita do usuário em 2026-11-15. Para manter o gate por fase do handoff, implemente por partes, com `cargo test --workspace` e `npm test` verdes ao fim de cada uma:
  1. **A** (CA-7 e CA-14 primeiro, porque removem UB e lock; depois CA-1..CA-6, CA-8..CA-13);
  2. **A′** (CA-15);
  3. **F** (requisitos do primeiro release);
  4. **C**;
  5. **D**;
  6. **B**, por último: o VST3 é evolução futura nas decisões de produto, e a Parte B depende da topologia (C) e dos Modifiers (D) para o estado completo do CA-19.
- **Arquitetura recomendada para o A1 (revisão de DSP, 2026-11-15):** blocos de tempo (looper, delay, reverb) devem ter **uma única instância**, fora do par ativa/sombra, como num rack real. A cena passa a controlar só o envio de entrada deles e os parâmetros com smoother. Com `spillover`, o envio fecha e a cauda continua; sem `spillover`, a saída do bloco faz fade. Só os blocos tonais e não lineares (amp, drive, EQ, cab) precisam de duas instâncias para o crossfade. Essa separação resolve CA-1, CA-2 e CA-3 de uma vez e reduz pela metade o custo de memória da sombra.
  - **Interação CA-3 × CA-30b na troca de canal:** o "estado zerado" do `CA-30b` aplica-se aos blocos tonais/por canal (amp, drive, EQ, cab, modulador), não às instâncias únicas pós-merge de delay e reverb quando `spillover = true` (`dly_1`/`rev_1` só limpam a cauda na troca se `spillover = false`). O DC blocker de entrada (`dc_x1`/`dc_y1`/`dc_x2`/`dc_y2`) também é pré-canal e não é zerado no meio de um sinal ativo.
  O status pode ficar como `implementada (Parte B pendente)` enquanto uma parte não fecha.
- **Licença do projeto (decisão do usuário, 2026-11-15):** o Sonix Pro é um projeto pessoal, sem fins comerciais, licenciado sob **GPL-3.0-or-later**. O texto oficial está em `LICENSE`, e o `Cargo.toml` do workspace declara `license = "GPL-3.0-or-later"` (antes, `"Proprietary"`). Situação das licenças de terceiros, conferida em 2026-11-15:
  - **SDK ASIO:** licença dupla desde outubro de 2025, acordo proprietário da Steinberg **ou** GPLv3. O projeto usa a **opção GPLv3**, sem precisar assinar o acordo proprietário.
  - **SDK VST3 3.8+:** MIT. As opções GPLv3 e proprietária deixaram de existir.
  - **CLAP:** MIT.
  - **Bindings Rust de VST3** (`vst3-sys`, caminho VST3 do `nih-plug`): GPLv3, compatível com o projeto.
- **ASIO, regras de build e distribuição (CA-36):**
  - o SDK ASIO **não** é versionado no repositório: o build o lê de um diretório local por `CPAL_ASIO_DIR`;
  - o suporte a ASIO fica atrás de uma feature de Cargo opcional no `sonix-desktop`, e sem o SDK o build continua funcionando só com WASAPI, o que mantém o CI e o CA-37 independentes do SDK;
  - o `bindgen` do `cpal` exige LLVM/Clang na máquina de build;
  - nenhum driver ASIO é empacotado (nem o ASIO4ALL): o driver é do fabricante da interface;
  - os nomes e logos "ASIO" e "VST" são marcas da Steinberg e são opcionais; se aparecerem na UI ou na documentação, seguem as regras de marca dela.
- **Parte B, decisão resolvida:** a pendência de licença foi fechada pela adoção da GPLv3, e **CLAP e VST3 entram juntos**, gerados do mesmo código por um framework que exporte os dois formatos (candidato: `nih-plug`), de modo que o CA-17 valha para os dois. A Parte B continua por último **por prioridade de produto**, já que plugin é evolução futura, e não mais por licença. Antes de codar o CA-16, confira de novo o `LICENSE` do framework e das bindings escolhidos e aplique a regra de versões do handoff (registro conferido e versão publicada há pelo menos duas semanas, registrada no Cortex).
- **Raiz comum de A1 e A2:** estado com memória duplicado entre duas instâncias, sem dono único. Os testes da 045 e da 034 mediam o fluxo de sinal (RMS, taxa), não a **identidade** do estado. Os CAs 1, 2, 4 e 7 medem identidade de propósito.
- **CA-13:** meça antes de otimizar e registre aqui o tempo atual e o novo, com CPU e build. Sem número medido, use `[medida pendente]`.
  - **Medição em `--release` (Windows x86_64, `cargo test --release -p ampforge-dsp --test contract_049_oversampler -- --nocapture`):**
    - Baseline (FIR 127 taps com deslocamento linear em todos os estágios): `stopband X2 = 112.27 dB`, `X4 = 112.41 dB`, `X8 = 112.77 dB`; aliasing 5 kHz `-6 dBFS` (8x) = `-27.23 dBc`; `WhiteBoxAmpProcessor` 1s@48kHz 8x = `188.81 ms`.
    - Pós-otimização (FIR polifásico simétrico em ring buffer `103/43/27` taps, Kaiser `beta = 11.5`): `stopband X2 = 114.54 dB`, `X4 = 114.47 dB`, `X8 = 115.39 dB`; aliasing 5 kHz `-6 dBFS` (8x) = `-27.23 dBc`; `HalfBandOversampler` 1s@48kHz 8x (upsample + downsample) = `11.73 ms` (`CA-12` latência por impulso exata dentro de ±1 amostra em 2x/4x/8x).
- **Build no Windows GNU (`windres`):** validado que `target-dir = "G:/Dev/.cargo-target/simulador-de-efeitos"` em `.cargo/config.toml` evita o erro de `windres` com espaços no caminho ao compilar `sonix-app`.
- **CA-36 (ASIO real):** `[verificação pendente]` — feature opcional `asio` (`cpal/asio`) e `start_realtime_duplex_with_mode` implementadas em `sonix-desktop` e `sonix-app`, mas não há `CPAL_ASIO_DIR` / hardware ASIO de referência conectado nesta estação de build.
- **CA-16 (validadores externos `clap-validator` e VST3 SDK `validator`):** `[verificação pendente de ferramenta externa]` — artefatos `sonix-pro.clap` e `sonix-pro.vst3` gerados via `nih_plug` (`nih_export_clap!` + `nih_export_vst3!`) e verificados em `crates/sonix-plugin/tests/contract_049_plugins.rs`; binários externos `clap-validator` e `validator.exe` não estão instalados nesta máquina.
- **Correções pós-review (`docs/handoff/ANTIGRAVITY-050-pos-review.md`, R1–R9, 2026-11-15):** concluídas em `ampforge-dsp`, `sonix-desktop` e `docs/ui-desktop/inventario-controles.md`: `Mod2` com split desligado (CA-40), spillover/DC blocker sem step na troca de canal (CA-3/CA-30b), restauração de base dos Modifiers e sincronia LFO com `shadow` no crossfade (CA-32), drenagem cooperativa de `retired_queue` (CA-24), `clear_tail()`/`reset_state()` O(1) sem memset de 3 MB no callback RT (CA-2), e `latency_samples()` exato em 2x/4x/8x (CA-12).
