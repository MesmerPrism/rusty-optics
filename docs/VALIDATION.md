# Validation

Run the full narrow source check before committing a slice:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\check_all.ps1
```

The check runs:

- `cargo fmt --all --check`
- `cargo test --workspace`
- fixture summary validation
- schema catalog validation
- Optics boundary scan

The boundary scan rejects legacy/default namespace drift and renderer/platform
dependencies in Optics core crates.

