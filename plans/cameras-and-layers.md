## Specification

### The Perfect Camera Setup

Here is the exact hierarchy from the very bottom (drawn first) to the very top (drawn last):

| Camera Name | `Camera::order` | `RenderLayers` | What this camera sees | How it works on screen |
| :--- | :--- | :--- | :--- | :--- |
| **`ActiveMainCamera`** | `0` | `Layer(0)` | Only the 3D world (stars, asteroids ahead). | Fills the **entire** screen. All 8 ship cameras render on `Layer(0)` with `order: 0` so the active camera can be switched without changing layers or draw order. |
| **`UiCamera`** | `1` | `Layer(1)` | The cockpit interior view (`.png`), static frames. | Fills the entire screen. Has transparent windows so the `ActiveMainCamera`'s world is visible through them. |
| **`CockpitMonitorCamera`** | `2` | `Layer(0)` | The 3D world from any ship camera's perspective. | **Uses a Viewport!** Draws a small box exactly over a designated monitor area of the cockpit PNG. Since it filters `Layer(0)`, it only sees the 3D world, ignoring the cockpit PNG. Any of the 8 ship cameras can be designated as a `CockpitMonitorCamera` — for example, the ship's rear view camera can be placed into a cockpit monitor to show the rear view on a small screen. When a ship camera is designated as a `CockpitMonitorCamera`, it is switched to active and its `Camera::order` changes from `0` to `2` so it renders on top of the cockpit overlay. |
| **`MenuCamera` (Lunex)** | `3` | `Layer(2)` | Lunex elements (windows, menus, keybindings). | Fills the entire screen (or parts of it). Since it sits at the very top (`order: 3`), your keybindings menu layers **on top of the cockpit and the monitors** when you open it. |

`ActiveMainCamera` is one of the 8 ship cameras (cockpit, chase, rear, front, left, right, top, bottom). Only one camera is active at a time. For now, the `ActiveMainCamera` is fixed to the chase view camera. A future milestone will make it switchable by pressing a key.

---

## Implementation Plan

### Current Bugs

1. **Ship cameras get individual layers** — `spawn_cameras` assigns cockpit=0, chase=1, rear=2, ..., bottom=7. Should all be `Layer(0)`.
2. **`UiCamera` uses `Layer(8)`** — should be `Layer(1)`.
3. **`MenuCamera` (Lunex) uses `Layer(8)`** — should be `Layer(2)`.
4. **`gameplay_render_layers()` covers layers 0-7** — since all ship cameras are now on `Layer(0)`, this should be just `Layer(0)`.

### Step 1: Fix ship camera render layers in `spawn_cameras`

**File:** `crates/delta-v-ships/src/spawn.rs`

Change the layer assignment so all 8 ship cameras use `Layer(0)`. Remove the per-camera layer from the loop tuple and use `RenderLayers::layer(0)` for all.

### Step 2: Fix `UiCamera` render layer

**File:** `crates/delta-v-core/src/camera/mod.rs`

Change `spawn_ui_camera` to use `RenderLayers::layer(1)` instead of `RenderLayers::layer(8)`.

### Step 3: Fix `MenuCamera` (Lunex) render layer

**File:** `crates/delta-v-ui/src/overlays/keybindings_menu/spawn.rs`

Change the Lunex `UiSourceCamera` render layer from `RenderLayers::layer(8)` to `RenderLayers::layer(2)`.

### Step 4: Remove `gameplay_render_layers()` and update its caller

**File:** `crates/delta-v-core/src/camera/mod.rs`

Remove the `gameplay_render_layers()` function — it has a single caller and always returns `RenderLayers::layer(0)`.

**File:** `crates/delta-v-core/src/lib.rs`

In `apply_gameplay_render_layers`, replace `camera::gameplay_render_layers()` with `RenderLayers::layer(0)` directly. Update the doc comment to reflect `Layer(0)`. Keep the system itself — it is still needed to dynamically add `RenderLayers` to entities spawned without one.

### Step 5: Run `cargo fmt --all` and verify

Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` to ensure zero warnings and all tests pass.
