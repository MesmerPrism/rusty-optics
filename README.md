# Rusty Optics

Rusty Optics is the renderer-neutral visual contract layer for the Rusty stack.
It consumes Matter payloads and describes how those payloads should be viewed,
projected, animated, inspected, and handed to renderer adapters.

The first source slice focuses on visual particles:

- visual particle frames derived from Matter particle payloads;
- appearance profiles for billboard draw mode, blend/depth policy, animated
  mask descriptors, facing response, frame scaling, and trail appearance;
- CPU billboard instance preparation for renderer adapters;
- flat-screen projection for desktop, browser, phone, or test previews;
- animated morphed-ring mask atlas generation;
- transparent particle render-budget summaries.

Optics does not own particle simulation, mesh/SDF truth, private visual-driver
bindings, shader source, GPU uploads, OpenXR/Vulkan/WebGL/Makepad integrations,
or downstream product profiles.

## Validation

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\check_all.ps1
```

