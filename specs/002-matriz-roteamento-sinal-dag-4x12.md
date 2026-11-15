---
spec: 002
titulo: Matriz de roteamento de sinal 4x12 (DAG), nós de Split/Mixer e múltiplos I/Os
status: implementada
atualizado: 2026-11-08
---
# 002 — Matriz de roteamento de sinal 4x12 (DAG), nós de Split/Mixer e múltiplos I/Os

## Objetivo
Permitir ao músico ou engenheiro de áudio construir cadeias de sinal seriais e paralelas em uma matriz de 4 linhas por 12 colunas com divisão inteligente de sinal, mixagem com controle de fase/pan e entradas/saídas independentes por linha, impedindo loops de realimentação inválidos.

## Contexto
Rigs profissionais de guitarra e contrabaixo exigem processamento paralelo (ex.: combinar um amplificador Brit 800 saturado na Linha 1 com um US Twin limpo na Linha 2, ou separar graves e agudos por crossover em contrabaixos e guitarras de 7/8 cordas) e saídas independentes (ex.: enviar XLR com simulação de gabinete para o PA enquanto envia saída P10 sem gabinete para um amplificador real no palco).

Fontes da verdade e evidências no repositório:
- `docs/01-SRS-Levantamento-de-Requisitos.md:82-102` — Define a topologia do Grid como um **Grafo Acíclico Dirigido (DAG)** de **4 linhas por 12 colunas** (48 slots endereçáveis).
- `docs/01-SRS-Levantamento-de-Requisitos.md:107-116` — Define **RF-001** (Matriz 4×12), **RF-002** (Nós de Split: Balance/A-B, Crossover em Hz, Dynamic Split), **RF-003** (Nós de Mixer: Nível $-\infty$ a $+12\text{ dB}$, Pan L100..R100, Polaridade $0^\circ/180^\circ$, Master), **RF-004** (Múltiplas entradas e saídas por linha) e **RF-005** (Prevenção de loops no DAG).
- `design-system/index.html:114-140` e `design-system/app.js:43-58,87-159` — O protótipo atual renderiza apenas 8 colunas por linha (`for (let c = 0; c < 8; c++)` em `design-system/app.js:103`), possui textos de I/O fixos (`rowInputs` e `rowOutputs` em `app.js:91-92`) e representa o Split apenas como um card estático sem edição dos modos Balance/Crossover/Dynamic nem validador de DAG.

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Dimensão da Matriz | 4 linhas × 8 colunas visíveis (`design-system/app.js:103`) | 4 linhas × 12 colunas completas (48 slots endereçáveis de `col: 0` a `col: 11`) (**RF-001**) |
| Nó de Split | Bloco estático `blk_split` (`design-system/app.js:46`) | Criação automática ao ramificar linha, com 3 modos: `Balance/A-B`, `Crossover (Hz)` e `Dynamic Split` (**RF-002**) |
| Nó de Mixer / Merger | Apenas rótulo `MIX MERGE →` (`design-system/app.js:92`) | Nó ativo na convergência com Ganho ($-\infty$ a $+12\text{ dB}$), Pan (`L100` a `R100`), Fase ($0^\circ/180^\circ$) por ramo e Nível Mestre (**RF-003**) |
| Roteamento de I/O por linha | Rótulos estáticos (`design-system/app.js:91-92`) | Seletor configurável por linha: 4 origens de entrada e 4 destinos de saída (**RF-004**) |
| Validação de Ciclos (DAG) | Inexistente | Validador topológico que bloqueia conexões cíclicas e sinaliza visualmente a rota inválida (**RF-005**) |

**Alternativas descartadas:**
1. *Roteamento livre por cabos ponto-a-ponto (patchbay aberto sem grade):* descartada porque inviabiliza a navegação tátil rápida no palco e a representação compacta na Zona 1 do Design System.
2. *Permitir loops de feedback arbitrários entre colunas anteriores:* descartada porque introduz atraso implícito de 1 buffer ou instabilidade numérica severa; qualquer realimentação deve ocorrer dentro de blocos dedicados (ex.: Delay Feedback).

## Comportamento esperado
1. **Matriz 4×12 Endereçável (RF-001):** O Grid disponibiliza exatamente 4 linhas (`row: 0..3`) e 12 colunas (`col: 0..11`), totalizando 48 células. Cada célula pode estar vazia, conter um conector passivo de passagem (`SHUNT`) ou hospedar uma instância de bloco de processamento.
2. **Divisão de Sinal — Split Node (RF-002):** Quando o usuário conecta a saída de uma célula na linha $r_1$ para uma célula na linha $r_2 \neq r_1$, o sistema instancia automaticamente um nó **Split** operando em um de 3 modos selecionáveis:
   - `BALANCE_AB`: controla a proporção de nível/pan entre os ramos A e B;
   - `CROSSOVER`: divide o sinal por frequência de corte ajustável em Hz (`40 Hz` a `4000 Hz`, Linkwitz-Riley), enviando graves para um ramo e agudos para o outro;
   - `DYNAMIC_SPLIT`: roteia o sinal entre os ramos A e B em função do envelope dinâmico da palhetada (threshold em dBFS, attack e release).
3. **Convergência de Sinal — Mixer / Merger Node (RF-003):** Quando duas ou mais linhas convergem em uma coluna posterior, o nó **Mixer** aplica para cada caminho de entrada: ganho individual de `$-\infty\text{ dB}$` (mute) a `$+12,0\text{ dB}$`, panorâmica de `L100` (`-1.0`) a `R100` (`+1.0`), chave de inversão de fase/polaridade (`0°` normal ou `180°` invertido) e controle de `Master Level` na saída combinada.
4. **Terminais de Entrada e Saída Independentes por Linha (RF-004):**
   - Cada uma das 4 linhas permite selecionar sua fonte de entrada entre: `Instrument In 1`, `In 2 / Aux`, `FX Return L/R` e `USB Reamp In 3/4`.
   - Cada uma das 4 linhas permite selecionar seu destino de saída entre: `Main Out XLR L/R`, `Amp Out P10 (No Cab)`, `FX Send` e `USB Record Out`.
5. **Prevenção de Loops de Realimentação (RF-005):** Antes de consolidar qualquer conexão entre células, o validador executa ordenação topológica do DAG. Qualquer aresta que aponte para a mesma coluna ou coluna anterior criando um ciclo fechado é rejeitada, mantendo o grafo anterior intacto e destacando a tentativa com estado visual `.invalid-route`.

## Critérios de aceite
- **CA-1:** A matriz do Grid inicializa e serializa exatamente `4` linhas (`0..3`) por `12` colunas (`0..11`), permitindo alocar, mover, substituir por `SHUNT` ou remover blocos em qualquer uma das `48` coordenadas `(row, col)`.
- **CA-2:** Ao criar uma ramificação de uma linha para outra, um nó `SPLIT` é criado automaticamente com suporte aos modos `BALANCE_AB`, `CROSSOVER` e `DYNAMIC_SPLIT`.
- **CA-3:** No modo `CROSSOVER` do nó `SPLIT`, a soma das saídas dos ramos passa-baixas e passa-altas reconstrói a magnitude plana do sinal de entrada na frequência de corte configurada (dentro de `±0,2 dB`), separando graves e agudos conforme a frequência em Hz.
- **CA-4:** No modo `DYNAMIC_SPLIT`, sinais com envelope RMS abaixo do `threshold` são direcionados prioritariamente ao ramo A e transientes acima do `threshold` são roteados para o ramo B respeitando os tempos de ataque e liberação.
- **CA-5:** Na convergência de duas ou mais linhas, o nó `MIXER` permite ajustar individualmente para cada linha convergente o Nível (`-Infinity` a `+12.0 dB`), Pan (`-1.0` / `L100` a `+1.0` / `R100`), Fase (`0°` ou `180°`) e o `Master Level`.
- **CA-6:** Quando dois caminhos paralelos transportam o mesmo sinal em nível `0.0 dB` e um deles tem a chave de Fase invertida para `180°` no nó `MIXER`, a soma resultante apresenta cancelamento de fase completo (`< -90 dBFS`).
- **CA-7:** Cada uma das 4 linhas permite configurar independentemente sua entrada (`INST_IN_1`, `AUX_IN_2`, `FX_RETURN_LR`, `USB_REAMP_3_4`) e sua saída (`MAIN_XLR_LR`, `AMP_OUT_P10_NO_CAB`, `FX_SEND`, `USB_REC_OUT`).
- **CA-8:** Quando uma saída está configurada como `AMP_OUT_P10_NO_CAB`, o sinal entregue a esse barramento ignora automaticamente a convolução de gabinete (`CAB`) subsequente naquela derivação para conexão em etapa de potência/caixa real de palco.
- **CA-9:** Caso de erro — a tentativa de conectar a saída de um slot `(row_a, col_a)` à entrada de um slot `(row_b, col_b)` que forme um ciclo no grafo (ou onde `col_b <= col_a` gerando realimentação) é recusada com erro `DagCycleError`, a rota recebe a classe visual de erro e o ponteiro de grafo na thread de áudio não é alterado.
- **CA-10:** Não-regressão — a ativação/bypass (duplo clique) ou seleção (clique simples) de um bloco em uma linha paralela não altera o estado de bypass, canal ativo nem parâmetros dos blocos nas demais linhas.

## Fora de escopo
- A implementação interna de cada algoritmo de efeito ou amplificador hospedado nos slots (coberta pelas **Specs 003 a 008**).
- A transição com crossfade entre Scenes (coberta pela **Spec 009**).

## Invariantes tocadas
- **RNF-005 (Real-Time Safe):** A validação do DAG (CA-9) e a construção da ordem topológica ocorrem inteiramente na Main Thread; a thread de áudio recebe apenas o grafo já linearizado via `Atomic Graph Pointer Swap` (**Spec 001**).
- **Hierarquia em Três Zonas (Design System Seção 1.2):** A Zona 1 (`Top Signal Grid`) exibe os 48 slots, cabos luminosos `.active-signal-path` e badges de canal sem sobrepor o Palco Central.

## Testes que cobrem esta spec
- `crates/ampforge-dsp/tests/contract_002_matriz_dag.rs` → CA-1 a CA-10 (10 testes unitários de contrato em Rust, 100% ok).
- `tests/contract/002-matriz-roteamento-sinal-dag-4x12.test.js` → Suíte de integração e validação de critérios CA-1 a CA-10.

## Notas
- Rastreabilidade SRS: Seção 2.2, **RF-001**, **RF-002**, **RF-003**, **RF-004**, **RF-005**.
- **2026-11-14 (Auditoria Spec 044):** `DagGraph` e `GridMatrix4x12` existem como biblioteca testada, enquanto `RealtimeDspChain` ainda processa cadeia fixa em produção. Ligação real do grafo dinâmico retomada pela **Spec 045** e execução ao vivo da topologia 4×12 em áudio real fechada pela **Spec 049** (CA-23..CA-28).
