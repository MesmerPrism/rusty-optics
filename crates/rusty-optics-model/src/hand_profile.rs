use crate::{
    ColorRgba, OpticsError, HAND_MESH_VISUAL_PROFILE_SCHEMA_ID, MESH_BROWSER_DEBUG_FRAME_SCHEMA_ID,
};

/// Renderer-neutral visual profile for hand mesh debug presentation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct HandMeshVisualProfile {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable profile identifier.
    pub profile_id: String,
    /// Source visual frame schema this profile is intended to style.
    pub source_visual_schema_id: String,
    /// Human-readable intent label for tools and reviewers.
    pub visual_intent: String,
    /// Suggested mesh edge color.
    pub mesh_edge_color: ColorRgba,
    /// Suggested coordinate axis scale in source units.
    pub coordinate_axis_scale: f32,
    /// Suggested collider shell color.
    pub collider_shell_color: ColorRgba,
    /// Suggested contact marker color.
    pub contact_marker_color: ColorRgba,
    /// Suggested SDF slice color.
    pub sdf_slice_color: ColorRgba,
    /// Whole-profile opacity multiplier.
    pub opacity: f32,
}

impl HandMeshVisualProfile {
    /// Creates a browser-debug hand mesh visual profile.
    #[must_use]
    pub fn browser_debug(profile_id: impl Into<String>) -> Self {
        Self {
            schema_id: HAND_MESH_VISUAL_PROFILE_SCHEMA_ID.to_owned(),
            profile_id: profile_id.into(),
            source_visual_schema_id: MESH_BROWSER_DEBUG_FRAME_SCHEMA_ID.to_owned(),
            visual_intent: "hand_mesh_browser_debug".to_owned(),
            mesh_edge_color: ColorRgba::new(0.74, 0.80, 0.86, 1.0),
            coordinate_axis_scale: 0.055,
            collider_shell_color: ColorRgba::new(0.25, 0.62, 0.92, 0.45),
            contact_marker_color: ColorRgba::new(1.0, 0.76, 0.26, 1.0),
            sdf_slice_color: ColorRgba::new(0.44, 0.86, 0.62, 0.52),
            opacity: 1.0,
        }
    }

    /// Validates visual profile shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != HAND_MESH_VISUAL_PROFILE_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: HAND_MESH_VISUAL_PROFILE_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.profile_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("profile_id"));
        }
        if self.source_visual_schema_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_visual_schema_id"));
        }
        if self.visual_intent.trim().is_empty() {
            return Err(OpticsError::EmptyId("visual_intent"));
        }
        if !self.mesh_edge_color.is_finite() {
            return Err(OpticsError::NonFiniteColor("mesh_edge_color"));
        }
        if !self.collider_shell_color.is_finite() {
            return Err(OpticsError::NonFiniteColor("collider_shell_color"));
        }
        if !self.contact_marker_color.is_finite() {
            return Err(OpticsError::NonFiniteColor("contact_marker_color"));
        }
        if !self.sdf_slice_color.is_finite() {
            return Err(OpticsError::NonFiniteColor("sdf_slice_color"));
        }
        if !self.coordinate_axis_scale.is_finite() || self.coordinate_axis_scale <= 0.0 {
            return Err(OpticsError::InvalidValue("coordinate_axis_scale"));
        }
        if !self.opacity.is_finite() || !(0.0..=1.0).contains(&self.opacity) {
            return Err(OpticsError::InvalidValue("opacity"));
        }
        Ok(())
    }
}
