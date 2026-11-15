use sonix_desktop::audio::DesktopAudioHost;

#[test]
fn test_ca_03_resolve_duplex_sample_rate_aligns_mismatched_defaults() {
    let supported_in = [44100, 48000];
    let supported_out = [48000, 96000];
    let (in_sr, out_sr) =
        DesktopAudioHost::resolve_duplex_sample_rate(44100, 48000, None, &supported_in, &supported_out)
            .expect("deve alinhar para 48000 Hz comum");
    assert_eq!(in_sr, 48000, "entrada deve alinhar à taxa da saída quando suportada");
    assert_eq!(out_sr, 48000, "saída mantém 48000 Hz");
}

#[test]
fn test_ca_03_resolve_duplex_sample_rate_falls_back_to_common_or_requested() {
    let supported_in = [44100];
    let supported_out = [44100, 48000];
    let (in_sr, out_sr) =
        DesktopAudioHost::resolve_duplex_sample_rate(44100, 48000, None, &supported_in, &supported_out)
            .expect("deve escolher taxa comum 44100 Hz");
    assert_eq!(in_sr, 44100);
    assert_eq!(out_sr, 44100, "saída deve alinhar à taxa comum suportada pela entrada");

    let req_unsupported =
        DesktopAudioHost::resolve_duplex_sample_rate(44100, 48000, Some(96000), &supported_in, &supported_out);
    assert!(
        req_unsupported.is_err(),
        "S1: taxa pedida não suportada (96000) nunca deve abrir o dispositivo, obtido {:?}",
        req_unsupported
    );

    let req_ok =
        DesktopAudioHost::resolve_duplex_sample_rate(44100, 48000, Some(44100), &supported_in, &supported_out);
    assert_eq!(req_ok, Ok((44100, 44100)));
}

#[test]
fn test_ca_03_resolve_duplex_sample_rate_rejects_when_no_common_rate_exists() {
    let supported_in = [44100];
    let supported_out = [48000, 96000];
    let res = DesktopAudioHost::resolve_duplex_sample_rate(
        44100,
        48000,
        None,
        &supported_in,
        &supported_out,
    );
    assert!(
        res.is_err(),
        "S2: sem taxa comum entre entrada e saída, deve recusar com erro em vez de desalinhar, obtido {:?}",
        res
    );
}
