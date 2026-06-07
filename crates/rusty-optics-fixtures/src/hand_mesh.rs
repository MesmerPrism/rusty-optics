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
    serialize_frame(&build_hand_mesh_browser_frame()?)
}

/// Serializes a browser debug frame from an external Matter mesh surface.
pub fn hand_mesh_browser_frame_json_from_surface(
    surface: TriangleMeshSurface,
    source_frame_id: &str,
    coordinate_count: usize,
    sdf_voxel_size: f32,
) -> Result<String, FixtureError> {
    if source_frame_id.trim().is_empty() {
        return Err(FixtureError::InvalidArgument(
            "--source-frame-id must be non-empty".to_owned(),
        ));
    }
    if coordinate_count == 0 {
        return Err(FixtureError::InvalidArgument(
            "--coordinate-count must be non-zero".to_owned(),
        ));
    }
    if !sdf_voxel_size.is_finite() || sdf_voxel_size <= 0.0 {
        return Err(FixtureError::InvalidArgument(
            "--sdf-voxel-size must be finite and positive".to_owned(),
        ));
    }

    surface
        .validate()
        .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let id_token = id_token(&surface.surface_id);
    let scale = surface_scale(&surface)?;
    let ids = BrowserFrameIds::external(&id_token, source_frame_id);
    let config = BrowserFrameBuildConfig::external(scale, coordinate_count, sdf_voxel_size);
    serialize_frame(&build_browser_frame_from_surface(&surface, ids, config)?)
}

fn serialize_frame(frame: &MeshBrowserDebugFrame) -> Result<String, FixtureError> {
    let mut json = serde_json::to_string_pretty(frame)?;
    json.push('\n');
    Ok(json)
}

fn build_hand_mesh_browser_frame() -> Result<MeshBrowserDebugFrame, FixtureError> {
    let hand_frame = synthetic_hand_validation_mesh_frame();
    hand_frame
        .validate()
        .map_err(|error| FixtureError::Matter(error.to_string()))?;
    build_browser_frame_from_surface(
        hand_frame.surface(),
        BrowserFrameIds::synthetic_left(&hand_frame.frame_id),
        BrowserFrameBuildConfig::synthetic_left(),
    )
}

fn build_browser_frame_from_surface(
    surface: &TriangleMeshSurface,
    ids: BrowserFrameIds,
    config: BrowserFrameBuildConfig,
) -> Result<MeshBrowserDebugFrame, FixtureError> {
    let mesh = MeshDebugFrame::from_surface(ids.mesh_frame_id.clone(), surface)
        .map_err(|error| FixtureError::Optics(error.to_string()))?;
    let coordinate_map = MeshCoordinateMap::from_surface(
        ids.coordinate_map_id.clone(),
        surface,
        coordinate_sample_config(&ids, config.coordinate_count),
        MeshCoordinateFrameConfig {
            frame_config_id: ids.coordinate_frame_config_id.clone(),
            max_displacement: config.max_displacement,
            clamp_mode: MeshLocalDisplacementClampMode::Ellipsoid,
            ..MeshCoordinateFrameConfig::default()
        },
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let coordinates = MeshCoordinateVisual::from_coordinate_map(
        ids.coordinate_visual_id.clone(),
        &coordinate_map,
        config.coordinate_axis_length,
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))?;

    let mut collider = DynamicMeshCollider::new(DynamicMeshColliderConfig {
        collider_config_id: ids.collider_config_id.clone(),
        surface_inflation: config.collider_surface_inflation,
        contact_padding: config.collider_contact_padding,
        prefer_convex: true,
        max_convex_triangle_count: 96,
        diagnostic_shell_inflation: config.collider_shell_inflation,
        ..DynamicMeshColliderConfig::default()
    });
    let update = collider.update_from_surface(surface);
    let contact = collider.closest_point(config.contact_probe);
    let collider_visual = MeshColliderVisual::from_collider_payload_with_contact_scale(
        ids.collider_visual_id.clone(),
        &surface.surface_id,
        &update,
        collider.diagnostic_shell(),
        contact.as_ref(),
        config.collider_contact_marker_radius,
        config.collider_contact_normal_length,
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))?;

    let sdf_mesh = TriangleMeshSnapshot::new(
        ids.mesh_snapshot_id.clone(),
        surface.positions.clone(),
        surface.triangles.clone(),
    );
    let sdf = build_sdf_from_mesh(
        &sdf_mesh,
        MeshToSdfConfig {
            voxel_size: config.sdf_voxel_size,
            padding_voxels: 2,
            max_voxels: config.sdf_max_voxels,
            sign_mode: MeshSdfSignMode::TriangleNormal,
        },
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let sdf_slice = SdfSliceVisual::middle_z(ids.sdf_slice_visual_id.clone(), &sdf)
        .map_err(|error| FixtureError::Optics(error.to_string()))?;

    MeshBrowserDebugFrame::new(
        ids.browser_frame_id,
        ids.source_frame_id,
        mesh,
        coordinates,
        collider_visual,
        sdf_slice,
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))
}

fn coordinate_sample_config(
    ids: &BrowserFrameIds,
    coordinate_count: usize,
) -> MeshSurfaceSampleConfig {
    let mut config = MeshSurfaceSampleConfig::high_quality_surface_points(coordinate_count);
    config.sample_config_id = ids.sample_config_id.clone();
    config.sample_set_id = ids.sample_set_id.clone();
    config
}

#[derive(Clone, Debug)]
struct BrowserFrameIds {
    source_frame_id: String,
    mesh_frame_id: String,
    coordinate_map_id: String,
    coordinate_frame_config_id: String,
    coordinate_visual_id: String,
    collider_config_id: String,
    collider_visual_id: String,
    mesh_snapshot_id: String,
    sdf_slice_visual_id: String,
    browser_frame_id: String,
    sample_config_id: String,
    sample_set_id: String,
}

impl BrowserFrameIds {
    fn synthetic_left(source_frame_id: &str) -> Self {
        Self {
            source_frame_id: source_frame_id.to_owned(),
            mesh_frame_id: "mesh.debug.frame.synthetic_hand.left".to_owned(),
            coordinate_map_id: "mesh.coordinate_map.synthetic_hand.left".to_owned(),
            coordinate_frame_config_id: "mesh.coordinate_frame.synthetic_hand.left".to_owned(),
            coordinate_visual_id: "mesh.coordinate.visual.synthetic_hand.left".to_owned(),
            collider_config_id: "mesh.dynamic_collider.synthetic_hand.left".to_owned(),
            collider_visual_id: "mesh.collider.visual.synthetic_hand.left".to_owned(),
            mesh_snapshot_id: "mesh.snapshot.synthetic_hand.left".to_owned(),
            sdf_slice_visual_id: "mesh.sdf.slice.synthetic_hand.left".to_owned(),
            browser_frame_id: "mesh.browser_debug_frame.synthetic_hand.left".to_owned(),
            sample_config_id: "mesh.surface_sample.synthetic_hand_browser.left".to_owned(),
            sample_set_id: "mesh.surface_samples.synthetic_hand_browser.left".to_owned(),
        }
    }

    fn external(id_token: &str, source_frame_id: &str) -> Self {
        Self {
            source_frame_id: source_frame_id.to_owned(),
            mesh_frame_id: format!("mesh.debug.frame.{id_token}"),
            coordinate_map_id: format!("mesh.coordinate_map.{id_token}"),
            coordinate_frame_config_id: format!("mesh.coordinate_frame.{id_token}"),
            coordinate_visual_id: format!("mesh.coordinate.visual.{id_token}"),
            collider_config_id: format!("mesh.dynamic_collider.{id_token}"),
            collider_visual_id: format!("mesh.collider.visual.{id_token}"),
            mesh_snapshot_id: format!("mesh.snapshot.{id_token}"),
            sdf_slice_visual_id: format!("mesh.sdf.slice.{id_token}"),
            browser_frame_id: format!("mesh.browser_debug_frame.{id_token}"),
            sample_config_id: format!("mesh.surface_sample.{id_token}"),
            sample_set_id: format!("mesh.surface_samples.{id_token}"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct BrowserFrameBuildConfig {
    coordinate_count: usize,
    max_displacement: Vec3,
    coordinate_axis_length: f32,
    collider_surface_inflation: f32,
    collider_contact_padding: f32,
    collider_shell_inflation: f32,
    collider_contact_marker_radius: f32,
    collider_contact_normal_length: f32,
    contact_probe: Vec3,
    sdf_voxel_size: f32,
    sdf_max_voxels: usize,
}

impl BrowserFrameBuildConfig {
    fn synthetic_left() -> Self {
        Self {
            coordinate_count: 36,
            max_displacement: Vec3::new(0.035, 0.035, 0.050),
            coordinate_axis_length: 0.055,
            collider_surface_inflation: 0.012,
            collider_contact_padding: 0.018,
            collider_shell_inflation: 0.020,
            collider_contact_marker_radius: 0.018,
            collider_contact_normal_length: 0.08,
            contact_probe: Vec3::new(0.03, 0.19, 0.22),
            sdf_voxel_size: 0.070,
            sdf_max_voxels: 16_384,
        }
    }

    fn external(scale: SurfaceScale, coordinate_count: usize, sdf_voxel_size: f32) -> Self {
        let axis_length = (scale.longest_axis * 0.055).clamp(0.004, 0.018);
        Self {
            coordinate_count,
            max_displacement: Vec3::new(
                (scale.longest_axis * 0.16).clamp(0.006, 0.035),
                (scale.longest_axis * 0.16).clamp(0.006, 0.035),
                (scale.longest_axis * 0.22).clamp(0.008, 0.050),
            ),
            coordinate_axis_length: axis_length,
            collider_surface_inflation: (scale.longest_axis * 0.04).clamp(0.002, 0.012),
            collider_contact_padding: (scale.longest_axis * 0.06).clamp(0.003, 0.018),
            collider_shell_inflation: (scale.longest_axis * 0.06).clamp(0.004, 0.020),
            collider_contact_marker_radius: (scale.longest_axis * 0.014).clamp(0.0015, 0.005),
            collider_contact_normal_length: (scale.longest_axis * 0.13).clamp(0.012, 0.045),
            contact_probe: scale.center
                + Vec3::new(
                    scale.longest_axis * 0.12,
                    scale.longest_axis * 0.18,
                    scale.longest_axis * 0.16,
                ),
            sdf_voxel_size,
            sdf_max_voxels: 65_536,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SurfaceScale {
    center: Vec3,
    longest_axis: f32,
}

fn surface_scale(surface: &TriangleMeshSurface) -> Result<SurfaceScale, FixtureError> {
    let mut positions = surface.positions.iter().copied();
    let Some(first) = positions.next() else {
        return Err(FixtureError::Matter(
            "surface must contain positions".to_owned(),
        ));
    };
    let mut min = first;
    let mut max = first;
    for position in positions {
        min = min.min(position);
        max = max.max(position);
    }
    let size = max - min;
    let longest_axis = size.x.max(size.y).max(size.z).max(0.001);
    Ok(SurfaceScale {
        center: (min + max) * 0.5,
        longest_axis,
    })
}

fn id_token(value: &str) -> String {
    let mut token = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
            token.push(character.to_ascii_lowercase());
        } else {
            token.push('_');
        }
    }
    let token = token.trim_matches(['.', '_', '-']).to_owned();
    if token.is_empty() {
        "surface".to_owned()
    } else {
        token
    }
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
