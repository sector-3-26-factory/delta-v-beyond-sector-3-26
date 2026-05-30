"# ADR-0041: Third-party asset acquisition and licensing

- **Status**: Accepted
- **Date**: 2025-05-30
- **Deciders**: Cute-Donkey
- **Relates to**: ADR-0019 (asset pipeline), ADR-0027 (open source licensing), ADR-0028 (third-party dependencies)

## Context

The game ships with default assets (3D models, textures, audio) sourced from
third-party creators under various open licenses. As the project grows, agents
(both AI and human) will need to acquire, validate, and integrate these assets
into the repository.

Current challenges:

1. **License compatibility**: Not all open licenses are compatible with GPL 3.0+.
   Agents must verify licenses before integration.
2. **Attribution**: Third-party creators must be credited in CREDITS.md with
   author, license, source URL, and modification notes.
3. **Asset organization**: Assets must be sorted into logical type directories
   (`ships/`, `asteroids/`, `stations/`, etc.) under `assets/glTF/`.
4. **Format consistency**: All 3D assets must be in glTF 2.0 format (per ADR-0019)
   to ensure compatibility with Bevy's loader.
5. **Reproducibility**: Agents need clear, automated rules for the acquisition
   workflow to ensure consistency and auditability.

## Decision

**Agents follow a structured asset acquisition workflow:**

### 1. License Compatibility Check (mandatory first step)

Before downloading any asset, the agent MUST:

- Identify the asset's license from the source platform (Sketchfab, Poly Haven,
  TurboSquid, etc.).
- Verify compatibility with GPL 3.0 or later using this decision tree:
  - ✅ **Accept**: CC0 (public domain), CC BY 4.0, CC BY-SA 4.0, MIT, Apache 2.0,
    GPL 2.0+, GPL 3.0+
  - ❌ **Reject**: CC BY-NC (no commercial use), CC BY-ND (no derivatives),
    proprietary licenses, restricted redistribution
  - ⚠️ **Ask first**: Any license not listed above; escalate to human decision-maker

If the license is incompatible, the agent MUST NOT download or integrate the asset.

### 1b. Copyright and Trademark Violation Check (mandatory)

Before downloading, the agent MUST examine the asset's metadata for indicators
of unauthorized use of copyrighted or trademarked intellectual property:

**Red flags to reject immediately:**

- Asset name, description, or tags reference copyrighted franchises or characters:
  - Star Wars, Star Trek, Stargate, Warhammer, Halo, Mass Effect, etc.
  - Marvel, DC, Disney properties
  - Anime/game character models without explicit fan-art permission
- Model explicitly claims to be "from [franchised universe]" or "based on [IP]"
- Tags include phrases like "fan art", "inspired by", "fan-made" paired with
  major IP franchises
- Creator notes indicate the model was extracted or ripped from a commercial game
  or proprietary software
- License text includes restrictions like "for personal use only" or disclaimers
  about fan-made status

**If ANY red flag is present:**

1. The agent MUST NOT download or integrate the asset
2. The agent MUST report the findings to the human with:
   - Asset URL
   - Specific red flags found
   - Recommendation: "Rejected due to [reason]"
3. The human makes the final decision (escalate if uncertain)

**Legitimate uses that ARE acceptable:**

- Generic sci-fi assets ("space fighter", "sci-fi ship") without franchise references
- Clearly original fan art with explicit CC BY 4.0 license
- Assets where the creator explicitly states "This is original work, not based
  on any franchise"
- Generic models with no franchise connection in name or description

### 2. Asset Type Classification

The agent MUST determine the asset's logical category:

- `ships/` — player ships, NPC vessels, fighters, freighters, etc.
- `asteroids/` — asteroid meshes and rock formations
- `stations/` — space stations, habitats, facilities
- `celestial/` — planets, moons, stars, nebulae (future)
- `misc/` — unclassified objects (to be avoided; prefer creating a new category)

Type is determined from:
- Source platform metadata (tags, model category)
- Model name and description
- Visual inspection if ambiguous

### 3. Download and Format Conversion

The agent MUST:

- Download the asset in its native format (usually `.glb` or `.gltf`).
- If the asset is NOT in glTF 2.0 format:
  - Convert using Blender (command-line `bpy` script) or equivalent tool
  - Validate the converted output
  - Document conversion steps in CREDITS.md under "Modifications"
- If conversion is not possible or produces errors, reject the asset and
  escalate to human review.

### 4. Asset Integration

Place the converted glTF file in:

```
assets/glTF/<type>/<asset-name>.glb
```

Example:
```
assets/glTF/ships/space-fighter-rauv.glb
```

Naming convention:
- Use lowercase, hyphen-separated filename
- Include creator name if helpful for disambiguation
- Avoid spaces, special characters, version numbers (store those in metadata)

### 5. CREDITS.md Entry

The agent MUST create or update `CREDITS.md` at the repository root with a
structured entry per asset:

**If CREDITS.md does not exist**, create it with this header:

```markdown
# Credits

This game contains third-party assets. Copyrights remain with their
respective creators. All third-party assets are distributed under
open licenses compatible with GPL 3.0 or later.

For license text, see the LICENSE file at the repository root.

## 3D Models & Assets

(entries follow below)
```

**For each asset, add or append an entry:**

```markdown
### "Asset Name"

- **Creator**: [Name](https://creator-url.com/)
- **Source**: [Sketchfab](https://sketchfab.com/models/...)
- **License**: CC BY 4.0
- **License URL**: https://creativecommons.org/licenses/by/4.0/
- **File**: `assets/glTF/ships/space-fighter.glb`
- **Modifications**: *None* (or describe changes: e.g., "decimated to 50k triangles, optimized for mobile")
```

**Important fields:**

- **Creator**: Human-readable name with link to portfolio/profile if available
- **Source**: Direct link to the asset's page (Sketchfab, Poly Haven, etc.)
- **License**: Short name (CC BY 4.0, MIT, etc.)
- **License URL**: Full URL to license text
- **File**: Exact path under `assets/glTF/`
- **Modifications**: Blank line with `*None*` if unmodified; describe any processing
  (mesh optimization, texture baking, format conversion, etc.)

### 6. Validation and Quality Checks

After integration, the agent MUST:

- Verify the glTF file loads in Bevy without errors
- Check that the asset is not corrupted or incomplete
- Confirm CREDITS.md entry is formatted correctly
- Run `cargo fmt` and `cargo clippy` to ensure no warnings

### 7. Human Review Requirement

**The agent MUST NOT commit changes to CREDITS.md without explicit human approval.**

After completing steps 1-6, the agent MUST:

1. Report completion with:
   - Asset path: `assets/glTF/<type>/<name>.glb`
   - CREDITS.md entry (exact text)
   - Commit message (exact text)
   - License confirmed as compatible
   - Copyright/trademark checks passed

2. **Wait for human approval** before committing

3. The human MUST verify:
   - Creator name matches the actual author (not placeholder names)
   - Source URL is correct and still accessible
   - License URL is accurate
   - Modifications field is truthful (e.g., "decimated to 50k triangles" is verifiable)
   - No copyright/trademark concerns were missed

4. Only after human approval does the agent commit the changes

**Rationale**: Attribution is permanent in version control. Errors in CREDITS.md
(wrong author names, misattribution, incorrect license URLs) are difficult to
fix retroactively. Human eyes catch mistakes that agents miss.

### 8. Commit and Attribution

After human approval, commit with a message following ADR-0004:

```
feat: add space-fighter asset from Rauv

Download from Sketchfab under CC BY 4.0 license.
- Creator: Rauv
- Source: https://sketchfab.com/3d-models/space-fighter-...
- File: assets/glTF/ships/space-fighter-rauv.glb
- CREDITS.md updated
```

## Usage by Agents

When a human says:

> "Download glTF from https://sketchfab.com/3d-models/space-fighter-e766136d4871441289d37d44a4bbcd3b"

The agent:

1. Fetches the asset page and checks the license
2. Verifies compatibility with GPL 3.0+
3. If incompatible, reports the license and stops
4. If compatible:
   - Downloads the asset
   - Converts to glTF 2.0 if needed (or reports if conversion failed)
   - Places in `assets/glTF/ships/space-fighter-rauv.glb`
   - Updates `CREDITS.md`
   - Commits with proper attribution
   - Reports success with asset path

When a human says:

> "Use ADR-0041 for https://sketchfab.com/3d-models/..."

The agent applies this entire workflow to that URL.

## Consequences

Positive:

- **Consistent attribution**: All third-party creators are properly credited with
  verifiable links.
- **License compliance**: GPL 3.0+ compatibility is checked before integration;
  no licensing surprises later.
- **Reproducibility**: The workflow is explicit and auditable; any agent can
  follow it consistently.
- **Asset organization**: Clear categorization makes assets easy to find and
  maintain.
- **Modification tracking**: CREDITS.md documents exactly what was changed,
  supporting transparent attribution.
- **Quality gates**: Validation steps catch broken or incompatible assets early.

Negative:

- **Manual work**: Asset conversion (e.g., `.fbx` to glTF) may require
  human intervention or tool setup (Blender).
- **License ambiguity**: Some licenses (e.g., variants of CC) require careful
  reading; edge cases may need human escalation.
- **CREDITS.md growth**: As the project accumulates assets, CREDITS.md becomes
  large; may warrant splitting into `CREDITS/*.md` later.

Follow-up:

- A Blender batch-conversion script to automate `.fbx` / `.obj` → glTF conversion.
- A license-compatibility database (JSON) for faster validation.
- A CI check that validates CREDITS.md entries (missing URLs, invalid paths, etc.).
- Extension to cover audio assets (same license checks, different directory structure).

## Notes

- Sketchfab API: https://sketchfab.com/developers
- Poly Haven (public domain): https://polyhaven.com/
- glTF 2.0 spec: https://www.khronos.org/gltf/
- Blender glTF export: https://docs.blender.org/manual/en/latest/addons/io_gltf_io.html
"