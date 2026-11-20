# Fontes das Curvas de Microfone e Tratamento de IR Estéreo (Spec 038 / Auditoria Spec 080)

## 1. Modelos de Microfone (`crates/ampforge-dsp/src/cab.rs:1082–1111`)

Cada modelo no modo Virtual Cab (Dual-Mic) aplica 2 filtros biquad em cascata aproximando a tendência tonal de eixo `0°` em campo próximo, baseados nas especificações e gráficos de resposta em frequência publicados pelos fabricantes (`PRIMARY_COMPONENT`), modelados por aproximação paramétrica (`INFERENCE`).

> **Nota de auditoria (2026-11-20 — `Spec 080`):** Um par de biquads de 2ª ordem é uma **aproximação comportamental de contorno tonal (`INFERENCE`)** e não substitui a resposta impulsiva completa de fase/magnitude da cápsula física. Abaixo estão registrados tanto os documentos primários de cada fabricante quanto os valores exatos implementados em `crates/ampforge-dsp/src/cab.rs:1082–1111` (corrigindo a divergência documental anterior no modelo `sm57`).

| Modelo (`MicModel`) | Fabricante / Documento Primário (`PRIMARY_COMPONENT`) | URL / Referência do Datasheet | Curva Implementada em `cab.rs:1082–1111` (`INFERENCE`) | Característica Física Representada |
|---|---|---|---|---|
| `sm57` (`Sm57`) | Shure Incorporated — *SM57 Cardioid Dynamic Instrument Microphone User Guide / Specification Sheet* | `https://pubs.shure.com/guide/SM57/en-US` (Gráfico Frequency Response, p. 2: roll-off < 200 Hz, pico de presença 4–6 kHz) | `peaking(5600.0 Hz, +4.0 dB, Q=2.5)` + `low_shelf(160.0 Hz, -3.5 dB)` *(nota: versão anterior deste doc citava 4,8 kHz/+5,0 dB/Q=1,4)* | Roll-off de graves naturais da cápsula Unidyne III + pico de presença em 5–6 kHz |
| `md421` (`Md421`) | Sennheiser electronic — *MD 421-II Cardioid Dynamic Microphone Datasheet* | `https://www.sennheiser.com/` (Spec Sheet MD 421-II, curva Bass Switch M, 30 Hz–17 kHz) | `peaking(280.0 Hz, -2.5 dB, Q=1.2)` + `peaking(4500.0 Hz, +3.5 dB, Q=1.3)` | Controle de médio-grave em ~280 Hz + presença ampla em 4,5 kHz |
| `m88` (`M88`) | Beyerdynamic — *M 88 TG Hypercardioid Dynamic Microphone Datasheet* | `https://global.beyerdynamic.com/` (M 88 TG Technical Specifications, 30 Hz–20 kHz) | `low_shelf(110.0 Hz, +2.5 dB)` + `peaking(4000.0 Hz, +3.0 dB, Q=1.5)` | Extensão firme de graves da cápsula hipercardioide + definição em 4 kHz |
| `r121` (`R121`) | Royer Labs — *R-121 Ribbon Microphone Owner's Manual & Specifications* | `https://royerlabs.com/r-121/` (Frequency Response Curve 30 Hz–15 kHz, Figure-8) | `low_shelf(130.0 Hz, +2.0 dB)` + `high_shelf(3800.0 Hz, -7.5 dB)` | Corpo grave natural de fita Figure-8 + roll-off suave de fita de 2,5 µm acima de ~4 kHz |
| `m160` (`M160`) | Beyerdynamic — *M 160 Double-Ribbon Hypercardioid Microphone Datasheet* | `https://global.beyerdynamic.com/` (M 160 Datasheet, 40 Hz–18 kHz) | `peaking(1800.0 Hz, +1.5 dB, Q=1.0)` + `high_shelf(5500.0 Hz, -5.0 dB)` | Médios articulados de fita dupla hipercardioide + atenuação suave acima de 5,5 kHz |
| `u87` (`U87`) | Georg Neumann GmbH — *U 87 Ai Studio Microphone Operating Instructions* | `https://www.neumann.com/` (U 87 Ai Cardioid Frequency Response, 20 Hz–20 kHz) | `high_shelf(8000.0 Hz, +2.5 dB)` + `peaking(250.0 Hz, +0.5 dB, Q=0.9)` | Extensão linear e ar de condensador de diafragma largo em 8 kHz |
| `c414` (`C414`) | AKG Acoustics — *C414 XLS Multi-Pattern Condenser Microphone User Manual* | `https://www.akg.com/` (C414 XLS Cardioid Frequency Chart, 20 Hz–20 kHz) | `peaking(3500.0 Hz, +3.0 dB, Q=1.2)` + `high_shelf(9500.0 Hz, +4.0 dB)` | Presença aberta em 3,5 kHz e brilho estendido acima de 9,5 kHz |

## 2. IR Estéreo (`POST /api/cab/ir` / Comando `load_ir`)

- O motor processa a cadeia de gabinete atualmente em **mono** (`ChannelLayout::Mono`).
- Ao enviar um arquivo WAV estéreo (ou multicanal), o loader extrai o **canal 1 (Left)** (`channels_used: 1`) e informa `source_channels` na resposta JSON da API (`INTERNAL_REFERENCE`: Spec 038 CA-7).
