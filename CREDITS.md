# Credits

This game contains third-party assets. Copyrights remain with their
respective creators. All third-party assets are distributed under
open licenses compatible with GPL 3.0 or later.

For license text, see the LICENSE file at the repository root.

## 3D Models & Assets

### Space Fighter

- **Creator:** [Comrade1280](https://sketchfab.com/comrade1280)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/space-fighter-e766136d4871441289d37d44a4bbcd3b)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/ships/space-fighter-comrade1280/mesh.glb`
- **Modifications:** 
  - Mesh rotated 180° around Y axis to align ship's nose with -Z (ADR-0006 coordinate convention). Original asset had +Z as forward.
  - Mesh scaled by factor of 0.05 (from ~300m to ~15m) to match realistic fighter size.

### Asteroid low poly

- **Creator:** [pasquill](https://sketchfab.com/pasquill)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/asteroid-low-poly-9a43ef48a70647188576ccb5987b7e64)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/asteroids/asteroid-low-poly-pasquill/mesh.glb`
- **Modifications:** *None*

### Daphne planetoid

- **Creator:** [SebastianSosnowski](https://sketchfab.com/SebastianSosnowski)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/daphne-planetoid-d7dd7ff088b04813ae26798a67d11c58)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/asteroids/daphne-planetoid-sebastiansosnowski/mesh.glb`
- **Modifications:**
  No artwork changed. Just technical optimizations to get the mesh.glb loaded.
  Optimizations: Removed nested root nodes, re-mapped PBR textures to standard PNGs, inverted UV Y-scale, and applied all transforms

