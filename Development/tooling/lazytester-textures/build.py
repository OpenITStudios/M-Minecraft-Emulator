#!/usr/bin/env python3
"""Explicit offline 8x8 block-texture preview; never accesses game profiles."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import stat
import sys
import zipfile

from PIL import Image, UnidentifiedImageError

SOURCE_TAG = "v1.26.30.5"
# Git tree SHA of Mojang/bedrock-samples resource_pack/textures/blocks at SOURCE_TAG.
SOURCE_BLOCK_TREE = "6a811ec148ce4af37981f1055e8b678f7d11905e"
MAX_FILES = 4_096
MAX_SOURCE_BYTES = 16 * 1024 * 1024
MAX_FILE_BYTES = 256 * 1024
MAX_TEXTURES = 1_300
NAME = re.compile(r"^[A-Za-z0-9_.-]+$")
MANIFEST = {
    "format_version": 2,
    "header": {
        "name": "LazyTester Idle Texture 8x (Preview)",
        "description": "Optional 16x16 to 8x8 block-albedo preview. Manual activation only; RAM unverified.",
        "uuid": "b7e96ea6-f23b-4b86-9282-18c7439c14fa",
        "version": [0, 1, 0],
        "min_engine_version": [1, 26, 10],
    },
    "modules": [{
        "type": "resources", "uuid": "248d986d-f860-42c7-84a0-e746c72c2e89",
        "version": [0, 1, 0],
    }],
}


def git_object(kind: bytes, body: bytes) -> str:
    return hashlib.sha1(kind + b" " + str(len(body)).encode("ascii") + b"\0" + body).hexdigest()


def _plain(path: Path, directory: bool) -> None:
    info = path.lstat()
    if path.is_symlink() or getattr(path, "is_junction", lambda: False)():
        raise ValueError(f"Linked source refused: {path.name}")
    expected = stat.S_ISDIR if directory else stat.S_ISREG
    if not expected(info.st_mode):
        raise ValueError(f"Unsupported source entry: {path.name}")


def pinned_tree(directory: Path) -> tuple[str, dict[str, bytes]]:
    """Recreate Git tree SHA from actual bytes; collect only PNG block sources."""
    _plain(directory, True)
    state = {"files": 0, "bytes": 0}
    images: dict[str, bytes] = {}

    def visit(folder: Path, relative: str) -> str:
        entries: list[tuple[bytes, bytes, bytes]] = []
        with os.scandir(folder) as iterator:
            children = list(iterator)
        for child in children:
            name = child.name
            if not NAME.fullmatch(name) or name in (".", ".."):
                raise ValueError(f"Unsafe source entry name: {name!r}")
            path = folder / name
            info = path.lstat()
            if path.is_symlink() or getattr(path, "is_junction", lambda: False)():
                raise ValueError(f"Linked source refused: {name}")
            if stat.S_ISDIR(info.st_mode):
                sha = visit(path, f"{relative}{name}/")
                entries.append((name.encode() + b"/", b"40000", bytes.fromhex(sha)))
            elif stat.S_ISREG(info.st_mode):
                state["files"] += 1
                state["bytes"] += info.st_size
                if (state["files"] > MAX_FILES or state["bytes"] > MAX_SOURCE_BYTES
                        or info.st_size > MAX_FILE_BYTES):
                    raise ValueError("Pinned source exceeds read limits")
                raw = path.read_bytes()
                if len(raw) != info.st_size:
                    raise ValueError("Source changed while reading")
                sha = git_object(b"blob", raw)
                entries.append((name.encode(), b"100644", bytes.fromhex(sha)))
                if name.endswith(".png"):
                    images[f"{relative}{name}"] = raw
            else:
                raise ValueError(f"Nonregular source entry: {name}")
        # Git orders a tree as though directory names end with a slash.
        entries.sort(key=lambda e: e[0])
        body = b"".join(mode + b" " + name.rstrip(b"/") + b"\0" + digest
                        for name, mode, digest in entries)
        return git_object(b"tree", body)

    result = visit(directory, "")
    return result, images


def downsample_png(raw: bytes) -> bytes | None:
    """Only single-frame, un-interlaced 16x16 albedo; preserve alpha via BOX."""
    if not raw.startswith(b"\x89PNG\r\n\x1a\n"):
        raise ValueError("Input is not a PNG")
    try:
        with Image.open(io.BytesIO(raw)) as image:
            if image.format != "PNG":
                raise ValueError("Unexpected source image format")
            if image.size != (16, 16) or getattr(image, "n_frames", 1) != 1:
                return None
            # Never alter an interlaced PNG without an explicit review.
            if raw[28] != 0:
                return None
            image.load()
            thumb = image.convert("RGBA").resize((8, 8), Image.Resampling.BOX)
            output = io.BytesIO()
            thumb.save(output, format="PNG", optimize=False, compress_level=9)
            return output.getvalue()
    except (UnidentifiedImageError, OSError) as error:
        raise ValueError("Invalid PNG in pinned source") from error


def compile_preview(source: Path, expected_tree: str = SOURCE_BLOCK_TREE) -> tuple[bytes, dict]:
    found, images = pinned_tree(source)
    if found != expected_tree:
        raise ValueError(f"Vanilla block texture source tree mismatch: {found}")
    converted = []
    for name, raw in sorted(images.items()):
        data = downsample_png(raw)
        if data is not None:
            converted.append((f"textures/blocks/{name}", data))
    if not converted or len(converted) > MAX_TEXTURES:
        raise ValueError("No eligible vanilla 16x16 block textures, or too many")
    sink = io.BytesIO()
    with zipfile.ZipFile(sink, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9,
                         allowZip64=False) as archive:
        entries = [("manifest.json", (json.dumps(MANIFEST, indent=2) + "\n").encode())]
        entries.extend(converted)
        for name, data in entries:
            # Fixed ZIP timestamp/attrs and ordinal ordering for reproducibility.
            item = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            item.compress_type = zipfile.ZIP_DEFLATED
            item.external_attr = 0o100644 << 16
            archive.writestr(item, data, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    receipt = {
        "converted": len(converted), "source_tree": found,
        "original_texels": len(converted) * 256,
        "preview_texels": len(converted) * 64,
        "texel_reduction_percent": 75,
        "pack_bytes": sink.tell(), "runtime_ram_saving": "NOT_MEASURED",
    }
    return sink.getvalue(), receipt


def build(args: list[str] | None = None) -> dict:
    parser = argparse.ArgumentParser(description="Offline LazyTester Idle Texture 8x preview")
    parser.add_argument("--source", required=True, help="Official Bedrock Samples resource_pack folder")
    parser.add_argument("--output", required=True, help="New .mcpack file, never overwritten")
    parsed = parser.parse_args(args)
    if not parsed.output.lower().endswith(".mcpack"):
        raise ValueError("Output must use .mcpack extension")
    supplied = Path(parsed.source).expanduser()
    for part in [supplied, supplied / "textures", supplied / "textures" / "blocks"]:
        _plain(part, True)
    source = supplied.resolve()
    destination = Path(parsed.output).expanduser().resolve()
    if len(destination.name) > 150 or destination == source or source in destination.parents:
        raise ValueError("Output must be outside vanilla source tree")
    if destination.exists() or destination.is_symlink():
        raise FileExistsError("Refusing to replace an existing .mcpack")
    pack, receipt = compile_preview(source / "textures" / "blocks")
    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open("xb") as output:
        output.write(pack)
    receipt["output"] = str(destination)
    return receipt


if __name__ == "__main__":
    try:
        print(json.dumps(build(), indent=2))
    except (ValueError, FileExistsError, OSError) as error:
        print(f"LazyTester Idle Texture: {error}", file=sys.stderr)
        sys.exit(1)
