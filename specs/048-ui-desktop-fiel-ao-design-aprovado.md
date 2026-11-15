---
spec: 048
titulo: A UI implementada na Spec 033 não se parece com o design aprovado em docs/ui-desktop/canvas
status: implementada
atualizado: 2026-11-14
---
# 048 — UI desktop fiel ao design aprovado (canvas `docs/ui-desktop/canvas`)

## Objetivo
As telas do app desktop reproduzem o layout, a hierarquia e o acabamento do design aprovado, com uma régua automática que reprova a divergência.

## Contexto
Relato do usuário (2026-11-14): "na implementação de ontem a ui não ficou nada parecida com o artefato que geramos no design".

**Causa raiz:** a Spec 033 tem CAs de inventário, de IPC e de ausência de scroll, mas **nenhum CA de fidelidade visual**. Uma tela com qualquer layout passa em todos. A spec cita o canvas como "design de referência", e o canvas `.dc.html` depende de `support.js`, que não está no repositório. Ou seja, nem dá para renderizar a referência a partir do repo.

Comparação estrutural (2026-11-14), feita por leitura do canvas e dos arquivos de `apps/sonix-app/ui/src`:

| Região do design (`Main.dc.html`) | Design | Implementado |
|---|---|---|
| Barra de título | "SONIX PRO — AMPFORGE · Setlist › Preset ●" | "SONIX PRO // AMPFORGE™ DESKTOP" (`main.ts:37-40`) |
| Barra de menu | Menus do app + indicador "ponte UI→DSP" | Abas de tela (`main.ts:72-87`) |
| Toolbar de hardware | 68 px em metal escovado com 4 parafusos; botões ‹ › 30×44; LCD âmbar 330×46 em 2 linhas (setlist/nº/cena · nome do preset 15 px); botão SALVAR laranja; 8 cenas 32×30 em Oswald; medidores de 20 segmentos; leitura de CPU/OS; TAP com BPM; AFINADOR, PALCO e engrenagem | Chips pequenos, um `<select>` de preset que não existe no design, LCD de 1 linha "SCENE x MODELO", medidores de 16 segmentos, OS como `select`, sem CPU, sem AFINADOR, sem engrenagem (`Estudio.ts:301-397`) |
| Matriz de sinal | Cabeçalho "Matriz de sinal" com dica e chips de OS; grid de 11 colunas × 2 linhas (52/46 px); linha B sempre presente | Fila flex com cabos; linha B só com split ligado e deslocada 200 px (`Estudio.ts:399-427`) |
| Palco do rig | AmpHead (1000 px) escalado para 788×226 + painel de válvulas do modelo (vidro, getter, placa, brilho); Pedalboard; "LINHA B / PÓS"; "Rack · loop / pós-amp" | Grid com AmpHead + caixa de 4 "12AX7" fixas; pedais e racks empilhados com `overflow-y:auto` (`Estudio.ts:434-510`) |
| Inspetor | Canal A–D; "Parâmetros" como trilhos horizontais (`m-track`); seção "Modifier" | Cartão genérico sem Modifier (`Estudio.ts:512-539`) |
| Barra de status | 26 px: LED do motor, backend·dispositivo, taxa, período, ida-e-volta, xruns, gate, entrada calibrada/pico | 2 textos genéricos (`main.ts:106-111, 201-211`) |

As demais telas seguem o mesmo padrão: o design tem ilustração, e a implementação tem cartões genéricos (`makePanelCard`).
- **Chassi:** o design desenha o chassi aberto, com turret board, transformador de força, capacitores de filtro, válvulas e transformador de saída + choke; o implementado é cartão + fila de válvulas.
- **Gabinete:** o design desenha o falante (baffle, carcaça, surround, cone, anéis, dust cap) com os microfones posicionados; o implementado é um `div` de cone + cartões.
- **Palco:** o design tem o cabeçalho OLED de 128 px com o nome do preset em 48 px, o painel de Tempo e os botões Sair/Afinador, e o dock de footswitches de 250 px, com 8 scribble strips de 58 px e chaves redondas de 90 px com anel colorido; o implementado são 3 cartões.
- **Áudio:** o design tem 3 seções (Dispositivo & latência, Entrada & calibração, Desempenho); o implementado tem 6 cartões.

## Comportamento esperado
- O repositório guarda a **referência renderizada** de cada board do canvas: PNG 1440×900 (componentes no tamanho do board) em `docs/ui-desktop/referencia/`, exportada do ambiente de design, com a data.
- Cada tela do app reproduz as regiões, dimensões, tipografia, cores e materiais do board correspondente, ligando os dados reais do motor (sem os placeholders do design, como "[medida] ms").
- A navegação segue o design: menus do app na barra de menu, botões de hardware (AFINADOR, PALCO, engrenagem) e links nas telas, mantendo os atalhos da Spec 033.
- Um teste de regressão visual compara cada tela com a referência e reprova a divergência.

## Critérios de aceite
- **CA-1:** `docs/ui-desktop/referencia/` contém um PNG para cada board de `canvas.json` (Main, Chassi, Gabinete, Áudio, Palco, Amps, Pedais, AmpHead, Pedal, Rack), com as dimensões do board.
- **CA-2:** A barra de título mostra "SONIX PRO — AMPFORGE · {setlist} › {preset}"; a barra de menu tem os menus do app e o indicador "ponte UI→DSP" com a cor do estado da ponte.
- **CA-3:** A toolbar da tela principal tem 68 px de altura e contém, na ordem do design: ‹ (30×44), LCD (330×46, 2 linhas), › , SALVAR, 8 cenas (32×30), medidores IN/OUT de 20 segmentos, CPU/OS, TAP com BPM, AFINADOR, PALCO e engrenagem. Não há `<select>` na toolbar (teste DOM).
- **CA-4:** A matriz de sinal é um grid de 11 colunas × 2 linhas (52 px e 46 px); a linha B fica sempre visível (opacidade reduzida com o split desligado), e o cabeçalho tem "Matriz de sinal" e os chips de oversampling.
- **CA-5:** O palco tem o AmpHead em 788×226 e o painel de válvulas com o complemento do modelo atual (Spec 047 CA-6). Pedalboard, "LINHA B / PÓS" e "Rack · loop / pós-amp" ficam na disposição do design **sem rolagem** em 1440×900.
- **CA-6:** O inspetor tem as abas de canal A–D, os parâmetros como trilhos horizontais com rótulo e a seção "Modifier · {alvo}".
- **CA-7:** A barra de status tem 26 px e mostra, com dados reais: LED do motor, backend·dispositivo, taxa, período, ida-e-volta (medida ou "estimada", Spec 036), xruns, estado do gate e pico de entrada.
- **CA-8:** A tela Chassi desenha os elementos do board (turret board, transformador de força, capacitores de filtro, válvulas, transformador de saída e choke), a linha do osciloscópio e o inspetor. A tela Gabinete desenha o falante (baffle, carcaça, surround, cone, anéis, dust cap), com os microfones arrastáveis sobre o cone e o painel direito.
- **CA-9:** A tela Palco tem o cabeçalho OLED de 128 px com o nome do preset em 48 px, o painel de Tempo e os botões Sair/Afinador, a área do afinador + looper, e o dock de footswitches de 250 px com seletor de modo e 8 FS (scribble strip de 58 px + chave redonda de 90 px). A tela Áudio tem as 3 seções do board.
- **CA-10 (régua visual):** Um teste E2E captura cada tela em 1440×900 com dados fixos de fixture e compara com o PNG de referência. A similaridade estrutural (SSIM) por região do design fica ≥ 0,85, e a tela inteira ≥ 0,80. Abaixo disso, o teste reprova e salva o diff.
- **CA-11:** Os tokens de cor e de fonte de `styles.css` são os do canvas: fundo `#08090a`, destaque `#ff8a3d`, foco `#3dd8ff`, LCD `#ffb347`, fontes Inter/Oswald/JetBrains Mono. O teste lê o CSS.
- **CA-12 (não-regressão):** Os CAs 3, 4 e 8 da Spec 033 (sem scroll em 1280×800 e 1920×1080, inventário completo, coalescência IPC) continuam passando, e todo controle do registro continua presente.

## Fora de escopo
- Mudar o design: divergências intencionais (ex.: dado real no lugar de placeholder) são listadas nas Notas, com aprovação do usuário.
- Animações e microinterações não descritas no canvas.

## Invariantes tocadas
- **Todo controle visível funciona (regra do projeto):** os elementos novos do design (Modifier, engrenagem, menus) entram no `controls.registry.ts` com efeito, ou aparecem desabilitados com a dica da spec que os implementa.
- **Spec 033 (shell Tauri sem HTTP):** sem mudança.

## Testes que cobrem esta spec
- `tests/contract/048-ui-desktop-fidelidade-visual.test.js` (CA-1..CA-12: PNGs de referência, barra de título/menu, toolbar, matriz de sinal, palco, inspetor, barra de status, Chassi/Gabinete/Palco/Áudio, régua visual SSIM em 1440×900, tokens CSS/fontes offline, não-regressão)
- `tests/contract/033-app-desktop-tauri-v2-telas-e-controles-reais.test.js` (CA-12: não-regressão de inventário, sem scroll em 1280×800 e 1920×1080, painel Marshall e coalescência IPC)

## Notas
- **Referência do CA-1 exportada em 2026-11-14:** os 10 PNGs estão em `docs/ui-desktop/referencia/`, cada um nas dimensões do board em `canvas.json` (Main/Chassi/Gabinete/Palco 1440×900, Audio 1100×760, Amps 1214×2140, Pedais 1446×1451, AmpHead 1000×282, Pedal 226×337, Rack 560×190). Origem: artefato de design "AMPFORGE Desktop UI" (claude.ai). Os `.dc.html` foram renderizados com o runtime do próprio artefato (`dc-runtime.js` servido como `support.js`), por servidor local, e capturados com Edge headless (`--window-size=W,H --virtual-time-budget=10000 --screenshot`). As fontes vieram do Google Fonts.
- Na mesma data, `docs/ui-desktop/canvas/canvas.json` foi atualizado com a versão do artefato, que tinha Amps e Pedais redimensionados. Os `.dc.html` já eram idênticos.
- O `support.js` (runtime do canvas) continua fora do repositório. Para re-renderizar a referência, é preciso baixá-lo de novo do artefato.
- Recomenda-se implementar tela por tela (Main → Palco → Chassi → Gabinete → Áudio) e fechar o CA-10 de cada uma antes da próxima.
