use rusty_optics_model::OpticsError;

/// Oscillator waveform available to procedural stimulus layers.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OscillatorWaveform {
    /// Smooth sinusoidal modulation.
    Sine,
    /// Linear rise and fall across each cycle.
    Triangle,
    /// Rising ramp from -1 to 1.
    SawUp,
    /// Falling ramp from 1 to -1.
    SawDown,
    /// Two-state square wave using the oscillator duty cycle.
    Square,
}

/// Layer parameter modulated by an oscillator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerOscillatorTarget {
    /// Add to layer phase in cycles.
    PhaseOffset,
    /// Scale the spatial frequency by `1 + value`.
    SpatialFrequencyScale,
    /// Add to layer rotation in radians.
    RotationRadians,
    /// Add to layer opacity before clamping.
    Opacity,
    /// Add to layer amplitude before clamping to a non-negative value.
    Amplitude,
    /// Add to sampled layer luma before clamping.
    LumaBias,
    /// Add to layer warp twist.
    WarpTwist,
}

/// Renderer-neutral oscillator binding for a procedural layer.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StimulusOscillator {
    /// Stable oscillator identifier.
    pub oscillator_id: String,
    /// Parameter affected by this oscillator.
    pub target: LayerOscillatorTarget,
    /// Waveform family.
    pub waveform: OscillatorWaveform,
    /// Oscillator frequency in cycles per second.
    pub frequency_hz: f32,
    /// Phase offset in cycles.
    pub phase_offset: f32,
    /// Output amplitude.
    pub amplitude: f32,
    /// Output bias added after waveform sampling.
    pub bias: f32,
    /// Duty cycle used by square-wave oscillators.
    pub duty_cycle: f32,
}

impl StimulusOscillator {
    /// Creates a sine oscillator.
    #[must_use]
    pub fn sine(
        oscillator_id: impl Into<String>,
        target: LayerOscillatorTarget,
        frequency_hz: f32,
        amplitude: f32,
    ) -> Self {
        Self {
            oscillator_id: oscillator_id.into(),
            target,
            waveform: OscillatorWaveform::Sine,
            frequency_hz,
            phase_offset: 0.0,
            amplitude,
            bias: 0.0,
            duty_cycle: 0.5,
        }
    }

    /// Samples the oscillator at elapsed seconds.
    #[must_use]
    pub fn sample(&self, elapsed_seconds: f32) -> f32 {
        let phase = fract01(elapsed_seconds.mul_add(self.frequency_hz, self.phase_offset));
        let waveform = match self.waveform {
            OscillatorWaveform::Sine => (phase * std::f32::consts::TAU).sin(),
            OscillatorWaveform::Triangle => 1.0 - 4.0 * (phase - 0.5).abs(),
            OscillatorWaveform::SawUp => phase.mul_add(2.0, -1.0),
            OscillatorWaveform::SawDown => 1.0 - phase * 2.0,
            OscillatorWaveform::Square => {
                if phase < self.duty_cycle {
                    1.0
                } else {
                    -1.0
                }
            }
        };
        waveform.mul_add(self.amplitude, self.bias)
    }

    /// Validates oscillator shape and finite parameters.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.oscillator_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("oscillator_id"));
        }
        validate_non_negative("oscillator.frequency_hz", self.frequency_hz)?;
        validate_finite("oscillator.phase_offset", self.phase_offset)?;
        validate_finite("oscillator.amplitude", self.amplitude)?;
        validate_finite("oscillator.bias", self.bias)?;
        if !self.duty_cycle.is_finite() || self.duty_cycle <= 0.0 || self.duty_cycle >= 1.0 {
            return Err(OpticsError::InvalidValue("oscillator.duty_cycle"));
        }
        Ok(())
    }
}

fn fract01(value: f32) -> f32 {
    value - value.floor()
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
