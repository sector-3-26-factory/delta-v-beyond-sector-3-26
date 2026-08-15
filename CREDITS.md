# Credits

This game contains third-party assets. Copyrights remain with their
respective creators. All third-party assets are distributed under
open licenses compatible with GPL 3.0 or later.

For license text, see the LICENSE file at the repository root.

## Common mesh.glb Processing Steps

The following technical transformations are applied to most or all `mesh.glb` files to make them loadable and compatible with Bevy's glTF 2.0 loader and the game's conventions. These steps are not listed individually for each asset unless they deviate from the common process.

### For all mesh.glb files:
- **Unsupported glTF features removed/transformed:** Features not handled by Bevy are removed or converted (e.g., glTF extensions beyond 2.0 core, KHR_materials_pbrSpecularGlossiness converted to metallic/roughness, mesh compression decompressed).
- **Main objects centered:** The primary mesh is translated so its geometric center is at the origin (0, 0, 0). This ensures collision shapes, reticles, and visual meshes align.
- **Structure simplified:** Nested root nodes are removed; node hierarchy is flattened to a single mesh node where practical for proper glTF loading.

### For planets:
- **Separate game objects removed:** Mesh components that the game handles as separate entities (e.g. moons) are removed from the planet mesh. Moons are spawned as independent objects with their own templates.

### For ships:
- **Coordinate system alignment:** Meshes are rotated to match Bevy/glTF 2.0 convention: +Y up, +X right, -Z forward (ADR-0006). Original assets often use +Z forward or other conventions.
- **Scaling:** Meshes are scaled to realistic gameplay sizes (e.g., from ~300m to ~15m for fighters).

## 3D Models & Assets

### Space Fighter

- **Creator:** [Comrade1280](https://sketchfab.com/comrade1280)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/space-fighter-e766136d4871441289d37d44a4bbcd3b)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/ships/space-fighter-comrade1280/mesh.glb`
- **Modifications:** Standard modifications

### Asteroid low poly

- **Creator:** [pasquill](https://sketchfab.com/pasquill)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/asteroid-low-poly-9a43ef48a70647188576ccb5987b7e64)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/asteroids/asteroid-low-poly-pasquill/mesh.glb`
- **Modifications:** Standard modifications

### Daphne planetoid

- **Creator:** [SebastianSosnowski](https://sketchfab.com/SebastianSosnowski)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/daphne-planetoid-d7dd7ff088b04813ae26798a67d11c58)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/asteroids/daphne-planetoid-sebastiansosnowski/mesh.glb`
- **Modifications:** Standard modifications

### Archimedes (Meshy AI)

- **Creator:** [Meshy AI](https://www.meshy.ai)
- **Source:** Meshy AI texture generation
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/ships/meshy-archimedes/mesh.glb`
- **Modifications:** Standard modifications

### Cargo-1 (Meshy AI)

- **Creator:** [Meshy AI](https://www.meshy.ai)
- **Source:** Meshy AI texture generation
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/ships/meshy-cargo-1/mesh.glb`
- **Modifications:** Standard modifications

### Asteroid 2 (Meshy AI)

- **Creator:** [Meshy AI](https://www.meshy.ai)
- **Source:** Meshy AI texture generation
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/asteroids/meshy-asteroid-2/mesh.glb`
- **Modifications:** Standard modifications

### Sci-Fi Spaceship 400 (Meshy AI)

- **Creator:** [Meshy AI](https://www.meshy.ai)
- **Source:** Meshy AI texture generation
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/ships/meshy-sci-fi-spaceship-0400/mesh.glb`
- **Modifications:** Standard modifications

### Sci-Fi Spaceship 2033 (Meshy AI)

- **Creator:** [Meshy AI](https://www.meshy.ai)
- **Source:** Meshy AI texture generation
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/ships/meshy-sci-fi-spaceship-2033/mesh.glb`
- **Modifications:** Standard modifications

### Organic 1 (Meshy AI)

- **Creator:** [Meshy AI](https://www.meshy.ai)
- **Source:** Meshy AI texture generation
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/ships/meshy-organic-1/mesh.glb`
- **Modifications:** Standard modifications

### Laser electric

- **Creator:** [photon (that one larry) (@Professor_E12)](https://sketchfab.com/Professor_E12)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/laser-electric-60c132ce26014f068052fde25c585262)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/components/projectiles/laser-standard/mesh.glb`
- **Modifications:** *None*

### "Sun"

- **Creator:** [SebastianSosnowski](https://sketchfab.com/SebastianSosnowski)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/sun-9ef1c68fbb944147bcfcc891d3912645)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/suns/sol/mesh.glb`
- **Modifications:** *None*

### "Mercury (planet)"

- **Creator:** [SebastianSosnowski](https://sketchfab.com/SebastianSosnowski)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/mercury-planet-ccb6c6a9ac3742109cc67c0f16032b49)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/planets/mercury/mesh.glb`
- **Modifications:** *None*

### "Venus (planet)"

- **Creator:** [uperesito](https://sketchfab.com/uperesito)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/venus-v11-99be254b68da48d092c3b8917020c67a)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/planets/venus/mesh.glb`
- **Modifications:** Standard modifications

### "Earth (planet)"

- **Creator:** [c4m5ron](https://sketchfab.com/c4m5ron)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/earth-16k-high-resolution-e2f01432233b4c0983583ba5066af83d)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/planets/earth/mesh.glb`
- **Modifications:** Standard modifications

### "Moon"

- **Creator:** [RenderX (@RenderX)](https://sketchfab.com/RenderX)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/moon-26cc0b7878bb4d919b68e2be399db466)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/moons/moon/mesh.glb`
- **Modifications:** *None*

### "Mars (planet)"

- **Creator:** [ARCTIC WOLVES™ (@arctic.wolves)](https://sketchfab.com/arctic.wolves)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/mars-156a95dfec244e07a3ae423f579ffb05)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/planets/mars/mesh.glb`
- **Modifications:** *None*

### "Phobos (moon of Mars)"

- **Creator:** [harperanneviolet (@harperanneviolet)](https://sketchfab.com/harperanneviolet)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/phobos-1-1000-434a2b5523554918a13c1c41e3ee9be6)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/moons/phobos/mesh.glb`
- **Modifications:** Standard modifications

### "Deimos (moon of Mars)"

- **Creator:** [uperesito (@uperesito)](https://sketchfab.com/uperesito)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/deimos-db9c281d0a7b4452aaac26eb1d3738ad)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/moons/deimos/mesh.glb`
- **Modifications:** Standard modifications

### "Jupiter (planet)"

- **Creator:** [Mieke Roth (@miekeroth)](https://sketchfab.com/miekeroth)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/jupiter-c5275eb96af245e4a8453837ac728a62)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/planets/jupiter/mesh.glb`
- **Modifications:** Standard modifications

### "Saturn (planet)"

- **Creator:** [SebastianSosnowski](https://sketchfab.com/SebastianSosnowski)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/saturn-planet-9ab1eb3bb97f4e4a9305c0aae2d280a6)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/planets/saturn/mesh.glb`
- **Modifications:** Standard modifications

### "Uranus (planet)"

- **Creator:** [Nestaeric](https://sketchfab.com/Nestaeric)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/uranus-0009a69dbace44608c0bd09af9ba20db)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/planets/uranus/mesh.glb`
- **Modifications:** Standard modifications

### "Neptune (planet)"

- **Creator:** [Nestaeric](https://sketchfab.com/Nestaeric)
- **Source:** [Sketchfab](https://sketchfab.com/3d-models/neptune-2a6f9ccc5c724a709912774caa197b77)
- **License:** CC Attribution 4.0 (CC BY 4.0)
- **License URL:** https://creativecommons.org/licenses/by/4.0/
- **File:** `assets/templates/planets/neptune/mesh.glb`
- **Modifications:** None

## Fonts

### DejaVu Sans Mono

- **Creator:** DejaVu Fonts project (Bitstream Vera + Arev Fonts)
- **Source:** [GitHub](https://github.com/dejavu-fonts/dejavu-fonts)
- **License:** GPL-2.0-or-later
- **License URL:** https://www.gnu.org/licenses/old-licenses/gpl-2.0.html
- **File:** `assets/fonts/DejaVuSansMono.ttf`
- **Modifications:** *None*
- **Usage:** Default UI font for all text rendering (replaces Bevy's embedded Fira Mono which lacks Unicode arrow glyphs U+2190–U+2193)

## Sounds

### Thrust (default thrust sound)

- **Creator:** [bretbernhoft](https://pixabay.com/de/users/bretbernhoft-30625489/)
- **Source:** [Pixabay](https://pixabay.com/de/sound-effects/film-spezialeffekte-space-flight-10-422488/)
- **License:** Pixabay-Inhaltslizenz
- **License URL:** https://pixabay.com/de/service/license-summary/
- **File:** `assets/audio/thrust.mp3`
- **Modifications:** *None*

### Fire (default weapon fire sound)

- **Creator:** [DavidDumaisAudio](https://pixabay.com/de/users/daviddumaisaudio-41768500/)
- **Source:** [Pixabay](https://pixabay.com/de/sound-effects/film-spezialeffekte-sci-fi-weapon-laser-shot-04-316416/)
- **License:** Pixabay-Inhaltslizenz
- **License URL:** https://pixabay.com/de/service/license-summary/
- **File:** `assets/audio/fire.mp3`
- **Modifications:** *None*

### Hit (default impact sound)

- **Creator:** [Black_Kumizhi](https://pixabay.com/de/users/black_kumizhi-40755270/)
- **Source:** [Pixabay](https://pixabay.com/de/sound-effects/film-spezialeffekte-huge-cinematic-reverb-impact-506132/)
- **License:** Pixabay-Inhaltslizenz
- **License URL:** https://pixabay.com/de/service/license-summary/
- **File:** `assets/audio/hit.mp3`
- **Modifications:** *None*
