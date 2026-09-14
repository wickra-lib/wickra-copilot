"""Cross-language golden: every binding must produce byte-identical context JSON.

The fixtures live in the repository-root ``golden/`` directory: bare context
specs, a combined feed universe (``feeds.json``) and the blessed responses.
Building each spec over the shared feeds through a ``build_context`` command must
reproduce ``expected/<spec>.json`` byte-for-byte. A missing corpus is a failure,
not a skip.

Plain functions and plain asserts, so the module runs unchanged under pytest
(3.10 and up) and under ``run_without_pytest.py`` (the 3.9 row).
"""

import json
import pathlib

from wickra_copilot import Copilot

ROOT = pathlib.Path(__file__).resolve().parents[3]
GOLDEN = ROOT / "golden"


def test_golden_context_is_byte_identical() -> None:
    specs = sorted((GOLDEN / "specs").glob("*.json"))
    assert specs, "golden corpus not found"
    feeds = json.loads((GOLDEN / "feeds.json").read_text(encoding="utf-8"))
    for spec_path in specs:
        expected = (GOLDEN / "expected" / spec_path.name).read_text(encoding="utf-8")
        copilot = Copilot(spec_path.read_text(encoding="utf-8"))
        response = copilot.command(json.dumps({"cmd": "build_context", "feeds": feeds}))
        assert response == expected.strip(), spec_path.name
