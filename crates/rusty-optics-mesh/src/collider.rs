use rusty_matter_mesh::{
    DynamicMeshColliderContact, DynamicMeshColliderShell, DynamicMeshColliderUpdate,
    DynamicMeshColliderUpdateStatus, DYNAMIC_MESH_COLLIDER_SHELL_SCHEMA_ID,
    DYNAMIC_MESH_COLLIDER_UPDATE_SCHEMA_ID,
};
use rusty_optics_model::{ColorRgba, OpticsError, MESH_COLLIDER_VISUAL_SCHEMA_ID};

use crate::{
    mesh_debug_lines_from_surface_edges, MeshDebugLine, MeshDebugLineRole, MeshDebugPoint,
    MeshDebugPointRole,
};

/// Renderer-neutral dynamic mesh collider diagnostic visualization.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct MeshColliderVisual {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable visual identifier.
    pub visual_id: String,
    /// Source Matter surface identifier.
    pub source_surface_id: String,
    /// Collider update status label.
    pub update_status: String,
    /// Collider surface vertex count.
    pub collider_vertex_count: usize,
    /// Collider surface triangle count.
    pub collider_triangle_count: usize,
    /// Diagnostic shell edge lines.
    pub shell_edges: Vec<MeshDebugLine>,
    /// Closest-point contact markers.
    pub contact_points: Vec<MeshDebugPoint>,
    /// Contact normal line segments.
    pub contact_normals: Vec<MeshDebugLine>,
}

impl MeshColliderVisual {
    /// Builds a collider diagnostic visual from Matter collider output.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the source payloads or generated visual are
    /// invalid.
    pub fn from_collider_payload(
        visual_id: impl Into<String>,
        source_surface_id: impl Into<String>,
        update: &DynamicMeshColliderUpdate,
        shell: Option<&DynamicMeshColliderShell>,
        contact: Option<&DynamicMeshColliderContact>,
    ) -> Result<Self, OpticsError> {
        if update.schema_id != DYNAMIC_MESH_COLLIDER_UPDATE_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: DYNAMIC_MESH_COLLIDER_UPDATE_SCHEMA_ID,
                actual: update.schema_id.clone(),
            });
        }
        let mut shell_edges = Vec::new();
        if let Some(shell) = shell {
            if shell.schema_id != DYNAMIC_MESH_COLLIDER_SHELL_SCHEMA_ID {
                return Err(OpticsError::UnexpectedSchema {
                    expected: DYNAMIC_MESH_COLLIDER_SHELL_SCHEMA_ID,
                    actual: shell.schema_id.clone(),
                });
            }
            shell_edges = mesh_debug_lines_from_surface_edges(
                &shell.surface,
                MeshDebugLineRole::ColliderShell,
                ColorRgba::new(0.98, 0.68, 0.24, 1.0),
            )?;
        }

        let mut contact_points = Vec::new();
        let mut contact_normals = Vec::new();
        if let Some(contact) = contact {
            contact_points.push(MeshDebugPoint::new(
                "mesh.collider.contact.0000",
                contact.point,
                0.018,
                MeshDebugPointRole::ColliderContact,
                ColorRgba::new(1.0, 0.26, 0.76, 1.0),
            ));
            contact_normals.push(MeshDebugLine::new(
                contact.point,
                contact.point + contact.normal * 0.08,
                MeshDebugLineRole::ContactNormal,
                ColorRgba::new(1.0, 0.26, 0.76, 1.0),
            ));
        }

        let visual = Self {
            schema_id: MESH_COLLIDER_VISUAL_SCHEMA_ID.to_owned(),
            visual_id: visual_id.into(),
            source_surface_id: source_surface_id.into(),
            update_status: collider_status_label(update.status).to_owned(),
            collider_vertex_count: update.vertex_count,
            collider_triangle_count: update.triangle_count,
            shell_edges,
            contact_points,
            contact_normals,
        };
        visual.validate()?;
        Ok(visual)
    }

    /// Validates collider visual shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != MESH_COLLIDER_VISUAL_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: MESH_COLLIDER_VISUAL_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.visual_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("visual_id"));
        }
        if self.source_surface_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_surface_id"));
        }
        if self.update_status.trim().is_empty() {
            return Err(OpticsError::EmptyId("update_status"));
        }
        if self.collider_vertex_count == 0 || self.collider_triangle_count == 0 {
            return Err(OpticsError::InvalidCount("collider surface"));
        }
        for edge in &self.shell_edges {
            edge.validate()?;
        }
        for point in &self.contact_points {
            point.validate()?;
        }
        for normal in &self.contact_normals {
            normal.validate()?;
        }
        Ok(())
    }
}

fn collider_status_label(status: DynamicMeshColliderUpdateStatus) -> &'static str {
    match status {
        DynamicMeshColliderUpdateStatus::Disabled => "disabled",
        DynamicMeshColliderUpdateStatus::Initialized => "initialized",
        DynamicMeshColliderUpdateStatus::Updated => "updated",
        DynamicMeshColliderUpdateStatus::ChangedTopology => "changed_topology",
        DynamicMeshColliderUpdateStatus::InvalidSurface => "invalid_surface",
    }
}
