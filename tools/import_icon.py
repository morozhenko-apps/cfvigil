#!/usr/bin/env python3
"""Import CFVigil's owner-supplied PNG into every desktop icon format.

Usage:
  python3 tools/import_icon.py
  python3 tools/import_icon.py --check

Requires Pillow (on Ubuntu: python3-pil); no network, no image generation.
Uses the exact committed source, never another application's imagery.
"""
import argparse
from io import BytesIO
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "assets" / "branding" / "cfvigil_icon.png"
ICONS = ROOT / "apps" / "desktop" / "src-tauri" / "icons"
WEBVIEW = ROOT / "apps" / "desktop" / "ui" / "icon.png"
SIZES = (16, 24, 32, 48, 64, 128, 256, 512)
ICO_SIZES = tuple((size, size) for size in SIZES if size <= 256)


def encoded(image: Image.Image, image_format: str, **options) -> bytes:
    out = BytesIO()
    image.save(out, format=image_format, **options)
    return out.getvalue()


def assets(base: Image.Image):
    for size in SIZES:
        yield ICONS / f"icon-{size}.png", encoded(
            base.resize((size, size), Image.Resampling.LANCZOS),
            "PNG", optimize=True
        )
    yield ICONS / "icon.png", encoded(
        base.resize((256, 256), Image.Resampling.LANCZOS),
        "PNG", optimize=True
    )
    yield WEBVIEW, encoded(
        base.resize((128, 128), Image.Resampling.LANCZOS),
        "PNG", optimize=True
    )
    yield ICONS / "icon.ico", encoded(
        base.resize((256, 256), Image.Resampling.LANCZOS),
        "ICO", sizes=ICO_SIZES
    )
    yield ICONS / "icon.icns", encoded(
        base.resize((1024, 1024), Image.Resampling.LANCZOS),
        "ICNS"
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=SOURCE)
    parser.add_argument("--check", action="store_true",
                        help="Fail if generated icon assets differ from the source")
    args = parser.parse_args()

    source = args.source.expanduser().resolve()
    if not source.is_file():
        parser.error(f"Source PNG missing: {source}")
    with Image.open(source) as img:
        if img.format != "PNG" or img.mode != "RGBA":
            parser.error("Expected an actual RGBA PNG, not a renamed image")
        if img.width != img.height or img.width < 512:
            parser.error("Expected a square source at least 512x512")
        img.load()
        base = img.copy()
    if base.getpixel((0, 0))[3] != 0:
        print("WARNING: source top-left corner is not fully transparent")

    changed = []
    for path, data in assets(base):
        if args.check:
            if not path.is_file() or path.read_bytes() != data:
                changed.append(str(path.relative_to(ROOT)))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            changed.append(str(path.relative_to(ROOT)))
    if args.check and changed:
        parser.error("Icon assets missing or stale: " + ", ".join(changed))
    if args.check:
        print("PASS: every platform icon matches the original RGBA PNG")
    else:
        print(f"Imported {source.relative_to(ROOT) if source.is_relative_to(ROOT) else source}")
        print(f"Generated {len(changed)} desktop / WebView icon files")
        print("Review icon-16.png, icon-24.png, icon-32.png on light/dark tray panels")
        print("Commit the original and the generated files together")


if __name__ == "__main__":
    main()
