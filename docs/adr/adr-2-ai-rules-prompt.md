# ADR-to-Rules Transformation Prompt

> **Purpose**: This file is the instruction recipe for transforming ADRs
> (Architecture Decision Records) from `docs/adr/` into the condensed,
> machine-readable `ARCHITECTURAL_RULES.md`. It is **not** an ADR itself.
>
> **When to use**: Run this prompt whenever a new ADR is added or an
> existing ADR is updated, to regenerate `ARCHITECTURAL_RULES.md`.
>
> **Why it lives in `docs/adr/`**: It is stored alongside the ADRs it
> operates on, so the transformation can be re-triggered by any agent
> with access to the ADR directory. The `adr-2-` prefix in the filename
> stands for "ADR-to-Rules" (not an ADR number).

You are an AI infrastructure expert. You are given our software
architecture documents (ADRs). Your task is to completely remove the
prose (discussions, pros/cons, history) and translate each ADR into an
ultra-concrete, machine-readable, and unambiguous behavioral rule for a
coding agent. Use the format:
`[ADR-XXXX]: [Strict instruction/prohibition]`.

An example of what the result must look like for your described rules:

```markdown
# ARCHITECTURE COMPLIANCE RULES (STRICT)

[ADR-0002]: Respect repository structure. Crates MUST only be placed in
the designated paths [/crates/...]. No root-level packages without approval.

[ADR-0012]: ABSOLUTE FORBIDDEN to hardcode defaults or fallbacks in Rust
code. Configuration defaults must exclusively exist in the JSON schema.

[ADR-0013]: Any configuration loader MUST look for overrides in the user's
home directory (`~/.config/myapp/`) first.

[ADR-0014]: Always use the existing `shared_defaults` crate to resolve
config values. Re-implementing JSON-schema parsing for defaults is strictly
prohibited.
```

Save this condensed list in a new file `ARCHITECTURAL_RULES.md` at
`[project-root]/docs/adr/`.

The ADRs are located under `docs/adr/`.