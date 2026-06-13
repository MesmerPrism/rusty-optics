use rusty_optics_model::{OpticsError, STIMULUS_VOLUME_SCHEMA_ID};

use crate::{ComputePassKind, StimulusProfile};

/// Compact renderer-adapter summary of an Optics stimulus volume profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StimulusVolumeProfileSummary {
    /// Whether the profile declares a volume descriptor.
    pub volume_present: bool,
    /// Volume descriptor schema id.
    pub volume_schema: Option<String>,
    /// Volume id read from the staged profile.
    pub volume_id: Option<String>,
    /// Optics volume field kind.
    pub field_kind: Option<String>,
    /// Intended renderer storage class.
    pub storage_hint: Option<String>,
    /// Bounded descriptor grid dimensions.
    pub grid_dimensions: Option<[u64; 3]>,
    /// Bounded raymarch/probe step count.
    pub step_count: Option<u64>,
    /// Kernel ABI id selected by the profile.
    pub kernel_abi_id: Option<String>,
    /// Count of declared compute passes in the Optics ABI.
    pub compute_pass_count: usize,
    /// Bounded readback samples declared by the volume probe pass.
    pub volume_readback_probe_samples: Option<u64>,
    /// Output layers declared by the stereo raymarch pass.
    pub stereo_field_output_layers: Option<u64>,
}

impl StimulusVolumeProfileSummary {
    /// Extracts the adapter-facing volume summary from a validated Optics profile.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the profile is invalid.
    pub fn from_profile(profile: &StimulusProfile) -> Result<Self, OpticsError> {
        profile.validate()?;
        let Some(volume) = &profile.volume else {
            return Ok(Self::default());
        };

        let volume_readback_probe_samples = profile
            .kernel_abi
            .compute_passes
            .iter()
            .find(|pass| pass.kind == ComputePassKind::VolumeReadbackProbe)
            .map(|pass| pass.bounded_readback_samples as u64);
        let stereo_field_output_layers = profile
            .kernel_abi
            .compute_passes
            .iter()
            .find(|pass| pass.kind == ComputePassKind::VolumeRaymarchStereoField)
            .map(|pass| u64::from(pass.output_layers));

        Ok(Self {
            volume_present: true,
            volume_schema: Some(volume.schema_id.clone()),
            volume_id: Some(volume.volume_id.clone()),
            field_kind: Some(volume.field_kind.as_str().to_owned()),
            storage_hint: Some(volume.storage_hint.as_str().to_owned()),
            grid_dimensions: Some([
                u64::from(volume.grid_dimensions[0]),
                u64::from(volume.grid_dimensions[1]),
                u64::from(volume.grid_dimensions[2]),
            ]),
            step_count: Some(u64::from(volume.step_count)),
            kernel_abi_id: Some(profile.kernel_abi.abi_id.clone()),
            compute_pass_count: profile.kernel_abi.compute_passes.len(),
            volume_readback_probe_samples,
            stereo_field_output_layers,
        })
    }

    /// Validates the bounded stereo preview shape expected by mobile adapters.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when required volume or stereo readback metadata
    /// is missing or outside the supplied readback bound.
    pub fn validate_bounded_stereo_preview(
        &self,
        max_readback_samples: u64,
    ) -> Result<(), OpticsError> {
        if !self.volume_present {
            return Err(OpticsError::InvalidPayload("volume summary missing volume"));
        }
        if self.volume_schema.as_deref() != Some(STIMULUS_VOLUME_SCHEMA_ID) {
            return Err(OpticsError::InvalidValue("volume_summary.volume_schema"));
        }
        if self.volume_id.as_deref().is_none_or(str::is_empty)
            || self.field_kind.as_deref().is_none_or(str::is_empty)
            || self.storage_hint.as_deref().is_none_or(str::is_empty)
            || self.kernel_abi_id.as_deref().is_none_or(str::is_empty)
        {
            return Err(OpticsError::EmptyId("volume_summary"));
        }
        let Some(grid_dimensions) = self.grid_dimensions else {
            return Err(OpticsError::InvalidValue("volume_summary.grid_dimensions"));
        };
        if grid_dimensions.iter().any(|dim| *dim == 0 || *dim > 512) {
            return Err(OpticsError::InvalidValue("volume_summary.grid_dimensions"));
        }
        if self
            .step_count
            .is_none_or(|step_count| step_count == 0 || step_count > 256)
        {
            return Err(OpticsError::InvalidCount("volume_summary.step_count"));
        }
        if self.volume_readback_probe_samples.is_none_or(|samples| {
            samples == 0 || samples > max_readback_samples || max_readback_samples == 0
        }) {
            return Err(OpticsError::InvalidCount(
                "volume_summary.volume_readback_probe_samples",
            ));
        }
        if self.stereo_field_output_layers != Some(2) {
            return Err(OpticsError::InvalidCount(
                "volume_summary.stereo_field_output_layers",
            ));
        }
        Ok(())
    }
}
