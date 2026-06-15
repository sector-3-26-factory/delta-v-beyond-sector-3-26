---
description: "ai driven review - prepare for human review"
---

This is a REPORTING-ONLY task. Agents MUST NOT apply fixes, make edits, or modify any files.
Output must be a structured report of violations found, not code changes.

**STOP AND WAIT FOR USER INSTRUCTIONS:**
After producing the report, STOP and wait for the user to read it.
NEVER proceed to fix any issues without explicit user instructions.
The user will specify which issues to fix, which to explain, or which to leave as-is.

Before starting the review, ask the user: "Should I review the whole codebase or only the uncommitted files in the current git branch?"

**ABSOLUTE PROHIBITION ON OPEN EDITORS:**
Agents MUST NEVER read, reference, or use VSCode open editor tabs for any purpose.
Open editor tabs may be stale and do not reflect the actual state of files.
The only authoritative sources are:
- `git diff` output (for uncommitted files review)
- `git ls-files` output (for whole codebase review)

**If reviewing uncommitted files only:**
1. Read `docs/adr/ARCHITECTURAL_RULES.md` in full
2. Use `git diff` to read changed code (MUST NOT use VSCode open editor tabs)
   - Run `git diff --name-only HEAD` to get the list of changed files
   - Output the list and confirm: "I will only review these X files from git diff"
   - Run `git diff HEAD` to read the actual changes
   - Focus only on files that appear in the git diff output
3. Run `bash scripts/check-forbidden-patterns.sh` to verify ADR-0013 compliance (no silent fallbacks via Default trait)
4. Check each changed file against all relevant ADR rules
5. OUTPUT: Present a structured violation report in your response - DO NOT write to any file

**If reviewing the whole codebase:**
1. Read `docs/adr/ARCHITECTURAL_RULES.md` in full
2. Use `git ls-files` to get the list of tracked files (MUST NOT use VSCode open editor tabs)
3. Run `bash scripts/check-forbidden-patterns.sh` to verify ADR-0013 compliance (no silent fallbacks via Default trait)
4. Check the whole codebase against all relevant ADR rules
5. OUTPUT: Present a structured violation report in your response - DO NOT write to any file

**WORKFLOW AFTER REPORT:**
After presenting the report, wait for user instructions. The user will specify actions like:
- "Issue 2: fix" - Agent applies the fix
- "Issue 3: explain" - Agent provides detailed explanation
- "Issue 5: fix" - Agent applies the fix

The agent will ONLY act on issues explicitly marked for action by the user.
Issues not explicitly marked for fixing will NOT be modified.

**ABSOLUTE PROHIBITIONS:**
- MUST NEVER read or reference VSCode open editor tabs under any circumstances
- MUST NOT use `read_file` on any file not in `git diff` output (for uncommitted review)
- MUST NOT use `apply_diff`, `write_to_file`, or any write operation
- MUST NOT run `cargo fmt`, `cargo clippy`, `cargo test`, or any build commands
- MUST NOT fix any violations found - only report them
- MUST NOT proceed after producing the report without explicit user instructions
