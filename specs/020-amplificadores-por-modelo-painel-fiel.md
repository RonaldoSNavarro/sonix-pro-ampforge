---
spec: 020
titulo: Trocar o modelo de amplificador não muda o som nem o painel, e os rótulos dos modelos têm dados a conferir
status: rascunho
atualizado: 2026-11-09
---
# 020 — Amplificadores por Modelo com Painel e Resposta Fiéis à Referência

## Objetivo
Fazer cada amplificador do seletor soar e se comportar como uma aproximação documentada do seu
circuito de referência — com os controles que o equipamento real tem — e trocar de modelo sem
estalo.

## Contexto
Origem: pedido do usuário em 2026-11-09 ("todos os controles, botões, pedais e amplificadores
presentes na interface devem funcionar realmente e serem parecidos com os reais, deve seguir a
skill"). Evidência:

- `#ampModelSelect` (`index.html:147-151`) oferece 4 modelos e **não tem nenhum handler** em
  `app.js` (0 ocorrências): trocar o modelo não muda nada. O preset `ac30` (`index.html:36`) usa um
  quinto modelo que existe no motor (`AmpModelId::UkAc30TopBoost`, `amp.rs:5-11`) mas não no
  seletor.
- O painel tem os mesmos 8 knobs + BRIGHT para todos os modelos (`app.js:18-28, 180-215`); amps
  reais não têm esse conjunto (ex.: Plexi e Twin não têm master; nenhum dos cinco tem "depth").
- `amp.rs:240-307` ignora `model_id`; a Spec 016 CA-5 exige que modelos soem diferentes, mas não
  define **o que** cada modelo deve reproduzir.
- `#oversamplingSelect` (`index.html:127-131`) só muda um cálculo de CPU estimado
  (`app.js:690-693, 565-579`).
- Rótulos a conferir (skill §2.4-2.5): "US Twin Blackface '65 (2x 6L6GC Clean)" — a documentação
  usual do Twin Reverb AB763 indica **quatro** 6L6GC; "Cali Dual Rectifier Rev-F (High Gain
  6L6/GZ34)" — a documentação usual indica retificação por **duas 5U4G** (comutável para diodos),
  não GZ34. Ambos ficam marcados "a conferir com a fonte" até o PR anexar o documento.

## Comportamento esperado
1. **Modelo selecionável ao vivo.** `amp_1.model` troca o modelo com crossfade de 20 ms entre a
   instância antiga e a nova (ambas pré-alocadas); o seletor passa a ter os 5 modelos do motor.
2. **Painel por modelo.** O painel mostra só os controles que o equipamento de referência tem, com
   os rótulos dele, mapeados às chaves genéricas da Spec 014. Controles ausentes ficam ocultos e
   fixos no valor "ausente" do modelo (ex.: sem master = estágio de potência sem atenuação antes
   do inversor, como no circuito real). Conjunto **proposto** (hipótese a confirmar com manual ou
   esquema da revisão citada; o PR anexa a fonte):

   | `amp_1.model` | Referência | Controles no painel → chave | Ausentes (valor fixo) |
   |---|---|---|---|
   | `jcm800_2203` | Marshall JCM800 2203 | Pre-Amp Volume → `gain`; Master → `master`; Treble, Middle, Bass, Presence | `overdrive` (0), `depth` (neutro), `bright` (sem chave) |
   | `plexi_1959` | Marshall 1959 Super Lead, canais em jumper | Loudness I (canal High Treble) → `gain`; Loudness II (Normal) → `volume_2`; Treble, Middle, Bass, Presence | `master` (ausente), `overdrive`, `depth`, `bright` |
   | `twin_ab763` | Fender Twin Reverb AB763 (canal Vibrato) | Volume → `gain`; Treble, Middle, Bass; Bright | `master` (ausente), `presence`, `depth`, `overdrive` |
   | `dual_rect` | Mesa/Boogie Dual Rectifier (revisão a fixar) | Gain → `gain`; Master → `master`; Treble, Mid, Bass, Presence | `overdrive`, `depth`, `bright` |
   | `ac30_tb` | Vox AC30 Top Boost (revisão a fixar) | Volume → `gain`; Treble, Bass; Cut → `cut` | `middle`, `presence`, `depth`, `master`, `overdrive`, `bright` |

   Chaves novas no registro da Spec 014 §2: `amp_1.model` (enum dos 5 valores, boot
   `jcm800_2203`), `amp_1.volume_2` (número 0–10, boot 0, suavizado), `amp_1.cut` (número 0–10,
   boot 5, suavizado). Chave enviada que o modelo atual não tem → não altera estado e volta em
   `ignored` com motivo `not_on_model`.
3. **Estrutura por modelo** (abordagem híbrida, skill §3 etapa D): para cada modelo, número de
   estágios de ganho do pré, cortes de acoplamento entre estágios, posição e topologia do tone
   stack (FMV com valores do modelo; Top Boost com topologia própria, não FMV — skill §5.2/§6),
   presença ou não de realimentação negativa global (presence/cut), tipo e quantidade de válvulas
   de potência e retificador padrão. Cada escolha cita a fonte na tabela de Notas; sem fonte, é
   marcada como hipótese e o modelo não pode fechar a spec.
4. **Tone stack fiel ao circuito.** A resposta do tone stack de cada modelo é a discretização da
   função de transferência do circuito citado (ref.: Yeh & Smith, DAFx-06), com a interação entre
   os controles preservada e o taper dos potenciômetros do circuito — o knob 0–10 nunca vira dB
   linear (skill §5.2).
5. **Bright dependente do volume.** Onde existe capacitor de bright no potenciômetro de volume
   (Twin com Bright ligado; Loudness I do 1959; Pre-Amp Volume do 2203 se a fonte confirmar o
   capacitor), o realce de agudos diminui à medida que o volume
   sobe, como no circuito real (Spec 003 CA-3).
6. **Sem master = potência dirigida pelo volume.** Em `plexi_1959` e `twin_ab763`, subir o volume
   aumenta a saturação do estágio de potência; o limitador de saída da Spec 016 continua garantindo
   0 dBFS.
7. **Oversampling selecionável.** `engine.oversampling` (enum `2x`, `4x`, `8x`; boot = fator
   escolhido pela medição da Spec 016 §8) troca o fator dos estágios não lineares com crossfade;
   a latência algorítmica resultante entra na telemetria da Spec 018.

## Critérios de aceite
- **CA-1:** Para cada modelo com fonte registrada nas Notas, a magnitude do tone stack do DSP, de
  50 Hz a 10 kHz, fica a ±1 dB da resposta analítica contínua do circuito citado, em três ajustes
  (todos em 5; bass 0/mid 10/treble 0; todos em 10) e nas taxas 44,1, 48 e 96 kHz (até 0,8 ×
  Nyquist).
- **CA-2:** Trocar `amp_1.model` com seno de 1 kHz na entrada produz
  `max |y[n] − y[n−1]|` na transição ≤ 1,1 × o maior valor em regime dos dois modelos; a troca
  termina em 20 ms ± 1 bloco; 0 alocações no callback durante a troca.
- **CA-3:** Na UI, selecionar cada modelo mostra exatamente os controles e rótulos da tabela do §2
  (teste JS), e arrastar um controle envia a chave mapeada; controles ocultos não enviam nada.
- **CA-4:** `POST {"amp_1.master": 3}` com `plexi_1959` ativo retorna `200`, lista a chave em
  `ignored` com `not_on_model` e o estado não muda; `{"amp_1.model":"jcm900"}` retorna `400`.
- **CA-5:** `twin_ab763` com Bright ligado vs desligado: diferença de energia em 3–5 kHz ≥ 3 dB com
  `gain`=3 e ≤ 1 dB com `gain`=10.
- **CA-6:** Em `plexi_1959` e `twin_ab763`, THD da saída do estágio de potência com `gain`=10 é
  maior que com `gain`=5 (entrada senoidal 0,1 de pico, 1 kHz), e o pico de saída da cadeia
  continua ≤ 1,0.
- **CA-7:** O seletor `#ampModelSelect` tem 5 opções, e carregar o preset `ac30` seleciona
  `ac30_tb`.
- **CA-8:** Mudar `engine.oversampling` de `2x` para `8x` não reduz a supressão de aliasing medida
  pelo método da Spec 016 CA-11 e aumenta `algorithmic_latency_samples` informado em
  `GET /api/audio/status`; a troca não gera descontinuidade (mesma régua do CA-2).
- **CA-9:** Não-regressão — testes das Specs 003, 014 e 016 continuam verdes.

## Fora de escopo
- Reverb e tremolo do Twin (o "Vibrato" da Fender é modulação de amplitude — skill §7.2), canal
  Normal do Twin e canais/modos adicionais do Dual Rectifier e do AC30: sem controles na UI hoje;
  spec futura.
- Modelagem circuit-level completa (WDF/MNA) dos estágios — a evolução fica para spec própria
  depois de medir custo de CPU com placa onboard.
- Comparação com amplificador real gravado: o usuário não tem interface para capturar referência;
  a validação objetiva desta spec é contra a resposta do circuito citado.

## Invariantes tocadas
- Áudio em tempo real (Spec 001/018): instâncias dos modelos pré-alocadas; troca por crossfade.
- Spec 016 CA-5/CA-9 (modelos diferentes; saída ≤ 0 dBFS): mantidas.
- Skill §2.5/§2.8: nenhum valor de componente sem fonte; nenhuma alegação de equivalência — o texto
  da UI diz "inspirado em"/"aproxima", nunca "idêntico".

## Testes que cobrem esta spec
Planejados (red → green): `crates/ampforge-dsp/tests/contract_020_amp_models.rs` (CA-1, CA-2, CA-5,
CA-6, CA-8), `crates/sonix-desktop/tests/contract_020_amp_models_api.rs` (CA-4, CA-9) e
`tests/contract/020-amplificadores-por-modelo-painel-fiel.test.js` (CA-3, CA-7).

## Notas
- Tabela de fontes a preencher no PR (modelo → documento, revisão, data): esquema/manual do
  JCM800 2203, do 1959 Super Lead (revisão do ano), do Fender AB763, do AC30 Top Boost e do Dual
  Rectifier (revisão). Sem documento, o modelo usa parametrização genérica marcada como hipótese e
  a spec não pode ir a `implementada`.
- Os números de CA-1 (±1 dB) e CA-5 (3 dB / 1 dB) são hipóteses de projeto, editáveis com medição.
- Nomes comerciais aparecem como referência técnica do circuito; a UI usa os nomes já definidos na
  Spec 011.
