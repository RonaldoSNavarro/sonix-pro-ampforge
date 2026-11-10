# SONIX PRO – AMPFORGE | Arquitetura de referência (Rust + Windows)

**Perfil confirmado do produto**
- Linguagem principal: Rust.
- Aplicação standalone para Windows 10 e Windows 11.
- WASAPI e ASIO são requisitos do primeiro release (não apenas itens de roadmap).
- Suporte a múltiplas taxas de amostragem, negociadas conforme as capacidades reais do dispositivo selecionado.
- Configuração dinâmica e independente de canais de entrada e saída, limitada aos formatos expostos pelo dispositivo/backend.
- Modos de entrada para instrumento/Hi-Z e linha, incluindo gravação DI e fluxo de reamp.
- Estratégia de fidelidade híbrida: circuit-level + modelagem comportamental + IR.
- Considerar VST3 futuramente; manter o motor DSP e o engine desacoplados da GUI e de qualquer host de plugin.
- Objetivo de áudio: processamento em tempo real para guitarra, com baixa latência medida, estabilidade e resposta dinâmica convincente.

Este documento é uma proposta de arquitetura, não uma descrição de código já existente. Antes de adotar crates ou APIs, confira versões, licenças, compatibilidade e estado de manutenção no repositório do projeto.

## 1. Decisões arquiteturais

1. **Separar o motor DSP da interface.** O núcleo de áudio não deve depender da GUI, do sistema de arquivos, de Windows APIs ou de formatos de preset.
2. **Um grafo de sinal compilado.** A GUI edita uma representação declarativa do rig; uma thread de controle valida e compila essa representação em uma cadeia DSP pronta. A callback recebe a nova cadeia por mecanismo sem bloqueio e troca estados de forma segura.
3. **Tempo real é uma restrição de projeto.** A callback de áudio não pode alocar memória, bloquear mutex, fazer I/O, carregar IR, compilar grafo, criar threads, formatar logs ou executar operações imprevisíveis.
4. **Estado por instância.** Filtros, delays, envelopes, memória de válvulas, buffers de convolução e estados de oversampling pertencem à instância do processador, nunca a variáveis globais compartilhadas.
5. **Parâmetros suaves.** Mudanças contínuas (gain, tone, mix, volume) devem ser suavizadas/interpoladas para evitar zipper noise. Mudanças estruturais (troca de amp/IR/topologia) exigem preparação fora da callback e transição controlada.
6. **Formato interno previsível.** Adotar inicialmente áudio `f32`, processamento em blocos e canais explícitos; converter formatos externos na borda do host.
7. **Medição antes de otimização.** CPU, latência, underruns, aliasing, ruído e resposta devem ser medidos em builds release e em máquinas-alvo.

## 2. Stack candidata

| Área | Candidato inicial | Papel e ressalvas |
|---|---|---|
| I/O de áudio | `cpal` como primeira candidata | WASAPI no Windows e ASIO via feature opcional `asio`. Embora ASIO seja opcional na crate, ele é obrigatório no produto desde o primeiro release. Validar toolchain LLVM/Clang, SDK, drivers, build e distribuição; se CPAL não atender aos requisitos, avaliar adaptador ASIO dedicado atrás da mesma interface `AudioBackend`. |
| GUI desktop | `egui` + `eframe` | Interface nativa em Rust, prototipagem rápida e controles customizados. Avaliar acessibilidade, escala, temas e experiência final antes de congelar a escolha. |
| DSP utilitário | `dasp` e/ou módulos próprios | Tipos e utilidades PCM. Não delegar automaticamente todo o DSP a uma crate genérica; algoritmos críticos devem ter testes e benchmarks próprios. |
| Resampling | `rubato` ou implementação validada | Conversão de sample rate fora da callback sempre que possível; mudanças de taxa em tempo real precisam de projeto explícito. |
| Convolução | Motor FIR próprio inicialmente; avaliar crates especializadas | FIR direta para IR curta/latência mínima; FFT/partitioned convolution para IR longa. Comparar custo e latência com medições. |
| Serialização | `serde` + `serde_json` ou `ron` | Presets, rigs e metadados versionados. Nunca desserializar arquivos de usuário dentro da callback. |
| Erros e logging | `thiserror`/`anyhow` fora do DSP; `tracing` fora da callback | A callback comunica contadores/flags simples; logging e diagnóstico são tratados por thread de controle. |
| Testes | `cargo test`, Criterion e fixtures de áudio | Testes determinísticos, comparação de sinais e benchmarks em release. |

A lista é uma shortlist, não uma ordem para adicionar dependências indiscriminadamente. Fixar versões no `Cargo.lock`, revisar licenças e evitar dependências sem necessidade.

## 3. Visão em camadas

```text
┌──────────────────────────────────────────────────────────────┐
│                    SONIX PRO – AMPFORGE                       │
├───────────────────────────────┬──────────────────────────────┤
│ UI / Application              │ Presets / Assets             │
│ egui + eframe                 │ JSON/RON + IR/WAV            │
│ Rig editor, meters, settings  │ Loader, validation, metadata│
└───────────────┬───────────────┴───────────────┬──────────────┘
                │ comandos/parâmetros            │ carga offline
                ▼                               ▼
┌──────────────────────────────────────────────────────────────┐
│ Control Plane (thread não-áudio)                              │
│ valida parâmetros • prepara IR • compila grafo • troca rig    │
│ gerenciamento de dispositivos • telemetria • persistência    │
└───────────────────────────────┬──────────────────────────────┘
                                │ snapshot / fila bounded
                                ▼
┌──────────────────────────────────────────────────────────────┐
│ Real-time Audio Engine                                        │
│ CPAL callback → input conditioning → DSP graph → output       │
│ sem alocação, locks, I/O, logs ou trabalho não determinístico │
├──────────────────────────────────────────────────────────────┤
│ Input → Pedals → Preamp → Tone Stack → FX Loop                │
│       → Phase Inverter / Power Amp → Reactive Load (opcional) │
│       → Cabinet/Mic IR → Post FX → Output                     │
└──────────────────────────────────────────────────────────────┘
```

## 4. Organização sugerida do workspace

```text
ampforge/
├── Cargo.toml                  # workspace
├── Cargo.lock
├── crates/
│   ├── ampforge-dsp/           # traits, buffers, parâmetros, utilidades DSP
│   ├── ampforge-circuits/       # modelos WDF/MNA e componentes
│   ├── ampforge-models/         # amps, pedais, power amp, speaker/load
│   ├── ampforge-ir/             # parser, metadados, resampling e convolução
│   ├── ampforge-engine/        # grafo, processamento, troca de configuração
│   ├── ampforge-audio/         # CPAL, dispositivos, callback e adaptadores
│   ├── ampforge-presets/       # schema, migrações, presets e rigs
│   └── ampforge-testkit/       # sinais, fixtures, medição e comparação
├── app/
│   └── ampforge-desktop/       # entry point, GUI, configuração e assets
├── assets/
│   ├── ir/                     # IRs próprias/licenciadas
│   └── presets/
└── docs/
```

Começar menor é válido: `dsp`, `engine`, `audio` e `desktop` podem ser os primeiros crates. Separar crates quando os limites de responsabilidade estiverem claros, não por cerimônia.

## 5. Contratos centrais do motor

Defina contratos pequenos e explícitos, por exemplo:

```rust
pub struct AudioSpec {
    pub sample_rate_hz: f64,
    pub max_block_size: usize,
    pub input_channels: usize,
    pub output_channels: usize,
    // Em código real, use um tipo que descreva também o mapeamento/ordem dos canais.
}

pub trait AudioProcessor: Send {
    fn prepare(&mut self, spec: AudioSpec);
    fn process(&mut self, input: &[f32], output: &mut [f32]);
    fn reset(&mut self);
    fn latency_samples(&self) -> usize;
}
```

O exemplo é ilustrativo: um projeto real pode preferir buffers por canal, frames intercalados ou tipos de bloco próprios. Não use `&[f32]` plano sem documentar o layout dos canais. `prepare` roda fora da callback. `process` não aloca nem bloqueia. A latência de cada nó deve ser conhecida para compensação e alinhamento.

Parâmetros devem ter identificadores estáveis, unidade, faixa, valor padrão, taper/escala e política de suavização. Separe parâmetros contínuos de eventos discretos. Um parâmetro `GainDb` não deve ser tratado como amplitude linear sem conversão explícita.

## 6. Threads, comunicação e troca de grafo

- **UI thread:** desenha a interface e envia comandos.
- **Control/asset thread:** carrega arquivos, valida preset/IR, cria instâncias e compila o grafo.
- **Audio callback:** processa somente o grafo ativo e publica métricas simples.

Use canais bounded ou estruturas lock-free adequadas para comandos e eventos. Para troca de grafo, não destrua o grafo antigo na callback: a desalocação pode ser imprevisível. Use estratégia de ownership/retirement para que a thread de controle recupere e destrua o grafo antigo depois que a callback não o estiver usando. Escolha uma implementação comprovadamente correta (por exemplo, mecanismo de troca atômica com reclamation apropriada); não improvise `Arc<Mutex<_>>` no caminho de áudio.

Se a fila de comandos encher, defina política explícita: coalescer parâmetros contínuos, descartar mensagens obsoletas ou rejeitar operações estruturais com feedback. Nunca deixar a callback esperar a UI.

## 7. Pipeline de áudio e modelos híbridos

Uma cadeia de referência configurável:

```text
Input trim / calibration
  → input impedance & pickup interaction (quando modelados)
  → stompbox chain
  → preamp stages (circuit-level ou behavioral)
  → tone stack específico do circuito
  → FX loop (serial/paralelo, quando presente)
  → phase inverter
  → power stage + supply sag
  → output transformer / reactive load (nível de fidelidade configurável)
  → cabinet + microphone IR
  → post effects / room / output trim
```

Não obrigar todo preset a usar todos os estágios. O grafo deve representar variantes reais: alguns amps têm reverb/tremolo integrado; outros não têm loop; o tone stack pode estar em posições diferentes; alguns modelos requerem EF86 ou canais com circuitos distintos.

### 7.1 Circuit-level

- Começar com componentes e subcircuitos bem delimitados: resistor/capacitor, redes RC, estágios de triodo, cathode follower, tone stack FMV, clipping e bias simplificados.
- Avaliar WDF para circuitos adequados a essa formulação e MNA/Newton para topologias que exijam solução de rede não linear. Não assumir que uma técnica resolve todos os circuitos com igual custo/robustez.
- Usar modelos de válvula com curvas e parâmetros identificados por documentação/medição; validar ponto de operação, ganho, clipping, impedância e recuperação.
- Pré-computar coeficientes lineares quando possível. Resolver elementos não lineares com limites de iteração, fallback e testes de estabilidade.
- Documentar simplificações (por exemplo, grid current omitida, fonte idealizada, transformador simplificado).

### 7.2 Modelagem comportamental

Use blocos com semântica física aproximada: filtros de acoplamento, curva de transferência, compressão dependente do envelope, memória/recuperação, sag dinâmico e interação com carga. Uma curva `tanh` pode ser útil como protótipo, mas não é modelo universal de válvula. Prefira curvas parametrizadas/calibradas e estados dinâmicos quando o comportamento exigir memória.

### 7.3 Oversampling e aliasing

Estágios não lineares geram harmônicos acima de Nyquist. Avalie oversampling por estágio ou por grupos de estágios, com filtros de interpolação/decimação adequados. Comece comparando 2x/4x/8x em testes controlados; a escolha final deve equilibrar aliasing residual, CPU e latência. Não declarar um fator como suficiente sem medir. O caminho linear não precisa necessariamente operar sempre no mesmo fator se a arquitetura permitir conversão segura.

### 7.4 IR e convolução

- IR representa a resposta linear medida do caminho capturado (gabinete, falante, microfone, posição, pré de microfone e sala conforme a captura); não representa por si só clipping, sag, interação dinâmica do power amp ou resposta não linear do falante.
- Importar WAV com validação de sample rate, canais, bit depth, duração, valores não finitos e silêncio/offset DC. Preservar o original; preparar uma cópia de trabalho.
- Resample, trim, remoção DC, normalização e ajuste de comprimento devem ser opções explícitas e não destrutivas. Normalização muda o nível e pode invalidar comparações A/B se aplicada silenciosamente.
- Manter metadados: fabricante/autoria, falante, gabinete, microfone, posição, distância, sample rate, tamanho, polaridade, licença e cadeia de captura, quando conhecidos.
- Para IR curta, FIR direta pode ser simples e previsível. Para IR longa, FFT partitioned convolution pode reduzir CPU, mas introduz escolhas de partição e latência. Medir ambas.
- Tratar IR estéreo, dual-mic e blends como formatos de roteamento explícitos; não reduzir silenciosamente estéreo para mono.
- Garantir alinhamento de latência e polaridade entre caminhos paralelos.

## 8. Áudio no Windows: WASAPI, ASIO, formatos e estabilidade

### 8.1 Backends obrigatórios no primeiro release

- Implementar WASAPI e ASIO atrás de uma abstração comum (`AudioBackend`), sem permitir que tipos específicos dos backends vazem para `ampforge-dsp` ou `ampforge-engine`.
- Em CPAL, WASAPI é disponibilizado no Windows e ASIO é habilitado pela feature opcional `asio`. No produto, ASIO deve estar compilado, selecionável e coberto por testes no primeiro release; não tratar sua natureza opcional na crate como uma decisão de produto.
- A documentação atual de CPAL lista ASIO como backend Windows opcional e informa requisitos de driver ASIO e LLVM/Clang. Validar a versão do SDK, variáveis de ambiente e toolchain em CI e em uma máquina Windows limpa.
- Verificar os termos/licenças aplicáveis do SDK ASIO e das dependências antes de distribuir builds. Desenvolvimento interno e distribuição pública podem ter obrigações diferentes; não presumir que a licença de uma crate resolva as obrigações do SDK subjacente.
- Se a integração ASIO do CPAL não cumprir requisitos de estabilidade, seleção de dispositivo, buffers ou roteamento, avaliar um adaptador ASIO específico, preservando o contrato interno `AudioBackend`.

### 8.2 Negociação de sample rate e canais

- Enumerar, por dispositivo e backend, as combinações realmente suportadas de sample rate, tamanho de buffer, formato e número de canais.
- Permitir selecionar múltiplas taxas ao longo do produto, mas abrir cada stream em uma taxa aceita pelo endpoint. `prepare()` do engine recebe a taxa efetivamente negociada.
- Uma mudança de sample rate deve preparar novamente filtros, oversampling, delays, convolução e demais estados fora da callback; não alterar coeficientes de forma improvisada no caminho real-time.
- Manter `input_channels` e `output_channels` independentes. Suportar layouts assimétricos (por exemplo, 1-in/2-out ou 2-in/4-out) quando o dispositivo os anunciar.
- Representar mapeamento de canais explicitamente; nunca assumir que o canal 1 físico corresponde sempre ao canal 1 lógico ou que todo dispositivo oferece estéreo.
- Resampling pode ser necessário para assets (IRs), reprodução de DI ou integração entre endpoints com taxas distintas. Faça-o fora da callback sempre que possível e documente a latência adicional.

### 8.3 Entrada de instrumento, linha e reamp

- Oferecer modos de fluxo Instrument/Hi-Z e Line, com calibração de nível, medidor e indicador de clipping. A impedância Hi-Z é uma propriedade elétrica da entrada/interface: o aplicativo não consegue transformar uma entrada fisicamente de linha em Hi-Z por software. Orientar o usuário a habilitar a entrada Hi-Z na interface ou usar DI apropriada.
- Capturar DI limpa e não destrutiva, preservando sample rate, canais, nível e metadados relevantes.
- Para reamp, permitir encaminhar uma DI para uma saída física escolhida, controlar nível e registrar o retorno em uma entrada selecionada. Explicar que normalmente é necessário um reamp box para adequar nível e impedância entre saída de linha e entrada do amplificador; não sugerir conexão direta sem verificar equipamento, nível e aterramento.
- Medir e compensar a latência de ida e volta do caminho de reamp. Registrar configuração de saída/entrada, backend, driver, sample rate, buffer e offset medido para permitir repetição.

### 8.4 Operação e diagnóstico

- Oferecer seleção explícita e independente de dispositivo de entrada e saída, backend, sample rate, buffer quando suportado, canais e modo de monitoramento.
- Para WASAPI, avaliar e documentar modos shared/exclusive e suas implicações de formato/latência; exibir claramente conversões impostas pelo sistema.
- Monitoramento direto pode criar loop ou feedback acústico; tornar o caminho de monitoração explícito e permitir muting seguro.
- Medir latência de ida e volta com loopback físico, não estimar apenas pelo tamanho do buffer. Separar latência do dispositivo, buffers, processamento, resampling e convolução.
- Detectar stream errors e underruns; publicar contadores na UI sem logging dentro da callback.
- Não prometer latência “zero”. Publicar resultados medidos por dispositivo, driver, backend, sample rate e buffer.

## 9. Modelo de dados para rigs e presets

O preset deve descrever uma cadeia, parâmetros e referências de assets, não serializar o estado interno bruto dos objetos DSP.

Campos recomendados:
- `schema_version`;
- `preset_id`, `name`, `author`, `description`, `tags`;
- `audio_assumptions` (nível de entrada de referência, mono/estéreo, etc.);
- `nodes`: `node_id`, `model_id`, `model_revision`, `enabled`, `parameters`;
- `connections`: origem, destino, canal e ordem;
- `assets`: caminho relativo/identificador e hash de IR;
- `latency_policy` e metadados opcionais.

Use IDs estáveis para modelos e parâmetros. Faça migrações explícitas de schema. Se um modelo ou IR estiver ausente, mostrar erro recuperável e não substituir por outro silenciosamente.

## 10. Plano de implementação por marcos

### Marco 0 — Especificação e medição
- Definir interface de referência, nível nominal, matriz inicial de sample rates/canais e hardware mínimo.
- Criar teste de loopback para medir latência e ruído da cadeia real.
- Validar WASAPI e ASIO como requisitos do primeiro release; resolver toolchain, SDK, driver e licenças de distribuição.

### Marco 1 — Aplicação mínima de áudio
- Enumerar e selecionar dispositivos independentemente para entrada e saída.
- Implementar WASAPI e ASIO desde esta etapa, com seleção explícita de backend.
- Negociar sample rate, buffer, formato e canais conforme capacidades reais de cada dispositivo.
- Capturar entrada e reproduzir saída com monitoramento controlado.
- Callback sem alocação/locks; medidor de pico/RMS fora do caminho crítico.
- Testes de reconexão, troca de dispositivo, canais assimétricos e formatos incompatíveis.

### Marco 2 — Motor DSP e grafo
- Implementar buffers, bypass, gain, filtros básicos e roteamento mono/estéreo.
- Compilar grafo fora da callback e trocar configuração com segurança.
- Medir CPU e latência de cada nó.

### Marco 3 — Primeiro rig tocável
- Input stage, um overdrive comportamental, um pré-amplificador de referência e tone stack.
- IR loader e convolução mono.
- A/B com cadeia de referência e teste de resposta em frequência/transientes.

### Marco 4 — Fidelidade dinâmica
- Oversampling seletivo.
- Sag, clipping por estágio, interação de impedância e power amp.
- Circuit-level para um circuito clássico de referência com esquema identificado.
- Testes de aliasing, estabilidade, recuperação e dinâmica de palhetada.

### Marco 5 — Produto
- Cadeias editáveis, presets versionados, IR estéreo/dual-mic, efeitos de modulação/tempo.
- UX de dispositivos, calibração de entrada, medição de latência e telemetria.
- Empacotamento Windows, instalação, logs fora da callback e matriz de compatibilidade.

## 11. Plano de testes e critérios de aceitação

**Testes determinísticos:** impulso, senoides, sweep logarítmico, multitone, silêncio, DC, ruído, degrau e bursts. Fixar seeds e fixtures para reprodutibilidade.

**Medições:** resposta em frequência, fase, ganho, THD/IMD, espectro de aliasing, envelope de ataque/release, ruído, latência, CPU média/pico e ocorrências de underrun.

**Testes de guitarra:** palhetada fraca/forte, acordes, palm mute, notas sustentadas, volume da guitarra em diferentes posições, troca de captador e interação com pedais antes/depois do amp.

**Comparação:** nível-match A/B, mesma guitarra/captador, mesma execução reampada, mesmo sample rate, mesmo gabinete/microfone/IR e cadeia de captura documentada. Evitar comparar sinais com níveis diferentes e concluir que o mais alto tem mais qualidade.

**Critérios de release:** nenhum panic na callback; nenhuma alocação ou lock no caminho crítico; sem underruns no cenário de referência acordado; latência de ida e volta medida e divulgada; resultados DSP reproduzíveis; presets inválidos tratados com erro; troca de preset/IR sem clicks ou bloqueios perceptíveis.

## 12. Riscos técnicos prioritários

1. **Callback real-time contaminada por GUI/locks/alocação:** risco de clicks e dropout. Tratar como requisito arquitetural, não otimização tardia.
2. **Modelagem não linear sem oversampling:** aliasing audível, especialmente em fuzz/high gain.
3. **IR confundida com amp model:** timbre estático razoável, mas dinâmica e resposta à palhetada artificiais.
4. **Troca de grafo com destruição na callback:** picos de CPU e pausas imprevisíveis.
5. **ASIO tratado como garantido:** problemas de build, SDK, licença/distribuição ou driver.
6. **Modelos de circuito sem dados de referência:** aparência de precisão sem validação; exigir schematic/revision e fixtures.
7. **Parâmetros sem unidades/taper:** controles que não se comportam como os equipamentos reais.
8. **Preset sem versão/hash de asset:** projetos quebrados após atualização ou mudança de IR.

## 13. Decisões confirmadas e pendências reais

As decisões de plataforma e escopo já confirmadas não devem ser reabertas pelo agente:

- Windows 10 e 11.
- WASAPI e ASIO no primeiro release.
- Suporte a múltiplas taxas de amostragem, conforme negociação com os dispositivos.
- Canais de entrada e saída configuráveis dinamicamente e de forma independente.
- Entrada Hi-Z e de linha; captura DI e fluxo de reamp.
- Rust, aplicação standalone e modelagem híbrida.
- VST3 considerado para o futuro; núcleo DSP independente de GUI e host.

Pendências que ainda exigem decisão, hardware ou medição:

- CPU/RAM mínimos e matriz de interfaces/drivers de referência.
- Taxas de amostragem e tamanhos de buffer que serão oficialmente certificados/testados no release.
- Limite de canais simultâneos e formatos PCM garantidos pela aplicação.
- Política inicial de WASAPI shared/exclusive e comportamento ao trocar de dispositivo.
- Estratégia de licenciamento/distribuição ASIO após revisão dos termos vigentes.
- Se a biblioteca inicial de IR será somente importada pelo usuário ou também incluirá conteúdo fornecido/licenciado pelo produto.
- Valores padrão de monitoramento, calibração de entrada e procedimento guiado de reamp.

## 14. Fontes primárias para validar a stack

- CPAL (áudio de baixo nível; backends, callback e dispositivos): https://github.com/RustAudio/cpal e https://docs.rs/cpal/latest/cpal/
- Guia ASIO de CPAL e requisitos de compilação: https://docs.rs/crate/cpal/latest
- Portal de desenvolvedores Steinberg (SDKs e licenciamento VST/ASIO): https://www.steinberg.net/developers/
- eframe/egui (GUI nativa e multiplataforma): https://docs.rs/eframe/latest/eframe/
- dasp (fundamentos e tipos PCM): https://docs.rs/dasp/latest/dasp/
- rubato (resampling): https://docs.rs/rubato/latest/rubato/

Consultar a documentação atual e os requisitos de cada versão antes de fixar dependências. O suporte a ASIO no CPAL é habilitado por feature e tem pré-requisitos próprios; isso não altera a decisão de produto de oferecer WASAPI e ASIO no primeiro release.
