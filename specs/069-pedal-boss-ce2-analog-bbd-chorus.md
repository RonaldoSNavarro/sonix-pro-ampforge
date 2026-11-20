---
spec: 069
titulo: Pedal Boss CE-2 Analog BBD Chorus (Rev. 1979 MN3007/MN3101 e Variante Tri-Chorus)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053]
---
# 069 — Pedal Boss CE-2 Analog BBD Chorus (Rev. 1979 MN3007)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.boss.ce2_chorus` (variante: `rack.dytronics.tri_chorus`; chaves `mod_2.*`)
- **Revisão (`model_revision`):** `1979-mij-mn3007-mn3101`
- **Categoria:** Pedal de Modulação — Chorus BBD (módulo independente)
- **Objetivo:** Especificar integralmente o comportamento elétrico, filtragem pré/pós-ênfase de BBD (`MN3007`), modulação LFO triangular/senoidal e mistura seca/modulada do **Boss CE-2 Analog Chorus**, além da extensão de 3 vozes defasadas em `0° / 120° / 240°` (`StereoTriChorus`).

## 2. Escopo
- Linha de atraso BBD (`MN3007`, 1024 estágios, janela `~4,5–11,5 ms`), LFO (`TL022`, `0,05–10 Hz`), filtro anti-aliasing / reconstrução de relógio BBD (roll-off HF `~4,5 kHz` + leve pré/de-ênfase), mistura dry/wet (`50%` nominal no CE-2, ajustável `0–100%`) e modo `StereoTriChorus` (3 vozes em 0°, 120°, 240°).

## 3. Fora de escopo
- Ruído de clock heteródino audível (clock whine) em taxas de atraso extremas (indesejado em uso digital limpo).
- Saída estéreo L/R completa do Tri-Chorus (requer migração estéreo do barramento global, prevista na Spec 053).

## 4. Equipamento ou componente de referência
- **Fabricante:** Roland / Boss.
- **Modelo exato:** CE-2 Chorus (Japan Black/Silver Screw).
- **Período / Ano:** 1979–1982.
- **Papel na cadeia de sinal:** Chorus analógico pós-amplificador/gabinete ou em ramo paralelo (Linha B), conferindo largura e espessura sem perder definição de graves.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1979 Boss CE-2 (MN3007 1024-stage BBD + MN3101 clock + TL022 LFO)`.
- **Variantes:** `StereoTriChorus` (inspirado no Dytronics CS-5 Tri-Stereo Chorus: 3 linhas independentes em fase 0°/120°/240° reduzindo o efeito "enjooso" de uma única senoide).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | ElectroSmash — *Boss CE-2 Analysis* | Boss CE-2 (`MN3007`) | Pre-emphasis, MN3007 BBD, MN3101 Clock, TL022 LFO, Anti-aliasing Filters | https://www.electrosmash.com/boss-ce2-analysis | 2026-11-19 | `[fonte secundária]` | `MN3007` 1024 estágios, LFO `TL022`, filtragem anti-clock pós-BBD (~4,5 kHz), mistura dry+wet fixa 50/50 no original |
| F2 | Panasonic `MN3007` / `MN3101` Datasheets | BBD 1024-stage | Delay range vs clock (`10 kHz–100 kHz` → `5.12–51.2 ms`) | Datasheets Panasonic | 2026-11-19 | `[confirmado em documentação]` | Relação de atraso e largura de banda limitada do BBD |
| F3 | Spec 040 CA-1 | Contratos existentes | `fx/modulation.rs` | `specs/040-*.md` | 2026-11-19 | `[confirmado em documentação]` | Separação real entre BBD Chorus e Tri-Chorus |

## 7. Fundamentação elétrica
- **Pré-ênfase + BBD `MN3007` + De-ênfase/LPF:** o CE-2 aplica leve pré-ênfase de agudos antes do BBD para melhorar SNR, atrasa o sinal na faixa típica de `5–11 ms` modulada pelo LFO sobre o clock `MN3101`, e filtra o retorno com múltiplos polos passa-baixas (roll-off característico acima de `~4,5 kHz`), somando-o ao sinal seco `[fonte secundária: F1]`.
- **Impedâncias:** `Z_in ≈ 470 kΩ`, `Z_out ≈ 1 kΩ` (bufferizado).

## 8. Comportamento esperado
- **Modulação de pitch suave + roll-off BBD:** a voz molhada exibe desvio periódico de atraso (`8 ± 3,5 ms` em `Depth = 10`) e atenuação natural de agudos acima de `4,5 kHz` (`>= 3 dB` em `8 kHz` no wet puro).
- **Diferença mensurável CE-2 vs Tri-Chorus:** com `Mix = 100%`, o Tri-Chorus distribui energia em 3 bandas laterais defasadas em `120°`, apresentando envelope de modulação mais denso e distinto da voz única do CE-2.
- **Tempo Sync opcional:** suporta sincronização com `tempo_bpm` + `NoteDivision`.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `mod_2.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa/bypassa pedal | Crossfade sem cliques |
| Rate | `mod_2.rate_hz` | number | Hz | `0.05..=10.0` | `1.0` | log | Velocidade do LFO (`TL022`) | Incremento de fase `2π·f/fs` |
| Depth | `mod_2.depth` | number | scale | `0.0..=10.0` | `5.0` | linear | Excursão do clock BBD (`0..±3,5 ms`) | Amplitude de modulação da linha de atraso |
| Mix | `mod_2.mix` | number | % | `0.0..=100.0` | `50.0` | linear | Proporção dry/wet (`50%` = CE-2 original; `100%` = vibrato) | `dry*(1-m) + wet*m` |

## 10. Contratos de integração
- `module_id = "pedal.boss.ce2_chorus"`, `model_revision = "1979-mij-mn3007-mn3101"`.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/modulation.rs` (linhas 251–274).
- **Dívidas registradas:** acoplado em `ModulationProcessor` junto com outros 5 efeitos; usa interpolação linear na linha de atraso (`modulation.rs:170`), que introduz atenuação extra dependente da fase fracionária — evoluir para interpolação cúbica (Hermite) mantendo o LPF BBD explícito em `4,5 kHz`; separar `BossCe2Chorus` e `StereoTriChorus`.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/boss_ce2_chorus.rs`.

## 12. Estratégia DSP
- Linha de atraso circular pré-alocada (`>= 8192` amostras cobrindo até `96 kHz`) com interpolação cúbica (Hermite), LFO contínuo `f64`, LPF BBD `4,5 kHz` (`f64` TDF-II) no caminho wet (e 3 leituras defasadas em `0/120/240°` no Tri-Chorus).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; atrasos em `ms` escalados pela `sample_rate` (`44,1 / 48 / 88,2 / 96 kHz`).

## 14. Gestão de estado
- `delay_buffer`, `write_pos`, `lfo_phase`, `bbd_lp_state` isolados por instância; `reset()` zera estados em O(1).

## 15. Latência
- **`0` amostras** no caminho seco (`dry`).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 1,5%`** (CE-2) / **`<= 2,5%`** (Tri-Chorus) do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Medição espectral da voz wet (`Mix = 100%`, `Depth = 0`) confirmando roll-off BBD em `> 4,5 kHz`; senoide `1 kHz` com `Depth = 8` confirmando bandas laterais de modulação FM/PM; comparação CE-2 vs Tri-Chorus; transparência em `Mix = 0%`; invariância entre `44,1 / 48 / 96 kHz`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Roll-Off BBD CE-2):** Com `Mix = 100%` e `Depth = 0`, a resposta em `8 kHz` fica `>= 3 dB` abaixo de `500 Hz` (característica do LPF pós-MN3007).
- **CA-2 (Modulação de Atraso Real):** Com `Mix = 50%`, `Rate = 1.5 Hz`, `Depth = 8`, uma senoide `1 kHz` apresenta bandas laterais moduladas mensuráveis (diferença espectral real vs sinal seco).
- **CA-3 (Diferenciação Tri-Chorus):** Na mesma entrada e controles, `StereoTriChorus` produz resposta temporal/espectral mensuravelmente distinta de `AnalogBbdChorus`.
- **CA-4 (Invariância de Taxa e RT Safety):** Tempo base de atraso (`8 ms ± 0,2 ms`) e frequência do LFO coincidem em `44,1`, `48` e `96 kHz`; zero alocações/desalocações em `process()`.
- **CA-5 (Não-regressão):** Testes das Specs 007, 024 e 040 para Chorus continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Relacionadas / Legadas:** `Specs 007, 024, 040, 052`.

## 20. Limitações conhecidas
- Saída do Tri-Chorus atualmente somada em mono até a conclusão do barramento estéreo global (Spec 053).

## 21. Riscos técnicos
- Interpolação linear simples degrada agudos de forma variável com a fase; usar interpolação cúbica/Hermite na evolução modular.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `007`, `024`, `040`, `052`.
- **Código atual:** `crates/ampforge-dsp/src/fx/modulation.rs`.
- **Testes existentes / futuros:** `contract_040_mod_delay_reverb.rs`, futuro `contract_069_pedal_boss_ce2.rs`.
