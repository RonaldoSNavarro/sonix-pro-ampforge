---
spec: 047
titulo: O osciloscópio "ao vivo" desenha curvas sintéticas e parte da bancada não chega ao motor
status: implementada
atualizado: 2026-11-14
---
# 047 — Bancada e osciloscópio com dados do motor (transferência, forma de onda, tone stack, impedância)

## Objetivo
Tudo que a bancada mostra (curvas do CRT, válvulas, B+) sai do estado real do motor, e todo controle da bancada muda o som.

## Contexto
Review manual das Specs 004 e 021 (2026-11-14), por leitura de código:

| Modo / elemento | Hoje | Evidência |
|---|---|---|
| CRT "transfer" | Usa `x/(1+\|x\|)` com assimetria linear. O amp usa `u/√(1+(k·u)²)` com joelhos por modelo e válvula, e vários estágios. A curva mostrada não é a do som. | `api.rs:740-763` vs `amp.rs:545-576` |
| CRT "waveform" | Uma senoide sintética e `tanh(1,5·s)`, fixas; não vem do sinal. | `api.rs:765-781`; idem `modshop.rs:416-434` |
| CRT "tonestack" | Uma gaussiana fixa em 650 Hz; ignora topologia e knobs, embora `analog_tonestack_magnitude_db` exista. | `api.rs:783-798`; `tonestack.rs:280-301` |
| Curva de impedância do falante | Lorentziana fixa em 98 Hz, independente do gabinete escolhido. | `api.rs:790` |
| Parâmetros da bancada | `ChassisWorkbench` tem `triode_hardness`, `phase_inverter_drive`, `b_plus_time_constant_ms`, `power_class`, `interstage_hp_cut_hz`, `tonestack_position` e `ghost_note_filtering_pct`, que não existem em `ChassisModSettings` e não chegam ao amp. | `modshop.rs:189-210`; `amp.rs:273-287` |
| Válvulas na tela Estúdio | Sempre 4× "12AX7", seja qual for o modelo; o B+ é a fórmula `plate_v·1,5 − GR·2`. | `ui/src/screens/Estudio.ts:450-477` |

## Comportamento esperado
- **Transfer:** amostra a função estática do estágio de pré realmente usado (modelo, válvula, assimetria e tensão de placa atuais), calculada pelo mesmo código do DSP.
- **Waveform:** últimas N amostras de DI e de saída, lidas de rings lock-free alimentados pela cadeia.
- **Tonestack:** `analog_tonestack_magnitude_db` com a topologia e os knobs atuais.
- **Impedância:** a curva do gabinete atual (`impedance_curve`).
- Cada parâmetro da bancada visível na UI tem efeito no DSP com teste; os que não forem ligados não aparecem.
- As válvulas exibidas e o B+ vêm do modelo (complemento de válvulas do esquema citado) e do estado de sag do motor.

## Critérios de aceite
- **CA-1:** Para cada modelo, em 21 pontos x ∈ [−2, 2], o `y` do modo transfer é igual (≤ 1e-6) à função de estágio do DSP avaliada com os mesmos parâmetros. Mudar `amp_1.preamp_tube` de 12AX7 para 12AT7 muda a curva.
- **CA-2:** Com a fonte `test_riff`, o `output` do modo waveform tem correlação > 0,99 com o mesmo trecho capturado da saída do motor no teste. Com a entrada em silêncio, a curva é plana (nunca uma senoide).
- **CA-3:** No modo tonestack, com Bass 0 → 10, o nível em 100 Hz sobe pelo menos 6 dB, e a curva é igual a `analog_tonestack_magnitude_db` (≤ 0,01 dB) nos pontos enviados.
- **CA-4:** Trocar `cab_1.speaker` de `v30_4x12` para `alnico_blue_2x12` muda a frequência do pico de `speaker_z` de acordo com `default_impedance_curve` de cada um.
- **CA-5:** Todo controle da bancada no `controls.registry.ts` tem teste de efeito no áudio. Os campos do `ChassisWorkbench` não ligados ao amp não aparecem na UI (teste de inventário).
- **CA-6:** Com o AC30 selecionado, a tela Estúdio mostra 4 válvulas de potência EL84 e as de pré do esquema citado; com o Twin, 6L6GC. O B+ exibido acompanha o sag do motor (cai sob ataque forte e volta no silêncio).
- **CA-7 (erro):** Sem áudio chegando (stream parado), o CRT mostra "sem sinal", e não uma curva sintética.
- **CA-8 (não-regressão):** Os contratos das Specs 004 e 021 continuam passando, exceto os CAs que validavam as curvas sintéticas. Esses são revertidos e listados nas Notas.

## Fora de escopo
- O desenho do chassi aberto igual ao design fica na Spec 048.
- Medição de corrente real das válvulas: o motor é uma simulação, e o B+ é o estado do modelo de fonte.

## Invariantes tocadas
- **Zero-alloc no callback:** os rings do osciloscópio são pré-alocados; quem lê é a thread de streams (Spec 033).
- **Todo controle visível funciona:** CA-5.

## Testes que cobrem esta spec
<Preenchido no fechamento.>
