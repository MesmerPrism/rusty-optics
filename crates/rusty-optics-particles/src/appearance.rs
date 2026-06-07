use rusty_optics_model::{
    OpticsError, PARTICLE_ANIMATED_MASK_SCHEMA_ID, PARTICLE_APPEARANCE_PROFILE_SCHEMA_ID,
};

/// How particle quads are expected to be expanded by a renderer adapter.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleDrawMode {
    /// Project the center and expand the quad in screen or clip space.
    CenterProjectedBillboard,
    /// Expand the quad as world-space vertices before projection.
    WorldExpandedBillboard,
}

impl Default for ParticleDrawMode {
    fn default() -> Self {
        Self::CenterProjectedBillboard
    }
}

/// Blend policy requested by a visual particle profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleBlendMode {
    /// Source alpha over destination.
    Alpha,
    /// Premultiplied alpha over destination.
    PremultipliedAlpha,
    /// Additive color contribution.
    Additive,
}

impl Default for ParticleBlendMode {
    fn default() -> Self {
        Self::Alpha
    }
}

/// Depth policy requested by a visual particle profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleDepthMode {
    /// Sort transparent particles back-to-front.
    SortedTransparent,
    /// Test depth but do not write depth.
    TestNoWrite,
    /// Disable depth testing.
    Disabled,
}

impl Default for ParticleDepthMode {
    fn default() -> Self {
        Self::SortedTransparent
    }
}

/// Animated particle mask family.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleMaskAnimationKind {
    /// No animated mask.
    None,
    /// A CPU-generated morphed-ring atlas sampled by frame phase.
    MorphedRingAtlas,
}

impl Default for ParticleMaskAnimationKind {
    fn default() -> Self {
        Self::None
    }
}

/// Renderer-neutral animated particle mask descriptor.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleAnimatedMaskDescriptor {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable descriptor identifier.
    pub descriptor_id: String,
    /// Mask family.
    pub kind: ParticleMaskAnimationKind,
    /// Number of animation frames.
    pub frame_count: usize,
    /// Atlas column count when the mask is stored as a 2D atlas.
    pub atlas_columns: usize,
    /// One frame cell width and height in pixels.
    pub frame_resolution_px: usize,
    /// Whether adapters should blend neighboring frames.
    pub blend_frames: bool,
    /// Whether frame phase is interpreted as a half-open `0..1` value.
    pub half_open_phase: bool,
}

impl ParticleAnimatedMaskDescriptor {
    /// Creates a disabled mask descriptor.
    #[must_use]
    pub fn none(descriptor_id: impl Into<String>) -> Self {
        Self {
            schema_id: PARTICLE_ANIMATED_MASK_SCHEMA_ID.to_owned(),
            descriptor_id: descriptor_id.into(),
            kind: ParticleMaskAnimationKind::None,
            frame_count: 1,
            atlas_columns: 1,
            frame_resolution_px: 1,
            blend_frames: false,
            half_open_phase: true,
        }
    }

    /// Creates a generic morphed-ring atlas descriptor.
    #[must_use]
    pub fn morphed_ring_atlas(descriptor_id: impl Into<String>) -> Self {
        Self {
            schema_id: PARTICLE_ANIMATED_MASK_SCHEMA_ID.to_owned(),
            descriptor_id: descriptor_id.into(),
            kind: ParticleMaskAnimationKind::MorphedRingAtlas,
            frame_count: 64,
            atlas_columns: 8,
            frame_resolution_px: 64,
            blend_frames: true,
            half_open_phase: true,
        }
    }

    /// Validates descriptor shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PARTICLE_ANIMATED_MASK_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PARTICLE_ANIMATED_MASK_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.descriptor_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("descriptor_id"));
        }
        if self.frame_count == 0 {
            return Err(OpticsError::InvalidCount("frame_count"));
        }
        if self.atlas_columns == 0 {
            return Err(OpticsError::InvalidCount("atlas_columns"));
        }
        if self.frame_resolution_px == 0 {
            return Err(OpticsError::InvalidCount("frame_resolution_px"));
        }
        Ok(())
    }
}

impl Default for ParticleAnimatedMaskDescriptor {
    fn default() -> Self {
        Self::none("particle.mask.none")
    }
}

/// Renderer-neutral trail appearance descriptor.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleTrailAppearance {
    /// Enables trail instances.
    pub enabled: bool,
    /// Trail copies per source particle.
    pub copies_per_particle: usize,
    /// Requested snapshot copies per second.
    pub copies_per_second: f32,
    /// Trail lifetime in seconds.
    pub lifetime_seconds: f32,
    /// Size multiplier applied to frozen trail snapshots.
    pub size_multiplier: f32,
    /// Whether trail particles preserve source visual state at spawn time.
    pub frozen_snapshots: bool,
}

impl Default for ParticleTrailAppearance {
    fn default() -> Self {
        Self {
            enabled: false,
            copies_per_particle: 1,
            copies_per_second: 0.0,
            lifetime_seconds: 0.25,
            size_multiplier: 1.0,
            frozen_snapshots: true,
        }
    }
}

impl ParticleTrailAppearance {
    /// Returns the maximum live trail instances for a source particle count.
    #[must_use]
    pub fn max_trail_instances(self, source_particles: usize) -> usize {
        if self.enabled {
            source_particles.saturating_mul(self.copies_per_particle.max(1))
        } else {
            0
        }
    }

    /// Validates trail descriptor shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(self) -> Result<(), OpticsError> {
        if self.copies_per_particle == 0 {
            return Err(OpticsError::InvalidCount("copies_per_particle"));
        }
        if !self.copies_per_second.is_finite() || self.copies_per_second < 0.0 {
            return Err(OpticsError::InvalidValue("copies_per_second"));
        }
        if !self.lifetime_seconds.is_finite() || self.lifetime_seconds < 0.0 {
            return Err(OpticsError::InvalidValue("lifetime_seconds"));
        }
        if !self.size_multiplier.is_finite() || self.size_multiplier < 0.0 {
            return Err(OpticsError::InvalidValue("size_multiplier"));
        }
        Ok(())
    }
}

/// Complete renderer-neutral particle appearance profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleAppearanceProfile {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable profile identifier.
    pub profile_id: String,
    /// Billboard expansion mode.
    pub draw_mode: ParticleDrawMode,
    /// Blend policy.
    pub blend_mode: ParticleBlendMode,
    /// Depth policy.
    pub depth_mode: ParticleDepthMode,
    /// Animated mask descriptor.
    pub mask: ParticleAnimatedMaskDescriptor,
    /// Trail appearance descriptor.
    pub trail: ParticleTrailAppearance,
    /// Minimum alpha emitted to a renderer adapter.
    pub min_alpha: f32,
    /// Minimum visual radius emitted to a renderer adapter.
    pub min_radius: f32,
    /// Base multiplier for animation-frame-dependent particle scale.
    pub frame_scale_base: f32,
    /// Added scale when frame phase reaches one.
    pub frame_scale_delta: f32,
    /// Scale applied when a normal faces away from the camera.
    pub facing_scale_min: f32,
    /// Scale applied when a normal faces the camera.
    pub facing_scale_max: f32,
    /// Global opacity multiplier.
    pub opacity_multiplier: f32,
    /// Exponential depth attenuation strength.
    pub depth_attenuation: f32,
}

impl ParticleAppearanceProfile {
    /// Creates a conservative default profile.
    #[must_use]
    pub fn new(profile_id: impl Into<String>) -> Self {
        Self {
            schema_id: PARTICLE_APPEARANCE_PROFILE_SCHEMA_ID.to_owned(),
            profile_id: profile_id.into(),
            draw_mode: ParticleDrawMode::CenterProjectedBillboard,
            blend_mode: ParticleBlendMode::Alpha,
            depth_mode: ParticleDepthMode::SortedTransparent,
            mask: ParticleAnimatedMaskDescriptor::none("particle.mask.none"),
            trail: ParticleTrailAppearance::default(),
            min_alpha: 1.0e-6,
            min_radius: 1.0e-6,
            frame_scale_base: 1.0,
            frame_scale_delta: 0.0,
            facing_scale_min: 1.0,
            facing_scale_max: 1.0,
            opacity_multiplier: 1.0,
            depth_attenuation: 0.0,
        }
    }

    /// Creates a generic animated-ring billboard profile.
    #[must_use]
    pub fn animated_ring_billboard(profile_id: impl Into<String>) -> Self {
        Self {
            mask: ParticleAnimatedMaskDescriptor::morphed_ring_atlas("particle.mask.morphed_ring"),
            frame_scale_delta: 0.125,
            facing_scale_min: 0.9,
            facing_scale_max: 1.05,
            opacity_multiplier: 0.5,
            depth_attenuation: 1.0,
            ..Self::new(profile_id)
        }
    }

    /// Validates appearance profile shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PARTICLE_APPEARANCE_PROFILE_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PARTICLE_APPEARANCE_PROFILE_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.profile_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("profile_id"));
        }
        self.mask.validate()?;
        self.trail.validate()?;
        validate_non_negative("min_alpha", self.min_alpha)?;
        validate_non_negative("min_radius", self.min_radius)?;
        validate_non_negative("frame_scale_base", self.frame_scale_base)?;
        validate_non_negative("frame_scale_delta", self.frame_scale_delta)?;
        validate_non_negative("facing_scale_min", self.facing_scale_min)?;
        validate_non_negative("facing_scale_max", self.facing_scale_max)?;
        validate_non_negative("opacity_multiplier", self.opacity_multiplier)?;
        validate_non_negative("depth_attenuation", self.depth_attenuation)?;
        if self.facing_scale_min > self.facing_scale_max {
            return Err(OpticsError::InvalidValue("facing scale range"));
        }
        Ok(())
    }
}

impl Default for ParticleAppearanceProfile {
    fn default() -> Self {
        Self::new("particle.appearance.default")
    }
}

fn validate_non_negative(field: &'static str, value: f32) -> Result<(), OpticsError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(OpticsError::InvalidValue(field))
    }
}
