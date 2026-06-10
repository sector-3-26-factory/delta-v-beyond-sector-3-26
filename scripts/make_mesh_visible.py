#!/usr/bin/env python3
# AGENTS: before modifying this file, read AGENTS.md at the repository root.

"""
Enhanced GLB Mesh Processing Pipeline for Bevy Engine (v0.14)

This script performs the following corrections on glTF/glb 3D models:
1. Geometry Analysis & Transform Baking
2. Texture Transcoding (JPEG to PNG)
3. PBR Material Correction (Metallic/Roughness)
4. Tangent Generation for Normal Shader

Technical Approach:
- Uses pygltflib for loading and material manipulation
- Uses trimesh for geometry processing and tangent calculation
- Packs everything sequentially into a single Buffer 0 as required by GLB specification
"""

import sys
import json
import numpy as np
from io import BytesIO
from PIL import Image

import trimesh
from pygltflib import (
    GLTF2,
    Accessor,
    Asset,
    Buffer,
    BufferView,
    Image as GltfImage,
    Material as GltfMaterial,
    Mesh as GltfMesh,
    Node,
    Primitive,
    Scene as GltfScene,
    Texture,
    TextureInfo,
    PbrMetallicRoughness,
    Sampler,
)


def load_glb(filepath: str) -> tuple:
    """Load a glb file and return both GLTF2 object and trimesh scene."""
    gltf = GLTF2.load(filepath)
    scene = trimesh.load(filepath, file_type='glb')
    return gltf, scene


def get_uvs_from_gltf(gltf: GLTF2) -> np.ndarray:
    """Extract UV coordinates directly from GLTF to avoid trimesh flipping."""
    binary_data = gltf.binary_blob()
    if binary_data is None:
        return None
    
    # Find the UV accessor (VEC2 type)
    for i, accessor in enumerate(gltf.accessors):
        if accessor.type == "VEC2":
            bv = gltf.bufferViews[accessor.bufferView]
            uv_data = binary_data[bv.byteOffset:bv.byteOffset + bv.byteLength]
            uvs = np.frombuffer(uv_data, dtype=np.float32).reshape(-1, 2)
            return uvs
    
    return None


def calculate_tangents(vertices: np.ndarray, faces: np.ndarray, uvs: np.ndarray) -> np.ndarray:
    """Calculate tangent vectors for each vertex using the standard algorithm."""
    n_vertices = len(vertices)
    n_faces = len(faces)
    
    tangents = np.zeros((n_vertices, 4), dtype=np.float32)
    tangent_counts = np.zeros(n_vertices, dtype=np.int32)
    
    for i in range(n_faces):
        i0, i1, i2 = faces[i]
        v0, v1, v2 = vertices[i0], vertices[i1], vertices[i2]
        uv0, uv1, uv2 = uvs[i0], uvs[i1], uvs[i2]
        
        delta_pos1 = v1 - v0
        delta_pos2 = v2 - v0
        delta_uv1 = uv1 - uv0
        delta_uv2 = uv2 - uv0
        
        denom = delta_uv1[0] * delta_uv2[1] - delta_uv2[0] * delta_uv1[1]
        
        if abs(denom) < 1e-10:
            continue
        
        tangent = np.zeros(4, dtype=np.float32)
        tangent[0:3] = (delta_pos1 * delta_uv2[1] - delta_pos2 * delta_uv1[1]) / denom
        tangent[3] = 1.0  # Set W component to 1.0 (handedness required by glTF spec)
        
        tangents[i0] += tangent
        tangents[i1] += tangent
        tangents[i2] += tangent
        tangent_counts[i0] += 1
        tangent_counts[i1] += 1
        tangent_counts[i2] += 1
    
    for i in range(n_vertices):
        if tangent_counts[i] > 0:
            tangents[i] /= tangent_counts[i]
            length = np.linalg.norm(tangents[i][:3])
            if length > 0:
                tangents[i][:3] /= length
                tangents[i][3] = 1.0  # Keep handedness consistent
        else:
            tangents[i] = [1.0, 0.0, 0.0, 1.0] # Fallback unit tangent
    
    return tangents


def correct_transforms(gltf: GLTF2, scene: trimesh.Scene) -> dict:
    """Reset node transforms to identity."""
    for node in gltf.nodes:
        if node.mesh is None:
            continue
        node.scale = [1.0, 1.0, 1.0]
        node.rotation = [0.0, 0.0, 0.0, 1.0]
        node.translation = [0.0, 0.0, 0.0]
        if hasattr(node, 'matrix'):
            node.matrix = None
    
    aabb = scene.bounds
    print(f"  AABB: min={aabb[0]}, max={aabb[1]}")
    return {"aabb": aabb.tolist()}


def transcode_textures_to_png(gltf: GLTF2) -> list:
    """
    Transcode all images to PNG format.
    Returns list of (image_data, mime_type) tuples in the same order as input images.
    """
    if not gltf.images:
        return []
    
    binary_data = gltf.binary_blob()
    if binary_data is None:
        return []
    
    processed_images = []
    
    for img in gltf.images:
        if img.bufferView is not None:
            bv = gltf.bufferViews[img.bufferView]
            image_data = binary_data[bv.byteOffset:bv.byteOffset + bv.byteLength]
            if image_data:
                try:
                    pil_img = Image.open(BytesIO(image_data))
                    print(f"  Detected image format: {pil_img.format}")
                    if pil_img.format == 'JPEG' or pil_img.mode != 'RGB':
                        print(f"  Transcoding image to standard PNG")
                        pil_img = pil_img.convert('RGB')
                        png_buffer = BytesIO()
                        pil_img.save(png_buffer, format='PNG')
                        processed_images.append((png_buffer.getvalue(), "image/png"))
                    else:
                        processed_images.append((image_data, "image/png"))
                except Exception as e:
                    print(f"  Warning: Could not process image: {e}")
                    processed_images.append((None, None))
        else:
            processed_images.append((None, None))
    
    return processed_images


def correct_pbr_materials(gltf: GLTF2) -> None:
    """Correct metallic and roughness factors."""
    for mat in gltf.materials:
        if mat.pbrMetallicRoughness:
            pbr = mat.pbrMetallicRoughness
            if pbr.metallicFactor == 1.0:
                pbr.metallicFactor = 0.0
                print(f"  Set metallicFactor to 0.0")
            
            if pbr.roughnessFactor == 1.0:
                pbr.roughnessFactor = 0.5
                print(f"  Set roughnessFactor to 0.5")


def generate_tangents(gltf: GLTF2, scene: trimesh.Scene) -> np.ndarray:
    """Generate tangent vectors for meshes with normal textures."""
    geom = list(scene.geometry.values())[0]
    
    print("  Generating tangents from geometry data...")
    vertices = np.asarray(geom.vertices, dtype=np.float32)
    faces = np.asarray(geom.faces, dtype=np.int32)
    uvs = get_uvs_from_gltf(gltf)
    
    if uvs is None:
        print("  Warning: Could not extract UVs from GLTF")
        return None
    
    try:
        tangents = calculate_tangents(vertices, faces, uvs)
        print(f"  Generated {len(tangents)} tangent vectors")
        return tangents
    except Exception as e:
        print(f"  Warning: Could not generate tangents: {e}")
        return None


def process_mesh(input_path: str, output_path: str) -> int:
    """Main processing pipeline."""
    print(f"Loading mesh from: {input_path}")
    
    # Load the GLB file
    gltf, scene = load_glb(input_path)
    
    # Get geometry data
    geom = list(scene.geometry.values())[0]
    
    vertices = geom.vertices.astype(np.float32)
    faces = geom.faces.astype(np.int32)
    normals = geom.vertex_normals.astype(np.float32)
    uvs = get_uvs_from_gltf(gltf)
    
    if uvs is None:
        print("  Error: Could not extract UVs from GLTF")
        return 1
    
    # Step 1: Correct transforms
    print("\n=== Step 1: Geometry Analysis & Transform Baking ===")
    correct_transforms(gltf, scene)
    
    # Step 2: Transcode textures
    print("\n=== Step 2: Texture Transcoding (JPEG to PNG) ===")
    processed_images = transcode_textures_to_png(gltf)
    # Filter out None entries (failed processing)
    processed_images = [(img_data, mime_type) for img_data, mime_type in processed_images if img_data is not None]
    print(f"  Processed {len(processed_images)} texture(s)")
    
    # Step 3: Correct PBR materials
    print("\n=== Step 3: PBR Material Correction ===")
    correct_pbr_materials(gltf)
    
    # Step 4: Generate tangents
    print("\n=== Step 4: Tangent Generation ===")
    tangents = generate_tangents(gltf, scene)
    if tangents is None:
        tangents = np.zeros((len(vertices), 4), dtype=np.float32)
        tangents[:, 3] = 1.0
    
    # Step 5: Build new GLTF structure using pygltflib
    print("\n=== Step 5: Building GLTF Structure ===")
    
    # Create new GLTF2 object
    new_gltf = GLTF2()
    
    # Convert geometry data structures to raw bytes
    vertex_bytes = vertices.tobytes()
    normal_bytes = normals.tobytes()
    tangent_bytes = tangents.tobytes()
    uv_bytes = uvs.tobytes()
    face_bytes = faces.tobytes()
    
    # Pack geometry data sequentially into buffer 0 for standard GLB compliance
    geometry_data = vertex_bytes + normal_bytes + tangent_bytes + uv_bytes + face_bytes
    
    # Pack image data sequentially (each image in its own slot)
    image_data_list = []
    for img_data, _ in processed_images:
        image_data_list.append(img_data)
    image_data_all = b''.join(image_data_list)
    full_binary_blob = geometry_data + image_data_all
    
    # Declare the single binary buffer
    buffer = Buffer()
    buffer.byteLength = len(full_binary_blob)
    new_gltf.buffers.append(buffer)
    
    # Track byte offsets sequentially inside buffer 0
    current_offset = 0
    
    # 0: Vertex BufferView & Accessor (CRITICAL FIX: min/max added for Bevy Frustum Culling)
    bv_vertex = BufferView(buffer=0, byteOffset=current_offset, byteLength=len(vertex_bytes), target=34962)
    new_gltf.bufferViews.append(bv_vertex)
    
    min_vals = vertices.min(axis=0).tolist()
    max_vals = vertices.max(axis=0).tolist()
    acc_vertex = Accessor(bufferView=0, componentType=5126, count=len(vertices), type="VEC3", min=min_vals, max=max_vals)
    new_gltf.accessors.append(acc_vertex)
    current_offset += len(vertex_bytes)
    
    # 1: Normal BufferView & Accessor
    bv_normal = BufferView(buffer=0, byteOffset=current_offset, byteLength=len(normal_bytes), target=34962)
    new_gltf.bufferViews.append(bv_normal)
    acc_normal = Accessor(bufferView=1, componentType=5126, count=len(normals), type="VEC3")
    new_gltf.accessors.append(acc_normal)
    current_offset += len(normal_bytes)
    
    # 2: Tangent BufferView & Accessor
    bv_tangent = BufferView(buffer=0, byteOffset=current_offset, byteLength=len(tangent_bytes), target=34962)
    new_gltf.bufferViews.append(bv_tangent)
    acc_tangent = Accessor(bufferView=2, componentType=5126, count=len(tangents), type="VEC4")
    new_gltf.accessors.append(acc_tangent)
    current_offset += len(tangent_bytes)
    
    # 3: UV BufferView & Accessor
    bv_uv = BufferView(buffer=0, byteOffset=current_offset, byteLength=len(uv_bytes), target=34962)
    new_gltf.bufferViews.append(bv_uv)
    acc_uv = Accessor(bufferView=3, componentType=5126, count=len(uvs), type="VEC2")
    new_gltf.accessors.append(acc_uv)
    current_offset += len(uv_bytes)
    
    # 4: Face BufferView & Accessor
    bv_face = BufferView(buffer=0, byteOffset=current_offset, byteLength=len(face_bytes), target=34963)
    new_gltf.bufferViews.append(bv_face)
    acc_face = Accessor(bufferView=4, componentType=5125, count=len(faces) * 3, type="SCALAR")
    new_gltf.accessors.append(acc_face)
    current_offset += len(face_bytes)
    
    # 5+: Image BufferView Setup (each texture gets its own Image and BufferView)
    for i, (img_data, mime_type) in enumerate(processed_images):
        bv_image = BufferView(buffer=0, byteOffset=current_offset, byteLength=len(img_data))
        new_gltf.bufferViews.append(bv_image)
        
        img = GltfImage()
        img.bufferView = 5 + i  # BufferView index: 5 is first image, then 6, 7, etc.
        img.mimeType = mime_type
        new_gltf.images.append(img)
        
        current_offset += len(img_data)
    
    # Material Setup - preserve original texture connections
    pbr = PbrMetallicRoughness(metallicFactor=0.0, roughnessFactor=0.5)
    
    # Connect textures: baseColorTexture=0, metallicRoughnessTexture=1
    if len(processed_images) >= 1:
        pbr.baseColorTexture = TextureInfo(index=0)
    if len(processed_images) >= 2:
        pbr.metallicRoughnessTexture = TextureInfo(index=1)
    
    mat_new = GltfMaterial(pbrMetallicRoughness=pbr)
    new_gltf.materials.append(mat_new)
    
    if processed_images:
        new_gltf.samplers.append(Sampler(magFilter=9729, minFilter=9987, wrapS=10497, wrapT=10497))
        # Create a Texture for each image
        for i in range(len(processed_images)):
            new_gltf.textures.append(Texture(sampler=0, source=i))
    
    # Primitive Mapping
    prim = Primitive(
        material=0,
        mode=4,  # TRIANGLES
        indices=4,
        attributes={
            "POSITION": 0,
            "NORMAL": 1,
            "TANGENT": 2,
            "TEXCOORD_0": 3
        }
    )
    
    mesh = GltfMesh(primitives=[prim])
    new_gltf.meshes.append(mesh)
    
    # Rebuild Node Hierarchy
    new_gltf.nodes.append(Node(mesh=0, scale=[1.0, 1.0, 1.0], translation=[0.0, 0.0, 0.0]))
    new_gltf.scenes.append(GltfScene(nodes=[0]))
    new_gltf.scene = 0
    new_gltf.asset = Asset(version="2.0", generator="mesh_corrections.py")
    
    # Save optimized file structure
    print(f"\nSaving corrected mesh to: {output_path}")
    new_gltf.set_binary_blob(full_binary_blob)
    new_gltf.save_binary(output_path)
    
    print("\nMesh corrections completed successfully!")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input.glb> <output.glb>")
        sys.exit(1)
    
    sys.exit(process_mesh(sys.argv[1], sys.argv[2]))