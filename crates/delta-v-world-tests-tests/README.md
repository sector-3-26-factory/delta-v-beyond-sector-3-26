# delta-v-world-tests-tests

Real-app integration tests for `delta-v-world`, compiled into the `delta-v`
binary behind the `test` feature flag.

Test functions live under `src/real_app_ITs/` and are registered in the global
registry via `init_registry()`, called from `main.rs` at startup when
`--test-function <name>` is passed.

Three panic-expecting tests (zero players, two players, collision
detection) are NOT Z2 entries — they expect `build_app` to panic during
world validation, which is incompatible with the Z2 `fn(&mut App)`
model. They live as separate `harness = false` `[[test]]` targets in this
crate's `Cargo.toml`.

See plans/testframework-poc.md Section 9 and ADR-0061.