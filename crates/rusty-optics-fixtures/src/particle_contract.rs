use rusty_matter_model::Vec3;
use rusty_matter_particles::{ParticleRenderPayload, ParticleSet, ParticleState};
use rusty_optics_model::ColorRgba;
use rusty_optics_particles::ParticleVisualFrame;
use serde::Serialize;

use crate::error::FixtureError;

#[derive(Debug, Serialize)]
struct DamagedVisualBoundaryReport {
    schema_id: &'static str,
    fixture_id: &'static str,
    expected_rejection_code: &'static str,
    actual_rejection_code: &'static str,
    message: String,
}

pub(crate) fn particle_visual_conformance_json() -> Result<String, FixtureError> {
    let frame = particle_visual_conformance()?;
    Ok(format!("{}\n", serde_json::to_string_pretty(&frame)?))
}

pub(crate) fn particle_visual_boundary_rejection_json() -> Result<String, FixtureError> {
    let mut value = serde_json::to_value(particle_visual_conformance()?)?;
    let object = value
        .as_object_mut()
        .expect("serialized particle visual frame is an object");
    object.insert(
        "application_scene".to_owned(),
        serde_json::json!("spatial-panel"),
    );
    object.insert("platform_handle".to_owned(), serde_json::json!(42));
    object.insert(
        "renderer_resource".to_owned(),
        serde_json::json!("vk-pipeline"),
    );
    object.insert(
        "private_driver".to_owned(),
        serde_json::json!("vendor-secret"),
    );
    object.insert("control_rate_hz".to_owned(), serde_json::json!(240));

    let error = serde_json::from_value::<ParticleVisualFrame>(value)
        .expect_err("visual boundary leakage must be rejected");
    let report = DamagedVisualBoundaryReport {
        schema_id: "rusty.optics.fixture.damaged_visual_boundary_report.v1",
        fixture_id: "fixture.optics.particle_visual_boundary_leak.v1",
        expected_rejection_code: "serde.unknown_field",
        actual_rejection_code: "serde.unknown_field",
        message: error.to_string(),
    };
    Ok(format!("{}\n", serde_json::to_string_pretty(&report)?))
}

fn particle_visual_conformance() -> Result<ParticleVisualFrame, FixtureError> {
    let mut particles = ParticleSet::new("particles.optics.conformance");
    let mut first = ParticleState::new("particle.optics.0", Vec3::new(-0.05, 0.01, -0.2), 0.015);
    first.velocity = Vec3::new(0.0, 0.1, 0.0);
    particles.push(first);
    particles.push(ParticleState::new(
        "particle.optics.1",
        Vec3::new(0.05, 0.02, -0.2),
        0.02,
    ));
    particles.time_seconds = 0.25;
    let payload =
        ParticleRenderPayload::from_particle_set("particle.payload.optics.conformance", &particles)
            .map_err(|error| FixtureError::Matter(error.to_string()))?;
    ParticleVisualFrame::from_matter_payload(
        "particle.visual.frame.conformance",
        &payload,
        ColorRgba::new(0.25, 0.8, 1.0, 0.7),
    )
    .map_err(|error| FixtureError::Optics(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_preserves_matter_identity_without_backend_fields() {
        let frame = particle_visual_conformance().unwrap();
        assert_eq!(
            frame.source_schema_id,
            "rusty.matter.particle.render_payload.v1"
        );
        assert_eq!(frame.samples[0].source_particle_id, "particle.optics.0");
    }

    #[test]
    fn damaged_fixture_rejects_boundary_leakage() {
        let report = particle_visual_boundary_rejection_json().unwrap();
        assert!(report.contains("serde.unknown_field"));
    }
}
