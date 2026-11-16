# Guia de Contribuição — SONIX PRO // AMPFORGE™

Seja muito bem-vindo! O **SONIX PRO // AMPFORGE™** é um projeto de código aberto em **desenvolvimento ativo**, criado com paixão pela engenharia de áudio, simulação analógica e tecnologia de ponta em Rust e Tauri v2.

Estamos abertos e entusiasmados para receber **colaboradores e parceiros** de todas as áreas:
- 🎸 **Guitarristas e Músicos:** Testes de resposta dinâmica, *feel*, sugestões de amplificadores e pedais clássicos, *presets* e relatos de usabilidade.
- 🔊 **Engenheiros de Áudio e Especialistas em DSP:** Modelagem analógica de circuitos, filtros digitais, *oversampling*, solvers de equações diferenciais não lineares e técnicas de baixa latência.
- 🦀 **Desenvolvedores Rust:** Otimização de concorrência sem travas (*lock-free*), arquitetura de plugins VST3/CLAP via `nih-plug`, drivers WASAPI/ASIO e resiliência de tempo real.
- 🎨 **Desenvolvedores Frontend e Designers:** Aprimoramento do Design System skeuomórfico, renderização acelerada por GPU (Canvas/WebGL), shaders de CRT e ergonomia de palco.

---

## 🛠️ Como Contribuir

### 1. Reportar Problemas ou Sugerir Ideias
Encontrou um bug, chiado atípico ou tem uma ideia para um novo pedal ou amplificador?
- Abra uma [Issue no GitHub](https://github.com/RonaldoSNavarro/sonix-pro-ampforge/issues).
- Descreva detalhadamente o comportamento observado, o hardware utilizado (interface de áudio, taxa de amostragem, SO) e os passos para reproduzir.

### 2. Metodologia: Spec Driven Development (SDD) & TDD Estrito
Neste projeto, **nenhuma mudança de comportamento entra sem especificação prévia**:
1. Toda nova funcionalidade, alteração de DSP ou novo controle de UI nasce de uma **spec numerada** em `specs/NNN-nome.md` (veja [specs/README.md](specs/README.md)).
2. A spec define critérios de aceite atômicos e auditáveis (`CA-1`, `CA-2`, etc.).
3. Seguimos o regime **TDD Estrito**: cada critério de aceite vira um teste automatizado que **falha antes** da implementação (*red → green*).
4. A spec, os testes de contrato e o código de produção são revisados e submetidos no **mesmo Pull Request**.

### 3. Regra de Ouro da Engenharia de Áudio: *Zero-Alloc Realtime Safety*
O callback de áudio em tempo real roda com prioridade crítica e restrições severas:
- **Zero alocações na heap:** Nunca use `Vec::new()`, `Box::new()`, `String`, `format!()` ou qualquer estrutura dinâmica dentro de `RealtimeDspChain::process` ou nos blocos de efeitos em tempo de execução.
- **Zero bloqueios (locks/mutexes):** A comunicação entre a thread de UI/controle e a thread de áudio é feita exclusivamente por filas circulares lock-free SPSC (`SpscQueue`).
- **Buffers pré-dimensionados:** Todo buffer de atraso, convolução ou scratchpad deve ser alocado na inicialização (`new` ou `prepare`).

---

## 💻 Ambiente de Desenvolvimento

### Pré-requisitos
- **Rust:** Versão estável recente (1.80+ recomendado) com alvo x86_64 (`rustup default stable`).
- **Node.js:** Versão 20+ (ou Node 18+) e `npm`.
- **Windows (recomendado para standalone):** C++ Build Tools (MSVC ou GNU toolchain).

### Configuração Inicial
Clone o repositório e instale as dependências da interface:
```bash
git clone https://github.com/RonaldoSNavarro/sonix-pro-ampforge.git
cd sonix-pro-ampforge
npm install
npm --prefix apps/sonix-app/ui install
```

### Executando em Modo Desenvolvimento
Inicie a interface Vite com recarregamento a quente:
```bash
npm --prefix apps/sonix-app/ui run dev
```

E para iniciar o shell desktop Tauri v2:
```bash
npm --prefix apps/sonix-app/ui run tauri dev
```

### Rodando a Suíte de Testes
Antes de abrir um Pull Request, certifique-se de que todos os testes passem com 100% de sucesso:
```bash
# Testes do workspace Rust (DSP, Core, Desktop, Plugin, App)
cargo test --workspace

# Testes de contrato automatizados da UI e integração (Node.js)
node --test tests/contract/
```

### Build de Produção
Para compilar o binário otimizado com a UI embutida:
```bash
# 1. Compila o frontend de produção
npm --prefix apps/sonix-app/ui run build

# 2. Compila o executável release com custom-protocol
cargo build --release -p sonix-app --features custom-protocol
```
O executável release será gerado em `target/release/sonix-app.exe` (ou no `target-dir` configurado em `.cargo/config.toml`).

---

## 📜 Licença e Propriedade Intelectual

Ao contribuir com código para este repositório, você concorda que suas contribuições serão disponibilizadas sob os mesmos termos da licença do projeto: **Creative Commons Atribuição-NãoComercial 4.0 Internacional (CC BY-NC 4.0)**.

- O sistema é livre para uso pessoal, pesquisa e aprendizado, desde que o autor original (**Ronaldo S. Navarro** / `@ronaldosnavarro`) e os colaboradores sejam devidamente mencionados.
- O software e suas derivações **não podem ser comercializados ou monetizados**.

Obrigado por ajudar a construir o simulador de guitarra definitivo! 🎸🔥
