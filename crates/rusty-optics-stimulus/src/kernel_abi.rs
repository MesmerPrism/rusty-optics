use rusty_optics_model::{OpticsError, STIMULUS_KERNEL_ABI_SCHEMA_ID};

/// Execution family preferred by a procedural stimulus adapter.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelExecutionModel {
    /// Stateless full-field fragment evaluation.
    FragmentField,
    /// Field-texture generation through a compute-like adapter.
    ComputeFieldTexture,
    /// Deterministic CPU sampling for fixtures and scorecards.
    CpuReference,
}

/// Renderer-neutral compute pass family requested by a stimulus profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputePassKind {
    /// Generate the current procedural field into a texture-like resource.
    FieldTexture,
    /// Prepare deterministic noise/cache resources.
    NoiseCache,
    /// Combine the current field with previous-frame history.
    HistoryFeedback,
    /// Read back a small bounded sample set for validation.
    BoundedReadbackProbe,
    /// Generate or cache a bounded volume density resource.
    VolumeDensityCache,
    /// Raymarch a volume into a low-resolution stereo eye field.
    VolumeRaymarchStereoField,
    /// Reproject prior-frame volume history.
    VolumeHistoryReproject,
    /// Read back bounded volume probe samples for CPU-oracle validation.
    VolumeReadbackProbe,
}

/// Renderer-neutral texture/sample format requested by a compute pass.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputeTextureFormat {
    /// Single-channel normalized 8-bit output.
    R8Unorm,
    /// Single-channel 16-bit float output.
    R16Float,
    /// Single-channel 32-bit float output.
    R32Float,
    /// Four-channel 16-bit float output.
    Rgba16Float,
    /// Four-channel 32-bit float output.
    Rgba32Float,
}

/// Workgroup dimensions requested by a compute-like adapter.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComputeWorkgroupSize {
    /// X dimension.
    pub x: u32,
    /// Y dimension.
    pub y: u32,
    /// Z dimension.
    pub z: u32,
}

impl ComputeWorkgroupSize {
    /// Creates a workgroup size descriptor.
    #[must_use]
    pub const fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    fn invocation_count(self) -> u32 {
        self.x.saturating_mul(self.y).saturating_mul(self.z)
    }
}

/// Portability limits for compute-like stimulus adapters.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ComputePortabilityProfile {
    /// Stable portability profile identifier.
    pub profile_id: String,
    /// Maximum invocations allowed in one workgroup.
    pub max_workgroup_invocations: u32,
    /// Maximum 2D resource extent requested by this profile.
    pub max_resource_dimension_px: u32,
    /// Parameter buffer alignment requested by generated adapter layouts.
    pub parameter_buffer_alignment_bytes: u32,
    /// Whether subgroup operations are required.
    pub requires_subgroup_ops: bool,
    /// Whether shader device address is required.
    pub requires_device_address: bool,
    /// Whether runtime descriptor arrays are required.
    pub requires_runtime_descriptor_arrays: bool,
    /// Whether 16-bit float storage is required rather than preferred.
    pub requires_fp16_storage: bool,
}

impl ComputePortabilityProfile {
    /// Conservative profile intended for mobile Vulkan/WebGPU-style lowering.
    #[must_use]
    pub fn mobile_vulkan_portable(profile_id: impl Into<String>) -> Self {
        Self {
            profile_id: profile_id.into(),
            max_workgroup_invocations: 64,
            max_resource_dimension_px: 2048,
            parameter_buffer_alignment_bytes: 16,
            requires_subgroup_ops: false,
            requires_device_address: false,
            requires_runtime_descriptor_arrays: false,
            requires_fp16_storage: false,
        }
    }

    /// Validates portability limits.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.profile_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("portability.profile_id"));
        }
        if self.max_workgroup_invocations == 0 || self.max_workgroup_invocations > 1024 {
            return Err(OpticsError::InvalidValue(
                "portability.max_workgroup_invocations",
            ));
        }
        if self.max_resource_dimension_px == 0 || self.max_resource_dimension_px > 8192 {
            return Err(OpticsError::InvalidValue(
                "portability.max_resource_dimension_px",
            ));
        }
        if !self.parameter_buffer_alignment_bytes.is_power_of_two()
            || self.parameter_buffer_alignment_bytes < 4
        {
            return Err(OpticsError::InvalidValue(
                "portability.parameter_buffer_alignment_bytes",
            ));
        }
        Ok(())
    }
}

/// One renderer-neutral compute pass requested by a stimulus kernel ABI.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StimulusComputePassDescriptor {
    /// Stable pass identifier.
    pub pass_id: String,
    /// Pass family.
    pub kind: ComputePassKind,
    /// Requested workgroup size.
    pub workgroup_size: ComputeWorkgroupSize,
    /// Nominal output width in pixels.
    pub output_width_px: u32,
    /// Nominal output height in pixels.
    pub output_height_px: u32,
    /// Output layer count for array-like stereo resources.
    #[cfg_attr(feature = "serde", serde(default = "default_output_layers"))]
    pub output_layers: u32,
    /// Output/sample format.
    pub output_format: ComputeTextureFormat,
    /// Whether this pass consumes layer parameter buffers.
    pub reads_layer_parameters: bool,
    /// Whether this pass consumes a deterministic noise texture/cache.
    pub reads_noise_texture: bool,
    /// Whether this pass consumes prior-frame history.
    pub reads_history_buffer: bool,
    /// Whether this pass writes prior-frame history for later frames.
    pub writes_history_buffer: bool,
    /// Whether this pass consumes a volume-field resource.
    #[cfg_attr(feature = "serde", serde(default))]
    pub reads_volume_field: bool,
    /// Whether this pass writes a volume-field resource.
    #[cfg_attr(feature = "serde", serde(default))]
    pub writes_volume_field: bool,
    /// Whether this pass writes a stereo eye-field resource.
    #[cfg_attr(feature = "serde", serde(default))]
    pub writes_stereo_field: bool,
    /// Bounded readback samples requested by this pass.
    pub bounded_readback_samples: usize,
}

impl StimulusComputePassDescriptor {
    /// Creates a field-texture compute pass descriptor.
    #[must_use]
    pub fn field_texture(pass_id: impl Into<String>, width_px: u32, height_px: u32) -> Self {
        Self {
            pass_id: pass_id.into(),
            kind: ComputePassKind::FieldTexture,
            workgroup_size: ComputeWorkgroupSize::new(8, 8, 1),
            output_width_px: width_px,
            output_height_px: height_px,
            output_layers: 1,
            output_format: ComputeTextureFormat::Rgba16Float,
            reads_layer_parameters: true,
            reads_noise_texture: true,
            reads_history_buffer: false,
            writes_history_buffer: false,
            reads_volume_field: false,
            writes_volume_field: false,
            writes_stereo_field: false,
            bounded_readback_samples: 0,
        }
    }

    /// Creates a deterministic noise-cache pass descriptor.
    #[must_use]
    pub fn noise_cache(pass_id: impl Into<String>, width_px: u32, height_px: u32) -> Self {
        Self {
            pass_id: pass_id.into(),
            kind: ComputePassKind::NoiseCache,
            workgroup_size: ComputeWorkgroupSize::new(8, 8, 1),
            output_width_px: width_px,
            output_height_px: height_px,
            output_layers: 1,
            output_format: ComputeTextureFormat::R16Float,
            reads_layer_parameters: true,
            reads_noise_texture: false,
            reads_history_buffer: false,
            writes_history_buffer: false,
            reads_volume_field: false,
            writes_volume_field: false,
            writes_stereo_field: false,
            bounded_readback_samples: 0,
        }
    }

    /// Creates a history-feedback pass descriptor.
    #[must_use]
    pub fn history_feedback(pass_id: impl Into<String>, width_px: u32, height_px: u32) -> Self {
        Self {
            pass_id: pass_id.into(),
            kind: ComputePassKind::HistoryFeedback,
            workgroup_size: ComputeWorkgroupSize::new(8, 8, 1),
            output_width_px: width_px,
            output_height_px: height_px,
            output_layers: 1,
            output_format: ComputeTextureFormat::Rgba16Float,
            reads_layer_parameters: true,
            reads_noise_texture: false,
            reads_history_buffer: true,
            writes_history_buffer: true,
            reads_volume_field: false,
            writes_volume_field: false,
            writes_stereo_field: false,
            bounded_readback_samples: 0,
        }
    }

    /// Creates a bounded readback probe pass descriptor.
    #[must_use]
    pub fn readback_probe(pass_id: impl Into<String>, samples: usize) -> Self {
        Self {
            pass_id: pass_id.into(),
            kind: ComputePassKind::BoundedReadbackProbe,
            workgroup_size: ComputeWorkgroupSize::new(64, 1, 1),
            output_width_px: 1,
            output_height_px: 1,
            output_layers: 1,
            output_format: ComputeTextureFormat::R32Float,
            reads_layer_parameters: true,
            reads_noise_texture: false,
            reads_history_buffer: false,
            writes_history_buffer: false,
            reads_volume_field: false,
            writes_volume_field: false,
            writes_stereo_field: false,
            bounded_readback_samples: samples,
        }
    }

    /// Creates a volume density cache pass descriptor.
    #[must_use]
    pub fn volume_density_cache(pass_id: impl Into<String>, width_px: u32, height_px: u32) -> Self {
        Self {
            pass_id: pass_id.into(),
            kind: ComputePassKind::VolumeDensityCache,
            workgroup_size: ComputeWorkgroupSize::new(8, 8, 1),
            output_width_px: width_px,
            output_height_px: height_px,
            output_layers: 1,
            output_format: ComputeTextureFormat::R32Float,
            reads_layer_parameters: true,
            reads_noise_texture: true,
            reads_history_buffer: false,
            writes_history_buffer: false,
            reads_volume_field: false,
            writes_volume_field: true,
            writes_stereo_field: false,
            bounded_readback_samples: 0,
        }
    }

    /// Creates a low-resolution stereo volume raymarch pass descriptor.
    #[must_use]
    pub fn volume_stereo_field(pass_id: impl Into<String>, width_px: u32, height_px: u32) -> Self {
        Self {
            pass_id: pass_id.into(),
            kind: ComputePassKind::VolumeRaymarchStereoField,
            workgroup_size: ComputeWorkgroupSize::new(8, 8, 1),
            output_width_px: width_px,
            output_height_px: height_px,
            output_layers: 2,
            output_format: ComputeTextureFormat::Rgba16Float,
            reads_layer_parameters: true,
            reads_noise_texture: true,
            reads_history_buffer: false,
            writes_history_buffer: false,
            reads_volume_field: true,
            writes_volume_field: false,
            writes_stereo_field: true,
            bounded_readback_samples: 0,
        }
    }

    /// Creates a bounded volume readback probe pass descriptor.
    #[must_use]
    pub fn volume_readback_probe(pass_id: impl Into<String>, samples: usize) -> Self {
        Self {
            pass_id: pass_id.into(),
            kind: ComputePassKind::VolumeReadbackProbe,
            workgroup_size: ComputeWorkgroupSize::new(64, 1, 1),
            output_width_px: 1,
            output_height_px: 1,
            output_layers: 1,
            output_format: ComputeTextureFormat::Rgba32Float,
            reads_layer_parameters: true,
            reads_noise_texture: false,
            reads_history_buffer: false,
            writes_history_buffer: false,
            reads_volume_field: true,
            writes_volume_field: false,
            writes_stereo_field: false,
            bounded_readback_samples: samples,
        }
    }

    /// Validates pass descriptor shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.pass_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("compute_pass.pass_id"));
        }
        if self.workgroup_size.x == 0 || self.workgroup_size.y == 0 || self.workgroup_size.z == 0 {
            return Err(OpticsError::InvalidValue("compute_pass.workgroup_size"));
        }
        if self.workgroup_size.invocation_count() > 1024 {
            return Err(OpticsError::InvalidValue(
                "compute_pass.workgroup_invocations",
            ));
        }
        if self.output_width_px == 0 || self.output_height_px == 0 || self.output_layers == 0 {
            return Err(OpticsError::InvalidValue("compute_pass.output_extent"));
        }
        if self.output_width_px > 8192 || self.output_height_px > 8192 || self.output_layers > 16 {
            return Err(OpticsError::InvalidValue("compute_pass.output_extent"));
        }
        if self.bounded_readback_samples > 4096 {
            return Err(OpticsError::InvalidCount(
                "compute_pass.bounded_readback_samples",
            ));
        }
        if !matches!(
            self.kind,
            ComputePassKind::BoundedReadbackProbe | ComputePassKind::VolumeReadbackProbe
        ) && self.bounded_readback_samples != 0
        {
            return Err(OpticsError::InvalidValue(
                "compute_pass.bounded_readback_samples",
            ));
        }
        if self.kind == ComputePassKind::VolumeRaymarchStereoField
            && (self.output_layers != 2 || !self.reads_volume_field || !self.writes_stereo_field)
        {
            return Err(OpticsError::InvalidValue(
                "compute_pass.volume_stereo_field",
            ));
        }
        if self.kind == ComputePassKind::VolumeReadbackProbe
            && (self.bounded_readback_samples == 0 || !self.reads_volume_field)
        {
            return Err(OpticsError::InvalidValue(
                "compute_pass.volume_readback_probe",
            ));
        }
        Ok(())
    }
}

const fn default_output_layers() -> u32 {
    1
}

/// Renderer-neutral kernel ABI descriptor for procedural stimuli.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StimulusKernelAbi {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable ABI identifier.
    pub abi_id: String,
    /// Parameter layout version.
    pub parameter_layout_version: u32,
    /// Preferred execution family.
    pub preferred_execution_model: KernelExecutionModel,
    /// Whether a stateless full-field path can run this profile.
    pub supports_fragment: bool,
    /// Whether a field-texture generation path can run this profile.
    pub supports_compute: bool,
    /// Whether this profile requires history from a previous frame.
    pub requires_history_buffer: bool,
    /// Whether this profile requires a seeded noise texture/cache.
    pub requires_noise_texture: bool,
    /// Maximum layer count supported by the profile ABI.
    pub max_layers: usize,
    /// Maximum bounded readback samples requested for validation.
    pub bounded_readback_samples: usize,
    /// Renderer-neutral compute pass plan.
    pub compute_passes: Vec<StimulusComputePassDescriptor>,
    /// Portability limits for adapter lowering.
    pub portability: ComputePortabilityProfile,
}

impl StimulusKernelAbi {
    /// Creates the first clean-room procedural-stimulus ABI.
    #[must_use]
    pub fn procedural_v1(abi_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_KERNEL_ABI_SCHEMA_ID.to_owned(),
            abi_id: abi_id.into(),
            parameter_layout_version: 1,
            preferred_execution_model: KernelExecutionModel::FragmentField,
            supports_fragment: true,
            supports_compute: true,
            requires_history_buffer: false,
            requires_noise_texture: true,
            max_layers: 16,
            bounded_readback_samples: 256,
            portability: ComputePortabilityProfile::mobile_vulkan_portable(
                "stimulus.portability.mobile_vulkan",
            ),
            compute_passes: vec![
                StimulusComputePassDescriptor::field_texture(
                    "stimulus.compute_pass.field_texture",
                    1024,
                    1024,
                ),
                StimulusComputePassDescriptor::readback_probe(
                    "stimulus.compute_pass.readback_probe",
                    256,
                ),
            ],
        }
    }

    /// Creates a compute-oriented ABI for interference fields with history.
    #[must_use]
    pub fn compute_interference_v1(abi_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_KERNEL_ABI_SCHEMA_ID.to_owned(),
            abi_id: abi_id.into(),
            parameter_layout_version: 1,
            preferred_execution_model: KernelExecutionModel::ComputeFieldTexture,
            supports_fragment: true,
            supports_compute: true,
            requires_history_buffer: true,
            requires_noise_texture: true,
            max_layers: 16,
            bounded_readback_samples: 512,
            portability: ComputePortabilityProfile::mobile_vulkan_portable(
                "stimulus.portability.mobile_vulkan",
            ),
            compute_passes: vec![
                StimulusComputePassDescriptor::noise_cache(
                    "stimulus.compute_pass.noise_cache",
                    512,
                    512,
                ),
                StimulusComputePassDescriptor::field_texture(
                    "stimulus.compute_pass.interference_field",
                    1024,
                    1024,
                ),
                StimulusComputePassDescriptor::history_feedback(
                    "stimulus.compute_pass.history_feedback",
                    1024,
                    1024,
                ),
                StimulusComputePassDescriptor::readback_probe(
                    "stimulus.compute_pass.readback_probe",
                    512,
                ),
            ],
        }
    }

    /// Creates a compute ABI for the first bounded stimulus volume proof.
    #[must_use]
    pub fn volume_compute_v1(abi_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_KERNEL_ABI_SCHEMA_ID.to_owned(),
            abi_id: abi_id.into(),
            parameter_layout_version: 2,
            preferred_execution_model: KernelExecutionModel::ComputeFieldTexture,
            supports_fragment: true,
            supports_compute: true,
            requires_history_buffer: false,
            requires_noise_texture: true,
            max_layers: 16,
            bounded_readback_samples: 512,
            portability: ComputePortabilityProfile::mobile_vulkan_portable(
                "stimulus.portability.mobile_vulkan",
            ),
            compute_passes: vec![
                StimulusComputePassDescriptor::volume_density_cache(
                    "stimulus.compute_pass.volume_density_cache",
                    32,
                    32,
                ),
                StimulusComputePassDescriptor::volume_readback_probe(
                    "stimulus.compute_pass.volume_probe",
                    512,
                ),
                StimulusComputePassDescriptor::volume_stereo_field(
                    "stimulus.compute_pass.volume_stereo_field",
                    512,
                    512,
                ),
            ],
        }
    }

    /// Validates ABI descriptor shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != STIMULUS_KERNEL_ABI_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: STIMULUS_KERNEL_ABI_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.abi_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("abi_id"));
        }
        if self.parameter_layout_version == 0 {
            return Err(OpticsError::InvalidValue("parameter_layout_version"));
        }
        if !self.supports_fragment && !self.supports_compute {
            return Err(OpticsError::InvalidValue("execution_support"));
        }
        if self.max_layers == 0 || self.max_layers > 32 {
            return Err(OpticsError::InvalidCount("max_layers"));
        }
        if self.bounded_readback_samples > 4096 {
            return Err(OpticsError::InvalidCount("bounded_readback_samples"));
        }
        self.portability.validate()?;
        if !self.supports_compute && !self.compute_passes.is_empty() {
            return Err(OpticsError::InvalidValue("compute_passes"));
        }
        if self.compute_passes.len() > 8 {
            return Err(OpticsError::InvalidCount("compute_passes"));
        }
        for pass in &self.compute_passes {
            pass.validate()?;
            if pass.workgroup_size.invocation_count() > self.portability.max_workgroup_invocations {
                return Err(OpticsError::InvalidValue(
                    "compute_pass.workgroup_invocations",
                ));
            }
            if pass.output_width_px > self.portability.max_resource_dimension_px
                || pass.output_height_px > self.portability.max_resource_dimension_px
            {
                return Err(OpticsError::InvalidValue("compute_pass.output_extent"));
            }
            if pass.reads_noise_texture && !self.requires_noise_texture {
                return Err(OpticsError::InvalidValue("compute_pass.noise_texture"));
            }
            if pass.bounded_readback_samples > self.bounded_readback_samples {
                return Err(OpticsError::InvalidCount(
                    "compute_pass.bounded_readback_samples",
                ));
            }
            if (pass.reads_history_buffer || pass.writes_history_buffer)
                && !self.requires_history_buffer
            {
                return Err(OpticsError::InvalidValue("compute_pass.history_buffer"));
            }
        }
        Ok(())
    }
}
