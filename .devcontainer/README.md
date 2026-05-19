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
   cargo run
   ```

   You should see an empty window titled "Delta-V beyond Sector 3.26" appear
   on your host display.

## Troubleshooting

**"cannot open display"**
: Make sure `echo $DISPLAY` inside the container prints the same value as on
  your host, and that you ran `xhost +local:` on the host.

**Vulkan errors / black window**
: Run `vulkaninfo --summary` inside the container. If no devices are
  listed, the GPU passthrough via `/dev/dri` is not working. Either install
  a Vulkan driver on the host or remove the `--device=/dev/dri` line in
  `devcontainer.json` to fall back to llvmpipe (software rendering, slow
  but functional).

**Slow build times**
: The container uses the `mold` linker by default. First-time builds still
  take a while because Bevy is large; subsequent incremental builds should
  be fast.

**Wayland**
: The default configuration targets X11/XWayland because it is the most
  portable. Native Wayland passthrough is possible but more setup-heavy
  and not configured here yet.
