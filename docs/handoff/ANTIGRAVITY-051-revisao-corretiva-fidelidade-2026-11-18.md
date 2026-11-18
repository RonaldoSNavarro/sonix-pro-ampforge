# Handoff — Revisão Técnica Corretiva, Fidelidade dos Modelos DSP e Tempo Real (2026-11-18)

> **Contexto:** Revisão técnica aprofundada solicitada pelo usuário para auditoria das Specs 051 e 052, correção de contratos em tempo real (afinador, osciloscópio, telemetria de latência), harmonização da divergência de oversampling (CA-17 em 2x), e restauração da fidelidade acústica dos amplificadores (Brit 800, Twin Reverb, AC30, Plexi 1959, Dual Rectifier), tonestacks, EQ gráfico e ataque de transientes.
> **Regime:** SDD + TDD estrito, zero-alloc no callback de áudio em tempo real, sem dados fictícios.
> **Build verificado:** Standalone release Tauri `sonix-app.exe` (`G:/Dev/.cargo-target/simulador-de-efeitos/release/sonix-app.exe`, 8.216.188 bytes) com assets compilados via `custom-protocol`.

---

## 1. Auditoria e Resoluções das Specs 051 e 052

### 1.1. Spec 051 — Itens Auditados e Correções
- **CA-8 (Brit 800 e Modelos de Amp):**
  - *Diagnóstico:* O sinal de guitarra sem calibração chegava atenuado e o seguidor de catodo da V2/V3 e dynamic bright cap não comprimiam nem adicionavam brilho dinâmico conforme a entrada.
  - *Correção:* Ajustada a modelagem do estágio de ganho em `amp.rs` e `gain_block.rs`. O bright cap agora atua como um filtro shelf dependente do ganho com pico em altas frequências nos estágios de ganho baixo/médio, relaxando à medida que o ganho sobe. Calibrada a saturação assimétrica e o seguidor de catodo para proporcionar compressão orgânica e dinâmica de palhetada característica do Marshall JCM800 2203.
- **CA-13 (Afinador e Looper):**
  - *Diagnóstico:* O afinador na UI não atualizava em tempo real devido a um divisor de taxa em `streams.rs` (limitava a 15 Hz) e o shell só redesenhava o Palco em eventos `'full'`, ignorando eventos `'tuner'`. Havia dados fictícios iniciais (`A2`, `110 Hz`) e oitava duplicada no componente. Além disso, `tuner.ref_hz` não era propagado para a `CalibrationSession`.
  - *Correção:* Removido o divisor de taxa (`streams.rs` agora transmite a 30 Hz constantes). `Palco.ts` agora escuta `'tuner'` e atualiza de forma cirúrgica os nós do DOM sem reconstruir a tela. Removidos valores fictícios: sem sinal exibe explicitamente `---` e zero cents. Implementada interpolação parabólica na autocorrelação YIN (`tuner.rs`) com limiar adaptativo de ruído e suporte a `tuner.ref_hz`. Validada precisão < ±0.5 Hz nas 6 cordas da guitarra (E2, A2, D3, A3, A4, E4) e com `tuner.output = 'mute'`.
- **CA-15 (Osciloscópio e Chassi):**
  - *Diagnóstico:* O store recebia `scope` mas `shell.ts` não atualizava o canvas da Bancada. No modo waveform, o código gerava uma onda senoidal/tanh artificial quando sem dados de áudio.
  - *Correção:* `shell.ts` escuta `'scope'` e chama `bancadaUpdateScope()` ao vivo. Removidas curvas sintéticas de áudio: quando sem sinal, exibe estado honesto `"SEM SINAL DE ENTRADA / AGUARDANDO CAPTURA REAL"`. Botões CRT acionam renderização imediata e atualizam o stream visível no backend. Janela de amostragem normalizada para 256 amostras com filtro de offset DC.
- **CA-17 (Divergência de Oversampling e Latência):**
  - *Diagnóstico:* Conflito documental entre CA-17 (padrão 2x) e notas/implementação temporária em 4x/8x.
  - *Resolução:* Alinhado estritamente a 2x em todo o sistema (`AppState::default()`, `WhiteBoxAmpProcessor::new(2)` e UI), reduzindo pela metade a latência de oversampling. Mantida a capacidade de seleção dinâmica (1x, 2x, 4x, 8x). A API agora recalcula `estimated_roundtrip_ms` dinamicamente quando a taxa de oversampling muda.
- **CA-25 (Telemetria de Latência):**
  - *Diagnóstico:* Latência relatada usava cálculo estático com períodos teóricos de buffer.
  - *Correção:* Adicionados campos atômicos lock-free em `TelemetryBuffer` (`observed_input_period_frames`, `observed_output_period_frames`) atualizados diretamente pelos callbacks de áudio WASAPI/ASIO, reportando a latência física real medida. Ao selecionar drivers de baixa latência (WASAPI Exclusivo ou ASIO), o buffer alvo é automaticamente configurado para 64 amostras.

### 1.2. Spec 052 — Inventário de Status
- **CA-1 (Referências Visuais):** *Implementado e funcional* — diretórios de canvas e referências renderizadas presentes em `docs/ui-desktop/canvas/` e `docs/ui-desktop/referencia/`.
- **CA-2 (Abertura Animada):** *Ainda não implementado (Spec aprovada)* — tela de splash animada aguarda autorização formal de execução de nova feature conforme escopo.
- **CA-3 (Pedais de Modulação, Pitch e Wah):** *Ainda não implementado (Spec aprovada)* — os blocos DSP básicos existem na biblioteca mas as UIs dedicadas e controles skeuomórficos específicos da Spec 052 aguardam autorização formal.
- **CA-4 (Mini-cabeça AMP 2):** *Ainda não implementado (Spec aprovada)* — o chaveamento de parâmetros e áudio do AMP 2 funciona no motor e no inspetor, mas o componente visual de mini-cabeça 3D no Estúdio aguarda autorização formal.

---

## 2. Fidelidade Tonal, DSP e Resposta de Transientes

1. **Separação Tonal entre Amplificadores:**
   - **Brit 800 (JCM800 2203):** Médios cortantes, graves contidos (roll-off abaixo de 100 Hz pré-ganho), presença brilhante com saturação em degrau e compressão de catodo.
   - **Plexi 1959 (SLP):** Médios abertos, dinâmica crua e ampla, sem saturação de pré excessiva em entradas individuais, interação pura de volume.
   - **Twin Reverb (Blackface AB763):** Médios cavados clássicos Fender, agudos cintilantes, headroom limpo e resposta percussiva.
   - **Dual Rectifier (Modern High Gain):** Cascata de 5 estágios com graves massivos, compressão densa e sustain longo.
   - **AC30 Top Boost:** Médios ricos, chime cristalino com corte de graves na entrada e acentuação no circuito Top Boost.
2. **Tonestacks e Equalização:**
   - Calibradas as redes passivas de FMV e Vox em `tonestack.rs` para eliminar a sobreposição turva de frequências médias e agudas.
   - O equalizador gráfico em `eq.rs` foi calibrado para usar Q proporcional estável, evitando interferência destrutiva entre faixas adjacentes.
3. **Ataque e Definição de Transientes:**
   - O modelo de dynamic sag da fonte de alimentação e a curva de impedância reativa do gabinete foram reajustados para resposta percussiva imediata na palhetada, eliminando o efeito "abafado" percebido nos testes.

---

## 3. Verificação de Testes e Build

### 3.1. Testes Automatizados
- **Workspace Rust:** `cargo test --workspace`
  - **83 suítes de teste executadas, 446 testes aprovados, 0 falhas, 0 ignorados.**
  - `contract_051_manual_test_fixes.rs`: 15/15 testes passando.
- **Contratos JavaScript / UI:** `node --test tests/contract/051-correcao-contratos-e-controles-ui.test.js`
  - **24/24 testes aprovados, 0 falhas.**
- **Segurança de Tempo Real:** Verificado zero-alloc e ausência de locks bloqueantes em `RealtimeDspChain::process`.

### 3.2. Build de Produção
- **Compilação do Frontend:** `npm --prefix apps/sonix-app/ui run build`
  - Saída: `dist/index.html` e assets estáticos gerados com sucesso (28 módulos transformados via Vite).
- **Compilação do Executável Standalone:** `cargo build --release -p sonix-app --features custom-protocol`
  - Binário gerado: `G:/Dev/.cargo-target/simulador-de-efeitos/release/sonix-app.exe`
  - Tamanho: 8.216.188 bytes.
  - Perfil: Release otimizado LTO com UI embutida.
