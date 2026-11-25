---
status: rascunho
---

# Spec 087: Módulo Stereo Tri-Chorus (3 Linhas de Atraso Moduladas a 0° / 120° / 240°)

> **Status:** `rascunho`
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Criado na Rodada Corretiva por Desacoplamento da Spec 069)
> **Domínio:** `dsp-modulation` (`modulo-dsp`)
> **Identificador canônico:** `module.mod.stereo_tri_chorus`
> **Variante / Revisão:** `Stereo Tri-Chorus de 3 Linhas Paralelas Defasadas em 120° (Inspirado em DyTronics CS-5 / Songbird TSC-1380 / Rack Estúdio LA Anos 80)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/modulation.rs`, `crates/sonix-desktop/src/audio.rs`
> **Specs relacionadas:** [032](032-pedais-gabinetes-e-efeitos-fieis-aos-esquemas.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [069](069-pedal-boss-ce2-analog-bbd-chorus.md)

---

## 1. Objetivo

Especificar o contrato técnico, matemático e de verificação do **Módulo Stereo Tri-Chorus (`module.mod.stereo_tri_chorus`)** — algoritmo de 3 linhas de atraso moduladas em fase trifásica ($0^\circ$, $120^\circ$ e $240^\circ$) atualmente implementado de forma trocada dentro do braço `ModModel::ChorusCe2` em [`crates/ampforge-dsp/src/fx/modulation.rs:196–218 @ c20cc65`](../crates/ampforge-dsp/src/fx/modulation.rs#L196-L218) — isolando-o definitivamente do pedal analógico mono de linha única **Boss CE-2** (**[Spec 069](069-pedal-boss-ce2-analog-bbd-chorus.md)**).

---

## 2. Escopo

1. **Arquitetura Trifásica (`0° / 120° / 240°`):** Três linhas de atraso moduladas independentes (`Voice L` em $0^\circ$, `Voice C` em $120^\circ$ e `Voice R` em $240^\circ$), centradas em $\tau_0 \approx 12..15\text{ ms}$ com excursão $\pm 6\text{ ms}$.
2. **Estéreo Verdadeiro (L / C / R Panning):** Distribuição panorâmica autêntica de rack de estúdio dos anos 80 (`Voice 1` na Esquerda `L`, `Voice 2` no Centro `L+R`, `Voice 3` na Direita `R`), criando sensação líquida tridimensional sem o efeito "enjoativo" de oscilação unidirecional de pitch (*pitch-seasick wobble*), pois a soma dos vetores de velocidade Doppler das 3 vozes é constantemente zero ($\cos\phi + \cos(\phi + 2\pi/3) + \cos(\phi + 4\pi/3) \equiv 0$!).
3. Filtragem suave de calor analógico (`~8–10 kHz`) e compatibilidade de somatório mono.

---

## 3. Fora de escopo

- **Boss CE-2 Chorus (`MN3007` linha única + LFO triangular + Pré/De-Ênfase):** Coberto exclusivamente na **[Spec 069](069-pedal-boss-ce2-analog-bbd-chorus.md)**.
- **Through-Zero Flanger (`Δτ` cruzando zero):** Coberto na **[Spec 070](070-pedal-through-zero-flanger.md)**.

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `modulo-dsp` (`modulation` / `stereo-tri-chorus`) |
| **Fabricante de referência** | SONIX PRO // AMPFORGE™ (inspirado no conceito trifásico do DyTronics CS-5 / Fulltone TERC dos racks de estúdio de Los Angeles dos anos 80) |
| **Modelo de referência** | Stereo 3-Phase Tri-Chorus (`module.mod.stereo_tri_chorus`) |
| **Ano / época / revisão** | 2026 (Revisão Modular v2 — Spec 087) |
| **Identificador no sistema** | `module.mod.stereo_tri_chorus` (`ModModel::TriChorus` futuro, extraído de `modulation.rs:196–218`) |
| **Papel na cadeia de sinal** | Bloco `MOD` (`mod_1` / `mod_2`) pós-pré-amplificador ou pós-gabinete IR em estéreo. |

---

## 5. Princípio físico e funcional

Em um chorus simples de 1 linha (como o Boss CE-2), enquanto o atraso cresce o pitch cai para baixo; enquanto diminui o pitch sobe para cima, criando uma oscilação perceptível de afinação. No **Tri-Chorus**, três linhas de atraso moduladas por três fases separadas de $120^\circ$ ($\frac{2\pi}{3}$ radianos) garantem que, em qualquer instante $t$, pelo menos uma voz esteja subindo em pitch enquanto outra desce e a terceira passa pelo ponto de inflexão:
$$\sum_{k=0}^{2} \frac{d\tau_k(t)}{dt} = \Delta\tau\,\omega \sum_{k=0}^{2} \cos\!\left(\omega t + \frac{2\pi k}{3}\right) \equiv 0$$
O centro de massa tonal permanece perfeitamente afinado enquanto a imagem estéreo (`L / C / R`) ganha amplitude e densidade exuberante.

---

## 6. Fontes técnicas autoritativas

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Udo Zölzer — *DAFX: Digital Audio Effects* (2nd ed., Wiley, 2011), Cap. 2.6 (*Multi-Voice Chorus*) | Literatura acadêmica de referência | `ACADEMIC` | Equações de chorus multi-voz com fases equidistribuídas $\frac{2\pi k}{M}$ e espalhamento estéreo. |
| `F-02` | Documentação histórica de arquitetura de estúdio (DyTronics CS-5 Tri-Stereo Chorus — conceito L/C/R a $120^\circ$) | Referência conceitual de estúdio | `SECONDARY_TECHNICAL` | Distribuição `Left (0°)`, `Center (120°)` e `Right (240°)` em torno de `12–15 ms`. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| 3 vozes moduladas a $0^\circ / 120^\circ / 240^\circ$ zeram o desvio médio de pitch instantâneo ($\sum \dot{\tau}_k \equiv 0$) | `F-01` | DAFX 2nd ed. | Cap. 2 §2.6 | `ACADEMIC` | `CONFIRMED` |
| Panning estéreo `L / C / R` das 3 vozes proporciona descorrelação estéreo elevada | `F-01`, `F-02` | Multi-Voice Stereo Chorus | §2.6 | `ACADEMIC` / `SECONDARY_TECHNICAL` | `CONFIRMED` |

---

## 7. Topologia e arquitetura real do sistema

```
[Input] ──┬──► [Delay Line 1: τ0 + Δτ·sin(φ + 0°)]   ──► [Left Bus]   ──┐
          ├──► [Delay Line 2: τ0 + Δτ·sin(φ + 120°)] ──► [Center L+R] ──┼──► [Warmth LPF ~8.5 kHz] ──► [Stereo Out L/R]
          └──► [Delay Line 3: τ0 + Δτ·sin(φ + 240°)] ──► [Right Bus]  ──┘
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`MODEL_COUPLING` (corrigido nesta revisão):** Em `crates/ampforge-dsp/src/fx/modulation.rs:196–218 @ c20cc65`, este algoritmo trifásico (`0°/120°/240°` a `15 ms ± 6 ms`) estava rotulado como `ModModel::ChorusCe2` e reduzido a mono `(w1 + w2 + w3) / 3.0`. Esta Spec o formaliza como seu próprio módulo estéreo (`module.mod.stereo_tri_chorus`).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle UI | Chave Canônica | Faixa Aceita | Unidade | Valor Padrão |
|---|---|---|---|---|
| **BYPASS** | `mod_1.enabled` | `false / true` | Booleano | `false` |
| **RATE** | `mod_1.rate_hz` | `0.1 .. 6.0` | `Hz` | `0.75` |
| **DEPTH** | `mod_1.depth` | `0.0 .. 1.0` | `%` | `0.65` |
| **MIX** | `mod_1.mix` | `0.0 .. 1.0` | `%` | `0.50` |

---

## 9. Modelo matemático e algoritmo DSP proposto

- Três leituras por interpolação cúbica Lagrange sobre o buffer circular em $\tau_k[n] = 14,0\text{ ms} + 5,5\text{ ms} \cdot \text{depth} \cdot \sin\!\left(\phi[n] + \frac{2\pi k}{3}\right)$ para $k \in \{0, 1, 2\}$.
- Saída Estéreo:
  $$W_L[n] = \frac{v_0[n] + 0,7071\,v_1[n]}{1,7071}, \qquad W_R[n] = \frac{v_2[n] + 0,7071\,v_1[n]}{1,7071}$$
  seguida de LPF suave em `8,5 kHz` e soma com o sinal seco `(1 - mix) * dry + mix * wet`.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Oversampling:** `1×` com interpolação cúbica Lagrange de 4 pontos.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Buffer circular pré-alocado em `prepare()` para até `30 ms`; zero alocações em `process_block`. WCET $\le 2,5\text{ }\mu\text{s}$ por bloco de `128` amostras.

---

## 12. Posição e interação na cadeia completa de guitarra

- Pós-pré-amplificador ou pós-gabinete IR para abertura estéreo de estúdio (`L/R`).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/modulation/stereo_tri_chorus.rs` (`pub struct StereoTriChorusProcessor` implementando `AudioProcessor` + `ModelDescriptor`, Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Invariante | Regra Verificável |
|---|---|
| Simetria trifásica | Fases exatamente $0^\circ, 120^\circ, 240^\circ$ ($\sum \dot{\tau}_k \equiv 0$). |
| Descorrelação Estéreo (`L` vs `R`) | Em `mix = 1.0, depth = 0.8`, `L` e `R` apresentam diferença RMS significativa (`L - R != 0`). |

---

## 15. Estratégia de testes, validação e benchmark

1. Teste das 3 linhas de atraso a `14 ms` com fases $0^\circ / 120^\circ / 240^\circ$.
2. Teste de largura estéreo (`L` vs `R` descorrelacionados) e preservação do centro de massa de pitch.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Em sistemas mono, a soma `(L + R) * 0.5` equivale a `0.293 * v0 + 0.414 * v1 + 0.293 * v2`, evitando cancelamento severo graças às 3 fases distintas.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Operação Trifásica Real (`0° / 120° / 240°`) com Estabilização do Centro de Pitch (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoide contínua de `440 Hz` a `-12 dBFS` por `2,0 s`, $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Configuração:* `rate_hz = 1.0`, `depth = 0.70`, `mix = 1.0`.
  - *Critério e tolerância:* A soma das derivadas das 3 linhas $\sum_{k=0}^2 \dot{\tau}_k[n]$ deve ser nula (`<= 1e-6`), gerando bandas laterais simétricas em torno de `440 Hz` (energia em `440 ± 1..5 Hz`) com centróide de frequência centrado em **`440,0 Hz ± 2,0 Hz`**.

- **CA-02 — Imagem Estéreo Verdadeira (`L / C / R`) (`ANALYTICAL`):**
  - *Sinal de entrada:* Senoide `440 Hz` mono (`L_in == R_in`) a `-12 dBFS`.
  - *Configuração:* `rate_hz = 1.0`, `depth = 0.80`, `mix = 1.0`.
  - *Critério e tolerância:* A energia do sinal diferencial estéreo (`Side = (L - R) / 2`) deve ser **pelo menos `-12 dB`** relativa à energia de `Mid = (L + R) / 2`, comprovando saída estéreo verdadeira (e não duplo-mono).

- **CA-03 — Preservação de Impulso em `depth = 0.0` (Atraso Central `14,0 ms`) (`ANALYTICAL`):**
  - *Sinal de entrada:* Impulso unitário (`x[0] = 1.0`).
  - *Configuração:* `depth = 0.0`, `mix = 1.0`.
  - *Critério e tolerância:* O pico da resposta ao impulso deve ocorrer exatamente em **`14,0 ms ± 0,2 ms`**, distinguindo-o do atraso de `7,53 ms` do Boss CE-2 (Spec 069).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Critério e tolerância:* `0` alocações na heap em `10 000` blocos de `128` amostras e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/modulation.rs` (e futuro `crates/ampforge-dsp/src/fx/modulation/stereo_tri_chorus.rs`)

---

## 19. Critérios de conclusão da Spec

- [x] Desacoplado integralmente da Spec 069 (Boss CE-2).
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Zölzer, U. (2011). *DAFX: Digital Audio Effects* (2nd ed.). Wiley, Cap. 2 §2.6.
- `[F-02]` Arquitetura DyTronics CS-5 Tri-Stereo Chorus (`L / C / R` $0^\circ / 120^\circ / 240^\circ$).

---

## 21. Histórico de decisões

- **2026-11-20:** Criada a Spec 087 na rodada corretiva para isolar o algoritmo trifásico atualmente codificado em `modulation.rs:196–218` sob o rótulo equivocado de `ChorusCe2`.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| 3 linhas de atraso moduladas em $0^\circ / 120^\circ / 240^\circ$ em torno de `14–15 ms` | `F-01` | Spec 087 / `CA-01`, `CA-03` | `crates/ampforge-dsp/src/fx/modulation.rs:196–218 @ c20cc65` (atualmente acoplado sob `ModModel::ChorusCe2`) | `contract_032_time_mod_fx.rs` | `IMPLEMENTED_UNVERIFIED` |
| Distribuição panorâmica estéreo verdadeira `L (0°) / C (120°) / R (240°)` | `F-01`, `F-02` | Spec 087 / `CA-02` | Ausente em `modulation.rs:212` (soma mono `(w1+w2+w3)/3.0`) | Ausente | `SPECIFIED` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 087 / `CA-04` | `crates/ampforge-dsp/src/fx/modulation.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
