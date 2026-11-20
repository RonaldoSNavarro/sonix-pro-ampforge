# ORDEM DE SERVIÇO — Correção integral dos achados da auditoria técnica de 2026-11-20

| Campo | Valor |
|---|---|
| Destinatário | Agente Antigravity |
| Emitida por | Dono do projeto |
| Base | `main` @ `8f499ac900fe2807e74e41707e380af031cd925d` (árvore limpa em 2026-11-20) |
| Origem dos achados | Auditoria técnica (rodada 1) + complemento com medições no código real |
| Itens | AUD-01 a AUD-47 (4 × P0, 19 × P1, 17 × P2, 7 × P3) |
| Entregável | Branch `fix/auditoria-2026-11-20` + relatório no formato da §6 |

---

## 0. Contrato de execução — leia inteiro antes de abrir qualquer arquivo

### 0.1 Natureza deste documento
Esta é uma **ordem de serviço fechada**, não uma proposta. Cada item já traz:
- a evidência;
- a decisão de correção;
- o teste que prova a correção;
- o critério numérico de pronto.

Você **executa**. Não resume, não reorganiza, não reprioriza, não abre discussão.

### 0.2 Sem specs novas
Por decisão do dono do projeto, esta rodada **não cria nenhuma spec**. Este documento é o requisito. Ele suspende a regra "Sem spec, sem código" do `AGENTS.md` **somente** para os itens AUD-01 a AUD-47.

**Proibido:**
- criar `specs/NNN-*.md`;
- acrescentar linhas ao índice de `specs/README.md`;
- criar "spec de correção", adendo, ADR, plano, *roadmap* ou qualquer documento de planejamento.

**Permitido:** editar specs existentes **apenas** onde um item manda (status, citações, trechos com evidência falsa, um CA explicitamente indicado).

### 0.3 TDD estrito continua obrigatório
Para cada item que não seja marcado **[DOC]**, siga estes passos:
1. Escreva o(s) teste(s) do item.
2. Rode e **cole no relatório o trecho da falha** (*red*), com o comando.
3. Implemente.
4. Rode e **cole o trecho do sucesso** (*green*), com o comando.

Se o teste passar antes da correção, ele está errado: reescreva até falhar pelo motivo descrito no item. A única exceção é quando o item diz textualmente que um resultado verde antes da correção refuta o achado; nesse caso, registre a medição.

### 0.4 As decisões estão tomadas
Toda escolha de projeto necessária está no campo **Decisão** de cada item.
- Não reabra, não ofereça alternativas, não pergunte.
- Se discordar, implemente mesmo assim e registre a objeção **com medição** na seção "Objeções" do relatório.
- Exceção única: a decisão não compila ou não pode ser executada como escrita. Aplique então a menor variação que cumpra o critério do teste e registre.

### 0.5 Proibições
Cada uma reprova a entrega.
- Perguntar "posso prosseguir?" ou "deseja que eu…?", pedir autorização ou parar entre itens ou fases.
- Classificar um item como "parcial", "aguardando autorização", "pendente de aprovação", "próxima etapa" ou "fora do escopo", salvo o que a §0.7 lista.
- Usar `#[ignore]`, `todo!()`, `unimplemented!()`, `test.skip`, `// TODO`, `// FIXME` ou `cfg` que desligue um teste no Windows.
- Usar `unwrap_or_default()` ou `let _ =` para engolir um erro que o item manda tratar.
- Afrouxar tolerância, limiar ou janela de medição deste documento ou de teste existente.
- Mudar a expectativa de um teste existente para casar com o código. A exceção é quando o item diz **"substitua a expectativa"**.
- Compensar a remoção de um estágio oculto com ganho, EQ ou *offset* "mágico".
- Usar `SimulatedAudioHost`, mocks ou cópias da lógica onde o item exige o caminho de produção.
- Escrever no relatório adjetivo sem número ("orgânico", "cintilante", "musical", "calibrado", "otimizado", "robusto", "fiel"). Todo resultado é **número + comando**.
- Declarar "todos os testes passando" sem colar o resumo da execução completa.
- Implementar qualquer coisa das specs 053–092, ou editá-las além do que o AUD-38 manda.
- Usar os handoffs anteriores de `docs/handoff/` como referência de formato ou de estado do projeto. Vários deles afirmam coisas que a auditoria refutou. Exemplo: o handoff 051 cita "WASAPI Exclusivo… 64 amostras", mas `has_mode_support(WasapiExclusive)` devolve `false`.
- Fazer *push*, *force-push*, reescrever histórico ou commitar em `main`.

### 0.6 Informação externa ausente (PENDENTE)
Não invente valor de componente, fonte, citação ou medição. Quando o item mandar usar uma **decisão de engenharia**:
- dê à constante o prefixo `DECISAO_SONIX_`;
- comente a data, o motivo (fonte ausente) e o que a confirmaria;
- nunca apresente uma decisão de engenharia como fidelidade ao hardware na UI, no README ou no inventário de controles.

### 0.7 Bloqueios aceitos — e só estes
1. SDK ASIO ausente (`CPAL_ASIO_DIR` vazio).
2. Hardware físico ausente: controlador MIDI ou interface USB para o teste de desconexão.
3. Validadores `pluginval` ou `clap-validator` não instalados.

Um bloqueio aceito **não interrompe o trabalho**: registre e siga para o próximo item. Qualquer outro problema é seu para resolver, inclusive teste difícil, refatoração grande ou conflito entre testes.

### 0.8 Git
- Crie a branch `fix/auditoria-2026-11-20` a partir de `8f499ac`.
- Faça **um commit por item**, com título `fix(AUD-NN): …`, `test(AUD-NN): …` ou `docs(AUD-NN): …`.
- O corpo do commit traz o comando de teste e o resultado numérico.
- Se o item exigir, use mais de um commit, mas nunca misture dois itens no mesmo commit.

### 0.9 Ordem e portões
- Execute as fases de 1 a 9 em sequência e, dentro de cada fase, na ordem listada.
- Ao fim de cada fase, rode o **portão** (§2.3). Tudo precisa ficar verde, e `git status` só pode mostrar o que você pretendia mudar. Só então comece a fase seguinte.
- Se algo quebrou, conserte antes de seguir.

### 0.10 Verificação independente
Depois da entrega, o dono do projeto vai **reexecutar as medições de forma independente**, com uma bancada própria, fora do repositório. Qualquer número do relatório que não se reproduzir com tolerância de 10% (tempos) ou com a tolerância do item (demais medidas) reprova o item inteiro.

---

## 1. Estado de partida medido (HEAD `8f499ac`, máquina do dono: Pentium G3250, Windows 10)

### 1.1 Testes

| Execução | Resultado |
|---|---|
| `cargo test --workspace --exclude sonix-app --locked --no-fail-fast -- --skip test_ca_08_generate_and_verify_rust_ipc_schema_fixtures` | 436 aprovados, **2 reprovados**: `015 test_ca_05` (instável) e `019 test_ca_07` (determinístico) |
| `node --test` em todos os contratos, exceto o 048 | 117/118. Falha: o *wrapper* da 019 |
| `cargo test -p sonix-app --locked --no-fail-fast` | 13/13. Todos usam *openers* e *streams* simulados |
| `node --test tests/contract/048-*.test.js` | 3/3. SSIM da tela Main entre 0,855 e 0,923 (limite 0,85); tela Áudio sem cobertura |
| `035 test_ca_08` | Passa, mas **reescreve** `tests/fixtures/ipc-schemas/presets_load.json`. Divergência na l. 63: `v30_4x12_closed` (versionado) contra `v30_4x12` (gerado) |

### 1.2 Medições de referência
Use esta tabela para reproduzir o *red*. Leia "2ª dif." como o pico de |x[n] − 2x[n−1] + x[n−2]| na transição dividido pelo pico em regime.

| Tema | Valor medido |
|---|---|
| Denormais | Sem FTZ/DAZ: 4,8% de um núcleo → 15–29% após 130 s → **69–77%** após 200 s de silêncio, até 100% de amostras subnormais na saída. Com FTZ/DAZ: 4,8% constante, 0 subnormais |
| Servidor dev-http | Com `Origin: http://evil.example`, sem token e `Content-Type: text/plain`, `/api/reamp/run` respondeu **200 e criou diretório e arquivo em caminho arbitrário**. Com `di_len` gigante, o **processo abortou** (`memory allocation of 4000000000000000 bytes failed`). `/api/modifiers`, `/api/footswitch/press` e `/api/graph` responderam 200 sem token |
| Overdrive oculto (5 contra 0, −80 dBFS) | JCM800 +18,0 dB; Recto +18,0; Plexi +9,0; AC30 +9,0; Twin 0 (mas +11 dB em 10) |
| Presence e depth ocultos | Twin: presence +4,0 dB @ 4 kHz, depth +2,6 dB @ 85 Hz. Depth: JCM800 +1,6 dB, Plexi +2,0 dB |
| Master do AC30 | −7,8 / −0,8 / +3,1 dB em 0 / 5 / 10 |
| Troca de modelo (2ª dif.) | Twin→JCM800 40×; Plexi→AC30 77×; Recto→Twin 10× |
| Degrau 0→10 em controles de tom (2ª dif.) | Treble 104×, bass 131×, middle 34×, presence 1× |
| Tone stack AB763 (carga de 1 MΩ, knobs em 5) | −3,8 / −3,8 / −3,6 / −2,6 dB em 50 / 100 / 200 / 400 Hz; até −4,4 dB com B9 M9 T2 |
| Pitch | Sinal seco **atrasado 1024 amostras (21,33 ms)** em qualquer mix. Harmonizer: p99 de 1,40 ms contra orçamento de 1,33 ms (bloco 64 @ 48 kHz) |
| Afinador mono | ≤ 0,01 cent entre 61,7 e 330 Hz; +0,23 a +0,31 cent em 1,3 kHz; **nada abaixo de 60 Hz** |
| Afinador polifônico | Corda D3 afinada lê **+16,6 cents** com harmônicos; ≤ 1,3 cent só com fundamentais |
| RE-201 | Modos 5 = 9, 7 = 10 = 4 e 8 = 11 (duplicatas). Nenhum modo de cabeça única com mola. Cauda da mola de −115 a −135 dB RMS após 0,5 s |
| Drives (faixa do knob @ 1 kHz) | TS808 18,1 dB (circuito: 19,9 dB); RAT 26,0 dB (circuito: até 67 dB); DS-1 23,1 dB |
| Plugin | **45 alocações** por evento MIDI CC no `process` |
| CPU média por bloco | 48 kHz: 6,2–6,7% (padrão) e 18–21% (pesada). 96 kHz: 12,4–12,9% e 39–42% |
| Convolução, 96 kHz, bloco 32, cadeia pesada | 26% dos blocos acima de 50% do orçamento; 1,4% acima de 100%. Piso de ruído do SO: cerca de 0,4% |
| Invariância ao tamanho de bloco | **Bit a bit idêntica** com blocos 32, 37, 64, 256, 480, 512 e 1024 contra 128 (a propriedade vale; falta o teste) |

---

## 2. Convenções obrigatórias desta rodada

### 2.1 Onde ficam os testes
- **Rust:** `crates/<crate>/tests/auditoria_2026_11_20_<tema>.rs`. Temas: `seguranca`, `tempo_real`, `parametros`, `artefatos`, `funcionalidades`, `fidelidade`, `afinador`. Para o shell Tauri, use `apps/sonix-app/src-tauri/tests/auditoria_2026_11_20_<tema>.rs`.
- **JS:** `tests/contract/auditoria-2026-11-20-<tema>.test.js`.
- **Nome do caso:** `aud_NN_<descricao>` em Rust e `'AUD-NN: <descrição>'` em JS.
- Quando o item manda cobrir um CA de spec existente, o teste vai no arquivo de contrato daquela spec, com o nome `caN_…` e o comentário `// AUD-NN`.

### 2.2 Réguas de medição
Crie o módulo público `ampforge_dsp::regua`, sempre compilado e documentado como "medição para testes; proibido no caminho de tempo real". Ele deve ter exatamente estas funções:

| Função | Definição |
|---|---|
| `ganho_db(entrada, saida, sr, f)` | DFT de um bin em `f`, com janela de número inteiro de ciclos, descartando os primeiros 0,5 s. Retorna `20·log10(\|Y\|/\|X\|)` |
| `pico_segunda_diferenca(x, ini, fim)` | `max \|x[n] − 2x[n−1] + x[n−2]\|` em `[ini, fim)` |
| `razao_transiente(x, evento, sr)` | Pico de 2ª diferença em `[evento − 128, evento + 0,030·sr)` dividido pelo **maior** entre os picos em `[evento − 0,040·sr, evento − 0,010·sr)` e em `[evento + 0,100·sr, evento + 0,130·sr)` |
| `contar_subnormais(x)` | Número de amostras com `v.is_subnormal()` |
| `atraso_por_impulso(saida)` | Índice do pico absoluto da resposta a um impulso unitário |
| `rt60_schroeder(ir, sr)` | Integral reversa de Schroeder com ajuste linear entre −5 e −35 dB, extrapolado para −60 dB |
| `cents(f, f_ref)` | `1200·log2(f/f_ref)` |

Regras de medição:
- **Eventos** (troca de modelo, degrau de controle, troca de cena) acontecem sempre em múltiplo de 128 amostras: use a amostra **24064**. A auditoria já errou uma vez com 24000, que nunca disparava a troca e produziu um falso "sem clique".
- **Níveis:** −80 dBFS para medir ganho e filtro em regime linear; −12 dBFS para medir transientes.
- **Zero alocação:** todo item que toca o caminho de tempo real inclui um teste `aud_NN_zero_alloc` que usa o alocador contador já existente da 037 CA-7.

### 2.3 Portão de fase
Comandos, todos verdes, nesta ordem:
```
cargo test --workspace --locked --no-fail-fast
cargo test -p sonix-desktop --locked --no-fail-fast --features fault-injection   # a partir do AUD-04
npm test
node --test tests/contract/048-*.test.js
npm run test:e2e                                                                # a partir do AUD-36
npm --prefix apps/sonix-app/ui run build
cargo build --release -p sonix-app --features custom-protocol
cargo build --release -p sonix-desktop
cargo build --release -p sonix-plugin
git status --porcelain   # só mudanças intencionais; nenhum fixture regravado por teste
```

### 2.4 Bancada de desempenho
É evidência para o relatório, não portão. Crie `crates/sonix-desktop/examples/bancada_auditoria.rs` e rode com `cargo run --release -p sonix-desktop --example bancada_auditoria -- <cenario> <sr> <bloco>`.

**Cenários:**
- `padrao`: cadeia default.
- `pesada`: TS808 + delay + reverb + mod + EQ + split com AMP 2 + IR de 0,49 s.
- `silencio`: 2 s de riff seguidos de 240 s de zeros.
- `pitch`: harmonizer.
- `ruido`: carga sintética constante de 45 µs por bloco, para medir o piso do sistema operacional.

**Saída por cenário:** média, p99 e máximo do tempo por bloco em % do orçamento (N/fs). Rode sempre o cenário `ruido` na mesma execução, para separar o ruído do sistema operacional do custo de DSP.

---

## 3. Mapa dos achados

| AUD | Sev. | Título | Origem na auditoria | Fase |
|---|---|---|---|---|
| 01 | P0 | Servidor dev-http: escrita de arquivo arbitrária, CSRF e DoS | P0-05 | 1 |
| 02 | P0 | Denormais: CPU 15× após silêncio | P0-04 (antes P1-04) | 1 |
| 03 | P1 | Testes 015 CA-5 e 019 CA-7 falhando | B/H | 1 |
| 04 | P1 | `panic = "abort"` + 186 `unwrap()` sem contenção | P1-05 | 1 |
| 05 | P1 | Erros do stream ignorados; sem recuperação de dispositivo; 033 CA-12 | P1-06 | 1 |
| 06 | P2 | Contenção do `try_lock` entrega DI cru e o mute do afinador vaza | E-1 | 1 |
| 07 | P2 | Recurso aposentado: `mem::forget` e `unpark()` no thread de tempo real | E-3 | 1 |
| 08 | P0 | Controles ocultos por modelo (overdrive, depth, presence, master) | P0-03 | 2 |
| 09 | P1 | "Tabela única de parâmetros" são ≥ 6 tabelas paralelas | P1-12 | 2 |
| 10 | P1 | Clique na troca de modelo | C-05 | 3 |
| 11 | P2 | *Zipper* nos controles de tom | C-07 | 3 |
| 12 | P1 | Pitch: seco atrasado 21,3 ms e pico de CPU | C-04 | 3 |
| 13 | P2 | Latência reportada diferente da medida | F | 3 |
| 14 | P2 | Filtros de oversampling cortam agudos | F | 3 |
| 15 | P2 | Pico de CPU da convolução com buffer < 128 | F/D | 3 |
| 16 | P1 | Plugin: tipo exportado ≠ testado, aloca, parâmetro morto, sem latência | C-01 | 4 |
| 17 | P1 | MIDI do standalone não recebe nada | C-02 | 4 |
| 18 | P1 | Calibração não persiste | C-03 | 4 |
| 19 | P2 | `.sforgepkg` só existe como biblioteca e não tem limite de tamanho | C-13 | 4 |
| 20 | P2 | Split: variante morta, parâmetros fixos, crossover de 1ª ordem | C-16 | 4 |
| 21 | P2 | Roteamento implícito (comp_2/mod_2/wah) | E-4 | 4 |
| 22 | P1 | Tone stack AB763 com C2/C3 trocados e teste tautológico | P0-02 → P1 | 5 |
| 23 | P1 | Pacote de amps: scoop duplo, Vox TB, stack do Recto, presence em Modern, modo Pushed | P1-13, D | 5 |
| 24 | P1 | Entrada LOW do JCM800 | P1-02 | 5 |
| 25 | P2 | Seletor de impedância invertido | C-08 | 5 |
| 26 | P2 | Taper do Middle do Marshall | C-09 | 5 |
| 27 | P2 | Rede 470k‖470pF do JCM800 e bright cap do Plexi | D | 5 |
| 28 | P1 | RE-201: tabela de modos, mola sem cauda, wow inaudível | P1-03, C-11 | 5 |
| 29 | P1 | Drives: sem referência V/FS; ganhos de TS808/RAT; DS-1 assimétrico | P1-09, C-19 | 5 |
| 30 | P1 | Efeitos com nome de equipamento e mecanismo diferente; Vib-Trem do AC30 | P1-10, C-19 | 5 |
| 31 | P2 | Reverbs: nomes, "drip", decaimento da mola do Twin | C-18, D | 5 |
| 32 | P2 | Gabinete: hi-cut e alto-falante inertes com IR; Greenback como identidade | C-12, D | 5 |
| 33 | P2 | Afinador: faixa a partir de 40 Hz, precisão, nomes | P1-01c, C-20 | 6 |
| 34 | P2 | Afinador polifônico +16,6 cents | C-10 | 6 |
| 35 | P1 | Testes tautológicos, de mock ou que se autoconferem | P1-07, C-22, C-23 | 7 |
| 36 | P2 | CAs de specs implementadas sem teste; E2E 033 sem executor | C-14 | 7 |
| 37 | P3 | 048 sem a tela Áudio | C-21 | 7 |
| 38 | P0 | Evidência fabricada nas specs 053–092 [DOC] | P0-01, C-15 | 8 |
| 39 | P1 | Estados "Implementada"/"TESTED" inflados [DOC + teste] | P1-08 | 8 |
| 40 | P1 | Afirmações públicas contraditas pelo código [DOC + código] | P1-01 | 8 |
| 41 | P1 | Fontes sem link ou mal classificadas [DOC] | P1-11 | 8 |
| 42 | P3 | Higiene documental: citações, `file:///`, links, README [DOC] | C-17, G | 8 |
| 43 | P3 | Licença do binário VST3 [DOC] | G | 8 |
| 44 | P3 | `nih_plug` via git sem `rev` | C-24 | 9 |
| 45 | P3 | Plugin puxa `cpal`; `BiquadFilter` duplicado 6× | E-5 | 9 |
| 46 | P3 | Desempenho: 7 biquads identidade, FFT complexa | F | 9 |
| 47 | P3 | 1176: rótulo de aproximação [DOC] | D | 9 |

---

## 4. Itens

Formato fixo de cada item:
- **Evidência:** o que foi medido ou lido, com `arquivo:linha` em `8f499ac`.
- **Decisão:** o que fazer. Não reabra.
- **Testes:** o que escrever. Cada um precisa falhar antes da correção.
- **Pronto quando:** o critério objetivo.

### FASE 1 — Segurança e falhas que derrubam o áudio

#### AUD-01 · P0 · Servidor dev-http: escrita de arquivo arbitrária, CSRF e DoS
**Evidência:**
- Só estas rotas passam por `verify_csrf_and_token`: `/api/params`, `/api/input/di`, `/api/cab/ir`, `/api/presets/{load,save,import}`, `/api/scenes/select` e `/api/input/calibrate` ([server.rs:304-347](../../crates/sonix-desktop/src/server.rs#L304-L347)).
- O resto cai em `_ => api::dispatch(...)` **sem token e sem Origin** ([server.rs:348](../../crates/sonix-desktop/src/server.rs#L348)).
- `handle_post_reamp_run` grava no `session_path` informado pelo requisitante ([api.rs:4727-4734](../../crates/sonix-desktop/src/api.rs#L4727-L4734)) e aceita `di_len` sem limite ([api.rs:4686-4690](../../crates/sonix-desktop/src/api.rs#L4686-L4690)).
- Prova de conceito executada: ver a tabela 1.2.
- O host legado com `dev-http` é a *feature default* e é o binário do `INICIAR_SONIX_PRO.bat`.

**Decisão:**
1. A proteção passa a ser por **método**, não por rota. Toda requisição com método diferente de `GET` e `HEAD` passa por `verify_csrf_and_token` **antes** do roteamento.
   - Apague as listas de rotas protegidas.
   - Os limites de corpo por rota (413) continuam e são aplicados antes do *handler*.
2. Cheque o `Host` em **todas** as requisições, inclusive GET, contra DNS rebinding. O valor tem de ser exatamente `127.0.0.1:<porta>` ou `localhost:<porta>`; caso contrário, responda 403 `Forbidden Host`.
3. Nenhuma rota GET pode mudar estado. Revise a tabela de rotas ([api.rs:1586-1685](../../crates/sonix-desktop/src/api.rs#L1586-L1685)); qualquer mutação via GET passa a ser POST.
4. `session_path` sai do contrato.
   - A metadata de reamp é gravada **somente** em `<pasta de dados do app>/reamp/sessao-<unix_ms>.json`, na mesma pasta-base dos presets (`%APPDATA%\SonixPro`). A resposta devolve o caminho gravado.
   - Requisição com `session_path` → 400 `{"error":"session_path não é aceito"}`.
   - A pasta de dados precisa ser injetável no construtor (`LocalAppServer::bind_port_with_data_dir`), para os testes não gravarem em `%APPDATA%`.
5. Limite todo comprimento vindo da requisição.
   - `di_len > reamp_rec.len()` → 400.
   - Procure em `api.rs` por `with_capacity(`, `vec![`, `resize(` e `reserve(` alimentados por valores da requisição: cada ocorrência ganha limite explícito e resposta 400.
6. As mudanças 4 e 5 valem também para o IPC do Tauri, que reaproveita `api::dispatch`. Confirme com um teste pelo comando Tauri.
7. Exponha a tabela de rotas como `pub static ROUTES: &[(&str, &str)]` (método, caminho), usada **pelo próprio despacho**. Não pode haver lista duplicada.

**Testes** (`crates/sonix-desktop/tests/auditoria_2026_11_20_seguranca.rs`, `required-features = ["dev-http"]`):
- `aud_01_toda_rota_mutante_exige_token`: para cada entrada de `ROUTES` com método ≠ GET, sem token → 403; com token e Origin válida → resposta ≠ 403.
- `aud_01_post_simples_cross_site_rejeitado`: com `Content-Type: text/plain`, `Origin: http://evil.example` e sem token, as rotas `/api/reamp/run`, `/api/modifiers`, `/api/footswitch/press` e `/api/graph` respondem 403.
- `aud_01_host_invalido_rejeitado`: `GET /` com `Host: evil.example:<porta>` → 403.
- `aud_01_reamp_recusa_session_path`: com token e `session_path` apontando para `<tempdir>/x/y.json` → 400, e o arquivo **não existe**.
- `aud_01_reamp_grava_na_pasta_do_app`: com a pasta de dados injetada (tempdir), o arquivo é criado dentro dela.
- `aud_01_di_len_absurdo_nao_derruba`: `di_len = 2^62` → 400; em seguida, `GET /api/state` → 200.
- Tauri, `aud_01_ipc_reamp_recusa_session_path`.

**Pronto quando:** os 7 testes passam e a prova de conceito da tabela 1.2 deixa de funcionar quando repetida.

#### AUD-02 · P0 · Denormais
**Evidência:**
- Não há FTZ, DAZ, MXCSR nem *flush* em `crates/` nem em `src-tauri`.
- Valores medidos: ver a tabela 1.2.
- Fontes de subnormais: malhas recursivas em f32 (combs e allpass do reverb, realimentação do delay e da modulação, mola) e em f64 (`interstage`, `sag`).

**Decisão:**
1. Crie o módulo `crates/ampforge-dsp/src/denormal.rs` **com este código** (não use `_mm_setcsr`, que está obsoleto):
   ```rust
   #[cfg(target_arch = "x86_64")]
   #[inline(always)]
   fn ler_mxcsr() -> u32 {
       let mut v: u32 = 0;
       unsafe { core::arch::asm!("stmxcsr [{p}]", p = in(reg) &mut v as *mut u32, options(nostack, preserves_flags)); }
       v
   }
   #[cfg(target_arch = "x86_64")]
   #[inline(always)]
   fn escrever_mxcsr(v: u32) {
       unsafe { core::arch::asm!("ldmxcsr [{p}]", p = in(reg) &v as *const u32, options(nostack, preserves_flags, readonly)); }
   }
   #[cfg(not(target_arch = "x86_64"))] #[inline(always)] fn ler_mxcsr() -> u32 { 0 }
   #[cfg(not(target_arch = "x86_64"))] #[inline(always)] fn escrever_mxcsr(_: u32) {}

   pub const FTZ_DAZ: u32 = 0x8040; // bit 15 = FTZ, bit 6 = DAZ
   pub struct DenormalGuard { anterior: u32 }
   impl DenormalGuard {
       #[inline(always)]
       pub fn new() -> Self { let anterior = ler_mxcsr(); escrever_mxcsr(anterior | FTZ_DAZ); Self { anterior } }
   }
   impl Drop for DenormalGuard { #[inline(always)] fn drop(&mut self) { escrever_mxcsr(self.anterior); } }
   pub fn mxcsr_atual() -> u32 { ler_mxcsr() }
   #[inline(always)] pub fn flush_f32(x: f32) -> f32 { if x.abs() < 1e-30 { 0.0 } else { x } }
   #[inline(always)] pub fn flush_f64(x: f64) -> f64 { if x.abs() < 1e-30 { 0.0 } else { x } }
   ```
2. A **primeira linha** de `RealtimeDspChain::process` ([audio.rs:4528](../../crates/sonix-desktop/src/audio.rs#L4528)) passa a ser `let _ftz = ampforge_dsp::denormal::DenormalGuard::new();`. Isso cobre o standalone, o Tauri e o plugin, que chama `chain.process`.
3. *Flush* explícito (`flush_f32` ou `flush_f64`) em **todo** armazenamento de estado recursivo:
   - combs e allpass de `fx/reverb.rs`;
   - realimentação de `fx/delay.rs`, incluindo RE-201 e mola;
   - realimentação de `fx/modulation.rs`;
   - `fx/pitch.rs`;
   - realimentação do looper, se houver;
   - `interstage` e `sag` em `amp.rs`.
4. Dentro de `process`, registre o MXCSR observado num campo `mxcsr_no_process: u32`, com o getter `pub fn mxcsr_observado(&self) -> u32`.

**Testes:**
- `aud_02_guard_liga_e_restaura`: dentro do *guard*, `mxcsr_atual() & FTZ_DAZ == FTZ_DAZ`; depois do `drop`, o valor é igual ao de antes.
- `aud_02_process_roda_com_ftz_daz`: depois de `chain.process`, `mxcsr_observado() & 0x8040 == 0x8040`, e o MXCSR do thread do teste não mudou.
- `aud_02_<processador>_sem_subnormais`, um teste para cada modelo de reverb, de delay (incluindo os modos 5–12 do RE-201) e de modulação:
  - chame o processador **diretamente**, sem *guard*;
  - entrada: impulso de amplitude 1e-30 seguido de 20 s de zeros;
  - critério: `contar_subnormais(saida) == 0`.
- `aud_02_zero_alloc`.

**Pronto quando:** os testes passam, e na bancada o cenário `silencio` a 48 kHz com bloco 128 mostra a média da última janela de 10 s ≤ 1,5 × a média da primeira.

#### AUD-03 · P1 · Testes 015 CA-5 e 019 CA-7 falhando
**Evidência:**
- `contract_015_server_hardening::test_ca_05_payload_too_large_rejected_with_413` falhou 3/3 isolado e passou dentro da suíte JS: é instável.
- `contract_019_input_calibration::test_ca_07_post_api_input_di_validation_and_resampling` falha sempre com "conexão resetada" ao enviar 20 MB + 1 KiB.
- **Causa:** [server.rs:95-113](../../crates/sonix-desktop/src/server.rs#L95-L113) lê no máximo `max_read` bytes, responde e fecha o socket com bytes ainda não lidos no buffer de recepção. No Windows, isso gera RST e o cliente vê *reset* em vez de 413.

**Decisão:**
1. Se `Content-Length` > limite da rota, **não leia o corpo**: escreva o 413 imediatamente.
2. Em seguida, `stream.shutdown(Shutdown::Write)`.
3. Drene (leia e descarte) até `Content-Length` bytes ou até 2 s de `set_read_timeout`, e só então feche.
4. Faça o mesmo com toda resposta de erro antecipada.
5. POST sem `Content-Length` → 411.

**Testes:** não altere os dois testes. Rode cada um **20 vezes seguidas**:
```
for i in $(seq 20); do cargo test -p sonix-desktop --test contract_015_server_hardening test_ca_05 -q || exit 1; done
for i in $(seq 20); do cargo test -p sonix-desktop --test contract_019_input_calibration test_ca_07 -q || exit 1; done
```

**Pronto quando:** 20/20 nos dois, com a saída colada.

#### AUD-04 · P1 · Pânico derruba o processo inteiro
**Evidência:**
- [Cargo.toml:21](../../Cargo.toml#L21) tem `panic = "abort"`.
- `api.rs` tem 186 `unwrap()` que dependem de uma validação feita em outra tabela; por exemplo, `v.as_f64().unwrap()` em [api.rs:3357-3362](../../crates/sonix-desktop/src/api.rs#L3357-L3362).
- Não há `catch_unwind` em produção.
- O 018 CA-11 só é testado no `SimulatedAudioHost`: `trigger_test_panic` só liga uma *flag* ([audio.rs:432-434](../../crates/sonix-desktop/src/audio.rs#L432-L434)).

**Decisão:**
1. Mude `[profile.release]` para `panic = "unwind"`.
2. No callback de saída, onde se chama `dsp.process` ([audio.rs:~990](../../crates/sonix-desktop/src/audio.rs#L990)):
   - envolva a chamada em `std::panic::catch_unwind(AssertUnwindSafe(|| dsp.process(..)))` **dentro** do escopo do *lock*, para o `Mutex` não ficar envenenado;
   - em `Err`: zere o bloco de saída, ligue `engine_faulted: AtomicBool` e não chame mais `process` nessa instância.
   - Faça o mesmo no `process_block` do plugin (AUD-16).
3. O laço de controle (telemetria a 30 Hz do Tauri em `streams.rs`; *fallback worker* no host legado), ao ver `engine_faulted`:
   - reconstrói `RealtimeDspChain` a partir do `AppState` atual, fora do tempo real;
   - troca a instância sob o *lock* por um instante;
   - limpa a *flag* e emite o evento `engine_recovered` com a contagem.
4. Remova **todo** `unwrap()` e `expect()` do código não-teste de `api.rs` e de `apps/sonix-app/src-tauri/src/commands.rs`. Os erros viram `400 E_VALIDATION: …` ou `500 E_INTERNAL: …`.
5. Crie a *feature* `fault-injection` em `sonix-desktop`, desligada por padrão e nunca usada nos builds do app. Ela traz o parâmetro `debug.inject_panic`, que provoca um pânico dentro de `RealtimeDspChain::process`.

**Testes:**
- `aud_04_dispatch_nunca_entra_em_panico`:
  - para cada chave da tabela de parâmetros, testar os valores `true`, `false`, `0`, `-1`, `1e308`, `-1e308`, `"x"`, `""`, `null`, `[]`, `{}` e `"NaN"`;
  - `dispatch` devolve 2xx ou 4xx e nunca entra em pânico (verificado com `catch_unwind` no teste).
- `aud_04_panico_no_callback_vira_silencio_e_recupera` (`--features fault-injection`):
  - chame **a mesma função de callback** usada em `build_output_stream`, com uma cadeia real;
  - injete o pânico: o bloco sai todo em zero e a *flag* liga;
  - rode a rotina de recuperação: o bloco seguinte, com entrada não nula, sai não nulo.
- `aud_04_api_sem_unwrap`, *lint* complementar: zero ocorrências de `.unwrap()` ou `.expect(` fora de `#[cfg(test)]` em `api.rs` e `commands.rs`.
- **Substitua a expectativa** do 018 CA-11 para usar o caminho real acima, não o simulador.

**Pronto quando:** os testes passam e o *release* compila com `panic = "unwind"`.

#### AUD-05 · P1 · Erro do stream ignorado; sem recuperação de dispositivo
**Evidência:**
- O callback de erro do `cpal` é `|_err| {}` ([audio.rs:1306](../../crates/sonix-desktop/src/audio.rs#L1306)). Verifique **todas** as chamadas de `build_input_stream` e `build_output_stream`.
- `device_lost` e `simulate_device_reconnected` existem só no simulador.
- O 033 CA-12 (faixa "Dispositivo desconectado") não está implementado e não tem teste. Só existe o botão manual "Reconectar".

**Decisão:**
1. Todo callback de erro grava um código num `Arc<AtomicU8>` (0 = ok, 1 = `DeviceNotAvailable`, 2 = outro), sem alocar e sem *lock*.
2. O laço de controle lê esse código a ≥ 10 Hz.
   - **Código 1:** descarta os streams, define o estado `audio.device_state = "lost"` e emite o evento.
     - A cada 1 s, tenta reabrir o **mesmo** id de dispositivo, com a mesma configuração, pelo mesmo *opener* do boot.
     - Quando conseguir: estado `"ok"` e evento.
   - **Código 2:** registra no log e tenta reabrir uma vez imediatamente; se falhar, trata como código 1.
3. **UI:** faixa fixa "Dispositivo de áudio desconectado — tentando reconectar…" enquanto o estado for `lost`; ela some com `ok`. O botão "Reconectar" continua.
4. Os testes deste item **não** usam o `SimulatedAudioHost`.

**Testes:**
- `aud_05_erro_do_stream_marca_perda` (sonix-app): usando o mesmo mecanismo de *opener* injetável dos 13 testes existentes, o stream falso dispara o callback de erro com `DeviceNotAvailable`; o estado vira `lost` em ≤ 200 ms.
- `aud_05_reabre_quando_volta`: o *opener* falha 2× e depois funciona; o estado vira `ok` em ≤ 3,5 s e a cadeia volta a ser chamada.
- JS: no arquivo de contrato da 033, o teste `CA-12` com o comentário `// AUD-05` verifica que a faixa aparece com `lost` e some com `ok`.
- **Manual:** desconectar e reconectar uma interface USB. Sem a interface, isso é o bloqueio aceito 2.

**Pronto quando:** os 3 testes automáticos passam.

#### AUD-06 · P2 · Contenção do `try_lock` entrega DI cru
**Evidência:**
- Em [audio.rs:990-997](../../crates/sonix-desktop/src/audio.rs#L990-L997), se o `try_lock` falha, a saída recebe a entrada seca (comentário: Spec 034 CA-6).
- Resultado: um bloco de DI cru no meio de um timbre saturado. Com o afinador em `mute`, o sinal seco vaza.
- A contenção acontece durante o `prepare()` da reconfiguração ([commands.rs:400](../../apps/sonix-app/src-tauri/src/commands.rs#L400)) e em qualquer outro *lock* da cadeia.

**Decisão:**
1. Faça o inventário de todo `.lock()` e `.try_lock()` no `Mutex` da cadeia fora do callback (`api.rs`, `commands.rs`, `streams.rs`, `server.rs`). Cada um deve:
   - virar mensagem na fila de parâmetros; ou
   - rodar só com o stream parado (reconfiguração).
2. Telemetria de leitura (estado do gate, B+) vem do `TelemetryBuffer`, que o callback já publica. Nunca de um *lock*.
3. O *fallback* da 034 CA-6 continua, mas respeita o *mute*: com o afinador em `mute` ou em standby, a saída é zero em vez do sinal seco.

**Testes:**
- `aud_06_sem_contencao_em_uso_normal`:
  - cadeia real num thread de "callback" processando 2 s em blocos de 128;
  - ao mesmo tempo: parâmetros a 60 Hz via `dispatch`, leituras de osciloscópio e afinador pelo código de produção, e 8 trocas de cena;
  - critério: contagem de contenção igual a **0**.
- `aud_06_contencao_com_mute_da_silencio`: o teste segura o *lock* e chama a função de callback com o afinador em `mute`; a saída sai toda em zero.

**Pronto quando:** os 2 testes passam.

#### AUD-07 · P2 · Recurso aposentado vaza e faz syscall no tempo real
**Evidência:** em [audio.rs:2052-2063](../../crates/sonix-desktop/src/audio.rs#L2052-L2063):
- fila cheia → `std::mem::forget(rejected)`, um vazamento intencional;
- `self.worker_thread.unpark()`, uma syscall, a cada aposentadoria, dentro do thread de tempo real.

**Decisão:**
1. A capacidade da fila de aposentadoria é ≥ 2 × o máximo de recursos em trânsito (IR, DI, buffers de preset). Calcule esse valor e faça um `assert` na construção.
2. Se o `push` falhar: guarde o recurso num slot fixo pré-alocado `pendentes: [Option<RetiredDspResource>; N]` e tente de novo no bloco seguinte. Nunca use `forget`; nunca faça `drop` no tempo real.
3. Remova o `unpark()` do tempo real. O *worker* consulta a fila com `park_timeout(20 ms)`.

**Testes:**
- `aud_07_sem_vazamento`: com o *worker* pausado, faça 10 000 trocas de IR usando um recurso que conta `Drop`; retome o *worker*; o número de `drop` é igual ao de criados.
- `aud_07_zero_alloc`.
- *Lint*: nenhum `unpark(` em `audio.rs` fora do *worker*.

**Pronto quando:** os testes passam.

### FASE 2 — Controles ocultos e contrato de parâmetros

#### AUD-08 · P0 · Controles ocultos ativos, sem correspondência no painel real
**Evidência:**
- `od_gain` em [amp.rs:1206-1215](../../crates/ampforge-dsp/src/amp.rs#L1206-L1215):
  - Plexi e AC30: `od × 18 dB`, ou seja +9 dB no padrão 5;
  - JCM800 e Recto: `od × 36 dB`, ou seja +18 dB no padrão e +36 dB em 10;
  - Twin: `(od − 0,5) × 22`.
- Depth e presence ficam sempre no caminho ([amp.rs:1160-1164](../../crates/ampforge-dsp/src/amp.rs#L1160-L1164)).
- `master` age no AC30 ([amp.rs:1282-1285](../../crates/ampforge-dsp/src/amp.rs#L1282-L1285)).
- O registro tem `amp.overdrive` ([controls.registry.ts:362-375](../../apps/sonix-app/ui/src/controls.registry.ts#L362-L375)), mas nenhuma tela o renderiza; o mesmo vale para `amp.depth`.
- Valores medidos: ver a tabela 1.2.
- O preset "US Deluxe Clean" usa o Twin com `presence` e `master`.

**Decisão — painel real por modelo:**

| Modelo | Controles reais | Removidos do caminho de áudio |
|---|---|---|
| Plexi 1959 | Presence, Bass, Middle, Treble, Volume I, Volume II; entradas High e Low dos 2 canais; seletor de impedância | overdrive, depth, master |
| JCM800 2203 | Presence, Bass, Middle, Treble, Master Volume, Pre-amp Volume; entradas High e Low; impedância | overdrive, depth |
| Twin AB763 | Bright, Volume, Treble, Middle, Bass (Normal e Vibrato); Reverb, Speed, Intensity (Vibrato) | overdrive, depth, presence, master |
| Dual Rectifier Rev F | Os controles que o código já expõe para o Recto, exceto os da coluna ao lado | overdrive, depth, modo Pushed (AUD-23e) |
| AC30 Top Boost | Normal Volume, Brilliant Volume, Vib-Trem Volume, Speed, seletor Vib/Trem, Treble, Bass, Cut | overdrive, depth, presence, master |

Regras:
1. Um controle que não existe no modelo tem o termo de DSP **removido** naquele modelo. Não basta deixá-lo fixo em 5.
   - Sem master: o estágio de potência funciona como master no máximo, porque não existe atenuador no circuito.
   - Sem presence ou depth: o filtro é desviado (0 dB).
2. Nenhum dos 5 modelos tem **overdrive** nem **depth**. Por isso, apague em todo o código:
   - `amp_1.overdrive` e `amp_1.depth`;
   - os itens do registro;
   - `set_amp_overdrive` e `set_amp_depth`;
   - `DspParamUpdate::AmpOverdrive` e `DspParamUpdate::AmpDepth`;
   - os parâmetros correspondentes do plugin.
3. Presence e master continuam como chaves, com **validade por modelo**:
   - em `dispatch`: 400 `E_VALIDATION: amp_1.<x> não existe no modelo <id>`;
   - na UI: o controle não é renderizado nos modelos sem ele (o registro ganha o campo `models`).
4. Carga de preset: uma chave inexistente no modelo do preset é descartada, nunca aplicada, e volta em `"ignored_keys": [...]` na resposta.
   - Limpe os presets de fábrica.
   - "US Deluxe Clean" vira "US Twin Clean"; o id antigo continua aceito como alias.
5. **Proibido compensar.** Não acrescente ganho em outro lugar. Se, sem o termo oculto, o ganho de pequeno sinal do pré com ganho máximo ficar abaixo do previsto pelo esquema, corrija o ganho **dos estágios reais**:
   - ganho de cada estágio: `G = μ·(Ra‖Rl) / ((Ra‖Rl) + rp + (μ+1)·Rk_sem_bypass)`, com μ = 100 e rp = 62,5 kΩ;
   - Ra, Rl e Rk vêm do esquema do modelo, e cada constante ganha a sua fonte;
   - sem fonte: `DECISAO_SONIX_`.

**Testes:**
- `aud_08_controle_inexistente_nao_age`: para cada modelo e cada controle removido, os valores 0 e 10 produzem saídas idênticas (diferença ≤ −120 dB).
- `aud_08_dispatch_rejeita_controle_alheio_ao_modelo`: `amp_1.presence` no Twin → 400.
- `aud_08_preset_descarta_chave_inexistente`: um preset com `amp_1.master` no Twin carrega, devolve `ignored_keys` com a chave e soa igual ao preset sem ela.
- JS `AUD-08`: todo controle de amp declara `models`; a tela Main só renderiza os controles do modelo ativo.

**Relatório:** tabela com o ganho de pequeno sinal por modelo nos valores padrão, antes e depois, em dB.

#### AUD-09 · P1 · Tabela única de parâmetros de verdade
**Evidência:**
- A mesma chave aparece em ≥ 6 `match` paralelos de `api.rs`:
  - conversão (~1790–1940);
  - validação (~2186 e ~2710);
  - estado (~3357);
  - lista (~4228);
  - carga de preset (~4440).
- A conversão do 1176 está duplicada em [api.rs:1914-1923](../../crates/sonix-desktop/src/api.rs#L1914-L1923) e em [audio.rs:4297-4301](../../crates/sonix-desktop/src/audio.rs#L4297-L4301).
- A deriva entre essas tabelas vira `unwrap()` e, sem o AUD-04, aborto do processo.

**Decisão:**
1. Crie `crates/sonix-desktop/src/params.rs` com `pub static PARAMS: &[ParamDesc]`, onde `ParamDesc` tem:
   - `key`;
   - `kind`: `Number{min,max,step}`, `Bool` ou `Enum(&[&str])`;
   - `default`;
   - `models: Option<&[AmpModelId]>`;
   - `to_dsp: fn(&Value) -> Option<DspParamUpdate>`;
   - `read_state: fn(&AppState) -> Value`;
   - `write_state: fn(&mut AppState, &Value)`.
2. Validação, conversão, leitura e escrita de estado, a lista de `/api/params`, a carga de preset e a regra por modelo do AUD-08 **iteram sobre `PARAMS`**. Apague os `match` paralelos. A conversão do 1176 passa a existir só em `to_dsp`.
3. Gere `apps/sonix-app/ui/src/params.generated.ts` a partir de `PARAMS` com `cargo run -p sonix-desktop --example gerar_params_ts`.
   - `controls.registry.ts` mantém só os metadados de UI e importa faixa, padrão e tipo do arquivo gerado.

**Testes:**
- `aud_09_toda_chave_ida_e_volta`: para cada chave, um valor válido → `dispatch` ok → o estado devolve o mesmo valor → sai exatamente **um** `DspParamUpdate`.
- `aud_09_ts_gerado_em_dia`: gera em memória e compara com o arquivo versionado; falha se houver diferença.
- JS `AUD-09`: toda `backendKey` do registro existe em `params.generated.ts`.
- *Lint*: nenhum literal de chave de `PARAMS` aparece em `api.rs`.

**Pronto quando:** os testes passam e os `match` paralelos não existem mais.

### FASE 3 — Artefatos audíveis e latência

#### AUD-10 · P1 · Clique na troca de modelo
**Evidência:**
- [amp.rs:1679-1691](../../crates/ampforge-dsp/src/amp.rs#L1679-L1691): o crossfade de 20 ms cobre só os filtros de voicing e o drive de potência.
- A topologia do pré e os `mod_settings` comutam de uma vez no fim dos 20 ms.
- Valores medidos: 40×, 77× e 10× (tabela 1.2).

**Decisão:**
1. Use duas instâncias completas do amp, `amp` e `amp_transicao`, pré-alocadas em `new`/`prepare`.
2. Na troca de modelo:
   - copie o painel (dados simples) para `amp_transicao`;
   - defina o modelo-alvo e chame `reset()` no estado dela;
   - processe as **duas** instâncias por 20 ms, com ganhos de potência constante: `cos` e `sin` de uma rampa linear de 0 a π/2;
   - no fim, `std::mem::swap(&mut amp, &mut amp_transicao)` e pare de processar a antiga.
3. Apague de `amp.rs` o mecanismo `target_model_id`, `target_voicing_*`, `target_mod_settings` e `model_crossfade`: só pode existir um mecanismo.
4. Vale também para o AMP 2.

**Testes:**
- `aud_10_troca_de_modelo_sem_clique`:
  - os 20 pares ordenados dos 5 modelos;
  - seno de 220 Hz a −12 dBFS, troca na amostra 24064;
  - critério: `razao_transiente ≤ 2,0`.
- `aud_10_troca_com_bloco_32`: o mesmo teste com blocos de 32.
- `aud_10_zero_alloc`.

**Pronto quando:** os 20 pares passam.

#### AUD-11 · P2 · *Zipper* nos controles de tom
**Evidência:**
- Os *setters* escrevem direto no painel ([audio.rs:3892-3908](../../crates/sonix-desktop/src/audio.rs#L3892-L3908)).
- O amp recalcula stack, presence e depth a cada chamada, sem suavização ([amp.rs:1140-1164](../../crates/ampforge-dsp/src/amp.rs#L1140-L1164)). Só o ganho é suavizado.
- Valores medidos: treble 104×, bass 131×, middle 34×.

**Decisão:**
1. Use um `ParamSmoother` (o mesmo tipo do `gain_smoother`), linear de 20 ms, em todo controle que muda coeficientes de filtro:
   - bass, middle, treble, presence, master, volume_2;
   - hicut e resonance do gabinete;
   - bandas de EQ;
   - wah.
2. Enquanto algum *smoother* estiver em rampa, `process` subdivide em sub-blocos de ≤ 32 amostras e recalcula os coeficientes a partir dos valores suavizados. Com tudo estável, recalcula uma vez por chamada.

**Testes:**
- `aud_11_degrau_sem_zipper`:
  - cada controle, com degraus de 0→10 e de 10→0 na amostra 24064;
  - seno de 1 kHz a −12 dBFS;
  - critério: `razao_transiente ≤ 2,0`.
- `aud_11_rampa_termina_em_25ms`: o ganho em 1 kHz fica a menos de 0,1 dB do valor final em ≤ 25 ms.
- `aud_11_zero_alloc`.

**Pronto quando:** todos os controles passam.

#### AUD-12 · P1 · Pitch: seco atrasado 21,3 ms e pico de CPU
**Evidência:**
- [pitch.rs:412](../../crates/ampforge-dsp/src/fx/pitch.rs#L412) faz `let dry = self.read_linear(center_delay)` com `center_delay = 1024·sr_scale` ([pitch.rs:325](../../crates/ampforge-dsp/src/fx/pitch.rs#L325)).
- Vozes *wet* usam atrasos-base de 1600 e 512 ([pitch.rs:369-374](../../crates/ampforge-dsp/src/fx/pitch.rs#L369-L374)), desalinhadas do seco.
- A detecção roda a cada 512 amostras ([pitch.rs:353](../../crates/ampforge-dsp/src/fx/pitch.rs#L353)), numa autocorrelação O(lags × 1024) feita inteira numa só chamada.
- O README diz "< 1,5 ms".

**Decisão:**
1. O seco passa a ser a amostra de entrada atual, sem atraso.
2. Todas as vozes *wet* usam o mesmo atraso central de `grain_len/2` (512·`sr_scale`).
   - Latência *wet* = 10,7 ms a 48 kHz. É latência do efeito, como no equipamento real.
   - `algorithmic_latency_samples()` do pitch passa a ser **0** (caminho seco). A telemetria expõe `pitch_wet_latency_ms`.
3. Autocorrelação amortizada:
   - cada chamada calcula `ceil(total_lags × frames / 512)` lags, fechando exatamente uma detecção a cada 512 amostras;
   - crie o contador `ops_ultima_chamada: u32`, sempre compilado.

**Testes:**
- `aud_12_seco_sem_atraso`: mix de 50% e 0 semitom; com um impulso, o primeiro pico sai na amostra 0 (hoje sai na 1024).
- `aud_12_latencia_wet`: pico *wet* em 512·`sr_scale` ± 2 amostras.
- `aud_12_custo_distribuido`: harmonizer, bloco 64, 5 s; `max(ops)/média(ops) ≤ 1,25` (hoje é cerca de 8).
- `aud_12_zero_alloc`.

**Pronto quando:** os testes passam e a bancada `pitch` com bloco 64 a 48 kHz mostra p99 ≤ 50% do orçamento.

#### AUD-13 · P2 · Latência reportada diferente da medida
**Evidência:**
- `algorithmic_latency_samples = amp + pitch_1` ([audio.rs:4091](../../crates/sonix-desktop/src/audio.rs#L4091)).
- O amp declara 4, 8 ou 16 amostras, valor fixo por fator de oversampling, que não corresponde ao atraso de grupo dos filtros IIR.

**Decisão:**
- Em `prepare()`, fora do tempo real, passe um impulso por uma instância de rascunho da cadeia padrão em regime linear.
- A latência é a posição do pico, arredondada. Guarde o valor e devolva-o em `algorithmic_latency_samples()`.
- Esse valor alimenta a telemetria e o plugin (AUD-16).

**Teste:** `aud_13_latencia_reportada_igual_medida`: para oversampling 1, 2, 4 e 8, a 44,1, 48 e 96 kHz, |reportada − medida| ≤ 1 amostra.

**Pronto quando:** as 12 combinações passam.

#### AUD-14 · P2 · Filtros de oversampling cortam agudos
**Evidência:**
- O par de filtros do amp ([amp.rs:1001-1055](../../crates/ampforge-dsp/src/amp.rs#L1001-L1055)) dá, por cálculo, −1,0 dB @ 10 kHz, −4,8 dB @ 15 kHz e −8,8 dB @ 18 kHz.
- Nenhum teste pega isso.

**Decisão:**
- Use filtros elípticos de 8ª ordem, gerados offline pelo script versionado `tools/gerar_filtros_os.py` (`scipy.signal.ellip(8, 0.1, 70, …, output="sos")`).
- Um conjunto por `fs_base` ∈ {44,1k, 48k, 88,2k, 96k} × fator ∈ {2, 4, 8}.
- Os coeficientes entram como constantes em Rust.
- **Requisitos:** perda ≤ 0,5 dB até 16 kHz e ≤ 1,5 dB em 18 kHz; atenuação ≥ 60 dB a partir de 0,6·`fs_base`.

**Testes:**
- `aud_14_banda_passante`: par sobe→desce com não linearidade identidade, nos requisitos acima, a 44,1 e 48 kHz.
- `aud_14_banda_de_rejeicao`: filtro isolado, ≥ 60 dB em 0,6·`fs_base`, 0,75·`fs_base` e 0,9·`fs_base` (frequências na taxa sobreamostrada).

**Pronto quando:** os dois testes passam para todas as combinações.

#### AUD-15 · P2 · Pico de CPU da convolução com buffer < 128
**Evidência:** em `cab.rs`, todo o trabalho de FFT cai na fronteira de 128 amostras. Valores medidos: ver a tabela 1.2.

**Decisão:**
1. Na fronteira de 128 amostras, faça só:
   - a FFT direta do último bloco;
   - o MAC da partição 1;
   - a FFT inversa.
2. Os MACs das partições 2 a P do período seguinte são distribuídos igualmente pelos sub-blocos do período atual (`ceil((P−1)·frames/128)` por chamada), acumulando num espectro pré-alocado.
3. Use FFT **real** (256 → 129 bins) no lugar da FFT complexa.
4. Crie um contador de MACs por chamada.

**Testes:**
- `aud_15_saida_igual_a_convolucao_direta`: IR aleatória de 0,49 s, ruído branco, blocos 32, 37, 64, 128 e 480; erro relativo ≤ 1e-5 contra uma convolução direta em f64 feita no teste.
- `aud_15_custo_distribuido`: bloco 32, IR com 512 partições; `max(MACs)/média(MACs) ≤ 2,0`.
- `aud_15_zero_alloc`.

**Pronto quando:** os testes passam e a bancada `pesada` a 96 kHz com bloco 32 mostra **0** blocos acima de 100% do orçamento, descontado o piso do `ruido`.

### FASE 4 — Funcionalidades anunciadas que não existem em produção

#### AUD-16 · P1 · Plugin VST3/CLAP
**Evidência:** em [sonix-plugin/src/lib.rs](../../crates/sonix-plugin/src/lib.rs):
- O `process` (l. 481-523) chama `DesktopMidiHost::default_midi_modifiers()` a cada MIDI CC: **45 alocações** por evento.
- Os testes 012 e 049 exercitam `SonixPluginProcessor` (l. 41), que **não** é exportado.
- Do `SonixProPlugin` exportado (l. 367), os testes só cobrem funções constantes, como `uses_embedded_webview_ipc() -> true` (l. 404).
- `rev_1.mix` (l. 335) é automatizável e nunca é aplicado.
- Não há `set_latency_samples`.
- O layout é 2→2, mas só `slices[0]` é usado: a entrada R é descartada.
- Não há editor.

**Decisão:**
1. `SonixProPlugin` contém um `SonixPluginProcessor`. O `Plugin::process` só faz isto:
   - copia os eventos MIDI para um array fixo pré-alocado (256 por bloco);
   - monta a entrada mono: L no layout de 1 canal, ou (L+R)/2 no de 2 canais;
   - chama `self.processor.process_block(...)`;
   - retorna.
   Toda a lógica fica no *processor*.
2. A tabela de modifiers é compilada uma vez, em `initialize`. Remova a chamada de `default_midi_modifiers()`.
3. Todos os parâmetros são aplicados, incluindo `rev_1.mix` (`chain.set_rev_1_mix`). O plugin passa valores-alvo (`.value()`); a suavização é feita na cadeia (AUD-11).
4. `AUDIO_IO_LAYOUTS` = `[1 in → 2 out, 2 in → 2 out]`.
5. `context.set_latency_samples(chain.algorithmic_latency_samples())` em `initialize` e a cada mudança do fator de oversampling (valor do AUD-13).
6. Sem editor nesta rodada.
   - Apague `uses_embedded_webview_ipc` e o teste dela.
   - README e inventário dizem: "plugin sem editor próprio; usa a interface genérica de parâmetros do host".
7. `catch_unwind` dentro de `process_block` (AUD-04).

**Testes:**
- `aud_16_process_block_nao_aloca_com_midi`: 64 eventos de CC por bloco → 0 alocações.
- `aud_16_todo_parametro_age`: para cada parâmetro de `SonixPluginParams`, mínimo contra máximo muda a saída em ≥ 0,5 dB RMS; para bool e enum, qualquer diferença conta. Hoje `rev_1.mix` reprova.
- `aud_16_entrada_direita_e_usada`: layout 2→2 com sinal só em R → saída não silenciosa.
- `aud_16_latencia_reportada`: o valor do *processor* é igual ao medido por impulso, ± 1 amostra.
- *Lint* estrutural: o `fn process` de `impl Plugin for SonixProPlugin` chama `self.processor.process_block` e não chama nenhum *setter* da cadeia.
- Validadores: rode `pluginval` e `clap-validator` se estiverem instalados; senão, é o bloqueio aceito 3.

**Pronto quando:** os testes passam e os três builds de *release* compilam.

#### AUD-17 · P1 · MIDI do standalone não recebe mensagens
**Evidência:**
- `connect_port` só guarda o nome e devolve sempre `Ok(())` ([midi.rs:124-127](../../crates/sonix-desktop/src/midi.rs#L124-L127)).
- `process_raw_midi_message` ([midi.rs:209](../../crates/sonix-desktop/src/midi.rs#L209)) empurra para uma `SpscCommandQueue` que a produção não consome.
- A Spec 012 CA-2 e a Spec 036 prometem CC, Program Change e MIDI Clock pelo mesmo validador.

**Decisão:**
1. Adicione a dependência `midir` (0.10.x, *backend* WinMM) em `sonix-desktop`.
2. `connect_port(nome)`:
   - procura a porta por nome exato; se não achar → `Err("porta MIDI não encontrada: <nome>")`;
   - abre com callback;
   - guarda a `MidiInputConnection` viva no *runtime*.
   - `disconnect()` fecha a conexão.
3. O callback do `midir` (thread sem requisito de tempo real) chama `MidiRouter::on_message(&[u8])`, que trata:
   - **CC n:** vira a fonte `MIDI_CC_n` dos modifiers, **pelo mesmo caminho dos parâmetros da UI** (validador da tabela única, serializado com os demais produtores).
   - **Program Change n:** carrega o preset n (base 0) do setlist ativo, pelo mesmo caminho de `presets/load`.
   - **CC 69, valor 0–7:** seleciona a cena 1–8 (`DECISAO_SONIX_CC_CENA = 69`).
   - **MIDI Clock (0xF8):** BPM calculado pela média de 24 *ticks*; envia `dly_1.tempo_bpm` no máximo 10×/s e só quando o valor variar ≥ 0,5 BPM.
   - Start e Stop são ignorados.
4. Apague o caminho morto `process_raw_midi_message` → fila não consumida. Só pode existir um caminho.
5. O comando Tauri de conexão MIDI chama o `connect_port` real. A UI mostra a porta conectada ou o erro.

**Testes:**
- `aud_17_porta_inexistente_falha`.
- `aud_17_cc_chega_ao_motor`: com um *runtime* real, `MidiRouter::on_message(&[0xB0, 1, 127])` contra `[0xB0, 1, 0]` → a saída RMS para a mesma entrada difere em ≥ 3 dB (modifier padrão `midi_cc_1_amp_gain`).
- `aud_17_program_change_carrega_preset`.
- `aud_17_cc69_seleciona_cena`.
- `aud_17_clock_120bpm`: 24 *ticks* espaçados de 20,833 ms → tempo de 120 ± 0,5 BPM.
- `aud_17_zero_alloc` no `process`.
- **Manual:** com um controlador real, ou bloqueio aceito 2.

**Pronto quando:** os testes automáticos passam.

#### AUD-18 · P1 · Calibração não persiste
**Evidência:**
- `CalibrationStore` é um `HashMap` em memória ([calibration.rs:167-192](../../crates/sonix-desktop/src/calibration.rs#L167-L192)).
- `build_runtime` cria um *store* novo e lê a chave `"default"` ([lib.rs:56-60](../../apps/sonix-app/src-tauri/src/lib.rs#L56-L60)), enquanto o salvamento usa `active_device_id` ([api.rs:4653-4656](../../crates/sonix-desktop/src/api.rs#L4653-L4656)).
- Consequência: o app sempre abre em Standby.
- Os testes 019 CA-8 e 050 CA-4 só passam porque injetam a calibração.

**Decisão:**
1. `CalibrationStore::open(dir)` lê e grava `<dir>/calibration.json` (`%APPDATA%\SonixPro\`) neste formato:
   `{ "versao": 1, "dispositivos": { "<device_id>": { "boost": f32, "gate_threshold_db": f32, "salvo_em": "<ISO-8601>" } } }`
2. A gravação é atômica: arquivo temporário + `rename`.
3. `build_runtime` abre o *store* e lê pelo **mesmo id** de dispositivo que vai abrir, que é o mesmo usado no `save`.
4. A pasta é injetável (`build_runtime_with_data_dir`).
5. Remova `CalibrationStore::new()` em memória da produção.
6. JSON inválido: o boot entra em standby, o arquivo é renomeado para `.corrompido` e não há pânico.

**Testes:**
- `aud_18_calibracao_sobrevive_ao_reinicio`:
  - pasta temporária;
  - calibração completa pelo fluxo real da API (begin, silence, strum);
  - descartar o *runtime* e criar outro com a mesma pasta e o mesmo dispositivo;
  - o amp não está em standby e boost e gate são iguais aos gravados.
- `aud_18_chave_de_gravacao_igual_a_de_leitura`: grava com X; boot com X encontra; boot com Y entra em standby.
- `aud_18_arquivo_corrompido`.
- **Substitua a injeção** nos testes 019 CA-8 e 050 CA-4 pela persistência.

**Pronto quando:** os testes passam.

#### AUD-19 · P2 · `.sforgepkg` só existe como biblioteca
**Evidência:**
- Não há uso na UI, no Tauri nem no host, embora a Spec 010 esteja como "implementada".
- O formato serializa bytes como array JSON de inteiros (3–4× maior).
- A importação não tem limite de tamanho (`sonix-core/src/persistence.rs`).

**Decisão:**
1. Formato v2 com bytes em base64; o leitor aceita v1 e v2.
2. Limites de importação: arquivo ≤ 64 MiB, cada recurso ≤ 20 MiB, no máximo 256 recursos.
3. Comandos Tauri `export_package(preset_ids, path)` e `import_package(path)`.
4. Botões "Exportar pacote" e "Importar pacote" na tela de presets, com diálogo de arquivo.
5. Registro e inventário de controles atualizados.

**Testes:**
- `aud_19_ida_e_volta`: presets e IRs idênticos bit a bit.
- `aud_19_v1_continua_lendo`.
- `aud_19_limites`: arquivo de 65 MiB → erro; recurso de 21 MiB → erro.
- JS `AUD-19`: os botões chamam os comandos IPC.

**Pronto quando:** os testes passam.

#### AUD-20 · P2 · Split
**Evidência:**
- `DspParamUpdate::SplitRatio(r) => self.split = SplitNode::new(SplitMode::BalanceAb{ratio:r})` ([audio.rs:3372](../../crates/sonix-desktop/src/audio.rs#L3372)) não tem produtor e, se chegar, troca o modo e zera o estado.
- A razão A/B está fixa em 0,5.
- O DynamicSplit está fixo em −30 dB / 5 ms / 80 ms.
- O crossover é de 1ª ordem ([dag.rs:202-219](../../crates/ampforge-dsp/src/dag.rs#L202-L219)), embora o comentário diga Linkwitz-Riley.

**Decisão:**
1. Parâmetro `split.balance` (0–1) com `SplitNode::set_ratio` **no lugar**, sem recriar o nó. Knob no bloco de split.
2. Parâmetros `split.dyn_threshold_db` (−60 a 0), `split.dyn_attack_ms` (1 a 50) e `split.dyn_release_ms` (10 a 500), com controles na UI.
3. Crossover LR4: dois Butterworth de 2ª ordem em cascata, Q 0,7071, nos caminhos LP e HP.

**Testes:**
- `aud_20_balance_sem_reset`: `razao_transiente ≤ 2,0` e níveis A/B iguais à razão ± 0,2 dB.
- `aud_20_dinamico_obedece_parametros`.
- `aud_20_crossover_lr4`: |LP + HP| = 0 dB ± 0,1 dB de 20 Hz a 20 kHz; inclinação de −24 dB/oitava ± 1 em 4× fc.

**Pronto quando:** os testes passam.

#### AUD-21 · P2 · Roteamento implícito
**Evidência:** [audio.rs:4894-4998](../../crates/sonix-desktop/src/audio.rs#L4894-L4998):
- com o split desligado, `comp_2` e `mod_2` da linha 1 rodam na linha 0;
- o wah é inserido implicitamente na coluna 1;
- nada disso aparece no grid nem está documentado.

**Decisão:**
1. Com o split desligado, os blocos da linha 1 **não processam**. A UI mostra-os esmaecidos, com a dica "Linha 2 inativa (split desligado)".
2. O wah vira bloco explícito do grid (visível, móvel, com bypass), na posição padrão atual (coluna 1). O grafo do IPC e de `/api/graph` o inclui.

**Testes:**
- `aud_21_linha2_inativa_sem_split`: `comp_2` em valores extremos não muda a saída com o split desligado.
- `aud_21_grafo_inclui_wah`.
- JS `AUD-21`: o grid renderiza o bloco de wah.

**Pronto quando:** os testes passam.

### FASE 5 — Fidelidade de circuito

#### AUD-22 · P1 · Tone stack AB763 com C2/C3 trocados
**Evidência:**
- Valores do AB763: `c2 = 100 nF` e `c3 = 47 nF` ([tonestack.rs:66-74](../../crates/ampforge-dsp/src/tonestack.rs#L66-L74)).
- A *netlist* põe `c3` no nó do topo do pot de **bass** (linha 2 da matriz) e `c2` no nó do topo do **mid** (linha 3) ([tonestack.rs:252-262](../../crates/ampforge-dsp/src/tonestack.rs#L252-L262)).
- Isso é o inverso de Yeh & Smith (C2 = capacitor do bass, C3 = do mid) e do próprio [voicing-fontes.md](../amps/voicing-fontes.md) ("C2 (Bass) = 0,1 µF").
- Os stacks Marshall passam porque neles C2 = C3.
- Erro medido: ver a tabela 1.2.
- O 039 CA-1 compara com `analog_tonestack_magnitude_db`, que usa a mesma *netlist*: é tautológico.
- O 039 CA-5 não tem teste.

**Decisão:** troque `c2` e `c3` nas linhas 2 a 4 da matriz (`c2` no nó do bass, `c3` no nó do mid). Os valores dos componentes não mudam.

**Teste** `aud_22_stack_igual_a_yeh`:
- O teste implementa a forma fechada de Yeh & Smith (DAFx-06) **sem chamar nada de `tonestack.rs`**: recebe só os valores dos componentes e as frações de pot já com o taper aplicado.
- Compara, para **todos** os stacks de `tonestack.rs`:
  - a resposta analógica de produção com `r_load = ∞` (tolerância 0,05 dB, de 20 Hz a 20 kHz em 1/3 de oitava);
  - o filtro digital a 48 kHz (tolerância 0,1 dB, de 20 Hz a 5 kHz).
- Grade de knobs: t, m, l ∈ {0; 0,25; 0,5; 0,75; 1}.
- **Substitua** o 039 CA-1 por este teste. O 039 CA-5 aponta para ele.
- Forma fechada, com R1 = pot de treble, R2 = pot de bass, R3 = pot de mid, R4 = resistor de *slope*, C1 = capacitor de treble, C2 = de bass, C3 = de mid e t, l, m = frações:
```
b1 = t*C1*R1 + m*C3*R3 + l*(C1*R2 + C2*R2) + (C1*R3 + C2*R3)
b2 = t*(C1*C2*R1*R4 + C1*C3*R1*R4) - m*m*(C1*C3*R3*R3 + C2*C3*R3*R3)
     + m*(C1*C3*R1*R3 + C1*C3*R3*R3 + C2*C3*R3*R3)
     + l*(C1*C2*R1*R2 + C1*C2*R2*R4 + C1*C3*R2*R4)
     + l*m*(C1*C3*R2*R3 + C2*C3*R2*R3)
     + (C1*C2*R1*R3 + C1*C2*R3*R4 + C1*C3*R3*R4)
b3 = l*m*(C1*C2*C3*R1*R2*R3 + C1*C2*C3*R2*R3*R4)
     - m*m*(C1*C2*C3*R1*R3*R3 + C1*C2*C3*R3*R3*R4)
     + m*(C1*C2*C3*R1*R3*R3 + C1*C2*C3*R3*R3*R4)
     + t*C1*C2*C3*R1*R3*R4 - t*m*C1*C2*C3*R1*R3*R4 + t*l*C1*C2*C3*R1*R2*R4
a0 = 1
a1 = (C1*R1 + C1*R3 + C2*R3 + C2*R4 + C3*R4) + m*C3*R3 + l*(C1*R2 + C2*R2)
a2 = m*(C1*C3*R1*R3 - C2*C3*R3*R4 + C1*C3*R3*R3 + C2*C3*R3*R3)
     + l*m*(C1*C3*R2*R3 + C2*C3*R2*R3) - m*m*(C1*C3*R3*R3 + C2*C3*R3*R3)
     + l*(C1*C2*R2*R4 + C1*C2*R1*R2 + C1*C3*R2*R4 + C2*C3*R2*R4)
     + (C1*C2*R1*R4 + C1*C3*R1*R4 + C1*C2*R3*R4 + C1*C2*R1*R3 + C1*C3*R3*R4 + C2*C3*R3*R4)
a3 = l*m*(C1*C2*C3*R1*R2*R3 + C1*C2*C3*R2*R3*R4)
     - m*m*(C1*C2*C3*R1*R3*R3 + C1*C2*C3*R3*R3*R4)
     + m*(C1*C2*C3*R3*R3*R4 + C1*C2*C3*R1*R3*R3 - C1*C2*C3*R1*R3*R4)
     + l*C1*C2*C3*R1*R2*R4 + C1*C2*C3*R1*R3*R4
H(s) = (b1*s + b2*s^2 + b3*s^3) / (a0 + a1*s + a2*s^2 + a3*s^3)
```
Essa forma já foi validada contra a *netlist* do Marshall (diferença ≤ 0,01 dB, sem carga).

**Prova de *red*:** com a *netlist* atual, o AB763 erra 4–5 dB entre 50 e 200 Hz.

**Pronto quando:** todos os stacks passam.

#### AUD-23 · P1 · Pacote de fidelidade dos amps
**(a) Scoop contado duas vezes no Twin.**
- `create_voicing_filters` aplica `peaking(450, −4,5)` ([amp.rs:820-823](../../crates/ampforge-dsp/src/amp.rs#L820-L823)) e a variante `FenderTmbMidScoop` aplica `peaking(450, −3,5)` ([amp.rs:1170](../../crates/ampforge-dsp/src/amp.rs#L1170)), por cima de um stack que já tem o scoop.
- **Decisão:** os dois viram identidade.
- **Teste:** `aud_23a_twin_sem_scoop_extra`. Na resposta do amp em regime linear (Twin, tone stack em 5), a diferença entre 450 Hz e a média de 200 Hz e 1 kHz é igual à do stack isolado, ± 0,5 dB.

**(b) Vox Top Boost forçado na matriz FMV** ([tonestack.rs:79-93](../../crates/ampforge-dsp/src/tonestack.rs#L79-L93)).
- **Decisão:** implemente a topologia Top Boost (Treble e Bass, sem Middle) no mesmo *solver* nodal, a partir do esquema AC30/6 TB (fonte: [voxac30.org.uk](https://www.voxac30.org.uk/vox_ac30_6_black_panels.html) ou esquema JMI citado com página).
- Se o esquema obtido não tiver **todos** os valores legíveis: mantenha o FMV, nomeie a topologia `DECISAO_SONIX_VOX_TB_APROX_FMV` e rotule-a como aproximação na UI e no inventário.
- **Teste:** `aud_23b_ac30_stack`. Resposta igual a uma análise nodal independente, escrita no teste, ± 0,05 dB. Middle inexistente (AUD-08).

**(c) Stack do Recto.**
- `MesaMarkPreGain` é usado como stack pós-ganho do Rectifier.
- **Decisão:** o Recto passa a usar um FMV **pós-ganho** no *solver*, com os valores da tabela da Spec 057 onde houver. O que faltar vira `DECISAO_SONIX_RECTO_*`.
- **Teste:** `aud_23c_recto_stack_pos_ganho`. O stack fica depois do último estágio de ganho: em sinal grande, o mesmo ajuste de tone produz a mesma variação espectral na saída saturada que em sinal pequeno, ± 1 dB.

**(d) Presence inerte no modo Modern.**
- `eff_nfb = 0` em Modern faz `pres_db = 0` ([amp.rs:1150-1164](../../crates/ampforge-dsp/src/amp.rs#L1150-L1164)), e um controle visível fica sem efeito.
- **Decisão:** em Modern, o filtro de presence usa `nfb_scale = 1` só para ele; a realimentação global continua 0. Constante `DECISAO_SONIX_RECTO_MODERN_PRESENCE`, com comentário: evidência indireta da Mesa (cada modo tem seu pot de presence); esquema PENDENTE.
- **Teste:** `aud_23d_presence_age_em_modern`. Presence de 0 para 10 sobe 4 kHz em ≥ 6 dB (regime linear).

**(e) Modo `Pushed` no Rev F.**
- Esse modo vem do Dual Rectifier de 3 canais (2000).
- **Decisão:** remova-o do modelo Rev F. Presets antigos com `Pushed` carregam no modo padrão do canal e devolvem `ignored_keys`.
- **Teste:** `aud_23e_revf_sem_pushed`.

**Pronto quando:** (a) a (e) passam.

#### AUD-24 · P1 · Entrada LOW do JCM800
**Evidência:**
- No LOW, o código aplica −6 dB e **ainda** o ganho linear do 1º estágio (`s_v1a_in * base_drive`) ([amp.rs:1481-1484](../../crates/ampforge-dsp/src/amp.rs#L1481-L1484)).
- O teste 031 exige LOW = HIGH − 6 ± 1 dB ([contract_031_authentic_amps.rs:215-228](../../crates/ampforge-dsp/tests/contract_031_authentic_amps.rs#L215-L228)), copiando a Spec 055 CA-3a.
- Esse CA contradiz o próprio texto da Spec 055 (l. 19 e 65: o LOW pula o 1º estágio).
- Fontes secundárias concordantes ([Robinette JCM800](https://robrobinette.com/How_the_Marshall_JCM800_Works.htm)): o LOW entra **depois** do 1º estágio, pelo divisor 68k/68k.

**Decisão:**
1. LOW = entrada × 0,5 (divisor 68k/68k) → entra direto no 2º estágio. O 1º estágio não é processado: sem `base_drive`, sem triodo. O HIGH não muda.
2. Exponha a função pura `ganho_estagio1_pequeno_sinal(&self) -> f64`, derivada das constantes do estágio (fórmula do AUD-08, regra 5).
3. **Substitua a expectativa** do teste da 031 (l. 215-228): HIGH − LOW @ 1 kHz, −80 dBFS = `20·log10(ganho_estagio1) + 6,02` ± 1,0 dB.
4. Corrija o texto da Spec 055 CA-3a para essa relação.
5. No `AGENTS.md`, troque o exemplo "entrada LOW fica 6 dB abaixo da HIGH" por "no Plexi 1959, a entrada LOW fica 6 dB abaixo da HIGH". O Plexi continua testado com −6 ± 1 dB.

**Testes:**
- `aud_24_low_jcm800_pula_estagio1`.
- `aud_24_low_plexi_6db`.

**Pronto quando:** os dois passam.

#### AUD-25 · P2 · Seletor de impedância invertido
**Evidência:**
- `imp_factor` = 0,78 em 4 Ω, 1,0 em 8 Ω e 1,24 em 16 Ω ([amp.rs:1335-1339](../../crates/ampforge-dsp/src/amp.rs#L1335-L1339)). Esse fator multiplica o reforço de ressonância.
- O manual 1959HW versionado em `docs/ui-desktop/fontes/marshall-1959HW-owners-manual.pdf` (Tonal Notes) diz: "*the lower the impedance setting … the more loose and more resonant*".

**Decisão:** 4 Ω → 1,24; 8 Ω → 1,0; 16 Ω → 0,78.

**Teste:** `aud_25_impedancia_menor_mais_ressonante`. No Plexi e no JCM800, em regime linear, o ganho em `res_hz` obedece 4 Ω > 8 Ω > 16 Ω, com diferença ≥ 1 dB entre cada degrau.

**Pronto quando:** o teste passa.

#### AUD-26 · P2 · Taper do Middle do Marshall
**Evidência:**
- `map_pot_mid_10pct = 0,35u + 0,65·audio_taper(u)` dá 23,4% da resistência a meio curso ([tonestack.rs:34-38](../../crates/ampforge-dsp/src/tonestack.rs#L34-L38)).
- O manual 1959HW diz: "*The Middle control is a 10% log pot*".
- `audio_taper(0,5)` = 9,1%.

**Decisão:**
- O Middle do Plexi e do JCM800 usa `audio_taper` direto. Apague `map_pot_mid_10pct` se ficar sem uso.
- O valor do pot (22k ou 25k) fica como está, com comentário PENDENTE.

**Teste:** `aud_26_middle_10pct_log`. Com o middle em 5, a resposta é igual à de Yeh (fórmula do AUD-22) com m = 0,0909, ± 0,05 dB.

**Pronto quando:** o teste passa.

#### AUD-27 · P2 · Rede 470k‖470pF do JCM800 e bright cap do Plexi
**JCM800**
- **Evidência:** o *voicing* usa biquads em vez da rede 470k‖470pF que a Spec 055 descreve.
- **Decisão:**
  - implemente a rede como filtro de 1ª ordem `H(s) = Rl / (Rl + (470k ‖ 1/(s·470p)))`;
  - `Rl` é a resistência à terra do nó seguinte segundo o esquema 2203 citado na Spec 055 (cite a fonte);
  - discretize com bilinear e *prewarp*;
  - apague o(s) biquad(s) de *voicing* que essa rede substitui.

**Plexi**
- **Evidência:** o bright cap é uma mistura heurística com HPF de 1200 Hz ([amp.rs:1330](../../crates/ampforge-dsp/src/amp.rs#L1330)).
- **Decisão:**
  - modele o capacitor `Cb` em paralelo com a parte superior do pot Volume I (1 MΩ, taper de áudio): `Ru = (1 − taper(v))·1M`, `Rd = taper(v)·1M` e `H(s) = Rd / (Rd + (Ru ‖ 1/(s·Cb)))`;
  - o valor de `Cb` vem do esquema do 1959 (cite a fonte); sem fonte, `DECISAO_SONIX_PLEXI_CB`.

**Testes:**
- `aud_27_rede_jcm800`: estágio isolado igual à fórmula ± 0,1 dB, de 20 Hz a 10 kHz.
- `aud_27_bright_cap_plexi`: o reforço de 5 kHz em relação a 100 Hz cai monotonicamente com o Volume I, é < 0,5 dB em 10 e bate com a fórmula ± 0,2 dB.

**Pronto quando:** os dois passam.

#### AUD-28 · P1 · RE-201
**Evidência:**
- Tabela de modos em [delay.rs:364-378](../../crates/ampforge-dsp/src/fx/delay.rs#L364-L378): duplicatas 5 = 9, 7 = 10 = 4 e 8 = 11; nenhum modo de cabeça única com mola.
- A mola usa `spring_combs`: reflexões discretas entre 8 e 35 ms, com cauda de −115 a −135 dB após 0,5 s.
- Wow de 0,35 amostra a 1,15 Hz ≈ 0,09 cent, inaudível.
- As razões das cabeças, 1,92/2,83 no código e 1,895/2,845 na Spec 068, não têm fonte.
- Fonte secundária ([MusicRadar](https://www.musicradar.com/news/producers-guide-roland-space-echo)): modos 1–4 só eco; 5–7 cabeça única + mola; 8–10 pares + mola; 11 todas + mola; 12 só mola.

**Decisão:**

| Modo | Cabeças | Mola |
|---|---|---|
| 1 / 2 / 3 | H1 / H2 / H3 | não |
| 4 | H2+H3 | não |
| 5 / 6 / 7 | H1 / H2 / H3 | sim |
| 8 / 9 / 10 | H1+H2 / H2+H3 / H1+H3 (`DECISAO_SONIX_RE201_ORDEM_PARES`) | sim |
| 11 | H1+H2+H3 | sim |
| 12 | nenhuma (eco desligado) | sim |

1. Peso de cada cabeça ativa: 1/√n.
2. A mola passa a ser uma instância interna de `ReverbProcessor` no modelo `TubeSpringReverb`, pré-alocada na struct do delay. Ela recebe a entrada do bloco, em paralelo ao eco, com nível `reverb_volume/10`. Apague `spring_combs`.
3. Wow: `DECISAO_SONIX_RE201_WOW_CENTS = 3,0` (pico, a 1,15 Hz).
4. As razões das cabeças continuam como estão, mas passam a ser constantes `DECISAO_SONIX_`.
5. Nada disso é apresentado como fiel ao hardware.

**Testes:**
- `aud_28_cabecas_por_modo`: na resposta ao impulso, há picos exatamente nos atrasos das cabeças ativas (± 1 amostra) e ≤ −60 dB nos atrasos das inativas. Nenhum par de modos entre 1 e 11 pode ter o mesmo conjunto de cabeças.
- `aud_28_mola_so_de_5_a_12`: nos modos 1–4, `reverb_volume` 0 e 10 produzem saídas idênticas; nos modos 5–12, elas diferem.
- `aud_28_mola_tem_cauda`: com `reverb_volume = 10`, a energia entre 0,5 e 1,0 s fica ≥ −60 dB em relação ao pico.
- `aud_28_wow_audivel`: o desvio de frequência de um seno de 1 kHz é de 3 ± 0,3 cents de pico.

**Pronto quando:** os testes passam.

#### AUD-29 · P1 · Drives
**Evidência:**
- Limiares de diodo "em volts" sem uma convenção V ↔ FS.
- Valores medidos de TS808, RAT e DS-1: ver a tabela 1.2. O DS-1 faz clip suave **assimétrico**, mas o circuito real (par de 1N4148 para a terra) é simétrico e rígido.

**Decisão:**
1. Crie `ampforge-dsp/src/unidades.rs` com `pub const VOLTS_POR_FS: f32 = 1.0`, ou seja, 0 dBFS = 1 V de pico na entrada do pedal. Todo limiar em volts é convertido por essa constante.
2. **TS808** ([ElectroSmash](https://electrosmash.com/tube-screamer-analysis)):
   - estágio de ganho `G(s) = 1 + Zf/Zg`, com `Zf = (51k + D·500k) ‖ 1/(s·51p)` e `Zg = 4,7k + 1/(s·0,047µ)`;
   - clip por 2 × 1N914 antiparalelos na realimentação, com Vf = 0,6 V;
   - passa-baixa fixo de 1k e 0,22 µF (≈ 723 Hz) antes do tone.
3. **RAT** ([ElectroSmash](https://electrosmash.com/proco-rat)):
   - `G(s) = 1 + Rd/Zg`, com `Rd = 100k·taper(D)` e `Zg = (47 + 1/(s·2,2µ)) ‖ (560 + 1/(s·4,7µ))`;
   - limite de GBW do LM308 com Cc = 30 pF, valor do *datasheet* citado com página; sem fonte, `DECISAO_SONIX_LM308_GBW`;
   - clip rígido por 2 × 1N914 para a terra.
4. **DS-1:** clip rígido **simétrico** por 2 × 1N4148 para a terra ([ElectroSmash](https://electrosmash.com/boss-ds1-analysis)).

**Testes:**
- `aud_29_ts808_ganho_do_esquema`: D ∈ {0; 0,5; 1}, em 100 Hz, 1 kHz e 3 kHz, a −80 dBFS; a fórmula fica no teste; tolerância ± 1 dB.
- `aud_29_rat_ganho_do_esquema`: idem.
- `aud_29_ds1_simetrico`: com distortion no máximo, os harmônicos pares ficam ≤ −40 dBc.
- `aud_29_limiar_em_volts`: no TS808 com D = 0, o início do clip (THD = 1%) acontece com uma entrada a ± 2 dB de `0,6 V / |G(1 kHz)| / VOLTS_POR_FS`.

**Pronto quando:** os testes passam.

#### AUD-30 · P1 · Efeitos com nome de equipamento e mecanismo diferente
**Evidência:** ([modulation.rs:213-286](../../crates/ampforge-dsp/src/fx/modulation.rs#L213-L286))
- O "Through-Zero Flanger" tem atraso mínimo de 0,3 ms e nenhum caminho seco atrasado.
- O "Tri-Chorus" soma as vozes em mono.
- O "Rotary" tem um rotor só.
- O "Optical Tremolo" é AM senoidal.
- O vibrato do AC30 age em qualquer canal e só como AM.

**Decisão:**
1. **TZF:**
   - seco atrasado 1,0 ms dentro do bloco de flanger;
   - *wet* varrendo de 0 a 2 ms, cruzando 1,0 ms;
   - opção de polaridade invertida.
2. **Rotary:**
   - dois rotores com crossover LR4 em 800 Hz;
   - corneta com AM + Doppler (atraso modulado);
   - tambor com AM;
   - velocidades lenta e rápida e constantes de aceleração, com a corneta acelerando mais rápido, como `DECISAO_SONIX_ROTARY_*`.
3. **Optical Tremolo:** LFO → modelo de LDR com polo único assimétrico: ataque de 5 ms e soltura de 60 ms (`DECISAO_SONIX_LDR_*`).
4. **Tri-Chorus:** o motor é mono de ponta a ponta. Renomeie para "Chorus 3 vozes (mono)" na UI, no registro e no README.
5. **AC30:**
   - vibrato e tremolo só agem no canal Vib-Trem;
   - Vibrato = modulação de **altura** (atraso modulado);
   - Tremolo = AM;
   - seletor Vib/Trem.

**Testes:**
- `aud_30_tzf_cancela_no_cruzamento`: ruído branco, LFO a 0,05 Hz, mix de 50%, polaridade invertida; a janela de 2 ms com menor RMS fica ≤ −30 dB em relação à entrada.
- `aud_30_rotary_dois_rotores`: a taxa de AM de um tom de 200 Hz é igual à velocidade do tambor ± 5%; a de um tom de 3 kHz, à da corneta ± 5%. Na passagem de lenta para rápida, a corneta chega a 90% da nova velocidade antes do tambor.
- `aud_30_optico_assimetrico`: tempo de subida da envoltória ≠ tempo de descida, por fator ≥ 2.
- `aud_30_ac30_vibtrem`:
  - canal Brilliant com depth máximo → saída idêntica à de depth 0;
  - Vibrato: desvio de frequência de um seno de 1 kHz ≥ 5 cents, envoltória ≤ 0,5 dB;
  - Tremolo: AM ≥ 6 dB, desvio ≤ 0,5 cent.

**Pronto quando:** os testes passam.

#### AUD-31 · P2 · Reverbs
**(a) Nomes.** Plate, Hall, Chamber e Shimmer são Freeverb (constantes idênticas às de Jezar) com fator de escala ([reverb.rs:13-14](../../crates/ampforge-dsp/src/fx/reverb.rs#L13-L14)).
- **Decisão:** na UI, no README e no inventário, descreva-os como "algoritmo Freeverb (Jezar) — voicing <nome>". Nenhuma alegação de emular unidades específicas. **[DOC + UI]**

**(b) "Drip" da Tube Spring.**
- **Teste:** `aud_31b_dispersao`. Na IR da mola, |atraso de grupo(3 kHz) − atraso de grupo(300 Hz)| ≥ 5 ms na primeira chegada.
- **Se o teste passar antes da correção:** cole a medição; o achado fica refutado e o nome continua.
- **Se falhar:** implemente a dispersão com uma cascata de ≥ 40 *allpass* de 1ª ordem (coeficiente ≈ 0,6) na malha da mola, conforme Välimäki, Parker e Abel (2010), até o teste passar.

**(c) Decaimento da mola do Twin.**
- O tanque do AB763 é o 4AB3C1B; no código de tanques Accutronics, decay "C" = longo, 2,75–4,0 s. Hoje a estimativa é de cerca de 0,8 s.
- **Decisão:** RT60 da mola do Twin = 3,0 s. Anexe as duas fontes: o esquema AB763 com o código do tanque e a tabela de códigos da Accutronics. Se não conseguir as duas, use a constante `DECISAO_SONIX_TWIN_RT60 = 3,0`.
- **Teste:** `aud_31c_rt60_twin`. `rt60_schroeder` na banda de 500 Hz a 2 kHz dá entre 2,75 e 4,0 s.

**Pronto quando:** (b) e (c) estão resolvidos e (a) está aplicado.

#### AUD-32 · P2 · Gabinete
**Evidência:**
- Com IR ativa, `cab.process` retorna antes ([cab.rs:1019-1022](../../crates/ampforge-dsp/src/cab.rs#L1019-L1022)), e hi-cut e escolha de alto-falante continuam habilitados sem efeito ([Gabinete.ts:513](../../apps/sonix-app/ui/src/screens/Gabinete.ts#L513)). Os microfones já são desabilitados corretamente ([Gabinete.ts:314-318](../../apps/sonix-app/ui/src/screens/Gabinete.ts#L314-L318)).
- Sem IR, o Greenback é um filtro identidade.

**Decisão:**
1. O hi-cut é aplicado **depois** da convolução.
2. Com IR ativa, o seletor de alto-falante fica desabilitado, com a dica "IR carregada substitui o alto-falante". O controle de ressonância continua, porque age no amp.
3. Curvas dos alto-falantes:
   - cada alto-falante usa uma curva amostrada em 1/3 de oitava do gráfico de resposta do *datasheet* do fabricante (Celestion), com a URL do PDF citada;
   - a curva é implementada como FIR de fase mínima calculado em `prepare`;
   - nenhum alto-falante pode ser identidade.

**Testes:**
- `aud_32_hicut_age_com_ir`: com IR ativa, levar o hi-cut do máximo a 5 kHz atenua 8 kHz em ≥ 6 dB.
- JS `AUD-32`: o seletor de alto-falante fica desabilitado com IR.
- `aud_32_alto_falantes_seguem_datasheet`: resposta dentro de ± 2 dB dos pontos amostrados.

**Pronto quando:** os testes passam.

### FASE 6 — Afinador

#### AUD-33 · P2 · Faixa, precisão e nomes
**Evidência:**
- O `max_lag` cobre só até 58 Hz ([tuner.rs:183-184](../../crates/ampforge-dsp/src/tuner.rs#L183-L184)), e o corte em `freq < 60.0 || freq > 1400.0` devolve "---" ([tuner.rs:290](../../crates/ampforge-dsp/src/tuner.rs#L290)).
- README e Spec 029 prometem 40 Hz.
- A precisão anunciada de ±0,1 cent só se verifica até 330 Hz.
- O modo "estroboscópico" é YIN.
- Os offsets "Peterson GTR" não têm fonte.
- `tuner_scratch` tem 4096 amostras ([api.rs:715](../../crates/sonix-desktop/src/api.rs#L715)).

**Decisão:**
1. Faixa de 38 a 1400 Hz: cobre o E1 (41,2 Hz); o B0 (30,9 Hz) fica de fora e isso é documentado.
2. `max_lag = sr/38`. `tuner_scratch` passa a 8192 amostras (fora do tempo real).
3. Para frequências acima de 330 Hz, refine τ pelo pico da autocorrelação em k·τ, usando o maior k que caiba na janela, e divida por k.
4. "Estroboscópico" passa a se chamar "Visual estroboscópico (detecção YIN)".
5. Os offsets "Peterson GTR" viram "Temperamento suavizado SONIX", com os valores como `DECISAO_SONIX_`. A marca sai.

**Testes:**
- `aud_33_detecta_de_41_a_1400`:
  - notas E1, F#1, A1, B1, E2, A2, D3, G3, B3, E4, E5, E6 e 1400 Hz;
  - seno puro e sinal harmônico (8 harmônicos com amplitude 1/n);
  - a 44,1, 48 e 96 kHz;
  - critério: |erro| ≤ 0,1 cent.
- `aud_33_abaixo_de_38_nao_detecta`: 30 Hz → "---".

**Pronto quando:** os testes passam.

#### AUD-34 · P2 · Afinador polifônico
**Evidência:** a corda D3 afinada lê +16,6 cents quando há harmônicos; só com fundamentais, ≤ 1,3 cent.

**Decisão:**
- Por corda, use janela ≥ 16 384 amostras (48 kHz), Hann, com *zero-padding* 4×.
- Estime a fundamental e o 2º harmônico **da própria corda** dentro de ±60 cents do esperado, combinando-os por peso de magnitude.
- Exclua parciais que coincidam (±15 cents) com harmônicos n ≤ 8 de cordas mais graves na afinação padrão. Se os dois parciais forem excluídos, use o 3º.
- **Se o método não atingir o critério, troque o método. Nunca o limiar.**

**Teste:** `aud_34_poli_com_harmonicos`.
- Sinal: 6 cordas, 8 harmônicos de amplitude 1/n, fases aleatórias.
- Cenários: todas afinadas; e cada corda desafinada independentemente em ±10 cents.
- Critério: ±3 cents por corda.

**Pronto quando:** o teste passa.

### FASE 7 — Testes que não medem

#### AUD-35 · P1 · Testes tautológicos, de mock ou que se autoconferem
Para cada item da lista, prove o *red* quebrando o comportamento **real**. Exemplo: destrocar C2/C3 tem de reprovar o `aud_22`.

1. **039 CA-1** → substituído pelo AUD-22.
2. **039 CA-2** mede só o filtro AA analítico. Novo teste ponta a ponta: a resposta do amp em regime linear a 44,1, 48, 88,2 e 96 kHz é igual entre as taxas, ± 0,5 dB, de 20 Hz a 16 kHz.
3. **018 CA-11 e CA-12** e o "WASAPI exclusivo" de 018/028 rodam no `SimulatedAudioHost`.
   - CA-11 e CA-12 → substituídos pelos testes reais do AUD-04 e do AUD-05.
   - Os testes de modo exclusivo são apagados junto com a correção de estado do AUD-39.
4. **035 CA-8** gera os próprios fixtures e regrava arquivos versionados.
   - Separe em um gerador (`cargo run -p sonix-desktop --example gerar_fixtures_ipc`, que grava) e um teste (gera em memória, compara com o versionado e reprova se houver diferença).
   - Depois corrija a deriva atual (`presets_load.json:63`, `v30_4x12_closed` contra `v30_4x12`):
     - descubra qual id o registro e a tela Gabinete aceitam;
     - o backend precisa emitir um id que a UI aceite;
     - corrija o lado que diverge e regenere.
5. **049 CA-16 e CA-22** (funções constantes do plugin) → substituídos pelos testes do AUD-16.
6. **019 CA-8 e 050 CA-4** (calibração injetada) → AUD-18.
7. **009 CA-3** (`count_active_channels_in_scene` devolve a constante 1), **009 CA-8** (confere string vazia) e **`GaplessCrossfader`** (ignora `crossfade_samples`):
   - apague as funções de `sonix-core` que a produção não usa e liste-as em [biblioteca-sem-uso-em-producao.md](../biblioteca-sem-uso-em-producao.md);
   - reescreva os testes da 009 contra a produção:
     - troca de cena em `RealtimeDspChain` com `razao_transiente ≤ 2,0`;
     - os 4 canais por bloco comutam.
8. **Contratos JS 011–030** testam a casca legada `design-system/`.
   - Monte a tabela CA → teste legado → teste da UI Tauri.
   - Para cada CA de UI sem teste da UI Tauri, escreva-o contra `apps/sonix-app/ui`.
   - Se falhar porque o recurso não existe na UI Tauri, **implemente o recurso**.
9. **JS 048** grava PNGs em `tests/fixtures/048-diffs` → passa a gravar em `os.tmpdir()`.

**Pronto quando:** os 9 pontos estão resolvidos e o `git status` fica limpo depois do portão.

#### AUD-36 · P2 · CAs sem teste e E2E sem executor
1. Para cada CA abaixo: leia, escreva o teste no arquivo de contrato da spec, com nome `caN_…` e `// AUD-36`; se ele falhar, corrija o código.
   - 033 CA-3;
   - 039 CA-5 (via AUD-22);
   - 044 CA-4;
   - 047 CA-5 (hoje só tem cobertura indireta);
   - 049 CA-15.
   - O 033 CA-12 sai pelo AUD-05.
2. **CAs manuais 018 CA-10 e 022 CA-14:** escreva o procedimento, com passos e números esperados, em `docs/verificacao-manual/018-CA-10.md` e `022-CA-14.md`. Situação: "aguardando execução pelo dono", o que conta como bloqueio aceito 2.
3. **E2E 033:** `apps/sonix-app/e2e/033.e2e.ts` não tem executor; `runSpec033E2e` só é exportada.
   - Crie `npm run test:e2e`, que roda o E2E contra a UI construída, com *fixtures* de IPC, usando a mesma infraestrutura de captura do JS 048.
   - Ele passa a fazer parte do portão.

**Pronto quando:** todos os testes existem e passam e o `test:e2e` roda no portão.

#### AUD-37 · P3 · 048 sem a tela Áudio
- **Evidência:** a tela Áudio não tem SSIM nem checagem de DOM. A tela Main está com 0,855 contra limite de 0,85.
- **Decisão:** inclua a tela Áudio (região e tela inteira, com os mesmos limiares) e checagem de DOM. Registre a margem da Main no relatório.

**Pronto quando:** o teste passa.

### FASE 8 — Documentação e estados

#### AUD-38 · P0 · [DOC] Evidência fabricada nas specs 053–092
**Evidência:**
- `git log --all -S` encontra **0 commits** com `pre_filter`, `post_filter`, `set_lowpass`, `muff_lpf`, `det_level`, `Eq8Processor`, `GraphicEqProcessor`, `TremoloOptical`, `bias_shift`, `rat_filter_state`, `re201_head_gains` ou `TapeRe201`.
- Mesmo assim, esses nomes são citados como "erro factual no código" em:
  - 059 §11 (l. 117–119), 060 l. 112–116, 061 l. 118–120, 062 l. 123, 064 l. 118–119;
  - 066 l. 115, 067 l. 115, 073 l. 100, 086 l. 87, 088 l. 92.
- A 068 (l. 134–136) diz que os modos 5–12 não têm mola e que o modo 12 é mudo. É falso.
- As specs 059–092 não têm `status:`.
- Há *enums* inexistentes (`TapeRe201`, `DigitalClean`, `AnalogBbd`, `PingPong`), `ProCoRat` na spec do Fuzz Face e `muff_lpf/muff_hpf` na spec do DS-1.

**Decisão:**
1. Em cada spec 053–092, toda afirmação sobre o código passa a ter `arquivo:linha @ <hash>` mais o trecho colado, conferido com `git show <hash>:<arquivo>`.
   - Afirmação com identificador inexistente: apague-a e deixe a linha "Removido em 2026-11-20 (AUD-38): identificador inexistente em qualquer commit".
2. Acrescente `status: rascunho` ao *front-matter* das 059–092.
3. Corrija os identificadores copiados de outras specs.
4. Reescreva a seção de modos da 068 conforme os fatos do AUD-28.
5. Crie `tools/verificar_specs.py`, somente leitura, que confere citações `arquivo:linha`, identificadores e *front-matter*. Rode e cole a saída.

**Pronto quando:** o script relata 0 citações inválidas em 001–092 e 0 identificadores inexistentes apresentados como código atual. Nenhuma implementação das 053–092.

#### AUD-39 · P1 · Estados inflados
1. **Spec 028:** status "parcial — WASAPI exclusivo/IAudioClient3 não implementado (`has_mode_support(WasapiExclusive) == false`, [audio.rs:825-830](../../crates/sonix-desktop/src/audio.rs#L825-L830))".
   - A UI não oferece modo exclusivo como selecionável: o item aparece desabilitado com a dica "não implementado".
   - Apague os testes simulados de modo exclusivo.
2. **Spec 053, matriz:**
   - **CA-4** vira TESTED com o novo teste `ca4_invariancia_ao_tamanho_de_bloco` (`// AUD-39`), em arquivo `contract_053_*.rs`:
     - cadeia padrão e pesada;
     - blocos 32, 37, 64, 256, 480, 512 e 1024 contra 128;
     - critério: saída bit a bit idêntica (a auditoria já verificou que a propriedade vale);
     - prova de *red*: injete temporariamente uma dependência do tamanho do bloco e mostre a falha.
   - **CA-9** vira "NÃO ATENDIDO — o amp limita em ±0,999".
3. **Spec 040:** o estado reflete o AUD-30.
4. **`specs/README.md`:** índice coerente com o que for corrigido.
5. **`AGENTS.md`:** reescreva a linha "Estado atual" **com os números da execução final do portão**, com a lista de bloqueios aceitos e sem a frase "auditoria documental concluída".

**Pronto quando:** o teste do CA-4 passa e os estados estão coerentes.

#### AUD-40 · P1 · Afirmações públicas contraditas pelo código
Escopo: README, [inventario-controles.md](../ui-desktop/inventario-controles.md), índice das specs e `AGENTS.md`.
1. **"Solvers Newton-Raphson para triodos":** [precision.rs:55-95](../../crates/ampforge-dsp/src/precision.rs#L55-L95) resolve x^1,5 = v^1,5 partindo de x = v (converge em zero passos) e não é usado. Apague o *solver* e a frase. Descreva a função de triodo real `u/√(1+(ku)²)`, com as constantes como `DECISAO_SONIX_`.
2. **"FIR polifásico > 96 dB":** troque pelos números medidos no AUD-14. Liste o `HalfBandOversampler` em [biblioteca-sem-uso-em-producao.md](../biblioteca-sem-uso-em-producao.md).
3. **Afinador:** a faixa e a precisão passam a ser as medidas no AUD-33.
4. **"Testes 100%":** troque pelo comando e pelas contagens da execução final.
5. **"Tabela única de parâmetros":** passa a ser verdade depois do AUD-09.
6. **"< 1,5 ms"** (README e regra do `AGENTS.md`): troque pela latência que a telemetria calcula, explicando a fórmula.

**Pronto quando:** cada frase corrigida aponta para o teste ou a medição que a sustenta.

#### AUD-41 · P1 · [DOC] Fontes
1. As 7 linhas `PRIMARY_OFFICIAL` sem link (Roland Service Notes, Leslie 147, Mesa, Fender AA1164, Eventide H3000, patente ISP) viram `UNKNOWN` até haver um link com página.
2. A página do plugin Roland Cloud usada como fonte do hardware de 1974 vira `SECONDARY`.
3. `mesaboogie.com/support/user-manuals.html` e o artigo Amplitudes de 2010 redirecionam para a Gibson: marque como inacessíveis.
4. As *homepages* da Sennheiser, Neumann, AKG e Beyerdynamic deixam de contar como fonte de curva.
5. Os manuais de reedição (1959HW e 2203 reissue) viram `SECONDARY` para as revisões vintage.
   - Exceção: as "Tonal Notes" do 1959HW, que reproduzem o texto da SL/A anterior a julho de 1969, ficam `PRIMARY` para essas notas.
   - O manual 2203 versionado é só imagem: registre isso.
6. O link 404 (`geofex …/uvtech.htm`) sai.

Escopo: [voicing-fontes.md](../amps/voicing-fontes.md), [cab-microfones-fontes.md](../cab-microfones-fontes.md) e as tabelas de fontes das specs.

#### AUD-42 · P3 · [DOC] Higiene documental
1. As 13 citações `arquivo:linha` desatualizadas nas specs 018, 029, 033, 035, 040, 043, 044 e 048 passam a apontar para a linha atual, com `@ <hash>`.
2. Todo link `file:///g:/...` vira link relativo: ≥ 53 em `specs/`, `docs/`, `AGENTS.md` e `README.md`; procure também em `.agents/`.
3. Corrija o link da 083 para o nome real do arquivo da Spec 026.
4. Troque "Specs 001 a 050" no README pela faixa e pelos estados reais.
5. Alinhe o título da 029 (40 Hz) e o texto da 026 (60 Hz) à faixa medida no AUD-33.

#### AUD-43 · P3 · [DOC] Licença
No README:
- o repositório está sob CC BY-NC 4.0;
- binários VST3 gerados com `nih-plug`/`vst3-sys` ficam sujeitos à GPLv3 e **não são distribuídos**;
- é um projeto pessoal, sem uso comercial.

Não faça nenhuma alegação jurídica além disso.

### FASE 9 — Manutenção

#### AUD-44 · P3 · `nih_plug` sem `rev`
- **Decisão:** em `crates/sonix-plugin/Cargo.toml`, acrescente `rev = "<hash fixado hoje no Cargo.lock>"`.
- **Pronto quando:** `cargo build -p sonix-plugin --locked` passa.

#### AUD-45 · P3 · Acoplamento e duplicação
1. Crie a *feature* `host-audio` (`cpal`) em `sonix-desktop`, ativa por padrão. O plugin depende com `default-features = false`.
   - **Pronto quando:** `cargo tree -p sonix-plugin | grep cpal` não retorna nada.
2. Unifique o `BiquadFilter` duplicado em 6 módulos em `ampforge_dsp::biquad`.
   - **Teste dourado:** capture a saída de cada processador antes da mudança; depois dela, a saída é bit a bit idêntica.

#### AUD-46 · P3 · Desempenho
1. Os 7 biquads identidade que rodam a cada subamostra ([amp.rs:1037-1049](../../crates/ampforge-dsp/src/amp.rs#L1037-L1049)) deixam de ser executados.
   - **Teste dourado:** saída bit a bit idêntica.
2. Se o AUD-15 não tiver trocado a FFT complexa pela real, troque aqui.
   - Tolerância: 1e-6.
3. Na bancada, a CPU média dos cenários `padrao` e `pesada` não pode piorar.

#### AUD-47 · P3 · [DOC] 1176
No inventário: "detector e estágio de entrada são aproximação comportamental; tempos de attack e release conforme o [manual da UA](https://media.uaudio.com/assetlibrary/1/1/1176ln_manual.pdf)".

---

## 5. Fora desta ordem de serviço (não iniciar)
- Implementar o WASAPI exclusivo ou o IAudioClient3. Nesta rodada só se corrige o estado (AUD-39).
- Build ASIO, medição por cabo de *loopback* (T-25) e validadores externos (bloqueios aceitos).
- Editor gráfico do plugin.
- Qualquer item das specs 053–092 além da correção documental do AUD-38.

---

## 6. Relatório final obrigatório
Arquivo: `docs/handoff/RELATORIO-auditoria-2026-11-20-execucao.md`. Ele tem **exatamente** estas seções:

1. **Tabela por item** com as colunas: `AUD | Estado | Commits | Testes (nomes) | Red (comando + trecho) | Green (comando + trecho) | Medição antes → depois`.
   - `Estado` só pode ser **FEITO** ou **BLOQUEIO ACEITO n** (1, 2 ou 3 da §0.7).
2. **Execução final do portão:** cada comando da §2.3 com o resumo de contagem (aprovados, reprovados, ignorados = 0) e o `git status --porcelain` final.
3. **Bancada (§2.4):** média, p99 e máximo por cenário, taxa de amostragem e bloco, junto com o piso do cenário `ruido`.
4. **Decisões `DECISAO_SONIX_` criadas:** constante, valor, `arquivo:linha` e o que a confirmaria.
5. **Objeções:** só com medição. Uma objeção sem número não é aceita.
6. **`git log --oneline main..fix/auditoria-2026-11-20`.**

O relatório não leva introdução, resumo executivo, adjetivos nem "próximos passos".
