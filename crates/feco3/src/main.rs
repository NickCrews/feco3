use std::path::PathBuf;

use clap_verbosity_flag::Verbosity;

use clap::{Parser, ValueEnum};

use feco3::writers::csv::CSVProcessor;
use feco3::writers::parquet::ParquetProcessor;
use feco3::FecFile;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    /// File path, or ":stdin:" to read from stdin
    input: String,

    /// Output directory
    #[arg(long, short, default_value = "out")]
    output: PathBuf,

    /// Writer to use
    #[arg(long, short, value_enum, default_value = "parquet")]
    writer: Writer,

    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Writer {
    Parquet,
    CSV,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();
    let mut fec = match cli.input.as_str() {
        ":stdin:" => FecFile::from_reader(Box::new(std::io::stdin())),
        _ => FecFile::from_path(&PathBuf::from(cli.input))?,
    };
    match cli.writer {
        Writer::Parquet => ParquetProcessor::new(cli.output, None).process(&mut fec)?,
        Writer::CSV => CSVProcessor::new(cli.output).process(&mut fec)?,
    };
    Ok(())
}
