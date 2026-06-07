use rusty_matter_model::Vec3;
use rusty_optics_model::{
    OpticsError, PARTICLE_BILLBOARD_BUILD_SCHEMA_ID, PARTICLE_RENDER_BUDGET_SCHEMA_ID,
};

use crate::ParticleVisualSample;

/// World-space basis used to place visual particle frames into a scene.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleSceneBasis {
    /// World-space center.
    pub center: Vec3,
    /// Local +X axis.
    pub right: Vec3,
    /// Local +Y axis.
    pub up: Vec3,
    /// Local +Z or forward axis.
    pub forward: Vec3,
    /// Uniform scale.
    pub scale: f32,
}

impl Default for ParticleSceneBasis {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            right: vec3_right(),
            up: vec3_up(),
            forward: vec3_forward(),
            scale: 1.0,
        }
    }
}

impl ParticleSceneBasis {
    /// Creates a scene basis.
    #[must_use]
    pub const fn new(center: Vec3, right: Vec3, up: Vec3, forward: Vec3, scale: f32) -> Self {
        Self {
            center,
            right,
            up,
            forward,
            scale,
        }
    }

    /// Returns a finite, normalized basis.
    #[must_use]
    pub fn normalized(self) -> Self {
        Self {
            center: if self.center.is_finite() {
                self.center
            } else {
                Vec3::ZERO
            },
            right: normalize_or(self.right, vec3_right()),
            up: normalize_or(self.up, vec3_up()),
            forward: normalize_or(self.forward, vec3_forward()),
            scale: if self.scale.is_finite() {
                self.scale
            } else {
                1.0
            },
        }
    }
}

/// Backend-neutral instance layout for animated particle billboards.
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ParticleBillboardInstance {
    /// World position plus visual radius.
    pub position_radius: [f32; 4],
    /// Linear RGBA color.
    pub color: [f32; 4],
    /// World normal plus animation frame phase.
    pub normal_frame: [f32; 4],
    /// Rotation, auxiliary values, and compact flags.
    pub aux: [f32; 4],
}

/// Billboard instance build configuration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleBillboardBuildConfig {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable build identifier.
    pub build_id: String,
    /// Maximum instances emitted.
    pub max_instances: usize,
    /// Minimum radius emitted.
    pub min_radius: f32,
    /// Minimum alpha emitted.
    pub min_alpha: f32,
    /// Whether instances should be sorted back-to-front.
    pub sort_back_to_front: bool,
}

impl ParticleBillboardBuildConfig {
    /// Creates a billboard build config.
    #[must_use]
    pub fn new(build_id: impl Into<String>) -> Self {
        Self {
            schema_id: PARTICLE_BILLBOARD_BUILD_SCHEMA_ID.to_owned(),
            build_id: build_id.into(),
            max_instances: usize::MAX,
            min_radius: 1.0e-6,
            min_alpha: 1.0e-6,
            sort_back_to_front: false,
        }
    }

    /// Returns a sanitized config for emission.
    #[must_use]
    pub fn sanitized(&self) -> Self {
        Self {
            schema_id: PARTICLE_BILLBOARD_BUILD_SCHEMA_ID.to_owned(),
            build_id: if self.build_id.trim().is_empty() {
                "particle.billboard.build.sanitized".to_owned()
            } else {
                self.build_id.clone()
            },
            max_instances: self.max_instances,
            min_radius: finite_or(self.min_radius, 0.0).max(0.0),
            min_alpha: finite_or(self.min_alpha, 0.0).clamp(0.0, 1.0),
            sort_back_to_front: self.sort_back_to_front,
        }
    }

    /// Validates build config shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PARTICLE_BILLBOARD_BUILD_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PARTICLE_BILLBOARD_BUILD_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.build_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("build_id"));
        }
        if !self.min_radius.is_finite() || self.min_radius < 0.0 {
            return Err(OpticsError::InvalidValue("min_radius"));
        }
        if !self.min_alpha.is_finite() || !(0.0..=1.0).contains(&self.min_alpha) {
            return Err(OpticsError::InvalidValue("min_alpha"));
        }
        Ok(())
    }
}

impl Default for ParticleBillboardBuildConfig {
    fn default() -> Self {
        Self::new("particle.billboard.build.default")
    }
}

/// Camera used for transparent billboard sorting.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleBillboardSortCamera {
    /// Camera position.
    pub position: Vec3,
    /// Camera forward direction.
    pub forward: Vec3,
}

impl Default for ParticleBillboardSortCamera {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            forward: vec3_forward(),
        }
    }
}

/// Billboard instance build statistics.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParticleBillboardBuildStats {
    /// Source sample count.
    pub source_count: usize,
    /// Emitted instance count.
    pub emitted_count: usize,
    /// Skipped source count.
    pub skipped_count: usize,
}

/// Builds one billboard instance from a visual particle sample.
#[must_use]
pub fn particle_billboard_instance(
    sample: &ParticleVisualSample,
    basis: ParticleSceneBasis,
) -> ParticleBillboardInstance {
    let basis = basis.normalized();
    particle_billboard_instance_from_normalized_basis(sample, basis)
}

/// Writes billboard instances into `out`.
#[must_use]
pub fn write_particle_billboard_instances(
    samples: &[ParticleVisualSample],
    basis: ParticleSceneBasis,
    config: &ParticleBillboardBuildConfig,
    sort_camera: Option<ParticleBillboardSortCamera>,
    sort_indices: &mut Vec<usize>,
    out: &mut Vec<ParticleBillboardInstance>,
) -> ParticleBillboardBuildStats {
    let basis = basis.normalized();
    let config = config.sanitized();
    let source_count = samples.len();
    let capped_count = source_count.min(config.max_instances);
    out.clear();

    if config.sort_back_to_front {
        sort_indices.clear();
        sort_indices.reserve(capped_count);
        for (index, sample) in samples.iter().take(capped_count).enumerate() {
            if should_emit_billboard_sample(sample, basis, &config) {
                sort_indices.push(index);
            }
        }

        if let Some(camera) = sort_camera {
            let forward = normalize_or(camera.forward, vec3_forward());
            sort_indices.sort_by(|left, right| {
                let left_depth = particle_billboard_depth_along_forward(
                    &samples[*left],
                    basis,
                    camera.position,
                    forward,
                );
                let right_depth = particle_billboard_depth_along_forward(
                    &samples[*right],
                    basis,
                    camera.position,
                    forward,
                );
                right_depth.total_cmp(&left_depth)
            });
        }

        out.reserve(sort_indices.len());
        for particle_index in sort_indices.iter().copied() {
            out.push(particle_billboard_instance_from_normalized_basis(
                &samples[particle_index],
                basis,
            ));
        }
    } else {
        out.reserve(capped_count);
        for sample in samples.iter().take(capped_count) {
            if should_emit_billboard_sample(sample, basis, &config) {
                out.push(particle_billboard_instance_from_normalized_basis(
                    sample, basis,
                ));
            }
        }
        sort_indices.clear();
    }

    ParticleBillboardBuildStats {
        source_count,
        emitted_count: out.len(),
        skipped_count: source_count.saturating_sub(out.len()),
    }
}

/// Returns depth along camera forward for one sample.
#[must_use]
pub fn particle_billboard_depth_along_forward(
    sample: &ParticleVisualSample,
    basis: ParticleSceneBasis,
    camera_position: Vec3,
    camera_forward: Vec3,
) -> f32 {
    let position = transform_point_with_normalized_basis(basis.normalized(), sample.position);
    (position - camera_position).dot(normalize_or(camera_forward, vec3_forward()))
}

/// Transparent particle billboard budget summary.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleBillboardRenderBudget {
    /// Schema identifier.
    pub schema_id: String,
    /// Stable budget identifier.
    pub budget_id: String,
    /// Source particle count.
    pub source_particles: usize,
    /// Active trail particle count.
    pub active_trails: usize,
    /// Total visible billboard instances.
    pub visible_instances: usize,
    /// Disc mesh segment count.
    pub disc_segments: usize,
    /// Indices per instance.
    pub indices_per_instance: usize,
    /// Total transparent billboard indices.
    pub total_indices: usize,
}

impl ParticleBillboardRenderBudget {
    /// Validates budget shape.
    ///
    /// # Errors
    ///
    /// Returns [`OpticsError`] when fields are invalid.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema_id != PARTICLE_RENDER_BUDGET_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: PARTICLE_RENDER_BUDGET_SCHEMA_ID,
                actual: self.schema_id.clone(),
            });
        }
        if self.budget_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("budget_id"));
        }
        if self.disc_segments < 3 {
            return Err(OpticsError::InvalidCount("disc_segments"));
        }
        if self.visible_instances != self.source_particles.saturating_add(self.active_trails) {
            return Err(OpticsError::InvalidPayload(
                "visible_instances must equal source_particles plus active_trails",
            ));
        }
        if self.indices_per_instance != self.disc_segments.saturating_mul(3) {
            return Err(OpticsError::InvalidPayload(
                "indices_per_instance must equal disc_segments * 3",
            ));
        }
        if self.total_indices
            != self
                .visible_instances
                .saturating_mul(self.indices_per_instance)
        {
            return Err(OpticsError::InvalidPayload(
                "total_indices must equal visible_instances * indices_per_instance",
            ));
        }
        Ok(())
    }
}

/// Estimates transparent particle billboard index pressure.
#[must_use]
pub fn particle_billboard_render_budget(
    budget_id: impl Into<String>,
    source_particles: usize,
    active_trails: usize,
    disc_segments: usize,
) -> ParticleBillboardRenderBudget {
    let disc_segments = disc_segments.max(3);
    let visible_instances = source_particles.saturating_add(active_trails);
    let indices_per_instance = disc_segments.saturating_mul(3);
    ParticleBillboardRenderBudget {
        schema_id: PARTICLE_RENDER_BUDGET_SCHEMA_ID.to_owned(),
        budget_id: budget_id.into(),
        source_particles,
        active_trails,
        visible_instances,
        disc_segments,
        indices_per_instance,
        total_indices: visible_instances.saturating_mul(indices_per_instance),
    }
}

fn particle_billboard_instance_from_normalized_basis(
    sample: &ParticleVisualSample,
    basis: ParticleSceneBasis,
) -> ParticleBillboardInstance {
    let position = transform_point_with_normalized_basis(basis, sample.position);
    let normal = normalize_or(
        transform_vector_with_normalized_basis(basis, sample.normal),
        vec3_up(),
    );
    let scale = basis.scale.abs();
    ParticleBillboardInstance {
        position_radius: [
            position.x,
            position.y,
            position.z,
            sample.radius.max(0.0) * scale,
        ],
        color: [
            sample.color.r,
            sample.color.g,
            sample.color.b,
            sample.color.a.clamp(0.0, 1.0),
        ],
        normal_frame: [normal.x, normal.y, normal.z, sample.frame01.clamp(0.0, 1.0)],
        aux: [
            sample.rotation_radians,
            sample.aux0,
            sample.aux1,
            sample.flags as f32,
        ],
    }
}

fn should_emit_billboard_sample(
    sample: &ParticleVisualSample,
    basis: ParticleSceneBasis,
    config: &ParticleBillboardBuildConfig,
) -> bool {
    sample.position.is_finite()
        && sample.radius.is_finite()
        && sample.radius * basis.scale.abs() >= config.min_radius
        && sample.color.is_finite()
        && sample.color.a >= config.min_alpha
        && sample.normal.is_finite()
        && sample.rotation_radians.is_finite()
        && sample.frame01.is_finite()
        && sample.aux0.is_finite()
        && sample.aux1.is_finite()
}

fn transform_point_with_normalized_basis(basis: ParticleSceneBasis, local: Vec3) -> Vec3 {
    basis.center + (transform_vector_with_normalized_basis(basis, local) * basis.scale)
}

fn transform_vector_with_normalized_basis(basis: ParticleSceneBasis, local: Vec3) -> Vec3 {
    (basis.right * local.x) + (basis.up * local.y) + (basis.forward * local.z)
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

fn vec3_right() -> Vec3 {
    Vec3::new(1.0, 0.0, 0.0)
}

fn vec3_up() -> Vec3 {
    Vec3::new(0.0, 1.0, 0.0)
}

fn vec3_forward() -> Vec3 {
    Vec3::new(0.0, 0.0, -1.0)
}
