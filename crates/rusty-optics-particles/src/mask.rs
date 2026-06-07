/// Configuration for a small morphed-ring billboard mask atlas.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MorphedRingMaskAtlasConfig {
    /// Width and height of each frame cell in pixels.
    pub frame_resolution_px: usize,
    /// Number of animation frames.
    pub frame_count: usize,
    /// Atlas columns.
    pub atlas_columns: usize,
    /// Core anti-aliased edge width in normalized cell units.
    pub edge_width: f32,
    /// Outer feather width in normalized cell units.
    pub outer_feather: f32,
    /// Ring radius in normalized cell units.
    pub ring_radius: f32,
    /// Ring thickness in normalized cell units.
    pub ring_thickness: f32,
    /// Offset between the paired morph traces.
    pub dual_offset_degrees: f32,
}

impl Default for MorphedRingMaskAtlasConfig {
    fn default() -> Self {
        Self {
            frame_resolution_px: 64,
            frame_count: 64,
            atlas_columns: 8,
            edge_width: 0.015,
            outer_feather: 0.06,
            ring_radius: 0.32,
            ring_thickness: 0.03,
            dual_offset_degrees: 180.0,
        }
    }
}

/// Generated morphed-ring RGBA mask atlas.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MorphedRingMaskAtlas {
    /// Atlas width in pixels.
    pub width_px: usize,
    /// Atlas height in pixels.
    pub height_px: usize,
    /// Frame cell width and height in pixels.
    pub frame_resolution_px: usize,
    /// Frame count.
    pub frame_count: usize,
    /// Column count.
    pub columns: usize,
    /// Row count.
    pub rows: usize,
    /// RGBA mask bytes.
    pub rgba: Vec<u8>,
}

/// Builds a CPU reference morphed-ring mask atlas.
#[must_use]
pub fn build_morphed_ring_mask_atlas_rgba(
    config: MorphedRingMaskAtlasConfig,
) -> MorphedRingMaskAtlas {
    let frame_resolution = config.frame_resolution_px.max(1);
    let frame_count = config.frame_count.max(1);
    let columns = config.atlas_columns.max(1);
    let rows = frame_count.div_ceil(columns);
    let width = frame_resolution * columns;
    let height = frame_resolution * rows;
    let mut rgba = vec![0_u8; width * height * 4];
    let aa = (1.0 / frame_resolution as f32).max(0.0001);

    for frame in 0..frame_count {
        let phase01 = if frame_count <= 1 {
            0.0
        } else {
            frame as f32 / (frame_count - 1) as f32
        };
        let frame_col = frame % columns;
        let frame_row = frame / columns;
        for y in 0..frame_resolution {
            let uv_y = (y as f32 + 0.5) / frame_resolution as f32;
            for x in 0..frame_resolution {
                let uv = [(x as f32 + 0.5) / frame_resolution as f32, uv_y];
                let distance = morphed_ring_distance(
                    uv,
                    phase01,
                    config.ring_radius,
                    config.ring_thickness,
                    config.dual_offset_degrees,
                );
                let core = 1.0 - smoothstep(config.edge_width, config.edge_width + aa, distance);
                let feather = 1.0
                    - smoothstep(
                        config.edge_width + aa,
                        config.edge_width + aa + config.outer_feather,
                        distance,
                    );
                let value = ((core.max(feather).clamp(0.0, 1.0) * 255.0) + 0.5)
                    .floor()
                    .clamp(0.0, 255.0) as u8;
                let atlas_x = frame_col * frame_resolution + x;
                let atlas_y = frame_row * frame_resolution + y;
                let offset = (atlas_y * width + atlas_x) * 4;
                rgba[offset..offset + 4].copy_from_slice(&[value, value, value, value]);
            }
        }
    }

    MorphedRingMaskAtlas {
        width_px: width,
        height_px: height,
        frame_resolution_px: frame_resolution,
        frame_count,
        columns,
        rows,
        rgba,
    }
}

/// Samples morphed-ring alpha at one normalized cell coordinate.
#[must_use]
pub fn morphed_ring_alpha(uv: [f32; 2], phase01: f32, config: MorphedRingMaskAtlasConfig) -> f32 {
    let resolution = config.frame_resolution_px.max(1) as f32;
    let aa = (1.0 / resolution).max(0.0001);
    let distance = morphed_ring_distance(
        uv,
        phase01,
        config.ring_radius,
        config.ring_thickness,
        config.dual_offset_degrees,
    );
    let core = 1.0 - smoothstep(config.edge_width, config.edge_width + aa, distance);
    let feather = 1.0
        - smoothstep(
            config.edge_width + aa,
            config.edge_width + aa + config.outer_feather,
            distance,
        );
    core.max(feather).clamp(0.0, 1.0)
}

fn morphed_ring_distance(
    uv: [f32; 2],
    phase01: f32,
    ring_radius: f32,
    ring_thickness: f32,
    dual_offset_degrees: f32,
) -> f32 {
    let p = [(uv[0] - 0.5) * 2.0, (uv[1] - 0.5) * 2.0];
    let full_offset = dual_offset_degrees.to_radians();
    let dynamic_offset = full_offset * ((phase01.clamp(0.0, 1.0) * 2.0) - 1.0).abs();
    let half_offset = dynamic_offset * 0.5;
    morphed_ring_distance_single(p, phase01, -half_offset, ring_radius, ring_thickness).min(
        morphed_ring_distance_single(p, phase01, half_offset, ring_radius, ring_thickness),
    )
}

fn morphed_ring_distance_single(
    p: [f32; 2],
    phase01: f32,
    rotation_offset: f32,
    ring_radius: f32,
    ring_thickness: f32,
) -> f32 {
    let morph = morph_factor(phase01);
    let mid_radius = ((ring_radius - (0.5 * ring_thickness)) * 2.0).max(0.0001);
    let arc_segments = 12usize;
    let mut best = f32::INFINITY;
    for arc in 0..3 {
        let a0 = rotation_offset + (arc as f32 * core::f32::consts::TAU / 3.0);
        let a1 = rotation_offset + ((arc + 1) as f32 * core::f32::consts::TAU / 3.0);
        let mut previous = morphed_arc_point(a0, a1, 0.0, morph, mid_radius);
        for index in 1..=arc_segments {
            let point = morphed_arc_point(
                a0,
                a1,
                index as f32 / arc_segments as f32,
                morph,
                mid_radius,
            );
            best = best.min(segment_distance(p, previous, point));
            previous = point;
        }
    }
    best
}

fn morphed_arc_point(a0: f32, a1: f32, s: f32, morph: f32, radius: f32) -> [f32; 2] {
    let angle = a0 + ((a1 - a0) * s);
    let circle = [angle.cos() * radius, angle.sin() * radius];
    let triangle = if s < 0.5 {
        let t = s * 2.0;
        let start = [a0.cos() * radius, a0.sin() * radius];
        let mid_angle = a0 + ((a1 - a0) * 0.5);
        let mid = [mid_angle.cos() * radius, mid_angle.sin() * radius];
        lerp2(start, mid, t)
    } else {
        let t = (s - 0.5) * 2.0;
        let mid_angle = a0 + ((a1 - a0) * 0.5);
        let mid = [mid_angle.cos() * radius, mid_angle.sin() * radius];
        let end = [a1.cos() * radius, a1.sin() * radius];
        lerp2(mid, end, t)
    };
    lerp2(circle, triangle, morph)
}

fn morph_factor(phase01: f32) -> f32 {
    let phase = phase01.clamp(0.0, 1.0);
    let triangle = if phase < 0.5 {
        phase * 2.0
    } else {
        (1.0 - phase) * 2.0
    };
    smoothstep(0.0, 1.0, triangle)
}

fn segment_distance(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let denom = ab[0].mul_add(ab[0], ab[1] * ab[1]).max(0.000_001);
    let t = ((ap[0].mul_add(ab[0], ap[1] * ab[1])) / denom).clamp(0.0, 1.0);
    let closest = [a[0] + (ab[0] * t), a[1] + (ab[1] * t)];
    let dx = p[0] - closest[0];
    let dy = p[1] - closest[1];
    dx.mul_add(dx, dy * dy).sqrt()
}

fn lerp2(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + ((b[0] - a[0]) * t), a[1] + ((b[1] - a[1]) * t)]
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge0 == edge1 {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - (2.0 * t))
}
