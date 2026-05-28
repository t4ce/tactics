use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR should be set"));
    let out_file = out_dir.join("embedded_assets.rs");

    if env::var_os("CARGO_FEATURE_TRUEOS_BLUEPRINT").is_none() {
        fs::write(&out_file, "pub static ASSETS: &[EmbeddedAsset] = &[];\n")
            .expect("embedded asset manifest should write");
        return;
    }

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    println!("cargo:rerun-if-changed=Cargo.toml");

    let mut files = Vec::new();
    for dir in metadata_array(&manifest_dir.join("Cargo.toml"), "asset-dirs") {
        collect_files(&manifest_dir.join(&dir), &dir, &mut files);
    }
    for file in metadata_array(&manifest_dir.join("Cargo.toml"), "asset-files") {
        let path = manifest_dir.join(&file);
        if path.is_file() {
            println!("cargo:rerun-if-changed={}", path.display());
            files.push((file, path));
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));

    let mut generated = String::from("pub static ASSETS: &[EmbeddedAsset] = &[\n");
    for (relative, absolute) in files {
        generated.push_str("    EmbeddedAsset { path: ");
        generated.push_str(&format!("{relative:?}"));
        generated.push_str(", bytes: include_bytes!(");
        generated.push_str(&format!("{:?}", absolute.display().to_string()));
        generated.push_str(") },\n");
    }
    generated.push_str("];\n");

    fs::write(&out_file, generated).expect("embedded asset manifest should write");
}

fn collect_files(dir: &Path, relative_dir: &str, files: &mut Vec<(String, PathBuf)>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let relative = format!("{relative_dir}/{name}");
        if path.is_dir() {
            collect_files(&path, &relative, files);
        } else if path.is_file() {
            println!("cargo:rerun-if-changed={}", path.display());
            files.push((relative, path));
        }
    }
}

fn metadata_array(manifest_path: &Path, key: &str) -> Vec<String> {
    let Ok(cargo_toml) = fs::read_to_string(manifest_path) else {
        return Vec::new();
    };
    let mut in_metadata = false;
    for line in cargo_toml.lines() {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if trimmed.starts_with('[') {
            in_metadata = trimmed == "[package.metadata.trueos-blueprint]";
            continue;
        }
        if !in_metadata {
            continue;
        }
        let Some((current_key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if current_key.trim() == key {
            return parse_string_array(value);
        }
    }
    Vec::new()
}

fn parse_string_array(value: &str) -> Vec<String> {
    let Some(inner) = value
        .trim()
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    else {
        return Vec::new();
    };
    inner
        .split(',')
        .map(str::trim)
        .filter_map(toml_string_value)
        .collect()
}

fn toml_string_value(value: &str) -> Option<String> {
    let inner = value.trim().strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next()? {
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            escaped => out.push(escaped),
        }
    }
    Some(out)
}
