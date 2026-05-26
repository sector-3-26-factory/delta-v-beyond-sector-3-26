# Workspace Guide

This document explains the Cargo workspace structure and how to work with it.

See [ADR-0002](adr/0002-repository-layout-and-workspace.md) for the rationale.

## Overview

The project is organized as a **Cargo workspace** with 12 member crates:

```
Repository root/
├── Cargo.toml               # Workspace definition
├── deny.toml                # cargo-deny config
├── crates/
│   ├── delta-v/             # Binary (App composition)
│   ├── delta-v-core/        # Technical: ECS fundamentals
│   ├── delta-v-config/      # Technical: JSON loading
│   ├── delta-v-physics/     # Technical: Physics integration
│   ├── delta-v-assets/      # Technical: Asset loading
│   ├── delta-v-net/         # Technical: Networking (stub)
│   ├── delta-v-ships/       # Domain: Ships
│   ├── delta-v-propulsion/  # Domain: Propulsion
│   ├── delta-v-weapons/     # Domain: Weapons
│   ├── delta-v-stations/    # Domain: Stations
│   ├── delta-v-items/       # Domain: Items
│   └── delta-v-world/       # Domain: World
```

## Common commands

### Building and testing

```bash
# Check entire workspace
cargo check --workspace

# Build release binary
cargo build --release --bin delta-v

# Run the game
cargo run --bin delta-v

# Run tests across all crates
cargo test --workspace

# Run tests for a specific crate
cargo test -p delta-v-physics
```

### Code quality

```bash
# Format all code
cargo fmt --all

# Lint all code
cargo clippy --workspace --all-targets -- -D warnings

# Check supply chain security
cargo deny check

# Run the pre-commit hook manually
./.githooks/pre-commit
```

### Working with a specific crate

```bash
# Add a dependency to delta-v-physics
cargo add -p delta-v-physics thiserror

# Build just delta-v-config
cargo build -p delta-v-config

# Run tests in delta-v-ships
cargo test -p delta-v-ships

# Generate docs for a crate
cargo doc -p delta-v-ships --open
```

## Crate categories

### Binary crate

**`crates/delta-v/`** -- The application entry point.

- Purpose: Composes all plugins, initializes the Bevy App.
- Dependencies: All library crates (should be minimal beyond that).
- Plugin: `DeltaVPlugin` (or similar; actually registers all plugins).
- Exports: Nothing (binary only).

### Technical crates

These provide cross-cutting infrastructure:

**`crates/delta-v-core/`**
- ECS fundamentals (shared components, resources)
- Logging initialization
- Plugin trait definitions
- May not depend on domain crates

**`crates/delta-v-config/`**
- JSON loading and deserialization
- Schema validation
- Configuration merging (defaults + user overrides)
- May not depend on domain crates

**`crates/delta-v-physics/`**
- Newtonian physics simulation (on top of avian3d)
- Gravity, collisions, forces, torques
- Floating origin implementation
- May not depend on domain crates

**`crates/delta-v-assets/`**
- Custom asset loaders
- glTF 2.0 helpers
- May not depend on domain crates

**`crates/delta-v-net/`**
- Networking infrastructure (placeholder for M7)
- Currently a stub
- May not depend on domain crates

### Domain crates

These implement game features and may depend on technical crates and each other (acyclic):

**`crates/delta-v-ships/`** -- Ship types and ship logic.

**`crates/delta-v-propulsion/`** -- Thrust, hyperdrive, acceleration.

**`crates/delta-v-weapons/`** -- Projectiles, damage, targeting.

**`crates/delta-v-stations/`** -- Space stations, docking.

**`crates/delta-v-items/`** -- Collectables, inventory.

**`crates/delta-v-world/`** -- Sectors, boundaries, hyperspace gates.

## Dependency graph

Dependency flow (arrows point to dependencies):

```
binary (delta-v)
    ↓
    ├→ delta-v-core
    ├→ delta-v-config
    ├→ delta-v-physics
    ├→ delta-v-assets
    ├→ delta-v-net
    ├→ delta-v-ships       ↓
    ├→ delta-v-propulsion  ├→ delta-v-core
    ├→ delta-v-weapons     ├→ delta-v-config
    ├→ delta-v-stations    ├→ delta-v-physics
    ├→ delta-v-items       ├→ delta-v-assets
    └→ delta-v-world       └→ (no cycles)
```

**Key rules:**
1. Technical crates never depend on domain crates.
2. Domain crates never depend on each other (or acyclic only).
3. All shared surface between domains goes into `delta-v-core`.

## Shared dependencies

All dependency versions are defined in the root `Cargo.toml` under `[workspace.dependencies]`:

```toml
[workspace.dependencies]
bevy = { version = "0.14", default-features = true }
avian3d = "0.1"
serde = { version = "1.0", features = ["derive"] }
# ...
```

Member crates inherit them:

```toml
[dependencies]
bevy = { workspace = true }
serde = { workspace = true }
```

**Why?**
- Single point of version management
- Prevents accidental version conflicts
- Makes CI auditing easier

## Adding a new crate

1. Create the directory: `crates/my-crate/`
2. Create `crates/my-crate/Cargo.toml`:
   ```toml
   [package]
   name = "delta-v-my-crate"
   version = "0.0.1"
   edition = "2021"
   rust-version = "1.88"
   license = "GPL-3.0-or-later"
   
   [dependencies]
   bevy = { workspace = true }
   
   [lints]
   rust.unsafe_code = "warn"
   clippy.all = "warn"
   ```
3. Create `crates/my-crate/src/lib.rs` with the plugin stub
4. Add the crate to the root `Cargo.toml` `[workspace] members` array
5. Update `docs/architecture.md`

## Workspace lints

Linting is centralized in the root `Cargo.toml`:

```toml
[workspace.lints.rust]
unsafe_code = "warn"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
```

Member crates inherit by adding to their `Cargo.toml`:

```toml
[lints]
rust.unsafe_code = "warn"
clippy.all = "warn"
```

## Troubleshooting

### "could not compile `delta-v-ships`"

Check that the dependency is added to the crate's `Cargo.toml`, not just a sibling's:

```bash
cargo tree -p delta-v-ships
```

### "cycle detected when adding a dependency"

You've created a circular dependency. Use `cargo tree` to visualize:

```bash
cargo tree --duplicates
```

Fix by:
- Moving shared types to `delta-v-core`
- Splitting the crate
- Using an enum/trait to break the cycle

### "workspace member not found"

Ensure the crate is listed in the root `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/delta-v",
    "crates/delta-v-my-new-crate",  # ← add here
]
```

## Further reading

- [Cargo Book: Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [ADR-0002: Repository layout and workspace](adr/0002-repository-layout-and-workspace.md)
- [ADR-0005: Plugin architecture](adr/0005-plugin-architecture.md)
