//! The second section of an .FEC file (after the header, before the itemizations)
//! is the cover line
//! This is a single line with summary information about the file.
//!
//! See the test case .fec files for examples.
use crate::record::Record;

#[derive(Debug, Clone, Default)]
pub struct Cover {
    form_type: String,
    filer_committee_id_number: String,
}

pub fn parse_cover_record(line: &Record) -> Result<Cover, String> {
    let mut cover = Cover::default();
    cover.form_type = get(line, "form_type")?;
    cover.filer_committee_id_number = get(line, "filer_committee_id_number")?;
    Ok(cover)
}

fn get(line: &Record, field_name: &str) -> Result<String, String> {
    Ok(line
        .get_value(field_name)
        .ok_or(format!("no '{}' in cover line", field_name))?
        .to_string())
}
