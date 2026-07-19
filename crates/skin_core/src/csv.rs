use crate::document::TextDocument;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvRecord {
    pub line_index: usize,
    pub fields: Vec<Vec<u8>>,
}

impl CsvRecord {
    pub fn from_line(line_index: usize, content: &[u8]) -> Self {
        let fields = content.split(|byte| *byte == b',').map(Vec::from).collect();
        Self { line_index, fields }
    }

    pub fn command(&self) -> Option<String> {
        let first = self.fields.first()?;
        let trimmed = trim_ascii(first);
        let trimmed = trim_ascii(trimmed.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(trimmed));
        if !trimmed.starts_with(b"#") {
            return None;
        }
        let command = String::from_utf8_lossy(trimmed).to_ascii_uppercase();
        Some(command)
    }

    pub fn is_command(&self, command: &str) -> bool {
        self.command()
            .is_some_and(|actual| actual.eq_ignore_ascii_case(command))
    }

    pub fn field(&self, index: usize) -> Option<&[u8]> {
        self.fields.get(index).map(Vec::as_slice)
    }

    pub fn field_equals_ascii(&self, index: usize, expected: &str) -> bool {
        self.field(index)
            .map(|value| trim_ascii(value) == expected.as_bytes())
            .unwrap_or(false)
    }

    pub fn field_i32(&self, index: usize) -> Option<i32> {
        let value = trim_ascii(self.field(index)?);
        let value = std::str::from_utf8(value).ok()?;
        value.parse::<i32>().ok()
    }

    pub fn set_field_ascii(&mut self, index: usize, value: &str) {
        while self.fields.len() <= index {
            self.fields.push(Vec::new());
        }
        self.fields[index] = value.as_bytes().to_vec();
    }

    pub fn to_line(&self) -> Vec<u8> {
        join_fields(&self.fields)
    }
}

pub fn records_for_document(document: &TextDocument) -> Vec<CsvRecord> {
    document
        .lines()
        .iter()
        .enumerate()
        .map(|(index, line)| CsvRecord::from_line(index, &line.content))
        .collect()
}

pub fn join_fields(fields: &[Vec<u8>]) -> Vec<u8> {
    let mut output = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            output.push(b',');
        }
        output.extend_from_slice(field);
    }
    output
}

pub fn trim_ascii(value: &[u8]) -> &[u8] {
    let mut start = 0usize;
    let mut end = value.len();
    while start < end && value[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && value[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    &value[start..end]
}
