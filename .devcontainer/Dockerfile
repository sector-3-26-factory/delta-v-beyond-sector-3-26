# Dev container for Delta-V beyond Sector 3.26
#
# Provides a reproducible Linux build environment with all system libraries
# Bevy needs (audio, input, windowing, Vulkan). The Rust toolchain version
# is pinned by `rust-toolchain.toml` at the repository root; rustup will
# install it on first use inside the container.

FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

ARG DEBIAN_FRONTEND=noninteractive

# System dependencies required by Bevy on Linux.
#
# Build-time (-dev packages provide headers + .so symlinks):
#  - libasound2-dev        : ALSA, audio
#  - libudev-dev           : gamepad / input hotplug
#  - libwayland-dev        : Wayland windowing
#  - libxkbcommon-dev      : keyboard handling (Wayland + X11) core
#  - libxkbcommon-x11-dev  : X11 backend for xkbcommon -- winit dlopens
#                            libxkbcommon-x11.so at runtime
#  - libx11-dev, libxi-dev,
#    libxcursor-dev,
#    libxrandr-dev,
#    libxinerama-dev       : X11 windowing + input
#  - libxcb1-dev,
#    libxcb-render0-dev,
#    libxcb-shape0-dev,
#    libxcb-xfixes0-dev    : XCB bits some winit/wgpu paths need
#  - libvulkan-dev         : Vulkan loader headers
#
# Runtime / drivers:
#  - mesa-vulkan-drivers   : Vulkan ICD for HW/software rendering
#  - vulkan-tools          : `vulkaninfo` for debugging
#
# Tooling:
#  - pkg-config            : build glue
#  - mold                  : fast linker (recommended by Bevy)
#  - clang                 : required by some -sys crates
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        clang \
        mold \
        libasound2-dev \
        libudev-dev \
        libwayland-dev \
        libxkbcommon-dev \
        libxkbcommon-x11-dev \
        libx11-dev \
        libxi-dev \
        libxcursor-dev \
        libxrandr-dev \
        libxinerama-dev \
        libxcb1-dev \
        libxcb-render0-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        libvulkan-dev \
        mesa-vulkan-drivers \
        vulkan-tools \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Pre-install useful cargo subcommands for the workflow.
# (Kept minimal; add more on demand to keep image size reasonable.)
USER vscode
RUN rustup component add rustfmt clippy
