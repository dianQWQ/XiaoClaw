# nanobot-rs

[English](#english) | [中文](#中文)

---

## English

Ultra-lightweight AI Agent core implemented in Rust, with Python FFI bindings.

Inspired by [nanobot](https://github.com/HKUDS/nanobot) - A lightweight AI assistant written in ~4000 lines of Python.

### Architecture

```
┌─────────────────────────────────────────┐
│         Python Integration Layer        │
│  (channel adapters, provider wrappers) │
└──────────────────┬──────────────────────┘
                   │ PyO3 FFI
┌──────────────────┴──────────────────────┐
│              Rust Core                   │
│  - Agent Loop (LLM + Tool calling)     │
│  - Context Builder                     │
│  - Tool Registry                       │
│  - Memory (SQLite)                     │
│  - Session Management                  │
└─────────────────────────────────────────┘
```

### Project Structure

```
nanobot-rs/
├── Cargo.toml              # Rust project config
├── src/
│   ├── lib.rs             # Library entry
│   ├── agent/             # Core agent logic
│   │   ├── mod.rs        # Agent, Message, Response
│   │   ├── loop_core.rs  # Main agent loop
│   │   └── context.rs    # Prompt builder
│   ├── config/           # Configuration system
│   ├── tools/            # Tool system
│   │   ├── mod.rs        # Tool trait
│   │   ├── registry.rs   # Tool registry
│   │   └── builtin.rs    # Built-in tools
│   ├── memory/           # SQLite memory store
│   ├── session/          # Session management
│   └── ffi/              # PyO3 bindings
└── python/
    ├── pyproject.toml    # Python package
    └── nanobot/          # Python modules
        ├── __init__.py
        ├── client.py     # High-level API
        ├── provider.py   # LLM providers
        └── channel/      # Chat platforms
```

### Features

- **Core Agent Loop**: LLM inference with tool calling (max 20 iterations)
- **Tool System**: Built-in tools (filesystem, web_search, message, cron) + custom tool registration
- **Memory**: SQLite-based persistent memory with session support
- **Session Management**: User session tracking with timeout
- **Python FFI**: Bindings for Python integration (via PyO3)
- **Channel Adapters**: Telegram, Discord, Feishu, CLI (Python side)
- **Provider Support**: OpenAI, Anthropic, LiteLLM and 100+ providers (Python side)

### Build

#### Prerequisites
- Rust 1.75+
- Python 3.11+ (for Python FFI)
- maturin (`pip install maturin`)

#### Build Rust library
```bash
cd nanobot-rs
cargo build --release
```

#### Build Python extension
```bash
cd nanobot-rs/python
maturin develop
```

### Usage

#### Rust

```rust
use nanobot_core::{Agent, AgentConfig, ToolRegistry};

#[tokio::main]
async fn main() {
    let config = AgentConfig {
        model: "claude-3-haiku".into(),
        provider: "openai".into(),
        temperature: 0.7,
        max_tokens: Some(4096),
        system_prompt: None,
        tools: vec![],
    };
    
    let tools = Arc::new(ToolRegistry::new());
    let agent = Agent::new(config, tools);
    
    let response = agent.process("Hello!").await;
    println!("Response: {:?}", response);
}
```

#### Python

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

### Design Principles

1. **Rust Core**: Performance-critical agent loop in Rust
2. **Python FFI**: Leverage existing Python SDKs for channels/providers
3. **Minimal**: ~4000 LOC core vs ~100k+ for similar projects
4. **Extensible**: Clean interfaces for channels and tools

### License

MIT

---

## 中文

超轻量级 AI Agent 核心，使用 Rust 实现，支持 Python FFI 绑定。

灵感来自 [nanobot](https://github.com/HKUDS/nanobot) - 一个用约 4000 行 Python 代码编写的轻量级 AI 助手。

### 架构

```
┌─────────────────────────────────────────┐
│           Python 集成层                  │
│    (频道适配器, LLM 提供商封装)          │
└──────────────────┬──────────────────────┘
                   │ PyO3 FFI
┌──────────────────┴──────────────────────┐
│              Rust 核心                   │
│  - Agent 循环 (LLM + 工具调用)           │
│  - 上下文构建器                          │
│  - 工具注册表                           │
│  - 记忆系统 (SQLite)                    │
│  - 会话管理                             │
└─────────────────────────────────────────┘
```

### 项目结构

```
nanobot-rs/
├── Cargo.toml              # Rust 项目配置
├── src/
│   ├── lib.rs             # 库入口
│   ├── agent/             # 核心 Agent 逻辑
│   │   ├── mod.rs        # Agent, Message, Response
│   │   ├── loop_core.rs  # 主循环
│   │   └── context.rs    # 提示词构建
│   ├── config/            # 配置系统
│   ├── tools/             # 工具系统
│   │   ├── mod.rs        # Tool trait
│   │   ├── registry.rs   # 工具注册表
│   │   └── builtin.rs    # 内置工具
│   ├── memory/            # SQLite 记忆存储
│   ├── session/           # 会话管理
│   └── ffi/               # PyO3 绑定
└── python/
    ├── pyproject.toml    # Python 包
    └── nanobot/          # Python 模块
        ├── __init__.py
        ├── client.py     # 高级 API
        ├── provider.py   # LLM 提供商
        └── channel/      # 聊天平台
```

### 功能特性

- **核心 Agent 循环**: LLM 推理 + 工具调用（最多 20 次迭代）
- **工具系统**: 内置工具（文件系统、网页搜索、消息、定时任务）+ 自定义工具注册
- **记忆系统**: 基于 SQLite 的持久化记忆，支持会话
- **会话管理**: 用户会话跟踪，支持超时
- **Python FFI**: PyO3 绑定，用于 Python 集成
- **频道适配器**: Telegram、Discord、飞书、CLI（Python 端）
- **提供商支持**: OpenAI、Anthropic、LiteLLM 及 100+ 提供商（Python 端）

### 构建

#### 前置条件
- Rust 1.75+
- Python 3.11+ (用于 Python FFI)
- maturin (`pip install maturin`)

#### 构建 Rust 库
```bash
cd nanobot-rs
cargo build --release
```

#### 构建 Python 扩展
```bash
cd nanobot-rs/python
maturin develop
```

### 使用方式

#### Rust

```rust
use nanobot_core::{Agent, AgentConfig, ToolRegistry};

#[tokio::main]
async fn main() {
    let config = AgentConfig {
        model: "claude-3-haiku".into(),
        provider: "openai".into(),
        temperature: 0.7,
        max_tokens: Some(4096),
        system_prompt: None,
        tools: vec![],
    };
    
    let tools = Arc::new(ToolRegistry::new());
    let agent = Agent::new(config, tools);
    
    let response = agent.process("你好！").await;
    println!("回复: {:?}", response);
}
```

#### Python

```python
from nanobot import NanobotClient
from nanobot.channel import create_channel

# 创建客户端
client = NanobotClient()

# 设置 LLM 提供商
from nanobot.provider import LiteLLMProvider
client.set_provider(LiteLLMProvider(model="claude-3-haiku"))

# 对话
response = client.chat("你好！")
print(response)

# 连接 Telegram
channel = create_channel("telegram", {"token": "YOUR_TOKEN"})
channel.on_message(lambda msg: print(f"用户: {msg.content}"))
```

### 设计原则

1. **Rust 核心**: 性能关键的 Agent 循环使用 Rust
2. **Python FFI**: 利用现有 Python SDK 实现频道/提供商
3. **极简**: 约 4000 行核心代码 vs 类似项目的 10万+ 行
4. **可扩展**: 频道和工具的清晰接口

### 许可证

MIT
