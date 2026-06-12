# Validation

Run the full narrow source check before committing a slice:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\check_all.ps1
```

The check runs:

- `cargo fmt --all --check`
- `cargo test --workspace`
- fixture summary validation
- ADF debug visual fixture validation
- hand-mesh browser debug fixture validation
- projection geometry and source-valid footprint unit tests
- procedural stimulus profile, research-protocol notice policy, full-screen
  stereo-eye presentation, run-plan, oscillator, Perlin-style noise,
  compute-pass ABI, mobile portability, seeded sampler, and CPU reference unit
  tests
- schema catalog validation
- Optics boundary scan

The boundary scan rejects legacy/default namespace drift and renderer/platform
dependencies in Optics core crates.

The hand-mesh browser fixture is regenerated with:

```powershell
cargo run -p rusty-optics-fixtures -- export-hand-mesh-browser
```

The ADF debug visual fixture is regenerated with:

```powershell
cargo run -p rusty-optics-fixtures -- export-adf-debug
```

That command builds a small Matter SDF grid, builds a Matter ADF field from
it, and serializes only the Optics renderer-neutral ADF debug payload in
`fixtures/mesh/adf_debug_visual.json`. Optics does not build ADF in production
paths and does not own ADF sampling truth.

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
sequence for AP-region bioelectric playback with sampled-node surface anchors,
or the live Planarian 3D view for Matter-Wasm-backed scenario switching,
node/edge picking, GLB triangle-anchor readout, Matter-exported node activity
coloring, and edit-intent requests.

The procedural stimulus browser preview is launched with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Start-StimulusPreview.ps1
```

Validate it by opening `http://127.0.0.1:8793/web/stimulus-preview/`,
checking that the page reports a loaded backend, and confirming the bounded
probe range is non-flat. WebGPU-capable browsers should report `webgpu
compute`; other browsers may report `cpu canvas` while still proving fixture
loading and full-screen presentation wiring.
For tuning work, also verify that the control panel appears when chrome is
visible, `Randomize` changes the probe/hashable tuning state, `Reset` restores
the default profile-derived tuning, `Copy Link` places a hash URL in the
address bar and copies it when clipboard access is available, and strobe-style
hash fragments import into the local color/geometry/noise/layer controls
without changing the renderer-neutral fixture. Eccentricity controls should
change warp scale through profile lowering, flat-only transition should collapse
the probe to a uniform field when `Geom=0`, `Flat Hz=0`, and temporal gate is
off, and per-layer oscillator settings should persist independently when
switching the selected layer target. The `Blend` mode should preserve the
existing layer-local compositor in `Stack`, change the probe/hash when moved to
weighted `Mean`, and use the selected layer target for `Cross` as the blend
amount moves from base layer toward target layer.
The live 3D browser preview loops the unedited educational scenario at the
Matter trace horizon so transient dynamics remain visible; selecting or editing
a target disables that loop until reset so inspected state is stable.
The default Planarian 3D layer is `activity dV`; browser smoke checks should
see `nodeActivityCount`, `nodeActivityActiveCount`, and
`nodeActivityMaxDelta` dataset values while the Matter runtime is stepping.
They should also see `surfaceFieldProjection=nearest_node_weights`,
`surfaceFieldInfluenceCount`, and `surfaceFieldColoredVertices` matching the
GLB body vertex count, proving that the body surface receives the same
Matter-owned graph dynamics as a smooth visual projection.
When the first Three.js import is still pending, browser smoke checks should
see the selected view remain `planarian3d`, the 2D canvas hidden, the 3D
viewport visible, `runtimeStatus=3D loading`, and the loading panel dataset
reporting the current Matter/adapter stage. Once ready, the same smoke should
see `runtimeStatus=Matter 3D` plus `planarian3dMatterStepMs`,
`planarian3dViewUpdateMs`, `planarian3dRenderMs`, and
`planarian3dUiDrawMs` dataset values.
The live Planarian 3D path also exposes the Matter comparison trace set through
the browser comparison selector; Optics validates selector wiring and drawing,
not the metrics themselves.

The committed planarian visual sequence intentionally uses Matter's compact
synthetic AP surface so the Optics fixture remains a deterministic visual
contract, not a copied body-asset cache. The browser's live Planarian 3D mode
uses the Matter Wasm runtime and receives the reviewed GLB-derived body surface
from Matter at runtime.

The interaction-intent fixture records renderer-neutral Planarian 3D node and
conductance-edge picks, node-voltage and edge-gate edit intents, and an
edit-feedback frame over recent edit events and affected targets. It validates
Optics' request and feedback shapes only; Matter remains the authority that
accepts, rejects, clamps, mutates, and advances revisions.

The live Planarian 3D scenario selector is a browser smoke-test surface over
Matter reset codes. It should show the GLB-derived body vertex/triangle counts,
render the converted body as a solid Matter-exported triangle mesh rather than
a procedural fallback, show Matter-exported GLB surface-anchor counters for the
sampled node graph, keep Body, Nodes, and Edges as separate visibility
controls, default conductance edges off in this GLB-backed mode, switch among
baseline, wound, gap-block, memory, and no-memory presets, render the
Matter-exported outcome trace panel, compare against alternate Matter scenario
traces, pick both surface nodes and conductance edges, apply a node voltage
edit, preview and apply a Matter-resolved tiered neighborhood voltage-brush
edit, and apply an edge gate-threshold edit. Smoke checks should show the
Matter-exported selection inspector for both targets, show the Matter-exported
recent edit event trail, draw the compact edit-event timeline, highlight
recently affected nodes and conductance edges from Matter-exported target rows
through the Optics feedback-frame shape, and keep those intents routed back
into Matter Wasm after switching. The Planarian 3D browser adapter
intentionally rejects a low-count
body surface or malformed node-anchor stream in this mode so a stale
synthetic/procedural body cannot be mistaken for the reviewed GLB-derived
Matter surface.

The Planarian 3D export smoke validates the default showcase export path for
both body-surface and graph views. It drives the browser UI with Playwright,
saves downloads, and decodes them with Pillow to check frame count and
dimensions.

GIF:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass `
  -File .\tools\Run-Planarian3DExportSmoke.ps1 `
  -Format gif -Width 720 -Height 860 -Fps 12 -DurationSeconds 8 -Views surface,graph -Density 3
```

APNG:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass `
  -File .\tools\Run-Planarian3DExportSmoke.ps1 `
  -Format apng -Width 720 -Height 860 -Fps 12 -DurationSeconds 8 -Views surface,graph -Density 3
```

The showcased dynamics remain Matter-owned synthetic educational planarian
circuit frames. Optics only controls color/material/camera/export framing and
the `Loop: Showcase` visual remapping described in
`docs\PLANARIAN_3D_EXPORT.md`.

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
