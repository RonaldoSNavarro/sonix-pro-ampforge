---
spec: 052
titulo: Abertura animada com emblema SVG vetorial, designs skeuomórficos dos pedais faltantes e mini-cabeça AMP 2 no Estúdio (itens T-27 a T-29)
status: implementada
atualizado: 2026-11-20
---
# 052 — Abertura animada com logomarca, pedais skeuomórficos faltantes e AMP 2 visível no Estúdio

## Objetivo
Entregar a abertura animada com emblema SVG vetorial da marca **SONIX PRO // AMPFORGE™** durante o carregamento real da aplicação, os designs skeuomórficos completos dos pedais de modulação, pitch e wah (com referência visual antes do código, conforme regra da Spec 048) e a visualização reduzida do **AMP 2** no palco do Estúdio quando o caminho paralelo estiver ativo.

## Contexto
Do relato do teste manual do usuário em 2026-11-16 (`docs/handoff/ANTIGRAVITY-051-teste-manual-2026-11-16.md`, itens T-27 a T-29):
1. **T-27 (Abertura animada):** *"Seria interessante exibir a logomarca Sonix-Pro // AmpForge como uma abertura animada enquanto a aplicação é carregada."* Decisão do usuário em 2026-11-17: criar emblema SVG vetorial dedicado da marca `"SONIX PRO // AMPFORGE™"` embutido no bundle, exibido antes do primeiro quadro do shell até a conclusão do `bootstrap()`, respeitando `prefers-reduced-motion` e funcionando 100% offline.
2. **T-28 (Pedais sem design):** *"Pedais como flanger, wah e outros sequer tiveram o design feito."* Os seletores de bloco já ofereciam modelos de modulação (`Tri-Chorus`, `TZ Flanger`, `Phaser 90`, `Tremolo`, `Uni-Vibe`, `Rotary`) e pitch (`Whammy`, `Drop Capo`, `Harmonizer`, `Detune`), além da demanda pelo pedal de **Wah** (indutor ressonante Cry Baby GCB95 / Clyde McCoy), mas a Biblioteca e o `Pedal.ts` tinham apenas 12 unidades desenhadas.
3. **T-29 (AMP 2 visível no Estúdio):** *"O amp paralelo poderia aparecer na tela mesmo que em um espaço menor."* Quando a linha B / split está ativa ou quando o bloco `amp2` é inspecionado, o usuário precisa ver e operar o cabeçote secundário (`AMP 2`) em escala reduzida no palco do Estúdio.

## Comportamento esperado
1. **Referência visual antes do código (Regra da Spec 048):**
   - Os arquivos de referência em `docs/ui-desktop/canvas/` e os PNGs correspondentes em `docs/ui-desktop/referencia/` são atualizados antes da implementação de UI da Spec 052.
2. **Abertura animada (T-27):**
   - O emblema SVG vetorial dedicado **SONIX PRO // AMPFORGE™** (com estética de válvulas aquecendo / filamento âmbar e chassi metálico) é exibido imediatamente na abertura da janela e permanece visível enquanto `bootstrap()` aguarda o IPC inicial (com tempo mínimo suave de percepção e encerramento imediato ao concluir + teto de segurança).
   - Com `prefers-reduced-motion: reduce`, a abertura renderiza estática sem animações de pulso/varredura e encerra imediatamente ao fim do `bootstrap()`.
   - Funciona 100% embutida no bundle (sem rede externa ou `localhost`).
3. **Pedais skeuomórficos completos e bloco Wah (T-28):**
   - `Pedal.ts` e a tela `Biblioteca.ts` incluem designs skeuomórficos dedicados, com controles registrados em `controls.registry.ts` e efeito real no motor DSP, para:
     - **Modulação:** `Tri-Chorus`, `TZ Flanger`, `Phaser 90`, `Tremolo`, `Uni-Vibe`, `Rotary`;
     - **Pitch:** `Whammy`, `Drop Capo`, `Harmonizer`, `Detune`;
     - **Wah (`wah_1`):** pedal de wah clássico com indutor ressonante (baseado no circuito Dunlop Cry Baby GCB95: indutor $L = 500\text{ mH}$, capacitor de varredura $0{,}01\ \mu\text{F}$, fator $Q$ ajustável e faixa de varredura $350\text{ Hz}$–$2{,}2\text{ kHz}$), controlado por `wah_1.position`, `wah_1.q`, `wah_1.mode` e integrável ao `EXP 1`.
4. **AMP 2 visível em tamanho reduzido no Estúdio (T-29):**
   - No palco do Estúdio (`Estudio.ts`), abaixo/ao lado do cabeçote principal (na seção `LINHA B / PÓS`), um mini-cabeçote **AMP 2** em escala reduzida exibe o modelo selecionado em `amp_2.model` e seus knobs principais (`Gain`, `Bass`, `Middle`, `Treble`, `Master`), permitindo edição direta ligada a `amp_2.*`.

## Critérios de aceite
- **CA-1 (Referência visual atualizada antes do código):** `docs/ui-desktop/canvas/` e `docs/ui-desktop/referencia/` contêm as referências atualizadas para os novos pedais e para a visão do AMP 2 antes da implementação de `T-28` e `T-29`.
- **CA-2 (T-27 — Abertura animada com emblema SVG vetorial):** O splash screen com emblema SVG `"SONIX PRO // AMPFORGE™"` está presente antes da resolução do `bootstrap()`, permanece visível quando o IPC é atrasado artificialmente, desaparece quando `bootstrap()` conclui e desativa animações CSS quando `prefers-reduced-motion: reduce` está ativo.
- **CA-3 (T-28 — Designs e efeito DSP de todos os pedais de Modulação, Pitch e Wah):** Todos os modelos de modulação (`ce2`, `trichorus`, `flanger`, `phaser90`, `tremolo`, `univibe`, `rotary`), pitch (`whammy`, `drop`, `harmonizer`, `detune`) e `wah` possuem componente visual próprio em `Pedal.ts`, card na `Biblioteca.ts` sem deformação e controles com efeito mensurável no backend (`wah_1.position` desloca o pico de ressonância entre $350\text{ Hz}$ e $2{,}2\text{ kHz}$).
- **CA-4 (T-29 — Mini-cabeça AMP 2 no Estúdio):** O palco do Estúdio exibe a unidade reduzida do **AMP 2** refletindo `amp_2.model` e permitindo ajustar os controles `amp_2.gain`, `amp_2.bass`, `amp_2.middle`, `amp_2.treble` e `amp_2.master` com efeito real no motor quando a linha paralela está ativa.

## Fora de escopo
- Novos amplificadores além dos 5 modelos White-Box já especificados na Spec 031.

## Invariantes tocadas
- **Zero-alloc no callback de áudio:** O filtro biquad/estado-variável do Wah (`WahProcessor`) opera sem alocação dinâmica, lock ou I/O no callback de áudio (verificado pelo alocador de contagem da Spec 037).
- **Todo controle visível funciona no backend:** Todos os controles dos novos pedais e do mini-AMP 2 estão registrados em `controls.registry.ts` e validados por `api::dispatch`.
- **Fidelidade visual (Spec 048):** Referências visuais atualizadas antes do código e validadas pela suíte de contrato.

## Testes que cobrem esta spec
- `crates/ampforge-dsp/tests/contract_052_wah_and_pedals.rs` → `CA-3`
- `crates/sonix-desktop/tests/contract_052_backend_wah_amp2.rs` → `CA-3`, `CA-4`
- `tests/contract/052-abertura-pedais-amp2.test.js` → `CA-1`, `CA-2`, `CA-3`, `CA-4`

## Notas
- Implementada e validada na Etapa 7 (itens T-27 a T-29), mantendo zero-alloc no callback de áudio e todas as invariantes SDD/UI verdes.
