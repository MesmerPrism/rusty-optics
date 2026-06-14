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
- target footprints, source-sampling modes, homography footprints, source-valid
  screen UV summaries, and per-view projection geometry reports;
- billboard instance preparation;
- animated mask atlas descriptors and CPU reference generation;
- trail appearance descriptors and particle render-budget summaries;
- mesh debug frames, coordinate-map visuals, collider visuals, SDF slice
  visuals, and ADF debug visuals over Matter mesh/field payloads;
- surface-field visual frames and playback sequences over Matter field debug
  frames/sequences;
- bioelectric circuit visual frames over Matter circuit state and step
  diagnostics;
- planarian AP bioelectric visual sequences over Matter-owned planarian
  scenario runs, with Optics-owned AP region colors and browser projection;
- renderer-neutral planarian 3D pick selections and edit intents that visual
  adapters can emit before Matter accepts or rejects the requested mutation;
- deterministic fixture and schema artifacts;
- procedural stimulus profiles, layer graphs, layer-local oscillator bindings,
  Perlin-style noise controls, ripple/interference source controls, temporal
  gating profiles, research-use notice policies, full-screen stereo-eye
  presentation targets, bounded stimulus volume descriptors,
  mobile-GPU-portable compute-capable kernel ABI descriptors, display-cadence
  run plans, and CPU reference samples;
- renderer-neutral diagnostics for visual payloads.

Optics does not own:

- mesh, field, SDF, or particle simulation truth;
- reference-space, pose, view-set, or runtime capability state owned by
  Lattice;
- platform hand mesh acquisition;
- command/session/stream authority;
- GPU buffers, shaders, draw calls, texture uploads, or swapchains;
- procedural shader source, GPU field generation, or runtime shader binding;
- OpenXR, Vulkan, WebGL, Android, Quest, Makepad, or UI framework imports;
- downstream app-specific scene behavior, visual-driver bindings, runtime
  tuning, or product profiles.

## Matter / Optics Boundary

Matter may prepare deterministic render-neutral payloads when those payloads
are direct, policy-free projections of Matter state. For particles, Matter owns
IDs, positions, radii, velocities, speed, age, flags, time, and bounds.

Optics adds color, alpha, normal/facing policy, animation frame phase,
billboard draw mode, projection, blend/depth policy, mask animation, trail
appearance, and render-budget summaries. Optics references Matter payload IDs
and schema IDs without duplicating particle simulation truth.

For meshes and fields, Matter owns the triangle mesh surface, hand validation
mesh wrapper, coordinate map, dynamic collider payloads, SDF grid, ADF field,
and accelerated
surface-distance runtime. Optics converts those validated Matter contracts into
bounded debug visuals: mesh wireframes, coordinate anchors and axes, collider
shell/contact markers, sampled SDF slices, and ADF leaf-cell debug payloads.
The browser preview consumes the same renderer-neutral debug frame that a later
renderer adapter can consume; for animated realtime hand-mesh previews it calls
the Matter Wasm surface-distance runtime instead of owning triangle-distance
math in browser code.

The surface-field preview keeps pure browser adapter helpers in
`web/surface-field-preview/surface-field-utils.js`: bounds calculation, color
mapping, scalar formatting, clamping, and small vector utilities. Stateful
playback, canvas drawing, live Matter Wasm calls, Planarian 3D interaction, and
export orchestration stay in `web/surface-field-preview/app.js`.

Historical browser-only brute-force SDF/particle previews are useful only as
prototype evidence for controls, metrics, and smoke-test shape. They must not
be revived as the default implementation path now that Matter owns the Rust
surface-distance and particle runtime exposed to Optics through Wasm.

For realtime surface-field previews, Optics calls the Matter surface-field
Wasm runtime for fixed-step dynamics and snapshots. Browser JavaScript maps
the returned Matter values into Optics-owned colors, arrows, overlays, and
controls; it must not duplicate diffusion, decay, perturbation, or vector
update rules. In the live Planarian 3D view, the `activity dV` layer is also
Matter-exported data: Optics colors the latest per-node voltage-delta rows but
does not derive circuit activity from renderer geometry.

For bioelectric circuit previews, Optics consumes Matter circuit state and
diagnostics as renderer-neutral visual frames. Browser JavaScript may select
voltage, memory, and readout layers and draw conductance/current overlays, but
it must not own voltage, conductance, current, gate, memory, or readout update
rules.

For interactive planarian 3D previews, Optics owns the pick-selection and
edit-intent payload shape: selected visual context, target node or conductance
edge, normalized pointer, expected revision, and proposed operation. Optics also
owns the edit-feedback frame shape used to present Matter-owned recent edit
events and affected targets. Matter remains the authority for edit validation,
clamping, state mutation, acceptance or rejection, revision advancement, and
scenario reset semantics over the GLB-derived body substrate. Browser adapters
may plot Matter-exported outcome traces and comparison trace sets, but they
must not recalculate the scenario metrics or treat plotted values as simulation
truth. Selected-node and selected-edge inspector panels consume Matter readout
accessors; they do not derive voltage, readout, conductance, or gate state from
visual geometry. Recent edit-event inspector feedback consumes bounded
Matter-exported event and affected-target rows, wraps them in an Optics
feedback-frame shape, and only formats operation, target, revision, status,
clamping metadata, renderer color, size, recency fade, and compact timeline
marks. The browser Planarian 3D adapter may style the converted Matter mesh as
a solid visible body, draw Matter-exported node activity as a visual layer,
preview Matter-resolved tiered node-neighborhood voltage-brush targets, and
project graph-node values onto the body vertex-color buffer through a bounded
nearest-node weighting cache. That cache is a renderer aid only: Matter remains
the source for graph state, voltages, memory, conductance, gates, and readouts.
The adapter may default first-tier conductance edges on for readability, but it
must use the Matter-exported triangle surface and GLB surface-anchor rows, keep
body/nodes/edges as separate visibility controls, and refuse a low-count
procedural fallback for this GLB-backed mode. Browser-adapter observability may
expose loading stages and split timing counters for Matter stepping, Optics
view-buffer updates, WebGL render, and panel drawing; those counters are
diagnostics, not simulation state.

## Renderer Adapter Boundary

Optics can prepare backend-neutral instance arrays, mask atlas pixels, and
small CPU reference samples, but renderer adapters decide how to allocate GPU
resources and submit draw or compute calls. Shader code, platform frame
lifecycle, swapchain behavior, runtime profiles, texture upload mechanics,
Vulkan storage images/buffers, barriers, command buffers, and descriptor sets
belong to adapters.

Procedural stimulus compute descriptors intentionally stay Vulkan/WebGPU
neutral while carrying enough limits for a Quest Vulkan adapter to reject an
unsupported profile before launch. The current mobile profile uses 8x8 or
64x1 workgroups, caps workgroups at 64 invocations, caps requested resource
dimensions to 2048 px, requires 16-byte parameter alignment, and does not
require subgroup operations, shader device address, runtime descriptor arrays,
or mandatory fp16 storage. A Quest adapter may prefer fp16 formats when
available but must provide an adapter-level fallback or rejection receipt when
a requested texture format is unsupported.

Stimulus volume descriptors are renderer-neutral proof contracts. Optics can
describe a procedural 3D layer-stack density, dense scalar grid, dense SDF
grid, or indexed ADF grid plus storage hints and bounded step/readback policy.
Matter still owns semantic SDF/ADF/particle-force truth when the volume comes
from simulation or geometry. Quest/Makepad adapters own Vulkan storage buffers,
3D images, descriptors, barriers, queue submission, and headset evidence.

The primary XR presentation target for procedural stimuli is a full-screen
`StereoEyeField`: the adapter generates the requested field texture and submits
it to both eye views, preferably through an XR composition layer when the
runtime supports it. Surface panels and world-locked surfaces remain supported
presentation variants for development previews, calibration targets, and
non-full-field experiments, but they are not the default research-stimulus
route. Optics describes coverage, stereo texture binding, and view-locking;
Lattice owns the live view/reference-space relation, and Quest/Makepad adapters
own OpenXR/Vulkan swapchain, image, barrier, descriptor, and submission
details.

## Lattice / Optics Boundary

Lattice owns runtime relation state: reference spaces, tracked poses, stereo
view sets, validity, confidence, staleness, and capability snapshots. Optics
owns what a renderer does with visual content in those views: target footprints,
source sampling modes, homography-derived screen coverage, source-valid
footprints, and projection geometry reports. A headset or app adapter should
convert runtime view data into Lattice contracts, then feed Optics projection
contracts without importing platform handles into either core repo.

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
  to propose node voltage, tiered node-neighborhood voltage, memory, current,
  and conductance/gate edits without becoming simulation authority;
- planarian 3D edit-feedback frame contracts for renderer adapters to display
  Matter-owned recent edit events and affected node/edge targets;
- live Planarian 3D browser outcome plotting over Matter-exported scenario
traces, comparison trace sets, selected target readouts, recent edit events,
recent affected-target highlights, an edit-event timeline over the Optics
feedback-frame shape, live stats, and the Matter-exported GLB-derived
  triangle body surface plus GLB-anchor node graph;
- coordinate-map, dynamic-collider, SDF-slice, and ADF debug visuals over
  Matter mesh/field payloads;
- procedural stimulus descriptors for clean-room layer stacks, seeded
  deterministic pattern controls, Perlin-style fBm noise, ripple/interference
  fields, layer-local oscillators, temporal gates, externally governed
  research-protocol metadata, full-screen stereo-eye presentation targets,
  bounded volume descriptors and CPU volume probes, mobile-GPU-portable
  compute-pass ABI requirements, display-cadence run plans, and small CPU
  reference samples;
- browser-development stimulus adapter that loads the same profile fixture,
  validates the `StereoEyeField` presentation contract, lowers layer/noise/
  interference descriptors into a WebGPU compute field texture when available,
  exposes bounded CPU/GPU probe summaries for browser validation, and keeps
  browser-only tuning controls in a translator module that rebuilds the
  renderer-neutral profile instead of becoming runtime authority, including a
  post-layer geometry compositor for stack, weighted-mean, and selected-layer
  crossfade exploration, local browser preset storage, and a Quest handoff
  export that stages tuned Optics profiles beside low-rate Makepad settings;
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
- `rusty-optics-mesh/src/adf_debug.rs`: renderer-neutral ADF leaf-cell debug
  visuals over Matter adaptive distance fields.
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
  planarian 3D pick-selection, bioelectric edit-intent, and edit-feedback
  frame contracts.
- `web/surface-field-preview/app.js`: browser preview for visual sequences and
  live Matter surface-field Wasm snapshots, plus static bioelectric circuit
  visual frames, planarian AP bioelectric sequence playback, and live Planarian
  3D scenario selection, outcome-trace comparison plotting, and pick/edit
  request UI for node, Matter-resolved node-neighborhood, and conductance-edge
  targets, plus inspector rendering over Matter-selected readout accessors plus
  Optics-shaped feedback frames over Matter recent edit-event and
  affected-target rows; owns playback, drawing, converted-GLB mesh styling,
  body/node/edge visibility toggles, feedback-frame timeline marks, and
  edit-intent construction only.
- `web/stimulus-preview/app.js`: browser-development adapter for procedural
  stimulus profiles. It owns WebGPU/CPU browser lowering, canvas resize,
  full-screen preview controls, and bounded probe readouts; it is not core
  shader authority and not the Quest runtime adapter.
- `web/stimulus-preview/tuning.js`: browser-development tuning module for
  compact hash presets, randomize/reset behavior, live control edits,
  eccentricity controls, per-layer oscillator banks, and smooth flat-strobe to
  geometry-layer transition state plus global geometry-stack blend controls. It
  translates external preset vocabulary into an Optics `StimulusProfile` shape
  and stays out of core crates, Quest adapters, and study-specific defaults.
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
- `rusty-optics-stimulus/src/layers.rs`: procedural layer graphs, base-pattern
  families, color ramps, coordinate mirroring, warp controls,
  ripple/interference source controls, oscillator bindings, and
  post-processing policy descriptors.
- `rusty-optics-stimulus/src/noise.rs`: deterministic cell, smooth value, and
  Perlin-style gradient fBm noise descriptors plus CPU oracle sampling.
- `rusty-optics-stimulus/src/oscillator.rs`: renderer-neutral oscillator
  waveforms and layer-parameter targets used by CPU references and later
  shader adapters.
- `rusty-optics-stimulus/src/presentation.rs`: full-screen stereo-eye,
  mono-eye, surface-panel, and world-locked presentation descriptors.
- `rusty-optics-stimulus/src/temporal.rs`: temporal pulse/gate profiles and
  lead-in sampling.
- `rusty-optics-stimulus/src/safety.rs`: preview/risk/research-protocol
  notice policies and temporal cross-checks.
- `rusty-optics-stimulus/src/kernel_abi.rs`: renderer-neutral procedural
  stimulus kernel capability descriptors, including compute pass, field
  texture, history, noise-cache, bounded-readback, volume-readback,
  volume-stereo-field, and mobile GPU portability metadata.
- `rusty-optics-stimulus/src/volume.rs`: bounded renderer-neutral stimulus
  volume descriptors, storage hints, grid bounds, step policy, and validation.
- `rusty-optics-stimulus/src/volume_probe.rs`: vec4-aligned deterministic
  volume probe ray and readback output records.
- `rusty-optics-stimulus/src/volume_profile.rs`: compact adapter-facing
  volume profile summaries and bounded stereo preview validation.
- `rusty-optics-stimulus/src/volume_preview.rs`: stable bounded volume probe,
  raymarch-preview, and scalable-image CPU oracles used by browser and
  Vulkan/Quest adapters.
- `rusty-optics-stimulus/src/volume_cpu_reference.rs`: CPU reference sampler
  for bounded volume probe rays over the procedural layer stack.
- `rusty-optics-stimulus/src/run_plan.rs`: display-refresh frame quantization
  for stimulus timing.
- `rusty-optics-stimulus/src/profile.rs`: complete procedural stimulus profile
  validation and run-plan entrypoint.
- `rusty-optics-stimulus/src/cpu_reference.rs`: small deterministic CPU
  sampler for fixtures and scorecards.
- `rusty-optics-fixtures/src/main.rs`: dispatch-only fixture CLI.
- `rusty-optics-fixtures/src/stimulus.rs`: generated procedural stimulus
  volume preview profile fixture.
- `rusty-optics-fixtures/src/hand_mesh.rs`: deterministic hand-validation mesh
  debug fixture using Matter mesh, coordinate, collider, and SDF APIs.
- `web/hand-mesh-browser-preview/realtime-sdf.js`: browser preview adapter for
  animated Matter mesh sequences; it loads the Matter Wasm distance runtime and
  keeps only visual/playback glue in Optics.
- `rusty-optics-schema/src/main.rs`: dispatch-only schema catalog CLI.
