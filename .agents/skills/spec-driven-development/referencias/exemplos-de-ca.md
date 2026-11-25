# Critérios de aceite — exemplos comentados

Exemplos extraídos de specs reais (projeto de origem: um servidor de memória incremental em
Python/FastAPI, 112 specs). Levemente genericizados. O que importa aqui não é o domínio — é a
**forma** do critério.

---

## 1. Desejo × critério

| ❌ Desejo | ✅ Critério | O que mudou |
|---|---|---|
| A busca fica mais relevante. | Query de 2+ palavras devolve resultado quando o AND estrito devolveria zero, e o estágio usado (`and`/`or`) volta no campo `matched`. | Nomeou a condição de entrada, o resultado esperado e o **observável** que prova qual caminho rodou. |
| Cookies mais seguros em produção. | URL pública em `https://` faz `cookie_secure` valer `True` sem configuração adicional; um `False` explícito continua respeitado. | Virou uma regra com precedência declarada — dá para reprovar nos dois sentidos. |
| Validar a configuração. | URL malformada (sem esquema, com caminho, com espaço) é recusada **no boot**, com mensagem explícita — não silenciosamente ignorada. | Enumerou as entradas inválidas, o momento da falha e proibiu o modo de falha silencioso. |
| Não quebrar quem já usa. | Sem a variável definida, tudo se comporta exatamente como hoje — nenhuma regressão para quem roda local ou em rede interna. | O CA de não-regressão, que é o que mais salva em refatoração. |
| Melhorar o sanitizador. | `chave` isolada deixa de marcar segredo (`Palavra-chave=jobs` sobrevive íntegro), enquanto `chave privada`, `chave secreta` e `chave de acesso` continuam sendo redigidas. | Trouxe o caso concreto do falso positivo **e** o caso que não pode afrouxar. |
| Registrar quem fez a mudança. | O evento gravado contém o autor autenticado e o motivo sanitizado; motivo ausente recusa a operação com 400. | Um comportamento, um erro, um observável. |

---

## 2. Padrões de CA que vale ter em quase toda spec

**Não-regressão.**
> **CA-7:** sem `<nova flag>`, tudo se comporta exatamente como hoje.

**Caso de erro / entrada malformada.**
> **CA-6:** `<entrada>` malformada é recusada no boot com mensagem explícita — não ignorada.

**Precedência explícita.** Toda vez que duas fontes de configuração podem discordar:
> **CA-3:** os subcomandos usam a URL pública quando ela existe; `--host/--port` explícitos
> continuam vencendo quando passados.

**Permissão / isolamento.**
> **CA-4:** usuário escopado ao projeto `p1` recebe 403 ao operar `p2`, e a resposta não vaza o
> conteúdo do projeto alheio.

**Efeito observável em disco/wire.** Prefira apontar para o artefato:
> **CA-2:** o arquivo de configuração gerado aponta para `<base>/mcp/` — com a barra final, que o
> cliente exige.

**Limite e orçamento.** Diga o que acontece **ao cortar**:
> **CA-14:** o corte por orçamento descarta o item inteiro; nenhuma linha é entregue decapitada.

---

## 3. CAs que não são CAs

- ❌ **CA:** criar o módulo `store/vec_store.py`. → passo de implementação; nada nele é reprovável
  sem inspecionar a estrutura de arquivos.
- ❌ **CA:** a função `search()` recebe o parâmetro `mode`. → descreve a assinatura, não o
  comportamento. Reescreva pelo efeito: *"com `mode=or`, a busca…"*.
- ❌ **CA:** o código fica mais legível / mais rápido / mais robusto. → sem régua. Se é performance,
  dê o número: *"a busca semântica deixa de fazer N+1 chamadas ao provedor: 1 chamada por lote"*.
- ❌ **CA:** cobrir com testes. → teste é o meio de verificar o CA, não o CA.
- ❌ **CA:** o hook captura o evento **e** devolve o contexto **e** respeita o orçamento. → são
  três CAs. Cada "e" que junta comportamentos independentes é uma linha a mais.

---

## 4. Do CA ao teste — esqueleto de contrato

O arquivo de teste espelha o número da spec e abre listando os CAs. Exemplo em Python/pytest
(traduza a forma para a linguagem do projeto — o que não muda é o cabeçalho-mapa e o número no
nome de cada caso):

```python
"""Testes de contrato da spec 111 — URL pública, TLS no proxy e cookies seguros.

Cada teste cita o critério de aceite (CA-n).

- CA-1: `install-hooks` grava o comando apontando para a URL pública (sem porta implícita).
- CA-2: a configuração do cliente aponta para `<base>/mcp/` — com a barra final.
- CA-3: os subcomandos usam a URL pública; `--host/--port` explícitos vencem.
- CA-4: `https://` faz `cookie_secure` valer True; `False` explícito é respeitado.
- CA-6: URL malformada é recusada no boot, com mensagem explícita.
- CA-7: sem a URL pública, nada muda (nenhuma regressão).
"""

import pytest

pytestmark = pytest.mark.contract


def test_ca4_https_liga_cookie_secure() -> None:
    """CA-4: `public_base_url` em https faz `cookie_secure` valer True."""
    ...


def test_ca4_false_explicito_e_respeitado() -> None:
    """CA-4 (segundo sentido): `cookie_secure=False` explícito vence o esquema."""
    ...


def test_ca6_url_malformada_recusada_no_boot() -> None:
    """CA-6: esquema ausente, caminho ou espaço → erro de validação explícito."""
    ...
```

Observações que fazem diferença na manutenção:

- **Um CA pode ter mais de um teste** (os dois sentidos de uma precedência, por exemplo). Todos
  citam o mesmo número.
- **Nem todo teste precisa citar CA.** Testes das peças novas (comportamento fora do eixo dos
  critérios) vivem em `tests/unit/` e são declarados assim na spec — isso evita inflar a lista de
  CAs só para justificar um teste.
- No fechamento, a spec recebe o mapa inverso:

  ```markdown
  ## Testes que cobrem esta spec
  - `tests/contract/test_111_deploy_dominio.py` → CA-1 … CA-4, CA-6 … CA-9
  - `tests/unit/test_urls.py` → construção da URL (peças novas, fora do eixo dos CAs)
  - CA-5 verificado manualmente: log de auditoria inspecionado atrás do proxy (print no PR #142)
  ```

---

## 5. Testes fracos / tautológicos × testes assertivos (Anti-Gaming)

Os exemplos abaixo mostram padrões reais em que uma suíte passa **100% verde** mesmo com código raso, quebrado ou inoperante — e como reescrevê-los para impedir implementações feitas "só para passar no teste":

| ❌ Teste Fraco / Enganável | Por que falha em pegar bugs reais | ✅ Teste Assertivo / Não-Enganável |
|---|---|---|
| `assert_eq!(post("/api/params", {"knob": 10}), 200)` | Prova apenas que o parser aceitou a chave; um parâmetro gravado em campo morto ou sem efeito passa verde. | Envia o parâmetro pela API real, processa sinal na cadeia de produção e mede mudança direcional e de magnitude na banda/envelope esperado (ex.: Δ ganho em `4 kHz ≥ +10 dB` com `treble: 0→10`). |
| `assert!(rms > 0.001)` para "provar" que 5 modelos diferem | Cinco passthroughs idênticos (`y = x`) passam com 100% de sucesso. | Compara pares de modelos em RMS igualado com sinal realista (incluindo transiente e sustentação) e exige distância espectral em múltiplas bandas, assinatura harmônica (THD / razão pares-ímpares) e resposta dinâmica distintas. |
| `assert!(corr(in, out) > 0.3)` para provar caminho limpo num overdrive | Onda quadrada de hard-clipping puro (`sign(sin(x))`, **zero** caminho limpo) já possui correlação `0.9003 >> 0.3`! | Mede diretamente a resposta em baixa frequência (`< 200 Hz`, abaixo do HPF do ramo saturado) em ganho mínimo e máximo e compara o sinal limpo residual contra o oráculo analítico do circuito (`±1.5 dB`). |
| Comparar `filter.process()` com `filter_analytical_db()` importado do **mesmo** módulo | Erro de topologia ou de coeficientes nas duas funções se cancela e dá erro `0.0 dB` (teste circular). | Usa oráculo independente (valores tabelados de esquema/artigo científico ou equação nodal independente) e testa pontos múltiplos (`min`, `mid`, `max` e interação entre controles). |
| Testar "zero-alloc" chamando `process()` e checando `out[0].is_finite()` | Não mede alocações; alocar `100 MB` por bloco continua passando verde. | Instala `CountingAllocator` ativo na thread de execução, verifica `allocs == 0 && deallocs == 0` sob sinal dinâmico e trocas reais de parâmetros, e confirma red real injetando alocação temporária. |
| Testar um `Oversampler` isolado de 96 dB que a produção não usa | A produção pode ter aliasing severo usando outro filtro enquanto o teste da biblioteca ociosa passa verde. | Mede aliasing diretamente na saída da cadeia real de produção (`RealtimeDspChain` com amp/drive ativos em alto ganho), injetando varredura senoidal de alta frequência. |
| Ajustar controle no teste para `level = 8.165` ou desligar o filtro `hicut` padrão | Mascara que no padrão real (`level = 5.0` ou `cab` ligado) o pedal atenua `-8.5 dB` ou o timbre fica abafado. | Executa sempre um cenário na **configuração padrão real do usuário** (default de boot/preset + nível realista de guitarra `-18` a `-12 dBFS` + piso de ruído realista `-50 dBFS`). |
| Gerar transiente de ataque nos samples `0..960` e fatiar `out[4096..]` na medição | Descarta 100% do ataque que o teste pretendia medir! | Mede explicitamente a janela do transiente (fator de crista, tempo de subida e separação entre batidas de palm-mute) **nos amps high-gain com gabinete ativo**, não apenas no canal limpo. |

---

## 6. Numeração de CA ao longo do tempo

Números de CA são **citados por testes, PRs e commits** — tratam-se como identificadores públicos.

- CA acrescentado durante o review entra no **fim** da lista, ou como `CA-9b` quando pertence
  logicamente a um existente.
- CA removido ou reprovado **não é renumerado**: marque no texto (`~~CA-18~~ revertida — ver Notas`)
  e registre o motivo. O status da spec no índice acompanha:
  `Implementada (CA-18 revertida)`.
- Spec que supersede outra cita o número da anterior, e a anterior ganha a nota do reverso.
