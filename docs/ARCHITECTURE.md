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
- surface-field visual frames and playback sequences over Matter field debug
  frames/sequences;
- bioelectric circuit visual frames over Matter circuit state and step
  diagnostics;
- planarian AP bioelectric visual sequences over Matter-owned planarian
  scenario runs, with Optics-owned AP region colors and browser projection;
- renderer-neutral planarian 3D pick selections and edit intents that visual
  adapters can emit before Matter accepts or rejects the requested mutation;
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

For realtime surface-field previews, Optics calls the Matter surface-field
Wasm runtime for fixed-step dynamics and snapshots. Browser JavaScript maps
the returned Matter values into Optics-owned colors, arrows, overlays, and
controls; it must not duplicate diffusion, decay, perturbation, or vector
update rules.

For bioelectric circuit previews, Optics consumes Matter circuit state and
diagnostics as renderer-neutral visual frames. Browser JavaScript may select
voltage, memory, and readout layers and draw conductance/current overlays, but
it must not own voltage, conductance, current, gate, memory, or readout update
rules.

For interactive planarian 3D previews, Optics owns the pick-selection and
edit-intent payload shape: selected visual context, target node or conductance
edge, normalized pointer, expected revision, and proposed operation. Matter
remains the authority for edit validation, clamping, state mutation, acceptance
or rejection, revision advancement, and scenario reset semantics over the
GLB-derived body substrate. Browser adapters may plot Matter-exported outcome
traces and comparison trace sets, but they must not recalculate the scenario
metrics or treat plotted values as simulation truth. Selected-node and
selected-edge inspector panels consume Matter readout accessors; they do not
derive voltage, readout, conductance, or gate state from visual geometry.
Recent edit-event inspector feedback consumes a bounded Matter-exported event
history and only formats operation, target, revision, status, and clamping
metadata.

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
- surface-field visual frames and playback sequences over Matter surface-field
  debug frames/sequences;
- bioelectric circuit visual frames over Matter voltage, conductance, current,
  memory, readout, and step-diagnostic payloads;
- planarian AP bioelectric visual sequences over Matter planarian scenario
  runs, including AP region labels, node-region colors, voltage/memory/readout
  playback frames, and circuit overlays;
- planarian 3D pick-selection and edit-intent contracts for renderer adapters
  to propose node voltage, memory, current, and conductance/gate edits without
  becoming simulation authority;
- live Planarian 3D browser outcome plotting over Matter-exported scenario
  traces, comparison trace sets, selected target readouts, recent edit events,
  and live stats;
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
- `rusty-optics-mesh/src/circuit_frame.rs`: renderer-neutral bioelectric
  circuit visual frame contracts over Matter circuit state and diagnostics.
- `rusty-optics-mesh/src/collider.rs`: dynamic mesh collider shell/contact
  debug visuals.
- `rusty-optics-mesh/src/coordinate.rs`: coordinate-map anchor and axis debug
  visuals.
- `rusty-optics-mesh/src/field_frame.rs`: renderer-neutral surface-field node,
  edge, scalar-color, perturbation-region, polarity-arrow, and visual sequence
  contracts over Matter field debug frames/sequences.
- `rusty-optics-mesh/src/planarian_frame.rs`: renderer-neutral planarian AP
  bioelectric visual sequences over Matter planarian scenario runs.
- `rusty-optics-mesh/src/planarian_interaction.rs`: renderer-neutral
  planarian 3D pick-selection and bioelectric edit-intent contracts.
- `web/surface-field-preview/app.js`: browser preview for visual sequences and
  live Matter surface-field Wasm snapshots, plus static bioelectric circuit
  visual frames, planarian AP bioelectric sequence playback, and live Planarian
  3D scenario selection, outcome-trace comparison plotting, and pick/edit
  request UI for node and conductance-edge targets, plus inspector rendering
  over Matter-selected readout accessors and recent edit-event history; owns
  playback, drawing, and edit-intent construction only.
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
