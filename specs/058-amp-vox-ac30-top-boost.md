---
spec: 058
titulo: Amplificador Vox AC30/6 Top Boost (1964 JMI, 30W EL84)
status: rascunho
tipo: amplificador
atualizado: 2026-11-19
dependencias: [053, 077, 078, 079]
---
# 058 — Amplificador Vox AC30/6 Top Boost (Rev. 1964 JMI)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `amp.vox.ac30_6_top_boost` (alias legado: `ac30_tb`, `uk_ac30_topboost`, `ac30`, `ac30_top_boost`)
- **Revisão (`model_revision`):** `1964-jmi-ac30-6-tb`
- **Categoria:** Amplificador valvulado (módulo independente)
- **Objetivo:** Especificar integralmente o circuito **JMI Vox AC30/6 Top Boost (1964)** — incluindo sua rede tonal **Top Boost dedicada** (não equivalente ao FMV), ausência real de NFB global, controle **Tone Cut** pós-inversor de fase, estágio de potência `4×EL84` cathode-biased (`50 Ω`), retificação `GZ34` e canais `Normal` / `Top Boost` — corrigindo as aproximações legadas do código atual.

## 2. Escopo
- Canais `Normal` (1 estágio de pré, sem Tone Stack Top Boost) e `Top Boost` / `Brilliant` (estágio adicional ECC83 + cathode follower + rede tonal Top Boost de 2 knobs `Treble` e `Bass`), com entradas High/Low (`−6 dB`).
- Circuito **Tone Cut** real (passa-baixas shunt RC entre os dois braços do inversor LTP antes das 4×EL84; girar horário corta agudos).
- Estágio de potência `4×EL84` polarizado por cátodo (`Rk = 50 Ω`, **sem NFB global**), retificador `GZ34 (5AR4)` com sag musical pronunciado e efeito Vib-Trem opcional da versão AC30/6.

## 3. Fora de escopo
- Variantes AC30 com válvula pentodo `EF86` no canal de entrada (AC30/4 de 1958–1960; disponível apenas como substituição de válvula na bancada Mod Shop, não como circuito AC30/4 completo).
- Versões modernas AC30CC2 / AC30C2 com Master Volume e Spring Reverb integrados (o AC30/6 1964 original não possui Master Volume nem reverb; ver §5/§11 sobre `master`).

## 4. Equipamento ou componente de referência
- **Fabricante:** Jennings Musical Industries (JMI) / Vox.
- **Modelo exato:** AC30/6 Top Boost (30W).
- **Período / Ano:** 1964 (circuito Top Boost integrado no painel superior, derivado do desenho JMI `OS/010`).
- **Papel na cadeia de sinal:** Referência de amplificador britânico EL84 sem NFB, com brilho articulado (*chime*), saturação progressiva do inversor/EL84 e controle de agudos pós-PI (`Tone Cut`).

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1964 JMI AC30/6 Top Boost (ECC83/EL84, GZ34, sem NFB, Tone Cut pós-LTP)`.
- **Variantes históricas relevantes (não misturadas):**
  - *AC30/4 (1958–1960):* usa pentodo EF86 no canal Normal e não possui Top Boost de fábrica.
  - *AC30/6 sem Top Boost (1961–1963):* 3 canais (Normal, Brilliant, Vib-Trem) sem controles de Treble/Bass; o módulo Top Boost surgiu como add-on traseiro (`OS/010`) antes de integrar o painel em 1964.
  - *AC30C2 / AC30CC2 (2005+):* adicionam Master Volume pós-PI e Reverb de mola.
- **Pendências de confirmação de revisão:**
  - **Master Volume:** a UI/API atual inclui `amp_1.master` em `amp.rs:1282–1285` para o AC30. Como o AC30/6 de 1964 não tem Master Volume (apenas versões C2/CC2 têm), confirmar em revisão manual se `master` será removido do painel AC30/6 (estrita fidelidade 1964) ou mantido como opção explícita *Post-PI Master (C2 Mod)*.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | Desenho JMI `OS/010` (Top Boost) e Esquema JMI `AC30/6` (1964) | AC30/6 Top Boost 1964 | Preamp, Top Boost, LTP, Tone Cut, 4×EL84 | Vox Showroom / Vox AC30 Guide (`voxac30.org.uk`) | 2026-11-19 | `[fonte secundária]` | `100k` slope, `50 pF` treble, `22 nF / 22 nF`, cathode follower ECC83, Tone Cut pós-LTP, `4×EL84` `50 Ω` cathode bias, `GZ34`, NFB = 0 |
| F2 | `docs/amps/voicing-fontes.md` | JMI Vox AC30/6 Top Boost (1964) | Seção 5 | `docs/amps/voicing-fontes.md` | 2026-11-19 | `[confirmado em documentação]` | Valores de componentes Top Boost, ausência de NFB e acoplamento 22 nF |
| F3 | Spec 039 (Contexto #6 e CA-6) | AC30 Top Boost | Auditoria do Tone Stack | `specs/039-amplificadores-fidelidade-e-robustez.md` | 2026-11-19 | `[confirmado em documentação]` | Registro de dívida: `tonestack.rs` ainda usa matriz FMV com `m_pos = 1.0` em vez da função de transferência específica do Top Boost |
| F4 | Mullard Datasheets — `EL84`, `ECC83`, `GZ34` | Válvulas padrão | Ratings / impedância | Datasheets Mullard | 2026-11-19 | `[confirmado em documentação]` | Curvas de operação de EL84 e resistência interna da retificadora GZ34 |

## 7. Fundamentação elétrica
- **Entradas:** `Normal High/Low`, `Brilliant (Top Boost) High/Low` (`Low` com atenuação `−6 dB`).
- **Canal Normal:** estágio único `1/2 ECC83` (`Rp = 220 kΩ`, `Rk = 1.5 kΩ || 25 µF` `[fonte secundária: F1]`), contornando a rede tonal Top Boost e indo direto ao nó de soma antes do LTP.
- **Canal Top Boost (JMI OS/010):**
  - V1 (ganho) → `Volume` (`1 MΩ` com bright cap) → V2a (estágio de ganho adicional `100 kΩ / 1.5 kΩ`) → V2b **Cathode Follower** (`Rk = 56–100 kΩ`) alimentando a **rede tonal Top Boost**.
  - **Rede Tonal Top Boost (3ª ordem própria, sem potenciômetro de Mid):** `R_slope = 100 kΩ`, `C1 = 50 pF`, `C2 = 22 nF`, `C3 = 22 nF`, `R_treble = 1 MΩ` (ou `220 kΩ` — confirmar variação 220k vs 1M no OS/010), `R_bass = 1 MΩ`, resistor fixo para terra `10 kΩ` `[fonte secundária: F1, F2]`. Atenção: a ligação do potenciômetro `Bass` no Top Boost difere do FMV clássico (o cursor/extremo cria interação característica onde abrir `Treble` simultaneamente reduz graves/médios).
- **Inversor de Fase e Tone Cut:** LTP `ECC83` acoplado por `22 nF` às grades das `4×EL84`. Entre os dois sinais fora de fase na saída do LTP encontra-se o **Tone Cut** (capacitor `0.0047 µF` em série com potenciômetro `250 kΩ` log `[fonte secundária: F1]`): conforme o potenciômetro é fechado em resistência (girado no sentido horário), as altas frequências dos dois braços se cancelam mutuamente (filtro passa-baixas variável pós-PI).
- **Estágio de Potência (4×EL84 + GZ34, sem NFB):** quatro `EL84` em push-pull com resistor de cátodo compartilhado `50 Ω` (classificação comercial "Classe A", operação elétrica real próxima a Classe AB1 muito quente com compressão precoce `[fonte secundária: F1]`), `B+ ≈ 320–330 V`, retificação por válvula `GZ34 (5AR4)` e **zero NFB global** (`nfb = 0`), maximizando a interação com a curva de impedância dos falantes Celestion Alnico Blue 2×12.

## 8. Comportamento esperado
- **Sem NFB global:** `nfb = 0` por padrão produz transientes abertos, projeção de médios-agudos (`~3–4 kHz`) e acoplamento reativo pronunciado com a ressonância da caixa (`~82 Hz` no Alnico Blue 2×12).
- **Ação real do Tone Cut:** girar `cut` de `0` para `10` atenua frequências acima de `2 kHz` em **`>= 6 dB`** (corte progressivo pós-inversor, domando brilho das EL84).
- **Canal Normal vs Top Boost:** no canal `Normal`, `Bass` e `Treble` não têm efeito (bypass físico do módulo Top Boost); no canal `Top Boost`, `Bass` e `Treble` exibem a interação assimétrica própria da rede OS/010 (sem knob `Middle`).
- **Compressão orgânica de EL84 + GZ34:** headroom inferior ao Twin Reverb e Plexi 100 W, entrando em saturação rica em harmônicos pares/ímpares em níveis médios de excitação com sag suave da retificadora GZ34.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Power | `amp_1.power` | boolean | bool | `false/true` | `true` | discreto | Alimentação AC | Mute suave `<= 2,5 ms` |
| Standby | `amp_1.standby` | boolean | bool | `false/true` | `false` | discreto | Corte B+ | Rampa exponencial invariante à taxa |
| Canal / Entrada | `amp_1.input_jack` | enum | id | `normal_high, normal_low, brilliant_high, brilliant_low` | `brilliant_high` | discreto | Seleciona Normal vs Top Boost e pad −6 dB | Bypass do Top Boost em Normal + pad `-6 dB` |
| Volume | `amp_1.gain` | number | scale | `0.0..=10.0` | `5.5` | audio log | Ganho do canal selecionado | Excitação de V1 (e V2a no Top Boost) |
| Treble | `amp_1.treble` | number | scale | `0.0..=10.0` | `6.0` | audio log | Braço agudo da rede Top Boost `50 pF` | Rede nodal dedicada Top Boost `f64` (Spec 078) |
| Bass | `amp_1.bass` | number | scale | `0.0..=10.0` | `5.5` | audio log | Braço grave `22 nF` da rede Top Boost | Rede nodal dedicada Top Boost `f64` (Spec 078) |
| Tone Cut | `amp_1.cut` | number | scale | `0.0..=10.0` | `5.0` | log invertido | Passa-baixas diferencial pós-LTP (horário corta agudos) | Filtro LPF `f64` `18 kHz → 750 Hz` pós-PI |
| Vib-Trem Speed | `amp_1.vibrato_speed` | number | scale | `0.0..=10.0` | `4.0` | linear/exp | Taxa do oscilador Vib-Trem (AC30/6) | Frequência LFO invariante à taxa |
| Vib-Trem Depth | `amp_1.vibrato_depth` | number | scale | `0.0..=10.0` | `0.0` | log | Intensidade Vib-Trem | Modulação AM/fase quando `> 0` |

*(Defaults de chassi/Mod Shop Spec 077: `12ax7`, `el84_variant`, `gz34`, `8 Ω` ou `16 Ω`, `bias=45 mA`, `sag=6.0`, `nfb=0.0`, `xformer_sat=5.0`, `B+_pre=240 V`, `B+_power=330 V`, `Rk_power=50 Ω`.)*

## 10. Contratos de integração
- `module_id = "amp.vox.ac30_6_top_boost"`, `model_revision = "1964-jmi-ac30-6-tb"`.
- Rejeita `amp_1.middle`, `amp_1.presence`, `amp_1.depth`, `amp_1.reverb`, `amp_1.jumpered`, `amp_1.volume_2`, `amp_1.spongy`, `amp_1.channel_mode` com `E_VALIDATION:` (e `master` conforme decisão de revisão em §5).

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/amp.rs` (linhas 353–366, 828–831, 1282–1285, 1345–1347, 1542–1553, 1590–1595) e `tonestack.rs:79–93`.
- **Dívidas registradas (críticas):**
  1. **Tone Stack Top Boost tratado como FMV (`tonestack.rs:79–93`, Spec 039 item #6):** o código atual usa a mesma matriz KCL 5×5 do FMV fixando `m_pos = 1.0`. Na implementação da Spec 058 + Spec 078, derivar a **matriz KCL própria da topologia Top Boost OS/010** (onde o braço de Bass conecta-se diferentemente do FMV) e validá-la contra SPICE/função analítica dentro de `±0,5 dB`.
  2. **Master Volume no AC30 (`amp.rs:1282–1285`):** decidir na revisão manual se será removido (fidelidade 1964) ou convertido em opção explícita *Post-PI Master*.
  3. **Aliasing de nome de canal `top_boost`:** Spec 051 T-04 já reabriu/corrigiu a aceitação da opção na UI/API; manter `brilliant_high`/`top_boost` mapeados de forma consistente no descritor.
- **Estrutura alvo:** `crates/ampforge-dsp/src/amps/ac30_tb.rs` (`VoxAc30TopBoost`).

## 12. Estratégia DSP
- **Circuit-level:** função de transferência nodal KCL dedicada da rede **Vox Top Boost** discretizada por bilinear `f64` (Spec 078); filtro `Tone Cut` em `f64` invariante à taxa.
- **Behavioral calibrado com memória:** V1 (+ V2a/V2b cathode follower no canal Top Boost) com ADAA1, estágio `4×EL84` com `nfb = 0.0` (alta projeção da curva de impedância reativa do falante), sag de `GZ34` (`tau_att=30 ms`, `tau_rel=80 ms`), e saturação LF do transformador Albion/Woden.
- **Oversampling:** seletivo (`2x` padrão, `4x`/`8x` selecionável, Spec 079).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; `alpha_cut` e coeficientes da rede Top Boost sempre calculados a partir da taxa ativa (`44,1 / 48 / 88,2 / 96 kHz`).

## 14. Gestão de estado
- Estados de V1/V2, `ac30_cut_lp`, `tonestack`, `lfo_phase`, `power_stage` e `sag_state` isolados na instância `VoxAc30TopBoost`.

## 15. Latência
- Igual à latência do fator de oversampling ativo (Spec 079).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 8%`** do deadline de 128 frames @ 48 kHz em `2x` OS e **`<= 20%`** em `8x` OS (build `--release`).

## 17. Testes
- Sweep 20 Hz–20 kHz validando a rede Top Boost dedicada em `Treble/Bass` = `0/5/10` contra referência analítica OS/010; medição da atenuação de `Tone Cut` em `> 2 kHz` (`0→10`); verificação de bypass do Tone Stack no canal `Normal`; teste de ausência de NFB (`nfb = 0`); teste de sag GZ34; aliasing em 5 kHz; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Tone Stack Top Boost Dedicado):** A rede Top Boost utiliza função de transferência derivada da topologia OS/010 (não a matriz FMV genérica) e coincide com a referência analítica dentro de `±0,5 dB` entre `50 Hz` e `10 kHz` nas posições `0`, `5` e `10` de `Treble` e `Bass`.
- **CA-2 (Tone Cut Real Pós-PI):** Variar `amp_1.cut` de `0` para `10` atenua a energia acima de `2 kHz` em `>= 6 dB` (e a resposta coincide em `±0,5 dB` nas taxas `44,1`, `48` e `96 kHz`).
- **CA-3 (Normal vs Top Boost):** Em `normal_high`, mover `treble` ou `bass` de `0` para `10` não altera o espectro (`diferença <= 0,1 dB`); em `brilliant_high`, a variação é `>= 8 dB`.
- **CA-4 (Ausência de NFB e Sag GZ34):** O amplificador inicia com `nfb = 0.0` e `rectifier = gz34`, produzindo pico de ressonância de carga reativa mais pronunciado e maior compressão de sag em bursts do que um amp com `nfb = 8.0` e `diode`.
- **CA-5 (Anti-Aliasing e RT Safety):** Aliasing residual `< -60 dBFS` em `4x`/`8x` OS para senoide 5 kHz a `-6 dBFS` com `gain = 10`; zero alocações/desalocações em `process()`; saída finita em `[-1.0, 1.0]`.
- **CA-6 (Erro e Não-regressão):** Parâmetros alheios (`middle`, `reverb`, `presence`, `spongy`) rejeitados com `E_VALIDATION:`; testes das Specs 020, 031 e 039 para `UkAc30TopBoost` permanecem verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 077` + `Spec 078` + `Spec 079`.
- **Relacionadas / Legadas:** `Specs 003, 020, 031, 039, 051`.

## 20. Limitações conhecidas
- Falta confirmação primária em esquema JMI 1964 digitalizado sobre a variante de potenciômetro `Treble` (`220 kΩ` vs `1 MΩ`) e se `Master Volume` será mantido como mod C2 ou removido.
- Efeito Vib-Trem simplificado (canal Vib-Trem completo do AC30/6 original usa rede de deslocamento de fase que mistura vibrato de fase + tremolo).

## 21. Riscos técnicos
- Substituir a aproximação FMV pela rede Top Boost real alterará ligeiramente o ganho de inserção e a curva de médios; exigirá recalibração de `tonestack_recovery_gain` e verificação de presets de fábrica baseados no AC30.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `003`, `020`, `031`, `039`, `051`.
- **Documentos locais:** `docs/amps/voicing-fontes.md`.
- **Código atual:** `crates/ampforge-dsp/src/amp.rs`, `crates/ampforge-dsp/src/tonestack.rs`.
- **Testes existentes / futuros:** `contract_031_authentic_amps.rs`, `contract_039_amps_fidelity.rs`, futuro `contract_058_amp_vox_ac30_tb.rs`.
