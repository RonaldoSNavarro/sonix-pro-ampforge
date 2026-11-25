---
name: spec-driven-development
description: Conduz o desenvolvimento por Spec Driven Development (SDD) com rigor técnico e proteções anti-superficialidade — toda mudança de comportamento nasce de uma spec numerada em `specs/NNN-nome.md`, com critérios de aceite verificáveis e não-tautológicos (CA-n), testes no caminho real de produção com dados realistas, prova de falha por mutação/red real, invariantes declaradas, fora de escopo explícito e rastreabilidade CA→teste auditável. Use quando o usuário pedir para criar/escrever/atualizar uma spec, iniciar uma feature nova, planejar uma mudança de comportamento, registrar dívida técnica ou achados de code review, adotar/instalar o padrão SDD num projeto, ou quando for implementar algo num repositório que já tenha o diretório `specs/`. Acione também quando o pedido for "vamos implementar X" num projeto que segue SDD — a spec vem antes do código.
---

# Spec Driven Development (SDD)

Padrão de desenvolvimento em que **nenhuma mudança de comportamento começa pelo código**. Começa
por um documento curto, numerado e versionado junto do código, que declara o que o sistema passará
a fazer e — principalmente — **como saber que ele faz**.

Extraído de um projeto real com 112 specs em produção. Funciona com ou sem TDD estrito, em
qualquer linguagem.

## Por que isto existe (leia antes de aplicar mecanicamente)

Três problemas concretos que a spec resolve, e que nenhum commit resolve sozinho:

1. **"Por que isto é assim?" seis meses depois.** O código diz *o quê*. O `git log` diz *quando*.
   A spec é o único lugar que guarda o *porquê* — a evidência, a alternativa descartada, o custo
   que se aceitou pagar. Sem ela, a próxima pessoa (ou o próximo agente) "conserta" a decisão.
2. **Escopo que escorre.** Uma seção obrigatória de *Fora de escopo* transforma "isso a gente vê
   depois" numa afirmação escrita, com motivo. O que não está escrito volta em toda revisão.
3. **"Está pronto" sem régua.** Critério de aceite numerado é uma afirmação que alguém pode
   verificar e reprovar. "Melhorar a performance da busca" não é; "a busca de 2+ palavras devolve
   resultado quando o AND estrito devolveria zero" é.

O ganho colateral, que na prática vira o maior: **o índice de specs é o changelog do projeto** —
o único documento que conta a história do sistema em ordem, com status e ponteiro para os testes.

## Quando acionar

- O usuário pede uma feature, correção de comportamento, mudança de contrato/rota/flag/tela.
- O usuário pede explicitamente uma spec, ou pede para adotar SDD num projeto.
- Um code review produziu achados que não cabem no PR atual — viram spec, não TODO solto.
- Existe `specs/` no repositório: **antes de editar código, verifique se há spec para o que foi
  pedido**. Se não há, a spec é o primeiro entregável.

**Quando NÃO acionar:** correção de typo, ajuste de formatação, atualização de dependência sem
mudança de comportamento, e qualquer coisa que não altere o que o sistema faz do ponto de vista de
quem o usa. Spec para não-mudança é burocracia, e burocracia mata o padrão pelo tédio.

## Instalar o padrão num projeto

Rode o instalador desta skill a partir da raiz do projeto alvo:

```bash
bash ~/.claude/skills/spec-driven-development/install.sh --dry-run   # mostra o que faria
bash ~/.claude/skills/spec-driven-development/install.sh             # instala
```

Ele cria `specs/README.md` (regras + índice), `specs/_template.md`, e **acrescenta ao
`CLAUDE.md`/`AGENTS.md` do projeto a seção que aponta para `specs/README.md`**. Esse último passo
não é decoração: sem ele o agente não abre `specs/` por conta própria, e o padrão não sobrevive à
primeira sessão nova.

É **idempotente** — nunca sobrescreve arquivo existente, e não acrescenta a seção duas vezes.

| Opção | Efeito |
|---|---|
| `--target DIR` | Projeto alvo (default: diretório atual). |
| `--lang L` | `python`, `go`, `node`, `ruby`, `java`, `csharp`, `generic`. Ajusta a **convenção de nomes de teste** no README. Detectado sozinho por `pyproject.toml`/`go.mod`/`package.json`/`Gemfile`/`pom.xml`/`*.csproj`. |
| `--tdd strict\|relaxed` | Regime de teste registrado no README (default `relaxed` — spec obrigatória, red→green não). |
| `--version-bump` | Inclui a regra de versão `0.<NNN>.<correção>` no fluxo. Só faça isso se o projeto publica um artefato versionado. |
| `--first-spec "Título"` | Já cria `specs/001-<slug>.md` preenchida no frontmatter e registrada no índice. Recusa (e diz o próximo número livre) se o projeto já tiver specs — numeração nunca é reaproveitada. |
| `--agent-file CLAUDE.md\|AGENTS.md\|none` | Onde fisgar o padrão. Default: o que já existir; se nenhum existir, cria `CLAUDE.md`. |
| `--dry-run` | Mostra o plano sem escrever nada. |

**Antes de rodar, decida duas coisas com o usuário** (as duas mudam o `specs/README.md` que fica
versionado no repo, e mudá-las depois exige reescrever o arquivo):

1. **Regime de teste** — TDD estrito por CA, ou testes obrigatórios só nos CAs de invariante?
2. **Versão amarrada à spec** — o projeto publica artefato versionado onde isso se paga?

Instalação manual, se preferir: copie `templates/` para `specs/` e substitua os tokens
`{{TDD_BLOCK}}`, `{{BUMP_BLOCK}}`, `{{CONV_BLOCK}}` e `{{INDEX_ROWS}}` no README.

Se o projeto já tem histórico, **não** escreva specs retroativas para tudo. Comece a numerar do
001 na primeira feature nova. Spec retroativa só se paga para a decisão que ainda gera dúvida.

## O ciclo de uma feature

1. **Spec.** `specs/NNN-slug.md` a partir do template. Numeração sequencial, **nunca reutilizada**
   (nem quando a spec é descartada — o número morre com ela).
2. **Registro no índice.** Linha em `specs/README.md`. Uma spec fora do índice é uma spec que
   ninguém encontra.
3. **Código.** Implemente o que a spec descreve — e só isso. Se durante a implementação aparecer
   comportamento não previsto, **volte e edite a spec** (ela ainda não foi mergeada; é barato).
4. **Verificação.** Cada `CA-n` é verificado antes do merge: por teste automatizado quando o
   critério toca invariante ou contrato público; por verificação manual **documentada no PR**
   quando o teste não paga o custo.
5. **Fechamento.** Status → `implementada`, seção *Testes que cobrem esta spec* preenchida,
   índice atualizado, versão bumpada (se o projeto usar a convenção abaixo).

Spec e implementação são revisadas **no mesmo PR**. Spec mergeada antes do código vira ficção;
código mergeado antes da spec derrota o padrão inteiro.

## Anatomia da spec

Use `templates/_template.md`. O que cada seção precisa entregar — e como ela falha:

| Seção | Entrega | Como falha na prática |
|---|---|---|
| **Objetivo** | Uma frase: que problema, para quem. | Vira resumo da implementação ("adicionar campo X"). O objetivo é o problema, não a solução. |
| **Contexto** | O *porquê agora*, com **evidência**: reprodução, medição, linha de código, relato de campo. | Vira opinião ("acho que ficaria melhor"). Contexto sem evidência não sustenta a decisão depois. |
| **Comportamento esperado** | Prosa do que o sistema passa a fazer: entradas, saídas, efeitos. | Vira lista de tarefas de implementação. Descreva o comportamento observável, não os arquivos que você vai tocar. |
| **Critérios de aceite** | `CA-n` verificáveis. Ver a régua abaixo. | Viram desejos. É o defeito mais comum e o mais caro. |
| **Fora de escopo** | O que a spec deliberadamente não cobre, **com motivo**. | Fica vazia. Seção vazia aqui é sinal de que o escopo não foi pensado. |
| **Invariantes tocadas** | Quais regras inegociáveis do projeto a feature encosta, e como as preserva. | É pulada em feature "pequena" — justamente onde a invariante quebra sem ninguém ver. |
| **Testes que cobrem esta spec** | Mapa `arquivo → CA-n`. Preenchido no fechamento. | Fica genérica ("tem testes"). Sem o mapa, não há rastreabilidade. |

### Contexto: traga a evidência, não a impressão

O que faz uma spec valer daqui a um ano é o contexto ter **prova**. Formas que funcionam:

- **Reprodução literal.** A entrada e a saída erradas, coladas:
  ```
  'Sintoma-chave=Aplicacao ja esta ativa'  ->  'Sintoma-chave=[REDACTED] ja esta ativa'
  ```
- **Medição.** "57.874 chars 'dentro' do orçamento de 60.000 viravam 75.367 no cliente."
- **Ponteiro de código.** "`config.py:184` — o default é `False`."
- **Tabela do que já existe** vs. **o que falta**, para não refazer o que está pronto.

Registre também **o que a implementação inicial errou e foi corrigido no review** — isso impede que
a próxima pessoa reintroduza o erro por parecer mais simples.

## A régua dos critérios de aceite

Um `CA-n` é uma **afirmação verificável no sistema pronto** — algo que se pode reprovar. Régua:
*consigo escrever o teste (ou o passo de verificação manual) lendo só esta linha — e esse teste **reprova** uma implementação rasa, quebrada ou feita só para enganar a métrica?*

| ❌ Desejo / CA Frouxo (Enganável) | ✅ Critério Assertivo e Não-Tautológico |
|---|---|
| A busca fica mais relevante. | Query de 2+ palavras devolve resultado quando o AND estrito devolveria zero, e o estágio usado volta no campo `matched`. |
| Controle `treble` funciona (ou `rms_10 > rms_5`). | Em `bass=5, mid=5`, mover `treble` de `0 → 5 → 10` altera o ganho em `4 kHz` de forma monotônica na faixa esperada do esquema (ex.: Δ ≥ +12 dB entre 0 e 10, com ponto médio coerente com o taper real do potenciômetro — sem ficar morto até 7), medido na cadeia completa de produção. |
| Modelos de amplificador soam diferentes (`max_band_diff >= 2 dB`). | Com entrada realista de guitarra (`-18 dBFS` RMS + transientes de palhetada), os 5 modelos calibrados em RMS igualado apresentam assinatura espectral, conteúdo harmônico (razão pares/ímpares e THD em faixa real) e preservação de ataque distintos e compatíveis com cada topologia, **com o gabinete padrão ativo**. |
| Sem alocação no callback (chamando `process()` e checando `is_finite()`). | Com alocador global de contagem ativo na thread RT, executar `process()` sob troca de parâmetros, presets e sinal dinâmico registra `allocs == 0` **e** `deallocs == 0`, e o teste falha quando uma alocação deliberada é injetada (red real). |
| Tratar erro de configuração. | URL malformada (sem esquema, com caminho, com espaço) é recusada no boot com mensagem explícita — não silenciosamente ignorada. |

Regras práticas:

- **Um comportamento por CA.** CA com "e" no meio quase sempre são dois.
- **Direção, magnitude e faixa física obrigatórias.** Nunca aceite apenas `a > b`, `diff > 1e-4`, `rms > 0.001` ou `HTTP 200` sem validar o efeito observável no sentido correto e na ordem de grandeza física/funcional esperada.
- **Condições realistas de operação.** Especifique explicitamente entrada, amplitude/nível real de uso (não valores artificiais que contornam defeitos de ganho!), taxa/ambiente, configuração padrão do usuário e método de medição.
- **Inclua o CA de não-regressão e o de discriminação negativa.** Não basta provar que funciona no caso feliz: prove que **não** ativa onde não deve, **não** vaza sinal indesejado e **não** destrói o comportamento em extremos (`0.0`, `5.0`, `10.0`).
- **Inclua o caso de erro.** Entrada malformada, dependência fora do ar, permissão ausente, valor fora da faixa, dispositivo desconectado.
- **Nomeie o observável.** O nome do campo, o código HTTP, a curva espectral, a atenuação em dB, o arquivo escrito — o CA precisa apontar para algo que existe fora da cabeça de quem escreveu.
- **Numeração estável.** `CA-7` continua sendo `CA-7` para sempre. CA acrescentado depois entra como `CA-9b` ou no fim da lista; CA removido é marcado como revertido, não renumerado — testes e PRs já citam o número.
- 5 a 15 CAs é a faixa saudável. Menos que 3, a spec provavelmente não precisava existir; mais de 20, são duas specs.

---

## Disciplina Anti-Superficialidade — Proibição de "Código Feito Só Para Passar nos Testes"

> **Lição dura de engenharia:** Testes verdes com código fraco são **pior** do que testes vermelhos — criam falsa sensação de segurança enquanto o sistema falha na mão do usuário real. O objetivo do SDD + TDD **nunca é fazer o teste ficar verde pelo caminho mais curto**; é construir uma implementação tecnicamente correta, coerente com o domínio real, cujos testes tornam impossível aprovar uma gambiarra.

### As 7 Regras Anti-Gaming Obrigatórias

1. **Resolva o problema real de domínio, nunca o mínimo para o `assert` passar.**
   - **Proibido** remendar um defeito estrutural com constantes mágicas, `if` ad-hoc para o sinal do teste, ou aproximações falsas de topologia (ex.: simular um tone stack James/Baxandall ou Vox na matriz FMV; trocar uma rede RC de cátodo por um ganho escalar plano que inunda graves).
   - Antes de codificar, entenda **a física / matemática / arquitetura do mecanismo real** e implemente o modelo correto de ponta a ponta.
2. **Teste o caminho REAL de produção, nunca um módulo órfão ou desligado.**
   - **Proibido** validar uma biblioteca/componente que a cadeia de produção não chama (ex.: testar um `HalfBandOversampler` de 96 dB enquanto a produção usa outro filtro; testar um bloco isolado contornando o hi-cut, o limiter interno, o ganho de entrada ou o gabinete padrão).
   - Toda spec que afeta comportamento observável exige pelo menos um teste **ponta-a-ponta na cadeia real de produção** (ex.: `RealtimeDspChain` + `api::dispatch` + configuração default real, não apenas um struct isolado).
3. **Proibição de testes tautológicos, circulares ou de fachada.**
   São explicitamente **inválidos** e devem ser rejeitados no code review:
   - **Teste circular:** comparar a saída do sistema contra uma função analítica/getter do *próprio código sob teste* sem oráculo independente (fórmula da literatura, valores tabelados de referência ou propriedade física externa).
   - **Teste de HTTP 200 / getter de struct:** enviar um parâmetro e conferir apenas `status == 200` ou `field == value` para "provar" que um controle funciona. Todo controle deve provar efeito mensurável no comportamento real.
   - **Limiar matematicamente frouxo:** ex. `corr > 0.3` para provar caminho limpo quando hard-clipping puro já tem correlação `0.90`; `rms > 0.001` para provar diferenciação entre modelos; `waveform_diff_db > -22 dB` sem direção espectral; aceitar `8%` de distorção num canal limpo; testar ataque apenas no modelo limpo descartando os primeiros samples de ataque!
   - **Teste de invariante sem instrumento de medida:** ex. teste de "zero-alloc" sem `CountingAllocator` medindo `alloc` **e** `dealloc`; teste de "zero-lock" sem provar ausência de contenção.
   - **Entrada artificial que mascara defeito real:** ex. ajustar manualmente um controle para `8.165` no teste para esconder que o default `5.0` atenua `-8.5 dB`, ou usar DC constante `[0.05; 256]` em vez de sinal dinâmico real.
4. **Prova de Red Real e Teste de Mutação Mental/Prática.**
   - Para cada teste de contrato, pergunte e valide: *"Se eu substituir a implementação por passthrough (`y = x`), inverter o sinal do controle, quebrar o taper, alocar um `Vec` no callback, ou desligar o efeito, este teste **quebra com certeza**?"*
   - Se a resposta for "não" ou "talvez", o teste é fraco e **deve ser endurecido antes de escrever a implementação**.
5. **Verificação de Ponta a Ponta do Fluxo de Dados, Unidades e Gain Staging.**
   - Ao implementar qualquer módulo, audite obrigatoriamente **suas fronteiras**: o que entra (escala, unidades físicas, pré/pós-boost, modo Line vs Instrument), o que sai (headroom, saturação interna, limiter, silenciamento em `0.0` e `standby`) e se algum estágio vizinho anula, duplica ou distorce o efeito (ex.: proximidade `+8 dB` + ressonância `+5 dB` + voicing `+4.5 dB` empilhando graves sem `low_cut`).
6. **Zero Código Morto, Zero Parâmetro Decorativo, Zero Referência Fantasma.**
   - Todo campo gravado na struct/estado deve ser efetivamente lido e aplicado no processamento; toda chave de parâmetro na spec deve existir com nome idêntico no catálogo canônico do código; todo arquivo de teste citado na spec deve **existir de fato no disco** e realmente exercitar os CAs atribuídos.
7. **Portão de Auditoria Técnica Adversarial (Obrigatório Antes de `status: implementada`).**
   Antes de concluir uma implementação, execute uma **auto-revisão adversarial independente** (não confie apenas no "cargo test verde"):
   - Leia o diff completo como um revisor cético procurando falhas de lógica, unidades trocadas, tapers irreais, casos de borda (`0.0`, `5.0`, `10.0`, bypass, troca de dispositivo, carga pesada) e efeitos colaterais em outros módulos.
   - Simule mentalmente (ou em teste de cenário realista) o **uso real pelo usuário final** (ex.: plugar guitarra real na interface ou Line-In, trocar de preset/amp, fazer palm mute com ruído real, girar cada knob de 0 a 10).
   - Se encontrar qualquer ponto onde "passa no teste, mas na prática soa/funciona mal", **corrija a spec, endureça o teste e corrija o código** antes de encerrar.

## Rastreabilidade CA → teste

A convenção que faz a rastreabilidade funcionar sem ferramenta:

- Arquivo de teste espelha o número da spec: `tests/contract/test_NNN_<slug>.py`,
  `test/contract/NNN_<slug>_test.go`, `spec/contract/NNN_<slug>_spec.rb` — o que a linguagem usar.
- Cada caso cita o critério no **nome**: `test_ca4_https_liga_cookie_secure`.
- **Docstring/comentário do arquivo lista todos os CAs cobertos**, em uma linha cada. É o resumo
  que a próxima pessoa lê antes de mexer.
- A spec, no fechamento, lista o caminho inverso: `arquivo → CA-1 … CA-9`.

Três tipos de teste, se o projeto os separar por marcador: **unit** (lógica pura), **integration**
(sobe o sistema), **contract** (verifica os CA-n de uma spec). O teste de contrato é o que morre
junto com a spec — os outros sobrevivem à refatoração.

## Numeração, status e versão

**Status:** `rascunho` → `aprovada` → `implementada`. Use também estados honestos quando for o
caso: `implementada (CA-N revertida)`, `descartada`. O estado que mente é pior que o estado feio.

**Numeração:** sequencial, monotônica, nunca reutilizada. Ordem de *criação*, não de implementação
— specs saem de ordem o tempo todo e isso é normal.

**Versão (opcional, mas recomendada):** amarre a versão do artefato ao número da spec —
`0.<NNN>.<correção>`. O ganho é operacional: `/status`, `--version` e o header da resposta passam a
dizer **qual spec está no ar**, o que transforma "está deployado?" numa pergunta com resposta.
Duas regras que só aparecem depois de doer:

- **A versão nunca anda para trás.** Spec implementada fora de ordem (a 104 depois da 110)
  incrementa o último dígito da versão corrente (`0.110.1`), não usa o próprio número. Qual spec
  entrou fica registrado no índice e na mensagem do commit — que é onde se procura isso.
- Se a linguagem tiver instalação editável/cache de metadata, **reinstale depois do bump**, senão o
  processo continua reportando a versão anterior.

## Casos especiais que o padrão precisa cobrir

**Achados de code review.** Viram uma spec própria ("correções pós-review"), com um CA por achado.
Assim o achado ganha número, teste e rastro — em vez de virar TODO que ninguém executa. Vale
registrar na spec **de qual review veio** e a data.

**Dívida técnica.** Dívida vira spec **com data e régua de remoção**. Alias de compatibilidade sem
data de remoção vira patrimônio: quem chega novo lê as duas formas na documentação e conclui que
ambas são legítimas. A spec da remoção deve dizer **como medir** que ninguém usa mais (log de
auditoria, telemetria) antes de remover — e separar o que **nunca** sai (leitores de dado já
gravado em disco costumam ser compatibilidade permanente, não dívida).

**Spec descartada.** Se a feature é abandonada, marque `descartada` com o motivo (custo, prioridade,
alternativa escolhida) e **mantenha o arquivo**. O número não é reaproveitado. Spec descartada com
motivo escrito é o que impede a mesma ideia de voltar a cada trimestre.

**CA reprovado na verificação.** Se um critério não passou e foi revertido, registre no status e
nas notas da spec. Índice com "Implementada (CA-18 revertida)" é informação; índice com
"Implementada" mentindo é armadilha.

**Mudança de comportamento já especificado.** Não reescreva a spec antiga em silêncio. Ou edite
declarando a mudança, ou escreva uma spec nova que **supersede** a anterior e diga isso nas duas.

## SDD com ou sem TDD

São ortogonais. A spec antes do código é inegociável; o ciclo red→green é uma escolha de projeto:

- **SDD + TDD estrito:** cada CA vira teste que falha antes da implementação. Custo alto, cabe bem
  em núcleo de domínio e nas primeiras dezenas de specs, enquanto os contratos assentam.
- **SDD sem TDD estrito:** testes escritos junto ou depois, obrigatórios só para CAs que tocam
  invariantes e contratos públicos; o resto é verificação manual documentada no PR.

Um projeto pode transicionar do primeiro para o segundo — registre **em qual spec** a transição
aconteceu, no `specs/README.md`, senão o histórico fica incoerente para quem lê de trás para frente.

## Antipadrões

- **Implementação feita "só para passar no teste" (reward hacking técnico).** Dá para reconhecer: o código usa atalhos, constantes arbitrárias, ou cobre apenas o ponto exato do `assert` enquanto quebra no uso real, nos extremos (`0.0`/`10.0`), com entrada realista ou integrado ao restante do sistema.
- **Teste tautológico, circular ou de fachada.** Conferir só `HTTP 200`, `rms > 0.001`, `a > b` sem magnitude/direção, comparar código consigo mesmo, testar "zero-alloc" sem alocador de contagem, ou validar um módulo que a produção nem usa.
- **Spec escrita depois do código, para justificá-lo.** Dá para reconhecer: os CAs descrevem a implementação ("a função `X` recebe o parâmetro `Y`") em vez do comportamento.
- **CA que é passo de implementação.** "Criar o módulo `foo.py`" não é critério de aceite — nada nele pode ser reprovado sem olhar a estrutura de arquivos.
- **Spec desalinhada dos contratos reais do repositório.** Citar traits, chaves de parâmetro ou arquivos de teste que não existem no codebase.
- **Fora de escopo vazio.**
- **Índice desatualizado.** Assim que o índice para de refletir a realidade, ele deixa de ser consultado, e o padrão morre em uma semana.
- **Spec gigante.** Mais de ~20 CAs, ou objetivo com dois "e" — divida. Duas specs pequenas se implementam, revisam e revertem; uma grande fica meio pronta.
- **Invariantes tocadas em branco por padrão.** Se o projeto tem invariantes declaradas, a seção ou lista quais são tocadas, ou afirma explicitamente que nenhuma é.

## Checklist antes do merge

- [ ] Spec existe, numerada, a partir do template, está no índice e usa nomes reais de traits/parâmetros/arquivos do repositório.
- [ ] Objetivo é o problema; contexto tem evidência verificável.
- [ ] Todo `CA-n` é reprovável, tem direção/magnitude/faixa real e condição de entrada realista — nenhum é desejo, tautologia ou passo de implementação.
- [ ] Há CA de não-regressão, CA de caso de erro e verificação nos extremos (`min`/`default`/`max`) e na cadeia real de produção (ponta a ponta).
- [ ] Cada teste de contrato passou pela prova de **Red Real / Mutação** (quebra de fato se a implementação for substituída por stub, invertida ou violar a invariante).
- [ ] Auditoria técnica adversarial executada no código: lógica sólida de domínio, unidades/escalas consistentes, zero código morto ou parâmetro ignorado, zero gambiarra feita só para enganar o teste.
- [ ] Fora de escopo preenchido, com motivo.
- [ ] Invariantes tocadas listadas (ou negadas explicitamente) e verificadas por **régua medida** (não inspeção visual).
- [ ] CAs de invariante/contrato têm teste automatizado citando o número no nome, em arquivos que **existem de fato no disco**.
- [ ] Seção *Testes que cobrem esta spec* mapeia arquivo existente → CAs realmente exercitados.
- [ ] CAs verificados manualmente estão documentados no PR.
- [ ] Status atualizado; versão bumpada; documentação de comportamento visível (rota, flag, tela) atualizada **no mesmo PR**.

## Arquivos desta skill

- `install.sh` — instalador idempotente do padrão num projeto (ver acima).
- `templates/_template.md` — template de spec, pronto para copiar em `specs/`.
- `templates/README-specs.md` — `specs/README.md` inicial, com tokens que o instalador substitui.
- `referencias/exemplos-de-ca.md` — CAs reais, bons e ruins, comentados; e o esqueleto do teste de
  contrato correspondente.
