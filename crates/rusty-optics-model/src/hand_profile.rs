use crate::{
    ColorRgba, OpticsError, HAND_MESH_VISUAL_PROFILE_SCHEMA_ID,
    HAND_SUBSTRATE_VISUAL_PROFILE_SCHEMA_ID, MESH_BROWSER_DEBUG_FRAME_SCHEMA_ID,
};

/// Renderer-neutral visual profile for hand mesh debug presentation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
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

/// Logical hand identity preserved by an Optics visual profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandVisualSide {
    /// Left hand.
    Left,
    /// Right hand.
    Right,
}

/// Renderer-neutral visual profile bound to one provider, Lattice frame, and Matter rig.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[derive(Clone, Debug, PartialEq)]
pub struct HandSubstrateVisualProfile {
    /// Schema id.
    pub schema: String,
    /// Stable visual profile id.
    pub profile_id: String,
    /// Provider identity preserved from Lattice and Matter.
    pub provider_id: String,
    /// Lattice frame identity.
    pub lattice_frame_id: String,
    /// Matter rig identity.
    pub matter_rig_id: String,
    /// Logical hand identity.
    pub hand: HandVisualSide,
    /// Source Lattice frame schema.
    pub lattice_frame_schema_id: String,
    /// Source Matter rig schema.
    pub matter_rig_schema_id: String,
    /// Renderer-neutral visual intent.
    pub visual_intent: String,
    /// Surface fill color.
    pub surface_color: ColorRgba,
    /// Optional wireframe suggestion encoded as a neutral policy bit.
    pub wireframe: bool,
    /// Whole-profile opacity.
    pub opacity: f32,
}

impl HandSubstrateVisualProfile {
    /// Validate schema, identity preservation, and renderer-neutral appearance values.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema != HAND_SUBSTRATE_VISUAL_PROFILE_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: HAND_SUBSTRATE_VISUAL_PROFILE_SCHEMA_ID,
                actual: self.schema.clone(),
            });
        }
        for (name, value) in [
            ("profile_id", self.profile_id.as_str()),
            ("provider_id", self.provider_id.as_str()),
            ("lattice_frame_id", self.lattice_frame_id.as_str()),
            ("matter_rig_id", self.matter_rig_id.as_str()),
            ("visual_intent", self.visual_intent.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(OpticsError::EmptyId(name));
            }
        }
        if self.lattice_frame_schema_id != "rusty.lattice.hand_joint_frame.v1"
            || self.matter_rig_schema_id != "rusty.matter.hand.rig_capture.v1"
        {
            return Err(OpticsError::InvalidValue("hand source schema ids"));
        }
        if !self.surface_color.is_finite() {
            return Err(OpticsError::NonFiniteColor("surface_color"));
        }
        if !self.opacity.is_finite() || !(0.0..=1.0).contains(&self.opacity) {
            return Err(OpticsError::InvalidValue("opacity"));
        }
        Ok(())
    }

    /// Whether the visual profile still addresses the expected provider, frame, rig, and hand.
    #[must_use]
    pub fn matches_identity(
        &self,
        provider_id: &str,
        lattice_frame_id: &str,
        matter_rig_id: &str,
        hand: HandVisualSide,
    ) -> bool {
        self.provider_id == provider_id
            && self.lattice_frame_id == lattice_frame_id
            && self.matter_rig_id == matter_rig_id
            && self.hand == hand
    }
}

#[cfg(all(test, feature = "serde"))]
mod conformance_tests {
    use super::*;

    const VALID: &str =
        include_str!("../../../fixtures/hand_mesh/hand-substrate-visual-profile.json");
    const DAMAGE: &str =
        include_str!("../../../fixtures/damaged/hand-substrate-visual-backend-leak.json");

    #[test]
    fn substrate_profile_preserves_cross_lane_identity() {
        let profile: HandSubstrateVisualProfile = serde_json::from_str(VALID).unwrap();
        profile.validate().unwrap();
        assert!(profile.matches_identity(
            "generic-tracked-hand-provider",
            "generic-left-hand-joint-frame-0001",
            "hand.rig.generic-left.v1",
            HandVisualSide::Left,
        ));
        assert!(!profile.matches_identity(
            "different-provider",
            "generic-left-hand-joint-frame-0001",
            "hand.rig.generic-left.v1",
            HandVisualSide::Left,
        ));
    }

    #[test]
    fn backend_leak_fixture_fails_strict_deserialization() {
        let damage: serde_json::Value = serde_json::from_str(DAMAGE).unwrap();
        let damaged = VALID.replace(
            "\"wireframe\": true",
            &format!(
                "\"wireframe\": true, \"{}\": \"private-shader\"",
                damage["forbidden_field"].as_str().unwrap()
            ),
        );
        assert!(serde_json::from_str::<HandSubstrateVisualProfile>(&damaged).is_err());
    }
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
