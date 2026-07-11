use std::{env, fs, path::PathBuf};

use color_eyre::eyre::{Context, Result};

fn main() -> Result<()> {
    let default_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("openapi/flora.openapi.json");
    let path = match env::args_os().nth(1) {
        Some(path) => PathBuf::from(path),
        None => default_path,
    };
    let document = flora::handlers::openapi_document();
    let mut json = serde_json::to_string_pretty(&document).context("serialize OpenAPI document")?;
    json.push('\n');
    let Some(parent) = path.parent() else {
        color_eyre::eyre::bail!("OpenAPI output path has no parent directory");
    };
    fs::create_dir_all(parent).context("create OpenAPI output directory")?;
    fs::write(&path, json)
        .with_context(|| format!("write OpenAPI document to {}", path.display()))?;
    println!("Generated {}", path.display());
    Ok(())
}
