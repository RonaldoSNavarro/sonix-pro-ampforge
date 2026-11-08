---
spec: 004
titulo: Bancada Chassi Aberto (Mod Shop) e osciloscópio CRT de transferência em tempo real
status: aprovada
atualizado: 2026-11-08
---
# 004 — Bancada Chassi Aberto (Mod Shop) e osciloscópio CRT de transferência em tempo real

## Objetivo
Permitir que o músico ou técnico de amplificadores abra o chassi virtual do cabeçote para substituir válvulas de pré e potência, recalibrar tensões de placa, corrente de bias, sag da fonte e topologia do tone stack, inspecionando o impacto físico imediato em um osciloscópio CRT de bancada em tempo real.

## Contexto
Engenheiros de timbres e guitarristas frequentemente modificam amplificadores reais (*hot-rodding*: troca de EL34 por 6L6GC, ajuste de bias mais quente, mudança do valor do capacitor de cátodo ou conversão de retificador de diodo para válvula GZ34). Em simuladores convencionais esses parâmetros são invisíveis ou apresentados sem feedback visual da curva de transferência.

Fontes da verdade e evidências no repositório:
- `docs/01-SRS-Levantamento-de-Requisitos.md:122-128` — Define **RF-007** (Modo Chassi Aberto dividido em Preamp, Tone Stack, Power Amp e Power Supply), **RF-008** (Customização do Preamp), **RF-009** (Topologia e Posição do Tone Stack), **RF-010** (Power Amp e Transformador), **RF-011** (Fonte de Alimentação e Sag Dinâmico) e **RF-012** (Osciloscópio de Bancada e Curva de Transferência em 3 modos).
- `docs/02-Design-System-Specification.md:97-108` — Especifica os componentes `<ampforge-vacuum-tube>` (clique abre seletor rápido; incandescência responde ao Bias mA e envelope de áudio) e `<ampforge-crt-scope>` (modos `WAVEFORM`, `TRANSFER CURVE` e `TONE STACK & IMPEDANCE`).
- `design-system/index.html:201-355` e `design-system/app.js:31-40,265-433,676-721` — O protótipo já possui a gaveta `#openChassisWorkbench`, 5 válvulas visuais, 6 sliders de trimpot e o `<canvas id="crtCanvas">` com os 3 modos, porém faltam controles exigidos pelos RFs: `12AY7` no preamp (**RF-008**), `Interstage High-Pass Cut` (**RF-008**), posição do Tone Stack `Pre-Distortion` / `Post-Preamp` / `Post-Power Amp` e topologia `Mesa Mark Pre-Gain` (**RF-009**), válvula `6V6`, chave `Class A` vs `Class AB Push-Pull` e `Phase Inverter Drive` (**RF-010**), `B+ Time Constant` e `Ghost Note Filtering` (**RF-011**), além do clique direto na válvula para abrir o seletor rápido (**Design System 4.2**).

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Subsistema Preamp (**RF-008**) | `12AX7A`, `ECC83S`, `12AT7`, `EF86` + `Plate Voltage` + `Triode Asym` | Inclui `12AY7`, `Preamp Tube Bias`, `Triode Hardness` e `Interstage High-Pass Cut (Hz)` |
| Subsistema Tone Stack (**RF-009**) | 4 topologias sem seletor de posição (`design-system/index.html:319-325`) | 5 topologias (`Passive James/Baxandall`, `Fender TMB Mid-Scoop`, `Marshall JCM Cathode Follower`, `Vox Top Boost`, `Mesa Mark Pre-Gain`) + Seletor de Posição (`PRE_DISTORTION`, `POST_PREAMP`, `POST_POWER_AMP`) |
| Subsistema Power Amp (**RF-010**) | `EL34`, `6L6GC`, `KT88`, `EL84` + `Power Bias` + `NFB` + `Xformer Sat` | Inclui `6V6`, seletor `Class A` vs `Class AB Push-Pull`, `Phase Inverter Drive` e `Transformer Hysteresis & Core Size` |
| Subsistema Power Supply (**RF-011**) | `diode`, `gz34`, `5u4gb` + `B+ Supply Sag` | Inclui `B+ Time Constant (ms)` e `Ghost Note Filtering` |
| Interação com `<ampforge-vacuum-tube>` | Apenas efeito `:hover` CSS (`design-system/styles.css:1053`) | Clicar em qualquer válvula V1–V5 foca/abre o seletor de modelo correspondente e atualiza o rótulo cerâmico e a curva |

**Alternativas descartadas:**
1. *Exibir gráficos estáticos pré-renderizados em SVG para o osciloscópio:* descartada porque o objetivo da bancada é refletir matematicamente a combinação exata de todos os trimpots, válvulas e sinal de entrada em tempo real.
2. *Manter o Tone Stack fixo após o Preamp:* descartada porque arquiteturas clássicas como Mesa/Boogie Mark Series posicionam o Tone Stack antes da distorção (`PRE_DISTORTION`), o que muda radicalmente a seletividade de saturação por frequência.

## Comportamento esperado
1. **Abertura da Bancada (RF-007):** Acionar a chave `#btnToggleChassis` alterna o estado `chassisOpen`, revelando ou recolhendo o painel `#openChassisWorkbench` dividido nos 4 subsistemas White-Box e no osciloscópio CRT.
2. **Customização de Preamp (RF-008):**
   - Seleção de válvula de pré entre `12AX7A` ($\mu=100$), `ECC83S` ($\mu=100$, corte assimétrico quente), `12AT7` ($\mu=60$), `12AY7` ($\mu=45$) e `EF86` (pentodo de alto ganho);
   - Ajuste de `Preamp Plate Voltage` (`180 V` a `340 V`), `Preamp Bias / Triode Asymmetry` (`0.00` a `0.60`), `Triode Hardness` (`0.0` a `10.0`) e `Interstage High-Pass Cut` (`20 Hz` a `400 Hz`).
3. **Topologia e Localização do Tone Stack (RF-009):**
   - Seleção de topologia: `PASSIVE_JAMES_BAXANDALL`, `FENDER_TMB_MID_SCOOP`, `MARSHALL_CATHODE_FOLLOWER`, `VOX_TOP_BOOST` e `MESA_MARK_PRE_GAIN`;
   - Seleção de localização no circuito: `PRE_DISTORTION` (antes dos triodos de ganho), `POST_PREAMP` (entre pré e inversora de fase) ou `POST_POWER_AMP`.
4. **Power Amp e Transformador de Saída (RF-010):**
   - Seleção de válvulas de saída entre `EL34`, `6L6GC`, `KT88`, `EL84` e `6V6`;
   - Seleção de classe de operação: `CLASS_A` (assimetria de harmônicos pares, compressão contínua) vs `CLASS_AB_PUSH_PULL` (crossover e harmônicos ímpares em alta potência);
   - Ajuste de `Power Tube Bias` (`20.0 mA` a `55.0 mA`), `Negative Feedback (NFB)` (`0.0` a `10.0`), `Phase Inverter Drive` (`0.0` a `10.0`) e `Transformer Saturation / Hysteresis` (`0.0` a `10.0`).
5. **Fonte de Alimentação e Sag Dinâmico (RF-011):**
   - Seleção de retificador: `SOLID_STATE_DIODE`, `GZ34_5AR4` ou `DUAL_5U4GB`;
   - Ajuste de `Power Supply Sag` (`0.0 dB` a `10.0 dB`), `B+ Time Constant` (`10 ms` a `250 ms`) e `Ghost Note Filtering` (`0%` a `100%`, controlando a modulação de intermodulação de 100/120 Hz da rede nos graves).
6. **Válvulas Interativas e Incandescência Dinâmica:** Clicar em um bulbo `.vacuum-tube` de pré (V1–V3) ou de potência (V4–V5) abre/aciona o seletor rápido de modelo correspondente. A luminosidade `--tube-dynamic-glow` e `--heater-opacity` varia proporcionalmente à corrente de `Power Tube Bias (mA)`, à tensão `Plate Voltage` e ao envelope dinâmico do sinal de entrada.
7. **Osciloscópio CRT de Bancada (RF-012):** O canvas `#crtCanvas` opera em 3 modos selecionáveis:
   - `waveform`: plota no domínio do tempo o sinal DI de entrada (verde tracejado `#00E676`) sobreposto à forma de onda saturada de saída (ciano brilhante `#00E5FF`);
   - `transfer`: plota no plano X-Y a curva de transferência não-linear instantânea (amarelo/âmbar `#FFCA28`), exibindo o deslocamento do ponto de operação de Bias e o joelho de clipping;
   - `tonestack`: plota no domínio da frequência a resposta em dB do Tone Stack selecionado (`#00E676`) sobreposta à curva de impedância reativa acoplada do alto-falante (`#FF9100`).

## Critérios de aceite
- **CA-1:** Acionar `#btnToggleChassis` alterna a visibilidade da bancada `#openChassisWorkbench` (classe `.visible`) e o estado `.open-active` do botão sem interromper o processamento de áudio.
- **CA-2:** O subsistema de Preamp permite selecionar os 5 modelos de válvula (`12AX7A`, `ECC83S`, `12AT7`, `12AY7`, `EF86`) e ajustar `Plate Voltage` (`180–340 V`), `Triode Asymmetry` (`0.00–0.60`), `Triode Hardness` (`0.0–10.0`) e `Interstage High-Pass Cut` (`20–400 Hz`), reduzindo o ganho máximo de pré quando `12AY7` ou `12AT7` substituem `12AX7A`.
- **CA-3:** O subsistema de Tone Stack permite selecionar qualquer uma das 5 topologias (`PASSIVE_JAMES_BAXANDALL`, `FENDER_TMB_MID_SCOOP`, `MARSHALL_CATHODE_FOLLOWER`, `VOX_TOP_BOOST`, `MESA_MARK_PRE_GAIN`) e reposicioná-lo entre `PRE_DISTORTION`, `POST_PREAMP` e `POST_POWER_AMP`, alterando a ordem de execução dos estágios no solver White-Box.
- **CA-4:** O subsistema de Power Amp permite selecionar os 5 tipos de válvula (`EL34`, `6L6GC`, `KT88`, `EL84`, `6V6`), alternar entre `CLASS_A` e `CLASS_AB_PUSH_PULL`, e ajustar `Power Tube Bias` (`20.0–55.0 mA`), `Negative Feedback` (`0.0–10.0`), `Phase Inverter Drive` (`0.0–10.0`) e `Transformer Saturation` (`0.0–10.0`), atualizando as bases cerâmicas `#pwrTubeLabel1`/`#pwrTubeLabel2` e o readout `#plateCurrentReadout`.
- **CA-5:** O subsistema de Power Supply permite escolher entre `SOLID_STATE_DIODE`, `GZ34_5AR4` e `DUAL_5U4GB` e ajustar `Power Supply Sag` (`0.0–10.0 dB`), `B+ Time Constant` (`10–250 ms`) e `Ghost Note Filtering` (`0–100%`), produzindo maior compressão transitória de tensão B+ nas opções valvuladas (`GZ34`/`5U4GB`) do que em `SOLID_STATE_DIODE`.
- **CA-6:** Clicar em uma válvula `.vacuum-tube` na baia abre/foca o seletor rápido de modelo (pré para V1–V3, potência para V4–V5), e o aumento de `biasMa` de `20 mA` para `55 mA` incrementa monotonicamente o raio e a opacidade de `--tube-dynamic-glow` e `--heater-opacity`.
- **CA-7:** No osciloscópio CRT (`#crtCanvas`), alternar entre as abas `waveform`, `transfer` e `tonestack` renderiza a visualização correspondente refletindo em tempo real as mudanças nos parâmetros de circuito e do gabinete acoplado.
- **CA-8:** Caso de erro — fornecer um valor fora de faixa para parâmetros críticos de circuito (ex.: `plateVolts < 180` ou `> 340`, `biasMa < 20` ou `> 55`) faz o controlador rejeitar ou limitar (*clamp*) ao intervalo seguro do modelo físico, impedindo divisão por zero ou divergência (`NaN`/`Infinity`) no solver Newton-Raphson.
- **CA-9:** Não-regressão — fechar a bancada (`chassisOpen = false`) mantém todos os parâmetros modificados no Mod Shop ativos no cálculo de áudio do canal atual.

## Fora de escopo
- A edição dos knobs básicos do Painel Frontal (coberta pela **Spec 003**).
- A edição da posição física dos microfones no cone do alto-falante (coberta pela **Spec 005**).

## Invariantes tocadas
- **RNF-003 (Solver 64-bit Double Precision):** O solver Newton-Raphson de polarização de placa, bias e histerese do transformador opera em 64-bit double para convergência incondicional.
- **RNF-008 (UI 60 FPS):** A renderização do `#crtCanvas` e da incandescência das válvulas roda no loop visual consumindo dados da `SPSC Telemetry Queue`, sem jamais bloquear a thread de áudio.

## Testes que cobrem esta spec
- `tests/contract/004-bancada-chassi-aberto-mod-shop-osciloscopio-crt.test.js` → CA-1, CA-2, CA-3, CA-4, CA-5, CA-6, CA-7, CA-8, CA-9 *(pendente — ciclo TDD red → green)*.

## Notas
- Rastreabilidade SRS: **RF-007**, **RF-008**, **RF-009**, **RF-010**, **RF-011**, **RF-012**, Seção 5 (`circuitModShop`); Design System Seções 4.2 e 4.3.
