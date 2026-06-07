use rusty_matter_model::Vec3;
use rusty_matter_sdf::{PackedSdfGrid, PACKED_SDF_GRID_SCHEMA_ID};
use rusty_optics_model::{OpticsError, SDF_SLICE_VISUAL_SCHEMA_ID};

/// Axis used for a two-dimensional SDF debug slice.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SdfSliceAxis {
    /// Slice at a fixed X index.
    X,
    /// Slice at a fixed Y index.
    Y,
    /// Slice at a fixed Z index.
    Z,
}

/// One sampled SDF slice cell.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SdfSliceCell {
    /// Grid coordinate `[x, y, z]`.
    pub grid: [u32; 3],
    /// Plane coordinate `[u, v]`.
    pub plane: [u32; 2],
    /// Cell center in source coordinates.
    pub position: Vec3,
    /// Signed or unsigned distance from the source grid.
    pub distance: f32,
    /// Distance normalized over this slice.
    pub normalized_distance: f32,
}

/// Renderer-neutral SDF slice visualization payload.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SdfSliceVisual {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable visual identifier.
    pub visual_id: String,
    /// Source Matter SDF grid identifier.
    pub source_grid_id: String,
    /// Source Matter SDF grid schema identifier.
    pub source_schema_id: String,
    /// Fixed slice axis.
    pub axis: SdfSliceAxis,
    /// Fixed slice index along `axis`.
    pub slice_index: u32,
    /// Slice width in cells.
    pub width: u32,
    /// Slice height in cells.
    pub height: u32,
    /// Minimum distance in the slice.
    pub min_distance: f32,
    /// Maximum distance in the slice.
    pub max_distance: f32,
    /// Slice cells.
    pub cells: Vec<SdfSliceCell>,
}

impl SdfSliceVisual {
    /// Builds a middle-Z SDF slice.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the grid or generated visual is invalid.
    pub fn middle_z(
        visual_id: impl Into<String>,
        grid: &PackedSdfGrid,
    ) -> Result<Self, OpticsError> {
        let slice_index = grid.dimensions[2] / 2;
        Self::from_grid_slice(visual_id, grid, SdfSliceAxis::Z, slice_index)
    }

    /// Builds an SDF slice on the requested axis.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the grid or generated visual is invalid.
    pub fn from_grid_slice(
        visual_id: impl Into<String>,
        grid: &PackedSdfGrid,
        axis: SdfSliceAxis,
        slice_index: u32,
    ) -> Result<Self, OpticsError> {
        grid.validate()
            .map_err(|_| OpticsError::InvalidPayload("source SDF grid is invalid"))?;
        if grid.schema_id != PACKED_SDF_GRID_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PACKED_SDF_GRID_SCHEMA_ID,
                actual: grid.schema_id.clone(),
            });
        }
        let [dim_x, dim_y, dim_z] = grid.dimensions;
        let limit = match axis {
            SdfSliceAxis::X => dim_x,
            SdfSliceAxis::Y => dim_y,
            SdfSliceAxis::Z => dim_z,
        };
        if slice_index >= limit {
            return Err(OpticsError::InvalidValue("slice_index"));
        }

        let (width, height) = match axis {
            SdfSliceAxis::X => (dim_y, dim_z),
            SdfSliceAxis::Y => (dim_x, dim_z),
            SdfSliceAxis::Z => (dim_x, dim_y),
        };
        let mut cells = Vec::with_capacity(
            usize::try_from(width)
                .ok()
                .and_then(|w| usize::try_from(height).ok().and_then(|h| w.checked_mul(h)))
                .ok_or(OpticsError::InvalidCount("sdf cells"))?,
        );
        let mut min_distance = f32::INFINITY;
        let mut max_distance = f32::NEG_INFINITY;
        for v in 0..height {
            for u in 0..width {
                let grid_coord = match axis {
                    SdfSliceAxis::X => [slice_index, u, v],
                    SdfSliceAxis::Y => [u, slice_index, v],
                    SdfSliceAxis::Z => [u, v, slice_index],
                };
                let distance = grid
                    .distance_at(grid_coord[0], grid_coord[1], grid_coord[2])
                    .ok_or(OpticsError::InvalidValue("sdf cell"))?;
                min_distance = min_distance.min(distance);
                max_distance = max_distance.max(distance);
                cells.push(SdfSliceCell {
                    grid: grid_coord,
                    plane: [u, v],
                    position: grid_cell_center(grid, grid_coord),
                    distance,
                    normalized_distance: 0.5,
                });
            }
        }
        let range = max_distance - min_distance;
        for cell in &mut cells {
            cell.normalized_distance = if range.abs() <= f32::EPSILON {
                0.5
            } else {
                ((cell.distance - min_distance) / range).clamp(0.0, 1.0)
            };
        }

        let visual = Self {
            schema_id: SDF_SLICE_VISUAL_SCHEMA_ID.to_owned(),
            visual_id: visual_id.into(),
            source_grid_id: grid.grid_id.clone(),
            source_schema_id: grid.schema_id.clone(),
            axis,
            slice_index,
            width,
            height,
            min_distance,
            max_distance,
            cells,
        };
        visual.validate()?;
        Ok(visual)
    }

    /// Validates SDF slice visual shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != SDF_SLICE_VISUAL_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: SDF_SLICE_VISUAL_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.visual_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("visual_id"));
        }
        if self.source_grid_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_grid_id"));
        }
        if self.source_schema_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_schema_id"));
        }
        if self.width == 0 || self.height == 0 {
            return Err(OpticsError::InvalidCount("sdf slice dimensions"));
        }
        let expected = usize::try_from(self.width)
            .ok()
            .and_then(|w| {
                usize::try_from(self.height)
                    .ok()
                    .and_then(|h| w.checked_mul(h))
            })
            .ok_or(OpticsError::InvalidCount("sdf cells"))?;
        if self.cells.len() != expected {
            return Err(OpticsError::InvalidCount("sdf cells"));
        }
        if !self.min_distance.is_finite() || !self.max_distance.is_finite() {
            return Err(OpticsError::InvalidValue("sdf distance bounds"));
        }
        for cell in &self.cells {
            if !cell.position.is_finite() {
                return Err(OpticsError::NonFiniteVec3("sdf cell.position"));
            }
            if !cell.distance.is_finite()
                || !cell.normalized_distance.is_finite()
                || !(0.0..=1.0).contains(&cell.normalized_distance)
            {
                return Err(OpticsError::InvalidValue("sdf cell.distance"));
            }
        }
        Ok(())
    }
}

fn grid_cell_center(grid: &PackedSdfGrid, cell: [u32; 3]) -> Vec3 {
    grid.origin
        + Vec3::new(
            (cell[0] as f32 + 0.5) * grid.voxel_size,
            (cell[1] as f32 + 0.5) * grid.voxel_size,
            (cell[2] as f32 + 0.5) * grid.voxel_size,
        )
}
