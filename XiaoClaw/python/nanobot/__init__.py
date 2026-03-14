"""
Nanobot - Ultra-lightweight AI Agent (Rust Core + Python FFI)
"""

__version__ = "0.1.0"
__all__ = []

try:
    from nanobot_core import (
        PyAgent,
        PyMemoryStore,
        PySessionManager,
        PyToolRegistry,
        load_config,
    )

    __all__.extend(
        [
            "PyAgent",
            "PyMemoryStore",
            "PySessionManager",
            "PyToolRegistry",
            "load_config",
        ]
    )
except ImportError:
    import warnings

    warnings.warn("nanobot-core Rust library not installed. Run: maturin develop")

try:
    from .client import NanobotClient
    from .provider import ProviderRegistry

    __all__.extend(["NanobotClient", "ProviderRegistry"])
except ImportError:
    pass
