---
spec: 011
titulo: Design System skeuomórfico, tokens de materiais, controles táteis 3D e modo Hardware Shell
status: implementada
atualizado: 2026-11-08
---
# 011 — Design System skeuomórfico, tokens de materiais, controles táteis 3D e modo Hardware Shell

## Objetivo
Garantir uma experiência visual e tátil fotorrealista de nível profissional — unificando tokens CSS de materiais físicos, tipografia serigráfica, componentes skeuomórficos 3D com ergonomia de precisão e alternância entre Visão de Estúdio e Carcaça Física da Pedaleira (*Hardware Shell*) a 60/120 FPS constantes.

## Contexto
A interface do **SONIX PRO // AMPFORGE™** é regida pela especificação formal `docs/02-Design-System-Specification.md`. Uma auditoria cruzada entre a especificação (`docs/02-Design-System-Specification.md`) e o protótipo atual (`design-system/styles.css`, `design-system/index.html`, `design-system/app.js`) revelou divergências concretas que precisam ser padronizadas e cobertas por testes de contrato:
1. **Divergência de nomes de Design Tokens CSS (`docs/02-Design-System-Specification.md:30-38,60-69` vs `design-system/styles.css:15-44`):**
   - A especificação exige `--surface-chassis-anodized`, `--surface-tolex-black`, `--surface-faceplate-gold`, `--surface-faceplate-chrome`, `--surface-circuit-board`, `--surface-oled-screen`, `--surface-crt-phosphor`, `--tube-glow-idle`, `--tube-glow-hot`, `--shadow-knob-3d`, `--shadow-chassis-inset` e `--shadow-footswitch`;
   - O arquivo `design-system/styles.css:15-44` definiu apenas apelidos abreviados (`--surface-anodized`, `--surface-brushed-brass`, `--surface-brushed-chrome`, `--surface-circuit-bay`, `--surface-oled`, `--surface-crt`, `--shadow-knob`, `--shadow-inset-deep`) e omitiu `--surface-tolex-black`, `--tube-glow-idle`, `--tube-glow-hot` e `--shadow-footswitch` no `:root` (embora use `var(--shadow-footswitch)` na linha `1514` sem tê-lo declarado no `:root`!).
2. **Variantes de Knob e Ergonomia (`docs/02-Design-System-Specification.md:21-22,86-96`):**
   - A especificação exige 3 variantes de `<sonix-knob>`: `Marshall Skirted Knob`, `Chicken Head Knob` e `Precision Studio Encoder`, além de tooltip exato em Hover, ajuste fino `0.01` com `Shift + Drag`, reset com `Duplo Clique` e abertura de `Modifier Matrix` com `Clique Direito`.
3. **Estado de Tap Tempo no Footswitch (`docs/02-Design-System-Specification.md:119`):**
   - O anel RGB do footswitch deve suportar os 3 estados: `Aceso Brilhante (100% + Glow)`, `Dimmed (25% opacidade)` e `Piscando no BPM` para `TAP TEMPO`.

Fontes da verdade e evidências no repositório:
- `docs/02-Design-System-Specification.md:1-138` — Especificação integral do Design System v1.0.0 (Seções 1 a 5).
- `docs/01-SRS-Levantamento-de-Requisitos.md:195` — Define **RNF-008** (Renderização constante a 60 FPS / 120 Hz sem interferir na thread de áudio).
- `design-system/styles.css:6-44,1514` — Tokens atuais com nomes divergentes e `--shadow-footswitch` referenciado mas não definido no `:root`.

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Tokens de Superfícies, Luz e Sombra (Seção 2) | Nomes abreviados e `--shadow-footswitch` ausente no `:root` (`styles.css:15-44,1514`) | Presença obrigatória no `:root` de todos os tokens canônicos da Seção 2.1, 2.2 e 2.3 (mantendo aliases de compatibilidade) |
| Catálogo de Knobs 3D (Seção 4.1) | Apenas variante *Marshall Skirted* (`styles.css:924-969`) | 3 variantes (`skirted`, `chicken-head`, `studio-encoder`), sombra fixa `.knob-shadow-base` que não gira com `.knob-rotor`, `Shift+Drag` com passo fino `0.01`, `dblclick` reset e `contextmenu` Modifier |
| Footswitch RGB & Tap BPM (Seção 4.5) | Estados ativo e `.dimmed` (`styles.css:1488-1506`) | Display OLED `128×32px` simulado + 3 estados de anel RGB: ativo (`100% + Glow`), bypass (`.dimmed` `25%`), e `.tap-tempo-pulse` piscando na frequência exata do `tempoBpm` |
| Hierarquia em 3 Zonas e Modo Hardware Shell (Seção 1.2 e 5) | Implementado em `index.html` e `styles.css:70-135` | Preservação estrita das 3 Zonas (`Top Bar & Grid`, `Main Stage`, `Footswitch Dock`), parafusos CNC no `.hardware-shell-mode` e catálogo interativo `#dsCatalogSection` |

**Alternativas descartadas:**
1. *Girar o elemento inteiro do knob (incluindo sua `box-shadow`) ao aplicar `transform: rotate(...)`:* descartada porque em iluminação física real a fonte de luz zenital é fixa no teto do estúdio; se a sombra girar junto com o botão, a ilusão 3D quebra imediatamente (por isso `.knob-shadow-base` permanece estacionária e apenas `.knob-rotor` recebe `rotate(var(--knob-deg))`).
2. *Usar texturas bitmap PNG/JPEG pesadas para Tolex, latão escovado e cromo:* descartada porque aumenta o tamanho do pacote, perde nitidez em telas 4K/Retina e prejudica o tempo de carregamento; utilizam-se microtexturas procedurais CSS/SVG aceleradas por GPU.

## Comportamento esperado
1. **Design Tokens Canônicos no `:root` (Seção 2):**
   - **Superfícies (Seção 2.1):** `--surface-chassis-anodized`, `--surface-tolex-black`, `--surface-faceplate-gold`, `--surface-faceplate-chrome`, `--surface-circuit-board`, `--surface-oled-screen` e `--surface-crt-phosphor` (além dos aliases existentes no protótipo).
   - **Categorias de Blocos e LEDs RGB (Seção 2.2):** `--led-gate` (`#00E676`), `--led-comp` (`#00B0FF`), `--led-drive` (`#FF9100`), `--led-amp` (`#FF3D00`), `--led-cab` (`#FFC400`), `--led-eq` (`#1DE9B6`), `--led-mod` (`#7C4DFF`), `--led-pitch` (`#E040FB`), `--led-delay` (`#00E5FF`) e `--led-reverb` (`#651FFF`).
   - **Incandescência e Sombras 3D (Seção 2.3):** `--tube-glow-idle`, `--tube-glow-hot`, `--shadow-knob-3d`, `--shadow-chassis-inset` e `--shadow-footswitch`.
2. **Tipografia e Serigrafia de Hardware (Seção 3):**
   - Serigrafia de painéis (`.faceplate-label`): família `'Oswald'`, `'Bebas Neue'`, `'Impact'`, caixa alta (`uppercase`), `letter-spacing: 0.12em`, peso `700`;
   - Displays OLED e telemetria DSP: `'JetBrains Mono'`, `'Share Tech Mono'`, monospace com `font-variant-numeric: tabular-nums` para impedir jitter horizontal em leituras a 60 FPS;
   - Interface de sistema e modais: `'Inter'`, `system-ui`, `sans-serif` com contraste WCAG AAA sobre fundo escuro.
3. **Anatomia e Ergonomia dos Controles 3D (Seção 1.3 e Seção 4):**
   - **Knob Rotativo (`<sonix-knob>`):** suporta as 3 variantes (`skirted`, `chicken-head`, `studio-encoder`), mantém `.knob-shadow-base` sem rotação enquanto `.knob-rotor` rotaciona de `-135deg` a `+135deg`, exibe tooltip/readout exato no hover/drag, ajusta com precisão fina (`0.01`) quando `Shift` está pressionado durante o arraste vertical, restaura `defaultVal` no `dblclick` e abre a matriz de Modifiers no `contextmenu`.
   - **Footswitch (`<sonix-footswitch>`):** exibe mini-display OLED superior com código (`FS 1..8`), modo e nome, êmbolo metálico e anel RGB operando em `100% + Glow` (ativo), `25% opacidade` (`.dimmed` em bypass) e pulsação sincronizada ao BPM (`60 / tempoBpm` segundos) quando atribuído a `TAP TEMPO`.
4. **Layout em Três Zonas e Modo Palco / Hardware Shell (Seção 1.2, Seção 5 e RNF-008):**
   - A janela principal mantém a separação clara entre Barra Mestre + Zona 1 (`#signalGridSection`), Zona 2 (`#mainStageSection`) e Zona 3 (`footer.hardware-footswitch-dock`).
   - Acionar `#btnHardwareShell` aplica `.hardware-shell-mode` ao `body`, exibindo a moldura chanfrada de alumínio CNC, os parafusos físicos `.chassis-screws` e o cabeçalho de identificação de hardware.
   - Todas as animações de medidores VU, osciloscópio CRT e incandescência de válvulas utilizam `requestAnimationFrame` e propriedades compostas na GPU (`transform`, `opacity`), mantendo 60 FPS (ou 120 Hz) sem bloquear a thread de áudio.

## Critérios de aceite
- **CA-1:** O seletor `:root` em `design-system/styles.css` declara explicitamente todos os 7 tokens de superfície da Seção 2.1 (`--surface-chassis-anodized`, `--surface-tolex-black`, `--surface-faceplate-gold`, `--surface-faceplate-chrome`, `--surface-circuit-board`, `--surface-oled-screen`, `--surface-crt-phosphor`), todos os 10 tokens `--led-*` da Seção 2.2 com seus valores hexadecimais exatos e todos os 5 tokens de incandescência/sombra da Seção 2.3 (`--tube-glow-idle`, `--tube-glow-hot`, `--shadow-knob-3d`, `--shadow-chassis-inset`, `--shadow-footswitch`).
- **CA-2:** Os elementos de telemetria numérica e displays OLED (`.preset-bank-badge`, `.meter-pill`, `.knob-value-readout`, `.trimpot-val`, `.scribble-strip-oled`, `.tuner-note-display`) aplicam `font-family: var(--font-mono)` e `font-variant-numeric: tabular-nums`, e os rótulos `.faceplate-label` aplicam `font-family: var(--font-faceplate)`, `text-transform: uppercase`, `font-weight: 700` e `letter-spacing >= 0.1em`.
- **CA-3:** O componente de knob 3D suporta as 3 variantes visuais (`skirted`, `chicken-head`, `studio-encoder`) e mantém a sombra `.knob-shadow-base` com rotação `0deg` independente do ângulo `--knob-deg` (`-135deg` a `+135deg`) aplicado ao `.knob-rotor`.
- **CA-4:** Arrastar um knob verticalmente com a tecla `Shift` pressionada altera seu valor com resolução fina de `0.01` por incremento; um duplo clique (`dblclick`) restaura exatamente o valor `defaultVal` de fábrica daquele parâmetro.
- **CA-5:** Os footswitches no `#footswitchArray` renderizam o estado ativo com brilho total, o estado em bypass com opacidade reduzida (`.dimmed`) e o estado de `TAP TEMPO` piscando em sincronia com o período $\frac{60}{\text{tempoBpm}}\text{ s}$.
- **CA-6:** Alternar `#btnHardwareShell` ativa/desativa a classe `body.hardware-shell-mode`, tornando visíveis os parafusos `.chassis-screws` e o chanfro externo da pedaleira física sem desmontar nem perder o estado das Zonas 1, 2 e 3.
- **CA-7:** A aba `#dsCatalogSection` (`data-view="ds-catalog"`) renderiza amostras visuais (*swatches*) das superfícies skeuomórficas, dos 10 tokens `--led-*`, da hierarquia tipográfica e das 3 variantes de knobs.
- **CA-8:** Caso de erro — passar um valor fora de `[min, max]` para a função de cálculo de ângulo do knob (`valueToAngle`) limita o ângulo resultante estritamente ao arco físico `[-135deg, +135deg]`, impedindo que o indicador gire $360^\circ$ ou aponte para a zona morta inferior do potenciômetro.
- **CA-9:** Não-regressão — alternar entre as 3 abas de visão (`rig-workspace`, `cab-room`, `ds-catalog`) ou ligar/desligar o Modo Palco (`hardware-shell-mode`) preserva todos os valores de parâmetros, cena ativa, canal selecionado e estado do gerador de sinal.

## Fora de escopo
- A matemática interna dos algoritmos DSP acionados pelos controles (coberta pelas **Specs 001 a 008**).

## Invariantes tocadas
- **RNF-008 (UI Rendering 60/120 FPS):** O ciclo visual usa `requestAnimationFrame` desacoplado do motor DSP.
- **Princípios de Direção de Arte (Design System Seção 1):** Honestidade Material, Hierarquia em Três Zonas e Ergonomia de Estúdio e Palco são verificadas diretamente nos CAs 1 a 7.

## Testes que cobrem esta spec
- `tests/contract/011-design-system-skeuomorfico-componentes-3d-hardware-shell.test.js` → CA-1 a CA-9 (9 testes automatizados de tokens canônicos, tipografia, 3D knobs, footswitch, hardware-shell e valueToAngle, 100% ok).

## Notas
- Rastreabilidade: `docs/02-Design-System-Specification.md` (Seções 1, 2, 3, 4 e 5) e `docs/01-SRS-Levantamento-de-Requisitos.md` (**RNF-008**).
