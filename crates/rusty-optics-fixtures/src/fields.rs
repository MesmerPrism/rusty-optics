use rusty_matter_fields::{
    BioelectricCircuitConfig, BioelectricCircuitRuntime, BioelectricCircuitState,
    BioelectricConductanceEdge, BioelectricCurrentKind, BioelectricCurrentTerm, BioelectricGate,
    BioelectricGateSource, BioelectricMemoryState, BioelectricReadoutLayer,
    BioelectricVoltageField, BioelectricVoltageUnit, PlanarianBioelectricPresetConfig,
    PlanarianBioelectricScenarioKind, PlanarianBioelectricScenarioRun, PlanarianBodySurfaceSource,
    SurfaceFieldDebugFrame, SurfaceFieldDebugFrameSequence, SurfaceFieldPerturbation,
    SurfaceFieldPerturbationEffect, SurfaceFieldRuntime, SurfaceFieldRuntimeConfig,
    SurfaceFieldState, SurfaceFieldSubstrate, SurfaceScalarField, SurfaceScalarFieldKind,
    SurfaceVectorField, SurfaceVectorFieldKind,
};
use rusty_matter_mesh::{MeshSurfaceSampleConfig, MeshSurfaceSamplePattern, TriangleMeshSurface};
use rusty_matter_model::Vec3;
use rusty_optics_mesh::{
    BioelectricCircuitVisualFrame, PlanarianBioelectricVisualSequence, SurfaceFieldVisualFrame,
    SurfaceFieldVisualFrameSequence,
};

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

/// Serializes the deterministic bioelectric circuit visual frame.
pub fn bioelectric_circuit_visual_frame_json() -> Result<String, FixtureError> {
    let (substrate, mut circuit) = build_bioelectric_circuit_state()?;
    let runtime = BioelectricCircuitRuntime::new(BioelectricCircuitConfig {
        config_id: "fields.bioelectric_circuit.optics_fixture".to_owned(),
        max_steps_per_run: 180,
        ..BioelectricCircuitConfig::default()
    })
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let diagnostics = runtime
        .step_fixed(&substrate, &mut circuit, 0)
        .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let visual = BioelectricCircuitVisualFrame::from_matter_circuit_state(
        "fields.visual.bioelectric_circuit.unit_square",
        &substrate,
        &circuit,
        Some(&diagnostics),
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))?;
    let mut json = serde_json::to_string_pretty(&visual)?;
    json.push('\n');
    Ok(json)
}

/// Serializes the deterministic planarian AP bioelectric visual sequence.
pub fn planarian_bioelectric_visual_sequence_json() -> Result<String, FixtureError> {
    let source = PlanarianBioelectricScenarioRun::build(
        PlanarianBioelectricScenarioKind::TransientDepolarizationMemory,
        PlanarianBioelectricPresetConfig {
            body_surface_source: PlanarianBodySurfaceSource::SyntheticAxis,
            sample_count: 80,
            step_count: 150,
            frame_stride: 15,
            seed: 130_363,
            ..PlanarianBioelectricPresetConfig::default()
        },
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let visual = PlanarianBioelectricVisualSequence::from_matter_planarian_run(
        "fields.visual.planarian_ap.transient_memory",
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

fn build_bioelectric_circuit_state(
) -> Result<(SurfaceFieldSubstrate, BioelectricCircuitState), FixtureError> {
    let surface = unit_square_surface();
    let samples = surface
        .sample_points(&MeshSurfaceSampleConfig {
            sample_config_id: "mesh.surface_sample.optics_bioelectric_fixture".to_owned(),
            sample_set_id: "mesh.surface_samples.optics_bioelectric_fixture".to_owned(),
            point_count: 12,
            first_tier_neighbor_count: 3,
            second_tier_neighbor_count: 3,
            seed: 48_161,
            pattern: MeshSurfaceSamplePattern::LowDiscrepancy,
            ..MeshSurfaceSampleConfig::default()
        })
        .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let substrate = SurfaceFieldSubstrate::from_sample_set(
        "fields.substrate.optics_bioelectric_unit_square",
        &samples,
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let node_count = substrate.node_count();
    let voltage_values = substrate
        .nodes
        .iter()
        .map(|node| 0.20 * (node.position.x - 0.5) + 0.12 * (node.position.y - 0.5))
        .collect::<Vec<_>>();
    let gate = BioelectricGate::new(
        "gate.optics_bioelectric_voltage_difference",
        BioelectricGateSource::VoltageDifference,
        0.07,
        0.018,
        0.3,
        1.65,
    );
    let conductance_edges =
        BioelectricConductanceEdge::from_substrate_neighbors(&substrate, 0.16, 0.045, Some(gate))
            .map_err(|error| FixtureError::Matter(error.to_string()))?;

    let source_nodes = nearest_nodes(&substrate, Vec3::new(0.28, 0.62, 0.0), 4);
    let sink_nodes = nearest_nodes(&substrate, Vec3::new(0.76, 0.38, 0.0), 4);
    let mut source = BioelectricCurrentTerm::new(
        "current.optics_bioelectric_source",
        source_nodes,
        BioelectricCurrentKind::Constant { current: 0.85 },
    );
    source.duration_steps = 24;
    let mut sink = BioelectricCurrentTerm::new(
        "current.optics_bioelectric_sink",
        sink_nodes,
        BioelectricCurrentKind::Constant { current: -0.35 },
    );
    sink.start_step = 8;
    sink.duration_steps = 40;
    let current_terms = vec![
        BioelectricCurrentTerm::new(
            "current.optics_bioelectric_leak",
            Vec::new(),
            BioelectricCurrentKind::Leak {
                conductance: 0.16,
                reversal_voltage: 0.0,
            },
        ),
        BioelectricCurrentTerm::new(
            "current.optics_bioelectric_pump",
            Vec::new(),
            BioelectricCurrentKind::Pump {
                rate: 0.10,
                target_voltage: 0.0,
            },
        ),
        BioelectricCurrentTerm::new(
            "current.optics_bioelectric_voltage_gate",
            Vec::new(),
            BioelectricCurrentKind::VoltageGated {
                max_conductance: 0.06,
                reversal_voltage: -0.25,
                threshold: 0.16,
                slope: 0.05,
            },
        ),
        source,
        sink,
    ];
    let memory = BioelectricMemoryState::zeroed(
        "memory.optics_bioelectric_hysteresis",
        node_count,
        0.24,
        -0.16,
        1.9,
        0.55,
    );
    let readout = BioelectricReadoutLayer::new(
        "readout.optics_bioelectric_voltage",
        vec![0.0; node_count],
        0.8,
        0.45,
        0.08,
        1.25,
        -1.0,
        1.0,
    );
    let circuit = BioelectricCircuitState::new(
        "circuit.optics_bioelectric_unit_square",
        &substrate,
        BioelectricVoltageField::new(
            "field.bioelectric_voltage",
            BioelectricVoltageUnit::Normalized,
            0.0,
            voltage_values,
        ),
        conductance_edges,
        current_terms,
        Some(memory),
        vec![readout],
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    Ok((substrate, circuit))
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
