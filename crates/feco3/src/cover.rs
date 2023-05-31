//! The second section of an .FEC file (after the header, before the itemizations)
//! is the cover line
//! This is a single line with summary information about the file.
//!
//! See the test case .fec files for examples.
use crate::record::Record;
use crate::schemas::{LineParser, LiteralLineParser};

#[derive(Debug, Clone, Default)]
pub struct Cover {
    form_type: String,
    filer_committee_id_number: String,
}

pub fn parse_cover_line<'a>(
    fec_version: &str,
    line: &mut impl Iterator<Item = &'a String>,
) -> Result<Cover, String> {
    let mut cover = Cover::default();
    let record = LiteralLineParser.parse_line(fec_version, line)?;
    cover.form_type = get(&record, "form_type")?;
    cover.filer_committee_id_number = get(&record, "filer_committee_id_number")?;
    Ok(cover)
}

fn get(record: &Record, field_name: &str) -> Result<String, String> {
    Ok(record
        .get_value(field_name)
        .ok_or(format!("no '{}' in cover line", field_name))?
        .to_string())
}
