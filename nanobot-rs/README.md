# nanobot-rs

Ultra-lightweight AI Agent core implemented in Rust, with Python FFI bindings.

## Architecture

```
┌─────────────────────────────────────────┐
│         Python Integration Layer        │
│  (channel adapters, provider wrappers) │
└──────────────────┬──────────────────────┘
                   │ PyO3 FFI
┌──────────────────┴──────────────────────┐
│            Rust Core                      │
│  - Agent Loop                            │
│  - Context Builder                       │
│  - Tool Registry                         │
│  - Memory (SQLite)                       │
│  - Session Management                   │
└─────────────────────────────────────────┘
```

## Project Structure

```
nanobot-rs/
├── Cargo.toml              # Rust project config
├── src/
│   ├── lib.rs             # Library entry
│   ├── agent/             # Core agent logic
│   │   ├── mod.rs
│   │   ├── loop_core.rs   # Main agent loop
│   │   └── context.rs     # Prompt builder
│   ├── config/            # Configuration
│   ├── tools/             # Tool system
│   ├── memory/            # SQLite memory
│   ├── session/           # Session management
│   └── ffi/               # PyO3 bindings
└── python/
    ├── pyproject.toml     # Python package
    └── nanobot/           # Python modules
        ├── __init__.py
        ├── client.py      # High-level API
        ├── provider.py    # LLM providers
        └── channel/       # Chat platforms
```

## Build

### Prerequisites
- Rust 1.75+
- Python 3.11+
- maturin (`pip install maturin`)

### Build Rust library
```bash
cd nanobot-rs
cargo build --release
```

### Build Python extension
```bash
cd nanobot-rs/python
maturin develop
```

## Usage

### Python API
```python
from nanobot import NanobotClient
from nanobot.channel import create_channel

# Create client
client = NanobotClient()

# Set LLM provider
from nanobot.provider import LiteLLMProvider
client.set_provider(LiteLLMProvider(model="claude-3-haiku"))

# Chat
response = client.chat("Hello!")
print(response)

# Connect to Telegram
channel = create_channel("telegram", {"token": "YOUR_TOKEN"})
channel.on_message(lambda msg: print(f"User: {msg.content}"))
```

## Design Principles

1. **Rust Core**: Performance-critical agent loop in Rust
2. **Python FFI**: Leverage existing Python SDKs for channels/providers
3. **Minimal**: ~4000 LOC core vs ~100k+ for similar projects
4. **Extensible**: Clean interfaces for channels and tools
