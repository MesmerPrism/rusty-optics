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

Start the static browser preview with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Start-HandMeshBrowserPreview.ps1
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
