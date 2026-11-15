# Handoff — Specs 034–048: correções do review manual (para o Antigravity)

> Origem: review manual das Specs 001–033 e code review do commit `e1ffff3`, feitos em 2026-11-14.
> As 15 specs (034–048) foram **aprovadas pelo usuário em 2026-11-14**, quando ele liberou a implementação.
> Substitui `ANTIGRAVITY-033-desktop-tauri.md` como trabalho em andamento. A Spec 033 consta como implementada, mas o review mostrou que vários CAs dela não se sustentam; a Spec 044 corrige isso.

## 0. Leia antes, nesta ordem
1. `AGENTS.md`: SDD + TDD estrito e as regras promovidas do Cortex.
2. `CLAUDE.md`: comandos de build e teste, arquitetura e fluxo de um controle.
3. `specs/README.md`: o índice e a seção **"Review manual de 2026-11-14"**, com a ordem de execução.
4. A spec da fase atual, inteira (Contexto com as evidências `arquivo:linha`, CAs, Fora de escopo e Invariantes).
5. `.agents/skills/sonix-pro-ampforge-expert/SKILL.md` para as fases de DSP (038–043).
6. Cortex: faça `query` por `rule`, `gotcha`, `spec-033`, `realtime`, `tauri` e pelo número da spec da fase.

## 1. Regras inegociáveis
Estas regras continuam valendo como no handoff da 033:
- **TDD estrito.** Escreva o teste do `CA-n` primeiro e **mostre que ele falha** (red); só depois implemente (green). O nome do teste cita o CA: `ca3_periodo_lido_do_stream` em Rust, `'CA-3: ...'` em JS. O arquivo segue o padrão `contract_0NN_*.rs` ou `tests/contract/0NN-*.test.js`.
- **Callback de áudio:** zero alocação, zero desalocação, zero lock, zero I/O e zero log. A partir da 037, isso passa a ser **medido** com um alocador global de contagem; não vale mais só ler o código.
- **Nenhum controle decorativo.** Controle sem DSP aparece `disabled`, com a dica "em implementação — Spec 0NN", e conta como pendência.
- **Fidelidade.** Não invente valores de componentes. Os CAs que pedem esquema (039 CA-5/6, 040 CA-4, 043 CA-2) exigem citar a fonte e a revisão. Sem fonte, pare e relate; não chute.
- **Não reescreva teste antigo para passar.** Um CA de spec anterior substituído por esta leva vai para as **Notas** da spec nova como "revertido", com o motivo.
- **Comportamento novo fora da spec:** edite a spec primeiro, registrando a mudança.
- **Git:** **nunca** rode `commit`, `push`, `rebase` ou `reset` sem autorização explícita. Deixe as mudanças no working tree.
- **Medições:** não invente números de latência, CPU ou precisão. Use o valor medido ou `[medida pendente]`.
- **Versões:** antes de adotar um crate ou pacote novo (`getrandom`, FFT, SSIM, etc.), confira-o no registro e prefira versões publicadas há pelo menos duas semanas. Registre a escolha no Cortex.

## 2. Estado do working tree ao receber
Nada foi commitado. O working tree contém:
- `CLAUDE.md` (novo);
- `specs/034–048` (novas, `aprovada`) e `specs/README.md` (índice e ordem);
- `docs/ui-desktop/referencia/*.png`: os 10 boards do design renderizados (Spec 048 CA-1, já cumprido);
- `docs/ui-desktop/canvas/canvas.json`, atualizado com a versão do artefato (Amps 1214×2140, Pedais 1446×1451).

## 3. Fases e gates
Siga a ordem do `specs/README.md`. Cada fase termina com `cargo test --workspace` e `npm test` verdes e com um relatório no formato da §5. Não comece a fase seguinte com a anterior vermelha.

### Fase 1 — 034 + 037: ponte de controle e tempo real (base de tudo)
- **034:**
  - parâmetro descartado quando o `try_lock` falha;
  - ação one-shot duplicada (`looper.action`);
  - período e taxa de amostragem lidos do config pedido em vez do stream aberto;
  - callback maior que 4096 frames;
  - fila com vários produtores;
  - silêncio sob contenção;
  - `set_audio_device` decorativo;
  - `dly_1.mode` inicial inconsistente.
- **037:**
  - alocações e desalocações no callback;
  - IR antiga liberada na thread de áudio;
  - buraco ao carregar IR ou preset;
  - `Mutex` no osciloscópio.
- Comece pelo **037 CA-7**, que cria o alocador de contagem sobre `RealtimeDspChain::process`. Esse alocador é a régua que as fases seguintes vão usar.
- **Gate:** 034 CA-1…11 e 037 CA-1…8 em red → green.

### Fase 2 — 046 + 044: segurança e honestidade do SDD
- **046:**
  - Host e Origin com **igualdade exata**;
  - token de 128 bits do sistema operacional;
  - id de preset com regex e checagem de que o caminho fica dentro da pasta;
  - caminhos de `.sforgepkg` sem `C:`, UNC ou verbatim;
  - manifesto conferido contra os arquivos.
- **044:**
  - índice com `implementada (CA-… revertida)`;
  - teste de itens `pub` não usados;
  - `HalfBandOversampler` e `CpuMonitor` reais ou removidos;
  - inventário de controles com o caminho do teste;
  - a Spec 012 e o README deixam de prometer VST3/CLAP.
- **Atenção:** o 044 CA-1 altera estados no índice. Altere só o que a tabela da spec manda.

### Fase 3 — 036 + 035: host real e contratos UI↔backend
- **036:**
  - dispositivos enumerados de verdade;
  - status lido do stream;
  - latência ida e volta `null` até ser medida, com o rótulo "estimada";
  - underruns contados;
  - modos L/R/Sum;
  - ASIO e exclusivo desabilitados quando o backend não suporta;
  - MIDI real.
- **035:**
  - arraste do knob não brigar com a telemetria;
  - nomes dos campos de telemetria;
  - osciloscópio usando os dados do backend;
  - presets e dispositivos vindos do backend.
- O **035 CA-8** gera fixtures JSON no Rust e as consome no JS. Isso trava o contrato dos dois lados; faça cedo.

### Fase 4 — 048: UI fiel ao design
- **Régua:** `docs/ui-desktop/referencia/*.png`. A fonte da verdade são os `docs/ui-desktop/canvas/*.dc.html`; a lógica dos modelos está em `renderVals()`.
- Ordem recomendada: Main → Palco → Chassi → Gabinete → Áudio. Feche o CA-10 (SSIM) de cada tela antes de passar à próxima.
- Fontes **locais** (Inter, Oswald, JetBrains Mono em `ui/public/fonts`). O design carrega as fontes do Google Fonts; o app não pode, porque o release é offline e não usa HTTP.
- O CA-5 depende do **047 CA-6** (válvulas do modelo). Até lá, mostre o complemento estático do modelo com a dica "Spec 047".
- Para re-renderizar a referência, é preciso o `support.js` (runtime do canvas), que **não está no repo**: peça ao usuário. Não gere PNG novo sem pedido.
- Mantenha a 033 sem regressão (CA-12): sem scroll em 1280×800 e 1920×1080, e o inventário completo.

### Fase 5 — comportamento dos equipamentos: 045, 038, 039, 041, 040, 047, 043
| Spec | Foco | Atenção |
|---|---|---|
| 045 | Cenas audíveis, troca sem buraco, salvar com cenas e canais, grafo real | CA-6 é atômico: preset inválido não muda nada |
| 038 | Convolução particionada (latência zero no CA-2: cabeça em FIR direto + partições FFT), modelo de microfone, ressonância, reamostragem da IR | CA-3 é benchmark medido; registre o número |
| 039 | Troca de modelo sem estalo, coeficientes por taxa, master fantasma no Plexi, pad `low2`, stacks AB763 e Top Boost pelo esquema, `voicing-fontes.md` | **Cite o esquema e a revisão** antes de codar CA-5/6 |
| 041 | All-Buttons-In monotônico, topologias distintas, expander, gate pela DI no Natural, anti-aliasing, TS808 e RAT pelo circuito | |
| 040 | Seletor `mod_2.model`, tremolo, flanger, phaser e BBD distintos, reverse, tempo sem degrau, ducking, RT60, Plate e Spring | CA-11: implementar o Shimmer de verdade **ou** removê-lo do enum, do registro e da UI |
| 047 | Transferência, forma de onda, tone stack e impedância vindos do motor; válvulas e B+ por modelo | Rings lock-free lidos pela thread de streams |
| 043 | Afinador polifônico, temperamento com fonte, precisão publicada, looper (meia velocidade, undo, 60 s a 96 kHz) | |

### Fase 6 — 042: bloco de pitch
- Decisão do usuário: **implementar** o bloco, sem remover a promessa.
- Monofônico de baixa latência (PSOLA, granular ou domínio da frequência), com a latência declarada e medida (CA-4).
- Harmonizador diatônico por grau da escala.
- Inclua `pitch_1.*` na tabela única, no registro e no teste de efeito.
- O pitch polifônico está fora de escopo.

## 4. Fechamento de cada spec
1. Status `implementada`, ou `implementada (CA-n revertida)` com o motivo.
2. Seção "Testes que cobrem esta spec" preenchida no formato `arquivo → CA-n`.
3. Linha do índice atualizada em `specs/README.md`.
4. `docs/ui-desktop/inventario-controles.md` atualizado quando a spec mexer em controles.
5. No `AGENTS.md`, mova o bloco "Trabalho em andamento" para a próxima fase.

## 5. Relatório de cada fase (formato)
- O que foi feito, com os arquivos.
- CAs cobertos, com a saída do teste **falhando antes** e **passando depois**.
- CAs revertidos ou substituídos de specs antigas, com o motivo.
- Pendências e dúvidas de fidelidade, com a fonte consultada.
- Comando para reproduzir.
- "Nada commitado (aguardando autorização)."

## 6. Cortex
Projeto: `simulador-de-efeitos`. Ao fim de cada fase:
- faça `capture` de decisões, gotchas e fatos;
- rode `consolidate` e `write_page`;
- rode `promote_rules` se surgir regra nova.

Ao começar, rode `handoff` com `target_agent: "antigravity"` e leia as páginas registradas em 2026-11-14:
- `20261114-131936-review-26f3` (decision): o review, as Specs 034–048, os achados por spec e a ordem de implementação.
- `20261114-131934-spec-042-a1f0` (decision): implementar o pitch.
- `20261114-131934-regra-d272` (rule, promovida no `AGENTS.md`): invariante só conta como verificada por régua medida (alocador de contagem e SSIM).
- `20261114-131935-server-9197` (gotcha): Host/Origin por prefixo no dev-http.
- `20261114-131933-referencia-3a73` (fact): PNGs de referência do design; o `support.js` não está versionado.
