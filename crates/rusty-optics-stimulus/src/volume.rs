use rusty_optics_model::{OpticsError, STIMULUS_VOLUME_SCHEMA_ID};

/// Volume-field family requested by a procedural stimulus profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StimulusVolumeFieldKind {
    /// Procedural 3D density derived from the existing stimulus layer stack.
    ProceduralLayerStack3d,
    /// Dense scalar density grid.
    DenseScalarGrid,
    /// Dense signed-distance grid owned by a future Matter payload.
    DenseSdfGrid,
    /// Indexed adaptive-distance field owned by a future Matter payload.
    IndexedAdfGrid,
}

impl StimulusVolumeFieldKind {
    /// Stable schema token for this volume field kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProceduralLayerStack3d => "ProceduralLayerStack3d",
            Self::DenseScalarGrid => "DenseScalarGrid",
            Self::DenseSdfGrid => "DenseSdfGrid",
            Self::IndexedAdfGrid => "IndexedAdfGrid",
        }
    }
}

/// Adapter storage preference for a stimulus volume.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StimulusVolumeStorageHint {
    /// Vec4-aligned storage-buffer records.
    StorageBuffer,
    /// Sampled 3D texture.
    SampledTexture3d,
    /// Writable 3D storage texture.
    StorageTexture3d,
    /// Sparse or bricked storage-buffer records.
    BrickedStorageBuffers,
}

impl StimulusVolumeStorageHint {
    /// Stable schema token for this volume storage hint.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StorageBuffer => "StorageBuffer",
            Self::SampledTexture3d => "SampledTexture3d",
            Self::StorageTexture3d => "StorageTexture3d",
            Self::BrickedStorageBuffers => "BrickedStorageBuffers",
        }
    }
}

/// Renderer-neutral volume-field descriptor for procedural stimuli.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StimulusVolumeDescriptor {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable volume identifier.
    pub volume_id: String,
    /// Semantic field family.
    pub field_kind: StimulusVolumeFieldKind,
    /// Preferred adapter storage.
    pub storage_hint: StimulusVolumeStorageHint,
    /// Volume minimum bounds in reference-local coordinates.
    pub bounds_min: [f32; 3],
    /// Volume maximum bounds in reference-local coordinates.
    pub bounds_max: [f32; 3],
    /// Nominal grid dimensions for cache and probe planning.
    pub grid_dimensions: [u32; 3],
    /// Density multiplier used by CPU reference and adapters.
    pub density_scale: f32,
    /// Opacity multiplier used by CPU reference and adapters.
    pub opacity_scale: f32,
    /// Maximum raymarch steps requested for the first proof.
    pub step_count: u32,
    /// Deterministic per-ray step jitter in `0..1`.
    pub step_jitter: f32,
    /// Whether adapters may use an empty-space skip structure.
    pub empty_space_skip_hint: bool,
}

impl StimulusVolumeDescriptor {
    /// Creates the default bounded procedural volume proof descriptor.
    #[must_use]
    pub fn procedural_layer_stack_3d(volume_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_VOLUME_SCHEMA_ID.to_owned(),
            volume_id: volume_id.into(),
            field_kind: StimulusVolumeFieldKind::ProceduralLayerStack3d,
            storage_hint: StimulusVolumeStorageHint::StorageBuffer,
            bounds_min: [-1.0, -1.0, -1.0],
            bounds_max: [1.0, 1.0, 1.0],
            grid_dimensions: [32, 32, 32],
            density_scale: 1.0,
            opacity_scale: 0.35,
            step_count: 32,
            step_jitter: 0.5,
            empty_space_skip_hint: false,
        }
    }

    /// Returns the nominal grid voxel count as a widened integer.
    #[must_use]
    pub fn voxel_count(&self) -> u64 {
        u64::from(self.grid_dimensions[0])
            .saturating_mul(u64::from(self.grid_dimensions[1]))
            .saturating_mul(u64::from(self.grid_dimensions[2]))
    }

    /// Validates descriptor shape and mobile-oriented bounds.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != STIMULUS_VOLUME_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: STIMULUS_VOLUME_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.volume_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("volume_id"));
        }
        validate_bounds(self.bounds_min, self.bounds_max)?;
        if self.grid_dimensions.contains(&0) || self.grid_dimensions.iter().any(|dim| *dim > 512) {
            return Err(OpticsError::InvalidValue("volume.grid_dimensions"));
        }
        if self.voxel_count() > 512_u64 * 512 * 512 {
            return Err(OpticsError::InvalidCount("volume.voxel_count"));
        }
        validate_non_negative("volume.density_scale", self.density_scale)?;
        validate_non_negative("volume.opacity_scale", self.opacity_scale)?;
        if self.density_scale > 64.0 || self.opacity_scale > 64.0 {
            return Err(OpticsError::InvalidValue("volume.scale"));
        }
        if self.step_count == 0 || self.step_count > 256 {
            return Err(OpticsError::InvalidCount("volume.step_count"));
        }
        validate_unit("volume.step_jitter", self.step_jitter)?;
        if self.field_kind == StimulusVolumeFieldKind::IndexedAdfGrid
            && self.storage_hint == StimulusVolumeStorageHint::StorageTexture3d
        {
            return Err(OpticsError::InvalidValue("volume.storage_hint"));
        }
        Ok(())
    }
}

fn validate_bounds(bounds_min: [f32; 3], bounds_max: [f32; 3]) -> Result<(), OpticsError> {
    if !bounds_min
        .iter()
        .chain(bounds_max.iter())
        .all(|value| value.is_finite())
    {
        return Err(OpticsError::NonFiniteVec3("volume.bounds"));
    }
    for axis in 0..3 {
        if bounds_max[axis] <= bounds_min[axis] {
            return Err(OpticsError::InvalidValue("volume.bounds"));
        }
    }
    Ok(())
}

fn validate_non_negative(field: &'static str, value: f32) -> Result<(), OpticsError> {
    if value.is_finite() && value >= 0.0 {
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
