#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemePathRebase {
    pub line_index: usize,
    pub before: Vec<u8>,
    pub after: Vec<u8>,
}

pub fn rebase_theme_path_line(
    line_index: usize,
    content: &[u8],
    source_theme: &str,
    target_theme: &str,
) -> Option<ThemePathRebase> {
    if source_theme.is_empty()
        || target_theme.is_empty()
        || source_theme.eq_ignore_ascii_case(target_theme)
    {
        return None;
    }

    let mut after = content.to_vec();
    after = replace_theme_component(
        &after,
        b"LR2files\\Theme\\",
        b"\\",
        source_theme.as_bytes(),
        target_theme.as_bytes(),
    );
    after = replace_theme_component(
        &after,
        b"LR2files/Theme/",
        b"/",
        source_theme.as_bytes(),
        target_theme.as_bytes(),
    );

    (after != content).then(|| ThemePathRebase {
        line_index,
        before: content.to_vec(),
        after,
    })
}

fn replace_theme_component(
    input: &[u8],
    prefix: &[u8],
    separator: &[u8],
    source_theme: &[u8],
    target_theme: &[u8],
) -> Vec<u8> {
    let mut needle = Vec::with_capacity(prefix.len() + source_theme.len() + separator.len());
    needle.extend_from_slice(prefix);
    needle.extend_from_slice(source_theme);
    needle.extend_from_slice(separator);

    let mut replacement = Vec::with_capacity(prefix.len() + target_theme.len() + separator.len());
    replacement.extend_from_slice(prefix);
    replacement.extend_from_slice(target_theme);
    replacement.extend_from_slice(separator);

    replace_all_ascii_case_insensitive(input, &needle, &replacement)
}

fn replace_all_ascii_case_insensitive(input: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    if needle.is_empty() {
        return input.to_vec();
    }

    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        let end = index + needle.len();
        if end <= input.len() && input[index..end].eq_ignore_ascii_case(needle) {
            output.extend_from_slice(replacement);
            index = end;
        } else {
            output.push(input[index]);
            index += 1;
        }
    }

    output
}
