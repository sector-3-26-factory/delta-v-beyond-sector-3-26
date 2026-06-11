#!/usr/bin/env python3
# AGENTS: before modifying this file, read AGENTS.md at the repository root.

"""
Calculate bounding box and collision shape from a GLB mesh file.
Outputs the results to stdout without modifying any files.

Usage:
    python calc_mesh_bounds.py <mesh.glb> [--collision-type box|sphere]
"""

import sys
import argparse
import numpy as np
import trimesh


def calculate_bounds_and_collision(mesh_path: str, collision_type: str = "box") -> tuple:
    """
    Calculate bounding box and collision shape from a GLB mesh.
    
    Returns:
        tuple: (bounding_box dict, collision_shape dict)
    """
    # Load mesh
    scene = trimesh.load(mesh_path, file_type='glb')
    
    if not scene.geometry:
        raise ValueError("No geometry found in mesh")
    
    # Get all vertices from all geometries
    all_vertices = []
    for geom in scene.geometry.values():
        all_vertices.append(geom.vertices)
    
    vertices = np.vstack(all_vertices)
    
    # Calculate bounding box
    min_coords = vertices.min(axis=0)
    max_coords = vertices.max(axis=0)
    
    bounding_box = {
        "min": {"x": float(min_coords[0]), "y": float(min_coords[1]), "z": float(min_coords[2])},
        "max": {"x": float(max_coords[0]), "y": float(max_coords[1]), "z": float(max_coords[2])}
    }
    
    # Calculate collision shape
    min_vec = np.array([bounding_box["min"]["x"], bounding_box["min"]["y"], bounding_box["min"]["z"]])
    max_vec = np.array([bounding_box["max"]["x"], bounding_box["max"]["y"], bounding_box["max"]["z"]])
    center = (max_vec + min_vec) / 2
    extents = (max_vec - min_vec) / 2
    
    if collision_type == "sphere":
        radius = float(extents.max())
        collision_shape = {
            "type": "sphere",
            "radius": radius,
            "offset": {"x": float(center[0]), "y": float(center[1]), "z": float(center[2])}
        }
    else:  # box
        collision_shape = {
            "type": "box",
            "half_extents": {"x": float(extents[0]), "y": float(extents[1]), "z": float(extents[2])},
            "offset": {"x": float(center[0]), "y": float(center[1]), "z": float(center[2])}
        }
    
    return bounding_box, collision_shape


def main():
    parser = argparse.ArgumentParser(
        description="Calculate bounding box and collision shape from a GLB mesh file."
    )
    parser.add_argument("mesh", help="Path to input GLB file")
    parser.add_argument("--collision-type", choices=["box", "sphere"], default="box",
                        help="Collision shape type (default: box)")
    
    args = parser.parse_args()
    
    try:
        bbox, collision = calculate_bounds_and_collision(args.mesh, args.collision_type)
        
        print(f"Mesh: {args.mesh}")
        print(f"Collision type: {args.collision_type}")
        print()
        print("Bounding Box:")
        print(f"  min: x={bbox['min']['x']:.6f}, y={bbox['min']['y']:.6f}, z={bbox['min']['z']:.6f}")
        print(f"  max: x={bbox['max']['x']:.6f}, y={bbox['max']['y']:.6f}, z={bbox['max']['z']:.6f}")
        print()
        print("Collision Shape:")
        for key, value in collision.items():
            if isinstance(value, dict):
                print(f"  {key}:")
                for k, v in value.items():
                    print(f"    {k}: {v}")
            else:
                print(f"  {key}: {value}")
        
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())