"""Wickra Copilot — the deterministic market-context core.

Build a :class:`Copilot` from a spec JSON, drive it with command JSONs, and read
back a ranked list of hard facts. The same command protocol crosses every
language binding, so this Python front-end drives the exact same deterministic
core as the native CLI. The LLM adapter is never part of this surface.
"""

from ._wickra_copilot import Copilot, __version__

__all__ = ["Copilot", "__version__"]
