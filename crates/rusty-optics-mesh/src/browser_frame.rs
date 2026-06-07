use rusty_optics_model::{OpticsError, MESH_BROWSER_DEBUG_FRAME_SCHEMA_ID};

use crate::{MeshColliderVisual, MeshCoordinateVisual, MeshDebugFrame, SdfSliceVisual};

/// Renderer-neutral mesh debug frame shaped for static browser previews.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct MeshBrowserDebugFrame {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable browser frame identifier.
    pub frame_id: String,
    /// Optional source frame identifier, such as a hand validation mesh frame.
    pub source_frame_id: String,
    /// Source Matter surface identifier.
    pub source_surface_id: String,
    /// Source Matter surface schema identifier.
    pub source_schema_id: String,
    /// Mesh topology and wireframe visual.
    pub mesh: MeshDebugFrame,
    /// Coordinate-map visual derived from the same surface.
    pub coordinates: MeshCoordinateVisual,
    /// Collider visual derived from the same surface.
    pub collider: MeshColliderVisual,
    /// SDF slice visual derived from the same surface snapshot.
    pub sdf_slice: SdfSliceVisual,
}

impl MeshBrowserDebugFrame {
    /// Creates a browser-ready debug frame.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the combined payload is inconsistent.
    pub fn new(
        frame_id: impl Into<String>,
        source_frame_id: impl Into<String>,
        mesh: MeshDebugFrame,
        coordinates: MeshCoordinateVisual,
        collider: MeshColliderVisual,
        sdf_slice: SdfSliceVisual,
    ) -> Result<Self, OpticsError> {
        let frame = Self {
            schema_id: MESH_BROWSER_DEBUG_FRAME_SCHEMA_ID.to_owned(),
            frame_id: frame_id.into(),
            source_frame_id: source_frame_id.into(),
            source_surface_id: mesh.source_surface_id.clone(),
            source_schema_id: mesh.source_schema_id.clone(),
            mesh,
            coordinates,
            collider,
            sdf_slice,
        };
        frame.validate()?;
        Ok(frame)
    }

    /// Validates the combined browser debug frame.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != MESH_BROWSER_DEBUG_FRAME_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: MESH_BROWSER_DEBUG_FRAME_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.frame_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("frame_id"));
        }
        if self.source_frame_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_frame_id"));
        }
        if self.source_surface_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_surface_id"));
        }
        if self.source_schema_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_schema_id"));
        }
        self.mesh.validate()?;
        self.coordinates.validate()?;
        self.collider.validate()?;
        self.sdf_slice.validate()?;
        if self.coordinates.source_surface_id != self.source_surface_id
            || self.collider.source_surface_id != self.source_surface_id
        {
            return Err(OpticsError::InvalidPayload(
                "mesh visuals must reference one source surface",
            ));
        }
        Ok(())
    }
}
