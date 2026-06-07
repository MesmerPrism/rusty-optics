use crate::{optics_schema_ids, ColorRgba, Vec2, COLOR_RGBA_SCHEMA_ID};

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
