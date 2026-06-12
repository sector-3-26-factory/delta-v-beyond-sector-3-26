# Delta-V beyond Sector 3.26

A spiritual successor to the classic space combat game
[Parsec / OpenParsec](https://github.com/OpenParsec/openparsec), rebuilt from
scratch in **Rust** with the **[Bevy](https://bevyengine.org/)** engine and
**Newtonian** flight physics.

> Status: **Pre-alpha (Milestone M4 - Weapons complete)**. Architecture foundation
> established. See [`docs/roadmap.md`](docs/roadmap.md) for the current milestone
> and [`AGENTS.md`](AGENTS.md) for development rules.

If you wonder about the name of the game you may take a look at
[`docs/the_name_of_the_game.md`](docs/the_name_of_the_game.md). Of course
nerds don't need to take a look :o)

## Goals

- Faithful in spirit to Parsec: fast, skill-based 6-DoF space combat.
- Newtonian physics: thrust applies force, inertia is preserved, there is no
  arbitrary top speed. An optional "flight assist" mode may be offered for
  accessibility.
- Modern, modular ECS architecture (Bevy).
- Cross-platform (Linux first, Windows/macOS later).
- Multiplayer as a first-class concern (target for a later milestone).

## Non-goals (for now)

- Photorealistic graphics.
- Persistent universe / MMO scope.
- Mobile platforms.

## Relationship to OpenParsec

This is a **clean-room reimplementation**. No source code is taken from
OpenParsec; only the gameplay ideas and feel serve as inspiration. The project
is nevertheless licensed under the GNU GPL v3 (or later) in the spirit of the
original.

## Building

### Recommended: Dev Container (VS Code)

The repository ships with a [Dev Container](.devcontainer/) configuration that
provides a reproducible Linux build environment with all system libraries
Bevy requires. This avoids polluting your host system.

1. Install [VS Code](https://code.visualstudio.com/) and the
   *Dev Containers* extension.
2. Open the project folder and choose **"Reopen in Container"**.
3. Inside the container:

   ```bash
   cargo run --bin delta-v
   ```

Graphical output is forwarded to the host via X11 (Linux host). See
[`.devcontainer/README.md`](.devcontainer/README.md) for details and
troubleshooting.

### Native build

If you prefer to build on the host, you will need a recent Rust toolchain
(see [`rust-toolchain.toml`](rust-toolchain.toml)) and the usual Bevy system
dependencies for your platform. See the
[Bevy setup guide](https://bevyengine.org/learn/quick-start/getting-started/setup/).

```bash
cargo run --bin delta-v
```

## Repository layout

```
.devcontainer/   Reproducible dev environment (Docker + VS Code)
.github/         CI workflows
.githooks/       Git hooks (pre-commit)
assets/          Game assets (models, textures, audio) -- empty for now
crates/          Workspace members (binary + 11 library crates)
docs/            Architecture Decision Records, design notes, roadmap
denied.toml      cargo-deny configuration (ADR-0028, ADR-0029)
AGENTS.md        Rules for AI agents working on this project
```

See [`docs/architecture.md`](docs/architecture.md) for an overview of the crate structure.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). By contributing you agree that your
contributions are licensed under GPL-3.0-or-later.

## License

Copyright (C) 2025 Cute-Donkey and contributors.

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version. See [`LICENSE`](LICENSE) for the full text.
