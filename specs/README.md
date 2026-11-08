# Specs — Spec Driven Development (SDD)

Cada mudança de comportamento deste projeto nasce de uma **spec leve** aqui, **antes** do código.
A spec é a fonte dos **critérios de aceite** (`CA-n`), que orientam a implementação e a verificação.

Este arquivo é, além das regras, o **índice** — e o índice é o changelog do projeto.

## Fluxo

1. **Spec** — crie `specs/NNN-nome-da-feature.md` a partir do `_template.md`, com objetivo,
   contexto (com evidência), comportamento esperado e **critérios de aceite numerados**.
2. **Índice** — registre a spec na tabela abaixo. Spec fora do índice é spec que ninguém acha.
3. **Código** — implemente o comportamento descrito. Comportamento não previsto que aparecer
   durante a implementação volta para a spec, antes do merge.
4. **Verificação** — cada `CA-n` é verificado antes do merge: teste automatizado quando o critério
   toca invariante ou contrato público; verificação manual **documentada no PR** quando não valer
   o custo do teste.
5. **Fechamento** — status → `implementada`, seção *Testes que cobrem esta spec* preenchida,
   índice atualizado.

## Regras

- **Sem spec, sem código.** Mudança de comportamento começa por editar ou criar a spec.
- Critérios de aceite são **verificáveis** — afirmações que alguém pode reprovar, não desejos.
- CAs que tocam **invariantes** do projeto exigem teste automatizado; os demais podem ser
  verificados manualmente, desde que o PR registre como.
- Specs são versionadas com o código e revisadas **no mesmo PR** da implementação.
- Numeração é sequencial e **nunca reutilizada** — nem quando a spec é descartada.
- Toda spec tem *Fora de escopo* preenchido.

> **Regime de teste: SDD + TDD estrito.** Cada `CA-n` vira um teste que **falha antes** da
> implementação (red → green). Quando o projeto relaxar esse ciclo, registre aqui em qual spec
> a transição aconteceu — senão o histórico fica incoerente para quem lê de trás para frente.

## Convenção de nomes

- Spec: `specs/001-captura-de-evento.md`
- Teste de contrato: arquivo que espelha o número da spec (`.../contract/001-captura-de-evento.*`).
- Cada caso cita o critério no nome: `ca4_<comportamento>` ou `CA-4: <afirmação>`.
- Deve existir um jeito de rodar **só** os testes de uma spec (marcador, filtro ou diretório).
- O cabeçalho do arquivo de teste lista, em uma linha cada, todos os CAs que ele cobre.
- Um CA pode ter mais de um teste (os dois sentidos de uma precedência, por exemplo); todos citam
  o mesmo número.

## Status

`rascunho` → `aprovada` → `implementada`. Use estados honestos quando for o caso:
`implementada (CA-N revertida)`, `descartada` (com motivo, arquivo mantido, número não reaproveitado).

## Índice de specs

| Spec | Título | Status | Testes |
|---|---|---|---|
| [001](001-nucleo-dsp-tempo-real-ipc-lock-free.md) | Isolamento em tempo real do motor DSP, comunicação lock-free e orçamento de CPU | Aprovada | `tests/contract/001-nucleo-dsp-tempo-real-ipc-lock-free.test.js` *(pendente TDD)* |
| [002](002-matriz-roteamento-sinal-dag-4x12.md) | Matriz de roteamento de sinal 4x12 (DAG), nós de Split/Mixer e múltiplos I/Os | Aprovada | `tests/contract/002-matriz-roteamento-sinal-dag-4x12.test.js` *(pendente TDD)* |
| [003](003-modelagem-amplificadores-white-box-painel-frontal.md) | Modelagem White-Box de amplificadores valvulados e painel frontal autêntico | Aprovada | `tests/contract/003-modelagem-amplificadores-white-box-painel-frontal.test.js` *(pendente TDD)* |
| [004](004-bancada-chassi-aberto-mod-shop-osciloscopio-crt.md) | Bancada Chassi Aberto (Mod Shop) e osciloscópio CRT de transferência em tempo real | Aprovada | `tests/contract/004-bancada-chassi-aberto-mod-shop-osciloscopio-crt.test.js` *(pendente TDD)* |
| [005](005-gabinete-sala-microfonacao-ir-acoplamento-impedancia.md) | Bloco de gabinete (Cab), sala de microfonação dupla, loader de IR e acoplamento de impedância | Aprovada | `tests/contract/005-gabinete-sala-microfonacao-ir-acoplamento-impedancia.test.js` *(pendente TDD)* |
| [006](006-blocos-dinamica-drive-equalizacao.md) | Blocos de dinâmica (Noise Gate, Compressor), Overdrive/Fuzz White-Box e Equalizadores (RTA) | Aprovada | `tests/contract/006-blocos-dinamica-drive-equalizacao.test.js` *(pendente TDD)* |
| [007](007-blocos-modulacao-pitch-delay-reverb.md) | Blocos de modulação, pitch polifônico, delay com ducking/spillover e reverb | Aprovada | `tests/contract/007-blocos-modulacao-pitch-delay-reverb.test.js` *(pendente TDD)* |
| [008](008-afinador-estroboscopico-polifonico-looper.md) | Afinador estroboscópico/polifônico de alta precisão e Looper de performance | Aprovada | `tests/contract/008-afinador-estroboscopico-polifonico-looper.test.js` *(pendente TDD)* |
| [009](009-cenas-gapless-canais-modifiers-footswitches.md) | Hierarquia Setlist/Preset, 8 Scenes gapless, 4 canais por bloco, Modifiers e modos de footswitch | Aprovada | `tests/contract/009-cenas-gapless-canais-modifiers-footswitches.test.js` *(pendente TDD)* |
| [010](010-persistencia-offline-schema-sonixpreset-pacotes-sforgepkg.md) | Persistência 100% offline-first, validação de schema .sonixpreset e pacotes .sforgepkg | Aprovada | `tests/contract/010-persistencia-offline-schema-sonixpreset-pacotes-sforgepkg.test.js` *(pendente TDD)* |
| [011](011-design-system-skeuomorfico-componentes-3d-hardware-shell.md) | Design System skeuomórfico, tokens de materiais, controles táteis 3D e modo Hardware Shell | Aprovada | `tests/contract/011-design-system-skeuomorfico-componentes-3d-hardware-shell.test.js` *(pendente TDD)* |
| [012](012-host-desktop-standalone-tauri-plugins-vst3-clap.md) | Arquitetura Desktop nativa (Tauri v2 + Rust) e plugins de DAW (VST3 e CLAP) | Aprovada | `tests/contract/012-host-desktop-standalone-tauri-plugins-vst3-clap.test.js` *(pendente TDD)* |
