---
spec: 055
titulo: Amplificador Marshall JCM800 2203 (Rev. 1981 Master Volume Lead, 100W)
status: rascunho
tipo: amplificador
atualizado: 2026-11-20
dependencias: [053, 077, 078, 079]
---
# 055 — Amplificador Marshall JCM800 2203 (Rev. 1981 Master Volume Lead)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `amp.marshall.jcm800_2203` (aliases legados: `jcm800_2203`, `brit_jcm800_2203`, `brit800`)
- **Revisão (`model_revision`):** `1981-jcm800-2203-jmp-circuit`
- **Categoria:** Amplificador valvulado (`amplificador`)
- **Natureza do módulo:** Equipamento físico real
- **Objetivo:** Especificar o circuito elétrico real, os controles do painel frontal, a cascata de pré-amplificação com *cold clipper* assimétrico V1b (`Rk = 10 kΩ` sem bypass), a rede inter-estágio `470 kΩ || 470 pF` + `470 kΩ`, o chaveamento real das entradas `High` e `Low` e os critérios de validação do **Marshall JCM800 Model 2203 (100W, revisão 1981)**.

## 2. Escopo
- Roteamento autêntico dos jacks `HIGH` (cascata de 3 estágios de ganho: V1a → `Pre-Amp Volume` → V1b *cold clipper* → divisor `470 kΩ || 470 pF` → V2a → V2b *cathode follower*) e `LOW` (desconecta V1a, enviando o sinal de entrada via `Pre-Amp Volume` diretamente para V1b).
- Tone Stack passivo FMV Marshall (`33 kΩ / 470 pF / 22 nF / 22 nF`, `Treble 220 kΩ`, `Bass 1 MΩ`, `Middle 22 kΩ`) acionado pelo seguidor de cátodo V2b.
- `Master Volume` (`1 MΩ` log) pós-tone stack alimentando o inversor de fase LTP (V3 `ECC83`) e o estágio de potência push-pull `4×EL34` fixed-bias com malha NFB e controle `Presence`.
- Resolução documental da divergência dos filtros de voicing por derivação analítica da rede inter-estágio `470 kΩ || 470 pF` para `470 kΩ` à terra.

## 3. Fora de escopo
- Série **Effects Loop** do painel traseiro da reedição **JCM800 2203X** (documentado no manual `BOOK-00067-00` pp. 17–18 como adição moderna da reedição com chave *True Bypass*; o circuito original de 1975–1981 não possui FX loop).
- Modelos JCM800 de dois canais com diodos de clipping (2205 / 2210) ou 2203KK.

## 4. Equipamento ou componente de referência
- **Fabricante:** Marshall Amplification plc.
- **Modelo exato:** JCM800 Lead Series Model 2203 (100W Master Volume Head).
- **Período / Ano:** 1981 (circuito idêntico ao JMP 2203 de 1975–1980 em gabinete JCM800 com entradas verticais; F1 p. 15).
- **Papel na cadeia de sinal:** Cabeçote valvulado de 100 W que introduziu pré-amplificação em cascata com Master Volume pós-tone stack.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1981 JCM800 2203 (entradas verticais, sem FX loop, 3×ECC83 + 4×EL34)`.
- **Distinção documental obrigatória entre o Manual F1 (`2203 Reissue`) e o Esquema de 1981 (F2/F3):**
  - O manual `marshall-jcm800-2203-owners-manual-de.pdf` (`BOOK-00067-00 / 2 / 02`) em `docs/ui-desktop/fontes/` refere-se à **reedição JCM800 2203X** (que adicionou o loop de efeitos traseiro com chave bypass, p. 16 e p. 18).
  - Esse manual de usuário (`PRIMARY_OFFICIAL`) comprova a **ordem e função dos controles do painel frontal** (p. 17, itens 1–10) e confirma historicamente que o circuito de 1981 é idêntico ao JMP 2203 de 1975 (p. 15), mas **não contém o esquema elétrico interno**.
  - Os valores internos de resistores e capacitores (`V1a Rk=2,7 kΩ || 0,68 µF`, `V1b Rk=10 kΩ` sem bypass, `470 kΩ || 470 pF`, `FMV 33k/470pF/22nF/22nF`) são sustentados pelos esquemas de serviço de fábrica JMP/JCM800 2203 (`SECONDARY_TECHNICAL`: F2, F3).

## 6. Fontes técnicas e evidência por informação

### 6.1 Catálogo de fontes verificadas
| ID | Fonte | Fabricante / Modelo / Revisão | Página / Folha / Seção | URL / Caminho direto | Consulta | Classificação | Limitações da fonte |
|---|---|---|---|---|---|---|---|
| F1 | *Marshall JCM800 2203 Owners Manual* (`BOOK-00067-00 / 2 / 02`) | Marshall Amplification / JCM800 2203 Reissue | pp. 13–18 (Histórico 1975/1981 p. 15; Frontplatte itens 1–10 p. 17; Rückseite itens 1–11 p. 18) | `docs/ui-desktop/fontes/marshall-jcm800-2203-owners-manual-de.pdf` | 2026-11-20 | `SECONDARY` | Manual de operação da reedição (em alemão); comprova controles do painel e histórico, mas não traz esquema elétrico |
| F2 | Dr. Tube — *Marshall JMP / JCM800 2203 100W Lead Schematics (1977–1981)* | Marshall Model 2203 (100W Master Volume) | Folha de esquema pré e potência 2203 | `https://drtube.com/marshall/` | 2026-11-20 | `SECONDARY_TECHNICAL` | Esquema digitalizado em acervo técnico independente |
| F3 | Rob Robinette — *How the Marshall JCM800 Works* | Marshall JCM800 2203 / 2204 | Seções: High & Low Inputs, Cold Clipper V1b, Treble Peaking Network, Cathode Follower, Tone Stack, LTP | `https://robrobinette.com/How_the_Marshall_JCM800_Works.htm` | 2026-11-20 | `SECONDARY_TECHNICAL` | Análise técnica detalhada do circuito 2203/2204 |
| F4 | D. T. Yeh & J. O. Smith — *Discretization of the '59 Fender Bassman Tone Stack* (DAFx-06) | Rede passiva FMV | pp. 1–8 | `https://ccrma.stanford.edu/~dtyeh/papers/yeh06_dafx.pdf` | 2026-11-20 | `ACADEMIC` | Equações KCL e transformada bilinear da rede FMV |
| F5 | Specs 003, 016, 020, 031, 039, 051 e `docs/amps/voicing-fontes.md` | Contratos internos AMPFORGE | Seção 2 de `voicing-fontes.md` | `docs/amps/voicing-fontes.md`, `specs/031-*.md`, `specs/039-*.md` | 2026-11-20 | `INTERNAL_REFERENCE` | Registra estado atual e divergência dos filtros de voicing |

### 6.2 Evidência por afirmação técnica
| Informação / Parâmetro de Circuito | Fonte (ID) | Revisão | Página / Folha / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Ordem frontal dos 10 controles: `Power (1), Standby (2), Presence (3), Bass (4), Middle (5), Treble (6), Master Volume (7), Pre-Amp Volume (8), High Input (9), Low Input (10)` | F1 | 2203 (1981 / Reissue) | p. 17 (*Frontplatte*, itens 1–10) | `SECONDARY` | Confirmado |
| Circuito de 1981 idêntico ao 2203 de 1975; FX loop traseiro pertence apenas à reedição (com True Bypass) | F1 | 2203 (1975 / 1981 / Reissue) | pp. 15–16 (*Einleitung*) e p. 18 (*Effektweg Bypass*) | `SECONDARY` | Confirmado |
| Entrada `HIGH` passa por V1a (`Rp=100 kΩ, Rk=2,7 kΩ || 0,68 µF`, acoplamento `22 nF`) antes do `Pre-Amp Volume` (`1 MΩ` com bright cap `1000 pF`); entrada `LOW` bypassa V1a | F2, F3 | 1981 JCM800 2203 | Esquema 2203 / *High & Low Inputs* | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| Estágio V1b *Cold Clipper*: `Rp = 100 kΩ`, **`Rk = 10 kΩ` sem capacitor de bypass**, gerando ceifamento fortemente assimétrico | F2, F3 | 1981 JCM800 2203 | Esquema 2203 / *Cold Clipper* | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| Rede de peaking V1b → V2a: acoplamento `22 nF` seguido por divisor `470 kΩ || 470 pF` em série com `470 kΩ` para terra | F2, F3 | 1981 JCM800 2203 | Esquema 2203 / *Treble Peaking* | `SECONDARY_TECHNICAL` | Confirmado em fonte secundária |
| V2a (`Rp=100 kΩ, Rk=820 Ω`) acoplado em DC ao seguidor de cátodo V2b (`Rk=100 kΩ`) acionando Tone Stack FMV (`33 kΩ, 470 pF, 22 nF, 22 nF`) e `Master Volume` (`1 MΩ`) | F2, F3, F4 | 1981 JCM800 2203 | Esquema 2203 / DAFx-06 | `SECONDARY_TECHNICAL` / `ACADEMIC` | Confirmado em fonte secundária e artigo |

## 7. Fundamentação elétrica e resolução da divergência de voicing
- **Cascata de Entrada (`HIGH` vs `LOW` — F2, F3):**
  - No jack `HIGH`, a guitarra passa pelo grid stopper `68 kΩ` para a grade de **V1a** (`Rp = 100 kΩ`, `Rk = 2,7 kΩ || 0,68 µF`, ganho `≈ +30 dB` acima de `~100 Hz`), acopla via `22 nF` ao potenciômetro `Pre-Amp Volume` (`1 MΩ` log com bright cap `1000 pF`) e segue para a grade de **V1b**.
  - No jack `LOW`, o contato normalizado do jack `HIGH` abre e o sinal da guitarra bypassa completamente V1a, passando pelo `Pre-Amp Volume` diretamente para **V1b** (operando com 2 estágios de ganho em vez de 3).
- **V1b Cold Clipper (`Rk = 10 kΩ` unbypassed — F2, F3):** o resistor de cátodo elevado (`10 kΩ`, ~6× maior que o padrão `1,5 kΩ`) polariza V1b muito frio (próximo ao corte de corrente de placa), fazendo o semiciclo negativo cortar cedo enquanto o semiciclo positivo conduz, gerando forte 2º harmônico (`H2`) e realimentação degenerativa de corrente que enxuga graves.
- **Resolução analítica da divergência dos filtros de voicing (`470 kΩ || 470 pF` + `470 kΩ`):**
  - Existia conflito entre `docs/amps/voicing-fontes.md` (`peaking 1600 Hz +2,0 dB`, `low_shelf 110 Hz -1,5 dB`) e `amp.rs:812–815` (`peaking 1650 Hz +3,8 dB`, `low_shelf 180 Hz -4,2 dB`). Ambos eram aproximações heurísticas (`INFERENCE`).
  - A função de transferência analítica exata da rede física entre V1b e V2a — capacitor de acoplamento $C_c = 22\text{ nF}$ seguido pelo divisor $Z_1 = R_1 \parallel C_p$ ($R_1 = 470\text{ k}\Omega$, $C_p = 470\text{ pF}$) sobre $R_2 = 470\text{ k}\Omega$ em paralelo com a capacitância de entrada/Miller de V2a ($C_{\text{in}} \approx 100\text{ pF}$) — possui:
    1. Passa-altas de acoplamento em $f_{\text{hp}} = \frac{1}{2\pi (R_1 + R_2) C_c} \approx 7{,}7\text{ Hz}$ (mais o corte de cátodo de V1a em $\sim 87\text{ Hz}$);
    2. Zero de transmissão do divisor em $f_z = \frac{1}{2\pi R_1 C_p} = \frac{1}{2\pi \cdot 470\times 10^3 \cdot 470\times 10^{-12}} \approx \mathbf{720{,}8\text{ Hz}}$ e polo em $f_p = \frac{R_1 + R_2}{2\pi R_1 R_2 (C_p + C_{\text{in}})} \approx \mathbf{1265\text{–}1441\text{ Hz}}$ (elevação de $+6{,}02\text{ dB}$ acima de $1{,}4\text{ kHz}$ em relação ao patamar de $-6{,}02\text{ dB}$ em baixas frequências), seguido pelo polo de Miller em $\sim 6{,}4\text{–}7{,}0\text{ kHz}$.
  - Na implementação modular da Spec 055, a rede inter-estágio deve ser discretizada diretamente por sua equação RC analítica (`ANALYTICAL`), substituindo os biquads arbitrários de `amp.rs:812–815`.

## 8. Comportamento esperado
- **Assimetria harmônica pronunciada no Cold Clipper V1b:** no jack `high` com `gain = 8.0`, a saída apresenta razão `H2/H1 >= -24 dB` (com `triode_asym = 0.20`), caindo `>= 15 dB` quando `triode_asym = 0.0`.
- **Diferença real entre `HIGH` e `LOW`:** em regime linear (`1 kHz`, `-80 dBFS`), a entrada `low` pula o 1º estágio V1a e passa pelo divisor 68k/68k, resultando em `HIGH − LOW = 20·log10(ganho_estagio1) + 6,02 ± 1,0 dB`, e reduz drasticamente a THD em `gain = 8.0` (redução relativa de THD `>= 30%`).
- **Independência entre `Pre-Amp Volume` e `Master Volume`:** `gain` controla a saturação da cascata V1a→V1b→V2a/b antes do Tone Stack; `master` controla o nível pós-Tone Stack e a saturação de potência do LTP + `4×EL34`.

## 9. Parâmetros e controles
> Ordem física do painel frontal (Manual `BOOK-00067-00` F1, p. 17, itens 1–10): `1. POWER · 2. STANDBY · 3. PRESENCE · 4. BASS · 5. MIDDLE · 6. TREBLE · 7. MASTER VOLUME · 8. PRE-AMP VOLUME · 9. HIGH · 10. LOW`.

| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Origem (`Físico` vs `Mod Shop`) | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|---|
| Power | `amp_1.power` | boolean | bool | `false/true` | `true` | discreto | `Físico` (Item 1) | Alimentação AC | Mute suave `<= 2,5 ms` |
| Standby | `amp_1.standby` | boolean | bool | `false/true` | `false` | discreto | `Físico` (Item 2) | Alta tensão B+ | Rampa exponencial invariante à taxa |
| Presence | `amp_1.presence` | number | scale | `0.0..=10.0` | `5.0` | log | `Físico` (Item 3) | Reduz NFB em agudos | Filtro acoplado a `eff_nfb` |
| Bass | `amp_1.bass` | number | scale | `0.0..=10.0` | `5.5` | audio log | `Físico` (Item 4) | Braço de graves FMV `1 MΩ / 22 nF` | KCL 5×5 + bilinear `f64` |
| Middle | `amp_1.middle` | number | scale | `0.0..=10.0` | `5.0` | log/linear | `Físico` (Item 5) | Braço de médios FMV `22 kΩ` | KCL 5×5 + bilinear `f64` |
| Treble | `amp_1.treble` | number | scale | `0.0..=10.0` | `6.0` | log | `Físico` (Item 6) | Braço agudo FMV `220 kΩ / 470 pF` | KCL 5×5 + bilinear `f64` |
| Master Volume | `amp_1.master` | number | scale | `0.0..=10.0` | `5.0` | audio log (`1 MΩ`) | `Físico` (Item 7) | Nível pós-tone stack enviado ao LTP/EL34 | Ganho pós-FMV + drive do power amp |
| Pre-Amp Volume | `amp_1.gain` | number | scale | `0.0..=10.0` | `6.5` | audio log (`1 MΩ`) | `Físico` (Item 8) | Ganho entre V1a e V1b + bright cap `1 nF` | Ganho pré-V1b + bright cap dinâmico |
| Input Jack | `amp_1.input_jack` | enum | id | `high, low` | `high` | discreto | `Físico` (Itens 9–10) | `high` = 3 estágios; `low` = bypassa V1a | Chaveamento do estágio V1a |

## 10. Contratos de integração
- Implementa `AudioProcessor` (Spec 053) com `module_id = "amp.marshall.jcm800_2203"` e `model_revision = "1981-jcm800-2203-jmp-circuit"`.
- Rejeita `volume_2`, `jumpered`, `cut`, `reverb`, `spongy`, `channel_mode`, `vibrato_speed`, `vibrato_depth` com `E_VALIDATION:`.

## 11. Arquitetura de implementação
- **Código atual mapeado (HEAD `d891389`):** `crates/ampforge-dsp/src/amp.rs` (linhas `297–310, 812–815, 1213, 1286–1289, 1472–1505`) e `crates/ampforge-dsp/src/tonestack.rs` (linhas `51–64`).
- **Problemas e dívidas encontrados no código atual:**
  1. `ARCHITECTURE_CONFLICT` (`amp.rs:812–815` vs `docs/amps/voicing-fontes.md`): biquads heurísticos divergentes em vez da função de transferência RC analítica do divisor `470 kΩ || 470 pF` + `470 kΩ`.
  2. `FACTUAL_ERROR` (`amp.rs:1482–1483`): quando `is_low == true`, o código faz `s_v1a_in * base_drive`, mantendo o ganho multiplicativo `base_drive` (que inclui `od_gain_lin` de até `+36 dB`), em vez de omitir o ganho de ~30 dB do triodo V1a como no jack `LOW` real.
  3. `FACTUAL_ERROR` (`amp.rs:1213, 1474`): o parâmetro `panel.overdrive` adiciona até `+36 dB` extras ao JCM800 2203, que fisicamente só possui `Pre-Amp Volume` e `Master Volume`.

## 12. Estratégia DSP
- **Circuit-level (`f64`):** Tone Stack FMV (`33 kΩ / 470 pF / 22 nF / 22 nF`, `R_src = 1,3 kΩ`) via KCL + bilinear (Spec 078); filtro lead-lag de 1ª ordem derivado exatamente do divisor `470 kΩ || 470 pF` + `470 kΩ` ($f_z = 720{,}8\text{ Hz}$, $f_p = 1441{,}6\text{ Hz}$).
- **Behavioral calibrado com memória (`f64`):** V1a (`Rk=2,7k||0,68µF`, bypassado em `LOW`) → `Pre-Amp` + bright cap `1 nF` → V1b *cold clipper* assimétrico (`Rk=10 kΩ`) → rede `470k||470pF` + Miller LP (`~6,8 kHz`) → V2a + V2b *cathode follower* → FMV → `Master` → LTP + `4×EL34` push-pull com NFB e `Presence`.
- **Oversampling:** seletivo (`2x` padrão, `4x`/`8x` selecionável — Spec 079).

## 13. Requisitos de tempo real
- `alloc_count == 0` e `dealloc_count == 0` em `process()` na thread RT; coeficientes invariantes a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- Estados de V1a, V1b, V2a/b, rede `470k||470pF`, Tone Stack, LTP/EL34, sag e oversampler isolados na struct `MarshallJcm800_2203`.

## 15. Latência
- Igual à latência do oversampler ativo (Spec 079): `4` (`2x`), `8` (`4x`) ou `16` (`8x`) amostras no modo `MinPhaseIir`.

## 16. Consumo de CPU
- Meta (`ENGINEERING_TARGET`): `<= 9,0%` do orçamento de `128` frames @ `48 kHz` em `2x` OS (`--release`).

## 17. Testes
- Sinais determinísticos em `44,1`, `48` e `96 kHz`, `block_size = 128`: senoides `100 Hz`, `1 kHz`, `5 kHz`; sweep `20 Hz–10 kHz`; medição DFT de `H2/H1` no cold clipper, ganho e THD `high` vs `low`, compressão de power via `master` e aliasing `< -60 dBFS`.

## 18. Critérios de aceite
- **CA-1 (`ANALYTICAL` — Tone Stack FMV 2203 e Divisor `470 kΩ || 470 pF`):** Com impulso unitário (`x[0] = 1.0`), `sample_rate = 48000 Hz` (`8x` OS), `block_size = 128`, a magnitude DFT do Tone Stack FMV (`33 kΩ / 470 pF / 22 nF / 22 nF`) entre `50 Hz` e `10 kHz` para `(B,M,T) ∈ {(5,5,5), (10,5,0), (0,5,10)}` permanece dentro de `±0,5 dB` da solução KCL contínua, e o filtro inter-estágio reproduz a curva analítica do divisor `470 kΩ || 470 pF` para `470 kΩ` ($f_z = 720{,}8\text{ Hz}$, $f_p = 1441{,}6\text{ Hz}$) dentro de `±0,5 dB` até `6 kHz`.
- **CA-2 (`DOCUMENTED` — Cold Clipper Assimétrico V1b `Rk = 10 kΩ`):** Com senoide de `1 kHz` a `-12 dBFS` (`0.2512` pico), `sample_rate = 48000 Hz`, `block_size = 128`, `input_jack = "high"`, `gain = 8.0`, `master = 5.0`, `triode_asym = 0.20`, a magnitude DFT do 2º harmônico (`2 kHz`) na saída é **`>= -24,0 dB`** em relação à fundamental (`1 kHz`); ao ajustar `triode_asym = 0.0`, o 2º harmônico reduz-se em `>= 15,0 dB` (F2, F3).
- **CA-3 (`DOCUMENTED` — Entradas High vs Low Sensitivity):** Com senoide de `1 kHz`, `sample_rate = 48000 Hz`, `block_size = 128`: (a) a `-80 dBFS` com `gain = 5.0`, `input_jack = "high"` vs `"low"` apresenta diferença **`HIGH − LOW = 20·log10(ganho_estagio1) + 6,02 ± 1,0 dB`** (bypass de V1a + divisor resistivo 68k/68k); (b) a `-12 dBFS` com `gain = 8.0`, `input_jack = "low"` apresenta THD **`>= 30%` menor** (relativa) do que `"high"` devido ao bypass de V1a (F1 p. 17 Itens 9–10; F2; F3).
- **CA-4 (`DOCUMENTED` — Independência Pré-Amp vs Master Volume):** Com senoide de `1 kHz` a `-12 dBFS`, `sample_rate = 48000 Hz`, `block_size = 128`, `gain = 4.0`, elevar `master` de `2.0` para `9.0` eleva a THD do estágio de potência e reduz o crest factor em `>= 1,5 dB` (F1 p. 17 Item 7).
- **CA-5 (`ENGINEERING_TARGET` — Anti-Aliasing e Invariância Multi-Taxa):** Com senoide de `5 kHz` a `-6 dBFS`, `gain = 10.0`, `sample_rate = 48000 Hz`, `block_size = 128`, DFT Hann de `16384` pontos após `4096` amostras, toda componente de aliasing não-harmônica em `50 Hz–20 kHz` permanece **`< -60 dBFS`** em `4x` e `8x` OS; a resposta linear em `gain = 2.0` varia `<= 0,5 dB` entre `44,1`, `48` e `96 kHz`.
- **CA-6 (`ENGINEERING_TARGET` — Validação de Painel e Segurança RT):** Enviar `volume_2`, `cut` ou `reverb` para `jcm800_2203` retorna HTTP 400 (`E_VALIDATION:`); `CountingAllocator` na thread RT registra `alloc_count == 0` e `dealloc_count == 0` em `process()`.
- **CA-7 (`REGRESSION` — Não-regressão):** Os testes existentes `contract_031_authentic_amps.rs`, `contract_039_amps_fidelity.rs` e `contract_051_manual_test_fixes.rs` aplicáveis ao `BritJcm800_2203` continuam passando.

## 19. Dependências
- **Obrigatórias:** `Spec 053`, `Spec 077`, `Spec 078`, `Spec 079`.
- **Relacionadas / Legadas:** `Specs 003, 016, 020, 031, 039, 051`.

## 20. Limitações conhecidas
- A condução de grade de V2a carregando o divisor `470 kΩ || 470 pF` é modelada por joelho suave + ADAA1 em vez de solução nodal iterativa MNA por amostra.

## 21. Riscos técnicos
- Substituir o biquad heurístico (`1650 Hz +3,8 dB` / `180 Hz -4,2 dB`) pelo filtro lead-lag analítico (`720,8 Hz / 1441,6 Hz`) exige calibração de ganho unitário em `1 kHz` para preservar a separação espectral verificada em `contract_039_amps_fidelity.rs` (`test_ca_09`).

## 22. Matriz de rastreabilidade
| Requisito | Fonte | Spec/CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Ordem e controles do painel frontal (`Power, Standby, Presence, Bass, Middle, Treble, Master, Pre-Amp, High/Low`) | F1 (p. 17) | `Spec 055 / CA-6` | `apps/sonix-app/ui/src/controls.registry.ts`, `crates/sonix-desktop/src/api.rs` | `tests/contract/031-amplificadores-fieis-ui.test.js` | `TESTED` |
| Tone Stack FMV `33k / 470pF / 22nF / 22nF` | F2, F3, F4 | `Spec 055 / CA-1` | `crates/ampforge-dsp/src/tonestack.rs:51–64 @ c20cc65` | `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs`, `contract_039_amps_fidelity.rs` | `TESTED` |
| Estágio V1b Cold Clipper assimétrico (`Rk = 10 kΩ` sem bypass) | F2, F3 | `Spec 055 / CA-2` | `crates/ampforge-dsp/src/amp.rs:1492–1497 @ c20cc65` | `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs` (`ca_4`) | `TESTED` |
| Rede inter-estágio analítica `470 kΩ || 470 pF` + `470 kΩ` (substituindo biquads divergentes) | F2, F3 | `Spec 055 / CA-1` | `crates/ampforge-dsp/src/amp.rs:812–815 @ c20cc65, 1499–1500` *(divergente de `voicing-fontes.md`)* | Planejado: `contract_055_amp_marshall_jcm800_2203.rs` | `SPECIFIED` |
| Chaveamento `High` (3 estágios com V1a) vs `Low` (bypassa V1a, −6 dB) | F1 (p. 17), F2, F3 | `Spec 055 / CA-3` | `crates/ampforge-dsp/src/amp.rs:1481–1491 @ c20cc65` | `crates/ampforge-dsp/tests/contract_031_authentic_amps.rs` (`ca_4`) | `TESTED` |
| Separação entre `Pre-Amp Volume` e `Master Volume` pós-tone stack | F1 (p. 17), F2 | `Spec 055 / CA-4` | `crates/ampforge-dsp/src/amp.rs:1286–1289 @ c20cc65` | Planejado: `contract_055_amp_marshall_jcm800_2203.rs` | `IMPLEMENTED_UNVERIFIED` |
