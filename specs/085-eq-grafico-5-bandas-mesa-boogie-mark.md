---
status: rascunho
---

# Spec 085: Equalizador Gráfico de 5 Bandas Mesa/Boogie Mark Series (80 Hz – 6,6 kHz)

> **Status:** `rascunho` (`bloqueio parcial documental` nos valores exatos das bobinas indutoras de fábrica Mark IIC+ vs Mark IV)
> **Criado em:** 2026-11-20
> **Última atualização:** 2026-11-20 (Criado na Rodada Corretiva por Desacoplamento da Spec 067)
> **Domínio:** `dsp-eq` (`amp-module` / `graphic-eq`)
> **Identificador canônico:** `amp.mesa.mark_5band_eq`
> **Variante / Revisão:** `Mesa/Boogie Mark IIC+ / Mark IV 5-Band Graphic EQ (80 Hz, 240 Hz, 750 Hz, 2200 Hz, 6600 Hz — Rede Ressonante LC Real com Indutores + Transistores Discretos)`
> **Crates / Módulos afetados:** `crates/ampforge-dsp/src/fx/eq.rs`, `crates/sonix-desktop/src/audio.rs`
> **Specs relacionadas:** [031](031-amplificadores-fieis-aos-esquemas-e-controles-originais.md), [053](053-arquitetura-dsp-modular-e-contratos-de-processadores.md), [067](067-pedal-mxr-m108-equalizador-grafico-10-bandas.md), [086](086-modulo-equalizador-parametrico-8-bandas.md)

---

## 1. Objetivo

Especificar o contrato técnico, topológico, matemático e de verificação do **Equalizador Gráfico de 5 Bandas Mesa/Boogie Mark Series (`amp.mesa.mark_5band_eq`)** como módulo independente, desacoplado do **MXR M108 Ten Band EQ** (**[Spec 067](067-pedal-mxr-m108-equalizador-grafico-10-bandas.md)**) e do **Equalizador Paramétrico Digital de 8 Bandas** (**[Spec 086](086-modulo-equalizador-parametrico-8-bandas.md)**).

---

## 2. Escopo

1. Banco de **5 bandas centradas exatamente nas frequências oficiais da série Mesa/Boogie Mark**:
   - **`80 Hz`** (peso subgrave / *thump* de palm-mute);
   - **`240 Hz`** (calor / corpo grave-médio);
   - **`750 Hz`** (centro nasal de médios — historicamente derrubado na clássica *"Curva em V"* do Thrash/Prog Metal dos anos 80/90);
   - **`2200 Hz`** (mordida / ataque de palheta);
   - **`6600 Hz`** (presença / brilho superior).
2. Comportamento interativo característico dos circuitos LC com **indutores reais** (*winding coils* / *gyrator-equivalent LC tanks*) e driver discreto pós-pré-amplificador: forte largura de banda sobreposta (*broad bell overlap*) e ganho assimétrico (`±12 dB` a `±15 dB` conforme revisão), capaz de esculpir o célebre *V-Scoop* (`80 Hz +10 dB`, `240 Hz +3 dB`, `750 Hz -12 dB`, `2200 Hz +6 dB`, `6600 Hz +4 dB`) sem perder definição de ataque.

---

## 3. Fora de escopo

- **MXR M108 Ten Band Graphic EQ (`31,25 Hz – 16 kHz`):** Coberto exclusivamente na **[Spec 067](067-pedal-mxr-m108-equalizador-grafico-10-bandas.md)**.
- **Equalizador Paramétrico de 8 Bandas (`EqProcessor (EqMode::Parametric8Band)`):** Coberto exclusivamente na **[Spec 086](086-modulo-equalizador-parametrico-8-bandas.md)**.
- Circuito completo do amplificador Mesa/Boogie Mark IIC+ / Mark IV (pré-amp de cascata com Tone Stack pré-ganho).

---

## 4. Identificação do equipamento ou componente

| Campo | Valor |
|---|---|
| **Tipo** | `modulo-eq` (`amp-post-preamp-graphic-eq`) |
| **Fabricante de referência** | Mesa Engineering / Mesa/Boogie (Randall Smith, Petaluma, CA, EUA) |
| **Modelo de referência** | Mesa/Boogie Mark Series 5-Band Graphic EQ (`Mark IIC+ / Mark III / Mark IV / Five-Band Graphic`) |
| **Ano / época / revisão** | 1984–1993 (`80 / 240 / 750 / 2200 / 6600 Hz`, estágio discreto pós-pré-amp antes do inversor de fase) |
| **Identificador no sistema** | `amp.mesa.mark_5band_eq` (`GraphicEqModel::Mesa5Band` em `crates/ampforge-dsp/src/fx/eq.rs:95 @ c20cc65`) |
| **Papel na cadeia de sinal** | Bloco `EQ` pós-pré-amplificador (no FX Loop ou entre pré-amp e power-amp, exatamente onde reside nos amplificadores Mark Series!). |

---

## 5. Princípio físico e funcional do equipamento real

Nos amplificadores Mesa/Boogie Mark Series, o Tone Stack passivo rotativo (`Bass/Mid/Treble`) fica localizado **antes** dos estágios de alto ganho (moldando a estrutura de saturação), enquanto o **Equalizador Gráfico de 5 Bandas** fica posicionado **após** todos os estágios de saturação do pré-amplificador (antes do inversor de fase).
1. **Frequências Não-ISO Espaçadas para Guitarra:** Diferente do espaçamento rígido de 1 oitava do MXR M108, Randall Smith escolheu 5 bandas espaçadas por aproximadamente **`1,6 oitavas`** (`80 → 240 → 750 → 2200 → 6600 Hz`, razões `3,00×`, `3,125×`, `2,933×`, `3,00×`), posicionando a banda central exatamente em **`750 Hz`** — o pico nasal típico de pré-amplificadores de alto ganho com overdrive de médios.
2. **Interação de Indutores LC e Saturação Sutil do Estágio Discreto:** Nos modelos clássicos (`Mark IIC+ / III / IV`), as 5 bandas usam redes ressonantes com indutores físicos (ou giradores discretos equivalentes na variante de pedal) acoplados em um estágio amplificador com transistores discretos (`Q1/Q2`), conferindo um $Q$ relativamente largo ($Q \approx 0,9..1,8$) que mescla suavemente as bandas adjacentes.

---

## 6. Fontes técnicas autoritativas

> **Transparência documental (`bloqueio parcial documental`):** As 5 frequências centrais (`80, 240, 750, 2200, 6600 Hz`), o posicionamento pós-pré-amplificador e a função da curva em V são amplamente confirmados pelos manuais oficiais do proprietário da Mesa/Boogie (`F-01`). Contudo, os valores exatos das indutâncias/capacitores e a tolerância de saturação magnética dos indutores da revisão específica `Mark IIC+` vs `Mark IV` dependem do esquema de serviço oficial de fábrica (não incluído no manual do usuário) e permanecem marcados como `PENDING`.

### 6.1. Classificação das fontes

| ID | Título / Documento | URL / Localização | Classificação | Revisão / Escopo |
|---|---|---|---|---|
| `F-01` | Mesa/Boogie — *Mark IV / Mark V / Five-Band Graphic EQ Owner's Manuals* | Manuais oficiais do fabricante (`80 Hz, 240 Hz, 750 Hz, 2200 Hz, 6600 Hz`, posicionamento pós-pré-amp, ±12 dB nominal) | `PRIMARY_OFFICIAL` | Frequências centrais oficiais das 5 bandas e papel na cadeia do amplificador. |
| `F-02` | Análises técnicas de circuitos LC da série Mark (comunidade técnica / engenharia reversa de PCB) | Referência secundária para fatores $Q$ e acoplamento entre bandas | `SECONDARY_TECHNICAL` | Fator de qualidade aproximado ($Q \approx 1,0..1,9$) e curva em V. |

### 6.2. Evidência documental por afirmação técnica

| Informação técnica | Fonte | Revisão | Página / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| 5 frequências centrais: `80 Hz, 240 Hz, 750 Hz, 2200 Hz, 6600 Hz` | `F-01` | Mark IIC+ / Mark IV / Mark V | Seção "5-Band Graphic EQ" | `PRIMARY_OFFICIAL` | `CONFIRMED` |
| Posicionamento pós-pré-amplificador (moldagem espectral pós-distorção) | `F-01` | Mark Series | Diagrama de blocos do manual | `PRIMARY_OFFICIAL` | `CONFIRMED` |
| Valores nominais exatos das 5 bobinas indutoras ($L_1..L_5$) e capacitores de tanque na PCB de fábrica `Mark IIC+` vs `Mark IV` | Esquema de serviço de fábrica não verificado em fonte primária local | Mark IIC+ / IV | N/A | `UNKNOWN` | `PENDING` (`BLOCKED` para modelagem component-level dos 5 indutores; modelado via banco Proportional-Q calibrado nas 5 frequências oficiais) |

---

## 7. Topologia e arquitetura real do sistema

```
[Preamp Out / FX Send] ──► [Discrete Driver Stage] ──► [5-Band Parallel/Cascade LC Resonators: 80 / 240 / 750 / 2200 / 6600 Hz] ──► [Output Stage]
```

### 7.1. Auditoria do código atual e classificação dos achados

1. **`MODEL_COUPLING` (corrigido nesta revisão):** Em [`crates/ampforge-dsp/src/fx/eq.rs:95 @ c20cc65, 112`](file:///g:/Dev/Projetos/Simulador%20de%20efeitos/crates/ampforge-dsp/src/fx/eq.rs#L95-L115), `MESA_5_FREQS = [80.0, 240.0, 750.0, 2200.0, 6600.0]` estava acoplado dentro da mesma struct `EqProcessor (EqMode::Graphic10BandPedal)` que o MXR M108 (Spec 067). Esta Spec estabelece seu contrato isolado (`amp.mesa.mark_5band_eq`).

---

## 8. Controles, parâmetros e mapeamento para a interface

| Controle Real | Chave Canônica | Faixa Aceita | Unidade | Valor Padrão | Papel |
|---|---|---|---|---|---|
| **EQ IN / OUT** | `eq5.enabled` | `false / true` | Booleano | `false` | Ativa/desativa o equalizador de 5 bandas. |
| **80 Hz** | `eq5.b80_db` | `-12.0 .. +12.0` | `dB` | `0.0` | Subgrave / peso de gabinete `4×12`. |
| **240 Hz** | `eq5.b240_db` | `-12.0 .. +12.0` | `dB` | `0.0` | Corpo grave-médio. |
| **750 Hz** | `eq5.b750_db` | `-12.0 .. +12.0` | `dB` | `0.0` | Centro de médios (*V-Scoop*). |
| **2200 Hz** | `eq5.b2200_db` | `-12.0 .. +12.0` | `dB` | `0.0` | Presença / definição de palheta. |
| **6600 Hz** | `eq5.b6600_db` | `-12.0 .. +12.0` | `dB` | `0.0` | Brilho e ar superior. |

---

## 9. Modelo matemático e algoritmo DSP proposto

- 5 filtros peaking *Proportional-Q* em `f64` (TDF-II) nas frequências exatas `[80.0, 240.0, 750.0, 2200.0, 6600.0] Hz` com largura de banda larga característica de tanques LC ($Q(G) = 0,95 + 0,95 \cdot \frac{|G|}{12,0} \in [0,95,\; 1,90]$) e excursão `±12,0 dB`, garantindo sobreposição suave entre as 5 bandas sem *ripples* artificiais.

---

## 10. Estratégia de oversampling, anti-aliasing e precisão numérica

- **Precisão:** `f64` nos coeficientes e estados para manter estabilidade absoluta em `80 Hz` e `240 Hz`.

---

## 11. Contrato de tempo real (Zero-Alloc, Lock-Free, WCET)

- Zero alocações na heap em `process_block` e `apply_param_rt`. WCET $\le 1,5\text{ }\mu\text{s}$ por bloco de `128` amostras.

---

## 12. Posição e interação na cadeia completa de guitarra

- Idealmente inserido pós-pré-amplificador (entre `Amp Preamp` e `Power Amp` ou entre `Amp` e `Cab IR`).

---

## 13. Arquitetura modular e contrato de código Rust

- **Módulo alvo:** `crates/ampforge-dsp/src/fx/eq/mesa_mark_5band.rs` (`pub struct MesaMark5BandEqProcessor` implementando `DspBlock` + `AutomatableParams`, Spec 053).

---

## 14. Parâmetros de calibração e invariantes físicas

| Parâmetro | Valor | Invariante |
|---|---|---|
| Frequências centrais | `80, 240, 750, 2200, 6600 Hz` | Razão de espaçamento `~3,0×` (`~1,6 oitavas`). |
| Transparência central (`0 dB`) | `0,00 dB ± 0,02 dB` | Plano de `20 Hz` a `20 kHz`. |

---

## 15. Estratégia de testes, validação e benchmark

1. Teste de transparência plana em `0,0 dB` e ganho exato `±12,0 dB` nas 5 frequências oficiais.
2. Teste da *"Curva em V"* (`80 Hz +10, 240 Hz +2, 750 Hz -12, 2200 Hz +6, 6600 Hz +4 dB`) verificando contraste de `>= 16 dB` entre `80/2200 Hz` e `750 Hz`.

---

## 16. Riscos conhecidos, limitações e decisões de engenharia

- Calibração component-level dos 5 indutores físicos da PCB Mesa/Boogie permanece `PENDING` até confirmação via esquema oficial de serviço.

---

## 17. Critérios de aceite (CA-01 a CA-04)

- **CA-01 — Transparência Plana em `0,0 dB` e Ação Individual de `±12,0 dB` nas 5 Frequências Oficiais (`80, 240, 750, 2200, 6600 Hz`) (`DOCUMENTED`):**
  - *Sinal de entrada:* Senoides nas 5 frequências centrais a `-20 dBFS`, $f_s = 48\,000\text{ Hz}$, bloco de `128` amostras.
  - *Critério e tolerância:* Em `0,0 dB`, ganho **`0,00 dB ± 0,02 dB`**; em `+12,0 dB` / `-12,0 dB` individual por banda, ganho medido **`±12,0 dB ± 0,3 dB`** na respectiva frequência central.

- **CA-02 — Escultura Clássica "Curva em V" Centrada em `750 Hz` (`DOCUMENTED`):**
  - *Configuração:* `80 Hz = +10 dB`, `240 Hz = +2 dB`, `750 Hz = -12 dB`, `2200 Hz = +6 dB`, `6600 Hz = +4 dB`.
  - *Critério e tolerância:* A atenuação em `750 Hz` deve ficar **pelo menos `16,0 dB` abaixo** da média das magnitudes em `80 Hz` e `2200 Hz`.

- **CA-03 — Comportamento *Proportional-Q* Largo Característico de Tanques LC (`ANALYTICAL` / `ENGINEERING_TARGET`):**
  - *Critério e tolerância:* Na banda de `750 Hz`, o fator $Q$ efetivo em `-12 dB` deve ser estritamente maior que em `-4 dB` (mantendo-se na faixa larga $0,8 \le Q \le 2,2$).

- **CA-04 — Segurança em Tempo Real (`ENGINEERING_TARGET`):**
  - *Critério e tolerância:* `0` alocações na heap em `10 000` blocos de `128` amostras e `100%` das amostras finitas.

---

## 18. Arquivos, módulos e testes afetados

- `crates/ampforge-dsp/src/fx/eq.rs` (e futuro `crates/ampforge-dsp/src/fx/eq/mesa_mark_5band.rs`)
- Testes atuais: `crates/ampforge-dsp/tests/contract_031_pedals_dsp.rs`

---

## 19. Critérios de conclusão da Spec

- [x] Desacoplado integralmente da Spec 067 (MXR M108) e da Spec 086 (Paramétrico 8-Band).
- [x] Frequências oficiais (`80, 240, 750, 2200, 6600 Hz`) documentadas e pendência dos valores exatos dos indutores registrada como `PENDING`.
- [x] Matriz de rastreabilidade preenchida na Seção 22.

---

## 20. Referências

- `[F-01]` Mesa/Boogie — *Mark IV / Mark V / Five-Band Graphic EQ Owner's Manuals*.
- `[F-02]` Análises técnicas de circuitos LC da série Mesa Mark.

---

## 21. Histórico de decisões

- **2026-11-20:** Criada a Spec 085 na rodada corretiva para eliminar o `MODEL_COUPLING` da Spec 067 original.

---

## 22. Matriz de Rastreabilidade

| Requisito | Fonte | Spec / CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| 5 frequências oficiais (`80, 240, 750, 2200, 6600 Hz`) com `±12,0 dB` e transparência em `0 dB` | `F-01` | Spec 085 / `CA-01`, `CA-02` | `crates/ampforge-dsp/src/fx/eq.rs:95 @ c20cc65, 112` (atualmente limitado a `±15 dB` dentro de `EqProcessor (EqMode::Graphic10BandPedal)`) | `contract_031_pedals_dsp.rs` | `IMPLEMENTED_UNVERIFIED` |
| *Proportional-Q* largo (`0,95..1,90`) em módulo isolado | `F-02` | Spec 085 / `CA-03` | `crates/ampforge-dsp/src/fx/eq.rs:166 @ c20cc65` | Ausente | `SPECIFIED` |
| Valores nominais das 5 bobinas indutoras da PCB `Mark IIC+` / `Mark IV` | Sem esquema oficial de fábrica | Spec 085 / §6.2 | N/A | N/A | `PENDING` |
| Segurança zero-alloc em tempo real | Spec 053 | Spec 085 / `CA-04` | `crates/ampforge-dsp/src/fx/eq.rs` | `contract_044_sdd_integrity.rs` | `TESTED` |
