use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    document::TextDocument,
    error::{CoreError, CoreResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkinFileKind {
    Csv,
    Lr2Skin,
}

#[derive(Debug, Clone)]
pub struct SkinFile {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub kind: SkinFileKind,
    pub document: TextDocument,
}

pub fn read_skin_files(root: impl AsRef<Path>) -> CoreResult<Vec<SkinFile>> {
    let root = root.as_ref();
    if !root.exists() {
        return Err(CoreError::MissingInput(root.to_path_buf()));
    }
    if !root.is_dir() {
        return Err(CoreError::NotDirectory(root.to_path_buf()));
    }

    let mut paths = Vec::new();
    collect_skin_paths(root, root, &mut paths)?;
    paths.sort();

    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let bytes = fs::read(&path).map_err(|source| CoreError::io(path.clone(), source))?;
        let kind = kind_for_path(&path).ok_or_else(|| CoreError::MissingInput(path.clone()))?;
        let relative_path = path
            .strip_prefix(root)
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| path.clone());
        let document = TextDocument::from_bytes(path.clone(), bytes);
        files.push(SkinFile {
            path,
            relative_path,
            kind,
            document,
        });
    }
    Ok(files)
}

fn collect_skin_paths(root: &Path, dir: &Path, output: &mut Vec<PathBuf>) -> CoreResult<()> {
    let entries = fs::read_dir(dir).map_err(|source| CoreError::io(dir.to_path_buf(), source))?;
    for entry in entries {
        let entry = entry.map_err(|source| CoreError::io(dir.to_path_buf(), source))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| CoreError::io(path.clone(), source))?;
        if file_type.is_dir() {
            collect_skin_paths(root, &path, output)?;
        } else if file_type.is_file() && kind_for_path(&path).is_some() {
            output.push(path);
        }
    }
    let _ = root;
    Ok(())
}

fn kind_for_path(path: &Path) -> Option<SkinFileKind> {
    let ext = path.extension()?.to_str()?;
    if ext.eq_ignore_ascii_case("csv") {
        Some(SkinFileKind::Csv)
    } else if ext.eq_ignore_ascii_case("lr2skin") {
        Some(SkinFileKind::Lr2Skin)
    } else {
        None
    }
}
