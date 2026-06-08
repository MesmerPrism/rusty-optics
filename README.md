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
- coordinate-map, dynamic-collider, and SDF-slice debug visuals over that same
  source surface;
- a browser preview that renders generated mesh debug JSON and, for animated
  hand-mesh sequences, drives realtime SDF/particle queries through the Matter
  WebAssembly runtime without renderer backend imports.

Optics does not own particle simulation, mesh/SDF truth, private visual-driver
bindings, shader source, GPU uploads, OpenXR/Vulkan/WebGL/Makepad integrations,
or downstream product profiles.

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
only formats those values for feedback. The same inspector displays a bounded
Matter-exported recent edit-event trail for accepted or rejected node and edge
mutations, and the 3D view highlights recently affected nodes or conductance
edges using Matter-exported affected-target rows. The browser wraps those
Matter reads in an Optics edit-feedback frame shape before drawing them. The
browser preview refuses a low-count procedural body in this mode and renders
the converted Matter triangle mesh as the visible source body, with the sampled
node/edge graph overlaid as simulation state.

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
