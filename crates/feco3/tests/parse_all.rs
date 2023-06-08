use std::path::PathBuf;

use feco3;

fn repo_root() -> PathBuf {
    PathBuf::from("../..")
}

#[test]
fn it_can_run_everything() {
    let fec_path = repo_root().join("test/fecs/slash_form.fec");
    let mut fec = feco3::FecFile::from_path(&fec_path).unwrap();
    let mut csv = feco3::writers::csv::CSVProcessor::new(PathBuf::from("tests/out"));
    csv.process(&mut fec).unwrap();
}
