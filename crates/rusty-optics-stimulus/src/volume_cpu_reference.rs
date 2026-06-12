use rusty_optics_model::{OpticsError, Vec2};

use crate::{
    sample_profile, StimulusProfile, StimulusVolumeDescriptor, StimulusVolumeProbeOutput,
    StimulusVolumeProbeRay, StimulusVolumeProbeStatus,
};

/// Samples one volume probe ray with the CPU reference path.
///
/// # Errors
///
/// Returns [`OpticsError`] when the profile, volume, or ray is invalid.
pub fn sample_volume_probe(
    profile: &StimulusProfile,
    volume: &StimulusVolumeDescriptor,
    ray: &StimulusVolumeProbeRay,
) -> Result<StimulusVolumeProbeOutput, OpticsError> {
    profile.validate()?;
    volume.validate()?;
    ray.validate()?;

    let Some((enter, exit)) = intersect_bounds(volume, ray) else {
        return Ok(StimulusVolumeProbeOutput::missed(volume.step_count));
    };
    let elapsed_seconds = ray.uv_eye_time[3];
    let step_count = volume.step_count.max(1);
    let segment = (exit - enter).max(f32::EPSILON);
    let jitter = deterministic_jitter(ray) * volume.step_jitter;
    let step_offset = (0.5 + jitter - volume.step_jitter * 0.5).clamp(0.0, 1.0);

    let mut accum_rgb = [0.0_f32; 3];
    let mut accum_alpha = 0.0_f32;
    let mut first_depth = 0.0_f32;
    let mut hit = false;

    for step in 0..step_count {
        let t = enter + ((step as f32 + step_offset) / step_count as f32) * segment;
        let point = [
            ray.ray_origin[0] + ray.ray_direction[0] * t,
            ray.ray_origin[1] + ray.ray_direction[1] * t,
            ray.ray_origin[2] + ray.ray_direction[2] * t,
        ];
        let normalized = normalize_point(volume, point);
        let uv = Vec2::new(normalized[0], normalized[1]);
        let layer_sample = sample_profile(profile, uv, elapsed_seconds + normalized[2] * 0.03125)?;
        let z_envelope = 1.0 - (normalized[2] * 2.0 - 1.0).abs() * 0.35;
        let density = clamp01(layer_sample.luma * volume.density_scale * z_envelope.max(0.0));
        let step_alpha = clamp01(density * volume.opacity_scale / step_count as f32);
        if step_alpha > 0.0 && !hit {
            hit = true;
            first_depth = ((t - enter) / segment).clamp(0.0, 1.0);
        }
        let contribution = (1.0 - accum_alpha) * step_alpha;
        accum_rgb[0] += layer_sample.color.r * contribution;
        accum_rgb[1] += layer_sample.color.g * contribution;
        accum_rgb[2] += layer_sample.color.b * contribution;
        accum_alpha = clamp01(accum_alpha + contribution);
    }

    let status = if hit {
        StimulusVolumeProbeStatus::Hit
    } else {
        StimulusVolumeProbeStatus::Missed
    };
    Ok(StimulusVolumeProbeOutput {
        rgba: [
            accum_rgb[0].clamp(0.0, 1.0),
            accum_rgb[1].clamp(0.0, 1.0),
            accum_rgb[2].clamp(0.0, 1.0),
            accum_alpha,
        ],
        density_depth_status: [accum_alpha, first_depth, status.as_f32(), step_count as f32],
    })
}

/// Samples a bounded probe set with the CPU reference path.
///
/// # Errors
///
/// Returns [`OpticsError`] when any input ray or profile field is invalid.
pub fn sample_volume_probe_set(
    profile: &StimulusProfile,
    volume: &StimulusVolumeDescriptor,
    rays: &[StimulusVolumeProbeRay],
) -> Result<Vec<StimulusVolumeProbeOutput>, OpticsError> {
    if rays.is_empty() || rays.len() > 4096 {
        return Err(OpticsError::InvalidCount("volume_probe.rays"));
    }
    rays.iter()
        .map(|ray| sample_volume_probe(profile, volume, ray))
        .collect()
}

fn intersect_bounds(
    volume: &StimulusVolumeDescriptor,
    ray: &StimulusVolumeProbeRay,
) -> Option<(f32, f32)> {
    let mut enter = f32::NEG_INFINITY;
    let mut exit = f32::INFINITY;
    for axis in 0..3 {
        let origin = ray.ray_origin[axis];
        let direction = ray.ray_direction[axis];
        if direction.abs() <= f32::EPSILON {
            if origin < volume.bounds_min[axis] || origin > volume.bounds_max[axis] {
                return None;
            }
            continue;
        }
        let inv_direction = 1.0 / direction;
        let mut t0 = (volume.bounds_min[axis] - origin) * inv_direction;
        let mut t1 = (volume.bounds_max[axis] - origin) * inv_direction;
        if t0 > t1 {
            core::mem::swap(&mut t0, &mut t1);
        }
        enter = enter.max(t0);
        exit = exit.min(t1);
        if exit < enter {
            return None;
        }
    }
    if exit < 0.0 {
        None
    } else {
        Some((enter.max(0.0), exit))
    }
}

fn normalize_point(volume: &StimulusVolumeDescriptor, point: [f32; 3]) -> [f32; 3] {
    [
        ((point[0] - volume.bounds_min[0]) / (volume.bounds_max[0] - volume.bounds_min[0]))
            .clamp(0.0, 1.0),
        ((point[1] - volume.bounds_min[1]) / (volume.bounds_max[1] - volume.bounds_min[1]))
            .clamp(0.0, 1.0),
        ((point[2] - volume.bounds_min[2]) / (volume.bounds_max[2] - volume.bounds_min[2]))
            .clamp(0.0, 1.0),
    ]
}

fn deterministic_jitter(ray: &StimulusVolumeProbeRay) -> f32 {
    let seed = ray.uv_eye_time[0] * 12.9898
        + ray.uv_eye_time[1] * 78.233
        + ray.uv_eye_time[2] * 15.127
        + ray.uv_eye_time[3] * 37.719;
    fract((seed.sin() * 43_758.547).abs())
}

fn fract(value: f32) -> f32 {
    value - value.floor()
}

fn clamp01(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
