use std::path::Path;
#[cfg(feature = "dev-http")]
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use sonix_desktop::audio::{
    DesktopAudioHost, RealtimeDspChain, RealtimeDuplexEngine, TelemetryBuffer,
};
use sonix_desktop::midi::DesktopMidiHost;
#[cfg(feature = "dev-http")]
use sonix_desktop::server::LocalAppServer;

fn main() {
    println!("============================================================");
    println!("     🎛️  SONIX PRO // AMPFORGE™ Desktop Host Standalone    ");
    println!("============================================================");

    let mut audio_host = DesktopAudioHost::new();
    let physical_devices = audio_host.enumerate_physical_devices();
    println!("-> Dispositivos Físicos detectados: {}", physical_devices.len());
    for dev in &physical_devices {
        let role = if dev.is_input { "ENTRADA (P2/P3)" } else { "SAÍDA (Fones)" };
        println!(
            "   * [{:?}] {} - {} (Taxas: {:?}, Buffers: {:?})",
            dev.backend, dev.name, role, dev.supported_sample_rates, dev.supported_buffer_sizes
        );
    }

    let midi_host = DesktopMidiHost::new();
    let ports = midi_host.enumerate_input_ports();
    println!("-> Portas MIDI USB detectadas: {}", ports.len());
    for p in &ports {
        println!("   * {}", p);
    }

    // Inicializa motor duplex e cadeia DSP em tempo real
    let duplex = Arc::new(RealtimeDuplexEngine::new(65536));
    let telemetry = Arc::new(TelemetryBuffer::new(512));
    let mut dsp_chain_initial = RealtimeDspChain::new(48000.0);
    dsp_chain_initial.set_gate_threshold_db(-65.0); // limiar sensível (-65dB) para manter sustain de captadores passivos
    dsp_chain_initial.set_input_boost(2.5); // +8dB boost para microfone P2 passivo de guitarra
    dsp_chain_initial.set_amp_gain(8.0);
    dsp_chain_initial.set_amp_master(7.0);
    dsp_chain_initial.set_cab_enabled(true);
    let dsp_chain = Arc::new(Mutex::new(dsp_chain_initial));

    println!("-> Motor AmpForge™ DSP ativo: Filtro DC + Boost + Noise Gate (-65dB) -> Brit 800 White-Box -> Virtual Cab IR.");

    // Inicia streams WASAPI duplex reais com CPAL
    let active_streams = match DesktopAudioHost::start_realtime_wasapi_duplex(
        duplex.clone(),
        dsp_chain.clone(),
        telemetry.clone(),
    ) {
        Ok(streams) => {
            let _ = audio_host.start_stream(&physical_devices[0], 48000, 128);
            println!("-> [WASAPI REAL-TIME] Captura física da guitarra e saída estéreo ATIVAS!");
            Some(streams)
        }
        Err(err) => {
            println!("-> [WASAPI INFO] Usando modo emulado de áudio: {}", err);
            None
        }
    };

    // Inicia Servidor Local IPC/HTTP (apenas quando dev-http estiver habilitado)
    let is_headless = std::env::var("SONIX_HEADLESS").map(|v| v == "1").unwrap_or(false);
    #[cfg(feature = "dev-http")]
    let _server = {
        let no_browser = std::env::var("SONIX_NO_BROWSER").map(|v| v == "1").unwrap_or(false);

        let calib_store = sonix_desktop::calibration::CalibrationStore::new();
        let server = match LocalAppServer::bind_port_with_all(
            7890,
            telemetry.clone(),
            Some(dsp_chain.clone()),
            Some("default"),
            Some(calib_store),
        ) {
            Ok(s) => s,
            Err(_) => {
                if is_headless {
                    LocalAppServer::bind_ephemeral().expect("Falha ao ligar servidor em 127.0.0.1")
                } else {
                    println!("-> Uma instância do Sonix Pro já está em execução na porta 7890.");
                    #[cfg(target_os = "windows")]
                    if !no_browser {
                        let _ = Command::new("cmd")
                            .args(["/c", "start", "msedge", "--app=http://127.0.0.1:7890", "--window-size=1280,820"])
                            .spawn();
                    }
                    return;
                }
            }
        };

        if let Some(ref streams) = active_streams {
            server.set_audio_telemetry(streams.telemetry.clone());
        }

        let app_url = server.app_url();
        let app_cmd = server.build_desktop_app_command();
        println!("-> Servidor Desktop Local: {}", app_url);
        println!("-> Modo Janela Dedicada: {}", app_cmd);

        if !is_headless && !no_browser {
            let url = app_url.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(300));
                #[cfg(target_os = "windows")]
                {
                    let _ = Command::new("cmd")
                        .args(["/c", "start", "msedge", &format!("--app={}", url), "--window-size=1280,820"])
                        .spawn();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    if let Ok(cmd_path) = which_cmd() {
                        let _ = Command::new(cmd_path)
                            .args(["/c", "start", "msedge", &format!("--app={}", url), "--window-size=1280,820"])
                            .spawn();
                    }
                }
            });
        }

        server
    };

    println!("============================================================");
    println!("    🎸 SONIX PRO ONLINE! Conecte a guitarra na entrada P2   ");
    println!("    Pressione Ctrl+C para encerrar.                         ");
    println!("============================================================");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Se iniciado em modo teste com timeout
    if let Some(timeout_secs) = std::env::var("SONIX_RUN_TIMEOUT_SECS").ok().and_then(|v| v.parse::<u64>().ok()) {
        thread::sleep(Duration::from_secs(timeout_secs));
    } else {
        // Loop de processamento de áudio em background (apenas se hardware WASAPI real não estiver ativo)
        let _audio_worker = if active_streams.is_none() {
            let duplex_clone = duplex.clone();
            let telemetry_clone = telemetry.clone();
            let dsp_chain_clone = dsp_chain.clone();
            let r_worker = r.clone();
            Some(thread::spawn(move || {
                let block = 128;
                let mut read_buf = vec![0.0f32; block];
                let mut out_buf = vec![0.0f32; block];
                while r_worker.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(3));
                    duplex_clone.pop_samples_for_processing(&mut read_buf);
                    if let Ok(mut dsp) = dsp_chain_clone.lock() {
                        dsp.process(&read_buf, &mut out_buf);
                    }
                    let peak = out_buf.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
                    let peak_db = if peak > 1e-6 { 20.0 * peak.log10() } else { -120.0 };
                    telemetry_clone.update_peaks(peak_db - 6.0, peak_db, 1.2);
                    telemetry_clone.push_waveform(&out_buf);
                }
            }))
        } else {
            None
        };

        // Mantém os streams de hardware WASAPI vivos enquanto o app roda
        let _hold_streams = active_streams;

        // Espera término gracioso se não for interativo
        if is_headless {
            thread::sleep(Duration::from_millis(500));
        } else {
            // Em execução interativa padrão, mantém rodando
            while r.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(500));
            }
        }
        r.store(false, Ordering::Relaxed);
        if let Some(worker) = _audio_worker {
            let _ = worker.join();
        }
    }
}

#[allow(dead_code)]
fn which_cmd() -> Result<String, ()> {
    if Path::new("/mnt/c/Windows/system32/cmd.exe").exists() {
        Ok("/mnt/c/Windows/system32/cmd.exe".to_string())
    } else {
        Err(())
    }
}
