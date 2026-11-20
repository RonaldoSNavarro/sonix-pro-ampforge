---
spec: 066
titulo: Compressor FET UREI 1176LN (Rev. D/E Blackface) e Topologias Auxiliares de Dinâmica
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053]
---
# 066 — Compressor FET UREI 1176LN (Rev. D/E Blackface)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `rack.urei.1176ln` (chaves `comp_2.*`)
- **Revisão (`model_revision`):** `rev-d-e-blackface-class-a`
- **Categoria:** Rack / Compressor FET de Estúdio (módulo independente)
- **Objetivo:** Especificar fielmente o comportamento elétrico, os tempos ultrarrápidos de ataque (`20–800 µs`) e release (`50–1100 ms`) em escala **invertida (1 = lento → 7 = rápido)**, os ratios `4:1`, `8:1`, `12:1`, `20:1` e o modo **All-Buttons-In ("British Mode") monotônico sem DC** do **UREI 1176LN**, registrando ainda a separação das demais topologias presentes hoje em `fx/dynamics.rs` (`PedalRossDyna`, `OpticalLa2a`, `VcaMasterBus`).

## 2. Escopo
- Modelagem do estágio atenuador JFET (limiar fixo controlado por `Input`, `Output` makeup real `-24..+24 dB`), escalas autênticas de `Attack` (`1..7` → `800 µs..20 µs`) e `Release` (`1..7` → `1100 ms..50 ms`), 5 modos de ratio (`4`, `8`, `12`, `20`, `all`), medição real de Gain Reduction (`GR` para VU) e `parallel_mix_pct`.
- Registro de desacoplamento das 3 topologias auxiliares atualmente em `CompressorTopology` (`PedalRossDyna`, `OpticalLa2a`, `VcaMasterBus`), que hoje existem apenas em biblioteca (`ampforge-dsp`) sem UI/IPC dedicados.

## 3. Fora de escopo
- Transformadores físicos de entrada/saída por elementos finitos (histerese/coloração de transformador será modelada de forma comportamental calibrada).
- UI dedicada para `Ross/Dyna Comp`, `LA-2A` e `VCA Bus` (serão promovidos a specs independentes apenas quando expostos no catálogo da UI).

## 4. Equipamento ou componente de referência
- **Fabricante:** UREI / Universal Audio (Bill Putnam).
- **Modelo exato:** 1176LN Peak Limiter.
- **Período / Ano:** 1969–1973 (Revisões D e E "Blackface", estágio de saída Classe A discreto + circuito LN *Low Noise*).
- **Papel na cadeia de sinal:** Compressor/limitador FET ultrarrápido pós-pré ou pós-gabinete (rack na Linha B / estúdio), conferindo punch, sustain controlado e saturação harmônica em `All-Buttons-In`.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `UREI 1176LN Rev. D / E (Blackface, estágio de saída Classe A 1108, feedback topology)`.
- **Variantes históricas:** Rev A "Bluestripe" (mais ruidoso/agressivo sem circuito LN), Rev F/G (saída push-pull + medidor op-amp).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | *UREI 1176LN Instruction Manual* | Rev D / E / F | Specifications: Attack, Release, Ratios, Meter | Universal Audio / History of Recording Archives | 2026-11-19 | `[confirmado em documentação]` | Attack `20 µs` a `800 µs` (sentido horário = mais rápido), Release `50 ms` a `1100 ms`, ratios `4/8/12/20:1` |
| F2 | Universal Audio — *The 1176 All-Buttons-In Mode* | 1176LN | Nota técnica oficial | https://www.uaudio.com/ | 2026-11-19 | `[confirmado em documentação]` | Alteração de bias do FET, ratio efetivo ~12:1–20:1+, lag inicial no ataque e saturação harmônica |
| F3 | D. Giannoulis, M. Massberg, J. Reiss — *Digital Dynamic Range Compressor Design* (JAES 2012) | Compressores digitais | Topologia feedback vs feedforward, smooth peak/RMS | JAES Vol. 60, No. 6, 2012 | 2026-11-19 | `[confirmado em documentação]` | Estabilidade de detectores feedback/feedforward e curvas de knee |
| F4 | Specs 032 CA-4 e 041 CA-1/CA-2 | Contratos existentes | `fx/dynamics.rs` | `specs/032-*.md`, `specs/041-*.md` | 2026-11-19 | `[confirmado em documentação]` | `Output` makeup real, `All-Buttons-In` estritamente monotônico sem inversão de polaridade nem DC |

## 7. Fundamentação elétrica
- **Atenuador JFET em realimentação (feedback):** no 1176LN real, o sinal entra no potenciômetro `Input` (que varia o nível contra um limiar interno fixo, funcionando simultaneamente como drive e threshold relativo), passa pelo divisor resistivo com um **JFET (`2N5457` / `2N3707`)** cuja resistência dreno-fonte varia conforme a tensão de controle derivada da **saída** do amplificador de ganho `[confirmado em documentação: F1]`.
- **Escalas invertidas (`1 → 7`):**
  - `Attack`: `1` ≈ `800 µs` (`0,8 ms`, mais lento) → `7` ≈ `20 µs` (`0,02 ms`, mais rápido).
  - `Release`: `1` ≈ `1100 ms` (mais lento) → `7` ≈ `50 ms` (mais rápido) `[confirmado em documentação: F1]`.
- **All-Buttons-In ("British Mode"):** pressionar os 4 botões simultaneamente altera a polarização do circuito de controle do FET, deslocando o limiar efetivo para baixo, elevando a compressão e introduzindo saturação não linear assimétrica/suave de FET e bombamento intenso `[confirmado em documentação: F2]`.

## 8. Comportamento esperado
- **Tempos ultrarrápidos reais:** em `Attack = 7` (`20 µs`), o pico inicial de um burst de 1 kHz é contido em `< 0,5 ms`; em `Attack = 1` (`800 µs`), o transiente inicial atravessa antes da redução plena.
- **Monotonicidade estrita em `All-Buttons-In`:** a função de transferência estática em `[-1, +1]` é estritamente não decrescente, `sign(y) == sign(x)` para `|x| > 1e-3`, offset DC `< -60 dBFS`, com THD e compressão maiores que em `20:1` (Spec 041 CA-1).
- **Output Makeup real:** mover `comp_2.output` altera o nível de saída em dB de forma contínua e verificável (`±12 dB` mínimo).
- **Medição real de Gain Reduction:** `gain_reduction_db()` reflete exatamente a atenuação instantânea em dB (`<= 0 dB`) para o medidor VU.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `comp_2.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa/bypassa 1176LN | Crossfade sem cliques `<= 5 ms` |
| Input | `comp_2.input` | number | scale | `0.0..=10.0` | `5.0` | audio log | Empurra sinal contra limiar fixo do FET | Mapeia para threshold/drive interno |
| Output | `comp_2.output` | number | scale | `0.0..=10.0` | `5.0` | audio log | Ganho de maquiagem pós-FET (`-24..+24 dB`) | Multiplicador linear `10^(db/20)` suavizado |
| Attack | `comp_2.attack` | number | scale | `1.0..=7.0` | `4.0` | log invertido (`800→20 µs`) | `1` = mais lento (`0,8 ms`), `7` = mais rápido (`0,02 ms`) | Constante de ataque exponencial |
| Release | `comp_2.release` | number | scale | `1.0..=7.0` | `4.0` | log invertido (`1100→50 ms`) | `1` = mais lento (`1100 ms`), `7` = mais rápido (`50 ms`) | Constante de release exponencial |
| Ratio | `comp_2.ratio` | enum | id | `4, 8, 12, 20, all` | `4` | discreto | Seleciona razão de compressão ou All-Buttons-In | Altera ratio, limiar efetivo e saturação FET |
| Parallel Mix | `comp_2.mix_pct` | number | % | `0.0..=100.0` | `100.0` | linear | Compressão paralela (New York) | `dry * (1-m) + wet * m` |
| VU Mode (Visual) | `comp_2.vu_mode` | visual | id | `GR, +4, +8` | `GR` | visualOnly | Seleciona escala do VU | Apenas exibição na UI (`visualOnly: true`) |

## 10. Contratos de integração
- `module_id = "rack.urei.1176ln"`, `model_revision = "rev-d-e-blackface-class-a"`.
- Expõe `gain_reduction_db()` na telemetria de 30 Hz (`streams.rs`).

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/dynamics.rs` (linhas 343–552).
- **Dívidas registradas:**
  1. `CompressorProcessor` mistura 4 topologias (`StudioFet1176`, `PedalRossDyna`, `OpticalLa2a`, `VcaMasterBus`) em uma única struct com `match self.topology`; apenas `StudioFet1176` está exposto na UI/API. Separar `Urei1176Ln` em `pedals/comp_1176ln.rs` e manter as demais topologias como módulos separados somente se/quando forem promovidas ao catálogo com specs próprias.
  2. No código atual (`dynamics.rs:480`), o detector do 1176 é puramente *feedforward* sobre `abs_x`; evoluir para topologia *feedback* (detectando o sinal já atenuado por 1 amostra de atraso implícito) reproduz melhor o joelho orgânico do 1176LN.
  3. O shaper de `All-Buttons-In` (`dynamics.rs:540`) não usa ADAA nem oversampling; adicionar ADAA1 (ou 2× OS local) para conter aliasing em saturação intensa.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/comp_1176ln.rs` (`Urei1176Ln`).

## 12. Estratégia DSP
- **Behavioral calibrado:** detector de envoltória feedback/peak em `f64` com tempos exatos `20–800 µs` / `50–1100 ms`, curva de ganho FET (knee suave proporcional ao ratio), saturação FET monotônica com ADAA1 em `All-Buttons-In` e `Output` makeup suavizado.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; coeficientes de ataque/release calculados a partir da `sample_rate` ativa (`44,1 / 48 / 88,2 / 96 kHz`).

## 14. Gestão de estado
- `envelope`, `prev_gain`, `adaa_prev`, `gain_reduction_db` encapsulados por instância; `reset()` zera estados em O(1).

## 15. Latência
- **`0` amostras** (ADAA1 sem oversampling, ou declarada explicitamente se `2x` OS opcional for habilitado).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 1,5%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Curva estática `[-1..+1]` em `ratio = "all"` (monotonicidade, sinal preservado, DC `< -60 dBFS`); burst de 1 kHz para verificar `Attack = 7` mais rápido que `Attack = 1` e `Release = 7` mais rápido que `Release = 1`; verificação de `comp_2.output` (`±12 dB`); comparação `THD(all) > THD(20)`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Escalas Invertidas Autênticas):** `Attack = 7` reduz o pico transiente inicial mais rapidamente que `Attack = 1`; `Release = 7` recupera `90%` da redução de ganho em `< 150 ms` enquanto `Release = 1` leva `> 500 ms`.
- **CA-2 (All-Buttons-In Monotônico e Sem DC):** Em `ratio = "all"`, a transferência estática é monotônica não decrescente, `sign(y) == sign(x)` para `|x| > 1e-3`, DC `< -60 dBFS` e `THD(all) > THD(20)`.
- **CA-3 (Makeup Output Real):** Variar `comp_2.output` altera o nível RMS na saída em `>= 12 dB` de ponta a ponta sem alterar a redução de ganho medida.
- **CA-4 (Invariância de Taxa e RT Safety):** Constantes de tempo coincidem em `±5%` entre `44,1`, `48` e `96 kHz`; zero alocações/desalocações em `process()`; entradas `NaN`/`Inf` sanitizadas.
- **CA-5 (Não-regressão):** Testes das Specs 032 CA-4 e 041 CA-1/CA-2 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Relacionadas / Legadas:** `Specs 006, 024, 032, 041`.

## 20. Limitações conhecidas
- Histerese magnética dos transformadores UTC O-12 / 5002 originais da Rev D/E não é simulada fisicamente nesta revisão.
- As 3 topologias auxiliares (`PedalRossDyna`, `OpticalLa2a`, `VcaMasterBus`) permanecem na biblioteca sem painel de UI até ganharem specs próprias.

## 21. Riscos técnicos
- Constante de ataque ultrarrápida (`20 µs` ≈ 1 amostra a 48 kHz) pode gerar modulação de forma de onda (distorção em baixas frequências) — comportamento autêntico do 1176LN real que deve ser validado sem gerar instabilidade numérica.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `006`, `024`, `032`, `041`.
- **Código atual:** `crates/ampforge-dsp/src/fx/dynamics.rs`.
- **Testes existentes / futuros:** `contract_032_pedals_fx_cabs.rs`, `contract_041_dynamics_and_drives.rs`, futuro `contract_066_rack_comp_1176ln.rs`.
