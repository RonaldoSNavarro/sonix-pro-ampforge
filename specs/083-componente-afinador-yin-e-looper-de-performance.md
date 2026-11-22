---
status: rascunho
---

# Spec 083: Componente de Afinador Estroboscópico e Polifônico (Algoritmo YIN Desacoplado da Thread RT)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Rodada Corretiva de Auditoria Documental e Fidelidade — Desacoplamento do Looper)
> **Domínio:** `dsp-utility` (`componente`)
> **Identificador canônico:** `component.tuner.yin_strobe`
> **Variante / Revisão:** `Detector de Pitch YIN Monofônico (±0,5 cent, Calibração A4 430–450 Hz, Desacoplado da Thread de Áudio RT) + Extensão Polifônica`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/tuner.rs`, `crates/sonix-desktop/src/audio.rs`, `crates/sonix-desktop/src/api.rs`
> **Specs relacionadas:** [026](026-afinador-ao-vivo-na-entrada.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [076](076-componente-entrada-condicionamento-impedancia-captadores.md), [092](092-componente-looper-de-performance.md)

---

## 1. Objetivo

Especificar o contrato técnico, matemático, de desacoplamento em tempo real e de verificação **exclusivamente** do **Componente de Afinador (`component.tuner.yin_strobe`)** em [`crates/ampforge-dsp/src/tuner.rs`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/tuner.rs#L1-L493), desfazendo o agrupamento indevido (`MODEL_COUPLING`) da versão anterior desta Spec — que misturava o Afinador YIN com o **Looper de Performance** (agora isolado integralmente na **[Spec 092](092-componente-looper-de-performance.md)**) — e mantendo conformidade rigorosa com a **Regra Promovida Cortex `20261111-023654-dsp-fd17`** (análise pesada YIN executada exclusivamente fora da thread RT, enquanto o callback RT apenas grava amostras em um ring buffer lock-free).

---

## 2. Escopo

1. **Captura Lock-Free na Thread RT (`O(1)`, Zero-Alloc):** No callback de áudio, logo após o DC Blocker do componente de entrada (Spec 076) e **antes** de qualquer distorção/modulação, o afinador apenas copia amostras do sinal limpo para um ring buffer circular pré-alocado (`2048` a `4096` amostras), opcionalmente silenciando a saída de áudio quando `tuner.mute == true` (com rampa de `5 ms` anti-clique).
2. **Análise YIN Desacoplada na Thread de Telemetria (`30–60 Hz`):** Executa o algoritmo YIN de Alain de Cheveigné & Hideki Kawahara (`F-01`) sobre **scratch buffers estáticos pré-alocados (`d_scratch`, `d_prime_scratch` de `2048` amostras)** sem qualquer alocação dinâmica durante a execução:
   - Função de diferença cumulativa normalizada $d'(\tau)$;
   - Limiar absoluto de periodicidade ($\theta_{\text{YIN}} = 0,15$) para imunidade a erros de oitava (*octave errors*);
   - Interpolação parabólica do mínimo local $\tau^*$ para precisão sub-amostra de **`<= ±0,5 cent`** de `27,5 Hz` (`A0` / baixo 5 cordas / guitarra 8 cordas) até `1400 Hz` (`F6`).
3. **Calibração de Referência (`A4 = 430,0 Hz .. 450,0 Hz`, padrão `440,0 Hz`)** e telemetria completa (`freq_hz`, `note_name`, `octave`, `cents_offset`, `confidence`, `strobe_phase`).

---

## 3. Fora de escopo

- **Looper de Performance (Gravação, Overdub, Undo/Redo, Half-Speed, Reverse):** Movido integralmente para a **[Spec 092](092-componente-looper-de-performance.md)**.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `componente` (`dsp-utility` / `pitch-tuner`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (baseado no algoritmo YIN de de Cheveigné & Kawahara, JASA 2002) |
| **Modelo de referência** | Decoupled Zero-Alloc YIN Strobe Tuner (`component.tuner.yin_strobe`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 083, compatível com Spec 026 e Regra Cortex `fd17`) |
| **Identificador no sistema** | `component.tuner.yin_strobe` (`crates/ampforge-dsp/src/tuner.rs`) |
| **Papel na cadeia de sinal** | Derivação paralela (*tap*) imediatamente após o condicionamento de entrada (`Spec 076`) antes de qualquer pedal ou amplificador. |

---

## 5. Princípio físico e funcional

O algoritmo **YIN** (`F-01`) supera a autocorrelação simples e a detecção por cruzamento por zero em instrumentos de corda dedilhada (onde harmônicos superiores muitas vezes superam a amplitude da fundamental na corda `E2 = 82,41 Hz`) em três etapas matemáticas:
1. **Função de Diferença:** $d_t(\tau) = \sum_{j=1}^{W} (x_{t+j} - x_{t+j+\tau})^2$.
2. **Normalização pela Média Cumulativa (`CMNDF`):**
   $$d'_t(\tau) = \begin{cases} 1, & \tau = 0 \\ \dfrac{d_t(\tau)}{\frac{1}{\tau}\sum_{j=1}^{\tau} d_t(j)}, & \tau > 0 \end{cases}$$
   Isso elimina o viés de energia para pequenos atrasos $\tau$ e evita falsos disparos em harmônicos superiores!
3. **Interpolação Parabólica:** Ajusta uma parábola nos 3 pontos $(\tau-1, \tau, \tau+1)$ em torno do primeiro mínimo abaixo do limiar $\theta_{\text{YIN}} = 0,15$, alcançando resolução melhor que `0,2 cent` mesmo em `E4 = 329,63 Hz` ou `E5 = 659,25 Hz` a `48 kHz`.

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Alain de Cheveigné & Hideki Kawahara — *YIN, a fundamental frequency estimator for speech and music* (JASA, Vol. 111, No. 4, pp. 1917–1930, 2002) | Literatura acadêmica fundadora do algoritmo YIN | `ACADEMIC` | Equações da CMNDF $d'(\tau)$, limiar absoluto $\theta$, interpolação parabólica e rejeição de erro de oitava. |
| `F-02` | Regra Promovida Cortex `20261111-023654-dsp-fd17` & Spec 026 | [`AGENTS.md`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/AGENTS.md) / [`specs/026-afinador-dsp-zero-alloc-desacoplado.md`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/specs/026-afinador-dsp-zero-alloc-desacoplado.md) | `INTERNAL_REFERENCE` | Contrato de desacoplamento RT vs Thread de Telemetria (`30–60 Hz`) e scratch buffers pré-alocados. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Função CMNDF $d'(\tau)$ + interpolação parabólica alcança precisão sub-cent (`<= ±0,5 cent`) imune a erro de oitava | `F-01` | JASA 2002 | §II.B–E, pp. 1919–1922 | `ACADEMIC` | `CONFIRMED` |
| Proibição absoluta de rodar a autocorrelação/YIN dentro do callback RT (apenas cópia para ring buffer lock-free) | `F-02` | Spec 026 | Regra Cortex `fd17` | `INTERNAL_REFERENCE` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Input Conditioner (Spec 076)] ──┬──► (if tuner.mute == true: 5ms fade to 0.0) ──► [Rest of Chain]
                                 │
                                 ▼ (RT Thread: O(1) Copy to Lock-Free Ring Buffer)
                        [Audio Snapshot Ring (2048..4096 samples)]
                                 │
                                 ▼ (Telemetry / I/O Thread @ 30–60 Hz, Pre-Allocated Scratch)
                        [YIN CMNDF + Parabolic Interpolation] ──► [TunerState JSON / UI Strobe]
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`MODEL_COUPLING` (corrigido nesta revisão):** A Spec 083 original agrupava o Afinador (`tuner.rs`) e o Looper (`looper.rs`) em um único documento. O Looper foi movido integralmente para a **[Spec 092](092-componente-looper-de-performance.md)**.
2. **`CONFIRMED` (`crates/ampforge-dsp/src/tuner.rs:1–493 @ c20cc65`):** O código atual já cumpre exemplarmente o desacoplamento da thread RT e o uso de `d_scratch` / `d_prime_scratch` pré-alocados (verificado pela Spec 026).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica (Spec 014) | Faixa Aceita | Unidade | Valor Padrão | Papel |
|---|---|---|---|---|---|
| **TUNER ACTIVE** | `tuner.enabled` | `false / true` | Booleano | `false` | Abre o visor estroboscópico e ativa análise na thread de telemetria. |
| **MUTE OUTPUT** | `tuner.mute` | `false / true` | Booleano | `true` | Silencia a saída do simulador (com rampa `5 ms`) para afinação silenciosa ao vivo. |
| **REFERENCE A4** | `tuner.a4_hz` | `430.0 .. 450.0` | `Hz` | `440.0` | Calibração de referência de concerto (`A4 = 432 Hz`, `440 Hz`, `442 Hz`, etc.). |

---

## 9. Modelo matemático e algoritmo DSP proposto

- Conversão de frequência detectada $f_{\text{det}}$ para nota MIDI e desvio em *cents*:
  $$m = 69,0 + 12,0 \cdot \log_2\!\left(\frac{f_{\text{det}}}{f_{A4}}\right), \qquad m_{\text{round}} = \text{round}(m), \qquad \text{cents} = 100,0 \cdot (m - m_{\text{round}})$$
- Porta de silêncio (*RMS Gate* em `-55 dBFS`) para não exibir ruído aleatório quando nenhuma corda está vibrando.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Precisão:** `f32`/`f64` nos buffers de diferença acumulada `d_scratch` (`2048` amostras) para suportar até `B0 = 30,87 Hz` ($\tau \approx 1555$ amostras a `48 kHz`) com sobra.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Na thread RT: apenas escrita sequencial de `128` amostras em buffer circular pré-alocado (`WCET <= 0,2 µs`, `0` alocações, `0` locks).
- Na thread de telemetria (`30–60 Hz`): `0` alocações dinâmicas por chamada de `analyze()` (buffers `d_scratch` e `d_prime_scratch` pré-alocados).

---

## 12. Posição e interação na cadeia completa de guitarra

- Lê sempre o sinal limpo pós-DC Blocker (antes de `Drive` e `Amp`), garantindo precisão independente do ganho do amplificador.

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo:** `crates/ampforge-dsp/src/tuner.rs` (`TunerProcessor`, `TunerResult`).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Valor |
|---|---|
| Precisão nas 6 cordas padrão (`E2` a `E4`) e drop (`B0`/`D2`) | Erro absoluto **`<= ±0,5 cent`**. |
| Custo na thread RT | `0` alocações, `0` locks, `0` cálculos de autocorrelação YIN na RT. |

---

## 15. Estratégia de testes, validação e benchmark

1. Teste de precisão em todas as 6 cordas padrão da guitarra (`82,41`, `110,00`, `146,83`, `196,00`, `246,94`, `329,63 Hz`) com mistura rica de harmônicos (`H2` e `H3` mais fortes que `H1`, simulando captador de ponte!).
2. Teste de calibração `A4 = 432 Hz` e `442 Hz`.
3. Teste de `zero-alloc` tanto na RT quanto em `TunerProcessor::analyze()`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- O modo polifônico simultâneo de 6 cordas (estilo TC PolyTune via banco de filtros FFT de alta resolução) permanece como extensão futura sobre o mesmo ring buffer desacoplado sem alterar o contrato RT.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Precisão Sub-Cent (`<= ±0,5 cent`) nas 6 Cordas da Guitarra com Harmônicos Dominantes (`DOCUMENTED` / `REGRESSION`):**
  - *Sinal de entrada:* Sinais sintéticos de guitarra contendo fundamental $f_0$ somada a 2º harmônico ($1,2\times$ mais forte que a fundamental!) e 3º harmônico ($0,8\times$) para as 6 cordas padrão (`82,407 Hz`, `110,000 Hz`, `146,832 Hz`, `195,998 Hz`, `246,942 Hz`, `329,628 Hz`) e afinação grave `B0 = 30,868 Hz` a `-12 dBFS`.
  - *Taxa de amostragem:* $f_s = 48\,000\text{ Hz}$.
  - *Configuração e parâmetros:* `tuner.a4_hz = 440.0`.
  - *Critério e tolerância:* Em todas as 7 frequências testadas, o afinador deve identificar a nota exata sem erro de oitava e reportar `|cents_offset| <= 0,50 cent`.
  - *Origem da tolerância:* Algoritmo YIN com interpolação parabólica (`F-01`).

- **CA-02 — Calibração de Referência `A4` (`432 Hz` e `442 Hz`) (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoide de `432,0 Hz` com `tuner.a4_hz = 432.0`, e senoide de `442,0 Hz` com `tuner.a4_hz = 442.0`.
  - *Critério e tolerância:* Ambos os casos devem reportar nota `A4` com `|cents_offset| <= 0,25 cent`.

- **CA-03 — Mute Silencioso Anti-Clique (`tuner.mute = true`) (`REGRESSION`):**
  - *Sinal de entrada:* Senoide contínua de `440 Hz` a `-6 dBFS`.
  - *Critério e tolerância:* Ativar `tuner.mute = true` leva a saída de áudio para `0,0` em `<= 10 ms` sem clique abrupto, enquanto o afinador continua detectando `440,0 Hz` normalmente (pois lê antes do mute).

- **CA-04 — Zero-Alloc e Desacoplamento Absoluto da Thread RT (Regra Cortex `fd17`) (`ENGINEERING_TARGET` / `REGRESSION`):**
  - *Método de medição:* Alocador global instrumentado durante `10 000` blocos RT de `128` amostras + `500` chamadas de `TunerProcessor::analyze()`.
  - *Critério e tolerância:* `0` alocações/desalocações na heap na thread RT e `0` alocações em `analyze()`.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/tuner.rs`, `crates/sonix-desktop/src/audio.rs`
- Testes atuais: `crates/ampforge-dsp/tests/contract_026_tuner_zero_alloc.rs`, `crates/ampforge-dsp/tests/contract_044_sdd_integrity.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Desacoplado completamente do **Looper de Performance** (movido para a **[Spec 092](092-componente-looper-de-performance.md)**).
- [x] Conformidade integral com a Regra Promovida Cortex `fd17` e Spec 026.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` de Cheveigné, A., & Kawahara, H. (2002). *YIN, a fundamental frequency estimator for speech and music*. JASA, 111(4), 1917–1930.
- `[F-02]` Spec 026 & Regra Promovida Cortex `20261111-023654-dsp-fd17`.

---

## 21. Histórico de decisões

- **2026-11-20:** Na rodada corretiva, desfez-se o acoplamento da Spec 083: a Spec 083 ficou exclusiva para o **Afinador YIN (`component.tuner.yin_strobe`)**, enquanto o **Looper de Performance** foi isolado na **Spec 092**.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Precisão `<= ±0,5 cent` sem erro de oitava (CMNDF + interpolação parabólica) | `F-01` | Spec 083 / `CA-01` | `crates/ampforge-dsp/src/tuner.rs:1–493 @ c20cc65` | `contract_026_tuner_zero_alloc.rs` | `TESTED` |
| Calibração `A4` (`430..450 Hz`) | Spec 026 | Spec 083 / `CA-02` | `crates/ampforge-dsp/src/tuner.rs` | `contract_026_tuner_zero_alloc.rs` | `TESTED` |
| Mute silencioso anti-clique mantendo leitura do afinador ativa | Spec 026 | Spec 083 / `CA-03` | `crates/sonix-desktop/src/audio.rs` | `contract_026_tuner_zero_alloc.rs` | `TESTED` |
| Desacoplamento total da RT e zero-alloc em `analyze()` | `F-02` | Spec 083 / `CA-04` | `crates/ampforge-dsp/src/tuner.rs` & `audio.rs` | `contract_026_tuner_zero_alloc.rs`, `contract_044_sdd_integrity.rs` | `TESTED` |
