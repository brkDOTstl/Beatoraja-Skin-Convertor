use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::model::RuleContext;

pub(crate) fn reference_exists(context: &RuleContext, theme: &str, reference: &str) -> bool {
    local_path(context, theme, reference).is_some_and(|path| {
        wildcard_exists(&path) || packed_parent_exists(&path, &context.output_root)
    })
}

fn local_path(context: &RuleContext, theme: &str, reference: &str) -> Option<PathBuf> {
    let normalized = reference.trim().replace('/', "\\");
    let lower = normalized.to_ascii_lowercase();
    let needle = format!("lr2files\\theme\\{}\\", theme.to_ascii_lowercase());
    let position = lower.find(&needle)?;
    let suffix = &normalized[position + needle.len()..];
    let mut path = context.output_root.clone();
    for component in suffix.split('\\').filter(|part| !part.is_empty()) {
        if component == "." || component == ".." || component.contains(':') {
            return None;
        }
        path.push(component);
    }
    Some(path)
}

fn wildcard_exists(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    if !name.contains('*') {
        return path.exists();
    }
    let Some(parent) = path.parent() else {
        return false;
    };
    let mut parts = name.splitn(2, '*');
    let prefix = parts.next().unwrap_or_default().to_ascii_lowercase();
    let suffix = parts.next().unwrap_or_default().to_ascii_lowercase();
    fs::read_dir(parent).is_ok_and(|entries| {
        entries.filter_map(Result::ok).any(|entry| {
            let candidate = entry.file_name().to_string_lossy().to_ascii_lowercase();
            candidate.starts_with(&prefix) && candidate.ends_with(&suffix)
        })
    })
}

fn packed_parent_exists(path: &Path, root: &Path) -> bool {
    let mut ancestor = path.parent();
    while let Some(directory) = ancestor {
        if directory == root {
            break;
        }
        if directory.with_extension("dxa").is_file() {
            return true;
        }
        ancestor = directory.parent();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::reference_exists;
    use crate::model::RuleContext;

    #[test]
    fn rejects_parent_directory_references() {
        let context = RuleContext {
            output_root: std::env::temp_dir(),
            source_theme_dir: None,
            output_theme_dir: None,
            detected_resolution: None,
            canvas_width: None,
            canvas_height: None,
        };
        assert!(!reference_exists(
            &context,
            "Example",
            r"LR2files\Theme\Example\..\outside.png"
        ));
    }
}
