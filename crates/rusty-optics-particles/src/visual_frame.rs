use rusty_matter_model::Vec3;
use rusty_matter_particles::{ParticleRenderPayload, ParticleRenderSample};
use rusty_optics_model::{
    ColorRgba, OpticsError, PARTICLE_VISUAL_FRAME_SCHEMA_ID, PARTICLE_VISUAL_SAMPLE_SCHEMA_ID,
};

/// One resolved visual particle sample.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleVisualSample {
    /// Schema identifier.
    pub schema_id: String,
    /// Source particle identifier.
    pub source_particle_id: String,
    /// Particle center in source coordinate space.
    pub position: Vec3,
    /// Visual radius in source units.
    pub radius: f32,
    /// Visual color.
    pub color: ColorRgba,
    /// Surface or facing normal.
    pub normal: Vec3,
    /// Rotation about the particle-facing axis.
    pub rotation_radians: f32,
    /// Animation frame phase in the half-open `0..1` interval.
    pub frame01: f32,
    /// Renderer-neutral auxiliary value.
    pub aux0: f32,
    /// Renderer-neutral auxiliary value.
    pub aux1: f32,
    /// Domain-neutral visual flags.
    pub flags: u32,
}

impl ParticleVisualSample {
    /// Creates a visual sample from a Matter render-neutral sample.
    #[must_use]
    pub fn from_matter_sample(sample: &ParticleRenderSample, color: ColorRgba) -> Self {
        Self {
            schema_id: PARTICLE_VISUAL_SAMPLE_SCHEMA_ID.to_owned(),
            source_particle_id: sample.particle_id.clone(),
            position: sample.position,
            radius: sample.radius,
            color: color.clamped01(),
            normal: Vec3::new(0.0, 1.0, 0.0),
            rotation_radians: 0.0,
            frame01: 0.0,
            aux0: 1.0,
            aux1: 0.0,
            flags: sample.flags,
        }
    }

    /// Validates visual sample shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PARTICLE_VISUAL_SAMPLE_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PARTICLE_VISUAL_SAMPLE_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.source_particle_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_particle_id"));
        }
        if !self.position.is_finite() {
            return Err(OpticsError::NonFiniteVec3("position"));
        }
        if !self.radius.is_finite() || self.radius < 0.0 {
            return Err(OpticsError::InvalidValue("radius"));
        }
        if !self.color.is_finite() {
            return Err(OpticsError::NonFiniteColor("color"));
        }
        if !self.normal.is_finite() {
            return Err(OpticsError::NonFiniteVec3("normal"));
        }
        if !self.rotation_radians.is_finite() {
            return Err(OpticsError::InvalidValue("rotation_radians"));
        }
        if !self.frame01.is_finite() || !(0.0..1.0).contains(&self.frame01) {
            return Err(OpticsError::InvalidValue("frame01"));
        }
        if !self.aux0.is_finite() || !self.aux1.is_finite() {
            return Err(OpticsError::InvalidValue("aux"));
        }
        Ok(())
    }
}

/// Resolved visual particle frame.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleVisualFrame {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable frame identifier.
    pub frame_id: String,
    /// Source Matter payload identifier.
    pub source_payload_id: String,
    /// Source Matter payload schema identifier.
    pub source_schema_id: String,
    /// Source payload time in seconds.
    pub time_seconds: f32,
    /// Resolved visual particle samples.
    pub samples: Vec<ParticleVisualSample>,
}

impl ParticleVisualFrame {
    /// Creates a visual frame from a Matter render-neutral payload.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the resulting frame is invalid.
    pub fn from_matter_payload(
        frame_id: impl Into<String>,
        payload: &ParticleRenderPayload,
        color: ColorRgba,
    ) -> Result<Self, OpticsError> {
        let frame = Self {
            schema_id: PARTICLE_VISUAL_FRAME_SCHEMA_ID.to_owned(),
            frame_id: frame_id.into(),
            source_payload_id: payload.payload_id.clone(),
            source_schema_id: payload.schema_id.clone(),
            time_seconds: payload.time_seconds,
            samples: payload
                .samples
                .iter()
                .map(|sample| ParticleVisualSample::from_matter_sample(sample, color))
                .collect(),
        };
        frame.validate()?;
        Ok(frame)
    }

    /// Validates visual frame shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PARTICLE_VISUAL_FRAME_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PARTICLE_VISUAL_FRAME_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.frame_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("frame_id"));
        }
        if self.source_payload_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_payload_id"));
        }
        if self.source_schema_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_schema_id"));
        }
        if !self.time_seconds.is_finite() || self.time_seconds < 0.0 {
            return Err(OpticsError::InvalidValue("time_seconds"));
        }
        for sample in &self.samples {
            sample.validate()?;
        }
        Ok(())
    }
}

/// Clamps an animation frame phase into the half-open `0..1` interval.
#[must_use]
pub fn to_half_open_frame01(value: f32) -> f32 {
    if !value.is_finite() || value <= 0.0 {
        0.0
    } else if value >= 1.0 {
        1.0 - f32::EPSILON
    } else {
        value
    }
}
