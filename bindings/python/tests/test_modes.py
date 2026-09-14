"""Operating-mode equivalence through the binding, over the golden corpus.

The command protocol offers two ways to reach the same answer, and both must
return the same bytes: ``facts`` is an alias of ``build_context``, and ``query``
answers either against the context the handle stored from its last
``build_context`` or against a context passed inline. The core pins this in
Rust (``operating_modes.rs``); this checks the boundary the Python binding
crosses. A missing corpus is a failure, not a skip.

Plain functions and plain asserts, so the module runs unchanged under pytest
(3.10 and up) and under ``run_without_pytest.py`` (the 3.9 row).
"""

import json
import pathlib

from wickra_copilot import Copilot

ROOT = pathlib.Path(__file__).resolve().parents[3]
GOLDEN = ROOT / "golden"
QUESTION = "what moved and why?"


def test_facts_alias_and_inline_query_match_the_stored_path() -> None:
    specs = sorted((GOLDEN / "specs").glob("*.json"))
    assert specs, "golden corpus not found"
    feeds = json.loads((GOLDEN / "feeds.json").read_text(encoding="utf-8"))
    for spec_path in specs:
        spec = spec_path.read_text(encoding="utf-8")
        expected = (GOLDEN / "expected" / spec_path.name).read_text(encoding="utf-8").strip()

        stored = Copilot(spec)
        built = stored.command(json.dumps({"cmd": "build_context", "feeds": feeds}))
        assert built == expected, spec_path.name
        from_stored = stored.command(json.dumps({"cmd": "query", "question": QUESTION}))

        fresh = Copilot(spec)
        from_inline = fresh.command(
            json.dumps({"cmd": "query", "question": QUESTION, "context": json.loads(built)})
        )
        assert from_inline == from_stored, spec_path.name

        alias = Copilot(spec)
        assert alias.command(json.dumps({"cmd": "facts", "feeds": feeds})) == built, spec_path.name
