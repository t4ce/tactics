#[cfg(feature = "trueos-blueprint")]
struct EmbeddedAsset {
    path: &'static str,
    bytes: &'static [u8],
}

#[cfg(feature = "trueos-blueprint")]
include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));

#[cfg(not(feature = "trueos-blueprint"))]
pub fn ensure_runtime_assets() {}

#[cfg(not(feature = "trueos-blueprint"))]
pub fn bytes_for(_path: &str) -> Option<&'static [u8]> {
    None
}

#[cfg(feature = "trueos-blueprint")]
pub fn ensure_runtime_assets() {
    let root = app_fs_root();

    let mut written = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for asset in ASSETS {
        let target = format!("{root}/{}", asset.path);
        if trueos::vfs::stat(target.as_bytes()).is_ok() {
            skipped += 1;
            continue;
        }

        if let Some((parent, _)) = target.rsplit_once('/') {
            if !parent.is_empty() && trueos::vfs::create_dir_all(parent.as_bytes()).is_err() {
                failed += 1;
                eprintln!("embedded-assets: failed to create {parent:?}");
                continue;
            }
        }

        match trueos::vfs::write_file(target.as_bytes(), asset.bytes) {
            Ok(()) => written += 1,
            Err(code) => {
                failed += 1;
                eprintln!("embedded-assets: failed to write {target:?} rc={code}");
            }
        }
    }

    eprintln!(
        "embedded-assets: hydrated app fs root={root:?} written={written} skipped={skipped} failed={failed}"
    );
}

#[cfg(feature = "trueos-blueprint")]
pub fn bytes_for(path: &str) -> Option<&'static [u8]> {
    let path = path.trim_start_matches('/');
    ASSETS
        .iter()
        .find(|asset| asset.path == path)
        .map(|asset| asset.bytes)
}

#[cfg(feature = "trueos-blueprint")]
fn app_fs_root() -> String {
    trueos::env::var("TRUEOS_APP_FS_ROOT")
        .ok()
        .map(|root| root.trim_matches('/').to_string())
        .filter(|root| !root.is_empty())
        .unwrap_or_else(|| {
            eprintln!("embedded-assets: TRUEOS_APP_FS_ROOT missing; using apps/tactics");
            "apps/tactics".to_string()
        })
}
