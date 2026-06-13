# Planarian 3D Export

## Decision

`Showcase` remains an Optics-owned export loop mode. It does not add a new
Matter dynamics preset and does not change the Matter-owned planarian circuit
state. Matter still owns the GLB-derived body surface, sampled graph, voltage,
conductance, gates, memory, readouts, deterministic stepping, scenario resets,
and comparison traces.

Optics owns the export presentation: surface versus graph visibility, palette,
opaque material policy, stable portrait camera, exact output dimensions,
GIF/APNG/video encoding, and export-time loop shaping.

Matter also owns the compact source/target anchors and voltage-unit policy that
explain the synthetic scenario. The browser preview reads those fields from the
Matter Wasm runtime and displays them in the dynamics panel; export metadata
records the same `evidence_type`, `expected_outcome`, `voltage_unit_policy`,
and `literature_anchors` values. It also serializes parsed `source_targets`
and a `source_target_policy` that preserves the non-calibrated physiology and
PlanformDB-provenance boundary for export validators.

## Showcase Behavior

The default showcase export uses the Matter-owned synthetic `Memory` scenario
unless the UI has selected another scenario. The scenario is a qualitative,
normalized educational planarian anterior/posterior bioelectric model with:

- AP-region voltage-like state over Matter surface nodes;
- conductance and gate activity over same-surface graph edges;
- transient activity and hysteresis-like memory/readout layers;
- posterior memory/head-identity behavior compared with the no-memory control.

The current source-target anchors connect this visual teaching model to public
source IDs for early transient-memory behavior, head-versus-tail voltage
context, gap/conductance perturbations, Matter-owned normalized size/readout
metrics, rights-safe head-label taxonomy, and tiny PlanformDB-derived review
fixtures. They do not claim calibrated millivolt values, source-fit thresholds,
or PlanformDB-derived prediction.

The outcome panel summarizes the active Matter trace relationship, such as
`memory vs no-memory control`, from Matter-exported trace values and scenario
metadata. It is a teaching readout over Matter-owned data, not a pass/fail
metric or browser-owned dynamics model.

`Loop: Showcase` captures Matter-stepped frames, selects the visually active
segment, resamples by image change, and mirrors that segment so the file loops
cleanly. This is a visual remapping only. It should be described as a sustained
showcase of the synthetic educational dynamics, not a calibrated physiology
claim and not a PlanformDB-derived predictor.

Use `Loop: Forward` when the exported file should preserve strict forward time
ordering without visual loop shaping.

## Export Settings

The Planarian 3D toolbar exposes:

- `Format`: APNG, GIF, WebM, or MP4 where the browser supports it;
- `View`: smooth body `Surface` or sampled-node/edge `Graph`;
- `Palette`: Optics neon RGB or teaching palette;
- `Start`: reset current Matter scenario or continue from current state;
- `Loop`: showcase visual loop or forward stepping;
- `Layer`: current layer or activity dV;
- `Material`: opaque or boosted opaque body material;
- `Duration`, `FPS`, `Width`, `Height`, `Step`, and `Warmup`.

Default showcase exports use reset start, showcase loop, activity layer, opaque
neon RGB material, a stable portrait camera, `8s * 12fps = 96` frames, and
`720x860` output. GIF export builds an adaptive palette and dithers the encoded
frames. APNG keeps full RGBA frames for higher-fidelity browser playback.

## Validation

Narrow JavaScript syntax checks:

```powershell
node --check web\surface-field-preview\app.js
node --check web\surface-field-preview\planarian-3d.js
node --check web\surface-field-preview\gif-encoder.js
node --check web\surface-field-preview\apng-encoder.js
node --check web\surface-field-preview\video-encoder.js
node --check tools\planarian_3d_export_smoke.cjs
```

Browser export and Pillow decode smoke for surface and graph GIFs:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass `
  -File .\tools\Run-Planarian3DExportSmoke.ps1 `
  -Format gif -Width 720 -Height 860 -Fps 12 -DurationSeconds 8 -Views surface,graph -Density 3
```

Browser export and Pillow/APNG chunk smoke for surface and graph APNGs:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass `
  -File .\tools\Run-Planarian3DExportSmoke.ps1 `
  -Format apng -Width 720 -Height 860 -Fps 12 -DurationSeconds 8 -Views surface,graph -Density 3
```

The smoke opens the local preview with Playwright, exports both views through
the UI, saves them under `local-artifacts\planarian3d-export-smoke`, and
validates frame count and dimensions with Pillow. It also checks the export
metadata for the normalized voltage policy, synthetic evidence label, parsed
Matter source targets, non-calibrated source-target policy, and
non-predictive PlanformDB boundary, then verifies that the default outcome
panel reports the memory-versus-no-memory teaching relation. For APNG it
checks the PNG animation chunks (`acTL` and `fcTL`) so a static PNG cannot pass
as a high-fidelity animation export.

PlanformDB-derived and morphology-label fixtures remain Matter-owned review
metadata. The export smoke should continue treating them as provenance and
annotation context only; it must not use them as browser-side dynamics,
stochastic prediction, or calibrated morphology authority.
