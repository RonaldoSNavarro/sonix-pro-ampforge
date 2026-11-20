# SONIX PRO // AMPFORGE™ — Guia de Engenharia DSP Avançada e Tempo Real

Este documento define as diretrizes técnicas de modelagem de circuitos, processamento digital de sinais, tempo real e validação quantitativa aplicadas às specs modulares do AMPFORGE (Specs 053–084).

---

## 1. Estratégias de Modelagem de Circuitos Analógicos

Não imponha uma técnica única a todos os equipamentos. Escolha o método conforme topologia, grau de não linearidade, acoplamento entre nós, estabilidade numérica e orçamento de CPU.

### 1.1 Modified Nodal Analysis (MNA)
- Formula equações de corrente (KCL) e tensão (KVL) diretamente da netlist do circuito, produzindo sistemas lineares/não lineares `G·v + C·dv/dt + i_nl(v) = u(t)`.
- **Quando usar:** tone stacks passivos interativos (ex.: matriz KCL 5×5 em `tonestack.rs`), subcircuitos com múltiplos nós acoplados (cathode follower carregando tone stack, loops de realimentação com diodos como TS808/Klon, estágio de entrada com impedância de captador).
- **Discretização:** transformada bilinear (regra trapezoidal) para estabilidade A-stable sem ganho espúrio; aplicar pre-warping quando uma frequência de corte crítica exigir exatidão em taxas mais baixas (44,1/48 kHz).

### 1.2 Wave Digital Filters (WDF)
- Decompõe circuitos em portas de onda incidente/refletida (`a`, `b`) ligadas por adaptadores série/paralelo e uma raiz (idealmente o elemento não linear, como um par de diodos ou triodo simplificado).
- **Quando usar:** topologias em árvore ou com uma não linearidade dominante na raiz (redes de clipping de diodos, filtros passivos, wah com indutor LC, tone stacks com parâmetros variantes no tempo livres de cliques).
- **Limitação:** circuitos com múltiplas não linearidades acopladas exigem adaptadores R-type ou decomposição iterativa, onde MNA/State-Space pode ser mais direto.

### 1.3 State-Space (Espaço de Estados / Método DK)
- Representa capacitores/indutores como vetor de estados `x[n]`, separando a parte linear pré-computável da avaliação não linear `i_nl(v_nl)`.
- **Quando usar:** estágios valvulados com capacitores de acoplamento, bypass de cátodo e condução de grade; pedais multi-estágio (Big Muff, Fuzz Face) onde a memória de bias e carga entre estágios define a dinâmica.

### 1.4 Solução Numérica de Não Linearidades
- **Newton-Raphson (em `f64`):** sempre com número máximo de iterações fixo e determinístico (ex.: 4–8 iterações em RT), amortecimento (*damped Newton* / limite de passo) para evitar divergência em exponenciais de Shockley, e fallback seguro caso não convirja.
- **Anti-Derivative Antialiasing (ADAA 1ª/2ª ordem):** eficaz para curvas sem memória avaliadas em oversampling moderado (2×–4×), protegendo o limite `|x[n] - x[n-1]| < eps` via avaliação no ponto médio `0.5 * (x[n] + x[n-1])` para evitar cancelamento catastrófico.
- **Modelos de válvulas:** preferir modelos físicos identificáveis (Norman Koren, Child-Langmuir modificado com corrente de grade) ou curvas parametrizadas calibradas contra curvas de placa/grade dos datasheets (`12AX7`, `EL34`, `6L6GC`, `EL84`), nunca `tanh(x)` genérico sem calibração.
- **Modelos de diodos e transistores:** equação de Shockley (`Is`, `n·Vt`) para silício (`1N4148`/`1N914`), germânio (`1N34A`, joelho menor e mais suave) e LEDs; modelos Ebers-Moll simplificados para BJTs (`AC128`, `2N5088`, `BC108`) e transcondutância/resistência de canal para JFETs (`2N5457`, `2N5952`, 1176LN).

### 1.5 Impedâncias, Acoplamento e Memória Dinâmica
- Modele impedância de entrada e saída sempre que houver interação audível (ex.: Fuzz Face `Z_in` baixo carregando captador passivo; cathode follower `Z_out` baixo vs saída de placa `Z_out` alto alimentando tone stack).
- Preserve memória dinâmica do circuito: deslocamento de bias em corrente de grade (*blocking distortion*), queda e recuperação de tensão B+ (*power supply sag*), histerese/saturação LF do transformador de saída, e variação de resistência de fotocélula (Uni-Vibe / LA-2A).

---

## 2. Processamento Digital de Sinais (Diretrizes Obrigatórias)

### 2.1 Oversampling Seletivo e Anti-Aliasing
- Aplique oversampling **somente nos estágios não lineares** (pré-amp, power amp, drives/fuzzes), evitando custo desnecessário em blocos puramente lineares (EQ, delay digital, mixer).
- Filtros de interpolação (anti-imaging) e decimação (anti-aliasing) devem ser validados por medição DFT real da resposta ao impulso (`stopband_attenuation_db()`) e por medição de aliasing residual com senoide de 5 kHz em ganho máximo.
- Documente e reporte exatamente a latência introduzida por cada fator (`1x`, `2x`, `4x`, `8x`) via `latency_samples()`.
- Alinhe o padrão do motor (`2x` conforme Spec 051 CA-17, selecionável até `8x`) sem divergência entre UI, API e cálculo de `estimated_roundtrip_ms`.

### 2.2 Filtros IIR, FIR e Estabilidade Numérica
- Use **Direct Form II Transposed (TDF-II)** ou **State-Variable Filter (SVF / Cytomic TPT)** em `f64` para filtros IIR abaixo de 200 Hz, filtros com Q alto ou coeficientes modulados em tempo real (Wah, Phaser, Tone Stack). Evite Direct Form I/II em `f32` para frequências baixas.
- Todo coeficiente dependente de tempo ou frequência **deve ser calculado a partir da `sample_rate` efetiva** — proibido constantes mágicas presas a 48 kHz (Spec 039 CA-2).
- Garanta estabilidade de polos (`|p| < 1`) mesmo sob automação rápida ou mudança de sample rate (44,1 / 48 / 88,2 / 96 kHz).

### 2.3 Convolução Direta e Particionada (IR)
- **FIR direta** na partição 0 (ex.: 128 amostras) garante **latência zero** no caminho de gabinete.
- **Convolução particionada por FFT Overlap-Add (FDL)** para partições `p >= 1` viabiliza IRs de até 500 ms dentro do orçamento de CPU.
- **Resampling de IR:** sinc janelado (ex.: 48+ taps Blackman/Kaiser) executado **exclusivamente fora do callback de áudio**.
- **Troca de IR:** preparação fora da RT → envio por `SpscOwnedQueue` → crossfade suave → devolução da IR antiga ao `RetiredReaper` para `drop` fora da thread de áudio.

### 2.4 Higiene Numérica e Processamento por Blocos
- **DC offset:** manter DC blocker `f64` na entrada (e após estágios altamente assimétricos quando necessário), com resíduo DC `< -60 dBFS` em sinais simétricos.
- **Denormals (subnormals):** prevenir degradação de CPU em caudas de filtros IIR/reverb/delay com flush-to-zero (FTZ/DAZ quando apropriado) ou corte de piso abaixo de `1e-24` / pequeno offset alternado fora da faixa audível.
- **Defesa contra não-finitos:** sanitizar `NaN`/`Inf` na borda e resetar estados corrompidos se ocorrerem; nunca propagar `NaN` para o DAC.
- **Formatos PCM:** processamento interno em `f32` (barramento) com estados críticos em `f64` (precisão mista); conversão PCM `i16`/`i24`/`f32` isolada na fronteira do host/loader.
- **Processamento por blocos:** suportar qualquer tamanho de bloco (incluindo blocos ímpares/não potências de 2 como 441 ou 480 frames do WASAPI compartilhado) subdividindo em chunks internos `<= max_block` pré-alocados.

### 2.5 Dinâmica, Transientes, Latência e CPU
- **Envelopes:** ataque e release exponenciais calibrados por taxa; distinguir detector Peak, True-RMS (`env_lpf` criticamente amortecido sobre `x²`) e sidechain filtrado (LR4).
- **Compensação de latência (PDC):** cada processador declara `latency_samples()`. Caminhos paralelos (Split A/B, dry/wet) devem alinhar latência para evitar cancelamento de fase indesejado.
- **Orçamento de CPU:** medir tempo médio e pico por bloco em build `--release` via `CpuMonitor` (sem alocações nem syscalls custosas no loop interno).

---

## 3. Invariantes de Tempo Real (Hard Real-Time)

No callback de áudio (`process()`):
1. **Zero alocação e zero desalocação na heap** (nem `Vec::push` com realocação, nem `String`, nem `Box`, nem `drop` de `Arc` quando refcount chega a 0). Verificado por alocador global de contagem na thread RT (Spec 037 CA-7).
2. **Zero lock bloqueante** (`Mutex::lock`, `RwLock`, condvars, syscalls de contenção).
3. **Zero I/O** (arquivos, sockets, dispositivos) e **zero logging síncrono** (`println!`, `tracing` formatado).
4. **Zero panic** — nada de `unwrap()`, `expect()`, divisão por zero ou índice fora dos limites.
5. **Suavização de parâmetros** — parâmetros contínuos usam `OnePoleSmoother` (5–20 ms); mudanças discretas/topológicas usam crossfade controlado (≤ 20 ms) ou troca por sombra pré-alocada.
6. **Underruns/Overruns** — em falta de dados, preencher silenciosamente com `0.0` e incrementar contador atômico; nunca repetir buffer velho causando zumbido.
7. **Estabilidade em troca de `sample_rate` / `block_size`** — toda realocação e recálculo de tabelas ocorre em `prepare()` fora do callback.

---

## 4. Protocolo de Validação e Métricas Objetivas

Todo módulo especificado deve ser validado com sinais determinísticos e métricas reprováveis:

| Sinal de Teste | Métricas Verificadas |
|---|---|
| **Senoide (100 Hz, 1 kHz, 5 kHz)** | Ganho (dB), THD (%), razão harmônicos pares/ímpares (`H2/H3`), alias residual fora dos harmônicos (`dBFS`), DC offset (`dBFS`). |
| **Sweep logarítmico (20 Hz–20 kHz)** | Resposta em frequência (`±dB` vs referência analítica/circuito), fase, interação de tone stack, inclinação de filtros (6/12/24 dB/oit). |
| **Impulso unitário** | Convolução IR exata, alinhamento temporal/latência (em amostras), densidade de ecos de reverb, taps multi-head de delay. |
| **Multitone (2 tons, ex.: 1.9 + 2.0 kHz)** | Distorção por intermodulação (IMD) e comportamento de compressão em acordes. |
| **Degrau / Bursts (forte → silêncio)** | Tempo de ataque/release/hold, recuperação de sag B+, ausência de overshoot/instabilidade, rampa de mute/bypass. |
| **Ruído branco/rosa** | Entalhes de Phaser/Flanger (comb peaks/notches), calibração de piso de ruído, estabilidade em banda larga. |
| **Arquivos DI reais + diferentes intensidades** | Crest factor, RMS, limpeza ao reduzir volume de entrada (Fuzz Face/Plexi), comparação A/B level-matched (`±0.1 dB LUFS/RMS`). |
| **Estresse RT** | Contador de alocações = `0`, desalocações = `0`, locks = `0`, panics = `0`, CPU pico dentro do deadline do buffer em 44,1/48/96 kHz. |
