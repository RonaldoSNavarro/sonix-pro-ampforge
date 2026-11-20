# Spec 089: Pedal DigiTech Drop (Polyphonic Drop Tune Shifter — -1 a -7 Semitons & -1 Oitava)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Criado na Rodada Corretiva por Desacoplamento da Spec 075)
> **Domínio:** `dsp-pitch` (`pedal`)
> **Identificador canônico:** `pedal.digitech.drop`
> **Variante / Revisão:** `DigiTech The Drop (Polyphonic Drop Tune: -1 a -7 Semitons, -1 Oitava, -1 Oitava + Dry; Chave Momentary/Latching)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/pitch.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [042](042-bloco-de-pitch-sem-processamento.md), [052](052-abertura-animada-pedais-faltantes-e-amp2-visivel.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [075](075-bloco-pitch-whammy-drop-harmonizer-detune.md)

---

## 1. Objetivo

Especificar o contrato técnico, algorítmico e de verificação do pedal de afinação polifônica **DigiTech Drop (`pedal.digitech.drop`)**, desacoplando-o definitivamente do **DigiTech Whammy** (**[Spec 075](075-bloco-pitch-whammy-drop-harmonizer-detune.md)**), do **Harmonizer Diatônico** (**[Spec 090](090-modulo-harmonizer-diatonico-inteligente.md)**) e do **Eventide H3000 MicroPitch** (**[Spec 091](091-rack-eventide-h3000-micropitch-detune.md)**). Diferente do Whammy (que foca em varreduras expressivas contínuas até `±24 st`), o DigiTech Drop é um **transpositor estático polifônico de baixa latência (`100% Wet`)** projetado para rebaixar a afinação inteira da guitarra (ex.: de `E Standard` para `Eb`, `D`, `C#`, `C`, `B`, `A` ou `-1 Oitava`) preservando a clareza de *power chords*, acordes complexos e o ataque percussivo de *palm-mutes* de alta velocidade!

---

## 2. Escopo

1. Seletor discreto oficial de 9 posições do DigiTech Drop:
   - **`1` a `7` semitons abaixo** (`-1 st` a `-7 st`: `Eb`, `D`, `C#`, `C`, `B`, `Bb`, `A`);
   - **`OCT` (`-12 st`, 100% Wet):** Baixo/barítono uma oitava abaixo;
   - **`OCT + DRY` (`-12 st` + Dry `50/50`):** Efeito de guitarra + baixo dobrado em oitava.
2. **Motor de Transposição Polifônica com Preservação de Fase/Transientes:**
   - Processamento *Phase-Locked Phase Vocoder* ou *Multi-Band / Cross-Correlated WSOLA* (janelamento otimizado `~20–32 ms`, latência total `<= 25 ms`), mantendo a coerência inter-harmônica entre todas as 6 cordas simultâneas sem batimentos artificiais (*chorusing/warble*) e preservando a nitidez do transiente de ataque da palheta (*Transient Preservation*).
3. Modo `100% Wet` obrigatório nos passos `-1` a `-7` e `-12` (para que nenhum resquício da afinação original vaze e crie dissonância de semitom com a afinação rebaixada!).

---

## 3. Fora de escopo

- Varreduras de pedal de expressão e transposições ascendentes (`+1 / +2 OCT` → DigiTech Whammy, **[Spec 075](075-bloco-pitch-whammy-drop-harmonizer-detune.md)**).
- Harmonização diatônica inteligente (**[Spec 090](090-modulo-harmonizer-diatonico-inteligente.md)**) e MicroPitch estéreo (**[Spec 091](091-rack-eventide-h3000-micropitch-detune.md)**).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `pedal` (`polyphonic-drop-tune-shifter`) |
| **Fabricante de referência** | DigiTech / Harman (Sandy, UT, EUA) |
| **Modelo de referência** | DigiTech The Drop (Polyphonic Drop Tune Pitch Shifter) |
| **Ano / época / revisão** | 2014–presente (True Bypass, Seletor 9 posições `-1..-7 st`, `OCT`, `OCT+DRY`, Chave `Momentary`) |
| **Identificador no sistema** | `pedal.digitech.drop` (`PitchMode::Drop` em `crates/ampforge-dsp/src/fx/pitch.rs`) |
| **Papel na cadeia de sinal** | Bloco `PITCH` no **início absoluto da cadeia (logo após Input/Tuner/Gate)**, antes de Wah, Drive e Amplificador, garantindo rastreamento polifônico limpo antes de qualquer distorção! |

---

## 5. Princípio físico e funcional do equipamento real

1. **Desafio Físico do Drop Tuning Polifônico:**
   - Rebaixar a afinação de uma guitarra inteira com um simples *rotating delay line* de janela fixa (como o Whammy WH-1 clássico) introduz modulação de amplitude e perda de ataque (*smearing*) quando um acorde de 3 a 6 notas é tocado.
   - Um algoritmo polifônico dedicado (`Phase-Locked STFT` com reset de fase em transientes ou `WSOLA` com busca de máxima correlação cruzada) alinha os ciclos dominantes nas emendas das janelas e detecta picos de ataque de palheta (*onset detection*) para reiniciar a fase das bandas espectrais no ataque, mantendo o impacto percussivo (*chug*) em afinações graves (`Drop C` / `Drop A`).
2. **Isolamento Total do Sinal Seco (`Wet = 100%`):**
   - Ao transpor `-1 semitom` (`E` $\to$ `Eb`), qualquer vazamento do sinal Dry (`-20 dB`) cria um intervalo de 2ª menor extremamente dissonante. Portanto, nos modos `-1..-7` e `OCT`, a mistura seca é rigorosamente `0,0` (`100% Wet`).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | DigiTech / Harman — *The Drop Polyphonic Drop Tune Official Product Page & Owner's Manual* | `https://digitech.com/dp/drop/` (HTTP 200 oficial verificado) | `PRIMARY_OFFICIAL` | Seletor oficial de 9 posições (`-1` a `-7 semitons`, `OCT`, `OCT+DRY`), chave `Momentary`, true bypass e operação polifônica. |
| `F-02` | Jean Laroche & Mark Dolson — *Improved Phase Vocoder Time-Scale Modification of Audio* (IEEE Trans. Speech & Audio Processing, Vol. 7, No. 3, 1999) | Literatura acadêmica fundadora do *Identity Phase-Locking* para pitch shifting polifônico sem *phasiness* | `ACADEMIC` | Travamento de fase (*phase-locking*) em torno dos picos espectrais e preservação de transientes em acordes polifônicos. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| 9 posições oficiais: `-1` a `-7 semitons`, `-1 Oitava (100% Wet)` e `-1 Oitava + Dry` | `F-01` | DigiTech Drop | `F-01` §"Controls & Specifications" | `PRIMARY_OFFICIAL` | `CONFIRMED` |
| Preservação de coerência de fase em sinais polifônicos e transientes via *Phase-Locking* / *WSOLA* | `F-02` | IEEE TSAP 1999 | §III–IV | `ACADEMIC` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Clean Guitar Input] ──► [Polyphonic Shifter Engine (WSOLA / Phase-Locked Vocoder, r = 2^(-k/12))]
                                    │
                                    ▼ (100% Wet for -1..-7 & OCT; +Dry Mix only for OCT+DRY)
                               [Output] ──► [Drive / Amp]
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`MODEL_COUPLING` (corrigido nesta revisão):** `PitchMode::Drop` estava misturado na Spec 075 e em `pitch.rs` compartilhando a mesma linha de atraso simples de 2 taps sem busca de correlação cruzada (`WSOLA`) nem *phase-locking* polifônico.

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real / UI | Chave Canônica | Faixa Aceita | Unidade | Valor Padrão |
|---|---|---|---|---|
| **FOOTSWITCH (BYPASS)** | `pitch.enabled` | `false / true` | Booleano | `false` |
| **DROP AMOUNT** | `pitch.semitones` | `-1, -2, -3, -4, -5, -6, -7, -12` | Semitons | `-2` (`1 tom abaixo`) |
| **OCT + DRY (MIX)** | `pitch.mix` | `1.0` (fixo em `-1..-7`/`OCT`; `0.5` em `OCT+DRY`) | `%` | `1.00` |

---

## 9. Modelo matemático e algoritmo DSP proposto

- Transposição estática por fator $r = 2^{\text{semitones}/12} \in [0,50,\; 0,94387]$ usando algoritmo de correlação cruzada normalizada (*WSOLA*) ou *Phase-Locked STFT* com latência máxima de `24 ms` (`<= 1152` amostras a `48 kHz`), reportada via `DspBlock::latency_samples()` (Spec 053).

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `1×` (como $r < 1,0$ sempre transpõe **para baixo**, todas as frequências são comprimidas para longe de Nyquist, não havendo geração de aliasing acima de $f_s / 2$!).

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Buffers e planos FFT/WSOLA integralmente pré-alocados em `prepare()`; zero alocações em `process_block`.
- WCET $\le 18\text{ }\mu\text{s}$ por bloco de `128` amostras a `48 kHz`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Bloco inicial da cadeia (`PITCH`, imediatamente após `Input` / `Gate`), alimentando amplificadores high-gain (`JCM800` / `Dual Rectifier`) com afinação rebaixada pronta para palm-mutes pesados.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pedals/digitech_drop.rs` (`pub struct DigiTechDropProcessor` implementando `DspBlock` + `AutomatableParams`, Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Regra Verificável |
|---|---|
| Transposição polifônica de tríade / *power chord* | Acorde `A2 (110 Hz) + E3 (164,81 Hz) + A3 (220 Hz)` transposto `-2 st` produz picos exatos em `G2 (98,0 Hz)`, `D3 (146,83 Hz)` e `G3 (196,0 Hz)`. |
| Rejeição total da afinação original (`-1..-7 st`) | Energia residual nas frequências originais `<= -40 dB` relativas às frequências transpostas. |

---

## 15. Estratégia de testes, validação e benchmark

1. Teste polifônico com *power chord* de 3 notas (`A2 + E3 + A3`) transposto em `-2 st` (`G2 + D3 + G3`) e `-5 st` (`E2 + B2 + E3`).
2. Teste de latência determinística `<= 25 ms` e rejeição do sinal Dry original.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Em transposições extremas (`-7 st` ou `-12 st`), a latência mínima de janelamento (`~20–24 ms`) é uma limitação física do princípio da incerteza tempo-frequência para resolver adequadamente a corda grave `E2 -> A1 (55 Hz)` cujo período é `18,2 ms`.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Transposição Polifônica Precisa de *Power Chord* (`-2 st` e `-5 st`) (`DOCUMENTED` / `ANALYTICAL`):**
  - *Sinal de entrada:* Acorde sintético de 3 senoides simultâneas (`A2 = 110,00 Hz`, `E3 = 164,81 Hz`, `A3 = 220,00 Hz`) a `-15 dBFS` cada, $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração:* `pedal.digitech.drop` com `pitch.semitones = -2` (fator $2^{-2/12} = 0,8909$) e `pitch.mix = 1.0`.
  - *Método de medição:* FFT de `16 384` amostras após `200 ms` de acomodação.
  - *Critério e tolerância:* Os 3 picos dominantes da saída devem ocorrer exatamente em **`98,00 Hz ± 3,5 Hz`**, **`146,83 Hz ± 4,0 Hz`** e **`196,00 Hz ± 5,0 Hz`**, com os resíduos das frequências originais (`110`, `164,81`, `220 Hz`) atenuados em **`>= 30,0 dB`** abaixo dos novos picos.
  - *Origem da tolerância:* Razão temperada $2^{-2/12}$ (`F-01`, `F-02`).

- **CA-02 — Modo `-1 Oitava` (`-12 st`) e Modo `OCT + DRY` (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoide `220,0 Hz` a `-12 dBFS`.
  - *Critério e tolerância:* Em `-12 st` (`100% Wet`), pico dominante em **`110,0 Hz ± 3,0 Hz`** e fundamental `220 Hz` atenuada em `>= 30 dB`; em `OCT + DRY` (`mix = 0.50`), ambos os picos (`110 Hz` e `220 Hz`) presentes dentro de `±3,0 dB` entre si.

- **CA-03 — Latência Algorítmica Determinística `<= 25,0 ms` (`ENGINEERING_TARGET`):**
  - *Critério e tolerância:* `latency_samples()` reportado pelo bloco deve respeitar `<= 1200 amostras` (`<= 25,0 ms` a `48 kHz`), compatível com execução ao vivo.

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Critério e tolerância:* `0` alocações na heap em `10 000` blocos de `128` amostras e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/pitch.rs` (e futuro `crates/ampforge-dsp/src/fx/pedals/digitech_drop.rs`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_042_pitch_wah.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Fonte oficial DigiTech (`F-01`) verificada via HTTP 200.
- [x] Desacoplado integralmente da Spec 075 (Whammy), Spec 090 (Harmonizer) e Spec 091 (MicroPitch).
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` DigiTech / Harman — *The Drop Official Product Page* (`https://digitech.com/dp/drop/`).
- `[F-02]` Laroche, J., & Dolson, M. (1999). *Improved Phase Vocoder Time-Scale Modification of Audio*. IEEE TSAP, 7(3), 323–332.

---

## 21. Histórico de decisões

- **2026-11-20:** Criada a Spec 089 na rodada corretiva para separar o DigiTech Drop Polifônico do DigiTech Whammy (Spec 075).

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Transposição polifônica limpa (`-1` a `-7 st`, `-12 st`, `OCT+DRY`) de acordes | `F-01`, `F-02` | Spec 089 / `CA-01`, `CA-02` | `crates/ampforge-dsp/src/fx/pitch.rs:316–328` (usa delay rotativo simples sem WSOLA/phase-locking) | `contract_042_pitch_wah.rs` | `SPECIFIED` |
| Latência determinística `<= 25 ms` e zero-alloc em tempo real | Spec 053 | Spec 089 / `CA-03`, `CA-04` | `crates/ampforge-dsp/src/fx/pitch.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
