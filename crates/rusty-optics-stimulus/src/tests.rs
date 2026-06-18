use rusty_optics_model::{OpticsError, Vec2};

use crate::{
    deterministic_bounded_stimulus_volume_image_preview_pixel,
    deterministic_bounded_stimulus_volume_probe_sample,
    deterministic_bounded_stimulus_volume_raymarch_preview_pixel, deterministic_volume_probe_rays,
    expected_bounded_stimulus_volume_image_preview_output,
    expected_bounded_stimulus_volume_probe_output,
    expected_bounded_stimulus_volume_raymarch_preview_output, sample_profile,
    sample_volume_probe_set, BasePatternKind, ComputePassKind, ComputeWorkgroupSize,
    KernelExecutionModel, LayerOscillatorTarget, NoiseControls, ParkMillerRng,
    PresentationReferenceSpace, RunPlanFeasibility, StimulusCoverageMode, StimulusKernelAbi,
    StimulusLayer, StimulusOscillator, StimulusPresentationDescriptor, StimulusPresentationMode,
    StimulusProfile, StimulusRunPlan, StimulusSafetyProfile, StimulusTemporalProfile,
    StimulusVolumeDescriptor, StimulusVolumeFieldKind, StimulusVolumeProbeStatus,
    StimulusVolumeProfileSummary, StimulusVolumeStorageHint,
    BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_TILE_HEIGHT,
    BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_TILE_WIDTH,
    BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_PIXELS,
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
fn volume_descriptor_validates_mobile_storage_shape() {
    let volume =
        StimulusVolumeDescriptor::procedural_layer_stack_3d("stimulus.volume.interference_probe");

    volume
        .validate()
        .expect("volume descriptor should validate");

    assert_eq!(
        volume.field_kind,
        StimulusVolumeFieldKind::ProceduralLayerStack3d
    );
    assert_eq!(
        volume.storage_hint,
        StimulusVolumeStorageHint::StorageBuffer
    );
    assert_eq!(volume.grid_dimensions, [32, 32, 32]);
    assert_eq!(volume.voxel_count(), 32 * 32 * 32);
    assert_eq!(volume.step_count, 32);
    assert!(!volume.empty_space_skip_hint);
}

#[test]
fn volume_descriptor_rejects_invalid_bounds_and_steps() {
    let mut volume = StimulusVolumeDescriptor::procedural_layer_stack_3d("stimulus.volume.bad");
    volume.bounds_max[2] = volume.bounds_min[2];

    assert!(matches!(
        volume.validate(),
        Err(OpticsError::InvalidValue("volume.bounds"))
    ));

    volume = StimulusVolumeDescriptor::procedural_layer_stack_3d("stimulus.volume.too_many_steps");
    volume.step_count = 512;

    assert!(matches!(
        volume.validate(),
        Err(OpticsError::InvalidCount("volume.step_count"))
    ));
}

#[test]
fn volume_compute_abi_describes_probe_and_stereo_passes() {
    let abi = StimulusKernelAbi::volume_compute_v1("stimulus.kernel.volume_compute_v1");
    abi.validate().expect("volume ABI should validate");

    assert!(abi.supports_compute);
    assert_eq!(abi.bounded_readback_samples, 512);
    assert!(abi.compute_passes.iter().any(|pass| pass.kind
        == ComputePassKind::VolumeReadbackProbe
        && pass.workgroup_size == ComputeWorkgroupSize::new(64, 1, 1)
        && pass.reads_volume_field
        && pass.bounded_readback_samples == 512));
    assert!(abi.compute_passes.iter().any(|pass| pass.kind
        == ComputePassKind::VolumeRaymarchStereoField
        && pass.workgroup_size == ComputeWorkgroupSize::new(8, 8, 1)
        && pass.output_layers == 2
        && pass.reads_volume_field
        && pass.writes_stereo_field));
}

#[test]
fn volume_profile_cpu_probe_is_deterministic_and_vec4_aligned() {
    let profile =
        StimulusProfile::volume_interference_preview("stimulus.profile.volume_interference");
    profile.validate().expect("volume profile should validate");
    let volume = profile
        .volume
        .as_ref()
        .expect("volume profile should carry descriptor");
    let rays = deterministic_volume_probe_rays(volume, 16, 1.2)
        .expect("probe rays should be deterministic");
    let a = sample_volume_probe_set(&profile, volume, &rays).expect("probe should sample");
    let b = sample_volume_probe_set(&profile, volume, &rays).expect("probe should repeat");

    assert_eq!(a, b);
    assert_eq!(a.len(), 16);
    assert!(a.iter().any(|sample| {
        sample.density_depth_status[2] == StimulusVolumeProbeStatus::Hit.as_f32()
            && sample.rgba[3] > 0.0
    }));
    for output in a {
        assert_eq!(output.rgba.len(), 4);
        assert_eq!(output.density_depth_status.len(), 4);
        assert_eq!(output.density_depth_status[3], volume.step_count as f32);
    }
}

#[test]
fn volume_profile_summary_extracts_compute_shape() {
    let profile =
        StimulusProfile::volume_interference_preview("stimulus.profile.volume_interference");
    let summary = StimulusVolumeProfileSummary::from_profile(&profile)
        .expect("volume summary should extract");

    assert!(summary.volume_present);
    assert_eq!(
        summary.volume_schema.as_deref(),
        Some("rusty.optics.stimulus.volume.v1")
    );
    assert_eq!(
        summary.field_kind.as_deref(),
        Some("ProceduralLayerStack3d")
    );
    assert_eq!(summary.storage_hint.as_deref(), Some("StorageBuffer"));
    assert_eq!(summary.grid_dimensions, Some([32, 32, 32]));
    assert_eq!(summary.step_count, Some(32));
    assert_eq!(summary.volume_readback_probe_samples, Some(512));
    assert_eq!(summary.stereo_field_output_layers, Some(2));
    summary
        .validate_bounded_stereo_preview(512)
        .expect("summary should match bounded mobile preview");
}

#[test]
fn bounded_volume_preview_oracles_are_deterministic() {
    let sample_a = deterministic_bounded_stimulus_volume_probe_sample(3, [32, 32, 32], 32);
    let sample_b = deterministic_bounded_stimulus_volume_probe_sample(3, [32, 32, 32], 32);
    assert_eq!(sample_a, sample_b);
    assert_eq!(
        sample_a.expected_density_depth_status,
        expected_bounded_stimulus_volume_probe_output(sample_a).density_depth_status
    );

    let pixel_a = deterministic_bounded_stimulus_volume_raymarch_preview_pixel(
        BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_PIXELS - 1,
        [32, 32, 32],
        32,
    );
    let pixel_b = deterministic_bounded_stimulus_volume_raymarch_preview_pixel(
        BOUNDED_STIMULUS_VOLUME_RAYMARCH_PREVIEW_PIXELS - 1,
        [32, 32, 32],
        32,
    );
    assert_eq!(pixel_a, pixel_b);
    assert_eq!(
        pixel_a.expected_density_depth_status,
        expected_bounded_stimulus_volume_raymarch_preview_output(pixel_a).density_depth_status
    );

    let image_pixel = deterministic_bounded_stimulus_volume_image_preview_pixel(
        0,
        [32, 32, 32],
        32,
        BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_TILE_WIDTH,
        BOUNDED_STIMULUS_VOLUME_IMAGE_PREVIEW_EYE_TILE_HEIGHT,
    );
    assert_eq!(image_pixel.uv_eye_time[0], 8.5 / 64.0);
    assert_eq!(
        image_pixel.expected_rgba,
        expected_bounded_stimulus_volume_image_preview_output(image_pixel).rgba
    );
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

#[cfg(feature = "serde")]
#[test]
fn volume_interference_fixture_deserializes_and_validates() {
    let json = include_str!("../../../fixtures/stimulus/volume_interference_preview_profile.json");
    let profile: StimulusProfile =
        serde_json::from_str(json).expect("volume stimulus fixture should deserialize");

    profile
        .validate()
        .expect("volume stimulus fixture should validate");

    let volume = profile
        .volume
        .as_ref()
        .expect("volume fixture should carry a volume descriptor");
    assert_eq!(
        volume.field_kind,
        StimulusVolumeFieldKind::ProceduralLayerStack3d
    );
    assert!(profile
        .kernel_abi
        .compute_passes
        .iter()
        .any(|pass| pass.kind == ComputePassKind::VolumeReadbackProbe));
}

#[cfg(feature = "serde")]
#[test]
fn volume_only_bright_fixture_deserializes_and_validates() {
    let json =
        include_str!("../../../fixtures/stimulus/volume_only_bright_interference_profile.json");
    let profile: StimulusProfile =
        serde_json::from_str(json).expect("bright volume stimulus fixture should deserialize");

    profile
        .validate()
        .expect("bright volume stimulus fixture should validate");

    assert!((profile.temporal.target_cycle_hz - 12.0).abs() < f32::EPSILON);
    assert!((profile.safety.max_cycle_hz - 15.0).abs() < f32::EPSILON);
    let layer = profile
        .layer_graph
        .layers
        .first()
        .expect("bright volume fixture should include an interference layer");
    assert_eq!(layer.pattern, BasePatternKind::Interference);
    assert!((8.0..=15.0).contains(&layer.temporal.speed_hz));
    assert!(layer
        .oscillators
        .iter()
        .all(|oscillator| (8.0..=15.0).contains(&oscillator.frequency_hz)));
    assert!(profile.volume.is_some());

    let raw: serde_json::Value =
        serde_json::from_str(json).expect("bright volume fixture should parse as raw json");
    let fragment_hints = &raw["adapter_hints"]["makepad_fragment_volume"];
    assert_eq!(fragment_hints["color_mode"], "DepthRamp");
    assert_eq!(fragment_hints["depth_color_mix"], 1.0);
    assert_eq!(fragment_hints["depth_contrast"], 0.9);
    assert_eq!(fragment_hints["depth_color_near"]["r"], 1.0);
    assert_eq!(fragment_hints["depth_color_mid"]["g"], 1.0);
    assert_eq!(fragment_hints["depth_color_far"]["b"], 1.0);
}

#[test]
fn park_miller_sequence_is_repeatable() {
    let mut a = ParkMillerRng::new(123);
    let mut b = ParkMillerRng::new(123);
    for _ in 0..8 {
        assert_eq!(a.next_unit(), b.next_unit());
    }
}
