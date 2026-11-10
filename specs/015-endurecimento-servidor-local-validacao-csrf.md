---
spec: 015
titulo: Servidor local aceita comandos de qualquer página web e um NaN silencia o motor até reiniciar
status: rascunho
atualizado: 2026-11-09
---
# 015 — Endurecimento do Servidor Local (CSRF, DNS rebinding, CORS, defesa contra NaN)

## Objetivo
Garantir que somente a janela do próprio SONIX PRO consiga alterar o motor de áudio — e que
nenhuma entrada, legítima ou hostil, consiga silenciá-lo permanentemente ou produzir volume
perigoso.

## Contexto
Origem: code review de 2026-11-09 (achados F-02 e F-03). Evidência reproduzida (crate descartável
fora do projeto, `RealtimeDspChain` real):

```
"inf".parse::<f32>() -> Ok(inf)      "NaN".parse::<f32>() -> Ok(NaN)
set_input_boost(inf)  -> process() -> set_input_boost(2.5) -> 50 blocos depois:  rms = NaN   (permanente)
set_gate_threshold_db(NaN) -> process()                                                  :  NaN
```

- `server.rs:93-101` converte `v.parse::<f32>()` e repassa direto a `set_input_boost`
  (`audio.rs:471`, só `max(0.1)`) e `set_gate_threshold_db` (`dynamics.rs:46`, `clamp` não filtra
  NaN) — um único `GET /api/params?boost=inf` envenena o envelope do gate e os estados IIR do amp
  (`NaN` realimenta a si mesmo) e o áudio fica mudo/ruidoso **até reiniciar o app**.
  `AmpFrontPanel` já sanitiza NaN (`amp.rs:68-74`, Spec 003 CA-7); gate, boost e cab não. A
  Spec 014 fecha a porta HTTP (validação de faixa e tipo, JSON não representa `NaN/inf`); esta
  spec fecha o DSP, para que nenhum outro caminho (host de plugin, preset, código futuro) consiga
  o mesmo efeito.
- `server.rs:116` responde `Access-Control-Allow-Origin: *` em todas as rotas. Qualquer página
  aberta no navegador do usuário pode ler `/api/state` e `/api/telemetry` (forma de onda do
  microfone) com `fetch()`; um domínio malicioso que resolva para `127.0.0.1` (DNS rebinding)
  passa, pois o servidor não confere `Host`. Mesmo com o `POST` JSON da Spec 014, sem checagem de
  `Origin` e de token um cliente que não seja navegador, ou um navegador via rebinding, muda o
  motor.
- `server.rs:44-129` atende em **uma** thread, serialmente; telemetria fica atrás de qualquer
  requisição lenta.
- A UI é servida do mesmo processo (`app.js` busca `/api/...` por caminho relativo) — mesma
  origem, então Origin/token não quebram o uso legítimo.

## Comportamento esperado
1. Todas as rotas verificam `Host` = `127.0.0.1:<porta>`; qualquer outro → `421`.
2. `POST` exige `Origin` igual a `http://127.0.0.1:<porta>` (ou `Origin` ausente com
   `Sec-Fetch-Site: same-origin`) **e** o cabeçalho `X-Sonix-Token`: 128 bits de fonte
   criptográfica do SO, gerado a cada execução e injetado no `index.html` servido; o despachante
   da Spec 014 o envia em toda requisição.
3. Nenhuma resposta envia `Access-Control-Allow-Origin`.
4. Corpo de `POST` > 4 KiB → `413` sem ler o resto e sem alterar estado.
5. Chaves, tipos, faixas e a validação atômica são os da **tabela da Spec 014 §2–§3** — esta spec
   não os redefine.
6. Defesa em profundidade no DSP: setters do `RealtimeDspChain` ignoram valores não finitos
   mantendo o anterior e limitam à faixa da tabela da Spec 014; se o processamento produzir
   amostra não finita, a cadeia zera a amostra e reinicia seus estados (auto-recuperação).
7. `/api/telemetry` e `/api/state` (somente leitura) continuam sem token, mas sujeitos à
   checagem de `Host`.
8. Telemetria não espera mutações: atendimento concorrente ou fila curta.

## Critérios de aceite
- **CA-1:** `POST /api/params` com `Origin: http://evil.example` (ou `Sec-Fetch-Site:
  cross-site`) e token válido retorna `403` e o estado do motor permanece idêntico (comparação de
  `GET /api/state` antes/depois).
- **CA-2:** Qualquer rota com `Host: evil.example:7890` retorna `421` (simulação de DNS rebinding).
- **CA-3:** Nenhuma resposta de nenhuma rota contém o cabeçalho `Access-Control-Allow-Origin`.
- **CA-4:** `POST` com `Origin` correto sem `X-Sonix-Token` ou com token errado retorna `401` e
  não altera estado; o token do `index.html` servido tem 32 hex, e dois boots consecutivos geram
  tokens diferentes.
- **CA-5:** Corpo de 5 KiB → `413` e o estado não muda.
- **CA-6:** `set_input_boost(f32::INFINITY)`, `set_input_boost(f32::NAN)`,
  `set_gate_threshold_db(f32::NAN)` e `set_amp_gain(f32::NAN)` seguidos de 50 blocos produzem
  saída 100 % finita e idêntica à de antes da chamada.
- **CA-7:** Injetando `NaN` no estado interno do gate/amp (hook de teste), em ≤ 1 bloco a saída
  volta a ser finita e `process` não propaga `NaN` para o buffer de saída.
- **CA-8:** Com 1.000 `POST /api/params`/s por 2 s, `GET /api/telemetry` responde com
  p99 < 20 ms.
- **CA-9:** Não-regressão — `POST /api/params` com `Host`, `Origin` e token corretos e corpo
  válido retorna `200` e `GET /api/state` reflete a mudança; o JS servido envia `X-Sonix-Token`
  em toda chamada de `syncParamsToBackend` (teste JS com `fetch` simulado); o bind continua apenas
  em `127.0.0.1` (conexão por IP de outra interface é recusada — Spec 012/013).

## Fora de escopo
- Contrato funcional do endpoint (chaves, tipos, faixas, `400`, `405`, `415`) → **Spec 014**.
- TLS/HTTPS no loopback (custo sem ganho: tráfego não sai da máquina).
- Autenticação multiusuário / controle remoto pela rede — o produto é local (Spec 012/013).
- Troca do HTTP local por WebView/IPC nativo (Spec 012 CA-6) — decisão de arquitetura futura.

## Invariantes tocadas
- Loopback only (Spec 012/013): reforçada (Host/Origin).
- Áudio em tempo real (Spec 001): o auto-reparo do CA-7 não aloca nem trava (reinicia estados
  in-place).
- Spec 003 CA-7 (NaN/limites): estendida do painel do amp para gate, boost e cab.

## Testes que cobrem esta spec
Planejados: `crates/sonix-desktop/tests/contract_015_server_hardening.rs` (CA-1…CA-9, cliente
HTTP mínimo via `std::net::TcpStream`) e
`tests/contract/015-endurecimento-servidor-local-validacao-csrf.test.js` (parte JS do CA-9).

## Notas
- Implementar **depois** da Spec 014 (ela cria o `POST` JSON que esta spec protege) e na mesma
  release.
- Esta versão substitui a anterior: os casos de faixa/tipo (antigo CA-6), JSON truncado (parte
  do antigo CA-7) e `GET → 405` (antigo CA-4) passaram para a Spec 014 CA-12/CA-13, que é a dona
  da tabela de parâmetros; o antigo CA-11 ("move knobs") virou o CA-9, verificável sem
  interação manual.
- Reprodução do envenenamento por `NaN` guardada em `scratch/repro` do conversation `c576a81c…`
  (não versionada).
