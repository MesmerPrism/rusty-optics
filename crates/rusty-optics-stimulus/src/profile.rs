use rusty_optics_model::{OpticsError, STIMULUS_PROFILE_SCHEMA_ID};

use crate::{
    StimulusKernelAbi, StimulusLayerGraph, StimulusPresentationDescriptor, StimulusRunPlan,
    StimulusSafetyProfile, StimulusTemporalProfile,
};

/// Complete renderer-neutral procedural stimulus profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StimulusProfile {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable profile identifier.
    pub profile_id: String,
    /// Procedural layer graph.
    pub layer_graph: StimulusLayerGraph,
    /// Temporal gating profile.
    pub temporal: StimulusTemporalProfile,
    /// Safety policy.
    pub safety: StimulusSafetyProfile,
    /// XR/browser presentation target.
    pub presentation: StimulusPresentationDescriptor,
    /// Kernel ABI request.
    pub kernel_abi: StimulusKernelAbi,
}

impl StimulusProfile {
    /// Creates a deterministic low-frequency interference preview profile.
    #[must_use]
    pub fn interference_preview(profile_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_PROFILE_SCHEMA_ID.to_owned(),
            profile_id: profile_id.into(),
            layer_graph: StimulusLayerGraph::interference_preview(
                "stimulus.graph.interference_preview",
            ),
            temporal: StimulusTemporalProfile::preview_pulse("stimulus.temporal.preview_pulse"),
            safety: StimulusSafetyProfile::preview("stimulus.safety.preview"),
            presentation: StimulusPresentationDescriptor::stereo_eye_field(
                "stimulus.presentation.stereo_eye_fullscreen",
            ),
            kernel_abi: StimulusKernelAbi::compute_interference_v1(
                "stimulus.kernel.compute_interference_v1",
            ),
        }
    }

    /// Builds a display run plan for this profile.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the profile or refresh rate is invalid.
    pub fn run_plan(
        &self,
        run_plan_id: impl Into<String>,
        target_refresh_hz: f32,
    ) -> Result<StimulusRunPlan, OpticsError> {
        self.validate()?;
        StimulusRunPlan::from_temporal(run_plan_id, &self.temporal, target_refresh_hz)
    }

    /// Validates profile shape and cross-field safety limits.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != STIMULUS_PROFILE_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: STIMULUS_PROFILE_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.profile_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("profile_id"));
        }
        self.layer_graph.validate()?;
        self.temporal.validate()?;
        self.safety.validate_temporal(&self.temporal)?;
        self.presentation.validate()?;
        self.kernel_abi.validate()?;
        if self.layer_graph.layers.len() > self.kernel_abi.max_layers {
            return Err(OpticsError::InvalidCount("layer_graph.layers"));
        }
        Ok(())
    }
}
