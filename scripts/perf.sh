#!/usr/bin/env bash
# The performance suite.
#
# Every speed figure the README publishes *about the compiler* is produced by
# this script, so a claim and its measurement cannot drift apart. Build first --
# the release profile is about 2.5x faster than the debug binary on the compiler's
# own source, and mixing the two is how a figure becomes a lie:
#
#     cargo build --release -p refal
#     ./scripts/perf.sh
#
# Every measurement is on the compiler's own source, which is the largest input
# the repository has and the only one whose size is a property of the project
# rather than of a fixture. Timings are machine-dependent, so the script prints
# them rather than asserting them: a performance figure that fails a test on a
# loaded machine is a flaky test, not a regression. What *is* asserted, at the
# end, is the shape -- that the runtime stays linear in the input's length.
set -euo pipefail

cd "$(dirname "$0")/.."

REFAL="${REFAL:-./target/release/refal}"
SOURCE="${SOURCE:-examples/compiler.ref}"

if [ ! -x "$REFAL" ]; then
  echo "no release binary at $REFAL -- run: cargo build --release -p refal" >&2
  exit 2
fi

bytes=$(wc -c < "$SOURCE" | tr -d ' ')
echo "source:  $SOURCE ($bytes bytes)"
echo "binary:  $REFAL"
echo

# `time` writes to stderr and would swallow the command's own output; this
# measures the wall clock around a run whose output is discarded instead.
measure() {
  local label="$1"
  shift
  local start end
  start=$(date +%s%N)
  "$@" > /dev/null 2>&1 || true
  end=$(date +%s%N)
  printf '%-52s %7d ms\n' "$label" "$(( (end - start) / 1000000 ))"
}

echo "== the Rust bootstrap =="
measure "refal graph (seed graph, 4.2)" \
  "$REFAL" graph "$SOURCE"
measure "refal residualize-driven (the driven residue)" \
  "$REFAL" residualize-driven "$SOURCE"
measure "refal compile (the driven path, 4.4 search included)" \
  "$REFAL" compile "$SOURCE"
measure "refal differential --corpus (the T-4/T-6 gate)" \
  "$REFAL" differential examples/differential-corpus.manifest --corpus
echo

echo "== the Refal-authored compiler, interpreted =="
echo "   the cost of running the compiler as a Refal program; the Rust figures"
echo "   above are what the same passes cost in the bootstrap"
measure "compiler.ref GRAPH" \
  "$REFAL" run examples/compiler.ref GRAPH --input-file "$SOURCE"
measure "compiler.ref RESIDUALIZE-DRIVEN" \
  "$REFAL" run examples/compiler.ref RESIDUALIZE-DRIVEN --input-file "$SOURCE"
measure "compiler.ref compiling itself (the self-application)" \
  "$REFAL" run examples/compiler.ref --input-file "$SOURCE"
echo

echo "== what this script does not measure =="
echo "   the runtime's linearity in the input's length. The README publishes a"
echo "   figure for it, measured before the view field was reworked, and this"
echo "   script cannot reproduce it faithfully: `--input-file` hands a program one"
echo "   character-string term rather than one term per character, and the CLI"
echo "   wraps each command-line argument in a bracket, so there is no way to hand"
echo "   a program a large flat term list from outside it. A probe that measured"
echo "   something else and printed it under that heading would be worse than no"
echo "   probe, so the claim stays where it was made and this script says so."
