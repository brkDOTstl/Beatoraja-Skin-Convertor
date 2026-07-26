use std::{
    ffi::OsString,
    fs,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use skin_core::read_skin_files;

use crate::{
    error::{RuleError, RuleResult},
    model::{ConversionOptions, EditableFile, GeneratedAsset, OutputManifest, RuleWarning},
};

pub(crate) fn load_editable_files(root: &Path) -> RuleResult<Vec<EditableFile>> {
    Ok(read_skin_files(root)?
        .into_iter()
        .map(|file| EditableFile {
            relative_path: file.relative_path,
            kind: file.kind,
            document: file.document,
            dirty: false,
        })
        .collect())
}

pub(crate) fn write_dirty_files(root: &Path, files: &[EditableFile]) -> RuleResult<()> {
    for file in files.iter().filter(|file| file.dirty) {
        let path = root.join(&file.relative_path);
        fs::write(&path, file.document.to_bytes()).map_err(|source| RuleError::io(path, source))?;
    }
    Ok(())
}

pub(crate) fn write_generated_assets(root: &Path, assets: &[GeneratedAsset]) -> RuleResult<()> {
    for asset in assets {
        let path = root.join(&asset.relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|source| RuleError::io(parent.to_path_buf(), source))?;
        }
        fs::write(&path, &asset.bytes).map_err(|source| RuleError::io(path, source))?;
    }
    Ok(())
}

pub(crate) fn build_manifest(
    input: &Path,
    output: Option<&Path>,
    options: &ConversionOptions,
    selected_rules: &[String],
    files: &[EditableFile],
    generated_assets: &[GeneratedAsset],
    additional_changed_files: &[PathBuf],
    warnings: &[RuleWarning],
) -> RuleResult<OutputManifest> {
    let generated_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let mut changed_files = files
        .iter()
        .filter(|file| file.dirty)
        .map(|file| file.relative_path.clone())
        .chain(
            generated_assets
                .iter()
                .map(|asset| asset.relative_path.clone()),
        )
        .chain(additional_changed_files.iter().cloned())
        .collect::<Vec<_>>();
    changed_files.sort();
    changed_files.dedup();
    Ok(OutputManifest {
        app_version: env!("CARGO_PKG_VERSION").to_owned(),
        source_path: input.to_path_buf(),
        output_path: output.map(Path::to_path_buf),
        source_fingerprint: fingerprint_dir(input)?,
        generated_at_unix,
        preset: options.preset.clone(),
        active_rules: selected_rules.to_vec(),
        changed_files,
        warnings: warnings.to_vec(),
    })
}

pub(crate) fn validate_output_path(input: &Path, output: &Path) -> RuleResult<()> {
    let input_resolved = resolve_for_comparison(input)?;
    let output_resolved = resolve_for_comparison(output)?;
    let input_key = path_key(&input_resolved);
    let output_key = path_key(&output_resolved);
    if input_key == output_key {
        return Err(RuleError::OutputEqualsInput);
    }
    if output_key.starts_with(&format!("{input_key}\\")) {
        return Err(RuleError::OutputInsideInput);
    }
    if output.exists() {
        return Err(RuleError::OutputExists(output.to_path_buf()));
    }
    Ok(())
}

pub(crate) fn copy_dir_recursive(input: &Path, output: &Path) -> RuleResult<()> {
    fs::create_dir_all(output).map_err(|source| RuleError::io(output.to_path_buf(), source))?;
    copy_dir_contents(input, output)
}

fn copy_dir_contents(input: &Path, output: &Path) -> RuleResult<()> {
    let entries =
        fs::read_dir(input).map_err(|source| RuleError::io(input.to_path_buf(), source))?;
    for entry in entries {
        let entry = entry.map_err(|source| RuleError::io(input.to_path_buf(), source))?;
        let source_path = entry.path();
        let target_path = output.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|source| RuleError::io(source_path.clone(), source))?;
        if file_type.is_dir() {
            fs::create_dir_all(&target_path)
                .map_err(|source| RuleError::io(target_path.clone(), source))?;
            copy_dir_contents(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path)
                .map_err(|source| RuleError::io(source_path.clone(), source))?;
        }
    }
    Ok(())
}

fn resolve_for_comparison(path: &Path) -> RuleResult<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let current = std::env::current_dir().map_err(|source| RuleError::io(".", source))?;
        current.join(path)
    };
    let normalized = lexical_normalize(&absolute);
    let mut existing = normalized.as_path();
    let mut suffix = Vec::<OsString>::new();
    while !existing.exists() {
        let Some(name) = existing.file_name() else {
            break;
        };
        suffix.push(name.to_os_string());
        let Some(parent) = existing.parent() else {
            break;
        };
        existing = parent;
    }
    let mut resolved =
        fs::canonicalize(existing).map_err(|source| RuleError::io(existing, source))?;
    for component in suffix.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

fn fingerprint_dir(root: &Path) -> RuleResult<String> {
    let mut paths = Vec::new();
    collect_all_file_paths(root, &mut paths)?;
    paths.sort();
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut byte_count = 0u64;
    for path in &paths {
        let relative = path.strip_prefix(root).unwrap_or(path.as_path());
        update_hash(&mut hash, relative.to_string_lossy().as_bytes());
        let bytes = fs::read(path).map_err(|source| RuleError::io(path.clone(), source))?;
        byte_count = byte_count.saturating_add(bytes.len() as u64);
        update_hash(&mut hash, &bytes);
    }
    Ok(format!("{:x}-{byte_count:x}-{hash:016x}", paths.len()))
}

fn collect_all_file_paths(root: &Path, output: &mut Vec<PathBuf>) -> RuleResult<()> {
    let entries = fs::read_dir(root).map_err(|source| RuleError::io(root.to_path_buf(), source))?;
    for entry in entries {
        let entry = entry.map_err(|source| RuleError::io(root.to_path_buf(), source))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| RuleError::io(path.clone(), source))?;
        if file_type.is_dir() {
            collect_all_file_paths(&path, output)?;
        } else if file_type.is_file() {
            output.push(path);
        }
    }
    Ok(())
}

fn update_hash(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}
