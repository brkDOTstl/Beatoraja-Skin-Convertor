use skin_core::CsvRecord;

use crate::{
    edit::replace_record,
    model::{EditableFile, FileChange},
};

const RULE_ID: &str = "lr2-poor-count-compat";
const SRC_NUMBER_VALUE_FIELD: usize = 11;
const LR2_POOR: i32 = 114;
const BEATORAJA_POOR_PLUS_MISS: i32 = 426;

pub(crate) fn apply(files: &mut [EditableFile], changes: &mut Vec<FileChange>) {
    for file in files {
        for index in 0..file.document.line_count() {
            let Some(before) = file.document.line_content(index).map(Vec::from) else {
                continue;
            };
            let mut record = CsvRecord::from_line(index, &before);
            if !record.is_command("#SRC_NUMBER")
                || record.field_i32(SRC_NUMBER_VALUE_FIELD) != Some(LR2_POOR)
            {
                continue;
            }

            record.set_field_ascii(
                SRC_NUMBER_VALUE_FIELD,
                &BEATORAJA_POOR_PLUS_MISS.to_string(),
            );
            replace_record(
                file,
                index,
                before,
                record,
                RULE_ID,
                "map LR2 POOR counter to beatoraja POOR + MISS",
                changes,
            );
        }
    }
}
