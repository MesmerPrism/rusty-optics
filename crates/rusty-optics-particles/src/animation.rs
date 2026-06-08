use rusty_matter_model::Vec3;
use rusty_matter_particles::ParticleRenderPayload;
use rusty_optics_model::{
    ColorRgba, OpticsError, PARTICLE_VISUAL_ANIMATION_PROFILE_SCHEMA_ID,
    PARTICLE_VISUAL_FRAME_SCHEMA_ID,
};

use crate::{
    to_half_open_frame01, ParticleBlendMode, ParticleDepthMode, ParticleVisualFrame,
    ParticleVisualSample,
};

/// Color ramp used by renderer-neutral particle visual animation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleColorRamp {
    /// Color at phase 0.
    pub low: ColorRgba,
    /// Color at phase 0.5.
    pub mid: ColorRgba,
    /// Color at phase 1.
    pub high: ColorRgba,
}

impl ParticleColorRamp {
    /// Creates a color ramp.
    #[must_use]
    pub const fn new(low: ColorRgba, mid: ColorRgba, high: ColorRgba) -> Self {
        Self { low, mid, high }
    }

    /// Samples the ramp at `phase01`.
    #[must_use]
    pub fn sample(self, phase01: f32) -> ColorRgba {
        let phase01 = clamp01(phase01);
        if phase01 <= 0.5 {
            lerp_color(self.low, self.mid, phase01 * 2.0)
        } else {
            lerp_color(self.mid, self.high, (phase01 - 0.5) * 2.0)
        }
        .clamped01()
    }

    /// Validates finite color channels.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when any ramp color is non-finite.
    pub fn validate(self) -> Result<(), OpticsError> {
        if !self.low.is_finite() || !self.mid.is_finite() || !self.high.is_finite() {
            return Err(OpticsError::NonFiniteColor("color_ramp"));
        }
        Ok(())
    }
}

impl Default for ParticleColorRamp {
    fn default() -> Self {
        Self {
            low: ColorRgba::new(0.10, 0.72, 1.00, 1.0),
            mid: ColorRgba::new(0.36, 0.94, 0.98, 1.0),
            high: ColorRgba::new(0.76, 1.00, 0.82, 1.0),
        }
    }
}

/// Scalar envelope for visual size, alpha, or animation response.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleScalarEnvelope {
    /// Minimum emitted value.
    pub minimum: f32,
    /// Maximum emitted value.
    pub maximum: f32,
    /// Multiplier applied to the source phase.
    pub cycle_multiplier: f32,
    /// Phase offset before wrapping.
    pub phase_offset: f32,
    /// Whether to use a raised hump instead of a linear phase.
    pub hump_shaped: bool,
}

impl ParticleScalarEnvelope {
    /// Creates an envelope.
    #[must_use]
    pub const fn new(
        minimum: f32,
        maximum: f32,
        cycle_multiplier: f32,
        phase_offset: f32,
        hump_shaped: bool,
    ) -> Self {
        Self {
            minimum,
            maximum,
            cycle_multiplier,
            phase_offset,
            hump_shaped,
        }
    }

    /// Samples the envelope and returns `(value, resolved_phase)`.
    #[must_use]
    pub fn sample(self, source_phase01: f32) -> (f32, f32) {
        let phase01 =
            cycle_phase01(source_phase01.mul_add(self.cycle_multiplier, self.phase_offset));
        let t = if self.hump_shaped {
            hump_envelope(phase01)
        } else {
            phase01
        };
        (lerp(self.minimum, self.maximum, t), t)
    }

    /// Validates finite, ordered envelope fields.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the envelope has non-finite or inverted
    /// bounds.
    pub fn validate(self, field: &'static str) -> Result<(), OpticsError> {
        if !self.minimum.is_finite()
            || !self.maximum.is_finite()
            || self.minimum < 0.0
            || self.maximum < self.minimum
            || !self.cycle_multiplier.is_finite()
            || self.cycle_multiplier < 0.0
            || !self.phase_offset.is_finite()
        {
            return Err(OpticsError::InvalidValue(field));
        }
        Ok(())
    }
}

/// Renderer-neutral animated transparent particle profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleVisualAnimationProfile {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable profile identifier.
    pub profile_id: String,
    /// Color ramp applied by resolved animation phase.
    pub color_ramp: ParticleColorRamp,
    /// Radius multiplier envelope.
    pub size: ParticleScalarEnvelope,
    /// Alpha envelope.
    pub alpha: ParticleScalarEnvelope,
    /// Source age cycles per second.
    pub animation_cycles_per_second: f32,
    /// Per-particle phase stride by source index.
    pub index_phase_stride: f32,
    /// Speed value mapped to `1.0` in `aux0`.
    pub speed_reference: f32,
    /// Rotation speed around the facing axis.
    pub spin_radians_per_second: f32,
    /// Global opacity multiplier.
    pub opacity_multiplier: f32,
    /// Maximum emitted alpha.
    pub max_alpha: f32,
    /// Depth attenuation hint for flat/browser previews and adapters.
    pub depth_attenuation: f32,
    /// Facing tint minimum.
    pub facing_tint_min: f32,
    /// Facing tint maximum.
    pub facing_tint_max: f32,
    /// Facing scale minimum.
    pub facing_scale_min: f32,
    /// Facing scale maximum.
    pub facing_scale_max: f32,
    /// Blend policy intended for transparent particles.
    pub blend_mode: ParticleBlendMode,
    /// Depth policy intended for transparent particles.
    pub depth_mode: ParticleDepthMode,
}

impl ParticleVisualAnimationProfile {
    /// Creates a conservative transparent particle profile.
    #[must_use]
    pub fn new(profile_id: impl Into<String>) -> Self {
        Self {
            schema_id: PARTICLE_VISUAL_ANIMATION_PROFILE_SCHEMA_ID.to_owned(),
            profile_id: profile_id.into(),
            color_ramp: ParticleColorRamp::default(),
            size: ParticleScalarEnvelope::new(0.92, 1.12, 1.0, 0.0, true),
            alpha: ParticleScalarEnvelope::new(0.22, 0.58, 1.0, 0.0, true),
            animation_cycles_per_second: 0.85,
            index_phase_stride: 0.013,
            speed_reference: 1.0,
            spin_radians_per_second: core::f32::consts::TAU * 0.18,
            opacity_multiplier: 0.72,
            max_alpha: 0.42,
            depth_attenuation: 1.5,
            facing_tint_min: 0.80,
            facing_tint_max: 1.0,
            facing_scale_min: 0.86,
            facing_scale_max: 1.08,
            blend_mode: ParticleBlendMode::Alpha,
            depth_mode: ParticleDepthMode::SortedTransparent,
        }
    }

    /// Creates a transparent ring profile for advanced particle previews.
    ///
    /// This is renderer-neutral vocabulary only: no oscillator coupling, live
    /// driver bindings, private scene tuning, shader code, or backend texture
    /// implementation is included.
    #[must_use]
    pub fn transparent_ring(profile_id: impl Into<String>) -> Self {
        Self {
            blend_mode: ParticleBlendMode::PremultipliedAlpha,
            depth_mode: ParticleDepthMode::SortedTransparent,
            ..Self::new(profile_id)
        }
    }

    /// Validates profile shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the profile has invalid identifiers,
    /// envelope fields, colors, or transparency hints.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PARTICLE_VISUAL_ANIMATION_PROFILE_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PARTICLE_VISUAL_ANIMATION_PROFILE_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.profile_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("profile_id"));
        }
        self.color_ramp.validate()?;
        self.size.validate("size")?;
        self.alpha.validate("alpha")?;
        validate_non_negative(
            "animation_cycles_per_second",
            self.animation_cycles_per_second,
        )?;
        validate_non_negative("index_phase_stride", self.index_phase_stride)?;
        validate_non_negative("speed_reference", self.speed_reference)?;
        validate_non_negative("opacity_multiplier", self.opacity_multiplier)?;
        validate_non_negative("max_alpha", self.max_alpha)?;
        validate_non_negative("depth_attenuation", self.depth_attenuation)?;
        validate_non_negative("facing_tint_min", self.facing_tint_min)?;
        validate_non_negative("facing_tint_max", self.facing_tint_max)?;
        validate_non_negative("facing_scale_min", self.facing_scale_min)?;
        validate_non_negative("facing_scale_max", self.facing_scale_max)?;
        if !self.spin_radians_per_second.is_finite() {
            return Err(OpticsError::InvalidValue("spin_radians_per_second"));
        }
        if self.max_alpha > 1.0
            || self.facing_tint_min > self.facing_tint_max
            || self.facing_scale_min > self.facing_scale_max
        {
            return Err(OpticsError::InvalidValue(
                "transparent particle profile range",
            ));
        }
        Ok(())
    }
}

impl Default for ParticleVisualAnimationProfile {
    fn default() -> Self {
        Self::new("particle.animation.default")
    }
}

/// Builds a visual frame by applying one animation profile to Matter particles.
///
/// # Errors
///
/// Returns [`OpticsError`] when the source payload, profile, or resolved visual
/// frame is invalid.
pub fn resolve_animated_particle_visual_frame(
    frame_id: impl Into<String>,
    payload: &ParticleRenderPayload,
    profile: &ParticleVisualAnimationProfile,
) -> Result<ParticleVisualFrame, OpticsError> {
    payload
        .validate()
        .map_err(|_| OpticsError::InvalidPayload("source Matter particle payload is invalid"))?;
    profile.validate()?;

    let sample_count = payload.samples.len().max(1) as f32;
    let samples = payload
        .samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let index_phase = index as f32 / sample_count;
            let phase01 = cycle_phase01(sample.age_seconds.mul_add(
                profile.animation_cycles_per_second,
                index_phase * profile.index_phase_stride,
            ));
            let (size_multiplier, size_t) = profile.size.sample(phase01);
            let (alpha_value, alpha_t) = profile.alpha.sample(phase01);
            let speed01 = if profile.speed_reference > 0.0 {
                (sample.speed / profile.speed_reference).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let color_phase = cycle_phase01(phase01 + (speed01 * 0.18));
            let mut color = profile.color_ramp.sample(color_phase);
            color.a = (alpha_value * profile.opacity_multiplier)
                .clamp(0.0, profile.max_alpha.clamp(0.0, 1.0));

            ParticleVisualSample {
                schema_id: rusty_optics_model::PARTICLE_VISUAL_SAMPLE_SCHEMA_ID.to_owned(),
                source_particle_id: sample.particle_id.clone(),
                position: sample.position,
                radius: (sample.radius * size_multiplier).max(0.0),
                color,
                normal: velocity_normal(sample.velocity),
                rotation_radians: sample.age_seconds.mul_add(
                    profile.spin_radians_per_second,
                    phase01 * core::f32::consts::TAU,
                ),
                frame01: to_half_open_frame01(phase01),
                aux0: speed01,
                aux1: (size_t + alpha_t) * 0.5,
                flags: sample.flags,
            }
        })
        .collect();

    let frame = ParticleVisualFrame {
        schema_id: PARTICLE_VISUAL_FRAME_SCHEMA_ID.to_owned(),
        frame_id: frame_id.into(),
        source_payload_id: payload.payload_id.clone(),
        source_schema_id: payload.schema_id.clone(),
        time_seconds: payload.time_seconds,
        samples,
    };
    frame.validate()?;
    Ok(frame)
}

/// Wraps a scalar into the half-open animation phase range `0..1`.
#[must_use]
pub fn cycle_phase01(value: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    value.rem_euclid(1.0)
}

/// Samples a sine-shaped envelope over a half-open animation phase.
#[must_use]
pub fn hump_envelope(phase01: f32) -> f32 {
    (cycle_phase01(phase01) * core::f32::consts::PI)
        .sin()
        .max(0.0)
}

fn velocity_normal(velocity: Vec3) -> Vec3 {
    let length = velocity.length();
    if velocity.is_finite() && length.is_finite() && length > 1.0e-6 {
        velocity / length
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    }
}

fn validate_non_negative(field: &'static str, value: f32) -> Result<(), OpticsError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(OpticsError::InvalidValue(field))
    }
}

fn clamp01(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn lerp(left: f32, right: f32, t: f32) -> f32 {
    left + ((right - left) * clamp01(t))
}

fn lerp_color(left: ColorRgba, right: ColorRgba, t: f32) -> ColorRgba {
    let t = clamp01(t);
    ColorRgba::new(
        lerp(left.r, right.r, t),
        lerp(left.g, right.g, t),
        lerp(left.b, right.b, t),
        lerp(left.a, right.a, t),
    )
}
