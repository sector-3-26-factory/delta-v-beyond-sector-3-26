# Dev Container

This directory contains a reproducible Linux build environment for the
project, based on the official Microsoft Rust dev container image plus all
system libraries Bevy needs.

## Why a dev container?

Bevy pulls in a fair amount of native dependencies (ALSA, udev, Wayland,
X11, Vulkan loader, ...). Installing all of these on the host pollutes the
system and ties the project to the host's library versions. The dev
container isolates them.

## Requirements (Linux host)

- Docker (or Podman with the Docker socket compatibility).
- VS Code with the **Dev Containers** extension.
- An X11 (or XWayland) session for graphical output.
- Optional but recommended: a working Vulkan driver on the host
  (`vulkaninfo` should list at least one GPU).

## One-time host setup per session

Allow the container to talk to your X server:

```bash
xhost +local:
```

This grants local-socket clients access to your display. Revoke later with
`xhost -local:` if you wish.

## Opening the container

1. Open the project folder in VS Code.
2. Command Palette → **Dev Containers: Reopen in Container**.
3. Wait for the image to build (only on first use).
4. Inside the container's terminal:

   ```bash
   cargo run --features dev
   ```

   You should see an empty window titled "Delta-V beyond Sector 3.26" appear
   on your host display. The `dev` feature enables Bevy's dynamic linking,
   which dramatically speeds up incremental builds.

## GPU access (important on Linux hosts)

The container needs to open `/dev/dri/renderD128` (the DRI render node) to
use your real GPU via Vulkan. Two things must be true:

1. The device is passed through (done in `devcontainer.json` via
   `--device=/dev/dri`).
2. The container user is in a group that has read/write access to that
   device.

On the host, check the owning groups:

```bash
ls -l /dev/dri/
getent group render video
```

Typical output:

```
crw-rw----+ 1 root video  226,   1 ... /dev/dri/card1
crw-rw----+ 1 root render 226, 128 ... /dev/dri/renderD128
video:x:44:
render:x:109:
```

The `render` GID **varies between distributions** (105 on some Ubuntu
releases, 109 on Debian 12, 110 elsewhere). If yours is not `109`, edit
the `--group-add=109` line in `.devcontainer/devcontainer.json` and rebuild
the container.

Note: the `+` in the permission column means an ACL is granting your host
user access. ACLs do *not* propagate into the container, which is why the
group membership above is required.

Verify from inside the container:

```bash
ls -l /dev/dri/
vulkaninfo --summary | head -40
```

The render node should be readable and `vulkaninfo` should list your real
GPU, not just `llvmpipe`.

## Troubleshooting

**"cannot open display"**
: Make sure `echo $DISPLAY` inside the container prints the same value as on
  your host, and that you ran `xhost +local:` on the host.

**`Could not open device /dev/dri/renderD128: Permission denied` / falls back to `llvmpipe`**
: The container user is not in the host's `render` group. See the
  *GPU access* section above; fix the GID in `devcontainer.json` and
  rebuild the container.

**Vulkan errors / black window**
: Run `vulkaninfo --summary` inside the container. If no devices are
  listed, the GPU passthrough via `/dev/dri` is not working. Either install
  a Vulkan driver on the host, fix the group membership (see above), or
  remove the `--device=/dev/dri` line in `devcontainer.json` to fall back
  to llvmpipe (software rendering, slow but functional).

**`Library libxkbcommon-x11.so could not be loaded`**
: An old image is still in use. Run **Dev Containers: Rebuild Container**
  so the updated `Dockerfile` (which installs `libxkbcommon-x11-dev`)
  takes effect.

**Slow build times**
: The container uses the `mold` linker by default and `cargo run --features dev`
  enables Bevy's dynamic linking. First-time builds still take a while
  because Bevy is large; subsequent incremental builds should be fast.

**Wayland**
: The default configuration targets X11/XWayland because it is the most
  portable. Native Wayland passthrough is possible but more setup-heavy
  and not configured here yet.
