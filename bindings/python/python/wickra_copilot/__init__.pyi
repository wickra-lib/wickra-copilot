"""Type stubs for the wickra_copilot package."""

__version__: str

class Copilot:
    """A copilot instance driven by JSON commands."""

    def __init__(self, spec_json: str) -> None:
        """Build a copilot from a spec JSON string.

        Raises ``ValueError`` if the spec is malformed or invalid.
        """
        ...

    def command(self, cmd_json: str) -> str:
        """Apply a command JSON and return the resulting response JSON.

        Raises ``ValueError`` if the command envelope cannot be handled.
        """
        ...

    @staticmethod
    def version() -> str:
        """The library version."""
        ...
