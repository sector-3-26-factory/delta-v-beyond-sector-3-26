#!/usr/bin/env python3
# AGENTS: before modifying this file, read AGENTS.md at the repository root.
"""
Inspect glTF/GLB mesh for materials, textures, UV coordinates, etc.
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


def read_accessor_data(gltf: dict, bin_data: bytes, accessor_idx: int) -> list:
    """Read vertex data from accessor."""
    accessor = gltf["accessors"][accessor_idx]
    buffer_view = gltf["bufferViews"][accessor["bufferView"]]
    buffer = gltf["buffers"][buffer_view["buffer"]]

    byte_offset = buffer_view.get("byteOffset", 0)
    if "byteOffset" in accessor:
        byte_offset += accessor["byteOffset"]

    component_type = accessor["componentType"]
    count = accessor["count"]
    type_str = accessor["type"]

    # Component type sizes
    type_sizes = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}
    comp_size = type_sizes.get(component_type, 4)

    # Type component counts
    type_counts = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT2": 4, "MAT3": 9, "MAT4": 16}
    comp_count = type_counts.get(type_str, 1)

    stride = buffer_view.get("byteStride", comp_size * comp_count)

    # Read data
    data = []
    for i in range(count):
        offset = byte_offset + i * stride
        vertex = []
        for j in range(comp_count):
            comp_offset = offset + j * comp_size
            if component_type == 5126:  # FLOAT
                val = struct.unpack("<f", bin_data[comp_offset : comp_offset + 4])[0]
            elif component_type == 5125:  # UNSIGNED_INT
                val = struct.unpack("<I", bin_data[comp_offset : comp_offset + 4])[0]
            elif component_type == 5123:  # UNSIGNED_SHORT
                val = struct.unpack("<H", bin_data[comp_offset : comp_offset + 2])[0]
            else:
                val = 0
            vertex.append(val)
        data.append(vertex if comp_count > 1 else vertex[0])

    return data


def inspect_gltf(filepath: Path):
    """Inspect glTF file for materials, textures, UVs, etc."""
    print(f"Inspecting {filepath}...")
    gltf, bin_data = read_gltf(filepath)

    if bin_data is None:
        print("  External .bin files not fully supported for inspection")
        bin_data = b""

    # Print basic info
    print(f"\n=== GLTF Info ===")
    print(f"  Asset: {gltf.get('asset', {})}")
    print(f"  Scenes: {len(gltf.get('scenes', []))}")
    print(f"  Nodes: {len(gltf.get('nodes', []))}")
    print(f"  Meshes: {len(gltf.get('meshes', []))}")
    print(f"  Materials: {len(gltf.get('materials', []))}")
    print(f"  Textures: {len(gltf.get('textures', []))}")
    print(f"  Images: {len(gltf.get('images', []))}")
    print(f"  Accessors: {len(gltf.get('accessors', []))}")
    print(f"  BufferViews: {len(gltf.get('bufferViews', []))}")
    print(f"  Buffers: {len(gltf.get('buffers', []))}")

    # Inspect materials
    print(f"\n=== Materials ===")
    for i, mat in enumerate(gltf.get("materials", [])):
        print(f"  Material {i}: {mat.get('name', 'unnamed')}")
        if "pbrMetallicRoughness" in mat:
            pbr = mat["pbrMetallicRoughness"]
            print(f"    PBR Metallic Roughness:")
            if "baseColorFactor" in pbr:
                print(f"      baseColorFactor: {pbr['baseColorFactor']}")
            if "baseColorTexture" in pbr:
                tex_idx = pbr["baseColorTexture"]["index"]
                print(f"      baseColorTexture: index {tex_idx}")
                if tex_idx < len(gltf.get("textures", [])):
                    tex = gltf["textures"][tex_idx]
                    img_idx = tex.get("source", -1)
                    if img_idx >= 0 and img_idx < len(gltf.get("images", [])):
                        img = gltf["images"][img_idx]
                        print(f"        -> Image {img_idx}: {img.get('uri', img.get('name', 'embedded'))}")
            if "metallicFactor" in pbr:
                print(f"      metallicFactor: {pbr['metallicFactor']}")
            if "roughnessFactor" in pbr:
                print(f"      roughnessFactor: {pbr['roughnessFactor']}")
        if "normalTexture" in mat:
            tex_idx = mat["normalTexture"]["index"]
            print(f"    normalTexture: index {tex_idx}")
        if "occlusionTexture" in mat:
            tex_idx = mat["occlusionTexture"]["index"]
            print(f"    occlusionTexture: index {tex_idx}")
        if "emissiveTexture" in mat:
            tex_idx = mat["emissiveTexture"]["index"]
            print(f"    emissiveTexture: index {tex_idx}")
        if "emissiveFactor" in mat:
            print(f"    emissiveFactor: {mat['emissiveFactor']}")

    # Inspect textures
    print(f"\n=== Textures ===")
    for i, tex in enumerate(gltf.get("textures", [])):
        print(f"  Texture {i}: {tex.get('name', 'unnamed')}")
        if "source" in tex:
            img_idx = tex["source"]
            if img_idx < len(gltf.get("images", [])):
                img = gltf["images"][img_idx]
                print(f"    Source Image {img_idx}: {img.get('uri', img.get('name', 'embedded'))}")
                if "mimeType" in img:
                    print(f"    MIME: {img['mimeType']}")

    # Inspect images
    print(f"\n=== Images ===")
    for i, img in enumerate(gltf.get("images", [])):
        print(f"  Image {i}: {img.get('name', 'unnamed')}")
        if "uri" in img:
            print(f"    URI: {img['uri']}")
        if "mimeType" in img:
            print(f"    MIME: {img['mimeType']}")
        if "bufferView" in img:
            print(f"    BufferView: {img['bufferView']}")

    # Inspect meshes and primitives
    print(f"\n=== Meshes ===")
    for mesh_idx, mesh in enumerate(gltf.get("meshes", [])):
        print(f"  Mesh {mesh_idx}: {mesh.get('name', 'unnamed')}")
        for prim_idx, prim in enumerate(mesh.get("primitives", [])):
            print(f"    Primitive {prim_idx}:")
            print(f"      Attributes: {list(prim.get('attributes', {}).keys())}")
            if "material" in prim:
                print(f"      Material: {prim['material']}")
            if "indices" in prim:
                print(f"      Indices: accessor {prim['indices']}")

            # Check for UV coordinates
            attrs = prim.get("attributes", {})
            if "TEXCOORD_0" in attrs:
                print(f"      HAS TEXCOORD_0 (UV channel 0)")
                acc_idx = attrs["TEXCOORD_0"]
                acc = gltf["accessors"][acc_idx]
                print(f"        Accessor {acc_idx}: count={acc['count']}, type={acc['type']}, componentType={acc['componentType']}")
                # Read first few UV values
                if bin_data:
                    uvs = read_accessor_data(gltf, bin_data, acc_idx)
                    print(f"        First 5 UVs: {uvs[:5]}")
            else:
                print(f"      NO TEXCOORD_0 (no UV coordinates)")

            if "TEXCOORD_1" in attrs:
                print(f"      HAS TEXCOORD_1 (UV channel 1)")

            if "COLOR_0" in attrs:
                print(f"      HAS COLOR_0 (vertex colors)")

            if "NORMAL" in attrs:
                print(f"      HAS NORMAL")

            if "TANGENT" in attrs:
                print(f"      HAS TANGENT")

    # Inspect nodes
    print(f"\n=== Nodes ===")
    for i, node in enumerate(gltf.get("nodes", [])):
        print(f"  Node {i}: {node.get('name', 'unnamed')}")
        if "mesh" in node:
            print(f"    Mesh: {node['mesh']}")
        if "children" in node:
            print(f"    Children: {node['children']}")
        if "translation" in node:
            print(f"    Translation: {node['translation']}")
        if "rotation" in node:
            print(f"    Rotation: {node['rotation']}")
        if "scale" in node:
            print(f"    Scale: {node['scale']}")

    # Inspect accessors for position data
    print(f"\n=== Position Accessors ===")
    for acc_idx, acc in enumerate(gltf.get("accessors", [])):
        if acc.get("type") == "VEC3" and "bufferView" in acc:
            bv = gltf["bufferViews"][acc["bufferView"]]
            # Check if this is likely position data by looking at min/max
            if "min" in acc and "max" in acc:
                print(f"  Accessor {acc_idx}: count={acc['count']}, min={acc['min']}, max={acc['max']}")


def main():
    if len(sys.argv) < 2:
        print("Usage: python inspect_gltf.py <file.glb>")
        sys.exit(1)

    filepath = Path(sys.argv[1])
    if not filepath.exists():
        print(f"Error: File not found: {filepath}")
        sys.exit(1)

    try:
        inspect_gltf(filepath)
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()