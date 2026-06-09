# Rusty Optics

Rusty Optics is the renderer-neutral visual contract layer for the Rusty stack.
It consumes Matter payloads and describes how those payloads should be viewed,
projected, animated, inspected, and handed to renderer adapters.

The first source slices focus on visual particles and mesh diagnostics:

- visual particle frames derived from Matter particle payloads;
- appearance profiles for billboard draw mode, blend/depth policy, animated
  mask descriptors, facing response, frame scaling, and trail appearance;
- CPU billboard instance preparation for renderer adapters;
- flat-screen projection for desktop, browser, phone, or test previews;
- animated morphed-ring mask atlas generation;
- transparent particle render-budget summaries;
- mesh debug frames derived from Matter triangle mesh surfaces;
- coordinate-map, dynamic-collider, and SDF-slice debug visuals over that same
  source surface;
- a static browser preview that renders the generated mesh debug JSON without
  renderer backend imports.

Optics does not own particle simulation, mesh/SDF truth, downstream
visual-driver bindings, shader source, GPU uploads, OpenXR/Vulkan/WebGL/Makepad
integrations, or downstream product profiles.

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

## Validation

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\check_all.ps1
```
