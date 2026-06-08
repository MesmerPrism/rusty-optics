use rusty_matter_fields::{
    BioelectricCircuitDebugFrame, BioelectricCircuitState, PlanarianAxisMap, PlanarianAxisRegion,
    PlanarianBioelectricScenarioRun, PLANARIAN_BIOELECTRIC_SCENARIO_RUN_SCHEMA_ID,
};
use rusty_matter_model::Vec3;
use rusty_optics_model::{ColorRgba, OpticsError, PLANARIAN_BIOELECTRIC_VISUAL_SEQUENCE_SCHEMA_ID};

use crate::BioelectricCircuitVisualFrame;

/// Visual style for one planarian AP region band.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PlanarianAxisRegionVisualBand {
    /// Stable atlas-style region identifier.
    pub region_id: String,
    /// Human-readable region label.
    pub label: String,
    /// Minimum normalized AP z coordinate.
    pub z_min: f32,
    /// Maximum normalized AP z coordinate.
    pub z_max: f32,
    /// Region fill color for renderer-neutral previews.
    pub color: ColorRgba,
}

/// Visual region assignment for one planarian surface node.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PlanarianAxisNodeVisualRegion {
    /// Surface node index.
    pub node_index: usize,
    /// Stable atlas-style region identifier.
    pub region_id: String,
    /// Normalized AP coordinate in 0..=1, posterior to anterior.
    pub ap_coordinate: f32,
    /// Lateral coordinate normalized by local half-width.
    pub lateral_coordinate: f32,
    /// Source surface anchor for this sampled Matter node, when available.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub surface_anchor: Option<PlanarianSurfaceNodeAnchor>,
    /// Region color.
    pub color: ColorRgba,
}

/// Source body-surface anchor for a sampled Matter node.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanarianSurfaceNodeAnchor {
    /// Source body triangle index.
    pub triangle_index: usize,
    /// Barycentric weights over the source triangle vertices.
    pub barycentric: [f32; 3],
}

/// One renderer-neutral planarian body-surface vertex.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PlanarianBodySurfaceVisualVertex {
    /// Source mesh vertex index.
    pub vertex_index: usize,
    /// Vertex position in Matter source coordinates.
    pub position: Vec3,
    /// AP region identifier assigned from the Matter axis bands.
    pub region_id: String,
    /// Region color.
    pub color: ColorRgba,
}

/// One renderer-neutral planarian body-surface triangle.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PlanarianBodySurfaceVisualTriangle {
    /// Source mesh triangle index.
    pub triangle_index: usize,
    /// Source mesh vertex indices.
    pub vertex_indices: [u32; 3],
    /// AP region identifier assigned from the triangle centroid.
    pub region_id: String,
    /// Region color.
    pub color: ColorRgba,
}

/// Renderer-neutral planarian body-surface visual contract.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PlanarianBodySurfaceVisual {
    /// Source Matter surface identifier.
    pub surface_id: String,
    /// Source topology index hash.
    pub topology_index_hash: u64,
    /// Minimum body position bound.
    pub bounds_min: Vec3,
    /// Maximum body position bound.
    pub bounds_max: Vec3,
    /// Source mesh vertices with visual region metadata.
    pub vertices: Vec<PlanarianBodySurfaceVisualVertex>,
    /// Source mesh triangles with visual region metadata.
    pub triangles: Vec<PlanarianBodySurfaceVisualTriangle>,
}

/// Renderer-neutral visual sequence for a synthetic planarian AP bioelectric run.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PlanarianBioelectricVisualSequence {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable visual sequence identifier.
    pub sequence_id: String,
    /// Source Matter planarian run identifier.
    pub source_run_id: String,
    /// Source Matter schema identifier.
    pub source_schema_id: String,
    /// Source Matter scenario identifier.
    pub scenario_id: String,
    /// Scenario kind label.
    pub scenario_kind: String,
    /// Evidence type copied from Matter.
    pub evidence_type: String,
    /// Expected qualitative behavior copied from Matter.
    pub expected_outcome: String,
    /// Source substrate identifier.
    pub substrate_id: String,
    /// Source surface identifier.
    pub surface_id: String,
    /// Fixed step duration in seconds.
    pub fixed_step_seconds: f32,
    /// Total executed fixed steps.
    pub step_count: u32,
    /// Step interval between visual frames.
    pub frame_stride: u32,
    /// Number of source per-step diagnostics available in Matter.
    pub diagnostic_count: usize,
    /// Compact literature/design anchors copied from Matter.
    pub literature_anchors: Vec<String>,
    /// Matter-owned body surface prepared as a renderer-neutral 3D visual payload.
    pub body_surface: PlanarianBodySurfaceVisual,
    /// AP region bands with Optics-owned colors.
    pub region_bands: Vec<PlanarianAxisRegionVisualBand>,
    /// One AP visual region assignment per node.
    pub node_regions: Vec<PlanarianAxisNodeVisualRegion>,
    /// Circuit visual frames over the Matter sequence frames.
    pub frames: Vec<BioelectricCircuitVisualFrame>,
}

impl PlanarianBioelectricVisualSequence {
    /// Creates a planarian visual sequence from a validated Matter scenario run.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the Matter run or generated visuals are
    /// invalid.
    pub fn from_matter_planarian_run(
        sequence_id: impl Into<String>,
        source: &PlanarianBioelectricScenarioRun,
    ) -> Result<Self, OpticsError> {
        if source.schema_id != PLANARIAN_BIOELECTRIC_SCENARIO_RUN_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PLANARIAN_BIOELECTRIC_SCENARIO_RUN_SCHEMA_ID,
                actual: source.schema_id.clone(),
            });
        }
        source
            .validate()
            .map_err(|_| OpticsError::InvalidPayload("source planarian run invalid"))?;
        let sequence_id = sequence_id.into();
        let frames = source
            .sequence
            .frames
            .iter()
            .map(|frame| {
                visual_frame_from_debug_frame(
                    &sequence_id,
                    &source.initial_circuit,
                    &source.substrate,
                    frame,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let sequence = Self {
            schema_id: PLANARIAN_BIOELECTRIC_VISUAL_SEQUENCE_SCHEMA_ID.to_owned(),
            sequence_id,
            source_run_id: source.run_id.clone(),
            source_schema_id: source.schema_id.clone(),
            scenario_id: source.scenario_id.clone(),
            scenario_kind: format!("{:?}", source.scenario_kind),
            evidence_type: source.evidence_type.clone(),
            expected_outcome: source.expected_outcome.clone(),
            substrate_id: source.substrate.substrate_id.clone(),
            surface_id: source.substrate.surface_id.clone(),
            fixed_step_seconds: source.sequence.fixed_step_seconds,
            step_count: source.sequence.step_count,
            frame_stride: source.sequence.frame_stride,
            diagnostic_count: source.sequence.diagnostics.len(),
            literature_anchors: source.literature_anchors.clone(),
            body_surface: visual_body_surface(source)?,
            region_bands: visual_region_bands(&source.axis_map)?,
            node_regions: visual_node_regions(
                &source.axis_map,
                &source.substrate,
                source.source_surface.triangles.len(),
            )?,
            frames,
        };
        sequence.validate()?;
        Ok(sequence)
    }

    /// Validates the planarian visual sequence.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when IDs, timing, regions, or frames are
    /// invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PLANARIAN_BIOELECTRIC_VISUAL_SEQUENCE_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PLANARIAN_BIOELECTRIC_VISUAL_SEQUENCE_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.sequence_id.trim().is_empty()
            || self.source_run_id.trim().is_empty()
            || self.source_schema_id.trim().is_empty()
            || self.scenario_id.trim().is_empty()
            || self.scenario_kind.trim().is_empty()
            || self.evidence_type.trim().is_empty()
            || self.expected_outcome.trim().is_empty()
        {
            return Err(OpticsError::EmptyId("planarian visual sequence metadata"));
        }
        if self.substrate_id.trim().is_empty() || self.surface_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("planarian visual source"));
        }
        if self.body_surface.surface_id != self.surface_id {
            return Err(OpticsError::InvalidPayload(
                "planarian body surface id must match sequence surface id",
            ));
        }
        if !self.fixed_step_seconds.is_finite() || self.fixed_step_seconds <= 0.0 {
            return Err(OpticsError::InvalidValue(
                "planarian visual fixed_step_seconds",
            ));
        }
        if self.frame_stride == 0 || self.frames.is_empty() {
            return Err(OpticsError::InvalidCount("planarian visual frames"));
        }
        if self.region_bands.len() != PlanarianAxisRegion::all().len() {
            return Err(OpticsError::InvalidCount("planarian region bands"));
        }
        let node_count = self.frames[0].nodes.len();
        if self.node_regions.len() != node_count {
            return Err(OpticsError::InvalidCount("planarian node regions"));
        }
        for band in &self.region_bands {
            validate_region_band(band)?;
        }
        validate_body_surface(&self.body_surface)?;
        for (expected_index, node_region) in self.node_regions.iter().enumerate() {
            validate_node_region(
                node_region,
                expected_index,
                self.body_surface.triangles.len(),
            )?;
        }
        let mut previous_step = None::<u32>;
        for frame in &self.frames {
            frame.validate()?;
            if frame.substrate_id != self.substrate_id || frame.surface_id != self.surface_id {
                return Err(OpticsError::InvalidPayload(
                    "planarian visual frame source must match sequence",
                ));
            }
            if frame.step_index() > self.step_count {
                return Err(OpticsError::InvalidValue("planarian visual frame step"));
            }
            if previous_step.is_some_and(|step| frame.step_index() <= step) {
                return Err(OpticsError::InvalidPayload(
                    "planarian visual frame steps must be increasing",
                ));
            }
            previous_step = Some(frame.step_index());
        }
        if self.literature_anchors.is_empty()
            || self
                .literature_anchors
                .iter()
                .any(|anchor| anchor.trim().is_empty())
        {
            return Err(OpticsError::InvalidPayload(
                "planarian visual sequence must preserve source anchors",
            ));
        }
        Ok(())
    }
}

fn visual_body_surface(
    source: &PlanarianBioelectricScenarioRun,
) -> Result<PlanarianBodySurfaceVisual, OpticsError> {
    source
        .source_surface
        .validate()
        .map_err(|_| OpticsError::InvalidPayload("planarian source body surface invalid"))?;
    let topology_key = source.source_surface.topology_key();
    let (bounds_min, bounds_max) = bounds_for_positions(&source.source_surface.positions)?;
    let vertices = source
        .source_surface
        .positions
        .iter()
        .copied()
        .enumerate()
        .map(|(vertex_index, position)| {
            let region = region_for_z(&source.axis_map, position.z)?;
            let vertex = PlanarianBodySurfaceVisualVertex {
                vertex_index,
                position,
                region_id: region.region_id().to_owned(),
                color: region_color(region),
            };
            validate_body_vertex(&vertex, vertex_index)?;
            Ok(vertex)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let triangles = source
        .source_surface
        .triangles
        .iter()
        .copied()
        .enumerate()
        .map(|(triangle_index, vertex_indices)| {
            let centroid_z = triangle_centroid_z(&source.source_surface.positions, vertex_indices)?;
            let region = region_for_z(&source.axis_map, centroid_z)?;
            let triangle = PlanarianBodySurfaceVisualTriangle {
                triangle_index,
                vertex_indices,
                region_id: region.region_id().to_owned(),
                color: region_color(region),
            };
            validate_body_triangle(&triangle, triangle_index, vertices.len())?;
            Ok(triangle)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let visual = PlanarianBodySurfaceVisual {
        surface_id: source.source_surface.surface_id.clone(),
        topology_index_hash: topology_key.index_hash,
        bounds_min,
        bounds_max,
        vertices,
        triangles,
    };
    validate_body_surface(&visual)?;
    Ok(visual)
}

trait VisualFrameStep {
    fn step_index(&self) -> u32;
}

impl VisualFrameStep for BioelectricCircuitVisualFrame {
    fn step_index(&self) -> u32 {
        self.diagnostics
            .as_ref()
            .map_or(0, |diagnostics| diagnostics.step_index)
    }
}

fn visual_frame_from_debug_frame(
    sequence_id: &str,
    template: &BioelectricCircuitState,
    substrate: &rusty_matter_fields::SurfaceFieldSubstrate,
    frame: &BioelectricCircuitDebugFrame,
) -> Result<BioelectricCircuitVisualFrame, OpticsError> {
    let mut state = template.clone();
    state.time_seconds = frame.time_seconds;
    state.voltage.values.clone_from(&frame.voltage_values);
    if let Some(memory_values) = &frame.memory_values {
        let Some(memory) = state.memory.as_mut() else {
            return Err(OpticsError::InvalidPayload(
                "planarian debug frame contains memory but template does not",
            ));
        };
        memory.values.clone_from(memory_values);
    }
    for readout in &mut state.readout_layers {
        let Some(source_layer) = frame
            .readout_layers
            .iter()
            .find(|layer| layer.layer_id == readout.layer_id)
        else {
            return Err(OpticsError::InvalidPayload(
                "planarian debug frame readout missing from template",
            ));
        };
        readout.values = source_layer.values.clone();
    }
    state
        .validate()
        .map_err(|_| OpticsError::InvalidPayload("planarian visual circuit state invalid"))?;
    let step = frame.step_index;
    BioelectricCircuitVisualFrame::from_matter_circuit_state(
        format!("{sequence_id}.frame.{step:04}"),
        substrate,
        &state,
        frame.diagnostics.as_ref(),
    )
}

fn visual_region_bands(
    axis_map: &PlanarianAxisMap,
) -> Result<Vec<PlanarianAxisRegionVisualBand>, OpticsError> {
    axis_map
        .bands
        .iter()
        .map(|band| {
            let visual = PlanarianAxisRegionVisualBand {
                region_id: band.region_id.clone(),
                label: band.label.clone(),
                z_min: band.z_min,
                z_max: band.z_max,
                color: region_color(band.region),
            };
            validate_region_band(&visual)?;
            Ok(visual)
        })
        .collect()
}

fn visual_node_regions(
    axis_map: &PlanarianAxisMap,
    substrate: &rusty_matter_fields::SurfaceFieldSubstrate,
    triangle_count: usize,
) -> Result<Vec<PlanarianAxisNodeVisualRegion>, OpticsError> {
    axis_map
        .node_regions
        .iter()
        .enumerate()
        .map(|(expected_index, node)| {
            let Some(substrate_node) = substrate.nodes.get(expected_index) else {
                return Err(OpticsError::InvalidPayload("planarian node region anchor"));
            };
            if substrate_node.node_index != node.node_index {
                return Err(OpticsError::InvalidPayload(
                    "planarian node region anchor index",
                ));
            }
            let surface_anchor = Some(PlanarianSurfaceNodeAnchor {
                triangle_index: substrate_node.triangle_index,
                barycentric: substrate_node.barycentric,
            });
            let visual = PlanarianAxisNodeVisualRegion {
                node_index: node.node_index,
                region_id: node.region_id.clone(),
                ap_coordinate: node.ap_coordinate,
                lateral_coordinate: node.lateral_coordinate,
                surface_anchor,
                color: region_color(node.region),
            };
            validate_node_region(&visual, expected_index, triangle_count)?;
            Ok(visual)
        })
        .collect()
}

fn bounds_for_positions(positions: &[Vec3]) -> Result<(Vec3, Vec3), OpticsError> {
    let mut bounds = positions.iter().copied();
    let Some(first) = bounds.next() else {
        return Err(OpticsError::InvalidCount("planarian body vertices"));
    };
    if !first.is_finite() {
        return Err(OpticsError::NonFiniteVec3("planarian body vertex"));
    }
    let mut bounds_min = first;
    let mut bounds_max = first;
    for position in bounds {
        if !position.is_finite() {
            return Err(OpticsError::NonFiniteVec3("planarian body vertex"));
        }
        bounds_min = bounds_min.min(position);
        bounds_max = bounds_max.max(position);
    }
    if !bounds_min.is_finite() || !bounds_max.is_finite() {
        return Err(OpticsError::NonFiniteVec3("planarian body bounds"));
    }
    Ok((bounds_min, bounds_max))
}

fn triangle_centroid_z(positions: &[Vec3], indices: [u32; 3]) -> Result<f32, OpticsError> {
    let [a, b, c] = indices;
    let a = usize::try_from(a).map_err(|_| OpticsError::InvalidPayload("planarian triangle"))?;
    let b = usize::try_from(b).map_err(|_| OpticsError::InvalidPayload("planarian triangle"))?;
    let c = usize::try_from(c).map_err(|_| OpticsError::InvalidPayload("planarian triangle"))?;
    let Some(a) = positions.get(a) else {
        return Err(OpticsError::InvalidPayload("planarian triangle vertex"));
    };
    let Some(b) = positions.get(b) else {
        return Err(OpticsError::InvalidPayload("planarian triangle vertex"));
    };
    let Some(c) = positions.get(c) else {
        return Err(OpticsError::InvalidPayload("planarian triangle vertex"));
    };
    let centroid_z = (a.z + b.z + c.z) / 3.0;
    if !centroid_z.is_finite() {
        return Err(OpticsError::InvalidValue("planarian triangle centroid"));
    }
    Ok(centroid_z)
}

fn region_for_z(axis_map: &PlanarianAxisMap, z: f32) -> Result<PlanarianAxisRegion, OpticsError> {
    if !z.is_finite() {
        return Err(OpticsError::InvalidValue("planarian AP z"));
    }
    let final_index = axis_map.bands.len().saturating_sub(1);
    axis_map
        .bands
        .iter()
        .enumerate()
        .find_map(|(index, band)| {
            (z >= band.z_min && (z < band.z_max || (index == final_index && z <= band.z_max)))
                .then_some(band.region)
        })
        .ok_or(OpticsError::InvalidPayload(
            "planarian body vertex outside AP bands",
        ))
}

fn validate_region_band(band: &PlanarianAxisRegionVisualBand) -> Result<(), OpticsError> {
    if band.region_id.trim().is_empty() || band.label.trim().is_empty() {
        return Err(OpticsError::EmptyId("planarian region band"));
    }
    if !band.z_min.is_finite() || !band.z_max.is_finite() || band.z_min >= band.z_max {
        return Err(OpticsError::InvalidValue("planarian region band z range"));
    }
    if !band.color.is_finite() {
        return Err(OpticsError::NonFiniteColor("planarian region band"));
    }
    Ok(())
}

fn validate_body_surface(surface: &PlanarianBodySurfaceVisual) -> Result<(), OpticsError> {
    if surface.surface_id.trim().is_empty() {
        return Err(OpticsError::EmptyId("planarian body surface"));
    }
    if surface.vertices.is_empty() || surface.triangles.is_empty() {
        return Err(OpticsError::InvalidCount("planarian body surface"));
    }
    if !surface.bounds_min.is_finite() || !surface.bounds_max.is_finite() {
        return Err(OpticsError::NonFiniteVec3("planarian body bounds"));
    }
    for (expected_index, vertex) in surface.vertices.iter().enumerate() {
        validate_body_vertex(vertex, expected_index)?;
    }
    for (expected_index, triangle) in surface.triangles.iter().enumerate() {
        validate_body_triangle(triangle, expected_index, surface.vertices.len())?;
    }
    Ok(())
}

fn validate_body_vertex(
    vertex: &PlanarianBodySurfaceVisualVertex,
    expected_index: usize,
) -> Result<(), OpticsError> {
    if vertex.vertex_index != expected_index || vertex.region_id.trim().is_empty() {
        return Err(OpticsError::InvalidPayload("planarian body vertex"));
    }
    if !vertex.position.is_finite() {
        return Err(OpticsError::NonFiniteVec3("planarian body vertex"));
    }
    if !vertex.color.is_finite() {
        return Err(OpticsError::NonFiniteColor("planarian body vertex"));
    }
    Ok(())
}

fn validate_body_triangle(
    triangle: &PlanarianBodySurfaceVisualTriangle,
    expected_index: usize,
    vertex_count: usize,
) -> Result<(), OpticsError> {
    if triangle.triangle_index != expected_index || triangle.region_id.trim().is_empty() {
        return Err(OpticsError::InvalidPayload("planarian body triangle"));
    }
    let mut seen = [usize::MAX; 3];
    for (slot, vertex_index) in triangle.vertex_indices.iter().copied().enumerate() {
        let as_usize = usize::try_from(vertex_index)
            .map_err(|_| OpticsError::InvalidPayload("planarian body triangle target"))?;
        if as_usize >= vertex_count || seen[..slot].contains(&as_usize) {
            return Err(OpticsError::InvalidPayload(
                "planarian body triangle target",
            ));
        }
        seen[slot] = as_usize;
    }
    if !triangle.color.is_finite() {
        return Err(OpticsError::NonFiniteColor("planarian body triangle"));
    }
    Ok(())
}

fn validate_node_region(
    node_region: &PlanarianAxisNodeVisualRegion,
    expected_index: usize,
    triangle_count: usize,
) -> Result<(), OpticsError> {
    if node_region.node_index != expected_index || node_region.region_id.trim().is_empty() {
        return Err(OpticsError::InvalidPayload("planarian node region"));
    }
    if !node_region.ap_coordinate.is_finite()
        || !(0.0..=1.0).contains(&node_region.ap_coordinate)
        || !node_region.lateral_coordinate.is_finite()
    {
        return Err(OpticsError::InvalidValue("planarian node region"));
    }
    if !node_region.color.is_finite() {
        return Err(OpticsError::NonFiniteColor("planarian node region"));
    }
    if let Some(anchor) = &node_region.surface_anchor {
        validate_surface_anchor(anchor, triangle_count, "planarian node region anchor")?;
    }
    Ok(())
}

pub(crate) fn validate_surface_anchor(
    anchor: &PlanarianSurfaceNodeAnchor,
    triangle_count: usize,
    label: &'static str,
) -> Result<(), OpticsError> {
    if anchor.triangle_index >= triangle_count {
        return Err(OpticsError::InvalidPayload(label));
    }
    validate_surface_anchor_shape(anchor, label)
}

pub(crate) fn validate_surface_anchor_shape(
    anchor: &PlanarianSurfaceNodeAnchor,
    label: &'static str,
) -> Result<(), OpticsError> {
    if !anchor
        .barycentric
        .iter()
        .all(|value| value.is_finite() && *value >= -1.0e-5 && *value <= 1.0 + 1.0e-5)
    {
        return Err(OpticsError::InvalidValue(label));
    }
    let sum = anchor.barycentric.iter().sum::<f32>();
    if (sum - 1.0).abs() > 1.0e-4 {
        return Err(OpticsError::InvalidValue(label));
    }
    Ok(())
}

fn region_color(region: PlanarianAxisRegion) -> ColorRgba {
    match region {
        PlanarianAxisRegion::Tail => ColorRgba::new(0.24, 0.54, 0.58, 0.30),
        PlanarianAxisRegion::PostpharyngealTrunk => ColorRgba::new(0.28, 0.62, 0.48, 0.25),
        PlanarianAxisRegion::PharyngealTrunk => ColorRgba::new(0.70, 0.64, 0.34, 0.23),
        PlanarianAxisRegion::PrepharyngealTrunk => ColorRgba::new(0.74, 0.48, 0.32, 0.25),
        PlanarianAxisRegion::Head => ColorRgba::new(0.78, 0.38, 0.34, 0.30),
    }
}
