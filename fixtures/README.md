# Fixtures

Optics fixtures are deterministic, low-volume artifacts for validating visual
payload shape, projection behavior, billboard budgets, and schema wiring. They
are not GPU captures and do not include downstream app-specific visual mappings.

Regenerate and check fixtures with:

```powershell
cargo run -p rusty-optics-fixtures -- export --check
```

## Hand Mesh Browser Debug Frame

`fixtures/hand_mesh/hand_mesh_browser_debug_frame.json` is a renderer-neutral
debug payload built from one synthetic Matter hand validation mesh frame. The
same underlying `TriangleMeshSurface` feeds the mesh wireframe, coordinate map,
dynamic collider, and SDF grid before Optics converts them into browser-ready
debug visuals.

Regenerate and check the hand-mesh fixture with:

```powershell
cargo run -p rusty-optics-fixtures -- export-hand-mesh-browser --check
```

External Matter surface JSON can be converted into a local browser debug frame
with:

```powershell
cargo run -p rusty-optics-fixtures -- export-hand-mesh-browser-from-surface `
  --surface-json "<surface.json>" `
  --include-sdf-particles `
  --output "local-artifacts\hand_mesh\hand_mesh_browser_debug_frame.json"
```

This path can include a Matter-owned SDF particle simulation overlay for
hardware-free smoke evidence and is intentionally not a committed fixture. When
that overlay is present, the local browser preview also receives the packed SDF
grid so its `Live` and `Reset Particles` controls can seed a sphere distribution
and show particles reacting to the SDF gradient in the browser.

For realtime movement smoke tests, the browser preview can also load an
external Matter animated surface sequence. That sequence must contain only
skinned mesh positions and shared topology. The preview recomputes its SDF
slice, collider shell, and 1000-particle response from the current mesh frame,
which keeps animated hand checks separate from precomputed SDF/collider caches.
Surface-field preview fixtures:

- `fixtures/fields/surface_field_visual_frame.json`: renderer-neutral Optics
  visual frame over a Matter-owned surface-field debug frame. It contains
  colored scalar node samples, tiered neighbor edges, perturbation highlights,
  and polarity arrows.
- `fixtures/fields/surface_field_visual_sequence.json`: renderer-neutral
  Optics visual sequence over a Matter-owned dynamic debug sequence. It
  contains 41 emitted frames over 120 Matter fixed steps for browser playback.
- `fixtures/fields/bioelectric_circuit_visual_frame.json`: renderer-neutral
  Optics visual frame over a Matter-owned bioelectric circuit snapshot and step
  diagnostics. It contains voltage samples, conductance edges, current regions,
  memory samples, and readout layers for browser inspection.
- `fixtures/fields/planarian_bioelectric_visual_sequence.json`:
  renderer-neutral Optics visual sequence over a Matter-owned synthetic
  planarian AP bioelectric scenario. It contains AP region bands, node-region
  colors, Matter surface anchors for sampled nodes, voltage/memory/readout
  playback frames, conductance edges, current regions, and diagnostics for
  browser inspection.
- `fixtures/fields/planarian_bioelectric_interaction_intent.json`:
  renderer-neutral Optics interaction fixture with Planarian 3D node and
  conductance-edge pick selections. Node picks preserve the Matter body
  triangle plus barycentric anchor before proposing node-voltage and edge-gate
  edit intents and an edit-feedback frame over recent edit events and affected
  targets. It validates visual request/feedback shape and target binding only;
  Matter still accepts, rejects, clamps, mutates, and advances revisions.

The planarian fixture stays on the compact synthetic AP surface by design. Live
Planarian 3D browser sessions import Matter Wasm and request Matter's
reviewed GLB-derived body surface at runtime, preserving Matter as the body
geometry and simulation authority while keeping committed Optics fixtures small.
The live outcome plot also uses Matter Wasm outcome traces and comparison trace
sets at runtime rather than committing GLB-body trace caches into Optics
fixtures. Planarian 3D preview code refuses a low-count procedural body in this
mode, validates the Matter-exported GLB surface-anchor rows, and displays the
Matter-exported triangle body mesh separately from sampled nodes and
conductance edges. Its default activity layer consumes Matter-exported
per-node voltage-delta rows at runtime; the live browser adapter also projects
those rows onto GLB body vertices through a bounded nearest-node weighting
cache. Optics fixtures do not duplicate those live circuit deltas or GLB-body
surface-color caches. Loading-stage and split timing counters are likewise
runtime browser-adapter observability, not committed fixture fields. The browser
edit-event timeline is a runtime view over the Optics feedback-frame shape, not
a committed timeline fixture.
