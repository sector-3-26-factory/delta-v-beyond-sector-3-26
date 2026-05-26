# ADR-0027: Open source licensing

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

The project is a spiritual successor to OpenParsec, which is GPL-
licensed. We have decided to keep the project open source under a
copyleft licence, both to match the spirit of the original and to
ensure that derivatives remain open.

In a game project there are also content questions: user-created
content (worlds, ships) shared peer-to-peer, third-party assets, and
the boundary between code and content.

## Decision

**Code**: All source code in this repository is licensed under
**GPL-3.0-or-later**. The full text lives in `LICENSE`. Source file
headers point at `AGENTS.md` (per
[ADR-0033](0033-agents-md-and-source-file-pointers.md)) which
references the project licence.

**Shipped assets** (under `assets/`, in this repository): licensed
under terms compatible with GPL-3.0-or-later. Where individual
assets carry their own licence (e.g. CC-BY-SA for an imported
model), that licence is recorded in a `LICENSES.md` or per-file
sidecar. Until any such asset is added, the default is "same as the
code".

**Third-party crates and tools**: must be compatible with
GPL-3.0-or-later (per
[ADR-0028](0028-third-party-dependency-policy.md)). The Steamworks
SDK is incompatible and is therefore explicitly out of the repo;
its potential use is discussed in
[ADR-0026](0026-release-process.md) and
[ADR-0031](0031-network-library-choice.md).

**User-created content shared peer-to-peer**: not licensed by this
project. Players who share content with one another are responsible
for the legal status of what they share. We do not host, distribute
or moderate user content from the project's repositories or
infrastructure. The game's loader treats user content as data; it
does not embed it into the game binary.

**Contributions**: by submitting a pull request or any other
contribution, a contributor agrees that their contribution is
licensed under GPL-3.0-or-later (per `CONTRIBUTING.md`).

## Consequences

Positive:

- Copyleft alignment with the project's intent.
- Players inherit the freedoms of the code; modifications must
  remain available under the same terms.
- The repository itself is free of any licence ambiguity.

Negative:

- GPL-incompatible third-party libraries (notably Steamworks) cannot
  be linked into the GPL-licensed binary in the usual way. We will
  address this via process separation or by accepting that a
  Steam-distributed build is a derivative governed by a separate,
  carefully scoped exception. ADR follows when the decision is
  needed.
- "Same as code" by default for assets must be revisited when we
  actually have notable assets.

Follow-up:

- A `LICENSES.md` (or `THIRD_PARTY.md`) is added once we begin
  importing third-party assets, listing each asset's source and
  licence.
- `cargo deny` (per
  [ADR-0028](0028-third-party-dependency-policy.md)) enforces the
  GPL-compatibility rule on the crate graph.
