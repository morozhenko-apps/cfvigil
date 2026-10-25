#!/usr/bin/env python3
"""Import CFVigil's owner-supplied PNG into every desktop icon format.

Usage:
  python3 tools/import_icon.py
  python3 tools/import_icon.py --check

Requires Pillow (on Ubuntu: python3-pil); no network, no image generation.
Uses the exact committed source, never another application's imagery.
"""
import argparse
import hashlib
import json
from io import BytesIO
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "assets" / "branding" / "cfvigil_icon.png"
ICONS = ROOT / "apps" / "desktop" / "src-tauri" / "icons"
WEBVIEW = ROOT / "apps" / "desktop" / "ui" / "icon.png"
SIZES = (16, 24, 32, 48, 64, 128, 256, 512)
ICO_SIZES = tuple((size, size) for size in SIZES if size <= 256)
# Deliberately small visual breathing room at every display size; not 5%
# additional alpha on top of a source that already has transparent margins.
PADDING_PERCENT = 5
ALPHA_THRESHOLD = 8


def normalize_icon(source: Image.Image) -> tuple[Image.Image, tuple[int, int, int, int]]:
    """Fit visible RGBA pixels inside a square source-sized transparent canvas.

    Alpha-bounds are computed before resampling; low-alpha dust is ignored.
    Crop preserves the image's existing alpha. IMPORTANT: paste WITHOUT a
    mask, otherwise RGBA alpha is multiplied by itself and becomes faded.
    The owner-provided source PNG remains untouched on disk.
    """
    if source.width != source.height or source.mode != "RGBA":
        raise ValueError("Expected square RGBA icon source")
    opaque_enough = source.getchannel("A").point(
        lambda a: 255 if a >= ALPHA_THRESHOLD else 0
    )
    bounds = opaque_enough.getbbox()
    if bounds is None:
        raise ValueError("Icon has no visible pixels above alpha threshold")
    cropped = source.crop(bounds)
    side = source.width
    inset = round(side * PADDING_PERCENT / 100)
    limit = side - 2 * inset
    factor = limit / max(cropped.width, cropped.height)
    scaled_size = tuple(max(1, min(limit, round(dim * factor)))
                        for dim in cropped.size)
    scaled = cropped.resize(scaled_size, Image.Resampling.LANCZOS)
    result = Image.new("RGBA", source.size, (0, 0, 0, 0))
    offset = ((side - scaled.width) // 2, (side - scaled.height) // 2)
    result.paste(scaled, offset)
    return result, bounds


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
    parser.add_argument("--check", action="store_true",
                        help="Fail if generated icon assets differ from the source")
    args = parser.parse_args()

    source = SOURCE.resolve()
    if not source.is_file():
        parser.error(f"Source PNG missing: {source}")
    with Image.open(source) as img:
        if img.format != "PNG" or img.mode != "RGBA":
            parser.error("Expected an actual RGBA PNG, not a renamed image")
        if img.width != img.height or img.width < 512:
            parser.error("Expected a square source at least 512x512")
        img.load()
        original = img.copy()
    try:
        base, bounds = normalize_icon(original)
    except ValueError as error:
        parser.error(str(error))

    rendered = list(assets(base))
    manifest = {
        "source": str(SOURCE.relative_to(ROOT)),
        "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "source_size": [original.width, original.height],
        "normalization": {
            "method": "alpha-bbox-fit",
            "alpha_threshold": ALPHA_THRESHOLD,
            "padding_percent": PADDING_PERCENT,
            "bbox_xyxy": list(bounds),
            "output_size": [base.width, base.height],
        },
        "files": {
            str(path.relative_to(ROOT)): hashlib.sha256(data).hexdigest()
            for path, data in rendered
        },
    }
    rendered.append((
        ICONS / "source-manifest.json",
        (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode("utf-8")
    ))
    changed = []
    for path, data in rendered:
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
        print("PASS: every normalized platform icon matches the owner RGBA PNG")
    else:
        print(f"Imported {source.relative_to(ROOT) if source.is_relative_to(ROOT) else source}")
        print(f"Normalized visible alpha bbox {bounds} with {PADDING_PERCENT}% outer padding")
        print(f"Generated {len(changed)-1} desktop / WebView icon files and source-manifest.json")
        print("Review icon-16.png, icon-24.png, icon-32.png on light/dark tray panels")
        print("Commit the original and the generated files together")


if __name__ == "__main__":
    main()
