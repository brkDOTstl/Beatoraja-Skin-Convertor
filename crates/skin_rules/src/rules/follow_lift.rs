use skin_core::{CsvRecord, analyze::follow_lift_kind};

use crate::{
    edit::{DST_OP4_FIELD, is_play_file, replace_record},
    model::{EditableFile, FileChange},
};

pub(crate) fn apply(files: &mut [EditableFile], changes: &mut Vec<FileChange>) {
    for file in files.iter_mut().filter(|file| is_play_file(file)) {
        let mut active = false;
        for line_index in 0..file.document.line_count() {
            let Some(content) = file.document.line_content(line_index).map(Vec::from) else {
                continue;
            };
            let mut record = CsvRecord::from_line(line_index, &content);
            if record
                .command()
                .is_some_and(|command| command.starts_with("#SRC_"))
            {
                active = follow_lift_kind(&record).is_some();
                continue;
            }
            if !active && record.is_command("#DST_IMAGE") && record.field_equals_ascii(17, "140") {
                active = true;
            }
            if active
                && record.is_command("#DST_IMAGE")
                && !record.field_equals_ascii(DST_OP4_FIELD, "3")
            {
                record.set_field_ascii(DST_OP4_FIELD, "3");
                replace_record(
                    file,
                    line_index,
                    content,
                    record,
                    "follow-lift-op4",
                    "set op4=3 on source-owned lane effect",
                    changes,
                );
            }
        }
    }
}
