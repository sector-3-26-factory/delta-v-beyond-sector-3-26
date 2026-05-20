# Contributing to Delta-V beyond Sector 3.26

Thanks for your interest in Delta-V beyond Sector 3.26! 

To maintain a consistent architecture and ensure the project's long-term stability, **core code development is handled exclusively by the core organization team**. We do not accept external code contributions via Pull Requests. 

However, you can deeply impact the project in two ways: **Translations** and (coming soon) **Peer-to-Peer Content**.

---

## How You Can Contribute

### 1. Translations (Localization)
We want to make the game accessible to everyone. We highly welcome contributions to our UI texts and localization files.
- If you find a typo or want to translate the game into a new language, please check our open issues or open a new one with the tag `localization`.
- Translation files are stored in `[/assets/locales/]`.

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

If you are an invited member of the sector-3.26-factory  organization with write/merge access, please adhere to our internal development guidelines:

- **Language:** All code, comments, commit messages, and documentation are in **English**.
- **Formatting:** Run `cargo fmt --all` before committing.
- **Linting:** Run `cargo clippy --all-targets --all-features -- -D warnings` and fix all warnings.
- **Tests:** Run `cargo test --all` where applicable.
- **Commit Messages:** Use the imperative mood ("Add ship thruster system"). Conventional Commits are encouraged.