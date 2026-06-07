//! Stable Optics schema identifiers.

/// RGBA color schema.
pub const COLOR_RGBA_SCHEMA_ID: &str = "rusty.optics.color.rgba.v1";
/// Visual particle sample schema.
pub const PARTICLE_VISUAL_SAMPLE_SCHEMA_ID: &str = "rusty.optics.particles.visual.sample.v1";
/// Visual particle frame schema.
pub const PARTICLE_VISUAL_FRAME_SCHEMA_ID: &str = "rusty.optics.particles.visual.frame.v1";
/// Particle appearance profile schema.
pub const PARTICLE_APPEARANCE_PROFILE_SCHEMA_ID: &str =
    "rusty.optics.particles.appearance.profile.v1";
/// Particle animated mask descriptor schema.
pub const PARTICLE_ANIMATED_MASK_SCHEMA_ID: &str = "rusty.optics.particles.mask.animated.v1";
/// Particle billboard build profile schema.
pub const PARTICLE_BILLBOARD_BUILD_SCHEMA_ID: &str = "rusty.optics.particles.billboard.build.v1";
/// Particle billboard render-budget schema.
pub const PARTICLE_RENDER_BUDGET_SCHEMA_ID: &str = "rusty.optics.particles.budget.billboard.v1";
/// Flat particle projection config schema.
pub const PARTICLE_FLAT_PROJECTION_SCHEMA_ID: &str = "rusty.optics.particles.projection.flat.v1";
/// Flat particle frame schema.
pub const PARTICLE_FLAT_FRAME_SCHEMA_ID: &str = "rusty.optics.particles.flat.frame.v1";

/// Returns the schema IDs currently emitted by Rusty Optics.
#[must_use]
pub const fn optics_schema_ids() -> [&'static str; 9] {
    [
        COLOR_RGBA_SCHEMA_ID,
        PARTICLE_VISUAL_SAMPLE_SCHEMA_ID,
        PARTICLE_VISUAL_FRAME_SCHEMA_ID,
        PARTICLE_APPEARANCE_PROFILE_SCHEMA_ID,
        PARTICLE_ANIMATED_MASK_SCHEMA_ID,
        PARTICLE_BILLBOARD_BUILD_SCHEMA_ID,
        PARTICLE_RENDER_BUDGET_SCHEMA_ID,
        PARTICLE_FLAT_PROJECTION_SCHEMA_ID,
        PARTICLE_FLAT_FRAME_SCHEMA_ID,
    ]
}
