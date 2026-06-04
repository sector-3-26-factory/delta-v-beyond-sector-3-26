# ADR-0042: Custom asset loader with two-root search

- **Status**: Proposed
- **Date**: 2025-05-30
- **Deciders**: Cute-Donkey
- **Supersedes**: None
- **Relates to**: ADR-0019 (asset pipeline and user content), ADR-0013 (no silent fallbacks)

## Context

ADR-0019 defines a "two roots, one loader" architecture for asset management:
- **Shipped assets**: `assets/` (immutable, game-owned, updated with releases)
- **User content**: `$XDG_DATA_HOME/delta-v-beyond-sector-3-26/` (user-owned, never touched by updates)

Both roots contain identically-structured subdirectories (`templates/`, `worlds/`, etc.).
When a world references an asset (e.g., `"path": "templates/ships/space-fighter-comrade1280/mesh.glb"`), the loader must:

1. Search the user root first (override mechanism)
2. Fall back to the shipped root if not found in user
3. Error if found in neither root

**Current problem**: Bevy's native `AssetServer` only supports a single `file_path` root.
We currently work around this with compile-time `CARGO_MANIFEST_DIR` detection, which fails
for distributed binaries (GitHub releases, package installs).

**Scope conflict in templates**: A user creating a shared world may want to:
- Use their custom ship mesh for their world
- OR force use of a shipped mesh (to avoid portability issues)
- OR require that a mesh exists only in user content (world not portable)

We need an explicit mechanism to express this intent in JSON templates.

## Decision

We implement a custom **`DeltaVAssetLoader`** that:

### 1. Manages Two Asset Roots

**Shipped root** is detected in this priority order:
- `./assets/` relative to the running binary (release installations)
- `/usr/share/delta-v-beyond-sector-3-26/assets/` (Linux system-wide install)
- `$CARGO_MANIFEST_DIR/../../assets/` (dev builds via `cargo run`)
- Hard error if none found (ADR-0013: no silent fallbacks)

**User root** is platform-specific:
- Linux: `$XDG_DATA_HOME/delta-v-beyond-sector-3-26/` (default: `~/.local/share/`)
- macOS: `~/Library/Application Support/delta-v-beyond-sector-3-26/`
- Windows: `%APPDATA%/delta-v-beyond-sector-3-26/`
- Use `directories` crate for cross-platform paths

### 2. Implements Search and Override Logic

When loading an asset by path (e.g., `templates/ships/space-fighter-comrade1280/mesh.glb`):

**Default behavior (no scope specified):**
1. Check `user_root/templates/ships/space-fighter-comrade1280/mesh.glb`
   - If exists: load from user (override)
   - If not exists: continue to step 2
2. Check `shipped_root/templates/ships/space-fighter-comrade1280/mesh.glb`
   - If exists: load from shipped (fallback)
   - If not exists: error (asset not found in either root)

**With `scope: "user_only"`:**
- Load from `user_root/templates/ships/space-fighter-comrade1280/mesh.glb` only
- Error if not found (do not fall back to shipped)
- Use case: World requires a custom asset; not portable without it

**With `scope: "shipped_only"`:**
- Load from `shipped_root/templates/ships/space-fighter-comrade1280/mesh.glb` only
- Error if not found (do not fall back to user)
- Use case: World explicitly requires the official game asset

### 3. Adds Optional `scope` Field to Asset References

JSON schema for mesh definitions:

```json
{
  "mesh": {
    "scope": null
  }
}
```

Where `scope` is:
- `null` or omitted: search both roots, user preferred (default)
- `"user_only"`: search user root only (error if not found)
- `"shipped_only"`: search shipped root only (error if not found)

Note: The mesh path is always implicit (`mesh.glb` in the template directory), so no `path` field is needed.

### 4. Error Handling

Per ADR-0013 (no silent fallbacks), all errors are hard failures:

- **Shipped root not found**: Panic with actionable message
  ```
  fatal: could not locate shipped assets root. Checked:
    - ./assets/
    - /usr/share/delta-v-beyond-sector-3-26/assets/
    - <CARGO_MANIFEST_DIR>/../../assets/
  ```

- **Asset not found in any root**: Panic with both roots and path
  ```
  fatal: asset not found: templates/ships/space-fighter-comrade1280/mesh.glb
  Searched:
    - /home/user/.local/share/delta-v-beyond-sector-3-26/templates/ships/space-fighter-comrade1280/mesh.glb
    - /usr/share/delta-v-beyond-sector-3-26/assets/templates/ships/space-fighter-comrade1280/mesh.glb
  ```

- **Asset violates scope constraint**: Panic with explanation
  ```
  fatal: asset scope violation for templates/ships/space-fighter-comrade1280/mesh.glb
  Required scope: shipped_only
  Found: user root only
  Solution: Create the asset in shipped root or remove scope constraint
  ```

### 5. Logging and Diagnostics

All asset loads are logged at DEBUG level with source root:

```
[DEBUG] Loaded templates/ships/space-fighter-comrade1280/mesh.glb from user root
        /home/user/.local/share/delta-v-beyond-sector-3-26/templates/ships/space-fighter-comrade1280/mesh.glb
```

A diagnostic command (future work) lists effective content and origins.

## Implementation Phases

### Phase 1: ADR and Planning (current)
- Document decision and rationale
- Finalize scope field design
- Create implementation plan

### Phase 2: Custom AssetLoader Crate (future)
- Create `delta-v-asset-loader` crate
- Implement `DeltaVAssetServer` wrapper around Bevy's `AssetServer`
- Root detection and caching logic
- Search and scope validation
- Comprehensive error handling

### Phase 3: Schema and Template Updates (future)
- Add `scope` field to all asset-referencing schemas
- Update built-in templates to use correct paths
- Migration guide for existing templates

### Phase 4: Integration and Testing (future)
- Replace Bevy's `AssetPlugin` with `DeltaVAssetLoaderPlugin`
- Unit tests for root detection
- Integration tests for search and override behavior
- Cross-platform testing (Linux, macOS, Windows)

## Consequences

Positive:

- **Portable worlds**: Players can share worlds that reference shipped content;
  worlds "just work" when the other player has the same game version.
- **Custom content**: Players can create custom assets in their user directory
  and mix them with official content without modifying the game.
- **No accidental overwrites**: Shipped content is never modified at runtime;
  user updates cannot lose shipped assets.
- **Explicit intent**: The `scope` field makes it clear whether an asset is
  required (user_only), optional (default), or locked to official (shipped_only).
- **Cross-platform**: Use of `directories` crate handles Linux, macOS, Windows
  path differences automatically.
- **Distributed releases**: Works with release binaries, system-wide installs,
  and dev builds without special configuration.

Negative:

- **Complexity**: Custom loader adds code; correctness is critical to avoid
  confusing override behavior.
- **Path resolution overhead**: Root detection happens on startup; care needed
  to avoid unnecessary filesystem checks.
- **Incomplete until implemented**: Until Phase 2, dev builds still use the
  temporary `CARGO_MANIFEST_DIR` workaround.
- **Migration**: Existing templates and code assuming single root must be updated.

Follow-up work:

- Implement custom AssetLoader crate (Phase 2)
- Write schema migration guide
- Add diagnostics command to list effective content packs
- Consider content signing if modding ecosystem grows
- Consider pack load ordering (ADR-0019 mentions this as deferred)

## Notes

- **directories crate**: https://docs.rs/directories/latest/directories/
- **Bevy AssetServer**: https://docs.rs/bevy/latest/bevy/asset/
- **XDG Base Directory spec**: https://specifications.freedesktop.org/basedir-spec/latest/
- **Related discussion**: `/plans/asset-loader-two-roots.md`

## Example: World Sharing

**Scenario 1: User shares world with default ships**

```json
{
  "name": "My Sector",
  "ships": [
    {
      "type": "local_player_ship",
      "template": "templates/ships/space-fighter-comrade1280/template.json"
    }
  ]
}
```

When recipient loads this world:
- Loader resolves the template path, derives `mesh.glb` from the template directory
- Searches user root first, falls back to shipped root
- World loads successfully

**Scenario 2: User shares world with custom ship mesh**

```json
{
  "name": "My Custom Sector",
  "ships": [
    {
      "template": "templates/ships/custom-fighter/template.json",
      "mesh": {
        "scope": "user_only"
      }
    }
  ]
}
```

Recipient must have `custom-fighter/mesh.glb` in their user root, or load fails with
explicit error: "asset scope violation: custom-fighter/mesh.glb required in user root"

**Scenario 3: User forces official asset**

```json
{
  "mesh": {
    "path": "templates/ships/space-fighter-comrade1280/mesh.glb",
    "scope": "shipped_only"
  }
}
```

Loader ignores any user version and uses shipped only. Useful for competitive
modes or when ensuring consistent visuals across all players.
