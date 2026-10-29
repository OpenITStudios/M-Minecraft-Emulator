"""Synthetic-only tests. The real Mojang sample assets are never bundled."""
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
import zipfile

from PIL import Image

from build import SOURCE_BLOCK_TREE, build, compile_preview, downsample_png, pinned_tree


def png(size=(16, 16), *, mode="RGBA"):
    image = Image.new(mode, size)
    if mode == "RGBA":
        for y in range(size[1]):
            for x in range(size[0]):
                image.putpixel((x, y), (200 if x % 2 else 0, 60 if y % 2 else 0, 20, 255))
    else:
        image.putpalette([255, 0, 0, 0, 0, 255] + [0] * 762)
        for y in range(size[1]):
            for x in range(size[0]):
                image.putpixel((x, y), (x + y) % 2)
    buffer = io.BytesIO()
    image.save(buffer, format="PNG")
    return buffer.getvalue()


class TexturePreviewTests(unittest.TestCase):
    def fixture(self, root: Path) -> Path:
        blocks = root / "textures" / "blocks"
        blocks.mkdir(parents=True)
        (blocks / "stone.png").write_bytes(png())
        (blocks / "brick.png").write_bytes(png(mode="P"))
        (blocks / "water_still.png").write_bytes(png((16, 512)))
        (blocks / "metadata.json").write_text('{"not copied":true}', encoding="utf-8")
        (blocks / "candles").mkdir()
        (blocks / "candles" / "candle.png").write_bytes(png())
        return blocks

    def git_tree(self, root: Path) -> str:
        subprocess.run(["git", "init", "--quiet", str(root)], check=True)
        subprocess.run(["git", "-C", str(root), "add", "textures/blocks"], check=True)
        tree = subprocess.check_output(["git", "-C", str(root), "write-tree"], text=True).strip()
        result = subprocess.check_output(["git", "-C", str(root), "ls-tree", tree,
                                          "textures/blocks"], text=True).strip()
        return result.split()[2]

    def test_exact_git_tree_sha_and_nested_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            blocks = self.fixture(root)
            sha, files = pinned_tree(blocks)
            self.assertEqual(sha, self.git_tree(root))
            self.assertNotEqual(sha, SOURCE_BLOCK_TREE)
            self.assertEqual(len(files), 4)
            self.assertIn("candles/candle.png", files)

    def test_box_downsampling_preserves_alpha_and_outputs_real_8x8_png(self):
        raw = png()
        transformed = downsample_png(raw)
        with Image.open(io.BytesIO(transformed)) as result:
            self.assertEqual(result.size, (8, 8))
            self.assertEqual(result.mode, "RGBA")
            self.assertEqual(result.getpixel((0, 0)), (100, 30, 20, 255))
        with Image.open(io.BytesIO(downsample_png(png(mode="P")))) as indexed:
            self.assertEqual(indexed.size, (8, 8))
            self.assertEqual(indexed.mode, "RGBA")
        self.assertIsNone(downsample_png(png((16, 512))))
        self.assertRaises(ValueError, downsample_png, b"not a PNG")

    def test_archive_saves_preview_and_skips_animated_tiles(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            blocks = self.fixture(root)
            expected, _ = pinned_tree(blocks)
            pack, receipt = compile_preview(blocks, expected)
            self.assertEqual(receipt["converted"], 3)
            self.assertEqual(receipt["original_texels"], 3 * 256)
            self.assertEqual(receipt["preview_texels"], 3 * 64)
            self.assertEqual(receipt["texel_reduction_percent"], 75)
            self.assertEqual(receipt["runtime_ram_saving"], "NOT_MEASURED")
            self.assertEqual(pack, compile_preview(blocks, expected)[0])
            with zipfile.ZipFile(io.BytesIO(pack)) as z:
                self.assertEqual(z.namelist(), [
                    "manifest.json", "textures/blocks/brick.png",
                    "textures/blocks/candles/candle.png", "textures/blocks/stone.png"])
                self.assertEqual(json.loads(z.read("manifest.json"))["modules"][0]["type"], "resources")
                with Image.open(io.BytesIO(z.read("textures/blocks/stone.png"))) as tile:
                    self.assertEqual(tile.size, (8, 8))
                self.assertEqual(z.testzip(), None)

    def test_source_change_and_cli_never_write_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            blocks = self.fixture(root)
            expected, _ = pinned_tree(blocks)
            (blocks / "stone.png").write_bytes(png((16, 16), mode="P"))
            with self.assertRaisesRegex(ValueError, "tree mismatch"):
                compile_preview(blocks, expected)
            output = root.parent / (root.name + ".mcpack")
            with self.assertRaisesRegex(ValueError, "source tree mismatch"):
                build(["--source", str(root), "--output", str(output)])
            self.assertFalse(output.exists())

    def test_linked_source_fails_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            blocks = self.fixture(root)
            try:
                (blocks / "linked.png").symlink_to(blocks / "stone.png")
            except (OSError, NotImplementedError):
                self.skipTest("Symlink unavailable")
            with self.assertRaisesRegex(ValueError, "Linked source refused"):
                pinned_tree(blocks)


if __name__ == "__main__":
    unittest.main()
