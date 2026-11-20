---
spec: NNN
titulo: <título curto e objetivo do módulo/equipamento>
status: rascunho | aprovada | implementada | descartada
tipo: modulo-dsp | amplificador | pedal | componente-compartilhado | arquitetura-dsp
atualizado: AAAA-MM-DD
dependencias: [053]
---
# NNN — <Título do Módulo ou Equipamento>

## 1. Identificação e objetivo
- **Identificador estável (`model_id` / `module_id`):** `<id>`
- **Categoria:** `<arquitetura | amplificador | pedal | rack | componente>`
- **Objetivo:** <Uma frase clara: qual equipamento/componente é especificado, que problema técnico resolve e qual contrato estabelece.>

## 2. Escopo
<Lista objetiva do que está coberto por esta spec.>

## 3. Fora de escopo
<O que esta spec deliberadamente NÃO cobre, com motivo e spec candidata quando existir.>

## 4. Equipamento ou componente de referência
- **Fabricante:** <fabricante ou interno>
- **Modelo exato:** <modelo>
- **Período / Ano:** <período ou N/A>
- **Papel na cadeia de sinal:** <posição típica e interação com blocos vizinhos>

## 5. Revisão e variantes
- **Revisão de referência selecionada:** <revisão exata>
- **Variantes históricas relevantes (não misturadas):** <diferenças documentadas>
- **Pendências de confirmação de revisão:** <nenhuma ou lacuna explícita>

## 6. Fontes técnicas
> Classificação obrigatória: `[confirmado em documentação]` · `[confirmado por medição]` · `[fonte secundária]` · `[inferência técnica]` · `[aproximação DSP]` · `[desconhecido]`.

| ID | Fonte | Modelo / Revisão | Seção / Página | URL / Caminho | Consulta | Classificação | Dado sustentado |
|---|---|---|---|---|---|---|---|
| F1 | ... | ... | ... | ... | AAAA-MM-DD | `[...]` | ... |

## 7. Fundamentação elétrica
<Topologia do circuito real, entradas/jacks, válvulas/semicondutores, estágios, resistores/capacitores com revisão citada, impedâncias de entrada/saída, alimentação/bias/NFB/transformador quando aplicável. Registre explicitamente valores `[desconhecido]` sem inventar.>

## 8. Comportamento esperado
<Comportamento elétrico, dinâmico e sonoro expresso em grandezas mensuráveis: headroom, ganho por estágio, pontos de clipping, razão harmônica par/ímpar, compressão, resposta transitória, recuperação pós-sobrecarga, interação entre controles, resposta em frequência.>

## 9. Parâmetros e controles
| Nome | Chave estável (`backendKey`) | Tipo | Unidade | Faixa | Padrão | Taper / Escala | Efeito no circuito real | Implementação DSP |
|---|---|---|---|---|---|---|---|---|
| ... | ... | ... | ... | ... | ... | ... | ... | ... |

## 10. Contratos de integração
<Integração com o engine (Spec 053), descritores para UI (`controls.registry.ts`), validação atômica no backend (`api::dispatch`), formato de preset (modelo + revisão), troca segura sem cliques.>

## 11. Arquitetura de implementação
- **Código atual mapeado:** `<arquivo:linhas>`
- **Dívidas / acoplamentos identificados:** <lista factual>
- **Estrutura modular alvo:** <módulo/crate independente, structs/traits, zero dependência cruzada entre modelos>

## 12. Estratégia DSP
<Estágios circuit-level vs behavioral vs híbridos, memória dinâmica, fator de oversampling e justificativa, filtros (TDF-II / SVF em `f64`), simplificações explícitas.>

## 13. Requisitos de tempo real
- Zero alocação e zero desalocação na heap durante `process()` (validado por alocador de contagem RT).
- Zero locks bloqueantes, zero I/O, zero logging síncrono, zero panic.
- Coeficientes invariantes à taxa de amostragem (`44,1 / 48 / 88,2 / 96 kHz`).

## 14. Gestão de estado
<Separação estrita entre estado DSP efêmero (filtros, atrasos, envelopes, sag) e estado serializável de preset; comportamento de `reset()`, `prepare()` e crossfade em troca de modelo/cena.>

## 15. Latência
<Latência algorítmica em amostras por taxa/oversampling, política de compensação (PDC) e impacto em caminhos paralelos.>

## 16. Consumo de CPU
<Orçamento alvo em `%` do deadline de 128 frames @ 48 kHz em build `--release` e método de medição (Criterion / `CpuMonitor`).>

## 17. Testes
- **Sinais:** senoide, sweep logarítmico, impulso, multitone, ruído, degrau, transientes, DI real.
- **Métricas:** resposta em frequência, fase, RMS, crest factor, THD, IMD, espectro harmônico, alias residual, ataque/release, latência, CPU, estabilidade numérica, ruído/DC.
- **Comparação A/B:** condições de nível-match (`±0,1 dB`), mesma DI, mesma taxa e mesmo gabinete/IR.

## 18. Critérios de aceite
- **CA-1:** <afirmação verificável e reprovável>
- ...
- **CA-N (Erro):** <entrada inválida / NaN / Inf / sample rate fora de faixa>
- **CA-N+1 (Não-regressão):** <compatibilidade com specs anteriores>

## 19. Dependências
- **Obrigatórias:** `Spec 053` → ...
- **Relacionadas / Legadas:** `Specs ...`

## 20. Limitações conhecidas
<Limitações explícitas da modelagem atual e alvo.>

## 21. Riscos técnicos
<Riscos de aliasing, instabilidade numérica, CPU, divergência de revisão ou regressão.>

## 22. Referências e rastreabilidade
- **Specs anteriores relacionadas:** ...
- **Arquivos de código atuais:** ...
- **Testes de contrato (existentes e futuros):** ...
