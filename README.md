# XiaoClaw

[中文](./README_ZH.md)

---

> ⚡️ An AI-reconstructed version of [nanobot](https://github.com/HKUDS/nanobot), rewritten in Rust for better performance.

Ultra-lightweight AI Agent core implemented in Rust, with Python FFI bindings.

## Architecture

```
┌─────────────────────────────────────────┐
│         Python Integration Layer        │
│  (channel adapters, provider wrappers) │
└──────────────────┬──────────────────────┘
                   │ PyO3 FFI
┌──────────────────┴──────────────────────┐
│              Rust Core                   │
│  - Agent Loop (LLM + Tool calling)     │
│  - Context Builder                      │
│  - Tool Registry                        │
│  - LLM Providers (OpenAI/Anthropic/OpenRouter) │
│  - Memory (SQLite)                      │
│  - Session Management                   │
└─────────────────────────────────────────┘
```

## Project Structure

```
XiaoClaw/
├── Cargo.toml              # Rust project config
├── src/
│   ├── lib.rs             # Library entry
│   ├── agent/             # Core agent logic
│   ├── config/            # Configuration system
│   ├── tools/             # Tool system
│   ├── providers/         # LLM providers
│   ├── memory/            # SQLite memory store
│   ├── session/           # Session management
│   └── ffi/              # PyO3 bindings
├── examples/
│   └── run.rs            # CLI example
└── python/               # Python modules
```

## Features

- **Core Agent Loop**: LLM inference with tool calling (max 20 iterations)
- **Tool System**: Built-in tools + custom tool registration
- **LLM Providers**: OpenAI, Anthropic, OpenRouter, Zhipu/GLM
- **Memory**: SQLite-based persistent memory
- **Session Management**: User session tracking
- **Python FFI**: Bindings for Python integration

## Quick Start

```bash
git clone https://github.com/dianQWQ/XiaoClaw.git
cd XiaoClaw
export OPENAI_API_KEY=sk-...
cargo run --example run
```

## Supported Providers

| Provider | Env Variable | Model Examples |
|----------|-------------|----------------|
| OpenAI | `OPENAI_API_KEY` | `gpt-4o`, `gpt-4o-mini` |
| Anthropic | `ANTHROPIC_API_KEY` | `claude-3-5-sonnet` |
| OpenRouter | `OPENROUTER_API_KEY` | `openai/gpt-4o` |
| Zhipu | `ZHIPU_API_KEY` | `glm-4-flash` |

## Build

```bash
cargo build --release
```

## Usage

```rust
use xiao_claw::{Agent, AgentConfig, ToolRegistry, providers::ZhipuProvider};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = AgentConfig {
        model: "glm-4-flash".into(),
        provider: "zhipu".into(),
        temperature: 0.7,
        max_tokens: Some(2048),
        system_prompt: Some("You are helpful.".into()),
        tools: vec![],
    };
    
    let tools = Arc::new(ToolRegistry::new());
    let agent = Agent::new(config, tools);
    
    let provider = Arc::new(ZhipuProvider::new("your-key".to_string()));
    agent.set_provider(provider);
    
    let response = agent.process("Hello!").await;
    println!("Response: {:?}", response);
}
```

## License

MIT
