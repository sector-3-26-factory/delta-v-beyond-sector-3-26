#!/usr/bin/env python3
# AGENTS: before modifying this file, read AGENTS.md at the repository root.
"""
Fix Earth mesh materials: set emissiveFactor to [0,0,0] for Earth surface materials.
"""

import json
import struct
import sys
from pathlib import Path
from typing import Any


def read_gltf(filepath: Path) -> tuple[dict, bytes | None]:
    """Read glTF JSON and binary data from .gltf or .glb file."""
    if filepath.suffix.lower() == ".glb":
        with open(filepath, "rb") as f:
            data = f.read()

        # GLB header
        magic = data[:4]
        if magic not in (b"glTF", b"gltF"):
            raise ValueError("Not a valid GLB file")

        version = struct.unpack("<I", data[4:8])[0]
        length = struct.unpack("<I", data[8:12])[0]

        offset = 12
        json_data = None
        bin_data = None

        while offset < len(data):
            chunk_length = struct.unpack("<I", data[offset : offset + 4])[0]
            chunk_type = data[offset + 4 : offset + 8]
            chunk_data = data[offset + 8 : offset + 8 + chunk_length]

            if chunk_type == b"JSON":
                json_data = json.loads(chunk_data.decode("utf-8"))
            elif chunk_type == b"BIN\x00":
                bin_data = chunk_data

            offset += 8 + chunk_length
            if offset % 4 != 0:
                offset += 4 - (offset % 4)

        if json_data is None:
            raise ValueError("No JSON chunk found in GLB")
        return json_data, bin_data

    else:
        # .gltf file (JSON + separate .bin)
        with open(filepath, "r") as f:
            json_data = json.load(f)
        return json_data, None


def write_glb(filepath: Path, gltf: dict, bin_data: bytes | None):
    """Write glTF as GLB file."""
    json_str = json.dumps(gltf, separators=(",", ":"), ensure_ascii=False)
    json_bytes = json_str.encode("utf-8")

    # Pad JSON to 4-byte alignment
    json_padding = (4 - (len(json_bytes) % 4)) % 4
    json_bytes += b" " * json_padding

    bin_padding = 0
    if bin_data:
        bin_padding = (4 - (len(bin_data) % 4)) % 4
        bin_data = bin_data + b"\x00" * bin_padding

    total_length = 12 + 8 + len(json_bytes)
    if bin_data:
        total_length += 8 + len(bin_data)

    with open(filepath, "wb") as f:
        # Header - use standard glTF magic
        f.write(b"glTF")
        f.write(struct.pack("<I", 2))  # version
        f.write(struct.pack("<I", total_length))

        # JSON chunk
        f.write(struct.pack("<I", len(json_bytes)))
        f.write(b"JSON")
        f.write(json_bytes)

        # BIN chunk
        if bin_data:
            f.write(struct.pack("<I", len(bin_data)))
            f.write(b"BIN\x00")
            f.write(bin_data)


def fix_earth_materials(filepath: Path, dry_run: bool = False):
    """Fix Earth surface materials by setting emissiveFactor to [0,0,0]."""
    print(f"Reading {filepath}...")
    gltf, bin_data = read_gltf(filepath)

    if bin_data is None:
        raise ValueError("External .bin files not supported; use .glb")

    materials = gltf.get("materials", [])
    print(f"Found {len(materials)} materials")

    # Earth surface materials are indices 0-3 (Earth_3, Earth_4, Earth, Earth_2)
    # Material 4 is Atmosphere (should keep its blue emissive)
    # Materials 5-8 are Clouds (no emissive, correct)
    earth_material_indices = [0, 1, 2, 3]

    for idx in earth_material_indices:
        if idx < len(materials):
            mat = materials[idx]
            name = mat.get("name", f"Material {idx}")
            old_emissive = mat.get("emissiveFactor", "not set")
            print(f"  Material {idx} ({name}): emissiveFactor {old_emissive} -> [0.0, 0.0, 0.0]")
            if not dry_run:
                mat["emissiveFactor"] = [0.0, 0.0, 0.0]
        else:
            print(f"  Warning: Material index {idx} not found")

    if not dry_run:
        print(f"Writing {filepath}...")
        write_glb(filepath, gltf, bin_data)
        print("Done!")
    else:
        print("Dry run - no changes written")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Fix Earth mesh materials")
    parser.add_argument("filepath", type=Path, help="Path to .glb file")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be done without writing")
    args = parser.parse_args()

    if not args.filepath.exists():
        print(f"Error: File not found: {args.filepath}")
        sys.exit(1)

    try:
        fix_earth_materials(args.filepath, args.dry_run)
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()