use rusty_optics_model::{OpticsError, Vec2};

/// Deterministic noise algorithm used by a procedural stimulus layer.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoiseAlgorithm {
    /// Nearest-cell hashed value noise.
    CellValue,
    /// Bilinearly smoothed value noise.
    SmoothValue,
    /// Gradient noise with Perlin-style fade interpolation.
    PerlinGradient,
}

/// Renderer-neutral noise controls for a stimulus layer.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoiseControls {
    /// Noise algorithm family.
    pub algorithm: NoiseAlgorithm,
    /// Number of fBm octaves.
    pub octaves: u8,
    /// Frequency multiplier between octaves.
    pub lacunarity: f32,
    /// Amplitude multiplier between octaves.
    pub gain: f32,
    /// Domain-warp strength in centered layer coordinates.
    pub domain_warp_strength: f32,
    /// Noise-domain velocity in normalized units per second.
    pub animation_velocity: Vec2,
    /// Output amplitude.
    pub amplitude: f32,
    /// Output bias added after normalization.
    pub bias: f32,
}

impl Default for NoiseControls {
    fn default() -> Self {
        Self {
            algorithm: NoiseAlgorithm::CellValue,
            octaves: 1,
            lacunarity: 2.0,
            gain: 0.5,
            domain_warp_strength: 0.0,
            animation_velocity: Vec2::ZERO,
            amplitude: 1.0,
            bias: 0.0,
        }
    }
}

impl NoiseControls {
    /// Creates a Perlin-style fBm control profile.
    #[must_use]
    pub fn perlin(octaves: u8) -> Self {
        Self {
            algorithm: NoiseAlgorithm::PerlinGradient,
            octaves,
            lacunarity: 2.0,
            gain: 0.5,
            domain_warp_strength: 0.0,
            animation_velocity: Vec2::ZERO,
            amplitude: 1.0,
            bias: 0.0,
        }
    }

    /// Validates noise controls.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(self) -> Result<(), OpticsError> {
        if self.octaves == 0 || self.octaves > 8 {
            return Err(OpticsError::InvalidCount("noise.octaves"));
        }
        validate_non_negative("noise.lacunarity", self.lacunarity)?;
        if self.lacunarity < 1.0 {
            return Err(OpticsError::InvalidValue("noise.lacunarity"));
        }
        validate_unit("noise.gain", self.gain)?;
        validate_non_negative("noise.domain_warp_strength", self.domain_warp_strength)?;
        if self.domain_warp_strength > 4.0 {
            return Err(OpticsError::InvalidValue("noise.domain_warp_strength"));
        }
        if !self.animation_velocity.is_finite() {
            return Err(OpticsError::NonFiniteVec2("noise.animation_velocity"));
        }
        validate_non_negative("noise.amplitude", self.amplitude)?;
        validate_finite("noise.bias", self.bias)?;
        Ok(())
    }
}

/// Samples deterministic layer noise in `0..1`.
#[must_use]
pub fn sample_noise(
    controls: NoiseControls,
    seed: u32,
    p: Vec2,
    frequency: f32,
    elapsed_seconds: f32,
) -> f32 {
    let mut sample_p = Vec2::new(
        p.x + controls.animation_velocity.x * elapsed_seconds,
        p.y + controls.animation_velocity.y * elapsed_seconds,
    );
    if controls.domain_warp_strength > 0.0 {
        let warp_x = perlin_single(seed ^ 0xA511_E9B3, sample_p, frequency * 0.5);
        let warp_y = perlin_single(seed ^ 0x63D8_35D1, sample_p, frequency * 0.5);
        sample_p.x += (warp_x - 0.5) * controls.domain_warp_strength;
        sample_p.y += (warp_y - 0.5) * controls.domain_warp_strength;
    }

    let mut amplitude = 1.0;
    let mut amplitude_sum = 0.0;
    let mut octave_frequency = frequency.max(0.001);
    let mut accumulated = 0.0;

    for octave in 0..controls.octaves {
        let octave_seed = seed ^ u32::from(octave).wrapping_mul(0x9E37_79B9);
        let octave_value = match controls.algorithm {
            NoiseAlgorithm::CellValue => cell_value_noise(octave_seed, sample_p, octave_frequency),
            NoiseAlgorithm::SmoothValue => {
                smooth_value_noise(octave_seed, sample_p, octave_frequency)
            }
            NoiseAlgorithm::PerlinGradient => {
                perlin_single(octave_seed, sample_p, octave_frequency)
            }
        };
        accumulated += octave_value * amplitude;
        amplitude_sum += amplitude;
        amplitude *= controls.gain;
        octave_frequency *= controls.lacunarity;
    }

    let normalized = if amplitude_sum > 0.0 {
        accumulated / amplitude_sum
    } else {
        0.0
    };
    clamp01(normalized.mul_add(controls.amplitude, controls.bias))
}

fn cell_value_noise(seed: u32, p: Vec2, cells: f32) -> f32 {
    let x = (p.x * cells).floor() as i32;
    let y = (p.y * cells).floor() as i32;
    hash_unit(seed, x, y)
}

fn smooth_value_noise(seed: u32, p: Vec2, cells: f32) -> f32 {
    let x = p.x * cells;
    let y = p.y * cells;
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = fade(x - x.floor());
    let ty = fade(y - y.floor());

    let a = hash_unit(seed, x0, y0);
    let b = hash_unit(seed, x0 + 1, y0);
    let c = hash_unit(seed, x0, y0 + 1);
    let d = hash_unit(seed, x0 + 1, y0 + 1);
    lerp(lerp(a, b, tx), lerp(c, d, tx), ty)
}

fn perlin_single(seed: u32, p: Vec2, cells: f32) -> f32 {
    let x = p.x * cells;
    let y = p.y * cells;
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();
    let u = fade(xf);
    let v = fade(yf);

    let aa = gradient_dot(seed, x0, y0, xf, yf);
    let ba = gradient_dot(seed, x0 + 1, y0, xf - 1.0, yf);
    let ab = gradient_dot(seed, x0, y0 + 1, xf, yf - 1.0);
    let bb = gradient_dot(seed, x0 + 1, y0 + 1, xf - 1.0, yf - 1.0);
    let value = lerp(lerp(aa, ba, u), lerp(ab, bb, u), v);
    clamp01(value * 0.5 + 0.5)
}

fn gradient_dot(seed: u32, x: i32, y: i32, dx: f32, dy: f32) -> f32 {
    let hash = hash_u32(seed, x, y);
    let angle = (hash as f32 / u32::MAX as f32) * std::f32::consts::TAU;
    angle.cos().mul_add(dx, angle.sin() * dy)
}

fn hash_unit(seed: u32, x: i32, y: i32) -> f32 {
    hash_u32(seed, x, y) as f32 / u32::MAX as f32
}

fn hash_u32(seed: u32, x: i32, y: i32) -> u32 {
    let mut hash = seed ^ (x as u32).wrapping_mul(0x9E37_79B9);
    hash ^= (y as u32).wrapping_mul(0x85EB_CA6B);
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7FEB_352D);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x846C_A68B);
    hash ^ (hash >> 16)
}

fn fade(value: f32) -> f32 {
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn clamp01(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
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
