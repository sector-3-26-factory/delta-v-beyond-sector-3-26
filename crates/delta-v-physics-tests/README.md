# delta-v-physics-tests

Real-app integration tests for `delta-v-physics`, compiled into the
`delta-v` binary behind the `test` feature flag.

See ADR-0061.

## Structure

```
src/real_app_ITs/          test functions (linked into binary)
tests/real_app_ITs/        #[test] drivers (spawn binary, check exit code)
tests/real_app_ITs/<cat>/fixtures/  world fixtures
```

Each category folder (e.g. `collision/sphere_sphere/`) holds its own
test functions, driver, and fixtures — everything for that test lives
together.