//! The second section of an .FEC file (after the header, before the itemizations)
//! is the summary section.
//! This section contains a single line with summary information about the file.
use crate::line::Line;

#[derive(Debug, Clone, Default)]
pub struct Summary {
    form_type: String,
    filer_committee_id_number: String,
}

pub fn parse_summary(line: &Line) -> Result<Summary, String> {
    let mut summary = Summary::default();
    summary.form_type = get(line, "form_type")?;
    summary.filer_committee_id_number = get(line, "filer_committee_id_number")?;
    Ok(summary)
}

fn get(line: &Line, field_name: &str) -> Result<String, String> {
    Ok(line
        .get_value(field_name)
        .ok_or(format!("no '{}' in summary line", field_name))?
        .to_string())
}
