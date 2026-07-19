use std::{collections::BTreeMap, path::Path};

use crate::model::EditableFile;

pub(crate) fn detect_internal_theme(files: &[EditableFile]) -> Option<String> {
    let mut counts: BTreeMap<String, (String, usize)> = BTreeMap::new();
    for file in files {
        for line in file.document.lines() {
            let text = String::from_utf8_lossy(&line.content).replace('/', "\\");
            let lower = text.to_ascii_lowercase();
            let needle = "lr2files\\theme\\";
            let Some(position) = lower.find(needle) else {
                continue;
            };
            let remainder = &text[position + needle.len()..];
            let name = remainder.split('\\').next().unwrap_or_default().trim();
            if name.is_empty() {
                continue;
            }
            let key = name.to_ascii_lowercase();
            let entry = counts.entry(key).or_insert_with(|| (name.to_owned(), 0));
            entry.1 = entry.1.saturating_add(1);
        }
    }
    counts
        .into_values()
        .max_by_key(|(_, count)| *count)
        .map(|(name, _)| name)
}

pub(crate) fn theme_dir_name(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_string_lossy().into_owned();
    (!name.is_empty()).then_some(name)
}
