use std::path::PathBuf;

use crate::{error::FixtureError, hand_mesh::hand_mesh_browser_frame_json, summary::summary_json};

/// Runs the fixture CLI.
pub fn run(args: impl IntoIterator<Item = String>) -> Result<(), FixtureError> {
    let mut args = args.into_iter();
    let command = args.next().unwrap_or_else(|| "export".to_owned());
    match command.as_str() {
        "export" => export(args),
        "export-hand-mesh-browser" => export_hand_mesh_browser(args),
        "validate" => validate(),
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
    write_or_check_json(output, json, check, "fixture summary")
}

fn export_hand_mesh_browser(args: impl IntoIterator<Item = String>) -> Result<(), FixtureError> {
    let mut check = false;
    let mut output = PathBuf::from("fixtures/hand_mesh/hand_mesh_browser_debug_frame.json");
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

    let json = hand_mesh_browser_frame_json()?;
    write_or_check_json(output, json, check, "hand mesh browser frame")
}

fn validate() -> Result<(), FixtureError> {
    export(["--check".to_owned()])?;
    export_hand_mesh_browser(["--check".to_owned()])
}

fn write_or_check_json(
    output: PathBuf,
    json: String,
    check: bool,
    label: &str,
) -> Result<(), FixtureError> {
    if check {
        let existing = std::fs::read_to_string(&output)?;
        if existing != json {
            return Err(FixtureError::FixtureMismatch(output.display().to_string()));
        }
        println!("[PASS] {label} {}", output.display());
        return Ok(());
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, json)?;
    println!("[WRITE] {label} {}", output.display());
    Ok(())
}
