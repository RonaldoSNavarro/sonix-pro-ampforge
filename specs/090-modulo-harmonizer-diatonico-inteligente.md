---
status: rascunho
---

# Spec 090: Módulo Harmonizer Diatônico Inteligente (Smart Pitch Shifter por Tonalidade e Escala)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Criado na Rodada Corretiva por Desacoplamento da Spec 075)
> **Domínio:** `dsp-pitch` (`modulo-dsp`)
> **Identificador canônico:** `module.pitch.diatonic_harmonizer`
> **Variante / Revisão:** `Harmonizer Diatônico Monofônico Inteligente (Detector de Pitch de Baixa Latência + Tabela Diatônica de Escalas + Shifter de Voz Harmônica)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/pitch.rs`, `crates/sonix-desktop/src/audio.rs`, `apps/sonix-app/ui/src/controls.registry.ts`
> **Specs relacionadas:** [042](042-bloco-de-pitch-sem-processamento.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [075](075-bloco-pitch-whammy-drop-harmonizer-detune.md), [083](083-componente-afinador-yin-e-looper-de-performance.md)

---

## 1. Objetivo

Especificar o contrato técnico, musical, algorítmico e de verificação do **Módulo Harmonizer Diatônico Inteligente (`module.pitch.diatonic_harmonizer`)**, desacoplando-o definitivamente do **DigiTech Whammy** (**[Spec 075](075-bloco-pitch-whammy-drop-harmonizer-detune.md)**), do **DigiTech Drop** (**[Spec 089](089-pedal-digitech-drop-transposicao-polifonica.md)**) e do **Eventide H3000 MicroPitch** (**[Spec 091](091-rack-eventide-h3000-micropitch-detune.md)**). Diferente de um pitch shifter de intervalo cromático fixo (que produz notas fora da escala em duetos de guitarra), o Harmonizer Diatônico detecta continuamente a nota tocada ($n_{\text{MIDI}}$) e ajusta dinamicamente o deslocamento em semitons ($\Delta s(n_{\text{MIDI}})$) conforme a **Tonalidade (`Key`)**, a **Escala (`Scale`)** e o **Grau Harmônico (`Interval`)** selecionados — reproduzindo os clássicos duetos de guitarra em terças/sextas (Iron Maiden, Thin Lizzy, Queen, Boston).

---

## 2. Escopo

1. **Detector de Frequência Fundamental em Tempo Real (Baixa Latência / Zero-Alloc):** Estimador leve (autocorrelação sub-amostrada ou AMDF / YIN compacto sobre janela deslizante pré-alocada de `512–1024` amostras com *histerese de troca de nota* de `±25 cents`) para rastrear solos monofônicos de guitarra (`82 Hz` a `1318 Hz`).
2. **Motor de Mapeamento Diatônico Exato (Teoria Musical):**
   - **12 Tonalidades (`Key`):** `C, C#, D, Eb, E, F, F#, G, Ab, A, Bb, B` (`0..11`).
   - **Escalas Suportadas (`Scale`):**
     - `Major (Ionian)`: `[0, 2, 4, 5, 7, 9, 11]`;
     - `Natural Minor (Aeolian)`: `[0, 2, 3, 5, 7, 8, 10]`;
     - `Harmonic Minor`: `[0, 2, 3, 5, 7, 8, 11]`;
     - `Dorian`: `[0, 2, 3, 5, 7, 9, 10]`;
     - `Mixolydian`: `[0, 2, 4, 5, 7, 9, 10]`.
   - **Intervalos Diatônicos (`Interval`):** `±3rd`, `±4th`, `±5th`, `±6th`, `±Octave`.
   - **Exemplo Verificável (`Key = C`, `Scale = Major`, `Interval = +3rd`):**
     - Nota `C` (`0`) $\to$ `E` (`4`, **`+4 semitons`**, terça maior);
     - Nota `D` (`2`) $\to$ `F` (`5`, **`+3 semitons`**, terça menor);
     - Nota `E` (`4`) $\to$ `G` (`7`, **`+3 semitons`**, terça menor);
     - Nota `F` (`5`) $\to$ `A` (`9`, **`+4 semitons`**, terça maior);
     - Nota `G` (`7`) $\to$ `B` (`11`, **`+4 semitons`**, terça maior);
     - Nota `A` (`9`) $\to$ `C` (`12`, **`+3 semitons`**, terça menor);
     - Nota `B` (`11`) $\to$ `D` (`14`, **`+3 semitons`**, terça menor).
3. **Suavização Anti-Glitch de Transição de Intervalo (`Glide 5 ms`)** quando o intervalo alterna entre `+3 st` e `+4 st`, mais controle de mistura `Mix` (`0.50` = dueto balanceado `1:1`).

---

## 3. Fora de escopo

- Rastreamento harmônico polifônico de acordes completos simultâneos (harmonizadores inteligentes de guitarra são fisicamente projetados para linhas melódicas monofônicas).
- Varreduras cromáticas contínuas por pedal de expressão (Whammy → Spec 075).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `modulo-dsp` (`pitch` / `diatonic-harmonizer`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (Arquitetura algorítmica inspirada nos harmonizadores diatônicos Eventide H3000 / Boss PS-6 / DigiTech HarmonyMan) |
| **Modelo de referência** | Intelligent Diatonic Pitch Harmonizer (`module.pitch.diatonic_harmonizer`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 090) |
| **Identificador no sistema** | `module.pitch.diatonic_harmonizer` (`PitchMode::Harmonizer` em `crates/ampforge-dsp/src/fx/pitch.rs`) |
| **Papel na cadeia de sinal** | Bloco `PITCH`, posicionado antes do amplificador (para som coeso de 2 guitarras num mesmo amp) ou pós-pré-amplificador (para clareza máxima de estúdio de duas vozes separadas). |

---

## 5. Princípio físico e funcional

Dado o grau $d \in \{0..6\}$ da nota detectada na escala diatônica $S = [s_0, s_1, s_2, s_3, s_4, s_5, s_6]$ (relativo à tônica $K \in \{0..11\}$) e o deslocamento de grau diatônico $\Delta d$ (ex.: `+2` graus para uma 3ª acima, `+4` graus para uma 5ª acima, `-2` graus para uma 3ª abaixo), a nota alvo é:
$$d_{\text{target}} = (d + \Delta d) \bmod 7, \qquad \text{oct\_shift} = \left\lfloor \frac{d + \Delta d}{7} \right\rfloor$$
$$\Delta\text{semitones} = (S[d_{\text{target}}] + 12 \cdot \text{oct\_shift}) - S[d]$$
Quando a nota tocada estiver fora da escala (nota cromática de passagem), o motor mantém o último intervalo diatônico válido com histerese ou usa o grau mais próximo (sem saltos erráticos).

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Teoria Musical Diatônica / Temperamento Igual de 12 Tons (`12-TET`) | Formulação matemática exata de graus diatônicos em escalas heptatônicas | `ANALYTICAL` | Mapeamento determinístico de `Key + Scale + Degree` para semitons exatos. |
| `F-02` | Udo Zölzer — *DAFX: Digital Audio Effects* (2nd ed., Wiley, 2011), Cap. 6 & Cap. 9 (*Pitch Detection & Shifting*) | Literatura acadêmica de referência | `ACADEMIC` | Detecção de frequência fundamental em baixa latência e transposição de voz harmônica. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| Tabela de intervalos diatônicos (`C Major +3rd`: `C→E +4st`, `D→F +3st`, `E→G +3st`, `F→A +4st`, `G→B +4st`, `A→C +3st`, `B→D +3st`) | `F-01` | 12-TET | Equação §5 | `ANALYTICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Input] ──┬───────────────────────────────────────────────────────────────► (Dry) ──┐
          │                                                                         ├──► [Mix Out]
          ├──► [Low-Latency Pitch Detector + Hysteresis] ──► [Diatonic Map Table]   │
          │                                                        │ (Δsemitones)   │
          └──► [Pitch Shifter Voice (Hann-Windowed Cubic Interp)] ◄┘ ────► (Wet) ───┘
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`MODEL_COUPLING` (corrigido nesta revisão):** Estava acoplado dentro da Spec 075 e de `pitch.rs`.

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica (Spec 042) | Faixa Aceita | Unidade | Valor Padrão |
|---|---|---|---|---|
| **BYPASS** | `pitch.enabled` | `false / true` | Booleano | `false` |
| **KEY** | `pitch.key` | `0..11` (`C..B`) | Nota | `0` (`C`) |
| **SCALE** | `pitch.scale` | `"major"`, `"minor"`, `"harmonic_minor"`, `"dorian"`, `"mixolydian"` | Enum | `"major"` |
| **INTERVAL** | `pitch.harmony_interval` | `-7..-2, +2..+7` (graus diatônicos) | Grau | `+3` (`+3rd`) |
| **MIX** | `pitch.mix` | `0.0 .. 1.0` | `%` | `0.50` (`Dueto 1:1`) |

---

## 9. Modelo matemático e algoritmo DSP proposto

- Execução do estimador leve de pitch a cada `128` amostras (bloco RT) usando apenas aritmética fixa sobre buffer pré-alocado (`<= 12 µs` WCET, zero alocação), atualizando o fator suavizado $r[n] = 2^{\Delta s_{\text{sm}}[n] / 12}$ no shifter de voz harmônica com janelamento complementar `sin²`.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Anti-aliasing:** Filtro passa-baixas adaptativo $f_c = \min(18\text{ kHz},\; 0,44 f_s / r)$ quando $r > 1,0$.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Buffers do detector e da linha de atraso pré-alocados integralmente em `prepare()`. Zero alocações na heap em `process_block`.

---

## 12. Posição e interação na cadeia completa de guitarra

- Pré-amp (intermodulação orgânica de 2 guitarras) ou pós-pré-amp (dueto limpo de alta definição).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/pitch/diatonic_harmonizer.rs` (`pub struct DiatonicHarmonizerProcessor` implementando `AudioProcessor` + `ModelDescriptor`, Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Regra Verificável |
|---|---|
| Exatidão diatônica em `C Major +3rd` | `C4 (261,63 Hz)` $\to$ `E4 (329,63 Hz, +4 st)`; `D4 (293,66 Hz)` $\to$ `F4 (349,23 Hz, +3 st)`. |
| Exatidão diatônica em `A Natural Minor +3rd` | `A3 (220,00 Hz)` $\to$ `C4 (261,63 Hz, +3 st)`; `C4 (261,63 Hz)` $\to$ `E4 (329,63 Hz, +4 st)`. |

---

## 15. Estratégia de testes, validação e benchmark

1. Teste de todas as 7 notas da escala de `C Maior` com intervalo `+3rd`, verificando alternância automática entre terça maior (`+4 st`) e terça menor (`+3 st`).
2. Teste da escala de `A Menor Natural` e `A Menor Harmônica` (`G#` sensível).

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Histerese de `±25 cents` no detector evita alternância frenética entre graus adjacentes durante *vibrato* de dedo do guitarrista.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Alternância Diatônica Automática entre Terça Maior (`+4 st`) e Terça Menor (`+3 st`) em `C Major` (`ANALYTICAL` / `REGRESSION`):**
  - *Sinal de entrada:* Duas notas sequenciais de `500 ms` cada a `-12 dBFS` ($f_s = 48\,000\text{ Hz}$, bloco de `128` amostras): Nota 1 = `C4 (261,63 Hz)` e Nota 2 = `D4 (293,66 Hz)`.
  - *Configuração:* `module.pitch.diatonic_harmonizer` com `key = C`, `scale = "major"`, `interval = +3rd`, `mix = 1.0` (voz harmônica isolada).
  - *Método de medição:* Pico espectral dominante FFT nos últimos `300 ms` de cada nota.
  - *Critério e tolerância:*
    1. Para entrada `C4 (261,63 Hz)`, a voz harmônica deve gerar `E4` em **`329,63 Hz ± 5,0 Hz`** (`+4 semitons`, terça maior);
    2. Para entrada `D4 (293,66 Hz)`, a voz harmônica deve gerar `F4` em **`349,23 Hz ± 5,0 Hz`** (`+3 semitons`, terça menor).

- **CA-02 — Escala `A Harmonic Minor` (`G#` Sensível) (`ANALYTICAL`):**
  - *Sinal de entrada:* `E4 (329,63 Hz)` em `key = A`, `scale = "harmonic_minor"`, `interval = +3rd`, `mix = 1.0`.
  - *Critério e tolerância:* A voz harmônica deve gerar `G#4` em **`415,30 Hz ± 6,0 Hz`** (`+4 semitons`, terça maior característica da menor harmônica, e não `G4 = 392 Hz` da menor natural).

- **CA-03 — Balanço Dueto (`mix = 0.50`) e Imunidade a Vibrato (`±20 cents`) (`ANALYTICAL`):**
  - *Sinal de entrada:* `C4` modulado em vibrato `±20 cents` a `5 Hz`.
  - *Critério e tolerância:* O grau diatônico travado deve permanecer estável em `+4 semitons` sem oscilar para `B` ou `C#`, e com `mix = 0.50` ambos os picos (`C4` e `E4`) devem coexistir dentro de `±3,0 dB`.

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Critério e tolerância:* `0` alocações na heap em `10 000` blocos de `128` amostras e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/pitch.rs` (e futuro `crates/ampforge-dsp/src/fx/pitch/diatonic_harmonizer.rs`)
- Testes atuais: `Planejado: crates/ampforge-dsp/tests/contract_042_pitch_wah.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Desacoplado integralmente da Spec 075 (Whammy), Spec 089 (Drop) e Spec 091 (MicroPitch).
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Teoria Musical Diatônica (12-TET).
- `[F-02]` Zölzer, U. (2011). *DAFX: Digital Audio Effects* (2nd ed.). Wiley, Cap. 6 & 9.

---

## 21. Histórico de decisões

- **2026-11-20:** Criada a Spec 090 na rodada corretiva para isolar o Harmonizer Diatônico Inteligente da Spec 075.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| Tabela diatônica exata (`Major`, `Natural Minor`, `Harmonic Minor`, `Dorian`, `Mixolydian`) com detector em tempo real | `F-01`, `F-02` | Spec 090 / `CA-01`, `CA-02` | `crates/ampforge-dsp/src/fx/pitch.rs:329–352 @ c20cc65` | `contract_042_pitch_wah.rs` | `IMPLEMENTED_UNVERIFIED` |
| Histerese anti-vibrato (`±25 cents`) e dueto balanceado `1:1` | `F-02` | Spec 090 / `CA-03` | Parcial em `pitch.rs` | Ausente | `SPECIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 090 / `CA-04` | `crates/ampforge-dsp/src/fx/pitch.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
