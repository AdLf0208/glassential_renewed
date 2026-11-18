# Validation

## Embedded content + re-verification (2026-11-10)

Horizon `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9` (game.rs `64813cf4…`),
horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57` (schema 6), every step
through `ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0`.

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --all-targets --locked --offline -- -D warnings` | exit 0, 0 warnings |
| `cargo test --locked --offline` | green |
| `horizon.py stage` / `check` / `test` | exit 0 / `cargo_exit: 0`, 0 warnings / embedded tests green |
| `tools/package.sh` | `content/glassential.hmod` 111,168 B `e9134c09e528856c…`, `glassential.hnmod` 76,387 B `56465263dab5a227…` (10 entries, embeds the content) |
| e2e (no `--dev-mod`) | default `PASS` + `EXIT:0`; `COUNTERFACTUAL=1` `PASS` + `EXIT:0`; `COUNTERFACTUAL=1 NEGATIVE=1` `FAIL` + `EXIT:1` |
| `upstream_ready.py glassential` | **READY (exit 0)** |
