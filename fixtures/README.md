# Fixtures

Optics fixtures are deterministic, low-volume artifacts for validating visual
payload shape, projection behavior, billboard budgets, and schema wiring. They
are not GPU captures and do not include downstream private visual mappings.

Regenerate and check fixtures with:

```powershell
cargo run -p rusty-optics-fixtures -- export --check
```

