# Stimulus Preview

This browser adapter loads a renderer-neutral `StimulusProfile` fixture and
renders it as a full-screen `StereoEyeField` development preview.

The primary path uses WebGPU compute to generate a field texture, then draws
that texture as a full-viewport pass. Browsers without WebGPU use the bounded
CPU canvas fallback so descriptor loading and visual validation remain
available. The fallback is not the Quest adapter path.

The tuning panel is browser-development glue. It exposes color, oscillator,
geometry, eccentricity, post, transition, geometry-stack blend, noise,
vignette, layer, and per-layer oscillator controls. `Randomize` creates a
bounded tuning state; `Reset` returns to the profile-derived defaults; and
`Copy Link` places a compact hash preset in the address bar and copies it when
browser clipboard access is available. Hash fragments that use compatible
strobe-style preset fields are translated into the local tuning shape and then
applied to a cloned `StimulusProfile`. The `Transition` group is the fast
flat-strobe path: `Geom=0` renders a full flat field, `Geom=1` renders the
geometry output, and the fade controls smoothly move between those states. The
`Blend` group controls how geometry layers are combined before that flat/field
transition: `Stack` keeps the layer-local Add/Multiply/Max behavior, `Mean`
fades toward a weighted average of active layers, and `Cross` fades from the
base layer to the selected layer target.

Run locally with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\Start-StimulusPreview.ps1
```
