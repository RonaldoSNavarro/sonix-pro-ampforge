---
spec: 057
titulo: Amplificador Mesa/Boogie Dual Rectifier Solo Head (Rev. F, 1992, 100W)
status: rascunho
tipo: amplificador
atualizado: 2026-11-19
dependencias: [053, 077, 078, 079]
---
# 057 — Amplificador Mesa/Boogie Dual Rectifier Solo Head (Rev. F, 1992)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `amp.mesa.dual_rectifier_rev_f` (alias legado: `dual_rect`, `cali_dual_recto_revf`, `dual_rectifier`, `recto`)
- **Revisão (`model_revision`):** `1992-rev-f-solo-100`
- **Categoria:** Amplificador valvulado (módulo independente)
- **Objetivo:** Especificar de forma autônoma e verificável a arquitetura elétrica, controles, modos de canal (incluindo desconexão real de NFB em `Modern`), seleção de retificação (`Silicon Diode` vs `Dual 5U4GB`), chave variac (`Bold` vs `Spongy`), modelagem DSP e validação do **Mesa/Boogie Dual Rectifier Solo Head Revision F (1992)**.

## 2. Escopo
- Cascata high-gain de até 4 estágios de triodos `12AX7` + seguidor de cátodo com filtragem passa-altas inter-estágio dependente do modo.
- Tone Stack pós-ganho (`39 kΩ / 470 pF` ou `250 pF` — divergência documentada em §5/§7 — `/ 22 nF / 22 nF`), modos `clean`, `pushed`, `vintage`, `modern` (com desconexão de NFB global em `modern`).
- Comutação real entre retificação `Silicon Diode` (ataque rígido, baixo sag) e `Dual 5U4GB` (maior resistência interna e sag dinâmico pronunciado), e chave `Bold` vs `Spongy` (redução de tensão B+ tipo variac).
- Potência push-pull `4×6L6GC` (ou `EL34` via bias select / Mod Shop), transformador Schumacher 100 W e carga reativa.

## 3. Fora de escopo
- Dual Rectifier Multi-Watt / Road King (arquitetura de mistura progressiva de 6L6+EL34 por canal).
- Loop de efeitos paralelo/serial interno do chassi (gerenciado pelo DAG 4×12 da Spec 084).
- Simulação de gabinete Mesa 4×12 Rectifier V30 (coberta pela Spec 080).

## 4. Equipamento ou componente de referência
- **Fabricante:** Mesa/Boogie (Mesa Engineering).
- **Modelo exato:** Dual Rectifier Solo Head 100W.
- **Período / Ano:** 1992 (**Revision F**).
- **Papel na cadeia de sinal:** Referência de amplificador high-gain moderno de 100 W, caracterizado por saturação densa em cascata, graves massivos, ausência de NFB no modo Modern e resposta de fonte comutável.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1992 Revision F (2-Channel Solo Head, RF-1F)` — com extensão controlada do modo `Pushed` (originário da revisão 3-Channel) documentada abaixo.
- **Variantes históricas relevantes (não misturadas silenciosamente):**
  - *Rev C / D / E (1992):* variações de placa/transformador e circuito de loop serial.
  - *Rev F (1992, referência principal):* 2 canais (Orange Vintage / Red Modern + clonagem de canal), transformador Schumacher inicial, médios mais abertos e menos comprimidos que Rev G.
  - *Rev G (1994–2000):* mudanças no loop paralelo e filtragem de médios-graves.
  - *3-Channel Dual Rectifier (2000+):* introduz formalmente o modo `Pushed` no Canal 1 (`Clean / Pushed`) e 3 canais independentes (`Raw / Vintage / Modern`).
- **Pendências de confirmação de revisão:**
  1. **Modo `Pushed`:** já existe na UI/API (`controls.registry.ts:316`, Spec 031 CA-5) como 4º modo junto a `Clean/Vintage/Modern`; decidir em revisão manual se permanece como extensão documentada da Rev F (compatibilidade de presets) ou se será rotulado explicitamente como variante híbrida Rev F + modo Pushed (3ch).
  2. **Capacitor `C1` do Tone Stack:** `docs/amps/voicing-fontes.md` cita `250 pF` enquanto `tonestack.rs:101` implementa `470 pF` (valor da família Soldano/Rectifier 39k/470pF/22nF/22nF). **Conferir diretamente no esquema RF-1F antes de implementar a Spec 057** — marcado como `[pendente confirmação em esquema primário]`.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | *Mesa/Boogie Dual Rectifier Solo Head Owner's Manual* | 2ch / 3ch Solo Head | Controles, Bold/Spongy, Silicon/5U4GB, Vintage vs Modern | Mesa Engineering Manuals | 2026-11-19 | `[confirmado em documentação]` | Função de Bold/Spongy (variac), Diode vs 5U4GB, desconexão de NFB em Modern |
| F2 | Esquema Mesa/Boogie Dual Rectifier Preamp `RF-1F` & Power Amp | Rev F (1992) | Preamp / Power / Switching | Schematic Heaven / Archive | 2026-11-19 | `[fonte secundária]` | Cascata high-gain 12AX7, NFB desconectado em Modern, 4×6L6GC + Dual 5U4GB |
| F3 | `docs/amps/voicing-fontes.md` | Dual Rectifier Rev F (1992) | Seção 4 | `docs/amps/voicing-fontes.md` | 2026-11-19 | `[confirmado em documentação]` | Tone stack 39k, contorno 85 Hz / 750 Hz, `eff_nfb = 0` em Modern |
| F4 | `crates/ampforge-dsp/src/tonestack.rs` | Implementação atual | Linhas 94–107 | `crates/ampforge-dsp/src/tonestack.rs` | 2026-11-19 | `[aproximação DSP]` | Divergência registrada `C1 = 470 pF` vs `250 pF` em `voicing-fontes.md` |

## 7. Fundamentação elétrica
- **Pré-amplificação (5×12AX7 no chassi):**
  - Estágios em cascata com número efetivo dependente do modo: `Clean` (2 estágios, ganho moderado), `Pushed` (3 estágios, crunch), `Vintage` (4 estágios + NFB ativo + presença suave), `Modern` (4 estágios com acoplamento passa-altas mais apertado pré-saturação para controle de graves + NFB global desconectado) `[confirmado em documentação: F1, F2, F3]`.
  - Cold clipper / assimetria em estágio intermediário (`Rk = 39 kΩ` na topologia SLO/Recto — `[fonte secundária: F2]`, confirmar valor exato no esquema RF-1F) produzindo saturação densa e sustain prolongado.
  - Seguidor de cátodo alimentando o Tone Stack pós-ganho.
- **Tone Stack:** `R_slope = 39 kΩ`, `C1 = 470 pF` *(ou `250 pF` — confirmar em RF-1F)*, `C2 = 22 nF`, `C3 = 22 nF`, `R_treble = 250 kΩ`, `R_bass = 250 kΩ` (ou `1 MΩ` conforme revisão — confirmar em RF-1F), `R_mid = 25 kΩ`, `R_load = 1 MΩ`.
- **Estágio de Potência, NFB e Presença:** LTP + `4×6L6GC` (`B+ ≈ 470 V` em Bold), com comutação de realimentação negativa:
  - Em `Clean`, `Pushed` e `Vintage`: NFB global ativo; `Presence` atua dentro da malha NFB.
  - Em **`Modern`**: malha NFB global desconectada (`eff_nfb = 0`), liberando ganho de malha aberta do estágio de potência e maior interação com a impedância reativa do gabinete `[confirmado em documentação: F1, F3]`.
- **Fonte e Variac:**
  - `Rectifier`: `Silicon Diode` (baixa impedância interna, ataque imediato) vs `Dual 5U4GB` (resistência interna alta, queda B+ e compressão dinâmica sob transientes).
  - `Power`: `Bold` (tensão nominal plena ~100%) vs `Spongy` (redução de tensão ~15–20% simulando variac a ~95–100 VAC, menor headroom e compressão antecipada `[confirmado em documentação: F1]`).

## 8. Comportamento esperado
- **Separação dramática entre modos:** `THD(modern) > 3 × THD(clean)` na mesma entrada e `Gain = 6.5`; no modo `Modern`, a desconexão de NFB produz ganho de potência mais agressivo e reduz a atuação do `Presence` na malha NFB (`eff_nfb = 0`).
- **Efeito mensurável de `Spongy` vs `Bold`:** ativar `spongy = true` reduz `current_b_plus_v()` em ~15–20% e antecipa a compressão/saturação em picos de entrada (reduzindo nível de pico limpo em `~1,3–2,0 dB`).
- **Efeito mensurável de `5U4GB` vs `Diode`:** em bursts percussivos fortes, `5U4GB` produz queda dinâmica de B+ (`sag_state`) maior e constante de recuperação mais lenta (~120 ms vs ~20 ms) que `Silicon Diode`.
- **Controle de subgraves em alto ganho:** acoplamento HPF inter-estágio impede bloqueio de baixas frequências em palm-mutes pesados no modo `Modern`, preservando pico de peso em ~85 Hz via acoplamento transformador/carga.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Power | `amp_1.power` | boolean | bool | `false/true` | `true` | discreto | Alimentação AC | Mute suave `<= 2,5 ms` |
| Standby | `amp_1.standby` | boolean | bool | `false/true` | `false` | discreto | Corte B+ | Rampa exponencial invariante à taxa |
| Channel Mode | `amp_1.channel_mode` | enum | id | `clean, pushed, vintage, modern` | `modern` | discreto | Comuta estágios, acoplamento e NFB (`modern` → NFB=0) | Roteamento de estágios + chave NFB |
| Gain | `amp_1.gain` | number | scale | `0.0..=10.0` | `6.5` | audio log | Excitação da cascata de pré-amp | Ganho pré-V1/inter-estágio |
| Treble | `amp_1.treble` | number | scale | `0.0..=10.0` | `6.0` | audio log | Braço agudo do tone stack 39k | KCL 5×5 + bilinear `f64` |
| Mid | `amp_1.middle` | number | scale | `0.0..=10.0` | `5.0` | 10% log | Braço médio 25kΩ | KCL 5×5 + bilinear `f64` |
| Bass | `amp_1.bass` | number | scale | `0.0..=10.0` | `5.5` | audio log | Braço grave 22nF | KCL 5×5 + bilinear `f64` |
| Presence | `amp_1.presence` | number | scale | `0.0..=10.0` | `5.0` | log | Controle de agudos (na malha NFB em Vintage/Clean; reduzido/anulado na malha NFB em Modern) | Filtro acoplado a `eff_nfb` |
| Master | `amp_1.master` | number | scale | `0.0..=10.0` | `5.0` | audio log | Nível enviado ao LTP e 4×6L6GC | Ganho pós-tone stack + drive de potência |
| Power Mode (Bold/Spongy) | `amp_1.spongy` | boolean | bool | `false (Bold) / true (Spongy)` | `false` | discreto | Reduz tensão interna tipo variac (~15–20%) | Escala `variac_scale` em B+ pré e power |
| Rectifier Select | `amp_1.rectifier` | enum | id | `diode, 5u4gb` (`gz34` Mod Shop) | `5u4gb` | discreto | Diodo silício rígido vs Dual 5U4GB com sag | Resistência interna e ataque/release de sag |

## 10. Contratos de integração
- `module_id = "amp.mesa.dual_rectifier_rev_f"`, `model_revision = "1992-rev-f-solo-100"`.
- Rejeita parâmetros alheios (`volume_2`, `jumpered`, `cut`, `reverb`, `vibrato_speed`, `vibrato_depth`, `bright`) com `E_VALIDATION:`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/amp.rs` (linhas 339–352, 824–827, 1150–1158, 1252–1281, 1516–1541) e `tonestack.rs:94–107`.
- **Dívidas registradas:**
  1. Divergência de componente `C1` (`470 pF` em `tonestack.rs:101` vs `250 pF` em `docs/amps/voicing-fontes.md:39`) e `R_bass` (`250 kΩ` vs `1 MΩ`) a resolver contra esquema primário RF-1F.
  2. Em `amp.rs:1151–1164`, no modo `Modern` (`eff_nfb = 0`), o controle `Presence` fica com ganho `0 dB`; no Dual Rectifier real, o Presence do canal Red Modern ainda atua como filtro passivo passa-baixas pré-inversor (taper de 25 kΩ) fora da malha NFB — registrar e modelar corretamente após conferência no esquema RF-1F!
  3. Troca de `channel_mode` atualmente altera instantaneamente o número de estágios (`2→4`) sem crossfade interno, podendo gerar clique se alternado durante nota sustentada — adicionar crossfade de 10 ms na troca de modo.
- **Estrutura alvo:** `crates/ampforge-dsp/src/amps/dual_rect_revf.rs` (`MesaDualRectifierRevF`).

## 12. Estratégia DSP
- **Circuit-level:** Tone Stack pós-ganho KCL 5×5 + bilinear `f64` (Spec 078); filtros HPF inter-estágio e Miller LP (`7,5 kHz`) pré-4º estágio.
- **Behavioral calibrado com memória:** cascata de 2 a 4 estágios ADAA1 com assimetria crescente no 2º/3º estágio, power amp 4×6L6GC com chave real de NFB, modelo de sag dependente de `diode` vs `5u4gb` e `variac_scale` (`Bold`/`Spongy`).
- **Oversampling:** crítico neste modelo pelo alto ganho em cascata (`2x` padrão para baixa latência, recomendável `4x`/`8x` para gravação; Spec 079).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; todos os 4 estágios ADAA pré-alocados mesmo quando `Clean` usa apenas 2.

## 14. Gestão de estado
- Estados dos 4 estágios, `sag_state`, `tonestack`, `xformer_lf_state` isolados por instância; troca de `channel_mode` usa crossfade suave sem alocação.

## 15. Latência
- Igual à latência do fator de oversampling ativo (Spec 079).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 11%`** do deadline de 128 frames @ 48 kHz em `2x` OS (modo `Modern` 4 estágios) e **`<= 26%`** em `8x` OS (build `--release`).

## 17. Testes
- Comparação de THD e harmônicos entre `clean`, `pushed`, `vintage`, `modern`; verificação de `current_b_plus_v()` e compressão em `Bold` vs `Spongy`; burst forte para comparar envelope de sag `diode` vs `5u4gb`; teste de aliasing com senoide 5 kHz em `Modern` + `gain=10`; teste de troca de modo sem degrau; DI real de palhetada palm-mute em afinação Drop D/C.

## 18. Critérios de aceite
- **CA-1 (Progressão dos 4 Modos):** Na mesma entrada (`1 kHz`, `-12 dBFS`, `gain = 6.5`), `THD(clean) < THD(pushed) < THD(vintage) <= THD(modern)` e `THD(modern) > 3.0 * THD(clean)`.
- **CA-2 (Bold vs Spongy):** Ativar `spongy = true` reduz `current_b_plus_v()` em `15%..=25%` e reduz o pico de saída em regime saturado sem introduzir DC (`< -60 dBFS`).
- **CA-3 (Retificador Diode vs 5U4GB):** Em um burst de `-6 dBFS` seguido de silêncio, `5u4gb` produz queda de B+ (`sag_state` pico) `>= 2×` maior e tempo de recuperação `>= 2×` mais lento que `diode`.
- **CA-4 (NFB em Modern):** No modo `Modern`, a realimentação negativa global efetiva é `0` (aumento mensurável de ressonância de carga reativa em relação a `Vintage` com `nfb > 0`).
- **CA-5 (Anti-Aliasing em High-Gain):** Com `channel_mode = modern`, `gain = 10` e senoide 5 kHz a `-6 dBFS`, o aliasing residual fica `< -60 dBFS` em `4x` e `8x` OS.
- **CA-6 (Troca de Modo Sem Clique e RT Safety):** Alternar `clean → modern` durante senoide de 100 Hz ocorre com crossfade suave sem picos > 1,5×; zero alocações/desalocações em `process()`; saída finita em `[-1.0, 1.0]`.
- **CA-7 (Erro e Não-regressão):** Parâmetros alheios rejeitados com `E_VALIDATION:`; testes das Specs 020, 031 e 039 para `CaliDualRectoRevF` continuam passando.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 077` + `Spec 078` + `Spec 079`.
- **Relacionadas / Legadas:** `Specs 003, 020, 031, 039`.

## 20. Limitações conhecidas
- Conferência pendente no esquema primário `RF-1F` para `C1` (`250 pF` vs `470 pF`), `R_bass` (`250 kΩ` vs `1 MΩ`) e topologia exata do `Presence` no modo `Modern` (filtro passivo pós-tonestack quando NFB está aberto).
- Ausência de medição física com unidade Rev F de 1992.

## 21. Riscos técnicos
- Alto ganho em cascata (4 estágios) amplifica qualquer descontinuidade de parâmetro ou aliasing se o oversampling for reduzido ou se `Modern` não tiver filtro anti-aliasing adequado.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `003`, `020`, `031`, `039`.
- **Documentos locais:** `docs/amps/voicing-fontes.md`.
- **Código atual:** `crates/ampforge-dsp/src/amp.rs`, `crates/ampforge-dsp/src/tonestack.rs`.
- **Testes existentes / futuros:** `contract_031_authentic_amps.rs`, `contract_039_amps_fidelity.rs`, futuro `contract_057_amp_mesa_dual_rectifier.rs`.
