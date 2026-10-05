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


SPECULAR_GLOSSINESS = "KHR_materials_pbrSpecularGlossiness"


def convert_specular_glossiness(material, roughness_floor):
    """Rewrite a deprecated specular/glossiness material as core metallic/roughness.

    `KHR_materials_pbrSpecularGlossiness` is not core glTF and this repository
    lists it in CREDITS.md among the features that are converted rather than
    kept. The diffuse map becomes the base colour map and the glossiness
    becomes a roughness.

    Glossiness 1.0 maps to roughness 0.0, which is a mirror, and that single
    value is held at `roughness_floor`. A glossiness authored alongside a
    specular factor of zero means the artist suppressed the specular highlight,
    not that the surface was polished, and metallic/roughness cannot express
    that exactly: a dielectric keeps a 4% reflectance the original had turned
    off. Every other glossiness is converted literally, so the floor does not
    flatten a surface that was already matte.

    Returns the new material, or the original when no conversion applies.
    """
    extensions = material.get("extensions") or {}
    if SPECULAR_GLOSSINESS not in extensions:
        return material

    source = extensions[SPECULAR_GLOSSINESS]
    glossiness = float(source.get("glossinessFactor", 1.0))
    pbr = {"metallicFactor": 0.0}
    if "diffuseTexture" in source:
        pbr["baseColorTexture"] = dict(source["diffuseTexture"])
    if "diffuseFactor" in source:
        pbr["baseColorFactor"] = list(source["diffuseFactor"])
    pbr["roughnessFactor"] = roughness_floor if glossiness >= 1.0 else 1.0 - glossiness

    converted = {"name": material.get("name"), "pbrMetallicRoughness": pbr}
    for key in ("normalTexture", "occlusionTexture", "emissiveTexture", "doubleSided", "alphaMode"):
        if key in material:
            converted[key] = material[key]
    return converted


def mesh_slug(name):
    """A short, stable file-name fragment for a mesh: `AST_01_LOD0` -> `01`."""
    match = re.search(r"\d+", name)
    return f"{int(match.group(0)):02d}" if match else re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")


def ordered_meshes(glb):
    """Meshes sorted by the numbers in their name, so `_2` precedes `_10`.

    Every Sketchfab name ends in `_0`, so the trailing integer alone cannot
    order anything; the whole sequence of numbers is compared instead. That
    puts `Asteroid_no_1` .. `_10` and `AST_01` .. `AST_03` in number order.
    """

    def key(index):
        name = glb.doc["meshes"][index]["name"]
        numbers = tuple(int(part) for part in re.findall(r"\d+", name))
        return (numbers, name)

    return sorted(range(len(glb.doc["meshes"])), key=key)


def split_mesh_into_groups(glb, mesh_index, radius):
    """Expand one joined mesh into several, one per spatial cluster of shells.

    A pack may ship several bodies as a single joined mesh. Its shells are
    found through the triangle adjacency graph, because two triangles belong
    to the same shell when they share a vertex, and the shells are then
    grouped by how close their centroids are. An asteroid modelled as a
    cluster of separate shells therefore comes back out as one body rather
    than as its fragments.

    Each group is appended to the document as its own mesh and node, so the
    rest of the tool treats it like any other. Returns the new mesh indices.
    """
    primitive = glb.doc["meshes"][mesh_index]["primitives"][0]
    positions = read_accessor(glb, primitive["attributes"]["POSITION"])

    index_accessor = glb.doc["accessors"][primitive["indices"]]
    start = (
        index_accessor.get("byteOffset", 0)
        + glb.doc["bufferViews"][index_accessor["bufferView"]].get("byteOffset", 0)
    )
    triangles = np.frombuffer(
        glb.blob[start : start + index_accessor["count"] * 4], dtype=np.uint32
    ).reshape(-1, 3)

    # Shells: flood fill over triangles that share a vertex.
    by_vertex = {}
    for number, triangle in enumerate(triangles):
        for vertex in triangle:
            by_vertex.setdefault(int(vertex), []).append(number)

    shells = []
    visited = np.zeros(len(triangles), dtype=bool)
    for seed in range(len(triangles)):
        if visited[seed]:
            continue
        visited[seed] = True
        stack, members = [seed], []
        while stack:
            number = stack.pop()
            members.append(number)
            for vertex in triangles[number]:
                for neighbour in by_vertex[int(vertex)]:
                    if not visited[neighbour]:
                        visited[neighbour] = True
                        stack.append(neighbour)
        vertices = np.unique(triangles[members].ravel())
        shell_positions = positions[vertices]
        centroid = (shell_positions.max(axis=0) + shell_positions.min(axis=0)) / 2.0
        shells.append((centroid, np.array(members), vertices))

    # Group shells by centroid proximity: one group per shell to begin with,
    # then repeatedly merge the closest pair while they sit within `radius`.
    groups = [[index] for index in range(len(shells))]
    while True:
        closest = None
        for i in range(len(groups)):
            for j in range(i + 1, len(groups)):
                a = np.mean([shells[k][0] for k in groups[i]], axis=0)
                b = np.mean([shells[k][0] for k in groups[j]], axis=0)
                distance = float(np.linalg.norm(a - b))
                if distance < radius and (closest is None or distance < closest[0]):
                    closest = (distance, i, j)
        if closest is None:
            break
        _, i, j = closest
        groups[i] = groups[i] + groups[j]
        del groups[j]
    groups.sort(key=lambda g: -sum(len(shells[k][1]) for k in g))

    attributes = {}
    for semantic, accessor_index in primitive["attributes"].items():
        attributes[semantic] = read_accessor(glb, accessor_index)
        attributes[semantic + "::type"] = glb.doc["accessors"][accessor_index]["type"]

    created = []
    for number, group in enumerate(groups, start=1):
        members = np.concatenate([shells[k][1] for k in group])
        vertices = np.unique(np.concatenate([shells[k][2] for k in group]))
        # Only vertices in `vertices` are ever looked up, so a zero fill is
        # enough and uint32 cannot hold the -1 that would read as a sentinel.
        lookup = np.zeros(len(positions), dtype=np.uint32)
        lookup[vertices] = np.arange(len(vertices), dtype=np.uint32)
        indices = lookup[triangles[members].ravel()].reshape(-1, 3)

        blob = glb.blob
        offset = len(blob)

        def append(values):
            nonlocal blob, offset
            payload = np.ascontiguousarray(values).tobytes()
            view = {
                "buffer": 0,
                "byteOffset": offset,
                "byteLength": len(payload),
            }
            glb.doc["bufferViews"].append(view)
            glb.doc["accessors"].append(
                {
                    "bufferView": len(glb.doc["bufferViews"]) - 1,
                    "componentType": 5126 if values.dtype == np.float32 else 5125,
                    "count": int(values.shape[0]),
                    "type": "VEC3" if values.ndim == 2 else "SCALAR",
                }
            )
            blob = blob + payload + b"\0" * ((4 - len(payload) % 4) % 4)
            offset = len(blob)
            return len(glb.doc["accessors"]) - 1

        index_accessor_index = append(indices.ravel().astype(np.uint32))
        primitive_attributes = {"POSITION": append(positions[vertices].astype(np.float32))}
        for semantic in primitive["attributes"]:
            if semantic == "POSITION":
                continue
            values = attributes[semantic][vertices]
            accessor_index = append(values.astype(np.float32))
            glb.doc["accessors"][accessor_index]["type"] = attributes[semantic + "::type"]
            primitive_attributes[semantic] = accessor_index

        glb.doc["meshes"].append(
            {
                "name": f"{glb.doc['meshes'][mesh_index]['name']}_group_{number}",
                "primitives": [
                    {
                        "attributes": primitive_attributes,
                        "indices": index_accessor_index,
                        "material": primitive.get("material"),
                        "mode": primitive.get("mode", 4),
                    }
                ],
            }
        )
        new_mesh = len(glb.doc["meshes"]) - 1
        glb.doc["nodes"].append(
            {"name": glb.doc["meshes"][new_mesh]["name"], "mesh": new_mesh}
        )
        glb.doc["buffers"][0]["byteLength"] = len(blob)
        glb.blob = blob
        created.append(new_mesh)

    print(
        f"mesh '{glb.doc['meshes'][mesh_index]['name']}': "
        f"{len(shells)} shells -> {len(created)} bodies "
        f"({[sum(len(shells[k][1]) for k in group) for group in groups]} triangles)"
    )
    return created


def build_template_json(minimum, maximum, entity_type):
    """The body template: a bounding sphere and the axis-aligned box.

    No `mass` key: per ADR-0058 mass belongs to the world entity. Fields the
    schema already defaults are omitted (ADR-0039: defaults live in the
    schema only), so an asteroid leaves out `is_gravity_source` and a moon
    leaves out both `is_gravity_source` and `animations_enabled`.
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
        "entity_type": entity_type,
        "collision_shape": {"type": "sphere", "radius": quantity(float(half_extent.max()))},
        "bounding_box": {"min": corner(minimum), "max": corner(maximum)},
    }


def extract_textures(glb, texture_dir, asset_name, materials, order):
    """Write the shared textures to disk once; return texture index -> file name.

    `materials` are the converted materials, so a texture belonging to
    `KHR_materials_pbrSpecularGlossiness` is named by its new slot rather than
    by the extension.

    Only a texture used by more than one mesh is written out. A texture that
    belongs to a single mesh is embedded in that mesh's own buffer instead, so
    extracting it would move the same bytes into a neighbouring directory
    without saving any space.
    """
    images = glb.doc["images"]
    textures = glb.doc["textures"]

    owners = {}
    roles = {}
    for position, mesh_index in enumerate(order, start=1):
        mesh = glb.doc["meshes"][mesh_index]
        for primitive in mesh["primitives"]:
            material_index = primitive.get("material")
            if material_index is None:
                continue
            material = materials[material_index]
            pbr = material.get("pbrMetallicRoughness", {})
            # The base colour and metallic/roughness maps sit inside
            # pbrMetallicRoughness; the rest sit on the material itself.
            slots = [pbr.get(key) for key in ("baseColorTexture", "metallicRoughnessTexture")]
            slots += [material.get(key) for key in ("normalTexture", "occlusionTexture", "emissiveTexture")]
            for info in slots:
                if not info:
                    continue
                index = info["index"]
                owners.setdefault(index, []).append(position)
                roles.setdefault(index, texture_role(material, index))

    files = {}
    for texture_index, texture in enumerate(textures):
        used_by = owners.get(texture_index, [])
        if len(used_by) < 2:
            print(f"texture {texture_index} -> embedded in its own mesh.glb")
            continue

        image = images[texture["source"]]
        if "bufferView" not in image:
            raise ValueError(f"image {texture['source']} has no uri and no bufferView")
        role = TEXTURE_SLOTS.get(roles.get(texture_index), f"texture-{texture_index}")
        file_name = f"{asset_name}-{role}{MIME_EXTENSIONS[image['mimeType']]}"
        payload = view_bytes(glb, image["bufferView"])

        texture_dir.mkdir(parents=True, exist_ok=True)
        (texture_dir / file_name).write_bytes(payload)
        files[texture_index] = file_name
        print(
            f"texture {texture_index} -> textures/{file_name} "
            f"({len(payload)/1048576:.2f} MiB, shared by {len(used_by)} meshes)"
        )
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
        "--start-index",
        type=int,
        default=1,
        help="Number the first output directory mesh_<start-index> (default: 1). "
        "Use it to add a pack to a directory that already holds templates.",
    )
    parser.add_argument(
        "--roughness-floor",
        type=float,
        default=0.0,
        help="Lowest roughness produced when converting "
        "KHR_materials_pbrSpecularGlossiness (default: 0.0, a literal conversion)",
    )
    parser.add_argument(
        "--template",
        choices=("asteroid", "moon"),
        default="asteroid",
        help="Body type: writes asteroid.json or moon.json (default: asteroid)",
    )
    parser.add_argument(
        "--cluster-radius",
        type=float,
        default=0.0,
        help="Split a joined mesh into one body per cluster of shells whose "
        "centroids lie within this distance. 0 (default) keeps each mesh whole.",
    )
    parser.add_argument(
        "--force", action="store_true", help="Overwrite output that already exists"
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

    # Converted up front, so texture naming and the written material both see
    # the core slots rather than the deprecated extension.
    materials = [
        convert_specular_glossiness(material, args.roughness_floor)
        for material in glb.doc["materials"]
    ]

    order = ordered_meshes(glb)
    if args.cluster_radius > 0.0:
        # Snapshot the mesh list first: splitting appends the new bodies to it.
        order = []
        for mesh_index in list(ordered_meshes(glb)):
            order += split_mesh_into_groups(glb, mesh_index, args.cluster_radius)

    # Texture ownership is decided over the meshes that will actually be
    # written. Running this before the shell split would see one owner per
    # texture and embed each copy into every body.
    texture_files = extract_textures(
        glb, out_dir / "textures", asset_name, materials, order
    )

    node_of_mesh = {}
    for node in glb.doc["nodes"]:
        if "mesh" in node and node["mesh"] not in node_of_mesh:
            node_of_mesh[node["mesh"]] = node

    for offset, mesh_index in enumerate(order):
        output_index = args.start_index + offset
        directory = out_dir / f"{args.prefix}_{output_index}"
        if directory.exists():
            if not args.force:
                print(
                    f"error: {directory} exists; pass --force to overwrite",
                    file=sys.stderr,
                )
                return 1
            shutil.rmtree(directory)

        mesh = glb.doc["meshes"][mesh_index]
        primitive = mesh["primitives"][0]
        material = materials[primitive["material"]]

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

        # A texture this mesh has to itself goes back into its own buffer, the
        # same way every other mesh.glb in the repository stores its images. A
        # texture several meshes share is written to `textures/` instead,
        # because embedding it would copy it once per mesh.
        images = []
        parts = [blob]
        offset = len(blob)
        for index in order:
            source = glb.doc["textures"][index]["source"]
            mime = glb.doc["images"][source]["mimeType"]
            if index in texture_files:
                images.append(
                    {"uri": f"../textures/{texture_files[index]}", "mimeType": mime}
                )
                continue
            payload = view_bytes(glb, glb.doc["images"][source]["bufferView"])
            padding = (4 - len(payload) % 4) % 4
            views.append(
                {"buffer": 0, "byteOffset": offset, "byteLength": len(payload)}
            )
            parts.append(payload)
            parts.append(b"\0" * padding)
            offset += len(payload) + padding
            images.append({"bufferView": len(views) - 1, "mimeType": mime})
        blob = b"".join(parts)

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
            "images": images,
            "accessors": accessors,
            "bufferViews": views,
            "buffers": [{"byteLength": len(blob)}],
        }
        if "samplers" in glb.doc:
            document["samplers"] = glb.doc["samplers"]

        directory.mkdir(parents=True, exist_ok=True)

        template = build_template_json(centred_min, centred_max, args.template)
        template_file = "moon.json" if args.template == "moon" else "asteroid.json"
        (directory / template_file).write_text(
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