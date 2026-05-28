#![allow(dead_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;

use asefile::AsepriteFile;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RgbaAsset {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub duration_ms: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AsepriteSet {
    pub frames: Vec<RgbaAsset>,
    pub tags: Vec<AsepriteTag>,
}

impl AsepriteSet {
    pub fn tag(&self, name: &str) -> Option<&AsepriteTag> {
        self.tags.iter().find(|tag| tag.name == name)
    }

    pub fn frames_for_tag(&self, name: &str) -> Option<&[RgbaAsset]> {
        let tag = self.tag(name)?;
        self.frames
            .get(tag.from_frame as usize..=tag.to_frame as usize)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AsepriteTag {
    pub name: String,
    pub from_frame: u32,
    pub to_frame: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TintMode {
    Multiply,
    ColorizeLuma,
}

#[derive(Debug)]
pub enum AsepriteLoadError {
    Parse {
        path: String,
        source: asefile::AsepriteParseError,
    },
    NotFound {
        path: String,
        attempts: Vec<String>,
    },
}

impl Display for AsepriteLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { path, source } => {
                write!(f, "failed to load aseprite file {path:?}: {source}")
            }
            Self::NotFound { path, attempts } => {
                write!(
                    f,
                    "failed to find aseprite file {path:?}; tried {attempts:?}"
                )
            }
        }
    }
}

impl Error for AsepriteLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parse { source, .. } => Some(source),
            Self::NotFound { .. } => None,
        }
    }
}

pub fn load_tinted_aseprite_set(
    path: impl AsRef<Path>,
    tint: [u8; 4],
    mode: TintMode,
) -> Result<AsepriteSet, AsepriteLoadError> {
    let ase = read_aseprite_file(path.as_ref())?;
    let mut frames = Vec::with_capacity(ase.num_frames() as usize);
    let tags = (0..ase.num_tags())
        .map(|tag_index| {
            let tag = ase.tag(tag_index);
            AsepriteTag {
                name: tag.name().to_string(),
                from_frame: tag.from_frame(),
                to_frame: tag.to_frame(),
            }
        })
        .collect();

    for frame_index in 0..ase.num_frames() {
        let frame = ase.frame(frame_index);
        let image = frame.image();
        let (width, height) = image.dimensions();
        let mut rgba = image.into_raw();
        tint_rgba(&mut rgba, tint, mode);

        frames.push(RgbaAsset {
            name: format!("frame_{frame_index}"),
            width,
            height,
            rgba,
            duration_ms: Some(frame.duration()),
        });
    }

    Ok(AsepriteSet { frames, tags })
}

#[cfg(not(feature = "trueos-blueprint"))]
fn read_aseprite_file(path: &Path) -> Result<AsepriteFile, AsepriteLoadError> {
    AsepriteFile::read_file(path).map_err(|source| AsepriteLoadError::Parse {
        path: path.display().to_string(),
        source,
    })
}

#[cfg(feature = "trueos-blueprint")]
fn read_aseprite_file(path: &Path) -> Result<AsepriteFile, AsepriteLoadError> {
    use std::io::Cursor;

    let path_text = path.to_string_lossy().replace('\\', "/");
    let candidates = blueprint_asset_path_candidates(&path_text);
    for candidate in &candidates {
        if let Ok(bytes) = trueos::vfs::read_file(candidate.as_bytes()) {
            return AsepriteFile::read(Cursor::new(bytes)).map_err(|source| {
                AsepriteLoadError::Parse {
                    path: candidate.clone(),
                    source,
                }
            });
        }
    }

    if let Some(bytes) = crate::embedded_assets::bytes_for(&path_text) {
        return AsepriteFile::read(Cursor::new(bytes)).map_err(|source| AsepriteLoadError::Parse {
            path: format!("embedded:{path_text}"),
            source,
        });
    }

    Err(AsepriteLoadError::NotFound {
        path: path_text,
        attempts: candidates,
    })
}

#[cfg(feature = "trueos-blueprint")]
fn blueprint_asset_path_candidates(path: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    push_unique_path_candidate(&mut candidates, path.to_string());

    if let Ok(root) = trueos::env::var("TRUEOS_APP_FS_ROOT") {
        let root = root.trim_matches('/');
        if !root.is_empty() {
            push_unique_path_candidate(&mut candidates, format!("{root}/{path}"));
            push_unique_path_candidate(&mut candidates, format!("/{root}/{path}"));
        }
    }

    push_unique_path_candidate(&mut candidates, format!("apps/tactics/{path}"));
    push_unique_path_candidate(&mut candidates, format!("/apps/tactics/{path}"));
    candidates
}

#[cfg(feature = "trueos-blueprint")]
fn push_unique_path_candidate(candidates: &mut Vec<String>, candidate: String) {
    if !candidates.iter().any(|existing| existing == &candidate) {
        candidates.push(candidate);
    }
}

pub fn tint_rgba(rgba: &mut [u8], tint: [u8; 4], mode: TintMode) {
    for pixel in rgba.chunks_exact_mut(4) {
        let source_alpha = pixel[3];
        match mode {
            TintMode::Multiply => {
                pixel[0] = multiply_u8(pixel[0], tint[0]);
                pixel[1] = multiply_u8(pixel[1], tint[1]);
                pixel[2] = multiply_u8(pixel[2], tint[2]);
            }
            TintMode::ColorizeLuma => {
                let luma = ((pixel[0] as u16 * 77 + pixel[1] as u16 * 150 + pixel[2] as u16 * 29)
                    >> 8) as u8;
                pixel[0] = multiply_u8(tint[0], luma);
                pixel[1] = multiply_u8(tint[1], luma);
                pixel[2] = multiply_u8(tint[2], luma);
            }
        }
        pixel[3] = multiply_u8(source_alpha, tint[3]);
    }
}

fn multiply_u8(a: u8, b: u8) -> u8 {
    ((a as u16 * b as u16 + 127) / 255) as u8
}
