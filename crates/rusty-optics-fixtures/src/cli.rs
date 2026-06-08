use std::path::PathBuf;

use crate::{
    error::FixtureError,
    fields::{
        bioelectric_circuit_visual_frame_json, planarian_bioelectric_visual_sequence_json,
        surface_field_visual_frame_json, surface_field_visual_sequence_json,
    },
    hand_mesh::{hand_mesh_browser_frame_json, hand_mesh_browser_frame_json_from_surface},
    summary::summary_json,
};

/// Runs the fixture CLI.
pub fn run(args: impl IntoIterator<Item = String>) -> Result<(), FixtureError> {
    let mut args = args.into_iter();
    let command = args.next().unwrap_or_else(|| "export".to_owned());
    match command.as_str() {
        "export" => export(args),
        "export-hand-mesh-browser" => export_hand_mesh_browser(args),
        "export-hand-mesh-browser-from-surface" => export_hand_mesh_browser_from_surface(args),
        "export-surface-field-preview" => export_surface_field_preview(args),
        "validate" => validate(),
        _ => Err(FixtureError::InvalidArgument(command)),
    }
}

fn export_surface_field_preview(
    args: impl IntoIterator<Item = String>,
) -> Result<(), FixtureError> {
    let mut check = false;
    let mut frame_output = PathBuf::from("fixtures/fields/surface_field_visual_frame.json");
    let mut sequence_output = PathBuf::from("fixtures/fields/surface_field_visual_sequence.json");
    let mut circuit_output = PathBuf::from("fixtures/fields/bioelectric_circuit_visual_frame.json");
    let mut planarian_output =
        PathBuf::from("fixtures/fields/planarian_bioelectric_visual_sequence.json");
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--check" => check = true,
            "--frame-output" => {
                let Some(path) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--frame-output requires a path".to_owned(),
                    ));
                };
                frame_output = PathBuf::from(path);
            }
            "--sequence-output" => {
                let Some(path) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--sequence-output requires a path".to_owned(),
                    ));
                };
                sequence_output = PathBuf::from(path);
            }
            "--circuit-output" => {
                let Some(path) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--circuit-output requires a path".to_owned(),
                    ));
                };
                circuit_output = PathBuf::from(path);
            }
            "--planarian-output" => {
                let Some(path) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--planarian-output requires a path".to_owned(),
                    ));
                };
                planarian_output = PathBuf::from(path);
            }
            _ => return Err(FixtureError::InvalidArgument(argument)),
        }
    }

    write_or_check_json(
        frame_output,
        surface_field_visual_frame_json()?,
        check,
        "surface field visual frame",
    )?;
    write_or_check_json(
        sequence_output,
        surface_field_visual_sequence_json()?,
        check,
        "surface field visual sequence",
    )?;
    write_or_check_json(
        circuit_output,
        bioelectric_circuit_visual_frame_json()?,
        check,
        "bioelectric circuit visual frame",
    )?;
    write_or_check_json(
        planarian_output,
        planarian_bioelectric_visual_sequence_json()?,
        check,
        "planarian bioelectric visual sequence",
    )
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

fn export_hand_mesh_browser_from_surface(
    args: impl IntoIterator<Item = String>,
) -> Result<(), FixtureError> {
    let mut check = false;
    let mut output = PathBuf::from("local-artifacts/hand_mesh/hand_mesh_browser_debug_frame.json");
    let mut surface_json: Option<PathBuf> = None;
    let mut source_frame_id = "external.mesh_surface.frame.0001".to_owned();
    let mut coordinate_count = 48_usize;
    let mut sdf_voxel_size = 0.008_f32;
    let mut include_sdf_particles = false;
    let mut particle_count = 80_usize;
    let mut particle_steps = 28_usize;
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--check" => check = true,
            "--include-sdf-particles" => include_sdf_particles = true,
            "--output" => {
                let Some(path) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--output requires a path".to_owned(),
                    ));
                };
                output = PathBuf::from(path);
            }
            "--surface-json" => {
                let Some(path) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--surface-json requires a path".to_owned(),
                    ));
                };
                surface_json = Some(PathBuf::from(path));
            }
            "--source-frame-id" => {
                let Some(value) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--source-frame-id requires a value".to_owned(),
                    ));
                };
                source_frame_id = value;
            }
            "--coordinate-count" => {
                let Some(value) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--coordinate-count requires a value".to_owned(),
                    ));
                };
                coordinate_count = parse_usize("--coordinate-count", &value)?;
            }
            "--sdf-voxel-size" => {
                let Some(value) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--sdf-voxel-size requires a value".to_owned(),
                    ));
                };
                sdf_voxel_size = parse_f32("--sdf-voxel-size", &value)?;
            }
            "--particle-count" => {
                let Some(value) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--particle-count requires a value".to_owned(),
                    ));
                };
                particle_count = parse_usize("--particle-count", &value)?;
            }
            "--particle-steps" => {
                let Some(value) = args.next() else {
                    return Err(FixtureError::InvalidArgument(
                        "--particle-steps requires a value".to_owned(),
                    ));
                };
                particle_steps = parse_usize("--particle-steps", &value)?;
            }
            _ => return Err(FixtureError::InvalidArgument(argument)),
        }
    }

    let surface_json = surface_json
        .ok_or_else(|| FixtureError::InvalidArgument("--surface-json is required".to_owned()))?;
    let surface_text = std::fs::read_to_string(&surface_json)?;
    let surface = serde_json::from_str(&surface_text)?;
    let json = hand_mesh_browser_frame_json_from_surface(
        surface,
        &source_frame_id,
        coordinate_count,
        sdf_voxel_size,
        include_sdf_particles,
        particle_count,
        particle_steps,
    )?;
    write_or_check_json(output, json, check, "external hand mesh browser frame")
}

fn validate() -> Result<(), FixtureError> {
    export(["--check".to_owned()])?;
    export_hand_mesh_browser(["--check".to_owned()])?;
    export_surface_field_preview(["--check".to_owned()])
}

fn parse_usize(label: &str, value: &str) -> Result<usize, FixtureError> {
    value
        .parse::<usize>()
        .map_err(|_| FixtureError::InvalidArgument(format!("{label} must be an unsigned integer")))
}

fn parse_f32(label: &str, value: &str) -> Result<f32, FixtureError> {
    value
        .parse::<f32>()
        .map_err(|_| FixtureError::InvalidArgument(format!("{label} must be a number")))
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
