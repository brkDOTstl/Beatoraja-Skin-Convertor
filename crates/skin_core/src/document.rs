use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    CrLf,
    Lf,
}

impl LineEnding {
    pub fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::CrLf => b"\r\n",
            Self::Lf => b"\n",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub content: Vec<u8>,
    pub ending: Option<LineEnding>,
}

impl Line {
    pub fn new(content: Vec<u8>, ending: Option<LineEnding>) -> Self {
        Self { content, ending }
    }
}

#[derive(Debug, Clone)]
pub struct TextDocument {
    path: PathBuf,
    lines: Vec<Line>,
    dominant_line_ending: LineEnding,
}

impl TextDocument {
    pub fn from_bytes(path: PathBuf, bytes: Vec<u8>) -> Self {
        let mut lines = Vec::new();
        let mut index = 0;
        let mut crlf_count = 0usize;
        let mut lf_count = 0usize;

        while index < bytes.len() {
            let line_start = index;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }

            if index < bytes.len() {
                let has_cr = index > line_start && bytes[index - 1] == b'\r';
                let content_end = if has_cr { index - 1 } else { index };
                let ending = if has_cr {
                    crlf_count += 1;
                    LineEnding::CrLf
                } else {
                    lf_count += 1;
                    LineEnding::Lf
                };
                lines.push(Line::new(
                    bytes[line_start..content_end].to_vec(),
                    Some(ending),
                ));
                index += 1;
            } else {
                lines.push(Line::new(bytes[line_start..].to_vec(), None));
            }
        }

        let dominant_line_ending = if lf_count > crlf_count {
            LineEnding::Lf
        } else {
            LineEnding::CrLf
        };

        Self {
            path,
            lines,
            dominant_line_ending,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn dominant_line_ending(&self) -> LineEnding {
        self.dominant_line_ending
    }

    pub fn line_content(&self, index: usize) -> Option<&[u8]> {
        self.lines.get(index).map(|line| line.content.as_slice())
    }

    pub fn replace_line_content(&mut self, index: usize, content: Vec<u8>) -> Option<Vec<u8>> {
        let line = self.lines.get_mut(index)?;
        Some(std::mem::replace(&mut line.content, content))
    }

    pub fn insert_generated_line(&mut self, index: usize, content: Vec<u8>) {
        let line = Line::new(content, Some(self.dominant_line_ending));
        if index >= self.lines.len() {
            self.lines.push(line);
        } else {
            self.lines.insert(index, line);
        }
    }

    pub fn push_generated_line(&mut self, content: Vec<u8>) {
        self.lines
            .push(Line::new(content, Some(self.dominant_line_ending)));
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for line in &self.lines {
            bytes.extend_from_slice(&line.content);
            if let Some(ending) = line.ending {
                bytes.extend_from_slice(ending.as_bytes());
            }
        }
        bytes
    }
}
