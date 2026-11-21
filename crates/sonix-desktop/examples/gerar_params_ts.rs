//! Gera `apps/sonix-app/ui/src/params.generated.ts` a partir de `sonix_desktop::params::PARAMS`.

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_path = manifest_dir
        .join("..")
        .join("..")
        .join("apps")
        .join("sonix-app")
        .join("ui")
        .join("src")
        .join("params.generated.ts");

    let content = sonix_desktop::params::render_params_ts();
    std::fs::write(&out_path, content)?;
    println!("Gerado: {}", out_path.display());
    Ok(())
}
