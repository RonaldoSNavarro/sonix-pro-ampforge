---
spec: 008
titulo: Afinador estroboscópico/polifônico de alta precisão e Looper de performance
status: implementada
atualizado: 2026-11-08
---
# 008 — Afinador estroboscópico/polifônico de alta precisão e Looper de performance

## Objetivo
Permitir ao músico afinar o instrumento no palco com precisão estroboscópica de ±0,1 cent (ou visualizar todas as 6 cordas simultaneamente) e gravar/sobrepor loops estéreo no início da cadeia (para esculpir timbres de amplificador com as mãos livres) ou no final da cadeia (para performance ao vivo).

## Contexto
Durante um show ou sessão de ajustes na bancada **AmpForge™**, o guitarrista precisa: (1) de um afinador imediato com opção de silenciar a saída (`Mute`) e suporte a temperamentos compensados (*Sweetened Guitar*), e (2) de um Looper capaz de ser posicionado **antes** do amplificador (`PRE` — reproduzindo um riff DI gravado enquanto o músico ajusta válvulas, bias e microfones com as duas mãos livres) ou **depois** de toda a cadeia (`POST` — acumulando camadas com timbres diferentes de cenas diferentes).

Fontes da verdade e evidências no repositório:
- `docs/01-SRS-Levantamento-de-Requisitos.md:151-153` — Define **RF-026** (Afinador Estroboscópico ±0,1 cent e Polifônico de 6 cordas, referência A4 `430–450 Hz`, temperamentos compensados `Sweetened Guitar` e modo `Mute` ou `Bypass`) e **RF-027** (Looper de Performance Integrado posicionável no início ou no final da cadeia, com `Record`, `Play`, `Overdub`, `Undo/Redo`, `Reverse`, `Half-Speed` e indicador circular de compasso).
- `design-system/index.html:74-76,89-97` e `design-system/app.js:748-754` — O protótipo possui o botão `#btnToggleTuner` e a barra retrátil `#tunerBar` com texto estático `E2 (-0.1¢)`, sem detecção real de frequência fundamental, sem ajuste de referência A4 (`430–450 Hz`), sem modo `Mute/Bypass`, sem acionamento por *long-press* em footswitch e sem interface/motor do Looper (**RF-027**).

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Acionamento do Afinador (**RF-026**) | Clique no botão `#btnToggleTuner` (`design-system/app.js:749`) | Clique em `#btnToggleTuner` **ou** segurar (*long-press* $\ge 500\text{ ms}$) o Footswitch dedicado |
| Precisão e Modos do Afinador (**RF-026**) | Texto estático `E2 (-0.1¢)` (`design-system/index.html:96`) | Detector de pitch Strobe com precisão de `±0.1 cent` + visão Polifônica das 6 cordas (`E2 A2 D3 G3 B3 E4`), ajuste `A4 = 430.0..450.0 Hz`, temperamento `EQUAL` vs `SWEETENED_GUITAR` e chave `MUTE` vs `BYPASS` |
| Posicionamento do Looper (**RF-027**) | Inexistente no protótipo | Posição comutável `PRE_CHAIN` (início do Grid, antes do Amp) vs `POST_CHAIN` (final do Grid, antes do Master Out) |
| Operações do Looper (**RF-027**) | Inexistente no protótipo | Máquina de estados `Record`, `Play`, `Overdub`, `Stop`, `Undo/Redo` (última camada), `Reverse`, `Half-Speed` (0,5x oitava abaixo) e indicador circular de progresso do compasso (`0%` a `100%`) |

**Alternativas descartadas:**
1. *Detector de afinação apenas por cruzamento por zero (zero-crossing):* descartada porque falha com harmônicos fortes de guitarra/baixo e não atinge a precisão de `±0.1 cent`; utiliza-se autocorrelação normalizada (YIN/MPM) combinada com rastreamento de fase estroboscópico para o modo Strobe e banco de filtros FFT de alta resolução para o modo Polifônico.
2. *Looper restrito apenas ao final da cadeia:* descartada porque viola explicitamente o **RF-027**, que exige o posicionamento no início da cadeia para permitir testar ajustes da bancada White-Box com as mãos livres.

## Comportamento esperado
1. **Afinador Estroboscópico e Polifônico (`TUNER` — RF-026):**
   - Pode ser aberto/fechado clicando em `#btnToggleTuner` no cabeçalho ou mantendo pressionado (*hold* $\ge 500\text{ ms}$) o footswitch de Tuner/Tap.
   - **Visão Strobe:** Calcula a nota mais próxima e o desvio em *cents* (`-50.0¢` a `+50.0¢`) com resolução e exatidão de `±0.1 cent`, desplazando a velocidade/direção da faixa `#tunerBar .strobe-track` proporcionalmente ao erro de fase.
   - **Visão Polifônica:** Quando múltiplas cordas são tocadas abertas simultaneamente, analisa e exibe o desvio individual das 6 cordas (`6:E2`, `5:A2`, `4:D3`, `3:G3`, `2:B3`, `1:E4`).
   - **Calibração e Temperamento:** Permite ajustar a frequência de referência `A4` entre `430.0 Hz` e `450.0 Hz` (padrão `440.0 Hz`) e alternar entre temperamento igual (`EQUAL_TEMPERAMENT`) e temperamento compensado para guitarra (`SWEETENED_GUITAR`, aplicando offsets específicos em cents por corda).
   - **Roteamento de Áudio do Afinador:** No modo `MUTE` (padrão de palco), enquanto o afinador está visível, a saída principal é silenciada (`-Infinity dB`); no modo `BYPASS`, o áudio continua soando normalmente enquanto afina.
2. **Looper de Performance Integrado (`LOOPER` — RF-027):**
   - **Seletor de Posição (`PRE` vs `POST`):** Quando em `PRE`, injeta o loop gravado na entrada do Grid 4×12 para que qualquer alteração posterior nos blocos (`AMP`, `CAB`, `Mod Shop`) processe o loop ao vivo; quando em `POST`, grava e reproduz o sinal estéreo já processado no final da cadeia.
   - **Controles de Transporte e Efeitos:**
     - `Record`: inicia a captura da camada base no buffer estéreo pré-alocado;
     - `Play`: fecha o loop na duração gravada e inicia a reprodução cíclica contínua;
     - `Overdub`: soma novas camadas em um buffer de *undo* separado permitindo desfazer;
     - `Undo / Redo`: remove ou restaura a última camada de *overdub* sem interromper a reprodução da base;
     - `Reverse`: inverte a direção do ponteiro de leitura do buffer do loop;
     - `Half-Speed`: reduz a taxa de leitura pela metade (`0.5x`), dobrando a duração do ciclo e baixando a afinação em 1 oitava (`-12 semitons`);
     - **Indicador Circular de Compasso:** Informa continuamente o ângulo/percentual da posição atual do cabeçote de reprodução (`0.0%` a `100.0%`).

## Critérios de aceite
- **CA-1:** O afinador é ativado/desativado tanto pelo clique em `#btnToggleTuner` quanto por um evento de *long-press* ($\ge 500\text{ ms}$) no footswitch dedicado, exibindo a barra `#tunerBar` com a classe `.visible`.
- **CA-2:** Diante de uma senóide pura de entrada em `440.0 Hz` (com referência `A4 = 440.0 Hz`), o afinador identifica a nota `A4` com erro inferior ou igual a `±0.1 cent` (`0.0¢`); para `440.254 Hz` (+1,0 cent), reporta `A4 (+1.0¢)` dentro da tolerância de `±0.1 cent`.
- **CA-3:** O afinador permite ajustar a referência `A4` continuamente entre `430.0 Hz` e `450.0 Hz`, alternar entre visão `STROBE` e `POLYPHONIC` (exibindo o status das 6 cordas) e ativar o temperamento `SWEETENED_GUITAR`.
- **CA-4:** Quando o afinador está ativo com `outputMode = 'MUTE'`, o sinal de saída principal é atenuado a zero (`0.0` linear / `-Infinity dBFS`) com rampa suave de 5 ms; com `outputMode = 'BYPASS'`, o sinal de saída permanece inalterado.
- **CA-5:** O Looper executa as transições de estado `IDLE → RECORD → PLAY ↔ OVERDUB → STOP`, suporta `Undo` e `Redo` da última camada de *overdub* e atualiza o indicador circular de progresso de `0%` a `100%` a cada ciclo.
- **CA-6:** Ativar `Reverse` no Looper faz o cabeçote de leitura percorrer o buffer do fim para o início; ativar `Half-Speed` lê o buffer com incremento de `0.5` amostra por ciclo (interpolado), dobrando o período do loop e reduzindo a frequência em 1 oitava.
- **CA-7:** Quando o Looper está posicionado em `PRE_CHAIN`, alterar parâmetros do bloco `AMP` ou trocar de Scene modifica em tempo real o timbre do áudio já gravado no Looper; quando posicionado em `POST_CHAIN`, o timbre gravado na camada anterior permanece fixo mesmo após trocar de Scene.
- **CA-8:** Caso de erro — tentar ajustar `A4` fora de `[430.0, 450.0] Hz` é recusado/limitado ao intervalo válido, e se a gravação do Looper atingir o limite máximo do buffer pré-alocado, o Looper comuta automaticamente de `RECORD` para `PLAY` (ciclo fechado) sem lançar exceção nem alocar memória na thread de áudio.
- **CA-9:** Não-regressão — com o Afinador fechado e o Looper em estado `IDLE`, ambos adicionam `0 samples` de latência e não alteram o sinal que atravessa o Grid.

## Fora de escopo
- A troca de Scenes e modos gerais dos 8 footswitches (coberta pela **Spec 009**).

## Invariantes tocadas
- **RNF-005 (Real-Time Safe):** Os buffers de gravação e *undo* do Looper são pré-alocados na inicialização; nenhuma alocação dinâmica ocorre ao pressionar `Record` ou `Overdub`.
- **RNF-006 (Zipper-Noise Free):** O silenciamento do modo `MUTE` do afinador e os pontos de loop (`loop seam`) aplicam micro-crossfade suave (2–5 ms) para evitar cliques no ponto de repetição.

## Testes que cobrem esta spec
- `crates/ampforge-dsp/tests/contract_008_tuner_looper.rs` → CA-1 a CA-9 (9 testes unitários de contrato em Rust, 100% ok).
- `tests/contract/008-afinador-estroboscopico-polifonico-looper.test.js` → Suíte de integração e validação de critérios CA-1 a CA-9.

## Notas
- Rastreabilidade SRS: **RF-026**, **RF-027**; Design System Seção 2.1 (`--surface-oled-screen`) e Seção 5.
- **2026-11-14 (Auditoria Spec 044):** Modos polifônico, temperamento `SWEETENED_GUITAR`, half-speed/reverse e capacidade por taxa do looper permanecem parciais na cadeia ao vivo; retomado pela **Spec 043**.
