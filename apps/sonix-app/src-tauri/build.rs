fn main() {
    tauri_build::build();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        if let Ok(out_dir) = std::env::var("OUT_DIR") {
            let out = std::path::PathBuf::from(out_dir);
            for name in ["libresource.a", "resource.lib"] {
                let res = out.join(name);
                if res.exists() {
                    println!("cargo:rustc-link-arg-tests={}", res.display());
                    break;
                }
            }
        }
    }
}
