# ADR-0019: Asset pipeline and user content

- **Status**: Accepted
- **Date**: 2026-05-19
- **Deciders**: Cute-Donkey

## Context

The game ships its own content (default worlds, ship types, schemas,
audio) under `assets/`. Players will also create their own worlds,
ships and other content -- ideally with the help of AI tools, in JSON
and glTF formats. Mixing the two in one directory creates two
serious problems:

1. **Updates can destroy user content.** If a player's world lives in
   `assets/worlds/`, an installer that "syncs" `assets/` from the
   game package overwrites or deletes it.
2. **Updates can silently change game-shipped content.** If the
   player has modified a file under `assets/`, an update either
   loses their change or refuses to install.

We need a clean separation between *shipped assets* and *user
content*, with a single loader that knows about both.

## Decision

**Two roots, one loader:**

- **Shipped assets** live under `assets/` in the repository and in
  the installed game directory. This tree is **owned by the game
  package**: updates may add, remove or change anything here. Users
  must not edit files in this tree; doing so is unsupported.
- **User content** lives in a platform-appropriate user data
  directory:

  ```
  Linux:   $XDG_DATA_HOME/delta-v-beyond-sector-3-26/
           (default ~/.local/share/delta-v-beyond-sector-3-26/)
  macOS:   ~/Library/Application Support/delta-v-beyond-sector-3-26/
  Windows: %APPDATA%/delta-v-beyond-sector-3-26/
  ```

  Its subdirectories mirror `assets/` where it makes sense:
  `worlds/`, `ships/`, `stations/`, `items/`, etc.

Loader behaviour:

- The loader searches both roots for content. User content takes
  precedence over shipped content with the same logical identifier
  ("override"), unless an explicit "shipped only" or "user only"
  scope is requested.
- An optional `loadorder.json` in the user root lets players prefer
  one content pack over another; deferred until needed.
- The user data root is created on first launch with restrictive
  permissions on Unix-like systems.
- The user data root is **never** modified by the installer or by
  application updates. The only writes to it are user-driven (e.g.
  saving a world the player created, exporting a ship).
- Shipped content is **immutable at runtime**. The game does not
  write to `assets/`.

User configuration (keybindings, etc.) follows a parallel scheme
under the **config** directory, not the **data** directory, per
[ADR-0010](0010-configuration-system.md). Data is "stuff the player
made"; config is "how the player likes things".

Asset format choices:

- 3D content: **glTF 2.0** (`.gltf` JSON or `.glb` binary). Bevy
  loads these natively.
- Data content (ship types, sectors, item definitions): JSON,
  validated by JSON Schemas per
  [ADR-0012](0012-json-schema-validation.md).
- Saves: JSON / glTF as appropriate, per
  [ADR-0020](0020-save-and-load-format.md).

## Consequences

Positive:

- Game updates cannot lose user content.
- Players know exactly where their work lives and can back it up.
- The override mechanism gives modders a low-friction extension
  point without changes to shipped files.

Negative:

- Asset lookup needs to consult two roots; correctness is essential
  to avoid surprising behaviour ("why did my ship change after
  update?" because shipped content moved and user override no
  longer matched).
- The split makes "load my own world" UX slightly more complex (file
  picker rooted in the user data directory).
- Cross-platform path handling needs care; the `dirs`/`directories`
  crate is the practical solution.

Follow-up:

- The asset loader code centralises the override logic so that no
  caller has to think about which root a file came from.
- A diagnostic command (later) lists effective content packs and
  which root each came from.
- A separate ADR may follow if multi-pack load ordering or content
  signing becomes relevant.
