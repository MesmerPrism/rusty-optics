# Rusty Optics Agent Notes

This is the clean source repository for Rusty Optics. Keep committed content
self-contained and free of private planning paths, downstream app names,
platform-specific runtime handles, shader backend imports, and historical
naming drift.

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
  surface-field visual frames/sequences over Matter-owned field payloads.
- Renderer adapters own GPU buffers, shaders, draw calls, texture uploads,
  swapchains, platform frame lifecycle, and backend imports.
- Animated hand-mesh browser previews must call Matter runtime code through the
  Matter Wasm package for mesh-distance/SDF/particle queries. Do not re-add
  browser-owned brute-force triangle-distance simulation as the default path.
- Surface-field browser previews must call Matter runtime code through the
  Matter surface-field Wasm package for live dynamics. Browser code may own
  playback and drawing, but not diffusion, decay, perturbation, or vector
  update rules.
- Downstream apps own private visual-driver mappings, exact runtime tuning,
  oscillator coupling, breath/control bindings, study defaults, and product
  profiles.
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

- `rusty-optics-model/src/lib.rs`: facade over `color`, `error`, `ids`, and
  `vec2`.
- `rusty-optics-mesh/src/lib.rs`: facade over `browser_frame`, `collider`,
  `coordinate`, `field_frame`, `mesh_frame`, `sdf_slice`, and tests.
- `rusty-optics-particles/src/lib.rs`: facade over `animation`, `appearance`,
  `billboard`, `mask`, `projection`, `tests`, and `visual_frame`.
- `rusty-optics-fixtures/src/main.rs`: dispatch-only binary over `cli`, `error`,
  `fields`, `hand_mesh`, and `summary`.
- `rusty-optics-schema/src/main.rs`: dispatch-only binary over `catalog`, `cli`,
  and `error`.

## Validation

Run narrow checks before committing a slice:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\check_all.ps1
```
