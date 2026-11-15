# Fontes das Curvas de Microfone e Tratamento de IR Estéreo (Spec 038)

## Modelos de Microfone (`crates/ampforge-dsp/src/cab.rs`)

Cada modelo aplica 2 filtros biquad em cascata baseados nas curvas de resposta em frequência publicadas pelos fabricantes (eixo 0°, campo próximo típico de gabinete):

| Modelo | Fabricante / Datasheet | Característica Modelada |
|---|---|---|
| `sm57` | Shure SM57 User Guide (Instrument Microphone) | Atenuação abaixo de 160 Hz (−3,5 dB low-shelf) + pico de presença em 4,8 kHz (+5,0 dB, Q=1,4) |
| `md421` | Sennheiser MD 421-II Datasheet | Recorte de médios-graves em 280 Hz (−2,5 dB, Q=1,2) + presença ampla em 4,5 kHz (+3,5 dB, Q=1,3) |
| `m88` | Beyerdynamic M 88 TG Datasheet | Extensão de graves em 110 Hz (+2,5 dB low-shelf) + brilho em 4,0 kHz (+3,0 dB, Q=1,5) |
| `r121` | Royer Labs R-121 Ribbon Microphone Manual | Realce quente em 130 Hz (+2,0 dB low-shelf) + roll-off suave de fita acima de 3,8 kHz (−7,5 dB high-shelf) |
| `m160` | Beyerdynamic M 160 Datasheet | Médios focados em 1,8 kHz (+1,5 dB, Q=1,0) + roll-off suave de fita dupla acima de 5,5 kHz (−5,0 dB high-shelf) |
| `u87` | Neumann U 87 Ai Operating Instructions (Cardioid) | Clareza de condensador em 8,0 kHz (+2,5 dB high-shelf) + leve corpo em 250 Hz (+0,5 dB) |
| `c414` | AKG C414 XLS Datasheet (Cardioid) | Presença em 3,5 kHz (+3,0 dB, Q=1,2) + ar em 9,5 kHz (+4,0 dB high-shelf) |

## IR Estéreo (`POST /api/cab/ir`)

- O motor processa a cadeia de gabinete em **mono**.
- Ao enviar um arquivo WAV estéreo (ou multicanal), o loader extrai o **canal 1 (Left)** (`channels_used: 1`) e informa `source_channels` na resposta JSON da API.
