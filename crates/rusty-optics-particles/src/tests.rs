use rusty_matter_model::Vec3;
use rusty_matter_particles::{ParticleRenderPayload, ParticleSet, ParticleState};
use rusty_optics_model::{ColorRgba, PARTICLE_VISUAL_FRAME_SCHEMA_ID};

use crate::{
    build_morphed_ring_mask_atlas_rgba, morphed_ring_alpha, particle_billboard_render_budget,
    project_particles_for_flat_screen, resolve_animated_particle_visual_frame,
    to_half_open_frame01, FlatScreenProjectionConfig, MorphedRingMaskAtlasConfig,
    ParticleAppearanceProfile, ParticleBillboardBuildConfig, ParticleBillboardInstance,
    ParticleBillboardSortCamera, ParticleScalarEnvelope, ParticleSceneBasis,
    ParticleVisualAnimationProfile, ParticleVisualFrame,
};

#[test]
fn visual_frame_from_matter_payload_preserves_particle_truth() {
    let payload = sample_payload();
    let frame = ParticleVisualFrame::from_matter_payload(
        "particle.visual.frame.test",
        &payload,
        ColorRgba::new(0.1, 0.2, 0.3, 0.4),
    )
    .expect("visual frame");

    assert_eq!(frame.schema_id, PARTICLE_VISUAL_FRAME_SCHEMA_ID);
    assert_eq!(frame.source_payload_id, payload.payload_id);
    assert_eq!(frame.samples.len(), 3);
    assert_eq!(frame.samples[0].source_particle_id, "particle.near");
    assert_eq!(frame.samples[0].position, Vec3::new(0.0, 0.0, 1.0));
    assert_eq!(frame.samples[0].radius, 0.05);
    assert_eq!(frame.samples[0].color, ColorRgba::new(0.1, 0.2, 0.3, 0.4));
}

#[test]
fn half_open_frame_clamp_keeps_one_out_of_payloads() {
    assert_eq!(to_half_open_frame01(-1.0), 0.0);
    assert!(to_half_open_frame01(1.0) < 1.0);
    assert_eq!(to_half_open_frame01(0.5), 0.5);
}

#[test]
fn flat_projection_sorts_far_to_near_and_preserves_animation_fields() {
    let payload = sample_payload();
    let mut frame = ParticleVisualFrame::from_matter_payload(
        "particle.visual.frame.project",
        &payload,
        ColorRgba::WHITE,
    )
    .expect("visual frame");
    frame.samples[0].frame01 = 0.25;
    frame.samples[1].frame01 = 0.75;

    let flat = project_particles_for_flat_screen(
        "particle.flat.frame.project",
        &frame,
        &FlatScreenProjectionConfig {
            cull_offscreen: false,
            ..FlatScreenProjectionConfig::default()
        },
    )
    .expect("flat projection");

    assert_eq!(flat.visible_particle_count, 3);
    assert_eq!(flat.particles[0].source_particle_id, "particle.far");
    assert!(flat.particles[0].camera_depth > flat.particles[2].camera_depth);
    assert_eq!(flat.particles[2].frame01, 0.25);
}

#[test]
fn billboard_instances_filter_sort_and_pack_visual_fields() {
    let payload = sample_payload();
    let mut frame = ParticleVisualFrame::from_matter_payload(
        "particle.visual.frame.billboard",
        &payload,
        ColorRgba::new(1.0, 0.5, 0.25, 0.75),
    )
    .expect("visual frame");
    frame.samples[2].color.a = 0.0;
    frame.samples[0].frame01 = 0.5;

    let config = ParticleBillboardBuildConfig {
        sort_back_to_front: true,
        ..ParticleBillboardBuildConfig::default()
    };
    let mut sort_indices = Vec::new();
    let mut instances = Vec::<ParticleBillboardInstance>::new();
    let stats = crate::write_particle_billboard_instances(
        &frame.samples,
        ParticleSceneBasis::default(),
        &config,
        Some(ParticleBillboardSortCamera::default()),
        &mut sort_indices,
        &mut instances,
    );

    assert_eq!(stats.source_count, 3);
    assert_eq!(stats.emitted_count, 2);
    assert_eq!(stats.skipped_count, 1);
    assert_eq!(instances[0].normal_frame[3], 0.5);
    assert_eq!(instances[0].color[3], 0.75);
}

#[test]
fn billboard_budget_accounts_for_trails() {
    let budget = particle_billboard_render_budget("particle.budget.test", 256, 512, 12);

    budget.validate().expect("valid budget");
    assert_eq!(budget.visible_instances, 768);
    assert_eq!(budget.indices_per_instance, 36);
    assert_eq!(budget.total_indices, 27_648);
}

#[test]
fn morphed_ring_mask_atlas_has_expected_layout() {
    let config = MorphedRingMaskAtlasConfig {
        frame_resolution_px: 16,
        frame_count: 8,
        atlas_columns: 4,
        ..MorphedRingMaskAtlasConfig::default()
    };
    let atlas = build_morphed_ring_mask_atlas_rgba(config);

    assert_eq!(atlas.width_px, 64);
    assert_eq!(atlas.height_px, 32);
    assert_eq!(atlas.rgba.len(), atlas.width_px * atlas.height_px * 4);
    assert!(atlas.rgba.iter().any(|value| *value > 0));
    assert_eq!(morphed_ring_alpha([0.5, 0.5], 0.0, config), 0.0);
}

#[test]
fn appearance_profile_validates_animation_descriptor_and_trails() {
    let mut profile =
        ParticleAppearanceProfile::animated_ring_billboard("particle.appearance.animated_ring");
    profile.trail.enabled = true;
    profile.trail.copies_per_particle = 7;

    profile.validate().expect("profile validates");
    assert_eq!(profile.trail.max_trail_instances(256), 1792);
}

#[test]
fn animation_profile_resolves_size_alpha_color_and_phase() {
    let payload = sample_payload();
    let profile = ParticleVisualAnimationProfile::transparent_ring("particle.animation.test");

    let frame = resolve_animated_particle_visual_frame(
        "particle.visual.frame.animated",
        &payload,
        &profile,
    )
    .expect("animated frame");

    assert_eq!(frame.source_payload_id, payload.payload_id);
    assert_eq!(frame.samples.len(), 3);
    assert!(frame
        .samples
        .iter()
        .zip(payload.samples.iter())
        .any(|(visual, source)| (visual.radius - source.radius).abs() > 1.0e-6));
    assert!(frame.samples[1].color.a <= profile.max_alpha);
    assert_ne!(frame.samples[0].color, frame.samples[1].color);
    assert!((0.0..1.0).contains(&frame.samples[2].frame01));
    assert!(frame.samples[0].rotation_radians.is_finite());
    assert!(frame.samples[1].aux0 > frame.samples[0].aux0);
    assert!(frame.samples[2].aux1 >= 0.0 && frame.samples[2].aux1 <= 1.0);
}

#[test]
fn animation_profile_validation_rejects_bad_ranges() {
    let bad_profile = ParticleVisualAnimationProfile {
        alpha: ParticleScalarEnvelope {
            minimum: 0.8,
            maximum: 0.2,
            ..ParticleScalarEnvelope::new(0.1, 0.9, 1.0, 0.0, true)
        },
        ..ParticleVisualAnimationProfile::transparent_ring("particle.animation.bad")
    };

    assert!(bad_profile.validate().is_err());
}

#[test]
fn animated_visual_frame_projects_and_packs_transparency_fields() {
    let payload = sample_payload();
    let profile = ParticleVisualAnimationProfile::transparent_ring("particle.animation.pack");
    let frame =
        resolve_animated_particle_visual_frame("particle.visual.frame.pack", &payload, &profile)
            .expect("animated frame");
    let flat = project_particles_for_flat_screen(
        "particle.flat.frame.pack",
        &frame,
        &FlatScreenProjectionConfig {
            cull_offscreen: false,
            ..FlatScreenProjectionConfig::default()
        },
    )
    .expect("flat projection");

    let mut sort_indices = Vec::new();
    let mut instances = Vec::<ParticleBillboardInstance>::new();
    let stats = crate::write_particle_billboard_instances(
        &frame.samples,
        ParticleSceneBasis::default(),
        &ParticleBillboardBuildConfig {
            sort_back_to_front: true,
            ..ParticleBillboardBuildConfig::default()
        },
        Some(ParticleBillboardSortCamera::default()),
        &mut sort_indices,
        &mut instances,
    );

    assert_eq!(flat.visible_particle_count, 3);
    assert_eq!(stats.emitted_count, 3);
    assert_eq!(instances[0].normal_frame[3], frame.samples[0].frame01);
    assert_eq!(instances[0].color[3], frame.samples[0].color.a);
}

fn sample_payload() -> ParticleRenderPayload {
    let mut set = ParticleSet::new("particle.set.optics_test");
    set.time_seconds = 0.25;
    set.push(ParticleState::new(
        "particle.near",
        Vec3::new(0.0, 0.0, 1.0),
        0.05,
    ));
    set.particles[0].velocity = Vec3::new(0.1, 0.0, 0.0);
    set.particles[0].age_seconds = 0.1;
    set.push(ParticleState::new(
        "particle.far",
        Vec3::new(0.0, 0.0, -2.0),
        0.08,
    ));
    set.particles[1].velocity = Vec3::new(0.0, 0.6, 0.0);
    set.particles[1].age_seconds = 0.5;
    set.push(ParticleState::new(
        "particle.hidden",
        Vec3::new(0.25, 0.0, 0.0),
        0.02,
    ));
    set.particles[2].velocity = Vec3::new(0.0, 0.0, 0.2);
    set.particles[2].age_seconds = 0.8;
    ParticleRenderPayload::from_particle_set("particle.render.optics_test", &set)
        .expect("matter payload")
}
