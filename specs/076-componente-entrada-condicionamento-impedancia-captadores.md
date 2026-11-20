---
spec: 076
titulo: Componente de Entrada, Condicionamento de Sinal, Calibração, DI/Reamp e Impedância de Captadores
status: rascunho
tipo: componente-compartilhado
atualizado: 2026-11-19
dependencias: [053]
---
# 076 — Componente de Entrada, Condicionamento, Calibração e Impedância de Captadores

## 1. Identificação e objetivo
- **Identificador estável (`module_id`):** `component.input.conditioning_and_impedance`
- **Revisão (`model_revision`):** `v1-dc64-calib-hiz-pickup`
- **Categoria:** Componente compartilhado de front-end DSP
- **Objetivo:** Especificar o módulo reutilizável de entrada da cadeia — seleção de fonte (`device`, `test_riff`, `di_file`), contagem de clipping ADC, modo nominal `Instrument / Line`, trim de ganho (`input.boost`), **DC blocker de precisão `f64`**, alimentação do ring buffer limpo (Afinador/Sidechain DI), calibração/diagnóstico (`calibration.rs`), alinhamento de Reamp (`reamp.rs`) e **contrato de modelagem opcional de impedância captador↔cabo↔primeiro pedal**.

## 2. Escopo
- Seleção de fonte de entrada sem alocação (`device`, `test_riff` Karplus-Strong calibrado a `-12 dBFS`, `di_file` WAV pré-carregado).
- Detecção lock-free de clipping de conversor (`|x| >= 0.999` → contador atômico `adc_clips`).
- Modo `instrument` (`-10 dBV` nominal) vs `line` (`+4 dBu` nominal, aviso honesto de que impedância física Hi-Z depende do hardware).
- DC blocker `f64` de 1 polo (`r = 0.99999` escalado por taxa → `f_c ≈ 0,08 Hz`, plano `±0,0001 dB` @ 40 Hz).
- Publicação do sinal limpo pré-efeitos em `CleanInputRing` (para Afinador Spec 083) e slice de sidechain DI limpa (para Noise Gate Natural Spec 065).
- Assistente de calibração (medição de piso de ruído, detecção de hum 50/60 Hz, alvo `-12 dBFS`, sugestão `boost` + `gate.threshold_db`) e utilitários de Reamp loopback (`reamp.rs`).
- Contrato opcional e explícito de interação **captador passivo (RLC) + capacitância de cabo ↔ impedância de entrada do primeiro módulo** (ex.: Fuzz Face Ge `8 kΩ` vs TS808 `510 kΩ`, sem reaplicar automaticamente sobre DIs que já trazem carga física!).

## 3. Fora de escopo
- Drivers WASAPI/ASIO de captura física (camada host `sonix-desktop`, Specs 018/036/049).
- Transformar eletricamente uma entrada de linha em Hi-Z por software (impossível fisicamente; sempre exibir orientação honesta ao usuário).

## 4. Equipamento ou componente de referência
- **Referência técnica:** Front-end analógico de guitarra passiva (captador magnético `R_dc + L + C_p` + potenciômetros `250k/500k` + cabo `~300–600 pF`) e padrão de ganho digital `-12 dBFS` de headroom de entrada.

## 5. Revisão e variantes
- **Revisão:** `v1-dc64-calib-hiz-pickup`.
- **Variantes de fonte:** `device` (captura ao vivo), `test_riff` (Karplus-Strong determinístico), `di_file` (arquivo WAV offline carregado fora do RT).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | `crates/sonix-desktop/src/calibration.rs` e `reamp.rs` | Workspace v1.0 | `calibrate_input`, `generate_karplus_strong_riff`, `measure_loopback_delay_samples` | `crates/sonix-desktop/src/` | 2026-11-19 | `[confirmado em documentação]` | Alvo `-12 dBFS` (`0.2512`), detecção de hum 50/60 Hz, alinhamento loopback `<= 1` amostra |
| F2 | Specs 019, 029, 049 CA-38/CA-39 | Contratos existentes | Calibração, modo Instrument/Line e Reamp | `specs/019-*.md`, `specs/049-*.md` | 2026-11-19 | `[confirmado em documentação]` | Contratos verificados de entrada e reamp |
| F3 | Helmuth Lemme — *The Secrets of Electric Guitar Pickups* | Captadores magnéticos | Circuito equivalente RLC (`L=2–8 H`, `R=5–15 kΩ`, `C_cabo=300–600 pF`) | Publicação técnica de engenharia | 2026-11-19 | `[fonte secundária]` | Ressonância de captador (2–5 kHz) amortecida por baixa impedância de carga (`Z_in`) |

## 7. Fundamentação elétrica
- **Modelo RLC equivalente de captador + cabo + carga (`Z_in`):** um captador passivo equivale a fonte de tensão em série com resistência `R_s` (`6–14 kΩ`) e indutância `L_s` (`2,5–7 H`), em paralelo com capacitância total `C_t` (`C_bobina + C_cabo ≈ 400–700 pF`) e a carga resistiva `R_load = R_pot || Z_in_pedal`. Quando `Z_in_pedal` cai de `1 MΩ` para `8 kΩ` (ex.: Fuzz Face), o fator de qualidade `Q` do pico ressonante cai drasticamente, suavizando agudos e reduzindo nível `[fonte secundária: F3]`.
- **DC Blocker `f64`:** `y[n] = x[n] - x[n-1] + r · y[n-1]` com `r = exp(-2π · f_c / fs)` (`f_c = 0,08 Hz`), eliminando offset DC de interfaces onboard sem alterar graves de guitarra (E2 = `82,4 Hz`).

## 8. Comportamento esperado
- **DC Blocker invariante à taxa:** resíduo DC em regime estacionário `< -80 dBFS` e atenuação em `40 Hz <= 0,01 dB` em `44,1 / 48 / 88,2 / 96 kHz`.
- **Calibração precisa:** com silêncio + palhetada forte, propõe `boost` levando pico para `-12 ± 1 dBFS`, `gate.threshold_db = piso_ruído_pós_boost + 6 dB`, e classifica `InputTooHot`, `InputTooLow`, `Hum50Hz`, `Hum60Hz` ou `Ok`.
- **Interação opcional de impedância (desligada por padrão em DI gravada):** quando ativada (`pickup_load_enabled = true`), lê `input_impedance_ohms` do 1º bloco ativo no grafo e aplica o filtro RLC de carga correspondente sem acoplamento direto a nenhum pedal específico.

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Fonte | `input.source` | enum | id | `device, test_riff, di_file` | `device` | discreto | Seleciona entrada ativa | Troca de buffer pré-alocado |
| Modo | `input.mode` | enum | id | `instrument, line` | `instrument` | discreto | Ganho nominal `-10 dBV` vs `+4 dBu` | Escala nominal pré-boost |
| Ganho Entrada | `input.boost` | number | x | `0.1..=8.0` | `1.0` | log | Escala nível antes da cadeia | Multiplicador suavizado |
| Simulação Carga Captador | `input.pickup_load` | enum | id | `off, single_coil, humbucker` | `off` | discreto | Emula carga RLC sobre `Z_in` do 1º bloco (desligada por padrão para não colorir DI pronta) | Biquad `f64` parametrizado por `Z_in` |

## 10. Contratos de integração
- `module_id = "component.input.conditioning_and_impedance"`.
- Fornece `clean_input_slice` no `ProcessContext` para o Noise Gate Natural (Spec 065) e escreve no `CleanInputRing` para o Afinador (Spec 083).

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/sonix-desktop/src/audio.rs` (linhas 4478–4522), `crates/sonix-desktop/src/calibration.rs`, `crates/sonix-desktop/src/reamp.rs`.
- **Dívidas registradas:**
  1. `r = 0.99999_f64` em `audio.rs:4507` está fixo por amostra em vez de calculado a partir de `sample_rate`.
  2. `input_boost` não passa por `OnePoleSmoother` atualmente (`audio.rs:4503`), podendo causar leve degrau se automatizado rapidamente.
  3. Falta o módulo opcional `PickupImpedanceLoad` em `ampforge-dsp` para habilitar a interação real de impedância com o Fuzz Face (Spec 063).
- **Estrutura alvo:** `crates/ampforge-dsp/src/shared/input_stage.rs` (`InputConditioningStage` + `PickupImpedanceModel`).

## 12. Estratégia DSP
- DC blocker `f64` invariante à taxa + `OnePoleSmoother` no ganho + contador atômico de clipping ADC + filtro RLC opcional de 2ª ordem `f64` (TDF-II) derivado de `(L_s, R_s, C_t, Z_in_first_block)`.

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; carga de `di_file` WAV ocorre fora da thread RT e transfere via `SpscOwnedQueue` + `RetiredReaper`.

## 14. Gestão de estado
- `dc_x1`, `dc_y1`, `test_riff_pos`, `di_pos`, `pickup_filter_state` encapsulados na instância; `reset()` limpa estados sem realocar buffers.

## 15. Latência
- **`0` amostras**.

## 16. Consumo de CPU
- Orçamento alvo: **`<= 0,5%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Teste de rejeição DC (`< -80 dBFS`) e planicidade em `40 Hz` (`±0,01 dB`) em `44,1 / 48 / 96 kHz`; teste de contagem exata de `adc_clips`; validação de `calibrate_input` nos 5 diagnósticos; teste de `PickupImpedanceModel` comparando carga `1 MΩ` vs `8 kΩ`; alinhamento reamp (`erro <= 1` amostra); zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (DC Blocker Invariante à Taxa):** Entrada com offset DC `+0,5` converge para `< -80 dBFS` de resíduo e atenua `40 Hz` em `<= 0,05 dB` nas taxas `44,1`, `48` e `96 kHz`.
- **CA-2 (Detecção de Clipping ADC e Fontes):** Amostras com `|x| >= 0.999` incrementam `adc_clips` atomicamente; `test_riff` gera pico em `-12 ± 0,5 dBFS`.
- **CA-3 (Carga de Impedância Captador↔Pedal):** Com `input.pickup_load = single_coil`, conectar primeiro bloco com `Z_in = 8 kΩ` (Fuzz Face) reduz o pico ressonante em `2–5 kHz` em `>= 4 dB` comparado a `Z_in = 1 MΩ`; com `off`, resposta é plana `±0,05 dB`.
- **CA-4 (Erro e RT Safety):** Entradas `NaN`/`Inf` sanitizadas; zero alocações/desalocações em `process()`.
- **CA-5 (Não-regressão):** Testes das Specs 019, 029 e 049 CA-38/CA-39 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053`.
- **Dependentes:** `Spec 063` (Fuzz Face Ge), `Spec 065` (Noise Gate Natural), `Spec 083` (Afinador YIN).
- **Relacionadas / Legadas:** `Specs 019, 029, 049`.

## 20. Limitações conhecidas
- A impedância física Hi-Z da interface de áudio real não pode ser alterada por software; o modelo `input.pickup_load` atua apenas como modelagem adicional quando desejado pelo usuário.

## 21. Riscos técnicos
- Ativar `pickup_load` por padrão sobre DIs gravadas em interfaces reais coloriria duplamente o sinal; manter `off` como padrão obrigatório.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `019`, `029`, `049`.
- **Código atual:** `crates/sonix-desktop/src/{audio.rs, calibration.rs, reamp.rs}`.
- **Testes existentes / futuros:** `contract_019_input_calibration.rs`, `contract_049_asio_input_reamp.rs`, futuro `contract_076_input_stage.rs`.
