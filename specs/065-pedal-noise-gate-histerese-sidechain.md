---
spec: 065
titulo: Pedal Noise Gate com Histerese, Hold e Sidechain LR4 (Ref. Boss NS-2 / ISP Decimator)
status: rascunho
tipo: pedal
atualizado: 2026-11-19
dependencias: [053, 076]
---
# 065 — Pedal Noise Gate com Histerese e Sidechain (Ref. Boss NS-2 / ISP Decimator)

## 1. Identificação e objetivo
- **Identificador estável (`model_id`):** `pedal.dynamics.noise_gate_ns2` (chaves `gate.*`)
- **Revisão (`model_revision`):** `v1-hysteresis-lr4-di-key`
- **Categoria:** Pedal de Dinâmica / Redução de Ruído (módulo independente)
- **Objetivo:** Especificar o contrato modular, o detector True-RMS com filtragem sidechain LR4, a máquina de estados `Open → Hold → Closed` com histerese anti-chatter, o acionamento por DI limpa (método 4 cabos / Boss NS-2 / ISP Decimator G-String) e o modo Expander proporcional do **Noise Gate** do AMPFORGE.

## 2. Escopo
- Detector True-RMS com filtro de sidechain passa-banda Linkwitz-Riley 4ª ordem (HPF 20–500 Hz + LPF 1–12 kHz — atua **somente** no detector, nunca no caminho de áudio).
- Modos `natural` (Intelligent Natural Gate alimentado por DI limpa — equivalente ao loop de detecção 4-cable do NS-2/Decimator G-String) e `classic` (Expander/Gate proporcional com `ratio`).
- Controles `threshold_db`, `hysteresis_db`, `attack_ms`, `hold_ms`, `release_ms` (DECAY), `range_db`, `ratio`, `key_hpf_hz`, `key_lpf_hz` e telemetria `gain_reduction_db`.

## 3. Fora de escopo
- Redução espectral de ruído (FFT denoising) ou subtração de hum adaptativa em banda estreita.
- Detecção de piso de ruído do assistente de calibração (coberta pela Spec 076 / `calibration.rs`).

## 4. Equipamento ou componente de referência
- **Fabricante / Referência:** Inspiração funcional combinada no **Boss NS-2 Noise Suppressor** (detecção por entrada limpa / 4-cable loop, controle `Threshold`/`Decay`) e **ISP Decimator II G-String** (rastreamento suave e rejeição de hum/hiss no sidechain).
- **Papel na cadeia de sinal:** Posicionado no início da cadeia (ou após drives em modo Natural com chaveamento pela DI limpa de entrada) para silenciar hum de single-coils e hiss de alto ganho sem decepar transientes de palhetada nem trepidar no decaimento.

## 5. Revisão e variantes
- **Revisão de referência:** `v1-hysteresis-lr4-di-key` (consolidando Specs 030 e 041 CA-3/CA-4).
- **Variantes:** Modo `natural` (gate rígido controlado por DI limpa até `range_db`) e `classic` (expander contínuo abaixo do limiar com `ratio 1.2..100`).

## 6. Fontes técnicas
| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | *Boss NS-2 Owner's Manual* | Boss NS-2 | Método de conexão 4-cabos (Send/Return key detection) | Roland/Boss Manuals | 2026-11-19 | `[confirmado em documentação]` | Conceito de detectar o envelope na entrada limpa da guitarra enquanto atenua pós-ruído |
| F2 | *ISP Technologies Decimator II G-String Manual* | Decimator II G-String | Tracking e detecção de entrada direta | ISP Technologies | 2026-11-19 | `[confirmado em documentação]` | Tracking linear/exponencial sem chatter no sustain |
| F3 | U. Zölzer — *DAFX: Digital Audio Effects* (2ª ed.) | Cap. 4 (Dynamic Range Control) | Noise Gates, Hysteresis, RMS Detectors | Wiley, 2011 | 2026-11-19 | `[confirmado em documentação]` | Equações de detector RMS, histerese de abertura/fechamento, hold e curva de expander |
| F4 | Specs 030 e 041 CA-3/CA-4 | Contratos existentes | `fx/dynamics.rs` | `specs/030-*.md`, `specs/041-*.md` | 2026-11-19 | `[confirmado em documentação]` | LR4 sidechain `-24 dB/oit`, modo expander e chaveamento por DI |

## 7. Fundamentação elétrica
- **Sidechain independente (zero coloração do áudio):** o sinal de controle (`raw_key`) passa por 2 biquads Butterworth HPF em cascata (`Q = 1/√2`, Linkwitz-Riley 4ª ordem = `-24 dB/oitava`) e 2 biquads LPF em cascata (`-24 dB/oitava`) antes da detecção de potência quadrática (`key_bp²`) suavizada por duplo LPF criticamente amortecido (`Q = 0.5`, `fc = 220 Hz`) para estimação True-RMS sem ripple nem overshoot `[confirmado em documentação: F3, F4]`.
- **Histerese (`open_thresh` vs `close_thresh`):** abre quando `env >= threshold_db`; inicia `Hold` e só fecha quando `env < threshold_db - hysteresis_db`, eliminando abertura/fechamento intermitente (*chatter*) na cauda de notas sustentadas.
- **Curva de ganho:** em `natural`, rampa exponencial entre `0 dB` e `range_db`; em `classic`, lei de expansão `GR_dB = clamp(-(threshold_db - env_db) * (ratio - 1), range_db, 0)`.

## 8. Comportamento esperado
- **Imunidade a hum 50/60 Hz e hiss:** com `key_hpf_hz = 100 Hz`, hum de `60 Hz` abaixo do corte é atenuado em `>= 12 dB` no detector sem alterar a resposta em frequência do sinal de áudio principal (resposta plana `±0,05 dB` de 20 Hz a 20 kHz quando aberto!).
- **Zero chatter:** sinal oscilando ±1,5 dB em torno de `threshold_db` com `hysteresis_db = 4 dB` mantém estado estável.
- **Detecção por DI no modo Natural:** com drive ligado, se a DI limpa está `-3 dB` abaixo do limiar mas a saída do drive está acima, o modo `natural` fecha o gate enquanto `classic` permanece aberto (Spec 041 CA-4).
- **Expansão exata no modo Classic:** com `ratio = 4` e `threshold = -40 dB`, sinal estável a `-50 dB` sai em `-80 ± 2 dB` e a `-45 dB` sai em `-60 ± 2 dB` (Spec 041 CA-3).

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito / algoritmo | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| Bypass | `gate.enabled` | boolean | bool | `false/true` | `true` | discreto | Ativa/bypassa gate | Bypass limpo |
| Mode | `gate.mode` | enum | id | `natural, classic` | `natural` | discreto | Natural = DI key + gate; Classic = entrada local + expander | Chave de fonte sidechain e lei de ganho |
| Threshold | `gate.threshold_db` | number | dB | `-90.0..=0.0` | `-65.0` | linear em dB | Limiar de abertura | `10^(db/20)` |
| Decay / Release | `gate.release_ms` | number | ms | `5.0..=1000.0` | `80.0` | log | Tempo de fechamento após hold | Constante exponencial `release/4.6` |
| Hysteresis | `gate.hysteresis_db` | number | dB | `0.0..=12.0` | `4.0` | linear | Margem abaixo do threshold para fechar | `close_thresh = thresh - hyst` |
| Attack | `gate.attack_ms` | number | ms | `0.1..=50.0` | `1.0` | log | Tempo de abertura no transiente | Coeficiente exponencial |
| Hold | `gate.hold_ms` | number | ms | `0.0..=500.0` | `35.0` | linear | Tempo mínimo aberto antes do release | Contador de amostras `hold_samples` |
| Range | `gate.range_db` | number | dB | `-90.0..=0.0` | `-70.0` | linear em dB | Atenuação máxima fechado | Piso de ganho `range_lin` |
| Ratio | `gate.ratio` | number | :1 | `1.2..=100.0` | `10.0` | log | Taxa de expansão no modo Classic | Curva de expansão abaixo do limiar |
| Sidechain HPF | `gate.key_hpf_hz` | number | Hz | `20.0..=500.0` | `80.0` | log | Corte de graves/hum no detector LR4 | 2× biquad HPF `Q=0.7071` |
| Sidechain LPF | `gate.key_lpf_hz` | number | Hz | `1000.0..=12000.0` | `6000.0` | log | Corte de hiss no detector LR4 | 2× biquad LPF `Q=0.7071` |

## 10. Contratos de integração
- `module_id = "pedal.dynamics.noise_gate_ns2"`, `model_revision = "v1-hysteresis-lr4-di-key"`.
- Recebe opcionalmente referência de buffer de sidechain DI limpa (`sidechain_di: Option<&[f32]>`) via `ProcessContext` (Spec 053) sem acoplar ao `RealtimeDspChain`.
- Expõe telemetria atômica de `gain_reduction_db` e `GateState` (`Open/Hold/Closed`) para o LED de atividade da UI.

## 11. Arquitetura de implementação
- **Código atual mapeado:** `crates/ampforge-dsp/src/fx/dynamics.rs` (linhas 4–341).
- **Dívidas registradas:** biquads de sidechain (`SidechainBiquad`) estão em `f32` (migrar para primitiva compartilhada `Biquad64` TDF-II para máxima estabilidade em `key_hpf_hz = 20 Hz` @ 96 kHz); separar em módulo próprio `pedals/noise_gate.rs` desacoplado de `CompressorProcessor`.
- **Estrutura alvo:** `crates/ampforge-dsp/src/pedals/noise_gate.rs` (`NoiseGateNs2`).

## 12. Estratégia DSP
- **Behavioral / DSP clássico de precisão:** sidechain LR4 (4 biquads `f64` TDF-II) + detector True-RMS criticamente amortecido (`220 Hz`) + FSM de 3 estados + suavização exponencial de ganho sem latência (zero lookahead obrigatório no modo padrão; latência = 0).

## 13. Requisitos de tempo real
- Zero alocação/desalocação heap, zero locks, zero panic; filtros recalculados apenas quando `key_hpf_hz`, `key_lpf_hz` ou `sample_rate` mudam.

## 14. Gestão de estado
- `state`, `envelope`, `current_gain`, `hold_remaining_samples` e estados dos 6 biquads internos encapsulados por instância; `reset()` retorna a `Closed` com `current_gain = range_lin`.

## 15. Latência
- **`0` amostras** em todas as taxas (`44,1 / 48 / 88,2 / 96 kHz`).

## 16. Consumo de CPU
- Orçamento alvo: **`<= 1,5%`** do deadline de 128 frames @ 48 kHz em build `--release`.

## 17. Testes
- Burst forte→silêncio validando tempos de `attack_ms`, `hold_ms` e `release_ms` (`±10%`); teste de histerese anti-chatter; medição da resposta em frequência do áudio principal (plana `±0,05 dB` quando aberto); teste de atenuação LR4 em hum 60 Hz no sidechain; modo `classic` expander (`ratio 1:4`); modo `natural` com DI limpa; zero-alloc RT.

## 18. Critérios de aceite
- **CA-1 (Transparência Aberto):** Quando aberto, a resposta de áudio principal de `20 Hz` a `20 kHz` é plana dentro de `±0,05 dB` mesmo com `key_hpf_hz = 500 Hz` e `key_lpf_hz = 1000 Hz`.
- **CA-2 (Histerese e Hold):** Sinal caindo `2 dB` abaixo de `threshold_db` com `hysteresis_db = 4 dB` mantém o gate aberto; ao cair abaixo de `threshold_db - hysteresis_db`, permanece aberto por `hold_ms ± 2 ms` antes de iniciar release.
- **CA-3 (Modo Expander):** Em `classic` com `ratio = 4` e `threshold = -40 dB`, sinal constante a `-50 dBFS` sai em `-80 ± 2 dBFS` e a `-45 dBFS` sai em `-60 ± 2 dBFS`.
- **CA-4 (Sidechain DI no Modo Natural):** Com DI `-3 dB` abaixo do limiar e áudio pós-drive acima, modo `natural` fecha enquanto `classic` abre.
- **CA-5 (Latência Zero, Erro e RT Safety):** `latency_samples() == 0`; `NaN`/`Inf` sanitizados para `0.0`; zero alocações/desalocações em `process()`.
- **CA-6 (Não-regressão):** Testes das Specs 030 e 041 CA-3/CA-4 continuam verdes.

## 19. Dependências
- **Obrigatórias:** `Spec 053` → `Spec 076` (fornecimento de DI limpa no contexto).
- **Relacionadas / Legadas:** `Specs 006, 030, 041`.

## 20. Limitações conhecidas
- Sem lookahead (decisão intencional para latência zero em tempo real); `attack_ms = 0,1 ms` já preserva o transiente de palhetada sem atraso de grupo.

## 21. Riscos técnicos
- Em `key_hpf_hz = 20 Hz` a `96 kHz`, biquads em `f32` perdem precisão de polo; obrigatório migrar `SidechainBiquad` para `f64` TDF-II.

## 22. Referências e rastreabilidade
- **Specs anteriores:** `006`, `030`, `041`.
- **Código atual:** `crates/ampforge-dsp/src/fx/dynamics.rs`.
- **Testes existentes / futuros:** `contract_030_noise_gate.rs`, `contract_041_dynamics_and_drives.rs`, `contract_041_gate_di.rs`, futuro `contract_065_pedal_noise_gate.rs`.
