# M0.5 Implementation Summary

**Milestone M0.5 - Architecture Foundation** has been successfully implemented.

## What was done

### 1. Workspace Conversion (ADR-0002)

The monolithic `src/` directory has been converted to a Cargo workspace with 12 crates:

```
crates/
├── delta-v/                 # Binary (App composition)
├── delta-v-core/            # ECS fundamentals, shared types
├── delta-v-config/          # JSON loading, schema validation
├── delta-v-physics/         # Newtonian physics, avian3d integration
├── delta-v-assets/          # Asset loaders, glTF helpers
├── delta-v-net/             # Networking (placeholder)
├── delta-v-ships/           # Ship domain
├── delta-v-propulsion/      # Propulsion systems
├── delta-v-weapons/         # Weapons systems
├── delta-v-stations/        # Space stations
├── delta-v-items/           # Collectables
└── delta-v-world/           # World management
```

**Benefits:**
- Better incremental compilation performance
- Compile-time cycle prevention
- Clear dependency boundaries between domains
- Each crate has a natural place for tests and documentation

### 2. Workspace Dependencies (ADR-0002 follow-up)

All shared dependencies are now defined in the root `Cargo.toml` under
`[workspace.dependencies]` and inherited by members via `workspace = true`.

Shared dependencies:
- `bevy` (0.14)
- `avian3d` (0.1)
- `serde` / `serde_json`
- `log` / `env_logger`
- `thiserror`
- `tokio`

### 3. Workspace Lints (ADR-0023, ADR-0034)

Consistent linting rules defined at workspace level:
- `rust.unsafe_code = "warn"` -- Unsafe code must be justified
- `clippy.all`, `clippy.pedantic`, `clippy.nursery` enabled

Every crate inherits these rules via `[lints]` section.

### 4. Plugin Architecture (ADR-0005)

Each crate exports exactly one Bevy `Plugin`:

```rust
pub struct CorePlugin;
impl Plugin for CorePlugin { .. }
```

The binary (`crates/delta-v/`) registers all plugins explicitly:

```rust
App::new()
    .add_plugins(CorePlugin)
    .add_plugins(ConfigPlugin)
    .add_plugins(PhysicsPlugin)
    // ... etc
    .run()
```

This makes the entire dependency graph explicit and auditable.

### 5. Source File Header Convention (ADR-0033)

All source files now begin with:

```rust
// See AGENTS.md
//
// Delta-V beyond Sector 3.26
// Copyright (C) 2025  Cute-Donkey
// ...
```

The "See AGENTS.md" comment is the single entry point for agents.
It points to `AGENTS.md`, which points to `docs/adr/README.md`.

This indirection keeps source files stable when ADRs are added or renumbered.

### 6. Cargo Deny Configuration (ADR-0028, ADR-0029)

File: `deny.toml`

Configures supply-chain security checks:
- **Advisories**: Warn on RustSec database hits
- **Licenses**: Allow GPL-3.0+, MIT, Apache-2.0, BSD-2/3-Clause, and other OSI-approved licenses
- **Bans**: Prevent accidentally pulling in forbidden crates
- **Sources**: Allow only official crates.io and GitHub (with whitelist)

Integration: CI will run `cargo deny check` to catch supply-chain issues.

### 7. Pre-commit Hook (ADR-0034)

File: `.githooks/pre-commit`

Before each commit, the hook enforces:

```bash
cargo fmt --all -- --check          # No formatting violations
cargo clippy --workspace --all-targets -- -D warnings  # No warnings
cargo test --workspace              # All tests pass
cargo deny check                     # Supply chain clean
```

Configuration:
```bash
git config core.hooksPath .githooks
```

### 8. Documentation Updates

- **README.md**: Updated status to M0.5, added crate structure info
- **docs/architecture.md**: Added concrete crate structure section
- **docs/workspace-guide.md**: Workspace developer guide
- **docs/M0.5/**: Complete milestone documentation

## How to build

```bash
# Check the entire workspace
cargo check --workspace

# Build the binary
cargo build --release

# Run the binary
cargo run --bin delta-v

# Run all tests
cargo test --workspace

# Format all code
cargo fmt --all

# Lint all code
cargo clippy --workspace --all-targets -- -D warnings

# Check supply chain
cargo deny check
```

## Compliance with ADRs

M0.5 is fully compliant with all Accepted ADRs (0001-0035, 0036, 0037):

1. **ADR-0001**: ADR process established and documented
2. **ADR-0002**: Workspace structure matches specification exactly
3. **ADR-0003**: Branching workflow documented (Gitflow-light)
4. **ADR-0004**: Conventional Commits format enforced in CI
5. **ADR-0005**: Each crate is a plugin; binary composes them
6. **ADR-0006 onwards**: All foundation ADRs recorded and referenced
7. **ADR-0033**: Source file headers in place
8. **ADR-0034**: Pre-commit hook enforces no-warnings policy

## Next milestone: M1 -- A ship in space

M1 will:
- Load a default world (JSON + glTF)
- Spawn a primitive ship mesh with a chase camera
- Wire up input (keybindings from JSON)
- Add logging and performance diagnostics

See [`../roadmap.md#m1----a-ship-in-space`](../roadmap.md) for details.

## Testing the setup

To verify everything is working:

```bash
# Run the pre-commit hook manually
./.githooks/pre-commit

# Or let git run it automatically on commit
git commit -m "feat: verify M0.5 implementation"
```

If all checks pass, M0.5 is ready for integration into `dev`.
