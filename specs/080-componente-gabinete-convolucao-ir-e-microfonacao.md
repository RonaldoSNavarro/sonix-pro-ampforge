---
spec: 080
titulo: Componente Compartilhado de Gabinete Virtual, Convolução IR Particionada e Microfonação Dupla
status: rascunho
tipo: componente-compartilhado
atualizado: 2026-11-19
dependencias: [053]
---
# 080 — Componente de Gabinete, Convolução IR Particionada e Microfonação Dupla

## 1. Identificação e objetivo
- **Identificador estável (`module_id`):** `component.cab.virtual_ir_dual_mic` (chaves `cab_1.*`, `cab_2.*`)
- **Revisão (`model_revision`):** `v1-upols-128-dual-mic`
- **Categoria:** Componente compartilhado DSP (Gabinete / Convolução IR / Microfones)
- **Objetivo:** Especificar integralmente o módulo reutilizável de gabinete e microfonação — **convolução particionada zero-latency (cabeça FIR 128 taps + partições FFT-256 Overlap-Add)**, loader e resampler sinc de IR WAV (até 500 ms), **4 modelos de caixa/falante**, **curva de impedância reativa**, **sala de microfonação dupla com 7 cápsulas baseadas em datasheets**, posição cap→edge, distância/proximidade, eixo 0°/45°, fase Ø e troca de IR sem cliques e sem desalocação na thread RT.

## 2. Escopo
- **Modo Virtual Cab (Dual-Mic):** 4 caixas (`Greenback4x12Closed` 98 Hz, `V304x12Closed` 108 Hz, `AlnicoBlue2x12Open` 82 Hz, `Twin2x12Open` 90 Hz) + 2 slots de microfone independentes (`SM57`, `MD421`, `M88`, `R121`, `M160`, `U87`, `C414`) com `position` (0..1 cap→edge), `distance_cm` (0..30 cm → atraso ~29 µs/cm + proximidade), `angle_deg` (`0° / 45°`), `level_db`, `phase_invert` e `hicut_khz`.
- **Modo IR Customizada (`ir_enabled = true`):** parser WAV PCM `16/24-bit` e `f32` (`8–384 kHz`, limite 500 ms), resampling sinc Blackman de 48 taps fora da RT, convolução particionada zero-latency (`PART_B = 128`, `FFT_N = 256`, até 512 partições), crossfade suave sem buraco e devolução da IR antiga ao `RetiredReaper` (Spec 037).
- Exposição de `ImpedanceCurve` e `resonance_hz` para acoplamento desacoplado com o power amp (Spec 077).

## 3. Fora de escopo
- Modelagem não linear física de excursão de cone / compressão térmica do alto-falante (uma IR convencional representa a resposta LTI; `speaker_thump` comportamental opcional fica isolado).
- Convolução estéreo True-Stereo de 4 canais (IR estéreo atualmente extrai canal 1 e reporta `source_channels` em JSON — evolução estéreo vinculada à Spec 053).

## 4. Equipamento ou componente de referência
- **Caixas/Falantes:** Marshall 1960A 4×12 Celestion Greenback G12M-25, Mesa 4×12 Celestion Vintage 30, Vox 2×12 Open Back Celestion Alnico Blue, Fender Twin 2×12 Open Back (Jensen C12N / JBL).
- **Microfones (datasheets documentados em `docs/cab-microfones-fontes.md`):** Shure SM57, Sennheiser MD 421-II, Beyerdynamic M 88 TG, Royer R-121, Beyerdynamic M 160, Neumann U 87 Ai, AKG C414 XLS.

## 5. Revisão e variantes
- **Revisão:** `v1-upols-128-dual-mic` (consolidando Specs 005, 017, 022, 032, 037 e 038).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | `docs/cab-microfones-fontes.md` | 7 modelos de microfone + IR estéreo | Tabela completa | `docs/cab-microfones-fontes.md` | 2026-11-19 | `[confirmado em documentação]` | Curvas de 2 biquads por cápsula baseadas nos manuais/datasheets oficiais Shure, Sennheiser, Beyerdynamic, Royer, Neumann, AKG |
| F2 | W. G. Gardner — *Efficient Convolution Without Input-Output Delay* (JAES 1995) | Convolução Particionada Zero-Latency | Partição 0 FIR direta + cauda FFT Overlap-Add | JAES Vol. 43, No. 3, 1995 | 2026-11-19 | `[confirmado em documentação]` | Convolução exata de IRs longas com latência `0` amostras |
| F3 | Specs 005, 017, 022, 037, 038 | Contratos existentes | `cab.rs` | `specs/038-gabinete-ir-convolucao-e-microfones-com-efeito.md` | 2026-11-19 | `[confirmado em documentação]` | Zero-latency UPOLS, validação de WAV, 7 cápsulas, zero-alloc e `RetiredReaper` |

## 7. Fundamentação elétrica e acústica
- **Impedância do alto-falante:** pico eletromecânico na frequência de ressonância `f_s` (`82–108 Hz`, `Q = 1.7–2.5`, `+5 a +7,5 dB`) + subida indutiva da bobina móvel em altas frequências (`+3,5 a +5 dB`), comunicada ao power amp para interação real com NFB e `impedance_ohms`.
- **Acústica de microfonação dupla:**
  - *Cap → Edge:* deslocar da calota central para a borda reduz brilho acima de `2–4 kHz`.
  - *Distância (0–30 cm):* atraso de propagação acústica `Δt = d / 343 m/s ≈ 29,15 µs/cm` (gerando filtro pente real ao somar Mic 1 + Mic 2 em distâncias diferentes!) + redução do efeito de proximidade em graves.
  - *Off-Axis 45°:* atenuação direcional acima de `4 kHz`.
  - *Fase Ø:* inversão de `180°` (`-1.0`) antes da soma Mic 1 + Mic 2.
- **Convolução LTI (`y[n] = Σ x[k]·h[n-k]`):** partição 0 (`0..127`) executada por FIR direto (latência `0`) + partições `p >= 1` por FFT-256 Radix-2 in-place (zero-alloc) com Frequency-Domain Delay Line (FDL) pré-alocada.

## 8. Comportamento esperado
- **Exatidão matemática da convolução IR:** convoluir um impulso unitário reproduz exatamente as amostras da IR carregada (`erro RMS < 1e-4`), inclusive para IRs longas de `500 ms` (`24.000` amostras @ 48 kHz), com **`0` amostras de latência adicional**.
- **7 Microfones realmente distintos:** cada modelo (`sm57`..`c414`) produz resposta em frequência distinta coerente com `docs/cab-microfones-fontes.md` (ex.: `r121` significativamente mais escuro acima de `4 kHz` que `sm57`/`c414`).
- **Cancelamento real de fase e comb entre Mic 1 e Mic 2:** com Mic 1 e Mic 2 idênticos e mesma distância, ativar `mic2.phase_invert = true` cancela a soma (`< -60 dBFS`); diferenciar `distance_cm` gera filtro pente acústico real.
- **Resiliência WAV:** rejeita arquivos vazios, truncados, `channels == 0`, `sample_rate == 0` ou fora de `8–384 kHz`, formatos não suportados ou duração `> 500 ms` sem panic (Spec 017 / 022).
- **Troca de IR sem buraco nem desalocação RT:** preparação e FFT das partições ocorrem na thread de controle; a thread de áudio aplica crossfade e devolve `Arc<ImpulseResponse>` antiga ao `RetiredReaper`.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass Cab | `cab_1.enabled` | boolean | bool | `false/true` | `true` | discreto | Ativa/bypassa gabinete | Crossfade sem cliques |
| Gabinete / Falante | `cab_1.speaker` | enum | id | `greenback_4x12, v30_4x12, alnico_blue_2x12, twin_2x12_open` | `v30_4x12` | discreto | Curva de caixa + `ImpedanceCurve` | Voicing + ressonância |
| Ressonância | `cab_1.resonance_hz` | number | Hz | `65.0..=135.0` | `95.0` | linear | Pico eletromecânico da caixa | Propaga ao power amp + cab |
| Hi-Cut | `cab_1.hicut_khz` | number | kHz | `3.5..=18.0` | `10.5` | log | Passa-baixas pós-gabinete | Biquad LPF 24 dB/oit `f64` |
| Modo IR | `cab_1.ir_enabled` | boolean | bool | `false/true` | `false` | discreto | Seleciona convolução IR vs Dual-Mic | Chaveamento suave |
| Mic 1/2 Controles | `cab_1.mic{1,2}.{enabled,model,position,distance_cm,level_db,angle_deg,phase_invert}` | misto | misto | ver `controls.registry.ts` | SM57 / R121 | misto | Cápsula, cone, distância, eixo, nível, polaridade | 2 biquads cápsula + filtros + delay line |

*(Idem para `cab_2.*` na Linha B paralela.)*

## 10. Contratos de integração
- `module_id = "component.cab.virtual_ir_dual_mic"`, `model_revision = "v1-upols-128-dual-mic"`.
- Publica `resonance_hz` / `ImpedanceCurve` para o `ProcessContext` (Spec 053) sem acoplar diretamente a `WhiteBoxAmpProcessor`.
- Carregamento de arquivo via comando IPC `load_ir` (validado fora do RT; nunca chama `std::fs` no callback).

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/cab.rs` (linhas 1–1206).
- **Dívidas registradas:**
  1. `cab.rs` duplica sua própria struct `BiquadFilter` DF-I (`cab.rs:6–160`); migrar para a primitiva compartilhada `Biquad64` TDF-II (Spec 053).
  2. `IrPartitionedData::from_samples` e `PartitionedConvolverState` já usam FFT-256 zero-alloc; adicionar opção de normalização explícita (não silenciosa) e preservação de metadados de IR (autor, falante, microfone, polaridade, hash SHA-256) para rastreabilidade em presets (Spec 010/053).
  3. Evolução futura para IR estéreo verdadeiro (2 canais de convolução) quando o barramento global migrar para estéreo.
- **Estrutura alvo:** `crates/ampforge-dsp/src/shared/cab_ir.rs` (separando `wav_ir.rs`, `convolver.rs`, `dual_mic.rs`).

## 12. Estratégia DSP
- **Black-box LTI exato (IR):** convolução uniformemente particionada (UPOLS) `B = 128`, `N = 256` Radix-2 in-place (zero-alloc) + resampler sinc Blackman 48 taps off-RT.
- **Behavioral acústico (Dual-Mic):** 2 biquads `f64` por cápsula (datasheets em `docs/cab-microfones-fontes.md`) + filtros de posição/proximidade/eixo + linha de atraso fracionário por microfone (`0–30 cm`).

## 13. Requisitos de tempo real
- Zero alocação e zero desalocação heap em `process()` (incluindo durante troca de IR — `retired_ir` sempre recolhida por `RetiredReaper` fora da RT); zero locks, zero I/O, zero panic.

## 14. Gestão de estado
- `conv_custom`, `conv_target` (`fdl` pré-alocada para `MAX_PARTITIONS = 512`), `delay_line_mic1/2` e biquads isolados por instância.

## 15. Latência
- **`0` amostras** (convolução particionada com cabeça FIR direta).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 5%`** com IR de `200 ms` e **`<= 9%`** com IR de `500 ms` @ 128 frames / 48 kHz em build `--release`.

## 17. Testes
- Impulso unitário comparando saída do convolver particionado contra convolução direta de referência para IRs de `64`, `512`, `4096` e `24000` amostras (`erro < 1e-4`, latência `0`); sweep validando os 7 microfones e `angle_deg 0 vs 45`; teste de cancelamento por `phase_invert` e comb por `distance_cm`; fuzzing de WAV malformado (`channels=0`, `sr=0`, truncado, `> 500 ms`); zero-alloc/zero-dealloc RT na troca de IR.

## 18. Critérios de aceite
- **CA-1 (Convolução Particionada Exata e Zero-Latency):** Para qualquer IR de `1` a `24.000` amostras (`500 ms` @ 48 kHz), a saída a um impulso coincide com a IR original com `erro RMS < 1e-4` e atraso de `0` amostras.
- **CA-2 (7 Cápsulas de Microfone Distintas):** Os 7 modelos (`sm57`, `md421`, `m88`, `r121`, `m160`, `u87`, `c414`) geram respostas espectrais distintas consistentes com `docs/cab-microfones-fontes.md`.
- **CA-3 (Fase, Distância e Eixo):** `phase_invert` com 2 mics iguais cancela a saída (`< -60 dBFS`); diferença de `distance_cm` produz filtro pente com entalhes na frequência acústica esperada (`c / (2·Δd)`); `angle_deg = 45` atenua `> 4 kHz`.
- **CA-4 (Validação Robusta de WAV):** Arquivos vazios, truncados, `channels=0`, `sr=0`, `sr` fora de `8–384 kHz` ou `> 500 ms` retornam `Err(InvalidImpulseResponseError)` sem panic.
- **CA-5 (Troca de IR Zero-Alloc/Zero-Dealloc RT):** Carregar/trocar IR durante áudio contínuo ocorre sem buraco de silêncio e com `alloc_count == 0` e `dealloc_count == 0` na thread RT.
- **CA-6 (Não-regressão):** Testes das Specs 005, 017, 022, 032 CA-7, 037 e 038 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Relacionadas / Legadas:** `Specs 005, 017, 022, 032, 037, 038, 051`.

## 20. Limitações conhecidas
- IRs estéreo atualmente têm o canal 1 (Left) extraído (informado na API via `source_channels`); convolução estéreo real aguarda barramento estéreo global.

## 21. Riscos técnicos
- Descartar `Arc<ImpulseResponse>` na thread RT causaria desalocação da `Vec` de partições FFT — obrigatório sempre mover para `retired_ir` e drenar no `RetiredReaper`.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `005`, `017`, `022`, `032`, `037`, `038`, `051`.
- **Documentos locais:** `docs/cab-microfones-fontes.md`.
- **Código atual:** `crates/ampforge-dsp/src/cab.rs`.
- **Testes existentes / futuros:** `contract_005_virtual_cab.rs`, `contract_017_resilience_ir_looper.rs`, `contract_022_cab_mic_ir.rs`, `contract_038_cab_ir_mics_resonance.rs`, futuro `contract_080_cab_ir_convolver.rs`.
