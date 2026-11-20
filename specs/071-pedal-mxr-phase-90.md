---
spec: 071
titulo: Pedal MXR Phase 90 (Rev. 1974 Script Logo — 4 Estágios All-Pass JFET)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053]
---
# 071 — Pedal MXR Phase 90 (Rev. 1974 Script Logo)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.mxr.phase_90` (`ModModel::Orange90Phaser`, chaves `mod_2.*`)
- **Revisão (`model_revision`):** `1974-script-4stage-jfet`
- **Categoria:** Pedal de Modulação — Phaser (módulo independente)
- **Objetivo:** Especificar integralmente o circuito do **MXR Phase 90 (Script Logo 1974)** — cascata de **4 filtros all-pass ativos de 1ª ordem** modulados por JFETs (`2N5952`) produzindo exatamente `N/2 = 2` entalhes (*notches*) profundos ao somar com o sinal direto, com realimentação controlada (`R28`) e LFO triangular/senoidal.

## 2. Escopo
- 4 estágios all-pass de 1ª ordem idênticos em cascata (`f_c` varrendo `~350 Hz` a `~3,2 kHz`), soma 50/50 com sinal seco gerando exatamente **2 notches móveis**, realimentação ajustável (Script moderado vs Block Logo mais ressonante) e controle `Speed` (`Rate`).

## 3. Fora de escopo
- Phasers de 6, 8 ou 12 estágios (ex.: Small Stone / Phase 100 — candidatos a specs futuras se adicionados ao catálogo).

## 4. Equipamento ou componente de referência
- **Fabricante:** MXR Innovations (Dunlop).
- **Modelo exato:** Phase 90 ("Script Logo").
- **Período / Ano:** 1974–1977.
- **Papel na cadeia de sinal:** Frequentemente usado **antes** do pré-amplificador (ex.: Van Halen na entrada de um Marshall Plexi) ou no loop para varredura suave de fase sem alterar o tempo do sinal.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1974 MXR Phase 90 Script Logo (4× all-pass 1ª ordem, 4× JFET 2N5952 casados, feedback suave)`.
- **Variante:** *Block Logo (1977+, resistor de feedback `R28` mais pronunciado)* — reproduzível elevando `feedback`.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | ElectroSmash — *MXR Phase 90 Analysis* | MXR Phase 90 (1974 Script) | 4-Stage All-Pass Cascade, JFET 2N5952, LFO, R28 Feedback | https://www.electrosmash.com/mxr-phase90 | 2026-11-19 | `[fonte secundária]` | 4 estágios all-pass de 1ª ordem com JFETs casados, LFO triangular, `R28` controlando profundidade/ressonância |
| F2 | J. O. Smith — *Physical Audio Signal Processing / Introduction to Digital Filters* | All-pass 1ª ordem | Relação `a = (tan(πf/fs) - 1) / (tan(πf/fs) + 1)` e `N/2` notches | CCRMA Stanford | 2026-11-19 | `[confirmado em documentação]` | Equação bilinear exata de all-pass de 1ª ordem |
| F3 | Spec 040 CA-4 | Contratos existentes | `fx/modulation.rs` | `specs/040-*.md` | 2026-11-19 | `[confirmado em documentação]` | 4 estágios produzindo exatamente `N/2 = 2` entalhes mensuráveis com LFO congelado |

## 7. Fundamentação elétrica
- **4 estágios all-pass idênticos:** cada estágio impõe deslocamento de fase de `0°` (DC) a `-180°` (∞), passando por `-90°` em `f_c = 1 / (2π · R_jfet · C_fix)` (`C_fix = 47 nF`) `[fonte secundária: F1]`.
- **2 Entalhes (`N/2 = 2`):** nos pontos onde a fase acumulada dos 4 estágios cruza `-180°` e `-540°`, a soma com o sinal direto (`50/50`) sofre cancelamento destrutivo profundo `[confirmado em documentação: F2, F3]`.
- **Feedback (`R28`):** realimenta fração da saída do 4º estágio para a entrada do 1º, acentuando picos entre os notches (Block Logo = mais feedback; Script = feedback suave).

## 8. Comportamento esperado
- **Exatamente 2 entalhes móveis:** com LFO congelado e ruído branco (ou sweep), a resposta em frequência apresenta **exatamente 2 vales profundos** (`>= 12 dB` de profundidade com `Mix = 50%`), deslocando-se continuamente entre `~200 Hz` e `~5 kHz` conforme o LFO varre.
- **Ausência de linha de atraso:** zero efeito Doppler de delay; alteração estritamente de fase.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `mod_2.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa/bypassa pedal | Crossfade sem cliques |
| Speed (Rate) | `mod_2.rate_hz` | number | Hz | `0.05..=10.0` | `0.8` | log | Velocidade do LFO triangular/senoidal | Incremento LFO |
| Depth | `mod_2.depth` | number | scale | `0.0..=10.0` | `7.0` | linear | Excursão dos JFETs (`350 Hz..3200 Hz`) | Amplitude de varredura log de `f_c` |
| Feedback (R28 Mod) | `mod_2.feedback` | number | ganho | `0.0..=0.90` | `0.35` | linear | Script (baixo) vs Block Logo (alto) | Realimentação do 4º estágio |
| Mix | `mod_2.mix` | number | % | `0.0..=100.0` | `50.0` | linear | Mistura dry/wet (`50%` = Phase 90 autêntico) | `dry*(1-m) + wet*m` |

## 10. Contratos de integração
- `module_id = "pedal.mxr.phase_90"`, `model_revision = "1974-script-4stage-jfet"`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/modulation.rs` (linhas 173–186, 232–240).
- **Dívidas registradas:** acoplado em `ModulationProcessor`; all-pass em `f32` DF-I com `tan()` calculado por amostra para os 4 estágios idênticos (calcular `a` uma única vez por amostra em `f64` TDF-II ou TPT all-pass reduz CPU em 4× e elimina ruído numérico em frequências baixas!).
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/mxr_phase90.rs` (`MxrPhase90`).

## 12. Estratégia DSP
- **Circuit-level (TPT / Bilinear `f64`):** 4 estágios all-pass de 1ª ordem em `f64` com coeficiente compartilhado `a(f_c)` + feedback de 1 amostra de atraso (ou solução fechada sem atraso de laço no TPT) + LFO triangular suavizado (como no op-amp original).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; `f_c` sempre clamped em `[20 Hz, 0.45 * sr]` para estabilidade em `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- `[f64; 4]` estados dos all-pass e `lfo_phase` isolados em `MxrPhase90`.

## 15. Latência
- **`0` amostras**.

## 16. Consumo de CPU
- Orçamento alvo: **`<= 1,0%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Sweep/ruído branco com LFO congelado contando exatamente `2` notches profundos (`>= 12 dB`); verificação de movimento dos notches em 2 fases distintas do LFO; estabilidade em `feedback = 0.90`; invariância de taxa (`44,1 / 48 / 96 kHz`); zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (2 Notches Reais de 4 Estágios):** Com LFO congelado, `Mix = 50%`, `Depth = 5`, o espectro apresenta exatamente `2` entalhes (`N/2 = 2`) com profundidade `>= 12 dB`.
- **CA-2 (Varredura Contínua de Fase):** Alterar a fase do LFO desloca as frequências centrais dos 2 notches de forma monotônica sem gerar cliques ou instabilidade.
- **CA-3 (Invariância de Taxa e RT Safety):** Frequências dos notches coincidem em `±2%` entre `44,1`, `48` e `96 kHz`; zero alocações/desalocações em `process()`.
- **CA-4 (Não-regressão):** Teste `CA-4` da Spec 040 continua verde.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Relacionadas / Legadas:** `Specs 007, 040, 052`.

## 20. Limitações conhecidas
- Leve distorção assimétrica dos JFETs `2N5952` sob sinal muito quente (> 0 dBFS) não é modelada na versão linear-variante base.

## 21. Riscos técnicos
- Feedback próximo a `1.0` com atraso unitário pode provocar pico ressonante elevado; limitar `feedback <= 0.90` com clamp seguro.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `007`, `040`, `052`.
- **Código atual:** `crates/ampforge-dsp/src/fx/modulation.rs`.
- **Testes existentes / futuros:** `contract_040_mod_delay_reverb.rs`, futuro `contract_071_pedal_mxr_phase90.rs`.
