---
spec: 017
titulo: Carregador de IR entra em pânico com cabeçalho malformado e o redo/undo do looper perde camadas
status: rascunho
atualizado: 2026-11-09
---
# 017 — Resiliência do Carregador de IR e Undo/Redo Fiel do Looper

## Objetivo
Impedir que um arquivo WAV malformado derrube o processo ao ser carregado como IR, e garantir que
`undo`/`redo` do looper restaurem exatamente as camadas que o músico ouvia — sem perder camadas
anteriores nem fingir que restaurou uma camada que foi descartada.

## Contexto
Origem: code review de 2026-11-09 (achado F-07) e revisão das specs 014–017 do mesmo dia, que
mostrou que a evidência original do F-08 era falsa: `undo()` já é guardado por `has_overdub` e
`can_undo()` já fica `false` após o undo (`looper.rs:150-156`; asserção existente em
`contract_008_tuner_looper.rs:142-145`). O F-08 foi refeito sobre os defeitos reais abaixo.

Reprodução (crate descartável com `ampforge-dsp` real, fora do repositório; base `B` = 1,0,
camadas `L1` = 0,5 e `L2` = 0,25, saída lida com entrada silenciosa):

```
B+L1                          = 1.5
undo                          = 1.0
redo  (esperado 1.5)          = 1.0   can_undo = true      ← camada perdida, flag mente
B+L1+L2                       = 1.75
undo de L2 (esperado 1.5)     = 1.0                        ← L1 também some
WAV channels=0, rate 48000    → panic "chunk size must be non-zero"   (cab.rs:182)
WAV channels=1, rate 0        → panic "capacity overflow"             (resample, cab.rs:214-217)
WAV channels=1, rate 1 Hz     → Ok, 16 amostras viram 768.000          (sem limite de taxa)
```

### F-07 — Pânicos no `WavIrLoader::load_from_bytes` (`cab.rs:134-228`)
- `channels` vem do cabeçalho sem validação; com `0`, `chunks_exact(2 * channels)` recebe `0` e
  entra em pânico (vale para os ramos de 16, 24 e 32 bits, `cab.rs:182,189,196`).
- `sample_rate` também não é validado: com `0`, `ratio = 0` → `new_len = inf.round() as usize`
  satura em `usize::MAX` → `Vec::with_capacity` entra em pânico. Taxas absurdamente baixas
  (1 Hz) não entram em pânico, mas multiplicam o tamanho da IR por 48.000 — vetor de alocação
  descontrolada a partir de um arquivo de poucos bytes.
- Já está correto (não refazer): arquivo vazio → `EmptyFile`; < 44 bytes, sem `RIFF/WAVE` ou sem
  chunk `data` → `CorruptedOrTruncated`; combinação formato/bits fora de PCM16, PCM24 e Float32
  (inclusive `bits_per_sample = 0`) → `UnsupportedFormat` (`cab.rs:135-206`).
- O tipo de erro é o existente `InvalidImpulseResponseError` (`cab.rs:89-93`); esta spec não cria
  tipo novo.

### F-08 — Undo/redo do looper (`looper.rs:135-164, 203-213`)
- O flag `has_overdub` acumula dois papéis: "há camada para desfazer" **e** "o `overdub_buffer` é
  audível" (`looper.rs:204`). Por isso o undo da 2ª camada silencia também a 1ª: `undo()` copia
  `undo_buffer` (que guardava `L1`) de volta para `overdub_buffer`, mas zera `has_overdub`, e a
  reprodução passa a ignorar o buffer inteiro.
- `undo()` **sobrescreve** `overdub_buffer` com `undo_buffer` (`looper.rs:153`); a camada desfeita
  deixa de existir em memória. `redo()` só religa o flag (`looper.rs:159-164`) — `can_undo()`
  volta a `true` mas o áudio da camada não volta.
- `undo()` chamado com o looper em `Overdub` não sai do estado `Overdub`: a gravação continua no
  `overdub_buffer` (`looper.rs:211-213`) enquanto ele está inaudível, e um `redo` posterior
  reproduz uma mistura do estado anterior com o trecho gravado depois do undo.
- `overdub()` copia o buffer **inteiro** (`capacity`, 2.880.000 amostras por padrão) para o
  backup (`looper.rs:138`), mesmo que o loop tenha poucas amostras.

## Comportamento esperado
1. **Validação de cabeçalho WAV.** `channels == 0` ou `sample_rate == 0` →
   `Err(InvalidImpulseResponseError::CorruptedOrTruncated)`. `sample_rate` fora de
   `8_000..=384_000` Hz → `Err(InvalidImpulseResponseError::UnsupportedFormat(..))`. A validação
   acontece antes de qualquer alocação proporcional ao conteúdo do arquivo.
2. **Nenhum cabeçalho derruba o processo.** Para qualquer sequência de bytes, `load_from_bytes`
   retorna `Ok` ou `Err` — nunca entra em pânico.
3. **Undo/redo por troca de buffers.** O looper mantém a mistura de camadas atual e uma cópia da
   mistura anterior à última camada. `undo()` e `redo()` trocam as duas (troca de referência,
   O(1), sem alocação). O buffer de overdub é sempre audível (é silêncio quando não há camadas);
   `can_undo`/`can_redo` são flags próprios, independentes da audibilidade.
4. **Undo durante overdub.** `undo()` com o looper em `Overdub` encerra a gravação (estado →
   `Play`) e descarta a camada em andamento, voltando à mistura anterior a ela.
5. **Novo overdub invalida o redo.** Gravar uma camada nova depois de um `undo` descarta a camada
   desfeita: `can_redo()` → `false`.
6. **Backup proporcional ao loop.** `overdub()` copia apenas as `loop_length` amostras usadas.

## Critérios de aceite
- **CA-1:** `load_from_bytes` com WAV PCM16 válido exceto `channels = 0` retorna
  `Err(InvalidImpulseResponseError::CorruptedOrTruncated)`, sem pânico (o teste envolve a chamada
  em `std::panic::catch_unwind`). Vale também com `bits_per_sample` 24 e Float32.
- **CA-2:** `load_from_bytes` com WAV PCM16 válido exceto `sample_rate = 0` retorna
  `Err(InvalidImpulseResponseError::CorruptedOrTruncated)`, sem pânico.
- **CA-3:** `sample_rate` ∈ {`1`, `7_999`, `384_001`, `u32::MAX`} retorna
  `Err(InvalidImpulseResponseError::UnsupportedFormat(_))`; `8_000` e `384_000` são aceitos.
- **CA-4:** Varredura determinística: partindo de um WAV válido, cada campo do chunk `fmt ` e os
  tamanhos de chunk `RIFF`/`fmt `/`data` recebem cada valor de {`0`, `1`, `2`, `MAX/2`, `MAX`} do
  seu tipo (um campo por vez, mais todos os pares `channels × sample_rate`); em 100 % dos casos
  `load_from_bytes` retorna sem pânico.
- **CA-5:** Com base `B` (1,0), camada `L1` (0,5), `undo()` e `redo()`, a saída com entrada
  silenciosa é `B + L1` (1,5) amostra a amostra, e `can_undo()` = `true`, `can_redo()` = `false`.
- **CA-6:** Com base `B`, camadas `L1` e `L2` gravadas em dois overdubs separados por `play()`,
  `undo()` resulta em saída `B + L1` (1,5), não `B` (1,0).
- **CA-7:** Com o looper em `Overdub` gravando `L2` sobre `B + L1`, `undo()` muda o estado para
  `Play`, e os blocos seguintes (com entrada silenciosa ou não) reproduzem `B + L1` sem nenhum
  trecho de `L2`; um `redo()` seguinte não traz `L2` de volta (`can_redo()` = `false`).
- **CA-8:** Não-regressão — após `undo()`, gravar `L3` com `overdub()` → `play()` resulta em
  `B + L3`, e `can_redo()` = `false` (o redo da camada desfeita é descartado).
- **CA-9:** Não-regressão de tempo real — `undo()`, `redo()` e `overdub()` não alocam (teste com
  alocador global contador: 0 alocações durante as chamadas, Spec 008 RNF-005).
- **CA-10:** Não-regressão — os 9 CAs da Spec 008 e os 8 CAs da Spec 005 continuam verdes.

## Fora de escopo
- Pilha de undo com várias camadas: o contrato da Spec 008 CA-5 é undo/redo **da última camada**.
- Convolução da IR carregada no sinal ao vivo (hoje `custom_ir` não é lido em `process`, Spec 016
  Contexto) → **Spec 022**.
- Demais validações e tratamento de IR que a skill do projeto (`sonix-pro-ampforge-expert` §9.2) pede e que hoje não existem — limite de duração, WAV com chunk `fmt ` estendido (`WAVE_FORMAT_EXTENSIBLE`), uso de todos os canais (hoje só o canal 0 é lido, `cab.rs:182-198`, sem aviso) e reamostragem com filtro de qualidade (hoje interpolação linear, `cab.rs:214-224`) — ficam para a **Spec 022** (convolução de IR), que é quando a IR passa a soar. Esta spec trata só do que derruba o processo ou aloca sem limite.
- Armazenamento em disco ou exportação de áudio do looper.

## Invariantes tocadas
- Áudio em tempo real (Spec 001/008 RNF-005): undo/redo passam a ser troca de referências; nenhuma
  alocação nem cópia proporcional à capacidade na thread de áudio (CA-9, §6).
- Nenhum parsing de arquivo externo pode entrar em pânico (CA-1…CA-4).
- Spec 008 CA-5 (undo/redo da última camada): mantida e agora fiel ao áudio.

## Testes que cobrem esta spec
Planejados (red → green): `crates/ampforge-dsp/tests/contract_017_resilience_ir_looper.rs`
(CA-1…CA-10). Red esperado antes da implementação: CA-1, CA-2, CA-3, CA-4 (pânico/`Ok` indevido),
CA-5, CA-6, CA-7 (camada perdida). CA-8, CA-9 e CA-10 são não-regressão e já passam.

## Notas
- A versão anterior desta spec propunha `LooperProcessor::clear()` e
  `IrLoaderError::InvalidFormat`; nenhum dos dois existe e nenhum é necessário — `record()` já
  zera `has_overdub`/`can_redo` (`looper.rs:117-123`) e o loader já tem
  `InvalidImpulseResponseError`. Removidos.
- Reprodução guardada fora do repositório (crate descartável `repro017` no scratchpad da sessão).
