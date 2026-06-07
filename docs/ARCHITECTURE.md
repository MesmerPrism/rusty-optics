# Architecture

Rusty Optics is the source of truth for renderer-neutral visual contracts and
CPU reference preparation that sit between Matter payloads and renderer
adapters.

## Ownership

Optics owns:

- visual particle frames;
- appearance profiles and material policy descriptors;
- view/projection contracts;
- billboard instance preparation;
- animated mask atlas descriptors and CPU reference generation;
- trail appearance descriptors and particle render-budget summaries;
- deterministic fixture and schema artifacts;
- renderer-neutral diagnostics for visual payloads.

Optics does not own:

- mesh, field, SDF, or particle simulation truth;
- platform hand mesh acquisition;
- command/session/stream authority;
- GPU buffers, shaders, draw calls, texture uploads, or swapchains;
- OpenXR, Vulkan, WebGL, Android, Quest, Makepad, or UI framework imports;
- downstream private scene behavior, visual-driver bindings, runtime tuning, or
  product profiles.

## Matter / Optics Boundary

Matter may prepare deterministic render-neutral payloads when those payloads
are direct, policy-free projections of Matter state. For particles, Matter owns
IDs, positions, radii, velocities, speed, age, flags, time, and bounds.

Optics adds color, alpha, normal/facing policy, animation frame phase,
billboard draw mode, projection, blend/depth policy, mask animation, trail
appearance, and render-budget summaries. Optics references Matter payload IDs
and schema IDs without duplicating particle simulation truth.

## Renderer Adapter Boundary

Optics can prepare backend-neutral instance arrays and mask atlas pixels, but
renderer adapters decide how to allocate GPU resources and submit draw calls.
Shader code, platform frame lifecycle, swapchain behavior, runtime profiles,
and texture upload mechanics belong to adapters.

## Current Slices

The implemented foundation slice is CPU/data-only:

- color and schema ID primitives;
- particle visual frames over Matter particle payloads;
- appearance profiles for billboard, blend/depth, animated mask, facing, frame
  scaling, and trail policy;
- billboard instance packing and transparent-particle budget summaries;
- flat-screen projection and far-to-near sorting;
- morphed-ring mask atlas CPU reference generation;
- fixture and schema catalog checks;
- dependency and namespace boundary scans.

## Module Map

Crate roots stay as facades so Optics does not rebuild monolithic `main.rs` and
`lib.rs` shapes.

- `rusty-optics-model/src/color.rs`: RGBA colors.
- `rusty-optics-model/src/error.rs`: shared validation errors.
- `rusty-optics-model/src/ids.rs`: Optics schema IDs.
- `rusty-optics-model/src/vec2.rs`: two-dimensional projection points.
- `rusty-optics-particles/src/appearance.rs`: particle appearance descriptors.
- `rusty-optics-particles/src/billboard.rs`: billboard instance preparation and
  render-budget summaries.
- `rusty-optics-particles/src/mask.rs`: animated morphed-ring mask atlas
  generation.
- `rusty-optics-particles/src/projection.rs`: flat projection and sorted
  screen-space frames.
- `rusty-optics-particles/src/visual_frame.rs`: visual particle samples and
  frames over Matter particle payloads.
- `rusty-optics-fixtures/src/main.rs`: dispatch-only fixture CLI.
- `rusty-optics-schema/src/main.rs`: dispatch-only schema catalog CLI.

