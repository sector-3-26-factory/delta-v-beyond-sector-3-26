# AGENTS.md

This file is the canonical entry point for **AI agents** working in this
repository. It collects the rules they must respect when modifying code,
configuration or documentation.

> **If you are an AI agent reading this**: read this file completely
> before producing any change. Then read **EVERY SINGLE** Architecture Decision
> Record under `docs/adr/` before making **ANY** decision or change. 
> You do not decide which ADRs are relevant. **ALL ADRs are always relevant.**

Scope: this file is written for AI agents. The project accepts
external human contributions only in narrowly defined areas (notably
translations); the rules and exceptions are described in
`CONTRIBUTING.md` and governed by
[ADR-0036](docs/adr/0036-external-human-contributors-process.md).

---

## 1. Architecture Decision Records (ADRs) are binding

All architectural decisions for this project are recorded as ADRs in
[`docs/adr/`](docs/adr/). They are not suggestions; they are binding
constraints on every change.

Rules:

1. **READ THE ENTIRE ADR INDEX** at [`docs/adr/README.md`](docs/adr/README.md) first.
   Then **read every single ADR file listed there**. You do not filter, you do not
   prioritize, you do not skip. **Read them all.** The ADR index shows which ADRs exist.
   Treat the list as mandatory reading, not a menu.
2. **Honor every ADR in status `Accepted`.** If a change requires
   violating an Accepted ADR, the ADR must be superseded by a new ADR
   *first*, in a separate commit / pull request, before the change is
   made.
3. **ADRs in status `Proposed`** describe future intent; they are
   informational but not yet enforceable. New code should still align
   with their direction whenever practical.
4. **Never delete or rewrite an Accepted ADR.** Supersede it instead by
   adding a new ADR that references the old one and updating the old
   one's status to `Superseded by ADR-NNNN`.
5. **When you make a non-trivial architectural choice that is not covered
   by an existing ADR, create a new ADR** in status `Proposed` and
   reference it from your code/PR. Do not bury hidden decisions in code.

The ADR template lives in [`docs/adr/0000-template.md`](docs/adr/0000-template.md).
The ADR process itself is described in
[ADR-0001](docs/adr/0001-adr-process.md).

## 2. Every source file points here

Every source file we author (Rust, JSON schema, shell scripts, build
scripts, ...) that supports comments contains a one-line header
instructing agents to read this file. The header does **not** list
ADRs; it only points at `AGENTS.md`, which points at the ADR index.
The exact rule and per-language examples are in
[ADR-0033](docs/adr/0033-agents-md-and-source-file-pointers.md).

The chain is intentional:

```
source file header  ->  AGENTS.md  ->  docs/adr/
```

This indirection keeps source files stable when ADRs are added or
renumbered.

## 3. Workflow rules (summary)

Full rules are in [`docs/workflow.md`](docs/workflow.md). The minimum an
agent must know:

- **Branching**: Gitflow-light. `main` = stable releases. `dev` = active
  integration. Feature work happens on `feature/<slug>` branches off
  `dev` and is merged back into `dev` via pull request. See
  [ADR-0003](docs/adr/0003-branching-and-pr-workflow.md).
- **Commits**: Conventional Commits format. See
  [ADR-0004](docs/adr/0004-commit-message-convention.md).
- **No warnings**: A build that emits any warning must not be committed.
  See [ADR-0034](docs/adr/0034-no-warnings-policy.md).
- **No silent fallbacks**: Missing or invalid configuration is a hard
  error. See [ADR-0013](docs/adr/0013-no-silent-fallbacks.md).

## 4. Coding rules (summary)

- **Language**: All code, comments, commit messages and documentation are
  written in **English**.
- **Formatting**: Run `cargo fmt --all` before committing.
- **Linting**: `cargo clippy --workspace --all-targets -- -D warnings`
  must pass.
- **Tests**: `cargo test --workspace` must pass.
- **No `unsafe`** without an ADR or an inline comment that justifies it
  and is approved on review. See
  [ADR-0028](docs/adr/0028-third-party-dependency-policy.md) for the
  related dependency rule.
- **JSON defaults in schema only** (ADR-0012, ADR-0013, ADR-0040 **strictly enforced**):
  - ❌ No `#[serde(default = "...")]` on JSON-backed struct fields
  - ❌ No custom `fn default_X()` functions
  - ❌ No `Option<T>` for fields that always exist after schema validation
  - ✅ All defaults in `*.schema.json` only
  - ✅ Use the `delta-v-json` crate for all JSON loading (see [ADR-0040](docs/adr/0040-delta-v-json-for-json-validation.md))
  - See [ADR-0039](docs/adr/0039-enforcement-of-json-only-defaults.md) for
    code review checklist and rationale. Violations are code review failures.

## 5. AI output handling (remote environment)

The remote environment does not return command output directly to the agent.
Instead, use the following pattern:

**Setup (one-time):**
- Directory `.ai-tmp/` exists at project root
- `.ai-tmp/` is in `.gitignore` (already configured)

**Workflow:**
1. Run a command and redirect output to `.ai-tmp/`:
   ```bash
   cargo test --workspace > .ai-tmp/test-output.txt 2>&1
   ```
   **Important:** Redirection order matters! `> file 2>&1` (correct) vs `2>&1 > file` (wrong).
   - ✅ `> .ai-tmp/file.txt 2>&1` — stdout redirected first, then stderr joined to stdout
   - ❌ `2>&1 > .ai-tmp/file.txt` — stderr joined to stdout, then only stdout redirected (stderr lost!)
2. Read the file back with the `read_file` tool:
   ```
   read_file: .ai-tmp/test-output.txt
   ```

**Benefits:**
- Output files stay temporary and never commit to git
- `.gitignore` prevents git tracking but doesn't block agent reading
- Clear separation: `.ai-tmp/` = scratch space, project files = persistent

**Why this works:**
- `.gitignore` entries only prevent git from tracking files
- Agent `read_file` tool can read any file on disk, including those in `.gitignore`
- This avoids temp files accumulating in the project directory

**CRITICAL: Working directory**

The `run_terminal_command` tool **already executes in the correct repository root directory**.
Do NOT use `cd` commands to change directories.

**INCORRECT:**
```bash
cd /workspace && python3 script.py > .ai-tmp/out.txt 2>&1
```
This fails because:
- The actual working directory may not be `/workspace`
- Hardcoding paths breaks portability across machines and environments
- It adds unnecessary complexity

**CORRECT:**
```bash
python3 script.py > .ai-tmp/out.txt 2>&1
```
The command runs in the repository root automatically. Reference files relative to the current directory.

**CRITICAL: Terminal commands take time**

Many commands (`cargo test`, `cargo run`, `cargo build`, etc.) require a full build
that can take several minutes, especially the first time. 

**NEVER use `sleep` in terminal commands.** It is useless. YOU do not wait; the USER does.

**Agent behavior (mandatory):**
1. Run terminal command with output redirected: `command > .ai-tmp/file.txt 2>&1`
2. **ALWAYS use `> file 2>&1` order.** The `2>&1` MUST be at the end.
   - ✅ `cargo build > .ai-tmp/build.txt 2>&1` (correct)
   - ❌ `cargo build 2>&1 > .ai-tmp/build.txt` (wrong - stderr lost)
3. Issue the `run_terminal_command` and STOP immediately after the tool block closes.
4. Do NOT assume the command finished.
5. Do NOT provide a summary.
6. Do NOT move to the next task.
7. Wait for the next user message.
8. When user responds, read the output file with `read_file` tool.
9. If the file is empty or incomplete, ask the user to wait and try again.
10. Do NOT try to work around delays with `sleep`, polling loops, or workarounds.

This prevents misleading summaries and premature progress assumptions based on incomplete builds.

**CRITICAL: One tool call per response (ALL tools)**

Only issue **exactly one** tool call per agent response. This applies to ALL tools:
- `run_terminal_command`
- `edit_existing_file`
- `create_new_file`
- `read_file`
- `file_glob_search`
- `ls`
- `fetch_url_content`
- `view_diff`
- `single_find_and_replace`
- All other tools

The remote environment queues tool calls for user approval before execution. If multiple tool calls appear in the same response, all get queued simultaneously and shown for approval — which is confusing and error-prone. The user may cancel all but one, leaving the agent with incomplete output and an inconsistent mental model of what executed.

**Pattern (mandatory):**
1. Issue exactly one tool call per response
2. Stop immediately after the tool block closes
3. Do NOT assume the tool completed
4. Do NOT provide a summary or analysis
5. Do NOT move to the next agenda item
6. WAIT for the next user message
7. The user will tell you explicitly: "proceed", "continue", "next", or give you a new task
8. Only when the user explicitly says to proceed should you attempt the next item
9. Items in the pipeline (to-do list, staged tasks) are SUSPENDED until user approval
10. Never batch multiple tool calls in a single response
11. Never chain tasks together; each requires explicit user approval

**CRITICAL: Task switching**

When the user gives you a NEW task that supersedes the current one:
1. **Completely abandon the old task** from your mental model
2. **Do NOT talk about the old task anymore**
3. **Do NOT summarize what you did on the old task**
4. **Focus 100% on the new task you were given**
5. **Only when the user explicitly says "continue with [old task]" or similar, bring it back into scope**

Example: If you are implementing Phase 1 and the user says "fix AGENTS.md instead", then Phase 1 is dead. You never mention Phase 1 again until the user explicitly tells you to continue with it. You do not provide summaries like "Phase 1 is complete" or recap what Phase 1 did. Phase 1 does not exist in your response.

## 6. MANDATORY SEQUENCE: Read all ADRs before doing anything else

**NON-NEGOTIABLE: This sequence must be followed in order before you take any action.**

Before reading task descriptions, examining code, running commands, or using any tool, you MUST:

1. **IMMEDIATELY read `docs/adr/README.md`** to obtain the complete list of all ADRs.
2. **IMMEDIATELY read EVERY SINGLE ADR file listed in that index**, starting from ADR-0001 
   and reading sequentially through the last ADR (currently ADR-0042). 
   - Do not skip any ADR.
   - Do not filter or decide "that one is not relevant".
   - Do not read them out of order.
   - Read all of them.
3. **ONLY AFTER you have read every ADR from first to last** may you:
   - Read task descriptions
   - Examine project files
   - Run any terminal command
   - Use any tool
   - Respond to the user

**VIOLATION:** If you attempt to do any work before reading all ADRs in sequence, you have 
violated this binding constraint. There is no exception, no context where this is acceptable.

**VERIFICATION REQUIRED:** After reading the last ADR in the index, you MUST explicitly 
state in your response:

```
All ADRs read (ADR-0001 through ADR-NNNN).
```

Then STOP and WAIT for the user's confirmation before proceeding to any other task.

---

## 7. What to read after ADRs are complete

After you have verified that all ADRs are read, proceed with:

1. [`docs/workflow.md`](docs/workflow.md) -- branching, PRs, commits, CI.
2. [`docs/architecture.md`](docs/architecture.md) -- the big-picture
   architecture, which references the ADRs.
3. [`docs/roadmap.md`](docs/roadmap.md) -- the milestone roadmap.

When in doubt, ask in a pull request comment before writing code.

## 8. MANDATORY: Agents NEVER commit changes

**ABSOLUTE RULE: Under no circumstances may an agent execute `git commit`, `git push`, `git merge`, or any other git command that modifies the repository history or branches.**

Git operations are reserved exclusively for human developers. This includes:

- ❌ `git commit` (any variant)
- ❌ `git push`
- ❌ `git merge`
- ❌ `git rebase`
- ❌ `git cherry-pick`
- ❌ `git tag`
- ❌ `git branch -d` (or any branch deletion)
- ❌ `git reset` (any form of history rewriting)
- ❌ Any other git command that modifies history or branches

**The only git operations an agent may perform are read-only:**

- ✅ `git log` (viewing history)
- ✅ `git diff` (viewing changes)
- ✅ `git status` (checking state)
- ✅ `git show` (viewing commits)
- ✅ `git branch -l` (listing branches)

**After making code changes:**

1. Use the edit tools (`edit_existing_file`, `create_new_file`, etc.) to modify files.
2. The user will see the changes in their editor.
3. The user will review the changes.
4. The user will commit and push when satisfied.

**Why this rule exists:**

- Only humans decide which changes go into the repository and when.
- Only humans write commit messages and are accountable for what they claim in the message.
- Only humans decide on branching strategy and integration timing.
- Accidental agent commits can corrupt the repository state and blame history.

**VIOLATION:** If an agent attempts to run any git write command, it has violated this binding constraint. There is no exception, no context where this is acceptable.
