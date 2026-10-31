# delta-v-ui-tests

Real-app integration tests for `delta-v-ui`, compiled into the `delta-v`
binary behind the `test` feature flag.

Test functions live under `src/real_app_ITs/` and are registered in the global
registry via `init_registry()`, called from `main.rs` at startup when
`--test-function <name>` is passed.

See plans/testframework-poc.md Section 9 and ADR-0061.