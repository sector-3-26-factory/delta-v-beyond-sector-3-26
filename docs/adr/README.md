# Architecture Decision Records

This directory holds the architecture decisions for
*Delta-V beyond Sector 3.26*.

If you have not yet read [`AGENTS.md`](../../AGENTS.md), do so first.

## Index

Numbering is monotonic; numbers are never reused. When an ADR becomes
obsolete it is superseded by a new ADR, but it remains in this index
with its original number for historical traceability.

| #    | Title                                                                     | Status   |
| ---- | ------------------------------------------------------------------------- | -------- |
| 0001 | [ADR process](0001-adr-process.md)                                        | Accepted |
| 0002 | [Repository layout and workspace](0002-repository-layout-and-workspace.md) | Accepted |
| 0003 | [Branching and PR workflow](0003-branching-and-pr-workflow.md)            | Accepted |
| 0004 | [Commit message convention](0004-commit-message-convention.md)            | Accepted |
| 0005 | [Plugin architecture](0005-plugin-architecture.md)                        | Accepted |
| 0006 | [Coordinate system and units](0006-coordinate-system-and-units.md)        | Accepted |
| 0007 | [Floating origin for large worlds](0007-floating-origin.md)               | Accepted |
| 0008 | [Physical units in JSON](0008-physical-units-in-json.md)                  | Accepted |
| 0009 | [Newtonian physics with gravity](0009-newtonian-physics-with-gravity.md)  | Accepted |
| 0010 | [Configuration system](0010-configuration-system.md)                      | Accepted |
| 0011 | [Keybindings configuration](0011-keybindings-configuration.md)            | Accepted |
| 0012 | [JSON schema validation](0012-json-schema-validation.md)                  | Accepted |
| 0013 | [No silent fallbacks](0013-no-silent-fallbacks.md)                        | Accepted |
| 0014 | [Engine constants vs gameplay values](0014-engine-constants-vs-gameplay-values.md) | Accepted |
| 0015 | [Logging strategy](0015-logging-strategy.md)                              | Accepted |
| 0016 | [Error handling strategy](0016-error-handling-strategy.md)                | Accepted |
| 0017 | [Fixed timestep and determinism](0017-fixed-timestep-and-determinism.md)  | Accepted |
| 0018 | [State management](0018-state-management.md)                              | Accepted |
| 0019 | [Asset pipeline and user content](0019-asset-pipeline-and-user-content.md) | Accepted |
| 0020 | [Save and load format](0020-save-and-load-format.md)                      | Accepted |
| 0021 | [Testing strategy](0021-testing-strategy.md)                              | Accepted |
| 0022 | [Performance instrumentation](0022-performance-instrumentation.md)        | Accepted |
| 0023 | [Code style and lints](0023-code-style-and-lints.md)                      | Accepted |
| 0024 | [Documentation policy](0024-documentation-policy.md)                      | Accepted |
| 0025 | [Versioning](0025-versioning.md)                                          | Accepted |
| 0026 | [Release process](0026-release-process.md)                                | Accepted |
| 0027 | [Open source licensing](0027-open-source-licensing.md)                    | Accepted |
| 0028 | [Third-party dependency policy](0028-third-party-dependency-policy.md)    | Accepted |
| 0029 | [Security and supply chain](0029-security-and-supply-chain.md)            | Accepted |
| 0030 | [Authoritative model](0030-authoritative-model.md)                        | Proposed |
| 0031 | [Network library choice](0031-network-library-choice.md)                  | Proposed |
| 0032 | [Snapshot and delta encoding](0032-snapshot-and-delta-encoding.md)        | Proposed |
| 0033 | [AGENTS.md and source file pointers](0033-agents-md-and-source-file-pointers.md) | Accepted |
| 0034 | [No warnings policy](0034-no-warnings-policy.md)                          | Accepted |
| 0035 | [Hot reload of configs in dev builds](0035-hot-reload-of-configs.md)      | Accepted |
| 0036 | [External human contributors process](0036-external-human-contributors-process.md) | Accepted |
| 0037 | [Internationalization (i18n)](0037-internationalization.md)               | Accepted |
| 0038 | [Entity template system](0038-entity-template-system.md)                  | Accepted |
| 0039 | [Enforcement of JSON-only defaults](0039-enforcement-of-json-only-defaults.md) | Accepted |
| 0040 | [delta-v-json for JSON validation](0040-delta-v-json-for-json-validation.md) | Accepted |
| 0041 | [Third-party asset acquisition and licensing](0041-third-party-asset-acquisition.md) | Accepted |

## How to add a new ADR

See [ADR-0001](0001-adr-process.md). In short:

1. Copy [`0000-template.md`](0000-template.md) to
   `NNNN-kebab-case-title.md` with the next free number.
2. Fill in Context, Decision, Consequences.
3. Start in status `Proposed`.
4. Open a pull request; on merge, change status to `Accepted` if
   consensus is reached.
5. Add the entry to the index table above in the same pull request.
