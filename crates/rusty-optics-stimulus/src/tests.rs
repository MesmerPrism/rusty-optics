use rusty_optics_model::{OpticsError, Vec2};

use crate::{
    sample_profile, BasePatternKind, ComputePassKind, ComputeWorkgroupSize, KernelExecutionModel,
    LayerOscillatorTarget, NoiseControls, ParkMillerRng, PresentationReferenceSpace,
    RunPlanFeasibility, StimulusCoverageMode, StimulusKernelAbi, StimulusLayer, StimulusOscillator,
    StimulusPresentationDescriptor, StimulusPresentationMode, StimulusProfile, StimulusRunPlan,
    StimulusSafetyProfile, StimulusTemporalProfile,
};

#[test]
fn interference_preview_validates_and_samples() {
    let profile = StimulusProfile::interference_preview("stimulus.profile.interference_preview");
    profile.validate().expect("preview profile should validate");

    let sample = sample_profile(&profile, Vec2::new(0.25, 0.75), 1.1)
        .expect("preview profile should sample");
    assert!((0.0..=1.0).contains(&sample.luma));
    assert!(sample.color.is_finite());
}

#[test]
fn black_lead_in_forces_dark_sample() {
    let profile = StimulusProfile::interference_preview("stimulus.profile.interference_preview");
    let sample =
        sample_profile(&profile, Vec2::new(0.5, 0.5), 0.25).expect("preview profile should sample");
    assert_eq!(sample.luma, 0.0);
}

#[test]
fn interference_preview_targets_fullscreen_stereo_eye_field() {
    let profile = StimulusProfile::interference_preview("stimulus.profile.interference_preview");

    assert_eq!(
        profile.presentation.mode,
        StimulusPresentationMode::StereoEyeField
    );
    assert_eq!(
        profile.presentation.coverage,
        StimulusCoverageMode::FullViewport
    );
    assert_eq!(
        profile.presentation.reference_space,
        PresentationReferenceSpace::ViewLocked
    );
    assert_eq!(profile.presentation.eye_count, 2);
    assert!(profile.presentation.prefer_xr_composition_layer);
}

#[test]
fn stereo_eye_field_rejects_non_fullscreen_reference() {
    let mut presentation =
        StimulusPresentationDescriptor::stereo_eye_field("stimulus.presentation.bad");
    presentation.reference_space = PresentationReferenceSpace::WorldSpace;

    assert!(matches!(
        presentation.validate(),
        Err(OpticsError::InvalidValue("presentation.reference_space"))
    ));
}

#[test]
fn run_plan_quantizes_against_refresh() {
    let profile = StimulusProfile::interference_preview("stimulus.profile.interference_preview");
    let plan = profile
        .run_plan("stimulus.run_plan.preview.72hz", 72.0)
        .expect("run plan should build");

    assert_eq!(plan.feasibility, RunPlanFeasibility::Exact);
    assert_eq!(plan.frames_per_cycle, 36);
    assert_eq!(plan.on_frames_per_cycle, 18);
    assert_eq!(plan.off_frames_per_cycle, 18);
}

#[test]
fn over_budget_run_plan_reports_feasibility() {
    let mut temporal = StimulusTemporalProfile::preview_pulse("stimulus.temporal.fast");
    temporal.target_cycle_hz = 60.0;

    let plan = StimulusRunPlan::from_temporal("stimulus.run_plan.fast.90hz", &temporal, 90.0)
        .expect("over-budget plan should still describe quantization");

    assert_eq!(plan.feasibility, RunPlanFeasibility::ExceedsSwitchBudget);
    assert_eq!(plan.frames_per_cycle, 2);
    assert_eq!(plan.on_frames_per_cycle, 1);
    assert_eq!(plan.off_frames_per_cycle, 1);
}

#[test]
fn risk_profiles_reject_autostart() {
    let mut profile = StimulusProfile::interference_preview("stimulus.profile.risk");
    profile.temporal.target_cycle_hz = 12.0;
    profile.temporal.duration_seconds = 10.0;
    profile.temporal.start_gate_required = false;
    profile.safety = StimulusSafetyProfile::photosensitive_risk("stimulus.safety.risk");

    assert!(matches!(
        profile.validate(),
        Err(OpticsError::InvalidValue("temporal.start_gate_required"))
    ));
}

#[test]
fn research_protocol_delegates_session_gate_to_external_protocol() {
    let mut temporal = StimulusTemporalProfile::preview_pulse("stimulus.temporal.research");
    temporal.target_cycle_hz = 60.0;
    temporal.duration_seconds = 120.0;
    temporal.black_lead_in_seconds = 0.0;
    temporal.start_gate_required = false;

    let safety = StimulusSafetyProfile::research_protocol("stimulus.safety.research");
    safety
        .validate_temporal(&temporal)
        .expect("research protocol should allow external session gating");
}

#[test]
fn seeded_noise_is_stable() {
    let mut layer = StimulusLayer::new("stimulus.layer.noise", BasePatternKind::NoiseField, 8.0);
    layer.seed = 42;

    let mut profile = StimulusProfile::interference_preview("stimulus.profile.noise");
    profile.layer_graph.layers = vec![layer];
    profile.layer_graph.post.contrast = 1.0;
    profile.layer_graph.post.brightness = 0.0;

    let a =
        sample_profile(&profile, Vec2::new(0.125, 0.25), 1.2).expect("noise sample should work");
    let b =
        sample_profile(&profile, Vec2::new(0.125, 0.25), 1.2).expect("noise sample should work");
    assert_eq!(a, b);
}

#[test]
fn perlin_noise_layer_is_deterministic_and_seeded() {
    let mut layer = StimulusLayer::perlin_noise("stimulus.layer.perlin", 3.0);
    layer.seed = 7;
    layer.noise.domain_warp_strength = 0.05;
    layer.noise.animation_velocity = Vec2::new(0.02, -0.01);

    let mut profile = StimulusProfile::interference_preview("stimulus.profile.perlin");
    profile.layer_graph.layers = vec![layer.clone()];
    profile.layer_graph.post.contrast = 1.0;
    profile.layer_graph.post.brightness = 0.0;

    let a =
        sample_profile(&profile, Vec2::new(0.375, 0.625), 1.1).expect("perlin sample should work");
    let b =
        sample_profile(&profile, Vec2::new(0.375, 0.625), 1.1).expect("perlin sample should work");
    assert_eq!(a, b);

    profile.layer_graph.layers[0].seed = 8;
    let c =
        sample_profile(&profile, Vec2::new(0.375, 0.625), 1.1).expect("perlin sample should work");
    assert_ne!(a.luma, c.luma);
}

#[test]
fn noise_controls_reject_unbounded_octaves_and_warp() {
    let mut noise = NoiseControls::perlin(9);
    assert!(matches!(
        noise.validate(),
        Err(OpticsError::InvalidCount("noise.octaves"))
    ));

    noise = NoiseControls::perlin(4);
    noise.domain_warp_strength = 5.0;
    assert!(matches!(
        noise.validate(),
        Err(OpticsError::InvalidValue("noise.domain_warp_strength"))
    ));
}

#[test]
fn oscillator_modulates_layer_phase() {
    let mut layer = StimulusLayer::stripes("stimulus.layer.oscillated", 4.0);
    layer.temporal.speed_hz = 0.0;
    layer.oscillators.push(StimulusOscillator::sine(
        "stimulus.oscillator.phase",
        LayerOscillatorTarget::PhaseOffset,
        0.25,
        0.25,
    ));

    let mut profile = StimulusProfile::interference_preview("stimulus.profile.oscillated");
    profile.layer_graph.layers = vec![layer];
    profile.layer_graph.post.contrast = 1.0;
    profile.layer_graph.post.brightness = 0.0;
    profile.layer_graph.post.trail_decay = 0.0;
    profile.kernel_abi = StimulusKernelAbi::procedural_v1("stimulus.kernel.procedural_v1");

    let early =
        sample_profile(&profile, Vec2::new(0.20, 0.5), 1.0).expect("early sample should work");
    let later =
        sample_profile(&profile, Vec2::new(0.20, 0.5), 2.0).expect("later sample should work");

    assert_ne!(early.luma, later.luma);
}

#[test]
fn ripple_and_interference_layers_sample_deterministically() {
    let mut ripple = StimulusLayer::ripple("stimulus.layer.ripple.test", 6.0);
    ripple.interference.radial_decay = 0.8;
    let mut interference = StimulusLayer::interference("stimulus.layer.interference.test", 5.0);
    interference.interference.source_b_weight = 0.7;
    interference.interference.wave_modulation = 0.25;

    let mut profile = StimulusProfile::interference_preview("stimulus.profile.interference.test");
    profile.layer_graph.layers = vec![ripple, interference];

    let a = sample_profile(&profile, Vec2::new(0.37, 0.62), 1.3).expect("sample should validate");
    let b = sample_profile(&profile, Vec2::new(0.37, 0.62), 1.3).expect("sample should validate");

    assert_eq!(a, b);
    assert!((0.0..=1.0).contains(&a.luma));
}

#[test]
fn compute_interference_abi_describes_history_and_readback_passes() {
    let abi = StimulusKernelAbi::compute_interference_v1("stimulus.kernel.compute");
    abi.validate().expect("compute ABI should validate");

    assert_eq!(
        abi.preferred_execution_model,
        KernelExecutionModel::ComputeFieldTexture
    );
    assert!(abi.requires_history_buffer);
    assert_eq!(abi.portability.max_workgroup_invocations, 64);
    assert!(!abi.portability.requires_subgroup_ops);
    assert!(!abi.portability.requires_device_address);
    assert!(!abi.portability.requires_runtime_descriptor_arrays);
    assert!(abi
        .compute_passes
        .iter()
        .any(|pass| pass.kind == ComputePassKind::HistoryFeedback
            && pass.reads_history_buffer
            && pass.writes_history_buffer));
    assert!(abi
        .compute_passes
        .iter()
        .any(|pass| pass.kind == ComputePassKind::BoundedReadbackProbe
            && pass.bounded_readback_samples == 512));
}

#[test]
fn portability_profile_rejects_non_mobile_workgroups() {
    let mut abi = StimulusKernelAbi::compute_interference_v1("stimulus.kernel.compute");
    abi.compute_passes[0].workgroup_size = ComputeWorkgroupSize::new(16, 16, 1);

    assert!(matches!(
        abi.validate(),
        Err(OpticsError::InvalidValue(
            "compute_pass.workgroup_invocations"
        ))
    ));
}

#[cfg(feature = "serde")]
#[test]
fn interference_fixture_deserializes_and_validates() {
    let json = include_str!("../../../fixtures/stimulus/interference_preview_profile.json");
    let profile: StimulusProfile =
        serde_json::from_str(json).expect("stimulus fixture should deserialize");

    profile
        .validate()
        .expect("stimulus fixture should validate");

    let noise_layer = profile
        .layer_graph
        .layers
        .last()
        .expect("fixture should include a noise layer");
    assert_eq!(noise_layer.pattern, BasePatternKind::PerlinNoise);
    assert_eq!(profile.kernel_abi.portability.max_workgroup_invocations, 64);
    assert!(!profile.kernel_abi.portability.requires_subgroup_ops);
}

#[test]
fn park_miller_sequence_is_repeatable() {
    let mut a = ParkMillerRng::new(123);
    let mut b = ParkMillerRng::new(123);
    for _ in 0..8 {
        assert_eq!(a.next_unit(), b.next_unit());
    }
}
