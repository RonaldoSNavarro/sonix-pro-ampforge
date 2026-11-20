# SONIX PRO // AMPFORGE™ — Guia de Engenharia DSP Avançada e Tempo Real

Este documento define as diretrizes técnicas de modelagem de circuitos, processamento digital de sinais, tempo real e validação quantitativa aplicadas às specs modulares do AMPFORGE (Specs 053–092).

---

## 1. Estratégias de Modelagem de Circuitos Analógicos

Não imponha uma técnica única a todos os equipamentos. Escolha o método conforme topologia, grau de não linearidade, acoplamento entre nós, estabilidade numérica e orçamento de CPU.

### 1.1 Modified Nodal Analysis (MNA)
- Formula equações de corrente (KCL) e tensão (KVL) diretamente da netlist do circuito, produzindo sistemas lineares/não lineares `G·v + C·dv/dt + i_nl(v) = u(t)`.
- **Quando usar:** tone stacks passivos interativos (FMV, Vox Top Boost `OS/010`, James/Baxandall, Big Muff, DS-1), subcircuitos com múltiplos nós acoplados (cathode follower carregando tone stack com impedância de fonte $R_{\text{src}}$ explícita, estágio de entrada com carga RLC de captador).
- **Discretização:** transformada bilinear (regra trapezoidal) em `f64` para estabilidade A-stable; aplicar pre-warping quando uma frequência crítica exigir exatidão próxima a Nyquist.

### 1.2 Wave Digital Filters (WDF)
- Decompõe circuitos em portas de onda incidente/refletida (`a`, `b`) ligadas por adaptadores série/paralelo e uma raiz não linear.
- **Quando usar:** topologias em árvore com uma não linearidade dominante na raiz (redes de clipping de diodos, wah com indutor LC, tone stacks variantes no tempo).

### 1.3 State-Space (Espaço de Estados / Método DK)
- Representa capacitores/indutores como vetor de estados `x[n]`, separando a parte linear pré-computável da avaliação não linear `i_nl(v_nl)`.
- **Quando usar:** estágios valvulados com capacitores de acoplamento, bypass de cátodo e condução de grade; pedais multi-estágio acoplados (Fuzz Face Q1–Q2 com realimentação global de 100 kΩ, Big Muff).

### 1.4 Solução Numérica de Não Linearidades
- **Newton-Raphson (em `f64`):** sempre com número máximo de iterações fixo e determinístico (4–8 iterações em RT), amortecimento (*damped Newton*) para exponenciais de Shockley e fallback seguro.
- **Anti-Derivative Antialiasing (ADAA 1ª/2ª ordem):** eficaz para curvas algébricas suaves em oversampling moderado (2×–8×), protegendo o limite `|x[n] - x[n-1]| < eps` via ponto médio `0.5 * (x[n] + x[n-1])` e expansão de Taylor assintótica.
- **Modelos de válvulas:** modelos físicos identificáveis (Norman Koren, Child-Langmuir modificado com corrente de grade) ou curvas parametrizadas calibradas contra datasheets (`12AX7/ECC83`, `12AT7`, `EF86`, `EL34`, `6L6GC`, `EL84`), separando triodo, pentodo, seguidor de cátodo e inversor de fase LTP (Spec 077).
- **Modelos de diodos e transistores:** equação de Shockley (`Is`, `n·Vt`) para silício (`1N4148`/`1N914`/`1S1588`) e germânio (`1N34A`); parâmetros $\beta$ ($h_{FE}$) e corrente de fuga $I_{CBO}$ diferenciados para transistores de germânio (`NKT275` vs `AC128` na Spec 063); transcondutância/resistência de canal para JFETs (`2N5952` no Phase 90, `2N5457` no 1176LN).

---

## 2. Processamento Digital de Sinais (Diretrizes Obrigatórias)

### 2.1 Oversampling Seletivo e Anti-Aliasing (Spec 079)
- Aplique oversampling **somente nos estágios não lineares** (pré-amp, power amp, drives/fuzzes).
- Todo critério de rejeição de filtro deve especificar:
  - **Banda passante** (ex.: `20 Hz–18 kHz` @ `fs = 48 kHz`) e **ripple máximo pico-a-pico** (ex.: `<= 0,25 dB`);
  - **Banda detransição** e **frequência de início da banda de rejeição** (`fs/2 = 24 kHz` até `M·fs/2`);
  - **Atenuação mínima na stopband** (`>= 70 dB` em `MinPhaseIir` Chebyshev 14ª ordem; `>= 96 dB` em `PolyphaseHalfBand` FIR Kaiser);
  - **Método de medição:** DFT de precisão `f64` sobre a resposta ao impulso completa sem janelamento truncado (ou senoide `5 kHz` a `-6 dBFS` com janela Hann de `N >= 16384` amostras após descarte de transiente inicial de `4096` amostras).

### 2.2 Filtros IIR, FIR e Estabilidade Numérica
- Use **Direct Form II Transposed (TDF-II)** ou **State-Variable Filter (SVF / Cytomic TPT)** em `f64` para filtros IIR `< 200 Hz`, filtros com Q alto e redes passivas moduladas.
- Todo coeficiente dependente de tempo ou frequência **deve ser calculado a partir da `sample_rate` efetiva** — proibido constantes por amostra presas a 48 kHz.
- Em frequências próximas de Nyquist (ex.: banda de `16 kHz` do MXR M108 em `44,1 kHz`, onde `f_N = 22,05 kHz`), considere o *frequency warping* da transformada bilinear (`ω_d = 2/T · arctan(ω_a · T/2)`) ou aplique pre-warping / correção de Orfanidis/Vicanek; nunca exija igualdade ingênua de largura de banda próxima a Nyquist sem compensação.

### 2.3 Convolução Particionada de Latência Algorítmica Adicional Zero (Spec 080)
- **Definição técnica precisa de "Zero-Latency Convolution":** significa **`0` amostras de atraso algorítmico adicional em relação ao bloco de entrada/saída do host** (`y[n]` contém instantaneamente a contribuição `h[0]·x[n]`).
- **Mecanismo (Gardner 1995 / UPOLS):**
  - Partição 0 (`h[0..B-1]`, `B = 128` amostras): calculada no domínio do tempo por convolução FIR direta causal em cada amostra.
  - Partições `p >= 1` (`h[p·B .. (p+1)·B-1]`): calculadas por FFT Overlap-Add/Save de tamanho `N = 2B = 256` em uma Frequency-Domain Delay Line (FDL), pois a saída da partição `p` só é necessária após `p·B` amostras.

### 2.4 Higiene Numérica, Headroom Interno e Barramento
- **Headroom interno vs Limiter de saída final:** o barramento `f32` entre blocos internos do grafo preserva headroom acima de `0 dBFS` (ex.: `±2.0` a `±4.0` linear, essencial para pedais de boost/Klon 18V empurrando estágios valvulados). A limitação estrita a `[-1.0, 1.0]` (`<= 0 dBFS`) é uma invariante da **saída final da cadeia para o DAC/host** (Spec 016), além da sanitização de `NaN`/`Inf` na fronteira de cada módulo.
- **Alinhamento de fase em caminhos paralelos internos:** sempre que um pedal misturar sinal seco com um ramo que passa por oversampling IIR/FIR (ex.: Klon Centaur `clean_blend + dirty`), o ramo seco deve receber compensação de atraso equivalente à latência do oversampler para evitar cancelamento comb em altas frequências.

---

## 3. Separação Estrita: Thread de Controle (Off-RT) vs. Thread de Áudio (RT)

1. **Thread de Controle (Off-RT):**
   - Executa `AudioProcessor::prepare(spec)`, leitura/validação/resampling de arquivos WAV/IR, compilação topológica do DAG (`CompiledGraph`), instanciação de novos modelos para troca estrutural e coleta de lixo (`RetiredReaper::drain()`).
   - **Pode alocar memória na heap** para preparar estruturas imutáveis ou pré-dimensionadas antes de enviá-las à thread RT via `SpscOwnedQueue`.
2. **Thread de Áudio (Hard Real-Time — `process()`, `apply_param_rt()`, `reset()`):**
   - **Zero alocação e zero desalocação na heap** (`alloc_count == 0` e `dealloc_count == 0` verificados por alocador global de contagem `CountingAllocator` ativo exclusivamente na thread RT).
   - **Zero lock bloqueante (`Mutex`, `RwLock`)**, **zero I/O**, **zero logging síncrono**, **zero panic**, **zero `memcpy` gigante (> 1 MB, ex.: buffers de 60 s do Looper)**.
   - Trocas de modelo ou IR na thread RT apenas alternam/interpolam instâncias já pré-alocadas (`active` / `shadow`) durante crossfade (≤ 20 ms) e empurram o ponteiro antigo para a fila SPSC do `RetiredReaper`.

---

## 4. Protocolo de Validação e Requisitos Obrigatórios para Critérios Numéricos

Todo critério de aceite numérico (`CA-n`) em uma Spec deve declarar explicitamente:
1. **Classificação:** `DOCUMENTED`, `MEASURED`, `ANALYTICAL`, `ENGINEERING_TARGET`, `REGRESSION` ou `PENDING`.
2. **Sinal de entrada:** tipo (senoide, sweep logarítmico, impulso unitário, multitone, burst, ruído branco, DI), amplitude em `dBFS` (ou linear) e frequência(s) em `Hz`.
3. **Ambiente digital:** `sample_rate` (`44100`, `48000`, `88200`, `96000 Hz`) e `block_size` (ex.: `64`, `128`, `441`, `480` amostras).
4. **Configuração do processador:** modo, fator de oversampling e valores exatos de todos os parâmetros relevantes.
5. **Método de medição:** janela de descarte de transiente, tamanho da DFT e janela (ex.: Hann `16384` pontos), fórmula de THD / RMS / RT60 / diferença espectral, e **level-matching (`±0,1 dB RMS`)** obrigatório sempre que comparar algoritmos ou modelos diferentes.
6. **Tolerância e fonte da tolerância:** justificativa analítica (ex.: erro de discretização bilinear), tolerância de componentes reais (ex.: `±5%` / `±10%` / `±20%` de potenciômetros e capacitores) ou meta de engenharia do software (`ENGINEERING_TARGET`).
