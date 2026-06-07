"""Check Optics dependency and namespace boundaries."""

from __future__ import annotations

import sys
from pathlib import Path


FORBIDDEN_CARGO_TERMS = {
    "rusty-xr",
    "rusty_xr",
    "viscereality",
    "astralkaratedojo",
    "akd",
    "kuramoto",
    "makepad",
    "openxr",
    "vulkan",
    "webgl",
    "android",
    "quest",
}

FORBIDDEN_SOURCE_TERMS = {
    "rusty.xr.",
    "debug.rustyxr.",
    "/rustyxr/v1/",
    "viscereality",
    "astralkaratedojo",
    "kuramoto",
}

SCAN_EXTENSIONS = {".rs", ".toml", ".json", ".js", ".html", ".css"}


def main() -> int:
    repo = Path(__file__).resolve().parents[1]
    failures: list[str] = []

    for cargo_toml in repo.rglob("Cargo.toml"):
        text = cargo_toml.read_text(encoding="utf-8")
        lower_text = text.lower()
        for term in FORBIDDEN_CARGO_TERMS:
            if term in lower_text:
                failures.append(f"{cargo_toml}: forbidden cargo boundary term {term!r}")

    roots = [
        repo.joinpath("crates"),
        repo.joinpath("schemas"),
        repo.joinpath("fixtures"),
        repo.joinpath("web"),
    ]
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file() or path.suffix.lower() not in SCAN_EXTENSIONS:
                continue
            text = path.read_text(encoding="utf-8").lower()
            for term in FORBIDDEN_SOURCE_TERMS:
                if term in text:
                    failures.append(f"{path}: forbidden source boundary term {term!r}")

    if failures:
        for failure in failures:
            print(f"[FAIL] {failure}")
        return 1

    print("[PASS] Optics dependency and namespace boundaries")
    return 0


if __name__ == "__main__":
    sys.exit(main())
