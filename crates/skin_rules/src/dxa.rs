use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    error::{RuleError, RuleResult},
    model::FileChange,
};

const DECODER_BYTES: &[u8] = include_bytes!("../assets/DxaDecode.exe");
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) struct DxaArchiveResult {
    relative_archive: PathBuf,
    relative_output: PathBuf,
}

#[derive(Debug, Default)]
pub(crate) struct DxaUnpackReport {
    archives: Vec<DxaArchiveResult>,
    pub(crate) extracted_files: Vec<PathBuf>,
}

impl DxaUnpackReport {
    pub(crate) fn changes(&self) -> Vec<FileChange> {
        self.archives
            .iter()
            .map(|archive| FileChange {
                rule_id: "dxa-font-unpack".to_owned(),
                path: archive.relative_archive.clone(),
                line: None,
                message: "Unpack DXA archive for beatoraja font compatibility.".to_owned(),
                before: archive.relative_archive.display().to_string(),
                after: archive.relative_output.display().to_string(),
            })
            .collect()
    }
}

pub(crate) fn unpack_archives(root: &Path) -> RuleResult<DxaUnpackReport> {
    let mut archives = Vec::new();
    collect_dxa_archives(root, &mut archives)?;
    archives.sort();
    if archives.is_empty() {
        return Ok(DxaUnpackReport::default());
    }

    let decoder = EmbeddedDecoder::create()?;
    let mut results = Vec::with_capacity(archives.len());
    let mut extracted_files = BTreeSet::new();

    for archive in archives {
        decode_archive(&decoder.executable, &archive)?;
        let output_directory = archive.with_extension("");
        if !output_directory.is_dir() {
            return Err(RuleError::DxaDecodeOutputMissing {
                archive,
                expected: output_directory,
            });
        }

        let mut output_files = Vec::new();
        collect_file_paths(&output_directory, &mut output_files)?;
        for path in output_files {
            extracted_files.insert(relative_to(root, &path));
        }
        results.push(DxaArchiveResult {
            relative_archive: relative_to(root, &archive),
            relative_output: relative_to(root, &output_directory),
        });
    }

    Ok(DxaUnpackReport {
        archives: results,
        extracted_files: extracted_files.into_iter().collect(),
    })
}

fn collect_dxa_archives(directory: &Path, output: &mut Vec<PathBuf>) -> RuleResult<()> {
    let entries =
        fs::read_dir(directory).map_err(|source| RuleError::io(directory.to_path_buf(), source))?;
    for entry in entries {
        let entry = entry.map_err(|source| RuleError::io(directory.to_path_buf(), source))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| RuleError::io(path.clone(), source))?;
        if file_type.is_dir() {
            collect_dxa_archives(&path, output)?;
        } else if file_type.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("dxa"))
        {
            output.push(path);
        }
    }
    Ok(())
}

fn collect_file_paths(directory: &Path, output: &mut Vec<PathBuf>) -> RuleResult<()> {
    let entries =
        fs::read_dir(directory).map_err(|source| RuleError::io(directory.to_path_buf(), source))?;
    for entry in entries {
        let entry = entry.map_err(|source| RuleError::io(directory.to_path_buf(), source))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| RuleError::io(path.clone(), source))?;
        if file_type.is_dir() {
            collect_file_paths(&path, output)?;
        } else if file_type.is_file() {
            output.push(path);
        }
    }
    Ok(())
}

fn decode_archive(decoder: &Path, archive: &Path) -> RuleResult<()> {
    let mut command = Command::new(decoder);
    command.arg(archive);
    if let Some(parent) = archive.parent() {
        command.current_dir(parent);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let output = command
        .output()
        .map_err(|source| RuleError::DxaDecoderLaunch {
            archive: archive.to_path_buf(),
            source,
        })?;
    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let details = format!("{stdout}\n{stderr}").trim().to_owned();
    Err(RuleError::DxaDecodeFailed {
        archive: archive.to_path_buf(),
        status: output.status.to_string(),
        details,
    })
}

fn relative_to(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

struct EmbeddedDecoder {
    directory: PathBuf,
    executable: PathBuf,
}

impl EmbeddedDecoder {
    fn create() -> RuleResult<Self> {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "beatoraja-skin-convertor-dxa-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&directory)
            .map_err(|source| RuleError::io(directory.clone(), source))?;
        let executable = directory.join("DxaDecode.exe");
        fs::write(&executable, DECODER_BYTES)
            .map_err(|source| RuleError::io(executable.clone(), source))?;
        Ok(Self {
            directory,
            executable,
        })
    }
}

impl Drop for EmbeddedDecoder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}
