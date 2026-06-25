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

### 2026-06-24 Ctrl+F2 not working

- **Context**: While working on current task
- **Observation**: Ctrl+F2 keybinding is not functioning
- **Follow-up**: Investigate keybindings configuration and input handling

### 2026-06-24 Asset loading and path separators

- **Context**: While working on current task
- **Observation**: Files under `assets/` may be loaded by Bevy's asset server. Need to determine if JSON configs are loaded by asset server or filesystem access. Using "/" as path separator in strings - need to check if Bevy provides Path types for cross-platform compatibility.
- **Follow-up**: Investigate Bevy asset loading mechanisms and path handling

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

### [YYYY-MM-DD] Brief description

- **Context**: What task you were working on
- **Observation**: What you noticed
- **Follow-up**: What needs to be done later

---

## TODOs

- [ ] Item 1
- [ ] Item 2