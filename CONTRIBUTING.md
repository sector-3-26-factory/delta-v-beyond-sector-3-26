# Contributing to Delta-V beyond Sector 3.26

Thanks for your interest in contributing!

## Licensing of contributions

This project is licensed under **GPL-3.0-or-later**. By submitting a pull
request, an issue patch, or any other contribution, you agree that your
contribution is licensed under the same terms.

If you contribute third-party code or assets, make sure their license is
compatible with GPL-3.0-or-later and document the origin in the commit
message and, where appropriate, in a `LICENSES/` or `THIRD_PARTY.md` file.

## Development environment

The recommended setup is the Dev Container shipped in `.devcontainer/`. This
gives you a known-good Linux toolchain with all system dependencies Bevy
needs, without touching your host system.

## Coding guidelines

- **Language**: All code, comments, commit messages and documentation are
  written in **English**.
- **Formatting**: Run `cargo fmt --all` before committing.
- **Linting**: Run `cargo clippy --all-targets --all-features -- -D warnings`
  and fix or justify all warnings.
- **Tests**: Run `cargo test --all` where applicable.
- **Commit messages**: Use the imperative mood ("Add ship thruster system",
  not "Added" or "Adds"). Conventional Commits are encouraged but not
  mandatory.

## Pull requests

- Keep PRs focused; one logical change per PR.
- Reference the milestone (M0..M8, see `docs/design.md`) the change belongs to
  in the description.
- Make sure CI is green before requesting review.

## Reporting bugs / proposing features

Please open a GitHub issue with:

- A clear title.
- Steps to reproduce (for bugs) or a concrete use case (for features).
- Your environment (host OS, GPU, container vs. native).
