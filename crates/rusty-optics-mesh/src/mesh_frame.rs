use std::collections::BTreeSet;

use rusty_matter_mesh::TriangleMeshSurface;
use rusty_matter_model::Vec3;
use rusty_optics_model::{ColorRgba, OpticsError, MESH_DEBUG_FRAME_SCHEMA_ID};

/// Renderer-neutral line role for mesh diagnostic payloads.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeshDebugLineRole {
    /// Edge from the source triangle mesh.
    SurfaceEdge,
    /// Edge from a diagnostic collider shell.
    ColliderShell,
    /// Local coordinate X tangent axis.
    CoordinateAxisX,
    /// Local coordinate Y tangent axis.
    CoordinateAxisY,
    /// Local coordinate Z normal axis.
    CoordinateAxisZ,
    /// Contact normal line from collider diagnostics.
    ContactNormal,
}

/// Renderer-neutral point role for mesh diagnostic payloads.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeshDebugPointRole {
    /// Local coordinate anchor on the mesh.
    CoordinateAnchor,
    /// Closest-point contact on a collider surface.
    ColliderContact,
}

/// One debug vertex copied from a Matter mesh surface.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct MeshDebugVertex {
    /// Source vertex index.
    pub index: usize,
    /// Vertex position in source coordinates.
    pub position: Vec3,
}

/// One debug line segment.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct MeshDebugLine {
    /// Start point in source coordinates.
    pub start: Vec3,
    /// End point in source coordinates.
    pub end: Vec3,
    /// Semantic line role.
    pub role: MeshDebugLineRole,
    /// Suggested debug color.
    pub color: ColorRgba,
}

impl MeshDebugLine {
    /// Creates a debug line.
    #[must_use]
    pub const fn new(start: Vec3, end: Vec3, role: MeshDebugLineRole, color: ColorRgba) -> Self {
        Self {
            start,
            end,
            role,
            color,
        }
    }

    /// Validates line shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when points or color are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if !self.start.is_finite() {
            return Err(OpticsError::NonFiniteVec3("line.start"));
        }
        if !self.end.is_finite() {
            return Err(OpticsError::NonFiniteVec3("line.end"));
        }
        if self.start.distance_squared(self.end) <= 1.0e-12 {
            return Err(OpticsError::InvalidValue("line length"));
        }
        if !self.color.is_finite() {
            return Err(OpticsError::NonFiniteColor("line.color"));
        }
        Ok(())
    }
}

/// One debug point marker.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct MeshDebugPoint {
    /// Stable marker identifier.
    pub point_id: String,
    /// Point position in source coordinates.
    pub position: Vec3,
    /// Suggested marker radius in source units.
    pub radius: f32,
    /// Semantic point role.
    pub role: MeshDebugPointRole,
    /// Suggested debug color.
    pub color: ColorRgba,
}

impl MeshDebugPoint {
    /// Creates a debug point.
    #[must_use]
    pub fn new(
        point_id: impl Into<String>,
        position: Vec3,
        radius: f32,
        role: MeshDebugPointRole,
        color: ColorRgba,
    ) -> Self {
        Self {
            point_id: point_id.into(),
            position,
            radius,
            role,
            color,
        }
    }

    /// Validates point shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.point_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("point_id"));
        }
        if !self.position.is_finite() {
            return Err(OpticsError::NonFiniteVec3("point.position"));
        }
        if !self.radius.is_finite() || self.radius < 0.0 {
            return Err(OpticsError::InvalidValue("point.radius"));
        }
        if !self.color.is_finite() {
            return Err(OpticsError::NonFiniteColor("point.color"));
        }
        Ok(())
    }
}

/// Renderer-neutral mesh wireframe and topology debug payload.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct MeshDebugFrame {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable debug frame identifier.
    pub frame_id: String,
    /// Source Matter surface identifier.
    pub source_surface_id: String,
    /// Source Matter surface schema identifier.
    pub source_schema_id: String,
    /// Source topology index hash.
    pub topology_index_hash: u64,
    /// Source vertices.
    pub vertices: Vec<MeshDebugVertex>,
    /// Source triangle indices.
    pub triangles: Vec<[u32; 3]>,
    /// Unique wireframe edges.
    pub edges: Vec<MeshDebugLine>,
    /// Minimum position bound.
    pub bounds_min: Vec3,
    /// Maximum position bound.
    pub bounds_max: Vec3,
}

impl MeshDebugFrame {
    /// Builds a mesh debug frame from a Matter triangle mesh surface.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the source surface or generated frame is
    /// invalid.
    pub fn from_surface(
        frame_id: impl Into<String>,
        surface: &TriangleMeshSurface,
    ) -> Result<Self, OpticsError> {
        surface
            .validate()
            .map_err(|_| OpticsError::InvalidPayload("source mesh surface is invalid"))?;
        let (bounds_min, bounds_max) = bounds_for_positions(&surface.positions)?;
        let frame = Self {
            schema_id: MESH_DEBUG_FRAME_SCHEMA_ID.to_owned(),
            frame_id: frame_id.into(),
            source_surface_id: surface.surface_id.clone(),
            source_schema_id: surface.schema_id.clone(),
            topology_index_hash: surface.topology_key().index_hash,
            vertices: surface
                .positions
                .iter()
                .copied()
                .enumerate()
                .map(|(index, position)| MeshDebugVertex { index, position })
                .collect(),
            triangles: surface.triangles.clone(),
            edges: mesh_debug_lines_from_surface_edges(
                surface,
                MeshDebugLineRole::SurfaceEdge,
                ColorRgba::new(0.74, 0.80, 0.86, 1.0),
            )?,
            bounds_min,
            bounds_max,
        };
        frame.validate()?;
        Ok(frame)
    }

    /// Validates mesh debug frame shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != MESH_DEBUG_FRAME_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: MESH_DEBUG_FRAME_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.frame_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("frame_id"));
        }
        if self.source_surface_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_surface_id"));
        }
        if self.source_schema_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_schema_id"));
        }
        if self.vertices.is_empty() {
            return Err(OpticsError::InvalidCount("vertices"));
        }
        if self.triangles.is_empty() {
            return Err(OpticsError::InvalidCount("triangles"));
        }
        for vertex in &self.vertices {
            if vertex.index >= self.vertices.len() {
                return Err(OpticsError::InvalidValue("vertex.index"));
            }
            if !vertex.position.is_finite() {
                return Err(OpticsError::NonFiniteVec3("vertex.position"));
            }
        }
        for triangle in &self.triangles {
            for index in triangle {
                let index = usize::try_from(*index)
                    .map_err(|_| OpticsError::InvalidValue("triangle.index"))?;
                if index >= self.vertices.len() {
                    return Err(OpticsError::InvalidValue("triangle.index"));
                }
            }
        }
        for edge in &self.edges {
            edge.validate()?;
        }
        if !self.bounds_min.is_finite() {
            return Err(OpticsError::NonFiniteVec3("bounds_min"));
        }
        if !self.bounds_max.is_finite() {
            return Err(OpticsError::NonFiniteVec3("bounds_max"));
        }
        Ok(())
    }
}

/// Builds unique edge lines from a Matter mesh surface.
///
/// # Errors
///
/// Returns [`OpticsError`] when a triangle index is invalid.
pub fn mesh_debug_lines_from_surface_edges(
    surface: &TriangleMeshSurface,
    role: MeshDebugLineRole,
    color: ColorRgba,
) -> Result<Vec<MeshDebugLine>, OpticsError> {
    let mut seen = BTreeSet::<(u32, u32)>::new();
    let mut lines = Vec::new();
    for triangle in &surface.triangles {
        let [a, b, c] = *triangle;
        push_unique_edge_line(&mut seen, &mut lines, surface, a, b, role, color)?;
        push_unique_edge_line(&mut seen, &mut lines, surface, b, c, role, color)?;
        push_unique_edge_line(&mut seen, &mut lines, surface, c, a, role, color)?;
    }
    Ok(lines)
}

fn push_unique_edge_line(
    seen: &mut BTreeSet<(u32, u32)>,
    lines: &mut Vec<MeshDebugLine>,
    surface: &TriangleMeshSurface,
    a: u32,
    b: u32,
    role: MeshDebugLineRole,
    color: ColorRgba,
) -> Result<(), OpticsError> {
    let key = if a <= b { (a, b) } else { (b, a) };
    if !seen.insert(key) {
        return Ok(());
    }
    let a_index = usize::try_from(a).map_err(|_| OpticsError::InvalidValue("edge.index"))?;
    let b_index = usize::try_from(b).map_err(|_| OpticsError::InvalidValue("edge.index"))?;
    let start = *surface
        .positions
        .get(a_index)
        .ok_or(OpticsError::InvalidValue("edge.index"))?;
    let end = *surface
        .positions
        .get(b_index)
        .ok_or(OpticsError::InvalidValue("edge.index"))?;
    lines.push(MeshDebugLine::new(start, end, role, color));
    Ok(())
}

fn bounds_for_positions(positions: &[Vec3]) -> Result<(Vec3, Vec3), OpticsError> {
    let mut iter = positions.iter().copied();
    let Some(first) = iter.next() else {
        return Err(OpticsError::InvalidCount("positions"));
    };
    if !first.is_finite() {
        return Err(OpticsError::NonFiniteVec3("position"));
    }
    let mut min = first;
    let mut max = first;
    for position in iter {
        if !position.is_finite() {
            return Err(OpticsError::NonFiniteVec3("position"));
        }
        min = min.min(position);
        max = max.max(position);
    }
    Ok((min, max))
}
