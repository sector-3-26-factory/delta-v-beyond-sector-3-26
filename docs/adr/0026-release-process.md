# ADR-0026: Release process

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

A version that exists only as a git tag is not actually released; a
player still needs a binary they can run. We need a documented path
from "commit on `main`" to "downloadable binary" and a plan for how
that path will evolve as the project grows.

## Decision

We release in stages, with the first stage already viable:

**Stage 1 -- GitHub Releases (current)**:

- A release is initiated by tagging on `main`
  (per [ADR-0025](0025-versioning.md)).
- A GitHub Actions workflow triggered by tags of the form `v*`
  builds the game for the supported platforms (initially Linux
  x86_64; Windows and macOS added when there is time and a tester
  with the hardware) and uploads the resulting artifacts as
  attachments to the GitHub Release.
- Release notes are generated from the Conventional Commits since
  the previous tag (per
  [ADR-0004](0004-commit-message-convention.md)). The maintainer
  edits the generated draft before publishing.
- Artifacts are deterministically named:
  `delta-v-beyond-sector-3-26-vX.Y.Z-<platform>-<arch>.<ext>`
  (`.tar.gz` on Unix, `.zip` on Windows).
- A `SHA256SUMS` file is uploaded alongside the artifacts.
- Source archives are GitHub's automatically generated `.tar.gz`
  and `.zip` (no separate signed source tarball at this stage).

**Stage 2 -- Steam (future)**:

- When the game is playable and we want a wider audience, we
  publish on Steam.
- Steam integration is mainly for distribution and for matchmaking
  / NAT traversal (see
  [ADR-0030](0030-authoritative-model.md) and
  [ADR-0031](0031-network-library-choice.md)).
- The Steam build is produced from the same source tree and the
  same version as the GitHub Release. We do not maintain Steam-only
  patches.
- Whether the Steam build links to the proprietary Steamworks SDK
  is a question that needs an ADR of its own when the time comes;
  the SDK's licence is not GPL-compatible, so the project may need
  to use it via a separate runtime mechanism.

**Stage 3 -- other channels** (Flathub, itch.io, distribution
packages): deferred, treated as out-of-scope until requested.

Always:

- The GitHub Release stays the canonical source for binaries. Other
  channels are mirrors or repackagings of it.
- Releases include a `LICENSE` file alongside the binary.
- The first time a release adds a new asset format or save format
  version, the release notes call that out explicitly.

## Consequences

Positive:

- The first release path is essentially free: GitHub gives us the
  hosting, the page, the download counts.
- Tag-driven automation removes manual steps when the maintainer
  is shipping.
- Stage 2 (Steam) is planned but not blocking; we can ship without
  it.

Negative:

- GitHub Releases is not a discovery channel; reaching players
  takes the Stage 2 work eventually.
- Cross-compiling from one CI runner to multiple platforms has
  edge cases (audio, Wayland, ...) that we will need to handle
  per-platform.
- Steamworks integration carries licence and architecture
  implications that we are deferring rather than solving.

Follow-up:

- The release workflow itself (`.github/workflows/release.yml`) is
  added when the first release is imminent.
- A separate ADR captures the decision when (and how) Steam
  integration lands.
