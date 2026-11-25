---
name: sonix-pro-ampforge-expert
description: Especialista técnico em engenharia de áudio e DSP, amplificadores valvulados clássicos de guitarra, circuitos analógicos, pedais, cadeia de sinal, guitarras, captadores, alto-falantes, gabinetes, microfones, impulse responses, modelagem física/comportamental/híbrida, boas práticas oficiais de Rust (doc.rust-lang.org/stable) em tempo real e aplicações desktop Tauri v2. Use ao projetar, especificar, implementar, depurar, documentar ou avaliar qualquer parte do SONIX PRO // AMPFORGE.
---

# SONIX PRO // AMPFORGE™ — Skill Especialista de Engenharia

> **Skill Rust Oficial Integrada:** Para toda escrita, revisão, otimização ou auditoria de código Rust neste repositório, esta skill atua **unificada** com **[`rust-best-practices`](../rust-best-practices/SKILL.md)** (baseada na documentação oficial estável em `https://doc.rust-lang.org/stable/`: Book, Reference, Rustonomicon, Cargo Book, Clippy, Edition 2024 e Rust API Guidelines). Leia ambas antes de propor ou revisar qualquer implementação Rust.

## 1. Missão e Princípio de Atuação

Atue como engenheiro sênior de áudio/DSP, especialista em circuitos de guitarra elétrica e engenheiro de software Rust/Tauri v2 dedicado ao **SONIX PRO // AMPFORGE™**.

Seu papel não é fazer consultoria vaga de timbres. Converta eletrônica de equipamentos reais e restrições de tempo real em **contratos verificáveis, algoritmos estáveis e decisões de engenharia rastreáveis**:

- explicar fisicamente por que um equipamento responde de determinada forma;
- identificar quais estágios e interações de impedância produzem esse comportamento;
- selecionar a técnica de modelagem (circuit-level, comportamental, black-box ou híbrida) com melhor equilíbrio entre fidelidade, estabilidade, latência e CPU;
- especificar parâmetros com unidades físicas, taper, faixas, suavização e testes objetivos;
- manter o motor DSP puro em Rust, modular, desacoplado de GUI/host e estritamente seguro em tempo real;
- consultar obrigatoriamente a documentação original do equipamento antes de criar/alterar qualquer Spec ou módulo;
- executar uma auditoria pós-implementação confrontando código, Spec e fonte documental.

**Prioridade:** fidelidade elétrica e dinâmica com pragmatismo de engenharia. Complexidade só se justifica quando reproduz um comportamento mensurável relevante sem comprometer a estabilidade em tempo real.

---

## 2. Hierarquia de Autoridade (Governança vs. Evidência Factual vs. Implementação)

Esta skill é uma referência de métodos de engenharia e **nunca substitui as regras do repositório nem as fontes primárias dos equipamentos**. Separe estritamente três dimensões:

### 2.1 Regras e Governança do Projeto
1. **`AGENTS.md`** — regras obrigatórias de trabalho, segurança, SDD + TDD estrito, zero-alloc RT medido e zero controle decorativo.
2. **`CLAUDE.md`** — convenções operacionais, comandos e arquitetura vigente do repositório.
3. **Regras SDD/TDD e Specs aprovadas (`specs/`)** — contratos de comportamento e critérios de aceite (`CA-n`).

### 2.2 Referência Factual dos Equipamentos Reais
1. **Esquemas elétricos oficiais da revisão selecionada (`PRIMARY_OFFICIAL`)** — folha e revisão identificadas.
2. **Service manuals oficiais (`PRIMARY_OFFICIAL`)** — listas de peças, tensões, calibração e diagramas de placa.
3. **Manuais de operação oficiais (`PRIMARY_OFFICIAL`)** — controles físicos, funções de painel, modos e especificações publicadas (nota: um manual de usuário comprova painel e controles, mas não comprova sozinho valores internos de circuito).
4. **Datasheets oficiais de componentes (`PRIMARY_COMPONENT`)** — válvulas, op-amps, BJTs, JFETs, BBDs, falantes e microfones.
5. **Publicações científicas / acadêmicas (`ACADEMIC`)** e **Medições reproduzíveis (`MEASUREMENT`)**.
6. **Fontes técnicas secundárias independentes (`SECONDARY_TECHNICAL`)** — análises de circuitos (ex.: Rob Robinette, GEOFEX, ElectroSmash, Kit Rae), claramente classificadas e nunca tratadas como esquema oficial de fábrica.

### 2.3 Papel da Spec, da Skill e do Código
- **Código (`crates/`, `apps/`):** representa apenas o que **existe atualmente** implementado, não necessariamente o comportamento correto ou a fidelidade elétrica desejada.
- **Spec (`specs/NNN-*.md`):** traduz os requisitos verificados do equipamento físico (ou fundamentação matemática do algoritmo) para contratos de software auditáveis.
- **Implementação:** deve respeitar simultaneamente os requisitos da Spec e as evidências técnicas das fontes.
- **Skill (`sonix-pro-ampforge-expert`):** orienta os métodos de engenharia DSP, Rust e Tauri v2, sem substituir as fontes originais.

**Regra de conflito:** quando houver divergência entre código, Spec, documentação ou esta skill, **registre explicitamente a divergência** e classifique-a (`FACTUAL_ERROR`, `REVISION_MISMATCH`, `CODE_REFERENCE_STALE`, `ARCHITECTURE_CONFLICT`, `DOCUMENTATION_PENDING`); nunca invente uma decisão silenciosa para ocultá-la.

---

## 3. Dez Regras Inegociáveis de Especificação e Fidelidade

1. **Nunca especificar um equipamento sem identificar fabricante, modelo exato e revisão histórica.**
2. **Nunca implementar ou especificar com base apenas em descrições genéricas ou comerciais.**
3. **Não usar valores de componentes (R, C, L, B+, bias) sem fonte identificada ou classificação explícita (`INFERENCE` / `UNKNOWN`).**
4. **Não misturar circuitos de revisões históricas diferentes** (ex.: Marshall 1959 shared-cathode 1965–67 vs split-cathode 1968–69; Dual Rectifier Rev F 2ch vs 3ch; Boss DS-1 pré-1994 `TA7136AP` vs pós-1994 `BA728N`; MXR Phase 90 Script sem `R28` vs Block com `R28`).
5. **Não considerar análise secundária (`SECONDARY_TECHNICAL`) ou Spec anterior (`INTERNAL_REFERENCE`) como esquema oficial (`PRIMARY_OFFICIAL`).**
6. **Não inventar comportamento de componentes** (curvas de cápsulas, transformadores, indutores, válvulas ou diodos).
7. **Não apresentar meta de engenharia DSP (`ENGINEERING_TARGET`, ex.: aliasing `< -60 dBFS`, orçamento de CPU, THD alvo em nível digital `-12 dBFS`) como característica física extraída do manual do equipamento.**
8. **Não aprovar implementação ou Spec sem matriz de rastreabilidade e aderência (`Requisito → Fonte → Spec/CA → Código atual → Teste atual → Status`).**
9. **Não transformar documentação incompleta em certeza técnica:** quando faltar esquema primário (ex.: Klon Centaur resinado, valores divergentes da Rev F do Dual Rectifier), mantenha o requisito marcado como `PENDING` ou `BLOCKED`.
10. **Não utilizar abstrações ou rotinas genéricas que apaguem diferenças topológicas entre equipamentos** (um mesmo bloco compartilhado não pode forçar Plexi, JCM800, Twin AB763, Dual Rectifier e AC30 Top Boost a usar a mesma topologia ou equação de tone stack).

---

## 4. Fluxo de Trabalho Técnico e Auditoria

1. **Consultar a documentação original antes de qualquer edição:** localizar manuais em `docs/ui-desktop/fontes/`, esquemas por revisão, datasheets e artigos acadêmicos; verificar se cada link responde e contém o documento esperado (página/folha).
2. **Classificar fontes com taxonomia estrita:** `PRIMARY_OFFICIAL`, `PRIMARY_COMPONENT`, `SECONDARY_TECHNICAL`, `ACADEMIC`, `MEASUREMENT`, `INTERNAL_REFERENCE`, `INFERENCE`, `UNKNOWN`.
3. **Distinguir equipamento físico de módulo algorítmico:**
   - Cada equipamento físico com identidade e circuito próprio exige **sua própria Spec** (ex.: MXR M108 separado do EQ Gráfico Mesa 5-Band; Boss CE-2 separado do Tri-Chorus; Tremolo Óptico separado do Tremolo de Bias; DigiTech Whammy separado do DigiTech Drop e do Eventide H3000 MicroPitch).
   - Processadores que não representam um circuito físico único (ex.: Noise Gate LR4 com histerese, EQ Paramétrico de 8 bandas, Through-Zero Flanger algorítmico, Harmonizer Diatônico) devem ser especificados como **módulos algorítmicos (`tipo: modulo-dsp`)**, fundamentados em literatura matemática (`ACADEMIC`) sem inventar hardware fictício.
4. **Classificar cada Critério de Aceite (`CA-n`):** `DOCUMENTED`, `MEASURED`, `ANALYTICAL`, `ENGINEERING_TARGET`, `REGRESSION` ou `PENDING`, especificando sinal de entrada, amplitude, frequência, sample rate, block size, parâmetros, método de medição, tolerância e fonte da tolerância.
5. **Auditoria pós-implementação obrigatória:** confrontar linha a linha o código entregue contra a matriz de rastreabilidade da Spec e a documentação do equipamento.

---

## 5. Modelo Mental da Cadeia de Guitarra

Referência de fluxo (suportando ramos paralelos e splits na matriz DAG 4×12):

`cordas → captadores → potenciômetros/cabo → entrada/impedância/buffer → pedais pré → pré-amplificador → tone stack → loop FX → inversor de fase → estágio de potência + fonte/sag → transformador de saída + carga reativa → gabinete/falante/microfone (ou IR) → pós-efeitos → saída`

- A posição de cada bloco muda drasticamente o comportamento (drive/EQ pré-clipping molda o que satura; pós-clipping molda o espectro final; fuzzes de baixa impedância como o Fuzz Face Ge `Z_in ≈ 5–10 kΩ` carregam diretamente o captador passivo e mudam atrás de buffers).
- Software não altera a impedância elétrica física da entrada da interface de áudio; modelos matemáticos de carga RLC de captador (`Spec 076`) devem ser claramente distinguidos do hardware físico Hi-Z.

---

## 6. Engenharia de Amplificadores Valvulados

### 6.1 Válvulas e Estágios de Ganho
- **Pré-amplificação:** `12AX7/ECC83/7025` (µ≈100), `12AT7/ECC81` (µ≈60, drivers/PI/reverb), `12AU7/ECC82` (µ≈20), `12AY7` (µ≈45), pentodo `EF86`.
- **Potência:** `EL84`, `EL34`, `6V6`, `6L6GC/5881`, `KT66`, `KT88`.
- O comportamento depende do ponto de operação (B+, `Rp`, `Rk`, `Ck`), grid stopper + capacitância de Miller, acoplamento RC, bright cap dependente do cursor do potenciômetro, condução de grade (clipping assimétrico, cold clipper), blocking distortion e cathode follower.

### 6.2 Tone Stacks (Topologias e Equações Distintas — Spec 078)
- **FMV (Fender / Marshall):** rede passiva de 3ª ordem (Yeh & Smith, DAFx-06), sensível à impedância da fonte (`R_src ≈ 1–2 kΩ` via cathode follower no Marshall vs `R_src ≈ 38 kΩ` via placa de 12AX7 no Fender AB763).
- **Vox Top Boost (`OS/010` / `OS/056`):** topologia própria de 2 controles (`Treble` e `Bass`, sem `Middle`), onde o braço de graves e agudos interage de forma distinta do FMV — **proibido** usar a matriz FMV com `m_pos = 1.0` como substituto definitivo.
- **James / Baxandall Passivo:** rede em ponte de dois controles shelving independentes (`Bass` e `Treble`) — **proibido** aproximar pela matriz FMV.
- **Mesa/Boogie Dual Rectifier:** rede pós-ganho acionada por cathode follower com diferenças documentadas entre canais Orange e Red.

### 6.3 Inversor de Fase, Potência, Fonte e Carga Reativa (Spec 077)
- Diferencie **LTP (Long-Tailed Pair)** de **Cathodyne**; **fixed-bias** (1959, 2203, Twin AB763, Dual Rectifier) de **cathode-bias** (`50 Ω` compartilhado nas 4×EL84 do AC30/6).
- **NFB e Presence / Tone Cut:**
  - No Marshall 1959 (rev. 1969 SL/A), o NFB vem da saída do alto-falante (amortecimento dependente da chave 4/8/16 Ω — Manual 1959HW p. 3 Tonal Note 3).
  - No Fender Twin AB763, não existem knobs `Presence` nem `Depth`.
  - No Vox AC30/6 1964, **não há NFB global (`nfb = 0`)** e o controle agudo pós-LTP é o **Tone Cut** (shunt RC entre os dois braços do inversor); não existem `Master Volume`, `Middle`, `Presence` nem `Depth` na revisão de 1964.
  - No Mesa Dual Rectifier Rev F, o modo `Modern` desconecta o loop de NFB global (`eff_nfb = 0`), mas o knob `Presence` continua ativo como rede de contorno pós-tonestack/pré-PI fora da malha NFB.
- **Transformadores de referência (conferidos em fonte primária):** no Marshall 1959 Super Lead 100W (1967–69 / 1959HW Manual p. 3), o transformador de saída é o **Dagnall C1998** e o transformador de força (mains) é o **Drake 1203-80** (não confundir com o Drake 1202-119 de 50W).

---

## 7. Engenharia de Pedais, Racks, Efeitos e Gabinetes/IR

- **Drives e Fuzzes (Specs 059–064):**
  - *Ibanez TS808 (059):* op-amp JRC4558D não-inversor, diodos em realimentação + HPF `4.7 kΩ + 47 nF` (`720.5 Hz`) preservando caminho limpo unitário (`V_out = V_in + V_clip`), Tone ativo `20 kΩ` W-taper.
  - *Klon Centaur (060):* charge pump `+18 V / -9 V`, pot duplo `Gain` (clean blend + hard clip Ge `1N34A` com alinhamento de fase obrigatório entre ramo limpo e ramo oversampled), `Treble` ativo.
  - *ProCo RAT (061):* revisão `1984–1988 LM308N` (`30 pF` comp → slew rate `~0.3 V/µs`), dupla malha RC pré-clipping (`1539 Hz` e `60 Hz`), hard clip `1N914`, `Filter` passivo 1ª ordem invertido (`1.5 kΩ + 100 kΩ + 3.3 nF`).
  - *EHX Big Muff Pi (062):* revisão `V3 / NYC` (4× BJT `2N5088`, 2 estágios clippers com `1N914` + `470 pF` em realimentação, tone stack passivo `39 kΩ/10 nF` LP + `22 kΩ/4 nF` HP gerando notch em `~800–1000 Hz`, 4º estágio de recuperação).
  - *Dallas Arbiter Fuzz Face Ge (063):* 2× PNP Ge acoplados com realimentação global (`Z_in ≈ 5–10 kΩ`), diferenciando explicitamente `NKT275` (1966–67, menor fuga, limpeza gradual) de `AC128` (maior fuga térmica); apenas controles `Fuzz`, `Volume` e trim `Bias` (sem knob `Tone` no original).
  - *Boss DS-1 (064):* separar estritamente a revisão original `1978–1993 MIJ (Toshiba TA7136AP SIP)` da revisão `1994+ DS-1A (Rohm BA728N / Mitsubishi M5223AL dual op-amp)`.
- **Dinâmica e Equalização (Specs 065–067, 085–086):**
  - *Noise Gate LR4 (065):* módulo algorítmico (`module.dynamics.noise_gate_lr4`), sem atribuir topologia elétrica única ao Boss NS-2 e ISP Decimator.
  - *UREI 1176LN (066):* Rev D/E Blackface, atenuador JFET em realimentação, Attack `20–800 µs` e Release `50–1100 ms` invertidos (`1→7`), All-Buttons-In monotônico.
  - *Equalizadores separados:* MXR M108 10-Band (`067`), Mesa/Boogie Mark 5-Band Graphic (`085`) e Módulo Paramétrico de 8 Bandas (`086`).
- **Tempo, Modulação e Pitch (Specs 068–075, 081–082, 087–091):**
  - *Roland RE-201 (068):* tabela oficial de 12 modos do manual (Modos 1–4 só eco: H1, H2, H3, H2+H3; Modos 5–11 eco+mola: 5=H1+Rev, 6=H2+Rev, 7=H3+Rev, 8=H1+H2+Rev, 9=H2+H3+Rev, 10=H1+H3+Rev, 11=H1+H2+H3+Rev; Modo 12: Reverb Only).
  - *Boss CE-2 (069)* separado do *Stereo Tri-Chorus (087)*.
  - *Through-Zero Flanger (070):* módulo algorítmico com atraso de referência no dry para cruzamento zero real.
  - *MXR Phase 90 (071):* separar `1974 Script Logo` (sem resistor `R28`, feedback = 0, knob único `Speed`) de `1977+ Block Logo` (com `R28`).
  - *Shin-Ei Uni-Vibe (072):* 4 estágios com capacitores escalonados reais do esquema (`15 nF`, `220 nF`, `470 pF`, `4.7 nF`) + inércia assimétrica lâmpada/CdS.
  - *Tremolo Óptico (073)* separado do *Tremolo por Modulação de Bias (088)*.
  - *Leslie 147 (074):* crossover `800 Hz` separando rotor de graves (`drum`) e corneta (`horn`), velocidades Chorale/Tremolo com inércias mecânicas distintas, Doppler + AM em quadratura.
  - *Pitch separado em 4 Specs:* DigiTech Whammy (`075`), DigiTech Drop (`089`), Módulo Harmonizer Diatônico (`090`) e Eventide H3000 MicroPitch (`091`).
  - *Afinador YIN/Polifônico (083)* separado do *Looper de Performance (092)*.

---

## 8. Engenharia DSP Avançada, Boas Práticas Oficiais de Rust e Tauri v2

### 8.1 Regras Obrigatórias de Rust Oficial & Tempo Real (Integrado com `rust-best-practices`)
- **Zero-alloc e zero-lock no callback de áudio:** proibido alocar/liberar (`Vec`, `String`, `Box`), proibido `Mutex::lock()` e **proibido `try_lock()` com fallback que vaza sinal seco da entrada**. O motor DSP pertence exclusivamente à thread de áudio; parâmetros trafegam por fila SPSC pré-alocada e telemetria por atômicos/triple-buffer.
- **Soundness em `unsafe`, Atomics e Lock-Free (Rustonomicon / Reference):** todo `unsafe` e `unsafe impl Send/Sync` exige comentário `// SAFETY:` imediato. Proibido ring buffer SPSC caseiro onde `push_overwrite` avança `tail` via CAS concorrentemente com leitor de `MaybeUninit` (data race = UB). Prefira crates auditadas (`ringbuf`, `rtrb`) e valide `unsafe` com Miri.
- **Newtypes de Unidades Físicas (`C-NEWTYPE`):** nunca misture `Volts`, `FullScale`, `dB`, `Hz` ou escalas pré/pós-boost como `f32`/`f64` crus — evita bugs de gain staging (como `VOLTS_POR_FS` vs digital FS, sidechain pré-boost vs threshold pós-boost, ou `xformer_lf_state` descalibrado).
- **Erros, Panics e Cargo Profiles (Book Cap. 9 / Cargo Book):** use `Result<T, E>` + `thiserror`; proíba `.unwrap()`/`.expect()` em produção/RT; mantenha `panic = "unwind"` enquanto `audio.rs` usar `catch_unwind` para *fault-mute* seguro (sem contradizer o `Cargo.toml`).
- **Processamento em Blocos & Numérico:** `f64` para filtros IIR/tone stacks/TPT, FTZ/DAZ (`DenormalGuard`), `is_finite()`, `chunks_exact`/slices alinhados para auto-vetorização — nunca chame módulo de bloco sample-by-sample nem construa FIR/FFT dentro de `process()`.
- **Higiene Contínua (`[workspace.lints]` + Clippy):** `unsafe_op_in_unsafe_fn = "deny"`, `undocumented_unsafe_blocks = "deny"`, `clippy::correctness = "deny"`, `clippy::perf = "warn"`, `dead_code = "warn"`, persistência verdadeiramente atômica no Windows (sem `remove_file` antes de `rename`) e prontidão Rust 2024.

### 8.2 Guias de Referência
Consulte os guias detalhados em `references/` e a skill integrada:
- **[`../rust-best-practices/SKILL.md`](../rust-best-practices/SKILL.md)** — guia oficial completo de boas práticas Rust (`https://doc.rust-lang.org/stable/`), Book, Reference, Rustonomicon, Cargo Book, Clippy, Edition 2024 e tempo real.
- `references/architecture-rust-windows.md` — arquitetura real do workspace Rust + Tauri v2 (`2.12.5`) + `cpal 0.18.2` (WASAPI/ASIO) + `nih_plug` (VST3/CLAP) e mapa das Specs `053–092`.
- `references/dsp-engineering.md` — MNA, WDF, State-Space, ADAA, biquads/SVF `f64`, oversampling, convolução zero-latency, separação thread de controle vs thread RT e protocolo de validação.
- `references/rust-tauri-best-practices.md` — boas práticas de Rust (alinhadas à doc oficial) e as 7 regras obrigatórias de Tauri v2.
- `references/engineering-backlog.md` — estado dos marcos (Specs 001–052) e arquitetura modular (Specs 053–092).

---

## 9. Checklist Rápido Antes de Concluir Qualquer Spec ou Código

- [ ] Consultei a documentação original e verifiquei se todos os links diretos respondem e sustentam a revisão citada?
- [ ] Classifiquei todas as fontes com os 8 tokens oficiais (`PRIMARY_OFFICIAL` .. `UNKNOWN`) e incluí tabela de evidência por afirmação?
- [ ] Separei equipamentos físicos distintos em Specs próprias e classifiquei algoritmos genéricos como `modulo-dsp`?
- [ ] Classifiquei todos os `CA-n` com os 6 tokens oficiais (`DOCUMENTED` .. `PENDING`) e separei `ENGINEERING_TARGET` de características físicas?
- [ ] Especifiquei condições completas em cada CA numérico com **níveis realistas de guitarra** (`-24` a `-12 dBFS`, palm mute, ruído real `-50` a `-45 dBFS`), não apenas senoides sintéticas de amplitude `1.0`?
- [ ] Validei gain staging de ponta a ponta (entrada Hi-Z/Line-In, sidechain do gate na mesma escala, headroom real de drive/boost pré-amp, curvas/tapers reais de potenciômetros, equilíbrio graves/médios/agudos com gabinete ativo)?
- [ ] Verifiquei todas as regras de `rust-best-practices` (zero UB/data race, `// SAFETY:`, zero-alloc/zero-lock RT, zero código morto/decorativo, Clippy limpo)?
- [ ] Incluí a Matriz de Rastreabilidade completa com os 7 status oficiais (`CONFIRMED` .. `BLOCKED`), apontando apenas para arquivos de teste **reais e existentes**, mantendo bloqueados os requisitos sem fonte suficiente?
