#!/usr/bin/env python3
# AGENTS: before modifying this file, read AGENTS.md at the repository root.

"""
Rotate a GLB mesh by a specified angle around an axis, preserving textures.
Uses pygltflib for proper GLB handling with embedded textures.
"""

import sys
import numpy as np
from pygltflib import GLTF2


def rotate_mesh(input_path: str, output_path: str, angle_deg: float, axis: str) -> int:
    """Rotate a GLB mesh and save to output path, preserving textures."""
    print(f"Loading mesh from: {input_path}")
    
    # Load the GLB file
    gltf = GLTF2.load(input_path)
    
    # Get the rotation matrix
    angle_rad = np.radians(angle_deg)
    
    if axis.lower() == 'x':
        # Rotation around X-axis
        cos_a, sin_a = np.cos(angle_rad), np.sin(angle_rad)
        rotation_matrix = np.array([
            [1, 0, 0],
            [0, cos_a, -sin_a],
            [0, sin_a, cos_a]
        ], dtype=np.float32)
    elif axis.lower() == 'y':
        # Rotation around Y-axis
        cos_a, sin_a = np.cos(angle_rad), np.sin(angle_rad)
        rotation_matrix = np.array([
            [cos_a, 0, sin_a],
            [0, 1, 0],
            [-sin_a, 0, cos_a]
        ], dtype=np.float32)
    elif axis.lower() == 'z':
        # Rotation around Z-axis
        cos_a, sin_a = np.cos(angle_rad), np.sin(angle_rad)
        rotation_matrix = np.array([
            [cos_a, -sin_a, 0],
            [sin_a, cos_a, 0],
            [0, 0, 1]
        ], dtype=np.float32)
    else:
        print(f"Error: Unknown axis '{axis}'. Use 'x', 'y', or 'z'.")
        return 1
    
    print(f"Rotation: {angle_deg}° around {axis.upper()}-axis")
    
    # Get binary data
    binary_data = gltf.binary_blob()
    
    # Process each mesh
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
            
            # Apply rotation
            vertices = vertices @ rotation_matrix.T
            
            # Write back
            vertex_bytes = vertices.astype(np.float32).tobytes()
            binary_data = binary_data[:offset] + vertex_bytes + binary_data[offset + len(vertex_bytes):]
            
            # Update accessor min/max for frustum culling
            pos_accessor.min = vertices.min(axis=0).tolist()
            pos_accessor.max = vertices.max(axis=0).tolist()
            
            # Update normal accessor if present
            normal_accessor_idx = prim.attributes.NORMAL
            if normal_accessor_idx is not None:
                normal_accessor = gltf.accessors[normal_accessor_idx]
                bv = gltf.bufferViews[normal_accessor.bufferView]
                offset = bv.byteOffset + normal_accessor.byteOffset
                count = normal_accessor.count
                normal_bytes = binary_data[offset:offset + count * 12]
                normals = np.frombuffer(normal_bytes, dtype=np.float32).reshape(count, 3)
                
                # Transform normals (same rotation, no translation)
                normals = normals @ rotation_matrix.T
                
                # Write back
                normal_bytes = normals.astype(np.float32).tobytes()
                binary_data = binary_data[:offset] + normal_bytes + binary_data[offset + len(normal_bytes):]
    
    # Update binary blob
    gltf.set_binary_blob(binary_data)
    
    # Save the rotated mesh
    print(f"Saving rotated mesh to: {output_path}")
    gltf.save_binary(output_path)
    
    print("Rotation completed successfully!")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 5:
        print(f"Usage: {sys.argv[0]} <input.glb> <output.glb> <angle> <axis>")
        print(f"Example: {sys.argv[0]} mesh.glb rotated.glb 180 y")
        sys.exit(1)
    
    input_path = sys.argv[1]
    output_path = sys.argv[2]
    angle = float(sys.argv[3])
    axis = sys.argv[4]
    
    sys.exit(rotate_mesh(input_path, output_path, angle, axis))