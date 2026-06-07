/// Linear RGBA color.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorRgba {
    /// Red channel.
    pub r: f32,
    /// Green channel.
    pub g: f32,
    /// Blue channel.
    pub b: f32,
    /// Alpha channel.
    pub a: f32,
}

impl ColorRgba {
    /// Opaque white.
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    /// Transparent black.
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Creates a color.
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Returns whether all channels are finite.
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.r.is_finite() && self.g.is_finite() && self.b.is_finite() && self.a.is_finite()
    }

    /// Returns a copy with all channels clamped to `0..1`.
    #[must_use]
    pub fn clamped01(self) -> Self {
        Self::new(
            clamp01(self.r),
            clamp01(self.g),
            clamp01(self.b),
            clamp01(self.a),
        )
    }
}

impl Default for ColorRgba {
    fn default() -> Self {
        Self::WHITE
    }
}

fn clamp01(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
