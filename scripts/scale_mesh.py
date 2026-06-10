#!/usr/bin/env python3
# AGENTS: before modifying this file, read AGENTS.md at the repository root.

"""
Scale a GLB mesh so that the longest side of its bounding box equals a target size in meters.
Uses pygltflib for GLB handling and trimesh for geometry analysis.
"""

import sys
import argparse
import numpy as np
import trimesh
from pygltflib import GLTF2


def calculate_bounding_box_size(gltf: GLTF2, binary_data: bytes) -> float:
    """Calculate the longest side of the mesh bounding box in meters."""
    scene = trimesh.load(file_obj=BytesIO(), file_type='glb')
    # We need to load via trimesh to get bounds
    # Actually, let's use the binary data directly
    pass


def scale_mesh(input_path: str, output_path: str, target_size_m: float) -> int:
    """
    Scale a GLB mesh so the longest side of its bounding box equals target_size_m.
    
    Args:
        input_path: Path to input GLB file
        output_path: Path to output GLB file
        target_size_m: Target size in meters for the longest side of the bounding box
    
    Returns:
        0 on success, 1 on error
    """
    from io import BytesIO
    
    print(f"Loading mesh from: {input_path}")
    
    # Load the GLB file
    gltf = GLTF2.load(input_path)
    binary_data = gltf.binary_blob()
    
    if binary_data is None:
        print("Error: No binary data found in GLB file")
        return 1
    
    # Load with trimesh to get bounds
    scene = trimesh.load(input_path, file_type='glb')
    
    # Get the first geometry (assuming single mesh)
    if not scene.geometry:
        print("Error: No geometry found in mesh")
        return 1
    
    geom = list(scene.geometry.values())[0]
    vertices = geom.vertices
    
    # Calculate bounding box
    min_coords = vertices.min(axis=0)
    max_coords = vertices.max(axis=0)
    extents = max_coords - min_coords
    current_longest_side = extents.max()
    
    print(f"  Current bounding box extents: {extents}")
    print(f"  Current longest side: {current_longest_side:.6f} units")
    print(f"  Target longest side: {target_size_m:.6f} m")
    
    if current_longest_side <= 0:
        print("Error: Invalid mesh - zero or negative extent")
        return 1
    
    # Calculate scale factor
    scale_factor = target_size_m / current_longest_side
    print(f"  Scale factor: {scale_factor:.6f}")
    
    # Process each mesh in the GLTF
    for mesh_idx, mesh in enumerate(gltf.meshes):
        for prim in mesh.primitives:
            if prim.mode != 4:  # Not TRIANGLES
                continue
            
            # Get position accessor
            pos_accessor_idx = prim.attributes.POSITION
            if pos_accessor_idx is None:
                continue
            
            pos_accessor = gltf.accessors[pos_accessor_idx]
            bv = gltf.bufferViews[pos_accessor.bufferView]
            
            # Read vertex data
            offset = bv.byteOffset + pos_accessor.byteOffset
            count = pos_accessor.count
            vertex_bytes = binary_data[offset:offset + count * 12]  # 3 floats * 4 bytes
            vertices = np.frombuffer(vertex_bytes, dtype=np.float32).reshape(count, 3)
            
            # Apply scaling
            vertices = vertices * scale_factor
            
            # Write back
            vertex_bytes = vertices.astype(np.float32).tobytes()
            binary_data = binary_data[:offset] + vertex_bytes + binary_data[offset + len(vertex_bytes):]
            
            # Update accessor min/max for frustum culling
            pos_accessor.min = vertices.min(axis=0).tolist()
            pos_accessor.max = vertices.max(axis=0).tolist()
            
            # Update normal accessor if present (normals don't need scaling, just re-normalization)
            normal_accessor_idx = prim.attributes.NORMAL
            if normal_accessor_idx is not None:
                normal_accessor = gltf.accessors[normal_accessor_idx]
                bv = gltf.bufferViews[normal_accessor.bufferView]
                offset = bv.byteOffset + normal_accessor.byteOffset
                count = normal_accessor.count
                normal_bytes = binary_data[offset:offset + count * 12]
                normals = np.frombuffer(normal_bytes, dtype=np.float32).reshape(count, 3)
                
                # Normals don't scale, but we need to ensure they're still normalized
                # (rotation/scaling shouldn't affect them, but let's be safe)
                norms = np.linalg.norm(normals, axis=1, keepdims=True)
                norms = np.where(norms > 0, norms, 1)  # Avoid division by zero
                normals = normals / norms
                
                # Write back
                normal_bytes = normals.astype(np.float32).tobytes()
                binary_data = binary_data[:offset] + normal_bytes + binary_data[offset + len(normal_bytes):]
    
    # Update binary blob
    gltf.set_binary_blob(binary_data)
    
    # Save the scaled mesh
    print(f"Saving scaled mesh to: {output_path}")
    gltf.save_binary(output_path)
    
    print("Scaling completed successfully!")
    return 0


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Scale a GLB mesh so the longest side of its bounding box equals a target size in meters."
    )
    parser.add_argument(
        "input",
        help="Path to input GLB file"
    )
    parser.add_argument(
        "output",
        help="Path to output GLB file"
    )
    parser.add_argument(
        "size",
        type=float,
        help="Target size in meters for the longest side of the mesh bounding box"
    )
    
    args = parser.parse_args()
    
    if args.size <= 0:
        print("Error: Size must be a positive number")
        sys.exit(1)
    
    sys.exit(scale_mesh(args.input, args.output, args.size))