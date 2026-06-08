use rusty_matter_fields::{
    BioelectricCircuitDebugFrame, BioelectricCircuitState, PlanarianAxisMap, PlanarianAxisRegion,
    PlanarianBioelectricScenarioRun, PLANARIAN_BIOELECTRIC_SCENARIO_RUN_SCHEMA_ID,
};
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
    /// Region color.
    pub color: ColorRgba,
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
            region_bands: visual_region_bands(&source.axis_map)?,
            node_regions: visual_node_regions(&source.axis_map)?,
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
        for (expected_index, node_region) in self.node_regions.iter().enumerate() {
            validate_node_region(node_region, expected_index)?;
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
    state.voltage.values = frame.voltage_values.clone();
    if let Some(memory_values) = &frame.memory_values {
        let Some(memory) = state.memory.as_mut() else {
            return Err(OpticsError::InvalidPayload(
                "planarian debug frame contains memory but template does not",
            ));
        };
        memory.values = memory_values.clone();
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
) -> Result<Vec<PlanarianAxisNodeVisualRegion>, OpticsError> {
    axis_map
        .node_regions
        .iter()
        .enumerate()
        .map(|(expected_index, node)| {
            let visual = PlanarianAxisNodeVisualRegion {
                node_index: node.node_index,
                region_id: node.region_id.clone(),
                ap_coordinate: node.ap_coordinate,
                lateral_coordinate: node.lateral_coordinate,
                color: region_color(node.region),
            };
            validate_node_region(&visual, expected_index)?;
            Ok(visual)
        })
        .collect()
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

fn validate_node_region(
    node_region: &PlanarianAxisNodeVisualRegion,
    expected_index: usize,
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
