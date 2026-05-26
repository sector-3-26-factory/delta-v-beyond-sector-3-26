# Contributing to Delta-V beyond Sector 3.26

Thanks for your interest in Delta-V beyond Sector 3.26! 

To maintain a consistent architecture and ensure the project's long-term stability, **core code development is handled exclusively by the core organization team**. We do not accept external code contributions via Pull Requests. 

However, you can deeply impact the project in two ways: **Translations** and (coming soon) **Peer-to-Peer Content**.

---

## How You Can Contribute

### 1. Translations (Localization)
We want to make the game accessible to everyone. We highly welcome contributions to our UI texts and localization files.
- If you find a typo or want to translate the game into a new language, please check our open issues or open a new one with the tag `localization`.
- Translation files are stored under `assets/i18n/`, one JSON file per language (e.g. `en.json`, `de.json`). The English file is the reference; it defines the complete set of keys. See [ADR-0037](docs/adr/0037-internationalization.md) for the file format and rules.
- For a translation PR, add or edit only the JSON file for your language. The corresponding Rust struct and schema are maintained by the core team; if you spot a missing key in the reference file, open an issue rather than editing Rust code.

### 2. Peer-to-Peer Content (Future Feature)
Delta-V is designed to be modular. We are working on a Peer-to-Peer (P2P) ecosystem that will allow players to share custom content directly with each other without altering the game's core repository.
- **What can be shared:** Custom Worlds, Spaceships, and other game objects.
- **How it works:** This system is currently in development. Once live, you will be able to export your creations in-game and share them via the P2P network.

---

## Licensing of Contributions

This project is licensed under **GPL-3.0-or-later**. 

By submitting translations, bug reports, or suggestions, you agree that your contribution is licensed under the same terms. If your translation includes third-party text or assets, ensure their license is compatible with GPL-3.0-or-later.

---

## Reporting Bugs & Proposing Features

Even though we don't accept external code, your feedback is invaluable! Please open a GitHub issue if you find a bug or have a feature idea.

When opening an issue, please provide:
- A clear, concise title.
- Steps to reproduce (for bugs) or a concrete use case (for features).
- Your environment (host OS, GPU, container vs. native).

---

## Note for Core Team Members

If you are an invited member of the sector-3-26-factory organization with write/merge access, please adhere to our internal development guidelines:

- **Language:** All code, comments, commit messages and documentation are in **English**.
- **Formatting:** Run `cargo fmt --all` before committing.
- **Linting:** Run `cargo clippy --workspace --all-targets -- -D warnings` and fix all warnings (per [ADR-0034](docs/adr/0034-no-warnings-policy.md)).
- **Tests:** Run `cargo test --workspace` and make sure everything passes.
- **Commit Messages:** Conventional Commits format (per [ADR-0004](docs/adr/0004-commit-message-convention.md)).
- **Branching and PRs:** Gitflow-light (per [ADR-0003](docs/adr/0003-branching-and-pr-workflow.md)); see [`docs/workflow.md`](docs/workflow.md) for the practical recipe.
- **Project rules:** every architectural decision is recorded as an ADR under [`docs/adr/`](docs/adr/). Read the index in [`docs/adr/README.md`](docs/adr/README.md) before making non-trivial changes.