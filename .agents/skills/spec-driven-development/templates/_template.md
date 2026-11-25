---
spec: NNN
titulo: <título curto — o problema, não a solução>
status: rascunho | aprovada | implementada | descartada
atualizado: AAAA-MM-DD
---
# NNN — <Título>

## Objetivo
<Uma frase: que problema esta mudança resolve e para quem. Não descreva a implementação.>

## Contexto
<Por que agora. Traga EVIDÊNCIA, não impressão: reprodução literal do defeito, medição,
ponteiro de código (`arquivo.ext:linha`), relato de campo, restrição externa.

Se parte do caminho já está pronta, registre — evita que seja refeito.
Se há alternativa descartada, diga qual e por quê.>

## Comportamento esperado
<Prosa do que o sistema passa a fazer: entradas, saídas, efeitos colaterais, casos de borda.
Descreva o observável, não os arquivos que serão tocados.>

## Critérios de aceite
> Cada item é uma afirmação verificável, não-tautológica e reprovável — especificando entrada realista, direção, magnitude/faixa esperada e observável no caminho real de produção. Proibido `a > b` sem magnitude, teste circular contra o próprio código ou checagem apenas de `HTTP 200` / getter de struct.

- **CA-1:** <afirmação verificável com entrada realista, direção, faixa numérica/comportamental e observável>
- **CA-2:** <afirmação verificável nos extremos min / default / max e integração ponta-a-ponta>
- **CA-3:** <caso de erro / discriminação negativa: entrada malformada, dependência fora, limite violado ou condição em que o efeito NÃO pode disparar>
- **CA-4:** <não-regressão: comportamento pré-existente preservado em condições reais de uso>

## Estratégia de validação anti-superficialidade
- **Oráculo independente:** <qual fonte externa, fórmula da literatura, medição física ou contrato define o resultado esperado — nunca o próprio código sob teste>
- **Caminho real exercitado:** <como o teste exercita a cadeia de produção completa com entradas realistas, e não apenas uma unidade isolada ou artificial>
- **Prova de reprovação (Red Real / Mutação):** <qual mutação simples no código (passthrough, sinal invertido, alocação injetada, constante errada) garante que os testes falham>

## Fora de escopo
<O que esta spec deliberadamente NÃO cobre — e por quê. Se houver spec candidata para
retomar o assunto, cite o número.>

## Invariantes tocadas
<Quais regras inegociáveis do projeto esta mudança encosta e como as preserva (com régua medida).
Se nenhuma, afirme explicitamente: "Nenhuma — isto é configuração de borda.">

## Testes que cobrem esta spec
<Preenchido no fechamento — cite APENAS arquivos que existem no repositório e realmente exercitam cada CA.>
- `<caminho/real/do/teste>` → CA-1, CA-2, CA-4
- CA-3 verificado manualmente: <como, e onde está registrado>

## Auditoria técnica pós-implementação
<Preenchido antes de marcar `status: implementada` — revisão adversarial confirmando solidez de lógica, unidades/escalas entre blocos, ausência de código morto/parâmetros ignorados e comportamento coerente no uso real pelo usuário.>

## Notas
<Opcional: CA revertido e por quê; correções aplicadas no code review; decisões que a
implementação precisou tomar e que a spec não previa.>
