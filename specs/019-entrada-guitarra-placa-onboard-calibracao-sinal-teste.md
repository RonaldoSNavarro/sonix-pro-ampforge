---
spec: 019
titulo: Ganho de entrada e gate fixos no código, sem indicação de clipping, e sinal de teste que não passa pelo motor
status: rascunho
atualizado: 2026-11-09
---
# 019 — Entrada de Guitarra na Placa Onboard: Calibração, Clipping e Sinal de Teste pelo Motor

## Objetivo
Fazer a guitarra ligada na entrada de microfone da placa onboard chegar ao motor no nível certo,
sem clipping no conversor e com o gate ajustado ao ruído real do PC do usuário — e permitir testar
o som sem tocar, com um sinal que passa pelo mesmo motor da guitarra.

## Contexto
Origem: pedido do usuário em 2026-11-09 (placa onboard, sem interface de áudio) e Spec 013
(Notas: captador passivo medido em ~−48,9 dB RMS na entrada P2). Evidência:

- `main.rs:41-42` fixa `gate` em −65 dB e `boost` em 2,5× a partir de **uma** medição registrada
  na Spec 013; outro PC, outro captador ou outro nível de "mic boost" do Windows invalidam os dois.
- Não existe indicador de clipping no conversor: `update_input_peak` (`audio.rs:188,219`) só guarda
  o pico em dB; nada conta amostras no fundo de escala.
- O botão "▶ Test Signal DSP" (`app.js:760-767`) toca um dente de serra sintetizado pelo **Web
  Audio do navegador**, com `WaveShaper` e passa-baixas próprios (`app.js:810-870`): não passa
  pelo motor Rust, então não testa nada da cadeia — e soa por cima dela.
- O afinador da UI usa a forma de onda de **saída** (`app.js:876-907`, telemetria de
  `audio.rs:266`) — tratado na Spec 026.

Base técnica (skill do projeto §8.2): o captador passivo é uma fonte de alta impedância
(indutância e capacitância próprias); uma entrada de microfone de PC tem impedância muito menor que
a de entradas de instrumento de amplificadores (tipicamente da ordem de 1 MΩ). A carga baixa
amortece a ressonância do captador e reduz agudos e nível — **isso acontece antes do conversor e
software nenhum recupera totalmente**. A entrada de microfone também fornece tensão de
polarização (plug-in power), já tratada pelo DC blocker da Spec 013. Hum de rede (60 Hz no
Brasil, 50 Hz em outros países) e ruído do PC entram junto com o sinal.

## Comportamento esperado
1. **Assistente "Calibrar entrada"** (`#inputCalibrationWizard`), em três passos:
   (a) **silêncio** — guitarra ligada, volume da guitarra no máximo, cordas abafadas, 3 s: mede
   RMS e pico do piso de ruído e detecta hum (energia em 50/60 Hz e 2ª/3ª harmônicas acima do
   ruído de banda larga); (b) **tocar forte** — 5 s de acordes com força: mede o pico;
   (c) **resultado** — propõe `input.boost` para que o pico medido fique em **−12 dBFS** depois do
   boost (headroom de projeto, hipótese ajustável) e `gate.threshold_db` = piso de ruído RMS depois
   do boost **+ 6 dB** (hipótese ajustável); o usuário aceita ou descarta. Os valores ficam salvos
   por dispositivo de entrada e são aplicados no boot no lugar das constantes de `main.rs`.
2. **Diagnósticos do assistente.** Pico bruto (antes do boost) ≥ −1 dBFS → `input_too_hot`: o
   assistente orienta a baixar o nível/"aumento de microfone" do Windows e não propõe boost > 1×.
   Pico que não chega a −24 dBFS nem com o boost máximo (8×) → `input_too_low`: orienta a subir o
   "aumento de microfone" do Windows. Hum detectado → `hum_50hz`/`hum_60hz` com orientação
   (afastar a guitarra do monitor/fonte, testar o PC na tomada aterrada).
3. **Indicador de clipping do conversor.** Cada amostra de entrada com |x| ≥ 0,999 (antes do boost)
   incrementa `adc_clips` (atômico); o VU de entrada (`#vuInFill`) fica vermelho por 1 s após
   qualquer clip.
4. **Orientação de hardware.** Painel de ajuda com: desligar "aprimoramentos"/AGC/supressão de
   ruído do microfone no Windows/Realtek (ou usar o modo exclusivo ou `RAW` da Spec 018), usar a
   entrada de linha quando o PC tiver uma, e usar um pedal com buffer antes do PC para reduzir a
   perda de agudos — redigido como recomendação, sem prometer resultado numérico.
5. **Fonte de entrada do motor.** `input.source` escolhe o que alimenta a cadeia:
   `device` (guitarra), `test_riff` (DI sintética gerada **no motor**: corda dedilhada por síntese
   de Karplus-Strong, riff fixo, semente fixa, pico −12 dBFS) ou `di_file` (WAV de DI carregado
   pelo usuário, tocado em laço). O botão "Test Signal DSP" passa a alternar entre `device` e
   `test_riff`; o sintetizador Web Audio é removido.
6. **Parâmetros** (registro da Spec 014 §2):

   | Chave | Tipo | Valores / faixa | Valor no boot | Aplicação |
   |---|---|---|---|---|
   | `input.source` | enum | `device`, `test_riff`, `di_file` | `device` | crossfade de 10 ms |

   O arquivo de DI chega por `POST /api/input/di` (corpo WAV, ≤ 20 MB) e é validado com as
   regras do loader da Spec 017 (canais, taxa, sem pânico); se a taxa diferir da do motor, é
   reamostrado fora do callback.

## Critérios de aceite
- **CA-1:** Calibração sobre entrada simulada com piso de ruído RMS de −70 dBFS e rajadas com pico
  de −30 dBFS propõe `input.boost` = 7,94 ± 0,05 (pico vai a −12 dBFS) e
  `gate.threshold_db` = −46 ± 0,5 dB (−70 + 18 + 6).
- **CA-2:** Pico bruto de −0,5 dBFS resulta em `input_too_hot` e boost proposto ≤ 1,0; pico bruto
  de −60 dBFS resulta em `input_too_low` e boost proposto = 8,0.
- **CA-3:** Piso de −80 dBFS com 60 Hz + 180 Hz a −50 dBFS → `hum_60hz`; com 50 Hz + 150 Hz →
  `hum_50hz`; só ruído branco → nenhum diagnóstico de hum.
- **CA-4:** Um bloco de entrada com 5 amostras em 1,0 incrementa `adc_clips` em 5 (exposto em
  `GET /api/audio/status`); na UI, `#vuInFill` recebe a classe de clip por 1 s ± 100 ms (teste JS
  com relógio falso).
- **CA-5:** Com `input.source=test_riff` e dispositivo de entrada em silêncio, a saída do motor não
  é silenciosa (RMS > −40 dBFS) e passa pela cadeia (desligar `amp_1.enabled` muda a saída). Na
  UI, clicar em "Test Signal DSP" envia `input.source` e **não** cria `AudioContext` (teste JS com
  construtor espionado).
- **CA-6:** Duas gerações de `test_riff` com a mesma semente são bit-idênticas; o pico do riff é
  −12 dBFS ± 0,5 dB.
- **CA-7:** `POST /api/input/di` com WAV mono PCM16 de 44,1 kHz em motor a 48 kHz é aceito e
  reamostrado (duração preservada ± 1 ms); WAV com `channels = 0` ou corpo > 20 MB retorna `400`
  ou `413`, a fonte anterior continua tocando e não há pânico.
- **CA-8:** Resultados aceitos ficam salvos por id de dispositivo; no boot seguinte com o mesmo
  dispositivo, `GET /api/state` devolve os valores calibrados; com dispositivo desconhecido,
  devolve os valores no boot da tabela da Spec 014.
- **CA-9:** Não-regressão — sem calibração salva e com `input.source=device`, o motor se comporta
  como antes desta spec (valores no boot da Spec 014) e os testes das Specs 013, 014 e 018 seguem
  verdes.

## Fora de escopo
- Filtro de compensação da carga da entrada P2 sobre o captador: exige medir a impedância de
  entrada da placa do usuário; sem medição seria um EQ inventado (skill §2.5, §2.9). Fica como
  hipótese para spec futura, condicionada a medição.
- Reamp para amplificador externo e entrada Hi-Z/linha dedicadas (backlog da skill, Épico D):
  exigem interface de áudio.
- Ajuste automático do "aumento de microfone" do Windows: o app orienta, não altera
  configurações do sistema.

## Invariantes tocadas
- Áudio em tempo real (Spec 001/018): medições do assistente rodam sobre cópias enviadas pela
  telemetria; síntese do `test_riff` é pré-calculada fora do callback; nada aloca no callback.
- Spec 017: o mesmo validador de WAV protege o carregamento de DI.

## Testes que cobrem esta spec
Planejados (red → green): `crates/sonix-desktop/tests/contract_019_input_calibration.rs` (CA-1…CA-4,
CA-5 lado Rust, CA-6…CA-9) e `tests/contract/019-entrada-guitarra-placa-onboard-calibracao-sinal-teste.test.js`
(CA-4 lado UI, CA-5 lado UI).

## Notas
- −12 dBFS de pico e +6 dB acima do ruído são pontos de partida de projeto, não medições; a spec é
  editada (com a medição que justificar) se a calibração no PC do usuário mostrar valores melhores.
- Depende da Spec 018 (canal de entrada, telemetria, status) e da Spec 014 (registro de parâmetros).
