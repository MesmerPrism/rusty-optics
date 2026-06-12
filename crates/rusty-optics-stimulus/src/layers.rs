use rusty_optics_model::{ColorRgba, OpticsError, Vec2, STIMULUS_LAYER_GRAPH_SCHEMA_ID};

use crate::{noise::NoiseControls, oscillator::StimulusOscillator};

/// Procedural base-pattern family for a stimulus layer.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BasePatternKind {
    /// Parallel light/dark bands.
    Stripes,
    /// Concentric rings around the layer center.
    Rings,
    /// Angular wedges around the layer center.
    Rays,
    /// Orthogonal square alternation.
    Checkerboard,
    /// Deterministic seeded cell noise.
    NoiseField,
    /// Perlin-style gradient fBm noise.
    PerlinNoise,
    /// Traveling ripple field from a layer-local source point.
    Ripple,
    /// Two-source wave interference field.
    Interference,
}

/// How a layer folds normalized coordinates before sampling.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MirrorMode {
    /// Leave coordinates unchanged.
    None,
    /// Mirror across the vertical center line.
    Horizontal,
    /// Mirror across the horizontal center line.
    Vertical,
    /// Mirror across both center axes.
    Axes,
    /// Fold into three repeated angular sectors.
    Kaleidoscope3,
    /// Fold into six repeated angular sectors.
    Kaleidoscope6,
}

impl Default for MirrorMode {
    fn default() -> Self {
        Self::None
    }
}

/// Layer blend policy used by CPU references and renderer adapters.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StimulusLayerBlendMode {
    /// Weighted additive contribution.
    Add,
    /// Multiply the accumulated value by this layer.
    Multiply,
    /// Keep the brighter of the accumulated value and this layer.
    Max,
}

impl Default for StimulusLayerBlendMode {
    fn default() -> Self {
        Self::Add
    }
}

/// Per-layer spatial warp controls.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WarpControls {
    /// Coordinate scale multiplier.
    pub scale: Vec2,
    /// Coordinate offset applied before rotation.
    pub offset: Vec2,
    /// Twist amount in radians per normalized radius.
    pub twist: f32,
    /// Radial pinch amount.
    pub pinch: f32,
    /// X shear as a function of Y.
    pub shear_x: f32,
    /// Y shear as a function of X.
    pub shear_y: f32,
}

impl Default for WarpControls {
    fn default() -> Self {
        Self {
            scale: Vec2::ONE,
            offset: Vec2::ZERO,
            twist: 0.0,
            pinch: 0.0,
            shear_x: 0.0,
            shear_y: 0.0,
        }
    }
}

impl WarpControls {
    /// Validates finite warp controls.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when any field is non-finite or scale is zero.
    pub fn validate(self) -> Result<(), OpticsError> {
        if !self.scale.is_finite() || self.scale.x == 0.0 || self.scale.y == 0.0 {
            return Err(OpticsError::NonFiniteVec2("warp.scale"));
        }
        if !self.offset.is_finite() {
            return Err(OpticsError::NonFiniteVec2("warp.offset"));
        }
        validate_finite("warp.twist", self.twist)?;
        validate_finite("warp.pinch", self.pinch)?;
        validate_finite("warp.shear_x", self.shear_x)?;
        validate_finite("warp.shear_y", self.shear_y)?;
        Ok(())
    }
}

/// Time binding for one layer inside a stimulus graph.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerTemporalBinding {
    /// Layer-local speed in cycles per second.
    pub speed_hz: f32,
    /// Phase offset added to the global temporal profile.
    pub phase_offset: f32,
    /// Modulation amplitude applied by adapters.
    pub amplitude: f32,
}

impl Default for LayerTemporalBinding {
    fn default() -> Self {
        Self {
            speed_hz: 0.0,
            phase_offset: 0.0,
            amplitude: 1.0,
        }
    }
}

impl LayerTemporalBinding {
    /// Validates finite layer temporal controls.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are non-finite or negative where
    /// only non-negative values are valid.
    pub fn validate(self) -> Result<(), OpticsError> {
        validate_non_negative("layer_temporal.speed_hz", self.speed_hz)?;
        validate_finite("layer_temporal.phase_offset", self.phase_offset)?;
        validate_non_negative("layer_temporal.amplitude", self.amplitude)?;
        Ok(())
    }
}

/// Source geometry and shaping controls for ripple/interference layers.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InterferenceControls {
    /// Primary source in centered layer coordinates.
    pub source_a: Vec2,
    /// Secondary source in centered layer coordinates.
    pub source_b: Vec2,
    /// Relative contribution of the secondary source.
    pub source_b_weight: f32,
    /// Radial decay strength. Zero disables decay.
    pub radial_decay: f32,
    /// Additional low-frequency wave modulation in `0..1`.
    pub wave_modulation: f32,
}

impl Default for InterferenceControls {
    fn default() -> Self {
        Self {
            source_a: Vec2::new(-0.24, 0.0),
            source_b: Vec2::new(0.24, 0.0),
            source_b_weight: 1.0,
            radial_decay: 0.0,
            wave_modulation: 0.0,
        }
    }
}

impl InterferenceControls {
    /// Validates finite source and shaping controls.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(self) -> Result<(), OpticsError> {
        if !self.source_a.is_finite() {
            return Err(OpticsError::NonFiniteVec2("interference.source_a"));
        }
        if !self.source_b.is_finite() {
            return Err(OpticsError::NonFiniteVec2("interference.source_b"));
        }
        validate_non_negative("interference.source_b_weight", self.source_b_weight)?;
        validate_non_negative("interference.radial_decay", self.radial_decay)?;
        validate_unit("interference.wave_modulation", self.wave_modulation)?;
        Ok(())
    }
}

/// One color ramp stop for a procedural stimulus graph.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorStop {
    /// Ramp position in `0..1`.
    pub position: f32,
    /// Linear color at this stop.
    pub color: ColorRgba,
}

impl ColorStop {
    /// Creates a color stop.
    #[must_use]
    pub const fn new(position: f32, color: ColorRgba) -> Self {
        Self { position, color }
    }

    /// Validates stop bounds and finite color.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when stop data is invalid.
    pub fn validate(self) -> Result<(), OpticsError> {
        if !self.position.is_finite() || !(0.0..=1.0).contains(&self.position) {
            return Err(OpticsError::InvalidValue("color_stop.position"));
        }
        if !self.color.is_finite() {
            return Err(OpticsError::NonFiniteColor("color_stop.color"));
        }
        Ok(())
    }
}

/// Post-processing policy requested by a stimulus graph.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PostProcessingPolicy {
    /// Contrast multiplier around mid-gray.
    pub contrast: f32,
    /// Brightness offset after contrast.
    pub brightness: f32,
    /// Edge fade amount in `0..1`.
    pub edge_fade: f32,
    /// Center fade amount in `0..1`.
    pub center_fade: f32,
    /// Requested blur radius in pixels.
    pub blur_radius_px: f32,
    /// Amount of deterministic output noise in `0..1`.
    pub noise_amount: f32,
    /// History decay used by trail-capable adapters in `0..1`.
    pub trail_decay: f32,
}

impl Default for PostProcessingPolicy {
    fn default() -> Self {
        Self {
            contrast: 1.0,
            brightness: 0.0,
            edge_fade: 0.0,
            center_fade: 0.0,
            blur_radius_px: 0.0,
            noise_amount: 0.0,
            trail_decay: 0.0,
        }
    }
}

impl PostProcessingPolicy {
    /// Validates post-processing bounds.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(self) -> Result<(), OpticsError> {
        validate_non_negative("post.contrast", self.contrast)?;
        validate_finite("post.brightness", self.brightness)?;
        validate_unit("post.edge_fade", self.edge_fade)?;
        validate_unit("post.center_fade", self.center_fade)?;
        validate_non_negative("post.blur_radius_px", self.blur_radius_px)?;
        validate_unit("post.noise_amount", self.noise_amount)?;
        validate_unit("post.trail_decay", self.trail_decay)?;
        Ok(())
    }
}

/// One procedural layer inside a stimulus graph.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StimulusLayer {
    /// Stable layer identifier.
    pub layer_id: String,
    /// Base pattern sampled by this layer.
    pub pattern: BasePatternKind,
    /// Blend policy.
    pub blend_mode: StimulusLayerBlendMode,
    /// Layer contribution weight.
    pub weight: f32,
    /// Layer opacity in `0..1`.
    pub opacity: f32,
    /// Pattern frequency in normalized coordinate space.
    pub spatial_frequency: f32,
    /// Layer rotation in radians.
    pub rotation_radians: f32,
    /// Static pattern phase offset.
    pub phase_offset: f32,
    /// Deterministic seed used by noise-like patterns.
    pub seed: u32,
    /// Coordinate mirroring mode.
    pub mirror_mode: MirrorMode,
    /// Spatial warp controls.
    pub warp: WarpControls,
    /// Noise controls used by noise-like patterns and adapter caches.
    #[cfg_attr(feature = "serde", serde(default))]
    pub noise: NoiseControls,
    /// Ripple/interference source controls.
    #[cfg_attr(feature = "serde", serde(default))]
    pub interference: InterferenceControls,
    /// Layer-local temporal binding.
    pub temporal: LayerTemporalBinding,
    /// Layer-local oscillator modulation bindings.
    #[cfg_attr(feature = "serde", serde(default))]
    pub oscillators: Vec<StimulusOscillator>,
    /// Color ramp stops from dark to bright.
    pub colors: Vec<ColorStop>,
}

impl StimulusLayer {
    /// Creates a stripe layer with a black-to-white ramp.
    #[must_use]
    pub fn stripes(layer_id: impl Into<String>, spatial_frequency: f32) -> Self {
        Self::new(layer_id, BasePatternKind::Stripes, spatial_frequency)
    }

    /// Creates a ring layer with a black-to-white ramp.
    #[must_use]
    pub fn rings(layer_id: impl Into<String>, spatial_frequency: f32) -> Self {
        Self::new(layer_id, BasePatternKind::Rings, spatial_frequency)
    }

    /// Creates a ray layer with a black-to-white ramp.
    #[must_use]
    pub fn rays(layer_id: impl Into<String>, spatial_frequency: f32) -> Self {
        Self::new(layer_id, BasePatternKind::Rays, spatial_frequency)
    }

    /// Creates a ripple layer with a black-to-white ramp.
    #[must_use]
    pub fn ripple(layer_id: impl Into<String>, spatial_frequency: f32) -> Self {
        Self::new(layer_id, BasePatternKind::Ripple, spatial_frequency)
    }

    /// Creates a two-source interference layer with a black-to-white ramp.
    #[must_use]
    pub fn interference(layer_id: impl Into<String>, spatial_frequency: f32) -> Self {
        Self::new(layer_id, BasePatternKind::Interference, spatial_frequency)
    }

    /// Creates a Perlin-style noise layer with a black-to-white ramp.
    #[must_use]
    pub fn perlin_noise(layer_id: impl Into<String>, spatial_frequency: f32) -> Self {
        let mut layer = Self::new(layer_id, BasePatternKind::PerlinNoise, spatial_frequency);
        layer.noise = NoiseControls::perlin(4);
        layer
    }

    /// Creates a generic base-pattern layer.
    #[must_use]
    pub fn new(
        layer_id: impl Into<String>,
        pattern: BasePatternKind,
        spatial_frequency: f32,
    ) -> Self {
        Self {
            layer_id: layer_id.into(),
            pattern,
            blend_mode: StimulusLayerBlendMode::Add,
            weight: 1.0,
            opacity: 1.0,
            spatial_frequency,
            rotation_radians: 0.0,
            phase_offset: 0.0,
            seed: 1,
            mirror_mode: MirrorMode::None,
            warp: WarpControls::default(),
            noise: NoiseControls::default(),
            interference: InterferenceControls::default(),
            temporal: LayerTemporalBinding::default(),
            oscillators: Vec::new(),
            colors: vec![
                ColorStop::new(0.0, ColorRgba::new(0.0, 0.0, 0.0, 1.0)),
                ColorStop::new(1.0, ColorRgba::new(1.0, 1.0, 1.0, 1.0)),
            ],
        }
    }

    /// Validates layer shape and numeric bounds.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.layer_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("layer_id"));
        }
        validate_finite("layer.weight", self.weight)?;
        validate_unit("layer.opacity", self.opacity)?;
        validate_non_negative("layer.spatial_frequency", self.spatial_frequency)?;
        if self.spatial_frequency == 0.0 {
            return Err(OpticsError::InvalidValue("layer.spatial_frequency"));
        }
        validate_finite("layer.rotation_radians", self.rotation_radians)?;
        validate_finite("layer.phase_offset", self.phase_offset)?;
        self.warp.validate()?;
        self.noise.validate()?;
        self.interference.validate()?;
        self.temporal.validate()?;
        if self.oscillators.len() > 8 {
            return Err(OpticsError::InvalidCount("layer.oscillators"));
        }
        for oscillator in &self.oscillators {
            oscillator.validate()?;
        }
        if self.colors.len() < 2 {
            return Err(OpticsError::InvalidCount("layer.colors"));
        }
        for color in &self.colors {
            color.validate()?;
        }
        Ok(())
    }
}

/// Renderer-neutral procedural layer graph.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StimulusLayerGraph {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable graph identifier.
    pub graph_id: String,
    /// Layer stack in back-to-front order.
    pub layers: Vec<StimulusLayer>,
    /// Post-processing requested after layer composition.
    pub post: PostProcessingPolicy,
}

impl StimulusLayerGraph {
    /// Creates an empty layer graph.
    #[must_use]
    pub fn new(graph_id: impl Into<String>) -> Self {
        Self {
            schema_id: STIMULUS_LAYER_GRAPH_SCHEMA_ID.to_owned(),
            graph_id: graph_id.into(),
            layers: Vec::new(),
            post: PostProcessingPolicy::default(),
        }
    }

    /// Creates a deterministic interference-style preview graph.
    #[must_use]
    pub fn interference_preview(graph_id: impl Into<String>) -> Self {
        let mut stripes = StimulusLayer::stripes("stimulus.layer.stripes.primary", 12.0);
        stripes.rotation_radians = 0.38;
        stripes.temporal.speed_hz = 0.125;
        stripes.oscillators.push(StimulusOscillator::sine(
            "stimulus.oscillator.stripes.phase_sway",
            crate::LayerOscillatorTarget::PhaseOffset,
            0.07,
            0.08,
        ));

        let mut ripple = StimulusLayer::ripple("stimulus.layer.ripple.center", 9.0);
        ripple.weight = 0.75;
        ripple.phase_offset = 0.15;
        ripple.warp.pinch = 0.2;
        ripple.interference.radial_decay = 1.1;
        ripple.interference.wave_modulation = 0.18;

        let mut interference =
            StimulusLayer::interference("stimulus.layer.interference.dual_source", 7.0);
        interference.weight = 0.72;
        interference.opacity = 0.85;
        interference.temporal.speed_hz = 0.18;
        interference.interference.source_a = Vec2::new(-0.28, -0.08);
        interference.interference.source_b = Vec2::new(0.28, 0.08);
        interference.interference.source_b_weight = 0.9;
        interference.interference.radial_decay = 0.35;
        interference.oscillators.push(StimulusOscillator::sine(
            "stimulus.oscillator.interference.luma",
            crate::LayerOscillatorTarget::LumaBias,
            0.11,
            0.08,
        ));

        let mut rays = StimulusLayer::rays("stimulus.layer.rays.angular", 18.0);
        rays.weight = 0.45;
        rays.opacity = 0.8;
        rays.mirror_mode = MirrorMode::Kaleidoscope6;
        rays.temporal.speed_hz = 0.05;

        let mut noise = StimulusLayer::perlin_noise("stimulus.layer.noise.perlin_fbm", 3.5);
        noise.weight = 0.16;
        noise.opacity = 0.5;
        noise.seed = 17_057;
        noise.noise.domain_warp_strength = 0.06;
        noise.noise.animation_velocity = Vec2::new(0.01, -0.015);

        Self {
            post: PostProcessingPolicy {
                contrast: 1.18,
                brightness: -0.04,
                edge_fade: 0.08,
                center_fade: 0.0,
                blur_radius_px: 0.0,
                noise_amount: 0.02,
                trail_decay: 0.18,
            },
            layers: vec![stripes, ripple, interference, rays, noise],
            ..Self::new(graph_id)
        }
    }

    /// Validates graph shape and layer bounds.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != STIMULUS_LAYER_GRAPH_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: STIMULUS_LAYER_GRAPH_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.graph_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("graph_id"));
        }
        if self.layers.is_empty() {
            return Err(OpticsError::InvalidCount("layers"));
        }
        if self.layers.len() > 32 {
            return Err(OpticsError::InvalidCount("layers"));
        }
        for layer in &self.layers {
            layer.validate()?;
        }
        self.post.validate()?;
        Ok(())
    }
}

fn validate_finite(field: &'static str, value: f32) -> Result<(), OpticsError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(OpticsError::InvalidValue(field))
    }
}

fn validate_non_negative(field: &'static str, value: f32) -> Result<(), OpticsError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(OpticsError::InvalidValue(field))
    }
}

fn validate_unit(field: &'static str, value: f32) -> Result<(), OpticsError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(OpticsError::InvalidValue(field))
    }
}
