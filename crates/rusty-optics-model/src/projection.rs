use crate::{
    OpticsError, Vec2, SOURCE_SAMPLING_MODE_SCHEMA_ID, TARGET_SCREEN_FOOTPRINT_SCHEMA_ID,
    VIDEO_PROJECTION_GEOMETRY_SCHEMA_ID,
};

/// Homography matrix over normalized UV coordinates.
pub type Homography3 = [[f32; 3]; 3];

/// Identity homography.
pub const IDENTITY_HOMOGRAPHY: Homography3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// Two-dimensional rectangle in normalized projection coordinates.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect2 {
    /// Rectangle origin.
    pub origin: Vec2,
    /// Rectangle size.
    pub size: Vec2,
}

impl Rect2 {
    /// Unit rectangle.
    pub const UNIT: Self = Self::new(Vec2::ZERO, Vec2::ONE);

    /// Creates a rectangle.
    #[must_use]
    pub const fn new(origin: Vec2, size: Vec2) -> Self {
        Self { origin, size }
    }

    /// Rectangle max corner.
    #[must_use]
    pub fn max(self) -> Vec2 {
        self.origin + self.size
    }

    /// Rectangle center.
    #[must_use]
    pub fn center(self) -> Vec2 {
        self.origin + (self.size * 0.5)
    }

    /// Aspect ratio, when non-empty.
    #[must_use]
    pub fn aspect(self) -> Option<f32> {
        if self.size.x > 0.0 && self.size.y > 0.0 && self.size.is_finite() {
            Some(self.size.x / self.size.y)
        } else {
            None
        }
    }

    /// Whether coordinates are finite and size is non-negative.
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.origin.is_finite() && self.size.is_finite() && self.size.x >= 0.0 && self.size.y >= 0.0
    }

    /// Whether the rectangle has positive area.
    #[must_use]
    pub fn is_non_empty(self) -> bool {
        self.is_valid() && self.size.x > 0.0 && self.size.y > 0.0
    }

    /// Whether the rectangle is fully inside the unit UV square.
    #[must_use]
    pub fn is_inside_unit(self) -> bool {
        let max = self.max();
        self.is_valid()
            && self.origin.x >= 0.0
            && self.origin.y >= 0.0
            && max.x <= 1.0
            && max.y <= 1.0
    }

    /// Whether a UV point lies inside this rectangle.
    #[must_use]
    pub fn contains_uv(self, uv: Vec2) -> bool {
        let max = self.max();
        uv.x >= self.origin.x && uv.x <= max.x && uv.y >= self.origin.y && uv.y <= max.y
    }

    /// Rectangle intersection.
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        if !self.is_valid() || !other.is_valid() {
            return None;
        }
        let self_max = self.max();
        let other_max = other.max();
        let min_x = self.origin.x.max(other.origin.x);
        let min_y = self.origin.y.max(other.origin.y);
        let max_x = self_max.x.min(other_max.x);
        let max_y = self_max.y.min(other_max.y);
        if max_x < min_x || max_y < min_y {
            return None;
        }
        Some(Self::new(
            Vec2::new(min_x, min_y),
            Vec2::new((max_x - min_x).max(0.0), (max_y - min_y).max(0.0)),
        ))
    }
}

/// Source raster sampling mode understood by projection adapters.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceSamplingMode {
    /// Source raster is placed in the metadata-authored target footprint.
    #[default]
    TargetLocalRaster,
    /// Display screen UV is mapped to source UV through a camera homography.
    ScreenToCameraHomography,
}

impl SourceSamplingMode {
    /// Parses a stable source sampling label.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "target-local-raster"
            | "target-local"
            | "target-raster"
            | "local-raster"
            | "raster"
            | "default" => Some(Self::TargetLocalRaster),
            "screen-to-camera-homography"
            | "screen-camera-homography"
            | "screen-to-source-homography"
            | "camera-homography"
            | "camera-projection"
            | "homography" => Some(Self::ScreenToCameraHomography),
            _ => None,
        }
    }

    /// Stable schema id.
    #[must_use]
    pub const fn schema_id() -> &'static str {
        SOURCE_SAMPLING_MODE_SCHEMA_ID
    }

    /// Stable value id.
    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::TargetLocalRaster => "target-local-raster",
            Self::ScreenToCameraHomography => "screen-to-camera-homography",
        }
    }
}

/// Explicit source-to-surface mapping behavior requested by a visual feed.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VideoProjectionMapping {
    /// Source UV is supplied by a screen-to-source homography.
    #[default]
    ScreenToSourceHomography,
    /// Source UV is supplied by a surface-to-source homography.
    SurfaceToSourceHomography,
    /// The feed fills the chosen surface directly.
    FullFrameSurface,
}

impl VideoProjectionMapping {
    /// Stable value id.
    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::ScreenToSourceHomography => "screen-to-source-homography",
            Self::SurfaceToSourceHomography => "surface-to-source-homography",
            Self::FullFrameSurface => "full-frame-surface",
        }
    }
}

/// Coordinate space used by metadata-authored target footprint requests.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TargetFootprintCoordinateSpace {
    /// Per-eye display screen UV, top-left origin, positive Y down.
    #[default]
    DisplayEyeScreenUv,
    /// Per-eye visible region mapped to X/Y in `[-1, 1]`, positive Y down.
    VisibleEyeNormalizedYDown,
}

impl TargetFootprintCoordinateSpace {
    /// Stable value id.
    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::DisplayEyeScreenUv => "display-eye-screen-uv",
            Self::VisibleEyeNormalizedYDown => "visible-eye-normalized-y-down",
        }
    }
}

/// How a target footprint that leaves the visible eye region is handled.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TargetFootprintClipPolicy {
    /// Draw the visible intersection and report the clipping.
    #[default]
    ClipToVisibleEye,
}

impl TargetFootprintClipPolicy {
    /// Stable value id.
    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::ClipToVisibleEye => "clip-to-visible-eye",
        }
    }
}

/// Explicit target footprint requested by a source or stimulus.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct TargetScreenFootprint {
    /// Schema id.
    pub schema: String,
    /// Coordinate space of the requested rect.
    pub coordinate_space: TargetFootprintCoordinateSpace,
    /// Requested screen UV rectangle.
    pub requested_screen_uv_rect: Rect2,
    /// Visible screen UV rectangle after clipping.
    pub visible_screen_uv_rect: Rect2,
    /// Clip policy.
    pub clip_policy: TargetFootprintClipPolicy,
    /// Whether the requested rect was clipped.
    pub clipped: bool,
}

impl TargetScreenFootprint {
    /// Builds a target footprint from display-eye screen UV.
    #[must_use]
    pub fn from_display_eye_screen_uv_rect(rect: Rect2) -> Option<Self> {
        Self::from_display_eye_screen_uv_rect_with_policy(
            rect,
            TargetFootprintClipPolicy::ClipToVisibleEye,
        )
    }

    /// Builds a target footprint from display-eye screen UV with a clip policy.
    #[must_use]
    pub fn from_display_eye_screen_uv_rect_with_policy(
        rect: Rect2,
        clip_policy: TargetFootprintClipPolicy,
    ) -> Option<Self> {
        if !rect.is_non_empty() {
            return None;
        }
        let visible = rect.intersection(Rect2::UNIT)?;
        let clipped = rect_xywh(visible) != rect_xywh(rect);
        Some(Self {
            schema: TARGET_SCREEN_FOOTPRINT_SCHEMA_ID.to_string(),
            coordinate_space: TargetFootprintCoordinateSpace::DisplayEyeScreenUv,
            requested_screen_uv_rect: rect,
            visible_screen_uv_rect: visible,
            clip_policy,
            clipped,
        })
    }

    /// Builds a footprint from visible-eye normalized center plus height.
    #[must_use]
    pub fn from_visible_eye_normalized_center_height(
        center: Vec2,
        height: f32,
        aspect_ratio: f32,
    ) -> Option<Self> {
        if !center.is_finite()
            || !height.is_finite()
            || height <= 0.0
            || !aspect_ratio.is_finite()
            || aspect_ratio <= 0.0
        {
            return None;
        }
        let center_uv = Vec2::new((center.x + 1.0) * 0.5, (center.y + 1.0) * 0.5);
        let size = Vec2::new(height * aspect_ratio * 0.5, height * 0.5);
        let rect = Rect2::new(center_uv - size * 0.5, size);
        let mut footprint = Self::from_display_eye_screen_uv_rect(rect)?;
        footprint.coordinate_space = TargetFootprintCoordinateSpace::VisibleEyeNormalizedYDown;
        Some(footprint)
    }

    /// Whether the footprint is usable.
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.schema == TARGET_SCREEN_FOOTPRINT_SCHEMA_ID
            && self.requested_screen_uv_rect.is_non_empty()
            && self.visible_screen_uv_rect.is_non_empty()
    }
}

/// Valid source footprint sampled in display-eye screen UV.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SourceValidScreenUvFootprint {
    /// Fraction of sampled screen UV cells that map into the source-valid rect.
    pub active_fraction: f32,
    /// Bounding screen UV rectangle of valid samples.
    pub bbox_screen_uv_rect: Rect2,
    /// Representative row spans.
    pub row_spans: Vec<SourceValidScreenUvRowSpan>,
}

impl SourceValidScreenUvFootprint {
    /// Bounding rectangle as `[x, y, width, height]`.
    #[must_use]
    pub fn bbox_xywh(&self) -> [f32; 4] {
        rect_xywh(self.bbox_screen_uv_rect)
    }
}

/// Valid span on one display-eye screen row.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceValidScreenUvRowSpan {
    /// Row Y coordinate.
    pub row_y: f32,
    /// Fraction of row samples that map into source-valid UV.
    pub active_fraction: f32,
    /// Valid X span, when any sample on the row is valid.
    pub span: Option<(f32, f32)>,
}

/// Per-view renderer-neutral projection geometry report.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectionGeometryReport {
    /// Schema id.
    pub schema: String,
    /// Stable report id.
    pub report_id: String,
    /// View id from the consuming adapter, such as left or right.
    pub view_id: String,
    /// Projection mapping mode.
    pub mapping: VideoProjectionMapping,
    /// Surface-to-screen homography.
    pub surface_to_screen_uv: Homography3,
    /// Screen-to-source homography.
    pub screen_to_source_uv: Homography3,
    /// Surface coverage in screen UV.
    pub surface_coverage_screen_uv_rect: Rect2,
    /// Source-valid UV rectangle.
    pub source_valid_uv_rect: Rect2,
    /// Screen-space source-valid footprint.
    pub source_valid_screen_uv_footprint: SourceValidScreenUvFootprint,
}

impl ProjectionGeometryReport {
    /// Builds a report from homographies.
    pub fn from_homographies(
        report_id: impl Into<String>,
        view_id: impl Into<String>,
        mapping: VideoProjectionMapping,
        surface_to_screen_uv: Homography3,
        screen_to_source_uv: Homography3,
        source_valid_uv_rect: Rect2,
        footprint_grid: usize,
    ) -> Result<Self, OpticsError> {
        if !source_valid_uv_rect.is_non_empty() || !source_valid_uv_rect.is_inside_unit() {
            return Err(OpticsError::InvalidPayload(
                "source_valid_uv_rect is invalid",
            ));
        }
        let surface_coverage_screen_uv_rect =
            homography_unit_square_bounding_rect(surface_to_screen_uv).ok_or(
                OpticsError::InvalidPayload("surface_to_screen_uv is invalid"),
            )?;
        let source_valid_screen_uv_footprint =
            if mapping == VideoProjectionMapping::FullFrameSurface {
                full_source_valid_screen_uv_footprint()
            } else {
                source_valid_screen_uv_footprint(
                    screen_to_source_uv,
                    source_valid_uv_rect,
                    footprint_grid,
                )
            };
        let report = Self {
            schema: VIDEO_PROJECTION_GEOMETRY_SCHEMA_ID.to_string(),
            report_id: report_id.into(),
            view_id: view_id.into(),
            mapping,
            surface_to_screen_uv,
            screen_to_source_uv,
            surface_coverage_screen_uv_rect,
            source_valid_uv_rect,
            source_valid_screen_uv_footprint,
        };
        report.validate()?;
        Ok(report)
    }

    /// Validates the report.
    pub fn validate(&self) -> Result<(), OpticsError> {
        if self.schema != VIDEO_PROJECTION_GEOMETRY_SCHEMA_ID {
            return Err(OpticsError::UnexpectedSchema {
                expected: VIDEO_PROJECTION_GEOMETRY_SCHEMA_ID,
                actual: self.schema.to_string(),
            });
        }
        if self.report_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("report_id"));
        }
        if self.view_id.trim().is_empty() {
            return Err(OpticsError::EmptyId("view_id"));
        }
        if !self.surface_coverage_screen_uv_rect.is_non_empty() {
            return Err(OpticsError::InvalidPayload(
                "surface_coverage_screen_uv_rect is invalid",
            ));
        }
        if !self.source_valid_uv_rect.is_non_empty() || !self.source_valid_uv_rect.is_inside_unit()
        {
            return Err(OpticsError::InvalidPayload(
                "source_valid_uv_rect is invalid",
            ));
        }
        if !self
            .source_valid_screen_uv_footprint
            .active_fraction
            .is_finite()
            || !(0.0..=1.0).contains(&self.source_valid_screen_uv_footprint.active_fraction)
        {
            return Err(OpticsError::InvalidValue("active_fraction"));
        }
        Ok(())
    }
}

/// Applies a homography to a normalized UV coordinate.
#[must_use]
pub fn apply_homography_uv(rows: Homography3, uv: Vec2) -> Option<Vec2> {
    let w = rows[2][0] * uv.x + rows[2][1] * uv.y + rows[2][2];
    if !w.is_finite() || w.abs() <= 1.0e-6 {
        return None;
    }
    let u = (rows[0][0] * uv.x + rows[0][1] * uv.y + rows[0][2]) / w;
    let v = (rows[1][0] * uv.x + rows[1][1] * uv.y + rows[1][2]) / w;
    (u.is_finite() && v.is_finite()).then_some(Vec2::new(u, v))
}

/// Returns the screen-space bounding rect of a homography-projected unit square.
#[must_use]
pub fn homography_unit_square_bounding_rect(rows: Homography3) -> Option<Rect2> {
    let points = [
        apply_homography_uv(rows, Vec2::new(0.0, 0.0))?,
        apply_homography_uv(rows, Vec2::new(1.0, 0.0))?,
        apply_homography_uv(rows, Vec2::new(1.0, 1.0))?,
        apply_homography_uv(rows, Vec2::new(0.0, 1.0))?,
    ];
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    let rect = Rect2::new(
        Vec2::new(min_x, min_y),
        Vec2::new((max_x - min_x).max(0.0), (max_y - min_y).max(0.0)),
    );
    rect.is_non_empty().then_some(rect)
}

/// Samples the part of display-eye screen UV that maps into a source-valid UV rectangle.
#[must_use]
pub fn source_valid_screen_uv_footprint(
    screen_to_source_uv: Homography3,
    source_valid_uv_rect: Rect2,
    grid: usize,
) -> SourceValidScreenUvFootprint {
    let grid = grid.max(2);
    let mut valid_count = 0_usize;
    let mut min_x = 1.0_f32;
    let mut min_y = 1.0_f32;
    let mut max_x = 0.0_f32;
    let mut max_y = 0.0_f32;
    let step = 1.0 / grid as f32;
    for iy in 0..grid {
        for ix in 0..grid {
            let x = (ix as f32 + 0.5) * step;
            let y = (iy as f32 + 0.5) * step;
            if screen_uv_maps_to_source_uv(
                screen_to_source_uv,
                source_valid_uv_rect,
                Vec2::new(x, y),
            ) {
                valid_count += 1;
                min_x = min_x.min((x - step * 0.5).clamp(0.0, 1.0));
                min_y = min_y.min((y - step * 0.5).clamp(0.0, 1.0));
                max_x = max_x.max((x + step * 0.5).clamp(0.0, 1.0));
                max_y = max_y.max((y + step * 0.5).clamp(0.0, 1.0));
            }
        }
    }
    let bbox_screen_uv_rect = if valid_count == 0 {
        Rect2::new(Vec2::ZERO, Vec2::ZERO)
    } else {
        Rect2::new(
            Vec2::new(min_x, min_y),
            Vec2::new((max_x - min_x).max(0.0), (max_y - min_y).max(0.0)),
        )
    };
    let row_spans = [0.0_f32, 0.5, 1.0]
        .into_iter()
        .map(|row_y| {
            source_valid_screen_uv_row_span(screen_to_source_uv, source_valid_uv_rect, row_y, 128)
        })
        .collect();
    SourceValidScreenUvFootprint {
        active_fraction: valid_count as f32 / (grid * grid) as f32,
        bbox_screen_uv_rect,
        row_spans,
    }
}

/// Samples one display-eye screen row for the source-valid span.
#[must_use]
pub fn source_valid_screen_uv_row_span(
    screen_to_source_uv: Homography3,
    source_valid_uv_rect: Rect2,
    row_y: f32,
    sample_count: usize,
) -> SourceValidScreenUvRowSpan {
    let sample_count = sample_count.max(2);
    let mut min_x = 1.0_f32;
    let mut max_x = 0.0_f32;
    let mut valid_count = 0_usize;
    for index in 0..sample_count {
        let x = index as f32 / (sample_count - 1) as f32;
        if screen_uv_maps_to_source_uv(
            screen_to_source_uv,
            source_valid_uv_rect,
            Vec2::new(x, row_y),
        ) {
            valid_count += 1;
            min_x = min_x.min(x);
            max_x = max_x.max(x);
        }
    }
    SourceValidScreenUvRowSpan {
        row_y,
        active_fraction: valid_count as f32 / sample_count as f32,
        span: (valid_count > 0).then_some((min_x, max_x)),
    }
}

/// Rectangle as `[x, y, width, height]`.
#[must_use]
pub fn rect_xywh(rect: Rect2) -> [f32; 4] {
    [rect.origin.x, rect.origin.y, rect.size.x, rect.size.y]
}

/// Rectangle token for marker lines.
#[must_use]
pub fn uv_rect_token(rect: [f32; 4]) -> String {
    format!(
        "{:.6},{:.6},{:.6},{:.6}",
        rect[0], rect[1], rect[2], rect[3]
    )
}

fn full_source_valid_screen_uv_footprint() -> SourceValidScreenUvFootprint {
    SourceValidScreenUvFootprint {
        active_fraction: 1.0,
        bbox_screen_uv_rect: Rect2::UNIT,
        row_spans: vec![
            SourceValidScreenUvRowSpan {
                row_y: 0.0,
                active_fraction: 1.0,
                span: Some((0.0, 1.0)),
            },
            SourceValidScreenUvRowSpan {
                row_y: 0.5,
                active_fraction: 1.0,
                span: Some((0.0, 1.0)),
            },
            SourceValidScreenUvRowSpan {
                row_y: 1.0,
                active_fraction: 1.0,
                span: Some((0.0, 1.0)),
            },
        ],
    }
}

fn screen_uv_maps_to_source_uv(
    screen_to_source_uv: Homography3,
    source_valid_uv_rect: Rect2,
    screen_uv: Vec2,
) -> bool {
    apply_homography_uv(screen_to_source_uv, screen_uv)
        .is_some_and(|source_uv| source_valid_uv_rect.contains_uv(source_uv))
}
