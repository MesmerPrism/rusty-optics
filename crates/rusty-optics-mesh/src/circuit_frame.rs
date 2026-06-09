use rusty_matter_fields::{
    BioelectricCircuitState, BioelectricCircuitStepDiagnostics, BioelectricCurrentKind,
    BioelectricCurrentTerm, SurfaceFieldSubstrate, BIOELECTRIC_CIRCUIT_STATE_SCHEMA_ID,
    BIOELECTRIC_STEP_DIAGNOSTICS_SCHEMA_ID,
};
use rusty_matter_model::Vec3;
use rusty_optics_model::{ColorRgba, OpticsError, BIOELECTRIC_CIRCUIT_VISUAL_FRAME_SCHEMA_ID};

/// Renderer-neutral visual node for a bioelectric circuit sample.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BioelectricCircuitVisualNode {
    /// Source node index.
    pub node_index: usize,
    /// Source node identifier.
    pub node_id: String,
    /// Node position.
    pub position: Vec3,
    /// Base point radius in surface units.
    pub radius: f32,
}

/// Voltage color sample for one circuit node.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BioelectricVoltageVisualSample {
    /// Source node index.
    pub node_index: usize,
    /// Source voltage value.
    pub voltage: f32,
    /// Normalized voltage in the frame range.
    pub normalized_voltage: f32,
    /// Resolved node color.
    pub color: ColorRgba,
}

/// Conductance edge visual for a gap-junction-like coupling edge.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BioelectricConductanceVisualEdge {
    /// Source node index.
    pub from: usize,
    /// Target node index.
    pub to: usize,
    /// Neighbor tier, starting at 1.
    pub tier: u8,
    /// Current conductance after Matter-owned gating.
    pub conductance: f32,
    /// Normalized conductance in the frame range.
    pub normalized_conductance: f32,
    /// Whether this edge has a gate.
    pub gated: bool,
    /// Edge color.
    pub color: ColorRgba,
}

/// Memory-state visual sample for one circuit node.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BioelectricMemoryVisualSample {
    /// Source node index.
    pub node_index: usize,
    /// Hysteresis memory value in 0..=1.
    pub value: f32,
    /// Resolved node color.
    pub color: ColorRgba,
}

/// Readout value visual sample for one node.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BioelectricReadoutVisualSample {
    /// Source node index.
    pub node_index: usize,
    /// Source readout value.
    pub value: f32,
    /// Normalized readout value in the layer range.
    pub normalized_value: f32,
    /// Resolved node color.
    pub color: ColorRgba,
}

/// Renderer-neutral visual layer for one voltage-driven readout.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BioelectricReadoutVisualLayer {
    /// Source readout layer identifier.
    pub layer_id: String,
    /// Minimum readout value.
    pub min_value: f32,
    /// Maximum readout value.
    pub max_value: f32,
    /// Per-node readout samples.
    pub samples: Vec<BioelectricReadoutVisualSample>,
}

/// Current term target region for visualization.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BioelectricCurrentTermVisualRegion {
    /// Source current term identifier.
    pub term_id: String,
    /// Current term label.
    pub kind: String,
    /// Region node indices. Empty means all nodes.
    pub node_indices: Vec<usize>,
    /// Whether the term targets all nodes.
    pub all_nodes: bool,
    /// Region color.
    pub color: ColorRgba,
}

/// Compact Optics-owned copy of Matter step diagnostics for display.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BioelectricCircuitVisualDiagnostics {
    /// Step index after the Matter update.
    pub step_index: u32,
    /// Directed conductance edges visited.
    pub visited_edges: usize,
    /// Active current terms.
    pub active_current_terms: usize,
    /// Conductance gates evaluated.
    pub active_gates: usize,
    /// Voltage values clamped.
    pub clamped_voltage_nodes: usize,
    /// Conductance edges clamped.
    pub clamped_conductance_edges: usize,
    /// Memory values crossing into active state.
    pub memory_activated_nodes: usize,
    /// Maximum absolute voltage delta.
    pub max_voltage_delta: f32,
    /// Sum of absolute net current.
    pub net_current_abs_sum: f32,
}

impl BioelectricCircuitVisualDiagnostics {
    fn from_matter(source: &BioelectricCircuitStepDiagnostics) -> Result<Self, OpticsError> {
        if source.schema_id != BIOELECTRIC_STEP_DIAGNOSTICS_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: BIOELECTRIC_STEP_DIAGNOSTICS_SCHEMA_ID,
                actual: source.schema_id.clone(),
            });
        }
        Ok(Self {
            step_index: source.step_index,
            visited_edges: source.visited_edges,
            active_current_terms: source.active_current_terms,
            active_gates: source.active_gates,
            clamped_voltage_nodes: source.clamped_voltage_nodes,
            clamped_conductance_edges: source.clamped_conductance_edges,
            memory_activated_nodes: source.memory_activated_nodes,
            max_voltage_delta: source.max_voltage_delta,
            net_current_abs_sum: source.net_current_abs_sum,
        })
    }

    fn validate(&self) -> Result<(), OpticsError> {
        if !self.max_voltage_delta.is_finite()
            || self.max_voltage_delta < 0.0
            || !self.net_current_abs_sum.is_finite()
            || self.net_current_abs_sum < 0.0
        {
            return Err(OpticsError::InvalidValue("bioelectric visual diagnostics"));
        }
        Ok(())
    }
}

/// Renderer-neutral visual frame for Matter bioelectric circuit state.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BioelectricCircuitVisualFrame {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable visual frame identifier.
    pub frame_id: String,
    /// Source Matter circuit identifier.
    pub source_circuit_id: String,
    /// Source Matter schema identifier.
    pub source_schema_id: String,
    /// Source Matter substrate identifier.
    pub substrate_id: String,
    /// Source Matter surface identifier.
    pub surface_id: String,
    /// State time in seconds.
    pub time_seconds: f32,
    /// Bounds minimum for fit-to-view.
    pub bounds_min: Vec3,
    /// Bounds maximum for fit-to-view.
    pub bounds_max: Vec3,
    /// Circuit nodes.
    pub nodes: Vec<BioelectricCircuitVisualNode>,
    /// Per-node voltage colors.
    pub voltage_samples: Vec<BioelectricVoltageVisualSample>,
    /// Directed conductance edge visuals.
    pub conductance_edges: Vec<BioelectricConductanceVisualEdge>,
    /// Current term target visuals.
    pub current_regions: Vec<BioelectricCurrentTermVisualRegion>,
    /// Optional per-node memory colors.
    pub memory_samples: Vec<BioelectricMemoryVisualSample>,
    /// Voltage-driven readout layers.
    pub readout_layers: Vec<BioelectricReadoutVisualLayer>,
    /// Optional step diagnostics.
    pub diagnostics: Option<BioelectricCircuitVisualDiagnostics>,
}

impl BioelectricCircuitVisualFrame {
    /// Creates a circuit visual frame from validated Matter circuit state.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when source or visual contracts are invalid.
    pub fn from_matter_circuit_state(
        frame_id: impl Into<String>,
        substrate: &SurfaceFieldSubstrate,
        state: &BioelectricCircuitState,
        diagnostics: Option<&BioelectricCircuitStepDiagnostics>,
    ) -> Result<Self, OpticsError> {
        if state.schema_id != BIOELECTRIC_CIRCUIT_STATE_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: BIOELECTRIC_CIRCUIT_STATE_SCHEMA_ID,
                actual: state.schema_id.clone(),
            });
        }
        substrate
            .validate()
            .map_err(|_| OpticsError::InvalidPayload("source bioelectric substrate invalid"))?;
        state
            .validate()
            .map_err(|_| OpticsError::InvalidPayload("source bioelectric circuit state invalid"))?;
        if state.substrate_id != substrate.substrate_id
            || state.node_count != substrate.node_count()
        {
            return Err(OpticsError::InvalidPayload(
                "bioelectric circuit must match substrate",
            ));
        }

        let (bounds_min, bounds_max) = bounds_from_substrate(substrate)?;
        let node_radius = visual_node_radius(bounds_min, bounds_max);
        let (voltage_min, voltage_max) = value_range(&state.voltage.values)?;
        let conductance_max = state
            .conductance_edges
            .iter()
            .map(|edge| edge.conductance)
            .max_by(f32::total_cmp)
            .unwrap_or(0.0)
            .max(1.0e-6);
        let diagnostics = diagnostics
            .map(BioelectricCircuitVisualDiagnostics::from_matter)
            .transpose()?;
        let frame = Self {
            schema_id: BIOELECTRIC_CIRCUIT_VISUAL_FRAME_SCHEMA_ID.to_owned(),
            frame_id: frame_id.into(),
            source_circuit_id: state.circuit_id.clone(),
            source_schema_id: state.schema_id.clone(),
            substrate_id: state.substrate_id.clone(),
            surface_id: substrate.surface_id.clone(),
            time_seconds: state.time_seconds,
            bounds_min,
            bounds_max,
            nodes: substrate
                .nodes
                .iter()
                .map(|node| BioelectricCircuitVisualNode {
                    node_index: node.node_index,
                    node_id: node.node_id.clone(),
                    position: node.position,
                    radius: node_radius,
                })
                .collect(),
            voltage_samples: state
                .voltage
                .values
                .iter()
                .copied()
                .enumerate()
                .map(|(node_index, voltage)| {
                    let normalized_voltage = normalize_value(voltage, voltage_min, voltage_max);
                    BioelectricVoltageVisualSample {
                        node_index,
                        voltage,
                        normalized_voltage,
                        color: voltage_color(voltage, normalized_voltage),
                    }
                })
                .collect(),
            conductance_edges: state
                .conductance_edges
                .iter()
                .map(|edge| {
                    let normalized_conductance =
                        (edge.conductance / conductance_max).clamp(0.0, 1.0);
                    BioelectricConductanceVisualEdge {
                        from: edge.from_node,
                        to: edge.to_node,
                        tier: edge.tier,
                        conductance: edge.conductance,
                        normalized_conductance,
                        gated: edge.gate.is_some(),
                        color: conductance_color(normalized_conductance, edge.gate.is_some()),
                    }
                })
                .collect(),
            current_regions: state
                .current_terms
                .iter()
                .map(current_region)
                .collect::<Vec<_>>(),
            memory_samples: state
                .memory
                .as_ref()
                .map(|memory| {
                    memory
                        .values
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(node_index, value)| BioelectricMemoryVisualSample {
                            node_index,
                            value,
                            color: memory_color(value),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            readout_layers: state
                .readout_layers
                .iter()
                .map(|layer| {
                    let (min_value, max_value) = value_range(&layer.values)?;
                    Ok(BioelectricReadoutVisualLayer {
                        layer_id: layer.layer_id.clone(),
                        min_value,
                        max_value,
                        samples: layer
                            .values
                            .iter()
                            .copied()
                            .enumerate()
                            .map(|(node_index, value)| {
                                let normalized_value = normalize_value(value, min_value, max_value);
                                BioelectricReadoutVisualSample {
                                    node_index,
                                    value,
                                    normalized_value,
                                    color: readout_color(normalized_value),
                                }
                            })
                            .collect(),
                    })
                })
                .collect::<Result<Vec<_>, OpticsError>>()?,
            diagnostics,
        };
        frame.validate()?;
        Ok(frame)
    }

    /// Validates the circuit visual frame.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when IDs, counts, colors, or values are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != BIOELECTRIC_CIRCUIT_VISUAL_FRAME_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: BIOELECTRIC_CIRCUIT_VISUAL_FRAME_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.frame_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("bioelectric frame_id"));
        }
        if self.source_circuit_id.trim().is_empty() || self.source_schema_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("bioelectric source"));
        }
        if self.substrate_id.trim().is_empty() || self.surface_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("bioelectric substrate"));
        }
        if !self.time_seconds.is_finite() || self.time_seconds < 0.0 {
            return Err(OpticsError::InvalidValue("bioelectric time_seconds"));
        }
        if self.nodes.is_empty() {
            return Err(OpticsError::InvalidCount("bioelectric nodes"));
        }
        if !self.bounds_min.is_finite() || !self.bounds_max.is_finite() {
            return Err(OpticsError::NonFiniteVec3("bioelectric bounds"));
        }
        let node_count = self.nodes.len();
        for (expected_index, node) in self.nodes.iter().enumerate() {
            if node.node_index != expected_index || node.node_id.trim().is_empty() {
                return Err(OpticsError::InvalidPayload(
                    "bioelectric visual nodes must match node order",
                ));
            }
            if !node.position.is_finite() || !node.radius.is_finite() || node.radius <= 0.0 {
                return Err(OpticsError::InvalidValue("bioelectric node"));
            }
        }
        if self.voltage_samples.len() != node_count {
            return Err(OpticsError::InvalidCount("bioelectric voltage samples"));
        }
        for (expected_index, sample) in self.voltage_samples.iter().enumerate() {
            if sample.node_index != expected_index
                || !sample.voltage.is_finite()
                || !sample.normalized_voltage.is_finite()
                || !sample.color.is_finite()
            {
                return Err(OpticsError::InvalidValue("bioelectric voltage sample"));
            }
        }
        for edge in &self.conductance_edges {
            if edge.from >= node_count || edge.to >= node_count || edge.from == edge.to {
                return Err(OpticsError::InvalidPayload("bioelectric conductance edge"));
            }
            if !(1..=2).contains(&edge.tier)
                || !edge.conductance.is_finite()
                || edge.conductance < 0.0
                || !edge.normalized_conductance.is_finite()
                || !edge.color.is_finite()
            {
                return Err(OpticsError::InvalidValue("bioelectric conductance edge"));
            }
        }
        for region in &self.current_regions {
            if region.term_id.trim().is_empty() || region.kind.trim().is_empty() {
                return Err(OpticsError::EmptyId("bioelectric current region"));
            }
            if !region.color.is_finite() {
                return Err(OpticsError::NonFiniteColor("bioelectric current region"));
            }
            for &node_index in &region.node_indices {
                if node_index >= node_count {
                    return Err(OpticsError::InvalidPayload("bioelectric current target"));
                }
            }
        }
        for sample in &self.memory_samples {
            if sample.node_index >= node_count
                || !sample.value.is_finite()
                || !(0.0..=1.0).contains(&sample.value)
                || !sample.color.is_finite()
            {
                return Err(OpticsError::InvalidValue("bioelectric memory sample"));
            }
        }
        for layer in &self.readout_layers {
            validate_readout_layer(layer, node_count)?;
        }
        if let Some(diagnostics) = &self.diagnostics {
            diagnostics.validate()?;
        }
        Ok(())
    }
}

fn current_region(term: &BioelectricCurrentTerm) -> BioelectricCurrentTermVisualRegion {
    BioelectricCurrentTermVisualRegion {
        term_id: term.term_id.clone(),
        kind: current_kind_label(term.kind).to_owned(),
        node_indices: term.target_node_indices.clone(),
        all_nodes: term.target_node_indices.is_empty(),
        color: current_color(term.kind),
    }
}

fn validate_readout_layer(
    layer: &BioelectricReadoutVisualLayer,
    node_count: usize,
) -> Result<(), OpticsError> {
    if layer.layer_id.trim().is_empty() {
        return Err(OpticsError::EmptyId("bioelectric readout layer"));
    }
    if layer.samples.len() != node_count {
        return Err(OpticsError::InvalidCount("bioelectric readout samples"));
    }
    if !layer.min_value.is_finite()
        || !layer.max_value.is_finite()
        || layer.min_value > layer.max_value
    {
        return Err(OpticsError::InvalidValue("bioelectric readout range"));
    }
    for (expected_index, sample) in layer.samples.iter().enumerate() {
        if sample.node_index != expected_index
            || !sample.value.is_finite()
            || !sample.normalized_value.is_finite()
            || !sample.color.is_finite()
        {
            return Err(OpticsError::InvalidValue("bioelectric readout sample"));
        }
    }
    Ok(())
}

fn bounds_from_substrate(substrate: &SurfaceFieldSubstrate) -> Result<(Vec3, Vec3), OpticsError> {
    let mut nodes = substrate.nodes.iter();
    let Some(first) = nodes.next() else {
        return Err(OpticsError::InvalidCount("bioelectric substrate nodes"));
    };
    let mut min = first.position;
    let mut max = first.position;
    for node in nodes {
        min = min.min(node.position);
        max = max.max(node.position);
    }
    Ok((min, max))
}

fn visual_node_radius(bounds_min: Vec3, bounds_max: Vec3) -> f32 {
    let size = bounds_max - bounds_min;
    size.x.max(size.y).max(size.z).max(1.0) * 0.026
}

fn value_range(values: &[f32]) -> Result<(f32, f32), OpticsError> {
    let mut values = values.iter().copied();
    let Some(first) = values.next() else {
        return Err(OpticsError::InvalidCount("bioelectric value range"));
    };
    let mut min_value = first;
    let mut max_value = first;
    for value in values {
        min_value = min_value.min(value);
        max_value = max_value.max(value);
    }
    if !min_value.is_finite() || !max_value.is_finite() {
        return Err(OpticsError::InvalidValue("bioelectric value range"));
    }
    Ok((min_value, max_value))
}

fn normalize_value(value: f32, min_value: f32, max_value: f32) -> f32 {
    let range = (max_value - min_value).max(1.0e-6);
    ((value - min_value) / range).clamp(0.0, 1.0)
}

fn current_kind_label(kind: BioelectricCurrentKind) -> &'static str {
    match kind {
        BioelectricCurrentKind::Leak { .. } => "leak",
        BioelectricCurrentKind::Constant { current } if current >= 0.0 => "source",
        BioelectricCurrentKind::Constant { .. } => "sink",
        BioelectricCurrentKind::Pump { .. } => "pump",
        BioelectricCurrentKind::VoltageGated { .. } => "voltage_gated",
    }
}

fn voltage_color(voltage: f32, normalized: f32) -> ColorRgba {
    if voltage < 0.0 {
        lerp_color(
            ColorRgba::new(0.12, 0.28, 0.30, 0.86),
            ColorRgba::new(0.30, 0.76, 0.70, 0.96),
            1.0 - normalized,
        )
    } else {
        lerp_color(
            ColorRgba::new(0.30, 0.25, 0.16, 0.86),
            ColorRgba::new(0.96, 0.62, 0.26, 0.96),
            normalized,
        )
    }
}

fn conductance_color(normalized: f32, gated: bool) -> ColorRgba {
    let high = if gated {
        ColorRgba::new(0.42, 0.82, 0.53, 0.72)
    } else {
        ColorRgba::new(0.62, 0.68, 0.70, 0.58)
    };
    lerp_color(ColorRgba::new(0.24, 0.29, 0.31, 0.20), high, normalized)
}

fn memory_color(value: f32) -> ColorRgba {
    lerp_color(
        ColorRgba::new(0.18, 0.20, 0.22, 0.32),
        ColorRgba::new(0.94, 0.42, 0.38, 0.78),
        value,
    )
}

fn readout_color(value: f32) -> ColorRgba {
    lerp_color(
        ColorRgba::new(0.14, 0.28, 0.20, 0.72),
        ColorRgba::new(0.50, 0.86, 0.46, 0.94),
        value,
    )
}

fn current_color(kind: BioelectricCurrentKind) -> ColorRgba {
    match kind {
        BioelectricCurrentKind::Leak { .. } => ColorRgba::new(0.68, 0.70, 0.66, 0.18),
        BioelectricCurrentKind::Constant { current } if current >= 0.0 => {
            ColorRgba::new(0.96, 0.56, 0.28, 0.28)
        }
        BioelectricCurrentKind::Constant { .. } => ColorRgba::new(0.30, 0.72, 0.72, 0.28),
        BioelectricCurrentKind::Pump { .. } => ColorRgba::new(0.46, 0.68, 0.52, 0.22),
        BioelectricCurrentKind::VoltageGated { .. } => ColorRgba::new(0.88, 0.68, 0.36, 0.22),
    }
}

fn lerp_color(a: ColorRgba, b: ColorRgba, t: f32) -> ColorRgba {
    let t = t.clamp(0.0, 1.0);
    ColorRgba::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}
