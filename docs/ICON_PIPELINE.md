# CFVigil icon assets

The committed source image is `assets/branding/cfvigil_icon.png`. All required desktop and WebView assets are generated from that image using `tools/import_icon.py`; do not substitute unrelated icons or screenshots.

To regenerate or verify assets in a correctly configured checkout:

```bash
python3 tools/import_icon.py
python3 tools/import_icon.py --check
```

Keep transparent small-size variants legible in the tray and ensure the source and generated PNG, ICO and ICNS outputs remain consistent. Desktop packages must reference the canonical CFVigil icon and application identity on Linux and Windows.

Run the icon validation script during the final source gate. Record actual display defects in issues with redacted evidence rather than accumulating machine-specific investigation logs here.
