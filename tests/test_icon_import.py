"""Focused RGBA icon normalization tests, using synthetic source pixels.

Run: python3 -m unittest discover -s tests -p 'test_icon_import.py'
Requires Pillow; never touches owner artwork or generated file paths.
"""
import importlib.util
from pathlib import Path
import unittest

from PIL import Image

MODULE_PATH = Path(__file__).resolve().parent.parent / "tools" / "import_icon.py"
SPEC = importlib.util.spec_from_file_location("cfvigil_icon_import", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
IMPORTER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(IMPORTER)


class NormalizeIconTests(unittest.TestCase):
    def test_alpha_crop_scales_shield_without_changing_source(self):
        source = Image.new("RGBA", (120, 120), (0, 0, 0, 0))
        for x in range(50, 70):
            for y in range(50, 70):
                source.putpixel((x, y), (230, 112, 8, 128))
        before = source.tobytes()
        normalized, bounds = IMPORTER.normalize_icon(source)
        self.assertEqual(bounds, (50, 50, 70, 70))
        self.assertEqual(normalized.size, (120, 120))
        self.assertEqual(normalized.getpixel((0, 0))[3], 0)
        self.assertEqual(normalized.getpixel((60, 60))[3], 128)
        self.assertEqual(source.tobytes(), before, "Owner's source must not mutate")
        visible = normalized.getchannel("A").getbbox()
        self.assertGreaterEqual(visible[2] - visible[0], 107)

    def test_reject_empty_and_nearly_invisible_artifacts(self):
        transparent = Image.new("RGBA", (120, 120), (0, 0, 0, 0))
        with self.assertRaises(ValueError):
            IMPORTER.normalize_icon(transparent)
        transparent.putpixel((0, 0), (255, 255, 255, 1))
        with self.assertRaises(ValueError):
            IMPORTER.normalize_icon(transparent)
        transparent.putpixel((60, 60), (255, 80, 5, 255))
        normalized, bounds = IMPORTER.normalize_icon(transparent)
        self.assertEqual(bounds, (60, 60, 61, 61))
        self.assertEqual(normalized.getpixel((0, 0))[3], 0)

    def test_nonsquare_foreground_retains_aspect_and_transparency(self):
        source = Image.new("RGBA", (120, 120), (0, 0, 0, 0))
        for x in range(45, 75):
            for y in range(55, 65):
                source.putpixel((x, y), (10, 200, 30, 255))
        output, _ = IMPORTER.normalize_icon(source)
        bounds = output.getchannel("A").getbbox()
        self.assertGreaterEqual(bounds[2] - bounds[0], 107)
        self.assertLess(bounds[3] - bounds[1], 50)
        self.assertEqual(output.getpixel((60, 60))[3], 255)


if __name__ == "__main__":
    unittest.main()
