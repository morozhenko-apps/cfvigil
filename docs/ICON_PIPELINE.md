# CFVigil owner-provided icon pipeline

Source of truth: `assets/branding/cfvigil_icon.png` (1254 × 1254 RGBA
in the owner's local workspace at the time this protocol was drafted).
**The local file is not accessible to a remote GitHub coding agent until
the owner commits and pushes it.** Never substitute the unrelated
RepoTicker PNG or a screenshot for the actual design.

## Reproducible import

Use the same Pillow/LANCZOS pipeline as RepoTicker's own
`tools/import_icon.py` and commit BOTH the original and generated output:

```bash
cd ~/Documents/GitHub/cfvigil
git switch dev
git pull --ff-only origin dev
python3 tools/import_icon.py
python3 tools/import_icon.py --check
git add assets/branding/cfvigil_icon.png \
  apps/desktop/src-tauri/icons apps/desktop/ui/icon.png
git commit -m "assets: install original CFVigil icon across desktop targets"
git push origin dev
```

**Current migration note (2026-10-25).** The first source commit
`34acd1e` successfully imported the original PNG and ten PNG derivatives,
but the existing Git ignore rules unintentionally excluded `icon.ico`
and `icon.icns`. The ignore rules have now been corrected in `dev`.
If the importer already ran locally, you **do not need to regenerate**:
run `git pull --ff-only origin dev`, then `git add
apps/desktop/src-tauri/icons/icon.ico
apps/desktop/src-tauri/icons/icon.icns`, verify
`git diff --cached --name-only` contains exactly those two files, commit
them and push `dev`. Preflight/candidate packaging must not be triggered
until the complete manifest outputs are present in GitHub.

If Python reports that Pillow is unavailable, install it using the Linux
distribution's `python3-pil` package; do not use the temporary screenshot
or fetch an untrusted icon from the web. Never overwrite unrelated
uncommitted work; inspect `git status --short` first.

The importer preserves the original source without modification, creates
transparent anti-aliased PNGs at 16, 24, 32, 48, 64, 128, 256, 512 px, a
256px `icon.png` for Tauri's bundled Linux icon/ksni tray and a 128px
`apps/desktop/ui/icon.png` for all WebViews. It also creates multi-size
Windows `.ico` and macOS `.icns` from the **same exact source**.
The tray runtime uses RGBA->ARGB conversion and 32px Lanczos as before;
there is no copied proprietary system/brand icon.

On program startup the native window icon is explicitly assigned to
main, Settings, History and About. This is in addition to the packaged
desktop entry/icon and tray applet; all share the committed source.
If a Wayland compositor ignores per-window overrides, ensure the
installed `.desktop` identifier matches its icon, and inspect the
actual KDE task switcher / title bar in the native-machine smoke.

### Verification before acceptance

- Verify a square RGBA source and transparent margin/edges; keep its
  original colors and content (no artistic transformation).
- Visually inspect `icon-16.png`, `icon-24.png`, `icon-32.png`
  against light/dark tray panels; no background square, no clipping.
- Check the DEB desktop entry, app menu, KDE tray, History/Settings/About
  window icons, and NSIS installer/start menu on actual Windows.
- Run Preflight and candidate packaging only after all artifacts have
  been committed. A green CI is not evidence that icons look good on
  the physical screen.
- This is an **owner-supplied image**, not previously used repo's tiny
  placeholder icon. Copyright/source-of-artwork confirmation must be
  recorded by the owner before public release; asset packaging alone
  does not constitute a rights review.

No `main` promotion, release tags, R2 publication or Mode B approval
is implied by the import.
