#!/usr/bin/env python3
"""The R wrapper must not call a C ABI symbol that does not exist.

bindings/r/src/<lib>.c is hand-written C that calls the C ABI directly. The
header it compiles against is staged by configure from a *released* archive, not
from this working tree -- so the wrapper can drift away from the ABI without
anything here noticing: the workspace builds, every Rust test passes, and the
break appears when a user installs the package and the linker cannot resolve a
symbol.

This compares the two sides as text, which is what makes it useful before a
release rather than after: every symbol the wrapper calls must be declared in
the header, and the arity must match.

    python scripts/check_r_abi_skew.py

Exits non-zero on the first symbol that does not line up.
"""
from __future__ import annotations

import glob
import os
import re
import sys

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))


def read(path: str) -> str:
    with open(path, encoding="utf-8", errors="ignore") as handle:
        return handle.read()


def strip_comments(src: str) -> str:
    src = re.sub(r"/\*.*?\*/", " ", src, flags=re.S)
    return re.sub(r"//[^\n]*", " ", src)


def header_decls(header: str) -> dict[str, int]:
    """{symbol: parameter count} for every function the header declares."""
    out: dict[str, int] = {}
    for m in re.finditer(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(([^;{)]*)\)\s*;", strip_comments(header)):
        name, params = m.group(1), m.group(2).strip()
        if name in ("if", "for", "while", "switch", "return", "sizeof"):
            continue
        if params in ("", "void"):
            count = 0
        else:
            count = len([p for p in params.split(",") if p.strip()])
        out[name] = count
    return out


def wrapper_calls(src: str, prefix: str) -> dict[str, int]:
    """{symbol: argument count} for every ABI call the wrapper makes."""
    out: dict[str, int] = {}
    body = strip_comments(src)
    for m in re.finditer(rf"\b({re.escape(prefix)}[A-Za-z0-9_]*)\s*\(", body):
        name = m.group(1)
        # walk the argument list, respecting nesting and strings
        i, depth, args, in_str = m.end(), 1, 1, None
        while i < len(body) and depth:
            ch = body[i]
            if in_str:
                if ch == "\\":
                    i += 2
                    continue
                if ch == in_str:
                    in_str = None
            elif ch in "\"'":
                in_str = ch
            elif ch in "([":
                depth += 1
            elif ch in ")]":
                depth -= 1
            elif ch == "," and depth == 1:
                args += 1
            i += 1
        # a call with an empty argument list
        if body[m.end():i - 1].strip() == "":
            args = 0
        out.setdefault(name, args)
    return out


def main() -> int:
    headers = sorted(glob.glob(os.path.join(ROOT, "bindings", "c", "include", "*.h")))
    wrappers = sorted(glob.glob(os.path.join(ROOT, "bindings", "r", "src", "*.c")))
    if not headers or not wrappers:
        print("no C ABI header or R wrapper in this repository; nothing to compare")
        return 0

    declared: dict[str, int] = {}
    for h in headers:
        declared.update(header_decls(read(h)))

    # The ABI prefix is the header's own name: wickra_copilot.h -> wickra_copilot_
    prefix = os.path.splitext(os.path.basename(headers[0]))[0] + "_"

    failures = 0
    for w in wrappers:
        calls = wrapper_calls(read(w), prefix)
        rel = os.path.relpath(w, ROOT).replace("\\", "/")
        if not calls:
            print(f"  {rel}: calls no {prefix}* symbol")
            continue
        for name, argc in sorted(calls.items()):
            if name not in declared:
                print(f"  {rel}: calls {name}, which the C ABI header does not declare")
                failures += 1
            elif declared[name] != argc:
                print(f"  {rel}: {name} called with {argc} argument(s), "
                      f"declared with {declared[name]}")
                failures += 1
            else:
                print(f"  {rel}: {name}({argc}) matches the header")

    print()
    if failures:
        print(f"{failures} symbol(s) do not match the C ABI", file=sys.stderr)
        return 1
    print(f"every {prefix}* call in the R wrapper matches the committed C ABI header")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
