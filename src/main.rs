use std::io::Write;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Parses a glTF file and dumps the validation data as JSON to stdout."
)]
struct Cli {
    /// The path to the glTF file to validate.
    input: std::path::PathBuf,
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let validator = gltf_validator::GltfValidator::new()?;
    let result = validator.run(&cli.input)?;

    Ok(writeln!(
        std::io::stdout(),
        "{}",
        serde_json::to_string_pretty(&result)?
    )?)
}

fn main() {
    run().unwrap()
}
