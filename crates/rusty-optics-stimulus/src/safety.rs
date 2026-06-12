use rusty_optics_model::{OpticsError, STIMULUS_SAFETY_PROFILE_SCHEMA_ID};

use crate::StimulusTemporalProfile;

/// Safety gate class for a procedural stimulus profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StimulusSafetyClass {
    /// Low-frequency development preview.
    Preview,
    /// High-contrast or fast-changing stimulus requiring explicit handling.
    PhotosensitiveRisk,
    /// Research protocol controlled outside the renderer adapter.
    ResearchProtocol,
}

/// Renderer-neutral safety policy attached to a procedural stimulus.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StimulusSafetyProfile {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable safety profile identifier.
    pub safety_id: String,
    /// Safety class.
    pub class: StimulusSafetyClass,
    /// Whether an adapter must require explicit acknowledgement.
    pub requires_acknowledgement: bool,
    /// Whether an adapter may start playback automatically.
    pub allow_autostart: bool,
    /// Maximum run duration after lead-in.
    pub max_duration_seconds: f32,
    /// Maximum requested cycle frequency.
    pub max_cycle_hz: f32,
    /// Maximum intended luminance delta in `0..1`.
    pub max_luminance_delta: f32,
    /// Whether a black lead-in is required.
    pub require_black_lead_in: bool,
}

impl StimulusSafetyProfile {
    /// Creates a conservative development preview policy.
    #[must_use]
    pub fn preview(safety_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_SAFETY_PROFILE_SCHEMA_ID.to_owned(),
            safety_id: safety_id.into(),
            class: StimulusSafetyClass::Preview,
            requires_acknowledgement: false,
            allow_autostart: false,
            max_duration_seconds: 60.0,
            max_cycle_hz: 3.0,
            max_luminance_delta: 0.5,
            require_black_lead_in: true,
        }
    }

    /// Creates a policy for high-contrast or faster-changing stimuli.
    #[must_use]
    pub fn photosensitive_risk(safety_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_SAFETY_PROFILE_SCHEMA_ID.to_owned(),
            safety_id: safety_id.into(),
            class: StimulusSafetyClass::PhotosensitiveRisk,
            requires_acknowledgement: true,
            allow_autostart: false,
            max_duration_seconds: 30.0,
            max_cycle_hz: 30.0,
            max_luminance_delta: 1.0,
            require_black_lead_in: true,
        }
    }

    /// Creates a policy for externally governed research protocols.
    #[must_use]
    pub fn research_protocol(safety_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_SAFETY_PROFILE_SCHEMA_ID.to_owned(),
            safety_id: safety_id.into(),
            class: StimulusSafetyClass::ResearchProtocol,
            requires_acknowledgement: false,
            allow_autostart: true,
            max_duration_seconds: 3600.0,
            max_cycle_hz: 120.0,
            max_luminance_delta: 1.0,
            require_black_lead_in: false,
        }
    }

    /// Validates standalone safety profile fields.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != STIMULUS_SAFETY_PROFILE_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: STIMULUS_SAFETY_PROFILE_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.safety_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("safety_id"));
        }
        validate_positive("max_duration_seconds", self.max_duration_seconds)?;
        validate_positive("max_cycle_hz", self.max_cycle_hz)?;
        validate_unit("max_luminance_delta", self.max_luminance_delta)?;
        match self.class {
            StimulusSafetyClass::Preview => {
                if self.max_cycle_hz > 3.0 || self.max_luminance_delta > 0.5 {
                    return Err(OpticsError::InvalidValue("preview_safety_bounds"));
                }
            }
            StimulusSafetyClass::PhotosensitiveRisk => {
                if !self.requires_acknowledgement || self.allow_autostart {
                    return Err(OpticsError::InvalidValue("risk_acknowledgement_gate"));
                }
            }
            StimulusSafetyClass::ResearchProtocol => {}
        }
        Ok(())
    }

    /// Validates a temporal profile against this safety policy.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when temporal settings exceed policy bounds.
    pub fn validate_temporal(&self, temporal: &StimulusTemporalProfile) -> Result<(), OpticsError> {
        self.validate()?;
        temporal.validate()?;
        if temporal.target_cycle_hz > self.max_cycle_hz {
            return Err(OpticsError::InvalidValue("temporal.target_cycle_hz"));
        }
        if temporal.duration_seconds > self.max_duration_seconds {
            return Err(OpticsError::InvalidValue("temporal.duration_seconds"));
        }
        if self.require_black_lead_in && temporal.black_lead_in_seconds <= 0.0 {
            return Err(OpticsError::InvalidValue("temporal.black_lead_in_seconds"));
        }
        if !self.allow_autostart && !temporal.start_gate_required {
            return Err(OpticsError::InvalidValue("temporal.start_gate_required"));
        }
        Ok(())
    }
}

fn validate_positive(field: &'static str, value: f32) -> Result<(), OpticsError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(OpticsError::InvalidValue(field))
    }
}

fn validate_unit(field: &'static str, value: f32) -> Result<(), OpticsError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(OpticsError::InvalidValue(field))
    }
}
