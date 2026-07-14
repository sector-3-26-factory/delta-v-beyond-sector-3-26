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

### 2026-07-14 Propulsion switching

- **Context**: While working on current task
- **Observation**: Propulsion switching is missing
- **Follow-up**: Implement propulsion switching functionality

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

### [YYYY-MM-DD] Brief description

- **Context**: What task you were working on
- **Observation**: What you noticed
- **Follow-up**: What needs to be done later

---

## TODOs

- [ ] Item 1
- [ ] Item 2