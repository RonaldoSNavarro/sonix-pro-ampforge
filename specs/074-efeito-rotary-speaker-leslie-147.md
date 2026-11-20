---
spec: 074
titulo: Efeito Rotary Speaker (Ref. Leslie 147 — Doppler, AM Direcional e Dispersão de Corneta)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053]
---
# 074 — Efeito Rotary Speaker (Ref. Leslie 147)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.mod.rotary_leslie_147` (`ModModel::RotarySpeaker`, chaves `mod_2.*`)
- **Revisão (`model_revision`):** `v1-leslie147-doppler-am`
- **Categoria:** Efeito de Modulação Acústica — Rotary Speaker (módulo independente)
- **Objetivo:** Especificar o modelo físico/comportamental do **Rotary Speaker (referência Leslie 147)** — combinando **efeito Doppler (modulação de atraso curto)**, **modulação direcional de amplitude (AM em quadratura com o Doppler)** e **filtragem/dispersão passa-baixas da corneta** — desacoplado de `ModulationProcessor`.

## 2. Escopo
- Simulação de rotação da corneta de agudos (`horn`) e rotor de graves (`drum`) com crossover opcional (~800 Hz), desvio de pitch por Doppler (atraso modulado `~3 ± 1,5 ms`), AM direcional em quadratura (`cos` vs `sin`), dispersão HF dependente do ângulo (~3,2 kHz) e rampa de aceleração/frenagem suave ao trocar velocidade (`Slow / Fast`).

## 3. Fora de escopo
- Amplificador valvulado interno de 40 W do gabinete Leslie 147 (já coberto pelos módulos de amplificador do AMPFORGE).
- Estéreo microfonado verdadeiro com dois microfones a 90°/180° (previsto como evolução quando o barramento global migrar para estéreo completo na Spec 053).

## 4. Equipamento ou componente de referência
- **Fabricante:** Electro Music / Leslie.
- **Modelo exato:** Leslie 147 Rotary Speaker Cabinet.
- **Período / Ano:** 1963–1980.
- **Papel na cadeia de sinal:** Final de cadeia ou ramo paralelo, conferindo rotação espacial acústica (Doppler + AM + filtragem direcional).

## 5. Revisão e variantes
- **Revisão de referência:** `Leslie 147 (crossover ~800 Hz, corneta + rotor de graves, velocidades Chorale ~0,8 Hz / Tremolo ~6,5 Hz)`.

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | J. O. Smith et al. — *Doppler Simulation and the Leslie* (DAFx-02) | Leslie 147 | Equações de Doppler, quadratura AM/FM, crossover 800 Hz | Proc. DAFx-02 (CCRMA Stanford) | 2026-11-19 | `[confirmado em documentação]` | Relação de quadratura (90°) entre desvio Doppler máximo e intensidade AM direta, crossover ~800 Hz |
| F2 | Spec 040 | Contratos existentes | `fx/modulation.rs` | `specs/040-*.md` | 2026-11-19 | `[confirmado em documentação]` | Modulação de amplitude + Doppler + dispersão HF |

## 7. Fundamentação elétrica e acústica
- Quando a corneta aponta diretamente para o microfone, o ganho de amplitude (AM) e o brilho HF são máximos, enquanto a velocidade radial em direção ao microfone é zero (desvio Doppler = 0). Quando está a 90° aproximando-se ou afastando-se, o desvio Doppler é máximo (`±Δf`) — portanto **Doppler e AM estão em quadratura de 90° (`sin` vs `cos`)** `[confirmado em documentação: F1]`.
- Um crossover de `~800 Hz` separa o rotor de graves (inércia maior, rotação ligeiramente mais lenta) da corneta de agudos.

## 8. Comportamento esperado
- **Combinação simultânea Doppler + AM em quadratura:** apresenta simultaneamente modulação de envelope (AM) e bandas laterais FM/Doppler, diferenciando-se tanto do Tremolo puro (sem Doppler) quanto do Chorus (sem AM direcional e dispersão).
- **Dispersão HF:** atenuação controlada acima de `~3,2 kHz` reproduzindo a diretividade da corneta.
- **Rampa inercial suave:** alterar `rate_hz` acelera/desacelera a fase sem saltos de pitch abruptos.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no equipamento real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `mod_2.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa/bypassa efeito | Crossfade sem cliques |
| Speed (Rate) | `mod_2.rate_hz` | number | Hz | `0.05..=10.0` | `1.5` | log | Velocidade de rotação (Chorale ~0,8 Hz → Tremolo ~6,5 Hz) | Fase do rotor com inércia suavizada |
| Depth | `mod_2.depth` | number | scale | `0.0..=10.0` | `7.0` | linear | Intensidade Doppler + AM direcional | Excursão `±1,5 ms` e AM `35%` |
| Mix | `mod_2.mix` | number | % | `0.0..=100.0` | `100.0` | linear | Mistura dry/rotary (`100%` = gabinete Leslie real) | `dry*(1-m) + wet*m` |

## 10. Contratos de integração
- `module_id = "pedal.mod.rotary_leslie_147"`, `model_revision = "v1-leslie147-doppler-am"`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/modulation.rs` (linhas 275–286).
- **Dívidas registradas:** acoplado em `ModulationProcessor`; modelo atual usa banda única (`horn`) sem crossover 800 Hz de separação entre rotor de graves (`drum`) e corneta (`horn`), e usa interpolação linear. Evoluir para banda dupla `800 Hz` (LR2) + interpolação cúbica na implementação da Spec 074.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/rotary_speaker.rs` (`RotaryLeslie147`).

## 12. Estratégia DSP
- Crossover 2ª ordem (`~800 Hz`, `f64`) dividindo `drum` e `horn`, linhas de atraso curtas pré-alocadas com interpolação cúbica para Doppler, AM direcional em quadratura (`cos`) + LPF direcional dependente do ângulo no ramo `horn`, e rampa de aceleração de velocidade.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; atrasos e filtros invariantes a `44,1 / 48 / 88,2 / 96 kHz`.

## 14. Gestão de estado
- Linhas de atraso, fases `horn`/`drum`, crossover e filtros de dispersão encapsulados em `RotaryLeslie147`.

## 15. Latência
- **`0` amostras** (ou atraso nominal da linha curta `~3 ms` apenas no wet).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 2,0%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Senoide `1 kHz` comprovando presença simultânea de modulação AM no envelope e bandas laterais Doppler (diferente do Tremolo); medição de roll-off HF > 3,2 kHz; transição suave de `rate_hz` (`0.8 → 6.5 Hz`); invariância de taxa; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Doppler + AM Simultâneos):** Com senoide `1 kHz`, `Rate = 4 Hz`, `Depth = 8`, `Mix = 100%`, a saída exibe modulação de amplitude mensurável (`>= 2 dB` pico-a-vale) **e** energia de bandas laterais Doppler fora do padrão AM puro.
- **CA-2 (Dispersão de Corneta):** A resposta acima de `5 kHz` apresenta atenuação média `>= 3 dB` relativa a `1 kHz` (filtragem direcional da corneta).
- **CA-3 (Invariância de Taxa e RT Safety):** Estável e consistente em `44,1 / 48 / 96 kHz`; zero alocações/desalocações em `process()`.
- **CA-4 (Não-regressão):** Contratos da Spec 040 para `RotarySpeaker` continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Relacionadas / Legadas:** `Specs 007, 040, 052`.

## 20. Limitações conhecidas
- Saída mono até a habilitação do barramento estéreo global (Spec 053), quando os dois microfones virtuais de corneta gerarão imagem estéreo ampla.

## 21. Riscos técnicos
- Aceleração rápida do rotor deve suavizar `rate_hz` (não saltar `lfo_phase`) para evitar cliques de fase.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `007`, `040`, `052`.
- **Código atual:** `crates/ampforge-dsp/src/fx/modulation.rs`.
- **Testes existentes / futuros:** `contract_040_mod_delay_reverb.rs`, futuro `contract_074_rotary_speaker.rs`.
