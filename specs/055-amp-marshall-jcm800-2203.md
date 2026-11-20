---
spec: 055
titulo: Amplificador Marshall JCM800 2203 (1981 Master Volume Lead, 100W)
status: rascunho
tipo: amplificador
atualizado: 2026-11-19
dependencias: [053, 077, 078, 079]
---
# 055 — Amplificador Marshall JCM800 2203 (Rev. 1981 Master Volume Lead)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `amp.marshall.jcm800_2203` (alias legado: `jcm800_2203`, `brit_jcm800_2203`, `brit800`)
- **Revisão (`model_revision`):** `1981-2203-vertical-inputs`
- **Categoria:** Amplificador valvulado (módulo independente)
- **Objetivo:** Especificar integralmente o circuito, controles reais, comportamento elétrico/harmônico (incluindo o *cold clipper* assimétrico V1b de 10 kΩ), modelagem DSP e critérios objetivos de validação do **Marshall JCM800 Model 2203 (100W, rev. 1981)**.

## 2. Escopo
- Modelagem das entradas `High` (3 estágios de ganho em cascata: V1a → Pre-Amp → V1b → V2a → V2b CF) e `Low` (bypassa V1a, entrando direto após o estágio de entrada com menor ganho e impedância distinta).
- Rede de peaking `470 kΩ || 470 pF` entre V1b e V2a, grid stopper `68 kΩ` + efeito Miller, *cold clipper* V1b (`Rk = 10 kΩ` sem bypass), cathode follower V2b, Tone Stack FMV Marshall (`33 kΩ / 470 pF / 22 nF / 22 nF`), Master Volume pós-tone stack, LTP + 4×EL34 fixed-bias e malha NFB com `Presence`.

## 3. Fora de escopo
- FX Loop do painel traseiro da reedição 2203X (o 2203 original de 1981 não possuía FX Loop; no AMPFORGE efeitos pós-pré são roteados pelo DAG 4×12 — Spec 084).
- Versão 2203KK ou modelos JCM800 split-channel 2210/2205 (circuitos com clipping por diodos).

## 4. Equipamento ou componente de referência
- **Fabricante:** Marshall Amplification.
- **Modelo exato:** JCM800 Lead Series Model 2203 (100W Master Volume).
- **Período / Ano:** 1981–1984 (entradas verticais; referência de painel no manual oficial `BOOK-00067-00`).
- **Papel na cadeia de sinal:** Referência clássica de crunch/hard rock com ganho em cascata no pré-amplificador e controle independente de Master Volume.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1981 JCM800 2203 (entradas verticais, 100W, sem FX loop, 4×EL34)`.
- **Variantes históricas relevantes (não misturadas):**
  - *1975–1980 JMP 2203:* circuito praticamente idêntico ao JCM800 1981 com pequenas variações de capacitância de filtro da fonte.
  - *1984+ JCM800 entradas horizontais:* mudança de layout PCB e redução de capacitância de filtragem da fonte B+ (mais sag/menos rigidez nos graves).
  - *JCM800 2203X Reissue:* adiciona FX Loop com chave bypass no painel traseiro (`BOOK-00067-00`).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | *Marshall JCM800 2203 Owners Manual* (`BOOK-00067-00`) | 2203 Reissue | Painel frontal e traseiro | `docs/ui-desktop/fontes/marshall-jcm800-2203-owners-manual-de.pdf` | 2026-11-19 | `[confirmado em documentação]` | Ordem frontal: POWER, STANDBY, PRESENCE, BASS, MIDDLE, TREBLE, MASTER, PRE-AMP, entradas HIGH/LOW |
| F2 | `docs/amps/voicing-fontes.md` | Marshall Model 2203 (1975–1984) | Seção 2 | `docs/amps/voicing-fontes.md` | 2026-11-19 | `[confirmado em documentação]` | Tone stack 33k/470pF/22nF/22nF, divisor 470k||470pF, V1b Rk=10k sem bypass |
| F3 | Esquema Marshall 2203 (Dr. Tube / Schematic Heaven / Rob Robinette) | 2203 1981 | Preamp V1a/V1b/V2a/V2b + LTP + 4×EL34 | https://drtube.com/ e https://robrobinette.com/ | 2026-11-19 | `[fonte secundária]` | Cascata V1a (`2.7k/0.68uF`) → Pre-Amp (`1M` + bright `1nF`) → V1b (`10k` unbypassed) → V2a (`820Ω`) → V2b CF → FMV → Master (`1M`) |
| F4 | D. T. Yeh & J. O. Smith (DAFx-06) | FMV Tone Stack | Seções 2–4 | Proc. DAFx-06 | 2026-11-19 | `[confirmado em documentação]` | Discretização bilinear da rede FMV |

## 7. Fundamentação elétrica
- **Entradas:**
  - `High Sensitivity`: entra em **V1a** (`1 MΩ`, grid stopper `68 kΩ`, `Rp = 100 kΩ`, `Rk = 2.7 kΩ || Ck = 0.68 µF`, acoplamento `22 nF`), passa pelo potenciômetro `Pre-Amp Volume` (`1 MΩ` com bright cap `1000 pF`) e alimenta V1b `[fonte secundária: F3]`.
  - `Low Sensitivity`: desconecta V1a do caminho de sinal (chaveamento do jack) e injeta a guitarra diretamente antes de V1b, reduzindo drasticamente o ganho (≈ −6 dB nominal no primeiro estágio em ganho moderado e limpeza muito superior) `[confirmado em documentação: F1, F3]`.
- **V1b — Cold Clipper Assimétrico:** `Rp = 100 kΩ`, **`Rk = 10 kΩ` sem capacitor de bypass** `[confirmado em documentação: F2, F3]`. Polariza o triodo próximo ao corte, produzindo ceifamento fortemente assimétrico (rico em 2º harmônico `2·f0`) e realimentação local de cátodo que contém subgraves.
- **Rede Inter-estágio V1b → V2a:** capacitor de acoplamento `22 nF` + divisor `470 kΩ || 470 pF` em série com `470 kΩ` para terra, formando shelf/peaking de médios-agudos (~1,6 kHz) + corte suave de altas por capacitância de Miller (`~6–7 kHz`) `[confirmado em documentação: F2, F3]`.
- **V2a + V2b Cathode Follower:** V2a (`Rk = 820 Ω`, `Rp = 100 kΩ`) acoplado em DC a V2b (`Rk = 100 kΩ`) alimentando o Tone Stack FMV (`33 kΩ`, `470 pF`, `22 nF`, `22 nF`, `Treble 220 kΩ`, `Bass 1 MΩ`, `Mid 22 kΩ`).
- **Master Volume:** potenciômetro `1 MΩ` audio taper imediatamente **após** o Tone Stack, controlando o nível entregue ao inversor de fase LTP (V3 ECC83) e aos `4×EL34` (`B+ ≈ 450 V`, `bias ≈ 38 mA`, retificador diodo silício, NFB com `Presence`).

## 8. Comportamento esperado
- **Assinatura harmônica do Cold Clipper:** em `Pre-Amp = 8` (`input_jack = high`), o espectro pós-pré apresenta 2º harmônico pronunciado (`H2` significativo quando `triode_asym > 0`), diferindo mensuravelmente de um clipper simétrico.
- **Controle independente de pré e potência:** `Master = 2, Pre-Amp = 8` entrega crunch focado de pré-amp com power limpo; `Master = 8, Pre-Amp = 4` entrega compressão de power 4×EL34 e interação com NFB/impedância.
- **Entrada Low:** em regime linear reduz nível em `~6 ± 1 dB` (ou mais em alto ganho por omitir V1a) e reduz THD drasticamente em comparação a `High`.
- **Foco em médios-agudos e graves enxutos:** atenuação progressiva abaixo de `110–180 Hz` antes do clipping impede *flub* em palm-mutes, enquanto a rede `470 kΩ || 470 pF` projeta a faixa de `1,4–2,0 kHz`.
- **Bright Cap do Pre-Amp:** eleva agudos em `Pre-Amp` baixo (`1–4`) e recua suavemente ao aproximar de `10`.

## 9. Parâmetros e controles
> Ordem física do painel (Manual JCM800 2203 `BOOK-00067-00`, F1): `POWER · STANDBY · PRESENCE · BASS · MIDDLE · TREBLE · MASTER VOLUME · PRE-AMP VOLUME · HIGH / LOW`.

| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Power | `amp_1.power` | boolean | bool | `false/true` | `true` | discreto | Alimentação AC | Mute suave `<= 2,5 ms` |
| Standby | `amp_1.standby` | boolean | bool | `false/true` | `false` | discreto | Corte B+ | Rampa exponencial invariante à taxa |
| Presence | `amp_1.presence` | number | scale | `0.0..=10.0` | `5.0` | log | Reduz NFB acima de ~2–4 kHz | Filtro na malha NFB escalado por `nfb` |
| Bass | `amp_1.bass` | number | scale | `0.0..=10.0` | `5.5` | audio log | Braço de graves FMV 33k/22nF | KCL 5×5 + bilinear `f64` |
| Middle | `amp_1.middle` | number | scale | `0.0..=10.0` | `5.0` | 10% log | Braço de médios 22kΩ FMV | KCL 5×5 + bilinear `f64` |
| Treble | `amp_1.treble` | number | scale | `0.0..=10.0` | `6.0` | híbrido log | Braço agudo 220kΩ/470pF FMV | KCL 5×5 + bilinear `f64` |
| Master Volume | `amp_1.master` | number | scale | `0.0..=10.0` | `5.0` | audio log (cúbico no drive) | Controla nível enviado ao LTP e 4×EL34 | Atenuação pós-tone stack + excitação do power amp |
| Pre-Amp Volume | `amp_1.gain` | number | scale | `0.0..=10.0` | `6.5` | audio log | Excitação de V1b após V1a (+ bright cap 1nF) | Ganho em cascata + bright cap dependente do pot |
| Input Sensitivity | `amp_1.input_jack` | enum | id | `high, low` | `high` | discreto | High = 3 estágios; Low = bypassa V1a (−6 dB) | Bypass de V1a + pad de entrada |

*(Controles de chassi/Mod Shop seguem Spec 077 com defaults nominais do 2203: `12ax7`, `el34`, `diode`, `16 Ω`, `38 mA`, `sag=3.0`, `nfb=5.0`, `xformer_sat=4.0`, `B+_pre=280 V`, `B+_power=450 V`, `triode_asym=0.20`.)*

## 10. Contratos de integração
- Implementa `AudioProcessor` (Spec 053) com `module_id = "amp.marshall.jcm800_2203"`, `model_revision = "1981-2203-vertical-inputs"`.
- Rejeita parâmetros inexistentes neste painel (`volume_2`, `jumpered`, `cut`, `reverb`, `spongy`, `channel_mode`, `vibrato_*`) com `E_VALIDATION:`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/amp.rs` (linhas 297–310, 812–815, 1286–1289, 1472–1505) e `tonestack.rs:51–64`.
- **Dívidas registradas:**
  1. Atualmente acoplado à struct monolítica `WhiteBoxAmpProcessor`.
  2. `amp_1.overdrive` atua como multiplicador extra sobre `raw_drive` (`amp.rs:1474`), embora o JCM800 2203 real tenha apenas `Pre-Amp Volume` e `Master Volume`; na modularização, o ganho nominal deve depender exclusivamente de `Pre-Amp` e `Master` (removendo multiplicador oculto de `overdrive` do descritor 2203).
  3. Diferença de parâmetros de voicing entre `amp.rs:813–814` (`1650 Hz +3.8 dB`, `180 Hz -4.2 dB`) e `docs/amps/voicing-fontes.md` (`1600 Hz +2.0 dB`, `110 Hz -1.5 dB`) deve ser conciliada e documentada com medições.
- **Estrutura alvo:** `crates/ampforge-dsp/src/amps/jcm800_2203.rs` (`MarshallJcm800_2203`).

## 12. Estratégia DSP
- **Circuit-level:** Tone Stack FMV KCL 5×5 + bilinear `f64` (Spec 078); filtros de acoplamento inter-estágio (`22 nF`, `68 kΩ` Miller LP ~6,4 kHz, rede `470 kΩ || 470 pF` + Miller ~7,0 kHz).
- **Behavioral calibrado com memória:** cascata V1a → V1b cold clipper assimétrico (ADAA1 com joelho positivo/negativo distinto) → V2a/V2b cathode follower → LTP + 4×EL34 push-pull com NFB, bias crossover, saturação LF de transformador e sag dinâmico (Spec 077).
- **Oversampling:** seletivo (`2x` padrão, `4x`/`8x` selecionável, Spec 079).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero I/O, zero panic; todos os coeficientes derivados de `sample_rate` (`44,1 / 48 / 88,2 / 96 kHz`).

## 14. Gestão de estado
- Registradores ADAA (`v1a`, `v1b`, `v2`, `power`), estados de filtros Miller/HPF, `tonestack`, `sag_state` e `oversampler` isolados na instância `MarshallJcm800_2203`.

## 15. Latência
- Igual à latência do fator de oversampling selecionado (Spec 079), reportada via `latency_samples()`.

## 16. Consumo de CPU
- Orçamento alvo: **`<= 9%`** do deadline de 128 frames @ 48 kHz em `2x` OS e **`<= 22%`** em `8x` OS (build `--release`).

## 17. Testes
- Senoide 1 kHz em múltiplos níveis (-24, -12, -6 dBFS) para validar razão `H2/H3` do cold clipper V1b; sweep 20 Hz–20 kHz para FMV e rede 470k||470pF; comparação `High` vs `Low` (`-6 ± 1 dB`); variação `Master 2→9` para compressão de power; senoide 5 kHz para aliasing; DI real com palm-mutes.

## 18. Critérios de aceite
- **CA-1 (Tone Stack FMV 2203):** Resposta 50 Hz–10 kHz dentro de `±0,5 dB` da referência analítica KCL (33 kΩ / 470 pF / 22 nF / 22 nF).
- **CA-2 (Cold Clipper V1b):** Com `input_jack = high`, `gain = 8`, `triode_asym = 0.20`, senoide 1 kHz a `-12 dBFS` produz energia de 2º harmônico (`2 kHz`) `>= -24 dB` abaixo da fundamental; com `triode_asym = 0`, `H2` cai em `>= 15 dB`.
- **CA-3 (High vs Low Sensitivity):** Em `gain = 3`, `input_jack = low` atenua `6,0 ± 1,0 dB` frente a `high` e reduz THD em `>= 30%` com `gain = 8`.
- **CA-4 (Separação Pré vs Master):** Com `gain = 4`, aumentar `master` de `2` para `9` aumenta THD e compressão do estágio de potência (crest factor reduz `>= 2 dB`) sem alterar o espectro pré-tone stack.
- **CA-5 (Anti-Aliasing e Invariância de Taxa):** Aliasing residual `< -60 dBFS` em `4x`/`8x` OS para senoide 5 kHz a `-6 dBFS` com `gain = 10`; resposta linear varia `<= 0,5 dB` entre `44,1`, `48` e `96 kHz`.
- **CA-6 (Validação e RT Safety):** Parâmetros alheios ao 2203 (`cut`, `reverb`, `volume_2`) são rejeitados com `E_VALIDATION:`; zero alocações/desalocações em `process()`; saída sempre finita em `[-1.0, 1.0]`.
- **CA-7 (Não-regressão):** Contratos existentes das Specs 003, 016, 020, 031, 039 e 051 CA-8 permanecem verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 077` + `Spec 078` + `Spec 079`.
- **Relacionadas / Legadas:** `Specs 003, 016, 020, 031, 039, 051`.

## 20. Limitações conhecidas
- Saturação de condução de grade da rede V1b→V2a está aproximada via joelho suave de ganho + ADAA1, não via solver MNA completo com corrente de grade explícita.
- Falta medição A/B com JCM800 2203 físico de 1981.

## 21. Riscos técnicos
- Ajuste de `voicing_filter` vs rede `470k||470pF` pode impactar testes antigos que verificam ganho/espectro do preset default `brit800`.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `003`, `016`, `020`, `031`, `039`, `051`.
- **Documentos locais:** `docs/ui-desktop/fontes/marshall-jcm800-2203-owners-manual-de.pdf`, `docs/amps/voicing-fontes.md`.
- **Código atual:** `crates/ampforge-dsp/src/amp.rs`, `crates/ampforge-dsp/src/tonestack.rs`.
- **Testes existentes / futuros:** `contract_031_authentic_amps.rs`, `contract_039_amps_fidelity.rs`, futuro `contract_055_amp_marshall_jcm800.rs`.
