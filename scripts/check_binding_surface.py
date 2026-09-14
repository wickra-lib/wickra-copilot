#!/usr/bin/env python3
"""Every binding must expose the same operations.

The product claim is that a request built in any language produces the same
canonical bytes. That only holds if every language can express the same
request, so the surface is small on purpose -- construct a handle, send a
command envelope, read the version, release the handle -- and identical
everywhere.

A binding that quietly drops one of those does not fail any test: its own
suite covers what it has. The gap only shows up when somebody tries to do in
one language what the README shows in another.

This reads each binding's source for its own idiom of the four operations,
because that is what the reader will actually call. It does not attempt to
verify behaviour -- the golden fixtures do that, byte for byte, in every
binding.

    python scripts/check_binding_surface.py

Exits non-zero if any present binding is missing an operation.
"""
from __future__ import annotations

import os
import re
import sys

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))

# (binding, files to read, {operation: pattern in that language's idiom})
#
# A binding directory that does not exist is skipped rather than failed: the
# repositories in this family ship different subsets, and an absent binding is
# not an incomplete one.
BINDINGS = [
    ("python", ["bindings/python/src/lib.rs"], {
        "new": r"fn new\b", "command": r"fn command\b", "version": r"fn version\b"}),
    ("node", ["bindings/node/src/lib.rs"], {
        "new": r"pub fn new\b", "command": r"pub fn command\b", "version": r"pub fn version\b"}),
    ("wasm", ["bindings/wasm/src/lib.rs"], {
        "new": r"pub fn new\b", "command": r"pub fn command\b", "version": r"pub fn version\b"}),
    ("c", ["bindings/c/include/*.h"], {
        "new": r"_new\s*\(", "command": r"_command\s*\(",
        "version": r"_version\s*\(", "free": r"_free\s*\("}),
    ("c++", ["bindings/c/include/*.hpp"], {
        "command": r"\bcommand\s*\(", "version": r"\bversion\s*\("}),
    ("go", ["bindings/go/*.go"], {
        "new": r"func New\s*\(", "command": r"\)\s*Command\s*\(",
        "version": r"func Version\s*\(", "free": r"\)\s*Close\s*\("}),
    ("csharp", ["bindings/csharp/*/*.cs"], {
        "command": r"\bCommand\s*\(", "version": r"\bVersion\b", "free": r"\bDispose\s*\("}),
    ("java", ["bindings/java/src/main/java/**/*.java"], {
        "command": r"\bcommand\s*\(", "version": r"\bversion\s*\(", "free": r"\bclose\s*\("}),
    ("r", ["bindings/r/R/*.R"], {
        "new": r"_new\s*<-\s*function", "command": r"_command\s*<-\s*function",
        "version": r"_version\s*<-\s*function"}),
]


def read_all(patterns: list[str]) -> str | None:
    import glob

    text, found = [], False
    for pat in patterns:
        for path in glob.glob(os.path.join(ROOT, pat), recursive=True):
            found = True
            with open(path, encoding="utf-8", errors="ignore") as handle:
                text.append(handle.read())
    return "\n".join(text) if found else None


def main() -> int:
    failures = present = absent = 0
    for name, patterns, ops in BINDINGS:
        text = read_all(patterns)
        if text is None:
            absent += 1
            continue
        present += 1
        missing = [op for op, pat in ops.items() if not re.search(pat, text)]
        if missing:
            failures += 1
            print(f"  {name}: missing {', '.join(sorted(missing))}")
        else:
            print(f"  {name}: {', '.join(sorted(ops))}")

    print()
    if failures:
        print(f"{failures} binding(s) do not expose the shared surface", file=sys.stderr)
        return 1
    print(f"{present} binding(s) expose the same surface, {absent} not present in this repository")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
