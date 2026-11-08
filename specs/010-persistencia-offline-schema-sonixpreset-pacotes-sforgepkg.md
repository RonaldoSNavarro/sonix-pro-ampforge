---
spec: 010
titulo: Persistência 100% offline-first, validação de schema .sonixpreset e pacotes .sforgepkg
status: aprovada
atualizado: 2026-11-08
---
# 010 — Persistência 100% offline-first, validação de schema .sonixpreset e pacotes .sforgepkg

## Objetivo
Garantir que o músico possa salvar, exportar, importar e compartilhar Presets individuais (`.sonixpreset`) e pacotes autocontidos com Impulse Responses embutidos (`.sforgepkg` verificados por SHA-256) de forma 100% determinística e offline, sem nenhuma dependência de rede ou nuvem.

## Contexto
Equipamentos de palco e estúdio não podem falhar porque o Wi-Fi caiu, um servidor de autenticação saiu do ar ou um arquivo `.wav` de Impulse Response mudou de pasta ao copiar um preset de um computador para outro. Além disso, o protótipo atual contém duas lacunas diretas em relação ao SRS:
1. Em `design-system/index.html:8-9`, o HTML faz requisições externas para `https://fonts.googleapis.com` e `https://fonts.gstatic.com`, contrariando o **RNF-009** (*"O aplicativo não deve realizar nenhuma requisição HTTP externa obrigatória"*).
2. Em `design-system/app.js:763-781`, o botão `#btnExportPreset` exporta um objeto parcial improviso (`{ schemaVersion, presetName, activeScene, oversampling, ampParams, circuitParams, blocks }`) que não obedece ao JSON Schema formal da Seção 5 do SRS (`docs/01-SRS-Levantamento-de-Requisitos.md:200-318`), omitindo `$schema`, `metadata` estruturado, `scenes`, `grid: { rows: 4, cols: 12, blocks: [...] }` com `activeChannelByScene`, `bypassByScene`, `channels` e `modifiers`, além de não possuir função de importação nem suporte a pacotes `.sforgepkg` com verificação SHA-256 (**RF-033**).

Fontes da verdade e evidências no repositório:
- `docs/01-SRS-Levantamento-de-Requisitos.md:23,180,196` — Define o objetivo estratégico **100% Offline-First**, o **RF-033** (Persistência Local Offline, `.sonixpreset` e `.sforgepkg` com verificação SHA-256) e o **RNF-009** (Zero requisição HTTP externa obrigatória e caminhos relativos dentro de `.sforgepkg`).
- `docs/01-SRS-Levantamento-de-Requisitos.md:200-318` — Especifica formalmente o esquema `https://sonixpro.local/schemas/sonixpreset-v1.json`.
- `design-system/index.html:8-9,78-80` e `design-system/app.js:763-781` — Implementação atual incompleta de exportação e carregamento de fontes externas.

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Formato `.sonixpreset` (Seção 5) | JSON simplificado fora do schema (`design-system/app.js:765-773`) | Conformidade integral com `https://sonixpro.local/schemas/sonixpreset-v1.json` (`$schema`, `schemaVersion`, `metadata`, `activeSceneIndex`, `scenes`, `grid` 4×12, `modifiers`) |
| Importação e Validação de Preset | Inexistente (apenas exporta) | Importação com validação estrita de schema e restauração completa do estado da aplicação |
| Pacote Autocontido `.sforgepkg` (**RF-033**) | Inexistente | Empacotador/desempacotador local contendo o manifesto `.sonixpreset`/Setlist + todos os arquivos `.wav` de IR referenciados por caminhos relativos + manifesto de hashes `SHA-256` |
| Operação 100% Offline (**RNF-009**) | Carrega Google Fonts via CDN (`design-system/index.html:8-9`) | Zero chamadas HTTP/HTTPS externas obrigatórias; fontes e assets servidos localmente com fallbacks de sistema imediatos |

**Alternativas descartadas:**
1. *Referenciar arquivos `.wav` de IR pelo caminho absoluto do sistema operacional (`C:\Users\...` ou `/home/...`):* descartada porque quebra o preset imediatamente ao abrir em outra máquina ou SO; dentro do `.sforgepkg` e da biblioteca local, todos os IRs usam caminhos relativos (`irs/<hash_ou_nome>.wav`) validados por SHA-256.
2. *Ignorar silenciosamente arquivos `.wav` corrompidos ao importar um `.sforgepkg`:* descartada porque alteraria o timbre do artista sem aviso; a divergência de hash SHA-256 deve abortar a importação do pacote corrompido com diagnóstico explícito.

## Comportamento esperado
1. **Serialização e Desserialização `.sonixpreset` (Seção 5 & RF-033):**
   - Ao exportar um Preset, o sistema gera um documento JSON compatível com `https://sonixpro.local/schemas/sonixpreset-v1.json` contendo obrigatoriamente:
     - `$schema`: `"https://sonixpro.local/schemas/sonixpreset-v1.json"`, `schemaVersion`: `"1.0.0"`;
     - `metadata`: `{ id (UUID v4), name, author, category, tempoBpm, createdAt (ISO 8601), footswitchMode }`;
     - `activeSceneIndex`: inteiro `0..7`;
     - `scenes`: array das 8 cenas (`index`, `id`, `name`, `color`);
     - `grid`: `{ rows: 4, cols: 12, blocks: [...] }`, onde cada bloco contém `instanceId`, `type`, `row`, `col`, `oversampling`, `activeChannelByScene` (8 inteiros), `bypassByScene` (8 booleanos) e `channels` (mapa `A..D` com `modelId`, `frontPanel`, `circuitModShop`, ou `coupledAmpInstanceId`/`impedanceCurve`/`mics` para blocos `CAB`);
     - `modifiers`: array de automações (`id`, `source`, `targetBlockId`, `targetParam`, `minVal`, `maxVal`, `curveSlope`, `dampingMs`).
   - Ao importar um `.sonixpreset`, o sistema valida o schema e reconstrói integralmente o estado do preset sem perda (*round-trip lossless*).
2. **Pacotes Autocontidos `.sforgepkg` com Verificação SHA-256 (RF-033):**
   - Ao exportar um `.sforgepkg`, o sistema gera um pacote contendo: (a) o JSON do Preset ou Setlist com referências relativas aos IRs (`irs/<arquivo>.wav`), (b) os binários `.wav` de todos os Impulse Responses utilizados pelo Preset/Setlist, e (c) o manifesto `manifest.json` registrando o hash `SHA-256` hexadecimal de cada arquivo embutido.
   - Ao importar um `.sforgepkg`, o sistema recalcula o `SHA-256` de cada `.wav` e do JSON antes de aplicá-los; se todos coincidirem, extrai os IRs para o repositório local offline e ativa o Preset/Setlist.
3. **Autonomia 100% Offline no Sistema de Arquivos Desktop (RNF-009):**
   - A aplicação Desktop (`Tauri v2` e plugins `VST3`/`CLAP`) inicializa, renderiza toda a interface e processa áudio com a interface de rede totalmente desconectada (`offline`), sem disparar `fetch`, `XMLHttpRequest` ou `<link href="https://...">` externos obrigatórios (fontes empacotadas localmente nos assets da aplicação).
   - Os presets de fábrica, biblioteca local de IRs e estado do usuário são persistidos diretamente no sistema de arquivos nativo do SO (`AppData` / `XDG_DATA_HOME` / `Application Support`) pelo backend Rust, sem depender de armazenamento volátil de navegador.

## Critérios de aceite
- **CA-1:** O JSON exportado para `.sonixpreset` contém todos os campos de topo exigidos pela Seção 5 do SRS (`$schema`, `schemaVersion`, `metadata`, `activeSceneIndex`, `scenes`, `grid` com `rows: 4, cols: 12` e `modifiers`), respeitando os tipos e estruturas de `frontPanel`, `circuitModShop`, `impedanceCurve` e `mics`.
- **CA-2:** Exportar um Preset em `.sonixpreset` e reimportá-lo em seguida (*round-trip*) restaura exatamente o mesmo estado de todas as 8 cenas, 4 canais por bloco, parâmetros White-Box, microfones do gabinete e Modifiers (`deepEqual` entre estado exportado e estado restaurado).
- **CA-3:** Exportar um pacote `.sforgepkg` inclui o documento do Preset/Setlist, todos os buffers `.wav` de IRs referenciados com caminhos relativos (`irs/...`) e o manifesto com os hashes `SHA-256` de cada entrada.
- **CA-4:** Importar um pacote `.sforgepkg` íntegro verifica o `SHA-256` de todos os itens embutidos, registra os IRs localmente e carrega o Preset pronto para tocar sem exigir busca manual dos arquivos `.wav`.
- **CA-5:** O carregamento da aplicação (`design-system/index.html` e scripts) não contém nenhuma requisição HTTP/HTTPS externa obrigatória (zero links para CDNs externas como `fonts.googleapis.com` ou APIs remotas), funcionando sem erros ou warnings de rede em ambiente 100% isolado (*air-gapped*).
- **CA-6:** Caso de erro — importar um `.sonixpreset` com JSON malformado, `schemaVersion` incompatível ou campos obrigatórios ausentes é recusado pelo validador com erro descritivo `PresetSchemaValidationError` listando o caminho do campo inválido, mantendo o preset atual intacto.
- **CA-7:** Caso de erro — importar um pacote `.sforgepkg` cujo conteúdo de arquivo `.wav` ou JSON tenha sido adulterado (divergência entre o `SHA-256` calculado e o `SHA-256` do manifesto, ou tentativa de *path traversal* `../`) é imediatamente rejeitado com `PackageIntegrityError` antes de gravar qualquer dado no armazenamento local.
- **CA-8:** Não-regressão — a operação de exportar um `.sonixpreset` ou `.sforgepkg` ocorre de forma assíncrona na Main/Worker Thread sem bloquear nem causar *dropouts* na thread de áudio.

## Fora de escopo
- Sincronização em nuvem, contas de usuário ou marketplace online (explicitamente proibidos pela diretriz estratégica 1.1.4 e pelo **RNF-009**).

## Invariantes tocadas
- **RNF-005 (Real-Time Safe):** Toda serialização JSON, cálculo de hash SHA-256 e leitura/escrita de arquivos ocorre fora da thread de áudio.
- **RNF-009 (Operação 100% Offline e Portabilidade):** Garantida diretamente nos CAs 1 a 5 e 7.

## Testes que cobrem esta spec
- `tests/contract/010-persistencia-offline-schema-sonixpreset-pacotes-sforgepkg.test.js` → CA-1, CA-2, CA-3, CA-4, CA-5, CA-6, CA-7, CA-8 *(pendente — ciclo TDD red → green)*.

## Notas
- Rastreabilidade SRS: Seção 1.1.4, **RF-033**, **RNF-009**, Seção 5 (`JSON Preset Schema`).
