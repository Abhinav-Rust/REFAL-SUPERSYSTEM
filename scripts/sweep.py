#!/usr/bin/env python3
"""Sweep every example against `refal lower` and the Refal-authored compiler.

The Rust `lower` command is the oracle. For each example this runs both and
reports the first divergence, so grammar gaps in compiler.ref show up as
concrete diffs rather than as a percentage.
"""
from __future__ import annotations

import glob
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIN = os.path.join(ROOT, "target", "release", "refal.exe")
COMPILER = os.path.join(ROOT, "examples", "compiler.ref")


def run(args: list[str], stdin: str | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(
        args, input=stdin, capture_output=True, text=True, errors="replace", cwd=ROOT
    )


def main() -> int:
    names = sorted(glob.glob(os.path.join(ROOT, "examples", "*.ref")))
    same, diff, skipped, failed = [], [], [], []

    for path in names:
        name = os.path.basename(path)
        if name == "compiler.ref":
            continue
        oracle = run([BIN, "lower", path])
        if oracle.returncode != 0:
            skipped.append(name)
            continue
        with open(path, "r", encoding="utf-8", errors="replace") as handle:
            source = handle.read()
        actual = run([BIN, "run", COMPILER, source])
        if actual.returncode != 0:
            failed.append((name, actual.stderr.strip().splitlines()[:2]))
            continue
        if actual.stdout != oracle.stdout:
            diff.append((name, oracle.stdout, actual.stdout))
        else:
            same.append(name)

    print(f"identical : {len(same)}")
    print(f"differ    : {len(diff)}")
    print(f"failed    : {len(failed)}")
    print(f"skipped   : {len(skipped)} (lower rejects: negative fixtures)")
    print()
    for name, exp, got in diff:
        print(f"--- DIFF {name}")
        print(f"  lower  : {exp!r}")
        print(f"  refal  : {got!r}")
    for name, err in failed:
        print(f"--- FAIL {name}: {err}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
