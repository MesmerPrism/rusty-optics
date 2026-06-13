/// Current bounded sample count for the stable volume probe oracle.
pub const BOUNDED_STIMULUS_VOLUME_PROBE_SAMPLES: usize = 8;
/// Conservative f32 tolerance for bounded volume probe comparisons.
pub const BOUNDED_STIMULUS_VOLUME_PROBE_DEFAULT_TOLERANCE: f32 = 0.001;
/// Low-resolution raymarch preview width per eye.
pub const BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_WIDTH: usize = 4;
/// Low-resolution raymarch preview height per eye.
pub const BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_HEIGHT: usize = 4;
/// Current bounded stereo output layer count.
pub const BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_EYE_COUNT: usize = 2;
/// Current bounded stereo pixel count.
pub const BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_PIXELS: usize =
    BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_WIDTH
        * BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_HEIGHT
        * BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_EYE_COUNT;
/// Conservative f32 tolerance for bounded raymarch preview comparisons.
pub const BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_DEFAULT_TOLERANCE: f32 = 0.002;
/// Default generated eye tile width for the scalable stereo atlas oracle.
pub const BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_TILE_WIDTH: usize = 64;
/// Default generated eye tile height for the scalable stereo atlas oracle.
pub const BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_TILE_HEIGHT: usize = 64;
/// Current bounded stereo image-preview eye count.
pub const BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_COUNT: usize =
    BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_EYE_COUNT;
/// Stereo atlas image width.
pub const BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_IMAGE_WIDTH: usize =
    BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_TILE_WIDTH
        * BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_COUNT;
/// Stereo atlas image height.
pub const BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_IMAGE_HEIGHT: usize =
    BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_TILE_HEIGHT;
/// Stereo atlas image layers.
pub const BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_IMAGE_LAYERS: usize = 1;
/// Bounded sample grid width used for image readback checks.
pub const BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_WIDTH: usize =
    BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_WIDTH;
/// Bounded sample grid height used for image readback checks.
pub const BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_HEIGHT: usize =
    BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_HEIGHT;
/// Current bounded stereo image readback sample count.
pub const BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_PIXELS: usize =
    BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_WIDTH
        * BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_HEIGHT
        * BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_COUNT;
/// Conservative f32 tolerance for bounded image-preview comparisons.
pub const BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_DEFAULT_TOLERANCE: f32 =
    BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_DEFAULT_TOLERANCE;
/// Initial image format used by Vulkan/WebGPU proof adapters.
pub const BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_FORMAT: &str = "R32G32B32A32_SFLOAT";

/// Vec4-aligned CPU-oracle sample for a bounded volume probe.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundedStimulusVolumeProbeSample {
    /// `[u, v, eye_index, time_seconds]`.
    pub uv_eye_time: [f32; 4],
    /// `[origin_x, origin_y, origin_z, probe_depth]`.
    pub ray_origin_depth: [f32; 4],
    /// `[dir_x, dir_y, dir_z, step_count]`.
    pub ray_direction_step: [f32; 4],
    /// `[frequency, phase, opacity, reserved]`.
    pub volume_params: [f32; 4],
    /// Expected RGBA from the CPU oracle.
    pub expected_rgba: [f32; 4],
    /// Expected `[density, depth, status, reserved]` from the CPU oracle.
    pub expected_density_depth_status: [f32; 4],
}

/// Vec4-aligned bounded volume probe output.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundedStimulusVolumeProbeOutput {
    /// RGBA volume probe output.
    pub rgba: [f32; 4],
    /// `[density, depth, status, reserved]`.
    pub density_depth_status: [f32; 4],
}

/// Vec4-aligned pixel input for the bounded stereo raymarch preview.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundedStimulusVolumeRaymarchPreviewPixel {
    /// `[u, v, eye_index, time_seconds]`.
    pub uv_eye_time: [f32; 4],
    /// `[origin_x, origin_y, origin_z, reserved]`.
    pub ray_origin: [f32; 4],
    /// `[dir_x, dir_y, dir_z, step_count]`.
    pub ray_direction_step: [f32; 4],
    /// `[frequency, phase, opacity, step_alpha_scale]`.
    pub volume_params: [f32; 4],
    /// Expected RGBA from the CPU oracle.
    pub expected_rgba: [f32; 4],
    /// Expected `[alpha, first_hit_depth, hit_status, step_count]`.
    pub expected_density_depth_status: [f32; 4],
}

/// Vec4-aligned bounded stereo raymarch output.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundedStimulusVolumeRaymarchPreviewOutput {
    /// Raymarched preview RGBA.
    pub rgba: [f32; 4],
    /// `[alpha, first_hit_depth, hit_status, step_count]`.
    pub density_depth_status: [f32; 4],
}

/// Vec4-aligned pixel input for the bounded scalable image preview.
pub type BoundedStimulusVolumeImagePreviewPixel = BoundedStimulusVolumeRaymarchPreviewPixel;

/// Bounded scalable image-preview output.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundedStimulusVolumeImagePreviewOutput {
    /// Raymarched preview RGBA stored in the stereo atlas image.
    pub rgba: [f32; 4],
}

/// Builds a deterministic bounded volume probe sample.
#[must_use]
pub fn deterministic_bounded_stimulus_volume_probe_sample(
    index: usize,
    grid_dimensions: [u64; 3],
    step_count: u64,
) -> BoundedStimulusVolumeProbeSample {
    let eye = (index % 2) as f32;
    let column = (index / 2) as f32;
    let u = (0.18 + column * 0.19).clamp(0.0, 1.0);
    let v = (0.22 + (index as f32 % 4.0) * 0.15).clamp(0.0, 1.0);
    let time = 0.125 * index as f32;
    let depth = 0.12 + 0.055 * index as f32;
    let frequency = bounded_stimulus_volume_frequency(grid_dimensions);
    let phase = bounded_stimulus_volume_phase(step_count);
    let opacity = 0.72;
    let mut sample = BoundedStimulusVolumeProbeSample {
        uv_eye_time: [u, v, eye, time],
        ray_origin_depth: [
            -0.42 + index as f32 * 0.07,
            -0.24 + (index as f32 % 3.0) * 0.12,
            -0.68,
            depth,
        ],
        ray_direction_step: [(u - 0.5) * 0.42, (v - 0.5) * 0.32, 1.0, step_count as f32],
        volume_params: [frequency, phase, opacity, 0.0],
        expected_rgba: [0.0; 4],
        expected_density_depth_status: [0.0; 4],
    };
    let expected = expected_bounded_stimulus_volume_probe_output(sample);
    sample.expected_rgba = expected.rgba;
    sample.expected_density_depth_status = expected.density_depth_status;
    sample
}

/// CPU oracle matching the stable bounded volume probe shader contract.
#[must_use]
pub fn expected_bounded_stimulus_volume_probe_output(
    sample: BoundedStimulusVolumeProbeSample,
) -> BoundedStimulusVolumeProbeOutput {
    let uv = sample.uv_eye_time;
    let origin = [
        sample.ray_origin_depth[0],
        sample.ray_origin_depth[1],
        sample.ray_origin_depth[2],
    ];
    let depth = sample.ray_origin_depth[3];
    let direction = [
        sample.ray_direction_step[0],
        sample.ray_direction_step[1],
        sample.ray_direction_step[2],
    ];
    let p = [
        origin[0] + direction[0] * depth,
        origin[1] + direction[1] * depth,
        origin[2] + direction[2] * depth,
    ];
    let density = bounded_stimulus_volume_density(p, uv, sample.volume_params);
    BoundedStimulusVolumeProbeOutput {
        rgba: [density, density, density, density],
        density_depth_status: [density, depth, 1.0, 0.0],
    }
}

/// Builds a deterministic bounded stereo raymarch preview pixel.
#[must_use]
pub fn deterministic_bounded_stimulus_volume_raymarch_preview_pixel(
    index: usize,
    grid_dimensions: [u64; 3],
    step_count: u64,
) -> BoundedStimulusVolumeRaymarchPreviewPixel {
    let pixels_per_eye = BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_WIDTH
        * BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_HEIGHT;
    let eye_index = (index / pixels_per_eye)
        .min(BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_EYE_COUNT.saturating_sub(1));
    let local = index % pixels_per_eye;
    let x = local % BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_WIDTH;
    let y = local / BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_WIDTH;
    let u = (x as f32 + 0.5) / BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_WIDTH as f32;
    let v = (y as f32 + 0.5) / BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_HEIGHT as f32;
    let eye = eye_index as f32;
    let eye_offset = (eye - 0.5) * 0.08;
    let frequency = bounded_stimulus_volume_frequency(grid_dimensions);
    let phase = bounded_stimulus_volume_phase(step_count);
    let preview_steps = (step_count as f32).clamp(4.0, 32.0);
    let mut pixel = BoundedStimulusVolumeRaymarchPreviewPixel {
        uv_eye_time: [u, v, eye, 0.125 + index as f32 * 0.0125],
        ray_origin: [u - 0.5 + eye_offset, v - 0.5, -0.72, 0.0],
        ray_direction_step: [
            (u - 0.5) * 0.42 + eye_offset * 0.25,
            (v - 0.5) * 0.32,
            1.0,
            preview_steps,
        ],
        volume_params: [frequency, phase, 0.72, 1.25],
        expected_rgba: [0.0; 4],
        expected_density_depth_status: [0.0; 4],
    };
    let expected = expected_bounded_stimulus_volume_raymarch_preview_output(pixel);
    pixel.expected_rgba = expected.rgba;
    pixel.expected_density_depth_status = expected.density_depth_status;
    pixel
}

/// CPU oracle matching the stable bounded stereo raymarch shader contract.
#[must_use]
pub fn expected_bounded_stimulus_volume_raymarch_preview_output(
    pixel: BoundedStimulusVolumeRaymarchPreviewPixel,
) -> BoundedStimulusVolumeRaymarchPreviewOutput {
    let uv = pixel.uv_eye_time;
    let origin = [
        pixel.ray_origin[0],
        pixel.ray_origin[1],
        pixel.ray_origin[2],
    ];
    let direction = [
        pixel.ray_direction_step[0],
        pixel.ray_direction_step[1],
        pixel.ray_direction_step[2],
    ];
    let step_count = pixel.ray_direction_step[3].clamp(1.0, 32.0);
    let step_alpha_scale = pixel.volume_params[3].clamp(0.001, 4.0);
    let mut accum_rgb = [0.0_f32; 3];
    let mut accum_alpha = 0.0_f32;
    let mut first_depth = 0.0_f32;
    let mut hit = 0.0_f32;

    for step in 0..32 {
        let step_f = step as f32;
        if step_f < step_count {
            let unit_depth = (step_f + 0.5) / step_count;
            let p = [
                origin[0] + direction[0] * unit_depth,
                origin[1] + direction[1] * unit_depth,
                origin[2] + direction[2] * unit_depth,
            ];
            let density = bounded_stimulus_volume_density(p, uv, pixel.volume_params);
            let sample_alpha = (density * step_alpha_scale / step_count).clamp(0.0, 1.0);
            let contribution = (1.0 - accum_alpha) * sample_alpha;
            accum_rgb[0] += density * contribution;
            accum_rgb[1] += density * contribution;
            accum_rgb[2] += density * contribution;
            if hit < 0.5 && density > 0.05 {
                first_depth = unit_depth;
                hit = 1.0;
            }
            accum_alpha = (accum_alpha + contribution).clamp(0.0, 1.0);
        }
    }

    BoundedStimulusVolumeRaymarchPreviewOutput {
        rgba: [accum_rgb[0], accum_rgb[1], accum_rgb[2], accum_alpha],
        density_depth_status: [accum_alpha, first_depth, hit, step_count],
    }
}

/// Builds a deterministic bounded scalable image-preview sample pixel.
#[must_use]
pub fn deterministic_bounded_stimulus_volume_image_preview_pixel(
    index: usize,
    grid_dimensions: [u64; 3],
    step_count: u64,
    eye_tile_width: usize,
    eye_tile_height: usize,
) -> BoundedStimulusVolumeImagePreviewPixel {
    let samples_per_eye = BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_WIDTH
        * BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_HEIGHT;
    let eye_index = (index / samples_per_eye)
        .min(BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_COUNT.saturating_sub(1));
    let local = index % samples_per_eye;
    let sample_x = local % BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_WIDTH;
    let sample_y = local / BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_WIDTH;
    let tile_width = eye_tile_width.max(BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_WIDTH);
    let tile_height = eye_tile_height.max(BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_HEIGHT);
    let pixel_x = scalable_sample_coordinate(
        sample_x,
        BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_WIDTH,
        tile_width,
    );
    let pixel_y = scalable_sample_coordinate(
        sample_y,
        BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_SAMPLE_GRID_HEIGHT,
        tile_height,
    );
    let u = (pixel_x as f32 + 0.5) / tile_width as f32;
    let v = (pixel_y as f32 + 0.5) / tile_height as f32;
    let eye = eye_index as f32;
    let eye_offset = (eye - 0.5) * 0.08;
    let frequency = bounded_stimulus_volume_frequency(grid_dimensions);
    let phase = bounded_stimulus_volume_phase(step_count);
    let preview_steps = (step_count as f32).clamp(4.0, 32.0);
    let mut pixel = BoundedStimulusVolumeImagePreviewPixel {
        uv_eye_time: [u, v, eye, 0.125],
        ray_origin: [u - 0.5 + eye_offset, v - 0.5, -0.72, 0.0],
        ray_direction_step: [
            (u - 0.5) * 0.42 + eye_offset * 0.25,
            (v - 0.5) * 0.32,
            1.0,
            preview_steps,
        ],
        volume_params: [frequency, phase, 0.72, 1.25],
        expected_rgba: [0.0; 4],
        expected_density_depth_status: [0.0; 4],
    };
    let expected = expected_bounded_stimulus_volume_raymarch_preview_output(pixel);
    pixel.expected_rgba = expected.rgba;
    pixel.expected_density_depth_status = expected.density_depth_status;
    pixel
}

/// CPU oracle for the bounded scalable image-preview RGBA output.
#[must_use]
pub fn expected_bounded_stimulus_volume_image_preview_output(
    pixel: BoundedStimulusVolumeImagePreviewPixel,
) -> BoundedStimulusVolumeImagePreviewOutput {
    BoundedStimulusVolumeImagePreviewOutput {
        rgba: expected_bounded_stimulus_volume_raymarch_preview_output(pixel).rgba,
    }
}

fn bounded_stimulus_volume_density(p: [f32; 3], uv: [f32; 4], params: [f32; 4]) -> f32 {
    let frequency = params[0].max(0.001);
    let phase = params[1];
    let opacity = params[2].clamp(0.0, 4.0);
    let wave_a =
        triangle_wave((p[0] + uv[0] * 0.25 + p[2] * 0.5) * frequency + uv[3] * 0.07 + phase);
    let wave_b = triangle_wave(
        (p[1] - p[2] * 0.35 + uv[1] * 0.25) * frequency * 0.75 - uv[3] * 0.11 + phase * 0.5,
    );
    let interference = (1.0 - (wave_a - wave_b).abs()).clamp(0.0, 1.0);
    (interference * opacity).clamp(0.0, 1.0)
}

fn bounded_stimulus_volume_frequency(grid_dimensions: [u64; 3]) -> f32 {
    let max_grid_axis = grid_dimensions.iter().copied().max().unwrap_or(32).max(1) as f32;
    (max_grid_axis / 8.0).clamp(1.0, 32.0)
}

fn bounded_stimulus_volume_phase(step_count: u64) -> f32 {
    0.37 + step_count as f32 * 0.003
}

fn triangle_wave(value: f32) -> f32 {
    ((value - value.floor()) * 2.0 - 1.0).abs()
}

fn scalable_sample_coordinate(sample: usize, sample_count: usize, tile_size: usize) -> usize {
    let sample_count = sample_count.max(1);
    let tile_size = tile_size.max(1);
    let center = tile_size / (sample_count * 2);
    ((sample * tile_size) / sample_count + center).min(tile_size.saturating_sub(1))
}
