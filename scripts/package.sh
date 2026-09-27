#!/usr/bin/env bash
# Build the release artifact.
#
# The archive holds the binary, the documentation, and the Refal sources the
# binary compiles -- nothing else, and nothing that was not produced by the tree
# it was cut from. The version comes from `Cargo.toml`, which is the same number
# the binary reports and the same number `CHANGELOG.md` heads its newest section
# with; `the_workspace_version_and_the_changelog_agree` is the test that keeps
# those three from drifting.
#
#     ./scripts/package.sh            # writes refal-<version>-<os>-<arch>.tar.gz
#     ./scripts/package.sh /tmp/out   # writes it there instead
set -euo pipefail

cd "$(dirname "$0")/.."

version=$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -1)
if [ -z "$version" ]; then
  echo "no workspace version in Cargo.toml" >&2
  exit 2
fi

os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
name="refal-${version}-${os}-${arch}"
out="${1:-.}"
mkdir -p "$out"

echo "building refal ${version} (release)"
cargo build --release -p refal

binary=target/release/refal
if [ -f "$binary.exe" ]; then
  binary="$binary.exe"
fi
if [ ! -f "$binary" ]; then
  echo "no release binary at $binary" >&2
  exit 2
fi

stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/$name"

cp "$binary" "$stage/$name/refal"
cp README.md CHANGELOG.md CONTRIBUTING.md LICENSE-MIT "$stage/$name/"
cp -r docs "$stage/$name/docs"
cp -r examples "$stage/$name/examples"
cp -r scripts "$stage/$name/scripts"
# The Refal sources are the point: this is a compiler that compiles them.
rm -rf "$stage/$name/docs/turchin/pdf"

cat > "$stage/$name/INSTALL.md" <<INSTALL
# Installing refal ${version}

The archive contains a single binary and no runtime dependencies.

    tar -xzf ${name}.tar.gz
    cd ${name}
    ./refal --version        # refal ${version}
    ./refal check examples/hello.ref
    ./refal compile examples/metasystem-unroll.ref

To put it on your PATH:

    install -m 0755 refal /usr/local/bin/refal

## What is in the archive

| | |
|---|---|
| \`refal\` | the compiler: the Rust bootstrap and the verification harness |
| \`examples/\` | the corpus, including \`compiler.ref\` -- the compiler written in Refal |
| \`docs/\` | the plan, the progress handoff, the Turchin objective matrix, the reference notes |
| \`scripts/\` | the performance suite, the profiler, and this packaging script |

The Rust binary is not the production compiler: \`compiler.ref\` is. The binary is
what runs it, and what the differential gates compare it against.
INSTALL

echo "archiving $name"
tar -czf "$out/${name}.tar.gz" -C "$stage" "$name"
echo "wrote $out/${name}.tar.gz"
