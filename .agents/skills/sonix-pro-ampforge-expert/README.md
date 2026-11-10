# SONIX PRO – AMPFORGE Expert Skill

Skill de domínio para agentes de desenvolvimento que trabalham com simulação de amplificadores valvulados, pedais, efeitos, guitarras, captadores, gabinetes, microfonação, IR e DSP.

## Conteúdo

- `SKILL.md` — instruções principais para o agente.
- `references/architecture-rust-windows.md` — arquitetura Rust/Windows, WASAPI/ASIO, áudio em tempo real, canais dinâmicos, Hi-Z/linha/reamp e DSP híbrido.
- `references/engineering-backlog.md` — backlog técnico, entregáveis e critérios de aceite.

## Perfil tecnológico confirmado

- Rust.
- Aplicação standalone para Windows 10 e 11.
- WASAPI e ASIO desde o primeiro release.
- Múltiplas taxas de amostragem, negociadas conforme os dispositivos.
- Canais de entrada e saída configuráveis dinamicamente.
- Entrada Hi-Z e de linha; captura DI e fluxo de reamp.
- Modelagem híbrida: circuit-level + modelagem comportamental + IR.
- Núcleo DSP desacoplado da aplicação, considerando VST3 futuramente.

## Instalação

Adicione a pasta `sonix-pro-ampforge-skill` ao diretório de skills do seu agente e habilite a skill no projeto. Se a ferramenta exigir outro formato de manifesto, preserve o conteúdo do `SKILL.md` e adapte apenas o cabeçalho/metadados.
