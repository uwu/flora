use crate::api_types::DeploymentFile;
use color_eyre::eyre::{Result, eyre};
use std::{fs, path::Path};

pub(crate) fn collect_files(root: &Path) -> Result<Vec<DeploymentFile>> {
    use ignore::WalkBuilder;

    let walker = WalkBuilder::new(root)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .hidden(true)
        .follow_links(false)
        .filter_entry(|entry| {
            let dominated_directory =
                |name| entry.file_type().is_some_and(|ft| ft.is_dir()) && entry.file_name() == name;
            !dominated_directory("node_modules")
                && !dominated_directory("target")
                && !dominated_directory("dist")
                && !dominated_directory(".output")
                && !dominated_directory(".next")
                && !dominated_directory(".nuxt")
                && !dominated_directory(".svelte-kit")
                && !dominated_directory("build")
                && !dominated_directory("out")
                && !dominated_directory(".turbo")
                && !dominated_directory(".cache")
                && !dominated_directory("coverage")
                && !dominated_directory(".parcel-cache")
                && !dominated_directory(".vite")
        })
        .build();

    let mut files = Vec::new();
    for result in walker {
        let entry = result.map_err(|err| eyre!("Failed to walk directory: {err}"))?;
        let path = entry.path();
        if path.is_dir() || !is_allowed_extension(path) {
            continue;
        }
        let contents = fs::read_to_string(path)
            .map_err(|err| eyre!("Failed to read {}: {err}", path.display()))?;
        let rel = path_to_relative(path, root)?;
        files.push(DeploymentFile {
            path: rel,
            contents,
        });
    }

    if files.is_empty() {
        return Err(eyre!("No files found under {}", root.display()));
    }
    Ok(files)
}

fn is_allowed_extension(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cts") => true,
        _ => false,
    }
}

pub(crate) fn path_to_relative(path: &Path, root: &Path) -> Result<String> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| eyre!("Entry file is not inside {}", root.display()))?;
    let rel = rel.to_string_lossy().replace('\\', "/");
    if rel.is_empty() {
        return Err(eyre!("Entry path is empty"));
    }
    Ok(rel)
}
