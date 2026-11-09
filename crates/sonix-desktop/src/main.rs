use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use sonix_desktop::audio::{
    DesktopAudioHost, RealtimeDspChain, RealtimeDuplexEngine, TelemetryBuffer,
};
use sonix_desktop::midi::DesktopMidiHost;
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

    if let Some(in_dev) = physical_devices.iter().find(|d| d.is_input) {
        if audio_host.start_stream(in_dev, 48000, 128).is_ok() {
            println!("-> Captura física ativada: 48 kHz / 128 samples (WASAPI Low-Latency)");
        }
    }

    let midi_host = DesktopMidiHost::new();
    let ports = midi_host.enumerate_input_ports();
    println!("-> Portas MIDI USB detectadas: {}", ports.len());
    for p in &ports {
        println!("   * {}", p);
    }

    // Inicializa motor duplex e cadeia DSP em tempo real
    let duplex = Arc::new(RealtimeDuplexEngine::new(2048));
    let telemetry = Arc::new(TelemetryBuffer::new(512));
    let mut dsp_chain = RealtimeDspChain::new(48000.0);
    dsp_chain.set_gate_threshold_db(-45.0);
    dsp_chain.set_amp_gain(7.0);
    dsp_chain.set_amp_master(5.5);
    dsp_chain.set_cab_enabled(true);

    println!("-> Motor AmpForge™ DSP ativo: Noise Gate -> Brit 800 White-Box -> Virtual Cab IR.");

    // Inicia Servidor Local IPC/HTTP
    let server = match LocalAppServer::bind_port(7890) {
        Ok(s) => s,
        Err(_) => LocalAppServer::bind_ephemeral().expect("Falha ao ligar servidor em 127.0.0.1"),
    };

    let app_url = server.app_url();
    let app_cmd = server.build_desktop_app_command();
    println!("-> Servidor Desktop Local: {}", app_url);
    println!("-> Modo Janela Dedicada: {}", app_cmd);

    // Tenta abrir a janela dedicada no Windows se não for modo headless
    let is_headless = std::env::var("SONIX_HEADLESS").map(|v| v == "1").unwrap_or(false);
    if !is_headless {
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
                // Em WSL ou Linux, se cmd.exe estiver acessível, abre no navegador Windows do host
                if let Ok(cmd_path) = which_cmd() {
                    let _ = Command::new(cmd_path)
                        .args(["/c", "start", "msedge", &format!("--app={}", url), "--window-size=1280,820"])
                        .spawn();
                }
            }
        });
    }

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
        // Loop de processamento de áudio em tempo real simulado em background
        let duplex_clone = duplex.clone();
        let telemetry_clone = telemetry.clone();
        let r_worker = r.clone();
        let audio_worker = thread::spawn(move || {
            let block = 128;
            let mut read_buf = vec![0.0f32; block];
            let mut out_buf = vec![0.0f32; block];
            while r_worker.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(3));
                duplex_clone.pop_samples_for_processing(&mut read_buf);
                dsp_chain.process(&read_buf, &mut out_buf);
                let peak = out_buf.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
                let peak_db = if peak > 1e-6 { 20.0 * peak.log10() } else { -120.0 };
                telemetry_clone.update_peaks(peak_db - 6.0, peak_db, 1.2);
                telemetry_clone.push_waveform(&out_buf);
            }
        });

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
        let _ = audio_worker.join();
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
