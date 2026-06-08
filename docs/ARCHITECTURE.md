# Architecture

Rusty Optics is the source of truth for renderer-neutral visual contracts and
CPU reference preparation that sit between Matter payloads and renderer
adapters.

## Ownership

Optics owns:

- visual particle frames;
- visual animation profiles that resolve renderer-neutral color, size,
  transparency, spin, and frame phase from Matter particle snapshots;
- appearance profiles and material policy descriptors;
- view/projection contracts;
- billboard instance preparation;
- animated mask atlas descriptors and CPU reference generation;
- trail appearance descriptors and particle render-budget summaries;
- mesh debug frames, coordinate-map visuals, collider visuals, and SDF slice
  visuals over Matter mesh payloads;
- surface-field visual frames over Matter field debug frames;
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

For meshes, Matter owns the triangle mesh surface, hand validation mesh wrapper,
coordinate map, dynamic collider payloads, SDF grid, and accelerated
surface-distance runtime. Optics converts those validated Matter contracts into
bounded debug visuals: mesh wireframes, coordinate anchors and axes, collider
shell/contact markers, and sampled SDF slices. The browser preview consumes the
same renderer-neutral debug frame that a later renderer adapter can consume; for
animated realtime hand-mesh previews it calls the Matter Wasm surface-distance
runtime instead of owning triangle-distance math in browser code.

## Renderer Adapter Boundary

Optics can prepare backend-neutral instance arrays and mask atlas pixels, but
renderer adapters decide how to allocate GPU resources and submit draw calls.
Shader code, platform frame lifecycle, swapchain behavior, runtime profiles,
and texture upload mechanics belong to adapters.

## Current Slices

The implemented foundation slice is CPU/data-only:

- color and schema ID primitives;
- particle visual frames over Matter particle payloads;
- renderer-neutral particle animation profiles and CPU reference resolution;
- appearance profiles for billboard, blend/depth, animated mask, facing, frame
  scaling, and trail policy;
- billboard instance packing and transparent-particle budget summaries;
- flat-screen projection and far-to-near sorting;
- morphed-ring mask atlas CPU reference generation;
- mesh debug frames over Matter triangle surfaces;
- surface-field visual frames over Matter surface-field debug frames;
- coordinate-map, dynamic-collider, and SDF-slice debug visuals over one shared
  source mesh surface;
- browser preview for generated mesh debug JSON and Matter-Wasm-backed animated
  hand-mesh SDF/particle smoke;
- fixture and schema catalog checks;
- dependency and namespace boundary scans.

## Module Map

Crate roots stay as facades so Optics does not rebuild monolithic `main.rs` and
`lib.rs` shapes.

- `rusty-optics-model/src/color.rs`: RGBA colors.
- `rusty-optics-model/src/error.rs`: shared validation errors.
- `rusty-optics-model/src/ids.rs`: Optics schema IDs.
- `rusty-optics-model/src/vec2.rs`: two-dimensional projection points.
- `rusty-optics-mesh/src/browser_frame.rs`: combined mesh debug frame for
  static browser previews and future renderer adapters.
- `rusty-optics-mesh/src/collider.rs`: dynamic mesh collider shell/contact
  debug visuals.
- `rusty-optics-mesh/src/coordinate.rs`: coordinate-map anchor and axis debug
  visuals.
- `rusty-optics-mesh/src/field_frame.rs`: renderer-neutral surface-field node,
  edge, scalar-color, perturbation-region, and polarity-arrow visuals over
  Matter field debug frames.
- `rusty-optics-mesh/src/mesh_frame.rs`: mesh wireframe and topology debug
  visuals.
- `rusty-optics-mesh/src/sdf_slice.rs`: two-dimensional SDF slice debug
  visuals.
- `rusty-optics-particles/src/appearance.rs`: particle appearance descriptors.
- `rusty-optics-particles/src/animation.rs`: renderer-neutral particle
  animation profiles and CPU reference resolution into visual frames.
- `rusty-optics-particles/src/billboard.rs`: billboard instance preparation and
  render-budget summaries.
- `rusty-optics-particles/src/mask.rs`: animated morphed-ring mask atlas
  generation.
- `rusty-optics-particles/src/projection.rs`: flat projection and sorted
  screen-space frames.
- `rusty-optics-particles/src/visual_frame.rs`: visual particle samples and
  frames over Matter particle payloads.
- `rusty-optics-fixtures/src/main.rs`: dispatch-only fixture CLI.
- `rusty-optics-fixtures/src/hand_mesh.rs`: deterministic hand-validation mesh
  debug fixture using Matter mesh, coordinate, collider, and SDF APIs.
- `web/hand-mesh-browser-preview/realtime-sdf.js`: browser preview adapter for
  animated Matter mesh sequences; it loads the Matter Wasm distance runtime and
  keeps only visual/playback glue in Optics.
- `rusty-optics-schema/src/main.rs`: dispatch-only schema catalog CLI.
