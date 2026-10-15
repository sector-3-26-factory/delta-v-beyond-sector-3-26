<!-- Agent instructions: Read docs/adr/ARCHITECTURAL_RULES.md before making changes. -->

# Scratchpad

A temporary file for capturing observations, TODOs, and follow-up items while focusing on a specific task.

## How to use

- Add entries as you discover them during focused work
- Use the format below to keep things organized
- Review and act on these items when you switch tasks

---

## Entries

<!-- Add your notes below this line -->

### 2026-06-24 Asset loading and path separators

- **Context**: While working on current task
- **Observation**: Files under `assets/` may be loaded by Bevy's asset server. Need to determine if JSON configs are loaded by asset server or filesystem access. Using "/" as path separator in strings - need to check if Bevy provides Path types for cross-platform compatibility.
- **Follow-up**: Investigate Bevy asset loading mechanisms and path handling
- **Status**: ✅ COMPLETED - JSON configs are loaded via filesystem access (delta-v-json), Bevy AssetServer is used for binary assets (glTF, images, audio). Fixed cross-platform path handling in `delta-v-ships/src/spawn.rs` using `Path::parent()` instead of `rsplit_once('/')`.

### 2026-06-24 Cockpit/station switching

- **Context**: While working on current task
- **Observation**: There are only test stations. Need to store real stations.
- **Follow-up**: Implement station storage for cockpit/station switching

### 2026-06-24 Cockpit view polish

- **Context**: While working on current task
- **Observation**: Cockpit views need polishing.
- **Follow-up**: Polish cockpit view visuals

### 2026-06-24 Cockpit PNG availability

- **Context**: While working on current task
- **Observation**: Only the space fighter has a cockpit PNG. All other ships have dummies.
- **Follow-up**: Add cockpit PNGs for other ships or improve dummy handling

### 2026-07-14 Weapons switching

- **Context**: While working on current task
- **Observation**: Weapons switching is missing
- **Follow-up**: Implement weapons switching functionality
- **Status**: ✅ COMPLETED - Implemented weapons switching via `weapons` field in ship templates and `WeaponSelected` event system. The `ShipTemplateBase` trait now provides `weapons()` method returning `&[String]`, and the weapons selection system in `crates/delta-v-weapons/src/systems.rs` handles weapon switching.

### 2026-07-14 Propulsion switching

- **Context**: While working on current task
- **Observation**: Propulsion switching is missing
- **Follow-up**: Implement propulsion switching functionality
- **Status**: ✅ COMPLETED - Implemented propulsion switching via `max_propulsions_count` field in ship templates and `PropulsionSelected` event system. The `ShipTemplateBase` trait now provides `max_propulsions_count()` method, and the propulsion selection system in `crates/delta-v-ships/src/systems.rs` handles thruster switching.

### 2026-07-14 Projectile mesh uniformity

- **Context**: While working on current task
- **Observation**: All projectiles use the same mesh
- **Follow-up**: Implement distinct meshes for different projectile types

### 2026-07-14 Audio variety

- **Context**: While working on current task
- **Observation**: Different weapons and propulsions should use different sounds
- **Follow-up**: Implement distinct audio for different weapon and propulsion types

### 2026-07-14 World music and directory structure

- **Context**: While working on current task
- **Observation**: Worlds need music defined per world. Worlds should become directories with the JSON files copied to `<world directory>/world.json` and music as `<world directory>/music.<sound extension>`
- **Follow-up**: Refactor world loading to use directory-based structure with per-world music

### 2026-07-14 Sound volume configuration

- **Context**: While working on current task
- **Observation**: Need a configuration window to setup sound volume for music and FX
- **Follow-up**: Implement audio settings UI for music and sound effects volume control

### 2026-07-14 World loading UI

- **Context**: While working on current task
- **Observation**: Need a world loading window and a splash screen while loading a world
- **Follow-up**: Implement world selection UI and loading splash screen

### 2026-07-14 World lighting

- **Context**: While working on current task
- **Observation**: Every world needs at least one sun as the light source
- **Follow-up**: Ensure world loading includes a sun light source

### 2026-07-14 World metadata

- **Context**: While working on current task
- **Observation**: Every world should have a short optional description and an optional lore
- **Follow-up**: Add description and lore fields to world configuration

### 2026-07-14 Real solar system world

- **Context**: While working on current task
- **Observation**: Should provide a real solar system world
- **Follow-up**: Create a real solar system world configuration

### 2026-07-14 Star background

- **Context**: While working on current task
- **Observation**: Should provide a background with stars
- **Follow-up**: Implement star field background for space environments

### 2026-07-14 Target info HUD

- **Context**: While working on current task
- **Observation**: Need an info field/window in the HUD nearby the reticle, showing information about the targeted object (type, name, mass, speed for moving objects) and a velocity indicator showing the direction in which the object is moving
- **Follow-up**: Implement target info display and velocity indicator in the HUD

### 2026-07-14 Gameplay systems

- **Context**: While working on current task
- **Observation**: Need gameplay with resources (fuel, projectiles) that are consumed, income to buy fuel and projectiles (e.g. mission-based cargo transports), and ships need a cargo capacity value
- **Follow-up**: Implement resource system, mission/cargo mechanics, and ship cargo capacity

### 2026-09-28 UI fade zones and target switching broken (pre-existing)

- **Context**: Noticed by the user while working on the asteroid belt plan. **Predates the
  lazy-loading and shared-mesh work of 2026-09-26/27; not caused by it.** User observed
  this roughly a week before 2026-09-28.
- **Observation 1**: The window fade in / fade out zones are not visible any more. The
  transition zones that should soften window edges are absent, so windows pop in and out.
- **Observation 2**: Target switching with `T` (next) and `Shift+T` (previous) is not
  working any more. Implemented in `camera_switch_system` for cameras and in
  `delta-v-ships` for target cycling per ADR-0054, so the regression is likely in the
  input translation or the `ActionState` edge detection, not in the cycling logic itself.
- **Follow-up**: Investigate both. Check `delta-v-ui/src/window/` fade/border handling for
  the first, and the `ActionState` press/just_pressed wiring plus keybinding overrides for
  the second. Keep separate from the asteroid belt work.

### 2026-09-30 World epoch: set the solar system to a point in time

- **Context**: Designing the belt, and thinking about what happens to orbital positions
  across a long absence. Raised in review.
- **Idea**: the world definition should be able to place the solar system at a chosen
  epoch, not always at zero. What did the system look like on 27 May 1996? What will it
  look like on 11 June 2367?
- **Why it is attractive**:
  - It is free on top of what already exists. Orbital elements plus a mean anomaly at epoch
    is the standard way to describe an orbit, and ADR-0055 already propagates Keplerian
    motion analytically. Adding an epoch to a body is a parameter, not a new mechanism.
  - It composes with sector generation. A belt sector already regenerates from
    `(seed, belt_id, sector_index, simulation_time)`. A world epoch just moves where
    `simulation_time` starts, so the same machinery gives a different, deterministic
    configuration of the whole system.
  - It gives naturally different playthroughs. The same world file with a different epoch
    is a different universe, and worlds are authored by agents anyway, so an epoch is one
    more number in the JSON.
  - It makes orbital phase a content decision rather than an accident. Where the planets
    are relative to each other at the moment a player arrives is currently incidental.
- **Open questions to resolve when this is picked up**:
  - Epoch must be an absolute instant, not an offset, or two clients cannot agree.
  - It interacts with save state: the delta store in the belt plan records positions with
    timestamps, so it already assumes a shared notion of time. An epoch needs the same.
  - Retrocomputing real ephemerides for a 1996 or 2367 epoch is a much larger job than
    picking a plausible epoch. Worth splitting into "epoch is a parameter" first and
    "accurate historical ephemerides" later, if at all.
- **Follow-up**: not a belt task. Keep separate, like the UI bugs above.

### [YYYY-MM-DD] Brief description

- **Context**: What task you were working on
- **Observation**: What you noticed
- **Follow-up**: What needs to be done later

---

## TODOs

- [ ] Item 1
- [ ] Item 2