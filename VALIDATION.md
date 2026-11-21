# Validation

## Slice 3 — clear-fluid connections (0.2.2, 2026-11-21)

Horizon/SDK `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` (nightly 2026-11-20), horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`; packaged by `cargo horizon-mod`, the game built by
`ports/tools/launcher-game.sh`, every step in a `build-slot.sh` lease with `CARGO_INCREMENTAL=0`
and `BUILD_LOCAL=1` (the port has no `tools/horizon.py` ladder any more).

| Check | Result |
|---|---|
| `cargo test --offline` | **17 passed, 0 failed** (3 new in `src/connect.rs`) |
| `cargo clippy --all-targets -- -D warnings` / `cargo fmt --check` | 0 warnings / clean |
| `cargo horizon-mod build` ×2 with `rm -f Cargo.lock` | `cmp`-identical, SHA-256 `2728a87ba5c469623230c137d16e0b1ca9b7e72ab99e1c836f16277e9f44fe92` |
| `cargo horizon-mod check glassential.hmod` | `glassential 0.2.2`, `native-1/0a875da91…/e59da909…`, 132,860 B, exit 0 |
| `cargo horizon-mod fits … --build 0a875da` / `--build 0b14f8a` | `0a875da made for` / `0b14f8a fits`, both exit 0 |
| `verify-sdk-package.py glassential` | 1/1 |
| e2e red (0.2.1 sources) | `FAIL: …: south is 'false', expected 'true'` + `EXIT:1` (`../_shared/evidence/glassential-e2e/e2e-red-connect.log`) |
| e2e default (0.2.2) | `PASS` + `EXIT:0` (`…/e2e-connect.log`) |
| e2e control (unmodded Nightly) | `FAIL: setblock glassential:glass_dark_ethereal failed` + `EXIT:1` |
| e2e `SELFTEST=1` | `FAIL: NEGATIVE run did not observe the false assertion` + `EXIT:1` |
| `check-e2e-logs.py glassential glassential-e2e --strict` | exit 0, 11 logs |

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
