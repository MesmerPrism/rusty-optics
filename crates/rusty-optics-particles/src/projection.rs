use rusty_matter_model::Vec3;
use rusty_optics_model::{
    ColorRgba, OpticsError, Vec2, PARTICLE_FLAT_FRAME_SCHEMA_ID, PARTICLE_FLAT_PROJECTION_SCHEMA_ID,
};

use crate::{ParticleVisualFrame, ParticleVisualSample};

/// Camera/projection settings for flat particle inspection.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct FlatScreenProjectionConfig {
    /// Schema identifier.
    pub schema_id: String,
    /// Viewport width in pixels.
    pub viewport_width_px: u32,
    /// Viewport height in pixels.
    pub viewport_height_px: u32,
    /// Camera distance from the projection center.
    pub camera_distance: f32,
    /// Vertical field of view in degrees.
    pub vertical_fov_degrees: f32,
    /// Projection center in source coordinate space.
    pub center: Vec3,
    /// Yaw rotation in radians.
    pub yaw_radians: f32,
    /// Pitch rotation in radians.
    pub pitch_radians: f32,
    /// Roll rotation in radians.
    pub roll_radians: f32,
    /// Near plane distance.
    pub near: f32,
    /// Far plane distance.
    pub far: f32,
    /// Radius multiplier applied after perspective projection.
    pub point_radius_multiplier: f32,
    /// Minimum projected radius in pixels.
    pub min_radius_px: f32,
    /// Maximum projected radius in pixels.
    pub max_radius_px: f32,
    /// Whether offscreen particles are culled.
    pub cull_offscreen: bool,
}

impl Default for FlatScreenProjectionConfig {
    fn default() -> Self {
        Self {
            schema_id: PARTICLE_FLAT_PROJECTION_SCHEMA_ID.to_owned(),
            viewport_width_px: 960,
            viewport_height_px: 720,
            camera_distance: 6.0,
            vertical_fov_degrees: 52.0,
            center: Vec3::ZERO,
            yaw_radians: 0.0,
            pitch_radians: 0.0,
            roll_radians: 0.0,
            near: 0.05,
            far: 32.0,
            point_radius_multiplier: 1.0,
            min_radius_px: 1.0,
            max_radius_px: 18.0,
            cull_offscreen: true,
        }
    }
}

impl FlatScreenProjectionConfig {
    /// Returns a finite, bounded projection config.
    #[must_use]
    pub fn sanitized(&self) -> Self {
        let viewport_width_px = self.viewport_width_px.max(1);
        let viewport_height_px = self.viewport_height_px.max(1);
        let near = finite_or(self.near, 0.05).max(0.001);
        let far = finite_or(self.far, 32.0).max(near + 0.001);
        let min_radius_px = finite_or(self.min_radius_px, 1.0).max(0.0);
        let max_radius_px = finite_or(self.max_radius_px, 18.0).max(min_radius_px);
        Self {
            schema_id: PARTICLE_FLAT_PROJECTION_SCHEMA_ID.to_owned(),
            viewport_width_px,
            viewport_height_px,
            camera_distance: finite_or(self.camera_distance, 6.0).max(0.001),
            vertical_fov_degrees: finite_or(self.vertical_fov_degrees, 52.0).clamp(5.0, 160.0),
            center: if self.center.is_finite() {
                self.center
            } else {
                Vec3::ZERO
            },
            yaw_radians: finite_or(self.yaw_radians, 0.0),
            pitch_radians: finite_or(self.pitch_radians, 0.0),
            roll_radians: finite_or(self.roll_radians, 0.0),
            near,
            far,
            point_radius_multiplier: finite_or(self.point_radius_multiplier, 1.0).max(0.0),
            min_radius_px,
            max_radius_px,
            cull_offscreen: self.cull_offscreen,
        }
    }

    /// Validates projection config shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PARTICLE_FLAT_PROJECTION_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PARTICLE_FLAT_PROJECTION_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.viewport_width_px == 0 || self.viewport_height_px == 0 {
            return Err(OpticsError::InvalidCount("viewport"));
        }
        if !self.center.is_finite() {
            return Err(OpticsError::NonFiniteVec3("center"));
        }
        if !self.camera_distance.is_finite() || self.camera_distance <= 0.0 {
            return Err(OpticsError::InvalidValue("camera_distance"));
        }
        if !self.vertical_fov_degrees.is_finite()
            || !(5.0..=160.0).contains(&self.vertical_fov_degrees)
        {
            return Err(OpticsError::InvalidValue("vertical_fov_degrees"));
        }
        if !self.near.is_finite()
            || !self.far.is_finite()
            || self.near <= 0.0
            || self.far <= self.near
        {
            return Err(OpticsError::InvalidValue("near/far"));
        }
        if !self.point_radius_multiplier.is_finite() || self.point_radius_multiplier < 0.0 {
            return Err(OpticsError::InvalidValue("point_radius_multiplier"));
        }
        if !self.min_radius_px.is_finite()
            || !self.max_radius_px.is_finite()
            || self.min_radius_px < 0.0
            || self.max_radius_px < self.min_radius_px
        {
            return Err(OpticsError::InvalidValue("radius_px"));
        }
        Ok(())
    }
}

/// One projected flat particle sample.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct FlatParticleSample {
    /// Source sample index.
    pub source_index: usize,
    /// Source particle identifier.
    pub source_particle_id: String,
    /// Projected center in pixels.
    pub center_px: Vec2,
    /// Projected radius in pixels.
    pub radius_px: f32,
    /// Normalized camera depth where `0` is near and `1` is far.
    pub depth01: f32,
    /// Camera depth in source units.
    pub camera_depth: f32,
    /// Normal-facing score against the view direction.
    pub facing01: f32,
    /// Visual color.
    pub color: ColorRgba,
    /// Visual flags.
    pub flags: u32,
    /// Rotation about the facing axis.
    pub rotation_radians: f32,
    /// Animation frame phase.
    pub frame01: f32,
    /// Renderer-neutral auxiliary value.
    pub aux0: f32,
    /// Renderer-neutral auxiliary value.
    pub aux1: f32,
}

impl FlatParticleSample {
    /// Validates flat sample shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.source_particle_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_particle_id"));
        }
        if !self.center_px.is_finite() {
            return Err(OpticsError::NonFiniteVec2("center_px"));
        }
        if !self.radius_px.is_finite() || self.radius_px < 0.0 {
            return Err(OpticsError::InvalidValue("radius_px"));
        }
        if !self.depth01.is_finite() || !(0.0..=1.0).contains(&self.depth01) {
            return Err(OpticsError::InvalidValue("depth01"));
        }
        if !self.camera_depth.is_finite() || self.camera_depth < 0.0 {
            return Err(OpticsError::InvalidValue("camera_depth"));
        }
        if !self.facing01.is_finite() || !(0.0..=1.0).contains(&self.facing01) {
            return Err(OpticsError::InvalidValue("facing01"));
        }
        if !self.color.is_finite() {
            return Err(OpticsError::NonFiniteColor("color"));
        }
        if !self.rotation_radians.is_finite()
            || !self.frame01.is_finite()
            || !self.aux0.is_finite()
            || !self.aux1.is_finite()
        {
            return Err(OpticsError::InvalidValue("flat particle scalar"));
        }
        Ok(())
    }
}

/// Flat projected particle frame sorted for transparent compositing.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct FlatParticleFrame {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable frame identifier.
    pub frame_id: String,
    /// Source visual frame identifier.
    pub source_frame_id: String,
    /// Viewport width in pixels.
    pub viewport_width_px: u32,
    /// Viewport height in pixels.
    pub viewport_height_px: u32,
    /// Source particle count.
    pub source_particle_count: usize,
    /// Visible projected particle count.
    pub visible_particle_count: usize,
    /// Far-to-near sorted particles.
    pub particles: Vec<FlatParticleSample>,
}

impl FlatParticleFrame {
    /// Validates flat frame shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PARTICLE_FLAT_FRAME_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PARTICLE_FLAT_FRAME_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.frame_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("frame_id"));
        }
        if self.source_frame_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("source_frame_id"));
        }
        if self.viewport_width_px == 0 || self.viewport_height_px == 0 {
            return Err(OpticsError::InvalidCount("viewport"));
        }
        if self.visible_particle_count != self.particles.len() {
            return Err(OpticsError::InvalidPayload(
                "visible_particle_count must equal particles length",
            ));
        }
        for particle in &self.particles {
            particle.validate()?;
        }
        Ok(())
    }
}

/// Projects visual particles to a flat screen frame.
///
/// # Errors
///
/// Returns [`OpticsError`] when the resulting frame is invalid.
pub fn project_particles_for_flat_screen(
    frame_id: impl Into<String>,
    visual_frame: &ParticleVisualFrame,
    config: &FlatScreenProjectionConfig,
) -> Result<FlatParticleFrame, OpticsError> {
    visual_frame.validate()?;
    let config = config.sanitized();
    config.validate()?;
    let half_width = config.viewport_width_px as f32 * 0.5;
    let half_height = config.viewport_height_px as f32 * 0.5;
    let fov_y_rad = config.vertical_fov_degrees.to_radians();
    let focal_px = half_height / (fov_y_rad * 0.5).tan();
    let depth_range = (config.far - config.near).max(0.001);

    let mut projected = Vec::with_capacity(visual_frame.samples.len());
    for (source_index, sample) in visual_frame.samples.iter().enumerate() {
        if !sample_is_projectable(sample) {
            continue;
        }
        let centered = sample.position - config.center;
        let view = rotate_yaw_pitch_roll(
            centered,
            config.yaw_radians,
            config.pitch_radians,
            config.roll_radians,
        );
        let view_normal = normalize_or(
            rotate_yaw_pitch_roll(
                sample.normal,
                config.yaw_radians,
                config.pitch_radians,
                config.roll_radians,
            ),
            vec3_up(),
        );
        let camera_depth = config.camera_distance - view.z;
        if camera_depth < config.near || camera_depth > config.far {
            continue;
        }

        let eye_dir = normalize_or(Vec3::new(-view.x, -view.y, camera_depth), vec3_forward());
        let facing01 = view_normal.dot(eye_dir).clamp(0.0, 1.0);
        let perspective = focal_px / camera_depth.max(0.001);
        let x_px = half_width + (view.x * perspective);
        let y_px = half_height - (view.y * perspective);
        let radius_px = (sample.radius.max(0.0) * config.point_radius_multiplier * perspective)
            .clamp(config.min_radius_px, config.max_radius_px);
        if config.cull_offscreen
            && (x_px < -radius_px
                || x_px > config.viewport_width_px as f32 + radius_px
                || y_px < -radius_px
                || y_px > config.viewport_height_px as f32 + radius_px)
        {
            continue;
        }

        projected.push(FlatParticleSample {
            source_index,
            source_particle_id: sample.source_particle_id.clone(),
            center_px: Vec2::new(x_px, y_px),
            radius_px,
            depth01: ((camera_depth - config.near) / depth_range).clamp(0.0, 1.0),
            camera_depth,
            facing01,
            color: sample.color,
            flags: sample.flags,
            rotation_radians: sample.rotation_radians,
            frame01: sample.frame01,
            aux0: sample.aux0,
            aux1: sample.aux1,
        });
    }

    projected.sort_by(|left, right| right.camera_depth.total_cmp(&left.camera_depth));
    let frame = FlatParticleFrame {
        schema_id: PARTICLE_FLAT_FRAME_SCHEMA_ID.to_owned(),
        frame_id: frame_id.into(),
        source_frame_id: visual_frame.frame_id.clone(),
        viewport_width_px: config.viewport_width_px,
        viewport_height_px: config.viewport_height_px,
        source_particle_count: visual_frame.samples.len(),
        visible_particle_count: projected.len(),
        particles: projected,
    };
    frame.validate()?;
    Ok(frame)
}

fn sample_is_projectable(sample: &ParticleVisualSample) -> bool {
    sample.position.is_finite()
        && sample.radius.is_finite()
        && sample.radius > 0.0
        && sample.color.is_finite()
        && sample.color.a > 0.0
        && sample.normal.is_finite()
        && sample.rotation_radians.is_finite()
        && sample.frame01.is_finite()
}

fn rotate_yaw_pitch_roll(position: Vec3, yaw_rad: f32, pitch_rad: f32, roll_rad: f32) -> Vec3 {
    let (yaw_sin, yaw_cos) = yaw_rad.sin_cos();
    let yawed = Vec3::new(
        position.x.mul_add(yaw_cos, position.z * yaw_sin),
        position.y,
        (-position.x).mul_add(yaw_sin, position.z * yaw_cos),
    );

    let (pitch_sin, pitch_cos) = pitch_rad.sin_cos();
    let pitched = Vec3::new(
        yawed.x,
        yawed.y.mul_add(pitch_cos, -(yawed.z * pitch_sin)),
        yawed.y.mul_add(pitch_sin, yawed.z * pitch_cos),
    );

    let (roll_sin, roll_cos) = roll_rad.sin_cos();
    Vec3::new(
        pitched.x.mul_add(roll_cos, -(pitched.y * roll_sin)),
        pitched.x.mul_add(roll_sin, pitched.y * roll_cos),
        pitched.z,
    )
}

fn normalize_or(value: Vec3, fallback: Vec3) -> Vec3 {
    let length = value.length();
    if value.is_finite() && length.is_finite() && length > 1.0e-6 {
        value / length
    } else {
        fallback
    }
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

fn vec3_up() -> Vec3 {
    Vec3::new(0.0, 1.0, 0.0)
}

fn vec3_forward() -> Vec3 {
    Vec3::new(0.0, 0.0, -1.0)
}
