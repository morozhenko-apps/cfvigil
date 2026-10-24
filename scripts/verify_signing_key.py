#!/usr/bin/env python3
"""Validate offline-primary APT signing authority before accessing R2."""
import argparse
from pathlib import Path
import re


def validate(text, fingerprint):
    fingerprint = fingerprint.upper()
    if not re.fullmatch(r"[A-F0-9]{40}", fingerprint):
        raise ValueError("Invalid primary fingerprint")
    lines = [row.split(":") for row in text.splitlines() if row.strip()]
    types = [row[0] for row in lines]
    if not lines or any(tag not in {"sec", "ssb", "fpr", "uid", "grp"} for tag in types):
        raise ValueError("Unrecognized GPG inventory")
    keys = [row for row in lines if row[0] in {"sec", "ssb", "fpr"}]
    if [row[0] for row in keys] != ["sec", "fpr", "ssb", "fpr"]:
        raise ValueError("Only one primary and one signing subkey are permitted")
    primary, primary_fpr, subkey, subkey_fpr = keys
    if len(primary) < 15 or primary[14] != "#":
        raise ValueError("Full primary secret present: refusing publication")
    if len(subkey) < 15 or subkey[14] != "+" or "s" not in subkey[11].lower():
        raise ValueError("No usable signing subkey")
    if primary_fpr[9].upper() != fingerprint or not re.fullmatch(
        r"[A-F0-9]{40}", subkey_fpr[9].upper()
    ):
        raise ValueError("Signing authority does not match fingerprint")
    if subkey_fpr[9].upper() == fingerprint:
        raise ValueError("Signing subkey must differ from certification primary")


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--listing", type=Path, required=True)
    p.add_argument("--fingerprint", required=True)
    a = p.parse_args()
    try:
        validate(a.listing.read_text(), a.fingerprint)
    except (ValueError, IndexError, OSError) as error:
        p.exit(1, f"Signing subkey verification failed: {error}\n")


if __name__ == "__main__":
    main()
