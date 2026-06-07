# Fixtures

Optics fixtures are deterministic, low-volume artifacts for validating visual
payload shape, projection behavior, billboard budgets, and schema wiring. They
are not GPU captures and do not include downstream private visual mappings.

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
