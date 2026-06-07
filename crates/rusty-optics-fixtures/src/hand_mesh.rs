use rusty_matter_mesh::{
    DynamicMeshCollider, DynamicMeshColliderConfig, HandValidationMeshFrame, Handedness,
    MeshCoordinateFrameConfig, MeshCoordinateMap, MeshLocalDisplacementClampMode,
    MeshSurfaceSampleConfig, TriangleMeshSurface,
};
use rusty_matter_model::{TriangleMeshSnapshot, Vec3};
use rusty_matter_sdf::{build_sdf_from_mesh, MeshSdfSignMode, MeshToSdfConfig};
use rusty_optics_mesh::{
    MeshBrowserDebugFrame, MeshColliderVisual, MeshCoordinateVisual, MeshDebugFrame, SdfSliceVisual,
};

use crate::error::FixtureError;

/// Serializes the deterministic hand-mesh browser debug frame.
pub fn hand_mesh_browser_frame_json() -> Result<String, FixtureError> {
    let mut json = serde_json::to_string_pretty(&build_hand_mesh_browser_frame()?)?;
    json.push('\n');
    Ok(json)
}

fn build_hand_mesh_browser_frame() -> Result<MeshBrowserDebugFrame, FixtureError> {
    let hand_frame = synthetic_hand_validation_mesh_frame();
    hand_frame
        .validate()
        .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let surface = hand_frame.surface();

    let mesh = MeshDebugFrame::from_surface("mesh.debug.frame.synthetic_hand.left", surface)
        .map_err(|error| FixtureError::Optics(error.to_string()))?;
    let coordinate_map = MeshCoordinateMap::from_surface(
        "mesh.coordinate_map.synthetic_hand.left",
        surface,
        coordinate_sample_config(),
        MeshCoordinateFrameConfig {
            frame_config_id: "mesh.coordinate_frame.synthetic_hand.left".to_owned(),
            max_displacement: Vec3::new(0.035, 0.035, 0.050),
            clamp_mode: MeshLocalDisplacementClampMode::Ellipsoid,
            ..MeshCoordinateFrameConfig::default()
        },
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let coordinates = MeshCoordinateVisual::from_coordinate_map(
        "mesh.coordinate.visual.synthetic_hand.left",
        &coordinate_map,
        0.055,
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))?;

    let mut collider = DynamicMeshCollider::new(DynamicMeshColliderConfig {
        collider_config_id: "mesh.dynamic_collider.synthetic_hand.left".to_owned(),
        surface_inflation: 0.012,
        contact_padding: 0.018,
        prefer_convex: true,
        max_convex_triangle_count: 96,
        diagnostic_shell_inflation: 0.020,
        ..DynamicMeshColliderConfig::default()
    });
    let update = collider.update_from_surface(surface);
    let contact = collider.closest_point(Vec3::new(0.03, 0.19, 0.22));
    let collider_visual = MeshColliderVisual::from_collider_payload(
        "mesh.collider.visual.synthetic_hand.left",
        &surface.surface_id,
        &update,
        collider.diagnostic_shell(),
        contact.as_ref(),
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))?;

    let sdf_mesh = TriangleMeshSnapshot::new(
        "mesh.snapshot.synthetic_hand.left",
        surface.positions.clone(),
        surface.triangles.clone(),
    );
    let sdf = build_sdf_from_mesh(
        &sdf_mesh,
        MeshToSdfConfig {
            voxel_size: 0.070,
            padding_voxels: 2,
            max_voxels: 16_384,
            sign_mode: MeshSdfSignMode::TriangleNormal,
        },
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let sdf_slice = SdfSliceVisual::middle_z("mesh.sdf.slice.synthetic_hand.left", &sdf)
        .map_err(|error| FixtureError::Optics(error.to_string()))?;

    MeshBrowserDebugFrame::new(
        "mesh.browser_debug_frame.synthetic_hand.left",
        &hand_frame.frame_id,
        mesh,
        coordinates,
        collider_visual,
        sdf_slice,
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))
}

fn coordinate_sample_config() -> MeshSurfaceSampleConfig {
    let mut config = MeshSurfaceSampleConfig::high_quality_surface_points(36);
    config.sample_config_id = "mesh.surface_sample.synthetic_hand_browser.left".to_owned();
    config.sample_set_id = "mesh.surface_samples.synthetic_hand_browser.left".to_owned();
    config
}

fn synthetic_hand_validation_mesh_frame() -> HandValidationMeshFrame {
    HandValidationMeshFrame::from_surface(
        "hand.validation_mesh.synthetic_debug.left.0001",
        Handedness::Left,
        "local_floor",
        "synthetic.hand_mesh_debug",
        0.125,
        synthetic_hand_surface(),
    )
}

fn synthetic_hand_surface() -> TriangleMeshSurface {
    let mut positions = Vec::new();
    let mut triangles = Vec::new();

    push_quad(&mut positions, &mut triangles, -0.32, -0.36, 0.32, 0.18);
    push_quad(&mut positions, &mut triangles, -0.29, 0.15, -0.18, 0.55);
    push_quad(&mut positions, &mut triangles, -0.15, 0.15, -0.04, 0.70);
    push_quad(&mut positions, &mut triangles, -0.01, 0.15, 0.10, 0.82);
    push_quad(&mut positions, &mut triangles, 0.13, 0.15, 0.24, 0.67);
    push_free_quad(
        &mut positions,
        &mut triangles,
        [(-0.33, -0.18), (-0.55, -0.05), (-0.46, 0.22), (-0.30, 0.09)],
    );

    TriangleMeshSurface::new(
        "mesh.synthetic_hand_debug_surface.left",
        positions,
        triangles,
    )
}

fn push_quad(
    positions: &mut Vec<Vec3>,
    triangles: &mut Vec<[u32; 3]>,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
) {
    push_free_quad(
        positions,
        triangles,
        [
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
        ],
    );
}

fn push_free_quad(
    positions: &mut Vec<Vec3>,
    triangles: &mut Vec<[u32; 3]>,
    corners: [(f32, f32); 4],
) {
    let base = u32::try_from(positions.len()).expect("fixture vertex count fits u32");
    positions.extend(corners.map(|(x, y)| curved_hand_point(x, y)));
    triangles.push([base, base + 1, base + 2]);
    triangles.push([base, base + 2, base + 3]);
}

fn curved_hand_point(x: f32, y: f32) -> Vec3 {
    let width_curve = (1.0 - (x / 0.62).abs()).max(0.0);
    Vec3::new(x, y, 0.028 * width_curve + 0.018 * y.max(-0.20))
}
