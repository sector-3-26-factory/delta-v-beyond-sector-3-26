# delta-v-world-tests

Integration tests for `delta-v-world` using `harness = false` targets.

These tests drive the real app via `delta_v_app::build_app` (per ADR-0060),
so they require a virtual display (`xvfb-run`) in CI.