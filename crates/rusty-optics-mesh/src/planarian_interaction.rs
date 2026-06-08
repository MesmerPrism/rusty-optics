use rusty_optics_model::{
    OpticsError, Vec2, PLANARIAN_BIOELECTRIC_EDIT_INTENT_SCHEMA_ID,
    PLANARIAN_BIOELECTRIC_PICK_SELECTION_SCHEMA_ID,
};

use crate::PlanarianBioelectricVisualSequence;

/// Renderer-neutral target selected from a planarian 3D visual.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PlanarianPickTarget {
    /// A sampled Matter surface-field node.
    SurfaceNode {
        /// Source node index.
        node_index: usize,
        /// Source node identifier.
        node_id: String,
        /// AP visual region identifier.
        region_id: String,
        /// AP coordinate in 0..=1, posterior to anterior.
        ap_coordinate: f32,
        /// Lateral coordinate normalized by Matter/Optics region metadata.
        lateral_coordinate: f32,
    },
    /// A body mesh triangle from the Matter-owned source surface.
    BodyTriangle {
        /// Source body triangle index.
        triangle_index: usize,
        /// AP visual region identifier.
        region_id: String,
    },
    /// A conductance edge from the Matter-owned bioelectric circuit.
    ConductanceEdge {
        /// Source conductance edge index.
        edge_index: usize,
        /// Source node index.
        from: usize,
        /// Target node index.
        to: usize,
        /// Neighbor tier, starting at 1.
        tier: u8,
    },
}

/// Renderer-neutral pick selection emitted by a 3D renderer adapter.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PlanarianPickSelection {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable selection identifier for audit and UI feedback.
    pub selection_id: String,
    /// Source visual context identifier.
    pub visual_id: String,
    /// Source Matter surface identifier.
    pub surface_id: String,
    /// Source Matter substrate identifier.
    pub substrate_id: String,
    /// Selected visual target.
    pub target: PlanarianPickTarget,
    /// Optional normalized pointer in renderer viewport coordinates.
    pub normalized_pointer: Option<Vec2>,
    /// Renderer-reported hit distance in view units.
    pub distance: f32,
    /// Matter circuit revision visible when the pick was emitted, when known.
    pub view_revision: Option<u64>,
}

impl PlanarianPickSelection {
    /// Builds a surface-node selection from a checked Planarian visual sequence.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the requested node is missing or the
    /// generated selection is invalid.
    pub fn from_sequence_node(
        selection_id: impl Into<String>,
        sequence: &PlanarianBioelectricVisualSequence,
        node_index: usize,
        normalized_pointer: Option<Vec2>,
        distance: f32,
        view_revision: Option<u64>,
    ) -> Result<Self, OpticsError> {
        sequence.validate()?;
        let Some(node_region) = sequence.node_regions.get(node_index) else {
            return Err(OpticsError::InvalidPayload("planarian pick node target"));
        };
        let Some(node) = sequence.frames[0].nodes.get(node_index) else {
            return Err(OpticsError::InvalidPayload("planarian pick node frame"));
        };
        let selection = Self {
            schema_id: PLANARIAN_BIOELECTRIC_PICK_SELECTION_SCHEMA_ID.to_owned(),
            selection_id: selection_id.into(),
            visual_id: sequence.sequence_id.clone(),
            surface_id: sequence.surface_id.clone(),
            substrate_id: sequence.substrate_id.clone(),
            target: PlanarianPickTarget::SurfaceNode {
                node_index,
                node_id: node.node_id.clone(),
                region_id: node_region.region_id.clone(),
                ap_coordinate: node_region.ap_coordinate,
                lateral_coordinate: node_region.lateral_coordinate,
            },
            normalized_pointer,
            distance,
            view_revision,
        };
        selection.validate_for_sequence(sequence)?;
        Ok(selection)
    }

    /// Builds a conductance-edge selection from a checked Planarian visual
    /// sequence.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the requested conductance edge is missing
    /// or the generated selection is invalid.
    pub fn from_sequence_conductance_edge(
        selection_id: impl Into<String>,
        sequence: &PlanarianBioelectricVisualSequence,
        edge_index: usize,
        normalized_pointer: Option<Vec2>,
        distance: f32,
        view_revision: Option<u64>,
    ) -> Result<Self, OpticsError> {
        sequence.validate()?;
        let Some(edge) = sequence.frames[0].conductance_edges.get(edge_index) else {
            return Err(OpticsError::InvalidPayload("planarian pick edge target"));
        };
        let selection = Self {
            schema_id: PLANARIAN_BIOELECTRIC_PICK_SELECTION_SCHEMA_ID.to_owned(),
            selection_id: selection_id.into(),
            visual_id: sequence.sequence_id.clone(),
            surface_id: sequence.surface_id.clone(),
            substrate_id: sequence.substrate_id.clone(),
            target: PlanarianPickTarget::ConductanceEdge {
                edge_index,
                from: edge.from,
                to: edge.to,
                tier: edge.tier,
            },
            normalized_pointer,
            distance,
            view_revision,
        };
        selection.validate_for_sequence(sequence)?;
        Ok(selection)
    }

    /// Validates selection shape without requiring a source visual sequence.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when IDs, numeric values, or target metadata are
    /// invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PLANARIAN_BIOELECTRIC_PICK_SELECTION_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PLANARIAN_BIOELECTRIC_PICK_SELECTION_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        validate_context_ids(
            &self.selection_id,
            &self.visual_id,
            &self.surface_id,
            &self.substrate_id,
            "planarian pick selection",
        )?;
        validate_pointer(self.normalized_pointer, "planarian pick pointer")?;
        validate_non_negative_finite(self.distance, "planarian pick distance")?;
        self.target.validate()
    }

    /// Validates the selection against a checked Planarian visual sequence.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the selection does not reference the
    /// sequence or target metadata does not match the sequence.
    pub fn validate_for_sequence(
        &self,
        sequence: &PlanarianBioelectricVisualSequence,
    ) -> Result<(), OpticsError> {
        self.validate()?;
        sequence.validate()?;
        validate_sequence_context(
            &self.visual_id,
            &self.surface_id,
            &self.substrate_id,
            sequence,
            "planarian pick selection",
        )?;
        self.target.validate_for_sequence(sequence)
    }

    /// Returns the selected node index when the target is a node.
    #[must_use]
    pub const fn node_index(&self) -> Option<usize> {
        match self.target {
            PlanarianPickTarget::SurfaceNode { node_index, .. } => Some(node_index),
            PlanarianPickTarget::BodyTriangle { .. }
            | PlanarianPickTarget::ConductanceEdge { .. } => None,
        }
    }

    /// Returns the selected conductance edge index when the target is an edge.
    #[must_use]
    pub const fn edge_index(&self) -> Option<usize> {
        match self.target {
            PlanarianPickTarget::ConductanceEdge { edge_index, .. } => Some(edge_index),
            PlanarianPickTarget::SurfaceNode { .. } | PlanarianPickTarget::BodyTriangle { .. } => {
                None
            }
        }
    }
}

/// Renderer-neutral target for a proposed bioelectric edit.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PlanarianBioelectricEditTarget {
    /// A sampled Matter surface-field node.
    SurfaceNode {
        /// Source node index.
        node_index: usize,
        /// Source node identifier.
        node_id: String,
    },
    /// A Matter-owned conductance edge.
    ConductanceEdge {
        /// Source conductance edge index.
        edge_index: usize,
        /// Source node index.
        from: usize,
        /// Target node index.
        to: usize,
        /// Neighbor tier, starting at 1.
        tier: u8,
    },
}

/// Requested bioelectric edit operation proposed by a visual/control surface.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PlanarianBioelectricEditIntentOperation {
    /// Add a voltage delta to one selected node.
    AddNodeVoltage {
        /// Requested voltage delta.
        delta: f32,
    },
    /// Set one selected node's hysteresis memory value.
    SetNodeMemory {
        /// Requested memory value.
        memory_value: f32,
    },
    /// Scale all conductance edges incident on one selected node.
    ScaleIncidentConductance {
        /// Requested conductance scale.
        scale: f32,
    },
    /// Add a transient constant current source to one selected node.
    AddTransientCurrent {
        /// Current contribution while active.
        current: f32,
        /// Active duration in fixed steps.
        duration_steps: u32,
    },
    /// Set a gate threshold on one selected conductance edge.
    SetEdgeGateThreshold {
        /// Requested threshold.
        threshold: f32,
        /// Optional requested slope.
        slope: Option<f32>,
    },
    /// Set gate multiplier bounds on one selected conductance edge.
    SetEdgeGateMultiplierBounds {
        /// Requested lower multiplier bound.
        min_multiplier: f32,
        /// Requested upper multiplier bound.
        max_multiplier: f32,
    },
}

/// Renderer-neutral edit intent proposed by a 3D visual/control surface.
///
/// Optics validates that the intent references a visible target and carries
/// finite request values. Matter remains the authority that accepts, rejects,
/// clamps, mutates state, and advances revisions.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PlanarianBioelectricEditIntent {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable intent identifier for audit and UI feedback.
    pub intent_id: String,
    /// Source pick selection identifier.
    pub selection_id: String,
    /// Source visual context identifier.
    pub visual_id: String,
    /// Source Matter surface identifier.
    pub surface_id: String,
    /// Source Matter substrate identifier.
    pub substrate_id: String,
    /// Matter circuit revision the requester believes it is editing, when known.
    pub expected_revision: Option<u64>,
    /// Proposed target.
    pub target: PlanarianBioelectricEditTarget,
    /// Proposed operation.
    pub operation: PlanarianBioelectricEditIntentOperation,
}

impl PlanarianBioelectricEditIntent {
    /// Builds a node voltage-delta intent from a node pick selection.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the selection is not a node selection or
    /// the generated intent is invalid.
    pub fn add_node_voltage(
        intent_id: impl Into<String>,
        selection: &PlanarianPickSelection,
        expected_revision: Option<u64>,
        delta: f32,
    ) -> Result<Self, OpticsError> {
        Self::from_node_selection(
            intent_id,
            selection,
            expected_revision,
            PlanarianBioelectricEditIntentOperation::AddNodeVoltage { delta },
        )
    }

    /// Builds a node memory-set intent from a node pick selection.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the selection is not a node selection or
    /// the generated intent is invalid.
    pub fn set_node_memory(
        intent_id: impl Into<String>,
        selection: &PlanarianPickSelection,
        expected_revision: Option<u64>,
        memory_value: f32,
    ) -> Result<Self, OpticsError> {
        Self::from_node_selection(
            intent_id,
            selection,
            expected_revision,
            PlanarianBioelectricEditIntentOperation::SetNodeMemory { memory_value },
        )
    }

    /// Builds an incident-conductance scale intent from a node pick selection.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the selection is not a node selection or
    /// the generated intent is invalid.
    pub fn scale_incident_conductance(
        intent_id: impl Into<String>,
        selection: &PlanarianPickSelection,
        expected_revision: Option<u64>,
        scale: f32,
    ) -> Result<Self, OpticsError> {
        Self::from_node_selection(
            intent_id,
            selection,
            expected_revision,
            PlanarianBioelectricEditIntentOperation::ScaleIncidentConductance { scale },
        )
    }

    /// Builds a transient-current intent from a node pick selection.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the selection is not a node selection or
    /// the generated intent is invalid.
    pub fn add_transient_current(
        intent_id: impl Into<String>,
        selection: &PlanarianPickSelection,
        expected_revision: Option<u64>,
        current: f32,
        duration_steps: u32,
    ) -> Result<Self, OpticsError> {
        Self::from_node_selection(
            intent_id,
            selection,
            expected_revision,
            PlanarianBioelectricEditIntentOperation::AddTransientCurrent {
                current,
                duration_steps,
            },
        )
    }

    /// Builds a gate-threshold edit intent from an edge pick selection.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the selection is not a conductance-edge
    /// selection or the generated intent is invalid.
    pub fn set_edge_gate_threshold(
        intent_id: impl Into<String>,
        selection: &PlanarianPickSelection,
        expected_revision: Option<u64>,
        threshold: f32,
        slope: Option<f32>,
    ) -> Result<Self, OpticsError> {
        Self::from_edge_selection(
            intent_id,
            selection,
            expected_revision,
            PlanarianBioelectricEditIntentOperation::SetEdgeGateThreshold { threshold, slope },
        )
    }

    /// Builds a gate multiplier-bounds edit intent from an edge pick selection.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the selection is not a conductance-edge
    /// selection or the generated intent is invalid.
    pub fn set_edge_gate_multiplier_bounds(
        intent_id: impl Into<String>,
        selection: &PlanarianPickSelection,
        expected_revision: Option<u64>,
        min_multiplier: f32,
        max_multiplier: f32,
    ) -> Result<Self, OpticsError> {
        Self::from_edge_selection(
            intent_id,
            selection,
            expected_revision,
            PlanarianBioelectricEditIntentOperation::SetEdgeGateMultiplierBounds {
                min_multiplier,
                max_multiplier,
            },
        )
    }

    /// Validates intent shape without requiring a source visual sequence.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when IDs, numeric values, or target metadata are
    /// invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PLANARIAN_BIOELECTRIC_EDIT_INTENT_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PLANARIAN_BIOELECTRIC_EDIT_INTENT_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.selection_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("planarian edit selection"));
        }
        validate_context_ids(
            &self.intent_id,
            &self.visual_id,
            &self.surface_id,
            &self.substrate_id,
            "planarian edit intent",
        )?;
        self.target.validate()?;
        self.operation.validate()?;
        validate_operation_matches_target(&self.operation, &self.target)
    }

    /// Validates the intent against a checked Planarian visual sequence.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the intent does not reference the sequence
    /// or its target metadata does not match the sequence.
    pub fn validate_for_sequence(
        &self,
        sequence: &PlanarianBioelectricVisualSequence,
    ) -> Result<(), OpticsError> {
        self.validate()?;
        sequence.validate()?;
        validate_sequence_context(
            &self.visual_id,
            &self.surface_id,
            &self.substrate_id,
            sequence,
            "planarian edit intent",
        )?;
        self.target.validate_for_sequence(sequence)
    }

    /// Validates the intent against the pick selection that produced it.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the intent and selection do not reference
    /// the same context or target.
    pub fn validate_for_selection(
        &self,
        selection: &PlanarianPickSelection,
    ) -> Result<(), OpticsError> {
        self.validate()?;
        selection.validate()?;
        if self.selection_id != selection.selection_id
            || self.visual_id != selection.visual_id
            || self.surface_id != selection.surface_id
            || self.substrate_id != selection.substrate_id
        {
            return Err(OpticsError::InvalidPayload(
                "planarian edit intent must reference its pick selection",
            ));
        }
        validate_edit_target_matches_pick(&self.target, &selection.target)
    }

    fn from_node_selection(
        intent_id: impl Into<String>,
        selection: &PlanarianPickSelection,
        expected_revision: Option<u64>,
        operation: PlanarianBioelectricEditIntentOperation,
    ) -> Result<Self, OpticsError> {
        selection.validate()?;
        let PlanarianPickTarget::SurfaceNode {
            node_index,
            node_id,
            ..
        } = &selection.target
        else {
            return Err(OpticsError::InvalidPayload(
                "planarian edit intent requires a node selection",
            ));
        };
        let intent = Self {
            schema_id: PLANARIAN_BIOELECTRIC_EDIT_INTENT_SCHEMA_ID.to_owned(),
            intent_id: intent_id.into(),
            selection_id: selection.selection_id.clone(),
            visual_id: selection.visual_id.clone(),
            surface_id: selection.surface_id.clone(),
            substrate_id: selection.substrate_id.clone(),
            expected_revision,
            target: PlanarianBioelectricEditTarget::SurfaceNode {
                node_index: *node_index,
                node_id: node_id.clone(),
            },
            operation,
        };
        intent.validate_for_selection(selection)?;
        Ok(intent)
    }

    fn from_edge_selection(
        intent_id: impl Into<String>,
        selection: &PlanarianPickSelection,
        expected_revision: Option<u64>,
        operation: PlanarianBioelectricEditIntentOperation,
    ) -> Result<Self, OpticsError> {
        selection.validate()?;
        let PlanarianPickTarget::ConductanceEdge {
            edge_index,
            from,
            to,
            tier,
        } = &selection.target
        else {
            return Err(OpticsError::InvalidPayload(
                "planarian edit intent requires an edge selection",
            ));
        };
        let intent = Self {
            schema_id: PLANARIAN_BIOELECTRIC_EDIT_INTENT_SCHEMA_ID.to_owned(),
            intent_id: intent_id.into(),
            selection_id: selection.selection_id.clone(),
            visual_id: selection.visual_id.clone(),
            surface_id: selection.surface_id.clone(),
            substrate_id: selection.substrate_id.clone(),
            expected_revision,
            target: PlanarianBioelectricEditTarget::ConductanceEdge {
                edge_index: *edge_index,
                from: *from,
                to: *to,
                tier: *tier,
            },
            operation,
        };
        intent.validate_for_selection(selection)?;
        Ok(intent)
    }
}

impl PlanarianPickTarget {
    fn validate(&self) -> Result<(), OpticsError> {
        match self {
            Self::SurfaceNode {
                node_id,
                region_id,
                ap_coordinate,
                lateral_coordinate,
                ..
            } => {
                if node_id.trim().is_empty() || region_id.trim().is_empty() {
                    return Err(OpticsError::EmptyId("planarian pick node target"));
                }
                if !ap_coordinate.is_finite()
                    || !(0.0..=1.0).contains(ap_coordinate)
                    || !lateral_coordinate.is_finite()
                {
                    return Err(OpticsError::InvalidValue("planarian pick node target"));
                }
            }
            Self::BodyTriangle { region_id, .. } => {
                if region_id.trim().is_empty() {
                    return Err(OpticsError::EmptyId("planarian pick body target"));
                }
            }
            Self::ConductanceEdge { from, to, tier, .. } => {
                if from == to || *tier == 0 {
                    return Err(OpticsError::InvalidValue("planarian pick edge target"));
                }
            }
        }
        Ok(())
    }

    fn validate_for_sequence(
        &self,
        sequence: &PlanarianBioelectricVisualSequence,
    ) -> Result<(), OpticsError> {
        match self {
            Self::SurfaceNode {
                node_index,
                node_id,
                region_id,
                ap_coordinate,
                lateral_coordinate,
            } => {
                let Some(node_region) = sequence.node_regions.get(*node_index) else {
                    return Err(OpticsError::InvalidPayload("planarian pick node target"));
                };
                let Some(node) = sequence.frames[0].nodes.get(*node_index) else {
                    return Err(OpticsError::InvalidPayload("planarian pick node frame"));
                };
                if node.node_id != *node_id
                    || node_region.region_id != *region_id
                    || (node_region.ap_coordinate - *ap_coordinate).abs() > 1.0e-4
                    || (node_region.lateral_coordinate - *lateral_coordinate).abs() > 1.0e-4
                {
                    return Err(OpticsError::InvalidPayload(
                        "planarian pick node metadata must match sequence",
                    ));
                }
            }
            Self::BodyTriangle {
                triangle_index,
                region_id,
            } => {
                let Some(triangle) = sequence.body_surface.triangles.get(*triangle_index) else {
                    return Err(OpticsError::InvalidPayload("planarian pick body target"));
                };
                if triangle.region_id != *region_id {
                    return Err(OpticsError::InvalidPayload(
                        "planarian pick body metadata must match sequence",
                    ));
                }
            }
            Self::ConductanceEdge {
                edge_index,
                from,
                to,
                tier,
            } => {
                let Some(edge) = sequence.frames[0].conductance_edges.get(*edge_index) else {
                    return Err(OpticsError::InvalidPayload("planarian pick edge target"));
                };
                if edge.from != *from || edge.to != *to || edge.tier != *tier {
                    return Err(OpticsError::InvalidPayload(
                        "planarian pick edge metadata must match sequence",
                    ));
                }
            }
        }
        Ok(())
    }
}

impl PlanarianBioelectricEditTarget {
    fn validate(&self) -> Result<(), OpticsError> {
        match self {
            Self::SurfaceNode { node_id, .. } => {
                if node_id.trim().is_empty() {
                    return Err(OpticsError::EmptyId("planarian edit node target"));
                }
            }
            Self::ConductanceEdge { from, to, tier, .. } => {
                if from == to || *tier == 0 {
                    return Err(OpticsError::InvalidValue("planarian edit edge target"));
                }
            }
        }
        Ok(())
    }

    fn validate_for_sequence(
        &self,
        sequence: &PlanarianBioelectricVisualSequence,
    ) -> Result<(), OpticsError> {
        match self {
            Self::SurfaceNode {
                node_index,
                node_id,
            } => {
                let Some(node) = sequence.frames[0].nodes.get(*node_index) else {
                    return Err(OpticsError::InvalidPayload("planarian edit node target"));
                };
                if node.node_id != *node_id {
                    return Err(OpticsError::InvalidPayload(
                        "planarian edit node metadata must match sequence",
                    ));
                }
            }
            Self::ConductanceEdge {
                edge_index,
                from,
                to,
                tier,
            } => {
                let Some(edge) = sequence.frames[0].conductance_edges.get(*edge_index) else {
                    return Err(OpticsError::InvalidPayload("planarian edit edge target"));
                };
                if edge.from != *from || edge.to != *to || edge.tier != *tier {
                    return Err(OpticsError::InvalidPayload(
                        "planarian edit edge metadata must match sequence",
                    ));
                }
            }
        }
        Ok(())
    }
}

impl PlanarianBioelectricEditIntentOperation {
    fn validate(&self) -> Result<(), OpticsError> {
        match self {
            Self::AddNodeVoltage { delta } => {
                validate_finite(*delta, "planarian edit voltage delta")?;
            }
            Self::SetNodeMemory { memory_value } => {
                if !memory_value.is_finite() || !(0.0..=1.0).contains(memory_value) {
                    return Err(OpticsError::InvalidValue("planarian edit memory value"));
                }
            }
            Self::ScaleIncidentConductance { scale } => {
                if !scale.is_finite() || *scale < 0.0 {
                    return Err(OpticsError::InvalidValue(
                        "planarian edit conductance scale",
                    ));
                }
            }
            Self::AddTransientCurrent {
                current,
                duration_steps,
            } => {
                validate_finite(*current, "planarian edit current")?;
                if *duration_steps == 0 {
                    return Err(OpticsError::InvalidValue("planarian edit current duration"));
                }
            }
            Self::SetEdgeGateThreshold { threshold, slope } => {
                validate_finite(*threshold, "planarian edit gate threshold")?;
                if slope.is_some_and(|value| !value.is_finite() || value == 0.0) {
                    return Err(OpticsError::InvalidValue("planarian edit gate slope"));
                }
            }
            Self::SetEdgeGateMultiplierBounds {
                min_multiplier,
                max_multiplier,
            } => {
                if !min_multiplier.is_finite()
                    || !max_multiplier.is_finite()
                    || *min_multiplier < 0.0
                    || *min_multiplier > *max_multiplier
                {
                    return Err(OpticsError::InvalidValue(
                        "planarian edit gate multiplier bounds",
                    ));
                }
            }
        }
        Ok(())
    }
}

fn validate_operation_matches_target(
    operation: &PlanarianBioelectricEditIntentOperation,
    target: &PlanarianBioelectricEditTarget,
) -> Result<(), OpticsError> {
    let node_operation = matches!(
        operation,
        PlanarianBioelectricEditIntentOperation::AddNodeVoltage { .. }
            | PlanarianBioelectricEditIntentOperation::SetNodeMemory { .. }
            | PlanarianBioelectricEditIntentOperation::ScaleIncidentConductance { .. }
            | PlanarianBioelectricEditIntentOperation::AddTransientCurrent { .. }
    );
    match (node_operation, target) {
        (true, PlanarianBioelectricEditTarget::SurfaceNode { .. })
        | (false, PlanarianBioelectricEditTarget::ConductanceEdge { .. }) => Ok(()),
        _ => Err(OpticsError::InvalidPayload(
            "planarian edit operation target mismatch",
        )),
    }
}

fn validate_edit_target_matches_pick(
    target: &PlanarianBioelectricEditTarget,
    pick: &PlanarianPickTarget,
) -> Result<(), OpticsError> {
    match (target, pick) {
        (
            PlanarianBioelectricEditTarget::SurfaceNode {
                node_index,
                node_id,
            },
            PlanarianPickTarget::SurfaceNode {
                node_index: pick_index,
                node_id: pick_id,
                ..
            },
        ) if node_index == pick_index && node_id == pick_id => Ok(()),
        (
            PlanarianBioelectricEditTarget::ConductanceEdge {
                edge_index,
                from,
                to,
                tier,
            },
            PlanarianPickTarget::ConductanceEdge {
                edge_index: pick_index,
                from: pick_from,
                to: pick_to,
                tier: pick_tier,
            },
        ) if edge_index == pick_index
            && from == pick_from
            && to == pick_to
            && tier == pick_tier =>
        {
            Ok(())
        }
        _ => Err(OpticsError::InvalidPayload(
            "planarian edit target must match pick target",
        )),
    }
}

fn validate_context_ids(
    primary_id: &str,
    visual_id: &str,
    surface_id: &str,
    substrate_id: &str,
    label: &'static str,
) -> Result<(), OpticsError> {
    if primary_id.trim().is_empty()
        || visual_id.trim().is_empty()
        || surface_id.trim().is_empty()
        || substrate_id.trim().is_empty()
    {
        return Err(OpticsError::EmptyId(label));
    }
    Ok(())
}

fn validate_sequence_context(
    visual_id: &str,
    surface_id: &str,
    substrate_id: &str,
    sequence: &PlanarianBioelectricVisualSequence,
    label: &'static str,
) -> Result<(), OpticsError> {
    if visual_id != sequence.sequence_id
        || surface_id != sequence.surface_id
        || substrate_id != sequence.substrate_id
    {
        return Err(OpticsError::InvalidPayload(label));
    }
    Ok(())
}

fn validate_pointer(pointer: Option<Vec2>, label: &'static str) -> Result<(), OpticsError> {
    let Some(pointer) = pointer else {
        return Ok(());
    };
    if !pointer.is_finite()
        || !(-1.0..=1.0).contains(&pointer.x)
        || !(-1.0..=1.0).contains(&pointer.y)
    {
        return Err(OpticsError::InvalidValue(label));
    }
    Ok(())
}

fn validate_non_negative_finite(value: f32, label: &'static str) -> Result<(), OpticsError> {
    if !value.is_finite() || value < 0.0 {
        return Err(OpticsError::InvalidValue(label));
    }
    Ok(())
}

fn validate_finite(value: f32, label: &'static str) -> Result<(), OpticsError> {
    if !value.is_finite() {
        return Err(OpticsError::InvalidValue(label));
    }
    Ok(())
}
