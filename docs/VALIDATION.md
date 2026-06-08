# Validation

Run the full narrow source check before committing a slice:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\check_all.ps1
```

The check runs:

- `cargo fmt --all --check`
- `cargo test --workspace`
- fixture summary validation
- hand-mesh browser debug fixture validation
- schema catalog validation
- Optics boundary scan

The boundary scan rejects legacy/default namespace drift and renderer/platform
dependencies in Optics core crates.

The hand-mesh browser fixture is regenerated with:

```powershell
cargo run -p rusty-optics-fixtures -- export-hand-mesh-browser
```

The surface-field browser fixture is regenerated with:

```powershell
cargo run -p rusty-optics-fixtures -- export-surface-field-preview
```

That command writes both `fixtures/fields/surface_field_visual_frame.json` and
`fixtures/fields/surface_field_visual_sequence.json`, plus
`fixtures/fields/bioelectric_circuit_visual_frame.json` and
`fixtures/fields/planarian_bioelectric_visual_sequence.json` and
`fixtures/fields/planarian_bioelectric_interaction_intent.json`. The browser
preview defaults to the sequence for dynamic playback and can switch to the
circuit frame for voltage/conductance/memory/readout inspection, the planarian
sequence for AP-region bioelectric playback, or the live Planarian 3D view for
Matter-Wasm-backed scenario switching, picking, and edit-intent requests.

The committed planarian visual sequence intentionally uses Matter's compact
synthetic AP surface so the Optics fixture remains a deterministic visual
contract, not a copied body-asset cache. The browser's live Planarian 3D mode
uses the Matter Wasm runtime and receives the reviewed GLB-derived body surface
from Matter at runtime.

The interaction-intent fixture records one renderer-neutral Planarian 3D node
pick and one voltage edit intent. It validates Optics' request shape only;
Matter remains the authority that accepts, rejects, clamps, mutates, and
advances revisions.

The live Planarian 3D scenario selector is a browser smoke-test surface over
Matter reset codes. It should show the GLB-derived body vertex/triangle counts,
switch among baseline, wound, gap-block, memory, and no-memory presets, and keep
pick/edit intents routed back into Matter Wasm after switching.

Start the static browser preview with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Start-HandMeshBrowserPreview.ps1
```

Start the surface-field static preview with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Start-SurfaceFieldPreview.ps1 -BuildMatterWasm
```

Without `-BuildMatterWasm`, the preview falls back to the checked visual
sequence fixture. With it, the preview imports
`local-artifacts\matter_surface_field_wasm\rusty_matter_fields_wasm.js` and
steps the Matter runtime live in the browser.

Build the Matter surface-field WebAssembly runtime package into Optics local
artifacts with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Build-SurfaceFieldPreviewMatterWasm.ps1 `
  -MatterRepoRoot "<rusty-matter repo root>"
```

## External Mesh Surface Browser Smoke

When Matter has extracted `TriangleMeshSurface` JSON files from an external
GLB, Optics can generate a browser-ready debug frame from one surface without
headset or renderer access:

```powershell
cargo run -p rusty-optics-fixtures -- export-hand-mesh-browser-from-surface `
  --surface-json "<rusty-matter\local-artifacts\...\surface.json>" `
  --include-sdf-particles `
  --output "local-artifacts\hand_mesh\hand_mesh_browser_debug_frame.json"

powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Start-HandMeshBrowserPreview.ps1 `
  -FramePath "local-artifacts\hand_mesh\hand_mesh_browser_debug_frame.json"
```

The generated frame feeds the mesh wireframe, coordinate-map visual, dynamic
collider visual, SDF slice, and optional SDF particle overlay from the same
Matter surface and SDF grid. It remains outside `check_all.ps1` because the
source surface is a local external artifact.

When `--include-sdf-particles` is enabled, the local external frame also carries
the packed Matter SDF grid used by the preview. The browser's `Live` toggle and
`Reset Particles` button are preview-only controls: they reset particles into a
sphere inside the SDF bounds and advance them against the sampled SDF gradient
so SDF reaction can be inspected without adding renderer or legacy runtime
dependencies.

## Animated Runtime SDF Browser Smoke

For realtime deformation checks, point the same browser preview at a Matter
animated surface-sequence JSON exported from a hand recording:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Start-HandMeshBrowserPreview.ps1 `
  -BuildMatterWasm `
  -FramePath "local-artifacts\hand_mesh\hand_mesh_realtime_sequence.json"
```

Animated sequence payloads contain mesh positions and topology only. The browser
preview recomputes the mesh wireframe, collider shell, visible SDF slice, and
particle SDF response from the current animation frame through the Matter Wasm
surface-distance runtime. Use `Pause` to freeze the hand pose and `Reset
Particles` to seed 1000 particles into a larger random sphere around the hand
before the live SDF force pulls them toward the current mesh surface. The
metrics panel should report Matter Wasm BVH node counts and actual accelerated
triangle tests; it should not report full particle-count x triangle-count
brute-force work.
