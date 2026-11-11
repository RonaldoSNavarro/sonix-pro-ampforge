---
spec: 022
titulo: Sala do gabinete — curva de impedância, ressonância, corte, microfones e IR carregada — não altera o som
status: implementada
atualizado: 2026-11-10
---
# 022 — Gabinete, Microfones e Impulse Response ao Vivo, sem Latência Adicional

## Objetivo
Fazer a sala do gabinete funcionar como num estúdio: escolher o falante/gabinete, posicionar dois
microfones e carregar uma IR própria muda o som de forma fisicamente coerente — e a convolução da
IR não acrescenta atraso.

## Contexto
Origem: pedido do usuário em 2026-11-09 (controles reais, parecidos com os reais, sem atraso).
Evidência:

- `#impedanceCurveSelect` e `#irFileInput` não têm handler em `app.js` (0 ocorrências);
  `sliderCabRes` (65–135 Hz) e `sliderCabHiCut` (3,5–18 kHz) só gravam em `state.circuitParams`
  (`app.js:730-731`). Os textos "(+6.8dB)" (`index.html:402`) e "Nível: -3.5 dB | Delay: 0.08 ms"
  (`index.html:397`) são fixos.
- Arrastar os microfones (`app.js:584-628`) calcula "Cap-Edge %" pela distância ao centro, mas os
  "cm" saem da **posição vertical** na tela (`yPct/100 × 12`), sem relação com distância ao cone.
- No motor, `VirtualCabProcessor::process` não lê `custom_ir`, `conditioning`, `impedance_curve`
  nem `model` (Spec 016 Contexto, `cab.rs:241,307-365`); o loader de IR reamostra por interpolação
  linear (`cab.rs:214-224`) e lê só o canal 0 (`cab.rs:182-198`).
- `MicSlotConfig` já tem modelo, posição cap→borda, distância, ângulo, nível, fase, delay e
  habilitação (`cab.rs:16-45`); `MicModel` tem 7 modelos (`cab.rs:5-13`).
- Ressonâncias nos rótulos (98, 108, 82 Hz — `index.html:425-427`) não citam fonte; ressonância de
  falante em caixa difere da ressonância ao ar livre (Fs) do datasheet — **a conferir**.

Base técnica (skill §4, §5.5, §9): o alto-falante é uma carga reativa (pico de impedância na
ressonância, subida em agudos pela indutância da bobina) que interage com o estágio de potência —
mais visível com pouca realimentação; essa interação elétrica é distinta da resposta acústica do
gabinete e do microfone. Microfone no centro do cone capta mais agudos que na borda; microfones
direcionais têm efeito de proximidade (graves sobem perto da fonte); dois microfones a distâncias
diferentes somam com atraso de distância ÷ 343 m/s e geram filtro pente. Uma IR de gabinete já
contém falante + caixa + microfone + posição; ela não modela o amplificador nem não-linearidades.

## Comportamento esperado
1. **Falante/gabinete selecionável.** `cab_1.speaker` escolhe a curva de impedância (ressonância,
   Q, subida indutiva) e a resposta acústica do gabinete (fechado vs aberto: caixa aberta tem menos
   graves por cancelamento traseiro). A curva de impedância alimenta a carga do estágio de potência
   (acoplamento elétrico, Spec 021 CA-4); a resposta acústica vem depois, no bloco do cab. Valores
   de cada falante vêm de datasheet citado nas Notas; sem fonte, marcados como hipótese.
2. **Ajustes finos.** `cab_1.resonance_hz` desloca a ressonância da curva escolhida;
   `cab_1.hicut_khz` define o corte de agudos (−3 dB no valor do slider, 12 dB/oitava). Os textos
   da UI mostram os valores aplicados pelo motor, não constantes.
3. **Dois microfones.** Cada slot tem modelo, posição cap→borda (distância do handle ao centro do
   cone, 0 = centro, 1 = borda), distância ao cone em cm (controle próprio, não a posição vertical),
   nível e fase; o slot 2 tem liga/desliga e nasce desligado (Spec 016 CA-8). A resposta de cada
   modelo de microfone vem da curva publicada pelo fabricante (citada); sem fonte, resposta plana
   marcada como hipótese. A distância gera atraso físico (d ÷ 343 m/s) e, para modelos
   direcionais, efeito de proximidade.
4. **IR do usuário.** Arquivo de `#irFileInput` vai a `POST /api/cab/ir` (≤ 10 MB), é validado
   pelas regras da Spec 017, reamostrado para a taxa da sessão com filtro de qualidade (sinc
   janelado) **fora do callback**, e substitui falante + gabinete + microfones do slot (a IR já os
   contém — a UI desabilita os controles de microfone e explica o motivo). Ganho original
   preservado (sem normalização por padrão; opção "normalizar" explícita). IR com mais de um canal
   usa o canal escolhido pelo usuário (padrão: 1º) e avisa. Duração máxima de 500 ms (limite de
   projeto por CPU; maior → erro explícito, sem cortar em silêncio — skill §9.2).
5. **Convolução sem latência adicional.** Início da IR por FIR direta (cobrindo ao menos um período)
   e restante por convolução particionada por FFT; a saída começa no mesmo índice de amostra que a
   convolução direta. Troca de IR com crossfade de 20 ms, preparação fora do callback.
6. **Parâmetros** (registro da Spec 014 §2; `<n>` = 1 ou 2):

   | Chave | Tipo | Faixa / valores | Valor no boot |
   |---|---|---|---|
   | `cab_1.speaker` | enum | `greenback_4x12_closed`, `v30_4x12_closed`, `alnico_blue_2x12_open` | `greenback_4x12_closed` |
   | `cab_1.resonance_hz` | número | 65–135 | nominal do falante |
   | `cab_1.hicut_khz` | número | 3,5–18 | 8,2 |
   | `cab_1.mic<n>.model` | enum | `sm57`, `md421`, `m88`, `r121`, `m160`, `u87`, `c414` | `sm57` |
   | `cab_1.mic<n>.position` | número | 0–1 | 0,2 |
   | `cab_1.mic<n>.distance_cm` | número | 0–30 | 2,0 |
   | `cab_1.mic<n>.level_db` | número | −24–+6 | 0 |
   | `cab_1.mic<n>.phase_invert` | booleano | — | `false` |
   | `cab_1.mic2.enabled` | booleano | — | `false` |
   | `cab_1.ir_enabled` | booleano | — | `false` (vira `true` ao carregar IR válida) |

## Critérios de aceite
- **CA-1:** Para cada `cab_1.speaker`, a curva de impedância devolvida por
  `GET /api/scope?mode=tonestack` tem máximo em `resonance_hz` ± 2 %; com `amp_1.nfb`=0 a saída do
  estágio de potência mostra realce local na mesma frequência ± 5 %.
- **CA-2:** O ponto de −3 dB da resposta acústica fica em `hicut_khz` ± 10 % para 4, 8 e 16 kHz.
- **CA-3:** `alnico_blue_2x12_open` tem energia em 60–120 Hz ≥ 3 dB menor que
  `greenback_4x12_closed` para a mesma entrada, normalizado em 1 kHz.
- **CA-4:** Microfone `sm57` com `position` 1,0 tem energia em 4–8 kHz ≥ 6 dB menor que com 0,0, e
  a energia cai monotonicamente em 0; 0,25; 0,5; 0,75; 1,0.
- **CA-5:** `sm57` a 1 cm tem (energia em 100 Hz − energia em 1 kHz) ≥ 3 dB maior que a 15 cm.
- **CA-6:** Dois microfones iguais, mesma posição, mic 2 com `distance_cm` 3,43 cm maior que mic 1:
  a soma tem o primeiro vale em 5 kHz ± 5 % (atraso de 0,1 ms).
- **CA-7:** Na UI, arrastar um microfone envia `cab_1.mic<n>.position` proporcional à distância ao
  centro do cone e não altera `distance_cm`; o controle de distância envia `distance_cm`; os textos
  de nível, delay e ressonância mostram valores de `GET /api/state` (teste JS).
- **CA-8:** IR de 48 kHz com 4.096 amostras: a saída do bloco difere da convolução direta de
  referência em ≤ −80 dB (erro relativo RMS) e um impulso na entrada produz `IR[0]` no mesmo índice
  (0 amostras de latência adicional).
- **CA-9:** IR de 44,1 kHz em sessão de 48 kHz: magnitude da IR reamostrada a ±0,5 dB da original
  até 18 kHz; a mesma IR com interpolação linear (comportamento atual) falha esse critério.
- **CA-10:** Sem normalização, ruído branco através da IR tem ganho de energia igual à energia da
  IR ± 0,1 dB; com "normalizar" ligado, ganho 0 dB ± 0,1 dB.
- **CA-11:** IR com 600 ms retorna `400` com `ir_too_long` e o limite; IR com `channels = 0`
  retorna `400` sem pânico; em ambos a IR anterior continua ativa.
- **CA-12:** Trocar de IR e ligar/desligar `ir_enabled` não gera descontinuidade (régua da
  Spec 020 CA-2); 0 alocações no callback.
- **CA-13:** Com IR ativa, chaves `cab_1.mic*` voltam em `ignored` com `ir_active` e os controles de
  microfone ficam desabilitados na UI.
- **CA-14 (manual, registrado no PR):** No PC do usuário, IR de 500 ms a 48 kHz com o menor período
  estável da Spec 018: `cpu_avg_pct` e `underruns` registrados nas Notas; aceite: 0 underruns em
  5 min.
- **CA-15:** Não-regressão — testes das Specs 005, 016 e 017 continuam verdes.

## Fora de escopo
- Saída estéreo, pan e microfone de sala (`room_mic_early_reflections_pct`): saída atual é mono
  duplicada (`audio.rs:258-263`); spec futura.
- Ângulo off-axis (`angle_off_axis_deg`) e solo/mute de microfone: sem controle na UI.
- Cab 2 (`blk_cab_2`): segue o mesmo contrato com prefixo `cab_2.` e entra com o caminho paralelo
  da Spec 024.

## Invariantes tocadas
- Áudio em tempo real (Spec 001/018): FFTs, partições e reamostragem preparadas fora do callback;
  latência do bloco declarada (0 amostras) e somada na telemetria.
- Spec 005: condicionamento, curva de impedância e IR passam a ser aplicados de fato.
- Spec 017: validação de WAV reutilizada.

## Testes que cobrem esta spec
Planejados (red → green): `crates/ampforge-dsp/tests/contract_022_cab_mic_ir.rs` (CA-1…CA-6,
CA-8…CA-12, CA-15), `crates/sonix-desktop/tests/contract_022_cab_api.rs` (CA-11, CA-13) e
`tests/contract/022-gabinete-microfones-ir-ao-vivo.test.js` (CA-7, CA-13 lado UI). CA-14 manual.

## Notas
- Fontes a anexar no PR: datasheets dos falantes (Fs, Q, curva de impedância) e curvas de resposta
  publicadas dos microfones. Os rótulos de ressonância da UI são corrigidos para o valor da fonte.
- 500 ms, 6 dB, 3 dB e ±2/5/10 % são hipóteses de projeto, editáveis com medição.
- Depende das Specs 016 (cab unitário, saída segura), 017 (loader) e 021 (realimentação).
