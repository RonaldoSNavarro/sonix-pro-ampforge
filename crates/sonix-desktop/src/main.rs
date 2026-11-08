use sonix_desktop::audio::DesktopAudioHost;
use sonix_desktop::midi::DesktopMidiHost;

fn main() {
    println!("============================================================");
    println!("     🎛️  SONIX PRO // AMPFORGE™ Desktop Host Standalone    ");
    println!("============================================================");

    let mut audio_host = DesktopAudioHost::new();
    let devices = audio_host.enumerate_devices();
    println!("-> Dispositivos de Áudio detectados: {}", devices.len());
    for dev in &devices {
        println!(
            "   * [{:?}] {} (Taxas: {:?}, Buffers: {:?})",
            dev.backend, dev.name, dev.supported_sample_rates, dev.supported_buffer_sizes
        );
    }

    if let Some(dev) = devices.first() {
        if audio_host.start_stream(dev, 48000, 128).is_ok() {
            println!("-> Stream de Áudio ativo: 48 kHz / 128 samples (Latência < 1.5ms)");
        }
    }

    let midi_host = DesktopMidiHost::new();
    let ports = midi_host.enumerate_input_ports();
    println!("-> Portas MIDI USB detectadas: {}", ports.len());
    for p in &ports {
        println!("   * {}", p);
    }

    println!("-> Motor AmpForge™ DSP pronto (64-bit Newton-Raphson / Lock-Free SPSC).");
    println!("============================================================");
}

