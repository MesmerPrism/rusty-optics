# Rusty Optics Agent Notes

This is the clean source repository for Rusty Optics. Keep committed content
self-contained and free of local-only planning paths, downstream app names,
platform-specific runtime handles, shader backend imports, and historical
naming drift.

Rusty Morphospace is the top-level project/platform umbrella. This repo remains
the Optics lane inside that umbrella: morphology of appearance, projection,
view, visual payload, debug visualization, and renderer-neutral evidence. Do
not introduce `rusty.morphospace.*` schemas here; use `rusty.optics.*` for
Optics contracts.

Project-owned source in this repo is licensed `AGPL-3.0-or-later`. Keep
third-party dependencies, screenshots, media, GLB or mesh assets, fonts,
imported shaders, renderer backends, binary releases, and external tools under
their own provenance and notice requirements; see `docs/LICENSING.md`.

## Purpose

Rusty Optics owns renderer-neutral view, projection, appearance, animation
descriptor, visual payload, debug-visualization, and optical scorecard
contracts.

It should remain usable without UI frameworks, renderer backends, platform
SDKs, XR runtimes, headset tooling, device APIs, dynamic plugin loading,
runtime sockets, media stacks, or downstream app crates.

## Read Order

1. `README.md`
2. `docs/ARCHITECTURE.md`
3. `docs/VALIDATION.md`
4. `fixtures/README.md`

## Architecture Rules

- Matter owns geometry, fields, particles, simulation state, and render-neutral
  payload truth. Optics consumes those payloads and adds view/appearance policy.
- Optics owns visual particle frames, billboard preparation contracts, flat
  projection, animation-mask descriptors, trail appearance descriptors,
  transparency/depth policy, renderer-neutral budget summaries, mesh debug
  frames, coordinate-map visuals, collider visuals, SDF slice visuals, and
  surface-field visual frames/sequences over Matter-owned field payloads,
  including planarian AP bioelectric visual sequences and renderer-neutral
  planarian 3D pick/edit-intent contracts. Optics also owns procedural stimulus
  descriptors, safety/timing/run-plan contracts, kernel ABI descriptors, and
  CPU reference samples; renderer adapters own shader source and GPU execution.
- Renderer adapters own GPU buffers, shaders, draw calls, texture uploads,
  swapchains, platform frame lifecycle, and backend imports.
- Lattice owns runtime-situated relation snapshots: reference spaces,
  transforms, tracked poses, view sets, spatial input roles, frame-state
  binding, calibration, validity, confidence, and runtime capability evidence.
  Optics may consume Lattice view sets for rendering decisions, but stereo
  projection, lenses, homographies, and appearance policy stay Optics-owned.
- Animated hand-mesh browser previews must call Matter runtime code through the
  Matter Wasm package for mesh-distance/SDF/particle queries. Do not re-add
  browser-owned brute-force triangle-distance simulation as the default path.
- Surface-field browser previews must call Matter runtime code through the
  Matter surface-field Wasm package for live dynamics. Browser code may own
  playback and drawing, but not diffusion, decay, perturbation, or vector
  update rules.
- Downstream apps own visual-driver mappings, exact runtime tuning,
  app-specific coupling/control behavior, study defaults, and product profiles.
- Use `rusty.optics.*` schema IDs for default Optics contracts. Legacy names may
  appear only in explicitly named compatibility layers outside Optics core.
- Keep high-rate particle arrays out of command/control JSON routes; use
  artifacts, bounded summaries, or data-plane adapters.

## File Organization Rules

- Keep `src/lib.rs` files as facades: module declarations, public reexports,
  and short crate-level docs only.
- Keep binary `src/main.rs` files as dispatch-only entrypoints. CLI parsing,
  artifact generation, validation checks, and catalog code belong in focused
  modules.
- Split before adding behavior when a file starts mixing independent families.
  For Optics, the important families are colors/model IDs, visual particle
  frames, appearance profiles, billboards, flat projection, animated masks,
  mesh debug frames, coordinate visuals, collider visuals, SDF slice visuals,
  fixtures, schema catalogs, and boundary scans.
- Preserve public names, schema IDs, serde field names, fixture outputs, CLI
  messages, validation outcomes, and dependency boundaries during mechanical
  splits. Validate with `.\tools\check_all.ps1` before continuing a feature
  slice.

Current crate-root maps:

- `rusty-optics-model/src/lib.rs`: facade over `color`, `error`, `ids`,
  `projection`, and `vec2`. The `projection` module owns target footprints,
  source sampling modes, source-valid screen UV summaries, homography helpers,
  and renderer-neutral projection geometry reports.
- `rusty-optics-mesh/src/lib.rs`: facade over `browser_frame`,
  `adf_debug`, `circuit_frame`, `collider`, `coordinate`, `field_frame`,
  `mesh_frame`, `planarian_frame`, `planarian_interaction`, `sdf_slice`, and
  tests. The `planarian_interaction` module owns pick-selection, edit-intent,
  and edit-feedback frame contracts for Planarian 3D renderer adapters.
- `rusty-optics-particles/src/lib.rs`: facade over `animation`, `appearance`,
  `billboard`, `mask`, `projection`, `tests`, and `visual_frame`.
- `rusty-optics-stimulus/src/lib.rs`: facade over `layers`, `noise`,
  `oscillator`, `presentation`, `temporal`, `safety`, `kernel_abi`,
  `run_plan`, `profile`, `cpu_reference`, `volume`, `volume_probe`,
  `volume_profile`, `volume_preview`, `volume_cpu_reference`, and tests.
- `web/stimulus-preview/app.js`: browser-development adapter that lowers a
  `StimulusProfile` fixture to a full-screen WebGPU compute preview with a CPU
  canvas fallback and bounded probe readouts. Keep Quest/OpenXR/Vulkan runtime
  allocation and submission in future adapters, not this browser surface.
- `web/stimulus-preview/tuning.js`: browser-development tuning translator for
  compact hash presets, randomization, live control-panel edits, eccentricity
  controls, flat-to-geometry transition state, global geometry-stack blend
  controls, and per-layer oscillator banks. It may translate external preset
  vocabulary into the Optics profile shape, but must not become core stimulus
  authority or a Quest runtime dependency.
- `web/surface-field-preview/app.js`: browser-development surface-field and
  Planarian preview adapter for playback, controls, drawing, live Matter Wasm
  calls, and Planarian 3D interaction wiring.
- `web/surface-field-preview/planarian-3d-readouts.js`: browser helper for
  Planarian scenario, graph-density, source-target, region, metric, and edit
  vocabulary plus pure readout formatting. It may format Optics presentation
  text only; Matter remains the scenario and dynamics authority.
- `web/surface-field-preview/planarian-3d-export.js`: browser helper for
  Planarian 3D export defaults, capture-frame shaping, encoder dispatch,
  downloads, and export metadata. It may drive Optics export presentation only;
  Matter remains the scenario and dynamics authority.
- `web/surface-field-preview/surface-field-utils.js`: pure browser helper
  module for bounds, color mapping, formatting, clamping, and small vector
  utilities shared by the surface-field preview adapter.
- `rusty-optics-fixtures/src/main.rs`: dispatch-only binary over `cli`,
  `adf`, `error`, `fields`, `hand_mesh`, `stimulus`, and `summary`.
- `rusty-optics-schema/src/main.rs`: dispatch-only binary over `catalog`, `cli`,
  and `error`.

## Sustainable Design Guardrails

- Treat monolithic file pressure as an ownership problem, not a line-count
  problem. Split only by durable authority, schema, route, validation, adapter,
  or test-family boundaries; preserve facades, schema IDs, serde fields,
  fixture outputs, CLI behavior, validation outcomes, and dependency boundaries.
- After a split, update the nearest distributed file map: this `AGENTS.md`,
  `README.md`, `docs/ARCHITECTURE.md`, fixture docs, validation docs, or the
  planning `agent-state\iteration-events.jsonl`.
- Keep `AGENTS.md`, README, and skill files as concise routing indexes. Move
  lane-specific recipes, device/build detail, compatibility ledgers, and long
  validation flows into named docs or runbooks.
- Keep legacy Rusty-XR names as explicit compatibility surfaces only. New
  schemas, routes, and types use the owning lane (`rusty.manifold.*`,
  `rusty.lattice.*`, `rusty.matter.*`, `rusty.optics.*`, `rusty.quest.*`, or
  repo-local names); do not introduce `rusty.morphospace.*` schemas or
  `Morphospace*` core types by default.
## Validation

Run narrow checks before committing a slice:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\check_all.ps1
```
