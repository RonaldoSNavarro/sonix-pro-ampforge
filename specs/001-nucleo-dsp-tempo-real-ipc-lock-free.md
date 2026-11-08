---
spec: 001
titulo: Isolamento em tempo real do motor DSP, comunicação lock-free e orçamento de CPU
status: aprovada
atualizado: 2026-11-08
---
# 001 — Isolamento em tempo real do motor DSP, comunicação lock-free e orçamento de CPU

## Objetivo
Garantir que o processamento de áudio de guitarra e contrabaixo ocorra com latência algorítmica inferior a 1,5 ms e zero cortes de áudio (*dropouts/xruns*), isolando completamente a thread de áudio em tempo real das operações de interface gráfica e sistema operacional para o músico em palco ou estúdio.

## Contexto
Sistemas de modelagem de amplificadores e efeitos que compartilham locks ou alocações dinâmicas entre a thread de interface (60/120 FPS) e o callback de áudio sofrem inversão de prioridade e *xruns* audíveis quando o usuário troca cenas, move knobs rapidamente ou carrega arquivos locais.

Fontes da verdade e evidências no repositório:
- `docs/01-SRS-Levantamento-de-Requisitos.md:18-22` — Define o núcleo nativo **AmpForge™ DSP Core Engine** em **Rust** sem dependências de SO no loop de áudio, compilado para **Desktop Standalone (`Tauri v2`)** e **Plugins de DAW (`VST3 / CLAP`)** (sem alvo Web/WASM no escopo atual).
- `docs/01-SRS-Levantamento-de-Requisitos.md:29-80` — Define a arquitetura híbrida Desktop com isolamento estrito via filas *Single-Producer Single-Consumer (SPSC)* e troca atômica de ponteiro de grafo (*RCU*).
- `docs/01-SRS-Levantamento-de-Requisitos.md:184-195` — Define os requisitos não-funcionais **RNF-001** a **RNF-007** (latência < 1,5 ms, sample rates de 44,1 a 96 kHz, buffers de 32 a 512 samples, precisão mista 32-bit/64-bit, oversampling 1x–8x, Real-Time Safe, suavização de parâmetros de 1 polo e teto de CPU de 88%).
- `design-system/index.html:65-68` e `design-system/app.js:546-560` — O protótipo atual calcula apenas uma estimativa estática de CPU (`total = 4.2 + soma(b.cpu * factor)`) e colore de vermelho acima de 75%, sem medição real por janela de buffer nem bloqueio preventivo em 88%.

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Execução do loop de áudio | Simulação visual em `requestAnimationFrame` (`design-system/app.js:310-433`) | Thread de áudio nativa em Rust (`process_block`) com garantia Zero-Alloc / Zero-Lock (**RNF-005**) |
| Comunicação UI ↔ DSP | Mutação direta do objeto global `state` (`design-system/app.js:6-71`) | `SPSC Command Queue`, `Atomic Graph Pointer Swap (RCU)` e `SPSC Telemetry Queue` |
| Precisão numérica e Oversampling | `Math.tanh` simples em JS (`design-system/app.js:300-308`) | Barramento 32-bit `f32` SIMD + Solvers/IIR < 100 Hz em 64-bit `f64` (**RNF-003**) + Oversampling polifásico Half-Band 1x/2x/4x/8x > 96 dB (**RNF-004**) |
| Suavização de parâmetros | Aplicação imediata no valor do knob (`design-system/app.js:246-260`) | Filtro suavizador de 1 polo (*One-Pole Parameter Smoother*, 5 ms a 20 ms) em nível de sample (**RNF-006**) |
| Orçamento e proteção de CPU | Soma estática sem bloqueio (`design-system/app.js:546-560`) | Medição real contra $\text{Deadline} = \text{BufferSize}/\text{SampleRate}$ e bloqueio de novos blocos pesados em $\ge 88\%$ (**RNF-007**) |

**Alternativas descartadas:**
1. *Sincronização por `std::sync::Mutex` / `RwLock` com herança de prioridade:* descartada porque mesmo um lock não disputado pode causar *priority inversion* ou chamada de kernel (`futex`) incompatível com buffers de 32 amostras (0,66 ms a 48 kHz).
2. *Processamento 100% em 64-bit `f64` em todo o barramento do Grid:* descartada porque reduz pela metade a largura de vetores SIMD (`AVX2` / `NEON`) nos blocos lineares; manteve-se 32-bit `f32` no barramento e 64-bit `f64` restrito aos solvers não-lineares (Newton-Raphson/DK-Method) e acumuladores IIR abaixo de 100 Hz.

## Comportamento esperado
1. **Configuração de Áudio e Latência:** O motor DSP inicializa e processa fluxos contínuos nas taxas de amostragem de `44100 Hz`, `48000 Hz`, `88200 Hz` e `96000 Hz`, com tamanhos de buffer configuráveis entre `32` e `512` amostras. A soma da latência algorítmica interna (filtros de oversampling de fase mínima + convolução particionada de gabinete) permanece estritamente abaixo de `1,5 ms` a 48 kHz e 96 kHz.
2. **Ponte Lock-Free (SPSC + RCU):**
   - Alterações de parâmetros contínuos, estados de bypass, seleção de canais e eventos MIDI entram exclusivamente pela fila **SPSC Command Queue** sem bloqueio.
   - Mudanças estruturais na topologia do Grid 4×12 são preparadas em background na Main Thread e aplicadas no início do próximo ciclo de áudio via troca atômica de ponteiro (**Atomic Graph Pointer Swap / RCU**).
   - O extrator de telemetria publica níveis VU/Peak (IN/OUT), redução de ganho (GR), janelas de osciloscópio/RTA, leitura do afinador e percentual de CPU na **SPSC Telemetry Queue** para consumo assíncrono da UI.
3. **Precisão Numérica e Oversampling Anti-Aliasing:**
   - Os buffers entre blocos do Grid trafegam em `32-bit IEEE 754 float` alinhados para instruções SIMD.
   - Todo solver não-linear de circuito e filtro IIR com frequência de corte $< 100\text{ Hz}$ executa acumulação interna em `64-bit double`.
   - Blocos não-lineares (`DRIVE`, `AMP`, `Fuzz`) operam com fator de oversampling selecionável (`1x`, `2x`, `4x`, `8x`) utilizando filtros polifásicos Half-Band com rejeição de banda de stop $> 96\text{ dB}$.
4. **Suavização de Parâmetros:** Qualquer atualização de parâmetro contínuo (ganho, frequência de EQ, mix, pedal de expressão) é interpolada amostra a amostra por um *One-Pole Parameter Smoother* com constante de tempo configurável entre `5 ms` e `20 ms`, eliminando degraus descontínuos (*zipper noise*).
5. **Orçamento de CPU/DSP e Proteção contra Dropouts:** A cada bloco de áudio, o motor mede o tempo de execução real individual de cada bloco e o tempo total frente ao deadline $\text{Deadline} = \frac{\text{BufferSize}}{\text{SampleRate}}$. Quando o consumo global atinge ou ultrapassa `88,0%`, o estado de proteção de sobrecarga é ativado, exibindo alerta preventivo na UI e recusando a inserção ou ativação de novos blocos pesados.

## Critérios de aceite
- **CA-1:** A latência algorítmica interna total medida entre a entrada do `Input RingBuffer` e a saída do `Crossfader` (com oversampling `4x` ativo e convolução de IR carregada) é inferior a `1,5 ms` (máximo de `72 samples` a `48 kHz` e `144 samples` a `96 kHz`).
- **CA-2:** O motor processa blocos sem falha ou subfluxo nas quatro taxas de amostragem (`44.1 kHz`, `48 kHz`, `88.2 kHz`, `96 kHz`) e em todos os tamanhos de buffer potências de dois de `32` a `512` amostras.
- **CA-3:** Durante a execução de `processBlock`, o contador de alocações dinâmicas de heap (`malloc`/`new`/`free`/`delete`), aquisições de mutex e chamadas de I/O permanece igual a `0` mesmo sob rajada de 1.000 mensagens/segundo na `SPSC Command Queue`.
- **CA-4:** Os buffers de barramento entre blocos operam em `float32` alinhado a 16/32 bytes, enquanto o estado interno dos solvers não-lineares e os coeficientes/acumuladores de filtros IIR com $f_c < 100\text{ Hz}$ operam em `float64`, mantendo erro de quantização DC inferior a `-120 dBFS`.
- **CA-5:** Nos modos de oversampling `2x`, `4x` e `8x`, os filtros polifásicos Half-Band de upsampling/downsampling apresentam atenuação na banda de rejeição superior a `96 dB` para frequências acima de Nyquist.
- **CA-6:** Uma variação abrupta em degrau (ex.: `0.0` para `10.0` em um único frame) em qualquer parâmetro contínuo produz uma transição suave no sinal de áudio governada por filtro de 1 polo com constante de tempo entre `5 ms` e `20 ms`, sem gerar picos espúrios de derivada (*clicks/zipper noise*).
- **CA-7:** A substituição da topologia do grafo via `Atomic Graph Pointer Swap (RCU)` ocorre entre ciclos de `processBlock` em tempo $O(1)$ na thread de áudio, sem zerar buffers nem produzir descontinuidade no sinal de saída.
- **CA-8:** O extrator de telemetria calcula o custo percentual de cada bloco ativo e o consumo global pela razão $\frac{T_{\text{exec}}}{\text{BufferSize} / \text{SampleRate}} \times 100$ e publica o snapshot na `SPSC Telemetry Queue` sem bloquear caso a fila esteja cheia (descarta o frame de telemetria mais antigo/excedente em vez de esperar).
- **CA-9:** Quando o consumo global de DSP atinge ou supera `88,0%`, o sistema ativa a flag `cpuOverloadWarning = true`, destaca o indicador `#cpuMeterText` em estado de alerta e recusa a adição ou ativação de novos blocos pesados com mensagem explícita de proteção contra *audio dropout (xrun)*.
- **CA-10:** Caso de erro — a tentativa de inicializar o motor com taxa de amostragem não suportada (ex.: `8000 Hz`) ou tamanho de buffer fora do intervalo `32..512` (ex.: `0` ou `4096`) é recusada na inicialização com erro estruturado `InvalidAudioConfigError`, sem iniciar a thread de tempo real.
- **CA-11:** Não-regressão — quando a `SPSC Command Queue` está vazia e nenhum parâmetro foi alterado, o motor executa o grafo ativo mantendo bit-a-bit o determinismo da saída para a mesma semente e sequência de amostras de entrada.

## Fora de escopo
- Compilação ou execução em navegador Web (`WebAssembly` / `AudioWorklet` / `WebAudio`), dado que o produto é estritamente Desktop (`Tauri v2 Standalone` + `VST3/CLAP`) nesta fase (ver **Spec 012**).
- A topologia de células, nós de Split/Mixer e prevenção de ciclos na matriz 4×12 (cobertos pela **Spec 002**).
- As equações específicas de cada válvula, tone stack e transformador do amplificador (cobertas pelas **Specs 003 e 004**).
- O crossfade gapless de troca de Scenes e canais A/B/C/D (coberto pela **Spec 009**).

## Invariantes tocadas
- **RNF-001 (Latência < 1,5 ms):** Fundada e verificada diretamente no CA-1.
- **RNF-003 (Precisão Mista 32-bit/64-bit):** Garantida no CA-4.
- **RNF-005 (Real-Time Safe — Zero Lock / Zero Alloc):** Invariante central do projeto estabelecida no CA-3 e CA-7.
- **RNF-006 (Suavização de Parâmetros):** Garantida no CA-6.
- **RNF-007 (Orçamento de CPU e Limite de 88%):** Garantida no CA-8 e CA-9.

## Testes que cobrem esta spec
- `tests/contract/001-nucleo-dsp-tempo-real-ipc-lock-free.test.js` → CA-1, CA-2, CA-3, CA-4, CA-5, CA-6, CA-7, CA-8, CA-9, CA-10, CA-11 *(pendente — ciclo TDD red → green)*.

## Notas
- Rastreabilidade SRS: Seção 2.1, **RNF-001**, **RNF-002**, **RNF-003**, **RNF-004**, **RNF-005**, **RNF-006**, **RNF-007**.
