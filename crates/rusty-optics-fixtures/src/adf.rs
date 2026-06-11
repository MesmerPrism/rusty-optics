use rusty_matter_adf::{build_adf_from_sdf_grid, AdfBuildConfig};
use rusty_matter_model::{TriangleMeshSnapshot, Vec3};
use rusty_matter_sdf::{build_sdf_from_mesh, MeshToSdfConfig};
use rusty_optics_mesh::AdfDebugVisual;

use crate::error::FixtureError;

/// Serializes the deterministic ADF debug visual fixture.
pub fn adf_debug_visual_json() -> Result<String, FixtureError> {
    let visual = build_adf_debug_visual()?;
    let mut json = serde_json::to_string_pretty(&visual)?;
    json.push('\n');
    Ok(json)
}

fn build_adf_debug_visual() -> Result<AdfDebugVisual, FixtureError> {
    let mesh = TriangleMeshSnapshot::new(
        "mesh.unit_triangle_adf_debug",
        vec![
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ],
        vec![[0, 1, 2]],
    );
    let sdf = build_sdf_from_mesh(
        &mesh,
        MeshToSdfConfig {
            voxel_size: 0.5,
            padding_voxels: 1,
            max_voxels: 1_000,
            ..MeshToSdfConfig::default()
        },
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let adf = build_adf_from_sdf_grid(
        &sdf,
        AdfBuildConfig {
            max_depth: 3,
            max_cells: 1_024,
            error_tolerance: 0.025,
        },
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;

    AdfDebugVisual::from_field("mesh.adf.debug_visual.unit_triangle", &adf)
        .map_err(|error| FixtureError::Optics(error.to_string()))
}
