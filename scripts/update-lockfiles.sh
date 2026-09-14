#!/usr/bin/env bash
#
# Regenerate every committed lockfile in the workspace:
#   - Rust:   Cargo.lock, fuzz/Cargo.lock        (cargo update)
#   - Node:   bindings/node/package-lock.json     (npm install --package-lock-only)
#   - Python: .github/requirements/*.txt          (uv pip compile --generate-hashes)
#
# Run from anywhere; the script cd's to the repo root itself:
#
#     ./scripts/update-lockfiles.sh
#
# The Python locks are hash-pinned (OpenSSF Scorecard PinnedDependencies) and
# generated with uv rather than pip-tools because uv can resolve a *target*
# Python version's full transitive closure — with hashes — without that
# interpreter being installed locally. ci-dev is locked twice (Python 3.9 and
# Python 3.10+): the 3.9 lock carries no pytest, see ci-dev-py39.in. If uv is not on PATH the script stops and
# tells you to install it (https://docs.astral.sh/uv/getting-started/installation/);
# WICKRA_BOOTSTRAP_UV=1 opts into fetching one pinned, checksum-verified release
# into a temporary directory instead.
#
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "==> Rust (Cargo.lock, fuzz/Cargo.lock)"
cargo update
(cd fuzz && cargo update)

echo "==> Node (bindings/node/package-lock.json)"
(cd bindings/node && npm install --package-lock-only --no-audit --no-fund)

echo "==> Python (.github/requirements/*.txt via uv)"
# uv is not installed for you unless you ask. The previous version piped
# https://astral.sh/uv/install.sh straight into a shell, which runs whatever is
# behind that URL at that moment, with your privileges, on the machine of
# everyone who regenerates a lockfile. Set WICKRA_BOOTSTRAP_UV=1 to opt in; the
# bootstrap then fetches one pinned release archive and refuses to use it unless
# its checksum matches the one recorded here.
UV_VERSION="0.12.13"
uv_sha256() {
  case "$1" in
    x86_64-unknown-linux-gnu)  echo "745765a3b6e360ad76743599ae5c42e9278c7edf8bbff9fc76d05bf2623a04dd" ;;
    aarch64-unknown-linux-gnu) echo "2eaa5d94f5db7b3a1a092156b9420459e42ab0217d917fe74a876309cef9b5e9" ;;
    aarch64-apple-darwin)      echo "7e6ddb9316acc00f2296c82ff4d99977870ee34b2f0ddcae9444d714db9364ed" ;;
    x86_64-apple-darwin)       echo "5e287ef61cb6a9b61b3a83fef124fd143e400468a7dac794230147a810e17119" ;;
    *)                         echo "" ;;
  esac
}

if ! command -v uv >/dev/null 2>&1; then
  if [ "${WICKRA_BOOTSTRAP_UV:-0}" != "1" ]; then
    echo "    uv is not on PATH." >&2
    echo "    Install it (https://docs.astral.sh/uv/getting-started/installation/)," >&2
    echo "    or re-run with WICKRA_BOOTSTRAP_UV=1 to fetch uv ${UV_VERSION} here." >&2
    exit 1
  fi

  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)   uv_target="x86_64-unknown-linux-gnu" ;;
    Linux-aarch64)  uv_target="aarch64-unknown-linux-gnu" ;;
    Darwin-arm64)   uv_target="aarch64-apple-darwin" ;;
    Darwin-x86_64)  uv_target="x86_64-apple-darwin" ;;
    *)
      echo "    No pinned uv build for $(uname -s)-$(uname -m); install uv yourself." >&2
      exit 1
      ;;
  esac
  uv_expected="$(uv_sha256 "$uv_target")"

  echo "    bootstrapping uv ${UV_VERSION} (${uv_target})..."
  uv_dir="$(mktemp -d)"
  trap 'rm -rf "$uv_dir"' EXIT
  uv_archive="uv-${uv_target}.tar.gz"
  curl -fsSL --retry 5 --retry-all-errors -o "${uv_dir}/${uv_archive}"     "https://github.com/astral-sh/uv/releases/download/${UV_VERSION}/${uv_archive}"
  echo "${uv_expected}  ${uv_dir}/${uv_archive}" | sha256sum -c -
  tar -xzf "${uv_dir}/${uv_archive}" -C "$uv_dir" --strip-components=1
  export PATH="${uv_dir}:$PATH"
fi

req=".github/requirements"
cc="./scripts/update-lockfiles.sh"
uv pip compile --quiet --python-version 3.9  --generate-hashes --custom-compile-command "$cc" "$req/ci-dev-py39.in" -o "$req/ci-dev-py39.txt"
uv pip compile --quiet --python-version 3.11 --generate-hashes --custom-compile-command "$cc" "$req/ci-dev-py3.in"  -o "$req/ci-dev-py3.txt"

echo "==> Done. Review 'git diff' before committing."
