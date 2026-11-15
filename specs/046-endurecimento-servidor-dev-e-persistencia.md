---
spec: 046
titulo: Host/Origin por prefixo aceitam localhost.evil.com, token previsível e id de preset grava fora da pasta
status: implementada
atualizado: 2026-11-14
---
# 046 — Endurecimento do servidor local (dev-http) e da persistência de presets/pacotes

## Objetivo
Impedir que uma página na internet controle o motor por DNS rebinding e que um id de preset grave ou leia fora da pasta de presets.

## Contexto
Review manual das Specs 010 e 015 (2026-11-14), por leitura de código. O servidor HTTP é `dev-http`, mas é **feature default** de `sonix-desktop`, e o `INICIAR_SONIX_PRO.bat` inicia esse binário. Ou seja, ele está no caminho do usuário.

| # | Achado | Evidência |
|---|---|---|
| 1 | **Host por prefixo.** O Host é validado com `starts_with("127.0.0.1") \|\| starts_with("localhost")`. `localhost.evil.com` passa, e a ausência de Host é aceita. Com DNS rebinding, uma página em `localhost.evil.com` lê o `/` (que injeta o token) e faz POST com o token válido. | `server.rs:264-268` |
| 2 | **Origin por prefixo.** O Origin é validado com `starts_with("http://localhost")`, então `http://localhost.evil.com` passa. | `server.rs:403` |
| 3 | **Token previsível.** O token CSRF é derivado de `SystemTime` em ns e do endereço de um `Box`, XOR com constantes; não usa CSPRNG. | `server.rs:133-147` |
| 4 | **Path traversal no id do preset.** `presets/save` usa o `id` da requisição direto em `save_dir.join(format!("{}.sonixpreset", id))`: `../../x` grava fora da pasta. `find_preset_file` (load/export) faz o mesmo na leitura. | `api.rs:2848-2858, 2720-2746` |
| 5 | **Caminhos absolutos do Windows no `.sforgepkg`.** `add_ir`/`import_bundle` rejeitam `..`, `/` e `\` no início, mas aceitam `C:\x`, `C:x` e `\\?\…` em outra forma. | `sonix-core/src/persistence.rs:202, 273` |
| 6 | **Manifesto não confere a lista de arquivos.** Uma entrada listada no manifesto sem arquivo correspondente é aceita em silêncio. | `persistence.rs:270-293` |

## Comportamento esperado
- Host e Origin são comparados por **igualdade exata** com `127.0.0.1:<porta>` e `localhost:<porta>`, ou com as origens `http://` equivalentes. Host ausente → 400.
- O token tem 128 bits do gerador do sistema operacional.
- O id do preset segue `^[a-z0-9][a-z0-9_-]{0,63}$`, e o caminho resolvido precisa ficar dentro da pasta de presets.
- Caminhos de pacote só aceitam segmentos relativos simples (sem `:`, sem raiz, sem `..`, sem prefixo UNC/verbatim).
- O manifesto e a lista de arquivos precisam coincidir exatamente.

## Critérios de aceite
- **CA-1:** Uma requisição com `Host: localhost.evil.com` responde 421; `Host: 127.0.0.1.evil.com` responde 421; sem Host responde 400; `Host: 127.0.0.1:<porta>` segue normalmente.
- **CA-2:** Um POST com `Origin: http://localhost.evil.com` e token válido responde 403.
- **CA-3:** O token tem 32 caracteres hex e é gerado por `getrandom` (ou equivalente do SO). Duas instâncias criadas no mesmo nanossegundo simulado produzem tokens diferentes.
- **CA-4:** `presets/save` com `id = "../../evil"` responde 400, e nenhum arquivo é criado fora da pasta de presets (o teste verifica o diretório pai). O mesmo vale para load e export com `id` contendo `..`, `/`, `\` ou `:`.
- **CA-5:** `SforgePackage::add_ir` e `import_bundle` rejeitam `C:\x.wav`, `C:x.wav`, `\\server\x.wav` e `\\?\C:\x.wav` com `PathTraversalDetected`.
- **CA-6:** Um bundle cujo manifesto lista `irs/a.wav` sem esse arquivo no payload é rejeitado com `CorruptedArchive`.
- **CA-7 (não-regressão):** A UI servida pelo próprio servidor, com Host/Origin corretos e token, continua salvando, carregando e importando presets. Os contratos da Spec 015 continuam passando.

## Fora de escopo
- Remover o `dev-http` do binário padrão ou do `.bat`. É uma decisão de distribuição, e o app Tauri (Spec 033) já não usa HTTP. Se for decidido, vira spec própria.
- Assinatura criptográfica de pacotes `.sforgepkg`: o manifesto SHA-256 detecta corrupção, não autoria.

## Invariantes tocadas
- **Servidor local endurecido (Spec 015):** esta spec corrige a implementação dessa invariante.
- **Offline-first (Spec 010):** sem mudança de formato.

## Testes que cobrem esta spec
- `crates/sonix-desktop/tests/contract_046_server_persistence_hardening.rs` → CA-1 (`Host` exato), CA-2 (`Origin` exato), CA-3 (token CSPRNG via `getrandom` 128 bits hex), CA-4 (path traversal em presets/save, load e export) e CA-7 (não-regressão).
- `crates/sonix-core/tests/contract_046_package_hardening.rs` → CA-5 (`PathTraversalDetected` em caminhos Windows `C:\`, `C:`, `\\server`, `\\?\`) e CA-6 (`CorruptedArchive` para manifesto e lista de arquivos divergentes).
