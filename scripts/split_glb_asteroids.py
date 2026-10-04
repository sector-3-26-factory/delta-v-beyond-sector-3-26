#!/usr/bin/env python3
# AGENTS: before modifying this file, read AGENTS.md at the repository root.

"""Split a multi-asteroid GLB into one GLB per asteroid, centred on its origin.

The source pack holds several asteroids in one file, all sharing a single
material and its textures. This tool writes one template directory per
asteroid, each containing:

    <out-dir>/mesh_<n>/mesh.glb      geometry only, centred on (0, 0, 0)
    <out-dir>/mesh_<n>/asteroid.json collision sphere + bounding box, no mass
    <out-dir>/textures/<asset>-*.jpg|png   written once, referenced by relative URI

Every asteroid spans the full 0..1 UV range of the same three textures, so
there is nothing to crop per asteroid. Embedding the textures in every output
would duplicate tens of megabytes once per asteroid, so they are extracted
once into `textures/` and each mesh.glb points at them with a relative URI.
Bevy's glTF loader resolves such URIs with RFC 1808 semantics relative to the
glTF's own directory and turns each into a separate `Image` asset, so the
shared file is decoded and uploaded to the GPU exactly once no matter how many
meshes reference it.

Usage:
    python scripts/split_glb_asteroids.py <pack.glb> [options]

Options:
    --out-dir DIR     destination root (default: assets/templates/asteroids/shared)
    --name NAME       base name for the extracted texture files
                      (default: the input file stem)
    --prefix NAME     directory name prefix for the per-asteroid output
                      (default: mesh)
    --force           overwrite an existing output directory
"""

import argparse
import json
import re
import shutil
import struct
import sys
from pathlib import Path

import numpy as np

GLB_MAGIC = 0x46546C67
JSON_CHUNK = 0x4E4F534A
BIN_CHUNK = 0x004E4942
FLOAT = 5126

# glTF componentType -> numpy dtype, for accessors that are read as floats.
COMPONENT_TYPES = {
    5120: np.int8,
    5121: np.uint8,
    5122: np.int16,
    5123: np.uint16,
    5125: np.uint32,
    5126: np.float32,
}

# Component count per accessor type.
TYPE_COMPONENTS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT2": 4, "MAT4": 16}

# Filename-safe name for each slot a material can put a texture in.
TEXTURE_SLOTS = {
    "base_color": "basecolor",
    "metallic_roughness": "metallic-roughness",
    "normal": "normal",
    "occlusion": "occlusion",
    "emissive": "emissive",
}

MIME_EXTENSIONS = {"image/jpeg": ".jpg", "image/png": ".png"}


class Glb:
    """A parsed GLB: the JSON document plus its single binary chunk."""

    def __init__(self, doc, blob):
        self.doc = doc
        self.blob = blob

    @classmethod
    def load(cls, path):
        with open(path, "rb") as handle:
            magic, version, total = struct.unpack("<III", handle.read(12))
            if magic != GLB_MAGIC:
                raise ValueError(f"{path} is not a GLB file")
            if version != 2:
                raise ValueError(f"{path} has unsupported GLB version {version}")

            json_chunk = None
            bin_chunk = b""
            while handle.tell() < total:
                header = handle.read(8)
                if len(header) < 8:
                    break
                length, chunk_type = struct.unpack("<II", header)
                data = handle.read(length)
                if chunk_type == JSON_CHUNK and json_chunk is None:
                    json_chunk = data
                elif chunk_type == BIN_CHUNK:
                    bin_chunk = data
            if json_chunk is None:
                raise ValueError(f"{path} has no JSON chunk")
        return cls(json.loads(json_chunk.decode("utf-8")), bin_chunk)

    def write(self, path):
        """Serialise back to a GLB with both chunks 4-byte aligned."""
        doc = json.dumps(self.doc, separators=(",", ":")).encode("utf-8")
        doc += b" " * ((4 - len(doc) % 4) % 4)
        blob = self.blob + b"\0" * ((4 - len(self.blob) % 4) % 4)

        total = 12 + 8 + len(doc) + 8 + len(blob)
        with open(path, "wb") as handle:
            handle.write(struct.pack("<III", GLB_MAGIC, 2, total))
            handle.write(struct.pack("<II", len(doc), JSON_CHUNK))
            handle.write(doc)
            handle.write(struct.pack("<II", len(blob), BIN_CHUNK))
            handle.write(blob)


def view_bytes(glb, buffer_view_index):
    """Return the exact bytes a bufferView covers."""
    view = glb.doc["bufferViews"][buffer_view_index]
    start = view.get("byteOffset", 0)
    return glb.blob[start : start + view["byteLength"]]


def accessor_layout(glb, accessor_index):
    """Where an accessor's data lives: (offset in buffer, stride, count, itemsize).

    `stride` is the distance between consecutive elements, which is wider than
    the element itself when the bufferView interleaves several attributes. The
    pack shares one bufferView between two asteroids, so an accessor must never
    be read or written as if it owned its whole view.
    """
    accessor = glb.doc["accessors"][accessor_index]
    if "bufferView" not in accessor:
        raise ValueError(f"accessor {accessor_index} is sparse and has no bufferView")

    view = glb.doc["bufferViews"][accessor["bufferView"]]
    components = TYPE_COMPONENTS[accessor["type"]]
    itemsize = np.dtype(COMPONENT_TYPES[accessor["componentType"]]).itemsize
    stride = view.get("byteStride") or itemsize * components
    offset = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    return offset, stride, accessor["count"], itemsize, components


def read_accessor(glb, accessor_index):
    """Read any accessor as a (count, components) array, dropping interleaving."""
    offset, stride, count, itemsize, components = accessor_layout(glb, accessor_index)
    accessor = glb.doc["accessors"][accessor_index]
    element = itemsize * components
    raw = glb.blob[offset : offset + stride * (count - 1) + element]
    if stride == element:
        return np.frombuffer(raw, dtype=COMPONENT_TYPES[accessor["componentType"]]).reshape(
            count, components
        )
    interleaved = np.frombuffer(raw, dtype=np.uint8).reshape(count, stride)
    tight = np.ascontiguousarray(interleaved[:, :element])
    return tight.view(COMPONENT_TYPES[accessor["componentType"]]).reshape(count, components)


def write_accessor(glb, accessor_index, values):
    """Write a (count, components) array back, preserving any interleaving."""
    offset, stride, count, itemsize, components = accessor_layout(glb, accessor_index)
    element = itemsize * components
    payload = np.ascontiguousarray(values, dtype=np.float32)

    if stride == element:
        glb.blob[offset : offset + payload.nbytes] = payload.tobytes()
        return

    interleaved = np.frombuffer(glb.blob, dtype=np.uint8)
    interleaved[offset : offset + stride * count].reshape(count, stride)[:, :element] = payload.tobytes().reshape(count, element)


def texture_role(material, texture_index):
    """Return the role a material gives to a texture, or None if it uses none."""
    pbr = material.get("pbrMetallicRoughness", {})
    slots = [
        ("base_color", pbr.get("baseColorTexture")),
        ("metallic_roughness", pbr.get("metallicRoughnessTexture")),
        ("normal", material.get("normalTexture")),
        ("occlusion", material.get("occlusionTexture")),
        ("emissive", material.get("emissiveTexture")),
    ]
    for role, info in slots:
        if info is not None and info.get("index") == texture_index:
            return role
    return None


def ordered_meshes(glb):
    """Meshes sorted by the number in their name: `Asteroid_no_2` before `_10`.

    The pack stores them out of order, so sorting on the trailing integer gives
    a stable mesh_1 -> Asteroid_no_1 mapping.
    """

    def key(index):
        name = glb.doc["meshes"][index]["name"]
        match = re.search(r"(\d+)$", name)
        return (0, int(match.group(1))) if match else (1, name)

    return sorted(range(len(glb.doc["meshes"])), key=key)


def build_template_json(minimum, maximum):
    """The asteroid template: a bounding sphere and the axis-aligned box.

    No `mass` key: per ADR-0058 mass belongs to the world entity, and a
    generated belt asteroid derives its mass from the region `density` and the
    radius the generator drew. `is_gravity_source` is omitted because the
    schema already defaults it to false (ADR-0039: defaults live in the
    schema only).
    """

    def quantity(value):
        return {"value": value, "unit": "m"}

    def corner(vector):
        return {
            "x": quantity(vector[0]),
            "y": quantity(vector[1]),
            "z": quantity(vector[2]),
        }

    half_extent = (np.asarray(maximum) - np.asarray(minimum)) / 2.0
    return {
        "entity_type": "asteroid",
        "collision_shape": {"type": "sphere", "radius": quantity(float(half_extent.max()))},
        "bounding_box": {"min": corner(minimum), "max": corner(maximum)},
    }


def extract_textures(glb, texture_dir, asset_name):
    """Write every embedded texture to disk once; return texture index -> file name."""
    materials = glb.doc["materials"]
    images = glb.doc["images"]
    textures = glb.doc["textures"]

    texture_dir.mkdir(parents=True, exist_ok=True)
    files = {}
    for texture_index, texture in enumerate(textures):
        image = images[texture["source"]]
        if "bufferView" not in image:
            raise ValueError(f"image {texture['source']} has no uri and no bufferView")

        role = None
        for mesh in glb.doc["meshes"]:
            for primitive in mesh["primitives"]:
                material_index = primitive.get("material")
                if material_index is not None:
                    role = role or texture_role(materials[material_index], texture_index)
        role = role or f"texture-{texture_index}"

        mime = image["mimeType"]
        file_name = f"{asset_name}-{TEXTURE_SLOTS[role]}{MIME_EXTENSIONS[mime]}"
        payload = view_bytes(glb, image["bufferView"])
        (texture_dir / file_name).write_bytes(payload)
        files[texture_index] = file_name
        print(f"texture {texture_index} -> textures/{file_name} ({len(payload)/1048576:.2f} MiB)")
    return files


def remap_texture_indices(value, order):
    """Rewrite every texture index in a material onto the reduced texture list.

    A texture reference is an object such as `{"index": 2, "texCoord": 0}`, and
    the same index also appears on nested properties such as
    `pbrMetallicRoughness.baseColorTexture`, so the whole material is walked
    rather than just its top level.
    """
    if isinstance(value, dict):
        if "index" in value and isinstance(value["index"], int):
            value = {**value, "index": order.index(value["index"])}
            return value
        return {key: remap_texture_indices(item, order) for key, item in value.items()}
    if isinstance(value, list):
        return [remap_texture_indices(item, order) for item in value]
    return value


def compact_geometry(glb, primitive, transform):
    """Rebuild the geometry of one primitive into its own buffer.

    The pack's buffer holds every asteroid plus every texture. Only what this
    primitive reads is kept, one tightly packed bufferView per accessor, so the
    output carries its own geometry and nothing else. Copying whole views would
    drag in a neighbour's vertices, because the pack interleaves two asteroids
    into the same view.

    `transform(accessor_index, values)` may return replacement values; it is
    used to move the positions onto the origin. Returns the new views,
    accessors, binary blob and the old-to-new accessor index mapping.
    """
    wanted = []
    if "indices" in primitive:
        wanted.append(primitive["indices"])
    for attribute in primitive["attributes"].values():
        if attribute not in wanted:
            wanted.append(attribute)

    views = []
    accessors = []
    chunks = []
    remap = {}
    offset = 0
    for position, accessor_index in enumerate(wanted):
        accessor = glb.doc["accessors"][accessor_index]
        source = glb.doc["bufferViews"][accessor["bufferView"]]
        values = read_accessor(glb, accessor_index)
        values = transform(accessor_index, values)
        payload = np.ascontiguousarray(values).tobytes()

        remap[accessor_index] = position
        views.append(
            {
                "buffer": 0,
                "byteOffset": offset,
                "byteLength": len(payload),
                "target": source.get("target"),
            }
        )
        chunks.append(payload)
        offset += len(payload)
        offset += (4 - len(payload) % 4) % 4

        # `byteOffset` and `byteStride` describe the layout inside the old,
        # shared view. The new view holds exactly this accessor's data, so both
        # offsets must go or the accessor reads past the end of the buffer.
        copied = {
            key: value
            for key, value in accessor.items()
            if key not in ("bufferView", "byteOffset")
        }
        copied["bufferView"] = position
        accessors.append(copied)

    return views, accessors, b"".join(chunks), remap


def main():
    parser = argparse.ArgumentParser(
        description="Split a multi-asteroid GLB into one centred GLB per asteroid."
    )
    parser.add_argument("pack", help="Path to the source GLB holding every asteroid")
    parser.add_argument(
        "--out-dir",
        default="assets/templates/asteroids/shared",
        help="Destination root (default: assets/templates/asteroids/shared)",
    )
    parser.add_argument(
        "--name",
        default=None,
        help="Base name for the extracted texture files (default: input file stem)",
    )
    parser.add_argument(
        "--prefix", default="mesh", help="Output directory prefix (default: mesh)"
    )
    parser.add_argument(
        "--force", action="store_true", help="Overwrite an existing output directory"
    )
    args = parser.parse_args()

    source = Path(args.pack)
    out_dir = Path(args.out_dir)
    asset_name = args.name or source.stem

    glb = Glb.load(source)
    if not glb.doc.get("textures"):
        raise ValueError("the pack has no textures to share")
    if any(len(mesh["primitives"]) != 1 for mesh in glb.doc["meshes"]):
        raise ValueError("every mesh must have exactly one primitive")

    if out_dir.exists() and any(out_dir.iterdir()):
        if not args.force:
            print(f"error: {out_dir} is not empty; pass --force to overwrite", file=sys.stderr)
            return 1
        shutil.rmtree(out_dir)

    texture_files = extract_textures(glb, out_dir / "textures", asset_name)

    node_of_mesh = {}
    for node in glb.doc["nodes"]:
        if "mesh" in node and node["mesh"] not in node_of_mesh:
            node_of_mesh[node["mesh"]] = node

    for output_index, mesh_index in enumerate(ordered_meshes(glb), start=1):
        mesh = glb.doc["meshes"][mesh_index]
        primitive = mesh["primitives"][0]
        material = glb.doc["materials"][primitive["material"]]

        position_accessor = primitive["attributes"]["POSITION"]
        bounds = {}

        def centre_positions(accessor_index, values):
            if accessor_index != position_accessor:
                return values
            minimum = values.min(axis=0).astype(np.float64)
            maximum = values.max(axis=0).astype(np.float64)
            centre = (minimum + maximum) / 2.0
            bounds["min"] = [round(float(v), 6) for v in (minimum - centre)]
            bounds["max"] = [round(float(v), 6) for v in (maximum - centre)]
            return (values.astype(np.float64) - centre).astype(np.float32)

        views, accessors, blob, remap = compact_geometry(glb, primitive, centre_positions)

        # The output numbers its accessors from zero, so the primitive's own
        # indices have to follow.
        output_primitive = dict(primitive)
        output_primitive["attributes"] = {
            name: remap[index] for name, index in primitive["attributes"].items()
        }
        if "indices" in primitive:
            output_primitive["indices"] = remap[primitive["indices"]]
        output_primitive["material"] = 0

        position_slot = remap[position_accessor]
        accessors[position_slot]["min"] = bounds["min"]
        accessors[position_slot]["max"] = bounds["max"]
        centred_min = bounds["min"]
        centred_max = bounds["max"]

        kept = set()
        pbr = material.get("pbrMetallicRoughness", {})
        for info in (
            pbr.get("baseColorTexture"),
            pbr.get("metallicRoughnessTexture"),
            material.get("normalTexture"),
            material.get("occlusionTexture"),
            material.get("emissiveTexture"),
        ):
            if info:
                kept.add(info["index"])
        order = sorted(kept)

        node = dict(node_of_mesh[mesh_index])
        node.pop("children", None)
        node.pop("matrix", None)
        # The output carries exactly one mesh, so the node must point at it.
        node["mesh"] = 0

        document = {
            "asset": glb.doc["asset"],
            "scene": 0,
            "scenes": [{"nodes": [0]}],
            "nodes": [node],
            "meshes": [{"name": mesh["name"], "primitives": [output_primitive]}],
            "materials": [remap_texture_indices(material, order)],
            "textures": [
                {
                    "sampler": glb.doc["textures"][index].get("sampler"),
                    "source": position,
                }
                for position, index in enumerate(order)
            ],
            "images": [
                {
                    "uri": f"../textures/{texture_files[index]}",
                    "mimeType": glb.doc["images"][glb.doc["textures"][index]["source"]]["mimeType"],
                }
                for index in order
            ],
            "accessors": accessors,
            "bufferViews": views,
            "buffers": [{"byteLength": len(blob)}],
        }
        if "samplers" in glb.doc:
            document["samplers"] = glb.doc["samplers"]

        directory = out_dir / f"{args.prefix}_{output_index}"
        directory.mkdir(parents=True, exist_ok=True)

        template = build_template_json(centred_min, centred_max)
        (directory / "asteroid.json").write_text(
            json.dumps(template, indent=4, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        Glb(document, blob).write(directory / "mesh.glb")

        size = (directory / "mesh.glb").stat().st_size
        print(
            f"{args.prefix}_{output_index}: {mesh['name']} -> {directory}/mesh.glb "
            f"({size/1024:.1f} KiB, bbox {centred_min} .. {centred_max}, "
            f"sphere r={template['collision_shape']['radius']['value']:.4f} m)"
        )

    return 0


if __name__ == "__main__":
    sys.exit(main())