use rusty_matter_fields::{SurfaceFieldDebugFrame, SURFACE_FIELD_DEBUG_FRAME_SCHEMA_ID};
use rusty_matter_model::Vec3;
use rusty_optics_model::{ColorRgba, OpticsError, SURFACE_FIELD_VISUAL_FRAME_SCHEMA_ID};

/// Renderer-neutral visual node for a surface-field sample.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceFieldVisualNode {
    /// Source node index.
    pub node_index: usize,
    /// Source node identifier.
    pub node_id: String,
    /// Node position.
    pub position: Vec3,
    /// Base point radius in surface units.
    pub radius: f32,
}

/// Renderer-neutral edge visual between surface-field nodes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceFieldVisualEdge {
    /// Source node index.
    pub from: usize,
    /// Target node index.
    pub to: usize,
    /// Neighbor tier, starting at 1.
    pub tier: u8,
    /// Edge color.
    pub color: ColorRgba,
}

/// Scalar color sample for one surface-field node.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceFieldScalarVisualSample {
    /// Source node index.
    pub node_index: usize,
    /// Source scalar value.
    pub value: f32,
    /// Normalized value in the layer range.
    pub normalized_value: f32,
    /// Resolved node color.
    pub color: ColorRgba,
}

/// Scalar visual layer for one Matter scalar field.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceFieldScalarVisualLayer {
    /// Source field identifier.
    pub field_id: String,
    /// Source field kind label.
    pub kind: String,
    /// Minimum scalar value.
    pub min_value: f32,
    /// Maximum scalar value.
    pub max_value: f32,
    /// One visual sample per field node.
    pub samples: Vec<SurfaceFieldScalarVisualSample>,
}

/// Polarity or vector arrow over a surface-field node.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceFieldVectorArrow {
    /// Source node index.
    pub node_index: usize,
    /// Arrow start position.
    pub start: Vec3,
    /// Arrow end position.
    pub end: Vec3,
    /// Source vector length.
    pub magnitude: f32,
    /// Arrow color.
    pub color: ColorRgba,
}

/// Vector visual layer for one Matter vector field.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceFieldVectorVisualLayer {
    /// Source field identifier.
    pub field_id: String,
    /// Source field kind label.
    pub kind: String,
    /// Arrow visuals.
    pub arrows: Vec<SurfaceFieldVectorArrow>,
}

/// Visual perturbation region over field nodes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceFieldPerturbationVisualRegion {
    /// Source perturbation identifier.
    pub perturbation_id: String,
    /// Optional target field identifier.
    pub target_field_id: Option<String>,
    /// Effect label.
    pub effect_kind: String,
    /// Region node indices.
    pub node_indices: Vec<usize>,
    /// Highlight color.
    pub color: ColorRgba,
}

/// Renderer-neutral visual frame for Matter surface-field debug data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceFieldVisualFrame {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable frame identifier.
    pub frame_id: String,
    /// Source Matter field debug frame identifier.
    pub source_frame_id: String,
    /// Source Matter frame schema identifier.
    pub source_schema_id: String,
    /// Source Matter substrate identifier.
    pub substrate_id: String,
    /// Source Matter surface identifier.
    pub surface_id: String,
    /// Bounds minimum for fit-to-view.
    pub bounds_min: Vec3,
    /// Bounds maximum for fit-to-view.
    pub bounds_max: Vec3,
    /// Field nodes.
    pub nodes: Vec<SurfaceFieldVisualNode>,
    /// Neighbor edges.
    pub edges: Vec<SurfaceFieldVisualEdge>,
    /// Scalar field visual layers.
    pub scalar_layers: Vec<SurfaceFieldScalarVisualLayer>,
    /// Vector field visual layers.
    pub vector_layers: Vec<SurfaceFieldVectorVisualLayer>,
    /// Perturbation region visuals.
    pub perturbation_regions: Vec<SurfaceFieldPerturbationVisualRegion>,
}

impl SurfaceFieldVisualFrame {
    /// Creates a renderer-neutral field visual frame.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the source frame or output visuals are
    /// invalid.
    pub fn from_matter_debug_frame(
        frame_id: impl Into<String>,
        source: &SurfaceFieldDebugFrame,
    ) -> Result<Self, OpticsError> {
        if source.schema_id != SURFACE_FIELD_DEBUG_FRAME_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: SURFACE_FIELD_DEBUG_FRAME_SCHEMA_ID,
                actual: source.schema_id.clone(),
            });
        }
        source
            .validate()
            .map_err(|_| OpticsError::InvalidPayload("source surface-field debug frame invalid"))?;
        let (bounds_min, bounds_max) = bounds_from_nodes(source)?;
        let node_radius = visual_node_radius(bounds_min, bounds_max);
        let frame = Self {
            schema_id: SURFACE_FIELD_VISUAL_FRAME_SCHEMA_ID.to_owned(),
            frame_id: frame_id.into(),
            source_frame_id: source.frame_id.clone(),
            source_schema_id: source.schema_id.clone(),
            substrate_id: source.substrate_id.clone(),
            surface_id: source.surface_id.clone(),
            bounds_min,
            bounds_max,
            nodes: source
                .nodes
                .iter()
                .map(|node| SurfaceFieldVisualNode {
                    node_index: node.node_index,
                    node_id: node.node_id.clone(),
                    position: node.position,
                    radius: node_radius,
                })
                .collect(),
            edges: source
                .edges
                .iter()
                .map(|edge| SurfaceFieldVisualEdge {
                    from: edge.from,
                    to: edge.to,
                    tier: edge.tier,
                    color: edge_color(edge.tier),
                })
                .collect(),
            scalar_layers: source
                .scalar_layers
                .iter()
                .map(build_scalar_visual_layer)
                .collect(),
            vector_layers: source
                .vector_layers
                .iter()
                .map(|layer| build_vector_visual_layer(layer, source, node_radius * 4.2))
                .collect::<Result<Vec<_>, _>>()?,
            perturbation_regions: source
                .perturbation_regions
                .iter()
                .map(|region| SurfaceFieldPerturbationVisualRegion {
                    perturbation_id: region.perturbation_id.clone(),
                    target_field_id: region.target_field_id.clone(),
                    effect_kind: region.effect_kind.clone(),
                    node_indices: region.node_indices.clone(),
                    color: perturbation_color(&region.effect_kind),
                })
                .collect(),
        };
        frame.validate()?;
        Ok(frame)
    }

    /// Validates the visual frame.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when IDs, counts, colors, or vectors are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != SURFACE_FIELD_VISUAL_FRAME_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: SURFACE_FIELD_VISUAL_FRAME_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.frame_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("frame_id"));
        }
        if self.source_frame_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_frame_id"));
        }
        if self.source_schema_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_schema_id"));
        }
        if self.substrate_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("substrate_id"));
        }
        if self.surface_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("surface_id"));
        }
        if self.nodes.is_empty() {
            return Err(OpticsError::InvalidCount("nodes"));
        }
        if !self.bounds_min.is_finite() || !self.bounds_max.is_finite() {
            return Err(OpticsError::NonFiniteVec3("bounds"));
        }
        let node_count = self.nodes.len();
        for (expected_index, node) in self.nodes.iter().enumerate() {
            if node.node_index != expected_index || node.node_id.trim().is_empty() {
                return Err(OpticsError::InvalidPayload(
                    "field visual nodes must match node order",
                ));
            }
            if !node.position.is_finite() {
                return Err(OpticsError::NonFiniteVec3("node.position"));
            }
            if !node.radius.is_finite() || node.radius <= 0.0 {
                return Err(OpticsError::InvalidValue("node.radius"));
            }
        }
        for edge in &self.edges {
            if edge.from >= node_count || edge.to >= node_count || edge.from == edge.to {
                return Err(OpticsError::InvalidPayload("field visual edge target"));
            }
            if !(1..=2).contains(&edge.tier) {
                return Err(OpticsError::InvalidValue("field visual edge tier"));
            }
            if !edge.color.is_finite() {
                return Err(OpticsError::NonFiniteColor("field edge"));
            }
        }
        for layer in &self.scalar_layers {
            validate_scalar_layer(layer, node_count)?;
        }
        for layer in &self.vector_layers {
            validate_vector_layer(layer, node_count)?;
        }
        for region in &self.perturbation_regions {
            if region.perturbation_id.trim().is_empty() || region.effect_kind.trim().is_empty() {
                return Err(OpticsError::EmptyId("perturbation_region"));
            }
            if !region.color.is_finite() {
                return Err(OpticsError::NonFiniteColor("perturbation_region"));
            }
            for &node_index in &region.node_indices {
                if node_index >= node_count {
                    return Err(OpticsError::InvalidPayload("perturbation node target"));
                }
            }
        }
        Ok(())
    }
}

fn build_scalar_visual_layer(
    layer: &rusty_matter_fields::SurfaceFieldScalarDebugLayer,
) -> SurfaceFieldScalarVisualLayer {
    let range = (layer.max_value - layer.min_value).max(1.0e-6);
    SurfaceFieldScalarVisualLayer {
        field_id: layer.field_id.clone(),
        kind: layer.kind.clone(),
        min_value: layer.min_value,
        max_value: layer.max_value,
        samples: layer
            .values
            .iter()
            .copied()
            .enumerate()
            .map(|(node_index, value)| {
                let normalized_value = ((value - layer.min_value) / range).clamp(0.0, 1.0);
                SurfaceFieldScalarVisualSample {
                    node_index,
                    value,
                    normalized_value,
                    color: scalar_color(&layer.field_id, &layer.kind, normalized_value),
                }
            })
            .collect(),
    }
}

fn build_vector_visual_layer(
    layer: &rusty_matter_fields::SurfaceFieldVectorDebugLayer,
    source: &SurfaceFieldDebugFrame,
    arrow_scale: f32,
) -> Result<SurfaceFieldVectorVisualLayer, OpticsError> {
    if layer.vectors.len() != source.nodes.len() {
        return Err(OpticsError::InvalidPayload(
            "vector visual layer must match node count",
        ));
    }
    Ok(SurfaceFieldVectorVisualLayer {
        field_id: layer.field_id.clone(),
        kind: layer.kind.clone(),
        arrows: layer
            .vectors
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(node_index, vector)| {
                let magnitude = vector.length();
                (magnitude > 1.0e-6).then(|| {
                    let start = source.nodes[node_index].position;
                    let direction = vector / magnitude;
                    SurfaceFieldVectorArrow {
                        node_index,
                        start,
                        end: start + direction * (arrow_scale * magnitude.clamp(0.25, 1.0)),
                        magnitude,
                        color: vector_color(&layer.field_id, &layer.kind, vector),
                    }
                })
            })
            .collect(),
    })
}

fn bounds_from_nodes(source: &SurfaceFieldDebugFrame) -> Result<(Vec3, Vec3), OpticsError> {
    let mut nodes = source.nodes.iter();
    let Some(first) = nodes.next() else {
        return Err(OpticsError::InvalidCount("source.nodes"));
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

fn validate_scalar_layer(
    layer: &SurfaceFieldScalarVisualLayer,
    node_count: usize,
) -> Result<(), OpticsError> {
    if layer.field_id.trim().is_empty() || layer.kind.trim().is_empty() {
        return Err(OpticsError::EmptyId("scalar_layer"));
    }
    if layer.samples.len() != node_count {
        return Err(OpticsError::InvalidCount("scalar_layer.samples"));
    }
    if !layer.min_value.is_finite()
        || !layer.max_value.is_finite()
        || layer.min_value > layer.max_value
    {
        return Err(OpticsError::InvalidValue("scalar_layer.range"));
    }
    for (expected_index, sample) in layer.samples.iter().enumerate() {
        if sample.node_index != expected_index {
            return Err(OpticsError::InvalidPayload(
                "scalar visual samples must match node order",
            ));
        }
        if !sample.value.is_finite() || !sample.normalized_value.is_finite() {
            return Err(OpticsError::InvalidValue("scalar sample"));
        }
        if !sample.color.is_finite() {
            return Err(OpticsError::NonFiniteColor("scalar sample"));
        }
    }
    Ok(())
}

fn validate_vector_layer(
    layer: &SurfaceFieldVectorVisualLayer,
    node_count: usize,
) -> Result<(), OpticsError> {
    if layer.field_id.trim().is_empty() || layer.kind.trim().is_empty() {
        return Err(OpticsError::EmptyId("vector_layer"));
    }
    for arrow in &layer.arrows {
        if arrow.node_index >= node_count {
            return Err(OpticsError::InvalidPayload("vector arrow node"));
        }
        if !arrow.start.is_finite() || !arrow.end.is_finite() {
            return Err(OpticsError::NonFiniteVec3("vector arrow"));
        }
        if !arrow.magnitude.is_finite() || arrow.magnitude < 0.0 {
            return Err(OpticsError::InvalidValue("vector arrow magnitude"));
        }
        if !arrow.color.is_finite() {
            return Err(OpticsError::NonFiniteColor("vector arrow"));
        }
    }
    Ok(())
}

fn edge_color(tier: u8) -> ColorRgba {
    match tier {
        1 => ColorRgba::new(0.56, 0.62, 0.66, 0.45),
        _ => ColorRgba::new(0.38, 0.43, 0.47, 0.24),
    }
}

fn scalar_color(field_id: &str, kind: &str, value: f32) -> ColorRgba {
    let key = format!("{field_id} {kind}").to_ascii_lowercase();
    if key.contains("wound") {
        lerp_color(
            ColorRgba::new(0.38, 0.18, 0.14, 0.82),
            ColorRgba::new(1.0, 0.36, 0.24, 0.96),
            value,
        )
    } else if key.contains("morphogen") {
        lerp_color(
            ColorRgba::new(0.13, 0.29, 0.20, 0.82),
            ColorRgba::new(0.44, 0.84, 0.48, 0.96),
            value,
        )
    } else {
        lerp_color(
            ColorRgba::new(0.25, 0.24, 0.42, 0.80),
            ColorRgba::new(0.72, 0.55, 0.92, 0.95),
            value,
        )
    }
}

fn vector_color(field_id: &str, kind: &str, vector: Vec3) -> ColorRgba {
    let key = format!("{field_id} {kind}").to_ascii_lowercase();
    if key.contains("polarity") && vector.x < 0.0 {
        ColorRgba::new(1.0, 0.77, 0.25, 0.94)
    } else {
        ColorRgba::new(0.86, 0.88, 0.74, 0.90)
    }
}

fn perturbation_color(effect_kind: &str) -> ColorRgba {
    match effect_kind {
        "wound_region" => ColorRgba::new(1.0, 0.26, 0.18, 0.34),
        "polarity_inversion" => ColorRgba::new(1.0, 0.72, 0.22, 0.34),
        "depolarize_region" => ColorRgba::new(0.78, 0.50, 0.96, 0.30),
        _ => ColorRgba::new(0.86, 0.86, 0.72, 0.24),
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
