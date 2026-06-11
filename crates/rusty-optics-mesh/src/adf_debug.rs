use rusty_matter_adf::{AdaptiveDistanceField, ADAPTIVE_DISTANCE_FIELD_SCHEMA_ID};
use rusty_matter_model::Vec3;
use rusty_optics_model::{OpticsError, ADF_DEBUG_VISUAL_SCHEMA_ID};

/// One renderer-neutral ADF leaf-cell debug sample.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct AdfDebugCell {
    /// Leaf cell index in the source ADF field.
    pub cell_index: usize,
    /// Leaf subdivision level.
    pub level: u32,
    /// Minimum corner of the leaf cell.
    pub origin: Vec3,
    /// Center point of the leaf cell.
    pub center: Vec3,
    /// Cubic leaf-cell extent.
    pub extent: f32,
    /// Distance stored at the leaf-cell center.
    pub center_distance: f32,
    /// Minimum source SDF sample distance observed in the cell.
    pub min_distance: f32,
    /// Maximum source SDF sample distance observed in the cell.
    pub max_distance: f32,
    /// Center distance normalized over the full debug visual range.
    pub normalized_center_distance: f32,
    /// Per-cell distance range normalized over the full debug visual range.
    pub normalized_range: f32,
    /// Number of source SDF samples represented by this cell.
    pub source_sample_count: usize,
}

/// Renderer-neutral ADF debug visualization payload.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct AdfDebugVisual {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable visual identifier.
    pub visual_id: String,
    /// Source Matter ADF field identifier.
    pub source_field_id: String,
    /// Source Matter ADF schema identifier.
    pub source_schema_id: String,
    /// Source Matter SDF grid identifier.
    pub source_grid_id: String,
    /// Root ADF origin.
    pub root_origin: Vec3,
    /// Root cubic extent.
    pub root_extent: f32,
    /// Maximum source ADF subdivision depth.
    pub max_depth: u32,
    /// Minimum cell distance in this visual.
    pub min_distance: f32,
    /// Maximum cell distance in this visual.
    pub max_distance: f32,
    /// Maximum emitted leaf-cell level.
    pub max_level: u32,
    /// Leaf-cell count in this visual.
    pub cell_count: usize,
    /// Leaf-cell debug samples.
    pub cells: Vec<AdfDebugCell>,
}

impl AdfDebugVisual {
    /// Builds a renderer-neutral debug visual from a Matter ADF field.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the source field or generated visual is
    /// invalid.
    pub fn from_field(
        visual_id: impl Into<String>,
        field: &AdaptiveDistanceField,
    ) -> Result<Self, OpticsError> {
        field
            .validate()
            .map_err(|_| OpticsError::InvalidPayload("source ADF field is invalid"))?;
        if field.schema_id != ADAPTIVE_DISTANCE_FIELD_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: ADAPTIVE_DISTANCE_FIELD_SCHEMA_ID,
                actual: field.schema_id.clone(),
            });
        }
        let (min_distance, max_distance) = field_distance_range(field)?;
        let range = max_distance - min_distance;
        let mut max_level = 0;
        let cells = field
            .cells
            .iter()
            .enumerate()
            .map(|(cell_index, cell)| {
                max_level = max_level.max(cell.level);
                AdfDebugCell {
                    cell_index,
                    level: cell.level,
                    origin: cell.origin,
                    center: cell.center(),
                    extent: cell.extent,
                    center_distance: cell.center_distance,
                    min_distance: cell.min_distance,
                    max_distance: cell.max_distance,
                    normalized_center_distance: normalize_distance(
                        cell.center_distance,
                        min_distance,
                        range,
                    ),
                    normalized_range: normalize_range(cell.max_distance - cell.min_distance, range),
                    source_sample_count: cell.source_sample_count,
                }
            })
            .collect::<Vec<_>>();
        let visual = Self {
            schema_id: ADF_DEBUG_VISUAL_SCHEMA_ID.to_owned(),
            visual_id: visual_id.into(),
            source_field_id: field.field_id.clone(),
            source_schema_id: field.schema_id.clone(),
            source_grid_id: field.source_grid_id.clone(),
            root_origin: field.origin,
            root_extent: field.extent,
            max_depth: field.max_depth,
            min_distance,
            max_distance,
            max_level,
            cell_count: cells.len(),
            cells,
        };
        visual.validate()?;
        Ok(visual)
    }

    /// Validates ADF debug visual shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != ADF_DEBUG_VISUAL_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: ADF_DEBUG_VISUAL_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.visual_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("visual_id"));
        }
        if self.source_field_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_field_id"));
        }
        if self.source_schema_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_schema_id"));
        }
        if self.source_grid_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_grid_id"));
        }
        if !self.root_origin.is_finite() {
            return Err(OpticsError::NonFiniteVec3("root_origin"));
        }
        if !self.root_extent.is_finite() || self.root_extent <= 0.0 {
            return Err(OpticsError::InvalidValue("root_extent"));
        }
        if self.max_level > self.max_depth {
            return Err(OpticsError::InvalidValue("max_level"));
        }
        if self.cell_count == 0 || self.cell_count != self.cells.len() {
            return Err(OpticsError::InvalidCount("adf cells"));
        }
        if !self.min_distance.is_finite() || !self.max_distance.is_finite() {
            return Err(OpticsError::InvalidValue("adf distance bounds"));
        }
        if self.min_distance > self.max_distance {
            return Err(OpticsError::InvalidValue("adf distance range"));
        }
        for cell in &self.cells {
            validate_cell(cell, self.max_depth)?;
        }
        Ok(())
    }
}

fn field_distance_range(field: &AdaptiveDistanceField) -> Result<(f32, f32), OpticsError> {
    let min_distance = field
        .cells
        .iter()
        .map(|cell| cell.min_distance)
        .reduce(f32::min)
        .ok_or(OpticsError::InvalidCount("adf cells"))?;
    let max_distance = field
        .cells
        .iter()
        .map(|cell| cell.max_distance)
        .reduce(f32::max)
        .ok_or(OpticsError::InvalidCount("adf cells"))?;
    Ok((min_distance, max_distance))
}

fn normalize_distance(distance: f32, min_distance: f32, range: f32) -> f32 {
    if range.abs() <= f32::EPSILON {
        0.5
    } else {
        ((distance - min_distance) / range).clamp(0.0, 1.0)
    }
}

fn normalize_range(cell_range: f32, visual_range: f32) -> f32 {
    if visual_range.abs() <= f32::EPSILON {
        0.0
    } else {
        (cell_range / visual_range).clamp(0.0, 1.0)
    }
}

fn validate_cell(cell: &AdfDebugCell, max_depth: u32) -> Result<(), OpticsError> {
    if cell.level > max_depth {
        return Err(OpticsError::InvalidValue("adf cell.level"));
    }
    if !cell.origin.is_finite() {
        return Err(OpticsError::NonFiniteVec3("adf cell.origin"));
    }
    if !cell.center.is_finite() {
        return Err(OpticsError::NonFiniteVec3("adf cell.center"));
    }
    if !cell.extent.is_finite() || cell.extent <= 0.0 {
        return Err(OpticsError::InvalidValue("adf cell.extent"));
    }
    if !cell.center_distance.is_finite()
        || !cell.min_distance.is_finite()
        || !cell.max_distance.is_finite()
    {
        return Err(OpticsError::InvalidValue("adf cell.distance"));
    }
    if cell.min_distance > cell.max_distance {
        return Err(OpticsError::InvalidValue("adf cell.distance_range"));
    }
    if !cell.normalized_center_distance.is_finite()
        || !(0.0..=1.0).contains(&cell.normalized_center_distance)
        || !cell.normalized_range.is_finite()
        || !(0.0..=1.0).contains(&cell.normalized_range)
    {
        return Err(OpticsError::InvalidValue("adf cell.normalized"));
    }
    Ok(())
}
