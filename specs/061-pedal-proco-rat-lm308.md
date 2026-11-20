---
spec: 061
titulo: Pedal ProCo RAT Distortion (Rev. 1984–1988 Whiteface / LM308N)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053, 079]
---
# 061 — Pedal ProCo RAT Distortion (Rev. 1984–1988 LM308N)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.proco.rat_lm308` (alias legado: `rat`, `proco_rat`, `black_rodent_rat`)
- **Revisão (`model_revision`):** `1985-whiteface-lm308n`
- **Categoria:** Pedal de Distortion / Fuzz-Overdrive (módulo independente)
- **Objetivo:** Especificar integralmente o circuito do **ProCo RAT (rev. LM308N)** — incluindo o **slew-rate limiting (~0,3 V/µs) do op-amp LM308N**, as duas malhas RC passa-altas pré-clipping (`~60 Hz` e `~1,54 kHz`), o hard clipping para terra por diodos de silício `1N914` e o controle **`Filter` passivo de 1ª ordem invertido** (horário escurece).

## 2. Escopo
- Estágio de entrada e ganho não-inversor `LM308N` (compensado por capacitor externo `30 pF`, slew rate lento ~0,3 V/µs), dupla malha RC no ramo inversor (ênfase acima de `~1,54 kHz`), hard clipping `1N914` para terra, filtro passivo de 1ª ordem **invertido** (`R_fixed 1.5 kΩ + pot 100 kΩ + C 3.3 nF`), buffer JFET `2N5458` e `Volume`.

## 3. Fora de escopo
- Turbo RAT (diodos LED em vez de silício — acessível apenas como mod de diodo), Dirty RAT (germânio) e RAT2 pós-2003 com op-amp OP07DP (slew rate e textura distintos).

## 4. Equipamento ou componente de referência
- **Fabricante:** ProCo Sound.
- **Modelo exato:** The RAT ("Small Box / Whiteface" com `LM308N`).
- **Período / Ano:** 1984–1988.
- **Papel na cadeia de sinal:** Distorção agressiva de alto ganho (até ~+67 dB no op-amp), capaz de ir de overdrive crocante a fuzz denso com controle fino de corte de agudos via `Filter`.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1984–1988 ProCo RAT (LM308N + 30 pF comp + 1N914 + Filter 1.5k/100k/3.3nF)`.
- **Variantes históricas:** Bud Box (1978, pot Tone normal), Big Box (1979–83), RAT2 com OP07DP (pós-2003).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | ElectroSmash — *ProCo RAT Analysis* | ProCo RAT (`LM308N`) | Slew Rate, Active RC Filters (`1539 Hz`), Hard Clipping, Tone Filter (`3.3 nF`) | https://www.electrosmash.com/proco-rat | 2026-11-19 | `[fonte secundária]` | LM308 `~0.3 V/µs` com `30 pF`, polo ativo `~1539 Hz`, `1N914` hard clip, filtro passivo 1ª ordem `1.5k + 100k` + `3.3 nF` invertido |
| F2 | National Semiconductor / Motorola `LM308` Datasheet | `LM308N` | Slew Rate & Compensation (`30 pF`) | Datasheet fabricante | 2026-11-19 | `[confirmado em documentação]` | Slew rate nominal ~0,3 V/µs com capacitor de compensação de 30 pF |
| F3 | Spec 032 CA-2 e Spec 041 CA-5/CA-7 | Contratos existentes | `fx/drive.rs` | `specs/032-*.md`, `specs/041-*.md` | 2026-11-19 | `[confirmado em documentação]` | Sentido invertido do `Filter`, inclinação de 1ª ordem (`6 dB/oit` → `12 ± 1,5 dB` em 2 oitavas) e anti-aliasing |

## 7. Fundamentação elétrica
- **Impedâncias:** `Z_in ≈ 1 MΩ` (resistor de pull-down + `1 kΩ` série); `Z_out ≈ 1–10 kΩ` após seguidor de fonte JFET `2N5458` `[fonte secundária: F1]`.
- **Estágio de Ganho LM308N + Filtragem Seletiva:**
  - Duas malhas RC em paralelo para terra no pino inversor: ramo A (`47 Ω + 2.2 µF`, `f_c ≈ 1,54 kHz`) e ramo B (`560 Ω + 4.7 µF`, `f_c ≈ 60 Hz`), controladas pelo pot `Distortion` (`100 kΩ` áudio) `[fonte secundária: F1]`. Mantém graves articulados enquanto dispara o ganho em médios-agudos.
  - **Slew-Rate Limiting:** a corrente limitada de saída interna sobre o capacitor de compensação de `30 pF` limita `|dV/dt| <= ~0,3 V/µs` `[confirmado em documentação: F2]`, arredondando transientes rápidos e agudos estridentes em alto ganho (atuando como passa-baixas dinâmico dependente de amplitude).
- **Hard Clipping:** após resistor série (`1 kΩ`), par de diodos de silício `1N914` em antiparalelo para terra ceifa abruptamente em `±0,7 V`.
- **Filtro Passivo Invertido (`Filter`):** resistor `1.5 kΩ` em série com potenciômetro `100 kΩ` (log/reverso) carregando capacitor `3.3 nF` para terra (`f_c ≈ 475 Hz` a `~32 kHz`, inclinação exata de **1ª ordem = 6 dB/oitava**) `[fonte secundária: F1]`. Girar no sentido horário **aumenta a resistência em série e derruba `f_c`**, escurecendo o som!

## 8. Comportamento esperado
- **Sentido invertido do `Filter`:** aumentar `Filter` (`tone`) de `0` para `10` reduz a energia acima de `3 kHz` em `>= 6 dB`, com inclinação de **1ª ordem (`12 ± 1,5 dB` duas oitavas acima de `f_c`)**.
- **Compressão dinâmica por Slew-Rate:** em sinais de alta frequência e alta amplitude (ex.: 4 kHz com `Distortion = 9`), o limitador de slew reduz a inclinação da onda (tendendo a triangularização), diferenciando o RAT de um hard clipper estático.
- **Alto ganho limpo→fuzz:** de `Distortion = 1` (crunch áspero) a `Distortion = 10` (sustain denso próximo a fuzz com graves contidos pela rede de `1,54 kHz`).

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `drive.enabled` | boolean | bool | `false/true` | `true` | discreto | Ativa/bypassa pedal | Crossfade sem cliques `<= 5 ms` |
| Distortion | `drive.drive` | number | scale | `0.0..=10.0` | `5.5` | audio log (`100 kΩ`) | Ganho do estágio LM308N + realce > 1,54 kHz | Ganho em cascata com pré-filtro seletivo |
| Filter | `drive.tone` | number | scale | `0.0..=10.0` | `4.5` | log invertido | Passa-baixas passivo 1ª ordem (`0`=aberto ~14–32 kHz, `10`=escuro ~500–600 Hz) | LPF 1ª ordem `f64` (bilinear) |
| Volume | `drive.level` | number | scale | `0.0..=10.0` | `6.0` | audio log (`100 kΩ`) | Atenuação pós-buffer JFET | Ganho de saída suavizado |

## 10. Contratos de integração
- `module_id = "pedal.proco.rat_lm308"`, `model_revision = "1985-whiteface-lm308n"`.
- A UI mapeia `Filter` para `drive.tone` indicando explicitamente o sentido invertido (`controls.registry.ts:880`).

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/drive.rs` (linhas 393–395, 567–598).
- **Dívidas registradas:**
  1. Falta a dupla malha RC pré-clipping (`~60 Hz` / `~1,54 kHz`) antes do slew limiter e clipping em `process_rat` (`drive.rs:578` hoje multiplica `input[i] * pre_gain` diretamente sem pré-ênfase seletiva de graves!).
  2. O slew limiter atual (`drive.rs:582`) usa `max_slew` escalado por `48000 / sr`, mas precisa de calibração física explícita por taxa para manter comportamento idêntico em `44,1`, `48` e `96 kHz`.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/proco_rat.rs` (`ProCoRatLm308`).

## 12. Estratégia DSP
- **Hybrid:** rede pré-clipping (duplo polo `60 Hz` / `1539 Hz` em `f64`) → integrador com limitação suave de derivada (`dV/dt` escalado por `1 / (sr * os)`) dentro do loop de oversampling (`8x` OS, Spec 079) → hard clipping de diodo (`1N914` Shockley / algébrico suave com ADAA1) → LPF passivo de 1ª ordem em `f64`.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; `slew_state` resetado em caso de não-finito; invariante à taxa (`44,1 / 48 / 88,2 / 96 kHz`).

## 14. Gestão de estado
- `rat_slew_state`, pré-filtros RC, `filter_lpf_1st` e oversampler isolados em `ProCoRatLm308`.

## 15. Latência
- Latência do oversampler (Spec 079).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 3,5%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Sweep de `Filter` validando inclinação de `12 ± 1,5 dB` duas oitavas acima de `f_c` e sentido invertido (`Filter=10` mais escuro que `0`); teste de resposta em frequência pré-clipping (atenuação relativa de graves vs 1,5 kHz); teste de slew rate (1 kHz vs 5 kHz em alta amplitude); aliasing `< -55 dBFS`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Filtro Passivo Invertido de 1ª Ordem):** Aumentar `Filter` (`tone`) de `0` para `10` reduz energia acima de `3 kHz` em `>= 6 dB`; duas oitavas acima de `f_c`, a atenuação é `12,0 ± 1,5 dB` (6 dB/oitava).
- **CA-2 (Slew-Rate LM308 Invariante à Taxa):** Em `Distortion = 9`, a resposta a senoide de alta amplitude em `4 kHz` coincide dentro de `±1,0 dB` entre `44,1`, `48` e `96 kHz` e exibe menor conteúdo harmônico agudo que o mesmo clipper sem limitação de slew.
- **CA-3 (Filtragem Pré-Clipping Seletiva):** Em `Distortion = 7`, a razão de energia `1,5 kHz / 120 Hz` antes do clipper é `>= 4 dB` maior que em `Distortion = 0` (mantendo graves controlados).
- **CA-4 (Anti-Aliasing e RT Safety):** Com `Distortion = 10` e senoide `5 kHz` a `-6 dBFS`, alias residual `< -55 dBFS`; zero alocações/desalocações em `process()`; entradas `NaN`/`Inf` sanitizadas.
- **CA-5 (Não-regressão):** Testes das Specs 032 CA-2 e 041 CA-5/CA-7 para `BlackRodentRat` continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 079` (Oversampling).
- **Relacionadas / Legadas:** `Specs 032, 041`.

## 20. Limitações conhecidas
- Assimetria residual do rail de 9 V no LM308N em extremos de bateria fraca não está modelada.

## 21. Riscos técnicos
- Alto ganho pré-clipping (até 40×+) exige oversampling rigoroso (`8x`) para impedir aliasing audível em agudos.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `032`, `041`.
- **Código atual:** `crates/ampforge-dsp/src/fx/drive.rs`.
- **Testes existentes / futuros:** `contract_032_pedals_fx_cabs.rs`, `contract_041_dynamics_and_drives.rs`, futuro `contract_061_pedal_proco_rat.rs`.
