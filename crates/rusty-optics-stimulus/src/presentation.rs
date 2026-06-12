use rusty_optics_model::{OpticsError, STIMULUS_PRESENTATION_SCHEMA_ID};

/// Presentation family requested by a procedural stimulus profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StimulusPresentationMode {
    /// Full-field stereo stimulus submitted to both eye views.
    StereoEyeField,
    /// Full-field mono stimulus submitted to one eye view or a flat preview.
    MonoEyeField,
    /// Texture mapped onto a panel or quad in an XR/browser scene.
    SurfacePanel,
    /// Texture mapped onto a world-locked calibrated surface.
    WorldLockedSurface,
}

/// Intended coverage of the target view or surface.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StimulusCoverageMode {
    /// Cover the full target viewport or eye view.
    FullViewport,
    /// Cover the largest centered area that preserves source aspect.
    AspectFit,
    /// Fill the target while preserving aspect, cropping as needed.
    AspectFill,
}

/// How one or more generated textures are bound to stereo views.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StereoFieldBinding {
    /// One generated field is sampled by both eyes.
    SharedFieldBothEyes,
    /// One generated field is stored as two array layers, one layer per eye.
    StereoArrayLayers,
    /// Separate generated fields are supplied for left and right eyes.
    SeparateEyeFields,
}

/// Runtime reference-space ownership expected by the presentation adapter.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationReferenceSpace {
    /// Presentation is locked to the user's eye views.
    ViewLocked,
    /// Presentation is attached to a local XR reference space.
    LocalSpace,
    /// Presentation is attached to a world or stage reference space.
    WorldSpace,
    /// Presentation is attached to a calibrated target surface.
    SurfaceAnchored,
}

/// Renderer-neutral XR/browser presentation target descriptor.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StimulusPresentationDescriptor {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable presentation identifier.
    pub presentation_id: String,
    /// Presentation family.
    pub mode: StimulusPresentationMode,
    /// Intended target coverage.
    pub coverage: StimulusCoverageMode,
    /// Stereo texture binding policy.
    pub stereo_binding: StereoFieldBinding,
    /// Reference-space role consumed from Lattice or the adapter shell.
    pub reference_space: PresentationReferenceSpace,
    /// Number of eye views expected by this profile.
    pub eye_count: u8,
    /// Whether per-eye UV/lens adjustment may be applied by the adapter.
    pub allow_per_eye_uv_transform: bool,
    /// Whether the presentation should occlude the underlying scene.
    pub require_opaque_background: bool,
    /// Whether the adapter should use an XR composition layer when available.
    pub prefer_xr_composition_layer: bool,
}

impl StimulusPresentationDescriptor {
    /// Creates the primary full-screen stereo-eye presentation target.
    #[must_use]
    pub fn stereo_eye_field(presentation_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_PRESENTATION_SCHEMA_ID.to_owned(),
            presentation_id: presentation_id.into(),
            mode: StimulusPresentationMode::StereoEyeField,
            coverage: StimulusCoverageMode::FullViewport,
            stereo_binding: StereoFieldBinding::SharedFieldBothEyes,
            reference_space: PresentationReferenceSpace::ViewLocked,
            eye_count: 2,
            allow_per_eye_uv_transform: true,
            require_opaque_background: true,
            prefer_xr_composition_layer: true,
        }
    }

    /// Creates a surface/panel presentation target for browser or scene previews.
    #[must_use]
    pub fn surface_panel(presentation_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_PRESENTATION_SCHEMA_ID.to_owned(),
            presentation_id: presentation_id.into(),
            mode: StimulusPresentationMode::SurfacePanel,
            coverage: StimulusCoverageMode::AspectFit,
            stereo_binding: StereoFieldBinding::SharedFieldBothEyes,
            reference_space: PresentationReferenceSpace::LocalSpace,
            eye_count: 2,
            allow_per_eye_uv_transform: false,
            require_opaque_background: false,
            prefer_xr_composition_layer: false,
        }
    }

    /// Validates presentation target shape and cross-field policy.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != STIMULUS_PRESENTATION_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: STIMULUS_PRESENTATION_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.presentation_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("presentation_id"));
        }
        if self.eye_count == 0 || self.eye_count > 2 {
            return Err(OpticsError::InvalidValue("presentation.eye_count"));
        }
        match self.mode {
            StimulusPresentationMode::StereoEyeField => {
                if self.eye_count != 2 {
                    return Err(OpticsError::InvalidValue("presentation.eye_count"));
                }
                if self.coverage != StimulusCoverageMode::FullViewport {
                    return Err(OpticsError::InvalidValue("presentation.coverage"));
                }
                if self.reference_space != PresentationReferenceSpace::ViewLocked {
                    return Err(OpticsError::InvalidValue("presentation.reference_space"));
                }
            }
            StimulusPresentationMode::MonoEyeField => {
                if self.eye_count != 1 {
                    return Err(OpticsError::InvalidValue("presentation.eye_count"));
                }
                if self.stereo_binding != StereoFieldBinding::SharedFieldBothEyes {
                    return Err(OpticsError::InvalidValue("presentation.stereo_binding"));
                }
            }
            StimulusPresentationMode::SurfacePanel
            | StimulusPresentationMode::WorldLockedSurface => {
                if self.prefer_xr_composition_layer {
                    return Err(OpticsError::InvalidValue(
                        "presentation.prefer_xr_composition_layer",
                    ));
                }
            }
        }
        Ok(())
    }
}
