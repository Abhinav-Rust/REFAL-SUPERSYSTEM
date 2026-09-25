#!/usr/bin/env python3
"""Count every call a `compiler.ref` mode makes.

The Refal-authored driver has no profiler, and a wall-clock figure cannot say
which of its 480 functions is the cost. This wraps every top-level definition in
a one-line function that prints a marker and forwards its arguments, runs a
mode, and counts the markers. The output is a call histogram -- exact counts,
not a sample.

Two constraints the generator has to respect, both of which cost a bisection to
rediscover otherwise:

* Refal-5 identifiers are capped at 15 characters, so the wrappers are named
  ``W001``.. rather than ``<name>P``.
* A wrapper forwards `e.Args`, so a function called with no arguments, or with
  several, is unaffected: the argument list is passed through whole.

Usage::

    python scripts/profile.py RESIDUALIZE-DRIVEN --input-file examples/compiler.ref
    python scripts/profile.py GRAPH examples/lexer.ref
    python scripts/profile.py --compare RESIDUALIZE-DRIVEN prog-25.ref prog-50.ref

The ``--compare`` form prints a growth ratio per function, which is what
separates a linear pass from a quadratic one.
"""
from __future__ import annotations

import argparse
import collections
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIN = os.path.join(ROOT, "target", "release", "refal.exe")
COMPILER = os.path.join(ROOT, "examples", "compiler.ref")

DEFINITION = re.compile(r"(?m)^([A-Za-z][A-Za-z0-9]*) \{$")
MARKER = re.compile(r"^W\d{3}$")


def instrument(source: str) -> tuple[str, dict[str, str]]:
    """Wrap every top-level function, returning the program and its marker map."""
    names: list[str] = []
    for match in DEFINITION.finditer(source):
        name = match.group(1)
        if name not in names:
            names.append(name)

    mapping: dict[str, str] = {}
    for index, name in enumerate(names, 1):
        wrapper = "W%03d" % index
        mapping[wrapper] = name
        source = re.sub(r"<" + re.escape(name) + r"\b", "<" + wrapper, source)
        source += "\n%s {\n  e.Args = <Prout '%s'> <%s e.Args>;\n}\n" % (
            wrapper,
            wrapper,
            name,
        )
    return source, mapping


def run_mode(program: str, mode: str, argument: str) -> collections.Counter[str]:
    with tempfile.TemporaryDirectory() as scratch:
        path = os.path.join(scratch, "profiled.ref")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(program)
        completed = subprocess.run(
            [BIN, "run", path, mode, "--input-file", argument],
            capture_output=True,
            text=True,
            errors="replace",
            cwd=ROOT,
        )
    if completed.returncode != 0:
        sys.stderr.write(
            "%s failed (%d):\n%s\n" % (argument, completed.returncode, completed.stderr[:800])
        )
    counts: collections.Counter[str] = collections.Counter()
    for line in completed.stdout.splitlines():
        if MARKER.match(line):
            counts[line] += 1
    return counts


def report(counts: collections.Counter[str], mapping: dict[str, str], limit: int) -> None:
    total = sum(counts.values())
    print("total calls: %d" % total)
    print("%10s  %6s  %s" % ("count", "share", "function"))
    for marker, count in counts.most_common(limit):
        print("%10d  %5.1f%%  %s" % (count, 100.0 * count / total, mapping.get(marker, marker)))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", help="a mode compiler.ref's Dispatch understands")
    parser.add_argument("inputs", nargs="+", help="programs to run the mode on")
    parser.add_argument("--limit", type=int, default=25, help="rows to print")
    parser.add_argument(
        "--compare",
        action="store_true",
        help="print each function's growth ratio across the inputs instead of one histogram",
    )
    args = parser.parse_args()

    with open(COMPILER, "r", encoding="utf-8", errors="replace") as handle:
        program, mapping = instrument(handle.read())

    runs = [(path, run_mode(program, args.mode, path)) for path in args.inputs]

    if not args.compare:
        for path, counts in runs:
            print("== %s ==" % path)
            report(counts, mapping, args.limit)
            print()
        return 0

    keys = sorted(
        {marker for _, counts in runs for marker in counts},
        key=lambda marker: -max(counts.get(marker, 0) for _, counts in runs),
    )
    header = "  ".join("%9s" % os.path.basename(path) for path, _ in runs)
    print("%-18s %s   growth" % ("function", header))
    for marker in keys[: args.limit]:
        series = [counts.get(marker, 0) for _, counts in runs]
        ratios = [
            series[index + 1] / series[index] if series[index] else float("nan")
            for index in range(len(series) - 1)
        ]
        print(
            "%-18s %s   %s"
            % (
                mapping.get(marker, marker),
                "  ".join("%9d" % value for value in series),
                " ".join("%.2f" % ratio for ratio in ratios),
            )
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
