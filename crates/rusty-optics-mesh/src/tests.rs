use rusty_matter_fields::{
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

use crate::{
    MeshBrowserDebugFrame, MeshColliderVisual, MeshCoordinateVisual, MeshDebugFrame,
    SdfSliceVisual, SurfaceFieldVisualFrame, SurfaceFieldVisualFrameSequence,
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
