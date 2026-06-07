use rusty_matter_model::Vec3;
use rusty_matter_particles::{ParticleRenderPayload, ParticleSet, ParticleState};
use rusty_optics_model::ColorRgba;
use rusty_optics_particles::{
    build_morphed_ring_mask_atlas_rgba, particle_billboard_render_budget,
    project_particles_for_flat_screen, write_particle_billboard_instances,
    FlatScreenProjectionConfig, MorphedRingMaskAtlasConfig, ParticleAppearanceProfile,
    ParticleBillboardBuildConfig, ParticleBillboardInstance, ParticleBillboardRenderBudget,
    ParticleBillboardSortCamera, ParticleSceneBasis, ParticleVisualFrame,
};
use serde::Serialize;

use crate::error::FixtureError;

/// Deterministic visual particle fixture summary.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OpticsParticleFixtureSummary {
    /// Fixture schema version.
    pub schema_version: u32,
    /// Source Matter render payload identifier.
    pub matter_payload_id: String,
    /// Source particle count.
    pub source_particle_count: usize,
    /// Visual sample count.
    pub visual_sample_count: usize,
    /// Flat projected visible count.
    pub flat_visible_count: usize,
    /// Billboard emitted count.
    pub billboard_emitted_count: usize,
    /// Mask atlas width in pixels.
    pub mask_atlas_width_px: usize,
    /// Mask atlas height in pixels.
    pub mask_atlas_height_px: usize,
    /// Mask atlas frame count.
    pub mask_frame_count: usize,
    /// Nonzero mask byte count.
    pub mask_nonzero_bytes: usize,
    /// Billboard budget.
    pub budget: ParticleBillboardRenderBudget,
}

/// Builds the deterministic fixture summary.
pub fn build_summary() -> Result<OpticsParticleFixtureSummary, FixtureError> {
    let matter_payload =
        sample_payload().map_err(|error| FixtureError::Matter(error.to_string()))?;
    let mut visual_frame = ParticleVisualFrame::from_matter_payload(
        "particle.visual.frame.fixture",
        &matter_payload,
        ColorRgba::new(0.35, 0.75, 1.0, 0.8),
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))?;
    visual_frame.samples[1].frame01 = 0.5;
    visual_frame.samples[2].rotation_radians = core::f32::consts::FRAC_PI_4;
    visual_frame
        .validate()
        .map_err(|error| FixtureError::Optics(error.to_string()))?;

    let flat = project_particles_for_flat_screen(
        "particle.flat.frame.fixture",
        &visual_frame,
        &FlatScreenProjectionConfig {
            cull_offscreen: false,
            ..FlatScreenProjectionConfig::default()
        },
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))?;

    let profile = ParticleAppearanceProfile::animated_ring_billboard("particle.appearance.fixture");
    profile
        .validate()
        .map_err(|error| FixtureError::Optics(error.to_string()))?;

    let build_config = ParticleBillboardBuildConfig {
        sort_back_to_front: true,
        ..ParticleBillboardBuildConfig::default()
    };
    let mut sort_indices = Vec::new();
    let mut instances = Vec::<ParticleBillboardInstance>::new();
    let billboard_stats = write_particle_billboard_instances(
        &visual_frame.samples,
        ParticleSceneBasis::default(),
        &build_config,
        Some(ParticleBillboardSortCamera::default()),
        &mut sort_indices,
        &mut instances,
    );

    let mask = build_morphed_ring_mask_atlas_rgba(MorphedRingMaskAtlasConfig {
        frame_resolution_px: 16,
        frame_count: profile.mask.frame_count,
        atlas_columns: profile.mask.atlas_columns,
        ..MorphedRingMaskAtlasConfig::default()
    });
    let active_trails = profile
        .trail
        .max_trail_instances(visual_frame.samples.len());
    let budget = particle_billboard_render_budget(
        "particle.budget.fixture",
        visual_frame.samples.len(),
        active_trails,
        12,
    );
    budget
        .validate()
        .map_err(|error| FixtureError::Optics(error.to_string()))?;

    Ok(OpticsParticleFixtureSummary {
        schema_version: 1,
        matter_payload_id: matter_payload.payload_id,
        source_particle_count: matter_payload.samples.len(),
        visual_sample_count: visual_frame.samples.len(),
        flat_visible_count: flat.visible_particle_count,
        billboard_emitted_count: billboard_stats.emitted_count,
        mask_atlas_width_px: mask.width_px,
        mask_atlas_height_px: mask.height_px,
        mask_frame_count: mask.frame_count,
        mask_nonzero_bytes: mask.rgba.iter().filter(|value| **value > 0).count(),
        budget,
    })
}

/// Serializes the fixture summary as stable pretty JSON.
pub fn summary_json() -> Result<String, FixtureError> {
    let mut json = serde_json::to_string_pretty(&build_summary()?)?;
    json.push('\n');
    Ok(json)
}

fn sample_payload() -> Result<ParticleRenderPayload, rusty_matter_particles::ParticleError> {
    let mut set = ParticleSet::new("particle.set.optics_fixture");
    set.time_seconds = 1.0 / 60.0;
    set.push(ParticleState::new(
        "particle.fixture.0",
        Vec3::new(-0.25, 0.0, 1.0),
        0.04,
    ));
    set.push(ParticleState::new(
        "particle.fixture.1",
        Vec3::new(0.25, 0.1, -1.0),
        0.06,
    ));
    set.push(ParticleState::new(
        "particle.fixture.2",
        Vec3::new(0.0, -0.2, -2.0),
        0.08,
    ));
    ParticleRenderPayload::from_particle_set("particle.render.optics_fixture", &set)
}
