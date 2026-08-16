#!/usr/bin/env python3
# AGENTS: before modifying this file, read AGENTS.md at the repository root.
"""
Normalize glTF/GLB mesh to center at origin.

This script reads a glTF or GLB file, computes the bounding box of all
mesh geometry (in local space, accounting for node transforms), and
translates all vertex positions so the center is at (0, 0, 0).

Usage:
    python scripts/normalize_mesh.py assets/templates/planets/earth/mesh.glb
    python scripts/normalize_mesh.py assets/templates/ships/space-fighter-comrade1280/mesh.glb --dry-run
"""

import argparse
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


def mat4_mult(a: list, b: list) -> list:
    """Multiply two 4x4 matrices (row-major)."""
    result = [[0.0] * 4 for _ in range(4)]
    for i in range(4):
        for j in range(4):
            result[i][j] = sum(a[i][k] * b[k][j] for k in range(4))
    return result


def mat4_vec4_mult(m: list, v: list) -> list:
    """Multiply 4x4 matrix by 4D vector."""
    return [sum(m[i][j] * v[j] for j in range(4)) for i in range(4)]


def node_to_matrix(node: dict) -> list:
    """Convert glTF node to 4x4 row-major matrix."""
    import math

    if "matrix" in node:
        m = node["matrix"]
        return [
            [m[0], m[4], m[8], m[12]],
            [m[1], m[5], m[9], m[13]],
            [m[2], m[6], m[10], m[14]],
            [m[3], m[7], m[11], m[15]],
        ]

    m = [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]]

    if "translation" in node:
        t = node["translation"]
        m[0][3] = t[0]
        m[1][3] = t[1]
        m[2][3] = t[2]

    if "rotation" in node:
        x, y, z, w = node["rotation"]
        norm = math.sqrt(x * x + y * y + z * z + w * w)
        x, y, z, w = x / norm, y / norm, z / norm, w / norm
        R = [
            [1 - 2 * y * y - 2 * z * z, 2 * x * y - 2 * z * w, 2 * x * z + 2 * y * w, 0],
            [2 * x * y + 2 * z * w, 1 - 2 * x * x - 2 * z * z, 2 * y * z - 2 * x * w, 0],
            [2 * x * z - 2 * y * w, 2 * y * z + 2 * x * w, 1 - 2 * x * x - 2 * y * y, 0],
            [0, 0, 0, 1],
        ]
        m = mat4_mult(m, R)

    if "scale" in node:
        s = node["scale"]
        for i in range(3):
            for j in range(3):
                m[i][j] *= s[j]

    return m


def compute_world_matrices(nodes: list) -> list:
    """Compute world matrix for each node."""
    world_matrices = [None] * len(nodes)

    def compute(node_idx: int, parent_matrix: list | None):
        local = node_to_matrix(nodes[node_idx])
        world = mat4_mult(parent_matrix, local) if parent_matrix else local
        world_matrices[node_idx] = world
        if "children" in nodes[node_idx]:
            for child in nodes[node_idx]["children"]:
                compute(child, world)

    # Find root nodes (nodes not referenced as children)
    is_child = [False] * len(nodes)
    for node in nodes:
        if "children" in node:
            for child in node["children"]:
                is_child[child] = True

    for i, child_flag in enumerate(is_child):
        if not child_flag:
            compute(i, None)

    return world_matrices


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


def write_accessor_data(gltf: dict, bin_data: bytearray, accessor_idx: int, data: list):
    """Write vertex data to accessor."""
    accessor = gltf["accessors"][accessor_idx]
    buffer_view = gltf["bufferViews"][accessor["bufferView"]]

    byte_offset = buffer_view.get("byteOffset", 0)
    if "byteOffset" in accessor:
        byte_offset += accessor["byteOffset"]

    component_type = accessor["componentType"]
    count = accessor["count"]
    type_str = accessor["type"]

    type_sizes = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}
    comp_size = type_sizes.get(component_type, 4)

    type_counts = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT2": 4, "MAT3": 9, "MAT4": 16}
    comp_count = type_counts.get(type_str, 1)

    stride = buffer_view.get("byteStride", comp_size * comp_count)

    for i in range(count):
        offset = byte_offset + i * stride
        vertex = data[i]
        if comp_count == 1:
            vertex = [vertex]
        for j in range(comp_count):
            comp_offset = offset + j * comp_size
            if component_type == 5126:  # FLOAT
                bin_data[comp_offset : comp_offset + 4] = struct.pack("<f", vertex[j])
            elif component_type == 5125:  # UNSIGNED_INT
                bin_data[comp_offset : comp_offset + 4] = struct.pack("<I", vertex[j])
            elif component_type == 5123:  # UNSIGNED_SHORT
                bin_data[comp_offset : comp_offset + 2] = struct.pack("<H", vertex[j])


def mat4_inverse(m: list) -> list:
    """Invert a 4x4 affine matrix (rotation + translation + scale)."""
    # Extract upper 3x3 (rotation * scale) and translation
    M = [[m[i][j] for j in range(3)] for i in range(3)]
    t = [m[0][3], m[1][3], m[2][3]]

    # Compute scale factors from column lengths
    scale = [
        (M[0][0]**2 + M[1][0]**2 + M[2][0]**2)**0.5,
        (M[0][1]**2 + M[1][1]**2 + M[2][1]**2)**0.5,
        (M[0][2]**2 + M[1][2]**2 + M[2][2]**2)**0.5,
    ]

    # Normalize to get pure rotation matrix
    R = [[0.0]*3 for _ in range(3)]
    for i in range(3):
        for j in range(3):
            if scale[j] > 1e-10:
                R[i][j] = M[i][j] / scale[j]
            else:
                R[i][j] = 0.0

    # Inverse rotation is transpose
    R_inv = [[R[j][i] for j in range(3)] for i in range(3)]

    # Inverse scale
    scale_inv = [1.0/s if s > 1e-10 else 0.0 for s in scale]

    # Inverse upper 3x3: R^T * S^-1
    M_inv = [[0.0]*3 for _ in range(3)]
    for i in range(3):
        for j in range(3):
            M_inv[i][j] = R_inv[i][j] * scale_inv[j]

    # Inverse translation: -M_inv * t
    t_inv = [
        -(M_inv[0][0] * t[0] + M_inv[0][1] * t[1] + M_inv[0][2] * t[2]),
        -(M_inv[1][0] * t[0] + M_inv[1][1] * t[1] + M_inv[1][2] * t[2]),
        -(M_inv[2][0] * t[0] + M_inv[2][1] * t[1] + M_inv[2][2] * t[2]),
    ]

    # Reconstruct inverse matrix
    result = [[0.0] * 4 for _ in range(4)]
    for i in range(3):
        for j in range(3):
            result[i][j] = M_inv[i][j]
        result[i][3] = t_inv[i]
    result[3][3] = 1.0
    return result


def mat4_inverse_dir(m: list) -> list:
    """Invert only the rotation+scale part of a 4x4 matrix (for direction vectors)."""
    # Extract upper 3x3 (rotation * scale)
    M = [[m[i][j] for j in range(3)] for i in range(3)]

    # Compute scale factors from column lengths
    scale = [
        (M[0][0]**2 + M[1][0]**2 + M[2][0]**2)**0.5,
        (M[0][1]**2 + M[1][1]**2 + M[2][1]**2)**0.5,
        (M[0][2]**2 + M[1][2]**2 + M[2][2]**2)**0.5,
    ]

    # Normalize to get pure rotation matrix
    R = [[0.0]*3 for _ in range(3)]
    for i in range(3):
        for j in range(3):
            if scale[j] > 1e-10:
                R[i][j] = M[i][j] / scale[j]
            else:
                R[i][j] = 0.0

    # Inverse rotation is transpose
    R_inv = [[R[j][i] for j in range(3)] for i in range(3)]

    # Inverse scale
    scale_inv = [1.0/s if s > 1e-10 else 0.0 for s in scale]

    # Inverse upper 3x3: R^T * S^-1
    M_inv = [[0.0]*3 for _ in range(3)]
    for i in range(3):
        for j in range(3):
            M_inv[i][j] = R_inv[i][j] * scale_inv[j]

    # Return as 4x4 with no translation
    result = [[0.0] * 4 for _ in range(4)]
    for i in range(3):
        for j in range(3):
            result[i][j] = M_inv[i][j]
    result[3][3] = 1.0
    return result


def mat4_vec3_mult(m: list, v: list) -> list:
    """Multiply 4x4 matrix by 3D point (w=1, includes translation)."""
    return [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2] + m[0][3],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2] + m[1][3],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2] + m[2][3],
    ]


def mat4_vec3_mult_dir(m: list, v: list) -> list:
    """Multiply 4x4 matrix by 3D direction vector (w=0, no translation)."""
    return [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]


def normalize_mesh(filepath: Path, dry_run: bool = False) -> dict:
    """Normalize mesh so its bounding box center is at origin.

    This works by finding the scene root node(s) and applying a translation
    to move the entire scene's geometry to be centered at origin.
    This preserves all internal hierarchy and relationships.
    """
    print(f"Reading {filepath}...")
    gltf, bin_data = read_gltf(filepath)

    if bin_data is None:
        raise ValueError("External .bin files not supported; use .glb")

    bin_data = bytearray(bin_data)
    nodes = gltf["nodes"]

    # Compute world matrices for all nodes
    world_matrices = compute_world_matrices(nodes)

    # Collect all vertex positions in world space
    all_positions = []

    for mesh_idx, mesh in enumerate(gltf.get("meshes", [])):
        for prim in mesh.get("primitives", []):
            if "POSITION" not in prim["attributes"]:
                continue
            acc_idx = prim["attributes"]["POSITION"]

            # Find which node uses this mesh
            node_idx = None
            for i, node in enumerate(nodes):
                if node.get("mesh") == mesh_idx:
                    node_idx = i
                    break

            if node_idx is None:
                continue

            # Read positions
            positions = read_accessor_data(gltf, bin_data, acc_idx)

            # Transform to world space
            m = world_matrices[node_idx]
            for pos in positions:
                v = [pos[0], pos[1], pos[2], 1.0]
                world_v = mat4_vec4_mult(m, v)
                if world_v[3] != 0:
                    world_v = [world_v[0] / world_v[3], world_v[1] / world_v[3], world_v[2] / world_v[3]]
                all_positions.append(world_v[:3])

    if not all_positions:
        print("No position data found")
        return gltf

    # Compute overall bounding box center in world space
    min_x = min(p[0] for p in all_positions)
    max_x = max(p[0] for p in all_positions)
    min_y = min(p[1] for p in all_positions)
    max_y = max(p[1] for p in all_positions)
    min_z = min(p[2] for p in all_positions)
    max_z = max(p[2] for p in all_positions)

    world_center = [(min_x + max_x) / 2, (min_y + max_y) / 2, (min_z + max_z) / 2]

    print(f"  World bbox: [{min_x:.4f}, {min_y:.4f}, {min_z:.4f}] to [{max_x:.4f}, {max_y:.4f}, {max_z:.4f}]")
    print(f"  World center: [{world_center[0]:.4f}, {world_center[1]:.4f}, {world_center[2]:.4f}]")

    if abs(world_center[0]) < 1e-6 and abs(world_center[1]) < 1e-6 and abs(world_center[2]) < 1e-6:
        print("  Already centered at origin")
        return gltf

    # Find scene root nodes (nodes referenced by scenes)
    scene_root_nodes = set()
    for scene in gltf.get("scenes", []):
        for node_idx in scene.get("nodes", []):
            scene_root_nodes.add(node_idx)

    # If no explicit scene roots, use nodes that are not children of any other node
    if not scene_root_nodes:
        is_child = [False] * len(nodes)
        for node in nodes:
            if "children" in node:
                for child in node["children"]:
                    is_child[child] = True
        for i, child_flag in enumerate(is_child):
            if not child_flag:
                scene_root_nodes.add(i)

    print(f"  Scene root nodes: {sorted(scene_root_nodes)}")

    # Apply translation to each scene root node
    # We want: new_world_center = M_root * (P_local + T_root) = 0
    # So T_root = -M_root^-1 * world_center
    for root_idx in scene_root_nodes:
        world_m = world_matrices[root_idx]
        world_m_inv = mat4_inverse(world_m)
        local_translation = mat4_vec3_mult(world_m_inv, world_center)
        # Negate: we want to subtract the world center
        local_translation = [-local_translation[0], -local_translation[1], -local_translation[2]]

        print(f"  Root node {root_idx} ({nodes[root_idx].get('name', 'unnamed')}): local translation = [{local_translation[0]:.4f}, {local_translation[1]:.4f}, {local_translation[2]:.4f}]")

        # Apply translation to the node's transform
        node = nodes[root_idx]
        
        # If node has a matrix, we need to multiply it by a translation matrix
        if "matrix" in node:
            # Create translation matrix
            tx, ty, tz = local_translation
            trans_matrix = [
                [1, 0, 0, tx],
                [0, 1, 0, ty],
                [0, 0, 1, tz],
                [0, 0, 0, 1]
            ]
            # Multiply: new_matrix = old_matrix * trans_matrix (translation in local space)
            # Our matrices are row-major
            old_matrix = node_to_matrix(node)
            new_matrix = mat4_mult(old_matrix, trans_matrix)
            # Convert back to column-major for glTF
            m = new_matrix
            node["matrix"] = [
                m[0][0], m[1][0], m[2][0], m[3][0],
                m[0][1], m[1][1], m[2][1], m[3][1],
                m[0][2], m[1][2], m[2][2], m[3][2],
                m[0][3], m[1][3], m[2][3], m[3][3],
            ]
            # Remove TRS if present (matrix takes precedence)
            node.pop("translation", None)
            node.pop("rotation", None)
            node.pop("scale", None)
        else:
            # Use TRS components
            if "translation" not in node:
                node["translation"] = [0.0, 0.0, 0.0]
            node["translation"][0] += local_translation[0]
            node["translation"][1] += local_translation[1]
            node["translation"][2] += local_translation[2]

    if not dry_run:
        print(f"Writing {filepath}...")
        write_glb(filepath, gltf, bytes(bin_data))
        print("Done!")
    else:
        print("Dry run - no changes written")

    return gltf


def main():
    parser = argparse.ArgumentParser(description="Normalize glTF/GLB mesh to center at origin")
    parser.add_argument("filepath", type=Path, help="Path to .glb or .gltf file")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be done without writing")
    args = parser.parse_args()

    if not args.filepath.exists():
        print(f"Error: File not found: {args.filepath}")
        sys.exit(1)

    try:
        normalize_mesh(args.filepath, args.dry_run)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()