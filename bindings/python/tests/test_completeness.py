"""Pin the public surface of the wickra_copilot module and Copilot class.

The class-surface guard mirrors the Node/R completeness checks; the module guard
pins the package's exported names (via ``__all__``) so a stray export — or a
dropped one — fails loudly, matching the exact-surface guard in the Node binding.
"""

import wickra_copilot
from wickra_copilot import Copilot

EXPECTED_METHODS = {"command", "version"}
EXPECTED_EXPORTS = ["Copilot", "__version__"]


def test_expected_methods_present() -> None:
    for name in EXPECTED_METHODS:
        assert hasattr(Copilot, name), f"missing method: {name}"


def test_no_unexpected_public_methods() -> None:
    public = {name for name in dir(Copilot) if not name.startswith("_")}
    assert public == EXPECTED_METHODS


def test_module_all_is_exact() -> None:
    assert wickra_copilot.__all__ == EXPECTED_EXPORTS


def test_module_exposes_copilot_and_version() -> None:
    assert isinstance(wickra_copilot.Copilot, type)
    assert isinstance(wickra_copilot.__version__, str)
    assert wickra_copilot.__version__ == Copilot.version()
