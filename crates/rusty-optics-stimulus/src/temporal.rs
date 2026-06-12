use rusty_optics_model::{OpticsError, STIMULUS_TEMPORAL_PROFILE_SCHEMA_ID};

/// Temporal pulse/gating profile for a procedural stimulus.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StimulusTemporalProfile {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable temporal profile identifier.
    pub temporal_id: String,
    /// Requested cycle frequency in cycles per second.
    pub target_cycle_hz: f32,
    /// On fraction of each cycle in `0..1`.
    pub duty_cycle: f32,
    /// Requested run duration after lead-in.
    pub duration_seconds: f32,
    /// Black lead-in before the first active frame.
    pub black_lead_in_seconds: f32,
    /// Whether an adapter must wait for an explicit start action.
    pub start_gate_required: bool,
    /// Static phase offset added to the temporal cycle.
    pub phase_offset: f32,
}

impl StimulusTemporalProfile {
    /// Creates a low-frequency development preview profile.
    #[must_use]
    pub fn preview_pulse(temporal_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_TEMPORAL_PROFILE_SCHEMA_ID.to_owned(),
            temporal_id: temporal_id.into(),
            target_cycle_hz: 2.0,
            duty_cycle: 0.5,
            duration_seconds: 20.0,
            black_lead_in_seconds: 1.0,
            start_gate_required: true,
            phase_offset: 0.0,
        }
    }

    /// Effective on/off transition budget in switches per second.
    #[must_use]
    pub fn target_switch_hz(&self) -> f32 {
        self.target_cycle_hz * 2.0
    }

    /// Samples the temporal gate at elapsed seconds.
    #[must_use]
    pub fn sample(&self, elapsed_seconds: f32) -> TemporalSample {
        if elapsed_seconds < self.black_lead_in_seconds {
            return TemporalSample {
                in_black_lead_in: true,
                cycle_phase: 0.0,
                gate_on: false,
            };
        }
        let active_time = elapsed_seconds - self.black_lead_in_seconds;
        let phase = fract01(active_time.mul_add(self.target_cycle_hz, self.phase_offset));
        TemporalSample {
            in_black_lead_in: false,
            cycle_phase: phase,
            gate_on: phase < self.duty_cycle,
        }
    }

    /// Validates temporal profile shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != STIMULUS_TEMPORAL_PROFILE_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: STIMULUS_TEMPORAL_PROFILE_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.temporal_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("temporal_id"));
        }
        validate_positive("target_cycle_hz", self.target_cycle_hz)?;
        if !self.duty_cycle.is_finite() || self.duty_cycle <= 0.0 || self.duty_cycle >= 1.0 {
            return Err(OpticsError::InvalidValue("duty_cycle"));
        }
        validate_positive("duration_seconds", self.duration_seconds)?;
        validate_non_negative("black_lead_in_seconds", self.black_lead_in_seconds)?;
        if !self.phase_offset.is_finite() {
            return Err(OpticsError::InvalidValue("phase_offset"));
        }
        Ok(())
    }
}

/// Result of temporal gate sampling.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TemporalSample {
    /// Whether the current time is inside the black lead-in.
    pub in_black_lead_in: bool,
    /// Cycle phase in `0..1`.
    pub cycle_phase: f32,
    /// Whether the stimulus gate is active.
    pub gate_on: bool,
}

fn fract01(value: f32) -> f32 {
    value - value.floor()
}

fn validate_positive(field: &'static str, value: f32) -> Result<(), OpticsError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(OpticsError::InvalidValue(field))
    }
}

fn validate_non_negative(field: &'static str, value: f32) -> Result<(), OpticsError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(OpticsError::InvalidValue(field))
    }
}
