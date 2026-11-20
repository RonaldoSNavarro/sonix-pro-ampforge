---
spec: 082
titulo: Componente Compartilhado de Motor de Reverb (Mola Dispersiva, Plate, Hall, Chamber e Shimmer)
status: rascunho
tipo: componente-compartilhado
atualizado: 2026-11-19
dependencias: [053]
---
# 082 — Componente de Motor de Reverb (Mola, Plate, Hall, Chamber e Shimmer)

## 1. Identificação e objetivo
- **Identificador estável (`module_id`):** `component.fx.reverb_engine` (chaves `rev_1.*`)
- **Revisão (`model_revision`):** `v1-comb-allpass-spring-shimmer`
- **Categoria:** Componente compartilhado DSP (Reverberação algorítmica e spillover)
- **Objetivo:** Especificar os 5 mecanismos distintos de reverberação do AMPFORGE — **Tube Spring Reverb** (dispersão all-pass com controle real de tensão `spring_tension` gerando o *drip* característico), **Studio Vintage Plate** (ref. EMT 140), **Concert Hall**, **Chamber** e **Shimmer Cloud Ambient** (geração real de oitava `+12 st` na malha de realimentação) — com RT60 exato em `44,1 / 48 / 88,2 / 96 kHz`, spillover e clear O(1) zero-alloc.

## 2. Escopo
- Pré-delay circular (`0–150 ms` / até `250 ms` conforme registry — alinhar limite máximo em §11!), banco de 4–8 combs paralelos com amortecimento passa-baixas (`damping`) e realimentação calibrada pela equação física `g_c = 10^(-3 · L_c / (fs · RT60))`.
- Difusão em série por 4 all-pass Schroeder para `Plate`, `Hall`, `Chamber` e `Shimmer` (alta densidade de ecos, sem metallic ringing).
- Cadeia dispersiva de 8 filtros all-pass de 1ª ordem em série modulada por `spring_tension` no `TubeSpringReverb` (criando o atraso de grupo dependente de frequência / *chirp* metálico das molas).
- Pitch shifter granular dual-grain `+12 st` pré-alocado na malha do `ShimmerCloudAmbient`.
- Spillover e invalidação de cauda em O(1) (`samples_since_clear`).

## 3. Fora de escopo
- Reverb por convolução de salas reais (IR > 2 s — candidato futuro sobre o motor particionado da Spec 080).
- Tanque de mola interno do Fender Twin Reverb (embutido na Spec 056) e do Roland RE-201 (embutido na Spec 068).

## 4. Equipamento ou componente de referência
- **Referências:** Fender 6G15 / Accutronics Tank (Spring), EMT 140 (Vintage Plate), Lexicon 224/480L (Hall/Chamber), Brian Eno / Strymon BigSky (Shimmer).

## 5. Revisão e variantes
- **Revisão:** `v1-comb-allpass-spring-shimmer` (consolidando Specs 007, 024, 040 e 049).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | M. R. Schroeder — *Natural Sounding Artificial Reverberation* (JAES 1962) & J. Dattorro — *Effect Design Part 1* (JAES 1997) | Reverberação algorítmica | Combs paralelos + difusores all-pass + fórmula RT60 | JAES 1962 / 1997 | 2026-11-19 | `[confirmado em documentação]` | Equação `g = 10^(-3·T_loop/RT60)` e difusão all-pass |
| F2 | J. Parker & H. Välimäki — *Modeling of Spring Reverberators* (IEEE/DAFx) | Reverb de mola | Cadeia dispersiva de all-pass de 1ª ordem (*chirp*) | DAFx / IEEE TASLP | 2026-11-19 | `[confirmado em documentação]` | Dispersão de velocidade de grupo produzindo o "drip" característico da mola |
| F3 | Spec 040 (`CA-9`, `CA-10`, `CA-11`, `CA-12`) | Contratos existentes | `fx/reverb.rs` | `specs/040-*.md` | 2026-11-19 | `[confirmado em documentação]` | RT60 `±15%` em 44,1/48/96 kHz, densidade Plate `>= 4×`, efeito de `spring_tension >= 20%`, shimmer `+12 st` real |

## 7. Fundamentação acústica / algorítmica
- **RT60 invariante à taxa:** para cada comb `c` de comprimento `L_c` escalado proporcionalmente a `fs / 48000`, `g_c = 10^(-3 · (L_c / fs) / decay_s)` garante que o tempo para cair `-60 dB` permaneça igual a `decay_s` em `44,1`, `48`, `88,2` e `96 kHz` `[confirmado em documentação: F1]`.
- **Dispersão de Mola:** 8 all-pass dispersivos em cascata criam atraso de grupo maior em baixas frequências do que em altas (ou vice-versa conforme o sinal do coeficiente `a`), gerando o som característico de mola excitada por transiente `[confirmado em documentação: F2]`.
- **Shimmer `+12 st`:** a saída difundida alimenta um buffer granular de 4096 amostras lido a taxa `2×` com duas janelas Hann defasadas em `0,5`, realimentado com ganho controlado (`0.55`) no tanque para criar ascensão harmônica progressiva em `2·f0`.

## 8. Comportamento esperado
- **RT60 fiel em múltiplas taxas:** o RT60 medido (queda de -5 a -35 dB extrapolada para -60 dB) fica dentro de **`±15%`** de `decay_s` em `44,1`, `48` e `96 kHz` (Spec 040 CA-9).
- **Densidade de ecos real no Plate/Hall:** a `100 ms` da cauda, a densidade de ecos acima de `-40 dB` é alta e difusa (sem eco metálico esparso — Spec 040 CA-10).
- **`spring_tension` mensurável:** mover `spring_tension` de `0` para `10` altera o atraso de grupo da cadeia dispersiva em **`>= 20%`**.
- **Shimmer real em `2·f0`:** excitado com senoide `f0` (ex.: `440 Hz`), a cauda apresenta energia crescente na oitava `2·f0` (`880 Hz`) em relação ao Plate/Hall (Spec 040 CA-11).
- **Spillover preservado:** instância única fora do par ativa/sombra mantém a cauda decaindo ao desligar ou trocar cena.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `rev_1.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa/bypassa reverb | Respeita `spillover` |
| Model | `rev_1.model` | enum | id | `spring, plate, hall, chamber, shimmer` | `plate` | discreto | Seleciona topologia de tanque/difusão | Escala combs, difusores e módulos ativos |
| Decay | `rev_1.decay_s` | number | s | `0.1..=20.0` (até `30.0`) | `2.5` | log | Tempo RT60 da cauda | Recalcula `comb_feedbacks[c]` |
| Pre-Delay | `rev_1.predelay_ms` | number | ms | `0.0..=250.0` | `20.0` | linear | Atraso inicial antes das reflexões | Buffer circular pré-tanque |
| Damping | `rev_1.damping` | number | scale | `0.0..=10.0` | `5.0` | linear | Absorção HF no laço dos combs | LPF 1 polo no feedback |
| Mix | `rev_1.mix` | number | % | `0.0..=100.0` | `40.0` | linear | Proporção dry/wet | `dry*(1-m) + wet*m` |
| Spring Tension | `rev_1.spring_tension` | number | scale | `0.0..=10.0` | `5.0` | linear | Tensão/dispersão da mola (somente Spring) | Coeficiente `spring_disp_a` e escala comb |
| Spillover | `rev_1.spillover` | boolean | bool | `false/true` | `true` | discreto | Mantém cauda ao bypassar | Zera entrada do tanque mantendo leitura |

## 10. Contratos de integração
- `module_id = "component.fx.reverb_engine"`, `model_revision = "v1-comb-allpass-spring-shimmer"`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/reverb.rs` (linhas 1–337).
- **Dívidas registradas (importantes!):**
  1. **Divergência de faixa de `predelay_ms`:** `controls.registry.ts:1398` declara `range: [0, 250]` ms, enquanto `reverb.rs:87` aloca `predelay_buffer` de `14400` amostras (`150 ms` @ 96 kHz) e `reverb.rs:114` faz clamp em `150.0 ms`! **Alinhar ambos para `250 ms` (`24000` amostras @ 96 kHz) ou limitar a UI a `150 ms`.**
  2. **Damping dependente de taxa (`reverb.rs:228, 274`):** `damp_coeff = (damping / 10.0) * 0.45` está fixo por amostra em vez de derivado de `sample_rate`, fazendo o reverb ficar mais brilhante em `96 kHz` do que em `44,1 kHz` — corrigir para constante temporal/frequência invariante à taxa!
  3. **Falta proteção anti-denormal** em `damp_states` e `comb_buffers` quando a cauda decai abaixo de `1e-24`.
- **Estrutura alvo:** `crates/ampforge-dsp/src/shared/reverb_engine.rs` (e/ou separação opcional por modelo sobre matriz FDN/Schroeder compartilhada).

## 12. Estratégia DSP
- 4–8 combs paralelos + 4 difusores all-pass Schroeder (ou FDN 8×8 Hadamard em evolução futura para densidade ainda maior em Hall/Plate) + cadeia dispersiva de 8 all-pass para Spring + dual-grain `+12 st` para Shimmer; tudo pré-alocado até `96 kHz` com corte anti-denormal e `clear_tail()` O(1).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap e zero `memset` em `process()` e `clear_tail()`; zero locks, zero panic; todos os comprimentos e `damping` invariantes a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- `comb_buffers`, `ap_buffers`, `predelay_buffer`, `shimmer_buf`, `damp_states`, `spring_disp_*` e `samples_since_clear` encapsulados na instância única (fora do par ativa/sombra).

## 15. Latência
- **`0` amostras** no caminho seco (`dry`).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 3,0%`** (Plate/Hall/Spring) e **`<= 4,0%`** (Shimmer) @ 128 frames / 48 kHz em build `--release`.

## 17. Testes
- Medição de RT60 em `44,1`, `48` e `96 kHz` (`±15%` de `decay_s`); densidade de ecos a `100 ms` no Plate; variação de atraso de grupo `>= 20%` com `spring_tension 0→10`; energia em `2·f0` no Shimmer; invariância de brilho (`damping`) entre `44,1` e `96 kHz`; spillover; teste de denormals após 30 s de silêncio; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (RT60 Invariante à Taxa):** Para todos os 5 modelos, o RT60 medido fica dentro de `±15%` de `decay_s` em `44,1`, `48` e `96 kHz`, e o espectro de decaimento sob `damping = 5` coincide em `±1,5 dB` entre `44,1` e `96 kHz`.
- **CA-2 (Spring Dispersivo e Plate Denso):** Mover `spring_tension` de `0` para `10` altera o atraso de grupo da dispersão em `>= 20%`; a `100 ms` da cauda, o Plate apresenta alta densidade de ecos difusos sem anelamento metálico.
- **CA-3 (Shimmer +12 st Real):** Excitado com senoide `440 Hz`, a cauda do `ShimmerCloudAmbient` apresenta energia mensuravelmente maior em `880 Hz` (`>= +12 dB`) do que o `StudioVintagePlate`.
- **CA-4 (Alinhamento Pre-Delay e RT Safety):** `predelay_ms` opera sem truncamento em toda a faixa declarada no registro (`0..=250 ms` ou faixa alinhada); `clear_tail()` e `process()` executam com `0` alocações e `0` desalocações; entradas `NaN`/`Inf` sanitizadas.
- **CA-5 (Não-regressão):** Testes das Specs 007, 024, 040 (`CA-9..CA-12`), 049 CA-1 e 051 CA-7 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Relacionadas / Legadas:** `Specs 007, 024, 040, 049, 051`.

## 20. Limitações conhecidas
- Saída atualmente mono (estéreo decorrelacionado L/R com combs/all-pass em contrafase será ativado com o barramento estéreo global da Spec 053).

## 21. Riscos técnicos
- Feedback elevado em `Shimmer` com `decay_s > 15 s` pode acumular energia em altas frequências; manter `damping` e clamp seguro `[-4.0, 4.0]` dentro das linhas de comb.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `007`, `024`, `040`, `049`, `051`.
- **Código atual:** `crates/ampforge-dsp/src/fx/reverb.rs`.
- **Testes existentes / futuros:** `contract_007_mod_pitch_delay_reverb.rs`, `contract_040_mod_delay_reverb.rs`, futuro `contract_082_reverb_engine.rs`.
