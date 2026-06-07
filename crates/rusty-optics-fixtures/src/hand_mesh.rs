use rusty_matter_mesh::{
    DynamicMeshCollider, DynamicMeshColliderConfig, HandValidationMeshFrame, Handedness,
    MeshCoordinateFrameConfig, MeshCoordinateMap, MeshLocalDisplacementClampMode,
    MeshSurfaceSampleConfig, TriangleMeshSurface,
};
use rusty_matter_model::{TriangleMeshSnapshot, Vec3};
use rusty_matter_particles::{
    ParticleFixedStepConfig, ParticleRenderPayload, ParticleSet, ParticleSimulationDiagnostics,
    ParticleSimulator, ParticleState, SdfParticleInteractionConfig,
};
use rusty_matter_sdf::{build_sdf_from_mesh, MeshSdfSignMode, MeshToSdfConfig};
use rusty_optics_mesh::{
    MeshBrowserDebugFrame, MeshColliderVisual, MeshCoordinateVisual, MeshDebugFrame, SdfSliceVisual,
};
use rusty_optics_model::{ColorRgba, PARTICLE_SDF_BROWSER_OVERLAY_SCHEMA_ID};
use rusty_optics_particles::{to_half_open_frame01, ParticleVisualFrame};
use serde::Serialize;

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
    include_sdf_particles: bool,
    particle_count: usize,
    particle_steps: usize,
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
    if include_sdf_particles && particle_count == 0 {
        return Err(FixtureError::InvalidArgument(
            "--particle-count must be non-zero".to_owned(),
        ));
    }
    if include_sdf_particles && particle_steps == 0 {
        return Err(FixtureError::InvalidArgument(
            "--particle-steps must be non-zero".to_owned(),
        ));
    }

    surface
        .validate()
        .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let id_token = id_token(&surface.surface_id);
    let scale = surface_scale(&surface)?;
    let ids = BrowserFrameIds::external(&id_token, source_frame_id);
    let config = BrowserFrameBuildConfig::external(scale, coordinate_count, sdf_voxel_size);
    let build = build_browser_frame_from_surface(&surface, ids, config)?;
    let particle_overlay = if include_sdf_particles {
        Some(build_sdf_particle_overlay(
            &build,
            scale,
            particle_count,
            particle_steps,
        )?)
    } else {
        None
    };
    serialize_browser_payload(&build.frame, particle_overlay.as_ref())
}

fn serialize_frame(frame: &MeshBrowserDebugFrame) -> Result<String, FixtureError> {
    let mut json = serde_json::to_string_pretty(frame)?;
    json.push('\n');
    Ok(json)
}

fn serialize_browser_payload(
    frame: &MeshBrowserDebugFrame,
    particle_overlay: Option<&ParticleSdfBrowserOverlay>,
) -> Result<String, FixtureError> {
    let mut value = serde_json::to_value(frame)?;
    if let Some(particle_overlay) = particle_overlay {
        let Some(object) = value.as_object_mut() else {
            return Err(FixtureError::Optics(
                "browser frame must serialize as a JSON object".to_owned(),
            ));
        };
        object.insert(
            "particle_sdf_overlay".to_owned(),
            serde_json::to_value(particle_overlay)?,
        );
    }
    let mut json = serde_json::to_string_pretty(&value)?;
    json.push('\n');
    Ok(json)
}

fn build_hand_mesh_browser_frame() -> Result<MeshBrowserDebugFrame, FixtureError> {
    let hand_frame = synthetic_hand_validation_mesh_frame();
    hand_frame
        .validate()
        .map_err(|error| FixtureError::Matter(error.to_string()))?;
    Ok(build_browser_frame_from_surface(
        hand_frame.surface(),
        BrowserFrameIds::synthetic_left(&hand_frame.frame_id),
        BrowserFrameBuildConfig::synthetic_left(),
    )?
    .frame)
}

#[derive(Clone, Debug)]
struct BrowserFrameBuild {
    frame: MeshBrowserDebugFrame,
    coordinate_map: MeshCoordinateMap,
    sdf: rusty_matter_sdf::PackedSdfGrid,
}

fn build_browser_frame_from_surface(
    surface: &TriangleMeshSurface,
    ids: BrowserFrameIds,
    config: BrowserFrameBuildConfig,
) -> Result<BrowserFrameBuild, FixtureError> {
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

    let frame = MeshBrowserDebugFrame::new(
        ids.browser_frame_id,
        ids.source_frame_id,
        mesh,
        coordinates,
        collider_visual,
        sdf_slice,
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))?;
    Ok(BrowserFrameBuild {
        frame,
        coordinate_map,
        sdf,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ParticleSdfBrowserOverlay {
    schema_id: String,
    overlay_id: String,
    source_surface_id: String,
    source_sdf_grid_id: String,
    initial_particle_count: usize,
    final_particle_count: usize,
    simulation_steps: usize,
    diagnostics: ParticleSimulationDiagnostics,
    particles: ParticleVisualFrame,
    trails: Vec<ParticleTrailVisual>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
struct ParticleTrailVisual {
    start: Vec3,
    end: Vec3,
    color: ColorRgba,
}

fn build_sdf_particle_overlay(
    build: &BrowserFrameBuild,
    scale: SurfaceScale,
    particle_count: usize,
    particle_steps: usize,
) -> Result<ParticleSdfBrowserOverlay, FixtureError> {
    let particle_radius = (scale.longest_axis * 0.012).clamp(0.0018, 0.0042);
    let initial_set = seed_particles_from_coordinates(
        &build.coordinate_map,
        scale,
        particle_count,
        particle_radius,
    )?;
    let initial_positions = initial_set
        .particles
        .iter()
        .map(|particle| particle.position)
        .collect::<Vec<_>>();

    let fixed_step = ParticleFixedStepConfig {
        step_config_id: "particle.fixed_step.sdf_browser_overlay".to_owned(),
        max_steps_per_frame: u32::try_from(particle_steps).unwrap_or(u32::MAX).max(1),
        neighbor_radius: particle_radius * 4.0,
        neighbor_repulsion_strength: 0.05,
        ..ParticleFixedStepConfig::default()
    };
    let interaction = SdfParticleInteractionConfig {
        interaction_id: "interaction.sdf_browser_overlay".to_owned(),
        target_distance: particle_radius * 0.35,
        strength: 18.0,
        damping: 1.8,
        max_speed: (scale.longest_axis * 1.4).clamp(0.05, 0.35),
        ..SdfParticleInteractionConfig::default()
    };
    let mut simulator = ParticleSimulator::new(initial_set, fixed_step.clone(), interaction)
        .map_err(|error| FixtureError::Matter(error.to_string()))?;
    simulator.set_sdf(Some(build.sdf.clone()));
    let diagnostics = run_particle_steps(&mut simulator, &fixed_step, particle_steps);
    let render_payload = ParticleRenderPayload::from_particle_set(
        "particle.render.sdf_browser_overlay",
        simulator.particles(),
    )
    .map_err(|error| FixtureError::Matter(error.to_string()))?;
    let mut particles = ParticleVisualFrame::from_matter_payload(
        "particle.visual.frame.sdf_browser_overlay",
        &render_payload,
        ColorRgba::new(0.22, 0.92, 1.0, 0.88),
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))?;
    tune_particle_visuals(&mut particles, &render_payload, scale);
    particles
        .validate()
        .map_err(|error| FixtureError::Optics(error.to_string()))?;

    let trails = initial_positions
        .into_iter()
        .zip(particles.samples.iter())
        .map(|(start, particle)| ParticleTrailVisual {
            start,
            end: particle.position,
            color: ColorRgba::new(0.20, 0.82, 1.0, 0.42),
        })
        .collect::<Vec<_>>();

    Ok(ParticleSdfBrowserOverlay {
        schema_id: PARTICLE_SDF_BROWSER_OVERLAY_SCHEMA_ID.to_owned(),
        overlay_id: "particle.sdf.browser_overlay.hand_mesh".to_owned(),
        source_surface_id: build.frame.source_surface_id.clone(),
        source_sdf_grid_id: build.sdf.grid_id.clone(),
        initial_particle_count: particle_count,
        final_particle_count: particles.samples.len(),
        simulation_steps: particle_steps,
        diagnostics,
        particles,
        trails,
    })
}

fn seed_particles_from_coordinates(
    coordinate_map: &MeshCoordinateMap,
    scale: SurfaceScale,
    particle_count: usize,
    particle_radius: f32,
) -> Result<ParticleSet, FixtureError> {
    let frames = &coordinate_map.frames.frames;
    if frames.is_empty() {
        return Err(FixtureError::Matter(
            "coordinate map must contain frames before particle seeding".to_owned(),
        ));
    }
    let mut set = ParticleSet::with_capacity("particle.set.sdf_browser_overlay", particle_count);
    for index in 0..particle_count {
        let frame = &frames[index % frames.len()];
        let tangent_x = signed_unit_hash(index, 17) * scale.longest_axis * 0.018;
        let tangent_y = signed_unit_hash(index, 53) * scale.longest_axis * 0.018;
        let normal_offset = (0.35 + unit_hash(index, 91) * 0.75) * scale.longest_axis * 0.040;
        let position = frame.anchor
            + frame.axis_x * tangent_x
            + frame.axis_y * tangent_y
            + frame.axis_z * normal_offset;
        let mut particle = ParticleState::new(
            format!("particle.sdf_browser.{index:04}"),
            position,
            particle_radius,
        );
        particle.velocity = frame.axis_z * (-(scale.longest_axis * 0.035));
        particle.flags = 1;
        set.push(particle);
    }
    set.validate()
        .map_err(|error| FixtureError::Matter(error.to_string()))?;
    Ok(set)
}

fn run_particle_steps(
    simulator: &mut ParticleSimulator,
    fixed_step: &ParticleFixedStepConfig,
    particle_steps: usize,
) -> ParticleSimulationDiagnostics {
    let mut diagnostics = ParticleSimulationDiagnostics::new("diagnostics.sdf_browser_overlay", 0);
    for _ in 0..particle_steps {
        let step = simulator.step_frame(fixed_step.fixed_step_seconds * 1.001);
        diagnostics.fixed_steps += step.fixed_steps;
        diagnostics.dropped_steps += step.dropped_steps;
        diagnostics.particle_count = step.particle_count;
        diagnostics.sampled_particles += step.sampled_particles;
        diagnostics.affected_particles += step.affected_particles;
        diagnostics.rejected_particles += step.rejected_particles;
        diagnostics.clamped_particles += step.clamped_particles;
        diagnostics.neighbor_checks += step.neighbor_checks;
        diagnostics.influence_samples += step.influence_samples;
        diagnostics.impulses_applied += step.impulses_applied;
        diagnostics.body_collisions += step.body_collisions;
        diagnostics.max_speed = diagnostics.max_speed.max(step.max_speed);
    }
    diagnostics
}

fn tune_particle_visuals(
    frame: &mut ParticleVisualFrame,
    payload: &ParticleRenderPayload,
    scale: SurfaceScale,
) {
    let speed_scale = (scale.longest_axis * 0.35).max(0.001);
    for (index, sample) in frame.samples.iter_mut().enumerate() {
        let speed = payload
            .samples
            .get(index)
            .map_or(0.0, |matter_sample| matter_sample.speed);
        let speed01 = (speed / speed_scale).clamp(0.0, 1.0);
        let pulse = to_half_open_frame01(sample.position.x.abs() * 7.0 + index as f32 * 0.037);
        sample.frame01 = pulse;
        sample.rotation_radians = pulse * core::f32::consts::TAU;
        sample.aux0 = speed;
        sample.aux1 = speed01;
        sample.color = ColorRgba::new(0.18 + 0.28 * speed01, 0.78, 1.0, 0.88);
    }
}

fn unit_hash(index: usize, salt: u32) -> f32 {
    let mut value = (index as u32).wrapping_mul(0x9E37_79B9) ^ salt;
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846C_A68B);
    value ^= value >> 16;
    (value & 0x00FF_FFFF) as f32 / 16_777_215.0
}

fn signed_unit_hash(index: usize, salt: u32) -> f32 {
    unit_hash(index, salt) * 2.0 - 1.0
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
