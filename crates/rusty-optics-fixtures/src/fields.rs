use rusty_matter_fields::{
    SurfaceFieldDebugFrame, SurfaceFieldDebugFrameSequence, SurfaceFieldPerturbation,
    SurfaceFieldPerturbationEffect, SurfaceFieldRuntime, SurfaceFieldRuntimeConfig,
    SurfaceFieldState, SurfaceFieldSubstrate, SurfaceScalarField, SurfaceScalarFieldKind,
    SurfaceVectorField, SurfaceVectorFieldKind,
};
use rusty_matter_mesh::{MeshSurfaceSampleConfig, MeshSurfaceSamplePattern, TriangleMeshSurface};
use rusty_matter_model::Vec3;
use rusty_optics_mesh::{SurfaceFieldVisualFrame, SurfaceFieldVisualFrameSequence};

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

/// Serializes the deterministic surface-field visual sequence.
pub fn surface_field_visual_sequence_json() -> Result<String, FixtureError> {
    let source = build_surface_field_debug_sequence()?;
    let visual = SurfaceFieldVisualFrameSequence::from_matter_debug_sequence(
        "fields.visual.sequence.unit_square_dynamic",
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

fn build_surface_field_debug_sequence() -> Result<SurfaceFieldDebugFrameSequence, FixtureError> {
    let surface = unit_square_surface();
    let samples = surface
        .sample_points(&MeshSurfaceSampleConfig {
            sample_config_id: "mesh.surface_sample.optics_field_dynamic_fixture".to_owned(),
            sample_set_id: "mesh.surface_samples.optics_field_dynamic_fixture".to_owned(),
            point_count: 64,
            first_tier_neighbor_count: 4,
            second_tier_neighbor_count: 4,
            seed: 65_537,
            pattern: MeshSurfaceSamplePattern::LowDiscrepancy,
            ..MeshSurfaceSampleConfig::default()
        })
        .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let substrate = SurfaceFieldSubstrate::from_sample_set(
        "fields.substrate.optics_unit_square_dynamic",
        &samples,
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let node_count = substrate.node_count();
    let vmem_values = substrate
        .nodes
        .iter()
        .map(|node| 0.16 + (node.position.y - 0.5) * 0.18)
        .collect::<Vec<_>>();
    let morphogen_values = substrate
        .nodes
        .iter()
        .map(|node| node.position.x.clamp(0.0, 1.0))
        .collect::<Vec<_>>();
    let polarity = substrate
        .nodes
        .iter()
        .map(|node| normalize(Vec3::new(1.0, (node.position.y - 0.5) * 0.45, 0.0)))
        .collect::<Vec<_>>();
    let state = SurfaceFieldState::new(
        "fields.state.optics_unit_square_dynamic",
        &substrate,
        vec![
            SurfaceScalarField::new(
                "field.vmem_like",
                SurfaceScalarFieldKind::VmemLike,
                vmem_values,
            ),
            SurfaceScalarField::constant(
                "field.wound_signal",
                SurfaceScalarFieldKind::WoundSignal,
                node_count,
                0.0,
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

    let wound_nodes = nearest_nodes(&substrate, Vec3::new(0.28, 0.64, 0.0), 6);
    let vmem_nodes = nearest_nodes(&substrate, Vec3::new(0.50, 0.48, 0.0), 10);
    let polarity_nodes = nearest_nodes(&substrate, Vec3::new(0.72, 0.34, 0.0), 8);
    let coupling_nodes = nearest_nodes(&substrate, Vec3::new(0.36, 0.58, 0.0), 14);

    let mut wound = SurfaceFieldPerturbation::new(
        "perturbation.wound.dynamic_center",
        Some("field.wound_signal".to_owned()),
        wound_nodes,
        SurfaceFieldPerturbationEffect::WoundRegion { signal_value: 1.0 },
    );
    wound.duration_steps = 30;
    let mut vmem = SurfaceFieldPerturbation::new(
        "perturbation.vmem.dynamic_offset",
        Some("field.vmem_like".to_owned()),
        vmem_nodes,
        SurfaceFieldPerturbationEffect::DepolarizeRegion { delta: 0.12 },
    );
    vmem.start_step = 10;
    vmem.duration_steps = 36;
    let mut polarity_inversion = SurfaceFieldPerturbation::new(
        "perturbation.polarity.dynamic_inversion",
        Some("field.polarity".to_owned()),
        polarity_nodes,
        SurfaceFieldPerturbationEffect::PolarityInversion,
    );
    polarity_inversion.start_step = 18;
    let mut coupling = SurfaceFieldPerturbation::new(
        "perturbation.coupling.dynamic_wound_shell",
        None,
        coupling_nodes,
        SurfaceFieldPerturbationEffect::CouplingMultiplierChange { multiplier: 1.45 },
    );
    coupling.duration_steps = 90;

    let runtime = SurfaceFieldRuntime::new(SurfaceFieldRuntimeConfig {
        config_id: "fields.runtime.optics_dynamic_fixture".to_owned(),
        fixed_step_seconds: 1.0 / 30.0,
        max_steps_per_run: 240,
        scalar_diffusion_rate: 2.8,
        scalar_decay_rate: 0.18,
        second_tier_coupling_weight: 0.42,
        vector_alignment_rate: 3.2,
        vector_gradient_rate: 1.9,
        ..SurfaceFieldRuntimeConfig::default()
    })
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    runtime
        .run_debug_sequence(
            "fields.debug_sequence.optics_unit_square_dynamic",
            &substrate,
            &state,
            &[wound, vmem, polarity_inversion, coupling],
            120,
            3,
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

fn nearest_nodes(substrate: &SurfaceFieldSubstrate, center: Vec3, count: usize) -> Vec<usize> {
    let mut nodes = substrate
        .nodes
        .iter()
        .map(|node| (node.node_index, node.position.distance_squared(center)))
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.1.total_cmp(&right.1));
    nodes
        .into_iter()
        .take(count.min(substrate.node_count()))
        .map(|(node_index, _)| node_index)
        .collect()
}

fn normalize(vector: Vec3) -> Vec3 {
    let length = vector.length();
    if length > 1.0e-6 {
        vector / length
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    }
}
