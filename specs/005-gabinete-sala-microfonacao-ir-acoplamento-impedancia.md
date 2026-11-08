---
spec: 005
titulo: Bloco de gabinete (Cab), sala de microfonação dupla, loader de IR e acoplamento de impedância
status: aprovada
atualizado: 2026-11-08
---
# 005 — Bloco de gabinete (Cab), sala de microfonação dupla, loader de IR e acoplamento de impedância

## Objetivo
Permitir ao usuário microfonar visualmente o cone do alto-falante com até dois microfones simultâneos, carregar Impulse Responses (`.wav`) externos locais com convolução de latência zero e acoplar dinamicamente a curva de impedância eletromecânica da caixa ao bloco de Power Amp anterior.

## Contexto
Mais de 50% da assinatura espectral de um rig de guitarra vem da interação entre o amplificador de potência valvulado, a impedância reativa do alto-falante e a posição exata dos microfones diante do cone (*Dust Cap* vs *Cone Edge*). Em um amplificador real, a impedância do falante não é um resistor fixo de $8\ \Omega$: ela apresenta um pico ressonante grave ($65\text{–}130\text{ Hz}$) e uma subida indutiva nos agudos que alteram o ganho da malha de realimentação negativa do Power Amp.

Fontes da verdade e evidências no repositório:
- `docs/01-SRS-Levantamento-de-Requisitos.md:129-138` — Define **RF-013** (Sala Visual de Microfonação com 2 microfones, Posição 0–100%, Distância 0–30 cm, Ângulo 0° vs 45°), **RF-014** (Catálogo de Microfones `SM57`, `MD421`, `M88`, `R121`, `M160`, `U87`, `C414` + Nível, Pan, Mute/Solo, Fase 0°/180°, Delay 0.00–2.00 ms), **RF-015** (Loader de IRs `.wav` 16/24/32-bit, 44.1–96 kHz com reamostragem automática), **RF-016** (Acoplamento da Curva de Impedância ao Power Amp) e **RF-017** (Filtros `Low Cut` 20–500 Hz 6/12/18/24 dB/oct, `High Cut` 2–20 kHz, `Speaker Thump / Cone Cry`, `Room Mic / Early Reflections`).
- `docs/01-SRS-Levantamento-de-Requisitos.md:266-303` — Define a representação JSON completa do bloco `CAB` (`coupledAmpInstanceId`, `impedanceCurve` e vetor `mics`).
- `docs/02-Design-System-Specification.md:109-114` — Define a anatomia do componente `<sonix-cab-mics>`.
- `design-system/index.html:360-434` e `design-system/app.js:565-607` — O protótipo implementa o arraste de `#mic1Handle` e `#mic2Handle` no `#speakerConeStage`, porém limita a distância máxima a `12 cm` (`distanceCm = ((yPct / 100) * 12).toFixed(1)` em `app.js:591`, divergindo do limite de `30 cm` do **RF-013**), exibe os metadados de Nível/Fase/Delay como `<small>` estático (`index.html:391,399`), não processa o arquivo selecionado em `#irFileInput` (`index.html:422`) e não propaga a mudança de `#impedanceCurveSelect` (`index.html:426`) para o badge `#coupledImpedanceBadge` nem para o osciloscópio.

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Posicionamento no Cone (**RF-013**) | Arraste X/Y com distância limitada a `0–12 cm` e sem ângulo (`design-system/app.js:590-591`) | Eixo Posição `Cap → Edge (0% a 100%)`, Distância `0.0 cm a 30.0 cm` (Y/Scroll) e chave de Ângulo (`0° On-Axis` vs `45° Off-Axis`) |
| Catálogo e Mixer de Mics (**RF-014**) | Rótulos fixos `SM57` e `R121` (`design-system/index.html:388-399`) | Seleção entre os 7 modelos (`SM57`, `MD421`, `M88`, `R121`, `M160`, `U87`, `C414`) e controles editáveis de Nível, Pan, Mute/Solo, Fase (`0°/180°`) e Delay (`0.00–2.00 ms`) |
| Loader de IR `.wav` (**RF-015**) | `<input type="file" id="irFileInput">` sem parser (`design-system/index.html:422`) | Parser RIFF/WAVE local (mono/estéreo, 16/24/32-bit, 44.1–96 kHz), conversão automática para a taxa da sessão, favoritos, busca e convolução particionada zero-latency |
| Acoplamento de Impedância (**RF-016**) | Slider `sliderCabRes` isolado (`design-system/app.js:714`) | Transmissão automática ou manual de `lowResonanceHz (65–130 Hz)`, `Resonance Q` e `highInductanceRiseDb` para o bloco `AMP` acoplado (`coupledAmpInstanceId`) |
| Condicionamento e Sala (**RF-017**) | Apenas `High Cut` 3.5–18 kHz (`design-system/index.html:415`) | `Low Cut` (`20–500 Hz`, inclinação `6/12/18/24 dB/oct`), `High Cut` (`2–20 kHz`), `Speaker Thump / Cone Cry` e `Room Mic / Early Reflections` |

**Alternativas descartadas:**
1. *Convolução direta no domínio do tempo sem particionamento:* descartada porque um IR de 2048 amostras em convolução direta consome excesso de CPU; a convolução particionada híbrida (cabeça no domínio do tempo + cauda em FFT uniforme/não-uniforme) mantém latência zero e baixo custo de CPU.
2. *Carregamento de IRs diretamente na thread de áudio:* descartada por violar o **RNF-005** (Zero I/O e Zero Alloc no loop de áudio); a leitura do `.wav`, decodificação PCM e reamostragem ocorrem em thread de trabalho e a tabela de partições é trocada atomicamente.

## Comportamento esperado
1. **Sala Visual de Microfonação (RF-013):** Na visão `#cabRoomWorkbench`, o usuário posiciona até 2 cápsulas simultâneas sobre o palco `#speakerConeStage`:
   - `positionCapToEdge`: `0.00` (`0%`, centro da calota / *Dust Cap*, máximo brilho) a `1.00` (`100%`, borda do cone / *Cone Edge*, timbre mais grave/escuro);
   - `distanceCm`: `0.0 cm` (*close-mic* com efeito de proximidade) a `30.0 cm` (ajustável via eixo vertical ou roda de scroll sobre a cápsula);
   - `angleOffAxisDeg`: alternável entre `0°` (*On-Axis*) e `45°` (*Off-Axis*, atenuação natural de agudos ásperos).
2. **Catálogo e Controles por Slot de Microfone (RF-014):** Cada slot (`slot: 1` e `slot: 2`) permite escolher entre os modelos dinâmicos (`DYN_57` / SM57, `DYN_421` / MD421, `DYN_88` / M88), fita (`RIBBON_121` / R121, `RIBBON_160` / M160) e condensadores (`COND_87` / U87, `COND_414` / C414), além de ajustar `levelDb` (`-60.0` a `+12.0 dB`), `pan` (`-1.0` a `+1.0`), `mute`, `solo`, `phaseInvert` (`false` = `0°`, `true` = `180°`) e `delayMs` (`0.00` a `2.00 ms` para alinhamento de fase entre as duas cápsulas).
3. **Loader de Impulse Responses `.wav` Locais (RF-015):** O usuário pode carregar arquivos `.wav` locais (mono ou estéreo, PCM 16-bit, 24-bit ou IEEE float 32-bit, taxas de `44.1 kHz` a `96 kHz`). Se a taxa de amostragem do arquivo diferir da taxa da sessão ativa, o loader reamostra automaticamente o IR com filtro sinc em janela Kaiser antes de alimentar o motor de convolução particionada zero-latency. O navegador local permite marcar favoritos e filtrar IRs por texto.
4. **Acoplamento da Curva de Impedância (RF-016):** Quando um bloco `CAB` está conectado após um bloco `AMP` (vinculado via `coupledAmpInstanceId`), a seleção do gabinete (ou escolha manual em `#impedanceCurveSelect`) envia os parâmetros `lowResonanceHz` (`65–130 Hz`), `resonanceQ`, `lowResonanceGainDb` e `highInductanceRiseDb` para o Power Amp, atualizando tanto o cálculo da malha de realimentação negativa do amplificador quanto o badge `#coupledImpedanceBadge` e a aba `tonestack` do osciloscópio CRT.
5. **Condicionamento Acústico e Sala (RF-017):** O bloco aplica filtros `Low Cut` (`20 Hz` a `500 Hz` com inclinação selecionável de `6`, `12`, `18` ou `24 dB/oct`), `High Cut` (`2.0 kHz` a `20.0 kHz`), compressão/não-linearidade mecânica do cone (`Speaker Thump / Cone Cry`, `0–10`) e ambiência de microfone de sala (`Room Mic / Early Reflections`, `0–100%`).

## Critérios de aceite
- **CA-1:** Arrastar ou ajustar os controles das cápsulas 1 e 2 na Sala de Microfonação atualiza `positionCapToEdge` no intervalo `0%` a `100%`, `distanceCm` no intervalo `0.0 cm` a `30.0 cm` e `angleOffAxisDeg` (`0°` ou `45°`), alterando em tempo real a resposta espectral daquele slot.
- **CA-2:** Cada um dos 2 slots de microfone permite selecionar qualquer um dos 7 modelos (`SM57`, `MD421`, `M88`, `R121`, `M160`, `U87`, `C414`) e ajustar `levelDb`, `pan`, `mute`, `solo`, `phaseInvert` (`0°/180°`) e `delayMs` (`0.00–2.00 ms`).
- **CA-3:** Ajustar `delayMs` no Slot 2 em incrementos de `0.01 ms` introduz o atraso fracionário correspondente (ex.: `0.08 ms` = `3,84 samples` a `48 kHz` via interpolação all-pass/sinc) e inverter `phaseInvert` multiplica as amostras daquele slot por `-1.0`.
- **CA-4:** O loader de IR aceita arquivos `.wav` válidos (mono/estéreo, 16/24/32-bit, 44.1/48/88.2/96 kHz), converte automaticamente a taxa de amostragem quando diferente da taxa da sessão ativa e instancia a convolução particionada adicionando `0 samples` de latência extra além do buffer corrente.
- **CA-5:** Alterar o gabinete ou a curva em `#impedanceCurveSelect` (ou `sliderCabRes` entre `65 Hz` e `130 Hz`) atualiza imediatamente o objeto `impedanceCurve` transmitido ao `coupledAmpInstanceId`, o texto de `#coupledImpedanceBadge` e a curva laranja na aba `tonestack` do `#crtCanvas`.
- **CA-6:** Os controles de condicionamento permitem configurar `Low Cut` (`20–500 Hz` com slopes de `6`, `12`, `18` e `24 dB/oct`), `High Cut` (`2–20 kHz`), `Speaker Thump / Cone Cry` e `Room Mic / Early Reflections`.
- **CA-7:** Caso de erro — tentar carregar no `#irFileInput` um arquivo que não seja RIFF/WAVE válido, esteja truncado/corrompido ou possua duração vazia (`0 samples`) é recusado com erro explícito `InvalidImpulseResponseError`, mantendo o IR ou modelo de gabinete anterior em operação sem ruído ou queda de áudio.
- **CA-8:** Não-regressão — quando o Slot 2 está desabilitado (`enabled = false` ou `mute = true`), o processamento do Slot 1 permanece idêntico em ganho e fase, e o consumo de CPU da convolução do Slot 2 é poupado.

## Fora de escopo
- A modelagem interna das válvulas de saída do Power Amp que consome a curva de impedância (coberta pelas **Specs 003 e 004**).
- O empacotamento binário dos arquivos `.wav` dentro de `.sforgepkg` com hash SHA-256 (coberto pela **Spec 010**).

## Invariantes tocadas
- **RNF-001 (Latência < 1,5 ms):** A convolução particionada opera em modo *zero-latency* (primeira partição processada diretamente sem atraso de bloco FFT).
- **RNF-005 (Real-Time Safe):** Decodificação de `.wav`, leitura de disco e reamostragem ocorrem fora da thread de áudio.
- **RNF-009 (100% Offline):** Todo o catálogo de microfones, curvas de impedância e carregamento de IRs opera exclusivamente sobre o sistema de arquivos local.

## Testes que cobrem esta spec
- `tests/contract/005-gabinete-sala-microfonacao-ir-acoplamento-impedancia.test.js` → CA-1, CA-2, CA-3, CA-4, CA-5, CA-6, CA-7, CA-8 *(pendente — ciclo TDD red → green)*.

## Notas
- Rastreabilidade SRS: **RF-013**, **RF-014**, **RF-015**, **RF-016**, **RF-017**, Seção 5 (`blk_cab_1`); Design System Seção 4.4.
