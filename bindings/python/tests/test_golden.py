"""Cross-language golden: every binding must produce byte-identical context JSON.

The fixtures live in the repository-root ``golden/`` directory: bare context
specs, a combined feed universe (``feeds.json``) and the blessed responses.
Building each spec over the shared feeds through a ``build_context`` command must
reproduce ``expected/<spec>.json`` byte-for-byte.
"""

import json
import pathlib

import pytest

from wickra_copilot import Copilot

ROOT = pathlib.Path(__file__).resolve().parents[3]
GOLDEN = ROOT / "golden"


def _spec_files() -> list[pathlib.Path]:
    specs = GOLDEN / "specs"
    if not specs.exists():
        return []
    return sorted(specs.glob("*.json"))


@pytest.mark.skipif(not GOLDEN.exists(), reason="golden fixtures not present")
@pytest.mark.parametrize("spec_path", _spec_files())
def test_golden_context_is_byte_identical(spec_path: pathlib.Path) -> None:
    feeds = json.loads((GOLDEN / "feeds.json").read_text(encoding="utf-8"))
    expected = (GOLDEN / "expected" / f"{spec_path.stem}.json").read_text(
        encoding="utf-8"
    )
    copilot = Copilot(spec_path.read_text(encoding="utf-8"))
    response = copilot.command(json.dumps({"cmd": "build_context", "feeds": feeds}))
    assert response == expected.strip()
