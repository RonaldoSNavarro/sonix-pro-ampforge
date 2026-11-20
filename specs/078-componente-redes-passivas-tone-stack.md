---
spec: 078
titulo: Componente Compartilhado de Redes Passivas de Tone Stack (FMV, Vox Top Boost e James/Baxandall)
status: rascunho
tipo: componente-compartilhado
atualizado: 2026-11-19
dependencias: [053]
---
# 078 — Componente de Redes Passivas de Tone Stack (FMV, Top Boost e James/Baxandall)

## 1. Identificação e objetivo
- **Identificador estável (`module_id`):** `component.tonestack.passive_nodal_networks`
- **Revisão (`model_revision`):** `v1-kcl-bilinear-f64`
- **Categoria:** Componente compartilhado DSP (redes passivas interativas)
- **Objetivo:** Especificar o motor matemático de discretização exata de redes passivas de Tone Stack em `f64` — suportando a **família FMV** (Marshall, Fender AB763, Mesa Dual Rectifier), a **topologia dedicada Vox Top Boost (OS/010)** (corrigindo a aproximação FMV atual!) e a **rede passiva James / Baxandall** — com curvas de taper fiéis e zero alocação em tempo real.

## 2. Escopo
- Solver polinomial KCL em tempo contínuo `H(s) = B(s) / A(s)` (até 3ª ordem) + discretização via transformada bilinear em `f64` + filtro Direct Form II Transposed (`PassiveToneStack`).
- **Topologia 1 — FMV (matriz KCL 5×5):** parametrizável por `(R_slope, R_treble, R_bass, R_mid, R_load, C1, C2, C3, R_src)` + tapers individuais por amplificador (Marshall 1959/2203, Fender AB763, Mesa Rev F).
- **Topologia 2 — Vox Top Boost (rede nodal própria OS/010):** derivação KCL própria (sem forçar a matriz FMV!), modelando corretamente a interação específica entre `Treble` (`50 pF`), `Bass` (`22 nF / 22 nF`), `R_slope = 100 kΩ` e resistor de cauda fixo `10 kΩ`.
- **Topologia 3 — James / Baxandall Passivo:** rede real de dois braços independentes de graves e agudos (substituindo a aproximação FMV atual em `tonestack.rs:108–121`!).
- Função analítica pública `analog_magnitude_db(freq_hz)` para validação automatizada e plotagem honesta na Bancada CRT (Spec 047).

## 3. Fora de escopo
- Equalizadores gráficos ativos ou paramétricos (Spec 067) e Tone Stacks de pedais específicos (Big Muff Spec 062, DS-1 Spec 064).

## 4. Equipamento ou componente de referência
- **Referências:** Duncan Amps *Tone Stack Calculator (TSC)*; Yeh & Smith (DAFx-06); esquemas originais Marshall 1959/2203, Fender AB763, Mesa Rev F, JMI Vox AC30 OS/010 e James/Baxandall clássico.

## 5. Revisão e variantes
- **Revisão:** `v1-kcl-bilinear-f64`.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | D. T. Yeh & J. O. Smith — *Discretization of the '59 Fender Bassman Tone Stack* (DAFx-06) | Rede FMV 3ª ordem | Polinômios `b0..b3`, `a0..a3` e bilinear | Proc. DAFx-06 (CCRMA) | 2026-11-19 | `[confirmado em documentação]` | Equações analíticas exatas do tone stack FMV e estabilidade bilinear |
| F2 | Duncan Amps *Tone Stack Calculator (TSC)* | Marshall, Fender, Vox, James | Valores R/C e topologias | https://www.duncanamps.com/tsc/ | 2026-11-19 | `[fonte secundária]` | Referência cruzada de curvas FMV vs Vox Top Boost vs James |
| F3 | `crates/ampforge-dsp/src/tonestack.rs` e Spec 039 CA-5/CA-6 | Implementação atual | Linhas 1–401 | `crates/ampforge-dsp/src/tonestack.rs` | 2026-11-19 | `[confirmado em documentação]` | Implementação KCL 5×5 FMV existente e dívidas em Vox Top Boost / Baxandall |

## 7. Fundamentação elétrica
- **Impedância de fonte `R_src`:** um tone stack alimentado por **cathode follower** (Marshall 1959/2203, AC30 Top Boost, Dual Rectifier) vê `R_src ≈ 1–2 kΩ`, enquanto um tone stack alimentado pela **placa** de um triodo 12AX7 (Fender Twin AB763) vê `R_src ≈ 38 kΩ` (`Rp || r_p`). Incluir `R_src` explicitamente na matriz KCL aumenta significativamente a fidelidade da perda de inserção e da curva de agudos!
- **Tapers de potenciômetros:** `audio_taper` 10% log (CTS/Alpha), linear (`Middle` do AB763) e híbrido 10% log (`Middle` do Marshall 1959HW confirmado no manual F1 da Spec 054).
- **Normalização temporal:** `p = s · (R_REF · C_REF)` (`100 kΩ · 10 nF = 1 ms`) previne subfluxo/overflow nos determinantes polinomiais em `f64`.

## 8. Comportamento esperado
- **Fidelidade `<= ±0,5 dB`:** em `48 kHz` (ou taxa oversampled `96–384 kHz`), a resposta em frequência medida por sweep/impulso coincide com a curva analítica `|H(jω)|` dentro de **`±0,5 dB`** entre `20 Hz` e `10 kHz` para todas as topologias (FMV Marshall, FMV Fender AB763, FMV Mesa, Vox Top Boost real e James/Baxandall real).
- **Interação passiva genuína:** mover `Treble` altera a profundidade e a frequência do vale de `Middle`/`Bass` conforme o circuito real; em `B = M = T = 0`, a saída cai para atenuação passiva profunda (no FMV clássico, quase silêncio).
- **Zero cliques sob automação:** atualização de coeficientes apenas quando algum parâmetro muda (`> 1e-4`), mantendo estabilidade de polos em TDF-II `f64` (ou suavização de coeficientes em varreduras rápidas).

## 9. Parâmetros e controles
| Nome | Chave / Campo | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Topologia | `amp_1.tonestack` | enum | id | `model, marshall, fender, vox, mesa_mark, baxandall` | `model` | discreto | Seleciona rede nodal e valores R/C | Matriz KCL específica da topologia |
| Bass | `bass` | number | scale | `0.0..=10.0` | `5.0` | por topologia | Posição do cursor de Bass | `b_pos ∈ [0, 1]` |
| Middle | `middle` | number | scale | `0.0..=10.0` | `5.0` | por topologia | Posição do cursor de Middle (ignorado no Vox TB) | `m_pos ∈ [0, 1]` |
| Treble | `treble` | number | scale | `0.0..=10.0` | `5.0` | por topologia | Posição do cursor de Treble | `t_pos ∈ [0, 1]` |

## 10. Contratos de integração
- Consumido pelos 5 amplificadores (Specs 054–058) e pela geração da curva de resposta da Bancada CRT (Spec 047).
- Exposto com construtor de componentes explícito (`FmvParams`, `VoxTopBoostParams`, `JamesParams`) para que cada amplificador forneça exatamente os valores de sua revisão sem alterar os demais!

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/tonestack.rs` (linhas 1–401).
- **Dívidas registradas:**
  1. `VoxTopBoost` (`tonestack.rs:79–93`) reutiliza a matriz FMV com `m_pos = 1.0` em vez da matriz nodal real da rede Vox Top Boost.
  2. `PassiveJamesBaxandall` (`tonestack.rs:108–121`) também reutiliza a matriz FMV em vez da rede James real.
  3. A impedância de fonte `R_src` (placa 38 kΩ no AB763 vs cathode follower ~1,3 kΩ no Marshall) atualmente é tratada como `0 Ω` (fonte ideal); incluir `r_src` na matriz KCL corrige o carregamento real de entrada.
- **Estrutura alvo:** `crates/ampforge-dsp/src/shared/tonestack.rs` com solvers nodais separados para `fmv_network`, `vox_top_boost_network` e `james_network`.

## 12. Estratégia DSP
- Determinantes polinomiais em `f64` sobre variáveis normalizadas `p = s·τ_ref` + transformada bilinear analítica exata → filtro 3ª ordem TDF-II em `f64` (operando preferencialmente na taxa oversampled do amp, eliminando quase totalmente o *frequency warping* em 10 kHz).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap (arrays fixos `[f64; 6]` na stack), zero locks, zero panic; recálculo de coeficientes protegido por guarda de delta (`1e-4`).

## 14. Gestão de estado
- `z: [f64; 3]` em `f64`, coeficientes `b: [f64; 4]`, `a: [f64; 4]` e cache dos últimos controles; `reset()` zera `z`.

## 15. Latência
- **`0` amostras** (filtro IIR de fase mínima).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 0,8%`** por instância @ 128 frames / 48 kHz (2× OS, `--release`).

## 17. Testes
- Comparação automatizada da magnitude DFT da resposta ao impulso contra `analog_magnitude_db()` em 30 frequências logarítmicas (`20 Hz–10 kHz`) para todas as 5 topologias e combinações extremas (`0/5/10`); verificação de estabilidade de polos (`|z| < 1`) em todo o cubo `[0..10]³`; teste de interação entre controles; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Exatidão Digital vs Analítica):** Para todas as topologias (Marshall FMV, Fender AB763, Mesa Rev F, Vox Top Boost dedicada e James/Baxandall dedicada), a resposta medida via impulso coincide com a referência analítica contínua dentro de `±0,5 dB` de `20 Hz` a `10 kHz` (em `sr >= 96 kHz` / 2× OS).
- **CA-2 (Estabilidade Global em Todo o Cubo de Controles):** Varredura exaustiva de `B, M, T ∈ {0, 2.5, 5, 7.5, 10}` em `44,1`, `48`, `88,2` e `96 kHz` não produz instabilidade, explosão numérica nem `NaN`/`Inf`.
- **CA-3 (Topologia Própria Vox Top Boost e James):** `VoxTopBoost` e `PassiveJamesBaxandall` possuem equacionamentos nodais próprios (distintos da matriz FMV) verificáveis contra curvas de referência.
- **CA-4 (Zero-Alloc e RT Safety):** `update()` e `process()` executam com `0` alocações e `0` desalocações.
- **CA-5 (Não-regressão):** Testes das Specs 031 CA-1 e 039 CA-5/CA-6 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Dependentes:** `Specs 054, 055, 056, 057, 058`.
- **Relacionadas / Legadas:** `Specs 003, 016, 031, 039, 047`.

## 20. Limitações conhecidas
- Em taxa base `44,1 kHz` sem oversampling (`1x`), a transformada bilinear apresenta compressão de frequência (*warping*) leve acima de `8 kHz` (resolvida automaticamente quando executada a `>= 2x` OS).

## 21. Riscos técnicos
- Introduzir `r_src = 38 kΩ` no AB763 reduz o ganho médio do tone stack em relação a `r_src = 0`; ajustar `tonestack_recovery_gain` de forma calibrada e verificar testes da Spec 039.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `003`, `016`, `031`, `039`, `047`.
- **Código atual:** `crates/ampforge-dsp/src/tonestack.rs`.
- **Testes existentes / futuros:** `contract_031_authentic_amps.rs`, `contract_039_amps_fidelity.rs`, futuro `contract_078_tonestack_networks.rs`.
