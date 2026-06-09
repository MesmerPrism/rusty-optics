use crate::{
    homography_unit_square_bounding_rect, optics_schema_ids, source_valid_screen_uv_footprint,
    ColorRgba, ProjectionGeometryReport, Rect2, SourceSamplingMode, TargetScreenFootprint, Vec2,
    VideoProjectionMapping, COLOR_RGBA_SCHEMA_ID, IDENTITY_HOMOGRAPHY,
    VIDEO_PROJECTION_GEOMETRY_SCHEMA_ID,
};

#[test]
fn color_clamps_non_finite_channels() {
    let color = ColorRgba::new(f32::NAN, 2.0, -1.0, 0.5).clamped01();

    assert_eq!(color, ColorRgba::new(0.0, 1.0, 0.0, 0.5));
}

#[test]
fn vec2_reports_finite_state() {
    assert!(Vec2::new(1.0, 2.0).is_finite());
    assert!(!Vec2::new(f32::INFINITY, 2.0).is_finite());
}

#[test]
fn schema_ids_use_optics_namespace() {
    assert_eq!(COLOR_RGBA_SCHEMA_ID, "rusty.optics.color.rgba.v1");
    assert!(optics_schema_ids()
        .iter()
        .all(|schema_id| schema_id.starts_with("rusty.optics.")));
}

#[test]
fn source_sampling_mode_parses_projection_aliases() {
    assert_eq!(
        SourceSamplingMode::parse("camera-projection"),
        Some(SourceSamplingMode::ScreenToCameraHomography)
    );
    assert_eq!(
        SourceSamplingMode::TargetLocalRaster.stable_id(),
        "target-local-raster"
    );
}

#[test]
fn target_screen_footprint_clips_to_visible_eye() {
    let footprint = TargetScreenFootprint::from_display_eye_screen_uv_rect(Rect2::new(
        Vec2::new(0.75, 0.25),
        Vec2::new(0.5, 0.5),
    ))
    .expect("valid clipped footprint");

    assert!(footprint.clipped);
    assert_eq!(
        footprint.visible_screen_uv_rect,
        Rect2::new(Vec2::new(0.75, 0.25), Vec2::new(0.25, 0.5))
    );
    assert!(footprint.is_valid());
}

#[test]
fn source_valid_footprint_samples_homography_domain() {
    let footprint = source_valid_screen_uv_footprint(
        IDENTITY_HOMOGRAPHY,
        Rect2::new(Vec2::new(0.25, 0.25), Vec2::new(0.5, 0.5)),
        8,
    );

    assert!(footprint.active_fraction > 0.2);
    assert!(footprint.active_fraction < 0.4);
    assert_eq!(
        footprint.bbox_screen_uv_rect,
        Rect2::new(Vec2::new(0.25, 0.25), Vec2::new(0.5, 0.5))
    );
}

#[test]
fn projection_geometry_report_uses_optics_schema() {
    let report = ProjectionGeometryReport::from_homographies(
        "projection.synthetic.left",
        "left",
        VideoProjectionMapping::ScreenToSourceHomography,
        IDENTITY_HOMOGRAPHY,
        IDENTITY_HOMOGRAPHY,
        Rect2::UNIT,
        8,
    )
    .expect("projection report");

    assert_eq!(report.schema, VIDEO_PROJECTION_GEOMETRY_SCHEMA_ID);
    assert_eq!(
        homography_unit_square_bounding_rect(report.surface_to_screen_uv),
        Some(Rect2::UNIT)
    );
    assert_eq!(report.source_valid_screen_uv_footprint.active_fraction, 1.0);
}
