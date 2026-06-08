use rusty_matter_fields::{
    BioelectricCircuitConfig, BioelectricCircuitRuntime, BioelectricCircuitState,
    BioelectricConductanceEdge, BioelectricCurrentKind, BioelectricCurrentTerm, BioelectricGate,
    BioelectricGateSource, BioelectricMemoryState, BioelectricReadoutLayer,
    BioelectricVoltageField, BioelectricVoltageUnit, PlanarianBioelectricPresetConfig,
    PlanarianBioelectricScenarioKind, PlanarianBioelectricScenarioRun, PlanarianBodySurfaceSource,
    SurfaceFieldDebugFrame, SurfaceFieldPerturbation, SurfaceFieldPerturbationEffect,
    SurfaceFieldRuntime, SurfaceFieldRuntimeConfig, SurfaceFieldState, SurfaceFieldSubstrate,
    SurfaceScalarField, SurfaceScalarFieldKind, SurfaceVectorField, SurfaceVectorFieldKind,
};
use rusty_matter_mesh::{
    DynamicMeshCollider, DynamicMeshColliderConfig, MeshCoordinateFrameConfig, MeshCoordinateMap,
    MeshSurfaceSampleConfig, MeshSurfaceSamplePattern, TriangleMeshSurface,
};
use rusty_matter_model::{TriangleMeshSnapshot, Vec3};
use rusty_matter_sdf::{build_sdf_from_mesh, MeshToSdfConfig};
use rusty_optics_model::Vec2;

use crate::{
    BioelectricCircuitVisualFrame, MeshBrowserDebugFrame, MeshColliderVisual, MeshCoordinateVisual,
    MeshDebugFrame, PlanarianBioelectricEditIntent, PlanarianBioelectricVisualSequence,
    PlanarianPickSelection, SdfSliceVisual, SurfaceFieldVisualFrame,
    SurfaceFieldVisualFrameSequence,
};

#[test]
fn mesh_debug_frame_preserves_surface_topology() {
    let surface = sample_surface();
    let frame =
        MeshDebugFrame::from_surface("mesh.debug.frame.test", &surface).expect("mesh debug frame");

    assert_eq!(frame.source_surface_id, surface.surface_id);
    assert_eq!(frame.vertices.len(), 4);
    assert_eq!(frame.triangles.len(), 2);
    assert_eq!(frame.edges.len(), 5);
    assert_eq!(frame.topology_index_hash, surface.topology_key().index_hash);
}

#[test]
fn browser_frame_combines_mesh_coordinate_collider_and_sdf_payloads() {
    let surface = sample_surface();
    let mesh = MeshDebugFrame::from_surface("mesh.debug.frame.bundle", &surface)
        .expect("mesh debug frame");
    let coordinate_map = MeshCoordinateMap::from_surface(
        "mesh.coordinate_map.bundle",
        &surface,
        MeshSurfaceSampleConfig::high_quality_surface_points(8),
        MeshCoordinateFrameConfig::default(),
    )
    .expect("coordinate map");
    let coordinates = MeshCoordinateVisual::from_coordinate_map(
        "mesh.coordinate.visual.bundle",
        &coordinate_map,
        0.04,
    )
    .expect("coordinate visual");

    let mut collider = DynamicMeshCollider::new(DynamicMeshColliderConfig {
        surface_inflation: 0.02,
        diagnostic_shell_inflation: 0.01,
        ..DynamicMeshColliderConfig::default()
    });
    let update = collider.update_from_surface(&surface);
    let contact = collider.closest_point(Vec3::new(0.25, 0.25, 0.10));
    let collider_visual = MeshColliderVisual::from_collider_payload(
        "mesh.collider.visual.bundle",
        &surface.surface_id,
        &update,
        collider.diagnostic_shell(),
        contact.as_ref(),
    )
    .expect("collider visual");

    let sdf_mesh = TriangleMeshSnapshot::new(
        "mesh.snapshot.bundle",
        surface.positions.clone(),
        surface.triangles.clone(),
    );
    let sdf = build_sdf_from_mesh(
        &sdf_mesh,
        MeshToSdfConfig {
            voxel_size: 0.2,
            padding_voxels: 1,
            max_voxels: 4096,
            ..MeshToSdfConfig::default()
        },
    )
    .expect("sdf");
    let sdf_slice = SdfSliceVisual::middle_z("mesh.sdf.slice.bundle", &sdf).expect("sdf slice");

    let browser = MeshBrowserDebugFrame::new(
        "mesh.browser.frame.bundle",
        "hand.validation_mesh.test",
        mesh,
        coordinates,
        collider_visual,
        sdf_slice,
    )
    .expect("browser frame");

    assert_eq!(browser.source_surface_id, surface.surface_id);
    assert!(browser.coordinates.anchors.len() >= 8);
    assert!(!browser.collider.shell_edges.is_empty());
    assert!(!browser.sdf_slice.cells.is_empty());
}

#[test]
fn surface_field_visual_frame_resolves_layers_edges_and_regions() {
    let source = sample_surface_field_debug_frame();
    let frame = SurfaceFieldVisualFrame::from_matter_debug_frame(
        "fields.visual.frame.unit_square",
        &source,
    )
    .expect("field visual frame");

    assert_eq!(frame.nodes.len(), source.nodes.len());
    assert_eq!(frame.edges.len(), source.edges.len());
    assert_eq!(frame.scalar_layers.len(), source.scalar_layers.len());
    assert_eq!(frame.vector_layers.len(), source.vector_layers.len());
    assert_eq!(
        frame.perturbation_regions.len(),
        source.perturbation_regions.len()
    );
    assert!(frame.vector_layers[0]
        .arrows
        .iter()
        .any(|arrow| arrow.end.x < arrow.start.x));
}

#[test]
fn surface_field_visual_sequence_preserves_matter_timing() {
    let source = sample_surface_field_debug_sequence();
    let sequence = SurfaceFieldVisualFrameSequence::from_matter_debug_sequence(
        "fields.visual.sequence.unit_square",
        &source,
    )
    .expect("field visual sequence");

    assert_eq!(sequence.frames.len(), source.frames.len());
    assert_eq!(sequence.diagnostic_count, source.diagnostics.len());
    assert_eq!(sequence.step_count, source.step_count);
    assert_eq!(sequence.frames[0].step_index, 0);
    assert!(sequence.frames.last().expect("final frame").time_seconds > 0.0);
}

#[test]
fn bioelectric_circuit_visual_frame_resolves_circuit_layers() {
    let (substrate, mut circuit) = sample_bioelectric_circuit();
    let runtime =
        BioelectricCircuitRuntime::new(BioelectricCircuitConfig::default()).expect("runtime");
    let diagnostics = runtime
        .step_fixed(&substrate, &mut circuit, 0)
        .expect("circuit step");

    let frame = BioelectricCircuitVisualFrame::from_matter_circuit_state(
        "fields.visual.bioelectric_circuit.test",
        &substrate,
        &circuit,
        Some(&diagnostics),
    )
    .expect("bioelectric visual frame");

    assert_eq!(frame.nodes.len(), substrate.node_count());
    assert_eq!(frame.voltage_samples.len(), substrate.node_count());
    assert_eq!(
        frame.conductance_edges.len(),
        circuit.conductance_edges.len()
    );
    assert_eq!(frame.current_regions.len(), circuit.current_terms.len());
    assert_eq!(frame.memory_samples.len(), substrate.node_count());
    assert_eq!(frame.readout_layers.len(), 1);
    assert!(frame.conductance_edges.iter().any(|edge| edge.gated));
    assert!(frame
        .diagnostics
        .as_ref()
        .is_some_and(|diagnostics| diagnostics.active_gates > 0));
}

#[test]
fn planarian_visual_sequence_preserves_ap_regions_and_memory_readout() {
    let source = sample_planarian_run();
    let visual = PlanarianBioelectricVisualSequence::from_matter_planarian_run(
        "fields.visual.planarian_ap.test",
        &source,
    )
    .expect("planarian visual sequence");

    assert_eq!(visual.frames.len(), source.sequence.frames.len());
    assert_eq!(visual.region_bands.len(), 5);
    assert_eq!(visual.node_regions.len(), source.substrate.node_count());
    assert_eq!(
        visual.body_surface.surface_id,
        source.source_surface.surface_id
    );
    assert_eq!(
        visual.body_surface.topology_index_hash,
        source.source_surface.topology_key().index_hash
    );
    assert_eq!(
        visual.body_surface.vertices.len(),
        source.source_surface.vertex_count()
    );
    assert_eq!(
        visual.body_surface.triangles.len(),
        source.source_surface.triangle_count()
    );
    assert_eq!(visual.diagnostic_count, source.sequence.diagnostics.len());
    assert!(visual
        .region_bands
        .iter()
        .any(|band| band.region_id == "region_head"));
    assert!(visual
        .region_bands
        .iter()
        .any(|band| band.region_id == "region_tail"));

    let posterior_nodes = visual
        .node_regions
        .iter()
        .filter_map(|node| {
            (node.region_id == "region_tail" || node.region_id == "region_postpharyngeal_trunk")
                .then_some(node.node_index)
        })
        .collect::<Vec<_>>();
    let final_frame = visual.frames.last().expect("final frame");
    let memory_average = average_memory(final_frame, &posterior_nodes);
    let head_readout = average_readout(
        final_frame,
        "readout.planarian_ap.head_identity",
        &posterior_nodes,
    );

    assert!(memory_average > 0.35);
    assert!(head_readout > 0.50);
}

#[test]
fn damaged_planarian_visual_node_region_is_rejected() {
    let source = sample_planarian_run();
    let mut visual = PlanarianBioelectricVisualSequence::from_matter_planarian_run(
        "fields.visual.planarian_ap.invalid",
        &source,
    )
    .expect("planarian visual sequence");
    visual.node_regions[0].node_index = visual.node_regions.len();
    let error = visual.validate().expect_err("bad node region rejects");

    assert!(matches!(
        error,
        rusty_optics_model::OpticsError::InvalidPayload(_)
    ));
}

#[test]
fn damaged_planarian_body_surface_triangle_is_rejected() {
    let source = sample_planarian_run();
    let mut visual = PlanarianBioelectricVisualSequence::from_matter_planarian_run(
        "fields.visual.planarian_ap.invalid_body",
        &source,
    )
    .expect("planarian visual sequence");
    visual.body_surface.triangles[0].vertex_indices[0] =
        u32::try_from(visual.body_surface.vertices.len()).expect("vertex count fits u32");
    let error = visual
        .validate()
        .expect_err("bad body triangle target rejects");

    assert!(matches!(
        error,
        rusty_optics_model::OpticsError::InvalidPayload(_)
    ));
}

#[test]
fn planarian_pick_selection_and_edit_intent_reference_visual_targets() {
    let source = sample_planarian_run();
    let visual = PlanarianBioelectricVisualSequence::from_matter_planarian_run(
        "fields.visual.planarian_ap.interaction",
        &source,
    )
    .expect("planarian visual sequence");

    let selection = PlanarianPickSelection::from_sequence_node(
        "fields.planarian.pick.node_0003",
        &visual,
        3,
        Some(Vec2::new(0.14, -0.28)),
        0.52,
        Some(17),
    )
    .expect("pick selection");
    selection
        .validate_for_sequence(&visual)
        .expect("selection validates against visual sequence");
    assert_eq!(selection.node_index(), Some(3));

    let intent = PlanarianBioelectricEditIntent::add_node_voltage(
        "fields.planarian.intent.add_voltage.node_0003",
        &selection,
        Some(17),
        0.25,
    )
    .expect("edit intent");
    intent
        .validate_for_sequence(&visual)
        .expect("intent validates against visual sequence");
    intent
        .validate_for_selection(&selection)
        .expect("intent references source selection");
    assert_eq!(intent.expected_revision, Some(17));
}

#[test]
fn planarian_edge_pick_selection_builds_gate_edit_intent() {
    let source = sample_planarian_run();
    let visual = PlanarianBioelectricVisualSequence::from_matter_planarian_run(
        "fields.visual.planarian_ap.edge_interaction",
        &source,
    )
    .expect("planarian visual sequence");

    let selection = PlanarianPickSelection::from_sequence_conductance_edge(
        "fields.planarian.pick.edge_0002",
        &visual,
        2,
        Some(Vec2::new(-0.10, 0.22)),
        0.61,
        Some(21),
    )
    .expect("edge pick selection");
    selection
        .validate_for_sequence(&visual)
        .expect("edge selection validates against visual sequence");
    assert_eq!(selection.node_index(), None);
    assert_eq!(selection.edge_index(), Some(2));

    let intent = PlanarianBioelectricEditIntent::set_edge_gate_threshold(
        "fields.planarian.intent.set_gate.edge_0002",
        &selection,
        Some(21),
        0.18,
        None,
    )
    .expect("gate edit intent");
    intent
        .validate_for_sequence(&visual)
        .expect("gate intent validates against visual sequence");
    intent
        .validate_for_selection(&selection)
        .expect("gate intent references source edge selection");
    assert_eq!(intent.expected_revision, Some(21));
}

#[test]
fn damaged_planarian_pick_selection_metadata_is_rejected() {
    let source = sample_planarian_run();
    let visual = PlanarianBioelectricVisualSequence::from_matter_planarian_run(
        "fields.visual.planarian_ap.interaction_invalid",
        &source,
    )
    .expect("planarian visual sequence");
    let mut selection = PlanarianPickSelection::from_sequence_node(
        "fields.planarian.pick.invalid_node",
        &visual,
        4,
        Some(Vec2::ZERO),
        0.40,
        None,
    )
    .expect("pick selection");
    if let crate::PlanarianPickTarget::SurfaceNode { region_id, .. } = &mut selection.target {
        *region_id = "region_wrong".to_owned();
    }
    let error = selection
        .validate_for_sequence(&visual)
        .expect_err("stale node metadata rejects");

    assert!(matches!(
        error,
        rusty_optics_model::OpticsError::InvalidPayload(_)
    ));
}

#[test]
fn damaged_planarian_edit_intent_value_is_rejected() {
    let source = sample_planarian_run();
    let visual = PlanarianBioelectricVisualSequence::from_matter_planarian_run(
        "fields.visual.planarian_ap.interaction_bad_edit",
        &source,
    )
    .expect("planarian visual sequence");
    let selection = PlanarianPickSelection::from_sequence_node(
        "fields.planarian.pick.bad_edit_node",
        &visual,
        5,
        None,
        0.33,
        Some(2),
    )
    .expect("pick selection");
    let error = PlanarianBioelectricEditIntent::add_node_voltage(
        "fields.planarian.intent.bad_voltage",
        &selection,
        Some(2),
        f32::NAN,
    )
    .expect_err("non-finite edit rejects");

    assert!(matches!(
        error,
        rusty_optics_model::OpticsError::InvalidValue(_)
    ));
}

#[test]
fn damaged_planarian_edge_gate_intent_requires_edge_selection() {
    let source = sample_planarian_run();
    let visual = PlanarianBioelectricVisualSequence::from_matter_planarian_run(
        "fields.visual.planarian_ap.bad_edge_interaction",
        &source,
    )
    .expect("planarian visual sequence");
    let selection = PlanarianPickSelection::from_sequence_node(
        "fields.planarian.pick.bad_gate_node",
        &visual,
        6,
        None,
        0.29,
        Some(4),
    )
    .expect("node selection");
    let error = PlanarianBioelectricEditIntent::set_edge_gate_threshold(
        "fields.planarian.intent.bad_gate_target",
        &selection,
        Some(4),
        0.15,
        None,
    )
    .expect_err("gate edit rejects node selection");

    assert!(matches!(
        error,
        rusty_optics_model::OpticsError::InvalidPayload(_)
    ));
}

fn sample_surface() -> TriangleMeshSurface {
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

fn sample_surface_field_debug_frame() -> SurfaceFieldDebugFrame {
    let surface = sample_surface();
    let config = MeshSurfaceSampleConfig {
        point_count: 8,
        first_tier_neighbor_count: 2,
        second_tier_neighbor_count: 2,
        pattern: MeshSurfaceSamplePattern::LowDiscrepancy,
        ..MeshSurfaceSampleConfig::default()
    };
    let samples = surface.sample_points(&config).expect("samples");
    let substrate =
        SurfaceFieldSubstrate::from_sample_set("fields.substrate.visual_test", &samples)
            .expect("substrate");
    let node_count = substrate.node_count();
    let wound = (0..node_count)
        .map(|index| {
            if index < 2 {
                1.0 - index as f32 * 0.35
            } else {
                0.0
            }
        })
        .collect::<Vec<_>>();
    let polarity = (0..node_count)
        .map(|index| {
            if index == 2 {
                Vec3::new(-1.0, 0.0, 0.0)
            } else {
                Vec3::new(1.0, 0.0, 0.0)
            }
        })
        .collect::<Vec<_>>();
    let state = SurfaceFieldState::new(
        "fields.state.visual_test",
        &substrate,
        vec![SurfaceScalarField::new(
            "field.wound_signal",
            SurfaceScalarFieldKind::WoundSignal,
            wound,
        )],
        vec![SurfaceVectorField::new(
            "field.polarity",
            SurfaceVectorFieldKind::Polarity,
            polarity,
        )],
    )
    .expect("state");
    let perturbations = vec![SurfaceFieldPerturbation::new(
        "perturbation.wound.visual_test",
        Some("field.wound_signal".to_owned()),
        vec![0, 1],
        SurfaceFieldPerturbationEffect::WoundRegion { signal_value: 1.0 },
    )];
    SurfaceFieldDebugFrame::from_contracts(
        "fields.debug_frame.visual_test",
        &substrate,
        &state,
        &perturbations,
    )
    .expect("debug frame")
}

fn sample_surface_field_debug_sequence() -> rusty_matter_fields::SurfaceFieldDebugFrameSequence {
    let surface = sample_surface();
    let config = MeshSurfaceSampleConfig {
        point_count: 10,
        first_tier_neighbor_count: 3,
        second_tier_neighbor_count: 3,
        pattern: MeshSurfaceSamplePattern::LowDiscrepancy,
        ..MeshSurfaceSampleConfig::default()
    };
    let samples = surface.sample_points(&config).expect("samples");
    let substrate =
        SurfaceFieldSubstrate::from_sample_set("fields.substrate.visual_sequence_test", &samples)
            .expect("substrate");
    let node_count = substrate.node_count();
    let state = SurfaceFieldState::new(
        "fields.state.visual_sequence_test",
        &substrate,
        vec![SurfaceScalarField::constant(
            "field.wound_signal",
            SurfaceScalarFieldKind::WoundSignal,
            node_count,
            0.0,
        )],
        vec![SurfaceVectorField::constant(
            "field.polarity",
            SurfaceVectorFieldKind::Polarity,
            node_count,
            Vec3::new(1.0, 0.0, 0.0),
        )],
    )
    .expect("state");
    let mut perturbation = SurfaceFieldPerturbation::new(
        "perturbation.wound.visual_sequence_test",
        Some("field.wound_signal".to_owned()),
        vec![0, 1],
        SurfaceFieldPerturbationEffect::WoundRegion { signal_value: 1.0 },
    );
    perturbation.duration_steps = 3;
    let runtime =
        SurfaceFieldRuntime::new(SurfaceFieldRuntimeConfig::default()).expect("runtime config");
    runtime
        .run_debug_sequence(
            "fields.debug_sequence.visual_test",
            &substrate,
            &state,
            &[perturbation],
            9,
            3,
        )
        .expect("debug sequence")
}

fn sample_bioelectric_circuit() -> (SurfaceFieldSubstrate, BioelectricCircuitState) {
    let surface = sample_surface();
    let config = MeshSurfaceSampleConfig {
        sample_config_id: "mesh.surface_sample.bioelectric_visual_test".to_owned(),
        sample_set_id: "mesh.surface_samples.bioelectric_visual_test".to_owned(),
        point_count: 10,
        first_tier_neighbor_count: 3,
        second_tier_neighbor_count: 3,
        pattern: MeshSurfaceSamplePattern::LowDiscrepancy,
        ..MeshSurfaceSampleConfig::default()
    };
    let samples = surface.sample_points(&config).expect("samples");
    let substrate = SurfaceFieldSubstrate::from_sample_set(
        "fields.substrate.bioelectric_visual_test",
        &samples,
    )
    .expect("substrate");
    let node_count = substrate.node_count();
    let voltage_values = substrate
        .nodes
        .iter()
        .map(|node| (node.position.x - 0.5) * 0.3 + (node.position.y - 0.5) * 0.12)
        .collect::<Vec<_>>();
    let gate = BioelectricGate::new(
        "gate.bioelectric_visual_test",
        BioelectricGateSource::VoltageDifference,
        0.08,
        0.025,
        0.35,
        1.4,
    );
    let conductance_edges =
        BioelectricConductanceEdge::from_substrate_neighbors(&substrate, 0.16, 0.04, Some(gate))
            .expect("conductance");
    let mut source = BioelectricCurrentTerm::new(
        "current.bioelectric_visual_source",
        vec![0, 1],
        BioelectricCurrentKind::Constant { current: 0.6 },
    );
    source.duration_steps = 4;
    let current_terms = vec![
        BioelectricCurrentTerm::new(
            "current.bioelectric_visual_leak",
            Vec::new(),
            BioelectricCurrentKind::Leak {
                conductance: 0.15,
                reversal_voltage: 0.0,
            },
        ),
        source,
    ];
    let memory = BioelectricMemoryState::zeroed(
        "memory.bioelectric_visual_hysteresis",
        node_count,
        0.24,
        -0.15,
        2.0,
        0.5,
    );
    let readout = BioelectricReadoutLayer::new(
        "readout.bioelectric_visual_voltage",
        vec![0.0; node_count],
        0.75,
        0.45,
        0.05,
        1.4,
        -1.0,
        1.0,
    );
    let circuit = BioelectricCircuitState::new(
        "circuit.bioelectric_visual_test",
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
    .expect("circuit");
    (substrate, circuit)
}

fn sample_planarian_run() -> PlanarianBioelectricScenarioRun {
    PlanarianBioelectricScenarioRun::build(
        PlanarianBioelectricScenarioKind::TransientDepolarizationMemory,
        PlanarianBioelectricPresetConfig {
            body_surface_source: PlanarianBodySurfaceSource::SyntheticAxis,
            sample_count: 64,
            step_count: 120,
            frame_stride: 12,
            seed: 88_003,
            ..PlanarianBioelectricPresetConfig::default()
        },
    )
    .expect("planarian run")
}

fn average_memory(frame: &BioelectricCircuitVisualFrame, node_indices: &[usize]) -> f32 {
    let sum = node_indices
        .iter()
        .map(|node_index| frame.memory_samples[*node_index].value)
        .sum::<f32>();
    sum / node_indices.len() as f32
}

fn average_readout(
    frame: &BioelectricCircuitVisualFrame,
    layer_id: &str,
    node_indices: &[usize],
) -> f32 {
    let layer = frame
        .readout_layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("readout layer");
    let sum = node_indices
        .iter()
        .map(|node_index| layer.samples[*node_index].value)
        .sum::<f32>();
    sum / node_indices.len() as f32
}
