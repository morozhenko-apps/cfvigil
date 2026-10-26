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

## 2026-10-25: owner-observed too-small icon / unchanged KDE launcher

On Alcotester, the owner confirms the new shield appears in KDE tray,
Settings and History, but is visibly **too small**; the application-menu
launcher remained on the previous image. This is a real-machine branding
defect, not a reason to redraw the source or rewrite Wayland geometry.

**Architectural decision:** keep the original
`assets/branding/cfvigil_icon.png` unchanged and auditable. Normalize
only its *generated* outputs. Crop the alpha channel to its nonzero
bounding box (ignore invisible margin) and fit that rectangle into a
square transparent 1254px source-sized canvas with a fixed 5% safety
margin. Use alpha-preserving RGBA composition; never paste with the alpha
channel as an extra mask (double transparency). All PNG/ICO/ICNS/WebView
and the native Linux tray are derived from that one normalized bitmap.
No separate tray artwork, and no repeated in-place cropping of source.
Record bounding box and normalization configuration in the SHA256
manifest, then run the deterministic asset check.

For the unchanged KDE menu icon, first **verify package contents** instead
of assuming a cache issue. A .deb must contain exactly one regular
`/usr/share/applications/*.desktop` entry with an `Icon` name that
resolves to at least one installed hicolor PNG under
`/usr/share/icons/hicolor/<size>/apps/<Icon>.png`. The packaged image
must use the current shield, not an old/generic Tauri icon. Add this
verification to the existing unsigned Linux candidate collection.
For the install-time check, inspect the real system launcher, then
rebuild KDE's service/icon cache and unpin/re-pin an old launcher only
if the package identity matches. Do not change Tauri's app identifier or
user settings location as an icon workaround.

**Order:** docs first, importer/source verifier, DEB installed-icon
contract, focused tests on synthetic RGBA pixels and fake installed
desktop/icon tree, then regenerate/commit actual binary outputs from the
owner source on Alcotester. Trigger the one intended Preflight and DEB
candidate only after all derived assets pass the source manifest.
Acceptance requires actual KDE tray/launch-menu/History/Settings visual
checks; Mode B remains blocked.

### Regenerating the normalized outputs on Alcotester

The source and original alpha are retained; the importer crops *generated*
icons only. The previous committed binaries intentionally fail the new
manifest verifier until they are rebuilt once. On the owner's existing
local repository, after pulling `dev` run:

```bash
python3 tools/import_icon.py
python3 tools/import_icon.py --check
python3 -m unittest discover -s tests -p 'test_icon_import.py'
node scripts/check-icon-assets.mjs
git diff -- assets/branding/cfvigil_icon.png  # must be empty
git add apps/desktop/src-tauri/icons apps/desktop/ui/icon.png
git diff --cached --stat
git commit -m "assets: enlarge icon artwork by trimming transparent padding"
git push origin dev
```

Do not commit the source again unless its actual artwork has changed.
Once the new binary assets are committed, run deliberate Preflight and
unsigned Linux candidate jobs on the same SHA.

**KDE menu triage, only after a new verified DEB is installed:** the
Linux bundler's generated desktop file and icon directories are
validated before package upload. On the installed workstation:

```bash
dpkg -L cf-vigil | grep -E '/applications/.*[.]desktop$|/icons/hicolor/'
for item in $(dpkg -L cf-vigil | grep -E '/applications/.*[.]desktop$'); do
  grep -E '^(Name|Exec|Icon|StartupWMClass)=' "$item"
done
kbuildsycoca6
```

If the themed image and desktop icon identity match but the pinned KDE
launcher is still old, unpin and re-pin that entry. Inspect
`~/.local/share/applications` for overrides instead of overwriting or
deleting arbitrary desktop files. Do not rename the application identifier
to force KDE to refresh its launcher; that would change persistence paths.

## 2026-10-25: owner observation on installed normalized icon

Following installation of candidate #40223814222, the owner verified
that the transparent-padding-trimmed shield displays at the intended
size in the KDE tray. **Tray icon PASS**. The application-menu launch
icon remains visually old; whether KDE's pinned launcher override,
icon-theme cache or desktop entry is responsible is **unverified**.
Do not reset application identifiers, delete user launchers, or assume
cache is the root cause. Inspect the actual installed .desktop Icon=
entry, hicolor themed paths and user-level overrides on Alcotester.


## 2026-10-26: owner still sees old KDE launcher and generic About/History icon

The existing icon source, normalized Linux/Windows packaged icons and
native window Tauri `set_icon` are separate from **KDE's desktop-entry
icon resolution**. App uninstall/reinstall is not a proven fix and may
preserve an old user-pinned launcher. Do not reset application IDs or
destroy settings; source now additionally applies the committed icon as
an explicit GTK native-window pixbuf on Linux, in addition to the existing
Tauri icon/favicons. Windows uses the existing Tauri native icon + ICO.

On the owner's installed machine, inspect exact package identity first:

```bash
dpkg-query -W -f='${Package} ${Version}\n' cf-vigil
dpkg -L cf-vigil | grep -E '/applications/.*[.]desktop$|/icons/hicolor/'
for file in $(dpkg -L cf-vigil | grep -E '/applications/.*[.]desktop$'); do
  grep -E '^(Name|Exec|Icon|StartupWMClass)=' "$file"
done
find "$HOME/.local/share/applications" -maxdepth 1 -type f -iname '*vigil*' -print
kbuildsycoca6
```

Compare the *actual* resolved `Icon=` in the installed desktop file,
the hicolor image and any same-name per-user overrides. Unpin/re-pin
a previously pinned KDE launcher **only** if an old entry was confirmed.
A new DEB may be installed with `sudo apt install ./package.deb`
without `apt purge`. On real KDE and Windows, verify the new icon in
About/History/task switcher, not just the WebView's HTML favicon.
Native acceptance remains owner-controlled.
