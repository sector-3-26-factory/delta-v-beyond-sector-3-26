---
description: "Download and integrate a new ship/mesh asset from Sketchfab or local source"
---

# /download-mesh — Ship & Mesh Asset Ingestion Workflow

This command automates the full workflow for importing a new 3D ship or mesh asset
into the delta-v project. It follows ADR-0041 (third-party asset acquisition),
ADR-0019 (asset pipeline), ADR-0038 (entity template system), and ADR-0040
(delta-v-json for JSON validation).

## Prerequisites

- Python 3 must be available (used for glTF binary parsing — no numpy or third-party deps)
- `curl` or `wget` for downloading from URLs
- Write access to `assets/glTF/`, `assets/templates/`, and `CREDITS.md`

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

### 2d. Determine asset filename

Convention: lowercase, hyphen-separated, include creator name for disambiguation.
Format: `<type>-<creator>.glb` or `<descriptive-name>-<creator>.glb`

Examples:
- `assets/glTF/ships/space-fighter-rauv.glb`
- `assets/glTF/ships/space-fighter-comrade1280.glb`
- `assets/glTF/asteroids/rock-cluster-milster.glb`

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

Copy the file to:

```
assets/glTF/<type>/<asset-name>.glb
```

---

## Step 5 — Analyze the Mesh (glTF Inspection)

Run the following pure-Python analysis script to extract mesh metadata. This uses
the proven technique of parsing the binary GLB header, extracting the JSON chunk,
and computing world-space transforms by walking the node hierarchy.

```python
#!/usr/bin/env python3
"""Analyze a binary glTF (.glb) file: nodes, materials, bounding boxes, cockpit detection."""

import struct, json, sys, os

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

    nodes = gltf.get('nodes', [])
    meshes = gltf.get('meshes', [])
    accessors = gltf.get('accessors', [])
    materials = gltf.get('materials', [])

    # --- Materials ---
    print("=== MATERIALS ===")
    glass_like = []
    for i, mat in enumerate(materials):
        name = mat.get('name', '<unnamed>')
        alpha = mat.get('alphaMode', 'OPAQUE')
        dbl = mat.get('doubleSided', False)
        pbr = mat.get('pbrMetallicRoughness', {})
        bc = pbr.get('baseColorFactor', [])
        print(f"  [{i}] name={name}, alpha={alpha}, double_sided={dbl}, base_color={bc}")
        # Detect glass/canopy-like materials
        if alpha in ('BLEND', 'MASK') or (len(bc) == 4 and bc[3] < 1.0):
            glass_like.append((i, name, alpha, bc))
        if any(kw in name.lower() for kw in ['glass', 'canopy', 'window', 'transparent', 'windshield']):
            glass_like.append((i, name, alpha, bc))

    if glass_like:
        print("\n  GLASS-LIKE MATERIALS FOUND:")
        for idx, name, alpha, bc in glass_like:
            print(f"    [{idx}] {name} (alpha={alpha}, color={bc})")

    # --- Node hierarchy ---
    parent_of = {}
    for i, n in enumerate(nodes):
        for c in n.get('children', []):
            parent_of[c] = i

    # --- Matrix math (pure Python, no numpy) ---
    def mat4_mul(a, b):
        r = [[0]*4 for _ in range(4)]
        for i in range(4):
            for j in range(4):
                for k in range(4):
                    r[i][j] += a[i][k] * b[k][j]
        return r

    def quat_to_mat4(qx, qy, qz, qw):
        return [
            [1-2*(qy*qy+qz*qz), 2*(qx*qy-qw*qz), 2*(qx*qz+qw*qy), 0],
            [2*(qx*qy+qw*qz), 1-2*(qx*qx+qz*qz), 2*(qy*qz-qw*qx), 0],
            [2*(qx*qz-qw*qy), 2*(qy*qz+qw*qx), 1-2*(qx*qx+qy*qy), 0],
            [0, 0, 0, 1]
        ]

    def get_local_matrix(node):
        if 'matrix' in node:
            m = node['matrix']
            return [m[i:i+4] for i in range(0, 16, 4)]
        t = node.get('translation', [0,0,0])
        r = node.get('rotation', [0,0,0,1])
        s = node.get('scale', [1,1,1])
        rot = quat_to_mat4(r[0], r[1], r[2], r[3])
        for i in range(3):
            for j in range(3):
                rot[i][j] *= s[j]
        for i in range(3):
            rot[i][3] = t[i]
        return rot

    world_mats = [None] * len(nodes)
    def get_world(idx):
        if world_mats[idx] is not None:
            return world_mats[idx]
        local = get_local_matrix(nodes[idx])
        if idx in parent_of:
            pw = get_world(parent_of[idx])
            world_mats[idx] = mat4_mul(pw, local)
        else:
            world_mats[idx] = [row[:] for row in local]
        return world_mats[idx]

    def transform_point(m, p):
        return tuple(sum(m[i][j]*p[j] for j in range(3)) + m[i][3] for i in range(3))

    # --- Search for cockpit-related nodes ---
    print("\n=== COCKPIT DETECTION ===")
    cockpit_nodes = []
    for i, node in enumerate(nodes):
        name = node.get('name', '').lower()
        if any(kw in name for kw in ['cockpit', 'canopy', 'glass', 'interior',
                                       'cabin', 'deck', 'bridge', 'window',
                                       'screen', 'hud', 'nose']):
            w = get_world(i)
            pos = (w[0][3], w[1][3], w[2][3])
            cockpit_nodes.append((i, node.get('name',''), pos))
            print(f"  FOUND: [{i}] '{node.get('name','')}' world_pos=({pos[0]:.2f}, {pos[1]:.2f}, {pos[2]:.2f})")

    if not cockpit_nodes:
        print("  No explicitly named cockpit/canopy nodes found.")

    # --- Per-mesh bounding boxes in world space ---
    print("\n=== PER-MESH BOUNDING BOXES (world space) ===")
    mesh_info = []
    for i, node in enumerate(nodes):
        if 'mesh' not in node:
            continue
        mesh_idx = node['mesh']
        if mesh_idx >= len(meshes):
            continue
        mesh = meshes[mesh_idx]
        w = get_world(i)
        parent_name = nodes[parent_of[i]].get('name', '') if i in parent_of else ''
        node_name = node.get('name', '<unnamed>')
        mesh_name = mesh.get('name', '?')

        for prim in mesh.get('primitives', []):
            pos_acc_idx = prim.get('attributes', {}).get('POSITION')
            if pos_acc_idx is None or pos_acc_idx >= len(accessors):
                continue
            acc = accessors[pos_acc_idx]
            if 'min' not in acc or 'max' not in acc:
                continue
            lmin, lmax = acc['min'], acc['max']
            corners = [(lmin[a], lmin[b], lmin[c]) for a in (0,1) for b in (0,1) for c in (0,1)]
            wc = [transform_point(w, c) for c in corners]
            wmin = [min(c[j] for c in wc) for j in range(3)]
            wmax = [max(c[j] for c in wc) for j in range(3)]
            center = [(wmin[j]+wmax[j])/2 for j in range(3)]
            info = {
                'node_idx': i, 'node_name': node_name, 'parent_name': parent_name,
                'mesh_idx': mesh_idx, 'mesh_name': mesh_name,
                'wmin': wmin, 'wmax': wmax, 'center': center
            }
            mesh_info.append(info)
            print(f"  '{parent_name}' -> '{node_name}':")
            print(f"    bbox min=({wmin[0]:.2f}, {wmin[1]:.2f}, {wmin[2]:.2f})")
            print(f"    bbox max=({wmax[0]:.2f}, {wmax[1]:.2f}, {wmax[2]:.2f})")
            print(f"    center =({center[0]:.2f}, {center[1]:.2f}, {center[2]:.2f})")

    # --- Overall model bounds ---
    print("\n=== OVERALL MODEL BOUNDS ===")
    fmin = [float('inf')]*3
    fmax = [float('-inf')]*3
    for acc in accessors:
        if 'min' in acc and 'max' in acc and acc.get('type') == 'VEC3':
            for j in range(3):
                fmin[j] = min(fmin[j], acc['min'][j])
                fmax[j] = max(fmax[j], acc['max'][j])
    print(f"  min=({fmin[0]:.2f}, {fmin[1]:.2f}, {fmin[2]:.2f})")
    print(f"  max=({fmax[0]:.2f}, {fmax[1]:.2f}, {fmax[2]:.2f})")
    extent = [fmax[j]-fmin[j] for j in range(3)]
    print(f"  extent=({extent[0]:.2f}, {extent[1]:.2f}, {extent[2]:.2f})")

    # --- Cockpit position estimation ---
    print("\n=== COCKPIT POSITION ESTIMATE ===")
    cockpit_pos = None

    # Strategy 1: Use explicitly named cockpit node
    if cockpit_nodes:
        # Prefer the one closest to the front (most negative Z)
        cockpit_nodes.sort(key=lambda x: x[2][2])
        _, name, pos = cockpit_nodes[0]
        cockpit_pos = pos
        print(f"  Strategy 1 (named node): '{name}' at ({pos[0]:.2f}, {pos[1]:.2f}, {pos[2]:.2f})")

    # Strategy 2: Find mesh with glass-like material in front-half of model
    if cockpit_pos is None and glass_like:
        for gi, gname, galpha, gbc in glass_like:
            # Find meshes using this material
            for info in mesh_info:
                m_idx = info['mesh_idx']
                for prim in meshes[m_idx].get('primitives', []):
                    if prim.get('material') == gi:
                        c = info['center']
                        if c[2] < 0:  # front half (negative Z = forward per ADR-0006)
                            cockpit_pos = c
                            print(f"  Strategy 2 (glass material): '{gname}' on mesh '{info['parent_name']}' at ({c[0]:.2f}, {c[1]:.2f}, {c[2]:.2f})")
                            break
                if cockpit_pos:
                    break
            if cockpit_pos:
                break

    # Strategy 3: Heuristic — front-upper-center of bounding box
    if cockpit_pos is None:
        # Cockpit is typically in the front 30-40% of the ship, upper half, centered X
        z_nose = fmin[2]  # most negative Z (front)
        z_tail = fmax[2]  # most positive Z (back)
        z_range = z_tail - z_nose
        # Place at ~30% from nose, upper quarter of Y, centered X
        est_x = (fmin[0] + fmax[0]) / 2.0
        est_y = fmax[1] - (fmax[1] - fmin[1]) * 0.25  # upper quarter
        est_z = z_nose + z_range * 0.30  # 30% from nose
        cockpit_pos = (est_x, est_y, est_z)
        print(f"  Strategy 3 (heuristic): front-upper-center estimate at ({est_x:.2f}, {est_y:.2f}, {est_z:.2f})")
        print(f"    NOTE: This is a rough estimate. Verify visually and adjust.")

    print(f"\n  RECOMMENDED COCKPIT POSITION: ({cockpit_pos[0]:.2f}, {cockpit_pos[1]:.2f}, {cockpit_pos[2]:.2f})")
    print(f"    In ship template coords (metres): x={cockpit_pos[0]:.2f}, y={cockpit_pos[1]:.2f}, z={cockpit_pos[2]:.2f}")

    return {
        'cockpit_pos': cockpit_pos,
        'model_min': fmin,
        'model_max': fmax,
        'model_extent': extent,
        'cockpit_nodes': cockpit_nodes,
        'glass_materials': glass_like,
    }

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <path-to-glb>")
        sys.exit(1)
    result = analyze_glb(sys.argv[1])
```

Save this script to a temporary path (e.g., `/tmp/analyze_glb.py`) and run:

```bash
python3 /tmp/analyze_glb.py assets/glTF/<type>/<asset-name>.glb
```

Capture the output. The key result is the **RECOMMENDED COCKPIT POSITION** in glTF
local space (= metres, right-handed, +Y up, -Z forward per ADR-0006).

---

## Step 6 — Generate Entity Template

Based on the asset type from Step 2c, create the appropriate JSON template file.

### 6a. For `ships/` type — create a ship template

Create `assets/templates/ships/<asset-name>.json` following the
[`ship.schema.json`](../assets/json/schema/ship.schema.json) schema.

Use [`fighter.json`](../assets/templates/ships/fighter.json) as a reference.

Required fields:
- `entity_type`: `"ship"`
- `mass`: prompt the user for mass in kg (or use a sensible default based on ship size)
- `inertia_scale`: default `1.0`
- `mesh.path`: `glTF/<type>/<asset-name>.glb`
- `propulsion`: prompt the user or use reasonable defaults:
  - `main_thrusters[0].id`: `"main"`
  - `main_thrusters[0].type`: `"chemical"`
  - `main_thrusters[0].max_forward_thrust`: `{"value": 100000, "unit": "N"}`
  - `main_thrusters[0].max_backward_thrust`: `{"value": 40000, "unit": "N"}`
  - `maneuvering_thruster.type`: `"rcs"`
  - `maneuvering_thruster.max_torque`: `{"value": 50000, "unit": "N⋅m"}`
  - `maneuvering_thruster.max_strafe_thrust`: `{"value": 50000, "unit": "N"}`

### 6b. For other types — check for existing schema

Look for `assets/json/schema/<type>.schema.json` or similar. If no schema exists,
note that a new schema and possibly a new ADR may be needed. Ask the user.

### 6c. Validate the template

Validate the generated JSON against its schema using `delta-v-json` or a Python
JSON Schema validator. Ensure it passes before proceeding.

---

## Step 7 — Generate Player Ship Template (if applicable)

If the asset is a `ships/` type and intended for player use, create a
`player_controlled_ship` template referencing the ship template.

Create `assets/templates/ships/<asset-name>-player.json` following the
[`player_controlled_ship.schema.json`](../assets/json/schema/player_controlled_ship.schema.json) schema.

Use [`player_ship.json`](../assets/templates/ships/player_ship.json) as a reference.

The `cameras.cockpit` position should use the **RECOMMENDED COCKPIT POSITION**
from Step 5 analysis:

```json
{
    "entity_type": "player_controlled_ship",
    "ship_template": "templates/ships/<asset-name>.json",
    "cameras": {
        "cockpit": {
            "x": <cockpit_x>,
            "y": <cockpit_y>,
            "z": <cockpit_z>
        }
    }
}
```

If the analysis used Strategy 3 (heuristic), add a comment noting that the
position should be verified visually and adjusted.

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
- **File:** `assets/glTF/<type>/<asset-name>.glb`
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
  - assets/glTF/<type>/<asset-name>.glb
  - assets/templates/ships/<asset-name>.json
  - assets/templates/ships/<asset-name>-player.json (if applicable)

Cockpit position: (x, y, z) [from Strategy 1/2/3]
  Note: <any caveats about the estimation method>

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
