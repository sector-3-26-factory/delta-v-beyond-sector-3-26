# ARCHITECTURE COMPLIANCE RULES (STRICT)

<!-- AGENTS: before modifying this file, read AGENTS.md at the repository root. -->

This file is the condensed, machine-readable translation of all ADRs in `docs/adr/`.
All rules are binding. Accepted ADRs are enforced immediately. Proposed ADRs are
informational and must be aligned with.

---

[ADR-0001]: Every non-trivial architectural decision MUST be recorded as an ADR in `docs/adr/NNNN-kebab-case-title.md`. Use the template `docs/adr/0000-template.md`. Accepted ADRs are immutable in content (typos/links only). To change a decision, write a new superseding ADR and update the old one's status to `Superseded by ADR-NNNN`. Never delete or rewrite an Accepted ADR.

[ADR-0002]: All crates MUST live under `crates/` with the naming pattern `delta-v-<domain>`. No root-level packages. Every domain crate exports exactly one Bevy `Plugin`. All shared dependency versions MUST be pinned in `[workspace.dependencies]` at the workspace root and inherited via `workspace = true`. No per-crate ad-hoc versioning.

[ADR-0003]: All feature work happens on `feature/<slug>` branches off `dev`. PRs target `dev`. `main` is release-only. Branch names must be lowercase kebab-case prefixed by intent (`feature/`, `fix/`, `chore/`, `docs/`, `refactor/`). PRs must pass CI (`fmt`, `clippy`, `check`, `test`, `cargo-deny`) before merge.

[ADR-0004]: ALL commits MUST follow Conventional Commits 1.0.0 format: `<type>(<scope>): <imperative summary, lowercase, no trailing period>`. Allowed types: `feat`, `fix`, `refactor`, `perf`, `docs`, `test`, `build`, `ci`, `chore`, `style`, `revert`. Scope is the crate name without the `delta-v-` prefix or a top-level concern.

[ADR-0005]: Each domain crate MUST export exactly one Bevy `Plugin` (or `PluginGroup`). Plugin registration is ONLY done in the binary `crates/delta-v`. Cross-domain communication uses events or shared components — NEVER direct function calls between plugin internals. Plugins must be safe to add to a minimal `App` for tests.

[ADR-0006]: Coordinate system is right-handed, `+Y` up, `-Z` forward (Bevy/glTF 2.0 default). Engine units: meters, kilograms, seconds, radians. NEVER invent a different convention. JSON content may use other units (km, AU, etc.) but MUST convert to base units at load time per ADR-0008.

[ADR-0007]: ALWAYS use the floating-origin technique for position representation. Physics and rendering use `f32` relative to a movable origin. When player ship exceeds configured threshold from origin, recentre all entity Transforms in one system pass. NEVER use raw world-absolute `f32` positions for physics. NEVER adopt `f64` everywhere or hierarchical chunk coordinates without a new ADR.

[ADR-0008]: ALL numeric physical quantities in JSON MUST be expressed as `{"value": <number>, "unit": "<unit>"}`. The `unit` field is REQUIRED — missing `unit` is a schema validation failure. `value` must be finite (no NaN, no Infinity). Allowed units are defined in `assets/json/schema/units.schema.json`. NEVER use bare numbers for physical magnitudes in JSON.

[ADR-0009]: Physics simulation is Newtonian with mass-coupled dynamics. Every rigid body MUST have a `mass` (kg). Bodies designated as gravity sources generate `g = G * M / r²` on all bodies within a configured cutoff radius. Gravity contributions MUST be summed in a fixed, stable order (e.g. by entity id) for determinism. NEVER implement a "no gravity" shortcut or a top-speed cap. Mass values for all simulated bodies are REQUIRED by schema — missing mass is a hard error.

[ADR-0010]: Configuration uses a two-layer model: default layer (`assets/config/`) and user override layer (`$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/`). Deep-merge user layer on top of defaults (objects merged recursively, arrays replaced wholesale). Re-validate merged result against schema. Missing user file is fine. Invalid user file is a hard startup error. NEVER write to `assets/config/` at runtime. Use `delta-v-json` for all loading (ADR-0040).

[ADR-0011]: Default keybindings live at `assets/config/keybindings.json`. User overrides at `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/keybindings.json`. Gameplay code MUST reference logical actions only (e.g. `thrust_forward`) — NEVER raw keys. Schema at `assets/json/schema/keybindings.schema.json` is the canonical source of allowed actions. Missing defaults file is a hard error; missing user file is fine.

[ADR-0012]: EVERY JSON file the game reads MUST be governed by a JSON Schema (Draft 2020-12+) stored under `assets/json/schema/`. Rules: (1) One schema per file kind. (2) Set `"additionalProperties": false` at every object level. (3) All field defaults in schema via `"default"` — schema is the ONLY source of defaults. (4) Every field carries a `description`. (5) Validation runs at load time; failure aborts with file path + JSON pointer + reason. (6) Validate defaults files at startup — invalid defaults = release-blocking bug. (7) FORBIDDEN: `oneOf`, `anyOf`, `if/then/else`, `dependentSchemas` when branches declare different defaults, required fields, or property sets. Model variants with separate schema files per variant instead.

[ADR-0013]: The game MUST NEVER silently substitute a fallback for missing or invalid configuration/content. Missing required field = hard error. Schema validation failure = hard error. Invalid defaults file = startup abort. Invalid user override file = startup abort with clear pointer to file and field. In release builds, NEVER substitute placeholder meshes or textures. Schema `"default"` values are NOT silent fallbacks — they are explicit documented behavior applied in a separate load-time pass: (1) Parse JSON, (2) Validate, (3) Walk schema recursively inserting `"default"` for omitted fields, (4) Re-validate, (5) Deserialise into Rust structs. `serde` does NOT perform step 3.

[ADR-0014]: Engine constants (G, c, PI, unit conversions, tolerances, buffer sizes) MUST be `pub const` in Rust. Gameplay values (ship mass, weapon damage, keybindings, etc.) MUST come from JSON, governed by schemas. There is NO Rust-side default for gameplay values — missing value is a hard error. NEVER add a Rust-side default for any field that represents a gameplay knob. When a field appears unused to rustc because it is only written by serde, suppress with a per-field `#[allow(dead_code)]` with an inline comment — NEVER blanket `#![allow(dead_code)]`.

[ADR-0015]: Log levels: `ERROR` = cannot proceed, `WARN` = unexpected but continues, `INFO` = lifecycle events, `DEBUG` = developer detail (off by default in release), `TRACE` = per-frame spam (off unless explicit). Default level: `INFO` for our crates, `WARN` for third-party. `--features dev` builds default to `DEBUG` for our crates. Use `tracing` targets matching crate name (e.g. `delta_v_physics`). NEVER log PII, full home-directory paths, or secrets.

[ADR-0016]: Library crates (`delta-v-*`) MUST use typed errors declared with `thiserror`. The binary may use `anyhow` only at the outermost boundary. FORBIDDEN: `unwrap()` or `expect()` outside tests and `main`-level code without a `// SAFETY:` or `// INVARIANT:` comment explaining why it cannot fail. NEVER use `?` that discards context — use `.map_err(...)` or typed variants. Startup errors are fatal and verbose (file + field). Bevy systems return `()`; use logging wrappers or Bevy's `Result`-returning signatures.

[ADR-0017]: ALL physics, gameplay state, and AI systems MUST run inside Bevy's `FixedUpdate` schedule at 60 Hz. Rendering is decoupled. Simulation systems MUST NOT read wall-clock time (`Instant::now()`, `SystemTime::now()`). Iteration over unordered collections (`HashMap`, `HashSet`) in simulation paths MUST be replaced with ordered iteration. Gravity contributions MUST be summed in fixed order. RNG in simulation MUST use an explicit seeded `SimRng` ECS resource — NEVER thread-local randomness. Maximum catch-up ticks per frame is bounded (initial: 4).

[ADR-0018]: State management MUST use Bevy `States`. Each plugin scopes systems using `.run_if(in_state(...))`. State transitions happen through explicit events/resource changes only — NEVER set `NextState` from arbitrary systems. Do NOT pre-emptively model every imaginable sub-state; introduce sub-states only when behaviour actually diverges.

[ADR-0019]: Shipped assets live under `assets/` (game-owned, immutable at runtime). User content lives under `$XDG_DATA_HOME/delta-v-beyond-sector-3-26/` (user-owned). User content takes precedence over shipped content with the same logical identifier. NEVER write to `assets/` at runtime. NEVER modify the user data root from the installer or application update. 3D content: glTF 2.0 only. Data content: JSON validated by schemas. Asset lookup MUST consult both roots via the central loader.

[ADR-0020]: Structured game state is saved as JSON, validated against schemas. 3D player-created content as glTF 2.0. Saves MUST reference the game version they were written by. Floating-point values written with full round-trip precision. Identifiers inside saves are stable strings, not raw integer indices. Saves are written atomically (temp file → fsync → rename). NEVER write saves into `assets/`.

[ADR-0021]: Unit tests live in sibling `_tests.rs` files (e.g. `foo_tests.rs`), wired in via `#[cfg(test)] #[path = "foo_tests.rs"] mod tests;`. NEVER mix test code with production code in the same file. Integration tests in `tests/` directory per crate, using real fixture files from `tests/fixtures/`. Every public function with non-trivial contract carries a rustdoc example (run as doctest). Tests MUST NOT depend on execution order or shared mutable global state.

[ADR-0022]: Every non-trivial Bevy system MUST be wrapped in a `tracing::info_span!` or use `#[instrument]` with a target matching the crate name. A diagnostic plugin MUST emit a `WARN` log when frame time exceeds 33 ms for more than 60 consecutive frames. Frame-time threshold is JSON-configurable. NEVER collect end-user telemetry. NEVER write profiling data to disk in shipping builds.

[ADR-0023]: Run `cargo fmt --all` with the workspace `rustfmt.toml` before every commit. Run `cargo clippy --workspace --all-targets -- -D warnings` and it MUST pass. Each crate's `lib.rs`/`main.rs` MUST include the curated lint set: `#![warn(missing_docs, rust_2018_idioms, unreachable_pub, clippy::all, clippy::pedantic, clippy::cargo)]` and `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented, clippy::dbg_macro)]`. Test code may relax `deny`s with localised `#[allow(...)]` inside `#[cfg(test)]` only. Naming: types `UpperCamelCase`, functions `snake_case`, constants `SCREAMING_SNAKE_CASE`, files `snake_case.rs`.

[ADR-0024]: Every `pub` item in every library crate MUST have at least a one-line rustdoc comment. Public functions with non-trivial contracts carry a documented example (run as doctest). Crate root `lib.rs` has a summary paragraph. Every JSON Schema field carries a `description`. The lint `#![warn(missing_docs)]` is enabled in every library crate. Per ADR-0034 (no warnings), missing docs blocks a commit.

[ADR-0025]: All commits follow Semantic Versioning 2.0.0. Pre-1.0: `0.MINOR.PATCH`. All workspace crates share one version bumped together via workspace inheritance. Tags are annotated only (`git tag -a vX.Y.Z`), on `main` only, named `vX.Y.Z`. NEVER tag `dev`. The binary reports `env!("CARGO_PKG_VERSION")` on startup.

[ADR-0026]: Releases are triggered by tagging `main`. GitHub Actions builds and uploads artifacts named `delta-v-beyond-sector-3-26-vX.Y.Z-<platform>-<arch>.<ext>`. A `SHA256SUMS` file is uploaded alongside artifacts. Release artifacts MUST include a `LICENSE` file. The GitHub Release is the canonical binary source.

[ADR-0027]: All source code is licensed GPL-3.0-or-later. All shipped assets must be GPL-3.0-or-later compatible. NEVER link GPL-incompatible libraries (e.g. Steamworks SDK) into the binary without a separate ADR. Third-party crate licences are enforced by `cargo deny`. By submitting a PR, contributors licence their contribution under GPL-3.0-or-later.

[ADR-0028]: A crate may be added as a dependency ONLY when: (1) licence is GPL-3.0-or-later compatible, (2) actively maintained, (3) no `unsafe` in our own code without inline justification and a test, (4) carries its weight. All deps MUST be pinned in `[workspace.dependencies]` and inherited via `workspace = true`. `cargo deny check` MUST pass in CI. `git = "..."` dependencies on `main`/`dev` require an ADR. `Cargo.lock` is committed.

[ADR-0029]: `Cargo.lock` is committed; CI builds use `--locked`. Rust toolchain is pinned via `rust-toolchain.toml`. `cargo deny check` runs in CI enforcing: licence allow-list, RUSTSEC advisories, source restrictions. A scheduled weekly CI run re-runs `cargo deny` against `dev`. NEVER commit secrets, tokens, keys, or credentials. `unsafe` in our code requires an ADR or inline rationale comment plus a test.

[ADR-0033]: EVERY source file we author that supports comments MUST contain a one-line header pointing at AGENTS.md: `// AGENTS: before modifying this file, read AGENTS.md at the repository root.` (or language-appropriate equivalent). JSON Schemas use the top-level `description` field for this pointer. Plain JSON content files without a `description` slot are exempt. The pointer MUST NOT name individual ADRs — only point at AGENTS.md.

[ADR-0034]: ZERO warnings policy. `cargo build --workspace --all-targets` MUST emit zero warnings. `cargo clippy --workspace --all-targets -- -D warnings` MUST pass. CI sets `RUSTFLAGS="-D warnings"`. Any unavoidable warning MUST be suppressed with the NARROWEST possible `#[allow(...)]` WITH an inline comment explaining why. Blanket `#![allow(...)]` at crate level requires an ADR amendment. `cargo deny check` warnings also count as violations.

[ADR-0035]: Hot-reload of config files is ONLY enabled under `--features dev`. In release builds, configuration is loaded once at startup. A reload that fails schema validation MUST NOT apply — previous valid config remains, ERROR logged with file and field. Hot-reload applies ONLY to safe-to-swap config (keybindings, language, audio mix, gameplay tuning). NEVER hot-reload world definitions, schema files, or engine-startup config.

[ADR-0036]: The project does NOT accept external human contributors for code or architecture changes. Translation files (`assets/i18n/<language>.json`) are the ONLY accepted external human contribution. Translators MUST NOT touch Rust code or schemas. If a missing/unclear key blocks a translator, they open an issue. AGENTS.md remains scoped to AI agents only — human contributors are pointed at CONTRIBUTING.md.

[ADR-0037]: All user-visible strings MUST be accessed through a typed Rust struct hierarchy (e.g. `i18n.ui.menu.help`). NEVER use runtime string-key lookups (`tr!("ui.menu.help")`). Translation files live at `assets/i18n/<language>.json`. The English file `en.json` is the REQUIRED reference — missing key in `en.json` is an error. All supported language files MUST contain all keys from the reference file — missing key in any language file is a hard error (ADR-0013). Adding a key requires updating schema, English reference file, AND Rust struct in lockstep. Language selection uses the config system (ADR-0010). Fluent/ICU/Gettext are NOT adopted for existing strings; only introduced alongside a future feature that genuinely needs rich grammar, scoped to that feature only.

[ADR-0038]: Entity templates are JSON files under `assets/templates/<category>/`, each describing ONE entity type with a required `entity_type` discriminator field. Each entity type has its OWN schema file (`assets/json/schema/<entity_type>.schema.json`). World definitions reference templates by path — NEVER inline full entity definitions. Loading sequence: (1) load+validate world JSON, (2) for each entity load+validate its template against its schema, (3) emit `SpawnEntity` event, (4) domain plugins listen and spawn. Spawn ordering is managed via Bevy `SystemSet` ordering — NOT by the loader. NEVER use `oneOf`/`anyOf`/`if-then-else` in schemas (per ADR-0012 rule 8).

[ADR-0039]: STRICTLY FORBIDDEN in all JSON-backed Rust structs: (1) `#[serde(default = "fn_name")]` attributes, (2) custom `fn default_X() -> T` functions, (3) `Option<T>` for fields that always exist after schema validation. REQUIRED: all defaults in `*.schema.json` only. Use `Option<T>` ONLY if the field is logically optional AND the schema has no default for it. All PRs touching JSON deserialization MUST pass the checklist: no serde defaults, no custom default fns, no unnecessary Options, all defaults in schema, struct fields match schema (required↔non-Option, optional↔Option), test covers missing required field producing hard error. Violations are code review failures — do not merge.

[ADR-0040]: The `delta-v-json` crate is the ONLY standard library for all JSON loading. EVERY schema and loader MUST use `delta-v-json` to: validate against schema, fill defaults from schema, re-validate, deserialise into Rust structs. ABSOLUTELY FORBIDDEN: `#[serde(default = "fn_name")]` on struct fields, `impl Default` for types loaded from JSON, custom `fn default_X()` functions. Call `delta_v_json::load::<T>(schema, json_string)` or equivalent — the schema is the only authority. Error messages from `delta-v-json` are propagated directly to users unchanged.

[ADR-0041]: Before downloading ANY third-party asset: (1) verify licence is GPL-3.0-or-later compatible (accept: CC0, CC BY 4.0, CC BY-SA 4.0, MIT, Apache 2.0, GPL; reject: CC BY-NC, CC BY-ND, proprietary); (2) check for copyright/trademark red flags (franchise names, "fan art", "ripped from game" — reject immediately if found). 3D assets MUST be in glTF 2.0 format placed at `assets/templates/<type>/<name>/mesh.glb`. Every integrated asset MUST have a CREDITS.md entry with creator, source URL, licence, licence URL, file path, and modifications. NEVER commit CREDITS.md changes without explicit human approval.

[ADR-0043]: Ship templates use a two-level pattern: base ship templates (`entity_type: "ship"`) define common properties (mass, inertia_scale, propulsion) with an implicit mesh at `mesh.glb` in the template directory. Player-controlled ship templates (`entity_type: "player_controlled_ship"`) reference a base ship template via `ship_template` and add camera definitions. The template loader merges the two at load time. The `entity_type` is derived from the template, not from the world definition. Switching ships requires changing only the `ship_template` reference.

[ADR-0044]: NO visual data generation or handling within Rust code. All visual data (textures, meshes, shaders, render targets, materials, animations) MUST come from external files loaded via the asset pipeline. 3D models MUST be glTF 2.0 (`.glb` files) in `assets/templates/<type>/<name>/mesh.glb`. Domain plugins use the spawn/marker/attachment pattern: spawn systems queue glTF loading, marker components track loading state, attachment systems attach loaded scenes in `InGame` state. This ensures separation of concerns between simulation and rendering, faster compile times, smaller binaries, and proper asset attribution. **Exceptions**: (1) Debug/diagnostic visual data (debug axes, collision visualization) is exempt. (2) Camera setup and render layer configuration (e.g. `RenderLayers`, camera `order`, `UiSourceCamera`) are infrastructure concerns, not visual content creation, and are exempt. (3) UI infrastructure types from third-party crates (e.g. `bevy_lunex` `UiLayout`, `Dimension`, `UiSourceCamera`) used in domain crates for layout and rendering hooks are exempt.

[ADR-0045]: Anti-cheat measures for client-side JSON configuration: (1) Schema validation with strict bounds on gameplay values (mass, thrust, collision shapes). (2) Collision shape containment validation (shape must fit within bounding box). (3) Server-authoritative game state in multiplayer (host validates all client inputs). (4) Optional cryptographic signing for official templates. (5) Hard errors for validation failures (ADR-0013). This provides defense-in-depth against cheating via JSON modification.

[ADR-0046]: The `delta-v-types` crate is the canonical source for shared domain types. ALL plain data types and serde deserialization structs used by 2+ crates MUST live in `delta-v-types`. This crate MUST NOT depend on any other `delta-v-*` crate. It MUST NOT contain systems, plugins, Bevy resources, or Bevy `Component` derives — only plain types and serde structs. `delta-v-types` MAY depend on `serde` and `bevy` (math types only: `Vec3`, `Quat`). JSON deserialization value types MUST use the `Json` suffix (e.g., `Vec3Json`, `QuatJson`, `CollisionShapeJson`). The corresponding runtime type MUST NOT use the `Json` suffix. FORBIDDEN: defining shared types in any crate other than `delta-v-types`. FORBIDDEN: per-entity-type collision shape variants — use `CollisionShapeJson` for ALL entity types.

[ADR-0047]: ALL entity spawning follows the centralized pattern. Every domain crate that spawns entities MUST have a `src/spawn.rs` module. Spawn systems are named `spawn_<entity_type>`. Mesh attachment functions are named `attach_<entity_type>_meshes`. Pending mesh markers are named `Pending<EntityType>Mesh`. Domain spawners MUST use `delta-v-spawn` utilities for template extraction, collision shape conversion, and mesh attachment. FORBIDDEN: manually parsing JSON template fields in domain spawners — use `delta-v-spawn::template_extraction`. FORBIDDEN: duplicating mesh attachment logic — use `delta-v-spawn::mesh_attachment`. FORBIDDEN: naming spawner files anything other than `spawn.rs` (e.g., `asteroid_spawner.rs` is forbidden).

[ADR-0048]: `delta-v-core` uses a subdirectory structure. Modules are organized by concern: `state/`, `events/`, `input/`, `camera/`, `debug/`, `diagnostics/`, `boundary/`, `flight_assist/`, `floating_origin/`, `spawn/`. Each subdirectory has a `mod.rs` and optionally `tests.rs`. FORBIDDEN: adding new files directly to `delta-v-core/src/` without placing them in the appropriate subdirectory. Debug code goes in `debug/`, NOT at the crate root.

[ADR-0049]: Template loading is split between two crates with clear responsibilities. `delta-v-json` owns the JSON pipeline (read, validate, fill defaults, deserialize). `delta-v-assets` owns asset path resolution, template loading, and template merging. FORBIDDEN: loading templates outside of `delta-v-assets`. FORBIDDEN: merging templates in domain crates — use `delta-v-assets`. `delta-v-world` uses `delta-v-assets` for template loading. `delta-v-config` uses `delta-v-json` directly for simple config files.

[ADR-0050]: Strict naming conventions for crates and modules. Spawning logic: `src/spawn.rs` with `spawn_<entity_type>` systems. Components: no suffix (e.g., `FlightAssist`). Resources: no suffix (e.g., `WorldDefResource`). Events: past tense or imperative (e.g., `SpawnEntity`). Test files: `_tests.rs` suffix. FORBIDDEN: `asteroid_spawner.rs`, `station_spawner.rs`, or any spawner filename other than `spawn.rs`. FORBIDDEN: `spawn_station_system` — use `spawn_station`. Every domain crate uses the standard module structure: `lib.rs`, `components.rs`, `systems.rs`, `spawn.rs`, `resources.rs`, `error.rs`.

[ADR-0051]: The crate architecture follows a strict dependency hierarchy. Technical crates (`delta-v-json`, `delta-v-types`, `delta-v-spawn`, `delta-v-assets`) MUST NOT depend on domain crates. `delta-v-core` is a **foundation crate** (not a domain crate) — domain crates MAY depend on `delta-v-core` and on technical crates. Domain crates MUST NOT depend on other domain crates (use events/components for cross-domain communication per ADR-0005). `delta-v-core` MUST NOT depend on domain crates. The binary `delta-v` is the only crate that registers plugins. See the crate architecture decision tree in ADR-0051 for "where does this go?" guidance.

---

<!-- Proposed ADRs - informational only, not strict rules -->
[ADR-0030] (Proposed): Multiplayer model is peer-to-peer with "player as server" semantics (host is authoritative simulator). Design physics, state, and input NOW so that the wire-side surface is a small, well-defined module (`delta-v-net`) swappable without touching the rest. Do NOT implement lockstep simulation. Until M7, `delta-v-net` is a stub only.

[ADR-0031] (Proposed): Do NOT commit to a network library yet. Keep `delta-v-net` as a stub. Ensure architectural decisions (fixed timestep, plugin isolation, no wall-clock in simulation) keep all candidates (lightyear, bevy_replicon, bevy_quinnet) viable. Steam MUST be a transport option behind a feature flag, NOT the only matchmaking story.

[ADR-0032] (Proposed): Replicated components MUST be explicitly marked — no "all components replicated" default. Each replicated component declares its fields and quantisation. Positions on the wire are in a stable sector-relative frame (not per-client floating-origin frame). Snapshots are reliable; per-tick deltas are unreliable but timestamped and idempotent.

---

## Summary of Absolute Prohibitions

- ❌ No `#[serde(default)]` or `fn default_X()` on JSON-backed structs
- ❌ No `Option<T>` for fields that always exist after schema validation
- ❌ No `unwrap()`/`expect()` without `// SAFETY:` or `// INVARIANT:` comment (outside tests)
- ❌ No `oneOf`/`anyOf`/`if-then-else` in schemas when branches differ in defaults/required fields
- ❌ No raw wall-clock time in simulation systems
- ❌ No bare physical numbers in JSON (must use `{"value": x, "unit": "y"}`)
- ❌ No silent fallbacks for missing/invalid config or content
- ❌ No writing to `assets/` at runtime
- ❌ No GPL-incompatible dependencies without a dedicated ADR
- ❌ No commits that introduce warnings
- ❌ No `git commit`/`push`/`merge` by agents (humans commit only)
- ❌ No cross-domain function calls between plugin internals (use events/components)
- ❌ No runtime string-key translation lookups
- ❌ No CREDITS.md commits without explicit human approval
- ❌ No visual data generation or handling in Rust code (textures, meshes, shaders, render targets, materials, animations) **except**: (1) debug/diagnostic visualizations, (2) camera setup and render layer configuration (`RenderLayers`, camera `order`, `UiSourceCamera`), (3) UI infrastructure types from third-party crates (e.g. `bevy_lunex` layout/rendering types in domain crates)
- ❌ No shared types outside `delta-v-types` (including `Vec3Json`, `QuatJson`, `CollisionShapeJson`, `BoundingBox`, `PhysicalQuantity`)
- ❌ No `ShipCollisionShape` or per-entity-type collision shape variants — use `CollisionShapeJson` for ALL entity types
- ❌ No spawner files named anything other than `spawn.rs` (e.g., `asteroid_spawner.rs`)
- ❌ No domain-to-domain crate dependencies (use events/components) — **Note:** `delta-v-core` is a foundation crate, NOT a domain crate; domain crates MAY depend on `delta-v-core`
- ❌ No loading templates outside `delta-v-assets`
- ❌ No flat files in `delta-v-core/src/` — use subdirectories
- ❌ No `CollisionShape` (Bevy Component) in `delta-v-types` — it stays in `delta-v-physics`
- ✅ Domain crates MAY depend on `delta-v-physics` for `RigidBody`/`CollisionShape` components (exception to ADR-0051)
- ✅ Domain crates MAY depend on `delta-v-core` for foundation types and systems (exception to ADR-0051 — `delta-v-core` is a foundation crate, not a domain crate)
- ❌ No JSON deserialization value types without the `Json` suffix in `delta-v-types`
