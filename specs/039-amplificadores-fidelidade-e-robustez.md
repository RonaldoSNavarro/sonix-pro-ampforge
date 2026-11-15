---
spec: 039
titulo: Troca de modelo estala, coeficientes presos a 48 kHz, master fantasma no Plexi e tone stacks fora do esquema
status: implementada
atualizado: 2026-11-14
---
# 039 — Amplificadores: fidelidade aos esquemas e robustez (troca de modelo, taxa, controles do painel)

## Objetivo
Cada um dos cinco amps responde como o circuito citado, em qualquer taxa de amostragem, sem estalos na troca de modelo e sem controles "fantasmas".

## Contexto
Review manual das Specs 003, 020 e 031 (2026-11-14), por leitura de `crates/ampforge-dsp/src/amp.rs` e `tonestack.rs`, com a skill `sonix-pro-ampforge-expert`.

| # | Achado | Evidência |
|---|---|---|
| 1 | **Troca de modelo estala.** `set_target_model` troca `mod_settings` na hora, e o crossfade de 20 ms mistura **só os filtros de voicing**. A topologia do pré (1 estágio do Twin ↔ 4 do Recto) usa `self.model_id` e só muda no fim do crossfade, de uma vez, compartilhando os estados ADAA. | `amp.rs:903-914, 1308-1401, 1525-1534` |
| 2 | **Coeficientes presos a 48 kHz.** O filtro do bright (0,85/0,15), o contorno do canal II (0,96/0,04), a rampa de standby (0,14 por amostra, "≈2,5 ms") e o LP da mola (0,38) estão fixos por amostra. Por isso a frequência de corte e o tempo mudam com a taxa. | `amp.rs:1246, 1256-1265, 528` |
| 3 | **Master fantasma no Plexi.** A API rejeita `amp_1.master` para Plexi e Twin (`api.rs:993`), mas o drive de potência do Plexi usa `master_volume³·28` (`amp.rs:1125-1128`). Vindo do JCM800 com master 10, o Plexi fica preso em saturação extrema, sem controle visível que mude isso. |
| 4 | **Pad duplo no canal II do Plexi.** `s_in` já tem `input_pad`, e `s_ch2_base` multiplica `input_pad` de novo (−12 dB em vez de −6 dB). | `amp.rs:1252, 1266` |
| 5 | **AB763 com capacitores errados.** O stack Fender usa C2 = C3 = 47 nF. O AB763 publicado usa 250 pF (treble), 0,1 µF (bass) e 0,047 µF (mid), com 100 k de slope; é o que aparece, por exemplo, no preset Fender da TSC de Duncan Amps. **Conferir no esquema Fender AB763 antes de implementar.** | `tonestack.rs:65-78` |
| 6 | **Top Boost tratado como FMV.** O AC30 Top Boost é uma rede própria (não FMV), mas está modelado com as equações do FMV, o mid fixo e o C1 = 50 pF. A Spec 033 já marca o AC30 como `fonte: pendente`. | `tonestack.rs:79-93` |
| 7 | **Voicing sem fonte.** Cada modelo soma dois EQs paramétricos fixos (ex.: Twin −8 dB em 450 Hz, Plexi +6 dB em 2,5 kHz) sobre o tone stack, que já é derivado do circuito. Não há fonte nem medição, e no Twin o scoop de médios é contado duas vezes. | `amp.rs:755-778` |
| 8 | **Anti-aliasing em 2x.** Com oversampling 2x, os dez notches em 0,52–2,6·fs são todos limitados a `0,48·fs_os` = 0,96·fs e se empilham na mesma frequência. | `amp.rs:946-955` |
| 9 | **Presence/Depth como EQ fixo.** São EQs de pico fixos depois da potência, sem relação com o `nfb` ajustado no Mod Shop. A interação real é presence/resonance **dentro** da malha de realimentação. | `amp.rs:1029-1032, 1474-1477` |

## Comportamento esperado
- A troca de modelo roda as duas topologias completas em paralelo, cada uma com seu próprio estado, e faz crossfade da saída em 20 ms.
- Todo coeficiente derivado de tempo ou frequência é calculado a partir da taxa da cadeia.
- O drive de potência de cada modelo depende só de controles que o painel daquele modelo tem; nos modelos sem master, `amp_1.master` não tem efeito oculto.
- Os tone stacks usam topologia e valores do esquema citado (modelo e revisão) em `docs/`. Enquanto a fonte não for conferida, o modelo aparece com `fonte: pendente` (Spec 033).
- Cada curva fixa de voicing tem fonte (esquema, medição ou documento) registrada, ou é removida.

## Critérios de aceite
- **CA-1:** Trocar JCM800 → Twin durante uma senoide de 100 Hz produz uma derivada máxima amostra-a-amostra durante a troca ≤ 1,5× a maior derivada fora da troca (sem degrau).
- **CA-2:** A resposta do filtro de bright medida a 44,1, 48 e 96 kHz coincide em 1, 3 e 6 kHz com diferença ≤ 0,5 dB. O standby leva a saída abaixo de 1e-6 em 2,5 ms ± 0,5 ms nas três taxas.
- **CA-3:** Com os mesmos knobs visíveis, o Plexi carregado depois do JCM800 com master 10 produz saída idêntica (≤ 1e-6) ao Plexi carregado do estado inicial.
- **CA-4:** No Plexi, o nível limpo (ganho baixo, senoide de 1 kHz) com `low2` fica 6,0 ± 0,3 dB abaixo do `high2`.
- **CA-5:** Com B = M = T = 5, a resposta analógica do stack `fender` coincide com a referência calculada a partir dos valores do esquema AB763 citado, com diferença ≤ 0,5 dB entre 20 Hz e 10 kHz.
- **CA-6:** O AC30 Top Boost usa uma função de transferência derivada do esquema citado. Com Treble 0→10 e Bass 0→10, a resposta coincide com a referência dessa rede (≤ 0,5 dB). Sem fonte conferida, o modelo é exibido com `fonte: pendente` e o teste é marcado como pendente na spec.
- **CA-7:** Todo EQ fixo de voicing por modelo tem uma entrada em `docs/amps/voicing-fontes.md` (modelo, curva, fonte). Um teste falha se existir um filtro de voicing sem entrada.
- **CA-8:** Em 2x, as frequências centrais dos notches anti-aliasing são distintas entre si. Com uma senoide de 5 kHz em fundo de escala e ganho 10, os componentes de aliasing (fora dos harmônicos de 5 kHz) ficam abaixo de −60 dBFS em 4x e 8x; o valor de 2x é registrado no PR.
- **CA-9:** Com `amp_1.nfb` em 0 e em 10, o efeito do Presence (0→10) em 4 kHz difere em pelo menos 3 dB. O presence passa a depender da realimentação, como no circuito.
- **CA-10 (não-regressão):** Os contratos das Specs 003, 016, 020 e 031 continuam passando, exceto os CAs que esta spec substitui. Esses ficam listados nas Notas, com o motivo.

## Fora de escopo
- Modelagem do inversor de fase com corrente de grade e blocking distortion: é uma melhoria de fidelidade candidata a spec própria.
- Novos modelos de amp.

## Invariantes tocadas
- **Todo controle visível funciona como o equipamento real:** CA-3, CA-9.
- **Zero-alloc no callback:** rodar duas topologias em paralelo usa instâncias pré-alocadas, sem criar nada no callback.

## Testes que cobrem esta spec
- `crates/ampforge-dsp/tests/contract_039_amps_fidelity.rs` (`test_ca_01`..`test_ca_04`, `test_ca_06`..`test_ca_08`)
- `crates/ampforge-dsp/tests/contract_003_amp_modeling.rs` (8/8)
- `crates/sonix-desktop/tests/contract_020_amp_models_api.rs` & `contract_037_realtime_safety.rs`
- `docs/amps/voicing-fontes.md` (referências de esquemas para os 5 modelos)

## Notas
Os valores de componentes citados no Contexto (itens 5 e 6) **precisam ser conferidos no esquema da revisão escolhida** antes de virar código. Esta spec não afirma valores sem essa conferência.
