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
/// Particle visual animation profile schema.
pub const PARTICLE_VISUAL_ANIMATION_PROFILE_SCHEMA_ID: &str =
    "rusty.optics.particles.animation.profile.v1";
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
/// Mesh debug frame schema.
pub const MESH_DEBUG_FRAME_SCHEMA_ID: &str = "rusty.optics.mesh.debug.frame.v1";
/// Mesh coordinate visualization schema.
pub const MESH_COORDINATE_VISUAL_SCHEMA_ID: &str = "rusty.optics.mesh.coordinate.visual.v1";
/// Dynamic mesh collider visualization schema.
pub const MESH_COLLIDER_VISUAL_SCHEMA_ID: &str = "rusty.optics.mesh.collider.visual.v1";
/// SDF slice visualization schema.
pub const SDF_SLICE_VISUAL_SCHEMA_ID: &str = "rusty.optics.sdf.slice.visual.v1";
/// ADF debug visualization schema.
pub const ADF_DEBUG_VISUAL_SCHEMA_ID: &str = "rusty.optics.adf.debug.visual.v1";
/// Browser-shaped mesh debug frame schema.
pub const MESH_BROWSER_DEBUG_FRAME_SCHEMA_ID: &str = "rusty.optics.mesh.browser.debug_frame.v1";
/// Browser SDF particle overlay schema.
pub const PARTICLE_SDF_BROWSER_OVERLAY_SCHEMA_ID: &str =
    "rusty.optics.particles.sdf.browser_overlay.v1";
/// Surface-field visual frame schema.
pub const SURFACE_FIELD_VISUAL_FRAME_SCHEMA_ID: &str =
    "rusty.optics.fields.surface.visual_frame.v1";
/// Surface-field visual sequence schema.
pub const SURFACE_FIELD_VISUAL_SEQUENCE_SCHEMA_ID: &str =
    "rusty.optics.fields.surface.visual_sequence.v1";
/// Bioelectric circuit visual frame schema.
pub const BIOELECTRIC_CIRCUIT_VISUAL_FRAME_SCHEMA_ID: &str =
    "rusty.optics.fields.bioelectric_circuit.visual_frame.v1";
/// Planarian bioelectric visual sequence schema.
pub const PLANARIAN_BIOELECTRIC_VISUAL_SEQUENCE_SCHEMA_ID: &str =
    "rusty.optics.fields.planarian_bioelectric.visual_sequence.v1";
/// Planarian bioelectric 3D pick selection schema.
pub const PLANARIAN_BIOELECTRIC_PICK_SELECTION_SCHEMA_ID: &str =
    "rusty.optics.fields.planarian_bioelectric.pick_selection.v1";
/// Planarian bioelectric edit intent schema.
pub const PLANARIAN_BIOELECTRIC_EDIT_INTENT_SCHEMA_ID: &str =
    "rusty.optics.fields.planarian_bioelectric.edit_intent.v1";
/// Planarian bioelectric edit feedback visual frame schema.
pub const PLANARIAN_BIOELECTRIC_EDIT_FEEDBACK_FRAME_SCHEMA_ID: &str =
    "rusty.optics.fields.planarian_bioelectric.edit_feedback_frame.v1";
/// Target screen footprint schema.
pub const TARGET_SCREEN_FOOTPRINT_SCHEMA_ID: &str = "rusty.optics.target_screen_footprint.v1";
/// Source sampling mode schema.
pub const SOURCE_SAMPLING_MODE_SCHEMA_ID: &str = "rusty.optics.source_sampling_mode.v1";
/// Video projection geometry schema.
pub const VIDEO_PROJECTION_GEOMETRY_SCHEMA_ID: &str = "rusty.optics.video_projection_geometry.v1";
/// Procedural stimulus profile schema.
pub const STIMULUS_PROFILE_SCHEMA_ID: &str = "rusty.optics.stimulus.profile.v1";
/// Procedural stimulus layer graph schema.
pub const STIMULUS_LAYER_GRAPH_SCHEMA_ID: &str = "rusty.optics.stimulus.layer_graph.v1";
/// Procedural stimulus temporal profile schema.
pub const STIMULUS_TEMPORAL_PROFILE_SCHEMA_ID: &str = "rusty.optics.stimulus.temporal_profile.v1";
/// Procedural stimulus safety profile schema.
pub const STIMULUS_SAFETY_PROFILE_SCHEMA_ID: &str = "rusty.optics.stimulus.safety_profile.v1";
/// Procedural stimulus presentation target schema.
pub const STIMULUS_PRESENTATION_SCHEMA_ID: &str = "rusty.optics.stimulus.presentation.v1";
/// Procedural stimulus kernel ABI schema.
pub const STIMULUS_KERNEL_ABI_SCHEMA_ID: &str = "rusty.optics.stimulus.kernel_abi.v1";
/// Procedural stimulus volume descriptor schema.
pub const STIMULUS_VOLUME_SCHEMA_ID: &str = "rusty.optics.stimulus.volume.v1";
/// Procedural stimulus run plan schema.
pub const STIMULUS_RUN_PLAN_SCHEMA_ID: &str = "rusty.optics.stimulus.run_plan.v1";

/// Returns the schema IDs currently emitted by Rusty Optics.
#[must_use]
pub const fn optics_schema_ids() -> [&'static str; 35] {
    [
        COLOR_RGBA_SCHEMA_ID,
        PARTICLE_VISUAL_SAMPLE_SCHEMA_ID,
        PARTICLE_VISUAL_FRAME_SCHEMA_ID,
        PARTICLE_APPEARANCE_PROFILE_SCHEMA_ID,
        PARTICLE_VISUAL_ANIMATION_PROFILE_SCHEMA_ID,
        PARTICLE_ANIMATED_MASK_SCHEMA_ID,
        PARTICLE_BILLBOARD_BUILD_SCHEMA_ID,
        PARTICLE_RENDER_BUDGET_SCHEMA_ID,
        PARTICLE_FLAT_PROJECTION_SCHEMA_ID,
        PARTICLE_FLAT_FRAME_SCHEMA_ID,
        MESH_DEBUG_FRAME_SCHEMA_ID,
        MESH_COORDINATE_VISUAL_SCHEMA_ID,
        MESH_COLLIDER_VISUAL_SCHEMA_ID,
        SDF_SLICE_VISUAL_SCHEMA_ID,
        ADF_DEBUG_VISUAL_SCHEMA_ID,
        MESH_BROWSER_DEBUG_FRAME_SCHEMA_ID,
        PARTICLE_SDF_BROWSER_OVERLAY_SCHEMA_ID,
        SURFACE_FIELD_VISUAL_FRAME_SCHEMA_ID,
        SURFACE_FIELD_VISUAL_SEQUENCE_SCHEMA_ID,
        BIOELECTRIC_CIRCUIT_VISUAL_FRAME_SCHEMA_ID,
        PLANARIAN_BIOELECTRIC_VISUAL_SEQUENCE_SCHEMA_ID,
        PLANARIAN_BIOELECTRIC_PICK_SELECTION_SCHEMA_ID,
        PLANARIAN_BIOELECTRIC_EDIT_INTENT_SCHEMA_ID,
        PLANARIAN_BIOELECTRIC_EDIT_FEEDBACK_FRAME_SCHEMA_ID,
        TARGET_SCREEN_FOOTPRINT_SCHEMA_ID,
        SOURCE_SAMPLING_MODE_SCHEMA_ID,
        VIDEO_PROJECTION_GEOMETRY_SCHEMA_ID,
        STIMULUS_PROFILE_SCHEMA_ID,
        STIMULUS_LAYER_GRAPH_SCHEMA_ID,
        STIMULUS_TEMPORAL_PROFILE_SCHEMA_ID,
        STIMULUS_SAFETY_PROFILE_SCHEMA_ID,
        STIMULUS_PRESENTATION_SCHEMA_ID,
        STIMULUS_KERNEL_ABI_SCHEMA_ID,
        STIMULUS_VOLUME_SCHEMA_ID,
        STIMULUS_RUN_PLAN_SCHEMA_ID,
    ]
}
