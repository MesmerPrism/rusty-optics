use std::path::PathBuf;

use crate::{catalog::catalog_json, error::SchemaError};

/// Runs the schema CLI.
pub fn run(args: impl IntoIterator<Item = String>) -> Result<(), SchemaError> {
    let mut args = args.into_iter();
    let command = args.next().unwrap_or_else(|| "export".to_owned());
    match command.as_str() {
        "export" => export(args),
        "validate" => export(["--check".to_owned()]),
        _ => Err(SchemaError::InvalidArgument(command)),
    }
}

fn export(args: impl IntoIterator<Item = String>) -> Result<(), SchemaError> {
    let mut check = false;
    let mut output = PathBuf::from("schemas/catalog.json");
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--check" => check = true,
            "--output" => {
                let Some(path) = args.next() else {
                    return Err(SchemaError::InvalidArgument(
                        "--output requires a path".to_owned(),
                    ));
                };
                output = PathBuf::from(path);
            }
            _ => return Err(SchemaError::InvalidArgument(argument)),
        }
    }

    let json = catalog_json()?;
    if check {
        let existing = std::fs::read_to_string(&output)?;
        if existing != json {
            return Err(SchemaError::CatalogMismatch(output.display().to_string()));
        }
        println!("[PASS] schema catalog {}", output.display());
        return Ok(());
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, json)?;
    println!("[WRITE] schema catalog {}", output.display());
    Ok(())
}
