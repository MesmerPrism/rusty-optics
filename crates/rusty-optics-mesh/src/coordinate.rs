use rusty_matter_mesh::MeshCoordinateMap;
use rusty_optics_model::{ColorRgba, OpticsError, MESH_COORDINATE_VISUAL_SCHEMA_ID};

use crate::{MeshDebugLine, MeshDebugLineRole, MeshDebugPoint, MeshDebugPointRole};

/// Renderer-neutral coordinate-map debug visualization.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct MeshCoordinateVisual {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable visual identifier.
    pub visual_id: String,
    /// Source Matter coordinate-map identifier.
    pub source_coordinate_map_id: String,
    /// Source Matter surface identifier.
    pub source_surface_id: String,
    /// Source topology index hash.
    pub topology_index_hash: u64,
    /// Anchor markers.
    pub anchors: Vec<MeshDebugPoint>,
    /// Local X/Y/Z axis line segments.
    pub axes: Vec<MeshDebugLine>,
}

impl MeshCoordinateVisual {
    /// Builds a coordinate-map visual from Matter mesh coordinates.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the source map or generated visual is
    /// invalid.
    pub fn from_coordinate_map(
        visual_id: impl Into<String>,
        map: &MeshCoordinateMap,
        axis_length: f32,
    ) -> Result<Self, OpticsError> {
        if axis_length <= 0.0 || !axis_length.is_finite() {
            return Err(OpticsError::InvalidValue("axis_length"));
        }
        let anchors = map
            .frames
            .frames
            .iter()
            .enumerate()
            .map(|(index, frame)| {
                MeshDebugPoint::new(
                    format!("{}.anchor.{index:04}", map.coordinate_map_id),
                    frame.anchor,
                    axis_length * 0.24,
                    MeshDebugPointRole::CoordinateAnchor,
                    ColorRgba::new(0.95, 0.95, 0.98, 1.0),
                )
            })
            .collect::<Vec<_>>();
        let mut axes = Vec::with_capacity(map.frames.frames.len() * 3);
        for frame in &map.frames.frames {
            axes.push(MeshDebugLine::new(
                frame.anchor,
                frame.anchor + frame.axis_x * axis_length,
                MeshDebugLineRole::CoordinateAxisX,
                ColorRgba::new(0.95, 0.24, 0.24, 1.0),
            ));
            axes.push(MeshDebugLine::new(
                frame.anchor,
                frame.anchor + frame.axis_y * axis_length,
                MeshDebugLineRole::CoordinateAxisY,
                ColorRgba::new(0.18, 0.78, 0.36, 1.0),
            ));
            axes.push(MeshDebugLine::new(
                frame.anchor,
                frame.anchor + frame.axis_z * axis_length,
                MeshDebugLineRole::CoordinateAxisZ,
                ColorRgba::new(0.26, 0.50, 0.95, 1.0),
            ));
        }
        let visual = Self {
            schema_id: MESH_COORDINATE_VISUAL_SCHEMA_ID.to_owned(),
            visual_id: visual_id.into(),
            source_coordinate_map_id: map.coordinate_map_id.clone(),
            source_surface_id: map.samples.surface_id.clone(),
            topology_index_hash: map.topology_key.index_hash,
            anchors,
            axes,
        };
        visual.validate()?;
        Ok(visual)
    }

    /// Validates coordinate-map visual shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != MESH_COORDINATE_VISUAL_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: MESH_COORDINATE_VISUAL_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.visual_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("visual_id"));
        }
        if self.source_coordinate_map_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_coordinate_map_id"));
        }
        if self.source_surface_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_surface_id"));
        }
        if self.anchors.is_empty() {
            return Err(OpticsError::InvalidCount("anchors"));
        }
        if self.axes.len() != self.anchors.len() * 3 {
            return Err(OpticsError::InvalidCount("axes"));
        }
        for anchor in &self.anchors {
            anchor.validate()?;
        }
        for axis in &self.axes {
            axis.validate()?;
        }
        Ok(())
    }
}
