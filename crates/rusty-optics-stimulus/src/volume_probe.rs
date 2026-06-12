use rusty_optics_model::OpticsError;

use crate::StimulusVolumeDescriptor;

/// CPU/GPU probe status encoded into vec4-friendly readback records.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StimulusVolumeProbeStatus {
    /// Ray did not intersect the volume.
    Missed,
    /// Ray intersected the volume and accumulated nonzero density.
    Hit,
}

impl StimulusVolumeProbeStatus {
    /// Numeric status used by GPU readback buffers.
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        match self {
            Self::Missed => 0.0,
            Self::Hit => 1.0,
        }
    }
}

/// Vec4-aligned volume probe ray input.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StimulusVolumeProbeRay {
    /// U, V, eye index, elapsed seconds.
    pub uv_eye_time: [f32; 4],
    /// Ray origin xyz and reserved w.
    pub ray_origin: [f32; 4],
    /// Ray direction xyz and reserved w.
    pub ray_direction: [f32; 4],
}

impl StimulusVolumeProbeRay {
    /// Validates finite ray records.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when the ray is non-finite or zero-length.
    pub fn validate(&self) -> Result<(), OpticsError> {
        validate_vec4("volume_probe.uv_eye_time", self.uv_eye_time)?;
        validate_vec4("volume_probe.ray_origin", self.ray_origin)?;
        validate_vec4("volume_probe.ray_direction", self.ray_direction)?;
        if !(0.0..=1.0).contains(&self.uv_eye_time[0])
            || !(0.0..=1.0).contains(&self.uv_eye_time[1])
        {
            return Err(OpticsError::InvalidValue("volume_probe.uv"));
        }
        let length_sq = self.ray_direction[0].mul_add(
            self.ray_direction[0],
            self.ray_direction[1].mul_add(
                self.ray_direction[1],
                self.ray_direction[2] * self.ray_direction[2],
            ),
        );
        if length_sq <= f32::EPSILON {
            return Err(OpticsError::InvalidValue("volume_probe.ray_direction"));
        }
        Ok(())
    }
}

/// Vec4-aligned volume probe output.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StimulusVolumeProbeOutput {
    /// Accumulated linear RGBA.
    pub rgba: [f32; 4],
    /// Accumulated density, normalized depth, status code, step count.
    pub density_depth_status: [f32; 4],
}

impl StimulusVolumeProbeOutput {
    /// Empty miss output.
    #[must_use]
    pub const fn missed(step_count: u32) -> Self {
        Self {
            rgba: [0.0, 0.0, 0.0, 0.0],
            density_depth_status: [
                0.0,
                0.0,
                StimulusVolumeProbeStatus::Missed.as_f32(),
                step_count as f32,
            ],
        }
    }
}

/// Builds deterministic probe rays through the volume bounds.
///
/// # Errors
///
/// Returns [`OpticsError`] when the volume, count, or time is invalid.
pub fn deterministic_volume_probe_rays(
    volume: &StimulusVolumeDescriptor,
    sample_count: usize,
    elapsed_seconds: f32,
) -> Result<Vec<StimulusVolumeProbeRay>, OpticsError> {
    volume.validate()?;
    if sample_count == 0 || sample_count > 4096 {
        return Err(OpticsError::InvalidCount("volume_probe.sample_count"));
    }
    if !elapsed_seconds.is_finite() {
        return Err(OpticsError::InvalidValue("volume_probe.elapsed_seconds"));
    }

    let columns = (sample_count as f32).sqrt().ceil() as usize;
    let rows = sample_count.div_ceil(columns);
    let span = [
        volume.bounds_max[0] - volume.bounds_min[0],
        volume.bounds_max[1] - volume.bounds_min[1],
        volume.bounds_max[2] - volume.bounds_min[2],
    ];
    let mut rays = Vec::with_capacity(sample_count);
    for index in 0..sample_count {
        let x = index % columns;
        let y = index / columns;
        let u = (x as f32 + 0.5) / columns as f32;
        let v = (y as f32 + 0.5) / rows as f32;
        let origin = [
            volume.bounds_min[0] + u * span[0],
            volume.bounds_min[1] + v * span[1],
            volume.bounds_max[2] + span[2] * 0.5,
            1.0,
        ];
        rays.push(StimulusVolumeProbeRay {
            uv_eye_time: [u, v, (index % 2) as f32, elapsed_seconds],
            ray_origin: origin,
            ray_direction: [0.0, 0.0, -1.0, 0.0],
        });
    }
    Ok(rays)
}

fn validate_vec4(field: &'static str, value: [f32; 4]) -> Result<(), OpticsError> {
    if value.iter().all(|component| component.is_finite()) {
        Ok(())
    } else {
        Err(OpticsError::InvalidValue(field))
    }
}
