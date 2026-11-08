---
spec: 003
titulo: Modelagem White-Box de amplificadores valvulados e painel frontal autêntico
status: aprovada
atualizado: 2026-11-08
---
# 003 — Modelagem White-Box de amplificadores valvulados e painel frontal autêntico

## Objetivo
Entregar ao guitarrista e baixista a resposta dinâmica, harmônica e tátil de cabeçotes valvulados clássicos e modernos modelados em nível de circuito (100% White-Box) através de um painel frontal fotorrealista com todos os controles originais de fábrica.

## Contexto
Capturas estáticas (*black-box snapshots* ou redes neurais treinadas em um único ajuste de knobs) não reproduzem a interação não-linear real entre `Input Gain`, `Bright Cap`, `Tone Stack`, `Presence`/`Depth` (inseridos na malha de realimentação negativa do Power Amp) e `Master Volume`. No **SONIX PRO // AMPFORGE™**, o amplificador é simulado em nível de componentes discretos.

Fontes da verdade e evidências no repositório:
- `docs/01-SRS-Levantamento-de-Requisitos.md:16-17` — Define o objetivo estratégico de fidelidade White-Box (triodos 12AX7, pentodos EL34/6L6, transformadores, sag, tone stacks e impedância reativa).
- `docs/01-SRS-Levantamento-de-Requisitos.md:121` — Define **RF-006** (Painel Frontal Autêntico com `Input Gain`, `Overdrive/Lead Gain`, `Bass`, `Middle`, `Treble`, `Presence`, `Depth/Resonance`, `Master Volume`, `Output Level`, `Bright Switch` e `Channel Toggle`).
- `docs/01-SRS-Levantamento-de-Requisitos.md:237-248` — Define a estrutura `frontPanel` no schema `.sonixpreset`.
- `design-system/index.html:145-195` e `design-system/app.js:18-28,73-82,174-260` — O protótipo implementa 8 knobs (`inputGain`, `overdrive`, `bass`, `middle`, `treble`, `presence`, `depth`, `masterVol`) e o `brightSwitchBtn`, mas falta o controle `outputLevel` (distinto do `masterVolume` que satura o Power Amp), o `Channel Toggle` no painel frontal e a troca efetiva de parâmetros de circuito ao mudar o seletor `#ampModelSelect` (`design-system/index.html:149-154`).

| Capacidade | Estado atual (`design-system/`) | O que esta spec estabelece para produção |
|---|---|---|
| Controles de Painel Frontal (**RF-006**) | 8 knobs + `Bright Switch` (`design-system/app.js:73-82`) | Todos os 11 controles de **RF-006**, incluindo `Output Level` (pós-Power Amp limpo) e `Channel Toggle` |
| Seletor de Modelos (`#ampModelSelect`) | 4 opções no HTML sem handler de mudança (`design-system/index.html:149-154`) | Catálogo ativo (`brit800`, `plexisuper`, `usdeluxe`, `calirecto`, `ac30`) que carrega topologia White-Box, tema visual do faceplate e valores padrão |
| Efeito do `Bright Switch` | Apenas muda o texto `BRIGHT: ON/OFF` (`design-system/app.js:724-727`) | Insere/remove o capacitor de bypass de agudos (`120 pF` / `470 pF`) sobre o potenciômetro de `Input Gain` (maior efeito em ganhos baixos, desaparecendo em `Input Gain = 10`) |
| Distinção `Master Volume` vs `Output Level` | Apenas `masterVol` controlando largura da barra VU (`design-system/app.js:429`) | `Master Volume` governa o drive do Phase Inverter / Power Amp (saturação de potência), enquanto `Output Level` ajusta o volume final em dB sem alterar o timbre |

**Alternativas descartadas:**
1. *Unificar `Master Volume` e `Output Level` em um único controle:* descartada porque em amplificadores valvulados reais (ex.: JCM 800 ou Plexi) abrir o `Master Volume` satura as válvulas EL34 e o transformador; o músico precisa de um `Output Level` pós-circuito para igualar o volume entre cenas sem perder a saturação de power amp.
2. *Aplicar um filtro shelving estático para o `Bright Switch`:* descartada porque em circuitos reais o capacitor Bright fica em paralelo com o potenciômetro de ganho; sua ênfase em agudos depende inversamente da posição do `Input Gain`.

## Comportamento esperado
1. **Catálogo de Modelos White-Box:** O bloco `AMP` disponibiliza os modelos base de circuito:
   - `brit_jcm800_2203` (*Brit JCM 800 2203 — 100W EL34 Push-Pull*, faceplate Latão Escovado);
   - `brit_plexi_1959` (*1959 Super Lead Plexi — Jumped Channels*, faceplate Latão Escovado);
   - `us_twin_blackface_65` (*US Twin Blackface '65 — 2× 6L6GC Clean*, faceplate Cromo Espelhado);
   - `cali_dual_recto_revf` (*Cali Dual Rectifier Rev-F — High Gain 6L6/GZ34*, faceplate Blackout Metal);
   - `uk_ac30_topboost` (*UK Chime Top Boost — 4× EL84 Class-A*, faceplate Latão/Vinho).
2. **Controles do Painel Frontal (RF-006):**
   - `inputGain` (`0.0` a `10.0`): ganho do primeiro estágio de triodo V1 com interação dependente de posição do capacitor `brightSwitch`.
   - `overdrive` / `leadGain` (`0.0` a `10.0`): ganho dos estágios de saturação subsequentes em cascata.
   - `bass`, `middle`, `treble` (`0.0` a `10.0`): potenciômetros acoplados à rede RC do Tone Stack.
   - `presence` (`0.0` a `10.0`) e `depth` / `resonance` (`0.0` a `10.0`): filtros de alta e baixa frequência inseridos na malha de realimentação negativa (*Negative Feedback Loop*) do Power Amp.
   - `masterVolume` (`0.0` a `10.0`): nível enviado ao inversor de fase (V3) e às válvulas de potência, determinando o *power tube breakup* e o *sag*.
   - `outputLevel` (`-24.0 dB` a `+12.0 dB`): atenuação/ganho linear pós-transformador para calibração de volume entre presets/cenas.
   - `brightSwitch` (`true`/`false`): ativa/desativa o capacitor de bypass de alta frequência no potenciômetro de `inputGain`.
   - `channelToggle` (`NORM` / `HIGH-LEAD`): alterna o voicing de entrada do cabeçote.
3. **Solução Não-Linear em Tempo Real:** O estágio White-Box resolve a função de transferência assimétrica a cada amostra (em `float64`) refletindo imediatamente qualquer alteração dos controles do painel frontal na saída de áudio e nos visores do osciloscópio.

## Critérios de aceite
- **CA-1:** O painel frontal do bloco `AMP` expõe e persiste todos os 11 controles especificados no **RF-006**: `inputGain`, `overdrive`, `bass`, `middle`, `treble`, `presence`, `depth`, `masterVolume`, `outputLevel`, `brightSwitch` e `channelToggle`.
- **CA-2:** Selecionar qualquer um dos modelos no seletor `#ampModelSelect` atualiza a topologia White-Box ativa, aplica os valores padrão de circuito daquele modelo, atualiza o logotipo `#ampBrandLogoText` e troca o tema de material do painel (`--surface-brushed-brass`, `--surface-brushed-chrome` ou `--surface-blackout-metal`).
- **CA-3:** Com `brightSwitch = true` e `inputGain <= 3.0`, a resposta em frequência em `4 kHz` apresenta reforço mínimo de `+3,5 dB` em relação a `brightSwitch = false`; quando `inputGain = 10.0`, a diferença entre `brightSwitch = true` e `false` cai para menos de `0,2 dB` (comportamento real de capacitor em paralelo ao potenciômetro).
- **CA-4:** Aumentar `masterVolume` de `2.0` para `9.0` (mantendo `inputGain` fixo) eleva a distorção harmônica total (THD) e a compressão de *sag* do estágio de potência; em contraste, variar `outputLevel` de `-12.0 dB` para `+6.0 dB` escala apenas a amplitude linear de saída sem alterar a taxa de THD (variação `< 0,01%`).
- **CA-5:** Os controles `presence` e `depth` atuam na malha de realimentação negativa do estágio de potência, alterando o amortecimento de agudos (`> 2 kHz`) e da frequência de ressonância grave do falante acoplado (`65–130 Hz`).
- **CA-6:** Alternar `channelToggle` entre `NORM` e `HIGH-LEAD` reconfigura o ganho de polarização do estágio de entrada e atualiza o indicador visual no painel frontal.
- **CA-7:** Caso de erro — passar um `modelId` inexistente ou valor numérico fora dos limites do parâmetro (ex.: `inputGain = 15.0` ou `NaN`) faz o validador rejeitar ou limitar (*clamp*) o valor ao intervalo válido `[0.0, 10.0]` (ou `[-24.0, +12.0]` para `outputLevel`) e retornar erro para `modelId` desconhecido sem corromper o estado do áudio.
- **CA-8:** Não-regressão — alterar qualquer knob do Painel Frontal preserva intactos os ajustes customizados feitos pelo usuário na bancada *Chassi Aberto / Mod Shop* (**Spec 004**) daquele mesmo canal.

## Fora de escopo
- A edição direta de válvulas, tensão de placa, bias, sag, topologia de tone stack e osciloscópio CRT na gaveta interna (coberta pela **Spec 004**).
- A sala de microfonação e convolução de IR do gabinete (coberta pela **Spec 005**).

## Invariantes tocadas
- **RNF-003 (Precisão 64-bit nos Solvers):** O cálculo da função de transferência de triodos/pentodos é executado em `float64`.
- **RNF-004 (Oversampling Anti-Aliasing):** O estágio não-linear do amplificador respeita o fator de oversampling (`1x`, `2x`, `4x`, `8x`) configurado para o bloco.
- **RNF-006 (Zipper-Noise Free):** Todos os potenciômetros do painel frontal passam por suavização de 1 polo antes de alterar ganhos ou coeficientes de filtro.
- **Fisicalidade Autêntica (Design System Seção 1.1):** O painel frontal utiliza texturas de Tolex, grill cloth com brilho de filamento ao fundo (`#grilleGlow`), serigrafia `Oswald` e lâmpada piloto jewel (`#pilotLamp`).

## Testes que cobrem esta spec
- `tests/contract/003-modelagem-amplificadores-white-box-painel-frontal.test.js` → CA-1, CA-2, CA-3, CA-4, CA-5, CA-6, CA-7, CA-8 *(pendente — ciclo TDD red → green)*.

## Notas
- Rastreabilidade SRS: Seção 1.1, **RF-006**, Seção 5 (`frontPanel`).
