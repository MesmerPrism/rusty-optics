use rusty_optics_model::{OpticsError, STIMULUS_RUN_PLAN_SCHEMA_ID};

use crate::StimulusTemporalProfile;

/// Frame-quantization outcome for a stimulus run.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunPlanFeasibility {
    /// Requested cycle and duty map exactly to whole frames.
    Exact,
    /// Requested timing is usable but quantized to the display cadence.
    Quantized,
    /// Requested switching exceeds one on/off transition pair per frame pair.
    ExceedsSwitchBudget,
}

/// Display-cadence run plan derived from a temporal profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StimulusRunPlan {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable run-plan identifier.
    pub run_plan_id: String,
    /// Source temporal profile identifier.
    pub temporal_profile_id: String,
    /// Target display refresh rate.
    pub target_refresh_hz: f32,
    /// Requested cycle rate.
    pub requested_cycle_hz: f32,
    /// Whole display frames per quantized cycle.
    pub frames_per_cycle: u32,
    /// Whole display frames with the gate on.
    pub on_frames_per_cycle: u32,
    /// Whole display frames with the gate off.
    pub off_frames_per_cycle: u32,
    /// Achieved cycle rate after frame quantization.
    pub quantized_cycle_hz: f32,
    /// Achieved duty cycle after frame quantization.
    pub quantized_duty_cycle: f32,
    /// Feasibility result.
    pub feasibility: RunPlanFeasibility,
}

impl StimulusRunPlan {
    /// Builds a run plan from temporal profile and display refresh.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when inputs are invalid.
    pub fn from_temporal(
        run_plan_id: impl Into<String>,
        temporal: &StimulusTemporalProfile,
        target_refresh_hz: f32,
    ) -> Result<Self, OpticsError> {
        temporal.validate()?;
        if !target_refresh_hz.is_finite() || target_refresh_hz <= 0.0 {
            return Err(OpticsError::InvalidValue("target_refresh_hz"));
        }

        let raw_frames_per_cycle = target_refresh_hz / temporal.target_cycle_hz;
        let frames_per_cycle = raw_frames_per_cycle.round().max(2.0) as u32;
        let feasibility = if temporal.target_switch_hz() > target_refresh_hz {
            RunPlanFeasibility::ExceedsSwitchBudget
        } else if (raw_frames_per_cycle - frames_per_cycle as f32).abs() <= f32::EPSILON {
            RunPlanFeasibility::Exact
        } else {
            RunPlanFeasibility::Quantized
        };

        let on_frames = (frames_per_cycle as f32 * temporal.duty_cycle).round();
        let on_frames_per_cycle = clamp_frame_count(on_frames as u32, frames_per_cycle);
        let off_frames_per_cycle = frames_per_cycle.saturating_sub(on_frames_per_cycle);
        let quantized_cycle_hz = target_refresh_hz / frames_per_cycle as f32;
        let quantized_duty_cycle = on_frames_per_cycle as f32 / frames_per_cycle as f32;

        let plan = Self {
            schema_id: STIMULUS_RUN_PLAN_SCHEMA_ID.to_owned(),
            run_plan_id: run_plan_id.into(),
            temporal_profile_id: temporal.temporal_id.clone(),
            target_refresh_hz,
            requested_cycle_hz: temporal.target_cycle_hz,
            frames_per_cycle,
            on_frames_per_cycle,
            off_frames_per_cycle,
            quantized_cycle_hz,
            quantized_duty_cycle,
            feasibility,
        };
        plan.validate()?;
        Ok(plan)
    }

    /// Validates run-plan shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != STIMULUS_RUN_PLAN_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: STIMULUS_RUN_PLAN_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.run_plan_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("run_plan_id"));
        }
        if self.temporal_profile_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("temporal_profile_id"));
        }
        validate_positive("target_refresh_hz", self.target_refresh_hz)?;
        validate_positive("requested_cycle_hz", self.requested_cycle_hz)?;
        if self.frames_per_cycle == 0 {
            return Err(OpticsError::InvalidCount("frames_per_cycle"));
        }
        if self.on_frames_per_cycle == 0 {
            return Err(OpticsError::InvalidCount("on_frames_per_cycle"));
        }
        if self.off_frames_per_cycle == 0 {
            return Err(OpticsError::InvalidCount("off_frames_per_cycle"));
        }
        if self.on_frames_per_cycle + self.off_frames_per_cycle != self.frames_per_cycle {
            return Err(OpticsError::InvalidPayload(
                "run-plan frame counts must sum",
            ));
        }
        validate_positive("quantized_cycle_hz", self.quantized_cycle_hz)?;
        if !self.quantized_duty_cycle.is_finite()
            || self.quantized_duty_cycle <= 0.0
            || self.quantized_duty_cycle >= 1.0
        {
            return Err(OpticsError::InvalidValue("quantized_duty_cycle"));
        }
        Ok(())
    }
}

fn clamp_frame_count(count: u32, frames_per_cycle: u32) -> u32 {
    if frames_per_cycle <= 1 {
        1
    } else {
        count.clamp(1, frames_per_cycle - 1)
    }
}

fn validate_positive(field: &'static str, value: f32) -> Result<(), OpticsError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(OpticsError::InvalidValue(field))
    }
}
