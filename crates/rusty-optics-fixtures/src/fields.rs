use rusty_matter_fields::{
    SurfaceFieldDebugFrame, SurfaceFieldPerturbation, SurfaceFieldPerturbationEffect,
    SurfaceFieldState, SurfaceFieldSubstrate, SurfaceScalarField, SurfaceScalarFieldKind,
    SurfaceVectorField, SurfaceVectorFieldKind,
};
use rusty_matter_mesh::{MeshSurfaceSampleConfig, MeshSurfaceSamplePattern, TriangleMeshSurface};
use rusty_matter_model::Vec3;
use rusty_optics_mesh::SurfaceFieldVisualFrame;

use crate::error::FixtureError;

/// Serializes the deterministic surface-field visual frame.
pub fn surface_field_visual_frame_json() -> Result<String, FixtureError> {
    let source = build_surface_field_debug_frame()?;
    let visual = SurfaceFieldVisualFrame::from_matter_debug_frame(
        "fields.visual.frame.unit_square",
        &source,
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))?;
    let mut json = serde_json::to_string_pretty(&visual)?;
    json.push('\n');
    Ok(json)
}

fn build_surface_field_debug_frame() -> Result<SurfaceFieldDebugFrame, FixtureError> {
    let surface = unit_square_surface();
    let samples = surface
        .sample_points(&MeshSurfaceSampleConfig {
            sample_config_id: "mesh.surface_sample.optics_field_fixture".to_owned(),
            sample_set_id: "mesh.surface_samples.optics_field_fixture".to_owned(),
            point_count: 12,
            first_tier_neighbor_count: 3,
            second_tier_neighbor_count: 3,
            seed: 48_161,
            pattern: MeshSurfaceSamplePattern::LowDiscrepancy,
            ..MeshSurfaceSampleConfig::default()
        })
        .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let substrate =
        SurfaceFieldSubstrate::from_sample_set("fields.substrate.optics_unit_square", &samples)
            .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let node_count = substrate.node_count();
    let wound_values = (0..node_count)
        .map(|index| match index {
            0 => 1.0,
            1 => 0.76,
            2 => 0.52,
            _ => 0.0,
        })
        .collect::<Vec<_>>();
    let morphogen_values = substrate
        .nodes
        .iter()
        .map(|node| node.position.x.clamp(0.0, 1.0))
        .collect::<Vec<_>>();
    let polarity = substrate
        .nodes
        .iter()
        .map(|node| {
            if node.node_index == 3 || node.node_index == 4 {
                Vec3::new(-1.0, 0.0, 0.0)
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            }
        })
        .collect::<Vec<_>>();
    let state = SurfaceFieldState::new(
        "fields.state.optics_unit_square",
        &substrate,
        vec![
            SurfaceScalarField::constant(
                "field.vmem_like",
                SurfaceScalarFieldKind::VmemLike,
                node_count,
                0.5,
            ),
            SurfaceScalarField::new(
                "field.wound_signal",
                SurfaceScalarFieldKind::WoundSignal,
                wound_values,
            ),
            SurfaceScalarField::new(
                "field.morphogen",
                SurfaceScalarFieldKind::Morphogen,
                morphogen_values,
            ),
        ],
        vec![SurfaceVectorField::new(
            "field.polarity",
            SurfaceVectorFieldKind::Polarity,
            polarity,
        )],
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let perturbations = vec![
        SurfaceFieldPerturbation::new(
            "perturbation.wound.center",
            Some("field.wound_signal".to_owned()),
            vec![0, 1, 2],
            SurfaceFieldPerturbationEffect::WoundRegion { signal_value: 1.0 },
        ),
        SurfaceFieldPerturbation::new(
            "perturbation.polarity.invert",
            Some("field.polarity".to_owned()),
            vec![3, 4],
            SurfaceFieldPerturbationEffect::PolarityInversion,
        ),
    ];
    SurfaceFieldDebugFrame::from_contracts(
        "fields.debug_frame.optics_unit_square",
        &substrate,
        &state,
        &perturbations,
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))
}

fn unit_square_surface() -> TriangleMeshSurface {
    TriangleMeshSurface::new(
        "mesh.unit_square_surface",
        vec![
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ],
        vec![[0, 1, 2], [0, 2, 3]],
    )
}
