---
description: "Download and integrate a new mesh asset from Sketchfab or local source"
---

# /download-mesh — Mesh Asset Ingestion Workflow

This command automates the full workflow for importing a new 3D mesh asset
into the delta-v project. It follows ADR-0041 (third-party asset acquisition),
ADR-0019 (asset pipeline), ADR-0038 (entity template system), and ADR-0040
(delta-v-json for JSON validation).

## Prerequisites

- Python 3 must be available (used for glTF binary parsing — no numpy or third-party deps)
- `curl` or `wget` for downloading from URLs
- Write access to `assets/templates/`, and `CREDITS.md`

---

## Step 1 — Collect Input

Ask the user for **one** of:

1. **A URL** — e.g. a Sketchfab model page URL (`https://sketchfab.com/3d-models/...`)
2. **A local HTML file path** — a saved `.html` page from a browser download

If neither is provided, stop and ask again.

---

## Step 2 — Parse Metadata from Input

### 2a. If URL provided

Fetch the page HTML (use `curl -sL "<url>"`) and extract:

| Field | How to find it |
|-------|----------------|
| **Model name** | `<title>` tag or `<meta property="og:title">` |
| **Author/Creator** | Sketchfab: `<a>` link to user profile, or `"author"` / `"creator"` JSON-LD |
| **Source URL** | The URL the user provided |
| **License** | Look for license text on the page (e.g. "CC BY 4.0", "CC Attribution"). Sketchfab shows this near the download button. |
| **License URL** | Derive from license name (e.g. `https://creativecommons.org/licenses/by/4.0/`) |
| **Download link** | Sketchfab: look for `glb` download URL in page JSON or `<a>` tags. May require navigating to the download API. |

### 2b. If local HTML file provided

Read the file and extract the same fields as above using the same parsing logic.

### 2c. Determine asset type

Classify the asset into one of these categories (per ADR-0041 §2):

- `ships/` — fighters, freighters, player ships, NPC vessels
- `asteroids/` — rocks, asteroid formations
- `stations/` — space stations, habitats, docks
- `celestial/` — planets, moons, stars (future)
- `misc/` — avoid; create a new category if none fit

Use the model name, description, tags, and visual appearance (if describable) to decide.
If ambiguous, ask the user.

### 2d. Determine asset directory name

Convention: lowercase, hyphen-separated, include creator name for disambiguation.
Format: `<type>-<creator>` or `<descriptive-name>-<creator>`

The asset is placed in a unified entity directory under `assets/templates/<type>/<name>/`
containing both the mesh file (`mesh.glb`) and the template JSON file.

Examples:
- `assets/templates/ships/space-fighter-rauv/mesh.glb`
- `assets/templates/ships/space-fighter-comrade1280/mesh.glb`
- `assets/templates/asteroids/rock-cluster-milster/mesh.glb`

---

## Step 3 — License & Legal Checks (mandatory, per ADR-0041 §1, §1b)

### 3a. License compatibility

Check the extracted license against the allow-list:

- ✅ **Accept**: CC0, CC BY 4.0, CC BY-SA 4.0, MIT, Apache 2.0, GPL 2.0+, GPL 3.0+
- ❌ **Reject**: CC BY-NC, CC BY-ND, proprietary, "personal use only"
- ⚠️ **Escalate**: Anything not listed above — ask the human

If incompatible → **STOP**. Report the license and reason to the user.

### 3b. Copyright/trademark check

Scan the model name, description, and tags for red flags:

- Franchise names: Star Wars, Star Trek, Stargate, Warhammer, Halo, Mass Effect, etc.
- "Fan art", "inspired by [IP]", "ripped from game"
- "For personal use only" disclaimers

If any red flag → **STOP**. Report findings to the human.

---

## Step 4 — Download the Mesh

### 4a. Download

If a direct `.glb` download URL was found in Step 2, download it:

```bash
curl -L -o "/tmp/<asset-name>.glb" "<download-url>"
```

If the asset page requires interaction (e.g., Sketchfab requires login or clicking
a download button), instruct the user to manually download the `.glb` file and
provide the local path.

### 4b. Validate format

Verify the file is a valid binary glTF:

```bash
python3 -c "
import struct
with open('<path-to-glb>', 'rb') as f:
    magic = f.read(4)
    assert magic == b'glTF', f'Not a GLB file: magic={magic}'
    version = struct.unpack('<I', f.read(4))[0]
    print(f'Valid GLB v{version}')
"
```

If invalid → **STOP**. Report the error.

### 4c. Place in assets directory

Copy the file to the unified entity directory:

```
assets/templates/<type>/<asset-name>/mesh.glb
```

Create the directory if it does not exist.

---

## Step 5 — Analyze the Mesh (glTF Inspection)

Run the following pure-Python analysis script to extract mesh metadata. This uses
the proven technique of parsing the binary GLB header, extracting the JSON chunk,
and computing world-space transforms by walking the node hierarchy.

```python
#!/usr/bin/env python3
"""Analyze a binary glTF (.glb) file: bounding box extraction."""

import struct, json, sys

def analyze_glb(filepath):
    with open(filepath, 'rb') as f:
        magic = f.read(4)
        assert magic == b'glTF', f"Not a GLB file: {magic}"
        version = struct.unpack('<I', f.read(4))[0]
        length = struct.unpack('<I', f.read(4))[0]
        chunk_length = struct.unpack('<I', f.read(4))[0]
        chunk_type = struct.unpack('<I', f.read(4))[0]
        json_data = f.read(chunk_length)
        gltf = json.loads(json_data)

    # Get overall bounds from accessors
    accessors = gltf.get('accessors', [])
    fmin = [float('inf')]*3
    fmax = [float('-inf')]*3
    for acc in accessors:
        if 'min' in acc and 'max' in acc and acc.get('type') == 'VEC3':
            for j in range(3):
                fmin[j] = min(fmin[j], acc['min'][j])
                fmax[j] = max(fmax[j], acc['max'][j])
    extent = [fmax[j]-fmin[j] for j in range(3)]
    return {'min': fmin, 'max': fmax, 'extent': extent}

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <path-to-glb>")
        sys.exit(1)
    result = analyze_glb(sys.argv[1])
    print(json.dumps(result, indent=2))
```

Save this script to a temporary path (e.g., `/tmp/analyze_glb.py`) and run:

```bash
python3 /tmp/analyze_glb.py assets/templates/<type>/<asset-name>/mesh.glb
```

Capture the output. The key result is the **bounding box** (min/max in glTF local space, metres).

---

## Step 6 — Generate Entity Template

Based on the asset type from Step 2c, create the appropriate JSON template file.

### 6a. For `ships/` type — create a ship template

Create `assets/templates/ships/<asset-name>/ship.json` following the
[`ship.schema.json`](../assets/json/schema/ship.schema.json) schema.

**Required fields:**
- `entity_type`: `"ship"`
- `mass`: prompt the user for mass in kg
- `inertia_scale`: default `1.0`
- `bounding_box`: computed from mesh analysis (Step 5)
- `collision_shape`: derived from bounding box (box shape with half_extents = bounding_box/2)
- `propulsion`: see existing templates for reference

### 6b. For other types (asteroids, stations, celestial, misc)

Create `assets/templates/<type>/<asset-name>/<entity_type>.json` following the
appropriate schema (e.g., `asteroid.schema.json`, `station.schema.json`).

**Required fields:**
- `entity_type`: the entity type discriminator
- `bounding_box`: computed from mesh analysis (Step 5)
- `collision_shape`: derived from bounding box (for asteroids, use sphere with radius = bounding_box extent/2)
- Other type-specific fields as defined in the schema

### 6c. Validate the template

Validate the generated JSON against its schema using `delta-v-json` or a Python
JSON Schema validator. Ensure it passes before proceeding.

---

## Step 7 — Generate Player Ship Template (ships only)

If the asset is a `ships/` type and intended for player use, create a
`player_controlled_ship` template.

Create `assets/templates/ships/<asset-name>/player_controlled_ship.json` following the
[`player_controlled_ship.schema.json`](../assets/json/schema/player_controlled_ship.schema.json) schema.

**Required fields:**
- `entity_type`: `"player_controlled_ship"`
- `cameras`: 8 cameras with position, target, and availability, computed from bounding box

Camera positions and targets are computed from the bounding box dimensions:
- Cockpit: near front-top of bounding box, looking forward
- Chase: behind and above, looking at ship center
- Rear/front: along Z axis
- Left/right: along X axis
- Top/bottom: along Y axis

### 7c. Validate the template

Validate the generated JSON against its schema.

---

## Step 8 — Update CREDITS.md

Append a new entry to `CREDITS.md` following the format from ADR-0041 §5 and the
existing entries. The entry must be placed under the `## 3D Models & Assets` section.

Format:

```markdown
### "<Asset Name>"

- **Creator:** [Creator Name](https://creator-profile-url/)
- **Source:** [Platform](https://asset-page-url/)
- **License:** <License Name> (<License Short>)
- **License URL:** <license-url>
- **File:** `assets/templates/<type>/<asset-name>/mesh.glb`
- **Modifications:** *None*
```

**Per ADR-0041 §7: The agent MUST NOT commit CREDITS.md changes without explicit
human approval.** Show the exact entry text to the user and wait for approval.

---

## Step 9 — Report Summary

Present a complete summary to the user:

```
Asset ingestion complete for "<Asset Name>":

Files created:
   - assets/templates/<type>/<asset-name>/mesh.glb
   - assets/templates/<type>/<asset-name>/<entity_type>.json
   - assets/templates/ships/<asset-name>/player_controlled_ship.json (if applicable)

Bounding box: min=(x, y, z), max=(x, y, z)
Cameras: 8 cameras with position and target computed from bounding box (ships only)

CREDITS.md entry:
   <exact text of the entry>

License: <license> ✅ compatible
Copyright/trademark check: ✅ passed

IMPORTANT: Per ADR-0041, CREDITS.md changes require your explicit approval
before committing. Please review the entry above and confirm.
```

---

## Error Handling

- **License incompatible** → STOP, report license name and reason
- **Copyright/trademark red flag** → STOP, report specific findings
- **Download fails** → instruct user to manually download and provide path
- **GLB validation fails** → STOP, report file corruption
- **Schema validation fails** → fix the template or report the error
- **No schema exists for asset type** → ask user for guidance

## Notes

- All JSON files must be validated against their schemas (ADR-0012, ADR-0040)
- All physical quantities must use `{"value": N, "unit": "U"}` format (ADR-0008)
- Never commit without human approval for CREDITS.md (ADR-0041 §7)
- Never commit any changes (AGENTS.md rule 7 — humans commit only)
- The glTF analysis script uses only Python 3 standard library (no numpy dependency)
- **Bounding box** is computed from the mesh and stored in the template JSON (single source of truth)
- **Collision shape** is derived from bounding box: for ships use box shape with half_extents = bounding_box/2; for asteroids use sphere with radius = bounding_box extent/2
- **Cameras** are computed from the bounding box and stored in player_controlled_ship.json (ships only)
- **Debug axes** use the bounding box from JSON, not computed from glTF at runtime
