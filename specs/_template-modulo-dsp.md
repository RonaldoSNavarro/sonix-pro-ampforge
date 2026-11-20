---
spec: NNN
titulo: <título curto e objetivo do módulo/equipamento>
status: rascunho
tipo: modulo-dsp | amplificador | pedal | rack | componente-compartilhado | arquitetura-dsp
atualizado: AAAA-MM-DD
dependencias: [053]
---
# NNN — <Título do Módulo ou Equipamento>

## 1. Identificação e objetivo
- **Identificador estável (`model_id` / `module_id`):** `<id>`
- **Revisão (`model_revision`):** `<revisao-exata>`
- **Categoria:** `<arquitetura-dsp | amplificador | pedal | rack | modulo-dsp | componente-compartilhado>`
- **Natureza do módulo:** `<Equipamento físico real | Módulo algorítmico / síntese DSP | Infraestrutura compartilhada>`
- **Objetivo:** <Uma frase clara: qual equipamento/componente é especificado, que problema técnico resolve e qual contrato estabelece.>

## 2. Escopo
<Lista objetiva do que está coberto por esta spec.>

## 3. Fora de escopo
<O que esta spec deliberadamente NÃO cobre, com motivo e spec candidata quando existir.>

## 4. Equipamento ou componente de referência
- **Fabricante / Origem:** <fabricante real ou algoritmo de referência>
- **Modelo exato:** <modelo>
- **Período / Ano:** <período ou N/A>
- **Papel na cadeia de sinal:** <posição típica e interação com blocos vizinhos>

## 5. Revisão e variantes
- **Revisão de referência selecionada:** <revisão exata>
- **Variantes históricas relevantes (não misturadas):** <diferenças documentadas>
- **Pendências de confirmação de revisão:** <nenhuma ou lacuna explícita bloqueante>

## 6. Fontes técnicas e evidência por informação
> **Classificação obrigatória de fontes (`§4.1`):**
> `PRIMARY_OFFICIAL` · `PRIMARY_COMPONENT` · `SECONDARY_TECHNICAL` · `ACADEMIC` · `MEASUREMENT` · `INTERNAL_REFERENCE` · `INFERENCE` · `UNKNOWN`.
> *(Nota: `INTERNAL_REFERENCE` comprova apenas requisito/código interno do AMPFORGE, nunca a fidelidade elétrica ao hardware real.)*

### 6.1 Catálogo de fontes verificadas
| ID | Fonte | Fabricante / Modelo / Revisão | Página / Folha / Seção | URL / Caminho direto | Consulta | Classificação | Limitações da fonte |
|---|---|---|---|---|---|---|---|
| F1 | ... | ... | ... | ... | AAAA-MM-DD | `PRIMARY_OFFICIAL` | ... |

### 6.2 Evidência por afirmação técnica
| Informação / Parâmetro de Circuito | Fonte (ID) | Revisão | Página / Folha / Seção | Classificação | Status documental |
|---|---|---|---|---|---|
| ... | F1 | ... | Folha 1 / p. X | `PRIMARY_OFFICIAL` | Confirmado / Pendente |

## 7. Fundamentação elétrica ou matemática
<Topologia do circuito real (ou formulação matemática para módulos algorítmicos), entradas/jacks, válvulas/semicondutores, estágios, resistores/capacitores com revisão citada, impedâncias de entrada/saída, alimentação/bias/NFB/transformador quando aplicável. Registre explicitamente valores `UNKNOWN` / `PENDING` sem inventar.>

## 8. Comportamento esperado
<Comportamento elétrico, dinâmico e sonoro expresso em grandezas mensuráveis, separando explicitamente características físicas do equipamento de metas de engenharia do software (`ENGINEERING_TARGET`).>

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Origem no hardware (`Físico` vs `Extensão DSP`) | Efeito no circuito / algoritmo | Implementação DSP |
|---|---|---|---|---|---|---|---|---|---|
| ... | ... | ... | ... | ... | ... | ... | ... | ... | ... |

## 10. Contratos de integração
<Integração com o engine (Spec 053), descritores para UI (`controls.registry.ts`), validação atômica no backend (`api::dispatch`), formato de preset (`model_id` + `model_revision`), troca segura sem cliques.>

## 11. Arquitetura de implementação
- **Código atual mapeado (HEAD `d891389`):** `<arquivo:linhas>`
- **Problemas e dívidas encontrados no código atual:** <lista factual classificada>
- **Estrutura modular alvo:** <módulo independente, structs/traits, zero dependência cruzada entre modelos>

## 12. Estratégia DSP
<Estágios circuit-level vs behavioral vs híbridos, memória dinâmica, fator de oversampling e justificativa, filtros (TDF-II / SVF em `f64`), simplificações explícitas.>

## 13. Requisitos de tempo real
- Separação explícita entre **thread de controle (off-RT)** (`prepare()`, compilação, I/O) e **thread de áudio (RT)** (`process()`, `apply_param_rt()`, crossfade entre instâncias pré-alocadas, envio para `RetiredReaper`).
- Zero alocação e zero desalocação na heap na thread RT (validado por alocador global de contagem).
- Zero locks bloqueantes, zero I/O, zero logging síncrono, zero panic.
- Coeficientes invariantes à taxa de amostragem (`44,1 / 48 / 88,2 / 96 kHz`).

## 14. Gestão de estado
<Separação estrita entre estado DSP efêmero (filtros, atrasos, envelopes, sag) e estado serializável de preset; comportamento de `reset()`, `prepare()` e crossfade em troca de modelo/cena.>

## 15. Latência
<Latência algorítmica exata em amostras por taxa/oversampling, política de compensação (PDC) e impacto em caminhos paralelos.>

## 16. Consumo de CPU
<Orçamento alvo (`ENGINEERING_TARGET`) em `%` do deadline de 128 frames @ 48 kHz em build `--release` e método de medição (Criterion / `CpuMonitor`).>

## 17. Testes
- **Condições padronizadas de medição:** sinal de entrada, amplitude (dBFS), frequência/espectro, `sample_rate` (`44100/48000/96000 Hz`), `block_size` (`128` frames), configuração de parâmetros, método DFT/RMS, tolerância e fonte da tolerância.

## 18. Critérios de aceite
> **Classificação obrigatória de cada CA (`§9`):**
> `DOCUMENTED` · `MEASURED` · `ANALYTICAL` · `ENGINEERING_TARGET` · `REGRESSION` · `PENDING`.

- **CA-1 (`[CLASSIFICAÇÃO]` — Título):** <Condições completas de entrada, taxa, bloco, parâmetros, método de medição, valor esperado, tolerância e justificativa da tolerância.>
- ...

## 19. Dependências
- **Obrigatórias:** `Spec 053` → ...
- **Relacionadas / Legadas:** `Specs ...`

## 20. Limitações conhecidas
<Limitações explícitas da modelagem atual, lacunas documentais e restrições físicas.>

## 21. Riscos técnicos
<Riscos de aliasing, instabilidade numérica, CPU, divergência de revisão ou regressão.>

## 22. Matriz de rastreabilidade
> **Status obrigatórios (`§11`):**
> `CONFIRMED` · `SPECIFIED` · `IMPLEMENTED_UNVERIFIED` · `TESTED` · `MEASURED` · `PENDING` · `BLOCKED`.

| Requisito | Fonte | Spec/CA | Código atual | Teste atual | Status |
|---|---|---|---|---|---|
| ... | ... | `Spec NNN / CA-X` | `crates/...` | `contract_...` | `...` |
