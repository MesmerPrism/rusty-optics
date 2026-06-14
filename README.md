# Rusty Optics

Rusty Optics is the renderer-neutral visual contract layer for the Rusty stack.
It consumes Matter payloads and describes how those payloads should be viewed,
projected, animated, inspected, and handed to renderer adapters.

The first source slices focus on visual particles and mesh diagnostics:

- visual particle frames derived from Matter particle payloads;
- appearance profiles for billboard draw mode, blend/depth policy, animated
  mask descriptors, facing response, frame scaling, and trail appearance;
- visual animation profiles that resolve color ramps, size envelopes,
  transparency envelopes, spin, and half-open animation phase from Matter
  particle snapshots;
- CPU billboard instance preparation for renderer adapters;
- flat-screen projection for desktop, browser, phone, or test previews;
- renderer-neutral target footprints, source-sampling modes, homography
  footprints, and video projection geometry reports for camera/projection
  adapters;
- animated morphed-ring mask atlas generation;
- transparent particle render-budget summaries;
- mesh debug frames derived from Matter triangle mesh surfaces;
- surface-field visual frames and playback sequences derived from Matter field
  debug frames/sequences;
- bioelectric circuit visual frames derived from Matter circuit state and step
  diagnostics;
- planarian AP bioelectric visual sequences derived from Matter-owned
  planarian scenario runs;
- planarian 3D pick selections and edit intents for renderer adapters that
  request Matter-owned node, conductance, and gate mutations;
- planarian 3D edit-feedback frames for renderer adapters that display
  Matter-owned recent edit events and affected targets;
- coordinate-map, dynamic-collider, SDF-slice, and ADF debug visuals over
  Matter mesh/field payloads;
- procedural stimulus profiles, layer graphs, oscillator bindings,
  Perlin-style noise controls, ripple/interference controls, temporal gates,
  research-use notice policies, full-screen stereo-eye presentation targets,
  bounded stimulus volume descriptors, mobile-GPU-portable compute-capable
  kernel ABI descriptors, run-plan quantization, and CPU reference samples for
  browser-development and renderer-adapter handoff;
- a browser preview that renders generated mesh debug JSON and, for animated
  hand-mesh sequences, drives realtime SDF/particle queries through the Matter
  WebAssembly runtime without renderer backend imports;
- a browser-development stimulus preview that loads the same renderer-neutral
  `StimulusProfile` fixture and lowers it to a full-screen WebGPU compute
  field texture, with a bounded CPU canvas fallback when WebGPU is unavailable.

Optics does not own particle simulation, mesh/SDF truth, tracked-space runtime
state, downstream visual-driver bindings, shader source, GPU uploads,
OpenXR/Vulkan/WebGL/Makepad integrations, procedural shader source, GPU field
passes, or downstream product profiles.

## Hand Mesh Browser Preview

Generate the deterministic hand-mesh browser fixture and start a local static
server:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Start-HandMeshBrowserPreview.ps1
```

Then open:

```text
http://127.0.0.1:8791/web/hand-mesh-browser-preview/
```

The preview consumes `fixtures/hand_mesh/hand_mesh_browser_debug_frame.json`.
That JSON is renderer-neutral and can also feed a later renderer adapter.

## Stimulus Browser Preview

Start the full-screen procedural stimulus browser adapter:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Start-StimulusPreview.ps1
```

Then open:

```text
http://127.0.0.1:8793/web/stimulus-preview/
```

The preview consumes `fixtures/stimulus/interference_preview_profile.json`.
When WebGPU is available it generates the stimulus in a compute pass before
drawing the field to a full-viewport canvas. The CPU fallback exists for
browser capability gaps and contract validation only.
The first volume-field proof fixture is
`fixtures/stimulus/volume_interference_preview_profile.json`. It adds a
renderer-neutral bounded volume descriptor and volume compute-pass ABI while
remaining a CPU/WebGPU/Quest-adapter proof target rather than Matter field or
particle-force authority.
When that profile is selected, the browser probe status includes bounded CPU
and WebGPU volume-probe summaries over the declared volume readback sample
count. The WebGPU path is a storage-buffer readback proof, not a full volume
image renderer; the visual canvas still uses the existing 2D field preview
until the separate WebGPU/Vulkan volume raymarch passes land.
The browser adapter also exposes a compact tuning panel for development:
colors, oscillator gating, geometry transforms, eccentricity, noise/vignette
controls, layer strength/frequency/speed, per-layer oscillator banks,
flat-color-to-geometry transitions, geometry-stack blend modes, randomize/reset,
and copyable hash links. It can translate compatible strobe-style preset hashes
into local tuning state, then rebuild the normalized `StimulusProfile` used by
the WebGPU and CPU preview paths.
Use `Save` to keep browser-local presets during development. Use
`Export Quest` to download a `rusty.optics.stimulus.quest_handoff.v1` JSON
that contains the tuned renderer-neutral profile, browser tuning sidecar, and
Quest Makepad effective-settings report. Downstream Quest tooling expands that
single browser export into an app-private settings bundle with the Optics
profile staged beside settings rather than embedded inside settings JSON.

## Surface Field Browser Preview

Generate the deterministic surface-field visual fixture, copy the Matter
surface-field Wasm runtime, and start a local static server:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Start-SurfaceFieldPreview.ps1 -BuildMatterWasm
```

Then open:

```text
http://127.0.0.1:8792/web/surface-field-preview/
```

The preview consumes `fixtures/fields/surface_field_visual_sequence.json` by
default as a fallback. When `local-artifacts\matter_surface_field_wasm` is
present, it runs the Matter-owned realtime surface-field Wasm runtime and uses
the sequence only as fallback/evidence. Optics owns playback, colors, edge
styling, perturbation highlights, and polarity arrows only.
Implementation is split between `web/surface-field-preview/app.js` for
stateful playback, drawing, live Matter Wasm calls, and Planarian 3D
interaction; `web/surface-field-preview/planarian-3d-readouts.js` for
Planarian scenario/source-target/edit vocabulary and pure readout formatting;
`web/surface-field-preview/planarian-3d-export.js` for Planarian 3D export
capture, frame shaping, encoder dispatch, downloads, and metadata; and
`web/surface-field-preview/surface-field-utils.js` for pure bounds, color,
formatting, clamp, and small vector helpers.

The same preview also has a `Circuit` view over
`fixtures/fields/bioelectric_circuit_visual_frame.json`. That view shows the
Matter-owned circuit snapshot as voltage colors, conductance edges, current
regions, memory values, readout layers, and compact step diagnostics. It does
not compute circuit dynamics in browser JavaScript.

The `Planarian` view consumes
`fixtures/fields/planarian_bioelectric_visual_sequence.json`. It plays a
Matter-owned synthetic planarian anterior/posterior bioelectric scenario with
AP region bands, voltage, memory, readout layers, conductance edges, and
current-region highlights. Browser JavaScript only projects and draws the
validated Optics sequence.

The `Planarian 3D` view imports Matter Wasm for the live GLB-derived body mesh
and circuit state. Browser raycasts become Optics pick-selection payloads, UI
node and conductance-edge edits become Optics edit-intent payloads, and Matter
remains the authority that accepts or rejects those edits and advances
revisions. The scenario selector only asks Matter to reset to one of its
deterministic planarian presets; browser JavaScript does not rebuild the body
graph, compute circuit dynamics, or decide gate behavior. The outcome plot
overlays Matter-exported current and comparison scenario traces with the
live-step marker without becoming the metric authority. The selection
inspector reads Matter-exported node and conductance-edge state accessors and
only formats those values for feedback. The default `activity dV` layer uses
Matter-exported per-node voltage-delta rows so realtime dynamics are visible
without moving circuit math into Optics. The 3D adapter also precomputes a
small nearest-node weighting cache from GLB body vertices to Matter sample
nodes so the body surface can be smoothly colored from those same graph values
at render time. The Brush dV control asks Matter for the selected node's
tiered voltage-neighborhood preview and then requests the matching
Matter-owned neighborhood mutation; browser JavaScript does not choose affected
nodes. While the Matter/Three.js path is importing, the browser keeps
the selected Planarian 3D viewport active and reports loading stages rather
than falling back to the 2D surface sequence. Once live, the stats/readout split
Matter step time from Optics view-buffer update, WebGL render, and panel draw
time. The same inspector displays a bounded
Matter-exported recent edit-event trail for accepted or rejected node and edge
mutations, draws a compact event timeline from the same feedback frame, and
the 3D view highlights recently affected nodes or conductance edges using
Matter-exported affected-target rows. The browser wraps those Matter reads in
an Optics edit-feedback frame shape before drawing them. The browser preview
refuses a low-count procedural body in this mode and renders the converted
Matter triangle mesh as the visible source body, with the sampled node/edge
graph overlaid as simulation state. Body, Nodes, and Edges are separate view
controls in Planarian 3D mode; first-tier Edges default on so coupling and
realtime activity are legible while tier-2 edges remain off by default.

Planarian 3D export semantics are documented in
`docs/PLANARIAN_3D_EXPORT.md`. The default showcase export is an Optics-owned
visual loop over Matter-stepped synthetic educational dynamics, with stable
portrait framing, opaque neon RGB rendering, exact output dimensions, and
GIF/APNG encoding policy kept out of Matter.

For animated hand-mesh sequence previews, build the Matter Wasm runtime into
Optics local artifacts before launching:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Start-HandMeshBrowserPreview.ps1 `
  -BuildMatterWasm `
  -FramePath "local-artifacts\hand_mesh\hand_mesh_realtime_sequence.json"
```

## Validation

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\check_all.ps1
```
