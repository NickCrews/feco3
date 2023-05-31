//! The second section of an .FEC file (after the header, before the itemizations)
//! is the cover line
//! This is a single line with summary information about the file.
//!
//! See the test case .fec files for examples.
use crate::record::Record;
use crate::schemas::{LineParser, LiteralLineParser};
use crate::Error;

#[derive(Debug, Clone, Default)]
pub struct Cover {
    form_type: String,
    filer_committee_id_number: String,
}

pub fn parse_cover_line<'a>(
    fec_version: &str,
    line: &mut impl Iterator<Item = &'a String>,
) -> Result<Cover, Error> {
    let mut cover = Cover::default();
    let line = line.collect::<Vec<&String>>();
    log::debug!("parsing cover line {} {:?}", fec_version, line);
    println!("parsing cover line {} {:?}", fec_version, line);
    let record = LiteralLineParser.parse_line(fec_version, &mut line.into_iter())?;
    cover.form_type = record.line_code.clone();
    cover.filer_committee_id_number = get(&record, "filer_committee_id_number")?;
    Ok(cover)
}

fn get(record: &Record, field_name: &str) -> Result<String, Error> {
    Ok(record
        .get_value(field_name)
        .ok_or(Error::CoverParseError(format!(
            "no '{}' in cover line",
            field_name
        )))?
        .to_string())
}
