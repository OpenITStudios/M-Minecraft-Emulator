"""Finite opt-in Ghast skin preview; exact Mojang source and model/UV checks.

The inputs are read-only. The caller's existing texture builder owns ZIP
construction; no files are installed into Minecraft profiles or worlds.
"""
from hashlib import sha1
import io
import json
import math
from pathlib import Path
import stat

from PIL import Image, UnidentifiedImageError

# Mojang bedrock-samples v1.26.30.5; no original assets are vendored.
SOURCE_TAG = "v1.26.30.5"
GHAST_PINS = {
    "entity/ghast.entity.json": "a04ef67eaf12674c321de7afdc6a44f11d904d7a",
    "models/entity/ghast.geo.json": "41a0bdc3905b4a5af8ef319902f794d722246c08",
    "render_controllers/ghast.render_controllers.json": "1d98f18a2c54f2025de51f73a059af2843bd9fc6",
    "textures/entity/ghast/ghast.texture_set.json": "521dfded43cc3c94630322c16be5f46d0298c6ec",
    "textures/entity/ghast/ghast_shooting.texture_set.json": "d2ef11224bedaa4b2d2a5c95fa3a8e36cd63fd2b",
    "textures/entity/ghast/ghast.png": "c473f667aa206b1721f88710cc9860be1a519719",
    "textures/entity/ghast/ghast_shooting.png": "0a052b80f5c4d21e527837a68fc7045bc3011765",
    "textures/entity/ghast/ghast_mers.tga": "e0172159e2ca0db827cce5799cf108f1243c9b80",
    "textures/entity/ghast/ghast_shooting.tga": "a7f989ed3c5e97cc52a910fd0f8b8af30f29596e",
    "textures/entity/ghast/ghast_shooting_mers.tga": "4078e70b3cac3e4cc522b42db8036899f3210bbe",
}
SKINS = ("ghast", "ghast_shooting")
IMAGES = (
    "textures/entity/ghast/ghast.png",
    "textures/entity/ghast/ghast_shooting.png",
    "textures/entity/ghast/ghast_mers.tga",
    "textures/entity/ghast/ghast_shooting.tga",
    "textures/entity/ghast/ghast_shooting_mers.tga",
)
MAX_SOURCE_FILE_BYTES = 128 * 1024


def git_blob_sha(raw: bytes) -> str:
    return sha1(b"blob " + str(len(raw)).encode("ascii") + b"\x00" + raw).hexdigest()


def _read_file(root: Path, relative: str, expected_sha: str) -> bytes:
    # Exact known relative paths only, but do not trust any symlink in the chain.
    current = root
    if not root.is_dir() or root.is_symlink() or getattr(root, "is_junction", lambda: False)():
        raise ValueError("Ghast resource_pack root is not an ordinary directory")
    parts = Path(relative).parts
    if not parts or any(part in (".", "..") for part in parts):
        raise ValueError("Unsafe Ghast source path")
    for index, part in enumerate(parts):
        current = current / part
        metadata = current.lstat()
        if stat.S_ISLNK(metadata.st_mode) or getattr(current, "is_junction", lambda: False)():
            raise ValueError("Linked Ghast source refused")
        if index != len(parts) - 1:
            if not stat.S_ISDIR(metadata.st_mode):
                raise ValueError("Ghast source has a non-directory parent")
        elif not stat.S_ISREG(metadata.st_mode) or metadata.st_size > MAX_SOURCE_FILE_BYTES:
            raise ValueError("Unsupported Ghast source size or type")
    raw = current.read_bytes()
    if len(raw) > MAX_SOURCE_FILE_BYTES or git_blob_sha(raw) != expected_sha:
        raise ValueError("Ghast source fingerprint mismatch: " + relative)
    return raw


def _json(raw: bytes) -> dict:
    try:
        value = json.loads(raw.decode("utf-8-sig"))
    except (ValueError, UnicodeError) as error:
        raise ValueError("Invalid pinned Ghast JSON") from error
    if not isinstance(value, dict):
        raise ValueError("Ghast metadata must be a JSON object")
    return value


def _verify_references(files: dict[str, bytes]) -> int:
    entity = _json(files["entity/ghast.entity.json"])
    geometry = _json(files["models/entity/ghast.geo.json"])
    controller = _json(files["render_controllers/ghast.render_controllers.json"])
    description = entity.get("minecraft:client_entity", {}).get("description", {})
    if (description.get("identifier") != "minecraft:ghast"
        or description.get("textures") != {
            "default": "textures/entity/ghast/ghast",
            "shooting": "textures/entity/ghast/ghast_shooting",
        }
        or description.get("geometry") != {"default": "geometry.ghast"}
        or description.get("render_controllers") != ["controller.render.ghast"]):
        raise ValueError("Unrecognized Ghast entity/skin references")
    r = controller.get("render_controllers", {}).get("controller.render.ghast", {})
    if (r.get("geometry") != "Geometry.default"
        or r.get("textures") != ["Array.skins[variable.ischarging]"]
        or r.get("arrays", {}).get("textures", {}).get("Array.skins")
            != ["Texture.default", "Texture.shooting"]):
        raise ValueError("Unrecognized Ghast skin render controller")
    for name in SKINS:
        t = _json(files[f"textures/entity/ghast/{name}.texture_set.json"])
        desc = t.get("minecraft:texture_set", {})
        if (desc.get("color") != name
            or desc.get("metalness_emissive_roughness_subsurface") != name + "_mers"):
            raise ValueError("Unrecognized Ghast color/PBR texture-set reference")
    models = geometry.get("minecraft:geometry")
    if not isinstance(models, list) or len(models) != 1:
        raise ValueError("Unknown Ghast geometry variants")
    model = models[0]
    d = model.get("description", {})
    if (d.get("identifier") != "geometry.ghast"
        or d.get("texture_width") != 64 or d.get("texture_height") != 32):
        raise ValueError("Ghast geometry does not use the 64x32 target atlas")
    uv_faces = 0
    bones = model.get("bones")
    if not isinstance(bones, list) or not bones:
        raise ValueError("Ghast has no inspectable cube geometry")
    for bone in bones:
        if "poly_mesh" in bone or "texture_meshes" in bone:
            raise ValueError("Unsupported Ghast mesh UV layout")
        for cube in bone.get("cubes", ()):
            faces = cube.get("uv")
            if not isinstance(faces, dict) or not faces:
                raise ValueError("Unknown Ghast cube UV layout")
            for uv in faces.values():
                pos, size = uv.get("uv"), uv.get("uv_size")
                if (not isinstance(pos, list) or not isinstance(size, list)
                    or len(pos) != 2 or len(size) != 2):
                    raise ValueError("Invalid Ghast face UV")
                if any(isinstance(v, bool) or not isinstance(v, (int, float))
                       or not math.isfinite(v) for v in pos + size):
                    raise ValueError("Invalid numeric Ghast UV")
                for a, b, limit in zip(pos, size, (64, 32)):
                    if min(a, a + b) < 0 or max(a, a + b) > limit:
                        raise ValueError("Ghast UV exceeds the target atlas")
                uv_faces += 1
    if not uv_faces:
        raise ValueError("No Ghast UV faces checked")
    return uv_faces


def _halve_image(raw: bytes, suffix: str) -> bytes:
    expected = "PNG" if suffix == ".png" else "TGA"
    if suffix == ".png":
        if not raw.startswith(b"\x89PNG\r\n\x1a\n") or len(raw) < 29 or raw[28] != 0:
            raise ValueError("Unsupported Ghast PNG encoding")
    with Image.open(io.BytesIO(raw)) as picture:
        if (picture.format != expected or picture.size != (128, 64)
            or getattr(picture, "n_frames", 1) != 1):
            raise ValueError("Unsupported Ghast texture dimensions or format")
        picture.load()
        # MERS texture channels encode material properties: nearest-neighbor
        # avoids inventing interpolated material flags between adjacent texels.
        selection = Image.Resampling.NEAREST if suffix == ".tga" else Image.Resampling.BOX
        smaller = picture.convert("RGBA").resize((64, 32), selection)
        destination = io.BytesIO()
        smaller.save(destination, format=expected)
    return destination.getvalue()


def compile_ghast(root: Path, pins: dict[str, str] = GHAST_PINS) -> tuple[dict[str, bytes], dict]:
    """Return only source-verified texture overrides; never mutate geometry."""
    if set(pins) != set(GHAST_PINS):
        raise ValueError("Unrecognized Ghast input manifest")
    files = {name: _read_file(root, name, digest) for name, digest in pins.items()}
    count = _verify_references(files)
    overrides = {path: _halve_image(files[path], Path(path).suffix) for path in IMAGES}
    return overrides, {
        "entity": "ghast",
        "skin_variants": 2,
        "image_overrides": len(overrides),
        "validated_uv_faces": count,
        "original_texels": len(overrides) * 128 * 64,
        "preview_texels": len(overrides) * 64 * 32,
        "texel_reduction_percent": 75,
        "hypothetical_rgba8_difference_bytes": len(overrides) * (128 * 64 - 64 * 32) * 4,
        "runtime_ram_saving": "NOT_MEASURED",
    }
