# Fontes de Circuitos e Voicing dos Amplificadores (Spec 039)

Este documento registra as referências de esquemas publicados para a modelagem de tone stack, estágios de ganho, realimentação negativa (NFB) e filtros de voicing dos 5 amplificadores do **SONIX PRO // AMPFORGE™**.

## 1. Fender Twin Reverb 1965 (`UsTwinBlackface65`)
- **Esquema de referência:** Fender AB763 (Twin Reverb / Showman, 1963–1967).
- **Tone Stack (`FenderTmbMidScoop`):**
  - `R_slope = 100 kΩ`
  - `R_treble = 250 kΩ` (audio taper)
  - `R_bass = 250 kΩ` (audio taper)
  - `R_mid = 10 kΩ` (linear)
  - `C1 (Treble) = 250 pF`
  - `C2 (Bass) = 0.1 µF` (`100 nF`)
  - `C3 (Mid) = 0.047 µF` (`47 nF`)
  - `R_load = 1 MΩ`
- **Voicing físico:**
  - O recorte natural de médios (~400–500 Hz) já é produzido pela rede passiva AB763 (`C2 = 100 nF`, `C3 = 47 nF`).
  - Filtro 1: `high_shelf(4500 Hz, +1.5 dB)` — brilho do cap de 120 pF / acoplamento de placa do canal Vibrato.
  - Filtro 2: `low_shelf(80 Hz, +0.5 dB)` — resposta de baixa frequência do transformador de saída Schumacher 022855 (85W).

## 2. Marshall JCM800 2203 (`BritJcm800_2203`)
- **Esquema de referência:** Marshall Model 2203 (1975–1984 Master Volume Lead).
- **Tone Stack (`MarshallCathodeFollower`):**
  - `R_slope = 33 kΩ`, `R_treble = 220 kΩ`, `R_bass = 1 MΩ`, `R_mid = 22 kΩ`, `C1 = 470 pF`, `C2 = 22 nF`, `C3 = 22 nF`.
- **Voicing físico:**
  - Filtro 1: `peaking(1600 Hz, +2.0 dB, Q=1.1)` — ênfase de médios-agudos do divisor capacitivo/resistor de acoplamento V1b→V2a (470 kΩ || 470 pF peaking network no estágio de entrada do JCM800).
  - Filtro 2: `low_shelf(110 Hz, -1.5 dB)` — capacitor de cátodo V1b (`Rk = 10 kΩ` sem bypass) e acoplamento de 22 nF que enxuga subgraves antes do seguidor de cátodo.

## 3. Marshall 1959SLP Plexi (`BritPlexi1959`)
- **Esquema de referência:** Marshall Model 1959 Super Lead 100W (1967–1969, circuito JMP).
- **Tone Stack (`MarshallCathodeFollower`):** `33 kΩ` slope, `470 pF` / `22 nF` / `22 nF`.
- **Voicing físico:**
  - Filtro 1: `peaking(2400 Hz, +2.5 dB, Q=0.9)` — rede de mistura de 470 kΩ com capacitor de bypass de 470 pF no canal High Treble (Ch I) + presença natural do transformador Drake 1202-119.
  - Filtro 2: `low_shelf(120 Hz, +1.5 dB)` — capacitor de cátodo de `330 µF` em V2a / transformador de saída Drake.
- **Controles:** Amp não possui Master Volume no circuito original; `master_volume` não adiciona saturação de potência neste modelo. Entradas `Low1`/`Low2` aplicam divisor resistivo de entrada de `-6 dB` (uma única vez).

## 4. Mesa/Boogie Dual Rectifier Rev F (`CaliDualRectoRevF`)
- **Esquema de referência:** Mesa/Boogie Dual Rectifier Solo Head Revision F (1992).
- **Tone Stack (`MesaMarkPreGain`):** `39 kΩ` slope, `250 pF` / `22 nF` / `22 nF`, potenciômetros de 250 kΩ / 1 MΩ / 25 kΩ.
- **Voicing físico:**
  - Filtro 1: `peaking(85 Hz, +2.0 dB, Q=1.4)` — acoplamento de graves do transformador Schumacher de 100W em modo Bold/Spongy.
  - Filtro 2: `peaking(750 Hz, -2.0 dB, Q=1.1)` — contorno de médios dos capacitores de bypass de cátodo assimétricos em V2/V3 da Rev F.
- **NFB:** No modo `Modern`, o loop de NFB global é desconectado (`eff_nfb = 0`), anulando a ação de `Presence` e `Depth` no loop de realimentação.

## 5. Vox AC30/6 Top Boost (`UkAc30TopBoost`)
- **Esquema de referência:** JMI Vox AC30/6 com módulo Top Boost (1964, circuito ECC83/EL84 Classe A sem NFB).
- **Tone Stack (`VoxTopBoost`):**
  - `R_slope = 100 kΩ`, `C1 = 50 pF`, `C2 = 22 nF`, `C3 = 22 nF`, `R_bass = 1 MΩ`, `R_treble = 220 kΩ`, resistor fixo de cauda `10 kΩ`.
- **Voicing físico:**
  - Filtro 1: `peaking(3800 Hz, +2.5 dB, Q=1.1)` — brilho característico dos agudos sem NFB e transformador Albion/Woden.
  - Filtro 2: `low_shelf(140 Hz, -2.5 dB)` — capacitores de acoplamento de 22 nF antes do inversor de fase LTP.
