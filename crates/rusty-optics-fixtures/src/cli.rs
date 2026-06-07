use std::path::PathBuf;

use crate::{error::FixtureError, summary::summary_json};

/// Runs the fixture CLI.
pub fn run(args: impl IntoIterator<Item = String>) -> Result<(), FixtureError> {
    let mut args = args.into_iter();
    let command = args.next().unwrap_or_else(|| "export".to_owned());
    match command.as_str() {
        "export" => export(args),
        "validate" => export(["--check".to_owned()]),
        _ => Err(FixtureError::InvalidArgument(command)),
    }
}

fn export(args: impl IntoIterator<Item = String>) -> Result<(), FixtureError> {
    let mut check = false;
    let mut output = PathBuf::from("fixtures/particle_visual_stack_summary.json");
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--check" => check = true,
            "--output" => {
                let Some(path) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--output requires a path".to_owned(),
                    ));
                };
                output = PathBuf::from(path);
            }
            _ => return Err(FixtureError::InvalidArgument(argument)),
        }
    }

    let json = summary_json()?;
    if check {
        let existing = std::fs::read_to_string(&output)?;
        if existing != json {
            return Err(FixtureError::FixtureMismatch(output.display().to_string()));
        }
        println!("[PASS] fixture summary {}", output.display());
        return Ok(());
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, json)?;
    println!("[WRITE] fixture summary {}", output.display());
    Ok(())
}
