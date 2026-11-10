---
spec: 016
titulo: Cadeia de guitarra sem tone stack, sem pedal de drive, com gabinete que dobra o nível e saída acima de 0 dBFS
status: rascunho
atualizado: 2026-11-09
---
# 016 — Cadeia DSP de Guitarra Funcional (tone stack, drive, gain staging, gabinete, saída segura)

## Objetivo
Fazer a cadeia que o guitarrista realmente ouve (Gate → [Drive] → Amp → Cab) reagir a todos os controles do painel, soar sem clipping digital e deixar de se comportar como "mesmo timbre para qualquer modelo".

## Contexto
Origem: relato "com distorção" de 2026-11-09 e code review do mesmo dia (achados F-04, F-05, F-06, F-07). Medições com o crate descartável de reprodução (`RealtimeDspChain`/blocos reais, 48 kHz, seno ~130 Hz):

| Observação | Resultado | Ponteiro |
|---|---|---|
| Cab com defaults | **ganho 2,000×** (rms 0,204 → 0,408) | `cab.rs:30-45` (`enabled: true` nos 2 slots, 0 dB, delay 0) e `cab.rs:314-365` (soma os dois) |
| Cab aplica IR/filtros? | **não**: `custom_ir`, `conditioning`, `impedance_curve`, `model` nunca são lidos em `process` | `cab.rs:241,307-365` |
| `bass`/`treble` 0 vs 10 | **saídas idênticas** (`==`) | `amp.rs:240-307` não lê `bass/middle/treble/overdrive` |
| `AmpModelId` JCM800 vs Twin | **saídas idênticas** | `amp.rs:240-307` ignora `model_id` |
| Drive 808 vs Fuzz, `tone`/`level` | **saídas idênticas**; `DriveProcessor` nem está na cadeia | `drive.rs:76-99`, `audio.rs:408-421` |
| Nível com config de `main.rs` (gate −65, boost 2,5, gain 8, master 7) | entrada pico 0,01→saída pico 0,23; 0,03→0,66; **0,10→1,63 (62 % das amostras > 1,0 FS)**; 0,30→2,00 (clamp) | `audio.rs:527-530` limita só em ±2,0 |
| `masterVol` 0/5/10 | rms 0,108 / 0,407 / 0,639 — o master é, na prática, o controle de ganho do estágio de potência (`1+6·master/10` antes da `tanh`) | `amp.rs:261-262` |

Leitura técnica: (a) `inputGain` multiplica por no máximo **1,0** (`amp.rs:243`, `(gain/10).max(0.01)`), então o knob de ganho só atenua; a saturação vem quase toda do master — estrutura que **diverge da intenção de um 2203** (onde o Pre-Amp Volume governa o ganho dos triodos de entrada e o Master atua no nível/estágio de potência; **posição exata do Master no circuito a confirmar com o esquema oficial da revisão**, sem inventar topologia) — hipótese de projeto a validar por medição, não afirmação de equivalência; (b) `depth` é implementado como **passa-altas** somado ao sinal (`amp.rs:294-296`, `x − lowpass(x)` com polo 0,98 ≈ 154 Hz) — realça acima de ~150 Hz, não é ressonância grave de 65–130 Hz (Spec 003 CA-5); (c) coeficientes de bright/presence/depth (`0.85/0.15`, `0.6/0.4`, `0.98/0.02`) são constantes de 48 kHz, não derivados de `sample_rate`; (d) não há oversampling nos estágios não lineares (`tanh`, `x/(1+|x|)`), embora `oversampling.rs` exista; (e) o clamp final ±2,0 (`audio.rs:527`) está 6 dB **acima** do fundo de escala: o mixer do Windows corta em ±1,0 → clipping digital áspero, provavelmente parte da "distorção" ouvida.

## Comportamento esperado
1. **Tone stack passivo de 3 bandas** aplicado no sinal, com topologia por modelo (documentada na spec/notas com **revisão do esquema e fonte**; implementação por função de transferência discretizada — ref.: Yeh & Smith, *Discretization of the '59 Fender Bassman Tone Stack*, DAFx-06). Valores de componentes do modelo inicial (BRIT JCM800 2203) marcados "a confirmar com esquema oficial" até haver fonte.
2. **Estrutura de ganho**: `inputGain`/`overdrive` controlam o ganho do pré (de ~1× a no mínimo +30 dB entre 0 e 10 — alvo de calibração, ajustável por medição); `masterVolume` controla nível pós-estágio de potência e quantidade de saturação de potência; `outputLevel` é ganho linear final.
3. **Pedal de drive na cadeia** (antes do amp), com bypass pelo FS1 (`blk_drive`, chave `drive_enabled` da Spec 014 §2): `gain`, `tone`, `level` e `model`/diodo realmente alteram o sinal (curva de clipping e pré/pós-EQ por modelo; hipóteses documentadas).
4. **Presence e Depth** corretos: presence = realce em alta (> 2 kHz) na malha de realimentação; depth = realce/ressonância em 65–130 Hz; coeficientes calculados a partir da taxa de amostragem efetiva.
5. **Seleção de modelo audível**: cada `AmpModelId` aplica estrutura de ganho + tone stack + voicing próprios; o painel (logo, tema) já muda no frontend.
6. **Gabinete funcional**: por padrão **um** slot de mic ativo; o slot 2 só soma quando habilitado. Usa resposta de alto-falante parametrizada por `CabAcousticConditioning` (passa-altas e passa-baixas de 12 dB/oit, ressonância do `ImpedanceCurve`). Ganho unitário (±0,5 dB) com defaults. Convolução de IR carregada fica fora desta spec.
7. **Saída segura**: limitador/saturador final, no fim do `RealtimeDspChain` (depois do cab, fora do `WhiteBoxAmpProcessor`), garante |y| ≤ 1,0 (0 dBFS) em qualquer combinação de controles; o clamp ±2,0 é removido.
8. **Aliasing controlado**: só os estágios não lineares (pré, potência, clipping do pedal) rodam com oversampling, usando o `HalfBandOversampler` existente (`oversampling.rs`) com buffers pré-alocados. O fator é escolhido por medição: 2× e 4× são medidos pelo método do CA-11 (aliasing, CPU por bloco e latência somada) e o resultado, com o fator escolhido, é registrado nas Notas antes do merge. O seletor `#oversamplingSelect` da UI continua sem efeito nesta spec.

## Critérios de aceite
- **CA-1:** Com o mesmo sinal de entrada, `bass`=0 vs `bass`=10 diferem em ≥ 6 dB de energia na banda 80–200 Hz, e `treble`=0 vs `treble`=10 diferem em ≥ 6 dB na banda 2–5 kHz (análise por DFT no teste).
- **CA-2:** `middle`=0 vs 10 difere em ≥ 4 dB na banda 400–800 Hz (mesmo método).
- **CA-3:** `overdrive`=0 vs 10 eleva o ganho de pequeno sinal do pré (entrada −40 dBFS) em ≥ 20 dB e a THD da saída em ≥ 10 pontos percentuais para entrada de 0,1 de pico.
- **CA-4:** Com `inputGain`=10 e `overdrive`=0, o ganho de pequeno sinal do pré (entrada −40 dBFS, medido da entrada do amp até a saída do estágio de pré) é **≥ +20 dB** em absoluto (hoje é no máximo 0 dB — `amp.rs:243` limita `inputGain/10` a 1,0); com `inputGain`=0 é ≤ −20 dB.
- **CA-5:** `AmpModelId::BritJcm800_2203` e `UsTwinBlackface65` produzem saídas diferentes para a mesma entrada e mesmos controles (RMS do espectro diferindo ≥ 1 dB em ao menos uma banda de oitava); idem para todos os 10 pares dos 5 modelos.
- **CA-6:** `DriveProcessor` está na cadeia; com `drive_enabled=false` a saída é bit-idêntica à cadeia sem o bloco, e com `true` + `drive_gain` 0 vs 10 a THD cresce ≥ 10 pontos percentuais; `drive_tone` 0 vs 10 muda a energia > 2 kHz em ≥ 3 dB; `drive_level` 0 vs 10 muda o nível de saída do bloco em ≥ 12 dB.
- **CA-7:** `depth` 0 vs 10 muda a energia em 65–130 Hz em ≥ 3 dB **e** não altera a energia > 1 kHz em mais de 0,5 dB; `presence` 0 vs 10 muda > 2 kHz em ≥ 3 dB e < 500 Hz em ≤ 0,5 dB.
- **CA-8:** Com defaults, o `VirtualCabProcessor` entrega ganho de pequeno sinal de 0 dB ±0,5 dB (hoje 2,000× = +6 dB) e atenua ≥ 12 dB em 8 kHz relativamente a 1 kHz; com defaults o slot 2 nasce desabilitado (`mic_slot(1).enabled == false`), e alterar fase, delay ou nível do slot 2 enquanto desabilitado deixa a saída bit-idêntica.
- **CA-9:** Para uma varredura de 1.000 combinações pseudo-aleatórias (semente fixa) de todos os controles com entrada senoidal de 0,3 de pico, o pico de saída fica ≤ 1,0 (0 dBFS) em 100 % dos casos e não há amostra não finita.
- **CA-10:** Sensibilidade preservada — com os controles de `main.rs` (gate −65 dB, boost 2,5, gain 8, master 7) e entrada senoidal de pico 0,01 (≈ captador passivo medido em −49 dB RMS), o pico de saída fica entre 0,10 e 0,50 (hoje 0,23; evita "corrigir" o clipping deixando o sinal mudo).
- **CA-11:** Aliasing: seno de 5 kHz de pico 0,3 a 48 kHz, `inputGain`=10, `overdrive`=10; a soma da energia das componentes não harmônicas (bins fora de ±1 bin de k·5 kHz) em 20 Hz–20 kHz é ≤ −45 dBc com o fator escolhido em §8, **e** ao menos 15 dB menor que a mesma medição com o oversampling desligado (1×, hook de teste).
- **CA-12:** Caso de erro — `AmpModelId` desconhecido por nome retorna `Err` (já existente em `find_model_by_name`) e a cadeia mantém o modelo anterior; valores `NaN`/fora de faixa mantêm o clamp/sanitização (Spec 003 CA-7).
- **CA-13:** Não-regressão — os testes das Specs 003, 005, 006 e 013 continuam verdes **sem alteração** (inclusive `test_ca_04_*` e `test_ca_05_*` da Spec 003: master 2→9 eleva THD, `outputLevel` não muda THD, depth eleva 100 Hz, presence eleva 5 kHz — todos compatíveis com §2, §4 e §7); coeficientes de bright/presence/depth em 44,1 kHz e 96 kHz dão resposta em frequência dentro de ±1 dB da de 48 kHz nas bandas de controle.

## Fora de escopo
- Captura/medição de amplificadores reais, IRs comerciais ou modelagem circuit-level completa (MNA/WDF) dos estágios — hoje o amp é comportamental; a evolução para circuit-level é spec própria.
- Robustez do loader de IR (cabeçalho malformado) → **Spec 017**. Convolução da IR carregada no sinal ao vivo → spec futura, sem número reservado.
- Reverb/delay/modulação/pitch na cadeia ao vivo → futuras specs de integração (hoje só estão como blocos isolados).
- Ligação UI→motor dos controles → **Spec 014**.

## Invariantes tocadas
- Áudio em tempo real (Spec 001): filtros e oversampling com buffers pré-alocados; nada de alocação no `process`.
- Spec 003 CA-4/CA-5 (master/THD, presence/depth) e CA-7 (NaN): **mantidas**. Esta spec só acrescenta exigências (CA-1…CA-7) compatíveis com elas; o limitador de saída fica fora do amp para não afetar a medição de THD por `outputLevel` da Spec 003 CA-4. Se a implementação exigir mudar um teste da Spec 003, a Spec 003 é editada no mesmo PR com justificativa — nunca o teste enfraquecido em silêncio.
- Spec 005 CA-6 (condicionamento do gabinete): passa a ser efetivamente aplicado.

## Testes que cobrem esta spec
Planejados: `crates/ampforge-dsp/tests/contract_016_guitar_chain.rs` (CA-1…CA-13, análise por DFT/varredura com semente fixa).

## Notas
- Limites numéricos (CA-1…CA-4, CA-7, CA-8, CA-10, CA-11) são **hipóteses iniciais de projeto, sem medição de referência** de amplificador real; estão fixados para que o critério seja reprovável e o teste os usa como estão. Não descrevem fidelidade a um equipamento — o modelo continua comportamental e "aproxima" o 2203, sem equivalência declarada. Se a medição mostrar que um limite é inadequado, a spec é editada (com a medição que justifica) antes do merge, nunca o teste enfraquecido em silêncio.
- Slot 2 com a mesma configuração do slot 1, delay 0 e fase invertida cancela o sinal — é o comportamento físico esperado de dois microfones idênticos em contrafase, não defeito; por isso o CA-8 não o proíbe.
- O DC blocker de entrada (`audio.rs:498-500`, r = 0,995) tem corte ≈ 38 Hz a 48 kHz (−1,3 dB em 65 Hz, ver Spec 013 Notas). O CA-7 é diferencial (depth 0 vs 10 no mesmo sinal), então não é afetado; medições absolutas de grave na cadeia completa devem descontá-lo.
- Valores de componentes de tone stack: nenhum será inventado — sem esquema/revisão citada, usa-se parametrização genérica TMB marcada como hipótese.
