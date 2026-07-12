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
   cargo run --bin delta-v
   ```

   You should see an empty window titled "Delta-V beyond Sector 3.26" appear
   on your host display.

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

## Audio output (PulseAudio)

The dev container forwards audio to the host via PulseAudio. This requires:

1. **PulseAudio running on the host** (standard on most Linux desktops).
2. **The PulseAudio native socket** mounted into the container.

### Finding your PulseAudio socket

The PulseAudio native socket is typically at:

```
/run/user/<UID>/pulse/native
```

where `<UID>` is your user's numeric ID. The `devcontainer.json` assumes
UID 1000 (the default on most single-user Linux installations).

To find your actual UID:

```bash
id -u
```

To find your PulseAudio socket path:

```bash
ls -la /run/user/$(id -u)/pulse/native 2>/dev/null || \
    ls -la ~/.config/pulse/native 2>/dev/null || \
    echo "No PulseAudio socket found"
```

### Adjusting for a different UID

If your user has a different UID (e.g., you are user 1001 or 1002), edit
the mount in `.devcontainer/devcontainer.json`:

```json
{
    "mounts": [
        "source=/tmp/.X11-unix,target=/tmp/.X11-unix,type=bind,consistency=cached",
        "source=/run/user/1000/pulse/native,target=/pulse-native,type=bind,consistency=cached"
    ]
}
```

Change `1000` to your actual UID in the source path, then rebuild the
container:

1. **VS Code** → **Dev Containers: Rebuild Container**

### Verifying audio works

After starting the container, check that the PulseAudio socket is accessible:

```bash
ls -l /pulse-native
# Should show: srw-rw-rw- ... /pulse-native
```

If the socket is not found, the `postStartCommand` in `devcontainer.json`
will print a warning with troubleshooting hints.

## Troubleshooting

**No audio / "Connection refused" errors**
: The PulseAudio socket is not accessible. Check:
  - Your UID matches the mount in `devcontainer.json` (see *Audio output* section)
  - PulseAudio is running on the host: `pulseaudio --check && echo "running" || echo "not running"`
  - The socket exists: `ls -la /run/user/$(id -u)/pulse/native`
  - Inside the container: `ls -l /pulse-native` should show a socket file

**`XOpenDisplayFailed` / "cannot open display"**
: The most common cause is a mismatch between `DISPLAY` and the X11 socket
  that was mounted. The container inherits `DISPLAY` from the host at build
  time, but the mounted socket directory `/tmp/.X11-unix/` may only contain
  a socket for a different display number.

  Diagnose inside the container:

  ```bash
  echo $DISPLAY           # e.g. :1
  ls /tmp/.X11-unix/      # e.g. only X0 -- MISMATCH
  ```

  Fix on the host: set `DISPLAY` to match the actual socket, then rebuild:

  ```bash
  export DISPLAY=:0       # if the socket is X0
  xhost +local:
  # Then: VS Code → Dev Containers: Rebuild Container
  ```

  Note: VS Code reads `DISPLAY` from the host environment at the time it
  starts the container. If VS Code was already running when you set
  `DISPLAY`, restart VS Code after setting it.

  **Switched-user / multi-session gotcha:** If you used your desktop's
  "Switch User" feature, two X sessions are active simultaneously (`:0`
  for the first user, `:1` for the second). VS Code picks up whichever
  `DISPLAY` its session has, but the container may only have the socket
  for the other session mounted. The safest fix is to log fully out of
  the switched-to session and work from a single user session.

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
: The container uses the `mold` linker by default, which speeds up linking.
  First-time builds take a while because Bevy is large; subsequent
  incremental builds should be fast.

**Wayland**
: The default configuration targets X11/XWayland because it is the most
  portable. Native Wayland passthrough is possible but more setup-heavy
  and not configured here yet.
