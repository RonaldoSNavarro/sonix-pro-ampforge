# SONIX PRO – AMPFORGE | Backlog de engenharia

Este backlog traduz as decisões de produto em entregas verificáveis. A ordem prioriza o caminho de áudio confiável antes de expandir a biblioteca de modelos.

## Decisões de produto já confirmadas

- Rust; aplicação standalone para Windows 10 e 11.
- WASAPI e ASIO disponíveis no primeiro release.
- Múltiplas taxas de amostragem, conforme capacidades do dispositivo.
- Entrada e saída com número de canais configurável dinamicamente.
- Entradas Hi-Z e de linha; suporte a fluxo de reamp.
- Fidelidade híbrida: circuit-level, modelagem comportamental e IR.
- Considerar VST3 futuramente; manter DSP independente de host desde o início.

## Épico A — Fundação e contratos

- [ ] Criar workspace Cargo e crates iniciais `ampforge-dsp`, `ampforge-engine`, `ampforge-audio`, `ampforge-presets` e `ampforge-desktop`.
- [ ] Definir `AudioSpec` com sample rate, block size, formato, input channels, output channels e layout/mapeamento.
- [ ] Definir contrato de processador sem dependência de GUI, CPAL, WASAPI, ASIO ou VST.
- [ ] Definir unidades, ranges, smoothing, IDs estáveis e schema versionado dos parâmetros.
- [ ] Criar harness de testes offline para processar WAV/fixtures sem abrir dispositivo de áudio.

**Aceite:** `cargo test` executa sem hardware; DSP pode ser instanciado e testado em qualquer host; layouts de canal inválidos geram erro explícito.

## Épico B — Host de áudio Windows

- [ ] Implementar enumeração e seleção de dispositivos WASAPI.
- [ ] Implementar enumeração e seleção de dispositivos ASIO.
- [ ] Encapsular ambos atrás de uma interface interna comum `AudioBackend`.
- [ ] Detectar backend/driver ausente e apresentar fallback/erro acionável.
- [ ] Negociar sample rate, buffer, formato e canais com o dispositivo.
- [ ] Implementar reconexão e recuperação de falhas sem travar a GUI.
- [ ] Testar instalação limpa em Windows 10 e 11 x64.
- [ ] Revisar dependências, SDKs, licenças e redistribuição de componentes ASIO.

**Aceite:** WASAPI e ASIO abrem streams em hardware de referência; formatos não suportados são reportados; falha de ASIO não impede uso de WASAPI; nenhuma promessa de suporte é feita para combinação de driver não testada.

## Épico C — Formatos, canais e roteamento

- [ ] Suportar taxas múltiplas anunciadas pelos dispositivos e preparar o engine com a taxa efetiva.
- [ ] Suportar mono, estéreo e N canais de entrada/saída por configuração.
- [ ] Criar matriz de roteamento físico ↔ buses internos.
- [ ] Tratar entradas/saídas assimétricas (por exemplo, 1-in/2-out ou 2-in/4-out).
- [ ] Implementar conversão de formato e resampling somente onde necessário.
- [ ] Exibir claramente taxa/canais solicitados versus efetivamente negociados.

**Aceite:** testes cobrem diferentes layouts; nenhum canal é duplicado, descartado ou somado sem regra explícita; troca de formato recria/prepara o pipeline fora da callback.

## Épico D — Instrumento, linha e reamp

- [ ] Criar modo de entrada Instrument/Hi-Z com orientação para conexão correta.
- [ ] Criar modo Line com calibração de nível e indicador de clipping.
- [ ] Implementar captura DI limpa sem processamento destrutivo.
- [ ] Implementar reprodução de DI para saída física selecionada.
- [ ] Implementar captura do retorno reampado e alinhamento/registro de latência.
- [ ] Incluir ajuste de nível/atenuação e avisos de impedância/nível.

**Aceite:** uma DI gravada pode ser reproduzida de forma repetível pela saída selecionada; o retorno é gravado e alinhado; clipping ADC/DAC é detectável; usuário não é induzido a conectar nível de linha diretamente a entrada de amp sem verificar nível e impedância.

## Épico E — Motor real-time

- [ ] Implementar callback sem alocação, locks bloqueantes, I/O, logging ou operações imprevisíveis.
- [ ] Preparar grafos, IRs e coeficientes em thread de controle.
- [ ] Trocar grafos sem destruir objetos na callback.
- [ ] Implementar smoothing de parâmetros e política para filas cheias.
- [ ] Expor métricas atômicas de underrun, peak/RMS e tempo de processamento.

**Aceite:** testes de estresse não identificam alocações/locks na callback; troca de preset/IR não causa clique ou bloqueio perceptível; nenhuma panic atravessa a fronteira do callback.

## Épico F — DSP e modelos

- [ ] Implementar gain, filtros, bypass, mixer e roteamento.
- [ ] Implementar primeiro pedal comportamental e pré-amplificador de referência.
- [ ] Implementar tone stack específico com dados/revisão documentados.
- [ ] Implementar convolução IR e validação de WAV/metadados.
- [ ] Adicionar oversampling seletivo e medir aliasing.
- [ ] Evoluir power amp, fonte/sag, transformador e carga reativa em etapas.
- [ ] Adicionar circuit-level (WDF/MNA ou método justificado) para estágios selecionados.

**Aceite:** cada modelo tem documentação, parâmetros, fixtures e testes; comparação A/B é level-matched; simplificações e limitações são explícitas.

## Épico G — Preparação para VST3 futuro

- [ ] Manter `ampforge-dsp` e `ampforge-engine` sem dependência da aplicação desktop.
- [ ] Definir API pública estável para preparar, processar, resetar e consultar latência.
- [ ] Separar estado serializável do estado interno DSP.
- [ ] Definir política de parâmetros, automação, canais e latência compatível com adaptação a host.
- [ ] Avaliar framework/SDK VST3, licenças e estratégia de distribuição quando o produto standalone estiver estável.

**Aceite:** um host futuro poderá adaptar o engine sem copiar algoritmos DSP nem importar código da GUI; não é necessário entregar plugin no primeiro release.

## Ordem recomendada de releases

1. **Prototype 0:** DSP offline, contrato de áudio e GUI mínima.
2. **Alpha 1:** WASAPI + ASIO, negociação de formatos e monitoramento.
3. **Alpha 2:** canais dinâmicos, entrada Hi-Z/linha, DI e reamp.
4. **Beta 1:** grafo DSP, primeiro amp, pedal e IR.
5. **Beta 2:** dinâmica avançada, power amp, sag, circuit-level e biblioteca de presets.
6. **Release 1.0:** matriz Windows 10/11 validada, instalação, documentação, telemetria e critérios de latência/estabilidade publicados.
7. **Futuro:** avaliação e implementação de host VST3 sobre o núcleo reutilizável.

## Pendências que exigem hardware/medição

- Interfaces e drivers ASIO de referência para certificação.
- Taxas e buffers mínimos/máximos a declarar como suportados.
- Limite de canais simultâneos e formatos PCM suportados.
- Hardware mínimo de CPU/RAM.
- Política de compensação de latência e calibração para reamp.
- Requisitos de WASAPI shared/exclusive e comportamento de troca de dispositivo.
