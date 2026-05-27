# M1 — A Ship in Space: Detailed Implementation Plan

> **Branch**: `feature/milestone_M1` off `dev`
> **ADRs governing this milestone**: 0005, 0006, 0007, 0008, 0009, 0010, 0011, 0012, 0013, 0014, 0015, 0016, 0017, 0018, 0019, 0020, 0021, 0022, 0023, 0024, 0033, 0034, 0035
> **Definition of done**: `cargo run` opens a 3-D window showing a primitive ship mesh in a minimal sector, a chase camera follows it, keybindings load from JSON, all input is logged (not acted on), frame-time diagnostics run, and `cargo test --workspace` passes with zero warnings.

---

## 0. Reading order for the implementing agent

Before writing a single line of code, read in this order:

1. `AGENTS.md` (already done if you are reading this)
2. `docs/adr/README.md` — confirm every ADR listed above is `Accepted`
3. `docs/architecture.md` — plugin composition map, data flow
4. `docs/workflow.md` — branching, commit format, pre-commit checks
5. The ADR documents listed in the header above, in numerical order
6. Every existing `src/lib.rs` in each crate to understand the current stub surface

All decisions in this plan are derived from those documents. Where this plan says "per ADR-NNNN", the ADR is the authority; this plan is the interpretation.

---

## 1. Milestone scope

M1 delivers exactly these observable outcomes:

| # | Observable outcome |
|---|--------------------|
| M1-O1 | `cargo run` opens a 1280×720 window with a 3-D scene |
| M1-O2 | A primitive ship mesh (Bevy `PbrBundle` with a capsule or box `Mesh`) is visible in the scene |
| M1-O3 | A chase camera (3rd-person, offset behind and above the ship) follows the ship entity |
| M1-O4 | A default world definition is loaded from `assets/worlds/default.world.json` at startup (validated against its JSON schema) |
| M1-O5 | Keybindings are loaded from `assets/config/keybindings.json`; an optional user override is merged from the XDG config path |
| M1-O6 | Every logical input action (thrust, pitch, yaw, roll, strafe) is received and logged at `DEBUG` level; no physics forces are applied yet |
| M1-O7 | Frame-time diagnostics plugin is active; a `WARN` is emitted when frame time exceeds the configured threshold for N consecutive frames |
| M1-O8 | `tracing`/`log` initialisation follows ADR-0015: `INFO` for our crates in release, `DEBUG` in `--features dev`; crate-name targets |
| M1-O9 | `cargo test --workspace` passes; `cargo clippy --workspace --all-targets -- -D warnings` passes; zero compiler warnings |
| M1-O10 | Every new source file carries the AGENTS.md header comment (ADR-0033) |

Explicitly **out of scope** for M1:
- Physics forces / torques (M2)
- Collision geometry (M3)
- Floating origin activation (M3, though the plugin stub must exist)
- HUD (M6)
- Any multiplayer code (M7)
- Save/load (beyond world definition loading)

---

## 2. Work breakdown

The work is divided into seven tracks that can be read as a rough sequential order, though tracks 2–5 can proceed in parallel once track 1 is complete.

```
Track 1 — Foundation & state machine         (prerequisite for all others)
Track 2 — Configuration & keybindings loader
Track 3 — World definition schema & loader
Track 4 — Ship entity & 3-D scene
Track 5 — Chase camera
Track 6 — Input logging
Track 7 — Logging init & diagnostics plugin
```

### Track 1 — Foundation & state machine

**Goal**: wire up Bevy `States` (ADR-0018) so every subsequent plugin can scope its systems to the correct app phase. Replace the current `Camera2dBundle` stub in `main.rs` with the proper state-driven startup.

#### 1.1 Define `AppState` in `delta-v-core`

File: `crates/delta-v-core/src/state.rs`

```rust
// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Application-level state machine.
//!
//! See ADR-0018 (State management).

use bevy::prelude::*;

/// Top-level application state.
///
/// Transitions follow the path:
/// `Boot -> LoadingDefaults -> LoadingWorld -> InGame`
/// for the default development flow. `MainMenu` and `Paused` are
/// reserved for later milestones.
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    /// Initial frame; no plugin has completed setup yet.
    #[default]
    Boot,
    /// Loading default world and configuration files.
    LoadingDefaults,
    /// World JSON has been parsed; entities are being spawned.
    LoadingWorld,
    /// Simulation is running; player has control.
    InGame,
}
```

Export from `delta-v-core/src/lib.rs`:
```rust
pub mod state;
pub use state::AppState;
```

Add `init_state::<AppState>()` inside `CorePlugin::build`.

#### 1.2 Update `main.rs`

- Remove the `Camera2dBundle` stub from the `setup` system.
- Remove the bare `setup` system registration.
- Logging is now initialised by `CorePlugin` (see Track 7).
- The binary remains responsible only for plugin composition (ADR-0005).

#### 1.3 State-transition logging

Inside `CorePlugin::build`, register a system that runs `OnEnter` each state variant and emits `info!("AppState -> {:?}", state)` (ADR-0015, ADR-0018).

---

### Track 2 — Configuration & keybindings loader

**Goal**: implement the two-layer config system (ADR-0010) for keybindings (ADR-0011) inside `delta-v-config`. All gameplay code reads logical actions, never raw keys.

#### 2.1 New dependencies for `delta-v-config`

Add to `Cargo.toml` (workspace-pinned):
- `serde` + `serde_json` (already in workspace)
- `jsonschema` — JSON Schema Draft 2020-12 validator
- `directories` — XDG / platform config path resolution (ADR-0010, ADR-0019)
- `thiserror` — typed errors (ADR-0016)

All versions pinned in `[workspace.dependencies]` and inherited via `workspace = true`.

#### 2.2 JSON Schema: `assets/json/schema/keybindings.schema.json`

Create this file. Key requirements (ADR-0011, ADR-0012):

- Top-level `description` carries the AGENTS pointer (ADR-0033).
- `"additionalProperties": false` at every object level (ADR-0012, rule 2).
- Every field has a `description` (ADR-0012, rule 4).
- No `oneOf`/`anyOf`/`if-then-else` with branch-dependent defaults (ADR-0012, rule 8).
- Physical unit objects are **not** needed here (keybindings are not physical quantities).

Minimal schema shape:
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://sector-3-26-factory.example/schema/keybindings",
  "title": "Keybindings",
  "description": "AGENTS: before modifying this schema, read AGENTS.md at the repository root.\n\nMaps logical actions to physical inputs.",
  "type": "object",
  "additionalProperties": false,
  "required": ["actions"],
  "properties": {
    "actions": {
      "description": "Map from logical action name to one or more input bindings.",
      "type": "object",
      "additionalProperties": {
        "$ref": "#/$defs/action_bindings"
      }
    }
  },
  "$defs": {
    "action_bindings": {
      "type": "object",
      "additionalProperties": false,
      "required": ["keyboard"],
      "properties": {
        "keyboard": {
          "description": "List of keyboard key codes (Bevy KeyCode names) that trigger this action.",
          "type": "array",
          "items": { "type": "string" },
          "default": []
        },
        "gamepad_button": {
          "description": "Optional gamepad button name.",
          "type": ["string", "null"],
          "default": null
        }
      }
    }
  }
}
```

#### 2.3 Default keybindings file: `assets/config/keybindings.json`

Defines bindings for the six logical actions required by M1 input logging:

```json
{
  "actions": {
    "thrust_forward":  { "keyboard": ["KeyW"] },
    "thrust_backward": { "keyboard": ["KeyS"] },
    "pitch_up":        { "keyboard": ["ArrowUp"] },
    "pitch_down":      { "keyboard": ["ArrowDown"] },
    "yaw_left":        { "keyboard": ["ArrowLeft"] },
    "yaw_right":       { "keyboard": ["ArrowRight"] },
    "roll_left":       { "keyboard": ["KeyQ"] },
    "roll_right":      { "keyboard": ["KeyE"] },
    "strafe_left":     { "keyboard": ["KeyA"] },
    "strafe_right":    { "keyboard": ["KeyD"] },
    "strafe_up":       { "keyboard": ["KeyR"] },
    "strafe_down":     { "keyboard": ["KeyF"] }
  }
}
```

#### 2.4 Rust types in `delta-v-config`

File: `crates/delta-v-config/src/keybindings.rs`

- `pub struct Keybindings { pub actions: HashMap<String, ActionBindings> }`
- `pub struct ActionBindings { pub keyboard: Vec<String>, pub gamepad_button: Option<String> }`
- Both derive `serde::Deserialize`.
- No `Default` impl — values must come from loaded JSON (ADR-0013, ADR-0014).

File: `crates/delta-v-config/src/loader.rs`

A `pub fn load_keybindings() -> Result<Keybindings, ConfigError>` that:
1. Loads `assets/config/keybindings.json` (hard error if missing — ADR-0013).
2. Validates against `assets/json/schema/keybindings.schema.json` using `jsonschema` (hard error on failure).
3. Applies schema `default` values for any omitted optional fields (the recursive fill-defaults pass — ADR-0013).
4. Attempts to locate user override at `$XDG_CONFIG_HOME/delta-v-beyond-sector-3-26/keybindings.json`; if present, deep-merges on top of defaults (ADR-0010); re-validates the merged result.
5. Deserialises into `Keybindings` with `serde_json::from_value`.
6. Returns typed `ConfigError` on any failure (ADR-0016).

Error type: `crates/delta-v-config/src/error.rs`
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{path}: file not found or unreadable: {source}")]
    Io { path: PathBuf, source: std::io::Error },
    #[error("{path}: JSON parse error: {source}")]
    Parse { path: PathBuf, source: serde_json::Error },
    #[error("{path}:{pointer}: schema validation failed: {reason}")]
    Schema { path: PathBuf, pointer: String, reason: String },
}
```

#### 2.5 `KeybindingsResource` Bevy resource

File: `crates/delta-v-config/src/resources.rs`

```rust
/// Loaded and validated keybindings. Inserted as a Bevy resource by
/// `ConfigPlugin` during `AppState::LoadingDefaults`.
#[derive(Resource)]
pub struct KeybindingsResource(pub Keybindings);
```

#### 2.6 Wire into `ConfigPlugin`

In `ConfigPlugin::build`:
- Register a system that runs in `OnEnter(AppState::LoadingDefaults)`.
- That system calls `load_keybindings()`, inserts the `KeybindingsResource`, logs success at `INFO`, and on failure calls `panic!` with a clear message (startup failure = hard error, ADR-0013).
- After inserting the resource, transition to `AppState::LoadingWorld` via `NextState`.

#### 2.7 Hot-reload stub (gated on `dev` feature)

Add a `dev` feature to `delta-v-config/Cargo.toml`. When active, register Bevy's asset file-watcher and a system that re-runs `load_keybindings()` on file change, replaces the resource, and logs `INFO` or `ERROR` (ADR-0035). No hot-reload code runs in release builds.

#### 2.8 Tests

Unit tests in `crates/delta-v-config/src/loader_tests.rs`:
- `test_loads_default_keybindings_ok` — load the shipped file, assert action count.
- `test_missing_defaults_file_errors` — point loader at a nonexistent path, assert `ConfigError::Io`.
- `test_schema_violation_errors` — feed a JSON object with an unknown field, assert `ConfigError::Schema`.
- `test_user_override_merges` — create a temp file overriding one key, assert merged result.

---

### Track 3 — World definition schema & loader

**Goal**: define the minimal JSON world format (ADR-0019, ADR-0020) and load it at startup so the scene knows what to spawn. The world file is the source of truth for what appears in the 3-D view.

#### 3.1 JSON Schema: `assets/json/schema/world.schema.json`

Minimal M1 shape — enough to describe a sector with one ship spawn point:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://sector-3-26-factory.example/schema/world",
  "title": "World definition",
  "description": "AGENTS: before modifying this schema, read AGENTS.md at the repository root.\n\nDefines a single playable sector: its name, ambient parameters and the list of entities to spawn at load time.",
  "type": "object",
  "additionalProperties": false,
  "required": ["format_version", "name", "player_ship"],
  "properties": {
    "format_version": {
      "description": "Integer schema version. Increment when the shape changes in a breaking way.",
      "type": "integer",
      "const": 1
    },
    "name": {
      "description": "Human-readable sector name shown in loading screens.",
      "type": "string"
    },
    "player_ship": {
      "description": "Spawn parameters for the player-controlled ship.",
      "$ref": "#/$defs/ship_spawn"
    }
  },
  "$defs": {
    "ship_spawn": {
      "type": "object",
      "additionalProperties": false,
      "required": ["position"],
      "properties": {
        "position": {
          "description": "Initial world position in metres (x, y, z). Forward is -Z per ADR-0006.",
          "type": "object",
          "additionalProperties": false,
          "required": ["x", "y", "z"],
          "properties": {
            "x": { "description": "X coordinate in metres.", "type": "number" },
            "y": { "description": "Y coordinate in metres.", "type": "number" },
            "z": { "description": "Z coordinate in metres.", "type": "number" }
          }
        },
        "facing": {
          "description": "Initial facing direction as a unit vector (x, y, z). Defaults to -Z (forward).",
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "x": { "description": "X component.", "type": "number", "default": 0.0 },
            "y": { "description": "Y component.", "type": "number", "default": 0.0 },
            "z": { "description": "Z component.", "type": "number", "default": -1.0 }
          },
          "default": { "x": 0.0, "y": 0.0, "z": -1.0 }
        }
      }
    }
  }
}
```

Note: positions here are bare floats in metres, not `{value, unit}` objects. The unit is **always metres** for in-world coordinates; the `{value, unit}` form (ADR-0008) is used for content that authors might naturally express in other units (orbital distances, stellar masses). A spawn position within a sector is always small-scale and always metres. The schema `description` makes this explicit.

#### 3.2 Default world file: `assets/worlds/default.world.json`

```json
{
  "format_version": 1,
  "name": "Sector 3.26 — Development Default",
  "player_ship": {
    "position": { "x": 0.0, "y": 0.0, "z": 0.0 }
  }
}
```

#### 3.3 Rust types in `delta-v-world`

File: `crates/delta-v-world/src/world_def.rs`

```rust
// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Deserialisation types for the world definition JSON format.
//!
//! See ADR-0019 (Asset pipeline) and ADR-0020 (Save and load format).

use serde::Deserialize;

/// Top-level world definition loaded from `*.world.json`.
#[derive(Debug, Deserialize)]
pub struct WorldDef {
    pub format_version: u32,
    pub name: String,
    pub player_ship: ShipSpawn,
}

/// Spawn parameters for a single ship entity.
#[derive(Debug, Deserialize)]
pub struct ShipSpawn {
    pub position: Vec3Json,
    #[serde(default)]
    pub facing: FacingJson,
}

/// A 3-component position in metres.
#[derive(Debug, Deserialize)]
pub struct Vec3Json {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// A 3-component direction (unit vector).
#[derive(Debug, Deserialize)]
pub struct FacingJson {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Schema default: -Z forward (ADR-0006).
impl Default for FacingJson {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: -1.0 }
    }
}
```

Note: `FacingJson::Default` is acceptable here because the default **matches the schema default** exactly (ADR-0013: "schema defaults are part of the contract"). The schema carries `"default": {"x":0.0,"y":0.0,"z":-1.0}` and the Rust impl mirrors it.

#### 3.4 World loader in `delta-v-world`

File: `crates/delta-v-world/src/loader.rs`

`pub fn load_default_world() -> Result<WorldDef, WorldError>`:
1. Reads `assets/worlds/default.world.json`.
2. Validates against `assets/json/schema/world.schema.json`.
3. Applies schema defaults (the fill-defaults pass — ADR-0013).
4. Deserialises into `WorldDef`.
5. Returns `WorldError` on failure.

Error type: `crates/delta-v-world/src/error.rs` — mirrors the `ConfigError` pattern.

#### 3.5 `WorldDefResource` Bevy resource

```rust
/// Loaded world definition. Inserted during `AppState::LoadingWorld`.
#[derive(Resource)]
pub struct WorldDefResource(pub WorldDef);
```

#### 3.6 Wire into `WorldPlugin`

- System in `OnEnter(AppState::LoadingWorld)`: call `load_default_world()`, insert `WorldDefResource`, log `INFO`. On failure: `panic!` with full path and reason.
- System in `OnEnter(AppState::LoadingWorld)` (after resource insertion, ordered via `after()`): read `WorldDefResource`, spawn the player ship entity (Track 4 provides the spawn helper), then transition to `AppState::InGame`.

#### 3.7 Tests

- `test_default_world_loads_ok` — loads the shipped file; asserts `format_version == 1`.
- `test_wrong_format_version_errors` — feeds `format_version: 2`; asserts schema error.
- `test_missing_position_errors` — omits `position`; asserts required-field error.

---

### Track 4 — Ship entity & 3-D scene

**Goal**: spawn a visible 3-D primitive representing the player ship, plus a directional light and ambient light so the mesh is lit. Physics components are stubs only at this milestone.

#### 4.1 `PlayerShip` component in `delta-v-ships`

File: `crates/delta-v-ships/src/components.rs`

```rust
// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Ship ECS components.

use bevy::prelude::*;

/// Marker component identifying the player-controlled ship entity.
///
/// At most one entity carries this component at a time.
#[derive(Component, Debug)]
pub struct PlayerShip;
```

#### 4.2 Ship spawn system in `delta-v-ships`

File: `crates/delta-v-ships/src/spawn.rs`

`pub fn spawn_player_ship` is a Bevy system that:
- Takes `Commands`, `ResMut<Assets<Mesh>>`, `ResMut<Assets<StandardMaterial>>`, `Res<WorldDefResource>` as parameters.
- Reads `WorldDefResource` to get the initial position.
- Spawns a `PbrBundle` with:
  - `Mesh`: `Capsule3d::new(0.5, 2.0)` (radius 0.5 m, length 2 m) — a cheap stand-in for a ship; clearly recognisable in 3-D (ADR-0013 dev placeholder rule does not apply here because this is intentional primitive geometry, not a missing asset).
  - `Material`: `StandardMaterial { base_color: Color::srgb(0.6, 0.7, 0.9), ..default() }`.
  - `Transform::from_translation(Vec3::new(x, y, z))` from world def, with `looking_to(Vec3::NEG_Z, Vec3::Y)` orientation (ADR-0006: -Z is forward).
- Also inserts the `PlayerShip` marker component on the same entity.
- Logs `INFO` with the spawn position.

#### 4.3 Lighting

In the same spawn system (or a separate `setup_scene` system registered in `ShipsPlugin`):
- Spawn a `DirectionalLightBundle` at a fixed orientation (e.g. `Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0))`) with `illuminance: 10_000.0`.
- Spawn `AmbientLight { color: Color::WHITE, brightness: 200.0 }` as a resource insert.

#### 4.4 Register in `ShipsPlugin`

```rust
app.add_systems(
    OnEnter(AppState::LoadingWorld),
    spawn_player_ship.after(world_loader_system),
);
```

The `after(world_loader_system)` ordering ensures `WorldDefResource` exists before the ship spawns. Use Bevy system sets to express this ordering explicitly (a `WorldLoadSet` defined in `delta-v-world` and a `ShipSpawnSet` in `delta-v-ships`; both in `OnEnter(LoadingWorld)`).

#### 4.5 Coordinate system verification

After spawning: the ship's local `-Z` axis should point along the global `-Z` axis (straight into the screen in a default top-down view). The chase camera (Track 5) will be placed behind and above, looking along `+Z` toward the ship.

#### 4.6 Tests

Integration test in `crates/delta-v-ships/tests/spawn_test.rs`:
- Build a minimal `App` with `MinimalPlugins`, `ShipsPlugin`, insert a fake `WorldDefResource`.
- Advance one frame.
- Query for `PlayerShip`; assert exactly one entity.
- Assert `Transform.translation` matches the world def position.

---

### Track 5 — Chase camera

**Goal**: spawn a 3-D perspective camera that follows the player ship from behind and above (ADR-0010 camera: standard Bevy `Camera3dBundle`).

#### 5.1 `CameraFollow` component in `delta-v-core`

File: `crates/delta-v-core/src/camera.rs`

```rust
// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Camera follow component and chase-camera system.
//!
//! The chase camera is intentionally simple at M1: fixed offset, no
//! lag, no spring damping. Those are M6 concerns.

use bevy::prelude::*;

/// Instructs the chase-camera system to follow a target entity.
///
/// Attach to the camera entity. Set `target` to the entity to follow.
#[derive(Component, Debug)]
pub struct CameraFollow {
    /// The entity to track.
    pub target: Entity,
    /// Offset from the target's position in the target's local space.
    /// Default: 20 m behind, 8 m above (Vec3 in target-local coords).
    pub offset: Vec3,
}
```

#### 5.2 Camera spawn system

In `CorePlugin` (or a dedicated `CameraPlugin` if the file grows large):

```rust
/// Spawns the 3-D chase camera after the player ship entity exists.
pub fn spawn_chase_camera(
    mut commands: Commands,
    ship_query: Query<Entity, With<PlayerShip>>,
) {
    let Ok(ship) = ship_query.get_single() else {
        // No ship yet; this system will not be called again; panic is correct.
        panic!("spawn_chase_camera: no PlayerShip entity found");
    };
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 8.0, 20.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraFollow {
            target: ship,
            offset: Vec3::new(0.0, 8.0, 20.0),
        },
    ));
}
```

Registered in `OnEnter(AppState::InGame)` — camera spawns after the world is loaded and the ship exists.

#### 5.3 Chase camera update system

```rust
/// Moves the camera to maintain its offset behind the followed entity.
///
/// Runs every frame in `Update` when `AppState::InGame`.
pub fn chase_camera_system(
    mut camera_query: Query<(&mut Transform, &CameraFollow)>,
    target_query: Query<&Transform, Without<CameraFollow>>,
) {
    for (mut cam_transform, follow) in &mut camera_query {
        let Ok(target_transform) = target_query.get(follow.target) else {
            continue;
        };
        // Rotate offset by the ship's current rotation so camera
        // stays behind the ship as it turns.
        let world_offset = target_transform.rotation * follow.offset;
        cam_transform.translation = target_transform.translation + world_offset;
        cam_transform.look_at(target_transform.translation, Vec3::Y);
    }
}
```

Runs in `Update` schedule, gated `.run_if(in_state(AppState::InGame))`.

#### 5.4 Remove the old `Camera2dBundle` stub

Delete the `setup` function and its `add_systems(Startup, setup)` call from `main.rs` (done in Track 1.2).

#### 5.5 Tests

Unit test in `crates/delta-v-core/src/camera_tests.rs`:
- `test_chase_camera_offset_applied` — build minimal App, spawn a fake ship at `(0,0,0)`, spawn camera with `CameraFollow { offset: (0,8,20) }`, run one frame, assert camera translation equals `(0,8,20)`.
- `test_chase_camera_follows_moved_ship` — move the fake ship to `(10,0,0)`, run another frame, assert camera translation equals `(10,8,20)`.

---

### Track 6 — Input logging

**Goal**: translate raw Bevy `KeyCode` / gamepad events into logical actions (ADR-0011) and log each active action at `DEBUG` level every fixed tick. No forces are applied; logging proves the pipeline works end-to-end.

#### 6.1 `LogicalAction` enum in `delta-v-core`

File: `crates/delta-v-core/src/input.rs`

```rust
// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Logical input actions. Gameplay code reads these; it never reads
//! raw keys directly. See ADR-0011.

/// All logical actions the player can perform.
///
/// This enum is the canonical set of actions. The keybindings schema
/// uses these names as string keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicalAction {
    ThrustForward,
    ThrustBackward,
    PitchUp,
    PitchDown,
    YawLeft,
    YawRight,
    RollLeft,
    RollRight,
    StrafeLeft,
    StrafeRight,
    StrafeUp,
    StrafeDown,
}

impl LogicalAction {
    /// The string key used in `keybindings.json` for this action.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ThrustForward  => "thrust_forward",
            Self::ThrustBackward => "thrust_backward",
            Self::PitchUp        => "pitch_up",
            Self::PitchDown      => "pitch_down",
            Self::YawLeft        => "yaw_left",
            Self::YawRight       => "yaw_right",
            Self::RollLeft       => "roll_left",
            Self::RollRight      => "roll_right",
            Self::StrafeLeft     => "strafe_left",
            Self::StrafeRight    => "strafe_right",
            Self::StrafeUp       => "strafe_up",
            Self::StrafeDown     => "strafe_down",
        }
    }

    /// All variants, in a stable iteration order.
    pub fn all() -> &'static [Self] {
        &[
            Self::ThrustForward,  Self::ThrustBackward,
            Self::PitchUp,        Self::PitchDown,
            Self::YawLeft,        Self::YawRight,
            Self::RollLeft,       Self::RollRight,
            Self::StrafeLeft,     Self::StrafeRight,
            Self::StrafeUp,       Self::StrafeDown,
        ]
    }
}
```

#### 6.2 `ActiveActions` resource

```rust
/// The set of logical actions currently held down this fixed tick.
///
/// Populated by `input_translation_system`; read by gameplay systems.
#[derive(Resource, Default, Debug)]
pub struct ActiveActions(pub std::collections::BTreeSet<LogicalAction>);
```

`BTreeSet` is used (not `HashSet`) to guarantee stable iteration order (ADR-0017: ordered collections in simulation-relevant paths).

#### 6.3 Input translation system

File: `crates/delta-v-core/src/input.rs` (continued) or `crates/delta-v-core/src/input_system.rs`

```rust
/// Translates raw keyboard input to `ActiveActions`.
///
/// Runs in `FixedUpdate`, gated to `AppState::InGame` (ADR-0017).
pub fn input_translation_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    keybindings: Res<KeybindingsResource>,
    mut active: ResMut<ActiveActions>,
) {
    active.0.clear();
    for action in LogicalAction::all() {
        let name = action.as_str();
        let Some(bindings) = keybindings.0.actions.get(name) else {
            continue;
        };
        let pressed = bindings.keyboard.iter().any(|key_name| {
            parse_key_code(key_name)
                .map(|kc| keyboard.pressed(kc))
                .unwrap_or(false)
        });
        if pressed {
            active.0.insert(*action);
        }
    }
}
```

`parse_key_code(name: &str) -> Option<KeyCode>` is a small helper that maps string names (e.g. `"KeyW"`) to `bevy::input::keyboard::KeyCode` variants. It lives in the same file. Only the 12 keys used in the default bindings need to be supported at M1; unknown names log a `WARN` once and return `None`.

#### 6.4 Input logging system

```rust
/// Logs active actions at DEBUG level each fixed tick.
///
/// This is the M1 proof-of-pipeline. It is not removed after M1;
/// it becomes silent in release builds because DEBUG is filtered out.
pub fn input_log_system(
    active: Res<ActiveActions>,
) {
    if !active.0.is_empty() {
        debug!("active actions: {:?}", active.0);
    }
}
```

Registered in `FixedUpdate` after `input_translation_system`, both gated `.run_if(in_state(AppState::InGame))`.

#### 6.5 Register in `CorePlugin`

```rust
app
    .init_resource::<ActiveActions>()
    .add_systems(
        FixedUpdate,
        (
            input_translation_system,
            input_log_system.after(input_translation_system),
        )
        .run_if(in_state(AppState::InGame)),
    );
```

#### 6.6 `KeybindingsResource` visibility

`KeybindingsResource` is defined in `delta-v-config` and read in `delta-v-core`. The dependency must go one direction only. Two options:

- **Option A** (preferred): Move `KeybindingsResource` to `delta-v-core` (as a re-export wrapper). `delta-v-config` inserts it; `delta-v-core` defines it and reads it. `delta-v-core` does not depend on `delta-v-config`; `delta-v-config` depends on `delta-v-core`.
- **Option B**: Define the resource in `delta-v-config`; make `delta-v-core` depend on `delta-v-config`.

Option A is cleaner given the declared layering (ADR-0002: technical crates must not form cycles; `core` is the base layer). Choose Option A.

#### 6.7 Tests

Unit tests in `crates/delta-v-core/src/input_tests.rs`:
- `test_no_keys_pressed_empty_active_actions` — no keyboard input; assert `ActiveActions` is empty.
- `test_thrust_forward_key_sets_action` — simulate `KeyW` pressed; assert `ThrustForward` in active set.
- `test_multiple_keys_set_multiple_actions` — simulate `KeyW` + `ArrowUp`; assert both actions present.
- `test_unknown_key_name_ignored` — feed a binding with `"keyboard": ["Nonsense"]`; assert no panic, action absent.

---

### Track 7 — Logging initialisation & diagnostics plugin

**Goal**: implement the logging strategy (ADR-0015) and the frame-time diagnostics plugin (ADR-0022).

#### 7.1 Logging initialisation in `CorePlugin`

Bevy's `DefaultPlugins` already include `LogPlugin`. We configure it via `DefaultPlugins.set(LogPlugin {...})` in `main.rs`:

```rust
.add_plugins(
    DefaultPlugins
        .set(WindowPlugin { /* ... */ })
        .set(LogPlugin {
            // Our crates at INFO in release, DEBUG with --features dev.
            #[cfg(not(feature = "dev"))]
            filter: "warn,delta_v=info,delta_v_core=info,delta_v_config=info,\
                     delta_v_physics=info,delta_v_assets=info,delta_v_ships=info,\
                     delta_v_propulsion=info,delta_v_weapons=info,delta_v_stations=info,\
                     delta_v_items=info,delta_v_world=info".to_string(),
            #[cfg(feature = "dev")]
            filter: "warn,delta_v=debug,delta_v_core=debug,delta_v_config=debug,\
                     delta_v_physics=debug,delta_v_assets=debug,delta_v_ships=debug,\
                     delta_v_propulsion=debug,delta_v_weapons=debug,delta_v_stations=debug,\
                     delta_v_items=debug,delta_v_world=debug".to_string(),
            level: bevy::log::Level::TRACE,
            ..default()
        }),
)
```

This satisfies ADR-0015: INFO for our crates in release, DEBUG in dev; third-party crates stay at WARN.

The `RUST_LOG` environment variable overrides this filter at runtime (Bevy/tracing standard behaviour).

#### 7.2 Frame-time diagnostics plugin

File: `crates/delta-v-core/src/diagnostics.rs`

This is a small Bevy plugin:

```rust
// AGENTS: before modifying this file, read AGENTS.md at the repository root.

//! Frame-time diagnostics: warns when sustained frame time exceeds a
//! configured threshold.
//!
//! See ADR-0022 (Performance instrumentation).

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

/// Configuration for the frame-time watchdog.
///
/// Inserted as a resource before `DiagnosticsPlugin::build` runs.
#[derive(Resource, Debug)]
pub struct DiagnosticsConfig {
    /// Frame time threshold in seconds. Default: 33 ms (30 fps floor).
    pub frame_time_warn_threshold_secs: f64,
    /// Number of consecutive frames over threshold before a WARN is emitted.
    pub consecutive_frames_threshold: u32,
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            frame_time_warn_threshold_secs: 0.033,
            consecutive_frames_threshold: 60,
        }
    }
}

#[derive(Resource, Default)]
struct OverrunStreak(u32);

/// Bevy plugin that wires `FrameTimeDiagnosticsPlugin` and the watchdog.
pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .init_resource::<DiagnosticsConfig>()
            .init_resource::<OverrunStreak>()
            .add_systems(Update, frame_time_watchdog_system);
    }
}

fn frame_time_watchdog_system(
    diagnostics: Res<DiagnosticsStore>,
    config: Res<DiagnosticsConfig>,
    mut streak: ResMut<OverrunStreak>,
) {
    let Some(ft) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
    else {
        return;
    };

    if ft > config.frame_time_warn_threshold_secs {
        streak.0 += 1;
        if streak.0 == config.consecutive_frames_threshold {
            warn!(
                "frame time {:.1} ms exceeded {:.1} ms threshold for {} consecutive frames",
                ft * 1000.0,
                config.frame_time_warn_threshold_secs * 1000.0,
                streak.0,
            );
        }
    } else {
        streak.0 = 0;
    }
}
```

`DiagnosticsPlugin` is registered inside `CorePlugin::build`.

#### 7.3 `diagnostics.json` config file

Per ADR-0022, the thresholds are JSON-configurable. Add schema `assets/json/schema/diagnostics.schema.json` and default config `assets/config/diagnostics.json`:

```json
{
  "frame_time_warn_threshold_ms": 33.0,
  "consecutive_frames_threshold": 60
}
```

The `ConfigPlugin` loads this file during `LoadingDefaults` and inserts a `DiagnosticsConfig` resource (overriding the default). This wiring is a small extension to the loader built in Track 2.

#### 7.4 `tracing` spans on systems

Per ADR-0022: add `#[instrument]` (or a `tracing::info_span!` guard) to the non-trivial systems introduced in this milestone:
- `input_translation_system`
- `chase_camera_system`
- `spawn_player_ship`
- `frame_time_watchdog_system` (already trivial; skip)

Targets must match the crate name (e.g. `target: "delta_v_core"`) so per-crate `RUST_LOG` filtering works.

#### 7.5 Startup log line

In `CorePlugin::build`, emit:
```rust
info!(version = env!("CARGO_PKG_VERSION"), "Delta-V starting");
```

This satisfies the ADR-0015 requirement that version is logged at startup (ADR-0025: version comes from `CARGO_PKG_VERSION`).

#### 7.6 Tests

- `test_overrun_streak_resets_on_fast_frame` — unit test on the streak logic: feed slow then fast frame times, assert streak resets to 0.
- `test_warn_emitted_after_n_consecutive_overruns` — feed N+1 overrun frames, capture log output (using `tracing_test` or similar), assert WARN was emitted exactly once.

---

## 3. Dependency graph changes

New crate-level dependencies introduced by this milestone. All versions are pinned in `[workspace.dependencies]` and inherited via `workspace = true` (ADR-0028).

| Crate added to | New dependency | Justification |
|---|---|---|
| `delta-v-config` | `jsonschema` | JSON Schema Draft 2020-12 validation (ADR-0012) |
| `delta-v-config` | `directories` | XDG / platform config path resolution (ADR-0010) |
| `delta-v-config` | `thiserror` | Typed error enum (ADR-0016) |
| `delta-v-config` | `serde` + `serde_json` | Already in workspace; add as explicit dep |
| `delta-v-world` | `thiserror` | Typed error enum |
| `delta-v-world` | `serde` + `serde_json` | JSON deserialisation |
| `delta-v-world` | `jsonschema` | Schema validation |
| `delta-v-core` | `delta-v-config` | To read `KeybindingsResource` (Option A from Track 6.6) |

Inter-crate dependency additions:
- `delta-v-config` gains no new crate-internal dependencies (it already depends on `bevy`).
- `delta-v-core` gains `delta-v-config` as a dependency (one new edge; no cycle introduced — `delta-v-config` does not depend on `delta-v-core`).
- `delta-v-ships` gains `delta-v-world` as a dependency (to read `WorldDefResource`).
- `delta-v-world` gains `delta-v-core` as a dependency (to use `AppState`).

Cargo dependency rule check (ADR-0002, ADR-0028):
- All new crate dependencies are checked for licence compatibility before adding (MIT / Apache-2.0 for `jsonschema`, `directories`, `thiserror` — all acceptable under ADR-0028).
- `cargo deny check` must pass after each addition.

---

## 4. File tree produced by this milestone

```
assets/
  config/
    keybindings.json          # Track 2 — default keybindings
    diagnostics.json          # Track 7 — frame-time thresholds
  json/
    schema/
      keybindings.schema.json # Track 2
      world.schema.json       # Track 3
      diagnostics.schema.json # Track 7
  worlds/
    default.world.json        # Track 3 — default sector

crates/
  delta-v-core/src/
    lib.rs                    # updated: export state, camera, input, diagnostics
    state.rs                  # Track 1 — AppState
    camera.rs                 # Track 5 — CameraFollow component + systems
    input.rs                  # Track 6 — LogicalAction, ActiveActions, systems
    diagnostics.rs            # Track 7 — DiagnosticsPlugin
    state_tests.rs            # tests for state transitions
    camera_tests.rs           # tests for chase camera
    input_tests.rs            # tests for input translation
    diagnostics_tests.rs      # tests for frame-time watchdog

  delta-v-config/src/
    lib.rs                    # updated: export loader, resources, error
    keybindings.rs            # Track 2 — Keybindings serde types
    loader.rs                 # Track 2 — load_keybindings(), load_diagnostics_config()
    error.rs                  # Track 2 — ConfigError
    resources.rs              # Track 2 — KeybindingsResource, DiagnosticsConfigResource
    loader_tests.rs           # tests for loader

  delta-v-world/src/
    lib.rs                    # updated: export world_def, loader, error, resources
    world_def.rs              # Track 3 — WorldDef serde types
    loader.rs                 # Track 3 — load_default_world()
    error.rs                  # Track 3 — WorldError
    resources.rs              # Track 3 — WorldDefResource
    loader_tests.rs           # tests for loader

  delta-v-ships/src/
    lib.rs                    # updated: export components, spawn
    components.rs             # Track 4 — PlayerShip marker
    spawn.rs                  # Track 4 — spawn_player_ship system
    spawn_tests.rs            # integration test for ship spawn

  delta-v/src/
    main.rs                   # updated: LogPlugin config, remove Camera2d stub
```

All new Rust files carry the AGENTS.md header comment (ADR-0033).
All new JSON files and schemas are also consistent with ADR-0033's JSON-specific pointer rule.

---

## 5. State machine & system scheduling map

This section shows the complete picture of which systems run in which schedule slot and which state.

```
Startup
  (none — all setup is state-driven)

OnEnter(AppState::Boot)
  CorePlugin:  log_state_transition        [always]

OnEnter(AppState::LoadingDefaults)
  CorePlugin:  log_state_transition        [always]
  ConfigPlugin: load_keybindings_system    -> inserts KeybindingsResource
                load_diagnostics_config    -> inserts DiagnosticsConfig
                -> transitions to LoadingWorld

OnEnter(AppState::LoadingWorld)
  CorePlugin:  log_state_transition        [always]
  WorldPlugin: load_world_system           -> inserts WorldDefResource  (step A)
  ShipsPlugin: spawn_player_ship           -> spawns ship entity         (step B, after A)
               setup_lighting              -> spawns lights
               -> WorldPlugin transitions to InGame

OnEnter(AppState::InGame)
  CorePlugin:  log_state_transition        [always]
  CorePlugin:  spawn_chase_camera          -> spawns Camera3dBundle + CameraFollow

FixedUpdate  [run_if InGame]
  CorePlugin:  input_translation_system   -> populates ActiveActions
  CorePlugin:  input_log_system           -> logs active actions  (after input_translation)

Update  [run_if InGame]
  CorePlugin:  chase_camera_system        -> moves camera behind ship
  CorePlugin:  frame_time_watchdog_system -> emits WARN on sustained overrun
```

Ordering within each schedule slot is made explicit via Bevy system sets:

```rust
// In delta-v-world:
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorldLoadSet { LoadDef, SpawnEntities }

// In delta-v-ships:
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShipSpawnSet { Spawn }
```

`SpawnEntities` is configured `.after(WorldLoadSet::LoadDef)` in `OnEnter(LoadingWorld)`.

---

## 6. ADR compliance checklist

This section cross-checks every Accepted ADR that governs M1 work. It is intended as a reviewer aid, not just a formality.

| ADR | Rule | Where satisfied |
|-----|------|-----------------|
| 0005 | One plugin per crate; binary composes all | `main.rs` plugin list; each crate exports exactly one plugin |
| 0006 | Right-handed, +Y up, -Z forward; metres | Ship spawned with `looking_to(Vec3::NEG_Z, Vec3::Y)`; positions in m |
| 0007 | Floating origin plugin stub exists | `PhysicsPlugin` remains a stub; origin system added in M3 |
| 0008 | Physical quantities in JSON use `{value, unit}` | Not applicable to M1 files (positions are always metres, scalars) |
| 0009 | Physics plugin stub; no forces yet | `PhysicsPlugin` stub; forces applied in M2 |
| 0010 | Two-layer config; deep merge; re-validate | `load_keybindings()` implements layers, merge, re-validate |
| 0011 | Logical actions in JSON; gameplay never reads raw keys | `LogicalAction` enum; `input_translation_system` intermediates |
| 0012 | Every JSON file has a schema; `additionalProperties:false`; descriptions on fields | Schemas in Track 2 and Track 3; rules verified |
| 0013 | No silent fallbacks; missing defaults = hard error; schema defaults are explicit | `load_keybindings()` panics on missing defaults file; `FacingJson::Default` mirrors schema default |
| 0014 | Engine constants in Rust; gameplay values in JSON | Camera offset is an engine default (not a gameplay knob); ship geometry is a dev primitive (replaced in M4+) |
| 0015 | INFO in release, DEBUG in dev; crate-name targets | `LogPlugin` filter in `main.rs` Track 7.1 |
| 0016 | `thiserror` in libraries; `anyhow` or `panic!` at binary top | `ConfigError`, `WorldError` use `thiserror`; startup panics on load failure |
| 0017 | `FixedUpdate` for simulation systems; no wall clock | Input translation in `FixedUpdate`; `Time` resource only |
| 0018 | Bevy `States`; transitions explicit | `AppState` in Track 1; `NextState` transitions only from designated systems |
| 0019 | Shipped assets under `assets/`; user content under XDG data | World file at `assets/worlds/`; user config looked up via `directories` crate |
| 0020 | World/saves in JSON; validated; format_version field | `default.world.json` has `format_version: 1` |
| 0021 | Unit tests in sibling `_tests.rs` files; integration tests in `tests/` | All test files follow this layout |
| 0022 | `tracing` spans on non-trivial systems; frame-time watchdog | `#[instrument]` on key systems; `DiagnosticsPlugin` in Track 7 |
| 0023 | `rustfmt`, `clippy::pedantic`, deny `unwrap_used` etc. | Pre-commit hook enforces; lint attrs in every `lib.rs` |
| 0024 | Every `pub` item has a rustdoc comment | Enforced by `#![warn(missing_docs)]` in each crate |
| 0033 | Every source file has AGENTS.md header | Checked in file tree (section 4) |
| 0034 | Zero warnings; `-D warnings` in CI | Pre-commit: `cargo clippy -- -D warnings` |
| 0035 | Hot-reload gated on `--features dev` | `dev` feature in `delta-v-config`; watcher registered only when active |

---

## 7. Commit sequence

M1 work lives on `feature/milestone_M1` (branch already exists per git refs). Commits follow Conventional Commits (ADR-0004). Each commit must be green (`cargo clippy -- -D warnings && cargo test --workspace`).

Suggested sequence:

```
feat(core): add AppState enum and state-transition logging
  - state.rs, CorePlugin wired, Camera2d stub removed from main.rs

build(config): add jsonschema, directories, thiserror dependencies
  - Cargo.toml workspace + delta-v-config deps

feat(config): implement keybindings schema, loader and resource
  - assets/json/schema/keybindings.schema.json
  - assets/config/keybindings.json
  - ConfigError, Keybindings types, load_keybindings(), KeybindingsResource
  - ConfigPlugin wired to LoadingDefaults
  - unit tests

feat(config): add diagnostics config schema and loader
  - assets/json/schema/diagnostics.schema.json
  - assets/config/diagnostics.json
  - DiagnosticsConfig resource, loader

build(world): add serde, serde_json, jsonschema, thiserror dependencies

feat(world): implement world definition schema, loader and resource
  - assets/json/schema/world.schema.json
  - assets/worlds/default.world.json
  - WorldDef types, load_default_world(), WorldDefResource
  - WorldPlugin wired to LoadingWorld
  - unit tests

feat(ships): add PlayerShip component and spawn system
  - components.rs, spawn.rs, lighting setup
  - ShipsPlugin wired to LoadingWorld (after world loader)
  - integration test

feat(core): add CameraFollow component and chase-camera system
  - camera.rs, spawn_chase_camera and chase_camera_system
  - CorePlugin wired to InGame
  - unit tests

feat(core): add LogicalAction, ActiveActions and input translation
  - input.rs, input_translation_system, input_log_system
  - CorePlugin wired to FixedUpdate/InGame
  - unit tests

feat(core): add DiagnosticsPlugin with frame-time watchdog
  - diagnostics.rs, DiagnosticsPlugin
  - CorePlugin registers DiagnosticsPlugin
  - unit tests

feat: configure LogPlugin filter for release and dev builds
  - main.rs LogPlugin.set() with crate-name filter strings

docs: update architecture.md system scheduling table for M1
chore: run cargo fmt --all; verify cargo test --workspace green
```

When all commits are green on `feature/milestone_M1`, open a PR to `dev`. The PR description must reference this plan and confirm each M1-O outcome is met.

---

## 8. Definition of done (expanded)

The milestone is complete when **all** of the following are true:

1. `cargo run` (without extra flags) opens a 1280×720 window showing a lit 3-D capsule mesh in empty space.
2. A perspective camera is visible behind and above the mesh.
3. Pressing `W`, `A`, `S`, `D`, `Q`, `E`, `R`, `F`, arrow keys logs the corresponding action names at DEBUG level in the terminal (run with `RUST_LOG=debug` to verify).
4. Stopping all key input produces no log spam.
5. `assets/config/keybindings.json` is loaded; changing a key in it and restarting changes which key triggers which log line.
6. `assets/worlds/default.world.json` is loaded; changing the `position` and restarting moves the ship.
7. Deleting `assets/config/keybindings.json` causes the game to exit with a non-zero status and a human-readable error message naming the file.
8. `cargo test --workspace` passes with zero test failures.
9. `cargo clippy --workspace --all-targets -- -D warnings` passes with zero warnings.
10. `cargo fmt --all -- --check` passes.
11. `cargo deny check` passes.
12. Every new `.rs` file contains the `// AGENTS:` header.
13. Every new `pub` item has at least a one-line rustdoc comment.
14. The `feature/milestone_M1` branch has been merged into `dev` via a PR that passes all CI checks.