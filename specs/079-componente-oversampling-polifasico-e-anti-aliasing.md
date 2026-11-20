---
spec: 079
titulo: Componente Compartilhado de Oversampling Polifásico Seletivo e Anti-Aliasing
status: rascunho
tipo: componente-compartilhado
atualizado: 2026-11-19
dependencias: [053]
---
# 079 — Componente de Oversampling Polifásico Seletivo e Anti-Aliasing

## 1. Identificação e objetivo
- **Identificador estável (`module_id`):** `component.dsp.selective_oversampling`
- **Revisão (`model_revision`):** `v1-polyphase-kaiser-and-iir`
- **Categoria:** Componente compartilhado DSP (infraestrutura anti-aliasing)
- **Objetivo:** Unificar e padronizar o subsistema de **oversampling seletivo (`1x`, `2x`, `4x`, `8x`)** em `ampforge-dsp` — consolidando o `HalfBandOversampler` polifásico FIR (janela de Kaiser, `>= 96 dB` stopband) e o modo de baixa latência IIR (Chebyshev/Butterworth `f64`) — eliminando implementações duplicadas e inconsistentes hoje espalhadas em `oversampling.rs`, `amp.rs` e `fx/drive.rs`.

## 2. Escopo
- Interface reutilizável `Oversampler` suportando fatores `1x`, `2x`, `4x`, `8x` sem alocação em RT.
- **Modo Fase Linear (`PolyphaseHalfBand` — `oversampling.rs`):** cascata half-band FIR polifásico Kaiser (`103 / 43 / 27` taps em ring buffers zero-shift), atenuação real na banda de rejeição `>= 96 dB` verificada por DFT, ideal para qualidade máxima.
- **Modo Baixa Latência (`MinPhaseIir` — usado em tempo real ao vivo):** interpolação + anti-imaging Butterworth + decimação Chebyshev Tipo I em `f64` TDF-II com ripple passband `<= 0,25 dB` até `18 kHz`, stopband `>= 70 dB` em `fs/2` e latência ultrabaixa (`4 / 8 / 16` amostras conforme fator).
- Cálculo exato e unificado de `latency_samples()` e medição automatizada `stopband_attenuation_db()` via DFT real.

## 3. Fora de escopo
- Resampling assíncrono de arquivos WAV/IR (coberto pelo `WavIrLoader` na Spec 080).
- Superamostragem de blocos puramente lineares (EQ, delay digital, mixer — oversampling é **seletivo** para estágios não lineares).

## 4. Equipamento ou componente de referência
- **Referência técnica:** Teoria de filtros multirate half-band polifásicos (Crochiere & Rabiner / Vaidyanathan) e filtros IIR Chebyshev Tipo I de fase mínima.

## 5. Revisão e variantes
- **Revisão:** `v1-polyphase-kaiser-and-iir`.
- **Padrão do sistema:** fator **`2x`** (Spec 051 CA-17), selecionável em tempo real para `2x`, `4x` e `8x` (e `1x` para bypass/diagnóstico).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | R. E. Crochiere & L. R. Rabiner — *Multirate Digital Signal Processing* | Filtros Half-Band Polifásicos | Decomposição polifásica, taps pares nulos | Prentice-Hall | 2026-11-19 | `[confirmado em documentação]` | Eficiência de half-band FIR (metade dos coeficientes = 0, simetria dobra economia) |
| F2 | `crates/ampforge-dsp/src/oversampling.rs` | Workspace v1.0 | `HalfBandOversampler` | `crates/ampforge-dsp/src/oversampling.rs` | 2026-11-19 | `[confirmado em documentação]` | Implementação zero-shift ring buffer e medição DFT `stopband_attenuation_db()` |
| F3 | Specs 039 CA-8, 041 CA-5, 044 CA-5, 049 CA-12/CA-13, 051 CA-17 | Contratos existentes | Oversampling e aliasing | `specs/` | 2026-11-19 | `[confirmado em documentação]` | Padrão `2x`, stopband real medida por DFT, aliasing `< -60 dBFS` em 4×/8× |

## 7. Fundamentação elétrica / matemática
- Quando uma não linearidade de ordem `N` atua sobre um sinal de frequência `f_0`, gera harmônicos em `k·f_0`. Qualquer harmônico acima de `fs/2` dobra de volta (*aliasing*) na banda audível como frequências inarmônicas metálicas.
- Sobreamostrar por fator `M` amplia Nyquist para `M·fs/2`, permitindo que o filtro de decimação remova a energia entre `fs/2` e `M·fs/2` antes de retornar à taxa base.

## 8. Comportamento esperado
- **Unificação sem duplicação:** amplificadores (Specs 054–058) e pedais de drive (Specs 059–064) consomem o mesmo componente `Oversampler` em vez de manter 3 implementações paralelas divergentes (`oversampling.rs`, `amp.rs` e `fx/drive.rs`).
- **Atenuação verificada por DFT:** `PolyphaseHalfBand` atinge `>= 96 dB` de atenuação na banda de rejeição; `MinPhaseIir` atinge `>= 70 dB` em `fs/2` com ripple `<= 0,25 dB` de `20 Hz` a `18 kHz`.
- **Latência honesta e consistente:** `latency_samples()` devolve o atraso de grupo exato correspondente ao modo/fator ativo e atualiza `estimated_roundtrip_ms` na telemetria.
- **Zero-alloc na troca de fator:** alternar `2x ↔ 4x ↔ 8x` não aloca nem gera estalos abruptos.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Fator Oversampling | `engine.oversampling` | enum | fator | `1x, 2x, 4x, 8x` | `2x` | discreto | Equilíbrio entre supressão de aliasing, latência e CPU | Configura estágios ativos |
| Modo Filtro | `engine.oversampling_mode` (interno/futuro) | enum | id | `min_phase_iir, linear_phase_fir` | `min_phase_iir` | discreto | Baixa latência ao vivo vs fase linear máxima rejeição | Seleciona backend do componente |

## 10. Contratos de integração
- Consumido por composição por `Specs 054–064` e reportado pelo grafo da `Spec 053` para cálculo de PDC e `algorithmic_latency_samples`.

## 11. Arquitetura de implementação
- **Código atual mapeado:**
  - `crates/ampforge-dsp/src/oversampling.rs` (`HalfBandOversampler` — hoje usado só nos testes/DspEngine, listado em `docs/biblioteca-sem-uso-em-producao.md`!)
  - `crates/ampforge-dsp/src/amp.rs` (linhas 797–806, 1001–1075 — sinc 8-tap + Chebyshev 14ª ordem próprio)
  - `crates/ampforge-dsp/src/fx/drive.rs` (linhas 397–435, 473–504 — Catmull-Rom + Butterworth 8ª + Chebyshev 12ª próprio chamado `oversample_4x` mas rodando `8x`!)
- **Dívidas registradas:** eliminar a tripla duplicação acima unificando ambos os perfis (`MinPhaseIir` e `PolyphaseHalfBand`) em `shared/oversampling.rs`.
- **Estrutura alvo:** `crates/ampforge-dsp/src/shared/oversampling.rs`.

## 12. Estratégia DSP
- Buffer de trabalho na stack `[f64; 8]` (zero heap), ring buffers de potência de 2 (`RING_SIZE = 64`) no modo polifásico FIR, e cascata TDF-II `f64` no modo IIR de fase mínima.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; todos os estágios pré-construídos em `new()`/`prepare()`.

## 14. Gestão de estado
- Ring buffers e estados de biquads encapsulados por instância; `reset()` zera estados em O(1).

## 15. Latência
- `MinPhaseIir`: `0` (`1x`), `4` (`2x`), `8` (`4x`), `16` (`8x`) amostras na taxa base.
- `PolyphaseHalfBand`: `0` (`1x`), `51` (`2x`), `61` (`4x`), `65` (`8x`) amostras na taxa base.

## 16. Consumo de CPU
- Orçamento alvo: **`<= 1,5%`** em `2x` e **`<= 5%`** em `8x` por instância @ 128 frames / 48 kHz (`--release`), validado via Criterion.

## 17. Testes
- Medição DFT da banda passante (`20 Hz–18 kHz`, ripple `<= 0,25 dB`) e banda de rejeição (`>= 70 dB` IIR / `>= 96 dB` FIR); teste de impulso validando latência exata de grupo; teste de senoide `5 kHz` através de `tanh(10·x)` medindo queda monotônica de aliasing de `1x → 2x → 4x → 8x`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Rejeição Medida por DFT):** `PolyphaseHalfBand` atinge `>= 96 dB` de rejeição e `MinPhaseIir` atinge `>= 70 dB` em `fs/2` com ripple `<= 0,25 dB` de `20 Hz` a `18 kHz`.
- **CA-2 (Redução Real de Aliasing):** Passando senoide `5 kHz` a `-6 dBFS` em `tanh(10·x)`, o aliasing residual cai monotonicamente de `1x` para `2x`, `4x` e `8x`, atingindo `< -60 dBFS` em `4x` e `8x`.
- **CA-3 (Latência Exata):** O pico do impulso na saída coincide com `latency_samples()` (`±1` amostra no IIR de fase mínima; `±0` amostras no FIR fase linear).
- **CA-4 (Invariância de Taxa e RT Safety):** Funciona em `44,1`, `48`, `88,2` e `96 kHz` sem divergência; zero alocações/desalocações em `upsample`/`downsample` e na troca de fator.
- **CA-5 (Não-regressão):** Testes das Specs 044 CA-5, 049 CA-12/CA-13 e 051 CA-17 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Dependentes:** `Specs 054–064, 077`.
- **Relacionadas / Legadas:** `Specs 001, 039, 041, 044, 049, 051`.

## 20. Limitações conhecidas
- No modo `PolyphaseHalfBand`, a latência de `51–65` amostras (`~1,06–1,35 ms` @ 48 kHz) é maior que o modo `MinPhaseIir` (`~0,08–0,33 ms`), sendo o modo IIR preferível para monitoramento ao vivo de ultrabaixa latência.

## 21. Riscos técnicos
- Acumular oversampling separado em Drive + Amp eleva latência total se ambos usarem `PolyphaseHalfBand` 8× (`130` amostras ≈ `2,7 ms`); manter `MinPhaseIir` como padrão ao vivo e declarar latência somada corretamente no grafo.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `001`, `039`, `041`, `044`, `049`, `051`.
- **Código atual:** `crates/ampforge-dsp/src/{oversampling.rs, amp.rs, fx/drive.rs}`.
- **Testes existentes / futuros:** `contract_049_oversampler.rs`, `contract_044_sdd_integrity.rs`, futuro `contract_079_selective_oversampling.rs`.
