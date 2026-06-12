use rusty_optics_model::{ColorRgba, OpticsError, Vec2};

use crate::{
    sample_noise, BasePatternKind, ColorStop, LayerOscillatorTarget, MirrorMode, NoiseAlgorithm,
    StimulusLayer, StimulusLayerBlendMode, StimulusProfile,
};

/// CPU reference sample for a procedural stimulus profile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StimulusSample {
    /// Output luminance in `0..1`.
    pub luma: f32,
    /// Output color in linear RGBA.
    pub color: ColorRgba,
}

/// Minimal Park-Miller seeded generator for deterministic fixtures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParkMillerRng {
    state: u32,
}

impl ParkMillerRng {
    /// Creates a deterministic generator from a non-zero seed.
    #[must_use]
    pub fn new(seed: u32) -> Self {
        let state = seed % 2_147_483_647;
        Self {
            state: state.max(1),
        }
    }

    /// Returns the next pseudo-random unit value.
    #[must_use]
    pub fn next_unit(&mut self) -> f32 {
        let product = u64::from(self.state) * 16_807;
        self.state = (product % 2_147_483_647) as u32;
        self.state as f32 / 2_147_483_647.0
    }
}

/// Samples a profile at normalized `uv` and elapsed seconds.
///
/// # Errors
///
/// Returns [`OpticsError`] when the profile or sample coordinate is invalid.
pub fn sample_profile(
    profile: &StimulusProfile,
    uv: Vec2,
    elapsed_seconds: f32,
) -> Result<StimulusSample, OpticsError> {
    profile.validate()?;
    if !uv.is_finite() {
        return Err(OpticsError::NonFiniteVec2("uv"));
    }
    if !elapsed_seconds.is_finite() {
        return Err(OpticsError::InvalidValue("elapsed_seconds"));
    }

    let temporal = profile.temporal.sample(elapsed_seconds);
    if temporal.in_black_lead_in || !temporal.gate_on {
        return Ok(StimulusSample {
            luma: 0.0,
            color: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
        });
    }

    let luma = sample_layer_stack(
        &profile.layer_graph.layers,
        uv,
        elapsed_seconds,
        profile.layer_graph.post.contrast,
        profile.layer_graph.post.brightness,
    );
    Ok(StimulusSample {
        luma,
        color: sample_color(&profile.layer_graph.layers[0].colors, luma),
    })
}

fn sample_layer_stack(
    layers: &[StimulusLayer],
    uv: Vec2,
    elapsed_seconds: f32,
    contrast: f32,
    brightness: f32,
) -> f32 {
    let mut accumulated = 0.0;
    let mut additive_weight = 0.0;
    for layer in layers {
        let value = sample_layer(layer, uv, elapsed_seconds);
        match layer.blend_mode {
            StimulusLayerBlendMode::Add => {
                accumulated += value * layer.weight;
                additive_weight += layer.weight.abs();
            }
            StimulusLayerBlendMode::Multiply => {
                accumulated *= value;
            }
            StimulusLayerBlendMode::Max => {
                accumulated = f32::max(accumulated, value);
            }
        }
    }
    if additive_weight > 0.0 {
        accumulated /= additive_weight;
    }
    let post = (accumulated - 0.5).mul_add(contrast, 0.5 + brightness);
    clamp01(post)
}

fn sample_layer(layer: &StimulusLayer, uv: Vec2, elapsed_seconds: f32) -> f32 {
    let mut temporal_phase = elapsed_seconds.mul_add(layer.temporal.speed_hz, layer.phase_offset)
        + layer.temporal.phase_offset;
    let mut spatial_frequency = layer.spatial_frequency;
    let mut rotation_radians = layer.rotation_radians;
    let mut opacity = layer.opacity;
    let mut amplitude = layer.temporal.amplitude;
    let mut luma_bias = 0.0;
    let mut warp_twist_offset = 0.0;

    for oscillator in &layer.oscillators {
        let value = oscillator.sample(elapsed_seconds);
        match oscillator.target {
            LayerOscillatorTarget::PhaseOffset => temporal_phase += value,
            LayerOscillatorTarget::SpatialFrequencyScale => {
                spatial_frequency *= (1.0 + value).max(0.001);
            }
            LayerOscillatorTarget::RotationRadians => rotation_radians += value,
            LayerOscillatorTarget::Opacity => opacity = clamp01(opacity + value),
            LayerOscillatorTarget::Amplitude => amplitude = (amplitude + value).max(0.0),
            LayerOscillatorTarget::LumaBias => luma_bias += value,
            LayerOscillatorTarget::WarpTwist => warp_twist_offset += value,
        }
    }

    let p = layer_space(layer, uv, rotation_radians, warp_twist_offset);
    let value = match layer.pattern {
        BasePatternKind::Stripes => wave01(p.x.mul_add(spatial_frequency, temporal_phase)),
        BasePatternKind::Rings => {
            let radius = (p.x.mul_add(p.x, p.y * p.y)).sqrt();
            wave01(radius.mul_add(spatial_frequency, temporal_phase))
        }
        BasePatternKind::Rays => {
            let turns = p.y.atan2(p.x) / std::f32::consts::TAU;
            wave01(turns.mul_add(spatial_frequency, temporal_phase))
        }
        BasePatternKind::Checkerboard => {
            let x = wave01(p.x.mul_add(spatial_frequency, temporal_phase));
            let y = wave01(p.y.mul_add(spatial_frequency, temporal_phase));
            if (x > 0.5) == (y > 0.5) {
                1.0
            } else {
                0.0
            }
        }
        BasePatternKind::NoiseField => sample_noise(
            layer.noise,
            layer.seed,
            p,
            spatial_frequency,
            elapsed_seconds,
        ),
        BasePatternKind::PerlinNoise => {
            let mut noise = layer.noise;
            noise.algorithm = NoiseAlgorithm::PerlinGradient;
            sample_noise(noise, layer.seed, p, spatial_frequency, elapsed_seconds)
        }
        BasePatternKind::Ripple => ripple_value(layer, p, spatial_frequency, temporal_phase),
        BasePatternKind::Interference => {
            interference_value(layer, p, spatial_frequency, temporal_phase)
        }
    };
    clamp01(value.mul_add(amplitude, luma_bias) * opacity)
}

fn layer_space(
    layer: &StimulusLayer,
    uv: Vec2,
    rotation_radians: f32,
    warp_twist_offset: f32,
) -> Vec2 {
    let mut p = Vec2::new(
        (uv.x - 0.5) + layer.warp.offset.x,
        (uv.y - 0.5) + layer.warp.offset.y,
    );
    p = apply_mirror(layer.mirror_mode, p);
    p = Vec2::new(p.x * layer.warp.scale.x, p.y * layer.warp.scale.y);

    let radius = (p.x.mul_add(p.x, p.y * p.y)).sqrt();
    let angle = rotation_radians + (layer.warp.twist + warp_twist_offset) * radius;
    let sin = angle.sin();
    let cos = angle.cos();
    p = Vec2::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos);

    let pinch = 1.0 + layer.warp.pinch * radius;
    if pinch != 0.0 {
        p = p / pinch;
    }
    Vec2::new(
        p.x + layer.warp.shear_x * p.y,
        p.y + layer.warp.shear_y * p.x,
    )
}

fn ripple_value(layer: &StimulusLayer, p: Vec2, spatial_frequency: f32, phase: f32) -> f32 {
    let distance = distance(p, layer.interference.source_a);
    let carrier = wave01(distance.mul_add(spatial_frequency, phase));
    let modulation = if layer.interference.wave_modulation > 0.0 {
        wave01((p.x + p.y).mul_add(spatial_frequency * 0.25, phase))
    } else {
        0.5
    };
    let decay = radial_decay(distance, layer.interference.radial_decay);
    clamp01(0.5 + (carrier - 0.5) * decay * (1.0 + layer.interference.wave_modulation * modulation))
}

fn interference_value(layer: &StimulusLayer, p: Vec2, spatial_frequency: f32, phase: f32) -> f32 {
    let distance_a = distance(p, layer.interference.source_a);
    let distance_b = distance(p, layer.interference.source_b);
    let source_a = (distance_a.mul_add(spatial_frequency, phase) * std::f32::consts::TAU).sin();
    let source_b = (distance_b.mul_add(spatial_frequency, phase) * std::f32::consts::TAU).sin()
        * layer.interference.source_b_weight;
    let denominator = (1.0 + layer.interference.source_b_weight).max(f32::EPSILON);
    let combined = (source_a + source_b) / denominator;
    let modulation = if layer.interference.wave_modulation > 0.0 {
        1.0 + layer.interference.wave_modulation * wave01(p.x.mul_add(spatial_frequency, phase))
    } else {
        1.0
    };
    let decay = radial_decay(
        distance_a.min(distance_b),
        layer.interference.radial_decay * 0.5,
    );
    clamp01(0.5 + 0.5 * combined * modulation * decay)
}

fn distance(a: Vec2, b: Vec2) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx.mul_add(dx, dy * dy).sqrt()
}

fn radial_decay(distance: f32, decay: f32) -> f32 {
    if decay > 0.0 {
        (-distance * decay).exp()
    } else {
        1.0
    }
}

fn apply_mirror(mode: MirrorMode, p: Vec2) -> Vec2 {
    match mode {
        MirrorMode::None => p,
        MirrorMode::Horizontal => Vec2::new(p.x.abs(), p.y),
        MirrorMode::Vertical => Vec2::new(p.x, p.y.abs()),
        MirrorMode::Axes => Vec2::new(p.x.abs(), p.y.abs()),
        MirrorMode::Kaleidoscope3 => kaleidoscope(p, 3.0),
        MirrorMode::Kaleidoscope6 => kaleidoscope(p, 6.0),
    }
}

fn kaleidoscope(p: Vec2, sectors: f32) -> Vec2 {
    let radius = (p.x.mul_add(p.x, p.y * p.y)).sqrt();
    let sector_angle = std::f32::consts::TAU / sectors;
    let mut angle = p.y.atan2(p.x);
    angle = (angle / sector_angle).rem_euclid(1.0) * sector_angle;
    if angle > sector_angle * 0.5 {
        angle = sector_angle - angle;
    }
    Vec2::new(angle.cos() * radius, angle.sin() * radius)
}

fn wave01(turns: f32) -> f32 {
    (turns * std::f32::consts::TAU).sin() * 0.5 + 0.5
}

fn sample_color(stops: &[ColorStop], value: f32) -> ColorRgba {
    let value = clamp01(value);
    let mut previous = stops[0];
    for stop in &stops[1..] {
        if value <= stop.position {
            let span = (stop.position - previous.position).max(f32::EPSILON);
            let t = ((value - previous.position) / span).clamp(0.0, 1.0);
            return lerp_color(previous.color, stop.color, t).clamped01();
        }
        previous = *stop;
    }
    previous.color.clamped01()
}

fn lerp_color(a: ColorRgba, b: ColorRgba, t: f32) -> ColorRgba {
    ColorRgba::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn clamp01(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
