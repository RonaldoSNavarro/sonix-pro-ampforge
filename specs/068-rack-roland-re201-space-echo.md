---
spec: 068
titulo: Unidade Roland RE-201 Space Echo (Rev. 1974 — 3 Cabeças, 12 Modos e Mola Interna)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053, 081]
---
# 068 — Unidade Roland RE-201 Space Echo (Rev. 1974)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `rack.roland.re201_space_echo` (chaves `dly_1.*`)
- **Revisão (`model_revision`):** `1974-re201-3head-spring`
- **Categoria:** Unidade de Tape Echo + Spring Reverb (módulo independente)
- **Objetivo:** Especificar integralmente a arquitetura eletromecânica e sonora do **Roland RE-201 Space Echo (1974)** — seletor autêntico de **12 modos** (combinações das 3 cabeças de reprodução + tanque de mola interno + modo 12 somente mola), **glide suave de motor de fita**, **wow/flutter**, **saturação de fita**, **EQ Bass/Treble restrito ao caminho molhado**, auto-oscilação controlada e spillover — desacoplado dos demais modelos de delay (Spec 081).

## 2. Escopo
- Sistema de fita magnética de 3 cabeças de leitura com razões temporais fixas (`H1 = 1.0×`, `H2 ≈ 1.92×`, `H3 ≈ 2.83×` do tempo base controlado por `Repeat Rate`).
- Seletor rotativo de **12 modos** conforme painel oficial RE-201 (modos 1–4 eco apenas, modos 5–11 combinações multi-head + spring reverb interno, modo 12 apenas spring reverb).
- `Intensity` (feedback com saturação de fita `tanh` permitindo auto-oscilação musical controlada sem explosão numérica), `Echo Volume`, `Reverb Volume`, `Bass` e `Treble` **exclusivos no caminho wet** (jamais alterando o sinal seco!), `Spillover` e `Tap Tempo`.

## 3. Fora de escopo
- Outros modelos de delay (`AnalogBbdDelay`, `PristineDigitalDelay`, `DualParallelDelay`, `ReverseDelay`, `PingPongStereo` — especificados na Spec 081).
- Pré-amplificador de entrada de microfone/instrumento com clipping de entrada do RE-201 em modo "Sound on Sound" (RE-501).

## 4. Equipamento ou componente de referência
- **Fabricante:** Roland Corporation.
- **Modelo exato:** RE-201 Space Echo.
- **Período / Ano:** 1974–1990.
- **Papel na cadeia de sinal:** Unidade de ambiência/eco de fita pós-amplificador/gabinete, oferecendo padrões rítmicos sincopados multi-head, degradação orgânica de fita e mola interna combinada.

## 5. Revisão e variantes
- **Revisão de referência selecionada:** `1974 Roland RE-201 Space Echo (3 cabeças de reprodução, 12 modos, tanque de mola integrado)`.
- **Variantes históricas:** RE-101 (sem mola), RE-301 / RE-501 Chorus Echo (adicionam chorus BBD e saída balanceada).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | *Roland RE-201 Space Echo Owner's & Service Manual* | RE-201 (1974) | Mode Selector 1–12, Tape Heads, Bass/Treble EQ, Spring Tank | Roland Service Manuals Archive | 2026-11-19 | `[confirmado em documentação]` | Matriz dos 12 modos (1–4 fita, 5–11 fita+mola, 12 só mola), 3 cabeças, EQ somente no eco, controle de velocidade do capstan |
| F2 | Specs 032 CA-5 e 040 CA-7/CA-12 | Contratos existentes | `fx/delay.rs` | `specs/032-*.md`, `specs/040-*.md` | 2026-11-19 | `[confirmado em documentação]` | 12 modos, EQ exclusivo no wet, glide suave sem degrau, spillover preservando cauda |

## 7. Fundamentação elétrica
- **Mecânica de fita (Capstan + 3 cabeças):** um laço contínuo de fita de `1/4"` passa por cabeça de apagamento, gravação e 3 cabeças de leitura espaçadas fisicamente ao longo da fita (`H1`, `H2`, `H3`). Alterar `Repeat Rate` varia a velocidade do motor DC com inércia mecânica (~30 ms de rampa — *glide* com variação de pitch transitória) `[confirmado em documentação: F1]`.
- **Tabela dos 12 Modos RE-201:**
  - `1`: H1 | `2`: H2 | `3`: H3 | `4`: H2 + H3 (somente eco).
  - `5`: H1 + Mola | `6`: H2 + Mola | `7`: H3 + Mola | `8`: H1 + H2 + Mola | `9`: H2 + H3 + Mola | `10`: H1 + H3 + Mola | `11`: H1 + H2 + H3 + Mola *(nota: confirmar mapeamento fino exato de cabeças dos modos 5–11 em `delay.rs:365–378` vs tabela oficial do manual RE-201 — ver dívida em §11!)*.
  - `12`: **Reverb Only** (cabeças de eco desconectadas; apenas tanque de mola) `[confirmado em documentação: F1]`.
- **EQ de eco (`Bass` e `Treble`):** filtros shelving aplicados **exclusivamente ao retorno das cabeças de fita** (e realimentados no loop de `Intensity`), nunca colorindo o sinal direto (`dry`) `[confirmado em documentação: F1]`.
- **Saturação de fita e Wow/Flutter:** compressão/saturação suave (`tanh`) e roll-off natural de agudos no laço de feedback + modulação lenta de velocidade (wow ~1,15 Hz + flutter leve).

## 8. Comportamento esperado
- **Multi-head rítmico:** em modo com múltiplas cabeças (ex.: modo `4` ou `8`), um impulso gera múltiplos ecos espaçados nas proporções físicas das cabeças (ex.: `1.0×`, `1.92×`, `2.83×`), ao contrário do modo `1` (eco único).
- **Isolamento do EQ no Wet:** mover `dly_1.bass` ou `dly_1.treble` com `echo_volume = 0` (ou durante o sinal direto) altera o sinal seco em `<= 0,05 dB`, enquanto altera o espectro das repetições em `>= 8 dB`.
- **Modo 12 (Reverb Only):** não gera ecos discretos de fita; produz apenas a cauda da mola interna comandada por `reverb_volume`.
- **Glide de fita sem estalo:** variar `repeat_rate` durante sinal contínuo desliza o tempo suavemente (`derivada <= 1,5×`, Spec 040 CA-7) com efeito Doppler característico de fita.
- **Spillover:** ao desligar `dly_1.enabled` com `spillover = true`, zera a gravação na fita mas deixa as repetições existentes decaírem naturalmente sobre o sinal direto.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `dly_1.enabled` | boolean | bool | `false/true` | `false` | discreto | Ativa/bypassa unidade | Respeita `spillover` |
| Mode Selector | `dly_1.mode` | enum | modo | `1..=12` | `4` | discreto | Combina cabeças H1/H2/H3 e mola interna | Matriz de pesos `(h1, h2, h3, spring_on, echo_on)` |
| Repeat Rate | `dly_1.repeat_rate` | number | scale | `0.0..=10.0` | `5.0` | log (velocidade motor) | Velocidade da fita / tempo entre ecos | Alvo de `delay_ms` com glide exponencial 30 ms |
| Intensity | `dly_1.intensity` | number | scale | `0.0..=10.0` | `5.0` | log | Feedback de fita (auto-oscilação > 8,5) | Ganho de loop + `tanh` de saturação de fita |
| Echo Volume | `dly_1.echo_volume` | number | scale | `0.0..=10.0` | `5.0` | audio log | Nível do retorno de eco misturado ao dry | Mix wet das cabeças |
| Reverb Volume | `dly_1.reverb_volume` | number | scale | `0.0..=10.0` | `2.5` | audio log | Nível da mola interna (modos 5–12) | Mix do tanque de mola interno |
| Bass (Wet) | `dly_1.bass` | number | scale | `0.0..=10.0` | `5.0` | linear (`±12 dB`) | Graves somente nas repetições | Low-shelf `220 Hz` `f64` TDF-II no wet |
| Treble (Wet) | `dly_1.treble` | number | scale | `0.0..=10.0` | `5.0` | linear (`±12 dB`) | Agudos somente nas repetições | High-shelf `2200 Hz` `f64` TDF-II no wet |
| Spillover | `dly_1.spillover` | boolean | bool | `false/true` | `true` | discreto | Preserva cauda ao bypassar/trocar cena | Zera escrita na fita mantendo leitura |

## 10. Contratos de integração
- `module_id = "rack.roland.re201_space_echo"`, `model_revision = "1974-re201-3head-spring"`.
- Sincroniza com `topbar.tap_tempo` e tempo de host DAW (`set_tempo_bpm`) convertendo BPM em `repeat_rate` sem saltos abruptos.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/delay.rs` (linhas 99–559).
- **Dívidas registradas:**
  1. **Mapeamento fino dos modos 5–11 (`delay.rs:365–378`):** conferir e alinhar exatamente cada modo `5..11` com a tabela impressa no painel frontal do RE-201 original (manual oficial: Modo 5=H1+Rev, 6=H2+Rev, 7=H3+Rev, 8=H1+H2+Rev, 9=H2+H3+Rev, 10=H1+H3+Rev, 11=H1+H2+H3+Rev — hoje `delay.rs:370` coloca H1+H2 no modo 5 e H1+H3 no modo 6!). **Corrigir na implementação da Spec 068 com atualização coordenada do teste da Spec 032 CA-5.**
  2. `delay.rs` acopla o RE-201 com outros 5 modelos de delay; extrair `RolandRe201SpaceEcho` para `pedals/re201_space_echo.rs` reutilizando as linhas de atraso fracionário e clear O(1) da Spec 081.
  3. Migrar `WetBiquad` para `f64` TDF-II e adicionar interpolação cúbica (Hermite) na leitura da fita durante glide/wow para reduzir perda de agudos por interpolação linear.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/re201_space_echo.rs` (`RolandRe201SpaceEcho`).

## 12. Estratégia DSP
- **Behavioral eletromecânico:** linha de atraso circular pré-alocada (até 4 s @ 96 kHz) com interpolação cúbica/linear, 3 cabeças de leitura em razões `1.0 / 1.92 / 2.83`, modulação wow+flutter, saturação `tanh` de fita no feedback, EQ `Bass`/`Treble` `f64` restrito ao retorno wet, tanque de mola de 4 combs escaladas por taxa e `clear_tail()` O(1) sem `memset`.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap (buffers de fita e mola pré-alocados em `prepare()`); `clear_tail()` estritamente O(1) por contador `samples_since_clear` (zero `memset` de ~3 MB no callback); zero locks, zero panic.

## 14. Gestão de estado
- Buffer de fita, estado de mola, `current_delay_samples`, `wow_phase`, `wet_bass_bq`/`wet_treble_bq` encapsulados na instância única (fora do par ativa/sombra para preservar spillover em troca de cena — Spec 049 CA-1).

## 15. Latência
- **`0` amostras** no caminho seco (`dry`).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 2,5%`** do deadline de 128 frames @ 48 kHz em build `--release` (modo 11 com 3 cabeças + mola ativa).

## 17. Testes
- Resposta ao impulso em todos os 12 modos verificando exatamente quais cabeças (H1/H2/H3) e mola respondem; verificação de que `Bass`/`Treble` não alteram o impulso seco (`<= 0,05 dB`) mas alteram os ecos (`>= 8 dB`); teste de glide sem degrau (`derivada <= 1,5×`); auto-oscilação limitada e finita com `Intensity = 10`; spillover em bypass; escalonamento de mola em `44,1 / 48 / 96 kHz`; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Matriz Oficial dos 12 Modos RE-201):** Um impulso unitário produz exatamente os taps de cabeças (H1/H2/H3) e presença/ausência da mola correspondentes à tabela oficial do manual RE-201 nos modos `1` a `12` (incluindo modo `12` exclusivamente mola sem eco discreto).
- **CA-2 (EQ Exclusivo no Caminho Molhado):** Variar `dly_1.bass` e `dly_1.treble` de `0` para `10` altera o espectro das repetições em `>= 8 dB` mantendo o sinal direto inalterado (`<= 0,05 dB`).
- **CA-3 (Glide Suave de Fita):** Mudar o tempo de eco de `300 ms` para `400 ms` com feedback `50%` não gera degrau (`derivada máxima <= 1,5×` a máxima anterior à mudança).
- **CA-4 (Auto-Oscilação Segura):** Com `Intensity = 10` por 5 segundos contínuos, o sinal entra em auto-oscilação saturada estável sem `NaN`/`Inf` e mantido dentro de `[-2.0, 2.0]` interno.
- **CA-5 (Spillover e O(1) Clear / RT Safety):** Bypass com `spillover = true` mantém cauda audível; `clear_tail()` e `process()` executam com `0` alocações e `0` desalocações.
- **CA-6 (Não-regressão):** Testes das Specs 024, 032 CA-5 e 040 CA-7/CA-12 continuam verdes (atualizando o teste de modo 5 caso corrigido para tabela oficial).

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 081` (primitivas de linha de atraso e spillover).
- **Relacionadas / Legadas:** `Specs 007, 024, 032, 040, 049`.

## 20. Limitações conhecidas
- Histerese magnética física do cabeçote e desgaste da emenda de fita (*tape splice*) não são simulados em nível eletromagnético.

## 21. Riscos técnicos
- Corrigir a tabela dos modos 5–11 para o manual oficial do RE-201 pode alterar testes existentes que esperavam dois taps no modo 5 (Spec 032 CA-5) — coordenar atualização do teste de contrato na mesma entrega.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `007`, `024`, `032`, `040`, `049`.
- **Código atual:** `crates/ampforge-dsp/src/fx/delay.rs`.
- **Testes existentes / futuros:** `contract_032_pedals_fx_cabs.rs`, `contract_040_mod_delay_reverb.rs`, futuro `contract_068_rack_re201_space_echo.rs`.
