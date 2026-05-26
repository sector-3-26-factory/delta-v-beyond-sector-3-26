# M0.5 Documentation Index

**Milestone:** M0.5 - Architecture Foundation
**Status:** ✅ COMPLETE AND READY FOR MERGE
**Date:** 2025-05-19

## Start Here

### For Reviewers
1. **[READY-FOR-MERGE.md](READY-FOR-MERGE.md)** -- Merge readiness checklist (10 min read)
2. **[COMPLETION-REPORT.txt](COMPLETION-REPORT.txt)** -- Full status report

### For Developers
1. **[../../AGENTS.md](../../AGENTS.md)** -- Entry point and rules
2. **[../architecture.md](../architecture.md)** -- Architecture overview
3. **[../workspace-guide.md](../workspace-guide.md)** -- Workspace developer guide

### For Integration
1. **[COMPLETION-REPORT.txt](COMPLETION-REPORT.txt)** -- Full status report
2. **[READY-FOR-MERGE.md](READY-FOR-MERGE.md)** -- Merge instructions

## Main M0.5 Documentation

| Document | Purpose | Type |
|----------|---------|------|
| [READY-FOR-MERGE.md](READY-FOR-MERGE.md) | Merge readiness checklist | Markdown |
| [COMPLETION-REPORT.txt](COMPLETION-REPORT.txt) | Full status report | Text |
| [HANDOFF.txt](HANDOFF.txt) | Handoff document with next steps | Text |

## Development Guides (in docs/ root)

| Document | Purpose |
|----------|----------|
| [../architecture.md](../architecture.md) | Big-picture architecture |
| [../workspace-guide.md](../workspace-guide.md) | Workspace developer guide |
| [../workflow.md](../workflow.md) | Development workflow |
| [../roadmap.md](../roadmap.md) | Milestone roadmap |

## Architecture Decision Records

- **All 37 ADRs** in [`../adr/`](../adr/)
- **ADR Index** at [`../adr/README.md`](../adr/README.md)
- **Key M0.5 ADRs:**
  - [ADR-0002](../adr/0002-repository-layout-and-workspace.md) -- Repository layout
  - [ADR-0005](../adr/0005-plugin-architecture.md) -- Plugin architecture
  - [ADR-0023](../adr/0023-code-style-and-lints.md) -- Code style and lints
  - [ADR-0028](../adr/0028-third-party-dependency-policy.md) -- Dependency policy
  - [ADR-0029](../adr/0029-security-and-supply-chain.md) -- Supply chain
  - [ADR-0033](../adr/0033-agents-md-and-source-file-pointers.md) -- Source headers
  - [ADR-0034](../adr/0034-no-warnings-policy.md) -- No warnings policy

## Workspace Structure

**Binary crate:**
- `crates/delta-v/` -- Application composition

**Technical crates (5):**
- `crates/delta-v-core/` -- ECS fundamentals
- `crates/delta-v-config/` -- JSON loading
- `crates/delta-v-physics/` -- Physics integration
- `crates/delta-v-assets/` -- Asset loading
- `crates/delta-v-net/` -- Networking (stub)

**Domain crates (6):**
- `crates/delta-v-ships/` -- Ships
- `crates/delta-v-propulsion/` -- Propulsion
- `crates/delta-v-weapons/` -- Weapons
- `crates/delta-v-stations/` -- Stations
- `crates/delta-v-items/` -- Items
- `crates/delta-v-world/` -- World

**Total: 12 crates** (1 binary + 5 technical + 6 domain)

See [`../workspace-guide.md`](../workspace-guide.md) for details.

## Quick Reference

### Common Commands

```bash
# Build and run
cargo build --release
cargo run --bin delta-v

# Quality checks (all must pass)
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
./.githooks/pre-commit
```

### Key Files

```
Repository Root/
├── AGENTS.md                     # AI agent entry point (global)
├── README.md                     # Project overview (updated)
├── Cargo.toml                    # Workspace definition (updated)
├── deny.toml                     # cargo-deny config
├── .githooks/pre-commit          # Quality gate hook
├── crates/                       # 12 member crates
└── docs/
    ├── adr/                      # 37 ADRs
    ├── M0.5/                     # M0.5 documentation (this folder)
    ├── architecture.md           # Architecture overview
    ├── workspace-guide.md        # Developer guide
    └── ...
```

## Compliance Matrix

| Requirement | Status | Evidence |
|-------------|--------|----------|
| All ADRs implemented | ✅ | 37/37 Accepted |
| Workspace structure correct | ✅ | 12 crates with proper structure |
| Plugin architecture | ✅ | Each crate has one Plugin |
| No warnings | ✅ | cargo clippy -D warnings passes |
| Supply chain security | ✅ | deny.toml, cargo deny check |
| Source file headers | ✅ | "See AGENTS.md" in all files |
| Documentation complete | ✅ | M0.5 index + guides |
| Pre-commit hook | ✅ | .githooks/pre-commit active |

## Next Milestone: M1 -- A ship in space

**M1 will add:**
- 3D scene with ship mesh and chase camera
- Default world loading (JSON + glTF)
- Keybindings system
- Input handling
- Logging and diagnostics

**See:** [`../roadmap.md#m1----a-ship-in-space`](../roadmap.md) for details.

## Support and Questions

| Topic | Document |
|-------|----------|
| **Architecture** | [`../architecture.md`](../architecture.md) |
| **Workspace** | [`../workspace-guide.md`](../workspace-guide.md) |
| **Workflow** | [`../workflow.md`](../workflow.md) |
| **ADRs** | [`../adr/README.md`](../adr/README.md) |
| **Roadmap** | [`../roadmap.md`](../roadmap.md) |
| **AI Agent Rules** | [`../../AGENTS.md`](../../AGENTS.md) |

## Sign-off

✅ **M0.5 Architecture Foundation is COMPLETE and READY FOR INTEGRATION into `dev` branch.**
