"""Offline safety checks for explicit reuse of an existing signing subkey.

The owner alone runs the interactive GPG signer and authenticated GitHub CLI.
CI deliberately never imports real signing material.
"""
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/provision_existing_apt_key.sh"


class ExistingSigningKeyProvisionTests(unittest.TestCase):
    def test_shell_syntax_and_unrecognized_modes(self):
        syntax = subprocess.run(["bash", "-n", str(SCRIPT)],
                                capture_output=True, text=True)
        self.assertEqual(syntax.returncode, 0, syntax.stderr)
        invalid = subprocess.run(["bash", str(SCRIPT), "--unapproved"],
                                 capture_output=True, text=True)
        self.assertEqual(invalid.returncode, 2)
        self.assertIn("Usage:", invalid.stderr)

    def test_no_interactive_tty_is_always_fail_closed(self):
        with tempfile.TemporaryDirectory(prefix="cfvigil-existing-key-test-") as root:
            filename = Path(root) / "FINGERPRINT"
            filename.write_text("a" * 40 + "\n", encoding="utf8")
            for mode in ("--check", "--apply"):
                with self.subTest(mode=mode):
                    check = subprocess.run(["bash", str(SCRIPT), mode, str(filename)],
                                           input="", capture_output=True, text=True)
                    self.assertEqual(check.returncode, 2)
                    self.assertIn("interactive terminal", check.stderr)
                    self.assertNotIn("[PASS]", check.stdout)

    def test_only_subkey_export_and_scoped_writes_are_permitted(self):
        source = SCRIPT.read_text(encoding="utf8")
        self.assertIn('gpg --batch --export-secret-subkeys "$FPR"', source)
        self.assertIn('gpg --homedir "$TEMP/gnupg" --batch --export-secret-subkeys "$FPR"', source)
        self.assertIn('python3 "$(dirname "$0")/verify_signing_key.py"', source)
        self.assertIn('gpgv --keyring "$TEMP/public.gpg"', source)
        self.assertNotIn("gpg --export-secret-keys", source)
        self.assertIn('if [[ "$MODE" == "--check" ]]; then', source)
        self.assertIn('EXISTING="$(gh secret list -R "$REPO" -e "$ENVIRONMENT"', source)
        writes = [line.strip() for line in source.splitlines()
                  if "gh secret set" in line or "gh variable set" in line]
        self.assertEqual(len(writes), 3)
        self.assertIn("CFVIGIL_APT_SIGNING_SUBKEY_BASE64", writes[0])
        self.assertIn("CFVIGIL_APT_SIGNING_PASSPHRASE", writes[1])
        self.assertIn("CFVIGIL_APT_FINGERPRINT", writes[2])
        self.assertFalse(any("--export-secret-keys " in line
                             for line in source.splitlines()
                             if line.lstrip().startswith(("gpg ", "gpg --homedir"))))
        self.assertNotIn("REPOTICKER_APT_SIGNING", " ".join(writes))


if __name__ == "__main__":
    unittest.main()
