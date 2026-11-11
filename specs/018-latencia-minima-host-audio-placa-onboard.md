---
spec: 018
titulo: O caminho de áudio soma até 50 ms de atraso evitável e não usa o modo de menor latência da placa onboard
status: implementada
atualizado: 2026-11-10
---
# 018 — Latência Mínima e Host de Áudio Robusto em Placa Onboard

## Objetivo
Fazer o atraso entre tocar a corda e ouvir o som ser o menor que a placa de som onboard do
usuário permite — sem atraso extra criado pelo SONIX PRO — e mostrar esse atraso medido, em vez
de estimado.

## Contexto
Origem: pedido do usuário em 2026-11-09 ("eu ainda não tenho interface de áudio, apenas a placa de
som onboard do computador, o sistema deve ser otimizado para isso e o áudio não pode sofrer
atrasos"). Hardware alvo: codec Realtek HD Audio onboard, guitarra passiva na entrada P2 de
microfone (Spec 013). Evidência no código atual:

| Problema | Ponteiro | Efeito |
|---|---|---|
| Fila captura→reprodução só é esvaziada acima de **2.400 amostras (50 ms)** e, quando esvazia, **mantém 960 (20 ms)** | `audio.rs:382-388` | atraso extra entre 0 e 50 ms, variável, somado ao do driver |
| WASAPI apenas em modo **compartilhado**: o `cpal` 0.18.2 fixa `AUDCLNT_SHAREMODE_SHARED` | `~/.cargo/registry/.../cpal-0.18.2/src/host/wasapi/device.rs:867,968` | o sinal passa pelo mixer do Windows e pelos efeitos de sinal (APOs) do driver |
| Configuração padrão do dispositivo (`default_input_config`/`default_output_config`), sem pedir buffer | `audio.rs:153-158` | período decidido pelo Windows, não pelo app |
| Cadeia DSP criada fixa em 48 kHz, sem consultar a taxa negociada | `main.rs:40` | com a placa em 44,1 kHz, filtros fora de calibração e captura/reprodução desencontradas |
| `Mutex` em volta do produtor e do consumidor da fila e `try_lock` na cadeia DSP | `audio.rs:335-337, 249-253` | trava no caminho crítico (a Spec 013 CA-3 declara ausência de mutex, mas o teste `contract_013…:78-105` não verifica isso) |
| Alocação a cada callback (`Vec::with_capacity`, `vec![0.0; n]`) | `audio.rs:173, 248-249` | alocação no caminho crítico (Spec 001, Spec 013 CA-3) |
| Mono por **máximo absoluto entre L e R, amostra a amostra** | `audio.rs:176-180` | quando L e R diferem, gera distorção que não pertence ao amp |
| Medidor de CPU da UI é **estimado em JavaScript** por soma de constantes por bloco | `app.js:565-579` | o número exibido não tem relação com o custo real do motor |
| Erros de underrun/overrun são descartados sem contagem | `audio.rs:191-195, 269-273` | não há como saber se o buffer está pequeno demais |

Fatos de plataforma usados (documentação da Microsoft para WASAPI; confirmar na versão do Windows
do usuário): no modo compartilhado o período padrão do mecanismo de áudio costuma ser da ordem de
10 ms; `IAudioClient3` (Windows 10+) permite períodos menores no modo compartilhado quando o driver
suporta; o modo exclusivo dispensa o mixer e os APOs; a opção de stream `RAW`
(`AUDCLNT_STREAMOPTIONS_RAW`) pede ao driver para não aplicar processamento de sinal no modo
compartilhado, quando suportada; a thread de áudio pode ser registrada no MMCSS com a tarefa
"Pro Audio". **Nenhum número de latência desta spec é afirmado sem medição** (skill do projeto
§2.9 e `references/architecture-rust-windows.md` §8.4): o valor real da placa do usuário é medido
pelo CA-10.

## Comportamento esperado
1. **Backends atrás de `AudioBackend`.** Três modos, nesta ordem de tentativa padrão:
   (a) WASAPI exclusivo orientado a eventos; (b) WASAPI compartilhado de baixa latência
   (`IAudioClient3` com o menor período suportado e `RAW` quando disponível); (c) WASAPI
   compartilhado padrão. ASIO (feature `asio` do `cpal`, para drivers genéricos como
   ASIO4ALL/FlexASIO instalados pelo usuário) é selecionável quando presente. O modo efetivo e o
   motivo de cada recuo ficam visíveis. Como o `cpal` não oferece modo exclusivo, o modo (a) usa
   um adaptador próprio (candidata: crate `wasapi`; versão, licença e estabilidade a validar antes
   do merge) sem vazar tipos do backend para `ampforge-dsp`.
2. **Formato negociado prepara o motor.** Captura e reprodução abrem na mesma taxa de amostragem;
   a cadeia DSP é preparada (`prepare(sample_rate, max_block)`) com a taxa e o tamanho máximo de
   bloco efetivos, fora do callback. Sem taxa comum aos dois dispositivos, o modo exclusivo é
   recusado com erro que lista as taxas de cada lado e o host recua para o compartilhado.
3. **Fila mínima com controle de deriva.** A fila entre captura e reprodução mira **1 período** de
   ocupação e trabalha na faixa [0,5; 2] períodos. Diferença de relógio entre os dois endpoints é
   corrigida por reamostragem fracionária lenta (ajuste contínuo da razão), sem descartar nem
   repetir blocos. O esvaziamento por limiar de 50 ms deixa de existir.
4. **Caminho crítico de tempo real.** Nenhuma alocação, lock, I/O ou log nos callbacks; buffers
   pré-alocados para o maior bloco; fila SPSC sem `Mutex`; parâmetros chegam pela ponte da
   Spec 014. A thread de áudio é registrada no MMCSS ("Pro Audio"). Pânico dentro do
   processamento não atravessa o callback: o bloco sai em silêncio e um contador é incrementado.
5. **Seleção de canal de entrada explícita.** A conversão para mono nunca escolhe canal amostra
   a amostra. Parâmetros desta spec, no registro da Spec 014 §2:

   | Chave | Tipo | Valores | Valor no boot | Aplicação |
   |---|---|---|---|---|
   | `input.channel` | enum | `left`, `right`, `sum` | `left` | crossfade de 10 ms |
   | `audio.mode` | enum | `auto`, `wasapi_exclusive`, `wasapi_shared_low_latency`, `wasapi_shared`, `asio` | `auto` | reabre o stream fora do callback |
   | `audio.period_frames` | número | 32–1024, potência de 2 | menor suportado pelo modo | reabre o stream |

   Se a Spec 018 for implementada antes da 014, os mesmos valores são lidos das variáveis de
   ambiente `SONIX_INPUT_CHANNEL`, `SONIX_AUDIO_MODE` e `SONIX_PERIOD_FRAMES`.
6. **Telemetria real.** CPU = tempo de processamento do bloco ÷ duração do período (média e pico
   em janela de 1 s), contadores de underrun/overrun/pânico, modo, taxa, períodos de entrada e
   saída, latências informadas pelo driver e a latência de ida e volta medida — expostos em
   `GET /api/audio/status` e usados pela UI (`#cpuMeterText` deixa de ser estimado). Um painel
   "Áudio" (`#audioSettingsPanel`) mostra modo, dispositivos, taxa, períodos e latência medida, e
   permite trocar modo, período e canal de entrada e disparar a medição do §7.
7. **Medição de ida e volta por loopback.** Com um cabo da saída de fone para a entrada de
   microfone, o app emite um sinal de teste curto (varredura ou ruído com envelope, em −20 dBFS)
   e encontra o atraso por correlação cruzada. O resultado fica salvo por combinação
   dispositivo + modo + taxa + período e aparece na UI. A UI instrui a usar volume baixo.
8. **Perda e retorno de dispositivo.** Desconexão vira estado `device_lost` (saída em silêncio,
   sem pânico); o host tenta reabrir a cada 2 s e retoma sem reiniciar o app.

## Critérios de aceite
- **CA-1:** Com um host simulado (relógios virtuais, mesma taxa, período de 128 quadros) rodando
  60 s simulados, a ocupação da fila fica em [0,5; 2] períodos em 100 % dos callbacks após o
  primeiro segundo (hoje pode chegar a 2.400 amostras).
- **CA-2:** Com deriva simulada de ±200 ppm entre captura e reprodução (valor de teste, não
  medição) por 10 min simulados, a ocupação continua em [0,5; 2] períodos, e um seno de 1 kHz
  atravessando a fila não tem descontinuidade: `max |y[n] − y[n−1]|` ≤ 1,05 × o do seno ideal.
- **CA-3:** Executando 10.000 callbacks de captura e de reprodução no host simulado, um alocador
  global contador registra **0** alocações e um wrapper de sincronização registra **0** aquisições
  de lock dentro dos callbacks.
- **CA-4:** Com o dispositivo simulado em 44.100 Hz, `RealtimeDspChain` é preparado com
  `sample_rate() == 44100.0`; com captura em 44.100 Hz e reprodução em 48.000 Hz sem taxa comum, a
  abertura em modo exclusivo retorna erro com as duas listas de taxas e o host recua para o
  compartilhado, informando o motivo em `GET /api/audio/status`.
- **CA-5:** Entrada estéreo com L = seno 0,5 e R = mesmo seno invertido: `input.channel=left`
  produz mono bit-idêntico a L, `right` bit-idêntico a R e `sum` resulta em zero (±1e-7). Hoje a
  regra do máximo absoluto produz um sinal retificado.
- **CA-6:** Com um backend simulado em que o modo exclusivo falha (formato não suportado ou
  dispositivo em uso), o host abre o compartilhado de baixa latência; se este também falhar, abre
  o compartilhado padrão; em nenhum caso entra em pânico, e `GET /api/audio/status` informa
  `mode` e `fallback_reason`.
- **CA-7:** `GET /api/audio/status` devolve JSON com `backend`, `mode`
  (`wasapi_exclusive`|`wasapi_shared_low_latency`|`wasapi_shared`|`asio`), `raw_processing`,
  `sample_rate`, `input_period_frames`, `output_period_frames`, `input_device`, `output_device`,
  `driver_input_latency_ms`, `driver_output_latency_ms`, `cpu_avg_pct`, `cpu_peak_pct`,
  `underruns`, `overruns`, `panics`, `measured_roundtrip_ms` (número ou `null`).
- **CA-8:** Na UI, `#cpuMeterText` mostra `cpu_avg_pct` recebido do motor e
  `#audioSettingsPanel` mostra `mode`, `sample_rate`, os dois períodos e `measured_roundtrip_ms`
  (ou "não medida"); mudar o seletor de canal envia `input.channel` (teste JS com `fetch`
  simulado). `updateCpuBudget` não escreve mais em `#cpuMeterText`.
- **CA-9:** Medição offline: laço simulado com atraso conhecido de 517 amostras e ruído a −40 dB
  retorna 517 ± 1 amostra; sem cabo (nenhum pico de correlação acima do limiar) retorna erro
  `"sinal de retorno não detectado"`, nunca um número.
- **CA-10 (manual, registrado no PR):** No PC do usuário, para cada modo disponível em 48 kHz, a
  medição do §7 é registrada nas Notas junto com período e latências do driver. Aceite: a latência
  adicionada pelo app (medida − períodos de entrada e saída − latências informadas pelo driver) é
  ≤ 1 período de saída.
- **CA-11:** Um pânico provocado dentro do processamento (hook de teste) gera um bloco de silêncio,
  incrementa `panics` e o stream continua: os 100 blocos seguintes são processados normalmente.
- **CA-12:** Remover o dispositivo simulado leva a `device_lost` com saída em silêncio; ao
  reaparecer, o host reabre em ≤ 2,5 s e volta a processar sem reiniciar o app.
- **CA-13:** Não-regressão — os testes das Specs 001 e 013 continuam verdes (o teste da Spec 013
  CA-3 passa a ser complementado pelo CA-3 desta spec).

## Fora de escopo
- Calibração de nível de entrada, piso de ruído e sinal de teste → **Spec 019**.
- Latência algorítmica dos blocos (oversampling, convolução) → declarada por bloco nas Specs 016,
  020 e 022 e somada na telemetria desta spec.
- Instalador ou redistribuição de drivers ASIO de terceiros (ASIO4ALL/FlexASIO): o usuário instala;
  o app só os detecta. Licença do SDK ASIO a verificar antes do merge (skill §3.1).
- Múltiplas entradas/saídas simultâneas e roteamento físico N×M (backlog da skill, Épico C).

## Invariantes tocadas
- Áudio em tempo real (Spec 001/013): reforçada e agora verificada por teste (CA-3).
- Núcleo DSP desacoplado (skill §3.1): tipos de WASAPI/ASIO ficam no crate desktop.
- Spec 013 CA-3 (lock-free, zero alocação): passa a ser cumprida de fato.

## Testes que cobrem esta spec
Planejados (red → green): `crates/sonix-desktop/tests/contract_018_low_latency_host.rs` (CA-1…CA-7,
CA-9, CA-11…CA-13, com host simulado) e
`tests/contract/018-latencia-minima-host-audio-placa-onboard.test.js` (CA-8). CA-10 é manual com
hardware, documentado no PR.

## Notas
- Para a medição do CA-10 o usuário precisa de um cabo P2 macho-macho (saída de fone → entrada de
  microfone). Sem ele, o CA-10 fica pendente e a spec não fecha.
- No modo exclusivo, outros programas não usam a placa enquanto o SONIX PRO estiver aberto; a UI
  avisa isso ao ativá-lo.
- Implementar antes da Spec 014: todo controle novo é sentido através deste caminho.
